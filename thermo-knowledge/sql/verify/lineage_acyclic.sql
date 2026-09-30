-- structural: lineage_acyclic
-- A record is its own ancestor through derivation lineage, or a dependency relation returns to itself.
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
) CYCLE node SET is_cycle USING path,
dependency_walk (start, node) AS (
    SELECT prerequisite, dependent FROM tk.dependency
    UNION ALL
    SELECT w.start, d.dependent FROM dependency_walk w JOIN tk.dependency d ON d.prerequisite = w.node
) CYCLE node SET is_cycle USING path,
problems (id, graph) AS (
    SELECT DISTINCT start, 'derivation lineage' FROM lineage WHERE node = start
    UNION ALL
    SELECT DISTINCT start, 'dependency' FROM dependency_walk WHERE node = start
)
SELECT p.id, loc.locator, p.graph
FROM problems p
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.id
) loc ON true
