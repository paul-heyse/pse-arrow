-- invariant: selection_policy.carrier_policy_is_a_scoped_assessed_fact
-- A policy a carrier asserts has no scope, states neither a repeated-row rule nor a table choice, or does not say whether it is as documented; or a policy no carrier asserts states one of the carrier's facts.
SELECT p.id, loc.locator, p.key, p.revision,
       CASE WHEN p.asserted_by IS NULL THEN 'a policy no carrier asserts states a carrier''s fact'
            WHEN p.scope_slot_group IS NULL AND p.scope_observable IS NULL THEN 'the carrier''s policy has no scope'
            WHEN p.repeated_rows IS NULL AND p.table_choice IS NULL THEN 'the carrier''s policy states no rule'
            ELSE 'the carrier''s policy does not say whether it is as documented' END AS reason
FROM tk.selection_policy p
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.id
) loc ON true
WHERE CASE WHEN p.asserted_by IS NULL
           THEN p.repeated_rows IS NOT NULL OR p.table_choice IS NOT NULL OR p.as_documented IS NOT NULL
           ELSE (p.scope_slot_group IS NULL AND p.scope_observable IS NULL)
                OR (p.repeated_rows IS NULL AND p.table_choice IS NULL)
                OR p.as_documented IS NULL END
