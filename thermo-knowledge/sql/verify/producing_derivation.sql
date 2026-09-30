-- structural: producing_derivation
-- A record presented in a role with the facet `requires_derivation` is the output of no derivation (a derivation is exempt).
-- The roles are the members of `origin_role` that `meta.enum_member_facet` marks, so a member with the facet is covered without editing this file.
WITH needing AS (
    SELECT f.member AS role
    FROM meta.enum_member_facet f
    WHERE f.enum = 'origin_role' AND f.facet = 'requires_derivation'
)
SELECT DISTINCT o.record AS id, loc.locator, o.role::text AS role
FROM prov.record_origin o
JOIN needing n ON n.role = o.role::text
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin x
    JOIN prov.import_record i ON i.id = x.import_record
    WHERE x.record = o.record
) loc ON true
WHERE NOT EXISTS (SELECT 1 FROM prov.derivation_output d WHERE d.record = o.record)
  AND NOT EXISTS (SELECT 1 FROM prov.derivation x WHERE x.id = o.record)  -- a derivation is not produced by one
