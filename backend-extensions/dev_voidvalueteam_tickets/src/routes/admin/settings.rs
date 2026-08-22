use crate::{errors::api_error, settings::ExtensionSettingsData};
use serde::Serialize;
use shared::{
    GetState, State,
    models::{admin_activity::GetAdminActivityLogger, user::GetPermissionManager},
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
#[derive(Serialize, ToSchema)]
struct Response {
    settings: ExtensionSettingsData,
}
#[utoipa::path(get,path="/",responses((status=OK,body=inline(Response))))]
async fn get(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-settings")?;
    let settings = match state.settings.get().await {
        Ok(v) => v,
        Err(e) => return api_error(e),
    };
    let ext: Result<&ExtensionSettingsData, _> = settings.find_extension_settings();
    match ext {
        Ok(v) => ApiResponse::new_serialized(Response {
            settings: v.clone(),
        })
        .ok(),
        Err(e) => api_error(e),
    }
}
#[utoipa::path(put,path="/",request_body=inline(ExtensionSettingsData),responses((status=OK,body=inline(Response))))]
async fn put(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    shared::Payload(data): shared::Payload<ExtensionSettingsData>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-settings")?;
    let mut settings = match state.settings.get_mut().await {
        Ok(v) => v,
        Err(e) => return api_error(e),
    };
    let ext: Result<&mut ExtensionSettingsData, _> = settings.find_mut_extension_settings();
    match ext {
        Ok(v) => *v = data.clone(),
        Err(e) => return api_error(e),
    }
    if let Err(e) = settings.save().await {
        return api_error(e.into());
    }
    logger.log("support:settings.update",serde_json::json!({"enabled":data.enabled,"attachments_enabled":data.attachments_enabled,"support_access_enabled":data.support_access_enabled})).await;
    ApiResponse::new_serialized(Response { settings: data }).ok()
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(put))
        .with_state(state.clone())
}
