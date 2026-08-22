use crate::{
    errors::api_error,
    models::{
        CreateTicketPayload, ListQuery, MessageType, Page, ReplyPayload, TicketDetail,
        TicketSummary,
    },
    repository::{self, TicketScope},
};
use axum::extract::{Path, Query};
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
struct TicketsResponse {
    tickets: Page<TicketSummary>,
}
#[derive(Serialize, ToSchema)]
struct TicketResponse {
    ticket: TicketDetail,
}

#[utoipa::path(get,path="/",params(ListQuery),responses((status=OK,body=inline(TicketsResponse))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    server: GetServer,
    Query(query): Query<ListQuery>,
) -> ApiResponseResult {
    permissions.has_server_permission("support.read")?;
    match repository::list_tickets(&state, TicketScope::Server(server.uuid), &query).await {
        Ok(tickets) => ApiResponse::new_serialized(TicketsResponse { tickets }).ok(),
        Err(e) => api_error(e),
    }
}

#[utoipa::path(post,path="/",request_body=inline(CreateTicketPayload),responses((status=OK,body=inline(TicketResponse))))]
async fn post(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    logger: GetServerActivityLogger,
    shared::Payload(mut data): shared::Payload<CreateTicketPayload>,
) -> ApiResponseResult {
    permissions.has_server_permission("support.create")?;
    data.server_uuid = Some(server.uuid);
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    match repository::create_ticket(&state, user.uuid, data).await {
        Ok(ticket) => {
            logger.log("server:support-ticket.create",serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"ticket_code":ticket.ticket.code})).await;
            ApiResponse::new_serialized(TicketResponse { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}

#[utoipa::path(get,path="/{ticket}",params(("server"=Uuid,description="Server UUID"),("ticket"=Uuid,description="Ticket UUID")),responses((status=OK,body=inline(TicketResponse))))]
async fn detail(
    state: GetState,
    permissions: GetPermissionManager,
    server: GetServer,
    Path((_server, ticket)): Path<(String, Uuid)>,
) -> ApiResponseResult {
    permissions.has_server_permission("support.read")?;
    match repository::get_ticket(&state, ticket, TicketScope::Server(server.uuid), false).await {
        Ok(Some(ticket)) => ApiResponse::new_serialized(TicketResponse { ticket }).ok(),
        Ok(None) => ApiResponse::error("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok(),
        Err(e) => api_error(e),
    }
}

#[utoipa::path(post,path="/{ticket}/reply",params(("server"=Uuid,description="Server UUID"),("ticket"=Uuid,description="Ticket UUID")),request_body=inline(ReplyPayload),responses((status=OK,body=inline(TicketResponse))))]
async fn reply(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    logger: GetServerActivityLogger,
    Path((_server, ticket)): Path<(String, Uuid)>,
    shared::Payload(data): shared::Payload<ReplyPayload>,
) -> ApiResponseResult {
    permissions.has_server_permission("support.reply")?;
    match repository::reply(
        &state,
        ticket,
        user.uuid,
        TicketScope::Server(server.uuid),
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
            ApiResponse::new_serialized(TicketResponse { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(post))
        .routes(routes!(detail))
        .routes(routes!(reply))
        .with_state(state.clone())
}
