-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "candidates"."requirement_id" AS c0,
        "candidates"."selection_id" AS c1,
        "candidates"."method_id" AS c2,
        "candidates"."applicable" AS c3,
        "candidates"."rank" AS c4,
        "candidates"."reason" AS c5,
        "candidates"."derivation_id" AS c6
    FROM "inferred"."property_requirements" AS "required" JOIN "inferred"."potential_method_candidates" AS "candidates" ON ("required"."requirement_id" = "candidates"."requirement_id")
)
SELECT
    q3.c0 AS "requirement_id",
    q3.c1 AS "selection_id",
    q3.c2 AS "method_id",
    q3.c3 AS "applicable",
    q3.c4 AS "rank",
    q3.c5 AS "reason",
    q3.c6 AS "derivation_id"
FROM q3
