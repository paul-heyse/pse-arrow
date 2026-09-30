-- invariant: column_uncertainty.magnitude_matches_kind
-- An uncertainty states a pair of magnitudes its assessment's kind does not allow, or lacks the pair it requires.
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
SELECT u.id, loc.locator, a.kind::text AS kind, u.minus, u.plus, u.relative_minus, u.relative_plus
FROM ev.column_uncertainty u
JOIN ev.uncertainty_assessment a ON a.id = u.assessment
JOIN ev.dataset_column c ON c.id = a."column"
JOIN kind_facets k ON k.kind = a.kind::text
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.dataset
) loc ON true
WHERE NOT CASE
    WHEN k.is_unquantified THEN
        u.minus IS NULL AND u.plus IS NULL AND u.relative_minus IS NULL AND u.relative_plus IS NULL
    WHEN k.is_relative THEN
        u.relative_minus IS NOT NULL AND u.relative_plus IS NOT NULL AND u.minus IS NULL AND u.plus IS NULL
    ELSE
        u.minus IS NOT NULL AND u.plus IS NOT NULL AND u.relative_minus IS NULL AND u.relative_plus IS NULL
END
