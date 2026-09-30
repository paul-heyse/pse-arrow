-- invariant: datum_uncertainty.datum_exists
-- An uncertainty qualifies a value that does not exist: its point has no datum in the column of its assessment, or the datum's state carries no value.
-- A state carries a value exactly when the datum's value is present (`datum.value_matches_state`), so no state is named here.
SELECT u.id, loc.locator, u.point, u.assessment, d.state::text AS state
FROM ev.datum_uncertainty u
JOIN ev.data_point p ON p.id = u.point
JOIN ev.uncertainty_assessment a ON a.id = u.assessment
LEFT JOIN ev.datum d ON d.point = u.point AND d."column" = a."column"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.dataset
) loc ON true
WHERE d.id IS NULL OR d.value IS NULL
