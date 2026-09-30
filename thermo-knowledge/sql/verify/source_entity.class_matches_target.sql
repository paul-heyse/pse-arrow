-- invariant: source_entity.class_matches_target
-- A source entity's target is not an entity of the refinement its class names.
-- A class names the refinement of the same name, and `undetermined` names `unclassified_entity`. The refinements are what the declaration says, so a helper reads them from meta.kind.
CREATE FUNCTION pg_temp.target_kinds() RETURNS TABLE (id uuid, kind_name text)
LANGUAGE plpgsql AS $helper$
DECLARE
    leaves text;
BEGIN
    WITH RECURSIVE descendant (name) AS (
        SELECT c.name FROM meta.kind c WHERE c.extends = 'material_entity'
        UNION ALL
        SELECT c.name FROM meta.kind c JOIN descendant d ON c.extends = d.name
    )
    SELECT string_agg(format('SELECT id, %L::text AS kind_name FROM %I.%I', k.name, k.schema_name, k.table_name), ' UNION ALL ' ORDER BY k.name)
    INTO leaves
    FROM meta.kind k JOIN descendant d ON d.name = k.name
    WHERE NOT k.abstract;
    RETURN QUERY EXECUTE coalesce(leaves, 'SELECT NULL::uuid AS id, NULL::text AS kind_name WHERE false');
END
$helper$;
-- query
SELECT se.id, loc.locator, se.entity_class::text AS entity_class, t.kind_name AS target_kind
FROM tk.source_entity se
LEFT JOIN pg_temp.target_kinds() t ON t.id = se.target
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = se.id
) loc ON true
WHERE t.kind_name IS DISTINCT FROM CASE se.entity_class::text
    WHEN 'undetermined' THEN 'unclassified_entity'
    ELSE se.entity_class::text
END
