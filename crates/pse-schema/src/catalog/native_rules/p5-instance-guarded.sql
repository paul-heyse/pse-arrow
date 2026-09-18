-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "guards"."instance_id" AS c0,
        "guards"."source_id" AS c1,
        "guards"."predicate_id" AS c2
    FROM "inferred"."predicate_outcomes" AS "guards"
    WHERE ((("guards"."outcome" = 'true') AND (array_length("guards"."index") = 0)))
),
q10 AS (
    SELECT
        "prospective"."instance_id" AS c0,
        "prospective"."parent_instance_id" AS c1,
        "prospective"."template_id" AS c2,
        "prospective"."path" AS c3,
        "prospective"."index" AS c4,
        "prospective"."derivation_id" AS c5
    FROM "normalized"."instance_bindings" AS "prospective" JOIN "normalized"."instance_binding_products" AS "binding_products" ON ("prospective"."instance_id" = "binding_products"."instance_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("binding_products"."product_id" = "valid"."product_id") AND ("binding_products"."index" = "valid"."tuple") JOIN "inferred"."instances" AS "parents" ON ("prospective"."parent_instance_id" = "parents"."instance_id") JOIN q8 ON ("prospective"."guard_source_id" = q8.c1) AND ("prospective"."guard_node_id" = q8.c2) AND ("prospective"."parent_instance_id" = q8.c0)
),
q11 AS (
    SELECT q10.c0, q10.c1, q10.c2, q10.c3, q10.c4, q10.c5 FROM q10
)
SELECT
    q11.c0 AS "instance_id",
    q11.c1 AS "parent_instance_id",
    q11.c2 AS "template_id",
    q11.c3 AS "path",
    q11.c4 AS "index",
    q11.c5 AS "derivation_id"
FROM q11
