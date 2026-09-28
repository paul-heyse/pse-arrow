-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The publication catalog (ADR-0114 Outcomes 6 and 7): one transaction is the visibility
-- boundary; readers hold short-lived lease rows; maintenance marks a publication
-- expiring, waits for its leases and marks it deleted. Members are inserted by binary
-- COPY (src/generated/copy.rs).

--! insert_workspace
INSERT INTO pse_ops.workspaces (workspace_id, name, root_uri)
VALUES (:workspace_id, :name, :root_uri);

--! insert_head
INSERT INTO pse_ops.publication_heads (workspace_id) VALUES (:workspace_id);

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

-- FOR SHARE on the attempt: a concurrent settlement (FOR UPDATE) waits for the commit.
--! share_attempt_state
SELECT state FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id
FOR SHARE;

--! lock_attempt_row
SELECT true AS found FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id
FOR UPDATE;

--! publication_of_attempt
SELECT p FROM pse_ops.publications AS p WHERE p.attempt_id = :attempt_id::pse_ops.attempt_id;

--! insert_publication (parent_publication?)
INSERT INTO pse_ops.publications (publication_id, workspace_id, parent_publication, attempt_id)
VALUES (:publication_id, :workspace_id, :parent_publication, :attempt_id);

--! members
SELECT m FROM pse_ops.publication_members AS m
WHERE m.publication_id = :publication_id::pse_ops.publication_id
ORDER BY m.member;

--! insert_settlement (publication_id?)
INSERT INTO pse_ops.settlements (settlement_id, attempt_id, outcome, publication_id)
VALUES (:settlement_id, :attempt_id, :outcome, :publication_id);

-- Maintenance takes the publication's row so that no lease is granted concurrently.
--! lock_publication : (phase?)
SELECT p.workspace_id, r.phase
FROM pse_ops.publications AS p
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id
WHERE p.publication_id = :publication_id::pse_ops.publication_id
FOR NO KEY UPDATE OF p;

-- FOR SHARE conflicts with maintenance's FOR NO KEY UPDATE: a lease is either granted
-- before a publication is marked expiring or refused after.
--! share_publication : (phase?)
SELECT r.phase
FROM pse_ops.publications AS p
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id
WHERE p.publication_id = :publication_id::pse_ops.publication_id
FOR SHARE OF p;

--! insert_reader_lease
INSERT INTO pse_ops.reader_leases AS l (lease_id, publication_id, holder, expires_at)
VALUES (:lease_id, :publication_id, :holder,
        now() + :ttl_us::bigint * interval '1 microsecond')
RETURNING l;

--! release_reader_lease
UPDATE pse_ops.reader_leases SET released_at = now()
WHERE lease_id = :lease_id::pse_ops.reader_lease_id AND released_at IS NULL;

--! active_leases
SELECT count(*) AS active FROM pse_ops.reader_leases
WHERE publication_id = :publication_id::pse_ops.publication_id
  AND released_at IS NULL
  AND expires_at > now();

-- Serializes maintainers of one workspace until the transaction ends.
--! maintenance_lock
SELECT true AS locked FROM pg_advisory_xact_lock(hashtextextended(:key, 0));

--! mark_expiring
INSERT INTO pse_ops.retention_marks (publication_id, phase)
VALUES (:publication_id, :phase)
ON CONFLICT (publication_id) DO NOTHING;

--! mark_deleted
UPDATE pse_ops.retention_marks SET phase = :phase, deleted_at = now()
WHERE publication_id = :publication_id::pse_ops.publication_id;

-- The members of every publication not marked for deletion, and of any publication a live
-- lease still reads.
--! protected_versions : ProtectedVersion()
SELECT DISTINCT m.table_uri, m.delta_version
FROM pse_ops.publication_members AS m
JOIN pse_ops.publications AS p ON p.publication_id = m.publication_id
LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id
WHERE p.workspace_id = :workspace_id::pse_ops.workspace_id
  AND (r.publication_id IS NULL OR EXISTS (
      SELECT 1 FROM pse_ops.reader_leases AS l
      WHERE l.publication_id = p.publication_id
        AND l.released_at IS NULL
        AND l.expires_at > now()))
ORDER BY m.table_uri, m.delta_version;
