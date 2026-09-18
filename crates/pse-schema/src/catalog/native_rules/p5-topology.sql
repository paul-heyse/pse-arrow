-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q7 AS (
    SELECT
        "from"."instance_id" AS c0,
        "to"."instance_id" AS c1,
        "connections"."connection_id" AS c2
    FROM "normalized"."connections" AS "connections" JOIN "inferred"."ports" AS "from" ON ("connections"."from_port_id" = "from"."port_id") JOIN "inferred"."ports" AS "to" ON ("connections"."to_port_id" = "to"."port_id") LEFT ANTI JOIN "inferred"."connection_violations" AS "invalid" ON ("connections"."connection_id" = "invalid"."connection_id")
),
q8 AS (
    SELECT q7.c0, q7.c1, q7.c2 FROM q7
)
SELECT
    q8.c0 AS "from_instance_id",
    q8.c1 AS "to_instance_id",
    q8.c2 AS "connection_id"
FROM q8
