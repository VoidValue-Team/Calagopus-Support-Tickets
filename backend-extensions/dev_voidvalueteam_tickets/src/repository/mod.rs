use crate::models::*;
use chrono::{Duration, Utc};
use shared::State;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

const SUMMARY_COLUMNS: &str = r#"
 t.uuid, t.number, t.code, t.user_uuid, t.server_uuid, t.department_uuid,
 d.name AS department_name, t.subject, t.status, t.priority, t.assigned_staff_uuid,
 t.created_at, t.updated_at, t.last_reply_at, t.first_response_due_at,
 t.resolution_due_at, t.first_response_sla_breached, t.resolution_sla_breached
"#;

#[derive(Clone, Copy)]
pub enum TicketScope {
    User(Uuid),
    Server(Uuid),
    Admin,
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

pub async fn create_ticket(
    state: &State,
    user_uuid: Uuid,
    payload: CreateTicketPayload,
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
        let owns_or_shares: bool = sqlx::query_scalar(
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
        .await?;
        if !owns_or_shares {
            anyhow::bail!("server is not accessible to this user");
        }
    }

    let number: i64 =
        sqlx::query_scalar("SELECT nextval('dev_voidvalueteam_tickets_number_seq')")
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
        "SELECT {SUMMARY_COLUMNS}, COUNT(*) OVER() AS total_count FROM dev_voidvalueteam_tickets_tickets t JOIN dev_voidvalueteam_tickets_departments d ON d.uuid=t.department_uuid WHERE TRUE"
    ));
    match scope {
        TicketScope::User(id) => {
            qb.push(" AND t.user_uuid = ").push_bind(id);
        }
        TicketScope::Server(id) => {
            qb.push(" AND t.server_uuid = ").push_bind(id);
        }
        TicketScope::Admin => {}
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
        "SELECT {SUMMARY_COLUMNS} FROM dev_voidvalueteam_tickets_tickets t JOIN dev_voidvalueteam_tickets_departments d ON d.uuid=t.department_uuid WHERE t.uuid=$1"
    );
    match scope {
        TicketScope::User(_) => sql.push_str(" AND t.user_uuid=$2"),
        TicketScope::Server(_) => sql.push_str(" AND t.server_uuid=$2"),
        TicketScope::Admin => {}
    }
    // The only dynamic fragments above are fixed, locally selected ownership clauses.
    let base = sqlx::query_as::<_, TicketSummary>(sqlx::AssertSqlSafe(sql)).bind(uuid);
    let ticket = match scope {
        TicketScope::User(id) | TicketScope::Server(id) => base.bind(id),
        TicketScope::Admin => base,
    }
    .fetch_optional(state.database.read())
    .await?;
    let Some(ticket) = ticket else {
        return Ok(None);
    };
    let messages = sqlx::query_as::<_, TicketMessage>(
        r#"
      SELECT uuid,ticket_uuid,author_uuid,message_type,body,created_at,edited_at
      FROM dev_voidvalueteam_tickets_messages
      WHERE ticket_uuid=$1 AND ($2 OR message_type <> 'internal_note') ORDER BY created_at
    "#,
    )
    .bind(uuid)
    .bind(include_internal)
    .fetch_all(state.database.read())
    .await?;
    Ok(Some(TicketDetail { ticket, messages }))
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
    let row: Option<(TicketStatus, Uuid, Option<Uuid>)> = sqlx::query_as(r#"
      SELECT status,user_uuid,server_uuid FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 FOR UPDATE
    "#).bind(ticket_uuid).fetch_optional(&mut *tx).await?;
    let (status, owner, server) = row.ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
    match scope {
        TicketScope::User(id) if id != owner => anyhow::bail!("ticket not found"),
        TicketScope::Server(id) if Some(id) != server => anyhow::bail!("ticket not found"),
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
    sqlx::query(
        "SELECT uuid FROM dev_voidvalueteam_tickets_tickets WHERE uuid=$1 FOR UPDATE",
    )
    .bind(ticket_uuid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("ticket not found"))?;
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
