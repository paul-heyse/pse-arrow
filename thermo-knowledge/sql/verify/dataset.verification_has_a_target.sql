-- invariant: dataset.verification_has_a_target
-- A dataset of kind `verification` verifies no record.
SELECT d.id, loc.locator, d.local_key
FROM ev.dataset d
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = d.id
) loc ON true
WHERE d.kind = 'verification'
  AND NOT EXISTS (SELECT 1 FROM ev.dataset_verifies v WHERE v.dataset = d.id)
