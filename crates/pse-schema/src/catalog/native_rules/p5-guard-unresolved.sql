-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q9 AS (
    SELECT
        "guards"."instance_id" AS c0,
        "guards"."source_id" AS c1,
        "guards"."predicate_id" AS c2,
        "guards"."outcome" AS c4
    FROM "inferred"."predicate_outcomes" AS "guards"
    WHERE ((array_length("guards"."index") = 0))
),
q12 AS (
    SELECT
        "prospective"."instance_id" AS c0,
        'unresolved' AS c1,
        "prospective"."derivation_id" AS c2
    FROM "normalized"."instance_bindings" AS "prospective" JOIN "normalized"."instance_binding_products" AS "binding_products" ON ("prospective"."instance_id" = "binding_products"."instance_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("binding_products"."product_id" = "valid"."product_id") AND ("binding_products"."index" = "valid"."tuple") JOIN "inferred"."instances" AS "parents" ON ("prospective"."parent_instance_id" = "parents"."instance_id") JOIN q9 ON ("prospective"."parent_instance_id" = q9.c0) AND ("prospective"."guard_source_id" = q9.c1) AND ("prospective"."guard_node_id" = q9.c2)
    WHERE (((((("prospective"."guard_source_id" IS NOT NULL) AND ("prospective"."guard_node_id" IS NOT NULL))))) AND (((NOT (q9.c4 = 'true')) AND (NOT (q9.c4 = 'false')))))
),
q13 AS (
    SELECT q12.c0, q12.c1, q12.c2 FROM q12
)
SELECT
    q13.c0 AS "instance_id",
    q13.c1 AS "reason",
    q13.c2 AS "derivation_id"
FROM q13
