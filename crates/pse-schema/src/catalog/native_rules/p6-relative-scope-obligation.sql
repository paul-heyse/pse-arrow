-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "requests"."read_id" AS c0,
        "requests"."seed_id" AS c1,
        "requests"."requester_instance_id" AS c2,
        "scopes"."scope_id" AS c3,
        "requests"."property_kind_id" AS c4,
        "requests"."derivation_id" AS c5
    FROM "inferred"."demand_scope_requests" AS "requests" JOIN "inferred"."scope_bindings" AS "scopes" ON ("requests"."scope_decl_id" = "scopes"."scope_decl_id") AND ("requests"."owner_instance_id" = "scopes"."owner_instance_id")
)
SELECT
    q3.c0 AS "read_id",
    q3.c1 AS "seed_id",
    q3.c2 AS "requester_instance_id",
    q3.c3 AS "scope_id",
    q3.c4 AS "property_kind_id",
    q3.c5 AS "derivation_id"
FROM q3
