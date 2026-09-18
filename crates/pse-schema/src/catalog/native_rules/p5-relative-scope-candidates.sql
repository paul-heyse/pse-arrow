-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q5 AS (
    SELECT
        "pse_named_id"("instances"."instance_id", "coalesce"("concat"('pse:scope:v1:', "coalesce"("encode"("scopes"."scope_id", 'hex'), '')), '')) AS c0,
        "scopes"."scope_id" AS c1,
        "instances"."instance_id" AS c2,
        "scopes"."scope_id" AS c3
    FROM "normalized"."scopes" AS "scopes" JOIN "normalized"."template_scopes" AS "declarations" ON ("scopes"."scope_id" = "declarations"."scope_id") JOIN "normalized"."instance_bindings" AS "instances" ON ("declarations"."template_id" = "instances"."template_id")
),
q6 AS (
    SELECT q5.c0, q5.c1, q5.c2, q5.c3 FROM q5
)
SELECT
    q6.c0 AS "scope_id",
    q6.c1 AS "scope_decl_id",
    q6.c2 AS "owner_instance_id",
    q6.c3 AS "derivation_id"
FROM q6
