-- invariant: slot_uncertainty.magnitude_matches_kind
-- An uncertainty states a magnitude its kind does not allow, or lacks the one it requires.
SELECT u.id, loc.locator, u.kind::text AS kind, u.magnitude, u.relative_magnitude
FROM tk.slot_uncertainty u
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = u.id
) loc ON true
WHERE NOT CASE u.kind::text
    WHEN 'relative' THEN u.relative_magnitude IS NOT NULL AND u.magnitude IS NULL
    WHEN 'standard' THEN u.magnitude IS NOT NULL AND u.relative_magnitude IS NULL
    WHEN 'expanded' THEN u.magnitude IS NOT NULL AND u.relative_magnitude IS NULL
    WHEN 'interval' THEN u.magnitude IS NOT NULL AND u.relative_magnitude IS NULL
    WHEN 'exact' THEN u.magnitude IS NULL AND u.relative_magnitude IS NULL
    WHEN 'not_stated' THEN u.magnitude IS NULL AND u.relative_magnitude IS NULL
    ELSE false
END
