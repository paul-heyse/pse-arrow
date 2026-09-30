-- structural: origin_carrier_has_rights
-- A carrier that is the origin of a record has no rights determination (not_stated is one).
SELECT c.id, min(i.locator) AS locator, c.manifest_id, c.resolved_pin
FROM prov.record_origin o
JOIN prov.import_record i ON i.id = o.import_record
JOIN prov.artifact a ON a.id = i.artifact
JOIN prov.carrier c ON c.id = a.carrier
WHERE NOT EXISTS (SELECT 1 FROM prov.rights_determination d WHERE d.carrier = c.id)
GROUP BY c.id, c.manifest_id, c.resolved_pin
