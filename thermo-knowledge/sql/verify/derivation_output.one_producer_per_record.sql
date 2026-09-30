-- invariant: derivation_output.one_producer_per_record
-- A record is the output of more than one derivation.
SELECT v.record AS id, loc.locator, v.producers, v.derivations
FROM (
    SELECT o.record, count(*) AS producers,
           string_agg(o.derivation::text, '; ' ORDER BY o.derivation::text) AS derivations
    FROM prov.derivation_output o
    GROUP BY o.record
    HAVING count(*) > 1
) v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.record
) loc ON true
