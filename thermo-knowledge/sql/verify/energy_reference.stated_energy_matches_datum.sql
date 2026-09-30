-- invariant: energy_reference.stated_energy_matches_datum
-- An energy reference states an energy at a state without naming which energy or without exactly one of the molar and the specific value, or states them although its enthalpy datum states no value at a state.
-- The data that state a value at a state are the members of `enthalpy_datum` with the facet `stated_value`, so a member with the facet is covered without editing this file.
SELECT r.id, loc.locator, r.key, r.enthalpy::text AS enthalpy, r.datum_energy::text AS datum_energy,
       r.energy_value, r.specific_energy_value
FROM tk.energy_reference r
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = r.id
) loc ON true
WHERE NOT CASE
    WHEN EXISTS (SELECT 1 FROM meta.enum_member_facet f
                 WHERE f.enum = 'enthalpy_datum' AND f.member = r.enthalpy::text AND f.facet = 'stated_value')
        THEN r.datum_energy IS NOT NULL AND (r.energy_value IS NULL) <> (r.specific_energy_value IS NULL)
    ELSE r.datum_energy IS NULL AND r.energy_value IS NULL AND r.specific_energy_value IS NULL
END
