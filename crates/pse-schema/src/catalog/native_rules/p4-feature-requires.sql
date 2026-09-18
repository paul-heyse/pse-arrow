-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q10 AS (
    SELECT
        "instances"."instance_id" AS c0,
        "declarations"."rule" AS c1,
        "declarations"."antecedent" AS c2,
        "declarations"."consequent" AS c3,
        "left_feature"."derivation_id" AS c4
    FROM "normalized"."template_feature_rules" AS "declarations" JOIN "normalized"."instance_bindings" AS "instances" ON ("declarations"."template_id" = "instances"."template_id") JOIN "inferred"."instance_features" AS "left_feature" ON ("instances"."instance_id" = "left_feature"."instance_id") AND ("declarations"."antecedent" = "left_feature"."name") JOIN "inferred"."instance_features" AS "right_feature" ON ("instances"."instance_id" = "right_feature"."instance_id") AND ("declarations"."consequent" = "right_feature"."name")
    WHERE (((((((("declarations"."rule" = 'requires'))))))) AND (((NOT get_field(get_field("left_feature"."value", 'boolean'), 'value')) OR get_field(get_field("right_feature"."value", 'boolean'), 'value'))))
)
SELECT
    q10.c0 AS "instance_id",
    q10.c1 AS "rule",
    q10.c2 AS "antecedent",
    q10.c3 AS "consequent",
    q10.c4 AS "derivation_id"
FROM q10
