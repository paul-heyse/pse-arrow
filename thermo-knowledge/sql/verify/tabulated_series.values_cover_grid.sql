-- invariant: tabulated_series.values_cover_grid
-- A series has a number of values other than the product of the numbers of points of its function's axes.
-- The product of the axis lengths is an aggregate over the integers, exact where a sum of logarithms is not.
CREATE AGGREGATE pg_temp.product (bigint) (SFUNC = int8mul, STYPE = bigint, INITCOND = '1');
-- query
WITH grid AS (
    SELECT f.id AS "function", COALESCE(pg_temp.product(cardinality(a."points")::bigint), 1) AS points,
           COALESCE(string_agg(cardinality(a."points")::text, ' x ' ORDER BY a.ordinal), 'no axes') AS shape
    FROM tk.tabulated_function f
    LEFT JOIN tk.tabulated_axis a ON a."function" = f.id
    GROUP BY f.id
)
SELECT s.id, loc.locator, s.name, cardinality(s."values") AS "values", g.points AS grid_points, g.shape
FROM tk.tabulated_series s
JOIN grid g ON g."function" = s."function"
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = s."function"
) loc ON true
WHERE cardinality(s."values") <> g.points
