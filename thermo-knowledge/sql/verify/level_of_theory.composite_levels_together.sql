-- invariant: level_of_theory.composite_levels_together
-- A level of theory states its frequency level or its energy level without the other.
SELECT l.id, loc.locator, l.key, l.frequency_level, l.energy_level
FROM prov.level_of_theory l
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = l.id
) loc ON true
WHERE (l.frequency_level IS NULL) <> (l.energy_level IS NULL)
