DROP INDEX IF EXISTS tvt_access_request_ticket_status_idx;
DROP INDEX IF EXISTS tvt_tickets_created_idx;
DROP TRIGGER IF EXISTS team_voidvalue_tickets_server_delete ON servers;
DROP FUNCTION IF EXISTS team_voidvalue_tickets_preserve_deleted_server();
