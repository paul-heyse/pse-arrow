-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "scopes"."scope_decl_id" AS c1,
        "scopes"."owner_instance_id" AS c2,
        "scopes"."derivation_id" AS c3
    FROM "inferred"."scope_candidates" AS "scopes" JOIN "inferred"."instances" AS "instances" ON ("scopes"."owner_instance_id" = "instances"."instance_id")
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
