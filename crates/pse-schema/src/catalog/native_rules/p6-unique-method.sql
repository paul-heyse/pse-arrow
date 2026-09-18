-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "winners"."requirement_id" AS c0,
        "winners"."method_id" AS c1
    FROM "inferred"."potential_method_winners" AS "winners"
),
q2 AS (
    SELECT DISTINCT q1.* FROM q1
),
q3 AS (
    SELECT
        q2.c0 AS c0,
        count(1) AS c1
    FROM q2
    GROUP BY q2.c0
),
q8 AS (
    SELECT
        "winners"."requirement_id" AS c0,
        "pse_checked_value"("named_struct"('kind', 'resolved', 'resolved', "named_struct"('method_id', "methods"."method_id")), 'inferred.method_resolutions', 'outcome') AS c1,
        "winners"."requirement_id" AS c2
    FROM q3 JOIN "inferred"."potential_method_winners" AS "winners" ON (q3.c0 = "winners"."requirement_id") JOIN "reference"."method_specs" AS "methods" ON ("winners"."method_id" = "methods"."method_id")
    WHERE ((((q3.c1 = 1))))
)
SELECT
    q8.c0 AS "requirement_id",
    q8.c1 AS "outcome",
    q8.c2 AS "derivation_id"
FROM q8
