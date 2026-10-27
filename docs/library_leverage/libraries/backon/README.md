# backon

Realizations considered: not in the workspace; upgrade route `backon` 1.6.0 (released 2025-10-18,
latest; source present in the local cargo registry). Default features include `std`,
`std-blocking-sleep`, `tokio-sleep` and a wasm-only `gloo-timers-sleep`.
Research date 2026-10-27. Sources: Context7 `/websites/rs_backon_1_6_0` (`Retry::when`,
`adjust`, `notify`, `sleep`); installed source `backon-1.6.0/src/retry.rs` (`Future for Retry`
poll loop; `adjust` signature `FnMut(&E, Option<Duration>) -> Option<Duration>`),
`src/backoff/exponential.rs` (jitter, `with_total_delay`, `without_max_times`), `Cargo.toml`.

## Types

### backon/retry-combinator
- **Need it serves:** re-run a fallible async operation, rebuilt from a factory each attempt, with a
  classification predicate and a pacing policy.
- **Recognizable code shapes:** `loop { match op().await { Err(e) if retryable(&e) && attempt < N
  => { sleep(backoff(attempt)).await; attempt += 1 } r => return r } }`.
- **Mechanisms and APIs:** `(|| op()).retry(builder)` via `Retryable`; `.when(FnMut(&E) -> bool)`;
  `.adjust(FnMut(&E, Option<Duration>) -> Option<Duration>)` overrides each delay (receives `None`
  when the backoff is exhausted, may return `Some` to continue, `None` to stop); `.notify(FnMut(&E,
  Duration))`; `.sleep(Sleeper)`; builders `ExponentialBuilder` (min/max delay, factor,
  `with_max_times`/`without_max_times`, `with_total_delay` for total *sleep*, `with_jitter`
  adding a random value in `(0, current_delay)`, `with_jitter_seed`), `ConstantBuilder`,
  `FibonacciBuilder`; `RetryWithContext` threads a value through attempts.
- **Obligations discharged:** loop state machine, sleeping, attempt bookkeeping, final error return.
- **Residual caller obligations:** classification, any per-class counting, attribution of the
  exhausted error, the overall deadline.
- **Semantic conditions:** the factory is called inside `Retry::poll`, so task-locals scoped around
  the whole retry future are visible to each attempt. Dropping the `Retry` future drops the current
  attempt or sleep — no detached retry. The backoff iterator advances on every retryable error
  (`backoff.next()` is called before `adjust`); "uncounted" retry classes need an `adjust` closure
  that keeps its own counters (making the builder a shell). No absolute deadline: `with_total_delay`
  bounds summed sleeps, not attempt time.
- **Version / build conditions:** pure Rust; `tokio-sleep` uses `tokio::time::sleep`.
- **Limits and negative results:** no per-attempt absolute deadline, no error-dependent counting,
  jitter is proportional to the current delay (not an additive fixed range).
- **Evidence:** Interface-checked: source and docs listed above.

## Current applicability

### [CBO-010](../../../library_leverage/codebase/persistence-and-transport.md#cbo-010) — Conflict retry assembled per site
- **Types:** backon/retry-combinator.
- **Realization:** upgrade route (new dependency, 1.6.0).
- **Conclusion:** fits with conditions — a retry future wrapped by the existing `within_clock`
  (`timeout_at` + task-local `OPERATION_DEADLINE` scope) preserves "the original clock bounds all
  attempts" and "no detached retry after cancellation"; each attempt still calls the build closure,
  so "each attempt rebuilds the complete decision" holds.
- **What it would discharge / what remains local:** the loop and sleep. Local: the classification
  (`is_conflict`, `is_capture_busy`, `retryable_stage`), the uncounted capture-busy class (an
  `adjust` closure with its own conflict counter over an unbounded constant builder), the
  `10ms << min(attempt,4)` + additive `0..64ms` pacing (backon's jitter differs), `exhaust_conflict`
  attribution via `map_err` after the retry, and `operation_failed` logging.
- **Semantic gaps:** none against the stated constraints; pacing shape differs unless reproduced in
  `adjust`, at which point the remaining library contribution is a ~15-line loop.
- **Unsettled premises:** whether the capture-busy retries are intentionally uncounted (they are
  bounded only by the clock) — this determines whether a library counter fits at all.
- **Evidence:** Interface-checked (backon source; repo `transport.rs`, `canonical.rs`,
  `canonical_staging.rs` loops); Proposed.

**Reviewer correction (2026-10-27).** There are ten retry loops with five pacing forms, not four of each; the additions are `begin_stage_inner` (no jitter), `cleanup_stage_query`, two in `canonical_retention.rs` (`yield_now`) and `acquire_source_stage` (fixed 20 ms). The `conflict()` predicate is duplicated in two modules, and three loops interleave receipt settlement between attempts, which a generic combinator does not model. The comparative review does not adopt backon: once the named policies keep today's caps and pacings, the library would contribute only the loop. It recommends one local `retry_conflicts(policy, operation, step -> Attempt::{Done, Conflict, Busy})` owner under the original clock, with one predicate replacing the duplicates. See the [comparative review](../../reviews/execution-lifecycle-and-observation.md#cbo-010).
