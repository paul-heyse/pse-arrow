-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "included"."scope_id" AS c0,
        "included"."port_id" AS c1,
        "included"."state_index" AS c2,
        true AS c3,
        "included"."derivation_id" AS c4
    FROM "inferred"."scope_port_states" AS "included"
),
q2 AS (
    SELECT q1.c0, q1.c1, q1.c2, q1.c3, q1.c4 FROM q1
)
SELECT
    q2.c0 AS "scope_id",
    q2.c1 AS "port_id",
    q2.c2 AS "state_index",
    q2.c3 AS "included",
    q2.c4 AS "derivation_id"
FROM q2
