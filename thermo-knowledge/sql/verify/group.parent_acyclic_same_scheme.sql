-- invariant: group.parent_acyclic_same_scheme
-- A group's parent or partition class is in another scheme, or its parent chain returns to itself.
WITH RECURSIVE ancestry (start, node) AS (
    SELECT g.id, g.parent FROM tk."group" g WHERE g.parent IS NOT NULL
    UNION ALL
    SELECT a.start, g.parent FROM ancestry a JOIN tk."group" g ON g.id = a.node WHERE g.parent IS NOT NULL
) CYCLE node SET is_cycle USING path,
problems (id, reason) AS (
    SELECT DISTINCT start, 'the parent chain contains a cycle' FROM ancestry WHERE node = start
    UNION ALL
    SELECT g.id, 'the parent is in another scheme'
    FROM tk."group" g JOIN tk."group" p ON p.id = g.parent WHERE p.scheme <> g.scheme
    UNION ALL
    SELECT g.id, 'the partition class is in another scheme'
    FROM tk."group" g JOIN tk."group" p ON p.id = g.partition_class WHERE p.scheme <> g.scheme
)
SELECT p.id, loc.locator, p.reason
FROM problems p
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = p.id
) loc ON true
