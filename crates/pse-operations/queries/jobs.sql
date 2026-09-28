-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The durable job queue (ADR-0114 Outcome 14): idempotent enqueue, SKIP LOCKED claims,
-- completion and requeue as a new attempt. Lock order is job row, then attempt row.

--! job_by_key
SELECT j FROM pse_ops.jobs AS j WHERE j.idempotency_key = :idempotency_key;

--! insert_job
INSERT INTO pse_ops.jobs (job_id, attempt_id, idempotency_key, payload_version, payload,
    priority, state, max_tries, backoff_base_us, backoff_cap_us)
VALUES (:job_id, :attempt_id, :idempotency_key, :payload_version, :payload, :priority, :state,
    :max_tries, :backoff_base_us, :backoff_cap_us)
ON CONFLICT (idempotency_key) DO NOTHING
RETURNING true AS created;

--! job
SELECT j FROM pse_ops.jobs AS j WHERE j.job_id = :job_id::pse_ops.job_id;

--! lock_job
SELECT j FROM pse_ops.jobs AS j WHERE j.job_id = :job_id::pse_ops.job_id FOR UPDATE;

--! lock_job_of_attempt
SELECT j FROM pse_ops.jobs AS j
WHERE j.attempt_id = :attempt_id::pse_ops.attempt_id
FOR UPDATE;

--! set_job_state (last_error?)
UPDATE pse_ops.jobs
SET state = :state, last_error = coalesce(:last_error, last_error), updated_at = now()
WHERE job_id = :job_id::pse_ops.job_id;

--! requeue_job
UPDATE pse_ops.jobs
SET attempt_id = :attempt_id,
    state = :state,
    available_at = now() + :delay_us::bigint * interval '1 microsecond',
    last_error = :last_error,
    updated_at = now()
WHERE job_id = :job_id::pse_ops.job_id
RETURNING available_at;

-- The claim reads the queued jobs' partial index: highest priority, then oldest
-- availability; rows another claimer locked are skipped, never waited for.
--! claim_job
WITH next AS (
    SELECT job_id FROM pse_ops.jobs
    WHERE state = 'queued' AND available_at <= now()
    ORDER BY priority DESC, available_at, job_id
    LIMIT 1
    FOR UPDATE SKIP LOCKED
)
UPDATE pse_ops.jobs AS j
SET state = :running, tries = j.tries + 1, updated_at = now()
FROM next
WHERE j.job_id = next.job_id
RETURNING j;

--! running_owner : (worker?)
SELECT worker FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id AND state = :running
FOR UPDATE;

-- The stale sweep: running jobs whose attempt went stale or whose lease expired.
--! expired_jobs
SELECT j.job_id FROM pse_ops.jobs AS j
JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id
WHERE j.state = 'running'
  AND (a.state = 'stale' OR (a.state = 'running' AND a.lease_expires_at <= now()))
ORDER BY j.job_id
LIMIT :limit
FOR UPDATE OF j SKIP LOCKED;
