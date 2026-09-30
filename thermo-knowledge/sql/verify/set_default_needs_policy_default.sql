-- structural: set_default_needs_policy_default
-- A parameter set leaves a slot at its stated default and a selection policy that may select it (it ranks the set's parameterisation, or overrides to the set) states no default for that slot.
-- The slot-group tables and their state columns are what the declaration says, so a helper reads the stateful slots from meta.slot.
CREATE FUNCTION pg_temp.sets_without_a_default() RETURNS TABLE (id uuid, slot text, policy uuid)
LANGUAGE plpgsql AS $helper$
DECLARE
    s record;
BEGIN
    FOR s IN
        SELECT sl.id AS slot_id, sl.qualified_name, sl.name, sg.table_name
        FROM meta.slot sl
        JOIN meta.slot_group sg ON sg.qualified_name = sl.slot_group
        WHERE sl.presence = 'stateful' AND sl.family IS NULL
        ORDER BY sl.qualified_name
    LOOP
        RETURN QUERY EXECUTE format(
            'SELECT t.id, %1$L::text, selecting.policy '
            'FROM param.%2$I t '
            'JOIN tk.parameter_set ps ON ps.id = t.id '
            'JOIN LATERAL ('
            'SELECT pp.policy FROM tk.policy_precedence pp WHERE pp.parameterization = ps.parameterization '
            'UNION SELECT po.policy FROM tk.policy_override po WHERE po.parameter_set = t.id'
            ') selecting ON true '
            'WHERE t.%3$I::text IN ('
            'SELECT f.member FROM meta.enum_member_facet f '
            'WHERE f.enum = ''value_state'' AND f.facet = ''policy_supplied'') '
            'AND NOT EXISTS (SELECT 1 FROM tk.policy_default d '
            'WHERE d.policy = selecting.policy AND d.slot = %4$L::uuid)',
            s.qualified_name, s.table_name, s.name || '__state', s.slot_id
        );
    END LOOP;
END
$helper$;
-- query
SELECT v.id, loc.locator, v.slot, v.policy
FROM pg_temp.sets_without_a_default() v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.id
) loc ON true
