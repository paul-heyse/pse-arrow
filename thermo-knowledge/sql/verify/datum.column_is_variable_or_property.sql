-- invariant: datum.column_is_variable_or_property
-- A datum's column is neither a variable nor a property column.
SELECT d.id, loc.locator, c.role::text AS role
FROM ev.datum d
JOIN ev.data_point p ON p.id = d.point
JOIN ev.dataset_column c ON c.id = d."column"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.dataset
) loc ON true
WHERE c.role::text NOT IN ('variable', 'property')
