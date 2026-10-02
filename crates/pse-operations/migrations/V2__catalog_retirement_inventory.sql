-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Immutable Plan 25f -> 25g catalog retirement inventory transition (ADR-0146).

ALTER TYPE pse_ops.publication_kind ADD VALUE 'migration' AFTER 'inspection';

CREATE TYPE pse_ops.orphan_disposition AS ENUM ('discovered', 'protected', 'claimed', 'deleted', 'unresolved');

CREATE TYPE pse_ops.orphan_ownership AS ENUM ('attributable', 'unattributable');

CREATE DOMAIN pse_ops.reset_id AS uuid;

CREATE DOMAIN pse_ops.scan_id AS uuid;

CREATE TABLE pse_ops."orphan_scans" (
    "scan_id" pse_ops.scan_id NOT NULL,
    "workspace_id" pse_ops.workspace_id NOT NULL,
    "root_uri" text NOT NULL,
    "maintenance_epoch" bigint NOT NULL,
    "generation" bigint NOT NULL,
    "listed_count" bigint NOT NULL,
    "complete" boolean NOT NULL,
    CONSTRAINT orphan_scans_pkey PRIMARY KEY ("scan_id"),
    CONSTRAINT orphan_scans_counts_nonnegative_check CHECK ("maintenance_epoch" >= 0 AND "listed_count" >= 0),
    CONSTRAINT orphan_scans_generation_positive_check CHECK ("generation" > 0),
    CONSTRAINT orphan_scans_root_nonempty_check CHECK ("root_uri" <> '')
);

CREATE TABLE pse_ops."orphan_candidates" (
    "scan_id" pse_ops.scan_id NOT NULL,
    "prefix" text NOT NULL,
    "generation" bigint NOT NULL,
    "discovery_epoch" bigint NOT NULL,
    "ownership" pse_ops.orphan_ownership NOT NULL,
    "evidence" jsonb NOT NULL,
    "protections" jsonb NOT NULL,
    "disposition" pse_ops.orphan_disposition NOT NULL,
    "claim_epoch" bigint,
    CONSTRAINT orphan_candidates_pkey PRIMARY KEY ("scan_id", "prefix"),
    CONSTRAINT orphan_candidates_claim_epoch_nonnegative_check CHECK ("claim_epoch" IS NULL OR "claim_epoch" >= 0),
    CONSTRAINT orphan_candidates_epoch_nonnegative_check CHECK ("discovery_epoch" >= 0),
    CONSTRAINT orphan_candidates_generation_positive_check CHECK ("generation" > 0),
    CONSTRAINT orphan_candidates_prefix_nonempty_check CHECK ("prefix" <> '')
);

CREATE TABLE pse_ops."reset_records" (
    "reset_id" pse_ops.reset_id NOT NULL,
    "manifest_digest" pse_ops.content_hash NOT NULL,
    "manifest_uri" text NOT NULL,
    "source_fingerprint" text NOT NULL,
    "inventory_rows" bigint NOT NULL,
    CONSTRAINT reset_records_pkey PRIMARY KEY ("reset_id"),
    CONSTRAINT reset_records_manifest_nonempty_check CHECK ("manifest_uri" <> ''),
    CONSTRAINT reset_records_rows_nonnegative_check CHECK ("inventory_rows" >= 0),
    CONSTRAINT reset_records_source_nonempty_check CHECK ("source_fingerprint" <> '')
);

CREATE TABLE pse_ops."retired_inventory" (
    "reset_id" pse_ops.reset_id NOT NULL,
    "ordinal" bigint NOT NULL,
    "workspace_id" pse_ops.workspace_id,
    "root_uri" text,
    "prefix" text,
    "record_kind" text NOT NULL,
    "document" jsonb NOT NULL,
    "protections" jsonb NOT NULL,
    "disposition" pse_ops.orphan_disposition NOT NULL,
    CONSTRAINT retired_inventory_pkey PRIMARY KEY ("reset_id", "ordinal"),
    CONSTRAINT retired_inventory_kind_nonempty_check CHECK ("record_kind" <> ''),
    CONSTRAINT retired_inventory_ordinal_nonnegative_check CHECK ("ordinal" >= 0)
);

ALTER TABLE pse_ops."orphan_candidates" ADD CONSTRAINT orphan_candidates_scan_id_fkey
    FOREIGN KEY ("scan_id") REFERENCES pse_ops."orphan_scans" ("scan_id");

ALTER TABLE pse_ops."orphan_scans" ADD CONSTRAINT orphan_scans_workspace_id_fkey
    FOREIGN KEY ("workspace_id") REFERENCES pse_ops."workspaces" ("workspace_id");

ALTER TABLE pse_ops."retired_inventory" ADD CONSTRAINT retired_inventory_reset_id_fkey
    FOREIGN KEY ("reset_id") REFERENCES pse_ops."reset_records" ("reset_id");

UPDATE pse_ops.schema_support_state SET target='bb3f11310b0fb5b58e2ec4222be9a71a57d6923a60f67541a61d793c67b20eb2',source=CASE WHEN source='fresh' THEN 'fresh-v25f' ELSE source END,ready=false WHERE history='catalog';
UPDATE pse_ops.schema_support_state SET ready=false WHERE history='operations';
