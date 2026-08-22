ALTER TABLE dev_voidvalueteam_tickets_attachments
  ADD COLUMN content bytea NOT NULL DEFAULT ''::bytea;

ALTER TABLE dev_voidvalueteam_tickets_attachments
  ALTER COLUMN content DROP DEFAULT;
