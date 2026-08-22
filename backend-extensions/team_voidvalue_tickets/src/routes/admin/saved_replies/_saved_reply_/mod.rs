use super::Payload;
use crate::errors::api_error;
use axum::{extract::Path, http::StatusCode};
use shared::{
    GetState, State,
    models::{admin_activity::GetAdminActivityLogger, user::GetPermissionManager},
    response::{ApiResponse, ApiResponseResult},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[utoipa::path(put,path="/",request_body=inline(Payload),responses((status=OK)))]
async fn put(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    Path(saved_reply): Path<Uuid>,
    shared::Payload(data): shared::Payload<Payload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-saved-replies")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(StatusCode::BAD_REQUEST)
            .ok();
    }
    match sqlx::query(
        r#"UPDATE team_voidvalue_tickets_saved_replies SET
           title=$2,body=$3,department_uuid=$4,enabled=$5,updated_at=now()
           WHERE uuid=$1"#,
    )
    .bind(saved_reply)
    .bind(&data.title)
    .bind(data.body)
    .bind(data.department_uuid)
    .bind(data.enabled)
    .execute(state.database.write())
    .await
    {
        Ok(result) if result.rows_affected() == 1 => {
            logger
                .log(
                    "support:saved-reply.update",
                    serde_json::json!({"saved_reply_uuid":saved_reply,"title":data.title}),
                )
                .await;
            ApiResponse::new_serialized(serde_json::json!({})).ok()
        }
        Ok(_) => ApiResponse::error("saved reply not found")
            .with_status(StatusCode::NOT_FOUND)
            .ok(),
        Err(error) => api_error(error.into()),
    }
}

#[utoipa::path(delete,path="/",responses((status=OK)))]
async fn delete(
    state: GetState,
    permissions: GetPermissionManager,
    logger: GetAdminActivityLogger,
    Path(saved_reply): Path<Uuid>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-saved-replies")?;
    match sqlx::query("DELETE FROM team_voidvalue_tickets_saved_replies WHERE uuid=$1")
        .bind(saved_reply)
        .execute(state.database.write())
        .await
    {
        Ok(result) if result.rows_affected() == 1 => {
            logger
                .log(
                    "support:saved-reply.delete",
                    serde_json::json!({"saved_reply_uuid":saved_reply}),
                )
                .await;
            ApiResponse::new_serialized(serde_json::json!({})).ok()
        }
        Ok(_) => ApiResponse::error("saved reply not found")
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
