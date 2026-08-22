use crate::{
    models::{
        AccessRequestPayload, AccessReviewAction, AccessReviewPayload, AccessStatus, TicketDetail,
    },
    repository::{TicketScope, add_history_tx, get_ticket},
    settings::ExtensionSettingsData,
};
use chrono::{Duration, Utc};
use shared::{
    State,
    models::{
        ByUuid, CreatableModel, DeletableModel, UpdatableModel,
        server::Server,
        server_subuser::{CreateServerSubuserOptions, ServerSubuser, UpdateServerSubuserOptions},
        user::User,
    },
};
use uuid::Uuid;

pub async fn create_request(
    state: &State,
    ticket_uuid: Uuid,
    staff_uuid: Uuid,
    payload: AccessRequestPayload,
) -> anyhow::Result<TicketDetail> {
    let settings = state.settings.get().await?;
    let extension: &ExtensionSettingsData = settings.find_extension_settings()?;
    if !extension.support_access_enabled {
        anyhow::bail!("temporary support access is disabled");
    }
    if payload.duration_minutes > extension.support_access_max_minutes {
        anyhow::bail!("temporary support access duration exceeds its maximum");
    }
    if !payload
        .permissions
        .iter()
        .all(|permission| extension.support_access_permissions.contains(permission))
    {
        anyhow::bail!("temporary support access contains a disallowed permission");
    }
    drop(settings);
    let mut tx = state.database.write().begin().await?;
    let available: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
          SELECT 1 FROM team_voidvalue_tickets_tickets ticket
          JOIN team_voidvalue_tickets_departments department ON department.uuid=ticket.department_uuid
          WHERE ticket.uuid=$1 AND ticket.server_uuid IS NOT NULL AND department.allow_server_access
            AND ticket.status NOT IN ('resolved','closed')
        )
        "#,
    )
    .bind(ticket_uuid)
    .fetch_one(&mut *tx)
    .await?;
    if !available {
        anyhow::bail!("temporary support access is unavailable for this ticket");
    }
    sqlx::query(
        r#"INSERT INTO team_voidvalue_tickets_access_requests
           (uuid,ticket_uuid,requested_by_uuid,permissions,requested_duration_minutes,reason)
           VALUES($1,$2,$3,$4,$5,$6)"#,
    )
    .bind(Uuid::new_v4())
    .bind(ticket_uuid)
    .bind(staff_uuid)
    .bind(&payload.permissions)
    .bind(i32::try_from(payload.duration_minutes)?)
    .bind(&payload.reason)
    .execute(&mut *tx)
    .await?;
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(staff_uuid),
        "ticket.access.requested",
        serde_json::json!({"permissions":payload.permissions,"duration_minutes":payload.duration_minutes}),
    )
    .await?;
    tx.commit().await?;
    get_ticket(state, ticket_uuid, TicketScope::Admin, true)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

#[derive(sqlx::FromRow)]
struct ReviewRow {
    ticket_uuid: Uuid,
    owner_uuid: Uuid,
    server_uuid: Uuid,
    requested_by_uuid: Uuid,
    permissions: Vec<String>,
    requested_duration_minutes: i32,
    reason: String,
}

