-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q10 AS (
    SELECT
        "keys"."state_scope_id" AS c0,
        "actual"."instance_id" AS c1,
        "packages"."property_package_id" AS c2,
        "keys"."derivation_id" AS c3
    FROM "inferred"."state_scope_keys" AS "keys" JOIN "inferred"."instances" AS "actual" ON ("keys"."state_instance_id" = "actual"."instance_id") JOIN "normalized"."instance_bindings" AS "bound" ON ("actual"."instance_id" = "bound"."instance_id") JOIN "normalized"."property_packages" AS "packages" ON ("bound"."property_package_id" = "packages"."property_package_id") JOIN "reference"."method_specs" AS "methods" ON ("packages"."state_definition_method_id" = "methods"."method_id")
    WHERE (((("methods"."family" = 'state_definition') AND (get_field("methods"."realization", 'kind') = 'equation_template') AND (get_field(get_field("methods"."realization", 'equation_template'), 'template_id') = "actual"."template_id"))))
)
SELECT
    q10.c0 AS "state_scope_id",
    q10.c1 AS "state_instance_id",
    q10.c2 AS "property_package_id",
    q10.c3 AS "derivation_id"
FROM q10
