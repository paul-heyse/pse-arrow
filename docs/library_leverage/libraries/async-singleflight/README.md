# async_singleflight

Realizations considered: not in the workspace; upgrade route `async_singleflight` 0.6.2
(released 2026-02-09).
Research date 2026-10-27. Sources: crate source 0.6.2 downloaded to scratch
(`dl/asf/async_singleflight-0.6.2/src/unary.rs`, `group.rs`).

## Types

### async-singleflight/leader-follower-dedup
- **Need it serves:** suppress duplicate concurrent work per key, Go `singleflight` style.
- **Mechanisms and APIs:** `Group::work(key, fut)` / `UnaryGroup::work`, `work_no_retry`,
  `purge_stale`; `GroupWorkError::{LeaderDropped, …}`.
- **Semantic conditions:** the first caller is the leader and runs its future in its own task.
  If the leader is dropped, followers observe `State::LeaderDropped` and one is promoted to run
  *its own* future (`work`), or they return `None` (`work_no_retry`). Not spawned onto a runtime.
- **Limits and negative results:** no producer survival, no last-waiter cancellation, no capacity
  limits; promoted-leader results can linger until `purge_stale`.
- **Evidence:** Interface-checked: source at 0.6.2.

## Current applicability

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011)
- **Types:** async-singleflight/leader-follower-dedup.
- **Conclusion:** ruled out here — same caller-owned producer model as moka/quick_cache.
- **Evidence:** Interface-checked; Proposed.
