-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q4 AS (
    SELECT
        "candidates"."port_id" AS c0,
        "candidates"."ordinal" AS c1,
        "candidates"."domain_ids" AS c2,
        "candidates"."product_id" AS c3,
        "candidates"."derivation_id" AS c4
    FROM "inferred"."port_member_domain_candidates" AS "candidates" JOIN "inferred"."ports" AS "ports" ON ("candidates"."port_id" = "ports"."port_id")
),
q5 AS (
    SELECT q4.c0, q4.c1, q4.c2, q4.c3, q4.c4 FROM q4
)
SELECT
    q5.c0 AS "port_id",
    q5.c1 AS "ordinal",
    q5.c2 AS "domain_ids",
    q5.c3 AS "product_id",
    q5.c4 AS "derivation_id"
FROM q5
