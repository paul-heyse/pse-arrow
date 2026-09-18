-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q4 AS (
    SELECT
        "connections"."connection_id" AS c0,
        "from"."kind" AS c9,
        "to"."kind" AS c15
    FROM "normalized"."connections" AS "connections" JOIN "inferred"."ports" AS "from" ON ("connections"."from_port_id" = "from"."port_id") JOIN "inferred"."ports" AS "to" ON ("connections"."to_port_id" = "to"."port_id")
),
q5 AS (
    SELECT
        q4.c0 AS c0
    FROM q4
    WHERE (q4.c9 IS DISTINCT FROM q4.c15)
),
q6 AS (
    SELECT
        q5.c0 AS c0,
        'port_kind_mismatch' AS c1,
        q5.c0 AS c2
    FROM q5
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2 FROM q6
)
SELECT
    q7.c0 AS "connection_id",
    q7.c1 AS "reason",
    q7.c2 AS "derivation_id"
FROM q7
