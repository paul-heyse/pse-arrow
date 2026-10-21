# Boundary and generation family: capability-gap inventory

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Capability-gap inventory: PyO3, pyo3-arrow, schemars, msgspec and related boundary libraries.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

This inventory covers pyo3 0.29.2, pyo3-arrow 0.19.0, pyo3-async-runtimes 0.29.0, pyo3-introspection, schemars 1.2.2, serde/serde_json, nutype 0.8.0, strum 0.28, enum-map 3.1, thiserror/miette, msgspec 0.21.1, cattrs 26.2, attrs 26.1 and pyarrow 25.0.1.

I read source at HEAD `0530dcaa1` (baseline `a791c533e` plus the audit pages). Nothing was built or run. The only inspection beyond `rg`/`sed` of installed sources was a byte search of the maturin binary for one string. I wrote no files.

Evidence labels: **Interface-checked** means read in the pinned source or docs. **Implemented** means seen in repository source. **Proposed** means a design inference. **Unresolved** is stated where it applies.

**Census correction.** Census §5 says pyo3-arrow's `PyRecordBatchReader` and `PySchema` have zero uses. They are used for export: `crates/pse-py/src/inspection/stream.rs:8`, `:109-133` (`__arrow_c_schema__`, `__arrow_c_stream__`). What has zero uses is Arrow **input**, meaning any pyo3-arrow `FromPyObject` in pse-py. I checked with `rg 'pyo3_arrow::'` over `crates/pse-py/src`; the only hits are `PyRecordBatch::new` exports.

---

## 1. Capability surface at the pins (relevant subset)

| Concern | Capability at pin | Source |
|---|---|---|
| Typed vocabularies across the boundary | `FromPyObject::INPUT_TYPE` / `IntoPyObject::OUTPUT_TYPE` consts, built with `type_hint_identifier!` / `type_hint_union!` / `type_hint_subscript!`. The introspector emits them into `.pyi`. | pyo3 `src/inspect.rs:21-67`; CHANGELOG 0.28 "Fill annotations…", #5634 |
| | `#[pyclass]` complex enums: variants become nested Python classes with stubs | CHANGELOG "Generate nested classes for complex enums" #5708 |
| | Simple-enum singletons | #5665 |
| | strum: `EnumString` (custom `parse_err_ty`), `VariantArray`, `IntoStaticStr` (`const_into_str`), plus unused `EnumDiscriminants`, `EnumTryAs`, `EnumIs`, `EnumCount`, `EnumTable` | `strum_macros-0.28.0/src/lib.rs:133-985` |
| | enum-map `Enum`/`EnumMap` (exhaustive per-key tables) | in use in pse-math, pse-kernels, backend |
| | msgspec decodes `enum.StrEnum` by value and `Literal` strictly | msgspec docs "Supported types" |
| Typed identities | schemars 1.x custom keywords in `json_schema!`, `#[schemars(extend(...))]`, `#[schemars(transform = …)]`, `SchemaSettings::with_transform` | `schemars-1.2.2/src/generate.rs:30-63,163`; repo precedent `x-pse-order` at `crates/pse-codegen/src/codegen/jsonschema.rs:275` |
| | msgspec `dec_hook(type, obj)` for custom types: a `TypeError`/`ValueError` it raises becomes a `ValidationError` with a path; `enc_hook` covers unsupported types; `NewType` is treated as its base | msgspec "Extending"; `json.pyi:19-56` |
| | msgspec `uuid.UUID` native, with `Encoder(uuid_format="hex")` | `json.pyi:27,35` |
| | pyo3 `PyBackedBytes`/`PyBackedStr`; optional pyo3 `uuid` feature (off) | `pyo3-0.29.2/src/pybacked.rs`; `src/conversions/uuid.rs` |
| Physical quantities and bulk data | pyo3-arrow input types: `PyRecordBatch`, `PyRecordBatchReader`, `PyArrayReader`, `AnyRecordBatch`, `AnyArray` (`FromPyObject` over the Arrow PyCapsule interface). `PyArray` keeps `FieldRef` so extension metadata survives. | `pyo3-arrow-0.19.0/src/input.rs:24-206`; `src/ffi/from_python/*.rs`; `src/array.rs:45` |
| | Arrow input needs no arro3; arro3 is needed only for `to_arro3` | `src/table.rs:83-96` |
| | pyo3-arrow types carry **no** `INPUT_TYPE`/`OUTPUT_TYPE` (stub limit) | `rg INPUT_TYPE` over the crate: empty |
| | msgspec `Meta(extra=…, extra_json_schema=…)` | `__init__.pyi:162-194` |
| | pyo3 `std::time::Duration` ↔ `datetime.timedelta`, including abi3, with `INPUT_TYPE` | `pyo3-0.29.2/src/conversions/std/time.rs:5-49` |
| Document versioning and schema-set identity | `SchemaSettings { contract: Contract::{Deserialize, Serialize} }` with `for_serialize()` / `for_deserialize()` | `generate.rs:176-214` |
| | Shared-generator `root_schema_for` plus `take_definitions` (one schema set, shared `$defs`) | `generate.rs:405-480` |
| | schemars honours serde `from`/`try_from`/`into`, contract-aware | `schemars_derive-1.2.2/src/schema_exprs.rs:55-75` |
| | msgspec `UNSET`, `omit_defaults`, int `tag`/`tag_field` | msgspec API docs |
| Runtime-owned document operations | serde internally or adjacently tagged enums plus schemars give a `oneOf` with a const tag. The repository generator already emits msgspec tagged unions for that shape. | `crates/pse-codegen/src/codegen/documents.rs:221-253,385-415` |
| Async and cancellation | `future_into_py_with_locals`: an asyncio cancel drops the Rust future (`Cancellable`) | `pyo3-async-runtimes-0.29.0/src/generic.rs:502-503,618-760` |
| | `unstable-streams` adapts Python async iterators to Rust streams (wrong direction for this need) | |
| | pyo3 `experimental-async` adds native `async fn` and `CancelHandle` (off) | |
| Structured errors | `create_exception!` works under abi3. `#[pyclass(extends=PyException)]` needs **Python ≥ 3.12** under abi3. | `pyo3-0.29.2/src/impl_/pyclass.rs:1134-1136` |
| | miette `Diagnostic::{code, help, related, labels, source_code, url}` | |
| Validated constructors | nutype on any inner type with `validate(predicate=…)` or `validate(with=…, error=…)`; `derive_unchecked(schemars::JsonSchema)` | `nutype-0.8.0/README.md:296-447` |
| Testability | pyo3 0.29 deprecates the `extension-module` feature in favour of the `PYO3_BUILD_EXTENSION_MODULE` env var | `pyo3-build-config-0.29.2/src/impl_.rs:1572-1582`; CHANGELOG 0.26 #5343 |
| | The maturin 1.15 binary contains that variable name (one byte match). Whether it sets it is unresolved. | |

