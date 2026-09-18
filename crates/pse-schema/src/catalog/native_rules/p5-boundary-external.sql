-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "from"."scope_id" AS c0,
        "connections"."connection_id" AS c1,
        'external' AS c2,
        "from"."derivation_id" AS c3
    FROM "inferred"."topology_edges" AS "edges" JOIN "normalized"."connections" AS "connections" ON ("edges"."connection_id" = "connections"."connection_id") JOIN "inferred"."scope_port_decisions" AS "from" ON ("connections"."from_port_id" = "from"."port_id") JOIN "inferred"."scope_port_decisions" AS "to" ON ("connections"."to_port_id" = "to"."port_id") AND ("from"."scope_id" = "to"."scope_id") AND ("from"."state_index" = "to"."state_index")
    WHERE (((("from"."included" = false) AND ("to"."included" = false))))
),
q9 AS (
    SELECT q8.c0, q8.c1, q8.c2, q8.c3 FROM q8
)
SELECT
    q9.c0 AS "scope_id",
    q9.c1 AS "connection_id",
    q9.c2 AS "classification",
    q9.c3 AS "derivation_id"
FROM q9
