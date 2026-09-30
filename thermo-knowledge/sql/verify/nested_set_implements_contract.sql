-- structural: nested_set_implements_contract
-- A nested parameter set is not in a nested-set slot of its parent's group, or implements another contract than the slot accepts.
SELECT s.id, loc.locator, sl.qualified_name AS slot, sl.accepts AS accepted_contract,
       f.implements AS implemented_contract
FROM tk.parameter_set s
JOIN tk.parameter_set p ON p.id = s.parent
JOIN meta.slot_group cg ON cg.id = s.slot_group
JOIN meta.form f ON f.name = cg.form
LEFT JOIN meta.slot sl ON sl.id = s.parent_slot
LEFT JOIN meta.slot_group pg ON pg.id = p.slot_group
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s.id
) loc ON true
WHERE sl.id IS NULL
   OR sl.shape <> 'nested_set'
   OR sl.slot_group IS DISTINCT FROM pg.qualified_name
   OR sl.accepts IS DISTINCT FROM f.implements