## 2. What the repository uses (verified)

- **Enums.** Generated Rust registry enums derive `strum::{EnumString, Display, VariantArray, IntoStaticStr}` and serde (`crates/pse-model/src/generated/enums.rs:5-60`). Python gets 232 generated `StrEnum`s (`python/pse/contracts/enums.py`).
  - **Output only.** `EnumValue<T>` builds the Python member by `import + call` and declares `OUTPUT_TYPE = type_hint_identifier!("pse.contracts.enums", T::NAME)` (`crates/pse-py/src/enums.rs:16-28`). There is no `FromPyObject` for `EnumValue`.
  - Inputs go through `settings::named`, which does `serde_json::from_value(Value::String)` (`workflow/settings.rs:52-56`), or through hand `match`es.
- **Documents.** `DocumentValue<T>` encodes with serde_json and has Python decode the bytes through `pse.codec.decode_json` into `pse.contracts.documents.<NAME>`. `DocumentInput<T>` does the reverse: an isinstance check, then msgspec encode, then serde decode. Both use `INPUT_TYPE`/`OUTPUT_TYPE` (`crates/pse-py/src/documents.rs:243-289`).
  - About 71 hand `DocumentType::NAME` strings, and `RecordedDocument::VERSION` constants that restate `Version<N>` (`:105-503`).
  - The published document list is a separate hand list in `xtask/src/codegen/schemas.rs` (`documents()`, about 150 entries), each `schemars::schema_for!(T)` with default settings (Deserialize contract).
