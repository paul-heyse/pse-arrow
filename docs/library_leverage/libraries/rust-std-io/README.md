# Rust standard library I/O adapters and typed I/O errors

Realizations considered: pinned nightly toolchain (`rustc 1.101.0-nightly 2026-10-23` reported;
`rust-toolchain.toml` owns the pin); stable features below are long available. Interacting
libraries: `serde_json` 1.0.151, `arrow-schema` 59.3.0 (installed).
Research date 2026-10-27. Sources: toolchain source `library/alloc/src/io/error.rs`
(`#[stable(feature = "io_error_downcast", since = "1.79.0")] pub fn downcast<E>`);
`serde_json-1.0.151/src/error.rs` (`impl From<Error> for io::Error` returns the original
`io::Error` for `ErrorCode::Io`); `arrow-schema-59.3.0/src/error.rs` (`ArrowError::IoError(String,
std::io::Error)`, `source()` returns the inner `io::Error`).

## Types

### rust-std-io/typed-error-through-io-channel
- **Need it serves:** carry a typed refusal (limit reached, cancelled, expired) out of an
  `io::Write` sink through a serializer or encoder that only speaks `io::Error`, and recognise it
  afterwards without string matching.
- **Recognizable code shapes:** `io::Error::other("…limit")` in a sink and later
  `matches!(e, …IoError(msg, _) if msg == "…limit")`; sinks recording an `exceeded: bool` side flag.
- **Mechanisms and APIs:** `io::Error::other(MarkerType)` (or `io::Error::new(kind, T)`);
  recovery with `err.get_ref().and_then(|e| e.downcast_ref::<MarkerType>())` or
  `io::Error::downcast::<MarkerType>()` (stable 1.79). `serde_json::Error` → `io::Error` returns the
  original sink error; `ArrowError::IoError` retains the `io::Error` as its second field and as
  `source()`.
- **Obligations discharged:** type-preserving transport of the refusal through intermediate layers
  that keep the `io::Error`.
- **Residual caller obligations:** defining the marker type, unwrapping each intermediate error type
  to its `io::Error`, ensuring no layer re-creates the error from its message.
- **Semantic conditions:** works only where intermediate layers keep the original `io::Error`
  object; a layer that formats the error into a string loses it.
- **Limits and negative results:** std has no bounded or counting `Write` adapter (`Read::take`
  exists only for readers; `io::sink()` discards without counting; `Cursor<&mut [u8]>` caps only a
  preallocated buffer, with `WriteZero`).
- **Evidence:** Interface-checked: sources listed above.

## Current applicability

### [CBO-002](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-002) — Bounded and counting output sinks re-implemented per site
- **Types:** rust-std-io/typed-error-through-io-channel (plus bytes.md negative).
- **Realization:** toolchain and installed pins.
- **Conclusion:** fits with conditions for the brittle discrimination — `result_blocks.rs`'s string
  match on `"result block limit"` through `ArrowError::IoError` can become a typed marker recovered
  by downcast, because Arrow 59.3.0 retains the `io::Error`. No library writer adapter found that
  covers the sinks; the consolidation remedy is a local generic sink.
- **What it would discharge / what remains local:** std discharges typed transport. Local: one
  `LimitedWriter<W>` (count-only, `Vec`, or forwarding `File`) with an optional per-write checkpoint
  closure (cancel or expiry; present in at least five sinks) and a typed `LimitReached` marker;
  hashing sinks can wrap `blake3::Hasher` / `sha2` writers in the same adapter.
- **Semantic gaps:** sinks that hash (`EncodingHasher`, `DigestWriter`, framed `PartWriter`) are
  different responsibilities; framed `PartWriter` keeps a sticky failure state.
- **Count sharpening:** at the baseline a broad pattern finds 32 `io::Write` and 6 `fmt::Write`
  implementations in non-test files under `crates/` (the CBO says 36 `io::Write`).
- **Unsettled premises:** Arrow IPC size prediction to avoid trial encoding is outside this cluster
  (Arrow assessment).
- **Evidence:** Interface-checked; Proposed. A probe could confirm the downcast through the actual
  `FileWriter`/`StreamWriter` path (README).
