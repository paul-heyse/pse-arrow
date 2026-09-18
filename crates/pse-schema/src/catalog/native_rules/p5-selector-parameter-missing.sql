-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "nodes"."op" AS c2,
        "nodes"."left_node_id" AS c3,
        "nodes"."right_node_id" AS c4,
        NULL AS c5,
        "nodes"."entity_kind" AS c6,
        "nodes"."constant" AS c7,
        "nodes"."node_id" AS c8
    FROM "inferred"."scope_candidates" AS "scopes" JOIN "normalized"."selector_nodes" AS "nodes" ON ("scopes"."scope_decl_id" = "nodes"."scope_decl_id") LEFT ANTI JOIN "inferred"."selector_parameter_targets" AS "targets" ON ("scopes"."scope_id" = "targets"."scope_id") AND ("nodes"."node_id" = "targets"."node_id")
    WHERE (((("nodes"."op" = 'instance_parameter'))))
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2, q6.c3, q6.c4, q6.c5, q6.c6, q6.c7, q6.c8 FROM q6
)
SELECT
    q7.c0 AS "scope_id",
    q7.c1 AS "node_id",
    q7.c2 AS "op",
    q7.c3 AS "left_node_id",
    q7.c4 AS "right_node_id",
    q7.c5 AS "target_entity_id",
    q7.c6 AS "target_kind",
    q7.c7 AS "constant",
    q7.c8 AS "derivation_id"
FROM q7
