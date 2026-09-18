-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "instances"."instance_id" AS c0,
        "declarations"."name" AS c1,
        "instances"."derivation_id" AS c2
    FROM "normalized"."instance_bindings" AS "instances" JOIN "normalized"."template_features" AS "declarations" ON ("instances"."template_id" = "declarations"."template_id") LEFT ANTI JOIN "inferred"."instance_features" AS "features" ON ("instances"."instance_id" = "features"."instance_id") AND ("declarations"."name" = "features"."name")
    WHERE NULL
)
SELECT
    q6.c0 AS "instance_id",
    q6.c1 AS "name",
    q6.c2 AS "derivation_id"
FROM q6
