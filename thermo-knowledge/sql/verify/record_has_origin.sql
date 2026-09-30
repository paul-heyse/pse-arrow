-- structural: record_has_origin
-- A record has no origin, and is not the record of a declared entity.
SELECT r.id, NULL::text AS locator, r.kind
FROM prov.record r
WHERE NOT EXISTS (SELECT 1 FROM prov.record_origin o WHERE o.record = r.id)
  AND NOT EXISTS (SELECT 1 FROM meta.entity e WHERE e.id = r.id)
