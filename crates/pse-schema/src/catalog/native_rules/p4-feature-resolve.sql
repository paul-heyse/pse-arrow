-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "features"."instance_id" AS c0,
        "features"."name" AS c1,
        "features"."value" AS c2,
        "features"."derivation_id" AS c3
    FROM "inferred"."feature_candidates" AS "features"
)
SELECT
    q1.c0 AS "instance_id",
    q1.c1 AS "name",
    q1.c2 AS "value",
    q1.c3 AS "derivation_id"
FROM q1
