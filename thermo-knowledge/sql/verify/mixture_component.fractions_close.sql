-- invariant: mixture_component.fractions_close
-- The fractions of the components of a defined mixture do not sum to one within 1e-6.
SELECT v.mixture AS id, loc.locator, v.components, v.total
FROM (
    SELECT c.mixture, count(*) AS components, sum(c.value) AS total
    FROM tk.mixture_component c
    GROUP BY c.mixture
    HAVING abs(sum(c.value) - 1) > 1e-6
) v
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = v.mixture
) loc ON true
