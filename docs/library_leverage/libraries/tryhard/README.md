# tryhard

Realizations considered: not in the workspace; upgrade route `tryhard` 0.5.2 (released 2025-06-23,
latest).
Research date 2026-10-27. Sources: docs.rs `RetryFuture` page (0.5.2); crate source 0.5.2
downloaded to scratch (`dl/tryhard/tryhard-0.5.2/src/lib.rs`, `backoff_strategies.rs`).

## Types

### tryhard/retry-with-error-aware-backoff
- **Need it serves:** retry a future factory a fixed number of times with a backoff that can see
  the error.
- **Mechanisms and APIs:** `retry_fn(f).retries(n)` then `.exponential_backoff(d)`,
  `.fixed_backoff`, `.linear_backoff`, `.no_backoff`, `.max_delay(d)`,
  `.custom_backoff(BackoffStrategy)` where `delay(&mut self, attempt: u32, error: &E)` returns a
  `Duration` or `RetryPolicy::{Delay(Duration), Break}`; `.on_retry(Fn(u32, Option<Duration>, &E)
  -> Fut)`.
- **Semantic conditions:** `attempts_remaining` decrements on every retry (no uncounted class);
  `Break` stops with the last error; the `on_retry` future is passed to `tokio::spawn` (detached
  side work that outlives cancellation of the retry). Tokio-only.
- **Limits and negative results:** no deadline support; no separate retry predicate (classification
  happens in `custom_backoff`).
- **Evidence:** Interface-checked: source at 0.5.2.

## Current applicability

### [CBO-010](../../../library_leverage/codebase/persistence-and-transport.md#cbo-010)
- **Types:** tryhard/retry-with-error-aware-backoff.
- **Conclusion:** fits with conditions, weaker than backon — classification and pacing fit in one
  `custom_backoff`, but capture-busy retries would consume the attempt budget, and `on_retry`
  must not be used where "no detached work after cancellation" applies.
- **Evidence:** Interface-checked; Proposed.
