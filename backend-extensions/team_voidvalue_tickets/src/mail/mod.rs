use crate::models::TicketDetail;
use shared::{
    State,
    extensions::email_templates::{EmailTemplate, ExtensionEmailTemplateBuilder},
    models::{ByUuid, user::User},
};
pub fn register(builder: ExtensionEmailTemplateBuilder) -> ExtensionEmailTemplateBuilder {
    let mut b = builder;
    macro_rules! template {
        ($id:literal,$subject:literal,$file:literal,$vars:expr) => {
            b = b.add_template(EmailTemplate {
                identifier: concat!("team.voidvalue.tickets.", $id),
                available_variables: $vars,
                default_subject: $subject,
                default_content: include_str!(concat!("../../mails/", $file)),
                default_enabled: true,
            });
        };
    }
    template!(
        "ticket-created-customer",
        "{{ settings.app.name }} - Ticket {{ ticket.code }} created",
        "ticket-created-customer.html",
        vec!["user", "ticket", "department", "server", "ticket_url"]
    );
    template!(
        "ticket-created-staff",
        "{{ settings.app.name }} - New ticket {{ ticket.code }}",
        "ticket-created-staff.html",
        vec!["user", "ticket", "department", "server", "ticket_url"]
    );
    template!(
        "ticket-staff-reply",
        "{{ settings.app.name }} - Reply to {{ ticket.code }}",
        "ticket-staff-reply.html",
        vec!["user", "staff", "ticket", "message", "ticket_url"]
    );
    template!(
        "ticket-customer-reply",
        "{{ settings.app.name }} - Customer replied to {{ ticket.code }}",
        "ticket-customer-reply.html",
        vec!["user", "ticket", "message", "ticket_url"]
    );
    template!(
        "ticket-assigned",
        "{{ settings.app.name }} - Ticket {{ ticket.code }} assigned",
        "ticket-assigned.html",
        vec!["staff", "ticket", "ticket_url"]
    );
    template!(
        "ticket-resolved",
        "{{ settings.app.name }} - Ticket {{ ticket.code }} resolved",
        "ticket-resolved.html",
        vec!["user", "ticket", "ticket_url"]
    );
    template!(
        "ticket-closed",
        "{{ settings.app.name }} - Ticket {{ ticket.code }} closed",
        "ticket-closed.html",
        vec!["user", "ticket", "ticket_url"]
    );
    template!(
        "ticket-reopened",
        "{{ settings.app.name }} - Ticket {{ ticket.code }} reopened",
        "ticket-reopened.html",
        vec!["user", "ticket", "ticket_url"]
    );
    template!(
        "access-granted",
        "{{ settings.app.name }} - Support access granted",
        "access-granted.html",
        vec!["user", "staff", "ticket", "server", "ticket_url"]
    );
    template!(
        "access-requested",
        "{{ settings.app.name }} - Support access requested",
        "access-requested.html",
        vec!["user", "staff", "ticket", "server", "ticket_url"]
    );
    template!(
        "access-expiring",
        "{{ settings.app.name }} - Support access expiring",
        "access-expiring.html",
        vec!["user", "staff", "ticket", "server", "ticket_url"]
    );
    template!(
        "sla-warning",
        "{{ settings.app.name }} - SLA warning for {{ ticket.code }}",
        "sla-warning.html",
        vec!["staff", "ticket", "department", "ticket_url"]
    );
    b
}

pub async fn send_customer(state: &State, template: &str, ticket: &TicketDetail) {
    let enabled = state.settings.get().await.ok().and_then(|settings| {
        settings
            .find_extension_settings::<crate::settings::ExtensionSettingsData>()
            .ok()
            .map(|extension| extension.customer_emails)
    });
    if enabled != Some(true) {
        return;
    }
    let Ok(user) = User::by_uuid(&state.database, ticket.ticket.user_uuid).await else {
        tracing::warn!(ticket = %ticket.ticket.uuid, "ticket customer unavailable for email");
        return;
    };
    let Ok(settings) = state.settings.get().await else {
        return;
    };
    let ticket_url = format!(
        "{}/account/support/{}",
        settings.app.url.trim_end_matches('/'),
        ticket.ticket.uuid
    );
    drop(settings);
    state
        .mail
        .send_template(
            state,
            &format!("team.voidvalue.tickets.{template}"),
            user.email.clone(),
            minijinja::context! {
                user => user,
                ticket => &ticket.ticket,
                department => serde_json::json!({"name": ticket.ticket.department_name}),
                server => serde_json::json!({"name": ticket.ticket.server_name}),
                ticket_url => ticket_url,
            },
        )
        .await;
}

pub async fn send_staff(state: &State, template: &str, ticket: &TicketDetail) {
    let enabled = state.settings.get().await.ok().and_then(|settings| {
        settings
            .find_extension_settings::<crate::settings::ExtensionSettingsData>()
            .ok()
            .map(|extension| extension.staff_emails)
    });
    if enabled != Some(true) {
        return;
    }
    let recipients: Vec<String> = match sqlx::query_scalar(
        r#"
        SELECT DISTINCT users.email
        FROM users
        LEFT JOIN roles ON roles.uuid=users.role_uuid
        LEFT JOIN team_voidvalue_tickets_department_staff department_staff
          ON department_staff.user_uuid=users.uuid
         AND department_staff.department_uuid=$1
        WHERE NOT users.suspended AND (
          users.uuid=$2 OR department_staff.user_uuid IS NOT NULL OR users.admin
          OR 'support.read'=ANY(COALESCE(roles.admin_permissions,'{}'))
        )
        "#,
    )
    .bind(ticket.ticket.department_uuid)
    .bind(ticket.ticket.assigned_staff_uuid)
    .fetch_all(state.database.read())
    .await
    {
        Ok(recipients) => recipients,
        Err(error) => {
            tracing::warn!(%error, ticket = %ticket.ticket.uuid, "failed to find ticket email recipients");
            return;
        }
    };
    let Ok(user) = User::by_uuid(&state.database, ticket.ticket.user_uuid).await else {
        return;
    };
    let Ok(settings) = state.settings.get().await else {
        return;
    };
    let ticket_url = format!(
        "{}/admin/support/{}",
        settings.app.url.trim_end_matches('/'),
        ticket.ticket.uuid
    );
    drop(settings);
    for recipient in recipients {
        state
            .mail
            .send_template(
                state,
                &format!("team.voidvalue.tickets.{template}"),
                recipient.into(),
                minijinja::context! {
                    user => &user,
                    staff => serde_json::json!({"username": ticket.ticket.assigned_staff_name}),
                    ticket => &ticket.ticket,
                    department => serde_json::json!({"name": ticket.ticket.department_name}),
                    server => serde_json::json!({"name": ticket.ticket.server_name}),
                    ticket_url => &ticket_url,
                },
            )
            .await;
    }
}
