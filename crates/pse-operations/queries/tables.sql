-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The query surface (Plan 22 O9): paged, filtered reads of the operational relations that
-- DataFusion sessions see under `pse_ops`. Each statement returns whole registry rows in
-- primary-key order after a keyset position, at most `limit` of them, so a scan streams in
-- bounded pages and never holds a connection between them. The typed filters are what a
-- provider pushes down from its query; an empty array or a null bound leaves that column
-- unconstrained. Time bounds are inclusive. A page is read at READ COMMITTED: a scan sees
-- every row that existed throughout it exactly once.

-- -------------------------------------------------------------------- attempts --

--! scan_attempts (after?, created_from?, created_to?)
SELECT a FROM pse_ops.attempts AS a
WHERE (:after::pse_ops.attempt_id IS NULL OR a.attempt_id > :after)
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR a.attempt_id = ANY(:attempts))
  AND (cardinality(:runs::pse_ops.run_id[]) = 0 OR a.run_id = ANY(:runs))
  AND (cardinality(:states::pse_ops.attempt_state[]) = 0 OR a.state = ANY(:states))
  AND (:created_from::timestamptz IS NULL OR a.created_at >= :created_from)
  AND (:created_to::timestamptz IS NULL OR a.created_at <= :created_to)
ORDER BY a.attempt_id
LIMIT :limit;

--! scan_attempt_transitions (after_attempt?, after_seq?, at_from?, at_to?)
SELECT t FROM pse_ops.attempt_transitions AS t
WHERE (:after_attempt::pse_ops.attempt_id IS NULL
       OR (t.attempt_id, t.seq) > (:after_attempt, :after_seq::integer))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR t.attempt_id = ANY(:attempts))
  AND (:at_from::timestamptz IS NULL OR t.at >= :at_from)
  AND (:at_to::timestamptz IS NULL OR t.at <= :at_to)
ORDER BY t.attempt_id, t.seq
LIMIT :limit;

-- ------------------------------------------------------------------------ jobs --

--! scan_jobs (after?, enqueued_from?, enqueued_to?)
SELECT j FROM pse_ops.jobs AS j
WHERE (:after::pse_ops.job_id IS NULL OR j.job_id > :after)
  AND (cardinality(:jobs::pse_ops.job_id[]) = 0 OR j.job_id = ANY(:jobs))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR j.attempt_id = ANY(:attempts))
  AND (cardinality(:states::pse_ops.job_state[]) = 0 OR j.state = ANY(:states))
  AND (:enqueued_from::timestamptz IS NULL OR j.enqueued_at >= :enqueued_from)
  AND (:enqueued_to::timestamptz IS NULL OR j.enqueued_at <= :enqueued_to)
ORDER BY j.job_id
LIMIT :limit;

-- --------------------------------------------------------------------- streams --

--! scan_progress_events (after_attempt?, after_seq?, at_from?, at_to?)
SELECT e FROM pse_ops.progress_events AS e
WHERE (:after_attempt::pse_ops.attempt_id IS NULL
       OR (e.attempt_id, e.seq) > (:after_attempt, :after_seq::bigint))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR e.attempt_id = ANY(:attempts))
  AND (:at_from::timestamptz IS NULL OR e.at >= :at_from)
  AND (:at_to::timestamptz IS NULL OR e.at <= :at_to)
ORDER BY e.attempt_id, e.seq
LIMIT :limit;

--! scan_progress_values (after_attempt?, after_seq?, after_name?)
SELECT v FROM pse_ops.progress_values AS v
WHERE (:after_attempt::pse_ops.attempt_id IS NULL
       OR (v.attempt_id, v.seq, v.name) > (:after_attempt, :after_seq::bigint, :after_name::text))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR v.attempt_id = ANY(:attempts))
ORDER BY v.attempt_id, v.seq, v.name
LIMIT :limit;

--! scan_incumbents (after_attempt?, after_seq?, at_from?, at_to?)
SELECT i FROM pse_ops.incumbents AS i
WHERE (:after_attempt::pse_ops.attempt_id IS NULL
       OR (i.attempt_id, i.seq) > (:after_attempt, :after_seq::bigint))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR i.attempt_id = ANY(:attempts))
  AND (:at_from::timestamptz IS NULL OR i.at >= :at_from)
  AND (:at_to::timestamptz IS NULL OR i.at <= :at_to)
ORDER BY i.attempt_id, i.seq
LIMIT :limit;

-- ------------------------------------------------------------------- solutions --

