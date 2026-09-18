-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q0 AS (
    SELECT
        "phase_members"."material_system_id" AS c0,
        "phase_members"."phase_ids" AS c4
    FROM "normalized"."material_systems" AS "phase_members"
),
q1 AS (
    SELECT
        q0.c0 AS c0,
        unnest(pse_require_nonnull(q0.c4)) AS c6
    FROM q0
),
q6 AS (
    SELECT
        "species_members"."material_system_id" AS c0,
        "species_members"."species_ids" AS c3
    FROM "normalized"."material_systems" AS "species_members"
),
q7 AS (
    SELECT
        q6.c0 AS c0,
        unnest(pse_require_nonnull(q6.c3)) AS c6
    FROM q6
),
q11 AS (
    SELECT
        q7.c0 AS c0,
        "species"."species_id" AS c1,
        "species"."valid_phase_types" AS c2
    FROM q7 JOIN "normalized"."species" AS "species" ON (q7.c6 = "species"."species_id")
),
q12 AS (
    SELECT
        q11.c0 AS c0,
        q11.c1 AS c1,
        unnest(q11.c2) AS c3
    FROM q11
),
q14 AS (
    SELECT
        q1.c0 AS c0,
        "phases"."phase_id" AS c1,
        q12.c1 AS c2
    FROM q1 JOIN "normalized"."phases" AS "phases" ON (q1.c6 = "phases"."phase_id") JOIN q12 ON (q1.c0 = q12.c0) AND ("phases"."phase_type" = q12.c3)
),
q15 AS (
    SELECT
        q7.c0 AS c0,
        "species"."species_id" AS c1
    FROM q7 JOIN "normalized"."species" AS "species" ON (q7.c6 = "species"."species_id")
    WHERE (("species"."valid_phase_types" IS NULL))
),
q18 AS (
    SELECT
        q1.c0 AS c0,
        "phases"."phase_id" AS c1,
        q15.c1 AS c2
    FROM q1 JOIN "normalized"."phases" AS "phases" ON (q1.c6 = "phases"."phase_id") JOIN q15 ON (q1.c0 = q15.c0)
    WHERE ((("phases"."phase_type" <> 'aqueousPhase')))
),
q19 AS (
    SELECT q14.c0, q14.c1, q14.c2 FROM q14
    UNION ALL
    SELECT q18.c0, q18.c1, q18.c2 FROM q18
),
q22 AS (
    SELECT
        q19.c0 AS c0,
        "restrictions"."phase_id" AS c1,
        "restrictions"."species_id" AS c2
    FROM q19 JOIN "normalized"."phase_species" AS "restrictions" ON (q19.c1 = "restrictions"."phase_id") AND (q19.c2 = "restrictions"."species_id")
),
q23 AS (
    SELECT
        q19.c0 AS c0,
        q19.c1 AS c1,
        q19.c2 AS c2
    FROM q19 LEFT ANTI JOIN "normalized"."phase_species" AS "restrictions" ON (q19.c1 = "restrictions"."phase_id")
),
q24 AS (
    SELECT q22.c0, q22.c1, q22.c2 FROM q22
    UNION ALL
    SELECT q23.c0, q23.c1, q23.c2 FROM q23
),
q27 AS (
    SELECT
        q24.c0 AS c0,
        "henry"."phase_id" AS c1,
        "henry"."species_id" AS c2,
        true AS c3,
        q24.c0 AS c4
    FROM q24 JOIN "normalized"."henry_declarations" AS "henry" ON (q24.c1 = "henry"."phase_id") AND (q24.c2 = "henry"."species_id")
)
SELECT
    q27.c0 AS "material_system_id",
    q27.c1 AS "phase_id",
    q27.c2 AS "species_id",
    q27.c3 AS "henry",
    q27.c4 AS "derivation_id"
FROM q27
