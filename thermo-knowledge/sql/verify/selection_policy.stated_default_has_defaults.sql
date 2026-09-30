-- invariant: selection_policy.stated_default_has_defaults
-- A policy that supplies stated defaults for unasserted subjects states no default for any slot.
SELECT p.id, loc.locator, p.key, p.revision
FROM tk.selection_policy p
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.id
) loc ON true
WHERE p.unasserted::text = 'stated_default'
  AND NOT EXISTS (SELECT 1 FROM tk.policy_default d WHERE d.policy = p.id)
