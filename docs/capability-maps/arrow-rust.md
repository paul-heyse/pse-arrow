---
status: evidence-map
blueprint_revision: 5
pins: Cargo.lock
regenerated: null
reviewed: 2026-09-13
---

# Arrow (Rust) — capability map

**Current binding:** blueprint revision 5 and ADR-0039–ADR-0048. The inventories and
original extraction/probe receipts below retain their historical scope and dates;
this update did not regenerate that corpus. Historical comparisons to revision 2
are not current acceptance claims. Corrected interpretations are recorded in place.
The current design remains **Proposed**; pinned library mechanisms are
**Interface-checked**, and only the named characterization experiments are **Tested**.

`[review:E1]`–`[review:E6]` refer to the [retained six-group characterization](../design_review/evidence/blueprint-rev4-2026-09-13/README.md)
and its conditions: Rust 1.98.1, Arrow 59.3.0, DataFusion 55.1.0, dev,
Arrow force_validate, zero failure baseline. The [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md)
explains the counterexamples; these markers do not certify a platform implementation.

| Current decision | Library mechanism and boundary | Evidence / adoption gate |
|---|---|---|
| Active semantic admission | Recursive `try_extension_type` / generated validators; registry factories are explicitly invoked | ADR-0039; [review:E1]; actual query/import rejection remains a platform gate |
| Canonical content and encoded integrity | Recursive typed normalization plus framed metadata/data IPC streams; separate stored-byte checksums | ADR-0045; [review:E4]; v2 metamorphic/round-trip fixtures remain open |
| Loss-aware conversion | Arrow casts only after exact representability and full quantity/conversion checks | ADR-0039; [review:E3]; castability alone is insufficient |
| Safe buffers and bounded transfer | `PrimitiveArray::values` / owning `ScalarBuffer`; eligible slices are contiguous; `BatchCoalescer` for bounded transfers | ADR-0047/0048; review L7; lifetime tests and R-24 measurement |
| Shared filters and explicit-schema readers | Arrow filters, Parquet projection/row filters, CSV/JSON `ReaderBuilder::new(schema)` | ADR-0048; review L3/L9; shared predicate correctness now, R-24 for storage optimization |

**Companion to** `docs/authoritative_design/blueprint.md` and to `supporting-rust-libraries.md`, which covered the §3.3 supporting crates and deliberately excluded Arrow and DataFusion. The DataFusion half of this pair is `datafusion-rust.md`. **Each map now carries its own principle-alignment register (§11) and its own acceptance-gate review (§12)**; in the previous edition both lived only in the DataFusion map, which meant neither document could be updated on its own.

**Compiled** 2026-09-13 against **`arrow` 59.3.0**.
**Historical survey adjudicated** blueprint **revision 2** (2026-09-13). The previous edition of this map adjudicated revision 1 and never said so; §0.4 lists what revision 2 changed underneath it.

---

## 0. Purpose, sourcing constraint, and evidence rules

### 0.1 What this document is

Per crate cluster: what Arrow can actually do, which of its APIs the blueprint's stated requirements land on, what we deliberately will not use, and where the blueprint assumes a capability that does not exist or is under-specified. A second pass asks a different question — what does Arrow offer that the blueprint has *not* claimed, which would improve alignment with `design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01…DM-60, gates G1–G7)?

### 0.2 The sourcing constraint, and how it is enforced

This document was written **without using any of the repository's existing Arrow or DataFusion material** — not the `docs/library_ref/` prose references, not the `datafusion-pyarrow-*` skills, not the pre-existing `build/facts/arrow59-default` surface (which is at 59.2.0 and would misrepresent the pin), and not the vendored `arrow-rs/` checkout.

Two of those are now moot rather than merely avoided: **`docs/library_ref/` has been deleted**, and the vendored checkouts survive only as orphaned git worktrees under `.worktrees/` whose parent repositories are gone, so their tags can no longer be verified. The constraint is now structural as well as chosen.

Everything here comes from four fresh lanes:

| Lane | Marker | What it is |
|---|---|---|
| context7 | `[c7:/id]` | `/websites/rs_arrow_arrow` (docs.rs mirror, 12171 snippets). **Never `/apache/arrow`** — that entry is the multi-language project at `apache_arrow_19_0_0`, i.e. the C++/Python line, and citing it would be a category error. |
| docs.rs | `[docs.rs:crate@version]` | version-addressed pages |
| **locally generated rustdoc JSON** | `[rustdoc:crate@version]` | the authoritative lane — see the receipt in §1 |
| **locally run probe program** | `[probe]` | a small Rust binary compiled against the pinned crates, used where documentation asserts behaviour that can simply be *measured* |
| upstream source at a release tag | `[gh:repo@tag/path]` | workspace manifests, feature lists and changelogs read at the exact pinned tag — used for the §10 enumerations, where a *complete* list matters and rustdoc gives only what was extracted |
| crates.io registry API | `[crates.io:crate]` | release dates and current-version checks |

