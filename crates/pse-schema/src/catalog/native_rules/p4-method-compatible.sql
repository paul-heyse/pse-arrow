-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "selections"."selection_id" AS c0,
        "provisions"."method_id" AS c1
    FROM "normalized"."method_selections" AS "selections" JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") JOIN "reference"."method_provisions" AS "provisions" ON ("selections"."method_id" = "provisions"."method_id") AND ("selections"."property_kind_id" = "provisions"."property_kind_id")
    WHERE (((((("selections"."family" = "methods"."family")))) AND (("selections"."property_kind_id" IS NOT NULL))))
),
q11 AS (
    SELECT
        "selections"."selection_id" AS c0,
        "provisions"."method_id" AS c1
    FROM "normalized"."method_selections" AS "selections" JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") JOIN "reference"."method_provisions" AS "provisions" ON ("selections"."method_id" = "provisions"."method_id")
    WHERE (((((("selections"."family" = "methods"."family")))) AND (("selections"."property_kind_id" IS NULL))))
),
q12 AS (
    SELECT q8.c0, q8.c1 FROM q8
    UNION ALL
    SELECT q11.c0, q11.c1 FROM q11
),
q13 AS (
    SELECT
        q12.c0 AS c0,
        q12.c1 AS c1,
        true AS c2,
        'applicable' AS c3,
        q12.c0 AS c4
    FROM q12
),
q14 AS (
    SELECT q13.c0, q13.c1, q13.c2, q13.c3, q13.c4 FROM q13
)
SELECT
    q14.c0 AS "selection_id",
    q14.c1 AS "method_id",
    q14.c2 AS "compatible",
    q14.c3 AS "reason",
    q14.c4 AS "derivation_id"
FROM q14
