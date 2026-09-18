-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "materials"."material_system_id" AS c0,
        'species' AS c1,
        0 AS c2,
        "materials"."material_system_id" AS c3
    FROM "normalized"."material_systems" AS "materials"
    WHERE (((array_length("materials"."species_ids") = 0)))
),
q3 AS (
    SELECT q2.c0, q2.c1, q2.c2, q2.c3 FROM q2
)
SELECT
    q3.c0 AS "material_system_id",
    q3.c1 AS "kind",
    q3.c2 AS "count",
    q3.c3 AS "derivation_id"
FROM q3
