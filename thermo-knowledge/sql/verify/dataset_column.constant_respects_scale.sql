-- invariant: dataset_column.constant_respects_scale
-- A constraint column states a negative constant for an observable whose quantity type has the scale `absolute`.
SELECT c.id, loc.locator, o.key AS observable, q.name AS quantity_type, c.constant
FROM ev.dataset_column c
JOIN tk.observable o ON o.id = c.observable
JOIN meta.quantity_type q ON q.id = o.quantity
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin x
    JOIN prov.import_record i ON i.id = x.import_record
    WHERE x.record = c.dataset
) loc ON true
WHERE q.scale = 'absolute' AND c.constant < 0
