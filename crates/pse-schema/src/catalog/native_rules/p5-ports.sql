-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q9 AS (
    SELECT
        "targets"."port_id" AS c0
    FROM "inferred"."port_state_candidates" AS "targets" JOIN "inferred"."port_state_domain_candidates" AS "domains" ON ("targets"."port_id" = "domains"."port_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("domains"."product_id" = "valid"."product_id") AND ("targets"."state_index" = "valid"."tuple")
),
q10 AS (
    SELECT DISTINCT q9.* FROM q9
),
q14 AS (
    SELECT
        "ports"."port_id" AS c0,
        "ports"."instance_id" AS c1,
        "ports"."name" AS c2,
        "ports"."kind" AS c3,
        "ports"."direction" AS c4,
        "ports"."state_instance_id" AS c5
    FROM "inferred"."port_candidates" AS "ports" JOIN "inferred"."instances" AS "instances" ON ("ports"."instance_id" = "instances"."instance_id") JOIN q10 ON ("ports"."port_id" = q10.c0) LEFT ANTI JOIN "inferred"."unbound_port_targets" AS "missing" ON ("ports"."port_id" = "missing"."port_id")
    WHERE (((((("ports"."guard_outcome" = 'true'))))))
),
q15 AS (
    SELECT q14.c0, q14.c1, q14.c2, q14.c3, q14.c4, q14.c5 FROM q14
)
SELECT
    q15.c0 AS "port_id",
    q15.c1 AS "instance_id",
    q15.c2 AS "name",
    q15.c3 AS "kind",
    q15.c4 AS "direction",
    q15.c5 AS "state_instance_id"
FROM q15
