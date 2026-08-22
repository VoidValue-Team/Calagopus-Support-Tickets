use serde::{Deserialize, Serialize};
use shared::extensions::settings::{
    ExtensionSettings, SettingsDeserializeExt, SettingsDeserializer, SettingsSerializeExt,
    SettingsSerializer,
};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ExtensionSettingsData {
    pub enabled: bool,
    pub ticket_prefix: String,
    pub allow_user_priority: bool,
    pub allow_reopen: bool,
    pub reopen_period_days: u32,
    pub default_department: Option<uuid::Uuid>,
    pub customer_emails: bool,
    pub staff_emails: bool,
    pub attachments_enabled: bool,
    pub attachment_max_bytes: u64,
    pub attachment_max_files: u32,
    pub allowed_mime_types: Vec<String>,
    pub support_access_enabled: bool,
    pub support_access_permissions: Vec<String>,
    pub support_access_default_minutes: u32,
    pub support_access_max_minutes: u32,
    pub revoke_access_on_resolved: bool,
    pub auto_close_enabled: bool,
    pub inactivity_days: u32,
    pub final_close_delay_days: u32,
}

impl Default for ExtensionSettingsData {
    fn default() -> Self {
        Self {
            enabled: true,
            ticket_prefix: "TCK".into(),
            allow_user_priority: true,
            allow_reopen: true,
            reopen_period_days: 14,
            default_department: None,
            customer_emails: true,
            staff_emails: true,
            attachments_enabled: true,
            attachment_max_bytes: 10 * 1024 * 1024,
            attachment_max_files: 5,
            allowed_mime_types: vec![
                "image/png".into(),
                "image/jpeg".into(),
                "text/plain".into(),
                "application/pdf".into(),
            ],
            support_access_enabled: false,
            support_access_permissions: vec![
                "control.read-console".into(),
                "files.read-content".into(),
            ],
            support_access_default_minutes: 240,
            support_access_max_minutes: 43200,
            revoke_access_on_resolved: true,
            auto_close_enabled: true,
            inactivity_days: 5,
            final_close_delay_days: 2,
        }
    }
}

#[async_trait::async_trait]
impl SettingsSerializeExt for ExtensionSettingsData {
    async fn serialize(
        &self,
        serializer: SettingsSerializer,
    ) -> Result<SettingsSerializer, anyhow::Error> {
        Ok(serializer.write_serde_setting("configuration", self)?)
    }
}

pub struct ExtensionSettingsDataDeserializer;

#[async_trait::async_trait]
impl SettingsDeserializeExt for ExtensionSettingsDataDeserializer {
    async fn deserialize_boxed(
        &self,
        deserializer: SettingsDeserializer<'_>,
    ) -> Result<ExtensionSettings, anyhow::Error> {
        Ok(Box::new(
            deserializer
                .read_serde_setting::<ExtensionSettingsData>("configuration")
                .unwrap_or_default(),
        ))
    }
}
