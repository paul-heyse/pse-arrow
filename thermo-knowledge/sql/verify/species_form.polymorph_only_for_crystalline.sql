-- invariant: species_form.polymorph_only_for_crystalline
-- A species form states a polymorph although its aggregation is not crystalline.
SELECT f.id, loc.locator, f.polymorph, a.name AS aggregation
FROM tk.species_form f
JOIN tk.aggregation a ON a.id = f.aggregation
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = f.id
) loc ON true
WHERE f.polymorph IS NOT NULL AND a.name <> 'crystalline'
