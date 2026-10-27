# Apache Arrow JSON (arrow-json)

Realizations considered: `arrow-json` 59.3.0, present in the lockfile as a dependency of the
`arrow` umbrella crate (its `json` feature is unified in by another family member) but not a
declared direct dependency; upgrade route 60.0.0 (family
major; adds `TapeDecoder` `flatten_top_level_arrays`).
Research date 2026-10-27. Sources: installed `arrow-json-59.3.0/src/reader/mod.rs`
(`Decoder::serialize`); `Cargo.lock` entry; release notes 60.0.0.

## Types

### arrow-json/serde-rows-to-arrow
- **Need it serves:** turn `serde::Serialize` records into an Arrow `RecordBatch` with a declared
  schema, without hand-written builders and without an intermediate JSON text.
- **Recognizable code shapes:** records serialized to a JSON array and stored as an opaque payload
  beside typed Arrow relations; per-field builder code for small event structs.
- **Mechanisms and APIs:** `arrow_json::ReaderBuilder::new(schema)[.with_strict_mode(true)]
  .build_decoder()? -> Decoder`; `Decoder::serialize(&[S: Serialize])` feeds rows through the
  tape decoder; `Decoder::flush() -> Result<Option<RecordBatch>, ArrowError>`. The reverse direction for diagnosis is
  `arrow_json::writer::{LineDelimitedWriter, ArrayWriter}`.
- **Obligations discharged:** mapping serde data model to Arrow columns, null handling, type
  coercion with strict-mode refusal of unknown fields.
- **Residual caller obligations:** the schema itself; identity and canonical-equality rules of
  the records; size bounds (`serialize` ignores the batch size and decodes all rows passed).
- **Semantic conditions:** numbers pass through serde's data model; floats round-trip by value,
  and IEEE-bit identity for NaN payloads should be verified before relying on it.
- **Version / build conditions:** available at 59.3.0 through `arrow::json` when the `json`
  feature is enabled, or as `arrow-json` directly; versioned with the arrow family.
- **Limits and negative results:** not a canonical-JSON library; does not answer document
  equality.
- **Evidence:** Interface-checked: installed `reader/mod.rs` (`serialize` doc and signature).

## Current applicability

### [CBO-008](../../../library_leverage/codebase/result-data-paths.md#cbo-008) — Progress events and completion receipts stored as JSON payloads
- **Types:** `arrow-json/serde-rows-to-arrow`.
- **Realization:** installed 59.3.0 (resolved through the `arrow` umbrella; a direct use would declare `arrow-json` or the `json` feature at the family pin).
- **Conclusion:** unsettled, leaning ruled out. If progress became a relation, the codebase's own
  registry-generated `RelationRow` → Arrow builders are the single authority for relation
  encoding, so arrow-json would be a second encoder; arrow-json fits only if progress stays an
  unregistered, schema-declared side table. The choice hinges on whether `__progress` should be a
  registered relation (a domain decision, not a library question).
- **What it would discharge / what remains local:** with either Arrow route the JSON-array payload
  and its overflow-halving in `durable.rs` (progress writer) collapse into the ordinary batch
  splitting path; replay identity, document schemas consumed by tools and the canonical equality
  used by replay and the reproducer stay local. `decode_progress` bounds the event count only after
  `serde_json::from_slice` has materialized the whole array (input bytes are bounded upstream).
- **Unsettled premises:** why progress is a distinct payload (CBO research question); whether
  `BoundaryDocument` defines canonical equality (outside the Arrow scope of this cluster).
- **Evidence:** Interface-checked; application Proposed.
