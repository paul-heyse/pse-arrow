-- invariant: column_uncertainty.column_is_constraint
-- An uncertainty of a constant is stated for an assessment of a column that is not a constraint column.
SELECT u.id, loc.locator, c.role::text AS role
FROM ev.column_uncertainty u
JOIN ev.uncertainty_assessment a ON a.id = u.assessment
JOIN ev.dataset_column c ON c.id = a."column"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.dataset
) loc ON true
WHERE c.role::text <> 'constraint'