pub async fn review_request(
    state: &State,
    ticket_uuid: Uuid,
    request_uuid: Uuid,
    owner_uuid: Uuid,
    payload: AccessReviewPayload,
) -> anyhow::Result<TicketDetail> {
    let mut tx = state.database.write().begin().await?;
    let request = sqlx::query_as::<_, ReviewRow>(
        r#"
        SELECT request.ticket_uuid,ticket.user_uuid AS owner_uuid,ticket.server_uuid,
               request.requested_by_uuid,request.permissions,request.requested_duration_minutes,
               request.reason
        FROM team_voidvalue_tickets_access_requests request
        JOIN team_voidvalue_tickets_tickets ticket ON ticket.uuid=request.ticket_uuid
        WHERE request.uuid=$1 AND request.ticket_uuid=$2 AND request.status='pending'
        FOR UPDATE OF request,ticket
        "#,
    )
    .bind(request_uuid)
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("pending access request not found"))?;
    if request.owner_uuid != owner_uuid {
        anyhow::bail!("ticket not found");
    }
    if payload.action == AccessReviewAction::Reject {
        sqlx::query(
            "UPDATE team_voidvalue_tickets_access_requests SET status='rejected',reviewed_by_uuid=$2,reviewed_at=now() WHERE uuid=$1",
        )
        .bind(request_uuid)
        .bind(owner_uuid)
        .execute(&mut *tx)
        .await?;
        add_history_tx(
            &mut tx,
            ticket_uuid,
            Some(owner_uuid),
            "ticket.access.rejected",
            serde_json::json!({"request_uuid":request_uuid}),
        )
        .await?;
        tx.commit().await?;
        return get_ticket(state, ticket_uuid, TicketScope::User(owner_uuid), false)
            .await?
            .ok_or_else(|| anyhow::anyhow!("ticket not found"));
    }

    let server = Server::by_uuid_with_transaction(&mut tx, request.server_uuid).await?;
    let staff = User::by_uuid_with_transaction(&mut tx, request.requested_by_uuid).await?;
    sqlx::query(
        "SELECT user_uuid FROM server_subusers WHERE server_uuid=$1 AND user_uuid=$2 FOR UPDATE",
    )
    .bind(server.uuid)
    .bind(staff.uuid)
    .fetch_optional(&mut *tx)
    .await?;
    let existing =
        ServerSubuser::by_server_uuid_user_uuid(&state.database, server.uuid, staff.uuid).await?;
    let prior_permissions = existing.as_ref().map(|subuser| {
        subuser
            .permissions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    });
    let created_subuser = existing.is_none();
    let mut effective_permissions = prior_permissions.clone().unwrap_or_default();
    for permission in &request.permissions {
        if !effective_permissions.contains(permission) {
            effective_permissions.push(permission.clone());
        }
    }
    if let Some(mut subuser) = existing {
        subuser
            .update_with_transaction(
                state,
                UpdateServerSubuserOptions {
                    permissions: Some(
                        effective_permissions
                            .iter()
                            .cloned()
                            .map(Into::into)
                            .collect(),
                    ),
                    ignored_files: None,
                },
                &mut tx,
            )
            .await?;
    } else {
        ServerSubuser::create_with_transaction(
            state,
            CreateServerSubuserOptions {
                server: &server,
                email: staff.email.clone(),
                permissions: effective_permissions
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
                ignored_files: Vec::new(),
            },
            &mut tx,
        )
        .await?;
    }
    let expires_at = Utc::now() + Duration::minutes(i64::from(request.requested_duration_minutes));
    sqlx::query(
        "UPDATE team_voidvalue_tickets_access_requests SET status='approved',reviewed_by_uuid=$2,reviewed_at=now() WHERE uuid=$1",
    )
    .bind(request_uuid)
    .bind(owner_uuid)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"INSERT INTO team_voidvalue_tickets_access_grants
           (uuid,ticket_uuid,request_uuid,server_uuid,user_uuid,granted_by_uuid,permissions,
            prior_subuser_permissions,created_subuser,reason,expires_at)
           VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
    )
    .bind(Uuid::new_v4())
    .bind(request.ticket_uuid)
    .bind(request_uuid)
    .bind(server.uuid)
    .bind(staff.uuid)
    .bind(owner_uuid)
    .bind(&request.permissions)
    .bind(prior_permissions)
    .bind(created_subuser)
    .bind(&request.reason)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?;
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(owner_uuid),
        "ticket.access.approved",
        serde_json::json!({"request_uuid":request_uuid,"expires_at":expires_at}),
    )
    .await?;
    tx.commit().await?;
    sync_permissions(state, &server, staff.uuid, effective_permissions, false).await;
    get_ticket(state, ticket_uuid, TicketScope::User(owner_uuid), false)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

#[derive(sqlx::FromRow)]
struct GrantRow {
    ticket_uuid: Uuid,
    request_uuid: Option<Uuid>,
    server_uuid: Uuid,
    user_uuid: Uuid,
    prior_subuser_permissions: Option<Vec<String>>,
    created_subuser: bool,
}

