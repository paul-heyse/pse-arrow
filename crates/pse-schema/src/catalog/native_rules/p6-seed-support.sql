-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "seeds"."requirement_id" AS c0,
        "seeds"."requester_id" AS c1,
        'seed' AS c2,
        "seeds"."requester_id" AS c3
    FROM "inferred"."demand_seed_bindings" AS "seeds"
)
SELECT
    q1.c0 AS "requirement_id",
    q1.c1 AS "requester_id",
    q1.c2 AS "source_kind",
    q1.c3 AS "derivation_id"
FROM q1
