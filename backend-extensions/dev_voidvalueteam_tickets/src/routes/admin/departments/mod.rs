use crate::{
    errors::api_error,
    models::{Department, DepartmentPayload},
    repository,
};
use axum::extract::Path;
use serde::Serialize;
use shared::{
    GetState, State,
    models::{
        admin_activity::GetAdminActivityLogger,
        user::{GetPermissionManager, GetUser},
    },
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
#[derive(Serialize, ToSchema)]
struct Response {
    departments: Vec<Department>,
}
#[utoipa::path(get,path="/",responses((status=OK,body=inline(Response))))]
async fn get(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    if permissions
        .has_admin_permission("support.manage-departments")
        .is_err()
    {
        permissions.has_admin_permission("support.manage-settings")?;
    }
    match repository::list_departments(&state, true).await {
        Ok(departments) => ApiResponse::new_serialized(Response { departments }).ok(),
        Err(e) => api_error(e),
    }
}
#[utoipa::path(post,path="/",request_body=inline(DepartmentPayload),responses((status=OK,body=inline(Response))))]
async fn post(
    state: GetState,
    permissions: GetPermissionManager,
    _user: GetUser,
    logger: GetAdminActivityLogger,
    shared::Payload(data): shared::Payload<DepartmentPayload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-departments")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    let duplicate: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM dev_voidvalueteam_tickets_departments WHERE lower(name)=lower($1))",
    )
    .bind(data.name.trim())
    .fetch_one(state.database.read())
    .await
    {
        Ok(duplicate) => duplicate,
        Err(error) => return api_error(error.into()),
    };
    if duplicate {
        return api_error(anyhow::anyhow!("department name already exists"));
    }
    let uuid = Uuid::new_v4();
    let result=sqlx::query(r#"INSERT INTO dev_voidvalueteam_tickets_departments(uuid,name,description,enabled,position,default_priority,first_response_sla_minutes,resolution_sla_minutes,autoresponse,allow_server_access,notification_enabled) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#).bind(uuid).bind(data.name.trim()).bind(&data.description).bind(data.enabled).bind(data.position).bind(data.default_priority).bind(data.first_response_sla_minutes).bind(data.resolution_sla_minutes).bind(data.autoresponse.as_deref().filter(|value| !value.trim().is_empty())).bind(data.allow_server_access).bind(data.notification_enabled).execute(state.database.write()).await;
    if let Err(e) = result {
        return api_error(e.into());
    }
    logger
        .log(
            "support:department.create",
            serde_json::json!({"department_uuid":uuid,"name":data.name}),
        )
        .await;
    get(state, permissions).await
}

async fn put(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    Path(department): Path<Uuid>,
    shared::Payload(data): shared::Payload<DepartmentPayload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-departments")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    let existing_enabled: Option<bool> = match sqlx::query_scalar(
        "SELECT enabled FROM dev_voidvalueteam_tickets_departments WHERE uuid=$1",
    )
    .bind(department)
    .fetch_optional(state.database.read())
    .await
    {
        Ok(enabled) => enabled,
        Err(error) => return api_error(error.into()),
    };
    let Some(existing_enabled) = existing_enabled else {
        return ApiResponse::error("department not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok();
    };
    let duplicate: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM dev_voidvalueteam_tickets_departments WHERE lower(name)=lower($1) AND uuid<>$2)",
    )
    .bind(data.name.trim())
    .bind(department)
    .fetch_one(state.database.read())
    .await
    {
        Ok(duplicate) => duplicate,
        Err(error) => return api_error(error.into()),
    };
    if duplicate {
        return api_error(anyhow::anyhow!("department name already exists"));
    }
    if existing_enabled && !data.enabled {
        let settings = match state.settings.get().await {
            Ok(settings) => settings,
            Err(error) => return api_error(error),
        };
        let extension: &crate::settings::ExtensionSettingsData =
            match settings.find_extension_settings() {
                Ok(extension) => extension,
                Err(error) => return api_error(error),
            };
        if extension.default_department == Some(department) {
            return api_error(anyhow::anyhow!("default department cannot be disabled"));
        }
        drop(settings);
        let other_enabled: i64 = match sqlx::query_scalar(
            "SELECT COUNT(*) FROM dev_voidvalueteam_tickets_departments WHERE enabled AND uuid<>$1",
        )
        .bind(department)
        .fetch_one(state.database.read())
        .await
        {
            Ok(count) => count,
            Err(error) => return api_error(error.into()),
        };
        if other_enabled == 0 {
            return api_error(anyhow::anyhow!(
                "last enabled department cannot be disabled"
            ));
        }
    }
    let result = sqlx::query(
        r#"UPDATE dev_voidvalueteam_tickets_departments SET
          name=$2,description=$3,enabled=$4,position=$5,default_priority=$6,
          first_response_sla_minutes=$7,resolution_sla_minutes=$8,autoresponse=$9,
          allow_server_access=$10,notification_enabled=$11,updated_at=now()
          WHERE uuid=$1"#,
    )
    .bind(department)
    .bind(data.name.trim())
    .bind(&data.description)
    .bind(data.enabled)
    .bind(data.position)
    .bind(data.default_priority)
    .bind(data.first_response_sla_minutes)
    .bind(data.resolution_sla_minutes)
    .bind(
        data.autoresponse
            .as_deref()
            .filter(|value| !value.trim().is_empty()),
    )
    .bind(data.allow_server_access)
    .bind(data.notification_enabled)
    .execute(state.database.write())
    .await;
    match result {
        Ok(result) if result.rows_affected() == 0 => {
            return ApiResponse::error("department not found")
                .with_status(axum::http::StatusCode::NOT_FOUND)
                .ok();
        }
        Ok(_) => {}
        Err(error) => return api_error(error.into()),
    }
    logger
        .log(
            "support:department.update",
            serde_json::json!({"department_uuid":department,"name":data.name}),
        )
        .await;
    get(state, permissions).await
}

