-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "instances"."parent_instance_id" AS c0,
        "instances"."instance_id" AS c1,
        "instances"."derivation_id" AS c2
    FROM "inferred"."instances" AS "instances"
    WHERE ((("instances"."parent_instance_id" IS NOT NULL)))
),
q3 AS (
    SELECT q2.c0, q2.c1, q2.c2 FROM q2
)
SELECT
    q3.c0 AS "ancestor_id",
    q3.c1 AS "descendant_id",
    q3.c2 AS "derivation_id"
FROM q3
