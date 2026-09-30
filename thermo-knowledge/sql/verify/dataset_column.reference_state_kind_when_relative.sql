-- invariant: dataset_column.reference_state_kind_when_relative
-- A column whose presentation is formed against a reference state states no reference state kind.
-- The presentations are the members of `value_presentation` with the facet `relative_to_reference_state`, so a member with the facet is covered without editing this file.
WITH needing AS (
    SELECT f.member AS presentation
    FROM meta.enum_member_facet f
    WHERE f.enum = 'value_presentation' AND f.facet = 'relative_to_reference_state'
)
SELECT c.id, loc.locator, c.presentation::text AS presentation
FROM ev.dataset_column c
JOIN needing n ON n.presentation = c.presentation::text
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.dataset
) loc ON true
WHERE c.reference_state_kind IS NULL
