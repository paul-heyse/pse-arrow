-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q4 AS (
    SELECT
        "selections"."selection_id" AS c0,
        "selections"."method_id" AS c1,
        false AS c2,
        'family_mismatch' AS c3,
        "selections"."selection_id" AS c4
    FROM "normalized"."method_selections" AS "selections" JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id")
    WHERE (((NOT ("selections"."family" = "methods"."family"))))
),
q5 AS (
    SELECT q4.c0, q4.c1, q4.c2, q4.c3, q4.c4 FROM q4
)
SELECT
    q5.c0 AS "selection_id",
    q5.c1 AS "method_id",
    q5.c2 AS "compatible",
    q5.c3 AS "reason",
    q5.c4 AS "derivation_id"
FROM q5
