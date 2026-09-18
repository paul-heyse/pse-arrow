-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "nodes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "nodes"."op" AS c2,
        "nodes"."left_node_id" AS c3,
        "nodes"."right_node_id" AS c4,
        "nodes"."derivation_id" AS c8
    FROM "inferred"."selector_contexts" AS "nodes" JOIN "inferred"."resolved_scopes" AS "scopes" ON ("nodes"."scope_id" = "scopes"."scope_id")
),
q3 AS (
    SELECT
        q2.c0 AS c0,
        q2.c1 AS c1,
        q2.c3 AS c3,
        q2.c4 AS c4,
        q2.c8 AS c8
    FROM q2
    WHERE (q2.c2 = 'union')
),
q8 AS (
    SELECT
        q3.c0 AS c0,
        q3.c1 AS c1,
        "left"."entity_id" AS c2,
        ("left"."included" OR "right"."included") AS c3,
        q3.c8 AS c4
    FROM q3 JOIN "inferred"."selector_decisions" AS "left" ON (q3.c0 = "left"."scope_id") AND (q3.c3 = "left"."node_id") JOIN "inferred"."selector_decisions" AS "right" ON (q3.c0 = "right"."scope_id") AND (q3.c4 = "right"."node_id") AND ("left"."entity_id" = "right"."entity_id")
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
