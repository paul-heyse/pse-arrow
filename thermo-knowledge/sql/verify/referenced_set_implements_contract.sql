-- structural: referenced_set_implements_contract
-- A set-reference slot holds a nested set, or a set of a form that implements another contract than the slot names.
-- The slot-group and family tables are what the declaration says, so a helper reads them from meta.slot.
CREATE FUNCTION pg_temp.referenced_sets() RETURNS TABLE (holder uuid, slot text, target uuid)
LANGUAGE plpgsql AS $helper$
DECLARE
    s record;
BEGIN
    FOR s IN
        SELECT sl.qualified_name, sl.name,
               CASE WHEN sl.family IS NULL THEN sg.table_name ELSE fam.table_name END AS table_name,
               CASE WHEN sl.family IS NULL THEN 'id' ELSE 'set_id' END AS owner_column
        FROM meta.slot sl
        JOIN meta.slot_group sg ON sg.qualified_name = sl.slot_group
        LEFT JOIN meta.family fam ON fam.qualified_name = sl.family
        WHERE sl.shape = 'set_reference'
        ORDER BY sl.qualified_name
    LOOP
        RETURN QUERY EXECUTE format(
            'SELECT t.%I, %L::text, t.%I FROM param.%I t WHERE t.%I IS NOT NULL',
            s.owner_column, s.qualified_name, s.name, s.table_name, s.name
        );
    END LOOP;
END
$helper$;
-- query
SELECT v.holder AS id, loc.locator, v.slot, v.target, t.parent IS NOT NULL AS target_is_nested,
       sl.references_contract AS referenced_contract, f.implements AS implemented_contract
FROM pg_temp.referenced_sets() v
JOIN meta.slot sl ON sl.qualified_name = v.slot
JOIN tk.parameter_set t ON t.id = v.target
JOIN meta.slot_group tg ON tg.id = t.slot_group
JOIN meta.form f ON f.name = tg.form
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.holder
) loc ON true
WHERE t.parent IS NOT NULL
   OR f.implements <> sl.references_contract
