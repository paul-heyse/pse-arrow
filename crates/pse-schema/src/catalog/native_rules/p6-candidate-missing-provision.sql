-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q11 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "selections"."selection_id" AS c1,
        "selections"."method_id" AS c2
    FROM "inferred"."requirement_universe" AS "requirements" JOIN "inferred"."selection_inventory" AS "selections" ON ("requirements"."property_package_id" = "selections"."property_package_id") JOIN "inferred"."requirement_scope_keys" AS "scope_keys" ON ("requirements"."requirement_id" = "scope_keys"."requirement_id") AND ("selections"."scope_kind" = "scope_keys"."scope_kind") AND ("selections"."scope_ids" = "scope_keys"."scope_ids") JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") JOIN "reference"."method_provisions" AS "provisions" ON ("selections"."method_id" = "provisions"."method_id") AND ("requirements"."property_kind_id" = "provisions"."property_kind_id")
    WHERE (((((((("selections"."property_kind_id" IS NULL) OR ("requirements"."property_kind_id" = "selections"."property_kind_id")))))) AND (("selections"."family" = "methods"."family"))))
),
q13 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "selections"."selection_id" AS c1,
        "selections"."method_id" AS c2,
        false AS c3,
        NULL AS c4,
        'missing_provision' AS c5,
        "selections"."derivation_id" AS c6
    FROM "inferred"."requirement_universe" AS "requirements" JOIN "inferred"."selection_inventory" AS "selections" ON ("requirements"."property_package_id" = "selections"."property_package_id") JOIN "inferred"."requirement_scope_keys" AS "scope_keys" ON ("requirements"."requirement_id" = "scope_keys"."requirement_id") AND ("selections"."scope_kind" = "scope_keys"."scope_kind") AND ("selections"."scope_ids" = "scope_keys"."scope_ids") JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") LEFT ANTI JOIN q11 ON ("requirements"."requirement_id" = q11.c0) AND ("selections"."selection_id" = q11.c1) AND ("selections"."method_id" = q11.c2)
    WHERE (((((((("selections"."property_kind_id" IS NULL) OR ("requirements"."property_kind_id" = "selections"."property_kind_id")))))) AND (("selections"."family" = "methods"."family"))))
)
SELECT
    q13.c0 AS "requirement_id",
    q13.c1 AS "selection_id",
    q13.c2 AS "method_id",
    q13.c3 AS "applicable",
    q13.c4 AS "rank",
    q13.c5 AS "reason",
    q13.c6 AS "derivation_id"
FROM q13
