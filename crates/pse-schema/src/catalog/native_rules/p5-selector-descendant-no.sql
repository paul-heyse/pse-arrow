-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q10 AS (
    SELECT
        "nodes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "entities"."entity_id" AS c2,
        false AS c3,
        "nodes"."derivation_id" AS c4
    FROM "inferred"."selector_contexts" AS "nodes" JOIN "inferred"."resolved_scopes" AS "scopes" ON ("nodes"."scope_id" = "scopes"."scope_id") JOIN "inferred"."scope_entities" AS "entities" ON (true = true) LEFT ANTI JOIN "inferred"."scope_reachability" AS "reach" ON ("nodes"."target_entity_id" = "reach"."ancestor_id") AND ("entities"."entity_id" = "reach"."entity_id")
    WHERE (((((("nodes"."op" = 'descendant_of'))))))
),
q11 AS (
    SELECT q10.c0, q10.c1, q10.c2, q10.c3, q10.c4 FROM q10
)
SELECT
    q11.c0 AS "scope_id",
    q11.c1 AS "node_id",
    q11.c2 AS "entity_id",
    q11.c3 AS "included",
    q11.c4 AS "derivation_id"
FROM q11
