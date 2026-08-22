use shared::State;
use uuid::Uuid;

pub async fn sweep_sla(state: &State) -> anyhow::Result<()> {
    let mut tx = state.database.write().begin().await?;
    let breached:Vec<Uuid>=sqlx::query_scalar(r#"UPDATE team_voidvalue_tickets_tickets SET
      first_response_sla_breached=first_response_sla_breached OR (first_staff_reply_at IS NULL AND first_response_due_at<now()),
      resolution_sla_breached=resolution_sla_breached OR (status NOT IN ('resolved','closed') AND resolution_due_at<now())
      WHERE status NOT IN ('resolved','closed') AND ((first_staff_reply_at IS NULL AND first_response_due_at<now() AND NOT first_response_sla_breached) OR (resolution_due_at<now() AND NOT resolution_sla_breached)) RETURNING uuid"#)
      .fetch_all(&mut *tx).await?;
    for ticket in breached {
        sqlx::query("INSERT INTO team_voidvalue_tickets_history(uuid,ticket_uuid,event,data)VALUES($1,$2,'ticket.sla.breached','{}')").bind(Uuid::new_v4()).bind(ticket).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn sweep_auto_close(state: &State) -> anyhow::Result<()> {
    let settings = state.settings.get().await?;
    let ext: &crate::settings::ExtensionSettingsData = settings.find_extension_settings()?;
    if !ext.auto_close_enabled {
        return Ok(());
    }
    let inactive = i64::from(ext.inactivity_days);
    let delay = i64::from(ext.final_close_delay_days);
    drop(settings);
    sqlx::query(r#"UPDATE team_voidvalue_tickets_tickets SET auto_close_warning_at=now(),updated_at=now()
      WHERE status='awaiting_customer' AND auto_close_warning_at IS NULL AND last_staff_reply_at < now()-make_interval(days=>$1)"#).bind(inactive as i32).execute(state.database.write()).await?;
    let closed:Vec<Uuid>=sqlx::query_scalar(r#"UPDATE team_voidvalue_tickets_tickets SET status='closed',closed_at=now(),updated_at=now()
      WHERE status='awaiting_customer' AND auto_close_warning_at < now()-make_interval(days=>$1) RETURNING uuid"#).bind(delay as i32).fetch_all(state.database.write()).await?;
    for ticket in closed {
        sqlx::query("INSERT INTO team_voidvalue_tickets_messages(uuid,ticket_uuid,message_type,body)VALUES($1,$2,'system','Ticket closed automatically due to inactivity.')").bind(Uuid::new_v4()).bind(ticket).execute(state.database.write()).await?;
    }
    Ok(())
}

pub async fn mark_expired_access(state: &State) -> anyhow::Result<()> {
    let grants: Vec<Uuid> = sqlx::query_scalar(
        "SELECT uuid FROM team_voidvalue_tickets_access_grants WHERE revoked_at IS NULL AND expires_at<=now()",
    )
    .fetch_all(state.database.read())
    .await?;
    for grant in grants {
        if let Err(error) =
            crate::repository::access::revoke_grant(state, grant, None, "expired", true).await
        {
            tracing::error!(%grant, %error, "failed to expire temporary support access");
        }
    }
    Ok(())
}
