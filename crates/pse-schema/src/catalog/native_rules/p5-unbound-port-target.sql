-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "ports"."port_id" AS c0
    FROM "inferred"."port_candidates" AS "ports"
    WHERE (("ports"."guard_outcome" = 'true'))
),
q10 AS (
    SELECT
        "targets"."port_id" AS c0,
        "targets"."state_index" AS c1,
        "targets"."state_instance_id" AS c2,
        "targets"."derivation_id" AS c3
    FROM "inferred"."port_state_candidates" AS "targets" JOIN "inferred"."port_state_domain_candidates" AS "domains" ON ("targets"."port_id" = "domains"."port_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("domains"."product_id" = "valid"."product_id") AND ("targets"."state_index" = "valid"."tuple") JOIN q6 ON ("targets"."port_id" = q6.c0) LEFT ANTI JOIN "inferred"."instances" AS "instances" ON ("targets"."state_instance_id" = "instances"."instance_id")
),
q11 AS (
    SELECT q10.c0, q10.c1, q10.c2, q10.c3 FROM q10
)
SELECT
    q11.c0 AS "port_id",
    q11.c1 AS "state_index",
    q11.c2 AS "state_instance_id",
    q11.c3 AS "derivation_id"
FROM q11