pub async fn revoke_grant(
    state: &State,
    grant_uuid: Uuid,
    actor_uuid: Option<Uuid>,
    reason: &str,
    expired: bool,
) -> anyhow::Result<Option<Uuid>> {
    let mut tx = state.database.write().begin().await?;
    let grant = sqlx::query_as::<_, GrantRow>(
        r#"SELECT ticket_uuid,request_uuid,server_uuid,user_uuid,prior_subuser_permissions,created_subuser
           FROM team_voidvalue_tickets_access_grants
           WHERE uuid=$1 AND revoked_at IS NULL FOR UPDATE"#,
    )
    .bind(grant_uuid)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(grant) = grant else {
        return Ok(None);
    };
    let server = Server::by_uuid_with_transaction(&mut tx, grant.server_uuid).await?;
    if let Some(mut subuser) =
        ServerSubuser::by_server_uuid_user_uuid(&state.database, server.uuid, grant.user_uuid)
            .await?
    {
        if grant.created_subuser {
            subuser.delete_with_transaction(state, (), &mut tx).await?;
        } else {
            subuser
                .update_with_transaction(
                    state,
                    UpdateServerSubuserOptions {
                        permissions: Some(
                            grant
                                .prior_subuser_permissions
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                        ),
                        ignored_files: None,
                    },
                    &mut tx,
                )
                .await?;
        }
    }
    sqlx::query(
        "UPDATE team_voidvalue_tickets_access_grants SET revoked_at=now(),revoked_by_uuid=$2,revoke_reason=$3 WHERE uuid=$1",
    )
    .bind(grant_uuid)
    .bind(actor_uuid)
    .bind(reason)
    .execute(&mut *tx)
    .await?;
    if let Some(request_uuid) = grant.request_uuid {
        sqlx::query("UPDATE team_voidvalue_tickets_access_requests SET status=$2 WHERE uuid=$1")
            .bind(request_uuid)
            .bind(if expired {
                AccessStatus::Expired
            } else {
                AccessStatus::Revoked
            })
            .execute(&mut *tx)
            .await?;
    }
    add_history_tx(
        &mut tx,
        grant.ticket_uuid,
        actor_uuid,
        if expired {
            "ticket.access.expired"
        } else {
            "ticket.access.revoked"
        },
        serde_json::json!({"grant_uuid":grant_uuid,"reason":reason}),
    )
    .await?;
    tx.commit().await?;
    sync_permissions(
        state,
        &server,
        grant.user_uuid,
        grant.prior_subuser_permissions.unwrap_or_default(),
        grant.created_subuser,
    )
    .await;
    Ok(Some(grant.ticket_uuid))
}

pub async fn revoke_ticket_grants(
    state: &State,
    ticket_uuid: Uuid,
    actor_uuid: Option<Uuid>,
    reason: &str,
) -> anyhow::Result<()> {
    let grants: Vec<Uuid> = sqlx::query_scalar(
        "SELECT uuid FROM team_voidvalue_tickets_access_grants WHERE ticket_uuid=$1 AND revoked_at IS NULL",
    )
    .bind(ticket_uuid)
    .fetch_all(state.database.read())
    .await?;
    for grant in grants {
        revoke_grant(state, grant, actor_uuid, reason, false).await?;
    }
    Ok(())
}

async fn sync_permissions(
    state: &State,
    server: &Server,
    user_uuid: Uuid,
    permissions: Vec<String>,
    deny: bool,
) {
    let Ok(node) = server.node.fetch_cached(&state.database).await else {
        return;
    };
    let Ok(client) = node.api_client(&state.database).await else {
        return;
    };
    if let Err(error) = client
        .post_servers_server_ws_permissions(
            server.uuid,
            &wings_api::servers_server_ws_permissions::post::RequestBody {
                user_permissions: vec![
                    wings_api::servers_server_ws_permissions::post::RequestBodyUserPermissions {
                        user: user_uuid,
                        permissions: permissions.into_iter().map(Into::into).collect(),
                        ignored_files: Vec::new(),
                    },
                ],
            },
        )
        .await
    {
        tracing::error!(server = %server.uuid, user = %user_uuid, error = ?error, "failed to sync temporary support permissions");
    }
    if deny
        && let Err(error) = client
            .post_servers_server_ws_deny(
                server.uuid,
                &wings_api::servers_server_ws_deny::post::RequestBody {
                    jtis: vec![user_uuid.to_string().into()],
                },
            )
            .await
    {
        tracing::error!(server = %server.uuid, user = %user_uuid, error = ?error, "failed to invalidate temporary support sessions");
    }
}
