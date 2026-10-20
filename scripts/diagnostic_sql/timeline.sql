-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
-- Bounds are source-clock half-open [a,b). Nesting gives inclusive work, not CPU.
-- The running MAX of ALL prior ends is needed for nested intervals (LAG is wrong).
WITH selected AS (
 SELECT s.id,s.track_id,s.ts,s.dur,s.name,t.name AS owner,
        max(s.ts, __START__) AS lo,min(s.ts+s.dur, __END__) AS hi
 FROM slice s JOIN track t ON t.id=s.track_id
 WHERE s.dur>0 AND s.ts<__END__ AND s.ts+s.dur>__START__
), preceding AS (
 SELECT *,max(hi) OVER(PARTITION BY track_id ORDER BY lo,hi,id
 ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING) AS preceding_hi FROM selected
), marked AS (
 SELECT *,CASE WHEN preceding_hi IS NULL OR lo>preceding_hi THEN 1 ELSE 0 END AS begins FROM preceding
), islands AS (
 SELECT *,sum(begins) OVER(PARTITION BY track_id ORDER BY lo,hi,id
 ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS island FROM marked
), unions AS (
 SELECT track_id,island,min(lo) AS lo,max(hi) AS hi FROM islands GROUP BY track_id,island
), union_totals AS (
 SELECT track_id,sum(hi-lo) AS union_ns FROM unions GROUP BY track_id
), totals AS (
 SELECT track_id,owner,count(*) AS intervals,sum(dur) AS original_inclusive_ns,
 sum(hi-lo) AS clipped_inclusive_ns,min(ts) AS first_original_ts,max(ts+dur) AS last_original_end
 FROM selected GROUP BY track_id,owner
), points AS (
 SELECT track_id,sum(CASE WHEN dur=0 AND ts>=__START__ AND ts<__END__ THEN 1 ELSE 0 END) AS instants,
 sum(CASE WHEN dur<0 AND ts<__END__ THEN 1 ELSE 0 END) AS open_intervals FROM slice GROUP BY track_id
), tracks AS (SELECT track_id FROM totals UNION SELECT track_id FROM points WHERE instants>0 OR open_intervals>0)
SELECT tracks.track_id,t.name AS owner,coalesce(totals.intervals,0) AS intervals,
 coalesce(original_inclusive_ns,0) AS original_inclusive_ns,coalesce(clipped_inclusive_ns,0) AS clipped_inclusive_ns,
 coalesce(union_ns,0) AS union_ns,first_original_ts,last_original_end,
 coalesce(instants,0) AS instants,coalesce(open_intervals,0) AS open_intervals
FROM tracks JOIN track t ON t.id=tracks.track_id
LEFT JOIN totals ON totals.track_id=tracks.track_id LEFT JOIN union_totals ON union_totals.track_id=tracks.track_id
LEFT JOIN points ON points.track_id=tracks.track_id ORDER BY tracks.track_id;
