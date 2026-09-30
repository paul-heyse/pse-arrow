-- structural: abstract_kind_has_one_refinement
-- A row of an abstract kind has no concrete refinement row, or more than one.
-- The refinement tables are what the declaration says, so a helper reads them from meta.kind.
CREATE FUNCTION pg_temp.abstract_kind_rows() RETURNS TABLE (kind_name text, id uuid, refinements bigint)
LANGUAGE plpgsql AS $helper$
DECLARE
    parent record;
    leaves text;
BEGIN
    FOR parent IN
        SELECT k.name, k.schema_name, k.table_name FROM meta.kind k WHERE k.abstract ORDER BY k.name
    LOOP
        WITH RECURSIVE descendant (name) AS (
            SELECT c.name FROM meta.kind c WHERE c.extends = parent.name
            UNION ALL
            SELECT c.name FROM meta.kind c JOIN descendant d ON c.extends = d.name
        )
        SELECT string_agg(format('SELECT id FROM %I.%I', k.schema_name, k.table_name), ' UNION ALL ' ORDER BY k.name)
        INTO leaves
        FROM meta.kind k JOIN descendant d ON d.name = k.name
        WHERE NOT k.abstract;
        RETURN QUERY EXECUTE format(
            'SELECT %L::text, a.id, count(r.id) FROM %I.%I a LEFT JOIN (%s) r ON r.id = a.id '
            'GROUP BY a.id HAVING count(r.id) <> 1',
            parent.name, parent.schema_name, parent.table_name,
            coalesce(leaves, 'SELECT NULL::uuid AS id WHERE false')
        );
    END LOOP;
END
$helper$;
-- query
SELECT v.id, loc.locator, v.kind_name AS kind, v.refinements
FROM pg_temp.abstract_kind_rows() v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.id
) loc ON true
