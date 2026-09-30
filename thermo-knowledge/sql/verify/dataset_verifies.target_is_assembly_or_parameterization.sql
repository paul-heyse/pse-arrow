-- invariant: dataset_verifies.target_is_assembly_or_parameterization
-- A verification dataset names a record that is neither a model assembly nor a parameterization.
SELECT v.id, loc.locator, v.dataset, v.target, r.kind AS target_kind
FROM ev.dataset_verifies v
JOIN prov.record r ON r.id = v.target
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.dataset
) loc ON true
WHERE r.kind NOT IN ('model_assembly', 'parameterization')
