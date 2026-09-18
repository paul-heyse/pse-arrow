-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q7 AS (
    SELECT
        "parameters"."template_id" AS c0,
        "parameters"."name" AS c1,
        "parameters"."logical_type_id" AS c2
    FROM "normalized"."template_params" AS "parameters"
    WHERE (("parameters"."enum_id" IS NULL))
),
q10 AS (
    SELECT
        "logical"."logical_type_id" AS c0
    FROM "reference"."schema_logical_types" AS "logical"
    WHERE (("logical"."name" = 'semantic_id'))
),
q14 AS (
    SELECT
        "values"."owner_id" AS c0,
        "values"."name" AS c1,
        get_field(get_field("values"."value", 'semantic_id'), 'value') AS c2
    FROM "normalized"."config_values" AS "values"
    WHERE (((("values"."category" = 'parameter') AND (get_field("values"."value", 'kind') = 'semantic_id') AND (get_field("values"."value", 'semantic_id') IS NOT NULL))))
),
q18 AS (
    SELECT
        "scopes"."scope_id" AS c0,
        "nodes"."node_id" AS c1,
        "target"."instance_id" AS c2,
        "nodes"."node_id" AS c3
    FROM "inferred"."scope_candidates" AS "scopes" JOIN "normalized"."selector_nodes" AS "nodes" ON ("scopes"."scope_decl_id" = "nodes"."scope_decl_id") JOIN "normalized"."instance_bindings" AS "owner" ON ("scopes"."owner_instance_id" = "owner"."instance_id") JOIN q7 ON ("owner"."template_id" = q7.c0) AND ("nodes"."parameter_name" = q7.c1) JOIN q10 ON (q7.c2 = q10.c0) JOIN q14 ON ("owner"."instance_id" = q14.c0) AND ("nodes"."parameter_name" = q14.c1) JOIN "normalized"."instance_bindings" AS "target" ON (q14.c2 = "target"."instance_id")
    WHERE (((((((("nodes"."op" = 'instance_parameter'))))))))
),
q19 AS (
    SELECT q18.c0, q18.c1, q18.c2, q18.c3 FROM q18
)
SELECT
    q19.c0 AS "scope_id",
    q19.c1 AS "node_id",
    q19.c2 AS "target_entity_id",
    q19.c3 AS "derivation_id"
FROM q19
