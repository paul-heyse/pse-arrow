-- invariant: reaction.standard_states_for_equilibrium_constant
-- A participant of a reaction that is the subject of an equilibrium-constant parameter set has no standard state: none of its own, and none from the convention set of the parameterisation for the member role it has in the chemical system.
-- An equilibrium-constant set is a set of a slot group with a subject of the kind `reaction`, of a form whose contract has an output with the observable `log10_equilibrium_constant`; the tables are what the declaration says, so a helper reads them from meta.
CREATE FUNCTION pg_temp.equilibrium_constant_sets() RETURNS TABLE (parameter_set uuid, reaction uuid)
LANGUAGE plpgsql AS $helper$
DECLARE
    g record;
BEGIN
    FOR g IN
        SELECT DISTINCT sg.table_name, s.name AS subject
        FROM meta.contract_output o
        JOIN tk.observable ob ON ob.id = o.observable AND ob.key = 'log10_equilibrium_constant'
        JOIN meta.form f ON f.implements = o.contract
        JOIN meta.slot_group sg ON sg.form = f.name
        JOIN meta.slot_group_subject s ON s.slot_group = sg.qualified_name AND s.kind = 'reaction'
        ORDER BY sg.table_name, s.name
    LOOP
        RETURN QUERY EXECUTE format('SELECT t.id, t.%I FROM param.%I t', g.subject, g.table_name);
    END LOOP;
END
$helper$;
-- query
SELECT r.id, loc.locator, v.parameter_set, p.form AS participant, pz.chemical_system, pz.convention_set,
       sm.member_role
FROM pg_temp.equilibrium_constant_sets() v
JOIN tk.parameter_set ps ON ps.id = v.parameter_set
JOIN tk.parameterization pz ON pz.id = ps.parameterization
JOIN tk.reaction r ON r.id = v.reaction
JOIN tk.reaction_participant p ON p.reaction = r.id
LEFT JOIN tk.system_member sm ON sm.system = pz.chemical_system AND sm.form = p.form
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = r.id
) loc ON true
WHERE p.standard_state IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM tk.convention_standard_state c
      WHERE c.convention_set = pz.convention_set AND c.member_role = sm.member_role
  )
