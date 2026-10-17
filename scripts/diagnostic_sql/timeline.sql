-- Inclusive intervals overlap. Grouping locates work; it does not sum exclusive CPU cost.
SELECT t.id AS track_id, t.name AS owner, s.name AS activity,
       count(*) AS intervals, sum(s.dur) / 1e9 AS inclusive_seconds,
       max(s.dur) / 1e9 AS maximum_seconds
FROM slice s JOIN track t ON t.id = s.track_id
WHERE s.dur >= 0
GROUP BY t.id, t.name, s.name
ORDER BY inclusive_seconds DESC;
