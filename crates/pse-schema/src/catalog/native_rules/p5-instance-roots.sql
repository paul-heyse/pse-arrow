-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "prospective"."instance_id" AS c0,
        "prospective"."parent_instance_id" AS c1,
        "prospective"."template_id" AS c2,
        "prospective"."path" AS c3,
        "prospective"."index" AS c4,
        "prospective"."derivation_id" AS c5
    FROM "normalized"."instance_bindings" AS "prospective" JOIN "normalized"."instance_binding_products" AS "binding_products" ON ("prospective"."instance_id" = "binding_products"."instance_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("binding_products"."product_id" = "valid"."product_id") AND ("binding_products"."index" = "valid"."tuple")
    WHERE ((("prospective"."parent_instance_id" IS NULL)))
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2, q6.c3, q6.c4, q6.c5 FROM q6
)
SELECT
    q7.c0 AS "instance_id",
    q7.c1 AS "parent_instance_id",
    q7.c2 AS "template_id",
    q7.c3 AS "path",
    q7.c4 AS "index",
    q7.c5 AS "derivation_id"
FROM q7
