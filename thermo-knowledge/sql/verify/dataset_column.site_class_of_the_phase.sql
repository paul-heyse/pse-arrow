-- invariant: dataset_column.site_class_of_the_phase
-- A column names a site class but no phase, a phase that names no phase definition, or a phase definition that does not own the site class.
SELECT c.id, loc.locator, c.site_class,
       CASE WHEN c.phase IS NULL THEN 'the column names no phase'
            WHEN p.phase_definition IS NULL THEN 'the phase names no phase definition'
            ELSE 'the site class belongs to another phase definition or to a material' END AS reason
FROM ev.dataset_column c
JOIN tk.site_class s ON s.id = c.site_class
LEFT JOIN ev.dataset_phase p ON p.id = c.phase
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.dataset
) loc ON true
WHERE c.phase IS NULL OR p.phase_definition IS NULL OR s.phase IS DISTINCT FROM p.phase_definition
