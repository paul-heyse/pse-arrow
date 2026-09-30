-- invariant: constituent_array_member.species_may_occupy_site_class
-- An array places a species on a site class that does not list the species among its admissible occupants.
SELECT m.id, loc.locator, m."array", m.site_class, m.species
FROM tk.constituent_array_member m
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = m."array"
) loc ON true
WHERE NOT EXISTS (
    SELECT 1 FROM tk.site_occupant so WHERE so.site_class = m.site_class AND so.occupant = m.species
)
