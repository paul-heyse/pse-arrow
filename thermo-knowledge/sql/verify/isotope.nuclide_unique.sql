-- invariant: isotope.nuclide_unique
-- Two isotopes state the same element and mass number.
SELECT i.id, loc.locator, q.key, other.key AS duplicate_of, e.key AS element, i.mass_number
FROM tk.isotope i
JOIN tk.conserved_quantity q ON q.id = i.id
JOIN tk.isotope j ON j.mass_number = i.mass_number AND j.id <> i.id
JOIN tk.conserved_quantity other ON other.id = j.id AND other.of_element = q.of_element
JOIN tk.conserved_quantity e ON e.id = q.of_element
LEFT JOIN LATERAL (
    SELECT string_agg(r.locator, '; ' ORDER BY r.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record r ON r.id = o.import_record
    WHERE o.record = i.id
) loc ON true
