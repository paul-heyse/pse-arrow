-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "inheritance"."instance_id" AS c0,
        "inheritance"."name" AS c1,
        "features"."source_owner_id" AS c2,
        "features"."source_name" AS c3,
        "features"."value" AS c4,
        "inheritance"."derivation_id" AS c5
    FROM "normalized"."feature_inheritance" AS "inheritance" JOIN "inferred"."feature_candidates" AS "features" ON ("inheritance"."source_instance_id" = "features"."instance_id") AND ("inheritance"."source_name" = "features"."name")
)
SELECT
    q3.c0 AS "instance_id",
    q3.c1 AS "name",
    q3.c2 AS "source_owner_id",
    q3.c3 AS "source_name",
    q3.c4 AS "value",
    q3.c5 AS "derivation_id"
FROM q3