The constraint is mechanically checked by counting **citations**, not mentions: no provenance marker in this document is of the form `[ref:…]` (the companion map's marker for local references) and none cites `/apache/arrow`. Both strings appear in this section and in the evidence ledger, where they record the rule and the rejected candidate respectively — that is the intended behaviour of the check, which greps for the bracketed citation forms. A claim carrying no marker is a defect.

### 0.3 Status vocabulary

**leverage** (the library provides it, use it) · **build on top** (foundation only; platform code required, cost stated) · **confirmed** (a specific blueprint claim is true as stated) · **refined** / **erratum** (claim wrong or imprecise; correction given) · **adopt** / **evaluate** / **reject** (Pass-2 recommendations) · **open item** (a question this lookup surfaced).

Pass-2 recommendations are disciplined by charter §G (common false positives) and **DM-58** (scale architectural machinery to demonstrated needs): a recommendation that names no principle and replaces no identified hand-written code does not appear.

**Pass 3** (§10) is new in this edition and answers a third question: taking the *complete* inventory of the family — every crate, every public trait, every feature flag, every release delta — what is this design not leveraging at all?

### 0.4 What changed underneath this map, and what it got wrong

The previous edition was compiled at 14:35 on 2026-09-13; the blueprint was rewritten at 15:24. Four things follow.

| # | Change | Consequence for this map |
|---|---|---|
| 1 | Blueprint **revision 2** absorbed this map's amendment **A** and must-settle items **#4** and **#6** verbatim — family-wide `=` pins, the `cargo tree -d` blind spot, the `ipc`/`ffi`/`canonical_extension_types` feature set, and dropping `arrow-avro` | §1.2's erratum is **resolved**; it is retained as a record, not a recommendation. §15 now marks every amendment *applied* / *parked in §26* / *new*. |
| 2 | Blueprint line 7 cites "the three capability maps" as a **library authority** | The citation was circular while the maps adjudicated revision 1. Naming the revision (header) closes it. |
| 3 | The rustdoc extraction behind the previous edition's 32 `[rustdoc:]` citations **no longer exists on disk** | Every marker was unreproducible. §1.4's new extraction is committed with its lockfile so this cannot recur. |
| 4 | Coverage was **13 of 28** arrow-family crates, and the map said so only implicitly | §1.4 now extracts **20** publishable crates, and §10 states the denominator explicitly. |

**And one correction to this document's own earlier reasoning.** The previous edition asserted, under A1's "Gaps and risks" and as register row #8, that because `Schema.metadata` is a `HashMap` the canonical IPC encoding is non-reproducible unless the platform sorts metadata keys first. **That is wrong for IPC.** Measured across six independently constructed schemas carrying 0, 1, 2, 3 and 5 metadata keys, the IPC byte stream was **identical every time** `[probe]`. Arrow's IPC writer does not inherit the `HashMap`'s iteration order. The concern is real, but it belongs to two *other* paths — a platform-side fingerprint that iterates the map directly, and `datafusion-proto`, which does inherit it and is measurably unstable (companion map D9). §2 and §10 carry the corrected finding.

---

## 1. Version anchors, pin discipline, and the extraction receipt

### 1.1 Anchors

| Component | This document | Blueprint §3.1 (rev 2) | Note |
|---|---|---|---|
| `arrow` and the whole `arrow-*` family | **59.3.0** (2026-09-01) | **59.3.0, each crate `=`-pinned** | **Now agreed.** Revision 2 raised the pin and adopted family-wide `=` pinning. 59.3.0 is still the current release as of this compile `[crates.io:arrow]`, so no re-pin is needed. |
| `parquet` and the `parquet-variant*` crates | **59.3.0** | `parquet` named; the variant crates are not | moves with the family; see §10 E1 |
| `pyo3` / `pyo3-arrow` | 0.29 / **0.19.0** | same | `pyo3-arrow 0.19.0` requires `arrow-* ^59`, so 59.3.0 is compatible with no change `[docs.rs:pyo3-arrow@0.19.0]` |
| `object_store` | **0.13.2** | 0.13.2 | resolved by cargo from DataFusion's requirement — §3.1's pin is correct |
| **What DataFusion 55.1.0 itself builds against** | `arrow = "59.2.0"` (caret) | not recorded | **New fact.** The DataFusion 55.1.0 workspace declares `arrow = "59.2.0"` `[gh:apache/datafusion@55.1.0/Cargo.toml]`. A caret requirement admits 59.3.0, so the `=59.3.0` pin resolves cleanly — but the platform runs one minor release **ahead of the Arrow version DataFusion is tested against**. Not a defect; a fact §3.1 should record, because it is the first place to look when an Arrow-level behaviour change surprises the engine. |

### 1.2 Pin-discipline erratum — §3.1 — **resolved in revision 2**

Retained as a record of *why* §3.1 reads as it now does, not as a live recommendation. Revision 2 adopted this finding verbatim, including the `cargo tree -d` reasoning.

Revision 1 of §3.1 stated the intent well: "*`arrow`, `arrow-*`, `parquet`, `arrow-flight`, `arrow-avro` | 59.2.0 | One release family; never mix Arrow majors in the graph (`cargo tree -d` in CI)*". **The stated mechanism does not achieve the stated intent.**

Resolving a project that pins only the umbrella crate:

```text
arrow        = "=59.2.0"     ->  arrow        59.2.0
                                 arrow-array  59.3.0   <-- floated
                                 arrow-schema 59.3.0   <-- floated
```

The umbrella depends on `arrow-* ^59.2.0`, so every sub-crate floats to the newest compatible release. This is not a *duplicate*, so **`cargo tree -d` does not catch it** — it is a silently mixed family, which is what §3.1 says it wants to prevent. The same applies to DataFusion, where the umbrella at 55.0.0 resolves sub-crates at 55.1.0.

**Recommended amendment (now applied):** pin every crate in both families with `=`, or commit a `Cargo.lock` and enforce it in CI. This document's extraction does **both** — every crate `=`-pinned *and* the resolved `Cargo.lock` committed under `evidence/rust/` — which is why its receipt shows a single version per family. Committing the lockfile is the stronger half and §3.1 still does not require it: `=` pins bind direct dependencies, but nothing stops a *transitive* path from pulling a second Arrow version, and only a lockfile plus a CI assertion catches that.

### 1.3 Feature flags (fresh, from crates.io/docs.rs)

`arrow` 59.3.0 is an umbrella over required `arrow-arith`, `arrow-array`, `arrow-buffer`, `arrow-cast`, `arrow-data`, `arrow-ord`, `arrow-row`, `arrow-schema`, `arrow-select`, `arrow-string`, plus optional `arrow-csv`, `arrow-ipc`, `arrow-json`, **`arrow-pyarrow`** `[docs.rs:arrow@59.3.0]`.

The complete feature list of the `arrow` umbrella at 59.3.0, so §10's E3 enumeration has a stated denominator `[gh:apache/arrow-rs@59.3.0/arrow/Cargo.toml]`:

| Feature | Default | §3.1 rev 2 | Bearing on this design |
|---|---|---|---|
| `ipc` | ✅ | named | §5.3 canonical serialization, §20.1 hot artifacts — **required** |
| `csv`, `json` | ✅ | — | not used for platform data (§9) |
| **`canonical_extension_types`** | ❌ | **named** | **applied.** §3 explains why; recognising inbound `arrow.*` columns (DM-42) |
| **`ffi`** | ❌ | **named** | **applied.** Required for §21/D12's C stream boundary |
| **`pyarrow`** (pulls `arrow-pyarrow`) | ❌ | — | an alternative to `pyo3-arrow`; still an unmade choice — §8, §15 item 5 |
| **`force_validate`** | ❌ | — | **new in §10.** Runs full structural validation on every array constructed. Too expensive for release builds, which is why it is off — but it is exactly a **G3** gate for `cargo test` and CI. Not mentioned anywhere in the blueprint. |
| **`pool`** | ❌ | — | **new in §10.** Enables `arrow_buffer::MemoryPool` / `MemoryReservation` — Arrow-level allocation accounting, distinct from DataFusion's `MemoryPool`. Bears on D11's native workspaces and **G5**. |
| `ipc_compression` | ❌ | — | **must stay off** on the hashing path (§7) |
| `prettyprint` | ❌ | — | diagnostics only, never on an identity path |
| `chrono-tz` | ❌ | — | required only if `Timestamp(_, tz)` arithmetic is used; §4.5 fixes `"UTC"`, so not required |
| `async` | ❌ | — | not required by anything in scope |

### 1.4 Extraction receipt

rustdoc JSON generated locally from a scratch project with every crate `=`-pinned, resolved offline from the crates.io mirror, one toolchain:

```text
toolchain       rustc 1.100.0-nightly (809936eac 2026-09-12)
cargo           1.100.0-nightly (7941be6fb 2026-09-11)
flags           -Z unstable-options --output-format json
format_version  61
run             2026-09-13T16:15:24-04:00 .. 16:16:22-04:00
targets         57 requested, 57 OK, 0 FAIL, 60 JSON documents
arrow family    20 crates, every crate_version 59.3.0
                arrow, arrow_arith, arrow_array, arrow_avro, arrow_buffer, arrow_cast,
                arrow_csv, arrow_data, arrow_flight, arrow_ipc, arrow_json, arrow_ord,
                arrow_row, arrow_schema, arrow_select, arrow_string,
                parquet, parquet_variant, parquet_variant_compute, parquet_variant_json
datafusion      37 crates, every crate_version 55.1.0  (companion map)
lockfile        361 packages; one version per family, asserted
```

This is a **substantially wider extraction than the previous edition's**, which covered 13 arrow crates and preserved nothing. Newly covered here: `arrow-avro`, `arrow-csv`, `arrow-flight`, `arrow-json`, and the three `parquet-variant*` crates. The normalized form is written to `build/facts/arrow593/` — **30,920 declarations, 101 public traits, 18,268 impl relations** — alongside, and never overwriting, the older `arrow59-default` profile at 59.2.0.

Every `[rustdoc:…@59.3.0]` marker in this document resolves against that extraction.

Committed under **`docs/capability-maps/evidence/rust/`**: the extraction manifest (`apisurface-Cargo.toml`), **the resolved `apisurface-Cargo.lock`**, the generation script, the generation log, the five probe programs, and their captured output (`probe_output.txt`). The lockfile is the part the previous edition lacked — without it "re-derivable" meant "re-resolvable to whatever is current", which is not the same claim.

### 1.5 Probe receipt

Measurements in this document come from five programs compiled against the pinned crates. Arrow-side probes 1–5 are unchanged from the previous edition; **6 and 7 are new**, and 6 corrects a claim this map previously made (§0.4).

| Probe | Program | Measures |
|---|---|---|
| 1–5 | `arrow_probe.rs` | IPC metadata round-trip; null/NaN/`-0.0` distinctness; `totalOrder` sort; batch-split byte equality; alignment sensitivity |
| **6** | `arrow_probe2.rs` | **whether metadata key count/order changes the canonical IPC bytes** |
| **7** | `arrow_probe2.rs` | **Parquet metadata survival, extension-name survival, and write determinism** — closes the previous edition's open item |

---

## 2. Type system, `Field`, `Schema`, and metadata (A1)

**Role:** §4.3 metadata conventions; §4.5 logical type catalog; §4.2's generated `fn schema() -> SchemaRef` and the `try_from(&RecordBatch)` that checks a schema fingerprint.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Type enumeration | `DataType` with **41 variants**: `Null`, `Boolean`, `Int8…64`, `UInt8…64`, `Float16/32/64`, `Timestamp`, `Date32/64`, `Time32/64`, `Duration`, `Interval`, `Binary`, `FixedSizeBinary`, `LargeBinary`, `BinaryView`, `Utf8`, `LargeUtf8`, `Utf8View`, `List`, `ListView`, `FixedSizeList`, `LargeList`, `LargeListView`, `Struct`, `Union`, `Dictionary`, `Decimal32`, `Decimal64`, `Decimal128`, `Decimal256`, `Map`, `RunEndEncoded` | `[rustdoc:arrow-schema@59.3.0]` |
| Field | `Field::new(name, DataType, nullable)`; typed constructors `new_struct`, `new_list`, `new_large_list`, `new_fixed_size_list`, `new_map`, `new_union`, `new_dictionary`, `new_dict(…, dict_id, dict_is_ordered)`, `new_list_field` | `[rustdoc:arrow-schema@59.3.0]` |
| Field metadata | `metadata() -> &HashMap<String, String>`, `metadata_mut`, `set_metadata`, `with_metadata` | `[rustdoc:arrow-schema@59.3.0]` |
| Schema | `pub struct Schema { pub fields: Fields, pub metadata: HashMap<String, String> }`; `Schema::new`, `new_with_metadata`, `with_metadata`, `metadata()` | `[c7:/websites/rs_arrow_arrow]`, `[rustdoc:arrow-schema@59.3.0]` |
| Schema operations | `project(&[usize])`, `try_merge(iter)`, `normalize(separator, max_level)`, `flattened_fields()`, `fields_with_dict_id(i64)`, **`contains(&Schema) -> bool`** | `[rustdoc:arrow-schema@59.3.0]` |
| Batch-level metadata mutation | `RecordBatch::schema_metadata_mut()` — inserts into schema metadata in place, cloning the schema if shared | `[c7:/websites/rs_arrow_arrow]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §4.3 schema-level keys (`pse.contract.id`, `pse.contract.version`, `pse.contract.fingerprint`, `pse.namespace`, `pse.snapshot_id`, `pse.producer_pass_id`) | `Schema.metadata: HashMap<String, String>` | **confirmed** — arbitrary string keys at schema level are a first-class field of `Schema`, not a convention bolted on |
| §4.3 field-level keys (`pse.semantic.logical_type`, `…quantity_type`, `…role`, `…fk`, `…enum`) | `Field::with_metadata` / `metadata()` | **confirmed** |
| §4.3 "metadata is a carrier, not the metamodel: nothing in DataFusion acts on these keys unless a platform component explicitly reads them" | — | **confirmed by construction**: Arrow assigns no meaning to non-`ARROW:`-prefixed keys. The one exception is the `ARROW:extension:*` pair, which Arrow *does* interpret (A2). |
| §4.5 logical type catalog (`Float64`, `Int64`, `UInt16/32/64`, `Boolean`, `Utf8`, `Timestamp(Nanosecond,"UTC")`, `List`, `Struct`) | all present | **confirmed** — the catalog is a conservative subset of the 41 variants |
| §4.2 generated `try_from(&RecordBatch)` "that checks the schema fingerprint" | the fingerprint check is platform code; but **`Schema::contains(&other)`** exists for structural compatibility | **refined** — see alignment note below |

### Pass 2 — alignment opportunities not in the blueprint

| Capability | DM | What it would replace | Recommendation |
|---|---|---|---|
| `Schema::contains(&Schema) -> bool` | DM-53 (verify equivalence across representations), G3 | a hand-written field-by-field comparison in the generated `try_from` | **adopt as a second check.** The fingerprint check answers "is this the contract I expect"; `contains` answers "is this batch structurally usable" — they fail differently, and a projected batch legitimately fails the first while passing the second. §4.2 currently specifies only the fingerprint. |
| `Schema::project(&[usize])` | DM-22 (every transformation has a contract) | hand-built projected schemas in the provider | **adopt** — projection is applied at read in §5.4; deriving the projected schema from the authoritative one keeps metadata attached rather than reconstructed |
| `RunEndEncoded` `DataType` | DM-36 (layouts for demonstrated access patterns) | nothing yet | **evaluate, do not adopt now.** `compiled.*` relations repeat instance/domain keys heavily, and REE is the Arrow-native encoding for run-structured columns. But DM-58 applies: no access-pattern measurement exists yet. Record as a measurement to take, not a design change. |
| `Utf8View` / `BinaryView` | DM-36 | `Utf8` for `doc`, `name`, `path` columns | **evaluate.** View types trade a wider header for cheap slicing and no re-copy on `take`. Relevant only if profiling shows string handling matters; §4.5 should not change speculatively. |
| `Decimal32` / `Decimal64` | DM-40 (declare precision) | `Float64` for exact-decimal quantities | **reject for physical quantities** — §4.5 is right that `Float64` is the type for numerical values a solver consumes. Worth a note only for **costing** (§19.5), where exact decimal arithmetic is arguably more honest than binary floating point. |
| `Schema::normalize` / `flattened_fields` | DM-10 (keep structure typed and queryable) | — | **reject** — these flatten nested structure into dotted names, which is the opposite of the blueprint's direction (structure stays typed, §4.5). Named here so nobody adopts them for convenience. |

### Gaps and risks

- **`Fields` is an `Arc`-backed shared slice**, so schemas are cheap to clone but *shared*; `RecordBatch::schema_metadata_mut` clones on write `[c7:/websites/rs_arrow_arrow]`. Mutating schema metadata after a batch is built is therefore possible but not free, and it silently diverges the batch's schema from the registry's. §4.3 says metadata is "attached by the provider" — that should be read as *attached at construction*, never patched later.
- **Metadata is `HashMap<String, String>` — unordered — but the IPC writer does not inherit that disorder.** This is a correction to the previous edition, which claimed the canonical IPC encoding was non-reproducible for this reason. Measured `[probe]`:

  ```text
  PROBE 6  canonical IPC bytes vs number of metadata keys (6 independently built schemas each)
     0 keys: len=312  distinct encodings = 1  STABLE
     1 key : len=416  distinct encodings = 1  STABLE
     2 keys: len=488  distinct encodings = 1  STABLE
     3 keys: len=560  distinct encodings = 1  STABLE
     5 keys: len=704  distinct encodings = 1  STABLE
  ```

  So §5.3's `content_hash` is **not** at risk from metadata ordering, and the previous edition's register row #8 overstated the problem. The risk is real but confined to two other paths, and the second one is serious:

  1. **Any platform code that fingerprints by iterating `metadata()` directly** — `pse.contract.fingerprint` as §4.3 describes it — *is* exposed, because that iteration is the `HashMap`'s. Sorting there is still required (DM-48).
  2. **Protobuf metadata remains unordered.** Sorted insertion into field/schema HashMaps does not stabilize `datafusion-proto` bytes [review:E5]. Blueprint §14.2 now records noncanonical diagnostic encodings outside semantic memo keys (ADR-0044); §5.3 separately canonicalizes metadata for logical relation identity.
- `fields_with_dict_id` / `Field::new_dict(dict_id, …)` expose Arrow's legacy dictionary-id mechanism. §4.5 says "dictionary encoding … never carries identity" — consistent, but the generated code must not set `dict_id`, and a governance check is cheap.

---

## 3. Extension types and the `canonical_extension_types` feature (A2)

The densest blueprint↔library coupling in the Arrow surface: §4.4 declares ten `pse.*` extension types and states exactly which Arrow machinery implements them.

This cluster is where **DM-06** ("type semantic distinctions, not only machine representations") is either satisfied or lost. `FixedSizeBinary(16)` is a machine representation; `pse.semantic_id` and `pse.content_hash` are two *different semantic types* that share it. Arrow's extension mechanism is the only place in the format where that distinction can be recorded and carried, so everything below is DM-06 infrastructure.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| The trait | `arrow_schema::extension::ExtensionType` — `const NAME: &str`; `type Metadata`; `fn metadata(&self) -> &Metadata`; `fn serialize_metadata(&self) -> Option<String>`; `fn deserialize_metadata(Option<&str>) -> Result<Metadata, ArrowError>`; `fn supports_data_type(&self, &DataType) -> Result<(), ArrowError>`; `fn try_new(&DataType, Metadata) -> Result<Self, ArrowError>`; provided `fn validate(&DataType, Metadata) -> Result<(), ArrowError>`; provided `fn try_new_from_field_metadata(&DataType, &HashMap<String,String>) -> Result<Self, ArrowError>` | `[rustdoc:arrow-schema@59.3.0]` |
| Naming rule | "Names should use *namespace*-style prefixes (e.g. `myorg.name_of_type`). Names beginning with **`arrow.` are reserved** for canonical types." | `[docs.rs:arrow-schema@59.2.0]` |
| Metadata keys | `EXTENSION_TYPE_NAME_KEY = "ARROW:extension:name"`, `EXTENSION_TYPE_METADATA_KEY = "ARROW:extension:metadata"` | `[rustdoc:arrow-schema@59.3.0]` |
| Field integration | `Field::with_extension_type<E>(self, E) -> Self`, `try_with_extension_type<E>(&mut self, E) -> Result<(), ArrowError>`, `try_extension_type<E>(&self) -> Result<E, ArrowError>`, `extension_type<E>(&self) -> E` (**panics**), `has_valid_extension_type<E>(&self) -> bool`, `extension_type_name() -> Option<&str>`, `extension_type_metadata() -> Option<&str>`, `try_canonical_extension_type() -> Result<CanonicalExtensionType, ArrowError>` | `[rustdoc:arrow-schema@59.3.0]`, `[c7:/websites/rs_arrow_arrow]` |
| Canonical types (7) | `Uuid` = `arrow.uuid`; `Json` = `arrow.json`; `Bool8` = `arrow.bool8`; `Opaque` = `arrow.opaque`; `FixedShapeTensor` = `arrow.fixed_shape_tensor`; `VariableShapeTensor` = `arrow.variable_shape_tensor`; `TimestampWithOffset` = `arrow.timestamp_with_offset`; plus the `CanonicalExtensionType` enum and the `*Metadata` structs | `[rustdoc:arrow-schema@59.3.0]` |
| Feature gate | all seven canonical types and `CanonicalExtensionType` require the **`canonical_extension_types`** cargo feature (off by default) | `[docs.rs:arrow-schema@59.2.0]`, confirmed by building with it enabled `[rustdoc:arrow-schema@59.3.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §4.4 "Rust implementations use `arrow_schema::extension::ExtensionType` with `NAME`, typed `Metadata`, `serialize_metadata`, `deserialize_metadata`, and `supports_data_type`, exactly as documented for Arrow 59" | the trait as quoted above | **confirmed, and incomplete.** The blueprint names five members; the trait also *requires* `try_new` and `metadata`, and provides `validate` and `try_new_from_field_metadata`. The two provided methods are the ones that matter operationally — they are the read path. |
| §4.3 `ARROW:extension:name` as a field-metadata key | `EXTENSION_TYPE_NAME_KEY` | **confirmed verbatim**, and `ARROW:extension:metadata` is its mandatory companion — §4.3's field-key table lists only `ARROW:extension:name`, so the metadata key should be added for completeness |
| §4.4 `pse.*` naming for ten extension types | the `myorg.name_of_type` convention; `arrow.` reserved | **confirmed** — `pse.semantic_id`, `pse.bound` etc. are correctly namespaced and cannot collide with canonical types |
| §4.4 "All extension types have a standard storage type so that an unaware consumer reads them safely" | `supports_data_type` is the enforcement point, called by `try_new`/`validate` | **confirmed** — but only if the platform calls it; see the validation gap below |
| §4.4 Python consumers register matching `pyarrow.ExtensionType` classes | out of scope here — see A7 | deferred |

### Pass 2 — alignment opportunities

| Capability | DM / gate | What it would replace | Recommendation |
|---|---|---|---|
| `Field::try_extension_type::<E>()` + `validate` called inside the generated `try_from(&RecordBatch)` | **DM-07** (enforce validity at identified boundaries), **G3** (invalid state must not reach an operation that assumes validity) | nothing today — §4.2's generated view checks the *schema fingerprint* only | **adopt.** A fingerprint match proves the schema is the contract we published; it does **not** prove a given field's extension metadata deserializes or that its storage type is supported. Those are exactly what `try_extension_type` checks, and the check is generated code, not hand-written. This is the single highest-value item in the Arrow surface. |
| `Field::extension_type::<E>()` — the panicking variant | **G3**, DM-30 (partial failure explicit) | — | **reject and ban.** A panic in a pass body is not a typed failure (§23.2) and cannot become a `validation.invariant` row. Add it to the governance greps beside `no_shadow_structs.rs`. |
| `canonical_extension_types` feature **on** | **DM-42** (loss-aware interchange; reject silent semantic degradation) | — | **adopt.** Not for what we *write* — for what we *read*. §20.5 imports IDAES JSON and §21 exchanges with Python; an inbound column carrying `arrow.json`, `arrow.uuid` or `arrow.bool8` is meaningful, and without the feature `try_canonical_extension_type` does not exist and the platform cannot even recognise it. Silently treating `arrow.bool8` storage (`Int8`) as a plain integer is precisely the degradation DM-42 forbids. **The blueprint does not mention this feature at all.** |
| `Opaque` (`arrow.opaque`) canonical type | **DM-04** (declare the semantic boundary and expose opaque behavior), DM-42 | a bare storage type plus a `pse.semantic.*` note | **evaluate.** §18.5 and §9.8 admit kernels whose values the platform does not interpret; `arrow.opaque` is the standard, cross-language way to say "this came from another system and we do not interpret it", carrying the originating type name in its metadata. Cheaper and more honest than inventing `pse.opaque`. |
| `Uuid` (`arrow.uuid`) for `pse.semantic_id` | DM-42, DM-44 | `pse.semantic_id` | **reject, with reason.** §5.1 puts *two* kinds of 128-bit value in one opaque column: `blake3_128` derived IDs and UUIDv7 interactive IDs. A blake3 digest is not a UUID and labelling it `arrow.uuid` would assert a structure (version/variant nibbles) it does not have — a G2 semantic-fidelity failure in exchange for cosmetic interoperability. The blueprint's opaque `pse.semantic_id` is correct. Worth recording *why*, since `arrow.uuid` looks like an obvious win. |
| A generated conformance test per `pse.*` extension type | **DM-44** (extensions complete, versioned, conformance-testable), DM-53 | ad-hoc tests | **adopt.** `ExtensionType` gives the exact contract to test: round-trip `serialize_metadata` → `deserialize_metadata`, `supports_data_type` accepts the declared storage and rejects a near-miss, and `try_new_from_field_metadata` reconstructs from a `Field`. Ten types × one generated test each, driven from §4.4's table — which is §4.2's existing generation pattern, extended. |

### Gaps and risks

- **Arrow has no extension-type registry — but DataFusion 55.1.0 does, and it changes this cluster's conclusion.** The Arrow trait is per-type and field-local; nothing in `arrow-schema` maps a name back to an implementation. That much is unchanged. What *is* new is that `datafusion_expr::registry::ExtensionTypeRegistry` provides exactly that mapping, keyed on the `ARROW:extension:name` string, and it is wired into `SessionStateBuilder`. Measured `[probe]`:

  ```text
  PROBE X1  ExtensionTypeRegistry
     canonical types preloaded (7): arrow.timestamp_with_offset, arrow.opaque,
        arrow.fixed_shape_tensor, arrow.uuid, arrow.variable_shape_tensor,
        arrow.bool8, arrow.json
     register pse.semantic_id          -> 8 registrations
     resolve from a good Field         -> storage_type=FixedSizeBinary(16)
     Field with the WRONG storage type -> REJECTED: "pse.semantic_id requires
                                          FixedSizeBinary(16), got Utf8"
     unregistered pse.* name           -> Err("Logical type not found")
  ```

  **Scope correction:** X1 invokes `create_extension_type_for_field` directly; it does not validate query admission. E1 shows malformed and unknown extension fields survive an ordinary SELECT with zero factory calls [review:E1]. Generated recursive admission plus a platform AnalyzerRule supplies the missing boundary (blueprint §4.4).

- **Extension-type metadata versioning remains unresolved.** An inbound field carrying `pse.quantity_value` written by a *different version* of the platform deserializes only if our current `Metadata` type still parses it. §4.4 has no versioning story — `pse.contract.version` versions the *relation*, not the extension type. The registry gives a place to put a resolution rule; it does not supply one. **Open item (DM-44, DM-51).**
- **`deserialize_metadata` must error on unexpected metadata**, not ignore it `[docs.rs:arrow-schema@59.2.0]`. For the blueprint's metadata-free types (`pse.content_hash`, `pse.ordinal_ref`) the implementation must therefore reject a field that carries an `ARROW:extension:metadata` key rather than shrug — otherwise two different producers disagree silently.
- **`pse.ordinal_ref` is the weak link in §4.4.** Its storage is `UInt64` and "metadata names the target relation" — but that target is carried in `pse.semantic.fk`-style *platform* metadata, not in the extension's own `Metadata`. Making the target relation part of the extension type's `Metadata` would let `supports_data_type`/`deserialize_metadata` enforce it. **Evaluate (DM-09, DM-44).**
- `has_valid_extension_type` returns `bool` and discards the reason `[rustdoc:arrow-schema@59.3.0]`; use `try_extension_type` wherever the failure must become a diagnostic (§23.2).

---

## 4. Arrays, buffers, and the zero-copy boundary (A3)

**Role:** §4.2 generated typed views (`StoichiometryView<'a> { reaction_id: &'a FixedSizeBinaryArray, … }`) and builders; **D11** — "the evaluation program, sparse Jacobian structures, and solver workspaces … borrow Arrow buffers when layouts permit and copy when they do not. Zero-copy is a preference, not an obligation."

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Buffer construction from Rust memory | `Buffer::from_vec<T>(Vec<T>)`, `Buffer::from_slice_ref` | `[rustdoc:arrow-buffer@59.3.0]` |
| **Buffer over foreign memory** | `Buffer::from_custom_allocation(ptr: NonNull<u8>, len: usize, owner: Arc<dyn Allocation>)` | `[rustdoc:arrow-buffer@59.3.0]` |
| Buffer inspection | `as_slice() -> &[u8]`, `data_ptr() -> NonNull<u8>`, **`ptr_offset() -> usize`**, `len()`, `slice(offset)` | `[rustdoc:arrow-buffer@59.3.0]` |
| Typed arrays | `arrow_array` — 4597 indexed items including the concrete array types the generated views name | `[rustdoc:arrow-array@59.3.0]` |
| Batch | `RecordBatch::try_new(SchemaRef, Vec<ArrayRef>)`, `schema()`, `column(i)`, `slice(offset, len)`, `schema_metadata_mut()` | `[c7:/websites/rs_arrow_arrow]`, verified by probe |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §4.2 typed column views holding `&'a FixedSizeBinaryArray` etc. | downcast from `ArrayRef` | **confirmed** — the generated view is a borrow of the batch's columns, which is what makes it free |
| D11 "borrow Arrow buffers when layouts permit" — native code reading Arrow memory | `Buffer::as_slice()` / `data_ptr()` + `ptr_offset()` | **confirmed**, with a hazard (below) |
| D11 "copy when they do not" | `Buffer::from_vec` for the reverse direction | **confirmed** |
| §18.2 "Arrow buffers of parameter values are borrowed when contiguous and copied otherwise; the workspace is native" | as above | **confirmed** — the contiguity test is real: a `ChunkedArray`-style multi-batch column has no single contiguous buffer, and a sliced array has an offset |

### Pass 2 — alignment opportunities

| Capability | DM | What it would replace | Recommendation |
|---|---|---|---|
| `Buffer::from_custom_allocation(ptr, len, owner)` | **DM-37** (cross expensive boundaries in coarse typed units), DM-29 (isolate mutable workspaces, commit outcomes) | copying the evaluation program's native `f64` workspace into a fresh Arrow array to publish results | **evaluate.** This is the *reverse* of D11's stated direction: it lets a native workspace be published **as** an Arrow buffer with an explicit ownership token, no copy. Attractive for §18.6's solution/residual relations where the solver already owns a dense `f64` vector. The cost is a lifetime obligation — the `Arc<dyn Allocation>` must outlive every batch derived from it — which is exactly the "scope mutable runtime workspaces and publish coherent outcomes" discipline DM-29 asks for. Worth a spike; not free. |
| `RecordBatch::slice(offset, len)` | DM-36 | manual re-batching | **adopt** where batches are windowed; it is O(1) and metadata-preserving. |

### Gaps and risks

- **Borrow through safe views.** `PrimitiveArray::values` incorporates the logical slice offset. A null-free contiguous one-chunk slice can be borrowed with its owning handle retained; raw base-pointer indexing is invalid. A filter need not produce a nonzero offset. Layout/coercion/multi-chunk cases use checked reserved copies (blueprint §18.2; review L7).
- Nullability: a borrowed `Float64Array` may carry a validity bitmap. §18.2 treats parameter values as dense `f64`; borrowing a nullable column without checking `null_count() == 0` reads uninitialised slots as data. **State the precondition.**

---

## 5. Null, NaN, and `-0.0` — semantic distinctness (A4)

The cluster with the highest gate stakes: **G2 — semantic fidelity** ("required meaning is ambiguous, silently dropped, reinterpreted, or represented by an indistinguishable sentinel") and **DM-08** ("represent absence, unknowns, uncertainty, invalidity, and failure distinctly").

**Role:** §7.6 null/bound/unknown semantics; §4.4's `pse.bound` ("explicit bound; never NaN or null sentinels"); §5.3 step 4 ("floats hash as IEEE-754 bits after canonicalizing NaN to the quiet NaN with zero payload; `-0.0` is preserved (it is distinguishable and can matter to solvers)").

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Validity is structural | every Arrow array carries an optional validity bitmap separate from its values; `is_null(i)`, `null_count()` | `[rustdoc:arrow-array@59.3.0]`, verified by probe |
| Comparison null semantics | `arrow::compute::kernels::cmp::{lt, lt_eq, gt, gt_eq, eq, neq}` — "comparing null values on either side will yield a **null** in the corresponding slot" | `[c7:/websites/rs_arrow_arrow]` |
| Null-aware equality | `arrow_ord::distinct` (and `not_distinct`) — the `IS DISTINCT FROM` semantics where null equals null | `[rustdoc:arrow-ord@59.3.0]` |
| **Float ordering** | sort and `cmp` kernels order `f32`/`f64` by the **IEEE 754 (2008) `totalOrder` predicate** | `[c7:/websites/rs_arrow_arrow]` |
| Null placement | `SortOptions { descending, nulls_first }`; default is `ASC NULLS FIRST`; `make_comparator(left, right, SortOptions)` | `[c7:/websites/rs_arrow_arrow]` |

### Empirical verification

Run locally against the pinned build (`arrow 59.3.0`, program and output in the evidence ledger, row 16):

```text
PROBE 2  null vs NaN vs -0.0 after an IPC round-trip
   is_null(1)=true   value(2)=-0   signbit(2)=true

PROBE 3  sort order of [0.0, -0.0, NaN, -1.0, null] with nulls_first
   sorted: ["null", "-1", "-0", "0", "NaN"]
```

Three facts follow, and all three matter to the blueprint:

1. **Null is structurally distinct from NaN and survives serialization.** DM-08 and G2 are satisfied *by the format*, not by convention — there is no sentinel to confuse.
2. **`-0.0` sorts strictly before `+0.0`.** totalOrder treats them as distinct and ordered, so §5.3's "`-0.0` is preserved" is not merely a hashing choice — Arrow's own ordering already distinguishes them, and a canonicalisation that collapsed them would *contradict* the sort.
3. **NaN sorts last, after all finite values.** Deterministic placement, so §5.3 step 1's key ordering is well defined even if a NaN reaches a sort column.

### What the blueprint binds to

| Blueprint requirement | Status |
|---|---|
| §4.4 `pse.bound` = `Struct<kind: Dictionary(Int8,Utf8), value: Float64>` with `kind ∈ {finite, unbounded}` — "never NaN or null sentinels" | **confirmed as the right call, and now with a second reason.** Arrow *would* let you encode unboundedness as null; the blueprint refuses. Beyond G2, the mechanical reason is that comparison kernels propagate null (`lt(null, x) = null`), so a null-encoded bound silently produces null rather than a decision — an indistinguishable-sentinel failure at exactly the place a solver needs an answer. |
| §5.3 step 4 "canonicalizing NaN to the quiet NaN with zero payload" | **confirmed necessary.** Because ordering and hashing are bit-level under totalOrder, two NaNs with different payloads or sign bits are *distinct*. Without canonicalisation, two logically identical relations could hash differently — a DM-48 reproducibility break. The blueprint is right and the mechanism is now explicit. |
| §5.3 step 4 "`-0.0` is preserved" | **confirmed** — and note the tension resolved: NaN is canonicalised, `-0.0` is not. That asymmetry is correct: NaN payload carries no meaning, the zero's sign does. |
| §7.6 null / bound / unknown as distinct | **confirmed** — Arrow supplies null; "unknown" and "conflict" (§14.2 rule 3's four-valued predicates) are platform-level `status` columns, not Arrow concepts, which is the right layering |

### Pass 2 — alignment opportunities

| Capability | DM / gate | Recommendation |
|---|---|---|
| `arrow_ord::distinct` / `not_distinct` | **DM-08**, G2 | **adopt** wherever the rule compiler (§14.2) compares nullable key columns. Ordinary `eq` yields null for null-vs-null, which in a join or dedup context silently drops candidate rows — exactly what §14.2 rule 3 forbids ("they never silently drop a candidate"). `distinct` gives the null-equals-null semantics that deduplication actually wants. **Not mentioned in the blueprint.** |
| Explicit `SortOptions` at every sort site | DM-40 (declare ordering and determinism requirements) | **adopt as a rule.** The default is `ASC NULLS FIRST`; relying on a default makes the canonical ordering of §5.3 depend on an Arrow default rather than on a written contract. Name it. |
| `CastOptions { safe }` (A5) for any numeric narrowing | DM-42 (reject silent degradation) | see A5 |

### Gaps and risks

- **The NaN canonicalisation is ours to implement and is easy to get wrong.** Arrow will not do it; `f64::to_bits()` on a signalling NaN, a negative NaN, or a NaN with a payload gives different bytes. §5.3 step 4 must be a named, tested function applied to every `Float64` column before hashing — and it must *not* be applied to data on the way to a solver, where a NaN's provenance may matter for diagnostics (§18.2 step 6).
- `make_comparator` returns `Box<dyn Fn(usize, usize) -> Ordering>` and does **not** support nested types; `lt`/`gt` likewise state "nested types (e.g. lists) are not supported" `[c7:/websites/rs_arrow_arrow]`. §4.4's `pse.index_tuple` (`List<FixedSizeBinary(16)>`) therefore cannot be compared or sorted by the ordinary kernels — see the row format in A5, which *can*. **This is a real constraint on §5.3 step 1 that the blueprint does not anticipate.**

---

## 6. Compute kernels, ordering, and the row format (A5)

**Role:** §5.3 step 1 (order rows by primary key) and step 2 (concatenate, expand dictionaries); §15.4 numerical checks over Jacobian values; §16 scaling transformations.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Selection | `arrow_select::{concat, concat_batches, filter, filter_record_batch, filter_nulls, take, interleave, zip, nullif}`; `FilterBuilder`, `FilterPredicate`, `TakeOptions`, `SlicesIterator`, `BatchCoalescer`, `ScalarZipper` | `[rustdoc:arrow-select@59.3.0]` |
| Dictionary hygiene | `arrow_select::{garbage_collect_dictionary, garbage_collect_any_dictionary}` | `[rustdoc:arrow-select@59.3.0]` |
| Ordering | `arrow_ord::{sort, sort_to_indices, lexsort, lexsort_to_indices, partition, rank}`; `SortColumn`, `LexicographicalComparator`, `FixedLexicographicalComparator`, `Partitions`, `make_comparator` | `[rustdoc:arrow-ord@59.3.0]`, `[c7:/websites/rs_arrow_arrow]` |
| Comparison | `arrow_ord::cmp::{eq, neq, lt, lt_eq, gt, gt_eq, distinct, not_distinct}` | `[rustdoc:arrow-ord@59.3.0]` |
| **Row format** | `arrow_row::{RowConverter, Rows, Row, OwnedRow, RowParser, SortField}`; `convert_columns`, `convert_rows`, `encode_dictionary_values` | `[rustdoc:arrow-row@59.3.0]` |
| Casting | `arrow_cast::{cast, cast_with_options, can_cast_types}`; `CastOptions`, `FormatOptions`, `ArrayFormatter`, `array_value_to_string` | `[rustdoc:arrow-cast@59.3.0]` |
| Arithmetic | `arrow_arith::{add, sub, mul, div, add_wrapping, …}`, boolean `and`/`or` plus **Kleene** variants `and_kleene`/`or_kleene`, bitwise family, and the generic `binary`/`binary_mut`/`unary` builders | `[rustdoc:arrow-arith@59.3.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §5.3 step 1 "Order rows by primary key (byte order of semantic IDs, then ordinals)" | `lexsort_to_indices` + `SortColumn`, or the row format | **leverage** — multi-column ordering is built in |
| §5.3 step 2 "Concatenate all batches into one" | `concat_batches` | **confirmed load-bearing** — see A6's PROBE 4: without it, identical data in different batch splits produces different IPC bytes |
| §5.3 step 2 "expand dictionaries" | `cast` from `Dictionary(K,V)` to `V` | **leverage** |
| §15.4 row/column norms, extreme-value scans over the Jacobian | `arrow_arith` + reductions | **partially** — these run over the *native* evaluation program's Jacobian (D11), not over Arrow arrays, so the kernels matter only where a diagnostic is expressed as a relation query |

### Pass 2 — alignment opportunities

| Capability | DM / gate | What it would replace | Recommendation |
|---|---|---|---|
| **`arrow_row::RowConverter`** | **DM-15** (define canonicalization and equivalence before using content identity), DM-38 | a hand-written multi-column comparator for §5.3's primary-key ordering | **adopt — the strongest single find in the Arrow surface.** `RowConverter` encodes a set of columns into a **byte-comparable** row format: sorting becomes `memcmp`, and two rows are equal iff their bytes are equal. That is precisely the canonical-ordering-and-equivalence primitive §5.3 needs, and it **handles nested types**, which `make_comparator`, `lt` and `gt` explicitly do not (A4). §4.4's `pse.index_tuple` (`List<FixedSizeBinary(16)>`) is a primary-key component in several `compiled.*` relations, so this is not hypothetical — it is the difference between §5.3 step 1 working on those relations and not. |
| `arrow_ord::partition` / `Partitions` | DM-38 | hand-written run detection over a sorted key column | **adopt** for grouped passes (per-instance, per-domain processing), which the rule compiler and §18.2's tape construction both need. |
| `arrow_cast::CastOptions { safe }` and `can_cast_types` | DM-42, G2 | unchecked or precision-losing conversion | **Adopt as implementation tools after admission checks.** `safe: false` can still truncate fractions and round integers [review:E3]; exact representability and complete quantity compatibility are platform checks (§14.2). |
| `and_kleene` / `or_kleene` | **DM-08** | ordinary `and`/`or` | **adopt** in the rule compiler. Kleene logic is three-valued (`true AND null = null`, `false AND null = false`), which is the semantics §14.2 rule 3's four-valued predicates need underneath. Ordinary `and` propagates null unconditionally and loses the `false` short-circuit — changing which rows a rule emits. |
| `garbage_collect_dictionary` | DM-15, DM-36 | — | **evaluate.** After filtering, a dictionary column retains unreferenced values; two logically identical relations can then carry different dictionary payloads. §5.3 step 2 expands dictionaries before hashing so the content hash is safe — but any *other* comparison of two batches is not. Worth knowing the kernel exists. |
| `BatchCoalescer` | DM-37 (coarse typed units) | manual re-batching before crossing the FFI boundary | **evaluate** for §21's Python handoff, where batch size affects per-call overhead. |

### Gaps and risks

- The arithmetic kernels are **checked by default** (`add` errors on overflow; `add_wrapping` is the opt-in wrapping form) `[rustdoc:arrow-arith@59.3.0]`. Good default; worth stating so nobody reaches for the wrapping variants to silence an error.
- `ArrayFormatter` / `array_value_to_string` exist for display only. They must never appear in a hashing or identity path — string rendering is locale- and option-dependent (`FormatOptions`).

---

## 7. IPC, Parquet, and canonical serialization determinism (A6)

The cluster that decides whether §5.3 — and therefore every content hash, snapshot id, and reproduction claim in the platform — actually holds.

**Role:** §5.3 canonical serialization and hashing; §20.1 artifact store (`.arrow` IPC hot artifacts, `.parquet` durable artifacts); §4.3 "metadata … preserved through IPC/Parquet".

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Write options | `IpcWriteOptions::try_new(alignment: usize, write_legacy_ipc_format: bool, metadata_version: MetadataVersion) -> Result<Self, ArrowError>`; `try_with_compression(Option<CompressionType>)`; `try_with_compression_level(Option<i32>)` | `[rustdoc:arrow-ipc@59.3.0]` |
| Writers | `StreamWriter::{try_new, try_new_buffered, try_new_with_options, write, finish, into_inner}`; `FileWriter` likewise | `[rustdoc:arrow-ipc@59.3.0]` |
| Readers | `StreamReader`, `FileReader` | `[rustdoc:arrow-ipc@59.3.0]` |
| Compression | opt-in via the `ipc_compression` feature and `try_with_compression`; **`None` is expressible explicitly** | `[rustdoc:arrow-ipc@59.3.0]`, `[docs.rs:arrow@59.3.0]` |
| Parquet | `parquet` 59.3.0, 6507 indexed items | `[rustdoc:parquet@59.3.0]` |

### Empirical verification — the four facts that matter

Run locally against the pinned build (evidence ledger row 16):

```text
PROBE 1  metadata through an IPC round-trip
   schema md preserved : Some("rel-1")            <- pse.contract.id
   field md preserved  : Some("qt-demo")          <- pse.semantic.quantity_type
   ext name preserved  : Some("pse.semantic_id")  <- ARROW:extension:name

PROBE 4  batch splitting
   1-batch len=872   2-batch len=1080   concat-then-write len=872
   1-batch == 2-batch      : false
   1-batch == concat-first : true

PROBE 5  alignment
   align8 len=872   align64 len=1032   equal=false

PROBE 7  Parquet — survival and determinism            [new in this edition]
   two writes of identical data: len=1093  identical=true
   schema md survives : Some("rel-1")
   field md survives  : Some("qt-demo")
   ext name survives  : Some("pse.semantic_id")
   null preserved=true   -0.0 sign preserved=true
   'created_by' writer string embedded in the file: true
```

| Blueprint claim | Verdict |
|---|---|
| §4.3 "Schema-level metadata keys … preserved through IPC/Parquet"; field-level likewise | **confirmed empirically for both**, including the `ARROW:extension:name` key — see PROBE 7 below. The previous edition could only confirm IPC. |
| §5.3 "Two logically identical relations split into different batches hash identically" | **confirmed — but only because of step 2.** Writing the same four rows as one batch versus two produces *different* byte streams (872 vs 1080). Concatenating first reproduces the single-batch bytes exactly. Step 2 ("concatenate all batches into one") is therefore not housekeeping; it is the step that makes the claim true. Anyone who "optimises" it away breaks every content hash. |
| §5.3 step 3 "Serialize to Arrow IPC stream with **no compression** and **deterministic alignment**" | **confirmed expressible**: `IpcWriteOptions::try_new(alignment, false, MetadataVersion::V5).try_with_compression(None)`. |

### Pass 2 — alignment opportunities and a specification gap

| Item | DM / gate | Recommendation |
|---|---|---|
| **Pin the alignment constant in the hashing contract** | **DM-48** (define and support the required reproducibility contract), DM-40 | **adopt — this is a specification gap in §5.3.** Alignment changes the bytes (872 vs 1032 for the same rows). §5.3 says "deterministic alignment" without naming a value. If the platform ever calls `IpcWriteOptions::default()`, the content hash silently depends on an Arrow default that can change between releases — and every `snapshot_id` in history becomes unreproducible. The canonical-serialization function must pass an explicit alignment and `MetadataVersion`, and those constants must be versioned alongside `pse.contract.fingerprint`. |
| Record the IPC `MetadataVersion` in the hash contract | DM-48, DM-51 | **adopt** for the same reason: `try_new` takes it explicitly, so it is a contract term, not an ambient default. |
| `write_legacy_ipc_format: false` | DM-42 | **adopt explicitly** rather than by default. |
| Parquet metadata + determinism | DM-48, DM-42 | **now measured — and the previous edition's reasoning was half right.** See PROBE 7. Parquet **does** preserve schema metadata, field metadata and `ARROW:extension:name`, and two writes of identical data are **byte-identical**. So the feared metadata loss does not occur, and Parquet is deterministic *within a fixed writer version*. What remains true is the version dependence: the file embeds a `created_by` writer string, so the bytes change when `parquet` is upgraded even though the data has not. The recommendation therefore stands but for a narrower reason — **state in §5.3/§20.1 that content identity is defined over the canonical IPC encoding only**, not because Parquet loses meaning, but because its bytes carry the writer's version. |

### Gaps and risks

- **Identity and integrity are separate.** Blueprint §5.3 defines a versioned logical hash; §20 names each stored encoding by its own byte checksum. IPC-file/Parquet encodings cannot be verified against the canonical stream digest [review:E4].
- Finish and validate every writer before publication. Verify physical checksum/length/format, then the decoded semantic contract and logical hash where required. An existing checksum-named object is not automatically trusted (blueprint §20.1; [review:E4]).

---

## 8. The C Data / C Stream FFI boundary and Python interop (A7)

**Role:** §21.1 extension module (`pyo3 + pyo3-arrow`, cdylib); D12 "one adapter consumes a `CanonicalMathProblem` bundle **over the Arrow C stream interface**"; §21.5 typed Python contracts.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| C Data interface | `arrow_schema::FFI_ArrowSchema`; `arrow_array::ffi::{to_ffi, from_ffi, export_array_into_raw}` | `[rustdoc:arrow-schema@59.3.0]`, `[rustdoc:arrow-array@59.3.0]` |
| **C Stream interface** | `arrow_array::FFI_ArrowArrayStream` and `ArrowArrayStreamReader` — the Rust side of the protocol Python exposes as `__arrow_c_stream__` | `[rustdoc:arrow-array@59.3.0]` |
| Feature gate | the umbrella re-exports these under the **`ffi`** cargo feature | `[docs.rs:arrow@59.3.0]` |
| First-party Python bridge | **`arrow-pyarrow`** — now its own crate, enabled through the umbrella's **`pyarrow`** feature | `[docs.rs:arrow@59.3.0]` |
| Third-party Python bridge | `pyo3-arrow` **0.19.0** (2026-06-15) — "zero-copy FFI conversions between Python objects and Rust representations … leveraging the **Arrow PyCapsule Interface**"; requires `pyo3 ^0.29`, `arrow-* ^59`, `numpy ^0.29`; default feature `buffer_protocol` (needs `abi3-py311` or non-abi3 wheels) | `[docs.rs:pyo3-arrow@0.19.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §3.1 pins `pyo3 0.29` / `pyo3-arrow 0.19.0` as "the Arrow 59 compatibility line" | `pyo3-arrow 0.19.0` requires `pyo3 ^0.29` and `arrow-* ^59` | **confirmed** — and the `^59` (not `^59.2`) requirement means the move to **59.3.0 is compatible**, no change needed |
| D12 / §21.1 "over the Arrow C stream interface", `__arrow_c_stream__` | `FFI_ArrowArrayStream` + `ArrowArrayStreamReader` on the Rust side; PyCapsule on the Python side | **confirmed** |
| §21.1 the extension module is a `cdylib` | pyo3 mechanics, out of scope here | — |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **`arrow-pyarrow` vs `pyo3-arrow` — an unmade choice** | **DM-57** (small coherent core with explicit extension mechanisms), DM-41 (adapters mechanical) | **evaluate and decide explicitly.** Two routes now exist and the blueprint names only one. `arrow-pyarrow` is first-party and moves in lockstep with the Arrow family — one fewer independently-versioned compatibility edge, which is exactly the class of risk that produced the `num-dual`/`feos-core` split in the companion map. `pyo3-arrow` adds the PyCapsule protocol and `numpy` integration, which §21.2's adapter may actually want. The decision should be recorded with its reason rather than inherited. |
| PyCapsule (`__arrow_c_stream__`) over pyarrow-object conversion | **DM-37** (cross expensive boundaries in coarse typed units), DM-42 | **adopt** — already the blueprint's direction. The capsule protocol is implementation-neutral: any Python Arrow implementation can consume it, whereas converting to concrete `pyarrow` objects binds the contract to one library version. Worth stating as the reason. |
| Stream, not batch-at-a-time, across the boundary | DM-37 | **adopt** — `FFI_ArrowArrayStream` amortises one FFI crossing over the whole relation. §21's bundle should cross as a stream per relation, not a call per batch. |
| Declare the loss profile of the Python adapter | **DM-42** (loss-aware interchange), **G7** (truthful capability claims) | **adopt.** §4.4's `pse.*` extension types reach Python as *storage types* unless matching `pyarrow.ExtensionType` classes are registered (§4.4 says they are). Where a class is not registered, meaning is lost silently — a Python consumer sees `FixedSizeBinary(16)` and cannot tell a semantic id from a content hash. The adapter should declare, per column, whether the extension survived or degraded. |

### Gaps and risks

- **`pyo3-arrow` depends on `thiserror ^1`** `[docs.rs:pyo3-arrow@0.19.0]` while the wider ecosystem has moved to `thiserror 2.x`. Two major versions of `thiserror` in one graph is legal (no shared types cross the boundary) but worth knowing before §23.2's failure taxonomy is built on `thiserror`.
- The `buffer_protocol` default feature "requires either the `abi3-py311` pyo3 feature or building non-abi3 wheels" `[docs.rs:pyo3-arrow@0.19.0]`. §3.1 targets Python 3.11+, so `abi3-py311` is available — but this is a wheel-building constraint that must be settled before packaging, not after.
- FFI ownership: `export_array_into_raw` and `to_ffi` transfer ownership across the boundary under the C Data contract. A panic unwinding across that boundary is undefined behaviour — the same rule the companion map records for the Ipopt callbacks. **State it once, apply it at every FFI edge.**

---

## 9. Flight, Flight SQL, and the file-format readers (A8) — survey

Surveyed for completeness and rejected, with reasons, so the rejection is on the record rather than an omission.

| Capability | Status for this design | Reason |
|---|---|---|
| `arrow-flight` / Flight SQL | **not used** | §20.1's artifact store is `object_store` over immutable content-addressed files, and §21's Python boundary is in-process FFI. Flight solves *network* transport between processes, which this architecture does not have. Adopting it would add a service boundary the blueprint explicitly does not want (§3.3's "explicitly not added"). Revisit only if a remote execution tier appears. |
| `arrow-csv`, `arrow-json` (umbrella features `csv`, `json`) | **adopt for declared tabular observations** | `ReaderBuilder::new(schema)` accepts an explicit schema; inference is optional. Use JSON strict mode and CSV header validation plus generated unit/target/invariant admission (blueprint §6.10; review L9). |
| `arrow-avro` | **not used** | No Avro source in scope. Listed in §3.1's pin table; it can be dropped from the dependency set entirely, which is one fewer crate in the version-pinning surface. |
| `prettyprint` feature | **development and diagnostics only** | Useful for `EXPLAIN`-style output and test failure messages; must never appear in an identity or hashing path (A5). |
---

## 10. Pass 3 — the under-leverage sweep

Passes 1 and 2 both start from the blueprint: Pass 1 asks whether what it claims is true, Pass 2 asks what would improve alignment *in the areas it already touches*. Neither can find a capability the blueprint never gestured at, because neither enumerates the library independently.

This pass does. Four complete enumerations, each with a stated denominator, each row adjudicated **bound** (already used), **adopt**, **evaluate** or **reject, with reason**. Charter §G and **DM-58** still apply: a row naming no principle and replacing no identified work does not appear. Where a whole group is uninteresting it is dismissed as a group rather than padded out.

The denominator matters because of the rule this corpus inherits: **absence from the extraction is not evidence that a feature does not exist.**

### 10.1 E1 — the crate inventory

`arrow-rs` 59.3.0 declares **27 workspace members** `[gh:apache/arrow-rs@59.3.0/Cargo.toml]`. Four are build or test scaffolding (`arrow-flight/gen`, `arrow-integration-test`, `arrow-integration-testing`, `parquet_derive_test`), leaving **23 publishable crates**. This map extracts **20**; the three gaps are named below rather than silently omitted.

| Crate | Status | Adjudication |
|---|---|---|
| `arrow` (umbrella) | **bound** | §4.2's generated code, everything downstream |
| `arrow-schema` | **bound** | §4.3, §4.4 — the densest coupling in the family (A1, A2) |
| `arrow-array`, `arrow-data`, `arrow-buffer` | **bound** | §4.2 typed views, D11 buffer borrowing (A3) |
| `arrow-ipc` | **bound** | §5.3 canonical serialization — the identity path (A6) |
| `arrow-ord`, `arrow-select`, `arrow-cast`, `arrow-arith` | **bound** | §5.3 ordering, §14.2 rule evaluation, §20.5 import (A4, A5) |
| `arrow-row` | **adopt** | register #4 — required, not optional, wherever a key contains `pse.index_tuple` |
| `arrow-string` | **bound (incidentally)** | pulled in by the umbrella; no direct requirement. `Utf8` columns in the platform are identifiers and documentation, not text to be searched |
| `parquet` | **bound** | §20.1 durable artifacts (A6) |
| `arrow-csv`, `arrow-json` (umbrella features `csv`, `json`) | **adopt for declared tabular observations** | `ReaderBuilder::new(schema)` accepts an explicit schema; inference is optional. Use JSON strict mode and CSV header validation plus generated unit/target/invariant admission (blueprint §6.10; review L9). |
| `arrow-avro` | **reject** | no Avro source in scope; §3.1 rev 2 already says "not used" |
| `arrow-flight` | **reject** | no network tier (A8). **Note the residue:** §3.1 rev 2 still lists `arrow-flight` in the pin row, so the dependency set carries a crate with no consumer — §15 |
| `parquet-variant`, `parquet-variant-compute`, `parquet-variant-json` | **reject, with reason** | The Variant type is a self-describing semi-structured value — a JSON-like tree in a binary encoding. It is genuinely useful for schema-on-read data, and it is genuinely wrong here: **D1 makes typed relations the only authority**, and a Variant column is a typed hole in that authority. Adopting it would let a future contributor park unmodelled structure in a relation instead of declaring it. Recorded as a rejection precisely because it looks convenient (DM-01, DM-10, G1) |
| `parquet-geospatial` | **reject** | no geospatial data in scope |
| `parquet_derive` | **reject** | derives Parquet schemas from Rust structs — the inverse of §4.2's direction, where the registry is authoritative and Rust types are generated from it (DM-02, G1) |
| `arrow-pyarrow` | **not extracted — open** | needs a Python interpreter with `pyarrow` at build time, so it is outside this extraction's offline constraint. This is §15 item 5's crate and the decision is still unmade (A7) |

### 10.2 E2 — the extension-point inventory

**101 public traits** across the 20 extracted crates `[rustdoc:arrow593@59.3.0]`. Most are type-level machinery — `ArrowPrimitiveType`, `ByteArrayType`, `OffsetSizeTrait`, the `Sealed` markers — which exist to make the generic kernels work and are not extension points in any useful sense. Roughly a dozen are places where a platform can insert its own behaviour. Those are the interesting rows.

| Trait | DM / gate | Adjudication |
|---|---|---|
| `arrow_schema::extension::ExtensionType` | **DM-06**, DM-07, G3 | **bound** — §4.4's ten types (A2) |
| **`arrow_cast::display::ArrayFormatterFactory`** | **DM-49** (make changes understandable at the level of meaning), DM-47 | **adopt.** A factory that produces a custom `ArrayFormatter` for a given array *and field*. Today a `pse.semantic_id` renders in any diagnostic as 16 raw bytes and a `pse.bound` as an anonymous struct. §23.2 wants failures that carry "the relation rows and source spans involved"; rows that render as byte soup satisfy the letter and not the purpose. One factory, driven from §4.4's table, makes every diagnostic surface render semantic types meaningfully — and DataFusion has the matching hook (`DFExtensionType::create_array_formatter`), so one implementation serves both. |
| **`arrow_buffer::MemoryPool` / `MemoryReservation`** | DM-30, **G5** | **evaluate.** Arrow-level allocation accounting (`reserve`, `available`, `used`, `capacity`), gated behind the `pool` feature. Distinct from DataFusion's `MemoryPool`, which governs *query* memory: this governs array construction, including arrays the platform builds outside any query — §18.6's result ingestion, §4.2's builders. Those are exactly the allocations DataFusion's pool cannot see. Worth evaluating once there is a memory budget to enforce; **DM-58** says not before. |
| **`arrow_json::EncoderFactory`** | **DM-42**, G7 | **evaluate.** Overrides how specific types are encoded to JSON. §19 and §23 export diagnostics and results; an extension column exported through the default encoder loses its semantic type silently. This is the export-side twin of the import-side `CastOptions { safe: false }` rule (register #6). Only worth it if JSON export becomes a supported interface rather than a debugging convenience. |
| `parquet::arrow::arrow_reader::ArrowPredicate` + `RowGroups` | DM-36, DM-26 | **evaluate.** Row-level filtering pushed into the Parquet reader, with page-index support. §5.4's provider answers key-filters from cached metadata; if a durable Parquet artifact ever becomes the scan source for a large relation, this is the mechanism that makes a filter cheap. Not yet — §5.4 reads IPC. |
| `parquet::file::metadata::memory::HeapSize` | DM-39 | **reject** — accounting for Parquet metadata memory; the platform's artifacts are small enough that this is noise. |
| `arrow_array::RecordBatchReader` / `RecordBatchWriter` | DM-37 | **bound (implicitly)** — the stream shape at §20.1 and §21's boundary |
| `arrow_buffer::alloc::Allocation` | DM-37, DM-29 | **evaluate** — the ownership token behind `Buffer::from_custom_allocation` (register #11) |
| `arrow_flight::sql::server::FlightSqlService` | — | **reject** — 46 provided methods for a service tier this architecture does not have (A8) |
| `parquet::geospatial::*`, `parquet_variant*::*` | — | **reject** — see E1 |
| ~85 type-level and sealed traits | — | **not extension points** — dismissed as a group |

### 10.3 E3 — the declared-policy surface

Arrow's policy surface is small and almost entirely compile-time, which is itself worth stating: unlike DataFusion (155 runtime config keys), **Arrow's behaviour is fixed at build time by feature flags**, so it enters the provenance record as the dependency manifest rather than as a settings hash.

The full feature enumeration is §1.3. Two rows are new findings:

- **`force_validate`** — **adopt for test and CI profiles.** It runs full structural validation on every array constructed, which is too expensive for release builds and is exactly right for a gate. G3 asks whether an invalid state can reach an operation that assumes validity; this turns "we believe our builders are correct" into a checked property across the entire test suite, at the cost of one line in a cargo profile. The blueprint's §24.1 test layers do not mention it.
- **`pool`** — **evaluate**, per E2.

Beyond features, one runtime policy surface matters and the blueprint already half-specifies it: **`IpcWriteOptions`**. Its three constructor arguments (alignment, legacy format, `MetadataVersion`) plus compression are the complete set of knobs that change the bytes §5.3 hashes. They are policy in the DM-16 sense — declared, versioned, and part of the reproducibility contract — and §5.3 currently names none of them. That is §15 item 1, unchanged from the previous edition and still open.

### 10.4 E4 — the release delta, 59.0 → 59.3

Features landed in the 59 line `[gh:apache/arrow-rs/dev/changelog]`. Most are performance work invisible to a consumer. These are the ones that touch a principle:

| Change | Release | Adjudication |
|---|---|---|
| **Parquet ↔ extension-type conversion fixed and tested** (via the GeoArrow work) | 59.1.0 | **explains PROBE 7.** Extension metadata surviving a Parquet round-trip is a recent property, not an ancient one. Since §20.1 stores durable Parquet artifacts, this is load-bearing, and it is a reason to treat the `=59.3.0` pin as a floor rather than an arbitrary choice. |
| `arrow-ipc` **sans-IO stream encoder** | 59.2.0 | **evaluate.** Separates encoding from writing, so the platform controls buffering and can hash the encoded bytes without a second pass. Relevant to A6's "hash the bytes actually written" recommendation. |
| `arrow-ipc` compression-level configuration | 59.1.0 | **reject** — compression stays off on the identity path (A6) |
| `DictionaryArray::is_normalized` flag | 59.0.0 | **adopt as a check.** A normalized dictionary has no unreferenced values. §5.3 step 2 expands dictionaries before hashing, so identity is safe — but any *other* comparison of two batches is not (A5). This flag makes the property assertable instead of assumed. |
| `arrow-csv` header validation against a `Schema` | 59.1.0 | **adopt at the explicit tabular observation boundary**, not as an authoring-language replacement (ADR-0048; review L9) |
| Writing **RunEndEncoded** arrays directly to Parquet | 59.1.0 | folds into register #14's REE evaluation — REE is now viable end-to-end, not just in memory |
| `uuid` extension type from `FixedSizeBinary(16)` | 59.1.0 | **reinforces register #17's rejection** — the canonical `arrow.uuid` is now easier to adopt and still wrong for a blake3 digest |
| `arrow-flight` `skip_validation`, zero-copy tonic path | 59.1.0–59.2.0 | **reject** — no Flight tier |
| `product` aggregate kernel; `MapArray` concat/lengths; `i256` numeric traits; `Float16` CSV parsing | 59.1.0–59.2.0 | **reject** — no requirement in scope |

### 10.5 Third-party extenders — bounded survey

Surveyed so that a future "we should use X" is already answered. None is adopted.

| Crate | Latest | Adjudication |
|---|---|---|
| `typed-arrow` / `typed-arrow-derive` | 0.7.1 | **reject, and worth explaining.** It derives Arrow schemas from Rust types at compile time — superficially very close to this platform's spirit. It is the wrong direction: §4.2 generates Rust types **from** the registry, which is the authority (D1). `typed-arrow` makes the Rust type the authority, which would invert the dependency and create exactly the second definition **G1** forbids. |
| `serde_arrow` | 0.15.0 | **bound elsewhere** — already §3.3's, covered by the supporting-library map; the boundary there ("explicit schemas from the registry; no inference from sample data") is the right one |
| `quiver` | 0.6.1 | **evaluate, low priority.** A schema specification and validator for record batches. The platform generates its own validators from the registry (§4.2), so this would duplicate an authority; the only interesting part is its failure reporting. |
| `arrow-udf-wasm` / `arrow-udf-python` | 0.5.1 / 0.4.2 | **reject.** Sandboxed UDF runtimes. §18.5's kernels are native Rust behind `KernelSpec`; adding a WASM or Python execution tier inside kernel evaluation contradicts D10's boundary and **DM-58**. |
| `minarrow`, `lance-arrow-scalar`, `delta_kernel` | — | **reject** — alternative Arrow implementations and table formats; the platform's storage protocol is its own (§20.1) |

---

## 11. Principle-alignment register (Arrow)

Capabilities the blueprint does **not** claim, which would improve alignment with `DATA_MODEL_DESIGN_CHARTER.md`. Every row names a principle and the hand-written work or risk it removes. Rejections are included, because a recorded rejection is worth as much as an adoption.

In the previous edition these rows lived in the DataFusion map as one combined 47-row table. They are split here so each map stands alone; the DataFusion rows are in that map's §13. Numbering is local to this map.

### Adopt

| # | Capability | Cluster | DM / gate | What it removes or prevents |
|---|---|---|---|---|
| 1 | `Field::try_extension_type::<E>()` / `validate` inside the generated `try_from(&RecordBatch)` | A2 | DM-07, **G3** | A fingerprint match does not prove a field's extension metadata deserializes or that its storage type is supported. Generated, not hand-written. |
| 2 | Enable `canonical_extension_types` | A2 | **DM-42** | Without it the platform cannot *recognise* an inbound `arrow.json`/`arrow.uuid`/`arrow.bool8` column and reads it as bare storage. **Applied in revision 2** — retained because the register row is what justified it. |
| 3 | Enable `ffi` | A7 | — | Required for D12's C stream boundary to compile at all. **Applied in revision 2.** |
| 4 | `arrow_row::RowConverter` for canonical ordering | A5 | **DM-15** | A hand-written multi-column comparator — and the *only* option for keys containing `pse.index_tuple`, since the ordinary comparison kernels reject nested types. |
| 5 | `distinct` / `not_distinct` and Kleene booleans in the rule compiler | A5, A4 | **DM-08** | Ordinary `eq` yields null for null-vs-null, silently dropping candidate rows — which §14.2 rule 3 explicitly forbids. |
| 6 | `CastOptions { safe: false }` + `can_cast_types` at import | A5 | **DM-42**, G2 | `safe: true` turns an out-of-range cast into a null. Silent degradation at the §20.5 import edge. |
| 7 | Name the IPC alignment constant and `MetadataVersion` in the §5.3 contract | A6 | **DM-48** | Measured: alignment changes the bytes (872 vs 1032). An unnamed constant makes every `content_hash` depend on an Arrow default. |
| 8 | **Canonicalize the hashing representation explicitly** | A1 | DM-48, DM-15 | Metadata key/value sets are validated at construction; HashMap insertion order is not encoding order. v2 hashes a separate ordered metadata representation; plan encodings remain noncanonical [review:E5]. |
| 9 | Generated conformance test per `pse.*` extension type | A2 | **DM-44**, DM-53 | Ad-hoc tests; the `ExtensionType` trait defines exactly what to assert. |
| 10 | **`force_validate` in the test and CI cargo profiles** | §10.3 | **G3**, DM-54 | Turns "we believe the generated builders produce valid arrays" into a property checked on every array in the whole suite. One profile line. **New.** |
| 11 | **`ArrayFormatterFactory` for `pse.*` rendering in diagnostics** | §10.2 | **DM-49**, DM-47 | Hand-written display helpers per semantic type, or diagnostics that render a `pse.semantic_id` as 16 raw bytes. One factory, driven from §4.4's table, shared with DataFusion's matching hook. **New.** |
| 12 | **`DictionaryArray::is_normalized` as an assertion** | §10.4 | DM-15 | Makes "two logically identical batches carry identical dictionaries" checkable rather than assumed outside the hashing path. **New.** |

### Evaluate

| # | Capability | Cluster | DM | Why it is not yet an adopt |
|---|---|---|---|---|
| 13 | `Buffer::from_custom_allocation` to publish a native workspace as Arrow | A3 | DM-37, DM-29 | Removes a copy when publishing solver results; costs a lifetime obligation on the `Arc<dyn Allocation>`. |
| 14 | `RunEndEncoded`, `Utf8View` for repeated/string columns | A1 | DM-36 | **DM-58 applies** — no access-pattern measurement exists. 59.1.0 made REE writable to Parquet, so the route is now end-to-end; the missing input is still a measurement. |
| 15 | `arrow-pyarrow` vs `pyo3-arrow` | A7 | **DM-57** | One is first-party and version-locked to the Arrow family; the other adds PyCapsule and numpy. An unmade choice, and the compatibility-edge class of risk. |
| 16 | `arrow.opaque` for genuinely uninterpreted kernel values | A2 | **DM-04**, DM-42 | The standard cross-language way to say "from another system, not interpreted" — cheaper and more honest than inventing `pse.opaque`. |
| 17 | `arrow_buffer::MemoryPool` (the `pool` feature) | §10.2 | DM-30, G5 | Accounts for allocations DataFusion's pool cannot see. Wait until there is a budget to enforce. **New.** |
| 18 | `arrow_json::EncoderFactory` for extension-aware export | §10.2 | DM-42, G7 | Only if JSON export becomes a supported interface rather than a debugging convenience. **New.** |
| 19 | `arrow-ipc` sans-IO stream encoder | §10.4 | DM-48 | Would let A6's "hash the bytes actually written" happen in one pass instead of two. **New.** |
| 20 | `parquet::ArrowPredicate` and page indexes | §10.2 | DM-36, DM-26 | Durable Parquet is already a blueprint §5.4 scan source. Evaluate under R-24 with full-result equivalence and measured decoding/total-cost improvement (ADR-0048). |

### Reject, with reason

| # | Capability | Cluster | Why rejected |
|---|---|---|---|
| 21 | `arrow.uuid` for `pse.semantic_id` | A2 | §5.1 puts blake3 digests *and* UUIDv7 in one opaque column. Labelling a blake3 digest `arrow.uuid` asserts version/variant structure it does not have — a **G2** failure traded for cosmetic interoperability. |
| 22 | `Field::extension_type::<E>()` (the panicking variant) | A2 | A panic is not a typed failure (§23.2) and cannot become a `validation.invariant` row. **G3.** Ban by governance grep. |
| 23 | `Schema::normalize` / `flattened_fields` | A1 | They flatten nested structure into dotted names — the opposite of §4.5's direction. Named so nobody adopts them for convenience. |
| 24 | `Decimal32`/`Decimal64` for physical quantities | A1 | §4.5 is right that `Float64` is what a solver consumes. Worth a note only for costing (§19.5). |
| 25 | Sample-inferred schemas at platform ingestion | A8 | Reject inference as authority; explicit-schema Arrow CSV/JSON readers are adopted for the declared §6.10 observation interface (ADR-0048). |
| 26 | **`parquet-variant` family** | §10.1 | A Variant column is a typed hole in D1's "typed relations are the only authority". Rejected because it looks convenient, not because it is obscure. **New.** |
| 27 | **`parquet_derive`** | §10.1 | Derives schemas from Rust structs; §4.2 generates Rust from the registry. Adopting it inverts the authority direction — **G1**. **New.** |
| 28 | **`typed-arrow`** | §10.5 | Same inversion as row 27, at compile time. **New.** |
| 29 | **`arrow-udf-wasm` / `arrow-udf-python`** | §10.5 | A sandboxed execution tier inside kernel evaluation contradicts D10's boundary and **DM-58**. **New.** |
| 30 | `arrow-flight` / Flight SQL | A8 | No network tier. Note the residue: §3.1 still pins a crate with no consumer. |

---

## 12. Acceptance-gate review (Arrow)

This map establishes library surfaces and bounded observations, not platform gate acceptance. The earlier gate summary credited registration and incomplete canonicalization too strongly; the revision-4 review supersedes it.

| Gate concern | Current mechanism | Remaining evidence |
|---|---|---|
| G1 / G2 / G3 | Registry authority, complete physical admission and v2 typed canonicalization | Supported boundary rejection and metamorphic fixtures; E1/E3/E4 are counterexamples, not implementation acceptance |
| G4 / G6 | Safe owning views, explicit conversion and ordered numerical contracts | Lifetime, conversion and backend conformance |
| G5 / G7 | Logical/encoded identity split and truthful interface scope | Corrupt/partial/existing-object publication fixtures and complete provider comparisons |

Use the revision-5 design review for current document-stage gates and plan 02 for implementation acceptance.

## 13. Leverage matrix

| Blueprint requirement | API | Status |
|---|---|---|
| §4.3 schema- and field-level `pse.*` metadata keys | `Schema.metadata`, `Field::with_metadata` (both `HashMap<String,String>`) | confirmed |
| §4.3 "preserved through IPC/Parquet" | — | **confirmed empirically for both** `[probe]` — schema md, field md and `ARROW:extension:name` survive IPC (PROBE 1) **and Parquet** (PROBE 7) |
| §4.3 `pse.contract.fingerprint` reproducibility | `Schema.metadata` iteration | **refined** — IPC is stable regardless of key count `[probe]` (PROBE 6), so the previous edition's claim was wrong; the exposure is platform-side fingerprints and `datafusion-proto` (§15 item 3) |
| §4.4 extension admission | `try_extension_type`, `ExtensionTypeRegistry` factories and generated validation | X1 is a direct-helper check; ordinary query enforcement requires the active boundary [review:E1] |
| §4.5 logical type catalog | `DataType` (41 variants; the catalog is a conservative subset) | confirmed |
| §4.2 `try_from(&RecordBatch)` schema check | fingerprint is platform code; `Schema::contains` available | **refined** — add a structural check beside the fingerprint |
| §4.4 `ExtensionType` with `NAME`/`Metadata`/`serialize_metadata`/`deserialize_metadata`/`supports_data_type` | the trait, verbatim | **confirmed, incomplete** — `try_new` also required; `validate` + `try_new_from_field_metadata` provided and are the read path |
| §4.4 `pse.*` namespacing | `arrow.` reserved for canonical types | confirmed |
| §4.4 extension validation at the boundary | `try_extension_type` / `validate` | **adopt** — highest-value Arrow item (DM-07, G3) |
| §4.4 recognising inbound canonical types | `canonical_extension_types` feature | **adopt** — unmentioned; DM-42 |
| D11 borrow Arrow buffers | `PrimitiveArray::values`, `ScalarBuffer` | **Interface-checked** safe slice-aware route; owner lifetime, null rejection and reservation checks are platform obligations (ADR-0047; review L7) |
| D11 publish native workspace as Arrow | `Buffer::from_custom_allocation` | evaluate (DM-37, DM-29) |
| §4.4 `pse.bound` never null/NaN sentinels | null propagates through `cmp` kernels | **confirmed, with a second reason** |
| §5.3 step 4 canonicalise NaN, preserve `-0.0` | IEEE 754 `totalOrder` in sort/cmp | **confirmed necessary** `[probe]` — sorted order was `null, -1, -0, 0, NaN` |
| §5.3 step 1 order by primary key | `lexsort_to_indices`, or `RowConverter` | leverage; **`RowConverter` required** where keys include `pse.index_tuple` (nested types are unsupported by `make_comparator`/`lt`) |
| §5.3 step 2 concatenate batches | `concat_batches` | **confirmed load-bearing** `[probe]` — without it, identical data in different splits yields different bytes |
| §5.3 step 2 expand dictionaries | `cast` from `Dictionary(K,V)` to `V` | leverage |
| §5.3 step 3 IPC, no compression, deterministic alignment | `IpcWriteOptions::try_new(alignment, false, MetadataVersion)` + `try_with_compression(None)` | **confirmed expressible; alignment value is unspecified in §5.3** `[probe]` |
| §14.2 rule compiler null-aware joins/dedup | `distinct` / `not_distinct`, `and_kleene` / `or_kleene` | **adopt** — unmentioned; DM-08 |
| §20.5 declared import conversions | `cast_with_options`, `can_cast_types` | **Interface-checked** mechanisms; exact numeric/physical admissibility must be established first [review:E3] |
| §21.1 / D12 Arrow C stream boundary | `FFI_ArrowArrayStream`, `ArrowArrayStreamReader`, `ffi` feature | confirmed |
| §3.1 `pyo3-arrow 0.19.0` on the Arrow 59 line | requires `arrow-* ^59`, `pyo3 ^0.29` | confirmed — compatible with 59.3.0 |
| §3.1 one Arrow release family | umbrella-only pinning does **not** pin the family | **erratum — resolved in revision 2** (§1.2); the lockfile half is still open (§15 item 6) |
| §3.1 `arrow-flight` in the pin row | — | **residue** — pinned, no consumer (§10.1, amendment A′) |
| §24.1 test layers | `force_validate` cargo feature | **adopt, unmentioned** — a whole-suite structural-validity gate for one profile line (register #10, G3) |
| §23.2 diagnostics render semantic values | `arrow_cast::display::ArrayFormatterFactory` | **adopt, unmentioned** — today a `pse.semantic_id` renders as 16 raw bytes (register #11, DM-49) |

---

## 14. Evidence ledger

Rows 1–22 are the previous edition's, retained so its claims stay traceable. Rows 23–32 are this edition's. Row 15's extraction no longer exists on disk and is superseded by row 26; row 14's open question is closed by row 28.

| # | Cluster | Instrument | Source | Query / target | Outcome |
|---|---|---|---|---|---|
| 1 | setup | WebSearch | crates.io, docs.rs, github.com, datafusion.apache.org | latest Apache DataFusion release | returned **55.0.0** — later found **stale**; see row 12 |
| 2 | setup | WebFetch | `https://docs.rs/crate/arrow/latest` | latest `arrow` version, feature flags, sub-crate deps | 59.3.0 (2026-09-01); features incl. `ffi`, `pyarrow`, `canonical_extension_types`; umbrella over 10 required + 4 optional sub-crates |
| 3 | setup | c7 resolve | `DataFusion` | Rust query engine, TableProvider, LogicalPlan, ScalarUDFImpl | `/apache/datafusion` (5032 snippets, High) |
| 4 | setup | c7 resolve | `arrow-rs` | Rust Arrow, RecordBatch, schema, extension types, IPC, FFI | `/websites/rs_arrow_arrow` (12171); `/apache/arrow-rs` (291); `/apache/arrow` = **wrong language line** |
| 5 | setup | c7 query-docs | `/websites/rs_datafusion` (direct ID probe) | `TableProvider::scan_with_args` | **not found** — no docs.rs mirror for DataFusion on context7 |
| 6 | D1 | c7 query-docs | `/apache/datafusion` | custom `TableProvider` with projection, filter pushdown, statistics | served upstream `library-user-guide/custom-table-providers.md` + `upgrading/48.0.0.md`; API-accurate |
| 7 | setup | cargo | `cargo generate-lockfile --offline` | resolve `arrow "=59.2.0"` + `datafusion "=55.0.0"` | **finding**: `arrow-array`/`arrow-schema` floated to 59.3.0 inside an `arrow` 59.2.0 build |
| 8 | A2 | c7 query-docs | `/websites/rs_arrow_arrow` | `ExtensionType` trait, attaching to a `Field` | `extension_type_name`, `try_extension_type`, `EXTENSION_TYPE_NAME_KEY`/`_METADATA_KEY` |
| 9 | A2 | WebFetch | `docs.rs/arrow-schema/59.2.0/…/extension/index.html` | module contents, canonical types, feature gate | 7 canonical types + `CanonicalExtensionType`, gated by `canonical_extension_types` |
| 10 | A2 | WebFetch | `docs.rs/arrow-schema/59.2.0/…/trait.ExtensionType.html` | full trait definition and method contracts | `NAME` namespacing rule (`arrow.` reserved); `deserialize_metadata` error obligations |
| 11 | A4 | c7 query-docs | `/websites/rs_arrow_arrow` | null vs NaN in comparison/sort kernels, float ordering | **IEEE 754 `totalOrder`** for sort and `cmp`; null propagates to null; `SortOptions`, `make_comparator` |
| 12 | setup | WebFetch | `https://docs.rs/crate/datafusion-expr/latest` | latest `datafusion-expr` version | **55.1.0 (2026-09-11)** — sub-crates ahead of the assumed 55.0.0 |
| 13 | setup | WebFetch | `https://docs.rs/crate/datafusion/latest` | latest umbrella version, deps, features | **55.1.0 (2026-09-11)** — corrects row 1. Depends on `arrow ^59.2.0`, `object_store ^0.13.2`; full default/optional feature list |
| 14 | A1 | c7 query-docs | `/websites/rs_arrow_arrow` | is schema/field custom metadata preserved through IPC? | `Schema{fields, metadata}`, `new_with_metadata`, `RecordBatch::schema_metadata_mut`; **preservation not directly evidenced — see open items** |
| 15 | all | rustdoc | locally generated, `arrow 59.3.0` + `datafusion 55.1.0`, format_version 61 | full API surface of 30 crates | authoritative lane; receipt in §1 |
| 16 | A4, A6 | **probe** | locally compiled `src/bin/probe.rs` against `arrow 59.3.0` | IPC metadata round-trip; null/NaN/`-0.0` distinctness; `totalOrder` sort; batch-split byte equality; alignment sensitivity | all five results quoted inline in §5 and §7 |
| 17 | A6 | rustdoc | `arrow_ipc@59.3.0` | `IpcWriteOptions`, `StreamWriter`, `FileWriter` | `try_new(alignment, write_legacy_ipc_format, metadata_version)`, `try_with_compression(Option<_>)` |
| 18 | A5 | rustdoc | `arrow_select`/`arrow_ord`/`arrow_cast`/`arrow_row`/`arrow_arith`@59.3.0 | full function and struct inventories | `RowConverter`, `distinct`, `and_kleene`, `CastOptions`, `partition`, `garbage_collect_dictionary`, `BatchCoalescer` |
| 19 | A3, A7 | rustdoc | `arrow_buffer`/`arrow_array`/`arrow_schema`@59.3.0 | buffer zero-copy surface; FFI types | `from_custom_allocation`, `ptr_offset`; `FFI_ArrowSchema`, `FFI_ArrowArrayStream`, `ArrowArrayStreamReader`, `to_ffi`/`from_ffi` |
| 20 | A2 | rustdoc | `arrow_schema@59.3.0` | canonical extension `NAME` constants | `arrow.uuid`, `arrow.json`, `arrow.bool8`, `arrow.opaque`, `arrow.fixed_shape_tensor`, `arrow.variable_shape_tensor`, `arrow.timestamp_with_offset` |
| 21 | A7 | WebFetch | `https://docs.rs/crate/pyo3-arrow/latest` | version, `arrow`/`pyo3` requirements, features | 0.19.0; `arrow-* ^59`, `pyo3 ^0.29`, `numpy ^0.29`, `thiserror ^1`; `buffer_protocol` default |
| 22 | setup | cargo | `cargo generate-lockfile` / `cargo fetch` | coherent `=`-pinned resolution | Arrow family all 59.3.0; DataFusion family all 55.1.0; `object_store` 0.13.2; `tokio` 1.53.1 |
| 23 | §1.1 | crates.io API | `crates.io/api/v1/crates/{arrow,datafusion}` | is either pin stale? | **no** — `arrow` 59.3.0 (2026-09-01) and `datafusion` 55.1.0 (2026-09-11) are both still the current releases at this compile `[crates.io:arrow]`, `[crates.io:datafusion]` |
| 24 | §1.1 | GitHub raw | `apache/datafusion@55.1.0/Cargo.toml` | which Arrow does DataFusion build against? | `arrow = "59.2.0"` (caret) — the `=59.3.0` pin is one minor **ahead** of DataFusion's tested Arrow `[gh:apache/datafusion@55.1.0/Cargo.toml]` |
| 25 | §10.1 | GitHub raw | `apache/arrow-rs@59.3.0/Cargo.toml` | the E1 denominator | 27 workspace members; 23 publishable after removing build/test scaffolding `[gh:apache/arrow-rs@59.3.0/Cargo.toml]` |
| 26 | all | **rustdoc** | locally generated, full family, `=`-pinned, lockfile committed | complete API surface | **57 targets, 57 OK, 0 FAIL, 60 JSON documents, format_version 61**; arrow family 20 crates all 59.3.0. Supersedes row 15, whose output was not preserved. Receipt in §1.4 |
| 27 | §10.2 | facts corpus | `build/facts/arrow593` (normalized from row 26) | the E2 denominator | **101 public traits**, 30,920 declarations, 18,268 impl relations `[rustdoc:arrow593@59.3.0]` |
| 28 | A1, A6 | **probe** | `arrow_probe2.rs` against `arrow 59.3.0` | does metadata key count change canonical IPC bytes? does Parquet preserve metadata and write deterministically? | **PROBE 6** — IPC stable at 0/1/2/3/5 keys, which **refutes** this map's previous claim (§0.4). **PROBE 7** — Parquet preserves schema md, field md and `ARROW:extension:name`; two writes byte-identical; `created_by` embedded. Closes row 14's open question and the previous edition's unresolved item 1 |
| 29 | A2 | **probe** | `df_probe_x.rs` against `datafusion 55.1.0` | can a `pse.*` type be registered in DataFusion's `ExtensionTypeRegistry`, and what happens to an unregistered one? | **PROBE X1, direct helper only** — 7 canonical types preloaded; `pse.semantic_id` registers and resolves; wrong storage type **rejected** with a typed error; unregistered name **errors** rather than degrading |
| 30 | §10.4 | GitHub | `apache/arrow-rs` `dev/changelog/59.{0,1,2}.0.md` | the E4 delta | Parquet/GeoArrow extension-type round-trip (59.1.0) — explains PROBE 7; sans-IO IPC stream encoder (59.2.0); `DictionaryArray::is_normalized` (59.0.0); REE→Parquet (59.1.0) `[gh:apache/arrow-rs/dev/changelog]` |
| 31 | §10.5 | crates.io API | crates.io search, `q=arrow` | third-party extenders worth adjudicating | `typed-arrow` 0.7.1, `serde_arrow` 0.15.0, `quiver` 0.6.1, `arrow-udf-wasm` 0.5.1, `arrow-udf-python` 0.4.2, `pyo3-arrow` 0.19.0 |
| 32 | §1.3 | GitHub raw | `apache/arrow-rs@59.3.0/arrow/Cargo.toml` | the E3 denominator | the complete umbrella feature list, including the undocumented-in-blueprint `force_validate` and `pool` `[gh:apache/arrow-rs@59.3.0/arrow/Cargo.toml]` |

---

## 15. Open items and recommended blueprint amendments

The reviewed amendments are now **Proposed** blueprint revision-5 contracts. The existing probe receipts remain historical observations; no library receipt proves their platform integration.

| Item | Current disposition | Remaining gate |
|---|---|---|
| Active extension validation | §4.3–§4.4, ADR-0039; generated recursive admission | Actual provider/query/bundle/output paths and nested values |
| Canonical bytes and physical encodings | §5.3/§20, ADR-0045; v2 framing and null normalization | Metamorphic/round-trip/integrity tests; R-23 for larger relations |
| Metadata ordering | Ordered hashing representation, noncanonical protobuf evidence | No sorted-insertion equivalence claim; [review:E5] |
| Numeric/physical casts | §14.2 semantic admission, ADR-0039 | Fractional/large-integer and wrong-unit/basis/reference rejection |
| Buffer views and transfer | §18.2/§21.1, ADR-0047/0048 | Lifetime/cancellation checks; R-24 for coalescing benefit |
| Shared predicates / Parquet | §5.4, ADR-0048 | Full pushed/unpruned results; R-24 before additional pruning |
| Explicit-schema readers | §6.10, ADR-0048 | Declared tabular descriptor, strict fields/headers and unit/target validation |
| Optional layouts / custom allocation | Remain consumer/measurement gated | No new layout admitted merely because Arrow exposes it |

The [revision-4 evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md) supplies E1/E3/E4/E5; review L3/L7/L9 records the pinned source interfaces for the added bindings. Regeneration of the original evidence corpus was not part of this update.
