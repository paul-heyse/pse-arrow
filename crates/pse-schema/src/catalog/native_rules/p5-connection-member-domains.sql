-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q8 AS (
    SELECT
        "connections"."connection_id" AS c0,
        "left_domains"."domain_ids" AS c20,
        "right_domains"."domain_ids" AS c25
    FROM "normalized"."connections" AS "connections" JOIN "inferred"."port_members" AS "members" ON ("connections"."from_port_id" = "members"."port_id") JOIN "inferred"."port_members" AS "other" ON ("connections"."to_port_id" = "other"."port_id") AND ("members"."ordinal" = "other"."ordinal") AND ("members"."symbol_group" = "other"."symbol_group") JOIN "inferred"."port_member_domains" AS "left_domains" ON ("members"."port_id" = "left_domains"."port_id") AND ("members"."ordinal" = "left_domains"."ordinal") JOIN "inferred"."port_member_domains" AS "right_domains" ON ("other"."port_id" = "right_domains"."port_id") AND ("other"."ordinal" = "right_domains"."ordinal")
),
q9 AS (
    SELECT
        q8.c0 AS c0
    FROM q8
    WHERE (q8.c20 IS DISTINCT FROM q8.c25)
),
q10 AS (
    SELECT
        q9.c0 AS c0,
        'member_domain_mismatch' AS c1,
        q9.c0 AS c2
    FROM q9
),
q11 AS (
    SELECT q10.c0, q10.c1, q10.c2 FROM q10
)
SELECT
    q11.c0 AS "connection_id",
    q11.c1 AS "reason",
    q11.c2 AS "derivation_id"
FROM q11
