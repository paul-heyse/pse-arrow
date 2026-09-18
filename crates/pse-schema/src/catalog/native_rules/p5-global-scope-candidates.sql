-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "scopes"."scope_id" AS c1,
        NULL AS c2,
        "scopes"."scope_id" AS c3
    FROM "normalized"."scopes" AS "scopes" LEFT ANTI JOIN "normalized"."template_scopes" AS "declarations" ON ("scopes"."scope_id" = "declarations"."scope_id")
),
q4 AS (
    SELECT q3.c0, q3.c1, q3.c2, q3.c3 FROM q3
)
SELECT
    q4.c0 AS "scope_id",
    q4.c1 AS "scope_decl_id",
    q4.c2 AS "owner_instance_id",
    q4.c3 AS "derivation_id"
FROM q4
