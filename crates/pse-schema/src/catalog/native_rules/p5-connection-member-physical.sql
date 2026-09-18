-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q12 AS (
    SELECT
        "connections"."connection_id" AS c0,
        "left_quantity"."quantity_kind_id" AS c29,
        "left_quantity"."basis_id" AS c30,
        "left_quantity"."reference_state_id" AS c31,
        "left_quantity"."scale_kind" AS c32,
        "left_quantity"."shape" AS c33,
        "left_quantity"."subject_kind" AS c34,
        "right_quantity"."quantity_kind_id" AS c39,
        "right_quantity"."basis_id" AS c40,
        "right_quantity"."reference_state_id" AS c41,
        "right_quantity"."scale_kind" AS c42,
        "right_quantity"."shape" AS c43,
        "right_quantity"."subject_kind" AS c44
    FROM "normalized"."connections" AS "connections" JOIN "inferred"."port_members" AS "members" ON ("connections"."from_port_id" = "members"."port_id") JOIN "inferred"."port_members" AS "other" ON ("connections"."to_port_id" = "other"."port_id") AND ("members"."ordinal" = "other"."ordinal") AND ("members"."symbol_group" = "other"."symbol_group") JOIN "inferred"."port_member_domains" AS "left_domains" ON ("members"."port_id" = "left_domains"."port_id") AND ("members"."ordinal" = "left_domains"."ordinal") JOIN "inferred"."port_member_domains" AS "right_domains" ON ("other"."port_id" = "right_domains"."port_id") AND ("other"."ordinal" = "right_domains"."ordinal") JOIN "reference"."quantity_types" AS "left_quantity" ON ("members"."quantity_type_id" = "left_quantity"."quantity_type_id") JOIN "reference"."quantity_types" AS "right_quantity" ON ("other"."quantity_type_id" = "right_quantity"."quantity_type_id")
),
q13 AS (
    SELECT
        q12.c0 AS c0
    FROM q12
    WHERE ((q12.c29 IS DISTINCT FROM q12.c39) OR (q12.c30 IS DISTINCT FROM q12.c40) OR (q12.c31 IS DISTINCT FROM q12.c41) OR (q12.c32 IS DISTINCT FROM q12.c42) OR (q12.c33 IS DISTINCT FROM q12.c43) OR (q12.c34 IS DISTINCT FROM q12.c44))
),
q14 AS (
    SELECT
        q13.c0 AS c0,
        'member_physical_mismatch' AS c1,
        q13.c0 AS c2
    FROM q13
),
q15 AS (
    SELECT q14.c0, q14.c1, q14.c2 FROM q14
)
SELECT
    q15.c0 AS "connection_id",
    q15.c1 AS "reason",
    q15.c2 AS "derivation_id"
FROM q15
