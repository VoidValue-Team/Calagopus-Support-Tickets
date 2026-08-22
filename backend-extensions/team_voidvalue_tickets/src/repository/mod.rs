use crate::models::*;
use chrono::{Duration, Utc};
use shared::State;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

pub mod access;

const SUMMARY_COLUMNS: &str = r#"
 t.uuid, t.number, t.code, t.user_uuid, ticket_user.username AS user_name,
 COALESCE(t.server_uuid,t.former_server_uuid) AS server_uuid,
 COALESCE(ticket_server.name,t.former_server_name) AS server_name, t.department_uuid,
 d.name AS department_name, t.subject, t.status, t.priority, t.assigned_staff_uuid,
 assigned_staff.username AS assigned_staff_name,
 t.created_at, t.updated_at, t.last_reply_at, t.first_response_due_at,
 t.resolution_due_at, t.first_response_sla_breached, t.resolution_sla_breached
"#;

const SUMMARY_JOINS: &str = r#"
 JOIN team_voidvalue_tickets_departments d ON d.uuid=t.department_uuid
 JOIN users ticket_user ON ticket_user.uuid=t.user_uuid
 LEFT JOIN servers ticket_server ON ticket_server.uuid=t.server_uuid
 LEFT JOIN users assigned_staff ON assigned_staff.uuid=t.assigned_staff_uuid
"#;

#[derive(Clone, Copy)]
pub enum TicketScope {
    User(Uuid),
    Server { user_uuid: Uuid, server_uuid: Uuid },
    Admin,
}

pub struct NewAttachment {
    pub original_filename: String,
    pub mime_type: String,
    pub content: Vec<u8>,
    pub sha256: String,
}

pub struct AttachmentDownload {
    pub original_filename: String,
    pub mime_type: String,
    pub content: Vec<u8>,
}

#[derive(sqlx::FromRow)]
struct StatusRow {
    status: TicketStatus,
    user_uuid: Uuid,
    closed_at: Option<chrono::DateTime<Utc>>,
    resolved_at: Option<chrono::DateTime<Utc>>,
    server_uuid: Option<Uuid>,
}

pub async fn list_departments(
    state: &State,
    include_disabled: bool,
) -> anyhow::Result<Vec<Department>> {
    Ok(sqlx::query_as::<_, Department>(
        r#"
        SELECT uuid, name, description, enabled, position, default_priority,
               first_response_sla_minutes, resolution_sla_minutes, autoresponse,
               allow_server_access, notification_enabled
        FROM team_voidvalue_tickets_departments
        WHERE enabled OR $1 ORDER BY position, name
    "#,
    )
    .bind(include_disabled)
    .fetch_all(state.database.read())
    .await?)
}

