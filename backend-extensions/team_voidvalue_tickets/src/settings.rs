use garde::Validate;
use serde::{Deserialize, Serialize};
use shared::extensions::settings::{
    ExtensionSettings, SettingsDeserializeExt, SettingsDeserializer, SettingsSerializeExt,
    SettingsSerializer,
};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct ExtensionSettingsData {
    #[garde(skip)]
    pub enabled: bool,
    #[garde(length(chars, min = 1, max = 12))]
    pub ticket_prefix: String,
    #[garde(skip)]
    pub allow_user_priority: bool,
    #[garde(skip)]
    pub allow_reopen: bool,
    #[garde(range(max = 3650))]
    pub reopen_period_days: u32,
    #[garde(skip)]
    pub default_department: Option<uuid::Uuid>,
    #[garde(skip)]
    pub customer_emails: bool,
    #[garde(skip)]
    pub staff_emails: bool,
    #[garde(skip)]
    pub attachments_enabled: bool,
    #[garde(range(min = 1024, max = 104857600))]
    pub attachment_max_bytes: u64,
    #[garde(range(min = 1, max = 20))]
    pub attachment_max_files: u32,
    #[garde(length(min = 1, max = 32), inner(length(chars, min = 3, max = 128)))]
    pub allowed_mime_types: Vec<String>,
    #[garde(skip)]
    pub support_access_enabled: bool,
    #[garde(length(max = 64), inner(length(chars, min = 1, max = 128)))]
    pub support_access_permissions: Vec<String>,
    #[garde(range(min = 1, max = 525600))]
    pub support_access_default_minutes: u32,
    #[garde(range(min = 1, max = 525600))]
    pub support_access_max_minutes: u32,
    #[garde(skip)]
    pub revoke_access_on_resolved: bool,
    #[garde(skip)]
    pub auto_close_enabled: bool,
    #[garde(range(min = 1, max = 3650))]
    pub inactivity_days: u32,
    #[garde(range(min = 1, max = 3650))]
    pub final_close_delay_days: u32,
}

impl ExtensionSettingsData {
    pub fn validate_consistency(&self) -> anyhow::Result<()> {
        if self.support_access_default_minutes > self.support_access_max_minutes {
            anyhow::bail!("default support access duration exceeds its maximum");
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::ExtensionSettingsData;
    use garde::Validate;

    #[test]
    fn defaults_are_valid_and_keep_support_access_disabled() {
        let settings = ExtensionSettingsData::default();
        assert!(settings.validate().is_ok());
        assert!(settings.validate_consistency().is_ok());
        assert!(!settings.support_access_enabled);
    }

    #[test]
    fn invalid_attachment_limits_are_rejected() {
        let settings = ExtensionSettingsData {
            attachment_max_files: 0,
            ..ExtensionSettingsData::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn support_access_default_cannot_exceed_maximum() {
        let settings = ExtensionSettingsData {
            support_access_default_minutes: 120,
            support_access_max_minutes: 60,
            ..ExtensionSettingsData::default()
        };
        assert!(settings.validate_consistency().is_err());
    }
}
