-- invariant: derivation.estimation_names_method
-- A derivation of kind `estimation` states no method, or an empty one.
SELECT d.id, loc.locator, d.key
FROM prov.derivation d
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = d.id
) loc ON true
WHERE d.kind::text = 'estimation' AND (d.method IS NULL OR btrim(d.method) = '')
