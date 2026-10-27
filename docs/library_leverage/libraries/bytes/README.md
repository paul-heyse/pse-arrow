# bytes

Realizations considered: installed `bytes` 1.12.1 (latest).
Research date 2026-10-27. Sources: installed source `bytes-1.12.1/src/buf/writer.rs`
(`impl io::Write for Writer<B>`), `src/buf/limit.rs` (`remaining_mut`), `src/buf/buf_mut.rs`.

## Types

### bytes/bufmut-limit-writer
- **Need it serves:** cap the bytes written into an in-memory buffer through `io::Write`.
- **Mechanisms and APIs:** `buf.limit(n)` → `Limit<B>` (`remaining_mut = min(inner, limit)`),
  `.writer()` → `Writer<B>: io::Write`; `into_inner()` recovers the buffer.
- **Semantic conditions:** `Writer::write` copies `min(remaining_mut, src.len())` and returns that
  count — a *short write*, then `Ok(0)`. The refusal surfaces only through `write_all`'s generic
  `ErrorKind::WriteZero` ("failed to write whole buffer"); the truncated prefix stays in the buffer.
- **Limits and negative results:** no typed limit error, no counting-only mode, no forwarding to a
  file, no per-write checkpoint hook; callers using `write` rather than `write_all` see silent
  truncation.
- **Evidence:** Interface-checked: source files at 1.12.1.

## Current applicability

### [CBO-002](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-002) — Bounded and counting output sinks
- **Types:** bytes/bufmut-limit-writer.
- **Realization:** installed.
- **Conclusion:** ruled out here — the limit refusal must be distinguishable from I/O errors, and
  most sinks count without storing or forward to a file; `WriteZero` is not a typed refusal.
- **Evidence:** Interface-checked; Proposed.
