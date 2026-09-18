-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q5 AS (
    SELECT
        "reads"."read_id" AS c0,
        "reads"."seed_id" AS c1,
        "reads"."requester_instance_id" AS c2,
        "properties"."scope_selector_id" AS c3,
        "reads"."owner_instance_id" AS c4,
        "properties"."property_kind_id" AS c5,
        "reads"."derivation_id" AS c6
    FROM "inferred"."demand_active_reads" AS "reads" JOIN "normalized"."property_path_demands" AS "demands" ON ("reads"."seed_id" = "demands"."demand_id") JOIN "normalized"."template_symbol_properties" AS "properties" ON ("reads"."symbol_decl_id" = "properties"."symbol_decl_id")
)
SELECT
    q5.c0 AS "read_id",
    q5.c1 AS "seed_id",
    q5.c2 AS "requester_instance_id",
    q5.c3 AS "scope_decl_id",
    q5.c4 AS "owner_instance_id",
    q5.c5 AS "property_kind_id",
    q5.c6 AS "derivation_id"
FROM q5