--! scan_solutions (after?, created_from?, created_to?)
SELECT s FROM pse_ops.solutions AS s
WHERE (:after::pse_ops.solution_id IS NULL OR s.solution_id > :after)
  AND (cardinality(:solutions::pse_ops.solution_id[]) = 0 OR s.solution_id = ANY(:solutions))
  AND (cardinality(:creators::pse_ops.attempt_id[]) = 0 OR s.created_by = ANY(:creators))
  AND (:created_from::timestamptz IS NULL OR s.created_at >= :created_from)
  AND (:created_to::timestamptz IS NULL OR s.created_at <= :created_to)
ORDER BY s.solution_id
LIMIT :limit;

-- --------------------------------------------------------------------- studies --

--! scan_studies (after?, created_from?, created_to?)
SELECT s FROM pse_ops.studies AS s
WHERE (:after::pse_ops.study_id IS NULL OR s.study_id > :after)
  AND (cardinality(:studies::pse_ops.study_id[]) = 0 OR s.study_id = ANY(:studies))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR s.attempt_id = ANY(:attempts))
  AND (cardinality(:states::pse_ops.study_state[]) = 0 OR s.state = ANY(:states))
  AND (:created_from::timestamptz IS NULL OR s.created_at >= :created_from)
  AND (:created_to::timestamptz IS NULL OR s.created_at <= :created_to)
ORDER BY s.study_id
LIMIT :limit;

--! scan_study_points (after_study?, after_index?)
SELECT p FROM pse_ops.study_points AS p
WHERE (:after_study::pse_ops.study_id IS NULL
       OR (p.study_id, p.point_index) > (:after_study, :after_index::integer))
  AND (cardinality(:studies::pse_ops.study_id[]) = 0 OR p.study_id = ANY(:studies))
  AND (cardinality(:jobs::pse_ops.job_id[]) = 0 OR p.job_id = ANY(:jobs))
  AND (cardinality(:states::pse_ops.study_point_state[]) = 0 OR p.state = ANY(:states))
ORDER BY p.study_id, p.point_index
LIMIT :limit;

-- ----------------------------------------------------------------- publication --

--! scan_workspaces (after?)
SELECT w FROM pse_ops.workspaces AS w
WHERE (:after::pse_ops.workspace_id IS NULL OR w.workspace_id > :after)
  AND (cardinality(:workspaces::pse_ops.workspace_id[]) = 0
       OR w.workspace_id = ANY(:workspaces))
ORDER BY w.workspace_id
LIMIT :limit;

--! scan_publications (after?, committed_from?, committed_to?)
SELECT p FROM pse_ops.publications AS p
WHERE (:after::pse_ops.publication_id IS NULL OR p.publication_id > :after)
  AND (cardinality(:publications::pse_ops.publication_id[]) = 0
       OR p.publication_id = ANY(:publications))
  AND (cardinality(:workspaces::pse_ops.workspace_id[]) = 0
       OR p.workspace_id = ANY(:workspaces))
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR p.attempt_id = ANY(:attempts))
  AND (:committed_from::timestamptz IS NULL OR p.committed_at >= :committed_from)
  AND (:committed_to::timestamptz IS NULL OR p.committed_at <= :committed_to)
ORDER BY p.publication_id
LIMIT :limit;

--! scan_publication_members (after_publication?, after_role?, after_catalog?, after_schema?, after_table?)
SELECT m FROM pse_ops.publication_members AS m
WHERE (:after_publication::pse_ops.publication_id IS NULL
       OR (m.publication_id, m.role, m.catalog_name, m.schema_name, m.table_name)
          > (:after_publication, :after_role::pse_ops.publication_member_role,
             :after_catalog::text, :after_schema::text, :after_table::text))
  AND (cardinality(:publications::pse_ops.publication_id[]) = 0
       OR m.publication_id = ANY(:publications))
ORDER BY m.publication_id, m.role, m.catalog_name, m.schema_name, m.table_name
LIMIT :limit;

--! scan_settlements (after?, settled_from?, settled_to?)
SELECT s FROM pse_ops.settlements AS s
WHERE (:after::pse_ops.settlement_id IS NULL OR s.settlement_id > :after)
  AND (cardinality(:attempts::pse_ops.attempt_id[]) = 0 OR s.attempt_id = ANY(:attempts))
  AND (:settled_from::timestamptz IS NULL OR s.settled_at >= :settled_from)
  AND (:settled_to::timestamptz IS NULL OR s.settled_at <= :settled_to)
ORDER BY s.settlement_id
LIMIT :limit;
