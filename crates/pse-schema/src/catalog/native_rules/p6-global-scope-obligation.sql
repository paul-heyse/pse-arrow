-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "scopes"."scope_decl_id" AS c1
    FROM "inferred"."resolved_scopes" AS "scopes"
    WHERE (("scopes"."owner_instance_id" IS NULL))
),
q4 AS (
    SELECT
        "requests"."read_id" AS c0,
        "requests"."seed_id" AS c1,
        "requests"."requester_instance_id" AS c2,
        q2.c0 AS c3,
        "requests"."property_kind_id" AS c4,
        "requests"."derivation_id" AS c5
    FROM "inferred"."demand_scope_requests" AS "requests" JOIN q2 ON ("requests"."scope_decl_id" = q2.c1)
)
SELECT
    q4.c0 AS "read_id",
    q4.c1 AS "seed_id",
    q4.c2 AS "requester_instance_id",
    q4.c3 AS "scope_id",
    q4.c4 AS "property_kind_id",
    q4.c5 AS "derivation_id"
FROM q4
