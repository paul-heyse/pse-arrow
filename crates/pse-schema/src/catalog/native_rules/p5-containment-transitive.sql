-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "paths"."ancestor_id" AS c0,
        "children"."instance_id" AS c1,
        "children"."derivation_id" AS c2
    FROM "inferred"."instance_reachability" AS "paths" JOIN "inferred"."instances" AS "children" ON ("paths"."descendant_id" = "children"."parent_instance_id")
)
SELECT
    q3.c0 AS "ancestor_id",
    q3.c1 AS "descendant_id",
    q3.c2 AS "derivation_id"
FROM q3
