-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "keys"."selection_id" AS c0,
        false AS c1,
        "packages"."property_package_id" AS c2,
        'package' AS c3,
        "pse_index_tuple"("pse_id_list"()) AS c4,
        NULL AS c5,
        'state_definition' AS c6,
        "packages"."state_definition_method_id" AS c7,
        'selected_state_definition' AS c8,
        "keys"."derivation_id" AS c9
    FROM "inferred"."state_method_selection_keys" AS "keys" JOIN "normalized"."property_packages" AS "packages" ON ("keys"."property_package_id" = "packages"."property_package_id")
)
SELECT
    q3.c0 AS "selection_id",
    q3.c1 AS "is_default",
    q3.c2 AS "property_package_id",
    q3.c3 AS "scope_kind",
    q3.c4 AS "scope_ids",
    q3.c5 AS "property_kind_id",
    q3.c6 AS "family",
    q3.c7 AS "method_id",
    q3.c8 AS "source_kind",
    q3.c9 AS "derivation_id"
FROM q3
