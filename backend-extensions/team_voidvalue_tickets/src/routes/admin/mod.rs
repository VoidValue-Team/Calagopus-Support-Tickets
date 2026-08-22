use shared::State;
use utoipa_axum::router::OpenApiRouter;
pub mod access_options;
pub mod departments;
pub mod saved_replies;
pub mod settings;
pub mod staff;
pub mod statistics;
pub mod tickets;
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/tickets", tickets::router(state))
        .nest("/access-options", access_options::router(state))
        .nest("/departments", departments::router(state))
        .nest("/saved-replies", saved_replies::router(state))
        .nest("/statistics", statistics::router(state))
        .nest("/staff", staff::router(state))
        .nest("/settings", settings::router(state))
        .with_state(state.clone())
}
