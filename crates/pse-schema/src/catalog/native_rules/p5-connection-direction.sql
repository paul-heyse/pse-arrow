-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "connections"."connection_id" AS c0,
        'direction_mismatch' AS c1,
        "connections"."connection_id" AS c2
    FROM "normalized"."connections" AS "connections" JOIN "inferred"."ports" AS "from" ON ("connections"."from_port_id" = "from"."port_id") JOIN "inferred"."ports" AS "to" ON ("connections"."to_port_id" = "to"."port_id")
    WHERE (((NOT ((("from"."direction" = 'outlet') OR ("from"."direction" = 'bidirectional')) AND (("to"."direction" = 'inlet') OR ("to"."direction" = 'bidirectional'))))))
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2 FROM q6
)
SELECT
    q7.c0 AS "connection_id",
    q7.c1 AS "reason",
    q7.c2 AS "derivation_id"
FROM q7
