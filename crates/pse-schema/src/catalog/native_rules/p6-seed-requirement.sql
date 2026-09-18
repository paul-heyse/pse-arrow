-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "seeds"."requirement_id" AS c0,
        "seeds"."state_scope_id" AS c1,
        "seeds"."property_kind_id" AS c2,
        "seeds"."index" AS c3,
        "seeds"."requirement_id" AS c4
    FROM "inferred"."demand_seed_bindings" AS "seeds"
)
SELECT
    q1.c0 AS "requirement_id",
    q1.c1 AS "state_scope_id",
    q1.c2 AS "property_kind_id",
    q1.c3 AS "index",
    q1.c4 AS "derivation_id"
FROM q1