- **Versions.** `Version<N>` emits `const N` in its schema and refuses any other version (`crates/pse-model/src/document.rs:37-75`).
  - Python reads the const as `_pse_required_version` and checks headers by decoding to `object` and walking `msgspec.inspect` before the typed decode (`python/pse/codec/__init__.py:150-260`).
  - `decode_current(…, _version: fn(&T) -> &Version<N>)` ties `N` to the field at compile time (`documents.rs:62-71`), but `RecordedInput` uses the restated const instead (`:259-293`).
  - The import check covers the package version and the registry fingerprint only (`python/pse/_build.py:108-118`).
- **Identities.** The JsonSchema of a role newtype delegates to its base and drops the role name (`crates/pse-ids/src/newtype.rs:118-130`). `SemanticId` is a hex `pattern` (`id.rs:241-251`).
  - Python `SemanticId` is a `bytes` subclass whose hex codec is a native call (`python/pse/contracts/values.py:30-44`).
  - The binding parses hex text at about 27 call sites: 8 `id(py, …)` and 19 `declaration(py, …)`, from `rg '[^a-z_](declaration|id)\(py, '` over `crates/pse-py/src`.
  - Arrow fields already carry the role as `pse.domain.identity` metadata (`crates/pse-schema/src/generated/standard-data.jsonl:1-2`).
- **Quantities.** The extension `pse.quantity_value` is a struct of `{value, quantity_type_id, unit_id}` (`python/pse/contracts/extension_types.py:150-160`). It is not used on any Python→Rust input.
- **Operations.**
  - The runtime already has a versioned, document-driven operation descriptor: `StudyOperation { version: Version<7>, preparation: PreparationSettings, operation: OperationRequest, … }`. `OperationRequest` is serde `tag="kind", content="request"` with the variants DeclaredCase/Simulation/Fit/Horizon (`crates/pse-runtime/src/workflow/study_operations.rs:66-115,305-349`).
  - The PyO3 binding does not use it. It has its own 3-kind `PreparedOperation` (`crates/pse-py/src/workflow.rs:2427-2431`).
  - `SimulationSettings` is a 20-keyword projection that applies defaults in the binding (`workflow.rs:2313-2390`), even though `SimulationProfile` is already a published document (`schemas.rs`, "simulation-profile").
- **nutype.** Four scalars only: `Tolerance`, `Fraction`, `PositiveCount`, `FiniteBound` (`crates/pse-model/src/scalars.rs:11-85`). `LimitsWire` and `pse_modeling::Limits` use bare `usize` (`study_operations.rs:109-115`; `crates/pse-modeling/src/specialize.rs:57-72`).
- **Errors.** There is one `create_exception!(InspectionError)` carrying a `DiagnosticReport` attached by `setattr` (`crates/pse-py/src/inspection/errors.rs:19-33,496-510`).
  - Recovery facts are nine `Option<String>` trait methods.
  - On a Python signal, those keys are `setattr`'d onto the signal exception (`workflow.rs:540-575`).
- **Async.** `wait_async` uses `future_into_py_with_locals` with a `CancelWait` guard whose `Drop` cancels the handle (`workflow.rs:1725-1773`). The sync paths poll `detach` and `check_signals` every 100 ms and join before propagating (`workflow.rs:392-600`).

## 3. Unused and under-used capabilities

### U1. One runtime-owned operation descriptor at the PyO3 boundary

**Capability.** Make `StudyOperation`/`OperationRequest`/`PreparationSettings` the one operation document the binding accepts. A runtime entry point (Proposed: `ModelingPackage::prepare(OperationDescriptor)`) composes `declared_execution` and the operation under the descriptor's `PreparationSettings`.
- Supporting library pieces:
  - Derive the operation set once with strum `EnumDiscriminants` (`#[strum_discriminants(name(OperationKind), derive(VariantArray, IntoStaticStr))]`).
  - Use `EnumTryAs` for typed per-kind extraction in place of `_ => None` arms.
  - Use `EnumMap<OperationKind, …>` for per-kind tables that must be exhaustive.
- The schema is already generated, and the Python generator already emits the adjacently-tagged union.

**Owns.**
- DA-015 operation chains, the 11 hard-coded `Profile`/`NumericalInputs` defaults, and the operation-dependent allowances. Every operation then takes `PreparationSettings`.
- The binding policies (deadline rule `workflow/modeling.rs:399-404`; synthesized events) move into the runtime, where they are testable.
- DA-009's "no single owner of the operation set" is owned by the discriminant.
- DA-021 is owned in part: composition leaves the `test = false` crate.

