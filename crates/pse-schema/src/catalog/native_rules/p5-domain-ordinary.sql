-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "members"."domain_id" AS c0,
        "members"."member_id" AS c1,
        "members"."member_id" AS c2
    FROM "normalized"."domain_members" AS "members" LEFT ANTI JOIN "normalized"."material_domain_members" AS "material" ON ("members"."domain_id" = "material"."domain_id") AND ("members"."member_id" = "material"."member_id")
),
q4 AS (
    SELECT q3.c0, q3.c1, q3.c2 FROM q3
)
SELECT
    q4.c0 AS "domain_id",
    q4.c1 AS "member_id",
    q4.c2 AS "derivation_id"
FROM q4
