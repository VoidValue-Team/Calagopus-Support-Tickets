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
        user::{GetPermissionManager, GetUser},
        user_activity::GetUserActivityLogger,
    },
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

pub mod _ticket_;

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
    Query(query): Query<ListQuery>,
) -> ApiResponseResult {
    permissions.has_user_permission("tickets.read")?;
    if let Err(errors) = shared::utils::validate_data(&query) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    match repository::list_tickets(&state, TicketScope::User(user.uuid), &query).await {
        Ok(tickets) => ApiResponse::new_serialized(TicketsResponse { tickets }).ok(),
        Err(e) => api_error(e),
    }
}

#[utoipa::path(post,path="/",request_body=inline(CreateTicketPayload),responses((status=OK,body=inline(TicketResponse))))]
async fn post(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetUserActivityLogger,
    shared::Payload(data): shared::Payload<CreateTicketPayload>,
) -> ApiResponseResult {
    permissions.has_user_permission("tickets.create")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    match repository::create_ticket(&state, user.uuid, data).await {
        Ok(ticket) => {
            logger.log("user:support-ticket.create",serde_json::json!({"ticket_uuid":ticket.ticket.uuid,"ticket_code":ticket.ticket.code,"server_uuid":ticket.ticket.server_uuid})).await;
            ApiResponse::new_serialized(TicketResponse { ticket }).ok()
        }
        Err(e) => api_error(e),
    }
}

#[utoipa::path(get,path="/departments",responses((status=OK,body=inline(DepartmentsResponse))))]
async fn departments(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_user_permission("tickets.read")?;
    match repository::list_departments(&state, false).await {
        Ok(departments) => ApiResponse::new_serialized(DepartmentsResponse { departments }).ok(),
        Err(e) => api_error(e),
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(post))
        .route("/departments", axum::routing::get(departments))
        .nest("/{ticket}", _ticket_::router(state))
        .with_state(state.clone())
}
