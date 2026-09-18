-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q5 AS (
    SELECT
        "targets"."port_id" AS c0,
        "targets"."state_index" AS c1,
        true AS c2,
        "targets"."derivation_id" AS c3
    FROM "inferred"."port_state_targets" AS "targets" JOIN "inferred"."ports" AS "ports" ON ("targets"."port_id" = "ports"."port_id")
),
q9 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        q5.c0 AS c1,
        q5.c1 AS c2,
        false AS c3,
        q5.c3 AS c4
    FROM "inferred"."resolved_scopes" AS "scopes" JOIN q5 ON (true = q5.c2) LEFT ANTI JOIN "inferred"."scope_port_states" AS "included" ON ("scopes"."scope_id" = "included"."scope_id") AND (q5.c0 = "included"."port_id") AND (q5.c1 = "included"."state_index")
),
q10 AS (
    SELECT q9.c0, q9.c1, q9.c2, q9.c3, q9.c4 FROM q9
)
SELECT
    q10.c0 AS "scope_id",
    q10.c1 AS "port_id",
    q10.c2 AS "state_index",
    q10.c3 AS "included",
    q10.c4 AS "derivation_id"
FROM q10
