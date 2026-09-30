-- invariant: dependency.acyclic
-- A record depends on itself, directly or through a chain of dependencies.
WITH RECURSIVE walk (start, node) AS (
    SELECT prerequisite, dependent FROM tk.dependency
    UNION ALL
    SELECT w.start, d.dependent FROM walk w JOIN tk.dependency d ON d.prerequisite = w.node
) CYCLE node SET is_cycle USING path
SELECT DISTINCT w.start AS id, loc.locator
FROM walk w
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = w.start
) loc ON true
WHERE w.node = w.start
