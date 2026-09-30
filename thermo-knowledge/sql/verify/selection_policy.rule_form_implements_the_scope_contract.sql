-- invariant: selection_policy.rule_form_implements_the_scope_contract
-- A policy applies a rule form that implements another contract than the form of the slot group the policy is scoped to.
SELECT p.id, loc.locator, p.key, rf.name AS rule_form, rf.implements AS rule_contract,
       sg.qualified_name AS scope, gf.implements AS scope_contract
FROM tk.selection_policy p
JOIN meta.form rf ON rf.id = p.rule_form
JOIN meta.slot_group sg ON sg.id = p.scope_slot_group
JOIN meta.form gf ON gf.name = sg.form
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.id
) loc ON true
WHERE rf.implements <> gf.implements
