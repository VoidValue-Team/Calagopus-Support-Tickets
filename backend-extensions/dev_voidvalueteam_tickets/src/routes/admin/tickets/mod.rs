use crate::{
    errors::api_error,
    models::{ListQuery, Page, PublicSettings, SupportAgent, SupportServer, TicketSummary},
    repository::{self, TicketScope},
};
use axum::extract::Query;
use serde::Serialize;
use shared::{
    GetState, State,
    models::user::GetPermissionManager,
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
pub mod _ticket_;
#[derive(Serialize, ToSchema)]
struct Response {
    tickets: Page<TicketSummary>,
}
#[derive(Serialize, ToSchema)]
struct AgentsResponse {
    agents: Vec<SupportAgent>,
}
#[derive(Serialize, ToSchema)]
struct ServersResponse {
    servers: Vec<SupportServer>,
}
#[derive(Serialize, ToSchema)]
struct ConfigurationResponse {
    configuration: PublicSettings,
}
#[utoipa::path(get,path="/",params(ListQuery),responses((status=OK,body=inline(Response))))]
async fn get(
    state: GetState,
    permissions: GetPermissionManager,
    Query(query): Query<ListQuery>,
) -> ApiResponseResult {
    permissions.has_admin_permission("support.read")?;
    match repository::list_tickets(&state, TicketScope::Admin, &query).await {
        Ok(tickets) => ApiResponse::new_serialized(Response { tickets }).ok(),
        Err(e) => api_error(e),
    }
}
async fn agents(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.assign")?;
    match repository::list_agents(&state).await {
        Ok(agents) => ApiResponse::new_serialized(AgentsResponse { agents }).ok(),
        Err(error) => api_error(error),
    }
}
async fn servers(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.edit")?;
    match repository::list_support_servers(&state).await {
        Ok(servers) => ApiResponse::new_serialized(ServersResponse { servers }).ok(),
        Err(error) => api_error(error),
    }
}
async fn configuration(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
    permissions.has_admin_permission("support.read")?;
    let settings = match state.settings.get().await {
        Ok(settings) => settings,
        Err(error) => return api_error(error),
    };
    let extension: &crate::settings::ExtensionSettingsData =
        match settings.find_extension_settings() {
            Ok(extension) => extension,
            Err(error) => return api_error(error),
        };
    ApiResponse::new_serialized(ConfigurationResponse {
        configuration: extension.into(),
    })
    .ok()
}
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get))
        .route("/agents", axum::routing::get(agents))
        .route("/servers", axum::routing::get(servers))
        .route("/configuration", axum::routing::get(configuration))
        .nest("/{ticket}", _ticket_::router(state))
        .with_state(state.clone())
}
