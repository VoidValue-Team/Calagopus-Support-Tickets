use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "team_voidvalue_tickets_status", rename_all = "snake_case")]
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

    pub fn can_be_set_by_customer(self) -> bool {
        matches!(self, Self::Open | Self::Closed)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "team_voidvalue_tickets_priority",
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
    type_name = "team_voidvalue_tickets_message_type",
    rename_all = "snake_case"
)]
pub enum MessageType {
    Customer,
    Staff,
    InternalNote,
    System,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "team_voidvalue_tickets_access_status",
    rename_all = "snake_case"
)]
pub enum AccessStatus {
    Pending,
    Approved,
    Rejected,
    Revoked,
    Expired,
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
pub struct TicketHistory {
    pub uuid: Uuid,
    pub actor_uuid: Option<Uuid>,
    pub actor_name: Option<String>,
    pub event: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct StaffOption {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct TicketTag {
    pub uuid: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct AccessRequest {
    pub uuid: Uuid,
    pub requested_by_uuid: Uuid,
    pub requested_by_name: String,
    pub permissions: Vec<String>,
    pub requested_duration_minutes: i32,
    pub reason: String,
    pub status: AccessStatus,
    pub reviewed_by_uuid: Option<Uuid>,
    pub reviewed_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, FromRow, Serialize, ToSchema)]
pub struct AccessGrant {
    pub uuid: Uuid,
    pub request_uuid: Option<Uuid>,
    pub server_uuid: Uuid,
    pub user_uuid: Uuid,
    pub user_name: String,
    pub permissions: Vec<String>,
    pub reason: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct TicketDetail {
    #[serde(flatten)]
    pub ticket: TicketSummary,
    pub messages: Vec<TicketMessage>,
    pub attachments: Vec<TicketAttachment>,
    pub history: Vec<TicketHistory>,
    pub access_requests: Vec<AccessRequest>,
    pub access_grants: Vec<AccessGrant>,
    pub tags: Vec<TicketTag>,
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
pub struct AccessRequestPayload {
    #[garde(length(min = 1, max = 64), inner(length(chars, min = 1, max = 128)))]
    pub permissions: Vec<String>,
    #[garde(range(min = 1, max = 525600))]
    pub duration_minutes: u32,
    #[garde(length(chars, min = 3, max = 500))]
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessReviewAction {
    Approve,
    Reject,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AccessReviewPayload {
    #[garde(skip)]
    pub action: AccessReviewAction,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TagsPayload {
    #[garde(length(max = 20), inner(length(chars, min = 1, max = 40)))]
    pub tags: Vec<String>,
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
    pub server_uuid: Option<Uuid>,
    #[garde(length(chars, max = 180))]
    pub server_search: Option<String>,
    #[garde(skip)]
    pub user_uuid: Option<Uuid>,
    #[garde(length(chars, max = 180))]
    pub user_search: Option<String>,
    #[garde(skip)]
    pub sla_breached: Option<bool>,
    #[garde(length(chars, max = 40))]
    pub tag: Option<String>,
    #[garde(skip)]
    pub created_from: Option<DateTime<Utc>>,
    #[garde(skip)]
    pub created_to: Option<DateTime<Utc>>,
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
    use super::{
        CreateTicketPayload, DepartmentPayload, ListQuery, ReplyPayload, TicketPriority,
        TicketStatus::*,
    };
    use garde::Validate;
    use uuid::Uuid;

    #[test]
    fn closed_requires_explicit_reopen() {
        assert!(!Closed.can_transition_to(Open, false));
        assert!(Closed.can_transition_to(Open, true));
        assert!(!Closed.can_transition_to(AwaitingStaff, true));
    }

    #[test]
    fn resolved_requires_explicit_reopen() {
        assert!(!Resolved.can_transition_to(Open, false));
        assert!(Resolved.can_transition_to(Open, true));
        assert!(Resolved.can_transition_to(Closed, false));
    }

    #[test]
    fn active_statuses_allow_normal_staff_transitions() {
        assert!(Open.can_transition_to(AwaitingStaff, false));
        assert!(AwaitingStaff.can_transition_to(InProgress, false));
        assert!(InProgress.can_transition_to(Resolved, false));
    }

    #[test]
    fn customers_can_only_request_open_or_closed() {
        assert!(Open.can_be_set_by_customer());
        assert!(Closed.can_be_set_by_customer());
        assert!(!AwaitingStaff.can_be_set_by_customer());
        assert!(!InProgress.can_be_set_by_customer());
    }

    #[test]
    fn create_ticket_payload_enforces_text_limits() {
        let valid = CreateTicketPayload {
            subject: "Server cannot start".into(),
            message: "The process exits immediately.".into(),
            department_uuid: Uuid::new_v4(),
            server_uuid: None,
            priority: Some(TicketPriority::High),
        };
        assert!(valid.validate().is_ok());

        let invalid = CreateTicketPayload {
            subject: "No".into(),
            message: String::new(),
            ..valid
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn reply_payload_rejects_empty_and_oversized_messages() {
        assert!(
            ReplyPayload {
                message: String::new(),
                internal_note: None,
            }
            .validate()
            .is_err()
        );
        assert!(
            ReplyPayload {
                message: "x".repeat(20_001),
                internal_note: Some(true),
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn department_payload_enforces_sla_ranges() {
        let invalid = DepartmentPayload {
            name: "Technical Support".into(),
            description: String::new(),
            enabled: true,
            position: 0,
            default_priority: TicketPriority::Normal,
            first_response_sla_minutes: 0,
            resolution_sla_minutes: 525_601,
            autoresponse: None,
            allow_server_access: false,
            notification_enabled: true,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn list_query_rejects_invalid_pagination() {
        let query = ListQuery {
            page: Some(0),
            per_page: Some(101),
            search: None,
            view: None,
            status: None,
            priority: None,
            department_uuid: None,
            assigned_staff_uuid: None,
            server_uuid: None,
            server_search: None,
            user_uuid: None,
            user_search: None,
            sla_breached: None,
            tag: None,
            created_from: None,
            created_to: None,
        };
        assert!(query.validate().is_err());
    }
}
