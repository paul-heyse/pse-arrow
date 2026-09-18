-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q18 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "selections"."selection_id" AS c7,
        "selections"."method_id" AS c14,
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
        q18.c7 AS c7,
        q18.c14 AS c14,
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
q31 AS (
    SELECT
        q19.c0 AS c0,
        q19.c7 AS c1,
        q19.c14 AS c2
    FROM q19 LEFT ANTI JOIN q29 ON (q19.c0 = q29.c0) AND (q19.c7 = q29.c1) AND (q19.c14 = q29.c2)
),
q33 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "selections"."selection_id" AS c1,
        "selections"."method_id" AS c2,
        false AS c3,
        NULL AS c4,
        'incompatible_signature' AS c5,
        "selections"."derivation_id" AS c6
    FROM "inferred"."requirement_universe" AS "requirements" JOIN "inferred"."selection_inventory" AS "selections" ON ("requirements"."property_package_id" = "selections"."property_package_id") JOIN "inferred"."requirement_scope_keys" AS "scope_keys" ON ("requirements"."requirement_id" = "scope_keys"."requirement_id") AND ("selections"."scope_kind" = "scope_keys"."scope_kind") AND ("selections"."scope_ids" = "scope_keys"."scope_ids") JOIN "reference"."method_specs" AS "methods" ON ("selections"."method_id" = "methods"."method_id") JOIN "reference"."method_provisions" AS "provisions" ON ("selections"."method_id" = "provisions"."method_id") AND ("requirements"."property_kind_id" = "provisions"."property_kind_id") LEFT ANTI JOIN q31 ON ("requirements"."requirement_id" = q31.c0) AND ("selections"."selection_id" = q31.c1) AND ("selections"."method_id" = q31.c2)
    WHERE ((((((((("selections"."property_kind_id" IS NULL) OR ("requirements"."property_kind_id" = "selections"."property_kind_id")))))) AND (("selections"."family" = "methods"."family")))))
)
SELECT
    q33.c0 AS "requirement_id",
    q33.c1 AS "selection_id",
    q33.c2 AS "method_id",
    q33.c3 AS "applicable",
    q33.c4 AS "rank",
    q33.c5 AS "reason",
    q33.c6 AS "derivation_id"
FROM q33
