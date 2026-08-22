use crate::{
    errors::api_error,
    models::{ListQuery, Page, TicketSummary},
    repository::{self, TicketScope},
};
use axum::extract::Query;
use serde::Serialize;
use shared::{
    GetState, State,
    models::user::GetPermissionManager,
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
pub mod _ticket_;
#[derive(Serialize, ToSchema)]
struct Response {
    tickets: Page<TicketSummary>,
}
#[utoipa::path(get,path="/",params(ListQuery),responses((status=OK,body=inline(Response))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    Query(query): Query<ListQuery>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.read")?;
    match repository::list_tickets(&state, TicketScope::Admin, &query).await {
        Ok(tickets) => ApiResponse::new_serialized(Response { tickets }).ok(),
        Err(e) => api_error(e),
    }
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .nest("/{ticket}", _ticket_::router(state))
        .with_state(state.clone())
}
