-- invariant: region_clause.relative_bound_matches_observable
-- A clause names an observable a bound is relative to for a bound it does not state, or one whose quantity type differs from that of the clause's observable.
SELECT c.id, loc.locator, c.region, c.ordinal, b.side, b.reference,
       CASE WHEN b.bound IS NULL THEN 'the ' || b.side || ' bound is not stated'
            ELSE 'the observable ' || reference_observable.key || ' has another quantity type than the clause''s observable' END AS reason
FROM tk.region_clause c
JOIN tk.observable clause_observable ON clause_observable.id = c.observable
CROSS JOIN LATERAL (
    VALUES ('lower', c.lower, c.lower_relative_to), ('upper', c.upper, c.upper_relative_to)
) AS b(side, bound, reference)
LEFT JOIN tk.observable reference_observable ON reference_observable.id = b.reference
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.region
) loc ON true
WHERE b.reference IS NOT NULL
  AND (b.bound IS NULL OR reference_observable.quantity IS DISTINCT FROM clause_observable.quantity)
