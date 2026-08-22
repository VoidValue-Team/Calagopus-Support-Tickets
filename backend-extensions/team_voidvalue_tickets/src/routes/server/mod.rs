use crate::{
    errors::api_error,
    models::{CreateTicketPayload, Department, ListQuery, Page, TicketDetail, TicketSummary},
    repository::{self, TicketScope},
};
use axum::extract::Query;
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

mod _ticket_;

#[derive(Serialize, ToSchema)]
struct TicketsResponse {
    tickets: Page<TicketSummary>,
}

#[derive(Serialize, ToSchema)]
struct TicketResponse {
    ticket: TicketDetail,
}

#[derive(Serialize, ToSchema)]
struct DepartmentsResponse {
    departments: Vec<Department>,
}

#[utoipa::path(get,path="/",params(ListQuery),responses((status=OK,body=inline(TicketsResponse))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    server: GetServer,
    Query(query): Query<ListQuery>,
) -> ApiResponseResult {
    permissions.has_server_permission("tickets.read")?;
    if let Err(errors) = shared::utils::validate_data(&query) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    let scope = TicketScope::Server {
        user_uuid: user.uuid,
        server_uuid: server.uuid,
    };
    match repository::list_tickets(&state, scope, &query).await {
        Ok(tickets) => ApiResponse::new_serialized(TicketsResponse { tickets }).ok(),
        Err(error) => api_error(error),
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
    permissions.has_server_permission("tickets.create")?;
    data.server_uuid = Some(server.uuid);
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    match repository::create_ticket(&state, user.uuid, data, false).await {
        Ok(ticket) => {
            logger
                .log(
                    "server:support-ticket.create",
                    serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"ticket_code":ticket.ticket.code}),
                )
                .await;
            crate::mail::send_customer(&state, "ticket-created-customer", &ticket).await;
            crate::mail::send_staff(&state, "ticket-created-staff", &ticket).await;
            ApiResponse::new_serialized(TicketResponse { ticket }).ok()
        }
        Err(error) => api_error(error),
    }
}

#[utoipa::path(get,path="/departments",responses((status=OK,body=inline(DepartmentsResponse))))]
async fn departments(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_server_permission("tickets.read")?;
    match repository::list_departments(&state, false).await {
        Ok(departments) => ApiResponse::new_serialized(DepartmentsResponse { departments }).ok(),
        Err(error) => api_error(error),
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(post))
        .routes(routes!(departments))
        .nest("/{ticket}", _ticket_::router(state))
        .with_state(state.clone())
}
