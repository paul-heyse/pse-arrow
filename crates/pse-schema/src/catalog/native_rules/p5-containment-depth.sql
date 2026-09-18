-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "paths"."ancestor_id" AS c0,
        "paths"."descendant_id" AS c1,
        "paths"."descendant_id" AS c2
    FROM "inferred"."instance_reachability" AS "paths"
),
q5 AS (
    SELECT
        "prefix"."ancestor_id" AS c0,
        "suffix"."descendant_id" AS c1,
        "prefix"."descendant_id" AS c2
    FROM "inferred"."instance_reachability" AS "prefix" JOIN "inferred"."instance_reachability" AS "suffix" ON ("prefix"."descendant_id" = "suffix"."ancestor_id")
),
q6 AS (
    SELECT q1.c0, q1.c1, q1.c2 FROM q1
    UNION ALL
    SELECT q5.c0, q5.c1, q5.c2 FROM q5
),
q7 AS (
    SELECT DISTINCT q6.* FROM q6
),
q8 AS (
    SELECT
        q7.c0 AS c0,
        q7.c1 AS c1,
        count(pse_require_nonnull(q7.c2)) AS c2
    FROM q7
    GROUP BY q7.c0, q7.c1
),
q9 AS (
    SELECT
        q8.c0 AS c0,
        q8.c1 AS c1,
        q8.c2 AS c2,
        q8.c0 AS c3
    FROM q8
)
SELECT
    q9.c0 AS "ancestor_id",
    q9.c1 AS "descendant_id",
    q9.c2 AS "depth",
    q9.c3 AS "derivation_id"
FROM q9
