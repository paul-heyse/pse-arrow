-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "resolved"."requirement_id" AS c0,
        "resolved"."outcome" AS c1,
        "resolved"."derivation_id" AS c2
    FROM "inferred"."property_requirements" AS "required" JOIN "inferred"."potential_method_resolutions" AS "resolved" ON ("required"."requirement_id" = "resolved"."requirement_id")
)
SELECT
    q3.c0 AS "requirement_id",
    q3.c1 AS "outcome",
    q3.c2 AS "derivation_id"
FROM q3
