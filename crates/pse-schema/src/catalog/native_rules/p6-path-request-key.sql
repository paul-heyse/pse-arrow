-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q5 AS (
    SELECT
        "seed"."demand_id" AS c0,
        "instances"."instance_id" AS c1,
        "pse_named_id"("seed"."demand_id", "coalesce"("concat"('pse:demand-occurrence:v1:', "encode"("instances"."instance_id", 'hex')), '')) AS c2,
        "seed"."demand_id" AS c3
    FROM "normalized"."property_path_demands" AS "seed" JOIN "normalized"."instance_bindings" AS "instances" ON (true = true)
),
q6 AS (
    SELECT q5.c0, q5.c1, q5.c2, q5.c3 FROM q5
)
SELECT
    q6.c0 AS "seed_id",
    q6.c1 AS "requester_instance_id",
    q6.c2 AS "requester_id",
    q6.c3 AS "derivation_id"
FROM q6
