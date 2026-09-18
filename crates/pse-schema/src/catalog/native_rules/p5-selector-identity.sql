-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "nodes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "child"."entity_id" AS c2,
        "child"."included" AS c3,
        "nodes"."derivation_id" AS c4
    FROM "inferred"."selector_contexts" AS "nodes" JOIN "inferred"."resolved_scopes" AS "scopes" ON ("nodes"."scope_id" = "scopes"."scope_id") JOIN "inferred"."selector_decisions" AS "child" ON ("nodes"."scope_id" = "child"."scope_id") AND ("nodes"."left_node_id" = "child"."node_id")
    WHERE (((("nodes"."op" = 'identity'))))
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2, q6.c3, q6.c4 FROM q6
)
SELECT
    q7.c0 AS "scope_id",
    q7.c1 AS "node_id",
    q7.c2 AS "entity_id",
    q7.c3 AS "included",
    q7.c4 AS "derivation_id"
FROM q7
