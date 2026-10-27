# object_store (LimitStore)

Realizations considered: installed `object_store` 0.13.2; upgrade route 0.14.2 (released
2026-09-15; DataFusion `main` and arrow 60 move to it, so it travels with the arrow/DataFusion
family upgrade).
Research date 2026-10-27. Sources: installed `object_store-0.13.2/src/limit.rs`; `src/limit.rs`
at tag `v0.14.2` and its commit history (GitHub contents and commits API).

## Types

### object-store/limit-store
- **Need it serves:** cap the number of concurrent requests issued to one object store.
- **Recognizable code shapes:** an `ObjectStore` wrapper holding a `tokio::sync::Semaphore` and
  acquiring a permit per call; streaming results that keep a permit until drained.
- **Mechanisms and APIs:** `object_store::limit::LimitStore::new(inner, max_requests)`; implements
  every `ObjectStore` method by acquiring a permit from a private `Arc<Semaphore>`; `get_opts`
  attaches an `OwnedSemaphorePermit` to a `GetResultPayload::Stream` so the permit lives until the
  stream drops; `list`/`list_with_offset`/`delete_stream` wrap streams the same way; multipart
  uploads use `LimitUpload` with the same semaphore.
- **Obligations discharged:** a per-store concurrency cap with permit lifetime tied to response
  streams; drop-based release on cancellation.
- **Residual caller obligations:** sharing one budget across several stores; nested or re-entrant
  use (a wrapper that calls the store again while holding a permit can deadlock at the cap);
  fairness, timeouts, metrics; admission semantics beyond a count.
- **Semantic conditions:** the semaphore is created inside `new`, so two `LimitStore`s never share
  a gate and no constructor accepts an external `Semaphore`; for `GetResultPayload::File` (local
  filesystem) the permit is released when `get_opts` returns, not when the file is read;
  `acquire().await.unwrap()` assumes the semaphore is never closed.
- **Version / build conditions:** identical structure at 0.14.2 (`new(inner, max_requests)`
  constructing `Arc::new(Semaphore::new(..))`); last functional changes to `limit.rs` predate both.
- **Limits and negative results:** no shared-gate constructor and no re-entrancy at 0.13.2 or
  0.14.2 (search scope: whole `limit.rs` at both versions).
- **Evidence:** Interface-checked: installed source and v0.14.2 source.

## Current applicability

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — I/O gate part (`stores/admission.rs::SharedIo`)
- **Types:** `object-store/limit-store`.
- **Realization:** installed 0.13.2; 0.14.2 changes nothing relevant.
- **Conclusion:** ruled out here. `SharedIo` provides one gate across every store (its module doc:
  "One I/O gate across every store. Native `LimitStore` owns a separate private gate") and a
  task-local lease chain so nested wrappers borrow the same permit; `LimitStore` offers neither,
  and releases permits early for local-file payloads.
- **What it would discharge / what remains local:** nothing beyond what `SharedIo` already does;
  composing `LimitStore` underneath would add a second, independent cap.
- **Unsettled premises:** none consequential; an upstream constructor taking `Arc<Semaphore>` would
  cover the shared gate but still not re-entrancy.
- **Evidence:** Interface-checked.
