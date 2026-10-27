# ciborium (CBOR)

Realizations considered: installed `ciborium` 0.2.2 (present in `Cargo.lock` as a transitive
dependency; not declared in `[workspace.dependencies]`); 0.2.2 is the latest release (2024-01-24).
Research date 2026-10-27. Sources: ciborium 0.2.2 source (`src/de/mod.rs`, `src/ser/mod.rs`);
crates.io API.

## Types

### ciborium/self-describing-binary-serde
- **Need it serves:** a compact binary serde format that, unlike positional formats, is
  self-describing, so dynamic values and serde features that need lookahead still work.
- **Recognizable code shapes:** a binary blob need where some members are `serde_json::Value`,
  flattened, untagged or internally tagged.
- **Mechanisms and APIs:** `ciborium::ser::into_writer(&value, writer)`;
  `ciborium::de::from_reader::<T: DeserializeOwned, R: Read>` (also `from_reader_with_buffer`,
  `from_reader_with_recursion_limit`); `ciborium::Value` for dynamic CBOR.
- **Obligations discharged:** encoding/decoding the full serde data model including
  `deserialize_any` (so `serde_json::Value`, flatten and untagged work).
- **Residual caller obligations:** deterministic map order (ciborium writes maps in iteration order;
  it does not implement RFC 8949 deterministic/canonical encoding); owned strings only.
- **Semantic conditions:** `is_human_readable()` is `false`; floats are written via `Header::Float`
  (ciborium-ll chooses the shortest lossless width), so a given value has a stable encoding; map key
  order is caller-determined; decoding requires `DeserializeOwned`, so borrowed `&str`/`&'static str`
  fields cannot be decoded.
- **Version / build conditions:** last release January 2024; maintenance activity low.
- **Limits and negative results:** no canonical-CBOR mode; no zero-copy borrowing.
- **Evidence:** Interface-checked: ciborium 0.2.2 source (`from_reader` bounds; `serialize_f64`;
  `is_human_readable`).

## Current applicability

### [CBO-040](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-040) — Registry products baked as ~87k lines of Rust plus JSONL plus IPC
- **Types:** ciborium/self-describing-binary-serde.
- **Realization:** installed transitive crate at 0.2.2; adoption means declaring it in
  `[workspace.dependencies]` at `=0.2.2` (no family move).
- **Conclusion:** fits with conditions — it removes the postcard restriction on
  `serde_json::Value` members, but costs the `&'static str` fields (must become owned or leaked) and
  still needs HashMap ordering handled.
- **What it would discharge / what remains local:** as for postcard (single blob replaces source
  literals + JSONL); additionally carries `descriptions` values natively. Local: hydrate, IPC
  self-description tables, Arrow `Field` metadata ordering.
- **Prerequisites, configuration or extension:** serde derives on all product types (absent today);
  `&'static str` → `String`/`Cow<'static, str>` or `Box::leak` at hydration.
- **Semantic gaps against the CBO's preservation constraints:** none beyond ordering; identity
  values are byte arrays and survive.
- **Unsettled premises and what would settle them:** as for postcard (probe P1).
- **Evidence:** Interface-checked (source as above). Application Proposed.
