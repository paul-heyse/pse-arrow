-- invariant: validity_coverage.state_matches_regions
-- A coverage row says `stated` where the record has no region of the kind or `not_stated` where it has one, or a record has a region of a kind and no coverage row.
WITH pairs AS (
    SELECT c.id AS coverage, c.record, c.kind, c.value::text AS state FROM tk.validity_coverage c
    UNION
    SELECT NULL::uuid, r.record, r.kind, NULL::text FROM tk.validity_region r
), counted AS (
    SELECT DISTINCT ON (p.record, p.kind) p.coverage, p.record, p.kind, p.state,
           (SELECT count(*) FROM tk.validity_region r WHERE r.record = p.record AND r.kind = p.kind) AS regions
    FROM pairs p
    ORDER BY p.record, p.kind, p.coverage NULLS LAST
)
SELECT coalesce(v.coverage, v.record) AS id, loc.locator, v.record, v.kind::text AS kind, v.state, v.regions
FROM counted v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.record
) loc ON true
WHERE v.state IS DISTINCT FROM CASE WHEN v.regions > 0 THEN 'stated' ELSE 'not_stated' END
