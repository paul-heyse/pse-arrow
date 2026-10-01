CREATE TYPE pse_ops.member_selection_kind AS ENUM ('full', 'revision');
CREATE TYPE pse_ops.publication_kind AS ENUM ('relations', 'source', 'model', 'case', 'problem', 'run', 'diagnostics', 'inspection');
CREATE TYPE pse_ops.publication_member_role AS ENUM ('output', 'input');
CREATE TYPE pse_ops.retention_phase AS ENUM ('expiring', 'deleted');
CREATE TYPE pse_ops.settlement_outcome AS ENUM ('committed', 'proved_noncommit', 'conflict');
CREATE DOMAIN pse_ops.attempt_id AS uuid;
CREATE DOMAIN pse_ops.publication_id AS uuid;
CREATE DOMAIN pse_ops.reader_lease_id AS uuid;
CREATE DOMAIN pse_ops.settlement_id AS uuid;
CREATE DOMAIN pse_ops.workspace_id AS uuid;

-- runtime.operational_publication_heads
CREATE TABLE pse_ops."publication_heads" (
    "workspace_id" pse_ops.workspace_id NOT NULL,
    "publication_id" pse_ops.publication_id,
    "advanced_at" timestamptz NOT NULL,
    CONSTRAINT publication_heads_pkey PRIMARY KEY ("workspace_id")
);

-- runtime.operational_publication_intents
CREATE TABLE pse_ops."publication_intents" (
    "publication_id" pse_ops.publication_id NOT NULL,
    "workspace_id" pse_ops.workspace_id NOT NULL,
    "attempt_id" pse_ops.attempt_id NOT NULL,
    "member_prefix" text NOT NULL,
    "prepared_at" timestamptz NOT NULL,
    "abandoned_at" timestamptz,
    "reclaimed_at" timestamptz,
    CONSTRAINT publication_intents_pkey PRIMARY KEY ("publication_id"),
    CONSTRAINT publication_intents_member_prefix_key UNIQUE ("member_prefix"),
    CONSTRAINT publication_intents_identity_key UNIQUE ("publication_id", "workspace_id", "attempt_id"),
    CONSTRAINT publication_intents_member_prefix_nonempty_check CHECK ("member_prefix" <> ''),
    CONSTRAINT publication_intents_reclaimed_after_abandoned_check CHECK ("reclaimed_at" IS NULL OR "abandoned_at" IS NOT NULL)
);

-- runtime.operational_publication_members
CREATE TABLE pse_ops."publication_members" (
    "publication_id" pse_ops.publication_id NOT NULL,
    "role" pse_ops.publication_member_role NOT NULL,
    "catalog_name" text NOT NULL,
    "schema_name" text NOT NULL,
    "table_name" text NOT NULL,
    "relation_id" uuid NOT NULL,
    "relation_version" bigint NOT NULL,
    "contract_fingerprint" pse_ops.content_hash NOT NULL,
    "table_uri" text NOT NULL,
    "delta_version" bigint NOT NULL,
    "selection_kind" pse_ops.member_selection_kind NOT NULL,
    "revision_column" text,
    "revision_id" uuid,
    CONSTRAINT publication_members_pkey PRIMARY KEY ("publication_id", "role", "catalog_name", "schema_name", "table_name"),
    CONSTRAINT publication_members_catalog_name_nonempty_check CHECK ("catalog_name" <> ''),
    CONSTRAINT publication_members_delta_version_nonnegative_check CHECK ("delta_version" >= 0),
    CONSTRAINT publication_members_one_selection_check CHECK (("selection_kind" = 'full' AND "revision_column" IS NULL AND "revision_id" IS NULL) OR ("selection_kind" = 'revision' AND "revision_column" IS NOT NULL AND "revision_column" <> '' AND "revision_id" IS NOT NULL)),
    CONSTRAINT publication_members_relation_version_nonnegative_check CHECK ("relation_version" >= 0),
    CONSTRAINT publication_members_schema_name_nonempty_check CHECK ("schema_name" <> ''),
    CONSTRAINT publication_members_table_name_nonempty_check CHECK ("table_name" <> ''),
    CONSTRAINT publication_members_table_uri_nonempty_check CHECK ("table_uri" <> '')
);

