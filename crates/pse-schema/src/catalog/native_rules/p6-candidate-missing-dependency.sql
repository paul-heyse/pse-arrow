-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q18 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "requirements"."state_instance_id" AS c2,
        "selections"."selection_id" AS c7,
        "selections"."method_id" AS c14,
        "selections"."derivation_id" AS c16,
        "provisions"."indexed_by" AS c36,
        "properties"."quantity_kind_id" AS c39,
        "properties"."basis_id" AS c40,
        "properties"."shape" AS c41,
        "quantities"."quantity_kind_id" AS c45,
        "quantities"."basis_id" AS c46,
        "quantities"."reference_state_id" AS c47,
        "quantities"."scale_kind" AS c48,
        "quantities"."shape" AS c49,
        "quantities"."subject_kind" AS c50,
        "natural"."dimension" AS c57,
        "natural"."reference_state_id" AS c61,
        "canonical"."dimension" AS c67
    FROM "inferred"."requirement_universe" AS "requirements" JOIN "inferred"."selection_inventory" AS "selections" ON ("requirements"."property_package_id" = "selections"."property_package_id") JOIN "inferred"."requirement_scope_keys" AS "scope_keys" ON ("requirements"."requirement_id" = "scope_keys"."requirement_id") AND ("selections"."scope_kind" = "scope_keys"."scope_kind") AND ("selections"."scope_ids" = "scope_keys"."scope_ids") JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") JOIN "reference"."method_provisions" AS "provisions" ON ("selections"."method_id" = "provisions"."method_id") AND ("requirements"."property_kind_id" = "provisions"."property_kind_id") JOIN "reference"."property_kinds" AS "properties" ON ("requirements"."property_kind_id" = "properties"."property_kind_id") JOIN "reference"."quantity_types" AS "quantities" ON ("provisions"."quantity_type_id" = "quantities"."quantity_type_id") JOIN "reference"."units" AS "natural" ON ("provisions"."natural_unit_id" = "natural"."unit_id") JOIN "reference"."units" AS "canonical" ON ("quantities"."canonical_unit_id" = "canonical"."unit_id")
    WHERE ((((((((((("selections"."property_kind_id" IS NULL) OR ("requirements"."property_kind_id" = "selections"."property_kind_id")))))) AND (("selections"."family" = "methods"."family")))))))
),
q19 AS (
    SELECT
        q18.c0 AS c0,
        q18.c2 AS c2,
        q18.c7 AS c7,
        q18.c14 AS c14,
        q18.c16 AS c16,
        q18.c45 AS c45,
        q18.c46 AS c46,
        q18.c47 AS c47,
        q18.c48 AS c48,
        q18.c49 AS c49,
        q18.c50 AS c50
    FROM q18
    WHERE ((q18.c45 = q18.c39) AND (q18.c46 IS NOT DISTINCT FROM q18.c40) AND (q18.c49 = q18.c41) AND (q18.c36 = q18.c41) AND (q18.c57 = q18.c67) AND ((q18.c61 IS NULL) OR (q18.c61 IS NOT DISTINCT FROM q18.c47)))
),
q27 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c7,
        q19.c14 AS c14,
        q19.c45 AS c45,
        q19.c46 AS c46,
        q19.c47 AS c47,
        q19.c48 AS c48,
        q19.c49 AS c49,
        q19.c50 AS c50,
        "expected_types"."quantity_kind_id" AS c110,
        "expected_types"."basis_id" AS c111,
        "expected_types"."reference_state_id" AS c112,
        "expected_types"."scale_kind" AS c113,
        "expected_types"."shape" AS c114,
        "expected_types"."subject_kind" AS c115
    FROM q19 JOIN "inferred"."demand_seed_bindings" AS "bound_reads" ON (q19.c0 = "bound_reads"."requirement_id") JOIN "inferred"."demand_active_reads" AS "read_declarations" ON ("bound_reads"."read_id" = "read_declarations"."read_id") JOIN "normalized"."template_symbols" AS "read_symbols" ON ("read_declarations"."symbol_decl_id" = "read_symbols"."symbol_decl_id") JOIN "reference"."quantity_types" AS "expected_types" ON ("read_symbols"."quantity_type_id" = "expected_types"."quantity_type_id")
),
q28 AS (
    SELECT
        q27.c0 AS c0,
        q27.c7 AS c7,
        q27.c14 AS c14
    FROM q27
    WHERE (NOT ((q27.c45 IS NOT DISTINCT FROM q27.c110) AND (q27.c46 IS NOT DISTINCT FROM q27.c111) AND (q27.c47 IS NOT DISTINCT FROM q27.c112) AND (q27.c48 IS NOT DISTINCT FROM q27.c113) AND (q27.c50 IS NOT DISTINCT FROM q27.c115) AND (q27.c49 IS NOT DISTINCT FROM q27.c114)))
),
q29 AS (
    SELECT
        q28.c0 AS c0,
        q28.c7 AS c1,
        q28.c14 AS c2
    FROM q28
),
q38 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c1,
        q19.c14 AS c2,
        "dependencies"."ordinal" AS c3
    FROM q19 LEFT ANTI JOIN q29 ON (q19.c0 = q29.c0) AND (q19.c7 = q29.c1) AND (q19.c14 = q29.c2) JOIN "reference"."method_dependencies" AS "dependencies" ON (q19.c14 = "dependencies"."method_id") JOIN "inferred"."dependency_key_maps" AS "dependency_maps" ON (q19.c0 = "dependency_maps"."requirement_id") AND (q19.c14 = "dependency_maps"."method_id") AND ("dependencies"."ordinal" = "dependency_maps"."dependency_ordinal") JOIN "inferred"."requirement_universe" AS "dependency_targets" ON ("dependency_maps"."target_requirement_id" = "dependency_targets"."requirement_id") AND ("dependencies"."target_id" = "dependency_targets"."property_kind_id") AND (q19.c2 = "dependency_targets"."state_instance_id")
    WHERE ((((("dependencies"."target_kind" = 'property')))))
),
q48 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c1,
        q19.c14 AS c2,
        "dependencies"."ordinal" AS c3
    FROM q19 LEFT ANTI JOIN q29 ON (q19.c0 = q29.c0) AND (q19.c7 = q29.c1) AND (q19.c14 = q29.c2) JOIN "reference"."method_dependencies" AS "dependencies" ON (q19.c14 = "dependencies"."method_id") JOIN "inferred"."state_dependency_keys" AS "state_maps" ON (q19.c0 = "state_maps"."requirement_id") AND (q19.c14 = "state_maps"."method_id") AND ("dependencies"."ordinal" = "state_maps"."dependency_ordinal") AND ("dependencies"."target_id" = "state_maps"."symbol_decl_id") JOIN "inferred"."valid_index_tuples" AS "state_tuples" ON ("state_maps"."product_id" = "state_tuples"."product_id") AND ("state_maps"."index" = "state_tuples"."tuple") JOIN "normalized"."instance_bindings" AS "state_instances" ON (q19.c2 = "state_instances"."instance_id") JOIN "normalized"."template_symbols" AS "state_symbols" ON ("state_maps"."symbol_decl_id" = "state_symbols"."symbol_decl_id") AND ("state_instances"."template_id" = "state_symbols"."template_id")
    WHERE ((((((("dependencies"."target_kind" = 'state_symbol')))))))
),
q49 AS (
    SELECT q38.c0, q38.c1, q38.c2, q38.c3 FROM q38
    UNION ALL
    SELECT q48.c0, q48.c1, q48.c2, q48.c3 FROM q48
),
q51 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c1,
        q19.c14 AS c2
    FROM q19 LEFT ANTI JOIN q29 ON (q19.c0 = q29.c0) AND (q19.c7 = q29.c1) AND (q19.c14 = q29.c2) JOIN "reference"."method_dependencies" AS "dependencies" ON (q19.c14 = "dependencies"."method_id") LEFT ANTI JOIN q49 ON (q19.c0 = q49.c0) AND (q19.c7 = q49.c1) AND (q19.c14 = q49.c2) AND ("dependencies"."ordinal" = q49.c3)
),
q53 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c1,
        q19.c14 AS c2
    FROM q19 LEFT ANTI JOIN q29 ON (q19.c0 = q29.c0) AND (q19.c7 = q29.c1) AND (q19.c14 = q29.c2) LEFT ANTI JOIN q51 ON (q19.c0 = q51.c0) AND (q19.c7 = q51.c1) AND (q19.c14 = q51.c2)
),
q55 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c1,
        q19.c14 AS c2,
        false AS c3,
        NULL AS c4,
        'incompatible_signature' AS c5,
        q19.c16 AS c6
    FROM q19 LEFT ANTI JOIN q29 ON (q19.c0 = q29.c0) AND (q19.c7 = q29.c1) AND (q19.c14 = q29.c2) LEFT ANTI JOIN q53 ON (q19.c0 = q53.c0) AND (q19.c7 = q53.c1) AND (q19.c14 = q53.c2)
)
SELECT
    q55.c0 AS "requirement_id",
    q55.c1 AS "selection_id",
    q55.c2 AS "method_id",
    q55.c3 AS "applicable",
    q55.c4 AS "rank",
    q55.c5 AS "reason",
    q55.c6 AS "derivation_id"
FROM q55
