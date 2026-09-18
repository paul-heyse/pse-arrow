-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "scopes"."scope_decl_id" AS c1,
        "scopes"."owner_instance_id" AS c2,
        "scopes"."derivation_id" AS c3
    FROM "inferred"."scope_candidates" AS "scopes"
    WHERE ((("scopes"."owner_instance_id" IS NULL)))
),
q3 AS (
    SELECT q2.c0, q2.c1, q2.c2, q2.c3 FROM q2
)
SELECT
    q3.c0 AS "scope_id",
    q3.c1 AS "scope_decl_id",
    q3.c2 AS "owner_instance_id",
    q3.c3 AS "derivation_id"
FROM q3
