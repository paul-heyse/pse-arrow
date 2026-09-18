-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q11 AS (
    SELECT
        "targets"."port_id" AS c0,
        "targets"."state_index" AS c1,
        "targets"."state_instance_id" AS c2,
        "targets"."derivation_id" AS c3
    FROM "inferred"."port_state_candidates" AS "targets" JOIN "inferred"."port_state_domain_candidates" AS "domains" ON ("targets"."port_id" = "domains"."port_id") JOIN "inferred"."valid_index_tuples" AS "valid" ON ("domains"."product_id" = "valid"."product_id") AND ("targets"."state_index" = "valid"."tuple") JOIN "inferred"."instances" AS "states" ON ("targets"."state_instance_id" = "states"."instance_id") JOIN "inferred"."ports" AS "ports" ON ("targets"."port_id" = "ports"."port_id")
),
q12 AS (
    SELECT q11.c0, q11.c1, q11.c2, q11.c3 FROM q11
)
SELECT
    q12.c0 AS "port_id",
    q12.c1 AS "state_index",
    q12.c2 AS "state_instance_id",
    q12.c3 AS "derivation_id"
FROM q12
