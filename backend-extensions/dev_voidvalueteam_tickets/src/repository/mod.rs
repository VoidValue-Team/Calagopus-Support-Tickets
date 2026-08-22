use crate::models::*;
use chrono::{Duration, Utc};
use shared::State;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

const SUMMARY_COLUMNS: &str = r#"
 t.uuid, t.number, t.code, t.user_uuid, ticket_user.username AS user_name,
 t.server_uuid, ticket_server.name AS server_name, t.department_uuid,
 d.name AS department_name, t.subject, t.status, t.priority, t.assigned_staff_uuid,
 assigned_staff.username AS assigned_staff_name,
 t.created_at, t.updated_at, t.last_reply_at, t.first_response_due_at,
 t.resolution_due_at, t.first_response_sla_breached, t.resolution_sla_breached
"#;

const SUMMARY_JOINS: &str = r#"
 JOIN dev_voidvalueteam_tickets_departments d ON d.uuid=t.department_uuid
 JOIN users ticket_user ON ticket_user.uuid=t.user_uuid
 LEFT JOIN servers ticket_server ON ticket_server.uuid=t.server_uuid
 LEFT JOIN users assigned_staff ON assigned_staff.uuid=t.assigned_staff_uuid
"#;

#[derive(Clone, Copy)]
pub enum TicketScope {
    User(Uuid),
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

pub async fn list_departments(
    state: &State,
    include_disabled: bool,
) -> anyhow::Result<Vec<Department>> {
    Ok(sqlx::query_as::<_, Department>(
        r#"
        SELECT uuid, name, description, enabled, position, default_priority,
               first_response_sla_minutes, resolution_sla_minutes, autoresponse,
               allow_server_access, notification_enabled
        FROM dev_voidvalueteam_tickets_departments
        WHERE enabled OR $1 ORDER BY position, name
    "#,
    )
    .bind(include_disabled)
    .fetch_all(state.database.read())
    .await?)
}

pub async fn list_agents(state: &State) -> anyhow::Result<Vec<SupportAgent>> {
    Ok(sqlx::query_as::<_, SupportAgent>(
        r#"
        SELECT u.uuid,u.username
        FROM users u
        LEFT JOIN roles r ON r.uuid=u.role_uuid
        WHERE NOT u.suspended AND NOT u.frozen
          AND (u.admin OR (r.admin_permissions IS NOT NULL AND 'support.read'=ANY(r.admin_permissions)))
        ORDER BY u.username
        LIMIT 100
        "#,
    )
    .fetch_all(state.database.read())
    .await?)
}

pub async fn create_ticket(
    state: &State,
    user_uuid: Uuid,
    payload: CreateTicketPayload,
    can_access_all_servers: bool,
) -> anyhow::Result<TicketDetail> {
    let mut tx = state.database.write().begin().await?;
    let department = sqlx::query_as::<_, Department>(
        r#"
        SELECT uuid, name, description, enabled, position, default_priority,
               first_response_sla_minutes, resolution_sla_minutes, autoresponse,
               allow_server_access, notification_enabled
        FROM dev_voidvalueteam_tickets_departments WHERE uuid = $1 AND enabled
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

    let number: i64 = sqlx::query_scalar("SELECT nextval('dev_voidvalueteam_tickets_number_seq')")
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
      INSERT INTO dev_voidvalueteam_tickets_tickets
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
        r#"INSERT INTO dev_voidvalueteam_tickets_messages
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
            r#"INSERT INTO dev_voidvalueteam_tickets_messages
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
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).clamp(1, 100);
    let mut qb = QueryBuilder::<Postgres>::new(format!(
        "SELECT {SUMMARY_COLUMNS}, COUNT(*) OVER() AS total_count FROM dev_voidvalueteam_tickets_tickets t {SUMMARY_JOINS} WHERE TRUE"
    ));
    match scope {
        TicketScope::User(id) => {
            qb.push(" AND t.user_uuid = ").push_bind(id);
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
    if query.sla_breached == Some(true) {
        qb.push(" AND (t.first_response_sla_breached OR t.resolution_sla_breached)");
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
    let mut sql = format!(
        "SELECT {SUMMARY_COLUMNS} FROM dev_voidvalueteam_tickets_tickets t {SUMMARY_JOINS} WHERE t.uuid=$1"
    );
    match scope {
        TicketScope::User(_) => sql.push_str(" AND t.user_uuid=$2"),
        TicketScope::Admin => {}
    }
    // The only dynamic fragments above are fixed, locally selected ownership clauses.
    let base = sqlx::query_as::<_, TicketSummary>(sqlx::AssertSqlSafe(sql)).bind(uuid);
    let ticket = match scope {
        TicketScope::User(id) => base.bind(id),
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
      FROM dev_voidvalueteam_tickets_messages m
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
      FROM dev_voidvalueteam_tickets_attachments a
      JOIN dev_voidvalueteam_tickets_messages m ON m.uuid=a.message_uuid
      WHERE m.ticket_uuid=$1 AND a.deleted_at IS NULL
        AND ($2 OR m.message_type <> 'internal_note')
      ORDER BY a.created_at
    "#,
    )
    .bind(uuid)
    .bind(include_internal)
    .fetch_all(state.database.read())
    .await?;
    Ok(Some(TicketDetail {
        ticket,
        messages,
        attachments,
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
    let owner: Uuid = sqlx::query_scalar(
        "SELECT user_uuid FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 FOR SHARE",
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    if matches!(scope, TicketScope::User(id) if id != owner) {
        anyhow::bail!("ticket not found");
    }
    let valid_message: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM dev_voidvalueteam_tickets_messages WHERE uuid=$1 AND ticket_uuid=$2 AND author_uuid=$3)",
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
        "SELECT COUNT(*) FROM dev_voidvalueteam_tickets_attachments WHERE message_uuid=$1 AND deleted_at IS NULL",
    )
    .bind(message_uuid)
    .fetch_one(&mut *tx)
    .await?;
    if existing + i64::try_from(files.len())? > i64::from(max_files) {
        anyhow::bail!("too many attachments");
    }
    for file in files {
        let uuid = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO dev_voidvalueteam_tickets_attachments
              (uuid,message_uuid,uploader_uuid,storage_path,original_filename,mime_type,size_bytes,sha256,content)
              VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
        )
        .bind(uuid)
        .bind(message_uuid)
        .bind(uploader_uuid)
        .bind(format!("database:{uuid}"))
        .bind(file.original_filename)
        .bind(file.mime_type)
        .bind(i64::try_from(file.content.len())?)
        .bind(file.sha256)
        .bind(file.content)
        .execute(&mut *tx)
        .await?;
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
    let owner: Option<Uuid> =
        sqlx::query_scalar("SELECT user_uuid FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1")
            .bind(ticket_uuid)
            .fetch_optional(state.database.read())
            .await?;
    let Some(owner) = owner else {
        return Ok(None);
    };
    if matches!(scope, TicketScope::User(id) if id != owner) {
        return Ok(None);
    }
    #[derive(sqlx::FromRow)]
    struct Row {
        original_filename: String,
        mime_type: String,
        content: Vec<u8>,
        message_type: MessageType,
    }
    let row = sqlx::query_as::<_, Row>(
        r#"SELECT a.original_filename,a.mime_type,a.content,m.message_type
           FROM dev_voidvalueteam_tickets_attachments a
           JOIN dev_voidvalueteam_tickets_messages m ON m.uuid=a.message_uuid
           WHERE a.uuid=$1 AND m.ticket_uuid=$2 AND a.deleted_at IS NULL"#,
    )
    .bind(attachment_uuid)
    .bind(ticket_uuid)
    .fetch_optional(state.database.read())
    .await?;
    Ok(row.and_then(|row| {
        if matches!(scope, TicketScope::User(_)) && row.message_type == MessageType::InternalNote {
            None
        } else {
            Some(AttachmentDownload {
                original_filename: row.original_filename,
                mime_type: row.mime_type,
                content: row.content,
            })
        }
    }))
}

pub async fn delete_ticket(state: &State, ticket_uuid: Uuid) -> anyhow::Result<Option<String>> {
    Ok(sqlx::query_scalar(
        "DELETE FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 RETURNING code",
    )
    .bind(ticket_uuid)
    .fetch_optional(state.database.write())
    .await?)
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
    let row: Option<(TicketStatus, Uuid)> = sqlx::query_as(
        r#"
      SELECT status,user_uuid FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 FOR UPDATE
    "#,
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?;
    let (status, owner) = row.ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    match scope {
        TicketScope::User(id) if id != owner => anyhow::bail!("ticket not found"),
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
        r#"INSERT INTO dev_voidvalueteam_tickets_messages
      (uuid,ticket_uuid,author_uuid,message_type,body) VALUES ($1,$2,$3,$4,$5)"#,
    )
    .bind(Uuid::new_v4())
    .bind(ticket_uuid)
    .bind(actor_uuid)
    .bind(kind)
    .bind(message)
    .execute(&mut *tx)
    .await?;
    sqlx::query(r#"UPDATE dev_voidvalueteam_tickets_tickets SET
      status=$2,updated_at=now(),last_reply_at=now(),
      last_customer_reply_at=CASE WHEN $3='customer'::dev_voidvalueteam_tickets_message_type THEN now() ELSE last_customer_reply_at END,
      last_staff_reply_at=CASE WHEN $3='staff'::dev_voidvalueteam_tickets_message_type THEN now() ELSE last_staff_reply_at END,
      first_staff_reply_at=CASE WHEN $3='staff'::dev_voidvalueteam_tickets_message_type THEN COALESCE(first_staff_reply_at,now()) ELSE first_staff_reply_at END,
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
    let current: TicketStatus = sqlx::query_scalar(
        "SELECT status FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 FOR UPDATE",
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    if !current.can_transition_to(next, reopen_allowed) {
        anyhow::bail!("invalid status transition");
    }
    sqlx::query(
        r#"UPDATE dev_voidvalueteam_tickets_tickets SET status=$2,updated_at=now(),
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
    sqlx::query("SELECT uuid FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 FOR UPDATE")
        .bind(ticket_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    if let Some(staff) = staff_uuid {
        let eligible: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
              SELECT 1 FROM users u LEFT JOIN roles r ON r.uuid=u.role_uuid
              WHERE u.uuid=$1 AND NOT u.suspended AND NOT u.frozen
                AND (u.admin OR (r.admin_permissions IS NOT NULL AND 'support.read'=ANY(r.admin_permissions)))
            )"#,
        )
        .bind(staff)
        .fetch_one(&mut *tx)
        .await?;
        if !eligible {
            anyhow::bail!("staff member is not eligible");
        }
    }
    sqlx::query("UPDATE dev_voidvalueteam_tickets_assignments SET ended_at=now() WHERE ticket_uuid=$1 AND ended_at IS NULL").bind(ticket_uuid).execute(&mut *tx).await?;
    if let Some(staff) = staff_uuid {
        sqlx::query("INSERT INTO dev_voidvalueteam_tickets_assignments(uuid,ticket_uuid,staff_uuid,assigned_by_uuid) VALUES($1,$2,$3,$4)").bind(Uuid::new_v4()).bind(ticket_uuid).bind(staff).bind(actor_uuid).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE dev_voidvalueteam_tickets_tickets SET assigned_staff_uuid=$2,updated_at=now() WHERE uuid=$1").bind(ticket_uuid).bind(staff_uuid).execute(&mut *tx).await?;
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
      FROM dev_voidvalueteam_tickets_tickets"#).fetch_one(state.database.read()).await?)
}

async fn add_history_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    ticket_uuid: Uuid,
    actor_uuid: Option<Uuid>,
    event: &str,
    data: serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO dev_voidvalueteam_tickets_history(uuid,ticket_uuid,actor_uuid,event,data) VALUES($1,$2,$3,$4,$5)")
      .bind(Uuid::new_v4()).bind(ticket_uuid).bind(actor_uuid).bind(event).bind(data).execute(&mut **tx).await?;
    Ok(())
}
