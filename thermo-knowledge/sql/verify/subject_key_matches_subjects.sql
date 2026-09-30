-- structural: subject_key_matches_subjects
-- A parameter set's subject key is not the canonical encoding of the subjects of its slot-group row (for a nested set: of its parent, slot and index).
-- The encoding is a compact JSON array, references as lowercase UUID text and a text index as a JSON string. The slot-group tables are what the declaration says, so a helper reads them from meta.
CREATE FUNCTION pg_temp.subject_key_rows() RETURNS TABLE (id uuid, stored text, expected text)
LANGUAGE plpgsql AS $helper$
DECLARE
    g record;
    n record;
    joined text;
    index_parts text;
    unsupported boolean;
BEGIN
    -- a top-level set: its subjects in role order
    FOR g IN SELECT sg.qualified_name, sg.table_name FROM meta.slot_group sg ORDER BY sg.qualified_name
    LOOP
        SELECT string_agg(format('t.%I::text', s.name), ' || ''","'' || ' ORDER BY s.position)
        INTO joined
        FROM meta.slot_group_subject s
        WHERE s.slot_group = g.qualified_name;
        RETURN QUERY EXECUTE format(
            'SELECT t.id, ps.subject_key, %s FROM param.%I t JOIN tk.parameter_set ps ON ps.id = t.id '
            'WHERE ps.parent IS NULL',
            CASE WHEN joined IS NULL THEN '''[]''' ELSE '''["'' || ' || joined || ' || ''"]''' END,
            g.table_name
        );
    END LOOP;
    -- a nested set held by a slot outside any family: parent, slot and an empty index
    RETURN QUERY
    SELECT ps.id, ps.subject_key,
           '["' || ps.parent::text || '","' || ps.parent_slot::text || '",""]'
    FROM tk.parameter_set ps
    JOIN meta.slot sl ON sl.id = ps.parent_slot
    WHERE ps.parent IS NOT NULL AND sl.family IS NULL;
    -- a nested set held by a slot of a family: the index is that of the family row that holds it
    FOR n IN
        SELECT sl.id AS slot_id, sl.name AS slot_name, fam.qualified_name, fam.table_name
        FROM meta.slot sl
        JOIN meta.family fam ON fam.qualified_name = sl.family
        WHERE sl.shape = 'nested_set'
        ORDER BY sl.qualified_name
    LOOP
        SELECT string_agg(p.part, ' || '','' || ' ORDER BY p.position), bool_or(p.part IS NULL)
        INTO index_parts, unsupported
        FROM (
            SELECT i.position,
                   CASE
                       WHEN i.element_kind = 'primitive' AND i.element IN ('Integer', 'Boolean')
                           THEN format('f.%I::text', i.name)
                       WHEN i.element_kind IN ('enum', 'identifier', 'kind', 'record', 'meta')
                            OR (i.element_kind = 'primitive' AND i.element = 'Text')
                           THEN format('to_json(f.%I::text)::text', i.name)
                   END AS part
            FROM meta.family_index i
            WHERE i.family = n.qualified_name
        ) p;
        IF index_parts IS NULL OR unsupported THEN
            RAISE EXCEPTION 'the index of family % has a type this check cannot encode', n.qualified_name;
        END IF;
        RETURN QUERY EXECUTE format(
            'SELECT ps.id, ps.subject_key, '
            '''["'' || ps.parent::text || ''","'' || ps.parent_slot::text || ''",'' || '
            'to_json(''['' || %s || '']'')::text || '']'' '
            'FROM tk.parameter_set ps JOIN param.%I f ON f.%I = ps.id WHERE ps.parent_slot = %L',
            index_parts, n.table_name, n.slot_name, n.slot_id
        );
    END LOOP;
END
$helper$;
-- query
SELECT v.id, loc.locator, v.stored, v.expected
FROM pg_temp.subject_key_rows() v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.id
) loc ON true
WHERE v.stored IS DISTINCT FROM v.expected
