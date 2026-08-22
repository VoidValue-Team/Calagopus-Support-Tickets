use crate::{errors::api_error, models::StaffOption, repository};
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
    staff: Vec<StaffOption>,
}

#[utoipa::path(get,path="/",responses((status=OK,body=inline(Response))))]
async fn get(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.assign")?;
    match repository::list_staff(&state).await {
        Ok(staff) => ApiResponse::new_serialized(Response { staff }).ok(),
        Err(error) => api_error(error),
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .with_state(state.clone())
}
