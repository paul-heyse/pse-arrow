-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q9 AS (
    SELECT
        "obligations"."read_id" AS c0,
        "obligations"."seed_id" AS c1,
        "requesters"."requester_id" AS c2,
        "obligations"."requester_instance_id" AS c3,
        "required"."requirement_id" AS c4,
        "required"."state_scope_id" AS c5,
        "required"."property_kind_id" AS c6,
        "required"."index" AS c7,
        "requesters"."requester_id" AS c8
    FROM "inferred"."demand_obligations" AS "obligations" JOIN "inferred"."scope_members" AS "members" ON ("obligations"."scope_id" = "members"."scope_id") JOIN "inferred"."demand_index_maps" AS "maps" ON ("obligations"."read_id" = "maps"."read_id") JOIN "inferred"."requirement_universe" AS "required" ON ("maps"."requirement_id" = "required"."requirement_id") AND ("members"."entity_id" = "required"."state_instance_id") AND ("obligations"."property_kind_id" = "required"."property_kind_id") JOIN "inferred"."demand_request_keys" AS "requesters" ON ("obligations"."seed_id" = "requesters"."seed_id") AND ("obligations"."requester_instance_id" = "requesters"."requester_instance_id")
)
SELECT
    q9.c0 AS "read_id",
    q9.c1 AS "seed_id",
    q9.c2 AS "requester_id",
    q9.c3 AS "requester_instance_id",
    q9.c4 AS "requirement_id",
    q9.c5 AS "state_scope_id",
    q9.c6 AS "property_kind_id",
    q9.c7 AS "index",
    q9.c8 AS "derivation_id"
FROM q9
