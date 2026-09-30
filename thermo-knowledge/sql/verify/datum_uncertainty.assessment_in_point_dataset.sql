-- invariant: datum_uncertainty.assessment_in_point_dataset
-- An uncertainty is stated for a point of another dataset than the column its assessment is of.
SELECT u.id, loc.locator, p.dataset AS point_dataset, c.dataset AS column_dataset
FROM ev.datum_uncertainty u
JOIN ev.data_point p ON p.id = u.point
JOIN ev.uncertainty_assessment a ON a.id = u.assessment
JOIN ev.dataset_column c ON c.id = a."column"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.dataset
) loc ON true
WHERE c.dataset <> p.dataset
