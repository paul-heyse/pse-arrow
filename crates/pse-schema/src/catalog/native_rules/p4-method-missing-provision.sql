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
q14 AS (
    SELECT
        "selections"."selection_id" AS c0,
        "selections"."method_id" AS c1,
        false AS c2,
        'missing_provision' AS c3,
        "selections"."selection_id" AS c4
    FROM "normalized"."method_selections" AS "selections" JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") LEFT ANTI JOIN q12 ON ("selections"."selection_id" = q12.c0) AND ("selections"."method_id" = q12.c1)
    WHERE ((((("selections"."family" = "methods"."family")))))
),
q15 AS (
    SELECT q14.c0, q14.c1, q14.c2, q14.c3, q14.c4 FROM q14
)
SELECT
    q15.c0 AS "selection_id",
    q15.c1 AS "method_id",
    q15.c2 AS "compatible",
    q15.c3 AS "reason",
    q15.c4 AS "derivation_id"
FROM q15
