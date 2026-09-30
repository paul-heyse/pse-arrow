-- invariant: group_bond_count.groups_in_assignment
-- A group bond names a group of another scheme than the scheme of its assignment, or a group that has no group count in the assignment.
SELECT b.id, loc.locator, b.first, b.second,
       CASE WHEN g1.scheme <> a.scheme OR g2.scheme <> a.scheme THEN 'a group is of another scheme than the assignment'
            ELSE 'a group has no group count in the assignment' END AS reason
FROM tk.group_bond_count b
JOIN tk.group_assignment a ON a.id = b.assignment
JOIN tk."group" g1 ON g1.id = b.first
JOIN tk."group" g2 ON g2.id = b.second
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = b.assignment
) loc ON true
WHERE g1.scheme <> a.scheme OR g2.scheme <> a.scheme
   OR NOT EXISTS (SELECT 1 FROM tk.group_count c WHERE c.assignment = b.assignment AND c."group" = b.first)
   OR NOT EXISTS (SELECT 1 FROM tk.group_count c WHERE c.assignment = b.assignment AND c."group" = b.second)
