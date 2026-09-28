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

-- Incumbents that kept a solution stay with it.
--! delete_finished_incumbents
DELETE FROM pse_ops.incumbents AS i USING pse_ops.attempts AS a
WHERE i.attempt_id = a.attempt_id
  AND i.solution_id IS NULL
  AND a.finished_at IS NOT NULL
  AND a.finished_at < now() - :age_us::bigint * interval '1 microsecond';

--! existing_incumbent_seqs
SELECT seq FROM pse_ops.incumbents
WHERE attempt_id = :attempt_id::pse_ops.attempt_id AND seq = ANY(:seqs::bigint[]);

--! latest_incumbent
SELECT i FROM pse_ops.incumbents AS i
WHERE i.attempt_id = :attempt_id::pse_ops.attempt_id
ORDER BY i.seq DESC
LIMIT 1;
