-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "resolution"."requirement_id" AS c0,
        "pse_require_nonnull"(get_field(get_field("resolution"."outcome", 'resolved'), 'method_id')) AS c1
    FROM "inferred"."potential_method_resolutions" AS "resolution"
    WHERE (((get_field("resolution"."outcome", 'kind') = 'resolved')))
),
q9 AS (
    SELECT
        "target"."requirement_id" AS c0,
        "required"."requirement_id" AS c1,
        'requirement' AS c2,
        "required"."requirement_id" AS c3
    FROM "inferred"."property_requirements" AS "required" JOIN q3 ON ("required"."requirement_id" = q3.c0) JOIN "inferred"."dependency_key_maps" AS "maps" ON ("required"."requirement_id" = "maps"."requirement_id") AND (q3.c1 = "maps"."method_id") JOIN "inferred"."requirement_universe" AS "target" ON ("maps"."target_requirement_id" = "target"."requirement_id")
)
SELECT
    q9.c0 AS "requirement_id",
    q9.c1 AS "requester_id",
    q9.c2 AS "source_kind",
    q9.c3 AS "derivation_id"
FROM q9
