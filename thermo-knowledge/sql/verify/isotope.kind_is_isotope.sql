-- invariant: isotope.kind_is_isotope
-- An isotope is a conserved quantity whose kind is not `isotope`.
SELECT i.id, loc.locator, q.key, q.kind::text AS kind
FROM tk.isotope i
JOIN tk.conserved_quantity q ON q.id = i.id
LEFT JOIN LATERAL (
    SELECT string_agg(r.locator, '; ' ORDER BY r.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record r ON r.id = o.import_record
    WHERE o.record = i.id
) loc ON true
WHERE q.kind::text <> 'isotope'
