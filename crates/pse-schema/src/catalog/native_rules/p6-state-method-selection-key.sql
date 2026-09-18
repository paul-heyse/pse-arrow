-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

WITH
q1 AS (
    SELECT
        "source"."property_package_id" AS c0,
        "pse_named_id"("source"."property_package_id", 'pse:state-method-selection:v1') AS c1,
        "source"."property_package_id" AS c2
    FROM "normalized"."property_packages" AS "source"
),
q2 AS (
    SELECT q1.c0, q1.c1, q1.c2 FROM q1
)
SELECT
    q2.c0 AS "property_package_id",
    q2.c1 AS "selection_id",
    q2.c2 AS "derivation_id"
FROM q2
