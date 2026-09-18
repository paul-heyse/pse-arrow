-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "decisions"."entity_id" AS c1,
        "decisions"."derivation_id" AS c2
    FROM "normalized"."selector_roots" AS "roots" JOIN "inferred"."resolved_scopes" AS "scopes" ON ("roots"."scope_decl_id" = "scopes"."scope_decl_id") JOIN "inferred"."selector_decisions" AS "decisions" ON ("scopes"."scope_id" = "decisions"."scope_id") AND ("roots"."node_id" = "decisions"."node_id")
    WHERE "decisions"."included"
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2 FROM q6
)
SELECT
    q7.c0 AS "scope_id",
    q7.c1 AS "entity_id",
    q7.c2 AS "derivation_id"
FROM q7
