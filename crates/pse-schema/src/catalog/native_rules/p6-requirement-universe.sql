-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "axes"."requirement_id" AS c0
    FROM "inferred"."requirement_key_axes" AS "axes" LEFT ANTI JOIN "inferred"."domain_eligible_members" AS "eligible" ON ("axes"."domain_id" = "eligible"."domain_id") AND ("axes"."member_id" = "eligible"."member_id")
),
q9 AS (
    SELECT
        "keys"."requirement_id" AS c0,
        "keys"."state_scope_id" AS c1,
        "keys"."state_instance_id" AS c2,
        "scopes"."property_package_id" AS c3,
        "keys"."property_kind_id" AS c4,
        "keys"."index" AS c5,
        "keys"."derivation_id" AS c6
    FROM "inferred"."requirement_keys" AS "keys" LEFT ANTI JOIN q3 ON ("keys"."requirement_id" = q3.c0) JOIN "inferred"."valid_index_tuples" AS "tuples" ON ("keys"."product_id" = "tuples"."product_id") AND ("keys"."index" = "tuples"."tuple") JOIN "inferred"."state_scopes" AS "scopes" ON ("keys"."state_scope_id" = "scopes"."state_scope_id") AND ("keys"."state_instance_id" = "scopes"."state_instance_id")
)
SELECT
    q9.c0 AS "requirement_id",
    q9.c1 AS "state_scope_id",
    q9.c2 AS "state_instance_id",
    q9.c3 AS "property_package_id",
    q9.c4 AS "property_kind_id",
    q9.c5 AS "index",
    q9.c6 AS "derivation_id"
FROM q9
