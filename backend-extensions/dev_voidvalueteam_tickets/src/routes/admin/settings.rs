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
    shared::Payload(mut data): shared::Payload<ExtensionSettingsData>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-settings")?;
    data.ticket_prefix = data.ticket_prefix.trim().to_string();
    data.allowed_mime_types = data
        .allowed_mime_types
        .into_iter()
        .map(|mime| mime.trim().to_ascii_lowercase())
        .filter(|mime| !mime.is_empty())
        .collect();
    data.allowed_mime_types.sort();
    data.allowed_mime_types.dedup();
    data.support_access_permissions = data
        .support_access_permissions
        .into_iter()
        .map(|permission| permission.trim().to_string())
        .filter(|permission| !permission.is_empty())
        .collect();
    data.support_access_permissions.sort();
    data.support_access_permissions.dedup();
    let invalid = data.ticket_prefix.is_empty()
        || data.ticket_prefix.chars().count() > 12
        || data.attachment_max_bytes == 0
        || data.attachment_max_bytes > 60 * 1024 * 1024
        || data.attachment_max_files == 0
        || data.attachment_max_files > 20
        || data.allowed_mime_types.is_empty()
        || data.support_access_default_minutes == 0
        || data.support_access_max_minutes == 0
        || data.support_access_default_minutes > data.support_access_max_minutes
        || data.inactivity_days == 0
        || data.final_close_delay_days == 0;
    if invalid {
        return ApiResponse::error("invalid support settings")
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    if let Some(department) = data.default_department {
        let available: bool = match sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM dev_voidvalueteam_tickets_departments WHERE uuid=$1 AND enabled)",
        )
        .bind(department)
        .fetch_one(state.database.read())
        .await
        {
            Ok(available) => available,
            Err(error) => return api_error(error.into()),
        };
        if !available {
            return ApiResponse::error("default department is unavailable")
                .with_status(axum::http::StatusCode::BAD_REQUEST)
                .ok();
        }
    }
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