async fn delete(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    Path(department): Path<Uuid>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-departments")?;
    let settings = match state.settings.get().await {
        Ok(settings) => settings,
        Err(error) => return api_error(error),
    };
    let extension: &crate::settings::ExtensionSettingsData =
        match settings.find_extension_settings() {
            Ok(extension) => extension,
            Err(error) => return api_error(error),
        };
    if extension.default_department == Some(department) {
        return api_error(anyhow::anyhow!("default department cannot be deleted"));
    }
    drop(settings);
    let enabled: Option<bool> = match sqlx::query_scalar(
        "SELECT enabled FROM dev_voidvalueteam_tickets_departments WHERE uuid=$1",
    )
    .bind(department)
    .fetch_optional(state.database.read())
    .await
    {
        Ok(enabled) => enabled,
        Err(error) => return api_error(error.into()),
    };
    let Some(enabled) = enabled else {
        return ApiResponse::error("department not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .ok();
    };
    let ticket_count: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM dev_voidvalueteam_tickets_tickets WHERE department_uuid=$1",
    )
    .bind(department)
    .fetch_one(state.database.read())
    .await
    {
        Ok(count) => count,
        Err(error) => return api_error(error.into()),
    };
    if ticket_count > 0 {
        return api_error(anyhow::anyhow!("department is in use"));
    }
    if enabled {
        let other_enabled: i64 = match sqlx::query_scalar(
            "SELECT COUNT(*) FROM dev_voidvalueteam_tickets_departments WHERE enabled AND uuid<>$1",
        )
        .bind(department)
        .fetch_one(state.database.read())
        .await
        {
            Ok(count) => count,
            Err(error) => return api_error(error.into()),
        };
        if other_enabled == 0 {
            return api_error(anyhow::anyhow!("last enabled department cannot be deleted"));
        }
    }
    if let Err(error) =
        sqlx::query("DELETE FROM dev_voidvalueteam_tickets_departments WHERE uuid=$1")
            .bind(department)
            .execute(state.database.write())
            .await
    {
        return api_error(error.into());
    }
    logger
        .log(
            "support:department.delete",
            serde_json::json!({"department_uuid":department}),
        )
        .await;
    get(state, permissions).await
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(post))
        .route("/{department}", axum::routing::put(put).delete(delete))
        .with_state(state.clone())
}
