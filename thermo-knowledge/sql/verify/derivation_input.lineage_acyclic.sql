-- invariant: derivation_input.lineage_acyclic
-- A record is its own ancestor through derivation lineage: an input a derivation used is produced, through some chain of derivations, from the record it produced.
WITH RECURSIVE lineage_edge (src, dst) AS (
    -- an input a derivation excluded did not influence its outputs
    SELECT i.record, o.record
    FROM prov.derivation_input i
    JOIN prov.derivation_output o ON o.derivation = i.derivation
    WHERE NOT i.excluded
), lineage (start, node) AS (
    SELECT src, dst FROM lineage_edge
    UNION ALL
    SELECT l.start, e.dst FROM lineage l JOIN lineage_edge e ON e.src = l.node
) CYCLE node SET is_cycle USING path
SELECT DISTINCT l.start AS id, loc.locator
FROM lineage l
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = l.start
) loc ON true
WHERE l.node = l.start
