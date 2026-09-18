-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "nodes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "left"."entity_id" AS c2,
        ("left"."included" AND "right"."included") AS c3,
        "nodes"."derivation_id" AS c4
    FROM "inferred"."selector_contexts" AS "nodes" JOIN "inferred"."resolved_scopes" AS "scopes" ON ("nodes"."scope_id" = "scopes"."scope_id") JOIN "inferred"."selector_decisions" AS "left" ON ("nodes"."scope_id" = "left"."scope_id") AND ("nodes"."left_node_id" = "left"."node_id") JOIN "inferred"."selector_decisions" AS "right" ON ("nodes"."scope_id" = "right"."scope_id") AND ("nodes"."right_node_id" = "right"."node_id") AND ("left"."entity_id" = "right"."entity_id")
    WHERE ((((("nodes"."op" = 'intersection')))))
),
q9 AS (
    SELECT q8.c0, q8.c1, q8.c2, q8.c3, q8.c4 FROM q8
)
SELECT
    q9.c0 AS "scope_id",
    q9.c1 AS "node_id",
    q9.c2 AS "entity_id",
    q9.c3 AS "included",
    q9.c4 AS "derivation_id"
FROM q9
