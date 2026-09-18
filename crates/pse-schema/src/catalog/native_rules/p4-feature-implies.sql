-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "features"."instance_id" AS c0,
        "features"."name" AS c1,
        "features"."source_owner_id" AS c2,
        "features"."source_name" AS c3,
        "features"."value" AS c4,
        "features"."derivation_id" AS c5
    FROM "inferred"."feature_candidates" AS "features"
    WHERE ((get_field(get_field("features"."value", 'boolean'), 'value') IS TRUE))
),
q8 AS (
    SELECT
        "instances"."instance_id" AS c0,
        "implications"."consequent" AS c1,
        q6.c2 AS c2,
        q6.c3 AS c3,
        q6.c4 AS c4,
        q6.c5 AS c5
    FROM "normalized"."template_feature_rules" AS "implications" JOIN "normalized"."instance_bindings" AS "instances" ON ("implications"."template_id" = "instances"."template_id") JOIN q6 ON ("instances"."instance_id" = q6.c0) AND ("implications"."antecedent" = q6.c1)
    WHERE (((((("implications"."rule" = 'implies'))))))
)
SELECT
    q8.c0 AS "instance_id",
    q8.c1 AS "name",
    q8.c2 AS "source_owner_id",
    q8.c3 AS "source_name",
    q8.c4 AS "value",
    q8.c5 AS "derivation_id"
FROM q8
