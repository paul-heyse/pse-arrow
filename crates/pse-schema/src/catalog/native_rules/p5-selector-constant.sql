-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "nodes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "entities"."entity_id" AS c2,
        ("nodes"."constant" IS TRUE) AS c3,
        "nodes"."derivation_id" AS c4
    FROM "inferred"."selector_contexts" AS "nodes" JOIN "inferred"."resolved_scopes" AS "scopes" ON ("nodes"."scope_id" = "scopes"."scope_id") JOIN "inferred"."scope_entities" AS "entities" ON (true = true)
    WHERE ((((("nodes"."op" = 'constant')))))
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
