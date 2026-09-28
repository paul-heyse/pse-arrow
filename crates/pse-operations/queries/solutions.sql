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
