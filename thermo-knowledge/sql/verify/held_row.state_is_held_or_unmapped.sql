-- invariant: held_row.state_is_held_or_unmapped
-- A row that was not loaded is recorded in a state other than `held` or `unmapped`.
SELECT h.id, h.locator, h.manifest_id, h.source_table, h.state::text AS state
FROM qual.held_row h
WHERE h.state::text NOT IN ('held', 'unmapped')
