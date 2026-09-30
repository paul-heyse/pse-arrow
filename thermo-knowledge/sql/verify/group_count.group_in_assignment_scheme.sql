-- invariant: group_count.group_in_assignment_scheme
-- A group count names a group of another scheme than the scheme of its assignment.
SELECT c.id, loc.locator, a.scheme AS assignment_scheme, g.scheme AS group_scheme
FROM tk.group_count c
JOIN tk.group_assignment a ON a.id = c.assignment
JOIN tk."group" g ON g.id = c."group"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = c.assignment
) loc ON true
WHERE g.scheme <> a.scheme
