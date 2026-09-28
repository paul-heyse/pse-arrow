-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
--
-- The solution and warm-start store (ADR-0114 Outcome 17), keyed by the coordinate
-- compatibility stamp and the preparation identity. Which vectors a kind carries is a
-- registry row check of the table.

--! insert_solution (primal?, lower_bound_duals?, upper_bound_duals?, column_duals?, row_duals?, barrier?, basis_columns?, basis_rows?, created_by?)
INSERT INTO pse_ops.solutions AS s (solution_id, compatibility_stamp, preparation_identity, kind,
    backend, profile_stamp, data_stamp, primal, lower_bound_duals, upper_bound_duals,
    column_duals, row_duals, barrier, basis_columns, basis_rows, created_by)
VALUES (:solution_id, :compatibility_stamp, :preparation_identity, :kind, :backend,
    :profile_stamp, :data_stamp, :primal, :lower_bound_duals, :upper_bound_duals,
    :column_duals, :row_duals, :barrier, :basis_columns, :basis_rows, :created_by)
RETURNING s;

--! solution
SELECT s FROM pse_ops.solutions AS s WHERE s.solution_id = :solution_id::pse_ops.solution_id;

-- The warm-start lookup: the newest compatible seed of a backend.
--! latest_compatible
SELECT s FROM pse_ops.solutions AS s
WHERE s.compatibility_stamp = :compatibility_stamp::pse_ops.content_hash
  AND s.preparation_identity = :preparation_identity::pse_ops.content_hash
  AND s.backend = :backend
ORDER BY s.created_at DESC, s.solution_id DESC
LIMIT 1;

-- The resume lookup (Plan 22 G8): the newest compatible solution that an incumbent of the
-- attempt or of one of its ancestors (following parent_attempt) references. The nearest
-- attempt of the chain wins, then its latest incumbent.
--! latest_in_attempt_chain
WITH RECURSIVE chain (attempt_id, depth) AS (
    SELECT a.attempt_id, 0 FROM pse_ops.attempts AS a
    WHERE a.attempt_id = :attempt_id::pse_ops.attempt_id
  UNION ALL
    SELECT a.parent_attempt, chain.depth + 1
    FROM chain JOIN pse_ops.attempts AS a ON a.attempt_id = chain.attempt_id
    WHERE a.parent_attempt IS NOT NULL AND chain.depth < 4096
)
SELECT s FROM chain
JOIN pse_ops.incumbents AS i ON i.attempt_id = chain.attempt_id
JOIN pse_ops.solutions AS s ON s.solution_id = i.solution_id
WHERE s.compatibility_stamp = :compatibility_stamp::pse_ops.content_hash
  AND s.preparation_identity = :preparation_identity::pse_ops.content_hash
  AND s.backend = :backend
ORDER BY chain.depth, i.seq DESC
LIMIT 1;

-- The seed of a study point's successor (Plan 22 O7): the newest compatible solution an
-- attempt stored, its accepted output seed or a captured incumbent.
--! latest_of_attempt
SELECT s FROM pse_ops.solutions AS s
WHERE s.created_by = :attempt_id::pse_ops.attempt_id
  AND s.compatibility_stamp = :compatibility_stamp::pse_ops.content_hash
  AND s.preparation_identity = :preparation_identity::pse_ops.content_hash
  AND s.backend = :backend
ORDER BY s.created_at DESC, s.solution_id DESC
LIMIT 1;
