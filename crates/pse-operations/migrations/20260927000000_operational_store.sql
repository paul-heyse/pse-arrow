-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The operational store and publication catalog (ADR-0112; Plan 22 architecture §9.3).
--
-- PostgreSQL owns what changes; Delta owns what is published. Enumerations are text with a
-- CHECK on their value domain, never database enum types. Transition legality is owned by
-- the one Rust transition table (pse_operations::lifecycle), never by SQL. Domain
-- identities (attempts, runs, publications, workspaces, studies, solutions) are minted by the
-- runtime; uuidv7() defaults key only rows without a domain identity. Content hashes are
-- the 32-byte BLAKE3 values of blueprint §5.1. CHECK constraints state local value
-- invariants only.

CREATE SCHEMA pse_ops;

-- ---------------------------------------------------------------- sources --

-- Authored sources for job execution, content-addressed by the §6.1 content hash.
CREATE TABLE pse_ops.source_bundles (
    bundle_hash bytea PRIMARY KEY CHECK (octet_length(bundle_hash) = 32),
    manifest jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE pse_ops.source_documents (
    bundle_hash bytea NOT NULL
        REFERENCES pse_ops.source_bundles (bundle_hash) ON DELETE CASCADE,
    path text NOT NULL CHECK (path <> ''),
    content_hash bytea NOT NULL CHECK (octet_length(content_hash) = 32),
    content bytea NOT NULL,
    PRIMARY KEY (bundle_hash, path)
);

-- --------------------------------------------------------------- attempts --

-- One row per attempt. The lease (worker, lease_expires_at) lives here only: a job
-- references its current attempt and never duplicates the lease (DP-01).
CREATE TABLE pse_ops.attempts (
    attempt_id uuid PRIMARY KEY,
    run_id uuid NOT NULL,
    kind text NOT NULL CHECK (kind <> ''),
    request_identity bytea NOT NULL CHECK (octet_length(request_identity) = 32),
    preparation_identity bytea CHECK (octet_length(preparation_identity) = 32),
    state text NOT NULL CHECK (state IN (
        'planned', 'queued', 'running', 'completed', 'partial',
        'failed', 'cancelled', 'stale', 'superseded'
    )),
    state_version integer NOT NULL DEFAULT 0 CHECK (state_version >= 0),
    parent_attempt uuid REFERENCES pse_ops.attempts (attempt_id),
    worker text CHECK (worker <> ''),
    lease_expires_at timestamptz,
    heartbeat_at timestamptz,
    cancel_requested boolean NOT NULL DEFAULT false,
    cancel_requested_at timestamptz,
    termination text CHECK (termination <> ''),
    termination_detail jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz,
    CHECK (parent_attempt IS DISTINCT FROM attempt_id),
    -- A running attempt always has an owner and a lease; nothing else holds a lease.
    CHECK ((state = 'running') = (lease_expires_at IS NOT NULL)),
    CHECK (state <> 'running' OR worker IS NOT NULL),
    CHECK (cancel_requested = (cancel_requested_at IS NOT NULL))
);

CREATE INDEX attempts_run_idx ON pse_ops.attempts (run_id);
CREATE INDEX attempts_state_idx ON pse_ops.attempts (state, created_at);
-- The stale sweep scans running attempts by lease expiry.
CREATE INDEX attempts_lease_idx ON pse_ops.attempts (lease_expires_at)
    WHERE state = 'running';
CREATE INDEX attempts_parent_idx ON pse_ops.attempts (parent_attempt)
    WHERE parent_attempt IS NOT NULL;

-- Append-only audit of every state change, written in the transaction that changes the
-- state. seq 0 records creation.
CREATE TABLE pse_ops.attempt_transitions (
    attempt_id uuid NOT NULL REFERENCES pse_ops.attempts (attempt_id),
    seq integer NOT NULL CHECK (seq >= 0),
    from_state text CHECK (from_state IN (
        'planned', 'queued', 'running', 'completed', 'partial',
        'failed', 'cancelled', 'stale', 'superseded'
    )),
    to_state text NOT NULL CHECK (to_state IN (
        'planned', 'queued', 'running', 'completed', 'partial',
        'failed', 'cancelled', 'stale', 'superseded'
    )),
    actor text,
    reason text,
    at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (attempt_id, seq),
    CHECK ((seq = 0) = (from_state IS NULL))
);

-- History is append-only for the application role (ADR-0112 Outcome 12). The owner keeps
-- INSERT and SELECT; UPDATE, DELETE and TRUNCATE are refused with SQLSTATE 42501.
REVOKE UPDATE, DELETE, TRUNCATE ON pse_ops.attempt_transitions FROM CURRENT_USER;

-- ------------------------------------------------------------------- jobs --

-- Durable work. Each try runs as a new attempt; attempt_id names the current one.
CREATE TABLE pse_ops.jobs (
    job_id uuid PRIMARY KEY DEFAULT uuidv7(),
    attempt_id uuid NOT NULL UNIQUE REFERENCES pse_ops.attempts (attempt_id),
    idempotency_key text NOT NULL UNIQUE CHECK (idempotency_key <> ''),
    payload_version integer NOT NULL CHECK (payload_version > 0),
    payload jsonb NOT NULL,
    priority integer NOT NULL DEFAULT 0,
    state text NOT NULL CHECK (state IN (
        'queued', 'running', 'completed', 'failed', 'cancelled'
    )),
    tries integer NOT NULL DEFAULT 0 CHECK (tries >= 0),
    max_tries integer NOT NULL CHECK (max_tries > 0),
    backoff_base interval NOT NULL CHECK (backoff_base >= interval '0'),
    backoff_cap interval NOT NULL,
    available_at timestamptz NOT NULL DEFAULT now(),
    enqueued_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    last_error text,
    CHECK (tries <= max_tries),
    CHECK (backoff_cap >= backoff_base)
);

-- The claim query: queued jobs by priority, then availability.
CREATE INDEX jobs_claim_idx ON pse_ops.jobs (priority DESC, available_at, job_id)
    WHERE state = 'queued';

-- ---------------------------------------------------------------- streams --

-- Live progress, keyed by (attempt, sequence) as the producer numbers it.
CREATE TABLE pse_ops.progress_events (
    attempt_id uuid NOT NULL REFERENCES pse_ops.attempts (attempt_id),
    seq bigint NOT NULL CHECK (seq >= 0),
    at timestamptz NOT NULL,
    phase text NOT NULL CHECK (phase <> ''),
    payload jsonb NOT NULL,
    PRIMARY KEY (attempt_id, seq)
);

-- Reusable solutions (primal, dual, working set, basis), keyed by the coordinate
-- compatibility stamp and the preparation identity. The payload is versioned Arrow IPC.
CREATE TABLE pse_ops.solutions (
    solution_id uuid PRIMARY KEY,
    compatibility_stamp bytea NOT NULL CHECK (octet_length(compatibility_stamp) = 32),
    preparation_identity bytea NOT NULL CHECK (octet_length(preparation_identity) = 32),
    kind text NOT NULL CHECK (kind <> ''),
    payload_format text NOT NULL CHECK (payload_format <> ''),
    payload_version integer NOT NULL CHECK (payload_version > 0),
    payload bytea NOT NULL,
    created_by uuid REFERENCES pse_ops.attempts (attempt_id),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX solutions_seed_idx
    ON pse_ops.solutions (compatibility_stamp, preparation_identity, kind, created_at DESC);

-- Incumbents and bounds, keyed by (attempt, sequence).
CREATE TABLE pse_ops.incumbents (
    attempt_id uuid NOT NULL REFERENCES pse_ops.attempts (attempt_id),
    seq bigint NOT NULL CHECK (seq >= 0),
    at timestamptz NOT NULL,
    objective double precision NOT NULL CHECK (objective <> 'NaN'::double precision),
    dual_bound double precision CHECK (dual_bound <> 'NaN'::double precision),
    gap double precision CHECK (gap >= 0),
    solution_id uuid REFERENCES pse_ops.solutions (solution_id),
    PRIMARY KEY (attempt_id, seq)
);

-- ---------------------------------------------------------------- studies --

CREATE TABLE pse_ops.studies (
    study_id uuid PRIMARY KEY,
    definition jsonb NOT NULL,
    state text NOT NULL CHECK (state IN ('open', 'completed', 'cancelled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE pse_ops.study_points (
    study_id uuid NOT NULL REFERENCES pse_ops.studies (study_id) ON DELETE CASCADE,
    point_index integer NOT NULL CHECK (point_index >= 0),
    binding_hash bytea NOT NULL CHECK (octet_length(binding_hash) = 32),
    state text NOT NULL CHECK (state IN (
        'pending', 'assigned', 'completed', 'failed', 'cancelled'
    )),
    attempt_id uuid UNIQUE REFERENCES pse_ops.attempts (attempt_id),
    result_ref text,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (study_id, point_index),
    UNIQUE (study_id, binding_hash)
);

CREATE INDEX study_points_pending_idx ON pse_ops.study_points (study_id, point_index)
    WHERE state = 'pending';

-- ---------------------------------------------------------------- catalog --

CREATE TABLE pse_ops.workspaces (
    workspace_id uuid PRIMARY KEY,
    name text NOT NULL UNIQUE CHECK (name <> ''),
    root_uri text NOT NULL CHECK (root_uri <> ''),
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Immutable publication records. The attempt identity is unique: publication is
-- idempotent per attempt, and settlement queries this table.
CREATE TABLE pse_ops.publications (
    publication_id uuid PRIMARY KEY,
    workspace_id uuid NOT NULL REFERENCES pse_ops.workspaces (workspace_id),
    parent_publication uuid REFERENCES pse_ops.publications (publication_id),
    attempt_id uuid NOT NULL UNIQUE REFERENCES pse_ops.attempts (attempt_id),
    committed_at timestamptz NOT NULL DEFAULT now(),
    CHECK (parent_publication IS DISTINCT FROM publication_id)
);

CREATE INDEX publications_workspace_idx ON pse_ops.publications (workspace_id, committed_at);

-- One head row per workspace, created with the workspace, so the compare-and-set commit
-- always locks an existing row. A NULL head means nothing has been published yet.
CREATE TABLE pse_ops.publication_heads (
    workspace_id uuid PRIMARY KEY REFERENCES pse_ops.workspaces (workspace_id),
    publication_id uuid REFERENCES pse_ops.publications (publication_id),
    advanced_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE pse_ops.publication_members (
    publication_id uuid NOT NULL REFERENCES pse_ops.publications (publication_id),
    member text NOT NULL CHECK (member <> ''),
    table_uri text NOT NULL CHECK (table_uri <> ''),
    delta_version bigint NOT NULL CHECK (delta_version >= 0),
    contract_fingerprint bytea NOT NULL CHECK (octet_length(contract_fingerprint) = 32),
    PRIMARY KEY (publication_id, member)
);

-- Protected-version computation joins members by table and version.
CREATE INDEX publication_members_version_idx
    ON pse_ops.publication_members (table_uri, delta_version);

-- Settlement inquiries after an uncertain commit acknowledgement, and their outcome.
CREATE TABLE pse_ops.settlements (
    settlement_id uuid PRIMARY KEY DEFAULT uuidv7(),
    attempt_id uuid NOT NULL REFERENCES pse_ops.attempts (attempt_id),
    outcome text NOT NULL CHECK (outcome IN ('committed', 'proved_noncommit')),
    publication_id uuid REFERENCES pse_ops.publications (publication_id),
    settled_at timestamptz NOT NULL DEFAULT now(),
    CHECK ((outcome = 'committed') = (publication_id IS NOT NULL))
);

CREATE INDEX settlements_attempt_idx ON pse_ops.settlements (attempt_id, settled_at);

-- Reader leases: taken in a short transaction and released when the read is done; no
-- reader holds a database session while it reads Delta files (finding T02).
CREATE TABLE pse_ops.reader_leases (
    lease_id uuid PRIMARY KEY DEFAULT uuidv7(),
    publication_id uuid NOT NULL REFERENCES pse_ops.publications (publication_id),
    holder text NOT NULL CHECK (holder <> ''),
    acquired_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    released_at timestamptz,
    CHECK (expires_at > acquired_at)
);

CREATE INDEX reader_leases_active_idx ON pse_ops.reader_leases (publication_id, expires_at)
    WHERE released_at IS NULL;

-- Two-phase deletion state. A publication without a mark is live; 'expiring' refuses new
-- leases; 'deleted' records that the member files were removed.
CREATE TABLE pse_ops.retention_marks (
    publication_id uuid PRIMARY KEY REFERENCES pse_ops.publications (publication_id),
    phase text NOT NULL CHECK (phase IN ('expiring', 'deleted')),
    marked_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz,
    CHECK ((phase = 'deleted') = (deleted_at IS NOT NULL))
);
