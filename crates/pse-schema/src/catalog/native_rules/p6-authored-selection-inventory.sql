-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "selections"."selection_id" AS c0,
        "selections"."is_default" AS c1,
        "selections"."property_package_id" AS c2,
        "selections"."scope_kind" AS c3,
        "selections"."scope_ids" AS c4,
        "selections"."property_kind_id" AS c5,
        "selections"."family" AS c6,
        "selections"."method_id" AS c7,
        'authored_selection' AS c8,
        "selections"."selection_id" AS c9
    FROM "normalized"."method_selections" AS "selections"
)
SELECT
    q1.c0 AS "selection_id",
    q1.c1 AS "is_default",
    q1.c2 AS "property_package_id",
    q1.c3 AS "scope_kind",
    q1.c4 AS "scope_ids",
    q1.c5 AS "property_kind_id",
    q1.c6 AS "family",
    q1.c7 AS "method_id",
    q1.c8 AS "source_kind",
    q1.c9 AS "derivation_id"
FROM q1
