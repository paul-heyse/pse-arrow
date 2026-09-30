-- structural: parameterization_has_conventions
-- A parameterization holds a parameter set of a form that reads a convention fact, and has no convention set or its convention set states no such fact.
-- The convention-set table and its columns are what the declaration says, so a helper reads the facts each form declares from meta.form_convention.
CREATE FUNCTION pg_temp.missing_conventions() RETURNS TABLE (id uuid, form text, fact text, reason text)
LANGUAGE plpgsql AS $helper$
DECLARE
    c record;
BEGIN
    FOR c IN
        SELECT fc.form, fc.name, k.schema_name, k.table_name
        FROM meta.form_convention fc
        JOIN meta.kind k ON k.name = fc.kind
        ORDER BY fc.form, fc.position
    LOOP
        RETURN QUERY EXECUTE format(
            'SELECT p.id, %1$L::text, %2$L::text, '
            'CASE WHEN cs.id IS NULL THEN ''has no convention set'' '
            'ELSE ''its convention set states no '' || %2$L END '
            'FROM tk.parameterization p '
            'LEFT JOIN %3$I.%4$I cs ON cs.id = p.convention_set '
            'WHERE (cs.id IS NULL OR cs.%2$I IS NULL) AND EXISTS ('
            'SELECT 1 FROM tk.parameter_set ps JOIN meta.slot_group sg ON sg.id = ps.slot_group '
            'WHERE ps.parameterization = p.id AND sg.form = %1$L)',
            c.form, c.name, c.schema_name, c.table_name
        );
    END LOOP;
END
$helper$;
-- query
SELECT v.id, loc.locator, v.form, v.fact, v.reason
FROM pg_temp.missing_conventions() v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.id
) loc ON true
