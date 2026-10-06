-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- Studies coordinated across workers (Plan 22 O7). A study, its coordinating attempt, its
-- publication intent, its finalization job and every point's job are created in one
-- transaction; a point's state follows its job in the transaction that moves the job.
-- Point transitions of one study are serialized by the study row (FOR UPDATE).
--
-- Lock order: job rows, then their attempts, then the study row, then the jobs and
-- attempts of the points it releases or cancels, then the study's own attempt.

--! insert_study
INSERT INTO pse_ops.studies (study_id, attempt_id, publication_id, finalization_job,
    definition, state)
VALUES (:study_id, :attempt_id, :publication_id, :finalization_job, :definition, :state);

--! insert_point
INSERT INTO pse_ops.study_points (study_id, point_index, binding_hash, job_id,
    state, revision, policy, outcome)
VALUES (:study_id, :point_index, :binding_hash, :job_id, :state, 0, :policy, :outcome);

--! insert_point_member (revision_column?, revision_id?)
INSERT INTO pse_ops.study_point_members (study_id, point_index, catalog_name, schema_name,
    table_name, relation_id, relation_version, contract_fingerprint, table_uri,
    delta_version, selection_kind, revision_column, revision_id)
VALUES (:study_id, :point_index, :catalog_name, :schema_name, :table_name, :relation_id,
    :relation_version, :contract_fingerprint, :table_uri, :delta_version, :selection_kind,
    :revision_column, :revision_id);

-- Inventory supersession is admitted by shared policy and exact prior-effect facts.
--! remove_point_members
DELETE FROM pse_ops.study_point_members
WHERE study_id=:study_id::pse_ops.study_id AND point_index=:point_index;

--! study
SELECT s FROM pse_ops.studies AS s WHERE s.study_id = :study_id::pse_ops.study_id;

--! lock_study
SELECT s FROM pse_ops.studies AS s
WHERE s.study_id = :study_id::pse_ops.study_id
FOR UPDATE;

--! study_of_attempt
SELECT s FROM pse_ops.studies AS s WHERE s.attempt_id = :attempt_id::pse_ops.attempt_id;

--! list_studies
SELECT s FROM pse_ops.studies AS s
WHERE cardinality(:states::pse_ops.study_state[]) = 0 OR s.state = ANY(:states)
ORDER BY s.created_at DESC, s.study_id DESC
LIMIT :limit;

--! set_study_state
UPDATE pse_ops.studies SET state = :state, updated_at = now()
WHERE study_id = :study_id::pse_ops.study_id;

-- The point a job runs, if any; read again under the study's lock before it changes.
--! point_of_job
SELECT p FROM pse_ops.study_points AS p WHERE p.job_id = :job_id::pse_ops.job_id;

--! point
SELECT p FROM pse_ops.study_points AS p
WHERE p.study_id = :study_id::pse_ops.study_id AND p.point_index = :point_index;

--! set_point_state
UPDATE pse_ops.study_points SET state = :state, updated_at = now()
WHERE study_id = :study_id::pse_ops.study_id AND point_index = :point_index;

--! set_point_outcome
UPDATE pse_ops.study_points
SET state = :state, outcome = :outcome, revision = revision + 1, updated_at = now()
WHERE study_id = :study_id::pse_ops.study_id AND point_index = :point_index
  AND revision = :expected_revision;

-- Scientific transitions preserve the exact publication ticket in its native row.
--! set_point_facts
UPDATE pse_ops.study_points
SET state = :state,
    outcome = jsonb_set(jsonb_set(outcome, '{outcome}', :outcome::jsonb),
                        '{member_attempt}', :member_attempt::jsonb),
    revision = revision + 1, updated_at = now()
WHERE study_id = :study_id::pse_ops.study_id AND point_index = :point_index
  AND revision = :expected_revision;

--! all_points
SELECT p FROM pse_ops.study_points AS p
WHERE p.study_id = :study_id::pse_ops.study_id
ORDER BY p.point_index;

--! unfinished_points
SELECT count(*) AS unfinished FROM pse_ops.study_points
WHERE study_id = :study_id::pse_ops.study_id AND state IN ('pending', 'assigned');

--! completed_points
SELECT count(*) AS completed FROM pse_ops.study_points
WHERE study_id = :study_id::pse_ops.study_id AND state = 'completed';

