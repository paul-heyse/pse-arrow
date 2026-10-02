-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The publication catalog (ADR-0114 Outcomes 6 and 7; Plan 22 X9, X10). One transaction
-- is the visibility boundary; publication intents are registered before any member write;
-- readers hold short-lived lease rows; maintainers of a workspace are serialized by a
-- transaction-scoped advisory lock and advance its maintenance epoch before any effect.
-- Members and windows are inserted by binary COPY (src/generated/copy.rs).
--
-- Commit lock order: attempt (FOR SHARE), intent (FOR UPDATE), every live publication a
-- retained member or input selects (FOR SHARE), head (FOR UPDATE). Maintenance takes the
-- advisory lock, then the publication (FOR NO KEY UPDATE), then the workspace row.
-- A lock waited for is followed by a separate statement, never read in the same one: a
-- row that was only locked (not updated) is not rechecked under READ COMMITTED.

-- ------------------------------------------------------------------ workspaces --

--! insert_workspace
INSERT INTO pse_ops.workspaces (workspace_id, name, root_uri, maintenance_epoch)
VALUES (:workspace_id, :name, :root_uri, 0);

--! insert_head
INSERT INTO pse_ops.publication_heads (workspace_id) VALUES (:workspace_id);

--! workspace
SELECT w FROM pse_ops.workspaces AS w
WHERE w.workspace_id = :workspace_id::pse_ops.workspace_id;

--! workspace_by_name
SELECT w FROM pse_ops.workspaces AS w WHERE w.name = :name;

--! head
SELECT h FROM pse_ops.publication_heads AS h
WHERE h.workspace_id = :workspace_id::pse_ops.workspace_id;

--! lock_head
SELECT h FROM pse_ops.publication_heads AS h
WHERE h.workspace_id = :workspace_id::pse_ops.workspace_id
FOR UPDATE;

--! advance_head
UPDATE pse_ops.publication_heads SET publication_id = :publication_id, advanced_at = now()
WHERE workspace_id = :workspace_id::pse_ops.workspace_id;

-- --------------------------------------------------------------------- intents --

--! insert_intent
INSERT INTO pse_ops.publication_intents (publication_id, workspace_id, attempt_id, member_prefix)
VALUES (:publication_id, :workspace_id, :attempt_id, :member_prefix)
ON CONFLICT (publication_id) DO NOTHING;

--! intent
SELECT i FROM pse_ops.publication_intents AS i
WHERE i.publication_id = :publication_id::pse_ops.publication_id;

--! lock_intent
SELECT i FROM pse_ops.publication_intents AS i
WHERE i.publication_id = :publication_id::pse_ops.publication_id
FOR UPDATE;

--! abandon_intent
UPDATE pse_ops.publication_intents SET abandoned_at = coalesce(abandoned_at, now())
WHERE publication_id = :publication_id::pse_ops.publication_id;

--! reclaim_intent
UPDATE pse_ops.publication_intents
SET abandoned_at = coalesce(abandoned_at, now()), reclaimed_at = coalesce(reclaimed_at, now())
WHERE publication_id = :publication_id::pse_ops.publication_id;

-- Unpublished intents of a workspace that can never commit: abandoned, their attempt
-- published as another publication, or their attempt stale or superseded. Locked so the
-- caller fences them (abandons them) before it removes any file.
--! reclaimable_intents
SELECT i FROM pse_ops.publication_intents AS i
JOIN pse_ops.attempts AS a ON a.attempt_id = i.attempt_id
WHERE i.workspace_id = :workspace_id::pse_ops.workspace_id
  AND i.reclaimed_at IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM pse_ops.publications AS p WHERE p.publication_id = i.publication_id)
  AND (i.abandoned_at IS NOT NULL
       OR EXISTS (SELECT 1 FROM pse_ops.publications AS p WHERE p.attempt_id = i.attempt_id)
       OR a.state = :stale
       OR a.state = :superseded)
ORDER BY i.prepared_at, i.publication_id
FOR UPDATE OF i;

-- ----------------------------------------------------------------- publications --

-- FOR SHARE on the attempt: a concurrent settlement (FOR UPDATE) waits for the commit.
--! share_attempt_state
SELECT state FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id
FOR SHARE;

--! lock_attempt_row
SELECT true AS found FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id
FOR UPDATE;

--! publication
SELECT p FROM pse_ops.publications AS p
WHERE p.publication_id = :publication_id::pse_ops.publication_id;

--! publication_of_attempt
SELECT p FROM pse_ops.publications AS p WHERE p.attempt_id = :attempt_id::pse_ops.attempt_id;

