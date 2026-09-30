-- invariant: policy_default.state_may_be_a_default
-- A policy states a default whose value state has not the facet `may_be_policy_default`: a default that is itself a default, a redirect or withheld.
SELECT d.id, loc.locator, d.policy, d.slot, d.state::text AS state
FROM tk.policy_default d
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = d.policy
) loc ON true
WHERE d.state::text NOT IN (
    SELECT f.member FROM meta.enum_member_facet f
    WHERE f.enum = 'value_state' AND f.facet = 'may_be_policy_default'
)
