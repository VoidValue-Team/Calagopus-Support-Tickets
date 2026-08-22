use crate::{
    errors::api_error,
    models::{
        AssignPayload, EditTicketPayload, MessageType, ReplyPayload, StatusPayload, TicketDetail,
    },
    repository::{self, TicketScope},
};
use axum::extract::{DefaultBodyLimit, Multipart, Path};
use serde::Serialize;
use shared::{
    GetState, State,
    models::{
        admin_activity::GetAdminActivityLogger,
        user::{GetPermissionManager, GetUser},
    },
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
#[derive(Serialize, ToSchema)]
struct Response {
    ticket: TicketDetail,
}
#[utoipa::path(get,path="/",params(("ticket"=Uuid,description="Ticket UUID")),responses((status=OK,body=inline(Response))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    Path(ticket): Path<Uuid>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.read")?;
    match repository::get_ticket(&state, ticket, TicketScope::Admin, true).await {
        Ok(Some(ticket)) => ApiResponse::new_serialized(Response { ticket }).ok(),
        Ok(None) => ApiResponse::error("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok(),
        Err(e) => api_error(e),
    }
}
#[utoipa::path(post,path="/reply",request_body=inline(ReplyPayload),responses((status=OK,body=inline(Response))))]
async fn reply(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetAdminActivityLogger,
    Path(ticket): Path<Uuid>,
    shared::Payload(data): shared::Payload<ReplyPayload>,
) -> ApiResponseResult {
    let kind = if data.internal_note.unwrap_or(false) {
        permissions.has_admin_permission("support.internal-note")?;
        MessageType::InternalNote
    } else {
        permissions.has_admin_permission("support.reply")?;
        MessageType::Staff
    };
    match repository::reply(
        &state,
        ticket,
        user.uuid,
        TicketScope::Admin,
        data.message,
        kind,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    if kind == MessageType::InternalNote {
                        "support:ticket.internal-note.create"
                    } else {
                        "support:ticket.reply"
                    },
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid}),
                )
                .await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}
#[utoipa::path(put,path="/status",request_body=inline(StatusPayload),responses((status=OK,body=inline(Response))))]
async fn status(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetAdminActivityLogger,
    Path(ticket): Path<Uuid>,
    shared::Payload(data): shared::Payload<StatusPayload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.update-status")?;
    match repository::update_status(
        &state,
        ticket,
        user.uuid,
        TicketScope::Admin,
        data.status,
        true,
        None,
    )
    .await
    {
        Ok(ticket) => {
            logger.log("support:ticket.status.update",serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"status":ticket.ticket.status})).await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}
#[utoipa::path(put,path="/assign",request_body=inline(AssignPayload),responses((status=OK,body=inline(Response))))]
async fn assign(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetAdminActivityLogger,
    Path(ticket): Path<Uuid>,
    shared::Payload(data): shared::Payload<AssignPayload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.assign")?;
    match repository::assign(&state, ticket, user.uuid, data.staff_uuid).await {
        Ok(ticket) => {
            logger.log("support:ticket.assignment.update",serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"staff_uuid":data.staff_uuid})).await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}

async fn properties(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetAdminActivityLogger,
    Path(ticket): Path<Uuid>,
    shared::Payload(data): shared::Payload<EditTicketPayload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.read")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    let current = match repository::get_ticket(&state, ticket, TicketScope::Admin, true).await {
        Ok(Some(current)) => current,
        Ok(None) => {
            return ApiResponse::error("ticket not found")
                .with_status(axum::http::StatusCode::NOT_FOUND)
                .ok();
        }
        Err(error) => return api_error(error),
    };
    if data.subject.trim() == current.ticket.subject
        && data.server_uuid == current.ticket.server_uuid
        && data.priority == current.ticket.priority
        && data.department_uuid == current.ticket.department_uuid
    {
        return ApiResponse::new_serialized(Response { ticket: current }).ok();
    }
    if data.subject.trim() != current.ticket.subject
        || data.server_uuid != current.ticket.server_uuid
    {
        permissions.has_admin_permission("support.edit")?;
    }
    if data.priority != current.ticket.priority {
        permissions.has_admin_permission("support.update-priority")?;
    }
    if data.department_uuid != current.ticket.department_uuid {
        permissions.has_admin_permission("support.move-department")?;
    }
    match repository::edit_ticket(&state, ticket, user.uuid, data).await {
        Ok(ticket) => {
            logger
                .log(
                    "support:ticket.properties.update",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid}),
                )
                .await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(error) => api_error(error),
    }
}

async fn upload_attachments(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetAdminActivityLogger,
    Path(ticket): Path<Uuid>,
    multipart: Multipart,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.attachments")?;
    match crate::routes::attachments::upload(
        &state,
        ticket,
        user.uuid,
        TicketScope::Admin,
        multipart,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    "support:ticket.attachments.add",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid}),
                )
                .await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(error) => api_error(error),
    }
}

async fn download_attachment(
    state: GetState,
    permissions: GetPermissionManager,
    Path((ticket, attachment)): Path<(Uuid, Uuid)>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.attachments")?;
    match repository::download_attachment(&state, ticket, attachment, TicketScope::Admin).await {
        Ok(Some(file)) => crate::routes::attachments::download_response(file).ok(),
        Ok(None) => ApiResponse::error("attachment not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok(),
        Err(error) => api_error(error),
    }
}

async fn delete(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    Path(ticket): Path<Uuid>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.delete")?;
    match repository::delete_ticket(&state, ticket).await {
        Ok(Some(code)) => {
            logger
                .log(
                    "support:ticket.delete",
                    serde_json::json!({"ticket_uuid":ticket,"ticket_code":code}),
                )
                .await;
            ApiResponse::new_serialized(serde_json::json!({})).ok()
        }
        Ok(None) => ApiResponse::error("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok(),
        Err(error) => api_error(error),
    }
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(reply))
        .routes(routes!(status))
        .routes(routes!(assign))
        .route("/properties", axum::routing::put(properties))
        .route("/", axum::routing::delete(delete))
        .route(
            "/attachments",
            axum::routing::post(upload_attachments).layer(DefaultBodyLimit::max(64 * 1024 * 1024)),
        )
        .route(
            "/attachments/{attachment}",
            axum::routing::get(download_attachment),
        )
        .with_state(state.clone())
}
