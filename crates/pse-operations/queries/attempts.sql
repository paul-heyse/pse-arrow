-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The attempt registry (ADR-0114 Outcome 12): creation, transitions with their audit row,
-- heartbeats and the stale sweep. Legality is the transition table in lifecycle.rs; these
-- statements apply what it allows, under the attempt's row lock.

--! insert_attempt (preparation_identity?, parent_attempt?)
INSERT INTO pse_ops.attempts
    (attempt_id, run_id, kind, request_identity, preparation_identity, state, parent_attempt)
VALUES (:attempt_id, :run_id, :kind, :request_identity, :preparation_identity, :state,
        :parent_attempt);

--! insert_transition (from_state?, actor?, reason?)
INSERT INTO pse_ops.attempt_transitions (attempt_id, seq, from_state, to_state, actor, reason)
VALUES (:attempt_id, :seq, :from_state, :to_state, :actor, :reason);

--! lock_attempt
SELECT state, state_version
FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id
FOR UPDATE;

-- A lease is installed exactly when the attempt enters running; leaving running clears it.
--! update_state (worker?, lease_us?)
UPDATE pse_ops.attempts SET
    state = :state,
    state_version = :state_version,
    updated_at = now(),
    worker = coalesce(:worker, worker),
    lease_expires_at = now() + :lease_us::bigint * interval '1 microsecond',
    heartbeat_at = CASE WHEN :lease_us::bigint IS NULL THEN heartbeat_at ELSE now() END,
    started_at = CASE WHEN :lease_us::bigint IS NULL THEN started_at ELSE now() END,
    finished_at = CASE WHEN :ends_work THEN coalesce(finished_at, now()) ELSE finished_at END
WHERE attempt_id = :attempt_id::pse_ops.attempt_id;

-- Every termination column at once: the class and exactly its typed value (X4).
--! set_termination (termination_native?, termination_run_state?, termination_trajectory?, termination_runtime?, termination_rule?, termination_detail?)
UPDATE pse_ops.attempts SET
    termination_class = :termination_class,
    termination_native = :termination_native,
    termination_run_state = :termination_run_state,
    termination_trajectory = :termination_trajectory,
    termination_runtime = :termination_runtime,
    termination_rule = :termination_rule,
    termination_detail = :termination_detail
WHERE attempt_id = :attempt_id::pse_ops.attempt_id;

--! attempt
SELECT a FROM pse_ops.attempts AS a WHERE a.attempt_id = :attempt_id::pse_ops.attempt_id;

--! heartbeat
UPDATE pse_ops.attempts
SET lease_expires_at = now() + :lease_us::bigint * interval '1 microsecond',
    heartbeat_at = now()
WHERE attempt_id = :attempt_id::pse_ops.attempt_id
  AND worker = :worker
  AND state = :running
  AND lease_expires_at > now()
RETURNING cancel_requested, lease_expires_at;

--! history
SELECT t FROM pse_ops.attempt_transitions AS t
WHERE t.attempt_id = :attempt_id::pse_ops.attempt_id
ORDER BY t.seq;

--! list_attempts (run?)
SELECT a FROM pse_ops.attempts AS a
WHERE (:run::pse_ops.run_id IS NULL OR a.run_id = :run)
  AND (cardinality(:states::pse_ops.attempt_state[]) = 0 OR a.state = ANY(:states))
ORDER BY a.created_at DESC, a.attempt_id DESC
LIMIT :limit;

-- The stale sweep scans the running attempts' partial index by lease expiry.
--! expired_attempts
SELECT attempt_id FROM pse_ops.attempts
WHERE state = 'running' AND lease_expires_at <= now()
ORDER BY lease_expires_at
LIMIT :limit
FOR UPDATE SKIP LOCKED;
