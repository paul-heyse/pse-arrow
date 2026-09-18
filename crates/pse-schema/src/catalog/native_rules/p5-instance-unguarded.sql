-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "prospective"."instance_id" AS c0,
        "prospective"."parent_instance_id" AS c1,
        "prospective"."template_id" AS c2,
        "prospective"."path" AS c3,
        "prospective"."index" AS c4,
        "prospective"."derivation_id" AS c5
    FROM "normalized"."instance_bindings" AS "prospective" JOIN "normalized"."instance_binding_products" AS "binding_products" ON ("prospective"."instance_id" = "binding_products"."instance_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("binding_products"."product_id" = "valid"."product_id") AND ("binding_products"."index" = "valid"."tuple") JOIN "inferred"."instances" AS "parents" ON ("prospective"."parent_instance_id" = "parents"."instance_id")
    WHERE (((("prospective"."guard_source_id" IS NULL) AND ("prospective"."guard_node_id" IS NULL))))
),
q9 AS (
    SELECT q8.c0, q8.c1, q8.c2, q8.c3, q8.c4, q8.c5 FROM q8
)
SELECT
    q9.c0 AS "instance_id",
    q9.c1 AS "parent_instance_id",
    q9.c2 AS "template_id",
    q9.c3 AS "path",
    q9.c4 AS "index",
    q9.c5 AS "derivation_id"
FROM q9