**Removes.** pse-py `PreparedOperation`; the 13 binding chains; the binding default selection; and the `SimulationSettings` keyword projection, replaced by `DocumentInput<SimulationProfile>`.

**Limits.**
- strum does **not** make the request/report pairing structural. That needs one enum whose variants carry both request and `Result<report>`, which is plain Rust.
- `Shooting` is behind `cfg(feature = "solver-diffsol")`, so the schema varies with the feature set (probe P6).
- `OperationRequest` lacks Shooting today, so it must be completed.

**Reverse check.**
- Value rebind: the studies' rebind path is the same descriptor, so it is strengthened.
- Lane parallelism and the SurrealDB lifecycle: untouched.
- Daemonless durability: improved, because the in-process and durable descriptors coincide.
- DP-03 ingress re-validation: still happens at the runtime entry.
- Strict decoding: `deny_unknown_fields` is preserved.

*Interface-checked* that the descriptor exists; adoption is *Proposed*.

### U2. Typed vocabulary input at PyO3, symmetric to the existing output

**Capability.** Add `impl FromPyObject for EnumValue<T: RegistryEnum + FromStr>`:
- `INPUT_TYPE = type_hint_identifier!("pse.contracts.enums", T::NAME)`.
- Strict: an `isinstance` check against the generated `StrEnum`, then `T::from_str` through the existing strum `EnumString` with `VocabularyError`.
- Declare the missing vocabularies as registry enumerations: scientific artifact kind, artifact owner kind, terminal class.
- Two options for `NativeLiveSimulationEvent`:
  - a pyo3 complex enum (nested classes with stubs, #5708; the `Sample` variant holds the `TableStream`);
  - keep the class, but make `kind` and `termination` `EnumValue<…>` (`TrajectoryTermination` exists).

**Owns.** All of DA-004 once the registry declarations exist. DA-007's store-column side belongs to the persistence family.

**Removes.**
- `workflow.rs:1157-1162`, the artifact-kind `match`.
- `:1345-1353`, the `TerminalClass` serde-`Value` trick.
- `settings::named`.
- `:2806-2884`, the event kind/termination strings.
- `python/pse/_artifacts.py:12-13`, the `Literal`s; and the `Sequence[str]` filters (`_workflow.py:252`).

**Limits.** A pyclass enum cannot be a `StrEnum`, and msgspec/cattrs cannot decode into one. Keep `StrEnum` as the single Python vocabulary type and use complex enums only for live native objects.

**Reverse check.** Strictness rises. No science moves to Python. *Proposed.*

### U3. Role-preserving identities in documents and at the binding

**Capability.**
- `semantic_id_newtype!` and `hash_role!` emit a custom keyword, e.g. `{"type":"string","pattern":…, "x-pse-identity":"RunId"}`, through `json_schema!`, following the `x-pse-order` precedent.
- The document generator maps the keyword to `identities.<Role>`, the same NewTypes the relation rows use.
- The codec registers msgspec `dec_hook`/`enc_hook` for `SemanticId`/`ContentHash`: strict `binascii.a2b_hex` plus a width check, with no native call.
- At PyO3, an `Identity<R>` `FromPyObject` accepts the Python `SemanticId` bytes (`cast::<PyBytes>()` plus a width check). `INPUT_TYPE` is `type_hint_identifier!("pse.contracts.identities", R)`, and a mirror `IntoPyObject` handles output.
- Canonical record keys get a Rust key newtype with an `OUTPUT_TYPE` NewType over `str`.

**Owns.** DA-005's identity-kind loss (about 262 hex-`str` fields; `study(StudyId)` versus `forget_study_results(study: str)`), the about 27 binding hex parses, and the per-element native hex calls.

**Limits (Unresolved, probe P1).** msgspec documents dict subclasses as "encoding only; decode needs `dec_hook`". If bytes subclasses behave the same, `SemanticId` would encode as **base64** and never reach `enc_hook`. Fallbacks:
- (a) make the Python `SemanticId` a non-`bytes` value class, with cattrs/Arrow hooks wrapping `bytes`;
- (b) per-role NewType over hex `str`. This is static-only and leaves two Python identity representations.
- (c) use `uuid.UUID` everywhere: msgspec natively with `uuid_format="hex"`, the pyo3 `uuid` feature, and pyarrow's canonical `arrow.uuid`. All are native, but uuid semantics such as `.version` are meaningless for hashed identities, and msgspec decodes UUID subclasses for encoding only.

I lean to (a).

**Reverse check.** Strict decoding is kept because the hook refuses bad input; Rust re-validates every input. *Proposed.*

### U4. Physical meaning on bulk inputs and outputs through pyo3-arrow input and existing extension types

**Capability.**
- Accept `pyo3_arrow::AnyRecordBatch`/`PyRecordBatch` from any `__arrow_c_stream__`/`__arrow_c_array__` producer (no arro3 needed).
- Validate it against a registry relation contract, using existing pse-columnar contract checking, whose columns are `pse.semantic_id` targets and `pse.quantity_value` values. Extension metadata crosses the C data interface in field metadata (`pyo3-arrow src/array.rs:45`).
- Uses:
  - `with_primal_start` (`python/pse/_runs.py:235`), initialization stages (`_modeling.py:926-930`), `diagnose_samples`;
  - `atol`, `out_atol` and `parameter_scales`, keyed by state identity instead of position;
  - output `NativeAttempt.primal()` as an exported `PyRecordBatch` instead of `list[tuple[str, float]]` (`workflow/strategies.rs:396-433`).
- For document fields: a `SimulationProfile` map keyed by state identity (the generator already supports single-pattern `patternProperties` maps, `documents.rs:911-925`).
- Seconds: `ClosedDuration` in documents, or `std::time::Duration` ↔ `timedelta` (pyo3 `time.rs:24-36`, abi3-safe) for the 15 seconds-as-float sites.
- Fixed-unit fields: `x-pse-unit` via schemars `extend`, mapped to msgspec `Meta(extra=…)`.

**Owns.** DA-005 inputs and outputs, and PS-12 (no solver-order knowledge).

**Limits.**
- Python still needs a resolver from unit/quantity name to id. Exposing the pse-quantity catalogue as an inspection `TableStream` is a minor completion; it is unresolved whether the registry can host it.
- pyo3-arrow has no `INPUT_TYPE`, so a local wrapper with `INPUT_TYPE = type_hint_identifier!("pyarrow", "Table")` is needed. Otherwise the stub check in `xtask/src/codegen/python_stubs/validate.rs:327` would reject `Incomplete`.

**Reverse check.** Arrow bulk transfer is extended to inputs. Results are still re-qualified. *Proposed*, plus probe P8.

### U5. One boundary-document catalogue and schema-set identity

**Capability.**
- Move the document list into a library crate as `(T, slug, Direction::{Input, Output, Recorded})`, consumed by both xtask and pse-py.
- Require a `Versioned` bound at catalogue roots, either through the existing `fn(&T) -> &Version<N>` accessor tie or an associated `type V = Version<N>`.
- Use one `SchemaGenerator` for the set. `Contract::Serialize` for outputs makes always-emitted fields required, so Python decoding of Rust output becomes stricter. Keep `Contract::Deserialize` for inputs.
- Take a blake3 fingerprint (blake3 is already in pse-ids) of the canonical set. Generate it into Python, and have the extension compute it at runtime from its compiled types (`_native.document_schema_fingerprint()`). schemars is already a normal dependency of every owner crate.
- `_check_native_compatibility` compares the two.
- Convert the about 37 bytes-returning methods to `DocumentValue<T>`, so stubs are typed and Python's hand `decode_json(…, Type)` pairings (`_runs.py:96` and the others) go away.
- Check `DocumentType::NAME` against schemars `schema_name()` in a runtime-crate unit test. `type_hint_identifier!` needs a const, so the name cannot be computed.

**Owns.**
- All of DA-011: two lists, hand pairing, restated versions, schema drift at import, and unversioned evolution such as `Completion`, which now needs `Version<N>` to enter the catalogue.
- The reverse check below.

**Limits.**
- Feature-dependent schemas (P6).
- Rust importing the generated Python package stays. The `pse.codec` import goes away if outputs decode with a cached `msgspec.json.Decoder(type)`, which is safe because Rust produced the bytes at the same build.

**Reverse check.**
- Version-first refusal for inputs and recorded documents is unchanged.
- Skipping the version walk is safe only for Rust-produced outputs.

**msgspec `UNSET` assessed.** Not recommended broadly. Python defaults are already derived from the Rust `Default` through the schema's `default`, which is one authority checked by codegen-check. `UNSET` would hide those values from Python tooling that reads them, such as conformance defaults. Use it only where omitted must differ from explicitly-equal.

**msgspec int `tag="version"` assessed.** Not recommended. It conflicts with the existing `tag_field="kind"` unions, removes the `.version` attribute, and has unresolved error ordering (P2).

### U6. Validated constructors in the owning types

**Capability.**
- `PositiveCount` (nutype) for `Limits`/`LimitsWire`. This gives schema `minimum: 1`, Python `Meta(ge=1)`, and a serde refusal.
- `LeasePolicy` through `#[serde(try_from = "LeasePolicyWire")]` (schemars follows `try_from`/`into`) or a nutype `validate(with=…, error=LeaseError)` over a struct inner type, so "heartbeat < lease" and the defaults have one owner.

**Owns.** DA-015's invariants that only the adapter enforces.

**Removes.** The `ModelingLimits` zero check (`workflow/modeling.rs:152-158`), the `ge=0` mismatch, the worker-only check (`xtask/src/bin/pse-worker.rs:404-411`) and the restated defaults.

**Reverse check.** None. *Proposed.*

### U7. Typed exception classes and typed recovery facts

**Capability.**
- Use `create_exception!` subclasses of `InspectionError`, one per registry `FailureClass`, generated from the registry so Python can `except` by class. This is abi3-py311-safe.
- Replace the nine `Option<String>` recovery methods and the `setattr`s on `KeyboardInterrupt` with one serde/schemars tagged `RecoveryHandle` document carried in the report. Today `artifact_owner_kind: Option<String>` is part of DA-004. Chain it as the `__cause__` of the signal, which is already done for the error branch.
- `#[pyclass(extends=PyException)]` with typed `#[pyo3(get)]` fields would delete the stub supplement (`python_stubs/exceptions.rs`, 156 lines) and the `setattr` attach. It **requires abi3-py312**, which drops the 3.11 floor (parity runs on 3.11–3.14). That is a tradeoff, so I recommend it only as a maintainer decision.

**Owns.** Part of DA-004; adjacent to DA-008. *Proposed.*

### U8. Binding-crate testability

**Capability.** Drop the deprecated `extension-module` feature; maturin signals extension builds through `PYO3_BUILD_EXTENSION_MODULE`. pse-py can then link libpython for `cargo test`, with `auto-initialize` for embedded tests.

**Limits.** U1 is the better remedy, because logic leaves the binding. The cdylib/sccache rationale (`crates/pse-py/Cargo.toml:19-31`) needs a probe (P4). Whether maturin sets the variable is unresolved.

**Owns.** The "untestable placement" residue of DA-015. *Proposed.*

### U9. Rust-owned conformance manifest

**Capability.** A versioned `ConformanceManifest` document in pse-runtime that composes `PreparationSettings`, `ConformanceControls`, `NativeSolveIntent` and `NativeBackend`, parsed in Rust by `toml::from_str` (the toml crate is already a pse-runtime dependency). The generated msgspec type is decoded in Python with `msgspec.toml.decode`.

**Owns.** The conformance part of DA-019.

**Removes.** `python/pse/conformance.py:49-108`, the restated derivative defaults, and the hand vocabulary subsets.

**Not this family.** `scripts/surreal_server.py`'s SQL writer belongs to the persistence family. For the diagnostics launch duplicate, the boundary's only contribution is that `_native.diagnostic_launch_capture` exists, so Python could call the Rust owner. I have not verified that interpretation.

### No family capability found

- **DA-031.** No family capability owns it; it is compiler/orchestration.
- **DA-021.** Only indirectly: U1 moves composition out of the binding, and policy inputs can be documents. The `Runtime`-in-values cause is not a boundary-library matter.

## 4. Clear-win improvements no DA finding names

1. **Cache the type lookups.**
   - Today: `DocumentValue`, `DocumentInput`, `RecordedInput` and `EnumValue` call `py.import(…)?.getattr(NAME)` on every value.
   - Change: `pyo3::sync::PyOnceLock::import` (`once_lock.rs:158`) and a cached `msgspec.json.Decoder` per type.
   - Speed benefit: *Proposed*, not measured.
2. **Zero-copy document ingress.**
   - Today: `DocumentContent` copies `str`/`bytes` into a `Vec` (`documents.rs:8-29`).
   - Change: `PyBackedBytes`/`PyBackedStr` are `Send`, keep the Python object alive, and avoid the copy across `detach`.
3. **Skip the version walk on Rust-produced output.** The decode-to-`object` walk for outputs Rust produced at the same build is redundant; keep it for inputs and recorded documents.
4. **Consistent input allowance.** `DocumentInput` hard-codes a 1 MiB allowance (`documents.rs:252`), while sibling routes take operation-specific allowances. Route it through the caller.
5. **Typed live events.** A complex-enum or `EnumValue` `NativeLiveSimulationEvent` also removes seven `Option` getters that hold per-variant fields (`workflow.rs:2810-2880`).
6. **Recycle route.** Read the route from the prepared strategy instead of `Route::Native(Kinsol)` (`workflow/strategies.rs:71-78`). This is plain Rust, listed because it rides with U1.

## 5. Unresolved questions and probes

Each probe is a read-only or scratch command; none was run.

| # | Question | Probe | Expected observation | Settles |
|---|---|---|---|---|
| P1 | Does msgspec 0.21.1 encode a `bytes` subclass natively (base64) without calling `enc_hook`, and does decoding into it call `dec_hook` with the JSON `str`? | `.venv/bin/python -I -c` that defines a `bytes` subclass and encodes/decodes it in a Struct with both hooks | base64 on encode; hook on decode | the U3 representation (a/b/c) |
| P2 | For a single (non-union) Struct with `tag_field="version", tag=2`, are a missing or mismatched tag refused, and does the tag error come before body errors? | Same, with `{"x":"bad","version":1}` | refusal; precedence unknown | whether msgspec can replace `_check_version_headers` |
| P3 | Does `JsonSchema::schema_name()` equal the generated class title for every catalogue type, including the aliases `ModelingSourceMap` and `InvocationEvidence`? | Unit test in pse-runtime over the catalogue | aliases differ, handled by explicit alias entries | U5 NAME checking |
| P4 | Does pse-py link and run `cargo t` once `extension-module` is removed, and do maturin builds still omit libpython? | Worktree: drop the feature, `scripts/pse-env -- cargo t -p pse-py --no-run`, `just py-sync`, `ldd` on the `.so` | test binary links `libpython3`; the wheel `.so` has no libpython NEEDED | U8 |
| P5 | What does pyo3-introspection 0.29.2 emit under abi3 for a complex enum whose variant holds a `#[pyclass]` `TableStream`? | Build a scratch crate and run `cargo xtask python-stubs` | nested classes, typed fields | U2 live-event option |
| P6 | Does the document schema set depend on the feature set (`solver-diffsol` Shooting variant)? | Generate schemas with and without the feature and diff | differs | the fingerprint must be the extension's own runtime computation, not a generated const |
| P7 | After asyncio cancels `wait_async`, does Python observe `CancelledError` before the native drain completes, unlike the sync path, which joins? | Scripted cancel, timestamping `CancelledError` against job join (with native solvers) | early return; compute release still waits for the join (`math/jobs.rs:1428-1490`) | whether async cancellation needs a "drained" acknowledgement (DP-19 truthfulness of "stopped") |
| P8 | Does a pyarrow table with `pse.quantity_value` and `pse.semantic_id` columns arrive in `PyRecordBatch` with `ARROW:extension:name`/`metadata` intact? | Scratch extension or an existing inspection round trip | metadata preserved | U4 |

**Material uncertainties.**
- **Units in the registry.** It is unresolved whether units and quantity types can be registry-declared identity domains (DA-005 says not today). U4's resolver depends on it.
- **Raising the abi3 floor** (U7) is a maintainer tradeoff, not a clear win.
- **Absence claims and their coverage.** The no-Arrow-input claim, the zero `EnumDiscriminants`/`EnumTryAs` uses and the zero `omit_defaults`/`UNSET` in generated documents come from `rg` over `crates/pse-py/src`, `crates/*/src` (strum) and `python/pse/contracts/documents/__init__.py`. An empty search is not proof of absence.