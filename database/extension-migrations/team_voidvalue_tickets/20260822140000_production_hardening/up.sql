CREATE OR REPLACE FUNCTION team_voidvalue_tickets_preserve_deleted_server()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  UPDATE team_voidvalue_tickets_access_grants
  SET revoked_at = now(),
      revoke_reason = 'server_deleted'
  WHERE server_uuid = OLD.uuid AND revoked_at IS NULL;

  UPDATE team_voidvalue_tickets_access_requests request
  SET status = 'revoked'
  FROM team_voidvalue_tickets_tickets ticket
  WHERE request.ticket_uuid = ticket.uuid
    AND ticket.server_uuid = OLD.uuid
    AND request.status IN ('pending', 'approved');

  UPDATE team_voidvalue_tickets_tickets
  SET former_server_uuid = OLD.uuid,
      former_server_name = OLD.name,
      server_uuid = NULL,
      updated_at = now()
  WHERE server_uuid = OLD.uuid;
  RETURN OLD;
END;
$$;

CREATE TRIGGER team_voidvalue_tickets_server_delete
BEFORE DELETE ON servers
FOR EACH ROW
EXECUTE FUNCTION team_voidvalue_tickets_preserve_deleted_server();

CREATE INDEX tvt_tickets_created_idx
ON team_voidvalue_tickets_tickets(created_at DESC);

CREATE INDEX tvt_access_request_ticket_status_idx
ON team_voidvalue_tickets_access_requests(ticket_uuid, status, created_at DESC);
