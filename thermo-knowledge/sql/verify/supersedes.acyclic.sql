-- invariant: supersedes.acyclic
-- A parameterisation supersedes itself, directly or through a chain of supersessions.
WITH RECURSIVE walk (start, node) AS (
    SELECT s.newer, s.older FROM tk.supersedes s
    UNION ALL
    SELECT w.start, s.older FROM walk w JOIN tk.supersedes s ON s.newer = w.node
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
