-- invariant: dataset_verifies.dataset_is_verification
-- A dataset that is not of kind `verification` states a record it verifies.
SELECT v.id, loc.locator, v.dataset, d.kind::text AS dataset_kind
FROM ev.dataset_verifies v
JOIN ev.dataset d ON d.id = v.dataset
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.dataset
) loc ON true
WHERE d.kind <> 'verification'
