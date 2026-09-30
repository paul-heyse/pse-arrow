-- invariant: conserved_quantity.of_element_matches_kind
-- A conserved quantity names an element although its kind does not carry the facet `names_element`, or names none although it does.
-- The kinds that need an element are the members of `conserved_kind` with the facet `names_element`, so a member with the facet is covered without editing this file.
SELECT q.id, loc.locator, q.key, q.kind::text AS kind, q.of_element
FROM tk.conserved_quantity q
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = q.id
) loc ON true
WHERE (q.of_element IS NOT NULL) <> EXISTS (
    SELECT 1 FROM meta.enum_member_facet f
    WHERE f.enum = 'conserved_kind' AND f.member = q.kind::text AND f.facet = 'names_element'
)
