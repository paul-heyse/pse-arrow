-- invariant: accuracy_statement.magnitude_matches_kind
-- An accuracy statement states a magnitude its kind does not allow, or lacks the one it requires.
-- The kinds whose magnitude is dimensionless (a fraction of the value, or a factor) and those that state no magnitude are the members of `uncertainty_kind` with the facets `relative` or `factor` and `unquantified`, so a member with a facet is covered without editing this file. A factor is stored where a relative magnitude is.
WITH kind_facets AS (
    SELECT m.name AS kind,
           EXISTS (SELECT 1 FROM meta.enum_member_facet f
                   WHERE f.enum = 'uncertainty_kind' AND f.member = m.name AND f.facet IN ('relative', 'factor')) AS is_relative,
           EXISTS (SELECT 1 FROM meta.enum_member_facet f
                   WHERE f.enum = 'uncertainty_kind' AND f.member = m.name AND f.facet = 'unquantified') AS is_unquantified
    FROM meta.enum_member m
    WHERE m.enum = 'uncertainty_kind'
)
SELECT a.id, loc.locator, a.kind::text AS kind, a.magnitude, a.relative_magnitude
FROM tk.accuracy_statement a
JOIN kind_facets k ON k.kind = a.kind::text
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = a.record
) loc ON true
WHERE NOT CASE
    WHEN k.is_unquantified THEN a.magnitude IS NULL AND a.relative_magnitude IS NULL
    WHEN k.is_relative THEN a.relative_magnitude IS NOT NULL AND a.magnitude IS NULL
    ELSE a.magnitude IS NOT NULL AND a.relative_magnitude IS NULL
END
