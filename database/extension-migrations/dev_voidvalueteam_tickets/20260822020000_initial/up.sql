CREATE SEQUENCE dev_voidvalueteam_tickets_number_seq START 10000;

CREATE TYPE dev_voidvalueteam_tickets_status AS ENUM (
  'open', 'awaiting_staff', 'awaiting_customer', 'in_progress', 'resolved', 'closed'
);
CREATE TYPE dev_voidvalueteam_tickets_priority AS ENUM ('low', 'normal', 'high', 'urgent');
CREATE TYPE dev_voidvalueteam_tickets_message_type AS ENUM ('customer', 'staff', 'internal_note', 'system');
CREATE TYPE dev_voidvalueteam_tickets_access_status AS ENUM ('pending', 'approved', 'rejected', 'revoked', 'expired');

CREATE TABLE dev_voidvalueteam_tickets_departments (
  uuid uuid PRIMARY KEY,
  name varchar(80) NOT NULL,
  description varchar(500) NOT NULL DEFAULT '',
  enabled boolean NOT NULL DEFAULT true,
  position integer NOT NULL DEFAULT 0,
  default_priority dev_voidvalueteam_tickets_priority NOT NULL DEFAULT 'normal',
  first_response_sla_minutes integer NOT NULL DEFAULT 720 CHECK (first_response_sla_minutes > 0),
  resolution_sla_minutes integer NOT NULL DEFAULT 2880 CHECK (resolution_sla_minutes > 0),
  autoresponse text,
  allow_server_access boolean NOT NULL DEFAULT false,
  notification_enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO dev_voidvalueteam_tickets_departments
  (uuid, name, description, position, allow_server_access)
VALUES
  ('00000000-0000-4000-8000-000000000001', 'Technical Support', 'Technical problems with a server.', 10, true),
  ('00000000-0000-4000-8000-000000000002', 'Billing', 'Billing and payment questions.', 20, false),
  ('00000000-0000-4000-8000-000000000003', 'Sales', 'Commercial questions.', 30, false),
  ('00000000-0000-4000-8000-000000000004', 'Abuse', 'Report abuse or policy violations.', 40, false),
  ('00000000-0000-4000-8000-000000000005', 'Other', 'Anything that does not fit another department.', 50, false);

CREATE TABLE dev_voidvalueteam_tickets_tickets (
  uuid uuid PRIMARY KEY,
  number bigint NOT NULL DEFAULT nextval('dev_voidvalueteam_tickets_number_seq') UNIQUE,
  code varchar(32) NOT NULL UNIQUE,
  user_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE RESTRICT,
  server_uuid uuid REFERENCES servers(uuid) ON DELETE SET NULL,
  former_server_uuid uuid,
  former_server_name varchar(255),
  department_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_departments(uuid) ON DELETE RESTRICT,
  subject varchar(180) NOT NULL,
  status dev_voidvalueteam_tickets_status NOT NULL DEFAULT 'open',
  priority dev_voidvalueteam_tickets_priority NOT NULL DEFAULT 'normal',
  assigned_staff_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  last_reply_at timestamptz NOT NULL DEFAULT now(),
  last_customer_reply_at timestamptz NOT NULL DEFAULT now(),
  last_staff_reply_at timestamptz,
  first_staff_reply_at timestamptz,
  resolved_at timestamptz,
  closed_at timestamptz,
  first_response_due_at timestamptz NOT NULL,
  resolution_due_at timestamptz NOT NULL,
  first_response_sla_breached boolean NOT NULL DEFAULT false,
  resolution_sla_breached boolean NOT NULL DEFAULT false,
  auto_close_warning_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX vv_tickets_user_updated_idx ON dev_voidvalueteam_tickets_tickets(user_uuid, updated_at DESC);
CREATE INDEX vv_tickets_server_updated_idx ON dev_voidvalueteam_tickets_tickets(server_uuid, updated_at DESC) WHERE server_uuid IS NOT NULL;
CREATE INDEX vv_tickets_queue_idx ON dev_voidvalueteam_tickets_tickets(status, priority, updated_at DESC);
CREATE INDEX vv_tickets_assignment_idx ON dev_voidvalueteam_tickets_tickets(assigned_staff_uuid, status) WHERE assigned_staff_uuid IS NOT NULL;
CREATE INDEX vv_tickets_department_idx ON dev_voidvalueteam_tickets_tickets(department_uuid, status);
CREATE INDEX vv_tickets_sla_idx ON dev_voidvalueteam_tickets_tickets(first_response_due_at, resolution_due_at) WHERE status NOT IN ('resolved', 'closed');
CREATE INDEX vv_tickets_subject_search_idx ON dev_voidvalueteam_tickets_tickets USING gin (to_tsvector('simple', subject));

CREATE TABLE dev_voidvalueteam_tickets_messages (
  uuid uuid PRIMARY KEY,
  ticket_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tickets(uuid) ON DELETE CASCADE,
  author_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  message_type dev_voidvalueteam_tickets_message_type NOT NULL,
  body text NOT NULL CHECK (char_length(body) BETWEEN 1 AND 20000),
  created_at timestamptz NOT NULL DEFAULT now(),
  edited_at timestamptz
);
CREATE INDEX vv_ticket_messages_idx ON dev_voidvalueteam_tickets_messages(ticket_uuid, created_at);

CREATE TABLE dev_voidvalueteam_tickets_history (
  uuid uuid PRIMARY KEY,
  ticket_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tickets(uuid) ON DELETE CASCADE,
  actor_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  event varchar(80) NOT NULL,
  data jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX vv_ticket_history_idx ON dev_voidvalueteam_tickets_history(ticket_uuid, created_at);

CREATE TABLE dev_voidvalueteam_tickets_attachments (
  uuid uuid PRIMARY KEY,
  message_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_messages(uuid) ON DELETE CASCADE,
  uploader_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  storage_path text NOT NULL UNIQUE,
  original_filename varchar(255) NOT NULL,
  mime_type varchar(127) NOT NULL,
  size_bytes bigint NOT NULL CHECK (size_bytes >= 0),
  sha256 char(64),
  created_at timestamptz NOT NULL DEFAULT now(),
  deleted_at timestamptz
);

CREATE TABLE dev_voidvalueteam_tickets_department_staff (
  department_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_departments(uuid) ON DELETE CASCADE,
  user_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE CASCADE,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (department_uuid, user_uuid)
);

CREATE TABLE dev_voidvalueteam_tickets_assignments (
  uuid uuid PRIMARY KEY,
  ticket_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tickets(uuid) ON DELETE CASCADE,
  staff_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  assigned_by_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  ended_at timestamptz
);
CREATE UNIQUE INDEX vv_active_assignment_idx ON dev_voidvalueteam_tickets_assignments(ticket_uuid) WHERE ended_at IS NULL;

CREATE TABLE dev_voidvalueteam_tickets_tags (
  uuid uuid PRIMARY KEY,
  name varchar(40) NOT NULL UNIQUE,
  color varchar(20) NOT NULL DEFAULT 'gray',
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE dev_voidvalueteam_tickets_ticket_tags (
  ticket_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tickets(uuid) ON DELETE CASCADE,
  tag_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tags(uuid) ON DELETE CASCADE,
  PRIMARY KEY (ticket_uuid, tag_uuid)
);

CREATE TABLE dev_voidvalueteam_tickets_saved_replies (
  uuid uuid PRIMARY KEY,
  title varchar(120) NOT NULL,
  body text NOT NULL,
  department_uuid uuid REFERENCES dev_voidvalueteam_tickets_departments(uuid) ON DELETE SET NULL,
  enabled boolean NOT NULL DEFAULT true,
  created_by_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX vv_saved_replies_search_idx ON dev_voidvalueteam_tickets_saved_replies USING gin (to_tsvector('simple', title || ' ' || body));

CREATE TABLE dev_voidvalueteam_tickets_access_requests (
  uuid uuid PRIMARY KEY,
  ticket_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tickets(uuid) ON DELETE CASCADE,
  requested_by_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE RESTRICT,
  permissions text[] NOT NULL,
  requested_duration_minutes integer NOT NULL CHECK (requested_duration_minutes BETWEEN 1 AND 43200),
  reason varchar(500) NOT NULL,
  status dev_voidvalueteam_tickets_access_status NOT NULL DEFAULT 'pending',
  reviewed_by_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  reviewed_at timestamptz
);
CREATE UNIQUE INDEX vv_pending_access_request_idx ON dev_voidvalueteam_tickets_access_requests(ticket_uuid) WHERE status = 'pending';

CREATE TABLE dev_voidvalueteam_tickets_access_grants (
  uuid uuid PRIMARY KEY,
  ticket_uuid uuid NOT NULL REFERENCES dev_voidvalueteam_tickets_tickets(uuid) ON DELETE CASCADE,
  request_uuid uuid REFERENCES dev_voidvalueteam_tickets_access_requests(uuid) ON DELETE SET NULL,
  server_uuid uuid NOT NULL,
  user_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE RESTRICT,
  granted_by_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE RESTRICT,
  permissions text[] NOT NULL,
  prior_subuser_permissions text[],
  created_subuser boolean NOT NULL DEFAULT false,
  reason varchar(500) NOT NULL,
  granted_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  revoked_by_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
  revoke_reason varchar(500)
);
CREATE UNIQUE INDEX vv_active_access_grant_idx ON dev_voidvalueteam_tickets_access_grants(ticket_uuid, user_uuid) WHERE revoked_at IS NULL;
CREATE INDEX vv_expiring_access_grants_idx ON dev_voidvalueteam_tickets_access_grants(expires_at) WHERE revoked_at IS NULL;

CREATE TABLE dev_voidvalueteam_tickets_automation_rules (
  uuid uuid PRIMARY KEY,
  name varchar(120) NOT NULL,
  enabled boolean NOT NULL DEFAULT true,
  position integer NOT NULL DEFAULT 0,
  trigger varchar(80) NOT NULL,
  conditions jsonb NOT NULL DEFAULT '{}'::jsonb,
  actions jsonb NOT NULL DEFAULT '[]'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