pub async fn create_ticket(
    state: &State,
    user_uuid: Uuid,
    payload: CreateTicketPayload,
    can_access_all_servers: bool,
) -> anyhow::Result<TicketDetail> {
    ensure_enabled(state).await?;
    let mut tx = state.database.write().begin().await?;
    let department = sqlx::query_as::<_, Department>(
        r#"
        SELECT uuid, name, description, enabled, position, default_priority,
               first_response_sla_minutes, resolution_sla_minutes, autoresponse,
               allow_server_access, notification_enabled
        FROM team_voidvalue_tickets_departments WHERE uuid = $1 AND enabled
        FOR SHARE
    "#,
    )
    .bind(payload.department_uuid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("department unavailable"))?;

    if let Some(server_uuid) = payload.server_uuid {
        let accessible: bool = if can_access_all_servers {
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM servers WHERE uuid=$1)")
                .bind(server_uuid)
                .fetch_one(&mut *tx)
                .await?
        } else {
            sqlx::query_scalar(
                r#"
              SELECT EXISTS(
                SELECT 1 FROM servers s LEFT JOIN server_subusers su
                  ON su.server_uuid = s.uuid AND su.user_uuid = $2
                WHERE s.uuid = $1 AND (s.owner_uuid = $2 OR su.user_uuid = $2)
              )
            "#,
            )
            .bind(server_uuid)
            .bind(user_uuid)
            .fetch_one(&mut *tx)
            .await?
        };
        if !accessible {
            anyhow::bail!("server is not accessible to this user");
        }
    }

    let number: i64 = sqlx::query_scalar("SELECT nextval('team_voidvalue_tickets_number_seq')")
        .fetch_one(&mut *tx)
        .await?;
    let settings = state.settings.get().await?;
    let ext: &crate::settings::ExtensionSettingsData = settings.find_extension_settings()?;
    let code = format!("{}-{number}", ext.ticket_prefix);
    let priority = if ext.allow_user_priority {
        payload.priority.unwrap_or(department.default_priority)
    } else {
        department.default_priority
    };
    drop(settings);
    let uuid = Uuid::new_v4();
    let now = Utc::now();
    let first_due = now + Duration::minutes(i64::from(department.first_response_sla_minutes));
    let resolution_due = now + Duration::minutes(i64::from(department.resolution_sla_minutes));

    sqlx::query(
        r#"
      INSERT INTO team_voidvalue_tickets_tickets
        (uuid, number, code, user_uuid, server_uuid, department_uuid, subject, priority,
         first_response_due_at, resolution_due_at)
      VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
    "#,
    )
    .bind(uuid)
    .bind(number)
    .bind(&code)
    .bind(user_uuid)
    .bind(payload.server_uuid)
    .bind(department.uuid)
    .bind(&payload.subject)
    .bind(priority)
    .bind(first_due)
    .bind(resolution_due)
    .execute(&mut *tx)
    .await?;
    let message_uuid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO team_voidvalue_tickets_messages
      (uuid,ticket_uuid,author_uuid,message_type,body) VALUES ($1,$2,$3,'customer',$4)"#,
    )
    .bind(message_uuid)
    .bind(uuid)
    .bind(user_uuid)
    .bind(&payload.message)
    .execute(&mut *tx)
    .await?;
    add_history_tx(
        &mut tx,
        uuid,
        Some(user_uuid),
        "ticket.created",
        serde_json::json!({"code": code}),
    )
    .await?;
    if let Some(autoresponse) = department.autoresponse.filter(|v| !v.trim().is_empty()) {
        sqlx::query(
            r#"INSERT INTO team_voidvalue_tickets_messages
          (uuid,ticket_uuid,message_type,body) VALUES ($1,$2,'system',$3)"#,
        )
        .bind(Uuid::new_v4())
        .bind(uuid)
        .bind(autoresponse)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    get_ticket(state, uuid, TicketScope::User(user_uuid), false)
        .await?
        .ok_or_else(|| anyhow::anyhow!("created ticket missing"))
}

