-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "reads"."read_id" AS c0,
        "reads"."seed_id" AS c1,
        "reads"."requester_instance_id" AS c2,
        "seeds"."scope_id" AS c3,
        "reads"."requester_instance_id" AS c4,
        "seeds"."property_kind_id" AS c5,
        "reads"."derivation_id" AS c6
    FROM "inferred"."demand_active_reads" AS "reads" JOIN "normalized"."property_demand_seeds" AS "seeds" ON ("reads"."seed_id" = "seeds"."seed_id")
)
SELECT
    q3.c0 AS "read_id",
    q3.c1 AS "seed_id",
    q3.c2 AS "requester_instance_id",
    q3.c3 AS "scope_decl_id",
    q3.c4 AS "owner_instance_id",
    q3.c5 AS "property_kind_id",
    q3.c6 AS "derivation_id"
FROM q3
