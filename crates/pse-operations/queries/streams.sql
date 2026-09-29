-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- Live progress and incumbent streams (ADR-0114 Outcome 17). Rows are inserted by binary
-- COPY (src/generated/copy.rs); these statements find the sequence numbers already
-- stored, read pages back and apply retention.

--! existing_progress_seqs
SELECT seq FROM pse_ops.progress_events
WHERE attempt_id = :attempt_id::pse_ops.attempt_id AND seq = ANY(:seqs::bigint[]);

--! progress_page (after?)
SELECT e FROM pse_ops.progress_events AS e
WHERE e.attempt_id = :attempt_id::pse_ops.attempt_id
  AND e.seq > coalesce(:after::bigint, -1)
ORDER BY e.seq
LIMIT :limit;

--! progress_values
SELECT v FROM pse_ops.progress_values AS v
WHERE v.attempt_id = :attempt_id::pse_ops.attempt_id AND v.seq BETWEEN :first AND :last
ORDER BY v.seq, v.name;

-- Deletes are explicit (no cascade): an event's values go before the event.
--! delete_finished_values
DELETE FROM pse_ops.progress_values AS v USING pse_ops.attempts AS a
WHERE v.attempt_id = a.attempt_id
  AND a.finished_at IS NOT NULL
  AND a.finished_at < now() - :age_us::bigint * interval '1 microsecond';

--! delete_finished_events
DELETE FROM pse_ops.progress_events AS e USING pse_ops.attempts AS a
WHERE e.attempt_id = a.attempt_id
  AND a.finished_at IS NOT NULL
  AND a.finished_at < now() - :age_us::bigint * interval '1 microsecond';

-- Incumbents expire with their stream, except in the attempt chain (parent_attempt) above
-- an unfinished attempt, whose resumed try may still start from one of them.
--! delete_finished_incumbents
WITH RECURSIVE resumable (attempt_id, depth) AS (
    SELECT a.parent_attempt, 1 FROM pse_ops.attempts AS a
    WHERE a.finished_at IS NULL AND a.parent_attempt IS NOT NULL
  UNION ALL
    SELECT a.parent_attempt, r.depth + 1
    FROM resumable AS r JOIN pse_ops.attempts AS a ON a.attempt_id = r.attempt_id
    WHERE a.parent_attempt IS NOT NULL AND r.depth < 4096
)
DELETE FROM pse_ops.incumbents AS i USING pse_ops.attempts AS a
WHERE i.attempt_id = a.attempt_id
  AND a.finished_at IS NOT NULL
  AND a.finished_at < now() - :age_us::bigint * interval '1 microsecond'
  AND NOT EXISTS (SELECT 1 FROM resumable AS r WHERE r.attempt_id = i.attempt_id);

-- Captured solutions (Plan 22 I13) go with their stream: those no remaining incumbent, no
-- unfinished job's stored start and no waiting study point's predecessor seed names.
-- Output seeds are never pruned here.
--! delete_unreferenced_captures
DELETE FROM pse_ops.solutions AS s
WHERE s.origin = 'incumbent'
  AND NOT EXISTS (SELECT 1 FROM pse_ops.incumbents AS i WHERE i.solution_id = s.solution_id)
  AND NOT EXISTS (
    SELECT 1 FROM pse_ops.jobs AS j
    WHERE j.state IN ('waiting', 'queued', 'running')
      AND j.payload #>> '{task,start,kind}' = 'stored_solution'
      AND (j.payload #>> '{task,start,solution}')::uuid = s.solution_id)
  AND NOT EXISTS (
    SELECT 1 FROM pse_ops.study_points AS p
    JOIN pse_ops.study_points AS q ON q.study_id = p.study_id AND q.point_index = p.predecessor
    JOIN pse_ops.jobs AS j ON j.job_id = q.job_id
    WHERE p.state IN ('pending', 'assigned') AND j.attempt_id = s.created_by);

--! existing_incumbent_seqs
SELECT seq FROM pse_ops.incumbents
WHERE attempt_id = :attempt_id::pse_ops.attempt_id AND seq = ANY(:seqs::bigint[]);

--! incumbent_page (after?)
SELECT i FROM pse_ops.incumbents AS i
WHERE i.attempt_id = :attempt_id::pse_ops.attempt_id
  AND i.seq > coalesce(:after::bigint, -1)
ORDER BY i.seq
LIMIT :limit;

--! latest_incumbent
SELECT i FROM pse_ops.incumbents AS i
WHERE i.attempt_id = :attempt_id::pse_ops.attempt_id
ORDER BY i.seq DESC
LIMIT 1;
