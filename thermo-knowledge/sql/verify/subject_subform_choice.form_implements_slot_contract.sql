-- invariant: subject_subform_choice.form_implements_slot_contract
-- A choice names a form that implements another contract than the sub-form slot it fills accepts.
SELECT c.id, loc.locator, sl.qualified_name AS slot, sl.accepts AS accepted_contract,
       f.name AS form, f.implements AS implemented_contract
FROM tk.subject_subform_choice c
JOIN meta.subform_slot sl ON sl.id = c.slot
JOIN meta.form f ON f.id = c.form
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.parameterization
) loc ON true
WHERE f.implements <> sl.accepts
