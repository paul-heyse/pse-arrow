-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q2 AS (
    SELECT
        "phases"."domain_id" AS c0,
        "phases"."member_id" AS c1,
        "phases"."material_system_id" AS c2,
        "phases"."phase_id" AS c3
    FROM "normalized"."material_domain_members" AS "phases"
    WHERE ((("phases"."phase_id" IS NOT NULL) AND ("phases"."species_id" IS NULL) AND ("phases"."element_id" IS NULL)))
),
q7 AS (
    SELECT
        "species"."domain_id" AS c0,
        "species"."member_id" AS c1,
        "species"."material_system_id" AS c2,
        "species"."species_id" AS c4
    FROM "normalized"."material_domain_members" AS "species"
    WHERE ((("species"."phase_id" IS NULL) AND ("species"."species_id" IS NOT NULL) AND ("species"."element_id" IS NULL)))
),
q9 AS (
    SELECT
        "species_members"."product_id" AS c0,
        "species_members"."tuple" AS c1,
        q7.c2 AS c2,
        q7.c4 AS c3
    FROM "normalized"."candidate_index_members" AS "species_members" JOIN q7 ON ("species_members"."domain_id" = q7.c0) AND ("species_members"."member_id" = q7.c1)
),
q13 AS (
    SELECT
        "phase_members"."product_id" AS c0,
        "phase_members"."tuple" AS c1,
        "phase_members"."derivation_id" AS c2
    FROM "normalized"."candidate_index_members" AS "phase_members" JOIN q2 ON ("phase_members"."domain_id" = q2.c0) AND ("phase_members"."member_id" = q2.c1) JOIN q9 ON ("phase_members"."product_id" = q9.c0) AND ("phase_members"."tuple" = q9.c1) AND (q2.c2 = q9.c2) LEFT ANTI JOIN "inferred"."phase_species" AS "allowed" ON (q2.c2 = "allowed"."material_system_id") AND (q2.c3 = "allowed"."phase_id") AND (q9.c3 = "allowed"."species_id")
)
SELECT
    q13.c0 AS "product_id",
    q13.c1 AS "tuple",
    q13.c2 AS "derivation_id"
FROM q13
