-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q6 AS (
    SELECT
        "material"."domain_id" AS c0,
        "material"."member_id" AS c1,
        "material"."derivation_id" AS c2
    FROM "normalized"."material_domain_members" AS "material" JOIN "normalized"."species_elements" AS "composition" ON ("material"."element_id" = "composition"."element_id") JOIN "inferred"."phase_species" AS "allowed" ON ("material"."material_system_id" = "allowed"."material_system_id") AND ("composition"."species_id" = "allowed"."species_id")
    WHERE (((((("material"."phase_id" IS NULL) AND ("material"."species_id" IS NULL) AND ("material"."element_id" IS NOT NULL))))))
),
q7 AS (
    SELECT q6.c0, q6.c1, q6.c2 FROM q6
)
SELECT
    q7.c0 AS "domain_id",
    q7.c1 AS "member_id",
    q7.c2 AS "derivation_id"
FROM q7
