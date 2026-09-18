-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q5 AS (
    SELECT
        "members"."scope_id" AS c0,
        "targets"."port_id" AS c1,
        "targets"."state_index" AS c2,
        "members"."derivation_id" AS c3
    FROM "inferred"."port_state_targets" AS "targets" JOIN "inferred"."ports" AS "ports" ON ("targets"."port_id" = "ports"."port_id") JOIN "inferred"."scope_members" AS "members" ON ("targets"."state_instance_id" = "members"."entity_id")
),
q6 AS (
    SELECT q5.c0, q5.c1, q5.c2, q5.c3 FROM q5
)
SELECT
    q6.c0 AS "scope_id",
    q6.c1 AS "port_id",
    q6.c2 AS "state_index",
    q6.c3 AS "derivation_id"
FROM q6
