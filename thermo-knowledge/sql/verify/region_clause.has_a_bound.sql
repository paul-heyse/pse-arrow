-- invariant: region_clause.has_a_bound
-- A clause of a validity region states neither a lower nor an upper bound.
SELECT c.id, loc.locator, c.region, c.ordinal
FROM tk.region_clause c
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.region
) loc ON true
WHERE c.lower IS NULL AND c.upper IS NULL
