-- invariant: snapshot_selection.no_failed_fit
-- A snapshot selects a parameter set whose producing derivation is a fit with the outcome `failed`.
SELECT s.id, loc.locator, s.snapshot, s.parameter_set, o.derivation
FROM tk.snapshot_selection s
JOIN prov.derivation_output o ON o.record = s.parameter_set
JOIN prov.fit f ON f.id = o.derivation
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s.parameter_set
) loc ON true
WHERE f.outcome::text = 'failed'
