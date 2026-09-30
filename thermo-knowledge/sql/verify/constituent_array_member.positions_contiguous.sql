-- invariant: constituent_array_member.positions_contiguous
-- The positions an array gives on one site class do not run from one without a gap.
WITH grouped AS (
    SELECT m."array" AS array_id, m.site_class,
           (array_agg(m.id ORDER BY m."position"))[1] AS id,
           count(*) AS members, min(m."position") AS lowest, max(m."position") AS highest
    FROM tk.constituent_array_member m
    GROUP BY m."array", m.site_class
)
SELECT g.id, loc.locator, g.array_id AS "array", g.site_class, g.members, g.lowest, g.highest
FROM grouped g
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = g.array_id
) loc ON true
WHERE g.lowest <> 1 OR g.highest <> g.members
