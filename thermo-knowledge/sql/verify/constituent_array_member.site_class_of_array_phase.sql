-- invariant: constituent_array_member.site_class_of_array_phase
-- An array places a species on a site class that does not belong to the array's own phase definition.
SELECT m.id, loc.locator, m."array", m.site_class, sc.phase AS class_phase, a.phase AS array_phase
FROM tk.constituent_array_member m
JOIN tk.constituent_array a ON a.id = m."array"
JOIN tk.site_class sc ON sc.id = m.site_class
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = m."array"
) loc ON true
WHERE sc.phase IS DISTINCT FROM a.phase
