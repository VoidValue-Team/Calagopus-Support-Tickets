use shared::extensions::email_templates::{EmailTemplate, ExtensionEmailTemplateBuilder};
pub fn register(builder: ExtensionEmailTemplateBuilder) -> ExtensionEmailTemplateBuilder {
    let mut b = builder;
    macro_rules! template {
        ($id:literal,$subject:literal,$file:literal,$vars:expr) => {
            b = b.add_template(EmailTemplate {
                identifier: concat!("dev.voidvalueteam.tickets.", $id),
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
