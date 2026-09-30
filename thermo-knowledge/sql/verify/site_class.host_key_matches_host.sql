-- invariant: site_class.host_key_matches_host
-- A site class's host key is not the identifier of the phase definition or material it names.
SELECT s.id, loc.locator, s.host_key, coalesce(s.phase, s.material)::text AS host
FROM tk.site_class s
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s.id
) loc ON true
WHERE s.host_key IS DISTINCT FROM coalesce(s.phase, s.material)::text
