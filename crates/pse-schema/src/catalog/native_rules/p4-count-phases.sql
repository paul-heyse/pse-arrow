-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q0 AS (
    SELECT
        "materials"."material_system_id" AS c0,
        "materials"."phase_ids" AS c4
    FROM "normalized"."material_systems" AS "materials"
),
q1 AS (
    SELECT
        q0.c0 AS c0,
        unnest(pse_require_nonnull(q0.c4)) AS c6
    FROM q0
),
q2 AS (
    SELECT
        q1.c0 AS c0,
        q1.c6 AS c1
    FROM q1
),
q3 AS (
    SELECT DISTINCT q2.* FROM q2
),
q4 AS (
    SELECT
        q3.c0 AS c0,
        count(1) AS c1
    FROM q3
    GROUP BY q3.c0
),
q5 AS (
    SELECT
        q4.c0 AS c0,
        'phase' AS c1,
        q4.c1 AS c2,
        q4.c0 AS c3
    FROM q4
)
SELECT
    q5.c0 AS "material_system_id",
    q5.c1 AS "kind",
    q5.c2 AS "count",
    q5.c3 AS "derivation_id"
FROM q5
