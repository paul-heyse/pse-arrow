-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "connections"."connection_id" AS c0,
        'from_port_missing' AS c1,
        "connections"."connection_id" AS c2
    FROM "normalized"."connections" AS "connections" LEFT ANTI JOIN "inferred"."ports" AS "ports" ON ("connections"."from_port_id" = "ports"."port_id")
),
q4 AS (
    SELECT q3.c0, q3.c1, q3.c2 FROM q3
)
SELECT
    q4.c0 AS "connection_id",
    q4.c1 AS "reason",
    q4.c2 AS "derivation_id"
FROM q4
