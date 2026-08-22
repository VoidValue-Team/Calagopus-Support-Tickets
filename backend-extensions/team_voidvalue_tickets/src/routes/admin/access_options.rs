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
    enabled: bool,
    permissions: Vec<String>,
    default_minutes: u32,
    max_minutes: u32,
}

#[utoipa::path(get,path="/",responses((status=OK,body=inline(Response))))]
async fn get(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.request-access")?;
    let settings = state.settings.get().await?;
    let extension: &crate::settings::ExtensionSettingsData = settings.find_extension_settings()?;
    ApiResponse::new_serialized(Response {
        enabled: extension.support_access_enabled,
        permissions: extension.support_access_permissions.clone(),
        default_minutes: extension.support_access_default_minutes,
        max_minutes: extension.support_access_max_minutes,
    })
    .ok()
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .with_state(state.clone())
}
