use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "dev_voidvalueteam_tickets_status",
    rename_all = "snake_case"
)]
pub enum TicketStatus {
    Open,
    AwaitingStaff,
    AwaitingCustomer,
    InProgress,
    Resolved,
    Closed,
}

impl TicketStatus {
    pub fn can_transition_to(self, next: Self, reopen_allowed: bool) -> bool {
        use TicketStatus::*;
        match (self, next) {
            (a, b) if a == b => true,
            (Closed, Open) => reopen_allowed,
            (Closed, _) => false,
            (_, Closed) | (_, Resolved) => true,
            (Resolved, Open | InProgress | AwaitingStaff | AwaitingCustomer) => reopen_allowed,
            (_, Open | InProgress | AwaitingStaff | AwaitingCustomer) => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "dev_voidvalueteam_tickets_priority",
    rename_all = "snake_case"
)]
pub enum TicketPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TicketView {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "dev_voidvalueteam_tickets_message_type",
    rename_all = "snake_case"
)]
pub enum MessageType {
    Customer,
    Staff,
    InternalNote,
    System,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct Department {
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub position: i32,
    pub default_priority: TicketPriority,
    pub first_response_sla_minutes: i32,
    pub resolution_sla_minutes: i32,
    pub autoresponse: Option<String>,
    pub allow_server_access: bool,
    pub notification_enabled: bool,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct TicketSummary {
    pub uuid: Uuid,
    pub number: i64,
    pub code: String,
    pub user_uuid: Uuid,
    pub user_name: String,
    pub server_uuid: Option<Uuid>,
    pub server_name: Option<String>,
    pub department_uuid: Uuid,
    pub department_name: String,
    pub subject: String,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub assigned_staff_uuid: Option<Uuid>,
    pub assigned_staff_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_reply_at: DateTime<Utc>,
    pub first_response_due_at: DateTime<Utc>,
    pub resolution_due_at: DateTime<Utc>,
    pub first_response_sla_breached: bool,
    pub resolution_sla_breached: bool,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct TicketMessage {
    pub uuid: Uuid,
    pub ticket_uuid: Uuid,
    pub author_uuid: Option<Uuid>,
    pub author_name: Option<String>,
    pub message_type: MessageType,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct TicketAttachment {
    pub uuid: Uuid,
    pub message_uuid: Uuid,
    pub uploader_uuid: Option<Uuid>,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sha256: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct SupportAgent {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct SupportServer {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PublicSettings {
    pub enabled: bool,
    pub allow_user_priority: bool,
    pub allow_reopen: bool,
    pub reopen_period_days: u32,
    pub default_department: Option<Uuid>,
    pub attachments_enabled: bool,
    pub attachment_max_bytes: u64,
    pub attachment_max_files: u32,
    pub allowed_mime_types: Vec<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct TicketDetail {
    #[serde(flatten)]
    pub ticket: TicketSummary,
    pub messages: Vec<TicketMessage>,
    pub attachments: Vec<TicketAttachment>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTicketPayload {
    #[garde(length(chars, min = 3, max = 180))]
    #[schema(min_length = 3, max_length = 180)]
    pub subject: String,
    #[garde(length(chars, min = 1, max = 20000))]
    #[schema(min_length = 1, max_length = 20000)]
    pub message: String,
    #[garde(skip)]
    pub department_uuid: Uuid,
    #[garde(skip)]
    pub server_uuid: Option<Uuid>,
    #[garde(skip)]
    pub priority: Option<TicketPriority>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ReplyPayload {
    #[garde(length(chars, min = 1, max = 20000))]
    #[schema(min_length = 1, max_length = 20000)]
    pub message: String,
    #[garde(skip)]
    pub internal_note: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct StatusPayload {
    #[garde(skip)]
    pub status: TicketStatus,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignPayload {
    #[garde(skip)]
    pub staff_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct EditTicketPayload {
    #[garde(length(chars, min = 3, max = 180))]
    #[schema(min_length = 3, max_length = 180)]
    pub subject: String,
    #[garde(skip)]
    pub department_uuid: Uuid,
    #[garde(skip)]
    pub priority: TicketPriority,
    #[garde(skip)]
    pub server_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct DepartmentPayload {
    #[garde(length(chars, min = 2, max = 80))]
    pub name: String,
    #[garde(length(chars, max = 500))]
    pub description: String,
    #[garde(skip)]
    pub enabled: bool,
    #[garde(range(min = 0, max = 10000))]
    pub position: i32,
    #[garde(skip)]
    pub default_priority: TicketPriority,
    #[garde(range(min = 1, max = 525600))]
    pub first_response_sla_minutes: i32,
    #[garde(range(min = 1, max = 525600))]
    pub resolution_sla_minutes: i32,
    #[garde(length(chars, max = 20000))]
    pub autoresponse: Option<String>,
    #[garde(skip)]
    pub allow_server_access: bool,
    #[garde(skip)]
    pub notification_enabled: bool,
}

#[derive(Debug, Deserialize, Validate, IntoParams, ToSchema)]
pub struct ListQuery {
    #[garde(range(min = 1))]
    pub page: Option<i64>,
    #[garde(range(min = 1, max = 100))]
    pub per_page: Option<i64>,
    #[garde(length(chars, max = 180))]
    pub search: Option<String>,
    #[garde(skip)]
    pub view: Option<TicketView>,
    #[garde(skip)]
    pub status: Option<TicketStatus>,
    #[garde(skip)]
    pub priority: Option<TicketPriority>,
    #[garde(skip)]
    pub department_uuid: Option<Uuid>,
    #[garde(skip)]
    pub assigned_staff_uuid: Option<Uuid>,
    #[garde(skip)]
    pub sla_breached: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T> {
    pub total: i64,
    pub per_page: i64,
    pub page: i64,
    pub data: Vec<T>,
}

#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct Statistics {
    pub open: i64,
    pub unassigned: i64,
    pub awaiting_staff: i64,
    pub awaiting_customer: i64,
    pub urgent: i64,
    pub sla_breached: i64,
    pub resolved_today: i64,
    pub created_today: i64,
}

#[cfg(test)]
mod tests {
    use super::TicketStatus::*;
    #[test]
    fn closed_requires_explicit_reopen() {
        assert!(!Closed.can_transition_to(Open, false));
        assert!(Closed.can_transition_to(Open, true));
        assert!(!Closed.can_transition_to(AwaitingStaff, true));
    }
}
