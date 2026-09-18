-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q7 AS (
    SELECT
        "phases"."material_system_id" AS c0,
        "phases"."count" AS c1
    FROM "inferred"."material_member_counts" AS "phases"
    WHERE ((("phases"."kind" = 'phase')))
),
q11 AS (
    SELECT
        "species"."material_system_id" AS c0,
        "species"."count" AS c1
    FROM "inferred"."material_member_counts" AS "species"
    WHERE ((("species"."kind" = 'species')))
),
q14 AS (
    SELECT
        "instances"."instance_id" AS c0,
        "packages"."material_system_id" AS c1,
        q7.c1 AS c2,
        q11.c1 AS c3,
        "instances"."derivation_id" AS c4
    FROM "normalized"."instance_bindings" AS "instances" JOIN "normalized"."template_material_constraints" AS "constraints" ON ("instances"."template_id" = "constraints"."template_id") JOIN "normalized"."property_packages" AS "packages" ON ("instances"."property_package_id" = "packages"."property_package_id") JOIN q7 ON ("packages"."material_system_id" = q7.c0) JOIN q11 ON ("packages"."material_system_id" = q11.c0)
    WHERE (((((q7.c1 >= "constraints"."min_phases") AND (("constraints"."max_phases" IS NULL) OR (q7.c1 <= "constraints"."max_phases"))) AND ((q11.c1 >= "constraints"."min_species") AND (("constraints"."max_species" IS NULL) OR (q11.c1 <= "constraints"."max_species"))))))
)
SELECT
    q14.c0 AS "instance_id",
    q14.c1 AS "material_system_id",
    q14.c2 AS "phase_count",
    q14.c3 AS "species_count",
    q14.c4 AS "derivation_id"
FROM q14
