-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "reads"."read_id" AS c0,
        "reads"."seed_id" AS c1,
        "reads"."requester_instance_id" AS c2,
        "reads"."owner_instance_id" AS c3,
        "reads"."symbol_decl_id" AS c4,
        "reads"."index" AS c5,
        "reads"."guard_source_id" AS c6,
        "reads"."guard_node_id" AS c7,
        "reads"."guard_index" AS c8,
        "reads"."outer_guard_source_id" AS c9,
        "reads"."outer_guard_node_id" AS c10,
        "reads"."derivation_id" AS c11
    FROM "inferred"."demand_read_keys" AS "reads"
    WHERE (((("reads"."guard_source_id" IS NULL) AND ("reads"."guard_node_id" IS NULL))))
),
q6 AS (
    SELECT
        "reads"."read_id" AS c0,
        "reads"."seed_id" AS c1,
        "reads"."requester_instance_id" AS c2,
        "reads"."owner_instance_id" AS c3,
        "reads"."symbol_decl_id" AS c4,
        "reads"."index" AS c5,
        "reads"."guard_source_id" AS c6,
        "reads"."guard_node_id" AS c7,
        "reads"."guard_index" AS c8,
        "reads"."outer_guard_source_id" AS c9,
        "reads"."outer_guard_node_id" AS c10,
        "reads"."derivation_id" AS c11
    FROM "inferred"."demand_read_keys" AS "reads" JOIN "inferred"."predicate_outcomes" AS "guards" ON ("reads"."requester_instance_id" = "guards"."instance_id") AND ("reads"."guard_source_id" = "guards"."source_id") AND ("reads"."guard_node_id" = "guards"."predicate_id") AND ("reads"."guard_index" = "guards"."index")
    WHERE ((("guards"."outcome" IN ('true', 'unknown'))))
),
q7 AS (
    SELECT q2.c0, q2.c1, q2.c2, q2.c3, q2.c4, q2.c5, q2.c6, q2.c7, q2.c8, q2.c9, q2.c10, q2.c11 FROM q2
    UNION ALL
    SELECT q6.c0, q6.c1, q6.c2, q6.c3, q6.c4, q6.c5, q6.c6, q6.c7, q6.c8, q6.c9, q6.c10, q6.c11 FROM q6
),
q8 AS (
    SELECT
        q7.c0 AS c0,
        q7.c1 AS c1,
        q7.c2 AS c2,
        q7.c3 AS c3,
        q7.c4 AS c4,
        q7.c5 AS c5,
        q7.c6 AS c6,
        q7.c7 AS c7,
        q7.c8 AS c8,
        q7.c9 AS c9,
        q7.c10 AS c10,
        q7.c11 AS c11
    FROM q7
    WHERE ((q7.c9 IS NULL) AND (q7.c10 IS NULL))
),
q11 AS (
    SELECT
        "outer_guard"."instance_id" AS c0,
        "outer_guard"."source_id" AS c1,
        "outer_guard"."predicate_id" AS c2
    FROM "inferred"."predicate_outcomes" AS "outer_guard"
    WHERE ((((array_length("outer_guard"."index") = 0) AND ("outer_guard"."outcome" IN ('true', 'unknown')))))
),
q13 AS (
    SELECT
        q7.c0 AS c0,
        q7.c1 AS c1,
        q7.c2 AS c2,
        q7.c3 AS c3,
        q7.c4 AS c4,
        q7.c5 AS c5,
        q7.c6 AS c6,
        q7.c7 AS c7,
        q7.c8 AS c8,
        q7.c9 AS c9,
        q7.c10 AS c10,
        q7.c11 AS c11
    FROM q7 JOIN q11 ON (q7.c2 = q11.c0) AND (q7.c9 = q11.c1) AND (q7.c10 = q11.c2)
),
q14 AS (
    SELECT q8.c0, q8.c1, q8.c2, q8.c3, q8.c4, q8.c5, q8.c6, q8.c7, q8.c8, q8.c9, q8.c10, q8.c11 FROM q8
    UNION ALL
    SELECT q13.c0, q13.c1, q13.c2, q13.c3, q13.c4, q13.c5, q13.c6, q13.c7, q13.c8, q13.c9, q13.c10, q13.c11 FROM q13
)
SELECT
    q14.c0 AS "read_id",
    q14.c1 AS "seed_id",
    q14.c2 AS "requester_instance_id",
    q14.c3 AS "owner_instance_id",
    q14.c4 AS "symbol_decl_id",
    q14.c5 AS "index",
    q14.c6 AS "guard_source_id",
    q14.c7 AS "guard_node_id",
    q14.c8 AS "guard_index",
    q14.c9 AS "outer_guard_source_id",
    q14.c10 AS "outer_guard_node_id",
    q14.c11 AS "derivation_id"
FROM q14