--! members
SELECT m FROM pse_ops.publication_members AS m
WHERE m.publication_id = :publication_id::pse_ops.publication_id
ORDER BY m.role, m.catalog_name, m.schema_name, m.table_name;

--! windows
SELECT w FROM pse_ops.publication_windows AS w
WHERE w.publication_id = :publication_id::pse_ops.publication_id
ORDER BY w.table_uri, w.from_version;

-- Every publication selecting an exact table version, locked against being marked for
-- deletion until this transaction ends.
--! share_selecting_publications
SELECT p.publication_id FROM pse_ops.publications AS p
WHERE EXISTS (
    SELECT 1 FROM pse_ops.publication_members AS m
    WHERE m.publication_id = p.publication_id
      AND m.table_uri = :table_uri
      AND m.delta_version = :delta_version)
ORDER BY p.publication_id
FOR SHARE;

-- Whether a live (unmarked) publication selects the exact table version; read after
-- `share_selecting_publications` locked them, in a new statement.
--! live_selection
SELECT count(*) AS live FROM pse_ops.publication_members AS m
WHERE m.table_uri = :table_uri AND m.delta_version = :delta_version
  AND NOT EXISTS (
      SELECT 1 FROM pse_ops.retention_marks AS r WHERE r.publication_id = m.publication_id);

--! insert_publication (parent_publication?)
INSERT INTO pse_ops.publications (publication_id, workspace_id, parent_publication, attempt_id, kind)
VALUES (:publication_id, :workspace_id, :parent_publication, :attempt_id, :kind);

-- ------------------------------------------------------------------ settlement --

--! insert_settlement (publication_id?, reason?, conflict_head?)
INSERT INTO pse_ops.settlements (settlement_id, attempt_id, outcome, publication_id, reason, conflict_head)
VALUES (:settlement_id, :attempt_id, :outcome, :publication_id, :reason, :conflict_head);

-- --------------------------------------------------------------- reader leases --

-- FOR SHARE conflicts with maintenance's FOR NO KEY UPDATE: a lease is either granted
-- before a publication is marked expiring or refused after. The mark is read by
-- `retention_phase`, in a new statement, once this lock is held.
--! share_publication
SELECT p FROM pse_ops.publications AS p
WHERE p.publication_id = :publication_id::pse_ops.publication_id
FOR SHARE;

--! retention_phase
SELECT r.phase FROM pse_ops.retention_marks AS r
WHERE r.publication_id = :publication_id::pse_ops.publication_id;

--! maintenance_epoch
SELECT maintenance_epoch FROM pse_ops.workspaces
WHERE workspace_id = :workspace_id::pse_ops.workspace_id;

--! insert_reader_lease (head_of?)
INSERT INTO pse_ops.reader_leases AS l (lease_id, publication_id, head_of, holder, expires_at)
VALUES (:lease_id, :publication_id, :head_of, :holder,
        clock_timestamp() + :ttl_us::bigint * interval '1 microsecond')
RETURNING l;

--! renew_reader_lease
UPDATE pse_ops.reader_leases AS l
SET expires_at = clock_timestamp() + :ttl_us::bigint * interval '1 microsecond'
WHERE l.lease_id = :lease_id::pse_ops.reader_lease_id
  AND l.released_at IS NULL
  AND l.expires_at > clock_timestamp()
RETURNING l;

--! reader_lease
SELECT l FROM pse_ops.reader_leases AS l WHERE l.lease_id = :lease_id::pse_ops.reader_lease_id;

--! release_reader_lease
UPDATE pse_ops.reader_leases SET released_at = now()
WHERE lease_id = :lease_id::pse_ops.reader_lease_id AND released_at IS NULL;

--! active_leases
SELECT count(*) AS active FROM pse_ops.reader_leases
WHERE publication_id = :publication_id::pse_ops.publication_id
  AND released_at IS NULL
  AND expires_at > now();

-- ------------------------------------------------------------------ maintenance --

-- Namespace protection fence: producers/readers share it, explicit selected deletion
-- takes it exclusively before the workspace lock. The key is declared once in Rust.
--! protection_shared_lock
SELECT true AS locked FROM pg_advisory_xact_lock_shared(:lock_key::bigint);

-- Serializes maintainers of one workspace until the transaction ends.
--! maintenance_lock
SELECT true AS locked FROM pg_advisory_xact_lock(hashtextextended(:key, 0));

-- The epoch advances before any maintenance effect; readers key their caches on it.
--! advance_epoch
UPDATE pse_ops.workspaces SET maintenance_epoch = maintenance_epoch + 1
WHERE workspace_id = :workspace_id::pse_ops.workspace_id
RETURNING maintenance_epoch;

