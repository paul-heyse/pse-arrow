-- invariant: source_entity.provisional_target_matches_status
-- A source entity's target is provisional although its status is unique, or canonical although it is not.
SELECT e.id, loc.locator, e.status::text AS status, m.provisional AS target_provisional
FROM tk.source_entity e
JOIN tk.material_entity m ON m.id = e.target
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = e.id
) loc ON true
WHERE (e.status IN ('ambiguous', 'unresolved', 'rejected')) <> m.provisional