pub async fn list_tickets(
    state: &State,
    scope: TicketScope,
    query: &ListQuery,
) -> anyhow::Result<Page<TicketSummary>> {
    if !matches!(scope, TicketScope::Admin) {
        ensure_enabled(state).await?;
    }
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).clamp(1, 100);
    let mut qb = QueryBuilder::<Postgres>::new(format!(
        "SELECT {SUMMARY_COLUMNS}, COUNT(*) OVER() AS total_count FROM team_voidvalue_tickets_tickets t {SUMMARY_JOINS} WHERE TRUE"
    ));
    match scope {
        TicketScope::User(id) => {
            qb.push(" AND t.user_uuid = ").push_bind(id);
        }
        TicketScope::Server {
            user_uuid,
            server_uuid,
        } => {
            qb.push(" AND t.user_uuid = ")
                .push_bind(user_uuid)
                .push(" AND t.server_uuid = ")
                .push_bind(server_uuid);
        }
        TicketScope::Admin => {}
    }
    if let Some(view) = query.view {
        match view {
            TicketView::Open => qb.push(" AND t.status NOT IN ('resolved','closed')"),
            TicketView::Closed => qb.push(" AND t.status IN ('resolved','closed')"),
        };
    }
    if let Some(status) = query.status {
        qb.push(" AND t.status = ").push_bind(status);
    }
    if let Some(priority) = query.priority {
        qb.push(" AND t.priority = ").push_bind(priority);
    }
    if let Some(id) = query.department_uuid {
        qb.push(" AND t.department_uuid = ").push_bind(id);
    }
    if let Some(id) = query.assigned_staff_uuid {
        qb.push(" AND t.assigned_staff_uuid = ").push_bind(id);
    }
    if let Some(id) = query.server_uuid {
        qb.push(" AND t.server_uuid = ").push_bind(id);
    }
    if let Some(id) = query.user_uuid {
        qb.push(" AND t.user_uuid = ").push_bind(id);
    }
    if let Some(search) = query
        .server_search
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        qb.push(" AND ticket_server.name ILIKE ")
            .push_bind(format!("%{}%", search.trim()));
    }
    if let Some(search) = query
        .user_search
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        qb.push(" AND ticket_user.username ILIKE ")
            .push_bind(format!("%{}%", search.trim()));
    }
    if query.sla_breached == Some(true) {
        qb.push(" AND (t.first_response_sla_breached OR t.resolution_sla_breached)");
    }
    if let Some(tag) = query
        .tag
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        qb.push(
            " AND EXISTS(SELECT 1 FROM team_voidvalue_tickets_ticket_tags ticket_tag JOIN team_voidvalue_tickets_tags tag ON tag.uuid=ticket_tag.tag_uuid WHERE ticket_tag.ticket_uuid=t.uuid AND tag.name ILIKE ",
        )
        .push_bind(tag.trim())
        .push(")");
    }
    if let Some(created_from) = query.created_from {
        qb.push(" AND t.created_at >= ").push_bind(created_from);
    }
    if let Some(created_to) = query.created_to {
        qb.push(" AND t.created_at <= ").push_bind(created_to);
    }
    if let Some(search) = query.search.as_deref().filter(|v| !v.trim().is_empty()) {
        qb.push(" AND (t.code ILIKE ")
            .push_bind(format!("%{}%", search.trim()))
            .push(" OR to_tsvector('simple', t.subject) @@ plainto_tsquery('simple', ")
            .push_bind(search.trim())
            .push("))");
    }
    qb.push(" ORDER BY t.updated_at DESC LIMIT ")
        .push_bind(per_page)
        .push(" OFFSET ")
        .push_bind((page - 1) * per_page);
    #[derive(sqlx::FromRow)]
    struct Row {
        #[sqlx(flatten)]
        ticket: TicketSummary,
        total_count: i64,
    }
    let rows = qb
        .build_query_as::<Row>()
        .fetch_all(state.database.read())
        .await?;
    let total = rows.first().map_or(0, |r| r.total_count);
    Ok(Page {
        total,
        per_page,
        page,
        data: rows.into_iter().map(|r| r.ticket).collect(),
    })
}

