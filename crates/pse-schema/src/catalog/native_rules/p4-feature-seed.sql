-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "config"."owner_id" AS c0,
        "config"."name" AS c1,
        "config"."owner_id" AS c2,
        "config"."name" AS c3,
        "config"."value" AS c4,
        "config"."derivation_id" AS c5
    FROM "normalized"."config_values" AS "config"
    WHERE ((("config"."category" = 'feature')))
),
q3 AS (
    SELECT q2.c0, q2.c1, q2.c2, q2.c3, q2.c4, q2.c5 FROM q2
)
SELECT
    q3.c0 AS "instance_id",
    q3.c1 AS "name",
    q3.c2 AS "source_owner_id",
    q3.c3 AS "source_name",
    q3.c4 AS "value",
    q3.c5 AS "derivation_id"
FROM q3
