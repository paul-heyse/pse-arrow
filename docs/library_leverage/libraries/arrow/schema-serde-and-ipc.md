# Arrow schema serde and IPC (arrow-schema, arrow-ipc, serde_arrow)

Realizations considered: installed `arrow-schema` =59.3.0 with feature `serde` (enabled in
`[workspace.dependencies]`), installed `arrow-ipc` =59.3.0; upgrade route `serde_arrow` 0.15.1
(released 2026-09-17, feature `arrow-59` available; not in the lockfile).
Research date 2026-10-27. Sources: arrow-schema 59.3.0 source (`src/field.rs` derive and tests,
`src/schema.rs`, `src/datatype.rs`); arrow-ipc 59.3.0 `src/convert.rs` (`metadata_to_fb`); crates.io
API for serde_arrow.

## Types

### arrow-schema/serde-schema-types
- **Need it serves:** persist or embed Arrow logical types, fields and schemas through serde.
- **Recognizable code shapes:** a generator that serializes `Field`/`DataType` via `serde_json` and a
  sort step to stabilize output; JSON slices rehydrated into `Field`.
- **Mechanisms and APIs:** feature `serde` on `arrow-schema` adds `#[derive(Serialize, Deserialize)]`
  to `Field`, `Schema`, `DataType` and related enums; works with binary formats (upstream tests
  round-trip `Field` through postcard).
- **Obligations discharged:** a complete serde mapping of Arrow schema types, version-matched to the
  crate.
- **Residual caller obligations:** deterministic ordering of `metadata: HashMap<String, String>`
  (derive writes hash order); format stability across Arrow majors (the derived shape follows the
  struct, including the deprecated `dict_id`).
- **Semantic conditions:** `Field` serde includes `dict_id` and `dict_is_ordered`; extension types
  travel only as metadata entries.
- **Version / build conditions:** serde shape may change on any Arrow major; tie regeneration to the
  family pin.
- **Limits and negative results:** no canonical ordering of metadata — byte-deterministic output needs
  a caller-side sort (for example `serde_json::Value::sort_all_objects`) or another encoding.
- **Evidence:** Interface-checked: arrow-schema 59.3.0 `src/field.rs` lines with
  `cfg_attr(feature = "serde", derive(...))` and `assert_binary_serde_round_trip`.

### arrow-ipc/deterministic-schema-and-batch-embedding
- **Need it serves:** embed tabular reference data and exact schemas as bytes with a canonical
  representation.
- **Recognizable code shapes:** `include_bytes!("x.arrow")` + `StreamReader` + `validate_full()`.
- **Mechanisms and APIs:** `arrow_ipc::writer::StreamWriter` / `FileWriter`; `reader::StreamReader`;
  `convert::metadata_to_fb` writes custom metadata with **sorted keys**; schema-only messages are
  possible.
- **Obligations discharged:** canonical, validated columnar bytes for `RecordBatch` and schemas,
  including deterministic metadata order.
- **Residual caller obligations:** mapping nested Rust structs to tables if non-tabular products are
  to travel this way; full validation on read when the bytes are untrusted.
- **Semantic conditions:** IPC preserves exact `DataType`, nullability and metadata.
- **Version / build conditions:** IPC format is stable across Arrow majors; Rust API is family-pinned.
- **Limits and negative results:** IPC is not a general object format; nested spec trees need a
  struct↔columns mapping.
- **Evidence:** Interface-checked: arrow-ipc 59.3.0 `src/convert.rs` `metadata_to_fb` sorts keys.

### serde-arrow/serde-structs-as-arrow-tables
- **Need it serves:** turn a `Vec<T: Serialize>` into Arrow arrays (and back) without hand-written
  builders, so heterogeneous spec rows can travel as IPC.
- **Recognizable code shapes:** hand-written row builders for registry/self-description tables.
- **Mechanisms and APIs:** `serde_arrow::to_record_batch` / `from_record_batch` with schema tracing
  (`SchemaLike::from_type`/`from_samples`); version features `arrow-53` … `arrow-60`.
- **Obligations discharged:** columnar mapping of nested structs, options, enums (as unions) and maps.
- **Residual caller obligations:** serde derives on the row types; choosing table splits for deeply
  nested trees; deterministic ordering of maps (same HashMap caveat).
- **Semantic conditions:** enum-as-union and map encodings follow serde_arrow conventions, not a
  domain schema.
- **Version / build conditions:** feature must match the pinned Arrow major.
- **Limits and negative results:** not researched beyond release metadata in this pass.
- **Evidence:** crates.io release metadata only (feature list). Not Interface-checked.

## Current applicability

### [CBO-040](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-040) — Registry products baked as ~87k lines of Rust plus JSONL plus IPC
- **Types:** arrow-schema/serde-schema-types; arrow-ipc/deterministic-schema-and-batch-embedding;
  serde-arrow/serde-structs-as-arrow-tables.
- **Realization:** installed at 59.3.0 for the first two; serde_arrow is an upgrade route.
- **Conclusion:** fits with conditions as a *companion* to a serde blob — IPC remains the right
  carrier for the fifteen self-description batches and is a deterministic carrier for the
  `ResolvedContractProducts::schemas` map (`SchemaRef`), where Arrow `Field` serde is not; "Arrow IPC
  for everything" is ruled out here because the bulk of the products are nested spec trees whose
  table mapping would be a new, larger translation layer.
- **What it would discharge / what remains local:** IPC: deterministic schema bytes (sorted metadata);
  serde derive: the field/datatype mapping. Local: the sort step for serde-encoded fields.
- **Prerequisites, configuration or extension:** none new for IPC; serde_arrow would add a dependency
  tied to the Arrow family pin.
- **Semantic gaps against the CBO's preservation constraints:** none for IPC; serde derive bytes are
  non-deterministic without a sort.
- **Unsettled premises and what would settle them:** whether `FieldContract` JSON slices (the current
  pooled JSONL) should stay JSON inside a binary blob or move to IPC schema messages — a probe could
  compare determinism and decode time.
- **Evidence:** Interface-checked (sources above). Application Proposed.
