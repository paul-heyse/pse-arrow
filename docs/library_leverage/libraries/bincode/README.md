# bincode

Realizations considered: installed `bincode` 2.0.1 (present in `Cargo.lock` transitively; also
1.3.3 in the local registry); crates.io 3.0.0 (2025-12-16) is a tombstone release.
Research date 2026-10-27. Sources: bincode 3.0.0 crate tarball (`README.md`, `src/lib.rs`),
crates.io API.

## Types

### bincode/unmaintained-positional-binary
- **Need it serves:** positional binary serde encoding (same niche as postcard).
- **Recognizable code shapes:** as postcard/embedded-serde-blob.
- **Mechanisms and APIs:** 2.x offers serde mode and its own `Encode`/`Decode`/`BorrowDecode`
  derives with explicit configuration (endianness, varint/fixint).
- **Obligations discharged:** as postcard for compile-time-known shapes.
- **Residual caller obligations:** as postcard; plus owning the maintenance risk.
- **Semantic conditions:** non-self-describing (no `deserialize_any` in serde mode).
- **Version / build conditions:** **the project has ceased.** The 3.0.0 crate contains only
  `compile_error!("https://xkcd.com/2347/");` and a README stating that development has ceased and no
  further releases will be published; it recommends `wincode`, `postcard` or `rkyv`.
- **Limits and negative results:** no future fixes; 2.0.1 remains usable but frozen.
- **Evidence:** Interface-checked: bincode 3.0.0 tarball contents (downloaded to cluster scratch
  `dl/bincode3/`).

## Current applicability

### [CBO-040](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-040) — Registry products baked as ~87k lines of Rust plus JSONL plus IPC
- **Types:** bincode/unmaintained-positional-binary.
- **Realization:** installed transitive 2.0.1; no upgrade route exists.
- **Conclusion:** ruled out here — postcard provides the same capability with a specified, maintained
  format; choosing a ceased project for a regeneration-critical format adds risk for no capability.
- **What it would discharge / what remains local:** n/a.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** none distinctive.
- **Unsettled premises and what would settle them:** none.
- **Evidence:** Interface-checked (tombstone release). Application Proposed (negative).
