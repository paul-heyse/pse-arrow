-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "prospective"."instance_id" AS c0,
        "prospective"."guard_source_id" AS c9,
        "prospective"."guard_node_id" AS c10,
        "prospective"."derivation_id" AS c11
    FROM "normalized"."instance_bindings" AS "prospective" JOIN "normalized"."instance_binding_products" AS "binding_products" ON ("prospective"."instance_id" = "binding_products"."instance_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("binding_products"."product_id" = "valid"."product_id") AND ("binding_products"."index" = "valid"."tuple") JOIN "inferred"."instances" AS "parents" ON ("prospective"."parent_instance_id" = "parents"."instance_id")
),
q7 AS (
    SELECT
        q6.c0 AS c0,
        q6.c11 AS c11
    FROM q6
    WHERE ((q6.c9 IS NULL) IS DISTINCT FROM (q6.c10 IS NULL))
),
q8 AS (
    SELECT
        q7.c0 AS c0,
        'malformed' AS c1,
        q7.c11 AS c2
    FROM q7
),
q9 AS (
    SELECT q8.c0, q8.c1, q8.c2 FROM q8
)
SELECT
    q9.c0 AS "instance_id",
    q9.c1 AS "reason",
    q9.c2 AS "derivation_id"
FROM q9
