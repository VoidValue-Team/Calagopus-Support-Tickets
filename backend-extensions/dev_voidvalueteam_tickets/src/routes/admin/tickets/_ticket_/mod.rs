use crate::{
    errors::api_error,
    models::{AssignPayload, MessageType, ReplyPayload, StatusPayload, TicketDetail},
    repository::{self, TicketScope},
};
use axum::extract::Path;
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
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(reply))
        .routes(routes!(status))
        .routes(routes!(assign))
        .with_state(state.clone())
}
