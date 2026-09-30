-- invariant: tabulated_axis.points_ascending
-- An axis of a tabulated function has points that are not strictly ascending.
SELECT a.id, loc.locator, a.ordinal, cardinality(a."points") AS points,
       (SELECT min(i) FROM generate_subscripts(a."points", 1) AS i
        WHERE i > 1 AND a."points"[i] <= a."points"[i - 1]) AS first_out_of_order
FROM tk.tabulated_axis a
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = a."function"
) loc ON true
WHERE EXISTS (
    SELECT 1 FROM generate_subscripts(a."points", 1) AS i
    WHERE i > 1 AND a."points"[i] <= a."points"[i - 1]
)
