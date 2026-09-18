-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "members"."product_id" AS c0,
        "members"."tuple" AS c1,
        "members"."derivation_id" AS c2
    FROM "normalized"."candidate_index_members" AS "members" LEFT ANTI JOIN "inferred"."domain_eligible_members" AS "eligible" ON ("members"."domain_id" = "eligible"."domain_id") AND ("members"."member_id" = "eligible"."member_id")
)
SELECT
    q3.c0 AS "product_id",
    q3.c1 AS "tuple",
    q3.c2 AS "derivation_id"
FROM q3
