-- invariant: pseudo_component.produced_by_matches_lineage
-- A pseudo-component names a producer that has no `derivation_output` row for it, or has a `derivation_output` row from a derivation it does not name.
SELECT p.id, loc.locator, p.produced_by::text AS produced_by, prod.derivations
FROM tk.pseudo_component p
LEFT JOIN LATERAL (
    SELECT string_agg(o.derivation::text, '; ' ORDER BY o.derivation::text) AS derivations
    FROM prov.derivation_output o
    WHERE o.record = p.id
) prod ON true
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin x
    JOIN prov.import_record i ON i.id = x.import_record
    WHERE x.record = p.id
) loc ON true
WHERE (p.produced_by IS NOT NULL AND NOT EXISTS (
          SELECT 1 FROM prov.derivation_output o WHERE o.record = p.id AND o.derivation = p.produced_by))
   OR (p.produced_by IS NULL AND EXISTS (SELECT 1 FROM prov.derivation_output o WHERE o.record = p.id))
   OR (p.produced_by IS NOT NULL AND EXISTS (
          SELECT 1 FROM prov.derivation_output o WHERE o.record = p.id AND o.derivation <> p.produced_by))
