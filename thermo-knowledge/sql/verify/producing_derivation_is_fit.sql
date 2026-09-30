-- structural: producing_derivation_is_fit
-- A record presented in a role with the facet `requires_fit` has a producing derivation that is not a fit.
-- A record with no producing derivation at all is the finding of `producing_derivation`, not of this check.
WITH needing AS (
    SELECT f.member AS role
    FROM meta.enum_member_facet f
    WHERE f.enum = 'origin_role' AND f.facet = 'requires_fit'
)
SELECT DISTINCT o.record AS id, loc.locator, o.role::text AS role, d.derivation AS derivation
FROM prov.record_origin o
JOIN needing n ON n.role = o.role::text
JOIN prov.derivation_output d ON d.record = o.record
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin x
    JOIN prov.import_record i ON i.id = x.import_record
    WHERE x.record = o.record
) loc ON true
WHERE NOT EXISTS (SELECT 1 FROM prov.fit f WHERE f.id = d.derivation)
