-- invariant: site_class.constant_ratio_present
-- A site class with a constant ratio has none or a non-positive one, or another kind states one.
SELECT s.id, loc.locator, s.ratio_kind::text AS ratio_kind, s.ratio
FROM tk.site_class s
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s.id
) loc ON true
WHERE (s.ratio_kind = 'constant' AND (s.ratio IS NULL OR s.ratio <= 0))
   OR (s.ratio_kind <> 'constant' AND s.ratio IS NOT NULL)
