-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q3 AS (
    SELECT
        "candidates"."product_id" AS c0,
        "candidates"."tuple" AS c1,
        "candidates"."derivation_id" AS c2
    FROM "normalized"."candidate_index_tuples" AS "candidates" LEFT ANTI JOIN "inferred"."invalid_index_tuples" AS "invalid" ON ("candidates"."product_id" = "invalid"."product_id") AND ("candidates"."tuple" = "invalid"."tuple")
)
SELECT
    q3.c0 AS "product_id",
    q3.c1 AS "tuple",
    q3.c2 AS "derivation_id"
FROM q3
