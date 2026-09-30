-- invariant: constituent_array.key_follows_members
-- A constituent array's canonical key is not the ordering convention applied to its members: the phase definition's key, then for each site class that has members, in ascending class index, a colon and the canonical keys of its species in position order, joined by commas.
WITH per_class AS (
    SELECT m."array" AS array_id, sc."index" AS class_index,
           string_agg(e.canonical_key, ',' ORDER BY m."position") AS members
    FROM tk.constituent_array_member m
    JOIN tk.site_class sc ON sc.id = m.site_class
    JOIN tk.material_entity e ON e.id = m.species
    GROUP BY m."array", sc."index"
),
expected AS (
    SELECT a.id, p.key || coalesce(
               (SELECT string_agg(':' || c.members, '' ORDER BY c.class_index)
                FROM per_class c WHERE c.array_id = a.id), '') AS canonical_key
    FROM tk.constituent_array a
    JOIN tk.phase_definition p ON p.id = a.phase
)
SELECT a.id, loc.locator, a.canonical_key AS stated, x.canonical_key AS expected
FROM tk.constituent_array a
JOIN expected x ON x.id = a.id
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = a.id
) loc ON true
WHERE a.canonical_key <> x.canonical_key
