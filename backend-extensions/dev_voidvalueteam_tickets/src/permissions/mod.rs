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
                    ("edit", "Edit ticket subject and affected server."),
                    ("attachments", "Upload and download ticket attachments."),
                    ("delete", "Permanently delete support tickets."),
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
