-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- Access paths, defaults and grants of the operational store (ADR-0114 Outcome 22).
--
-- The generated schema.sql (crates/pse-operations/src/generated/) owns what the store
-- means: tables, keys, references and checks, all from the registry. This file owns how
-- the store is read and written: indexes, partial indexes, column defaults and the
-- append-only grant. It is SQL because SQL is the declaration language for access paths.
-- Store::open applies it after schema.sql in the same transaction; every enum literal
-- below is checked against its ENUM type then, and a misspelling fails the open.
-- The schema fingerprint covers this file: editing it requires `just codegen` and
-- an explicit supported migration after workers drain and store generations close.

-- --------------------------------------------------------------- attempts --

CREATE INDEX attempts_run_idx ON pse_ops.attempts (run_id);
CREATE INDEX attempts_state_idx ON pse_ops.attempts (state, created_at);
-- The stale sweep scans running attempts by lease expiry.
CREATE INDEX attempts_lease_idx ON pse_ops.attempts (lease_expires_at)
    WHERE state = 'running';
CREATE INDEX attempts_parent_idx ON pse_ops.attempts (parent_attempt)
    WHERE parent_attempt IS NOT NULL;
ALTER TABLE pse_ops.attempts
    ALTER COLUMN state_version SET DEFAULT 0,
    ALTER COLUMN cancel_requested SET DEFAULT false,
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN updated_at SET DEFAULT now();

-- History is append-only for the application role (ADR-0114 Outcome 12). The owner
-- keeps INSERT and SELECT; UPDATE, DELETE and TRUNCATE are refused with SQLSTATE 42501.
ALTER TABLE pse_ops.attempt_transitions
    ALTER COLUMN at SET DEFAULT clock_timestamp();
REVOKE UPDATE, DELETE, TRUNCATE ON pse_ops.attempt_transitions FROM CURRENT_USER;

-- ------------------------------------------------------------------- jobs --

-- The claim query: queued jobs by priority, then availability.
CREATE INDEX jobs_claim_idx ON pse_ops.jobs (priority DESC, available_at, job_id)
    WHERE state = 'queued';
ALTER TABLE pse_ops.jobs
    ALTER COLUMN priority SET DEFAULT 0,
    ALTER COLUMN tries SET DEFAULT 0,
    ALTER COLUMN available_at SET DEFAULT now(),
    ALTER COLUMN enqueued_at SET DEFAULT now(),
    ALTER COLUMN updated_at SET DEFAULT now();

-- ---------------------------------------------------------------- streams --

-- Retention removes the events of finished attempts by age.
CREATE INDEX progress_events_attempt_idx ON pse_ops.progress_events (attempt_id, step, seq);

-- ---------------------------------------------------------------- sources --

ALTER TABLE pse_ops.source_bundles
    ALTER COLUMN created_at SET DEFAULT now();

-- -------------------------------------------------------------- solutions --

-- The warm-start lookup: the newest compatible seed of a kind.
CREATE INDEX solutions_seed_idx
    ON pse_ops.solutions (compatibility_stamp, preparation_identity, kind, created_at DESC);
ALTER TABLE pse_ops.solutions
    ALTER COLUMN created_at SET DEFAULT now();

-- ---------------------------------------------------------------- studies --

ALTER TABLE pse_ops.studies
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN updated_at SET DEFAULT now();
CREATE INDEX study_points_pending_idx ON pse_ops.study_points (study_id, point_index)
    WHERE state = 'pending';
-- Releasing or cancelling the points that wait on a point.
CREATE INDEX study_points_predecessor_idx ON pse_ops.study_points (study_id, predecessor)
    WHERE predecessor IS NOT NULL;
ALTER TABLE pse_ops.study_points
    ALTER COLUMN updated_at SET DEFAULT now();

-- ---------------------------------------------------------------- catalog --

ALTER TABLE pse_ops.workspaces
    ALTER COLUMN created_at SET DEFAULT now();
ALTER TABLE pse_ops.publication_intents
    ALTER COLUMN prepared_at SET DEFAULT now();
-- Reclamation and attempt-prefix protection read a workspace's unpublished intents.
CREATE INDEX publication_intents_workspace_idx
    ON pse_ops.publication_intents (workspace_id, prepared_at)
    WHERE reclaimed_at IS NULL;
CREATE INDEX publication_intents_attempt_idx ON pse_ops.publication_intents (attempt_id);
CREATE INDEX publications_workspace_idx ON pse_ops.publications (workspace_id, committed_at);
ALTER TABLE pse_ops.publications
    ALTER COLUMN committed_at SET DEFAULT now();
ALTER TABLE pse_ops.publication_heads
    ALTER COLUMN advanced_at SET DEFAULT now();
-- Protected-version computation joins members by table and version.
CREATE INDEX publication_members_version_idx
    ON pse_ops.publication_members (table_uri, delta_version);
CREATE INDEX publication_windows_table_idx
    ON pse_ops.publication_windows (table_uri, from_version);
CREATE INDEX settlements_attempt_idx ON pse_ops.settlements (attempt_id, settled_at);
ALTER TABLE pse_ops.settlements
    ALTER COLUMN settled_at SET DEFAULT now();
CREATE INDEX reader_leases_active_idx ON pse_ops.reader_leases (publication_id, expires_at)
    WHERE released_at IS NULL;
ALTER TABLE pse_ops.reader_leases
    ALTER COLUMN acquired_at SET DEFAULT now();
ALTER TABLE pse_ops.retention_marks
    ALTER COLUMN marked_at SET DEFAULT now();
