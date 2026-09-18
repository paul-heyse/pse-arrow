-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "source"."instance_id" AS c0,
        "pse_named_id"("source"."instance_id", 'pse:state-scope:v1') AS c1,
        "source"."instance_id" AS c2
    FROM "normalized"."instance_bindings" AS "source"
),
q2 AS (
    SELECT q1.c0, q1.c1, q1.c2 FROM q1
)
SELECT
    q2.c0 AS "state_instance_id",
    q2.c1 AS "state_scope_id",
    q2.c2 AS "derivation_id"
FROM q2
