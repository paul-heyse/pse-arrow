# object_store (`LimitStore`)

Realizations considered: installed `object_store` 0.13.2 (pinned family member).
Research date 2026-10-27. Sources: installed source `object_store-0.13.2/src/limit.rs`
(`LimitStore::new`, permit acquisition per method, `list` holding a permit for the stream).

## Types

### object-store/request-concurrency-limit
- **Need it serves:** bound concurrent requests to one store.
- **Mechanisms and APIs:** `LimitStore::new(inner, max_requests)` wraps any `ObjectStore`; each
  method acquires a permit from its private `Arc<Semaphore>`; streaming results (`get` payloads,
  `list`) keep an owned permit inside a `PermitWrapper` for the stream's lifetime.
- **Semantic conditions:** the semaphore is created inside `new` and is private: two `LimitStore`s
  never share a gate; a `list` stream holds its permit across consumer pauses; no re-entrancy, so
  nested wrapped calls on the same task each need a permit (risk of self-deadlock at low limits).
- **Limits and negative results:** no shared gate across stores, no nested-scope reuse, no release
  between list items.
- **Evidence:** Interface-checked: `limit.rs` at 0.13.2.

## Current applicability

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — I/O gate (`stores/admission.rs` `SharedIo`)
- **Types:** object-store/request-concurrency-limit.
- **Conclusion:** ruled out here — `SharedIo` deliberately shares one semaphore across every store,
  reuses an enclosing permit for nested calls through the `ACTIVE_IO` task-local lease chain, and
  releases between list items; `LimitStore` does none of these (the module comment already notes it
  owns a separate private gate).
- **Evidence:** Interface-checked; Proposed.
