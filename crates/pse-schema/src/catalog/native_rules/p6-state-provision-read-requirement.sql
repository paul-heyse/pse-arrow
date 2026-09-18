-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q14 AS (
    SELECT
        "reads"."read_id" AS c0,
        "reads"."seed_id" AS c1,
        "requesters"."requester_id" AS c2,
        "reads"."requester_instance_id" AS c3,
        "required"."requirement_id" AS c4,
        "required"."state_scope_id" AS c5,
        "required"."property_kind_id" AS c6,
        "required"."index" AS c7,
        "requesters"."requester_id" AS c8
    FROM "inferred"."demand_active_reads" AS "reads" JOIN "normalized"."property_path_demands" AS "demands" ON ("reads"."seed_id" = "demands"."demand_id") JOIN "reference"."method_provisions" AS "provisions" ON ("reads"."symbol_decl_id" = (get_field(get_field("provisions"."output", 'template_symbol'), 'symbol_decl_id'))) JOIN "inferred"."demand_index_maps" AS "maps" ON ("reads"."read_id" = "maps"."read_id") JOIN "inferred"."requirement_universe" AS "required" ON ("maps"."requirement_id" = "required"."requirement_id") AND ("reads"."owner_instance_id" = "required"."state_instance_id") AND ("provisions"."property_kind_id" = "required"."property_kind_id") JOIN "normalized"."property_packages" AS "packages" ON ("required"."property_package_id" = "packages"."property_package_id") AND ("provisions"."method_id" = "packages"."state_definition_method_id") JOIN "inferred"."demand_request_keys" AS "requesters" ON ("reads"."seed_id" = "requesters"."seed_id") AND ("reads"."requester_instance_id" = "requesters"."requester_instance_id")
)
SELECT
    q14.c0 AS "read_id",
    q14.c1 AS "seed_id",
    q14.c2 AS "requester_id",
    q14.c3 AS "requester_instance_id",
    q14.c4 AS "requirement_id",
    q14.c5 AS "state_scope_id",
    q14.c6 AS "property_kind_id",
    q14.c7 AS "index",
    q14.c8 AS "derivation_id"
FROM q14
