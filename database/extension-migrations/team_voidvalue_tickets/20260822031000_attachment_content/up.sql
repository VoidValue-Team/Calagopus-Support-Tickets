ALTER TABLE team_voidvalue_tickets_attachments
  ADD COLUMN content bytea NOT NULL DEFAULT ''::bytea;

ALTER TABLE team_voidvalue_tickets_attachments
  ALTER COLUMN content DROP DEFAULT;
