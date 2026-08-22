use crate::{
    errors::api_error,
    models::{
        AccessReviewPayload, MessageType, ReplyPayload, StatusPayload, TicketDetail, TicketStatus,
    },
    repository::{self, TicketScope},
};
use axum::extract::{DefaultBodyLimit, Multipart, Path};
use serde::Serialize;
use shared::{
    GetState, State,
    models::{
        server::{GetServer, GetServerActivityLogger},
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

fn scope(user_uuid: Uuid, server_uuid: Uuid) -> TicketScope {
    TicketScope::Server {
        user_uuid,
        server_uuid,
    }
}

#[utoipa::path(get,path="/",responses((status=OK,body=inline(Response))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    Path((_server, ticket)): Path<(String, Uuid)>,
) -> ApiResponseResult {
    permissions.has_server_permission("tickets.read")?;
    match repository::get_ticket(&state, ticket, scope(user.uuid, server.uuid), false).await {
        Ok(Some(ticket)) => ApiResponse::new_serialized(Response { ticket }).ok(),
        Ok(None) => ApiResponse::error("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok(),
        Err(error) => api_error(error),
    }
}

#[utoipa::path(post,path="/reply",request_body=inline(ReplyPayload),responses((status=OK,body=inline(Response))))]
async fn reply(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    logger: GetServerActivityLogger,
    Path((_server, ticket)): Path<(String, Uuid)>,
    shared::Payload(data): shared::Payload<ReplyPayload>,
) -> ApiResponseResult {
    permissions.has_server_permission("tickets.reply")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    match repository::reply(
        &state,
        ticket,
        user.uuid,
        scope(user.uuid, server.uuid),
        data.message,
        MessageType::Customer,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    "server:support-ticket.reply",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid}),
                )
                .await;
            crate::mail::send_staff(&state, "ticket-customer-reply", &ticket).await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(error) => api_error(error),
    }
}

#[utoipa::path(put,path="/status",request_body=inline(StatusPayload),responses((status=OK,body=inline(Response))))]
async fn status(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    logger: GetServerActivityLogger,
    Path((_server, ticket)): Path<(String, Uuid)>,
    shared::Payload(data): shared::Payload<StatusPayload>,
) -> ApiResponseResult {
    if !data.status.can_be_set_by_customer() {
        return ApiResponse::error("customers can only close or reopen tickets")
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    let permission = if data.status == TicketStatus::Closed {
        "tickets.close"
    } else {
        "tickets.reopen"
    };
    permissions.has_server_permission(permission)?;
    let reopen = state
        .settings
        .get()
        .await
        .ok()
        .and_then(|settings| {
            settings
                .find_extension_settings::<crate::settings::ExtensionSettingsData>()
                .ok()
                .map(|extension| extension.allow_reopen)
        })
        .unwrap_or(false);
    match repository::update_status(
        &state,
        ticket,
        user.uuid,
        scope(user.uuid, server.uuid),
        data.status,
        reopen,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    "server:support-ticket.status.update",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"status":ticket.ticket.status}),
                )
                .await;
            crate::mail::send_staff(
                &state,
                if data.status == TicketStatus::Closed {
                    "ticket-closed"
                } else {
                    "ticket-reopened"
                },
                &ticket,
            )
            .await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(error) => api_error(error),
    }
}

#[utoipa::path(put,path="/access-requests/{request}",request_body=inline(AccessReviewPayload),responses((status=OK,body=inline(Response))))]
async fn review_access(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    logger: GetServerActivityLogger,
    Path((_server, ticket, request)): Path<(String, Uuid, Uuid)>,
    shared::Payload(data): shared::Payload<AccessReviewPayload>,
) -> ApiResponseResult {
    permissions.has_server_permission("tickets.read")?;
    let owns_ticket: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_voidvalue_tickets_tickets WHERE uuid=$1 AND user_uuid=$2 AND server_uuid=$3)",
    )
    .bind(ticket)
    .bind(user.uuid)
    .bind(server.uuid)
    .fetch_one(state.database.read())
    .await
    {
        Ok(owns_ticket) => owns_ticket,
        Err(error) => return api_error(error.into()),
    };
    if !owns_ticket {
        return ApiResponse::error("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok();
    }
    let approved = data.action == crate::models::AccessReviewAction::Approve;
    match repository::access::review_request(&state, ticket, request, user.uuid, data).await {
        Ok(ticket) => {
            logger
                .log(
                    "server:support-ticket.access.review",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"request_uuid":request}),
                )
                .await;
            if approved {
                crate::mail::send_staff(&state, "access-granted", &ticket).await;
            }
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(error) => api_error(error),
    }
}

async fn upload_attachments(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    logger: GetServerActivityLogger,
    Path((_server, ticket)): Path<(String, Uuid)>,
    multipart: Multipart,
) -> ApiResponseResult {
    permissions.has_server_permission("tickets.attachments")?;
    match crate::routes::attachments::upload(
        &state,
        ticket,
        user.uuid,
        scope(user.uuid, server.uuid),
        multipart,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    "server:support-ticket.attachments.add",
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
    user: GetUser,
    server: GetServer,
    Path((_server, ticket, attachment)): Path<(String, Uuid, Uuid)>,
) -> ApiResponseResult {
    permissions.has_server_permission("tickets.attachments")?;
    match repository::download_attachment(&state, ticket, attachment, scope(user.uuid, server.uuid))
        .await
    {
        Ok(Some(file)) => crate::routes::attachments::download_response(file).ok(),
        Ok(None) => ApiResponse::error("attachment not found")
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
        .routes(routes!(review_access))
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
