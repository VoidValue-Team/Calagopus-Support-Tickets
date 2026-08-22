use crate::{errors::api_error, models::DepartmentPayload};
use axum::{extract::Path, http::StatusCode};
use shared::{
    GetState, State,
    models::{admin_activity::GetAdminActivityLogger, user::GetPermissionManager},
    response::{ApiResponse, ApiResponseResult},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[utoipa::path(put,path="/",request_body=inline(DepartmentPayload),responses((status=OK)))]
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
            .with_status(StatusCode::BAD_REQUEST)
            .ok();
    }
    let updated = sqlx::query(
        r#"UPDATE team_voidvalue_tickets_departments SET
           name=$2,description=$3,enabled=$4,position=$5,default_priority=$6,
           first_response_sla_minutes=$7,resolution_sla_minutes=$8,autoresponse=$9,
           allow_server_access=$10,notification_enabled=$11,updated_at=now()
           WHERE uuid=$1"#,
    )
    .bind(department)
    .bind(&data.name)
    .bind(&data.description)
    .bind(data.enabled)
    .bind(data.position)
    .bind(data.default_priority)
    .bind(data.first_response_sla_minutes)
    .bind(data.resolution_sla_minutes)
    .bind(data.autoresponse)
    .bind(data.allow_server_access)
    .bind(data.notification_enabled)
    .execute(state.database.write())
    .await;
    match updated {
        Ok(result) if result.rows_affected() == 1 => {
            logger
                .log(
                    "support:department.update",
                    serde_json::json!({"department_uuid":department,"name":data.name}),
                )
                .await;
            ApiResponse::new_serialized(serde_json::json!({})).ok()
        }
        Ok(_) => ApiResponse::error("department not found")
            .with_status(StatusCode::NOT_FOUND)
            .ok(),
        Err(error) => api_error(error.into()),
    }
}

#[utoipa::path(delete,path="/",responses((status=OK),(status=CONFLICT)))]
async fn delete(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    Path(department): Path<Uuid>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-departments")?;
    let tickets: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM team_voidvalue_tickets_tickets WHERE department_uuid=$1",
    )
    .bind(department)
    .fetch_one(state.database.read())
    .await
    {
        Ok(count) => count,
        Err(error) => return api_error(error.into()),
    };
    if tickets > 0 {
        return ApiResponse::error("department has tickets; disable it instead")
            .with_status(StatusCode::CONFLICT)
            .ok();
    }
    match sqlx::query("DELETE FROM team_voidvalue_tickets_departments WHERE uuid=$1")
        .bind(department)
        .execute(state.database.write())
        .await
    {
        Ok(result) if result.rows_affected() == 1 => {
            logger
                .log(
                    "support:department.delete",
                    serde_json::json!({"department_uuid":department}),
                )
                .await;
            ApiResponse::new_serialized(serde_json::json!({})).ok()
        }
        Ok(_) => ApiResponse::error("department not found")
            .with_status(StatusCode::NOT_FOUND)
            .ok(),
        Err(error) => api_error(error.into()),
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(put))
        .routes(routes!(delete))
        .with_state(state.clone())
}
