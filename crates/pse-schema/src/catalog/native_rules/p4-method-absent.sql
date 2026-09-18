-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "selections"."selection_id" AS c0,
        "selections"."method_id" AS c1,
        false AS c2,
        'missing_provision' AS c3,
        "selections"."selection_id" AS c4
    FROM "normalized"."method_selections" AS "selections" LEFT ANTI JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id")
),
q4 AS (
    SELECT q3.c0, q3.c1, q3.c2, q3.c3, q3.c4 FROM q3
)
SELECT
    q4.c0 AS "selection_id",
    q4.c1 AS "method_id",
    q4.c2 AS "compatible",
    q4.c3 AS "reason",
    q4.c4 AS "derivation_id"
FROM q4