-- Maintenance takes the publication's row so that no lease is granted concurrently.
--! lock_publication : (phase?)
SELECT p.workspace_id, r.phase
FROM pse_ops.publications AS p
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id
WHERE p.publication_id = :publication_id::pse_ops.publication_id
FOR NO KEY UPDATE OF p;

--! mark_expiring
INSERT INTO pse_ops.retention_marks (publication_id, phase)
VALUES (:publication_id, :phase)
ON CONFLICT (publication_id) DO NOTHING;

--! mark_deleted
UPDATE pse_ops.retention_marks SET phase = :phase, deleted_at = coalesce(deleted_at, now())
WHERE publication_id = :publication_id::pse_ops.publication_id;

-- The tables a deletion removes: the tables the publication selects (its outputs, and
-- inputs whose writer is already deleted) that a publication of this workspace wrote
-- (under one of its intents' prefixes), that no other undeleted publication selects and
-- no undeleted publication's change window covers.
--! deletion_candidates
SELECT DISTINCT m.table_uri FROM pse_ops.publication_members AS m
WHERE m.publication_id = :publication_id::pse_ops.publication_id
  AND EXISTS (
      SELECT 1 FROM pse_ops.publication_intents AS i
      WHERE i.workspace_id = :workspace_id::pse_ops.workspace_id
        AND starts_with(m.table_uri, i.member_prefix))
  AND NOT EXISTS (
      SELECT 1 FROM pse_ops.publication_members AS o
      LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = o.publication_id
      WHERE o.table_uri = m.table_uri
        AND o.publication_id <> m.publication_id
        AND (r.phase IS NULL OR r.phase <> :deleted))
  AND NOT EXISTS (
      SELECT 1 FROM pse_ops.publication_windows AS w
      LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = w.publication_id
      WHERE w.table_uri = m.table_uri
        AND w.publication_id <> m.publication_id
        AND (r.phase IS NULL OR r.phase <> :deleted))
ORDER BY m.table_uri;

-- The tables collection maintains: every table a publication of this workspace wrote
-- (under one of its intents' prefixes) that an undeleted publication still selects, as a
-- member or an input.
--! collected_tables
SELECT DISTINCT m.table_uri FROM pse_ops.publication_members AS m
JOIN pse_ops.publication_intents AS i ON starts_with(m.table_uri, i.member_prefix)
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = m.publication_id
WHERE i.workspace_id = :workspace_id::pse_ops.workspace_id
  AND (r.phase IS NULL OR r.phase <> :deleted)
ORDER BY m.table_uri;

-- The exact versions live publications select (members and inputs) of the workspace's
-- tables. A publication is live while unmarked, or while a live lease still reads it.
--! protected_members
SELECT DISTINCT m.table_uri, m.delta_version
FROM pse_ops.publication_members AS m
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = m.publication_id
WHERE EXISTS (
      SELECT 1 FROM pse_ops.publication_intents AS i
      WHERE i.workspace_id = :workspace_id::pse_ops.workspace_id
        AND starts_with(m.table_uri, i.member_prefix))
  AND (r.publication_id IS NULL OR (r.phase = :expiring AND EXISTS (
      SELECT 1 FROM pse_ops.reader_leases AS l
      WHERE l.publication_id = m.publication_id
        AND l.released_at IS NULL
        AND l.expires_at > now())))
ORDER BY m.table_uri, m.delta_version;

-- The change windows live publications read, on the workspace's tables.
--! protected_windows
SELECT DISTINCT w.table_uri, w.from_version, w.through_version
FROM pse_ops.publication_windows AS w
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = w.publication_id
WHERE EXISTS (
      SELECT 1 FROM pse_ops.publication_intents AS i
      WHERE i.workspace_id = :workspace_id::pse_ops.workspace_id
        AND starts_with(w.table_uri, i.member_prefix))
  AND (r.publication_id IS NULL OR (r.phase = :expiring AND EXISTS (
      SELECT 1 FROM pse_ops.reader_leases AS l
      WHERE l.publication_id = w.publication_id
        AND l.released_at IS NULL
        AND l.expires_at > now())))
ORDER BY w.table_uri, w.from_version;

-- The member prefixes of the workspace's live intents: unpublished, never abandoned.
-- Every table under one keeps its complete history, which recovery reads.
--! live_intent_prefixes
SELECT i.member_prefix FROM pse_ops.publication_intents AS i
WHERE i.workspace_id = :workspace_id::pse_ops.workspace_id
  AND i.abandoned_at IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM pse_ops.publications AS p WHERE p.publication_id = i.publication_id)
ORDER BY i.member_prefix;
