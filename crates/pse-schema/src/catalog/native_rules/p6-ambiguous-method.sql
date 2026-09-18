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
q5 AS (
    SELECT
        q3.c0 AS c0,
        "pse_checked_value"("named_struct"('kind', 'ambiguous', 'resolved', "nullif"("named_struct"('method_id', q3.c0), "named_struct"('method_id', q3.c0))), 'inferred.method_resolutions', 'outcome') AS c1,
        q3.c0 AS c2
    FROM q3
    WHERE ((q3.c1 > 1))
)
SELECT
    q5.c0 AS "requirement_id",
    q5.c1 AS "outcome",
    q5.c2 AS "derivation_id"
FROM q5
