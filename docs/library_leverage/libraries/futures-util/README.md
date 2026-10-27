# futures-util (`Shared`)

Realizations considered: installed `futures-util` 0.3.34.
Research date 2026-10-27. Sources: futures-util 0.3 `future::Shared` documentation
(`FutureExt::shared`, `Shared::{strong_count, weak_count, downgrade, peek}`); repository use at
baseline (`pse-columnar/src/flight.rs`) for context.

## Types

### futures-util/shared-completion
- **Need it serves:** let several awaiters observe one future's output (`Output: Clone`).
- **Recognizable code shapes:** waiter lists with oneshot senders; `Arc<Mutex<Option<T>>>` plus
  notify; keyed tables of in-progress loads.
- **Mechanisms and APIs:** `fut.shared()` → `Shared<F>` (clonable); `strong_count`, `weak_count`,
  `downgrade` → `WeakShared`, `peek`.
- **Obligations discharged:** result fan-out and waker bookkeeping for all clones.
- **Residual caller obligations:** keying, admission limits, error sharing (`Arc<E>`), last-waiter
  policy.
- **Semantic conditions:** the inner future is driven by whichever clone is polled; when every
  clone is dropped the inner future is dropped. Producer survival beyond its awaiters therefore
  requires running the producer elsewhere (for example a spawned task feeding a oneshot that the
  `Shared` wraps). No drop callback exists, so "last waiter left" needs a caller-side guard.
- **Limits:** not a single-flight table; no capacity, retirement or cancellation signalling.
- **Evidence:** Interface-checked: futures-util `Shared` docs (0.3 line).

## Current applicability

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — Completion-owned single flights
- **Types:** futures-util/shared-completion.
- **Realization:** installed (already used).
- **Conclusion:** already adopted to its useful extent — `flight.rs` combines `tokio::spawn`
  (producer survives waiter cancellation), a `oneshot` wrapped in `Shared` (fan-out), an atomic
  waiter count with a drop guard (last-waiter cancel) and a retiring state. `strong_count` could
  replace the waiter `AtomicUsize` only partially (the table holds a clone and no drop hook exists).
- **Evidence:** Interface-checked; Proposed.
