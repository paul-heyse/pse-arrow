-- invariant: disordered_partner.same_system
-- An ordered phase names a disordered partner that belongs to another chemical system.
SELECT d.id, loc.locator, o.system AS ordered_system, p.system AS disordered_system
FROM tk.disordered_partner d
JOIN tk.phase_definition o ON o.id = d.ordered
JOIN tk.phase_definition p ON p.id = d.disordered
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin r
    JOIN prov.import_record i ON i.id = r.import_record
    WHERE r.record = d.ordered
) loc ON true
WHERE o.system <> p.system
