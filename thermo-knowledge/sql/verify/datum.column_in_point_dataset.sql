-- invariant: datum.column_in_point_dataset
-- A datum's column belongs to another dataset than its point.
SELECT d.id, loc.locator, p.dataset AS point_dataset, c.dataset AS column_dataset
FROM ev.datum d
JOIN ev.data_point p ON p.id = d.point
JOIN ev.dataset_column c ON c.id = d."column"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.dataset
) loc ON true
WHERE c.dataset <> p.dataset
