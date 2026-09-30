-- invariant: species.charge_matches_composition
-- A species states a composition entry for charge that differs from its charge.
SELECT s.id, loc.locator, s.charge, c.value AS composition_charge
FROM tk.species s
JOIN tk.composition c ON c.entity = s.id
JOIN tk.conserved_quantity q ON q.id = c.quantity AND q.key = 'charge'
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s.id
) loc ON true
WHERE abs(s.charge - c.value) > 1e-9
