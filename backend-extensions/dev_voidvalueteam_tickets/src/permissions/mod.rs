use indexmap::IndexMap;
use shared::{extensions::ExtensionPermissionsBuilder, permissions::PermissionGroup};

pub fn register(builder: ExtensionPermissionsBuilder) -> ExtensionPermissionsBuilder {
    builder
        .add_user_permission_group(
            "tickets",
            PermissionGroup {
                description: "Permissions for personal support tickets.",
                permissions: IndexMap::from([
                    ("read", "View own tickets."),
                    ("create", "Create tickets."),
                    ("reply", "Reply to own tickets."),
                    ("close", "Close own tickets."),
                    ("reopen", "Reopen eligible tickets."),
                    ("attachments", "Upload and download own ticket attachments."),
                ]),
            },
        )
        .add_server_permission_group(
            "support",
            PermissionGroup {
                description: "Support tickets associated with this server.",
                permissions: IndexMap::from([
                    ("read", "View server support tickets."),
                    ("create", "Create a server support ticket."),
                    ("reply", "Reply to server tickets."),
                    ("grant-access", "Grant approved temporary support access."),
                    ("revoke-access", "Revoke temporary support access."),
                    ("attachments", "Manage server ticket attachments."),
                ]),
            },
        )
        .add_admin_permission_group(
            "support",
            PermissionGroup {
                description: "Staff helpdesk permissions.",
                permissions: IndexMap::from([
                    ("read", "View support tickets."),
                    ("reply", "Reply as staff."),
                    ("internal-note", "Create internal notes."),
                    ("assign", "Assign tickets."),
                    ("update-status", "Change ticket status."),
                    ("update-priority", "Change ticket priority."),
                    ("move-department", "Move tickets between departments."),
                    ("request-access", "Request temporary server access."),
                    ("manage-departments", "Manage departments."),
                    ("manage-saved-replies", "Manage saved replies."),
                    ("manage-automation", "Manage automation rules."),
                    ("view-statistics", "View support statistics."),
                    ("manage-settings", "Manage support settings."),
                ]),
            },
        )
}
