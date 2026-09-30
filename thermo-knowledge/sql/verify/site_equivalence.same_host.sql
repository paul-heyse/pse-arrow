-- invariant: site_equivalence.same_host
-- Two site classes declared equivalent have different host keys.
SELECT r.id, loc.locator, a.host_key AS host_a, b.host_key AS host_b
FROM tk.site_equivalence r
JOIN tk.site_class a ON a.id = r.a
JOIN tk.site_class b ON b.id = r.b
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = r.a
) loc ON true
WHERE a.host_key <> b.host_key
