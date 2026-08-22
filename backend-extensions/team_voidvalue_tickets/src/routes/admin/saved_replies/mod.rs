use crate::errors::api_error;
use axum::extract::Query;
use garde::Validate;
use serde::{Deserialize, Serialize};
use shared::{
    GetState, State,
    models::{
        admin_activity::GetAdminActivityLogger,
        user::{GetPermissionManager, GetUser},
    },
    response::{ApiResponse, ApiResponseResult},
};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
mod _saved_reply_;
#[derive(Deserialize, IntoParams, Validate, ToSchema)]
struct Search {
    #[garde(length(chars, max = 120))]
    search: Option<String>,
}
#[derive(Deserialize, Validate, ToSchema)]
pub(super) struct Payload {
    #[garde(length(chars, min = 2, max = 120))]
    title: String,
    #[garde(length(chars, min = 1, max = 20000))]
    body: String,
    #[garde(skip)]
    department_uuid: Option<Uuid>,
    #[garde(skip)]
    enabled: bool,
}
#[derive(Serialize, ToSchema, FromRow)]
struct SavedReply {
    uuid: Uuid,
    title: String,
    body: String,
    department_uuid: Option<Uuid>,
    enabled: bool,
    created_by_uuid: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Serialize, ToSchema)]
struct Response {
    saved_replies: Vec<SavedReply>,
}
#[utoipa::path(get,path="/",params(Search),responses((status=OK,body=inline(Response))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    Query(query): Query<Search>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-saved-replies")?;
    let result=sqlx::query_as::<_,SavedReply>(r#"SELECT uuid,title,body,department_uuid,enabled,created_by_uuid,created_at,updated_at FROM team_voidvalue_tickets_saved_replies WHERE ($1 IS NULL OR to_tsvector('simple',title||' '||body)@@plainto_tsquery('simple',$1)) ORDER BY title LIMIT 100"#).bind(query.search).fetch_all(state.database.read()).await;
    match result {
        Ok(saved_replies) => ApiResponse::new_serialized(Response { saved_replies }).ok(),
        Err(e) => api_error(e.into()),
    }
}
#[utoipa::path(post,path="/",request_body=inline(Payload),responses((status=OK,body=inline(Response))))]
async fn post(
    state: GetState,
    permissions: GetPermissionManager,
    user: GetUser,
    logger: GetAdminActivityLogger,
    shared::Payload(data): shared::Payload<Payload>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.manage-saved-replies")?;
    if let Err(errors) = shared::utils::validate_data(&data) {
        return ApiResponse::error(errors.join(", "))
            .with_status(axum::http::StatusCode::BAD_REQUEST)
            .ok();
    }
    let uuid = Uuid::new_v4();
    if let Err(e)=sqlx::query("INSERT INTO team_voidvalue_tickets_saved_replies(uuid,title,body,department_uuid,enabled,created_by_uuid)VALUES($1,$2,$3,$4,$5,$6)").bind(uuid).bind(&data.title).bind(data.body).bind(data.department_uuid).bind(data.enabled).bind(user.uuid).execute(state.database.write()).await{return api_error(e.into())}
    logger
        .log(
            "support:saved-reply.create",
            serde_json::json!({"saved_reply_uuid":uuid,"title":data.title}),
        )
        .await;
    get(state, permissions, Query(Search { search: None })).await
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .routes(routes!(post))
        .nest("/{saved_reply}", _saved_reply_::router(state))
        .with_state(state.clone())
}