-- runtime.operational_publication_windows
CREATE TABLE pse_ops."publication_windows" (
    "publication_id" pse_ops.publication_id NOT NULL,
    "table_uri" text NOT NULL,
    "from_version" bigint NOT NULL,
    "through_version" bigint NOT NULL,
    CONSTRAINT publication_windows_pkey PRIMARY KEY ("publication_id", "table_uri", "from_version"),
    CONSTRAINT publication_windows_from_version_nonnegative_check CHECK ("from_version" >= 0),
    CONSTRAINT publication_windows_ordered_window_check CHECK ("from_version" <= "through_version"),
    CONSTRAINT publication_windows_table_uri_nonempty_check CHECK ("table_uri" <> '')
);

-- runtime.operational_publications
CREATE TABLE pse_ops."publications" (
    "publication_id" pse_ops.publication_id NOT NULL,
    "workspace_id" pse_ops.workspace_id NOT NULL,
    "parent_publication" pse_ops.publication_id,
    "attempt_id" pse_ops.attempt_id NOT NULL,
    "kind" pse_ops.publication_kind NOT NULL,
    "committed_at" timestamptz NOT NULL,
    CONSTRAINT publications_pkey PRIMARY KEY ("publication_id"),
    CONSTRAINT publications_attempt_id_key UNIQUE ("attempt_id"),
    CONSTRAINT publications_parent_is_another_publication_check CHECK ("parent_publication" IS DISTINCT FROM "publication_id")
);

-- runtime.operational_reader_leases
CREATE TABLE pse_ops."reader_leases" (
    "lease_id" pse_ops.reader_lease_id NOT NULL,
    "publication_id" pse_ops.publication_id NOT NULL,
    "head_of" pse_ops.workspace_id,
    "holder" text NOT NULL,
    "acquired_at" timestamptz NOT NULL,
    "expires_at" timestamptz NOT NULL,
    "released_at" timestamptz,
    CONSTRAINT reader_leases_pkey PRIMARY KEY ("lease_id"),
    CONSTRAINT reader_leases_expires_after_acquired_check CHECK ("expires_at" > "acquired_at"),
    CONSTRAINT reader_leases_holder_nonempty_check CHECK ("holder" <> '')
);

-- runtime.operational_retention_marks
CREATE TABLE pse_ops."retention_marks" (
    "publication_id" pse_ops.publication_id NOT NULL,
    "phase" pse_ops.retention_phase NOT NULL,
    "marked_at" timestamptz NOT NULL,
    "deleted_at" timestamptz,
    CONSTRAINT retention_marks_pkey PRIMARY KEY ("publication_id"),
    CONSTRAINT retention_marks_deleted_when_marked_deleted_check CHECK (("phase" = 'deleted') = ("deleted_at" IS NOT NULL))
);

-- runtime.operational_schema_support_state
CREATE TABLE pse_ops."schema_support_state" (
    "history" text NOT NULL,
    "shared_version" integer NOT NULL,
    "source" text NOT NULL,
    "target" text NOT NULL,
    "ready" boolean NOT NULL,
    CONSTRAINT schema_support_state_pkey PRIMARY KEY ("history"),
    CONSTRAINT schema_support_state_history_known_check CHECK ("history" IN ('catalog', 'operations')),
    CONSTRAINT schema_support_state_shared_version_positive_check CHECK ("shared_version" > 0)
);

-- runtime.operational_settlements
CREATE TABLE pse_ops."settlements" (
    "settlement_id" pse_ops.settlement_id NOT NULL,
    "attempt_id" pse_ops.attempt_id NOT NULL,
    "outcome" pse_ops.settlement_outcome NOT NULL,
    "publication_id" pse_ops.publication_id,
    "reason" text,
    "conflict_head" pse_ops.publication_id,
    "settled_at" timestamptz NOT NULL,
    CONSTRAINT settlements_pkey PRIMARY KEY ("settlement_id"),
    CONSTRAINT settlements_committed_names_publication_check CHECK (("outcome" = 'committed') = ("publication_id" IS NOT NULL)),
    CONSTRAINT settlements_conflict_has_reason_check CHECK (("outcome" = 'conflict') = ("reason" IS NOT NULL)),
    CONSTRAINT settlements_conflict_head_of_conflict_check CHECK ("conflict_head" IS NULL OR "outcome" = 'conflict'),
    CONSTRAINT settlements_reason_nonempty_check CHECK ("reason" IS NULL OR "reason" <> '')
);

