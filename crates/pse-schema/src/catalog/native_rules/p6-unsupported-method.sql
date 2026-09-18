-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "requirements"."requirement_id" AS c0,
        "pse_checked_value"("named_struct"('kind', 'unresolved', 'resolved', "nullif"("named_struct"('method_id', "requirements"."requirement_id"), "named_struct"('method_id', "requirements"."requirement_id"))), 'inferred.method_resolutions', 'outcome') AS c1,
        "requirements"."requirement_id" AS c2
    FROM "inferred"."requirement_universe" AS "requirements" LEFT ANTI JOIN "inferred"."potential_method_winners" AS "winners" ON ("requirements"."requirement_id" = "winners"."requirement_id")
)
SELECT
    q3.c0 AS "requirement_id",
    q3.c1 AS "outcome",
    q3.c2 AS "derivation_id"
FROM q3