-- Every point job of a study, locked in job order: cancellation takes them before the
-- study row, as the job queue does.
--! lock_point_jobs
SELECT j FROM pse_ops.jobs AS j
WHERE j.job_id IN (
    SELECT p.job_id FROM pse_ops.study_points AS p
    WHERE p.study_id = :study_id::pse_ops.study_id)
ORDER BY j.job_id
FOR UPDATE;

-- Each point with its job's state, current attempt and that attempt's state.
--! point_status : (last_error?, termination_detail?)
SELECT p.point_index, p.binding_hash, p.state, p.job_id, p.revision, p.policy, p.outcome,
       j.state AS job_state, j.attempt_id, a.state AS attempt_state, j.last_error,
       COALESCE(a.termination_detail, parent.termination_detail) AS termination_detail, j.tries
FROM pse_ops.study_points AS p
JOIN pse_ops.jobs AS j ON j.job_id = p.job_id
JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id
LEFT JOIN pse_ops.attempts AS parent ON parent.attempt_id = a.parent_attempt
WHERE p.study_id = :study_id::pse_ops.study_id
ORDER BY p.point_index;

-- One point with its job's state, current attempt and that attempt's state.
--! one_point_status : (last_error?, termination_detail?)
SELECT p.point_index, p.binding_hash, p.state, p.job_id, p.revision, p.policy, p.outcome,
       j.state AS job_state, j.attempt_id, a.state AS attempt_state, j.last_error,
       COALESCE(a.termination_detail, parent.termination_detail) AS termination_detail, j.tries
FROM pse_ops.study_points AS p
JOIN pse_ops.jobs AS j ON j.job_id = p.job_id
JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id
LEFT JOIN pse_ops.attempts AS parent ON parent.attempt_id = a.parent_attempt
WHERE p.study_id = :study_id::pse_ops.study_id AND p.point_index = :point_index;

-- Policy consumes scientific facts and fences, never native publication tickets.
--! point_policy_status : (last_error?, termination_detail?)
SELECT p.point_index, p.binding_hash, p.state, p.job_id, p.revision, p.policy,
       p.outcome - 'receipt' AS outcome,
       j.state AS job_state, j.attempt_id, a.state AS attempt_state, j.last_error,
       COALESCE(a.termination_detail, parent.termination_detail) AS termination_detail, j.tries
FROM pse_ops.study_points AS p
JOIN pse_ops.jobs AS j ON j.job_id = p.job_id
JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id
LEFT JOIN pse_ops.attempts AS parent ON parent.attempt_id = a.parent_attempt
WHERE p.study_id = :study_id::pse_ops.study_id
ORDER BY p.point_index;

-- Only stopped, unresolved writes need exact receipt-owner reconciliation.
--! unresolved_point_status : (last_error?, termination_detail?)
SELECT p.point_index, p.binding_hash, p.state, p.job_id, p.revision, p.policy, p.outcome,
       j.state AS job_state, j.attempt_id, a.state AS attempt_state, j.last_error,
       COALESCE(a.termination_detail, parent.termination_detail) AS termination_detail, j.tries
FROM pse_ops.study_points AS p
JOIN pse_ops.jobs AS j ON j.job_id = p.job_id
JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id
LEFT JOIN pse_ops.attempts AS parent ON parent.attempt_id = a.parent_attempt
WHERE p.study_id = :study_id::pse_ops.study_id
  AND j.state <> 'running' AND p.outcome #>> '{outcome,effect}' = 'unknown'
ORDER BY p.point_index;

-- The result members of the completed points, in point and name order.
--! available_members
SELECT m FROM pse_ops.study_point_members AS m
JOIN pse_ops.study_points AS p
  ON p.study_id = m.study_id AND p.point_index = m.point_index
WHERE m.study_id = :study_id::pse_ops.study_id
ORDER BY m.point_index, m.catalog_name, m.schema_name, m.table_name;

-- The native dispatch fence requires current ownership and a live lease.
--! dispatch_owner : (worker?)
SELECT worker FROM pse_ops.attempts
WHERE attempt_id = :attempt_id::pse_ops.attempt_id AND state = 'running'
  AND lease_expires_at > now()
FOR UPDATE;
