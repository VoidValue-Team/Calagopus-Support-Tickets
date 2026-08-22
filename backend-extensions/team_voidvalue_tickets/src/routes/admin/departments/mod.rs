use crate::{
    errors::api_error,
    models::{Department, DepartmentPayload},
    repository,
};
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
mod _department_;
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
        permissions.has_admin_permission("support.read")?;
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
    let uuid = Uuid::new_v4();
    let result=sqlx::query(r#"INSERT INTO team_voidvalue_tickets_departments(uuid,name,description,enabled,position,default_priority,first_response_sla_minutes,resolution_sla_minutes,autoresponse,allow_server_access,notification_enabled) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#).bind(uuid).bind(&data.name).bind(&data.description).bind(data.enabled).bind(data.position).bind(data.default_priority).bind(data.first_response_sla_minutes).bind(data.resolution_sla_minutes).bind(data.autoresponse).bind(data.allow_server_access).bind(data.notification_enabled).execute(state.database.write()).await;
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
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(post))
        .nest("/{department}", _department_::router(state))
        .with_state(state.clone())
}
