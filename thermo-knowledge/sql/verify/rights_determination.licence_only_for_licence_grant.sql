-- invariant: rights_determination.licence_only_for_licence_grant
-- A rights determination names a licence although its basis is not a licence grant.
SELECT r.id, r.carrier, r.scope, r.basis::text AS basis, l.key AS licence
FROM prov.rights_determination r
JOIN prov.licence l ON l.id = r.licence
WHERE r.basis::text <> 'licence_grant'
