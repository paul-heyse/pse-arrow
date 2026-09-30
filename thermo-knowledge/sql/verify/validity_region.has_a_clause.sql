-- invariant: validity_region.has_a_clause
-- A validity region has no clause.
SELECT r.id, loc.locator, r.record, r.kind::text AS kind, r.ordinal
FROM tk.validity_region r
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = r.id
) loc ON true
WHERE NOT EXISTS (SELECT 1 FROM tk.region_clause c WHERE c.region = r.id)
