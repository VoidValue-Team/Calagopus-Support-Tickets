use crate::{
    errors::api_error,
    models::{MessageType, ReplyPayload, StatusPayload, TicketDetail, TicketStatus},
    repository::{self, TicketScope},
};
use axum::extract::{DefaultBodyLimit, Multipart, Path};
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
    if !matches!(data.status, TicketStatus::Open | TicketStatus::Closed) {
        return ApiResponse::error("customers can only close or reopen tickets")
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
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
    let (reopen, reopen_period_days) = ext
        .map(|settings| (settings.allow_reopen, settings.reopen_period_days))
        .unwrap_or((false, 0));
    drop(settings);
    match repository::update_status(
        &state,
        ticket,
        user.uuid,
        TicketScope::User(user.uuid),
        data.status,
        reopen,
        Some(reopen_period_days),
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

async fn upload_attachments(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetUserActivityLogger,
    Path(ticket): Path<Uuid>,
    multipart: Multipart,
) -> ApiResponseResult {
    permissions.has_user_permission("tickets.attachments")?;
    match crate::routes::attachments::upload(
        &state,
        ticket,
        user.uuid,
        TicketScope::User(user.uuid),
        multipart,
    )
    .await
    {
        Ok(ticket) => {
            logger
                .log(
                    "user:support-ticket.attachments.add",
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
    Path((ticket, attachment)): Path<(Uuid, Uuid)>,
) -> ApiResponseResult {
    permissions.has_user_permission("tickets.attachments")?;
    match repository::download_attachment(&state, ticket, attachment, TicketScope::User(user.uuid))
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
        .route("/reply", axum::routing::post(reply))
        .route("/status", axum::routing::put(status))
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
