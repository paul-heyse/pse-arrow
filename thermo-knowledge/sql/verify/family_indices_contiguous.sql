-- structural: family_indices_contiguous
-- The integer indices of a family's rows for one set are not contiguous from the declared minimum.
-- The family tables are what the declaration says, so a helper reads them from meta.family.
CREATE FUNCTION pg_temp.family_index_rows()
RETURNS TABLE (family text, id uuid, index_name text, lowest bigint, highest bigint, distinct_values bigint)
LANGUAGE plpgsql AS $helper$
DECLARE
    f record;
BEGIN
    FOR f IN
        SELECT fam.qualified_name, fam.table_name, i.name AS index_name, i.minimum
        FROM meta.family fam
        JOIN meta.family_index i ON i.family = fam.qualified_name
        WHERE i.element_kind = 'primitive' AND i.element = 'Integer'
        ORDER BY fam.qualified_name, i.position
    LOOP
        RETURN QUERY EXECUTE format(
            'SELECT %1$L::text, t.set_id, %2$L::text, min(t.%2$I)::bigint, max(t.%2$I)::bigint, '
            'count(DISTINCT t.%2$I) FROM param.%3$I t GROUP BY t.set_id '
            'HAVING count(DISTINCT t.%2$I) <> max(t.%2$I) - min(t.%2$I) + 1 '
            'OR ($1 IS NOT NULL AND min(t.%2$I) <> $1)',
            f.qualified_name, f.index_name, f.table_name
        ) USING f.minimum;
    END LOOP;
END
$helper$;
-- query
SELECT v.id, loc.locator, v.family, v.index_name, v.lowest, v.highest, v.distinct_values
FROM pg_temp.family_index_rows() v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.id
) loc ON true
