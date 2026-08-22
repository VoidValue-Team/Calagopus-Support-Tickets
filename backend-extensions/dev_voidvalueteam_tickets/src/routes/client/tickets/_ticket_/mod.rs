use crate::{
    errors::api_error,
    models::{MessageType, ReplyPayload, StatusPayload, TicketDetail, TicketStatus},
    repository::{self, TicketScope},
};
use axum::extract::Path;
use serde::Serialize;
use shared::{
    GetState, State,
    models::{
        user::{GetPermissionManager, GetUser},
        user_activity::GetUserActivityLogger,
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
    user: GetUser,
    Path(ticket): Path<Uuid>,
) -> ApiResponseResult {
    permissions.has_user_permission("tickets.read")?;
    match repository::get_ticket(&state, ticket, TicketScope::User(user.uuid), false).await {
        Ok(Some(ticket)) => ApiResponse::new_serialized(Response { ticket }).ok(),
        Ok(None) => ApiResponse::error("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok(),
        Err(e) => api_error(e),
    }
}

#[utoipa::path(post,path="/reply",params(("ticket"=Uuid,description="Ticket UUID")),request_body=inline(ReplyPayload),responses((status=OK,body=inline(Response))))]
async fn reply(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetUserActivityLogger,
    Path(ticket): Path<Uuid>,
    shared::Payload(data): shared::Payload<ReplyPayload>,
) -> ApiResponseResult {
    permissions.has_user_permission("tickets.reply")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    match repository::reply(
        &state,
        ticket,
        user.uuid,
        TicketScope::User(user.uuid),
        data.message,
        MessageType::Customer,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    "user:support-ticket.reply",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid}),
                )
                .await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}

#[utoipa::path(put,path="/status",params(("ticket"=Uuid,description="Ticket UUID")),request_body=inline(StatusPayload),responses((status=OK,body=inline(Response))))]
async fn status(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetUserActivityLogger,
    Path(ticket): Path<Uuid>,
    shared::Payload(data): shared::Payload<StatusPayload>,
) -> ApiResponseResult {
    let permission = if data.status == TicketStatus::Closed {
        "tickets.close"
    } else {
        "tickets.reopen"
    };
    permissions.has_user_permission(permission)?;
    let settings = match state.settings.get().await {
        Ok(v) => v,
        Err(e) => return api_error(e),
    };
    let ext: Result<&crate::settings::ExtensionSettingsData, _> =
        settings.find_extension_settings();
    let reopen = ext.map(|v| v.allow_reopen).unwrap_or(false);
    drop(settings);
    match repository::update_status(
        &state,
        ticket,
        user.uuid,
        TicketScope::User(user.uuid),
        data.status,
        reopen,
    )
    .await
    {
        Ok(ticket) => {
            logger.log("user:support-ticket.status.update",serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"status":ticket.ticket.status})).await;
            ApiResponse::new_serialized(Response { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .route("/reply", axum::routing::post(reply))
        .route("/status", axum::routing::put(status))
        .with_state(state.clone())
}
