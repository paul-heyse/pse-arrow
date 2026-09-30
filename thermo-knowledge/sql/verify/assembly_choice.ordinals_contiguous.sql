-- invariant: assembly_choice.ordinals_contiguous
-- The choices at one path of an assembly do not number from one without a gap, or number more than one for a slot whose multiplicity is not many.
WITH grouped AS (
    SELECT c.assembly, c.path, c.slot,
           (array_agg(c.id ORDER BY c.ordinal))[1] AS id,
           count(*) AS choices, min(c.ordinal) AS lowest, max(c.ordinal) AS highest
    FROM tk.assembly_choice c
    GROUP BY c.assembly, c.path, c.slot
)
SELECT g.id, loc.locator, g.path, sl.qualified_name AS slot, sl.multiplicity, g.choices, g.lowest, g.highest
FROM grouped g
JOIN meta.subform_slot sl ON sl.id = g.slot
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = g.assembly
) loc ON true
WHERE g.lowest <> 1
   OR g.highest <> g.choices
   OR (g.choices > 1 AND sl.multiplicity <> 'many')
