use shared::{
    State,
    extensions::{
        Extension, ExtensionPermissionsBuilder, ExtensionRouteBuilder,
        background_tasks::BackgroundTaskBuilder, email_templates::ExtensionEmailTemplateBuilder,
        settings::ExtensionSettingsDeserializer,
    },
};
use std::{str::FromStr, sync::Arc};
mod automation;
mod errors;
mod mail;
pub mod models;
mod permissions;
mod repository;
mod routes;
pub mod settings;
#[derive(Default)]
pub struct ExtensionStruct;
#[async_trait::async_trait]
impl Extension for ExtensionStruct {
    async fn initialize(&mut self, _state: State) {
        tracing::info!("Calagopus Support / Tickets initialized");
    }
    async fn initialize_router(
        &mut self,
        state: State,
        builder: ExtensionRouteBuilder,
    ) -> ExtensionRouteBuilder {
        builder
            .add_client_api_router(|r| r.merge(routes::client::router(&state)))
            .add_client_server_api_router(|r| {
                r.nest(
                    "/extensions/team.voidvalue.tickets",
                    routes::server::router(&state),
                )
            })
            .add_admin_api_router(|r| {
                r.nest(
                    "/extensions/team.voidvalue.tickets",
                    routes::admin::router(&state),
                )
            })
    }
    async fn initialize_permissions(
        &mut self,
        _state: State,
        builder: ExtensionPermissionsBuilder,
    ) -> ExtensionPermissionsBuilder {
        permissions::register(builder)
    }
    async fn initialize_email_templates(
        &mut self,
        _state: State,
        builder: ExtensionEmailTemplateBuilder,
    ) -> ExtensionEmailTemplateBuilder {
        mail::register(builder)
    }
    async fn initialize_background_tasks(
        &mut self,
        _state: State,
        builder: BackgroundTaskBuilder,
    ) -> BackgroundTaskBuilder {
        builder
            .add_cron_task(
                "support-sla-sweep",
                croner::Cron::from_str("0 * * * * *").expect("valid support SLA cron"),
                |state| async move { automation::sweep_sla(&state).await },
            )
            .await;
        builder
            .add_cron_task(
                "support-access-expiry",
                croner::Cron::from_str("15 * * * * *").expect("valid support access cron"),
                |state| async move { automation::mark_expired_access(&state).await },
            )
            .await;
        builder
            .add_cron_task(
                "support-auto-close",
                croner::Cron::from_str("0 0 * * * *").expect("valid support auto-close cron"),
                |state| async move { automation::sweep_auto_close(&state).await },
            )
            .await;
        builder
    }
    async fn settings_deserializer(&self, _state: State) -> ExtensionSettingsDeserializer {
        Arc::new(settings::ExtensionSettingsDataDeserializer)
    }
}