-- runtime.operational_workspaces
CREATE TABLE pse_ops."workspaces" (
    "workspace_id" pse_ops.workspace_id NOT NULL,
    "name" text NOT NULL,
    "root_uri" text NOT NULL,
    "maintenance_epoch" bigint NOT NULL,
    "created_at" timestamptz NOT NULL,
    CONSTRAINT workspaces_pkey PRIMARY KEY ("workspace_id"),
    CONSTRAINT workspaces_name_key UNIQUE ("name"),
    CONSTRAINT workspaces_root_uri_key UNIQUE ("root_uri"),
    CONSTRAINT workspaces_maintenance_epoch_nonnegative_check CHECK ("maintenance_epoch" >= 0),
    CONSTRAINT workspaces_name_nonempty_check CHECK ("name" <> ''),
    CONSTRAINT workspaces_root_uri_nonempty_check CHECK ("root_uri" <> '')
);

ALTER TABLE pse_ops."publication_heads" ADD CONSTRAINT publication_heads_workspace_id_fkey
    FOREIGN KEY ("workspace_id") REFERENCES pse_ops."workspaces" ("workspace_id");

ALTER TABLE pse_ops."publication_heads" ADD CONSTRAINT publication_heads_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."publication_intents" ADD CONSTRAINT publication_intents_workspace_id_fkey
    FOREIGN KEY ("workspace_id") REFERENCES pse_ops."workspaces" ("workspace_id");

ALTER TABLE pse_ops."publication_intents" ADD CONSTRAINT publication_intents_attempt_id_fkey
    FOREIGN KEY ("attempt_id") REFERENCES pse_ops."attempts" ("attempt_id");

ALTER TABLE pse_ops."publication_members" ADD CONSTRAINT publication_members_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."publication_windows" ADD CONSTRAINT publication_windows_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."publications" ADD CONSTRAINT publications_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publication_intents" ("publication_id");

ALTER TABLE pse_ops."publications" ADD CONSTRAINT publications_workspace_id_fkey
    FOREIGN KEY ("workspace_id") REFERENCES pse_ops."workspaces" ("workspace_id");

ALTER TABLE pse_ops."publications" ADD CONSTRAINT publications_parent_publication_fkey
    FOREIGN KEY ("parent_publication") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."publications" ADD CONSTRAINT publications_attempt_id_fkey
    FOREIGN KEY ("attempt_id") REFERENCES pse_ops."attempts" ("attempt_id");

ALTER TABLE pse_ops."publications" ADD CONSTRAINT publications_intent_fkey
    FOREIGN KEY ("publication_id", "workspace_id", "attempt_id") REFERENCES pse_ops."publication_intents" ("publication_id", "workspace_id", "attempt_id");

ALTER TABLE pse_ops."reader_leases" ADD CONSTRAINT reader_leases_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."reader_leases" ADD CONSTRAINT reader_leases_head_of_fkey
    FOREIGN KEY ("head_of") REFERENCES pse_ops."workspaces" ("workspace_id");

ALTER TABLE pse_ops."retention_marks" ADD CONSTRAINT retention_marks_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."settlements" ADD CONSTRAINT settlements_attempt_id_fkey
    FOREIGN KEY ("attempt_id") REFERENCES pse_ops."attempts" ("attempt_id");

ALTER TABLE pse_ops."settlements" ADD CONSTRAINT settlements_publication_id_fkey
    FOREIGN KEY ("publication_id") REFERENCES pse_ops."publications" ("publication_id");

ALTER TABLE pse_ops."settlements" ADD CONSTRAINT settlements_conflict_head_fkey
    FOREIGN KEY ("conflict_head") REFERENCES pse_ops."publications" ("publication_id");
