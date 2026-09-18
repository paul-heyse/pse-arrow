-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q4 AS (
    SELECT
        "material"."domain_id" AS c0,
        "material"."member_id" AS c1,
        "material"."derivation_id" AS c2
    FROM "normalized"."material_domain_members" AS "material" JOIN "normalized"."phases" AS "phases" ON ("material"."phase_id" = "phases"."phase_id")
    WHERE ((((("material"."phase_id" IS NOT NULL) AND ("material"."species_id" IS NULL) AND ("material"."element_id" IS NULL)))))
),
q5 AS (
    SELECT q4.c0, q4.c1, q4.c2 FROM q4
)
SELECT
    q5.c0 AS "domain_id",
    q5.c1 AS "member_id",
    q5.c2 AS "derivation_id"
FROM q5
