use shared::State;
use utoipa_axum::router::OpenApiRouter;
pub mod tickets;
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/support/tickets", tickets::router(state))
        .with_state(state.clone())
}
