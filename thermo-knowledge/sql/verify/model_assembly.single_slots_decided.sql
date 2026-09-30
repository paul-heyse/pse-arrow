-- invariant: model_assembly.single_slots_decided
-- A sub-form slot with multiplicity one that a model decides (per = model), of the root form or of a form the assembly chooses, has no choice at its path in the assembly.
WITH placed AS (
    SELECT a.id AS assembly, ''::text AS path, root.name AS form
    FROM tk.model_assembly a
    JOIN meta.form root ON root.id = a.root
    UNION
    SELECT c.assembly, c.path, f.name
    FROM tk.assembly_choice c
    JOIN meta.form f ON f.id = c.form
)
SELECT p.assembly AS id, loc.locator, p.path AS at_path, p.form, sl.qualified_name AS open_slot
FROM placed p
JOIN meta.subform_slot sl ON sl.form = p.form AND sl.multiplicity = 'one' AND sl.per = 'model'
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.assembly
) loc ON true
WHERE NOT EXISTS (
    SELECT 1
    FROM tk.assembly_choice d
    WHERE d.assembly = p.assembly
      AND d.slot = sl.id
      AND d.path = CASE WHEN p.path = '' THEN sl.name ELSE p.path || '/' || sl.name END
)
