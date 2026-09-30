-- invariant: datum.value_respects_scale
-- A datum holds a negative value, or limit, in a column of an observable whose quantity type has the scale `absolute`.
SELECT d.id, loc.locator, o.key AS observable, q.name AS quantity_type, d.value
FROM ev.datum d
JOIN ev.data_point p ON p.id = d.point
JOIN ev.dataset_column c ON c.id = d."column"
JOIN tk.observable o ON o.id = c.observable
JOIN meta.quantity_type q ON q.id = o.quantity
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin x
    JOIN prov.import_record i ON i.id = x.import_record
    WHERE x.record = p.dataset
) loc ON true
WHERE q.scale = 'absolute' AND d.value < 0