pub async fn get_ticket(
    state: &State,
    uuid: Uuid,
    scope: TicketScope,
    include_internal: bool,
) -> anyhow::Result<Option<TicketDetail>> {
    if !matches!(scope, TicketScope::Admin) {
        ensure_enabled(state).await?;
    }
    let mut sql = format!(
        "SELECT {SUMMARY_COLUMNS} FROM team_voidvalue_tickets_tickets t {SUMMARY_JOINS} WHERE t.uuid=$1"
    );
    match scope {
        TicketScope::User(_) => sql.push_str(" AND t.user_uuid=$2"),
        TicketScope::Server { .. } => sql.push_str(" AND t.user_uuid=$2 AND t.server_uuid=$3"),
        TicketScope::Admin => {}
    }
    // The only dynamic fragments above are fixed, locally selected ownership clauses.
    let base = sqlx::query_as::<_, TicketSummary>(sqlx::AssertSqlSafe(sql)).bind(uuid);
    let ticket = match scope {
        TicketScope::User(id) => base.bind(id),
        TicketScope::Server {
            user_uuid,
            server_uuid,
        } => base.bind(user_uuid).bind(server_uuid),
        TicketScope::Admin => base,
    }
    .fetch_optional(state.database.read())
    .await?;
    let Some(ticket) = ticket else {
        return Ok(None);
    };
    let messages = sqlx::query_as::<_, TicketMessage>(
        r#"
      SELECT m.uuid,m.ticket_uuid,m.author_uuid,u.username AS author_name,
             m.message_type,m.body,m.created_at,m.edited_at
      FROM team_voidvalue_tickets_messages m
      LEFT JOIN users u ON u.uuid=m.author_uuid
      WHERE m.ticket_uuid=$1 AND ($2 OR m.message_type <> 'internal_note') ORDER BY m.created_at
    "#,
    )
    .bind(uuid)
    .bind(include_internal)
    .fetch_all(state.database.read())
    .await?;
    let attachments = sqlx::query_as::<_, TicketAttachment>(
        r#"
      SELECT a.uuid,a.message_uuid,a.uploader_uuid,a.original_filename,a.mime_type,
             a.size_bytes,a.sha256,a.created_at
      FROM team_voidvalue_tickets_attachments a
      JOIN team_voidvalue_tickets_messages m ON m.uuid=a.message_uuid
      WHERE m.ticket_uuid=$1 AND a.deleted_at IS NULL
        AND ($2 OR m.message_type <> 'internal_note')
      ORDER BY a.created_at
    "#,
    )
    .bind(uuid)
    .bind(include_internal)
    .fetch_all(state.database.read())
    .await?;
    let history = if include_internal {
        sqlx::query_as::<_, TicketHistory>(
            r#"
          SELECT h.uuid,h.actor_uuid,u.username AS actor_name,h.event,h.data,h.created_at
          FROM team_voidvalue_tickets_history h
          LEFT JOIN users u ON u.uuid=h.actor_uuid
          WHERE h.ticket_uuid=$1 ORDER BY h.created_at
        "#,
        )
        .bind(uuid)
        .fetch_all(state.database.read())
        .await?
    } else {
        Vec::new()
    };
    let access_requests = sqlx::query_as::<_, AccessRequest>(
        r#"
        SELECT request.uuid,request.requested_by_uuid,requester.username AS requested_by_name,
               request.permissions,request.requested_duration_minutes,request.reason,request.status,
               request.reviewed_by_uuid,reviewer.username AS reviewed_by_name,
               request.created_at,request.reviewed_at
        FROM team_voidvalue_tickets_access_requests request
        JOIN users requester ON requester.uuid=request.requested_by_uuid
        LEFT JOIN users reviewer ON reviewer.uuid=request.reviewed_by_uuid
        WHERE request.ticket_uuid=$1 ORDER BY request.created_at DESC
        "#,
    )
    .bind(uuid)
    .fetch_all(state.database.read())
    .await?;
    let access_grants = sqlx::query_as::<_, AccessGrant>(
        r#"
        SELECT grant_.uuid,grant_.request_uuid,grant_.server_uuid,grant_.user_uuid,
               users.username AS user_name,grant_.permissions,grant_.reason,grant_.granted_at,
               grant_.expires_at,grant_.revoked_at,grant_.revoke_reason
        FROM team_voidvalue_tickets_access_grants grant_
        JOIN users ON users.uuid=grant_.user_uuid
        WHERE grant_.ticket_uuid=$1 ORDER BY grant_.granted_at DESC
        "#,
    )
    .bind(uuid)
    .fetch_all(state.database.read())
    .await?;
    let tags = sqlx::query_as::<_, TicketTag>(
        r#"SELECT tag.uuid,tag.name,tag.color
           FROM team_voidvalue_tickets_tags tag
           JOIN team_voidvalue_tickets_ticket_tags ticket_tag ON ticket_tag.tag_uuid=tag.uuid
           WHERE ticket_tag.ticket_uuid=$1 ORDER BY tag.name"#,
    )
    .bind(uuid)
    .fetch_all(state.database.read())
    .await?;
    Ok(Some(TicketDetail {
        ticket,
        messages,
        attachments,
        history,
        access_requests,
        access_grants,
        tags,
    }))
}

