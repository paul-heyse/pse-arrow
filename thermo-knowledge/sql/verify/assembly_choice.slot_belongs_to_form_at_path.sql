-- invariant: assembly_choice.slot_belongs_to_form_at_path
-- A choice of a model assembly whose path does not end in the name of its slot, or whose slot is not a slot of the form that stands at the rest of the path: the assembly's root form for a path of one segment, else a form the assembly chooses at the parent path.
WITH placed AS (
    SELECT c.id, c.assembly, c.path, sl.qualified_name AS slot, sl.name AS slot_name, sl.form AS slot_form,
           regexp_replace(c.path, '/?[^/]+$', '') AS parent_path
    FROM tk.assembly_choice c
    JOIN meta.subform_slot sl ON sl.id = c.slot
)
SELECT p.id, loc.locator, p.path, p.slot, p.slot_form
FROM placed p
JOIN tk.model_assembly a ON a.id = p.assembly
JOIN meta.form root ON root.id = a.root
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.assembly
) loc ON true
WHERE substring(p.path FROM '[^/]+$') IS DISTINCT FROM p.slot_name
   OR CASE
          WHEN p.parent_path = '' THEN p.slot_form <> root.name
          ELSE NOT EXISTS (
              SELECT 1
              FROM tk.assembly_choice parent
              JOIN meta.form parent_form ON parent_form.id = parent.form
              WHERE parent.assembly = p.assembly
                AND parent.path = p.parent_path
                AND parent_form.name = p.slot_form
          )
      END
