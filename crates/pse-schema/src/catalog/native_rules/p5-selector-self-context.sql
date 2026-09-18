-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q4 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "nodes"."op" AS c2,
        "nodes"."left_node_id" AS c3,
        "nodes"."right_node_id" AS c4,
        "scopes"."owner_instance_id" AS c5,
        "nodes"."entity_kind" AS c6,
        "nodes"."constant" AS c7,
        "nodes"."node_id" AS c8
    FROM "inferred"."scope_candidates" AS "scopes" JOIN "normalized"."selector_nodes" AS "nodes" ON ("scopes"."scope_decl_id" = "nodes"."scope_decl_id")
    WHERE ((("nodes"."op" = 'self')))
),
q5 AS (
    SELECT q4.c0, q4.c1, q4.c2, q4.c3, q4.c4, q4.c5, q4.c6, q4.c7, q4.c8 FROM q4
)
SELECT
    q5.c0 AS "scope_id",
    q5.c1 AS "node_id",
    q5.c2 AS "op",
    q5.c3 AS "left_node_id",
    q5.c4 AS "right_node_id",
    q5.c5 AS "target_entity_id",
    q5.c6 AS "target_kind",
    q5.c7 AS "constant",
    q5.c8 AS "derivation_id"
FROM q5
