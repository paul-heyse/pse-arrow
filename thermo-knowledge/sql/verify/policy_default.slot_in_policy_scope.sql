-- invariant: policy_default.slot_in_policy_scope
-- A policy scoped to a slot group states a default for a slot of another group.
SELECT d.id, loc.locator, d.policy, sl.qualified_name AS slot, sl.slot_group AS slot_group
FROM tk.policy_default d
JOIN tk.selection_policy p ON p.id = d.policy
JOIN meta.slot sl ON sl.id = d.slot
JOIN meta.slot_group sg ON sg.qualified_name = sl.slot_group
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = d.policy
) loc ON true
WHERE p.scope_slot_group IS NOT NULL AND sg.id <> p.scope_slot_group