pub async fn add_attachments(
    state: &State,
    ticket_uuid: Uuid,
    message_uuid: Uuid,
    uploader_uuid: Uuid,
    scope: TicketScope,
    files: Vec<NewAttachment>,
    max_files: u32,
) -> anyhow::Result<TicketDetail> {
    let mut tx = state.database.write().begin().await?;
    let (owner, ticket_server): (Uuid, Option<Uuid>) = sqlx::query_as(
        "SELECT user_uuid,server_uuid FROM team_voidvalue_tickets_tickets WHERE uuid=$1 FOR SHARE",
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    if matches!(scope, TicketScope::User(id) if id != owner)
        || matches!(scope, TicketScope::Server { user_uuid, server_uuid } if user_uuid != owner || Some(server_uuid) != ticket_server)
    {
        anyhow::bail!("ticket not found");
    }
    let valid_message: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_voidvalue_tickets_messages WHERE uuid=$1 AND ticket_uuid=$2 AND author_uuid=$3)",
    )
    .bind(message_uuid)
    .bind(ticket_uuid)
    .bind(uploader_uuid)
    .fetch_one(&mut *tx)
    .await?;
    if !valid_message {
        anyhow::bail!("message not found");
    }
    let existing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM team_voidvalue_tickets_attachments WHERE message_uuid=$1 AND deleted_at IS NULL",
    )
    .bind(message_uuid)
    .fetch_one(&mut *tx)
    .await?;
    if existing + i64::try_from(files.len())? > i64::from(max_files) {
        anyhow::bail!("too many attachments");
    }
    let mut stored_paths = Vec::with_capacity(files.len());
    for file in files {
        let uuid = Uuid::new_v4();
        let storage_path = format!(
            "privatedata/extensions/team.voidvalue.tickets/{ticket_uuid}/{message_uuid}/{uuid}"
        );
        if let Err(error) = state
            .storage
            .store(
                &storage_path,
                std::io::Cursor::new(file.content.clone()),
                &file.mime_type,
            )
            .await
        {
            for path in &stored_paths {
                state.storage.remove(Some(path)).await.ok();
            }
            return Err(error);
        }
        stored_paths.push(storage_path.clone());
        let result = sqlx::query(
            r#"INSERT INTO team_voidvalue_tickets_attachments
              (uuid,message_uuid,uploader_uuid,storage_path,original_filename,mime_type,size_bytes,sha256,content)
              VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
        )
        .bind(uuid)
        .bind(message_uuid)
        .bind(uploader_uuid)
        .bind(storage_path)
        .bind(file.original_filename)
        .bind(file.mime_type)
        .bind(i64::try_from(file.content.len())?)
        .bind(file.sha256)
        .bind(Vec::<u8>::new())
        .execute(&mut *tx)
        .await;
        if let Err(error) = result {
            tx.rollback().await.ok();
            for path in &stored_paths {
                state.storage.remove(Some(path)).await.ok();
            }
            return Err(error.into());
        }
    }
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(uploader_uuid),
        "ticket.attachments.added",
        serde_json::json!({"message_uuid":message_uuid}),
    )
    .await?;
    tx.commit().await?;
    get_ticket(
        state,
        ticket_uuid,
        scope,
        matches!(scope, TicketScope::Admin),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

pub async fn download_attachment(
    state: &State,
    ticket_uuid: Uuid,
    attachment_uuid: Uuid,
    scope: TicketScope,
) -> anyhow::Result<Option<AttachmentDownload>> {
    let owner: Option<(Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT user_uuid,server_uuid FROM team_voidvalue_tickets_tickets WHERE uuid=$1",
    )
    .bind(ticket_uuid)
    .fetch_optional(state.database.read())
    .await?;
    let Some((owner, ticket_server)) = owner else {
        return Ok(None);
    };
    if matches!(scope, TicketScope::User(id) if id != owner)
        || matches!(scope, TicketScope::Server { user_uuid, server_uuid } if user_uuid != owner || Some(server_uuid) != ticket_server)
    {
        return Ok(None);
    }
    #[derive(sqlx::FromRow)]
    struct Row {
        storage_path: String,
        original_filename: String,
        mime_type: String,
        content: Vec<u8>,
        message_type: MessageType,
    }
    let row = sqlx::query_as::<_, Row>(
        r#"SELECT a.storage_path,a.original_filename,a.mime_type,a.content,m.message_type
           FROM team_voidvalue_tickets_attachments a
           JOIN team_voidvalue_tickets_messages m ON m.uuid=a.message_uuid
           WHERE a.uuid=$1 AND m.ticket_uuid=$2 AND a.deleted_at IS NULL"#,
    )
    .bind(attachment_uuid)
    .bind(ticket_uuid)
    .fetch_optional(state.database.read())
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    if !matches!(scope, TicketScope::Admin) && row.message_type == MessageType::InternalNote {
        return Ok(None);
    }
    let content = if row.storage_path.starts_with("database:") {
        row.content
    } else {
        crate::routes::attachments::read_private_file(state, &row.storage_path).await?
    };
    Ok(Some(AttachmentDownload {
        original_filename: row.original_filename,
        mime_type: row.mime_type,
        content,
    }))
}

pub async fn delete_ticket(state: &State, ticket_uuid: Uuid) -> anyhow::Result<Option<String>> {
    let paths: Vec<String> = sqlx::query_scalar(
        r#"SELECT a.storage_path FROM team_voidvalue_tickets_attachments a
           JOIN team_voidvalue_tickets_messages m ON m.uuid=a.message_uuid
           WHERE m.ticket_uuid=$1 AND a.storage_path NOT LIKE 'database:%'"#,
    )
    .bind(ticket_uuid)
    .fetch_all(state.database.read())
    .await?;
    let deleted = sqlx::query_scalar(
        "DELETE FROM team_voidvalue_tickets_tickets WHERE uuid=$1 RETURNING code",
    )
    .bind(ticket_uuid)
    .fetch_optional(state.database.write())
    .await?;
    if deleted.is_some() {
        for path in paths {
            if let Err(error) = state.storage.remove(Some(&path)).await {
                tracing::warn!(%path, %error, "failed to remove deleted ticket attachment");
            }
        }
    }
    Ok(deleted)
}

pub async fn reply(
    state: &State,
    ticket_uuid: Uuid,
    actor_uuid: Uuid,
    scope: TicketScope,
    message: String,
    kind: MessageType,
) -> anyhow::Result<TicketDetail> {
    let mut tx = state.database.write().begin().await?;
    let row: Option<(TicketStatus, Uuid, Option<Uuid>)> = sqlx::query_as(
        r#"
      SELECT status,user_uuid,server_uuid FROM team_voidvalue_tickets_tickets WHERE uuid=$1 FOR UPDATE
    "#,
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?;
    let (status, owner, ticket_server) = row.ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    match scope {
        TicketScope::User(id) if id != owner => anyhow::bail!("ticket not found"),
        TicketScope::Server {
            user_uuid,
            server_uuid,
        } if user_uuid != owner || Some(server_uuid) != ticket_server => {
            anyhow::bail!("ticket not found")
        }
        _ => {}
    }
    if status == TicketStatus::Closed {
        anyhow::bail!("ticket closed");
    }
    let next_status = match kind {
        MessageType::Customer => TicketStatus::AwaitingStaff,
        MessageType::Staff => TicketStatus::AwaitingCustomer,
        _ => status,
    };
    sqlx::query(
        r#"INSERT INTO team_voidvalue_tickets_messages
      (uuid,ticket_uuid,author_uuid,message_type,body) VALUES ($1,$2,$3,$4,$5)"#,
    )
    .bind(Uuid::new_v4())
    .bind(ticket_uuid)
    .bind(actor_uuid)
    .bind(kind)
    .bind(message)
    .execute(&mut *tx)
    .await?;
    sqlx::query(r#"UPDATE team_voidvalue_tickets_tickets SET
      status=$2,updated_at=now(),last_reply_at=now(),
      last_customer_reply_at=CASE WHEN $3='customer'::team_voidvalue_tickets_message_type THEN now() ELSE last_customer_reply_at END,
      last_staff_reply_at=CASE WHEN $3='staff'::team_voidvalue_tickets_message_type THEN now() ELSE last_staff_reply_at END,
      first_staff_reply_at=CASE WHEN $3='staff'::team_voidvalue_tickets_message_type THEN COALESCE(first_staff_reply_at,now()) ELSE first_staff_reply_at END,
      auto_close_warning_at=NULL WHERE uuid=$1"#)
      .bind(ticket_uuid).bind(next_status).bind(kind).execute(&mut *tx).await?;
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(actor_uuid),
        "ticket.replied",
        serde_json::json!({"message_type": kind}),
    )
    .await?;
    tx.commit().await?;
    get_ticket(
        state,
        ticket_uuid,
        scope,
        matches!(scope, TicketScope::Admin),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

pub async fn update_status(
    state: &State,
    ticket_uuid: Uuid,
    actor_uuid: Uuid,
    scope: TicketScope,
    next: TicketStatus,
    reopen_allowed: bool,
) -> anyhow::Result<TicketDetail> {
    let mut tx = state.database.write().begin().await?;
    let row: Option<StatusRow> = sqlx::query_as(
        "SELECT status,user_uuid,closed_at,resolved_at,server_uuid FROM team_voidvalue_tickets_tickets WHERE uuid=$1 FOR UPDATE",
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?;
    let row = row.ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    let current = row.status;
    let owner = row.user_uuid;
    let closed_at = row.closed_at;
    let resolved_at = row.resolved_at;
    let ticket_server = row.server_uuid;
    if matches!(scope, TicketScope::User(id) if id != owner)
        || matches!(scope, TicketScope::Server { user_uuid, server_uuid } if user_uuid != owner || Some(server_uuid) != ticket_server)
    {
        anyhow::bail!("ticket not found");
    }
    if !current.can_transition_to(next, reopen_allowed) {
        anyhow::bail!("invalid status transition");
    }
    if !matches!(scope, TicketScope::Admin)
        && matches!(current, TicketStatus::Resolved | TicketStatus::Closed)
        && next == TicketStatus::Open
    {
        let settings = state.settings.get().await?;
        let extension: &crate::settings::ExtensionSettingsData =
            settings.find_extension_settings()?;
        let period = extension.reopen_period_days;
        drop(settings);
        let terminal_at = closed_at.or(resolved_at);
        if period == 0
            || terminal_at.is_none_or(|at| at < Utc::now() - Duration::days(i64::from(period)))
        {
            anyhow::bail!("ticket reopening period has expired");
        }
    }
    sqlx::query(
        r#"UPDATE team_voidvalue_tickets_tickets SET status=$2,updated_at=now(),
      resolved_at=CASE WHEN $2='resolved' THEN now() ELSE resolved_at END,
      closed_at=CASE WHEN $2='closed' THEN now() ELSE NULL END WHERE uuid=$1"#,
    )
    .bind(ticket_uuid)
    .bind(next)
    .execute(&mut *tx)
    .await?;
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(actor_uuid),
        "ticket.status.updated",
        serde_json::json!({"before":current,"after":next}),
    )
    .await?;
    tx.commit().await?;
    if matches!(next, TicketStatus::Resolved | TicketStatus::Closed) {
        let revoke = state
            .settings
            .get()
            .await
            .ok()
            .and_then(|settings| {
                settings
                    .find_extension_settings::<crate::settings::ExtensionSettingsData>()
                    .ok()
                    .map(|extension| extension.revoke_access_on_resolved)
            })
            .unwrap_or(true);
        if revoke {
            access::revoke_ticket_grants(state, ticket_uuid, Some(actor_uuid), "ticket resolved")
                .await?;
        }
    }
    get_ticket(
        state,
        ticket_uuid,
        scope,
        matches!(scope, TicketScope::Admin),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

pub async fn assign(
    state: &State,
    ticket_uuid: Uuid,
    actor_uuid: Uuid,
    staff_uuid: Option<Uuid>,
) -> anyhow::Result<TicketDetail> {
    let mut tx = state.database.write().begin().await?;
    sqlx::query("SELECT uuid FROM team_voidvalue_tickets_tickets WHERE uuid=$1 FOR UPDATE")
        .bind(ticket_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    sqlx::query("UPDATE team_voidvalue_tickets_assignments SET ended_at=now() WHERE ticket_uuid=$1 AND ended_at IS NULL").bind(ticket_uuid).execute(&mut *tx).await?;
    if let Some(staff) = staff_uuid {
        sqlx::query("INSERT INTO team_voidvalue_tickets_assignments(uuid,ticket_uuid,staff_uuid,assigned_by_uuid) VALUES($1,$2,$3,$4)").bind(Uuid::new_v4()).bind(ticket_uuid).bind(staff).bind(actor_uuid).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE team_voidvalue_tickets_tickets SET assigned_staff_uuid=$2,updated_at=now() WHERE uuid=$1").bind(ticket_uuid).bind(staff_uuid).execute(&mut *tx).await?;
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(actor_uuid),
        "ticket.assignment.updated",
        serde_json::json!({"staff_uuid":staff_uuid}),
    )
    .await?;
    tx.commit().await?;
    get_ticket(state, ticket_uuid, TicketScope::Admin, true)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

pub async fn update_tags(
    state: &State,
    ticket_uuid: Uuid,
    actor_uuid: Uuid,
    tags: Vec<String>,
) -> anyhow::Result<TicketDetail> {
    let mut normalized = tags
        .into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    let mut tx = state.database.write().begin().await?;
    sqlx::query("SELECT uuid FROM team_voidvalue_tickets_tickets WHERE uuid=$1 FOR UPDATE")
        .bind(ticket_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    sqlx::query("DELETE FROM team_voidvalue_tickets_ticket_tags WHERE ticket_uuid=$1")
        .bind(ticket_uuid)
        .execute(&mut *tx)
        .await?;
    for name in &normalized {
        let tag_uuid: Uuid = sqlx::query_scalar(
            r#"INSERT INTO team_voidvalue_tickets_tags(uuid,name)
               VALUES($1,$2) ON CONFLICT(name) DO UPDATE SET name=EXCLUDED.name RETURNING uuid"#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO team_voidvalue_tickets_ticket_tags(ticket_uuid,tag_uuid) VALUES($1,$2)",
        )
        .bind(ticket_uuid)
        .bind(tag_uuid)
        .execute(&mut *tx)
        .await?;
    }
    add_history_tx(
        &mut tx,
        ticket_uuid,
        Some(actor_uuid),
        "ticket.tags.updated",
        serde_json::json!({"tags":normalized}),
    )
    .await?;
    tx.commit().await?;
    get_ticket(state, ticket_uuid, TicketScope::Admin, true)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))
}

pub async fn statistics(state: &State) -> anyhow::Result<Statistics> {
    Ok(sqlx::query_as::<_,Statistics>(r#"SELECT
      COUNT(*) FILTER (WHERE status NOT IN ('resolved','closed')) AS open,
      COUNT(*) FILTER (WHERE assigned_staff_uuid IS NULL AND status NOT IN ('resolved','closed')) AS unassigned,
      COUNT(*) FILTER (WHERE status='awaiting_staff') AS awaiting_staff,
      COUNT(*) FILTER (WHERE status='awaiting_customer') AS awaiting_customer,
      COUNT(*) FILTER (WHERE priority='urgent' AND status NOT IN ('resolved','closed')) AS urgent,
      COUNT(*) FILTER (WHERE first_response_sla_breached OR resolution_sla_breached) AS sla_breached,
      COUNT(*) FILTER (WHERE resolved_at::date=CURRENT_DATE) AS resolved_today,
      COUNT(*) FILTER (WHERE created_at::date=CURRENT_DATE) AS created_today
      FROM team_voidvalue_tickets_tickets"#).fetch_one(state.database.read()).await?)
}

pub async fn list_staff(state: &State) -> anyhow::Result<Vec<StaffOption>> {
    Ok(sqlx::query_as::<_, StaffOption>(
        r#"
        SELECT DISTINCT users.uuid, users.username
        FROM users
        LEFT JOIN roles ON roles.uuid=users.role_uuid
        WHERE NOT users.suspended
          AND (users.admin OR 'support.read'=ANY(COALESCE(roles.admin_permissions,'{}')))
        ORDER BY users.username
        "#,
    )
    .fetch_all(state.database.read())
    .await?)
}

async fn add_history_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    ticket_uuid: Uuid,
    actor_uuid: Option<Uuid>,
    event: &str,
    data: serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO team_voidvalue_tickets_history(uuid,ticket_uuid,actor_uuid,event,data) VALUES($1,$2,$3,$4,$5)")
      .bind(Uuid::new_v4()).bind(ticket_uuid).bind(actor_uuid).bind(event).bind(data).execute(&mut **tx).await?;
    Ok(())
}

async fn ensure_enabled(state: &State) -> anyhow::Result<()> {
    let settings = state.settings.get().await?;
    let extension: &crate::settings::ExtensionSettingsData = settings.find_extension_settings()?;
    if !extension.enabled {
        anyhow::bail!("support tickets are disabled");
    }
    Ok(())
}
