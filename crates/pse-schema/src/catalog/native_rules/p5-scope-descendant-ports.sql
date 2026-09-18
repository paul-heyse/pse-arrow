-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "paths"."ancestor_id" AS c0,
        "ports"."port_id" AS c1,
        "paths"."derivation_id" AS c2
    FROM "inferred"."instance_reachability" AS "paths" JOIN "inferred"."ports" AS "ports" ON ("paths"."descendant_id" = "ports"."instance_id")
),
q4 AS (
    SELECT q3.c0, q3.c1, q3.c2 FROM q3
)
SELECT
    q4.c0 AS "ancestor_id",
    q4.c1 AS "entity_id",
    q4.c2 AS "derivation_id"
FROM q4
