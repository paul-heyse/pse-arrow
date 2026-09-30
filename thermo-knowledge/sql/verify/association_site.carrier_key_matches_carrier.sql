-- invariant: association_site.carrier_key_matches_carrier
-- An association site's carrier key is not the identifier of the entity or group it names.
SELECT s.id, loc.locator, s.carrier_key, coalesce(s.on_entity, s.on_group)::text AS carrier
FROM tk.association_site s
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s.id
) loc ON true
WHERE s.carrier_key IS DISTINCT FROM coalesce(s.on_entity, s.on_group)::text
