use crate::{errors::api_error, models::Statistics, repository};
use serde::Serialize;
use shared::{
    GetState, State,
    models::user::GetPermissionManager,
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
#[derive(Serialize, ToSchema)]
struct Response {
    statistics: Statistics,
}
#[utoipa::path(get,path="/",responses((status=OK,body=inline(Response))))]
async fn get(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.view-statistics")?;
    match repository::statistics(&state).await {
        Ok(statistics) => ApiResponse::new_serialized(Response { statistics }).ok(),
        Err(e) => api_error(e),
    }
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .with_state(state.clone())
}
