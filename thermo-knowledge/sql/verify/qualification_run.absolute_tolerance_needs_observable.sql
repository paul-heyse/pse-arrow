-- invariant: qualification_run.absolute_tolerance_needs_observable
-- A qualification run states an absolute tolerance but names no observable.
SELECT q.id, loc.locator, q.key, q.absolute_tolerance
FROM qual.qualification_run q
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = q.id
) loc ON true
WHERE q.absolute_tolerance IS NOT NULL AND q.observable IS NULL
