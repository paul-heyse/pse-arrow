-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "selections"."selection_id" AS c1,
        "selections"."method_id" AS c2
    FROM "inferred"."requirement_universe" AS "requirements" JOIN "inferred"."selection_inventory" AS "selections" ON ("requirements"."property_package_id" = "selections"."property_package_id") JOIN "inferred"."requirement_scope_keys" AS "scope_keys" ON ("requirements"."requirement_id" = "scope_keys"."requirement_id") AND ("selections"."scope_kind" = "scope_keys"."scope_kind") AND ("selections"."scope_ids" = "scope_keys"."scope_ids")
    WHERE ((((("selections"."property_kind_id" IS NULL) OR ("requirements"."property_kind_id" = "selections"."property_kind_id")))))
),
q8 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "selections"."selection_id" AS c1,
        "selections"."method_id" AS c2,
        false AS c3,
        NULL AS c4,
        'scope_mismatch' AS c5,
        "selections"."derivation_id" AS c6
    FROM "inferred"."requirement_universe" AS "requirements" JOIN "inferred"."selection_inventory" AS "selections" ON ("requirements"."property_package_id" = "selections"."property_package_id") LEFT ANTI JOIN q6 ON ("requirements"."requirement_id" = q6.c0) AND ("selections"."selection_id" = q6.c1) AND ("selections"."method_id" = q6.c2)
    WHERE ((((("selections"."property_kind_id" IS NULL) OR ("requirements"."property_kind_id" = "selections"."property_kind_id")))))
)
SELECT
    q8.c0 AS "requirement_id",
    q8.c1 AS "selection_id",
    q8.c2 AS "method_id",
    q8.c3 AS "applicable",
    q8.c4 AS "rank",
    q8.c5 AS "reason",
    q8.c6 AS "derivation_id"
FROM q8
