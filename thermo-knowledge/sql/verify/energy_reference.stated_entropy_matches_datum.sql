-- invariant: energy_reference.stated_entropy_matches_datum
-- An energy reference states an entropy at a state without exactly one of the molar and the specific value, or states them although its entropy datum states no value at a state.
-- The data that state a value at a state are the members of `entropy_datum` with the facet `stated_value`, so a member with the facet is covered without editing this file.
SELECT r.id, loc.locator, r.key, r.entropy::text AS entropy, r.entropy_value, r.specific_entropy_value
FROM tk.energy_reference r
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = r.id
) loc ON true
WHERE NOT CASE
    WHEN EXISTS (SELECT 1 FROM meta.enum_member_facet f
                 WHERE f.enum = 'entropy_datum' AND f.member = r.entropy::text AND f.facet = 'stated_value')
        THEN (r.entropy_value IS NULL) <> (r.specific_entropy_value IS NULL)
    ELSE r.entropy_value IS NULL AND r.specific_entropy_value IS NULL
END
