-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q5 AS (
    SELECT
        "connections"."connection_id" AS c0,
        'member_missing_to' AS c1,
        "connections"."connection_id" AS c2
    FROM "normalized"."connections" AS "connections" JOIN "inferred"."port_members" AS "members" ON ("connections"."from_port_id" = "members"."port_id") LEFT ANTI JOIN "inferred"."port_members" AS "other" ON ("connections"."to_port_id" = "other"."port_id") AND ("members"."ordinal" = "other"."ordinal") AND ("members"."symbol_group" = "other"."symbol_group")
),
q6 AS (
    SELECT q5.c0, q5.c1, q5.c2 FROM q5
)
SELECT
    q6.c0 AS "connection_id",
    q6.c1 AS "reason",
    q6.c2 AS "derivation_id"
FROM q6
