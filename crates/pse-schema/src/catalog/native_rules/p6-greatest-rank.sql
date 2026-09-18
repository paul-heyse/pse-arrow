-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q4 AS (
    SELECT
        "other"."requirement_id" AS c0,
        "other"."rank" AS c4
    FROM "inferred"."potential_method_candidates" AS "other"
    WHERE "other"."applicable"
),
q7 AS (
    SELECT
        "current"."requirement_id" AS c0,
        "current"."selection_id" AS c1,
        "current"."method_id" AS c2
    FROM "inferred"."potential_method_candidates" AS "current" JOIN q4 ON ("current"."requirement_id" = q4.c0)
    WHERE ((("current"."applicable" AND (("current"."rank" IS NOT NULL)))) AND ((q4.c4 > "current"."rank")))
),
q9 AS (
    SELECT
        "current"."requirement_id" AS c0,
        "current"."selection_id" AS c1,
        "current"."method_id" AS c2,
        "current"."rank" AS c3,
        "current"."derivation_id" AS c4
    FROM "inferred"."potential_method_candidates" AS "current" LEFT ANTI JOIN q7 ON ("current"."requirement_id" = q7.c0) AND ("current"."selection_id" = q7.c1) AND ("current"."method_id" = q7.c2)
    WHERE (("current"."applicable" AND (("current"."rank" IS NOT NULL))))
)
SELECT
    q9.c0 AS "requirement_id",
    q9.c1 AS "selection_id",
    q9.c2 AS "method_id",
    q9.c3 AS "rank",
    q9.c4 AS "derivation_id"
FROM q9
