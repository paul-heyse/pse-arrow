-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "ports"."instance_id" AS c0,
        "ports"."port_id" AS c1,
        "ports"."port_id" AS c2
    FROM "inferred"."ports" AS "ports"
),
q2 AS (
    SELECT q1.c0, q1.c1, q1.c2 FROM q1
)
SELECT
    q2.c0 AS "ancestor_id",
    q2.c1 AS "entity_id",
    q2.c2 AS "derivation_id"
FROM q2
