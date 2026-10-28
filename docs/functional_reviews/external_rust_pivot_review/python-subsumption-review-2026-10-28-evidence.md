# Evidence appendix: Python subsumption review (2026-10-28)

These are the six independent slice reports behind `claude/python-subsumption-review-2026-10-28.md`, taken at pse-arrow snapshot `bd1c4891e`.

Each slice was produced by a separate reviewer reading the source. Measurements come from a cloud sandbox (CPython 3.13, warm bytecode, no systemd user manager) unless a report states otherwise.

| Slice | Scope |
|---|---|
| A | Product package and PyO3 boundary |
| B | SurrealDB operator and stores |
| C | Host control, admission, build and caches |
| D | Test orchestration, validation, evidence and scientific references |
| E1 | FastMCP capability interface |
| E2 | Telemetry and developer tooling |

---

## A — `python/pse` and the PyO3 binding boundary

Snapshot: `/home/claude/pse` (HEAD bd1c4891e). Scope: `python/pse/**`, `python/control_tests/`,
`python/stubs/`, root `conftest.py`, compared against `crates/pse-py` (PyO3 extension
`pse._native`) and `crates/pse-codegen` (+ the xtask generators that emit Python).
Evidence labels: *Interface-checked* = read from source; *Measured* = run here (conditions stated).

### 0. Headline numbers

| Bucket | LOC | Share of non-test `python/pse` |
|---|---:|---:|
| Generated document mirror `contracts/documents/__init__.py` | 35,835 | 60.3% |
| Other generated registry mirrors `contracts/*.py` (rows, enums, values, ext types, identities) | 12,179 | 20.5% |
| Generated operation projections `generated/*.py` | 1,272 | 2.1% |
| Generated stub `_native.pyi` (pyo3-introspection) | 1,848 | 3.1% |
| **Generated subtotal** | **51,134** | **86.0%** |
| Handwritten (excl. tests) | 8,331 | 14.0% |
| — of which forwarding facades + export gate | 4,447 | (53% of handwritten) |
| — mirror support (codec, governance) | 776 | (9%) |
| — CLIs / subprocess orchestration (diagnostics CLI, conformance, trace query, capsule reader) | ~1,650 | (20%) |
| — durable-control HTTP client (`control.py`) | 573 | (7%) |
| — genuinely Python-ecosystem logic (`_transfer`, `_array`, `parity/host`) | 425 | (5%) |
| Tests `python/pse/tests` | 20,921 | — |
| Parity tests `python/pse/parity/tests` | 1,161 | — |
| `python/control_tests` | 781 | — |
| root `conftest.py` | 477 | — |

Total `python/pse` = 81,547 lines (59,465 non-test + 22,082 tests). *Interface-checked* (wc -l).

Handwritten code contains essentially no domain logic. An AST pass classifying functions with
≤2 statements and no control flow as "trivial" gives, e.g., `_modeling.py` 137/142 trivial,
`_runs.py` 94/97, `_workflow.py` 50/50, `_project_operations_glue.py` 41/43. The non-trivial
remainder is concentrated in `control.py`, the `diagnostics.py` CLI, `conformance.py`,
`_diagnostic_query.py`, `codec/`, `governance.py` and `_transfer.py`.

### 1. How the contracts are generated (the actual pipeline)

Two independent generated Python class systems mirror Rust, plus two operation/stub generators:

1. **Boundary documents (msgspec).** Rust serde types are the authority (≈260 catalogue roots
   registered in `crates/pse-py/src/generated/documents.rs`, 171 of them with distinct
   input/output directions). schemars derives draft-2020-12 JSON Schemas, published to
   `docs/generated/schema/` (498 files, 661K lines, 21 MB). A custom closed emitter
   (`crates/pse-codegen/src/codegen/documents.rs`, ~1,250 lines of Python emission in
   `Emitter::*`/`python_module`, plus ~680 lines of tests) turns the schemas into frozen
   `msgspec.Struct` classes in `python/pse/contracts/documents/__init__.py`: **1,873 classes**
   (1,792 Structs, 81 local StrEnums), every one with a generated `_pse_equality_key`, 463 with
   `_pse_required_version`, 1,331 `msgspec.Meta` constraints. datamodel-code-generator was
   rejected because it emits `Any`/unfrozen structs (blueprint §21.5).
2. **Registry rows/vocabularies (attrs + cattrs).** `pse-codegen/src/codegen/python/{mod,types,
   extensions,identifiers}.rs` + two `.py.template`s (~1,500 lines) emit `enums.py` (265 StrEnums),
   `values.py` (SemanticId/ContentHash + validators), `identities.py` (NewTypes), `canonical.py`,
   `structures.py`, `extension_types.py` (11 pyarrow ExtensionTypes) and one attrs module per
   relation namespace (`runtime.py` 320 row classes, `authored.py` 126, `reference.py` 39, …).
3. **Project operations.** `xtask/src/codegen/project_operations.rs` reads
   `pse_runtime::host_operations::project::methods()` and *parses* `crates/pse-py/src/workflow/
   project_operation_glue.rs` with `syn` to pair each declaration with a hand-written Rust
   `_host_*` helper; it emits both `crates/pse-py/src/generated/project_operations.rs` (PyO3
   methods) and `python/pse/generated/project_operations.py` (Python methods calling
   hand-written Python `_host_*` glue). 53 operations × 4 layers.
4. **Stub.** `xtask/src/codegen/python_stubs.rs` uses `pyo3_introspection::introspect_cdylib` on
   the built extension to emit `_native.pyi` — this one is already the PyO3-native mechanism.

**What `pse._native` exposes directly** (*Interface-checked*, `crates/pse-py/src`): 51
`#[pyclass]` (50 `frozen`), 52 `#[pymethods]` blocks, 48 `#[pyfunction]`; Arrow via `pyo3-arrow`
(`TableStream.__arrow_c_stream__`); async via `pyo3-async-runtimes` (2 `future_into_py` sites);
86 `py.detach` GIL releases; signal-aware waits in `wait.rs`. Documents cross as
`DocumentValue<T>` (402 uses) / `DocumentInput<T>` (160 uses) — see §3.1.

### 2. Summary table

| Module/group | LOC | What it does | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---:|---|---|---|---|---|---|
| `contracts/documents/__init__.py` | 35,835 | msgspec mirror of every Rust boundary document, incl. 489 `*Output` twins | GEN (DUP) | serde types in pse-runtime/pse-operations/pse-backend-native/pse-buildinfo; registration `pse-py/src/generated/documents.rs` | DELETE (expose Rust types directly) | pyo3 `#[pyclass(frozen)]` wrappers generated by pse-codegen, or `pythonize`/`serde-pyobject` + TypedDict stubs | L |
| `contracts/runtime.py`, `reference.py`, `normalized.py`, `provenance.py`, `canonical.py` | 6,319 | attrs row classes for registry relations; **0 production consumers** (runtime rows used by 5 test files only) | GEN (DUP) | pse-relations/pse-model generated rows; results cross as Arrow streams | DELETE | — | S |
| `contracts/authored.py`, `structures.py`, `values.py`, `identities.py` | 2,426 | attrs authoring rows (2 used in prod: `AuthoredModelingDeclarationsRow`, `AuthoredFitCasesRow`), id byte types, validators | GEN (DUP) | pse-authoring, pse-ids, pse-schema | DELETE rows (native constructors); keep id types only if not pyclass | pyo3 pyclass | M |
| `contracts/enums.py` | 3,098 | 265 StrEnums (+ unused `IDAES_NAMES`) | GEN | `pse_model::generated::enums`; bridge `pse-py/src/enums.rs` | KEEP-GEN (cheap) or pyclass enums | pyo3 `#[pyclass(eq)]` enums | S |
| `contracts/extension_types.py` | 317 | 11 pyarrow ExtensionType subclasses + metadata codec | GEN (DUP) | `pse-schema/src/ext_metadata.rs`, `pse-relations/src/ext` | KEEP-PY only if pyarrow typed-extension UX wanted; else DELETE (pyo3-arrow already carries `ARROW:extension:*` metadata) | — | S |
| `generated/project_operations.py` (+defaults) + `_project_operations_glue.py`, `_research.py`, `_study_workbench.py` | ~1,990 | 53 public Runtime methods: generated wrapper → hand glue → native → Rust glue → owner | GEN + WRAP | `pse_runtime::host_operations`, `pse-py/src/workflow/project_operation_glue.rs` | DELETE (make `NativeRuntime` the public `Runtime`; generated PyO3 methods already carry doc comments, the Python-only "Effects:" block moves there) | pyo3 | M |
| `_native.pyi` | 1,848 | stub from compiled metadata | GEN | pyo3-introspection | KEEP (grows when natives become public) | pyo3-introspection | — |
| Facades: `_modeling`, `_runs`, `_workflow`, `_strategies`, `_artifacts`, `_analyses`, `_inspection`, `_physical`, `_project_bundle`, `_host_operations`, `_telemetry`, `_diagnostic_launch`, `modeling`, `authoring` | 2,822 | ~43 attrs/slot classes each wrapping one native pyclass; properties forward, `codec.encode_json(controls)`, rewrap returned handles | WRAP | the 51 pse-py pyclasses | DELETE (Plan 57i I02/I03 "native public types" already drafted) | pyo3 | M |
| `__init__.py`, `_scientific/__init__.py`, `_build.py` | 908 | lazy re-export of ~150 names; sole `_native` import gate; package/registry/schema fingerprint check; env-selected native image loader | WRAP (+small ORCH) | `registry_fingerprint`, `boundary_schema_fingerprint` in `pse-py/src/lib.rs` | DELETE re-exports and fingerprint checks with the mirror; KEEP-PY a ~30-line shim (`_load_published_image`, lazy import) | — | S |
| `codec/__init__.py`, `codec/_generated.py` | 575 | strict cattrs/msgspec (de)serialisation, version-header tree walk calling native `check_document_header`, identity hooks, attrs-row→document bridge | DUP | `pse_runtime::boundary_documents::{Wire, check_required_header}`, serde `deny_unknown_fields`, nutype domains | DELETE with the mirror | — | S (after mirror) |
| `governance.py` | 201 | lint: no `Any`/bare dict/list in generated classes; also run inside every `codec.converter()` | NEW-DEVTOOL (redundant) | generator already refuses open constructs (`documents.rs` "closed mapping") | DELETE | — | S |
| `diagnostics.py` | 1,316 | ~35 thin wrappers over native diagnostic/review functions (~450 L) + `argparse` CLI "just diagnostics" with 16 subcommands, text rendering, request-file reading, handoff evidence merge (~850 L) | WRAP + NEW-IFACE | `pse-py/src/diagnostics.rs` → `pse_runtime::tooling::{capture, review_pack, review_query, review_analysis, frozen_inspection}`; clap CLI `pse-project` exists | PORT-EXISTING (clap subcommands in a pse-runtime bin; evidence merge into `frozen_inspection`) / DELETE wrappers | clap, arrow-ipc | M |
| `_diagnostic_query.py` + `_diagnostic_query_child.py` | 292 | runs optional `trace_processor_shell` via a Python re-exec child that sets `RLIMIT_FSIZE`; process-group kill/drain, sleep-polling, CSV result validation | ORCH (DUP) | SQL/spec via `diagnostic_query_specification`; `pse-diagnostics/src/supervised_child.rs` (`SupervisedChild`: process group, drain, cancellation) | PORT-EXISTING | `SupervisedChild` + `rustix`/`libc` setrlimit in `pre_exec`; `csv` or arrow-csv | S–M |
| `conformance.py` | 404 | `python -m pse.conformance` CLI: manifest decode→native admit, package file walk, RunSettings→EngineSettings/ModelingLimits mapping, IPC report writing, summary/exit code | ORCH + NEW-IFACE | `pse_runtime::workflow::conformance_documents`, native conformance | PORT-EXISTING | clap, arrow-ipc, `ignore`/std fs | S–M |
| `_scientific_inputs.py` | 98 | hand-written reader for xtask's `scientific-inputs-v1` capsule (bytes stored as JSON int arrays) | DUP | `xtask/src/scientific_inputs.rs::Capsule` (also re-read in `scripts/` and `tests/fixtures/scientific-inputs.rs`) | PORT-EXISTING (one Rust reader) | serde | S |
| `control.py` | 573 | async HTTP client (httpx2+anyio) to SurrealDB `DEFINE API /pse/control/v1/{browse,cancel,reconcile}`: bounded request/response, original deadline, cancellation-unresolved identity, receipt matching, u64 decimal-string projection | NEW-IFACE + NEW-CONTROL (partial DUP) | `pse-operations/src/canonical_control.rs` (same browse/cancel/reconcile via SurrealDB SDK + `fn::pse_control_v1`, `finish_control_cancel` receipt predicate); server surql generated by `pse-codegen/src/codegen/surreal_control.rs` | PORT-LIB (separate small PyO3 module sharing the Rust types/predicates) or KEEP-PY if "no native load" stays a hard requirement without a second cdylib | reqwest or hyper (in tree), pyo3-async-runtimes | M |
| `_transfer.py` | 233 | classifies per-field extension-type survival (retained/storage_only/metadata_lost/mismatch) between source and a pyarrow consumer schema | NEW-OBS | extension metadata owner in pse-schema; no Rust comparator | KEEP-PY (it observes pyarrow registration state, which is lost through the C schema) | (arrow-schema if ported) | S |
| `_array.py` | 75 | null-refusing, zero-copy-by-default Arrow→numpy | KEEP-PY boundary | — | KEEP-PY | (rust-numpy if ever) | — |
| `parity/` (+tests) | 117 + 1,161 | IDAES/Pyomo 2.13.0 comparisons, host probe | ORACLE | — | KEEP-PY | — | — |
| root `conftest.py` | 477 | pytest policy: marker discipline, parity/perf/producer gating, native-library preflight, xdist inventory | NEW-DEVTOOL | — | KEEP-PY (shrinks with suites) | — | — |
| `python/stubs/` | 26 | README only; references removed `python/pse/adapters/pyomo/**` | stale | — | DELETE | — | — |
| `control_tests/` | 781 | tests for `control.py` (fake transports) | test | — | follows `control.py` | — | — |

### 3. Per-group notes with evidence

#### 3.1 The document crossing (why the mirror costs more than its line count)

`crates/pse-py/src/documents.rs` (*Interface-checked*):

- **Rust→Python** `DocumentValue<T>::into_pyobject`: `T::WIRE.to_vec(&self.0)` (serde→JSON bytes),
  then `py.import("pse.contracts.documents").getattr(T::OUTPUT_NAME)` and
  `py.import("pse.codec").call_method1("decode_json", …)` on *every* crossing. `codec.decode_json`
  then, for any type containing a versioned struct (463 of them), walks the raw JSON tree in
  Python (`_check_version_headers`, re-decoding `msgspec.Raw` slices at each level) and calls back
  into Rust (`pse._build.check_document_header`) per nested versioned struct, then decodes with
  msgspec; every `SemanticId`/`ContentHash` field goes through `dec_hook` → `SemanticId.from_hex`
  → native `semantic_id_from_hex` (one FFI call per identity value). Plan 57i ("Trust-aware
  document crossings", I01) already identifies the redundant walk and per-crossing imports.
- **Python→Rust** `DocumentInput<T>::extract`: `isinstance` against the Python mirror class,
  then Rust calls *back into Python* `pse.codec.encode_json(value)` to get bytes, then
  `decode_document` (header pre-check + `Wire::decode_slice`). A second convention coexists:
  ~35 call sites in the wrapper modules pre-encode with `codec.encode_json(controls)` and the native signature
  takes `&[u8]` (28 such parameters).
- Telemetry uses a third convention (native returns a JSON `str`, `_telemetry.py` decodes it with
  `msgspec.json.decode(..., type=dict[str, str | None])`).

So validation runs twice (msgspec `forbid_unknown_fields` + `Meta` bounds on the Python side;
serde `deny_unknown_fields` + nutype + version headers on the Rust side), and defaults are
written twice (e.g. `AdjointSettings.max_checkpoints = 400` in the generated class and in the
Rust `Default`). §21 says "Python declares no compiler defaults", but the mirror does carry them.

#### 3.2 Direction twins

489 `X`/`XOutput` pairs exist; **187 are byte-identical modulo the class name** (3,734 lines)
and the other 302 differ only in that an `Optional` field has `= None` on input and is required
(nullable) on output (e.g. `AdviceObservationOutput.value`). All `*Output` classes total 10,868
lines (30% of the documents module). They leak into hand code: `_project_operations_glue.
_host_project_branches` copies `ProjectCursorBody` → `ProjectCursor` field by field;
`conformance._preparation_document` does the same for `PreparationSettingsBody(Output)`.
*Interface-checked* (script over the generated module).

#### 3.3 Registry attrs rows: almost entirely unused

522 attrs row classes are generated. Outside `contracts/`, production code references exactly two
(`AuthoredModelingDeclarationsRow`, `AuthoredFitCasesRow` via `_modeling.py`, `modeling.py`,
`codec/_generated.py`). All 15 `runtime.*Row` classes referenced anywhere are referenced only
from tests (`test_native_workflow`, `test_physical_boundary`, `test_generated_contracts`,
`test_modeling_strategies`, `test_plan14_acceptance`), which `codec.structure_rows(...)` Arrow
results back into attrs for assertions. Production results cross as Arrow streams
(`TableStream`), never as rows. The two used row types are themselves converted
attrs→cattrs dict→msgspec document→JSON in `codec/_generated._encode` before reaching Rust —
two Python class systems bridged to reach one serde type.

#### 3.4 Facades (WRAP)

Pattern (e.g. `_modeling.ModelingResult`, `_runs.RunResult`, `_workflow.Runtime`): an
`@attrs.frozen` class holding `_handle: _NativeX`, every property `return self._handle.x`,
every method returning `X(self._handle.method(...))`. ~43 such classes map 1:1 to native
pyclasses. Residual non-forwarding logic is tiny: `StudyReport.result` (pick
Stored/Compact/Run wrapper), `StrategyResult.failures` (filter), `StudyObserver.follow`
(Python `time.sleep` polling loop until terminal), `RunResult.bind_parameter_covariance`
(exists only to unwrap a handle the facade hides). The project-operation path is
4 layers deep for each of 53 methods (Rust owner → Rust `_host_*` → generated `#[pymethods]` →
Python `_host_*` → generated Python method that copies annotations via `_public_metadata`).
Plan 57i I02/I03 (draft, operator selected the "conditional native-public-type target" on
2026-10-27) already proposes making forwarding-only native classes public.

#### 3.5 Export gate (`__init__`, `_scientific`, `_build`)

`pse/__init__.py` lists ~150 names lazily resolved from `pse._scientific`, which is 379 lines of
`from X import Y as Y`. `_build.py` is the only module allowed to import `_native` (ast-grep
`no-direct-native-import`) and re-binds ~90 native symbols to private names, then at import:
checks wheel/extension version, `registry_fingerprint()` vs `contracts.REGISTRY_FINGERPRINT`,
`boundary_schema_fingerprint()` vs `SCHEMA_SET_FINGERPRINT`, and registers the pyarrow extension
types. The two fingerprint checks exist only because a separately generated Python mirror can
drift from the binary. `_load_published_image` (load `pse._native` from
`$_PSE_NATIVE_PUBLISHED_IMAGE`) is real deployment behaviour worth keeping as a shim.

#### 3.6 `diagnostics.py`

Lines ~139–440 and 711–1100 are one-line wrappers over `_diagnostic_*` pyfunctions. Lines
440–710 and 790–1316 are an argparse CLI (`main`, `_add_review_commands`, `_review_command`)
invoked by `just diagnostics` (`justfile:332`) and as `python -m pse.diagnostics`, with
hand-written text rendering (`_render_inspection`), exit-code policy, 64 MiB request-file
reading, `EngineSettings` construction from `DiagnosticRuntimeSettings` (hand field mapping,
also repeated in the `export` branch and in `conformance.run_once`), and `_subject`, which merges
a handoff evidence set into a primary one (dedup by operation id) — evidence-composition logic
that belongs beside `pse_runtime::tooling::frozen_inspection`. A Rust clap CLI already exists
(`crates/pse-runtime/src/bin/pse-project.rs`, `project/tools.rs`).

#### 3.7 `_diagnostic_query.py` (subprocess ORCH duplicating Rust supervision)

`run_query` asks Rust for the declared SQL/columns (`_diagnostic_query_specification`), then
supervises the external tool itself: `subprocess.Popen([sys.executable, "-I",
_diagnostic_query_child.py, allowance, *argv], start_new_session=True)` — a fresh interpreter
whose only job is `setrlimit(RLIMIT_FSIZE)` + `execv` — then 20 ms sleep-polling, `os.killpg`
on timeout/cancel/descendants, a 2 s drain poll, and CSV header/int64 validation. This is run
twice per query (`--version`, then `query`). Rust already has
`pse_diagnostics::supervised_child::SupervisedChild` (618 lines: process-group ownership,
cancellation, drain policy, stderr prefix capture) and `pse-runtime/src/tooling/review_launch/
execution.rs`. Port = reuse `SupervisedChild` with a `pre_exec` setrlimit.

#### 3.8 `conformance.py`

CLI over native conformance (`ModelingConformance.pure` / `ModelingPackage.conform`). Reads
the TOML manifest through `codec.decode_toml` then `_admit_conformance_manifest` (Rust), walks
package directories for `.toml/.yaml/.pse/.parquet`, maps `RunSettings` fields onto
`EngineSettings` and `ModelingLimits` by hand, writes six Arrow IPC files with pyarrow, counts
fixture statuses, prints one line. `just conformance` runs it as
`python scripts/scientific_inputs.py run -- python -m pse.conformance` — two interpreters per run.

#### 3.9 `_scientific_inputs.py` (DUP of an xtask format)

Declares `_Capture{schema, files: dict[str, list[int]], families}` for the
`scientific-inputs-v1` capsule whose writer is `xtask/src/scientific_inputs.rs::Capsule`
(`files: BTreeMap<String, Vec<u8>>`, serialized as JSON integer arrays). Same format is
re-read by `scripts/scientific_inputs.py` and `tests/fixtures/scientific-inputs.rs` — four
readers, one writer, no generated contract. Lives in the product package although it serves
repository reference journeys only.

#### 3.10 `control.py` (new today)

Added under ADR-0232 (proposed, dated 2026-10-28) so durable browse/cancel/reconcile can run
without loading the scientific extension. It re-implements a bounded HTTP client
(`urlsplit` loopback check, request-size pre-estimation `_extent`, streamed bounded response,
`anyio.fail_after` original deadline, `BaseExceptionGroup` cleanup accounting,
`ControlUnresolvedError` on ambiguous delivery) and response matching (`receipt.issuing_store/
operation_id/study` equality, 409 refusal classification, u64 decimal-string parsing and
projection `ControlBrowseResponse`→`StudyWorkbenchPageOutput`). Rust's
`CanonicalStore::cancel_study_command`/`finish_control_cancel`/`reconcile_study_cancel`
implement the same command semantics (same receipt predicate, same
`control_command_identity_reused` refusal) through the SurrealDB SDK and
`fn::pse_control_v1::*` with a `'native-system'` actor. Two ingress paths, two validators of one
server transition. If the user's thesis wins, the natural Rust shape is a second, tiny PyO3
module (no scientific crates linked) that reuses `canonical_control` types — ADR-0232 explicitly
chose "no new crate" for the first delivery, so this is a live decision point.

#### 3.11 `_transfer.py`, `_array.py`, `parity/` (keep)

`_transfer.compare_schemas` distinguishes `pa.ExtensionType` (registered in this pyarrow) from
`pa.BaseExtensionType` and raw `ARROW:extension:*` metadata; a Rust comparator receiving the
consumer schema through `__arrow_c_schema__` would see only metadata, so the
"retained vs storage_only" distinction is inherently a pyarrow-consumer observation. `_array`
is the documented numpy boundary (§21.6). The parity harness needs Pyomo/IDAES and must stay
independent of Rust (ORACLE); it consumes the public `pse` API plus a few contract types.

### 4. Can PyO3 expose the Rust types directly?

Already in place: frozen pyclasses for every handle, `pyo3-introspection` stubs, `pyo3-arrow`
capsules, `pyo3-async-runtimes`. What is not: documents and rows are mirrored rather than
exposed. Options, in increasing depth:

1. **Native classes become the public API** (Plan 57i I02/I03). Delete the ~2,800-line facade
   layer, the 908-line export gate (keep a small shim), and the Python half of the
   project-operation projection (~2,000 lines incl. generated). The xtask generator then emits
   only `#[pymethods]` (they already carry doc comments; the "Effects:" block now emitted only
   into Python docstrings would move there). Equality, `repr`, context-manager
   and iterator protocols move onto pyclasses (`__eq__/__hash__`, `__enter__/__exit__`,
   `__iter__/__next__`, `__anext__` via pyo3-async-runtimes). Stubs keep coming from
   introspection. Effort M; no Python mirror change required.
2. **Documents without a Python class system.** Two realistic realisations:
   - *Generated frozen pyclass wrappers.* Extend `pse-codegen` (which already emits
     `impl DocumentType` for all 260 roots into pse-py) to emit `#[pyclass(frozen)] struct
     X(Arc<T>)` with `#[getter]`s returning nested wrappers/enums, `#[new]` built through serde
     (so serde defaults, `deny_unknown_fields`, nutype bounds and `Version<N>` remain the only
     authority), `__eq__/__hash__` from Rust `PartialEq/Hash`. Orphan rules mean the wrappers
     live in pse-py, not on the owner types. Shapes needing care: 64 `serde(tag=…)` unions
     (pyclass complex enums fit), 5 `flatten`, 3 `untagged`. Removes the JSON hop, the
     header walk, per-id FFI hooks, and both directions' Python validation.
   - *`pythonize`/`serde-pyobject`* (neither is in the tree today): Rust↔Python plain
     dict/list values via serde, typed only by generated `TypedDict` stubs. Smallest code, but
     loses attribute access and runtime immutability. Plan 57i records the trade-off ("would
     remove the JSON hop but change byte allowance, finite-value and encoded-document
     guarantees; no measured need"). Note the finite-value guarantee currently relies on the
     Python side (`codec/_generated._check_finite`, because msgspec encodes NaN as `null`),
     i.e. the Rust types would have to own finiteness. That is arguably where it belongs.
   Either way `contracts/documents` (35.8K), `codec` (575), `governance` (201), most of
   `values.py`, the attrs rows, the two import-time fingerprint checks, `GENERATED.sha256` ×3,
   `python-contracts-check`, and the ~1,250-line Python emitter in `documents.rs` +
   `codegen/python/*` (~1,500) disappear. `docs/generated/schema` stays only if MCP/scripts
   keep consuming it (`scripts/capabilities/native_schema.py` does). Effort L.
3. **Irreducibly Python.** IDAES/Pyomo parity harness and its pytest config; numpy/pyarrow
   consumer conveniences (`_array`, `_transfer`, optional pyarrow extension registration);
   a thin package shell (lazy import, published-image loader); possibly a Python control
   client if the "no native image" requirement is kept *and* a second cdylib is refused.
   Everything else in `python/pse` is either mirror, forwarding, or a CLI that has a Rust
   home.

### 5. Tests (brief)

`python/pse/tests` (20,921 lines), classified by file docstring and imports (*Interface-checked*):

| Category | LOC | Files (examples) |
|---|---:|---|
| Mirror/codec/stub/facade plumbing | 2,489 | `test_generated_contracts` 592, `test_native_boundary_contracts` 479, `test_any_lint` 231, `test_generated_edit_codec` 220, `test_standalone_document_wire` 189, `test_project_operation_facade` 188, `test_extension_round_trip` 135, `test_native_stub_surface` 127, `test_native_registry` 112, … |
| Mixed (document versioning/settings projection; part Rust admission) | 881 | `test_solve_settings_projection` 630, `test_contextual_accuracy_transport` 251 |
| Handwritten-Python logic | 553 | `test_transfer_contracts` 281, `test_parity_context` 157, `test_nullable_ndarray_refusal` 60, `test_preflight_capability_backend` 55 |
| Fixtures/helpers | 1,705 | `managed_study_fixture` 538, `conftest` 385, `canonical_signal_control` 265, … |
| End-to-end Rust behaviour driven through the Python API | 15,293 | `test_modeling_kernel` 1,572, `test_studies` 1,446, `test_native_workflow` 1,399, `test_canonical_results` 1,380, `test_diagnostics` 1,374, `test_review_workflows` 959, … |

Plus parity 1,161 (ORACLE) and `control_tests` 781 (for `control.py`). ~12% of the suite
(16% with the mixed rows) exists only because of the mirror; ~73% asserts Rust semantics through Python (studies, canonical
results, diagnostics, modeling) — under a Rust-first split most of that belongs in Rust
integration tests, keeping a slim Python smoke layer for GIL release, signals
(`test_execution_waits`), Arrow capsule export and async. 16 test files use `subprocess`
(47 referencing lines) to spawn fresh interpreters for CLI and isolation checks.

### 6. Cross-cutting observations

**Duplicated sources of truth.**
1. Boundary documents: serde → JSON Schema (498 files, 21 MB) → 1,873 msgspec classes, with
   defaults and 1,331 constraints restated; validation on both sides.
2. Version-header admission: Rust `check_required_header` vs Python `_check_version_headers`
   (which calls back into Rust, and has a Python-only `native_header=False` path used by
   `control.py`).
3. Registry rows: 522 attrs classes; 2 used in production.
4. Record equality: 1,792 generated `_pse_equality_key` methods re-implement Rust equality
   (signed zero, unordered maps) for Python uniqueness validators.
5. Arrow extension types: `pse-schema/ext_metadata.rs` vs 11 pyarrow `ExtensionType`s.
6. Operation surface: Rust declaration + 2 Rust layers + 2 Python layers per method.
7. `scientific-inputs-v1` capsule: one xtask writer, four hand readers.
8. Durable control: Rust SDK path vs Python HTTP path to one server transition.
9. Subprocess supervision: `SupervisedChild` vs `_diagnostic_query._bounded_child`.
10. Settings mapping (RunSettings/DiagnosticRuntimeSettings → EngineSettings) hand-coded
    3× in Python; a Rust `From` impl would be the single mapping.

**Assurance that exists only because of the mirror:** `codegen-check` Python targets +
`GENERATED.sha256`, `python-contracts-check` (`scripts/check_python_contracts.py` +
`governance.py`), import-time registry/schema fingerprint checks, the `DocumentType`
registration file (1,915 generated Rust lines), ~2.5–3.4K lines of mirror tests, and ~3.4K
lines of Rust generator + generator tests emitting Python.

**Performance-relevant.**
- No Python on a numerical hot path (§21: no numerical callbacks; numpy only in `_array`;
  waits/signals are native). *Interface-checked*.
- Control-plane crossings pay JSON serialize + Python decode + a Python tree walk with nested
  native callbacks + one FFI call per identity field. Not measured here (no built extension in
  the snapshot); Plan 57i I01 targets part of it.
- Import cost of the mirror, *Measured* (this sandbox: CPython 3.13.18, msgspec 0.22.0,
  attrs 26.1.0, warm bytecode, copy of the tree, 3 runs): `contracts.enums` ≈42 ms,
  `contracts.documents` ≈52 ms, `contracts.authored` ≈95 ms (imported by `_modeling`), and
  `contracts.runtime` ≈250 ms (test-only). The first import of documents+runtime+authored,
  including bytecode compilation, took ≈0.9 s. Every CLI invocation (`just diagnostics`, `pse.conformance`) and every interpreter
  that imports `pse` pays this before the extension loads.
- Per-invocation interpreters: `just conformance` = 2 interpreters; each trace query = 2 extra
  `python -I` children used only to call `setrlimit` and `exec`.

**Smaller findings.** `contracts.enums.IDAES_NAMES` is generated but has no consumer.
`authoring/__init__.py` is an empty phase-0 package. The `referencing` runtime dependency in
`pyproject.toml` is used only by `scripts/evidence_comparison_report.py`, not by `pse`.
`python/stubs/README.md` refers to the removed `python/pse/adapters/pyomo/**`.
Hand-written projections between `*Body`/`*Output` twins (`_host_project_branches`,
`_preparation_document`) are friction that the direction split creates.

**Suggested order (lowest risk first).** (1) Delete the unused attrs row modules and the test
helpers that structure Arrow into them. (2) Make native pyclasses public (I02/I03) and delete
the facades, the Python project-operation layer and the re-export gate. (3) Move the
diagnostics/conformance CLIs and the trace-query runner into a Rust clap binary on
`SupervisedChild`. (4) Replace the msgspec document mirror with generated pyclass wrappers
(or pythonize + TypedDict stubs), then delete codec, governance, fingerprints and the Python
emitters. (5) Decide ADR-0232's control client: Rust micro-extension or retained Python.

---

## Slice B: SurrealDB server/tooling and durable-store cluster

Snapshot: `/home/claude/pse` (HEAD bd1c4891e). All paths are relative to the repo root.
Scope: 20 assigned modules (21,254 LOC), `scripts/surrealdb-source/` (2,962 patch+doc lines),
`database/canonical-schema/` (24,323 lines of frozen generated copies, 1.4 MB) and their tests
(27,249 LOC across 38 test/check files).

Evidence labels: the measurements here are *Measured* only where stated (sandbox, Python 3.13,
not the project's pinned venv); everything else is code reading (*Interface-checked* at most).

---

### 1. Summary table

| Module/group | LOC | What it does | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---|---|---|---|---|---|---|
| `surreal_server.py` | 11,748 (+26,171-line frozen import closure) | Supervises the patched `surreal` server as a systemd user unit. Python `_serve` is the long-lived ExecStart. Owns setup/config/credentials, CPU/memory placement, worker/observer/primary scopes, readiness, quiesce/stop, backup/restore/rebuild/maintenance, file-owner reaping, recovery qualification, MCP headers and metrics. | NEW-CONTROL + ORCH (secondary: DUP, NEW-STORE, NEW-OBS) | The gRPC client, schema, codecs, payload file ops, deployment allocation, process identity and pidfd supervision (see §2.4) | PORT-LIB into a Rust supervisor binary that keeps the external server | `zbus` + `zbus_systemd` (or `tokio::process` `systemctl` as a first step), `sd-notify`, `rustix` (already in tree: pidfd, flock, affinity, statvfs), `procfs-core` (in tree), the existing `pse_operations` gRPC client, `pse_diagnostics::supervised_child` | L |
| `surreal_source.py` + `scripts/surrealdb-source/` | 615 + 2,962 | Clones SurrealDB 3.3.4, applies 13 patches plus a DiskANN fix, runs `cargo build`, then records receipts and installs | ORCH | Pins in `Cargo.toml` | PORT-EXISTING (xtask build step); patches stay | `cargo_metadata`, `sha2`, `tar` (all in tree); `gix` optional | S–M |
| `surrealism_module.py` | 604 | Runs `surreal module build` on `crates/pse-study-module`, inspects the `.surli` tar and publishes an immutable receipt | ORCH + DUP | `canonical_study_module.rs::StudyModuleReceipt` reads the same receipt, and Rust reads the same `surrealism.toml` | PORT-EXISTING (xtask; Rust owns the receipt type) | `tar`, `sha2`, `toml` | S |
| `surreal_diagnostics.py` + `surreal_telemetry.py` | 264 + 244 | Health, metrics and audit CLI. `exact_parser` runs `surreal validate --stdin`. A hand-rolled HTTP/2 + gRPC + protobuf OTLP trace sink. | NEW-OBS | `xtask surrealql-validate` (surrealdb-syn in-process), `scripts/grpc_wire`-style decoding, `tonic`/`prost` | PORT-LIB | `tonic` + `opentelemetry-proto` (0.33.1 already in lock), `hyper` | S–M |
| `storage_health.py` + `build_storage.py` | 769 + 217 | Read-only host/cgroup/meminfo/statvfs/`/metrics` snapshot (msgspec), and a `du`-style survey of build stores | NEW-OBS | `pse_diagnostics::process_facts` (procfs-core) | storage_health: PORT-EXISTING. build_storage: KEEP-PY or xtask | `procfs-core`, `rustix::fs::statvfs`, `walkdir`/`ignore` (in tree) | S |
| `backup_image.py` | 246 | Verified copy of the stopped metadata image (O_TMPFILE + linkat + SHA256) | NEW-STORE / DUP | `canonical_files/offline.rs::copy` implements the same O_TMPFILE+linkat protocol for payloads | DELETE (reuse the Rust offline copy) | `rustix` (in tree) | S |
| SurrealKit beta cluster: `surrealkit.py`, `surrealkit_maintenance.py`, `surrealkit_population.py`, `surrealkit_source.py`, `surreal_rollout_producer.py` + `database/canonical-schema/` | 899 + 1,297 + 299 + 279 + 425 = 3,199, plus 24,323 data lines | Builds the upstream `surrealkit` CLI from git, provisions fixture namespaces and runs beta rollouts through child processes. Also hand-writes fixture rows/SQL and publishes frozen rollout chains. | ORCH + DUP | `xtask canonical-validate-definitions` / `canonical-activate` (`pse_operations::canonical_maintenance`), registry-generated schema/codecs | DELETE (beta parity tooling, unused in production under the fresh-regeneration policy). If rollouts become production, PORT-LIB. | `surrealkit` as a library (uncertain whether usable/published at 1.0.0-beta.6) | M (delete: S) |
| `evidence_store.py` | 658 | Builds the `pse-evidence` xtask binary, writes tooling credentials/config, takes the tooling activity lease and execs the Rust tool for init/import/query/analyze | WRAP/ORCH + DUP | `xtask/src/development/evidence.rs` (all DB I/O, DDL, users, validation), generated `store/evidence.surql` | PORT-EXISTING into `pse-evidence`. Option: embedded engine (§5.2). | existing; optionally `surrealdb` crate `kv-surrealkv` | S–M |
| `knowledge_store.py` | 495 | Builds the knowledge corpus page from `workspace_domains` (outside this slice), checks bounds and execs `pse-evidence knowledge-*` | ORCH + DUP | `xtask/src/development/knowledge.rs` (same bounds, version 2) | PORT-EXISTING, together with `workspace_domains` | `pulldown-cmark`/`comrak` if doc parsing moves | M |
| `assessment_store.py` | 248 | Python-only report codec: content-addressed `details/*.json` plus an atomic `checks.json` root (VERSION 6) | NEW-STORE | Nothing (Rust sees only the evidence projection) | KEEP-PY until `validation`/`test_resources` move; then PORT-EXISTING | `serde`, `sha2` | S |
| `resource_store.py` + `service_borrowers.py` + `resource_conversion.py` | 1,005 + 179 + 631 | Test-resource registry: JSON files with flock, invocation/resource/borrower authority. It is used for service drain decisions and is called *from Rust fixtures* through a Python subprocess bridge. Also a one-time v2→v3 format migration. | NEW-STORE + NEW-CONTROL | Rust fixtures only shell out to it (`pse-operations/src/testing.rs::resource_bridge`) | PORT-EXISTING (Rust registry crate). resource_conversion: DELETE. | `rustix::fs::flock`, `serde_json`, `tempfile` | M |
| `artifact_publication.py` | 132 | Copies a built binary into an immutable generation dir and atomically replaces a pydantic descriptor | NEW-STORE | Same pattern exists in Rust (`publish` logic in canonical files, xtask producer receipts) | PORT-EXISTING (xtask) | `sha2`, `tempfile`, `serde` | S |

Third-party Python usage across the slice is close to zero: only `msgspec` (surreal_server via generated
`deployment.py`, storage_health, surreal_diagnostics) and `pydantic` (artifact_publication). Everything
else is stdlib (`subprocess`, `fcntl`, `os`, `urllib`, `http.client`, `socketserver`). **No module in this
slice depends on the Python ecosystem.**

---

### 2. `surreal_server.py`: internal structure and what it actually executes

#### 2.1 Section map (by line range)

| Lines | LOC | Responsibility |
|---|---|---|
| 1–158 | 158 | Imports 14 sibling modules. Loads `crates/pse-operations/src/generated/surreal-policy.json`. `SupervisorError`, `ServiceObservation`. |
| 159–571 | 413 | Borrowed-service drain for exclusive host allocations (`drain_borrowed_services`, `service_processes_drained`); `ServiceAdmission` composes `host_admission` with storage lifetime |
| 572–734 | 163 | JSON/fs helpers, private-directory checks, `state_lock` (fcntl flock) |
| 735–1151 | 417 | `publish_generation`: freezes the supervisor's Python import closure into `.generations/<sha>`. `lifecycle_reservation`; context admission open/close gates. |
| 1152–1552 | 401 | `readmit_supervisor`, legacy telemetry settlement, `verify_generation` |
| 1553–1998 | 446 | `register_context` (receiver contexts), `tooling_activity` lease (used by evidence/knowledge), admitted runtime/controller checks |
| 1999–2429 | 431 | `config_for` (untyped `config.json` validation), `resources` (shells to Rust `pse-deployment`), `install`, `setup`, `reconfigure` |
| 2430–3212 | 783 | `systemctl()` wrapper; worker slot units (`systemd-run --user --scope`, MemoryMax/TasksMax), slices, cgroup limit reads, `taskset` CPU pinning, `worker()` |
| 3213–3487 | 275 | Host-owner registration and reconciliation of pending launches |
| 3488–4908 | 1,421 | Observer and primary-receiver lifecycle: scopes, control limits, `ensure_primary`, `observe_primary`, dispatch handoff |
| 4909–5455 | 547 | `drain_context`, `public_status`, readiness: `/health` HTTP probe, gRPC readiness via the `surreal sql` CLI, `/proc/*/fd` socket-inode listener ownership |
| 5456–6564 | 1,109 | Storage placement/launch records, `materialize_service` (writes the unit file, `daemon-reload`, `enable`), park/unpark, `start`/`stop`/`release_stopped_service` |
| 6565–7200 | 636 | Study-module selection, `server_environment` (≈25 `SURREAL_*` env knobs), log config, `BoundedLog` (rotation + password redaction), `_serve` (Popen of `surreal start`, signal forwarding) |
| 7201–7856 | 656 | Payload verifier selection, `native_payload_operation` (JSONL to Rust `pse-canonical-recovery`), file-owner paging/reaping, freeze |
| 7857–8297 | 441 | Backup: application inventory pages (SurrealQL over HTTP), metadata image copy, manifest, resume, inspect |
| 8298–9333 | 1,036 | Maintenance listener, `maintenance_sql` (`surreal sql` CLI with REPL-prompt stripping), catalog checks, ROOT credential rotation, `REMOVE DATABASE`, maintenance payload records |
| 9334–10517 | 1,184 | `copy_current_database` (`surreal export`/`import`), `initialize`, `fresh_engine`, `rebuild`, credential recovery, `recover_maintenance`, `restore`, `validate` |
| 10518–10940 | 423 | Interpretation check over HTTP `/sql`; `qualify_recovery` (destructive kill/reopen probe) |
| 10941–11362 | 422 | `mcp-headers` (credential helper for the native MCP endpoint, re-execs itself as a child), `/metrics` snapshot/compare |
| 11363–11748 | 386 | argparse CLI: 30 subcommands |

At least 73 lines still carry retirement paths for the old RocksDB/WebSocket profile v2 (`config_for`
`retiring` branch, `fresh_engine`). AGENTS.md's delete-on-replacement rule would remove them.

#### 2.2 What runs at run time (process chain)

- **Service:** user unit `pse-surreal-<sha16>.service` with `ExecStart=<python> -I -B <state>/.generations/<sha>/scripts/surreal_server.py _serve`
  (`materialize_service`, l.5791). `_serve` (l.7048) re-validates config, binds the host allocation,
  calls `sched_setaffinity` and Popens `surreal start --bind 127.0.0.1:PORT ... mem://<state>/database?aol=sync&sync=every`.
  It then **stays resident as the parent for the server's whole lifetime**, forwarding SIGTERM/SIGINT and pumping
  stdout into `BoundedLog`.
- **Native worker:** `surreal_server.py worker` → `systemd-run --user --scope` → `taskset` →
  `.venv/bin/python scripts/native_operation.py` → `pse-worker` (`worker_scope_command`, l.2951).
  That is two Python interpreter starts per worker launch. Observers and primaries follow the same pattern.
- **Control-plane transports used by Python against the same server:**
  1. HTTP `/sql` with Basic auth (`administrative_results`, l.10648; `validate_interpretation`, l.10518).
  2. The `surreal sql --json --hide-welcome` CLI over gRPC, with REPL prompt text stripped from stdout
     (`establish_protocol_readiness`, l.5110; `maintenance_sql`/`maintenance_results`, l.8514–8566).
  3. `surreal export`/`import` CLI (l.9405).
  4. The `/health` and `/metrics` HTTP endpoints.

  Rust uses a fourth, separate path: the bounded gRPC client in `pse_operations::canonical_client`.
- **Subprocess calls to other tools:** `systemctl` (35 call sites), `systemd-run` (4), `taskset`, Rust
  `pse-deployment` (allocation materialization, via `execution_profiles`), Rust `pse-canonical-recovery`
  (payload inventory/backup/verify/restore/owner observations), `pse-canonical-init`/`xtask canonical-init`
  (schema install), the `surreal` binary (`start`, `sql`, `export`, `import`, `validate`, `module build`), `cargo` and `git` (build-server).

#### 2.3 Lifecycle facts (answering Q1)

- **Process lifecycle:** systemd user units for storage and user scopes for workers/observers/primaries. Readiness
  combines unit ActiveState, cgroup population, PID start identity, socket-inode ownership, `/health` and
  an authenticated gRPC probe. Restart-on-failure is only enabled after `qualify-recovery` (3 starts per 5 min).
  Park/unpark lets resident services yield to host lanes.
- **Config:** `config.json` is an **untyped Python dict** written in `setup` (l.2280–2310), with roughly 30 keys.
  Rust declares only projections of it: `pse_operations::deployment::{ServiceIdentity, CanonicalConnectionSettings}`
  (`projection()` filters fields), `canonical.rs::WriteAdmission`, `canonical_study_module::StudyModuleReceipt`
  and `pse-runtime/src/workflow/native_deployment.rs::ServiceControlProjection`.
- **Auth/credentials:** Python mints `credentials.json` (`username/password/selection_username/selection_password`,
  l.2311). Rust reads it with its own `#[serde(deny_unknown_fields)] struct Credentials` (`canonical.rs:661`).
  ROOT/VIEWER users are defined and rotated through SQL strings in Python (l.5137, 8989).
- **Resource budgets:** the numbers are materialized by Rust `pse-deployment`. Python realizes them as systemd
  properties (MemoryMax, CPUQuota, TasksMax, Slice), `SURREAL_MEMORY_THRESHOLD`, `SURREAL_RUNTIME_WORKER_THREADS`
  and taskset/sched_setaffinity.
- **Health/obs:** `public_status`, `/metrics` snapshot/compare, an MCP audit log reader, and log-config restarts.
- **Upgrades:** `readmit` replaces artifacts on a stopped service. `fresh-engine` destructively retires an old engine.
  Schema adoption happens by regeneration (`rebuild`), not migration (ADR-0214: "Fresh fenced installation regenerates existing data").
- **Backup/restore:** offline only. Python freezes admission, pages 10+ application tables over HTTP SQL,
  copies the metadata image (`backup_image`), hands payload files to Rust `pse-canonical-recovery`, and writes a
  v4 manifest. Restore does export/import into a fresh DB identity, then rotates credentials and source-authority incarnations.
- **Schema:** Python does not install schema. It invokes Rust (`pse-canonical-init` / `xtask canonical-validate-definitions|canonical-activate`)
  under a maintenance nonce (`PSE_CANONICAL_INITIALIZER`) that Rust re-checks against `lifecycle-owner.json`.

#### 2.4 Duplication of Rust-owned knowledge (DUP evidence)

1. **Canonical row shapes in hand-written SurrealQL.** `payload_pages` (l.7430) hard-codes column lists and
   casts for `canonical_file_owners`/`canonical_file_instances`, e.g. `<string>pid`, `group_inode`, `batch_count`.
   Those fields are registry-generated in `crates/pse-operations/src/generated/surreal.surql:182–194`.
   `reap_payload_owners` (l.7540) then rebuilds a SurrealQL object literal and re-adds `<decimal>` for `pid`/`group_inode`
   because "JSON administrative transport does not retain native Decimal kind". This is a type-loss workaround that
   Rust's typed gRPC codec does not need.
   `application_inventory_query` (l.7862) hard-codes 26 reference field names across the backup tables. Only the
   table list is generated (`surreal-policy.json: application_backup_tables`).
2. **Process identity in four places.** Python `process_observation.boot/start_identity` (`/proc/<pid>/stat[19]`);
   Rust `canonical.rs::WriteAdmission::require_initializer` (re-parses `/proc/<pid>/stat` field 19 by hand);
   Rust `canonical_files.rs::ProcessIdentity::current` (l.300–380); Rust `pse_diagnostics::process_facts::ProcessIdentity`
   (procfs-core). These are three independent Rust copies plus the Python one, all of the same boot/pidns/start triple.
3. **systemd unit naming as an undeclared cross-language convention.** Python `worker_unit()` produces
   `pse-surreal-worker-<sha16>-<slot>.scope`. Rust `canonical_files.rs:333–345` matches the string prefixes
   `pse-surreal-worker-`/`pse-native-` and shells out to `systemctl --user show` itself. Python
   `managed_file_owner_drained` (l.7476) runs the inverse check.
4. **Supervisor identity hard-wired into Rust admission.** `pse-runtime/src/workflow/native_deployment.rs:412–445`
   refuses a storage controller unless its file is named exactly `surreal_server.py` under `scripts/` in a
   `.generations/<sha>` directory whose `generation.json` lists that script's SHA256. The Python implementation
   detail is therefore part of the Rust runtime contract.
5. **Credential and receipt documents declared twice:** `credentials.json` (Python dict literal vs Rust `Credentials`).
   `study_module` receipt (`surrealism_module.install` dict vs `StudyModuleReceipt` + `ModuleLimits`; `ABI = 2`
   appears in both). Evidence tooling `config.json`/`credentials.json` (`evidence_store.prepare` vs
   `xtask/src/development/evidence.rs:158–228`, with the same 4-key and distinct-user validation written twice).
6. **Verified-copy protocol:** `backup_image.copy` (O_TMPFILE, `/proc/self/fd` linkat, SHA256 re-read) duplicates
   `canonical_files/offline.rs::copy` (rustix `OFlags::TMPFILE` + `linkat`, l.325, 583–690).
7. **SurrealQL validation:** `surreal_diagnostics.exact_parser` shells to `surreal validate --stdin`, while
   `xtask surrealql-validate` parses in process with `surrealdb-syn`. These overlap but are not identical, since the
   server binary carries the patches.

#### 2.5 Complexity drivers

- **Making a Python process an admitted, immutable service binary.** `publish_generation` (l.735) computes the
  static import closure (`receiver_closure.source_files`). That closure is 29 files and 26,171 lines (*Measured*).
  It copies them plus configuration/patch inputs into `.generations/<sha>`, probe-imports them with the exact
  interpreter, records runtime providers, and re-verifies on every start. A Rust supervisor would need only one
  binary digest, as is already done for `bin/pse-worker` and `bin/pse-deployment` in the same generation.
- **Subprocess and text-protocol plumbing** to the `surreal` CLI (REPL output parsing), `systemctl show` key=value
  parsing, HTTP `/sql` JSON and JSONL pipes to Rust helpers. Each hop needs its own bounds, deadlines and redaction
  (`main()` deliberately hides `SubprocessError` text because argv may contain secrets).
- **Re-deriving Rust state.** Rust writes file-owner rows, Python pages them back over HTTP, pipes them to Rust for
  liveness, then writes a drain acknowledgement through a generated SQL function.
- **Lifecycle edge cases with no single owner of record:** pending launches, borrowed services, parked services,
  maintenance nonces and lost unit-publication ACKs. These live in a dozen JSON state files (`config.json`,
  `lifecycle-owner.json`, `service-launch.json`, `server-process.json`, `protocol-readiness.json`,
  `.maintenance-listener.json`, `fresh-engine-pending.json`, ...). Rust reads some of them directly.
- **Tests:** `test_surreal_server.py` has 12,127 lines and 307 tests with 1,006 `patch.object` calls (646 on
  `server.*`, e.g. `active` ×42, `systemctl` ×26). The unit tests are mostly mock choreography of the
  subprocess/systemd surface.

#### 2.6 Performance-relevant (*Measured*, sandbox Python 3.13, 3 runs)

- Importing the supervisor closure takes about 120 ms with bytecode cache and about 310 ms without (`-I -B` from a
  copied generation, which is how the unit's ExecStart and the frozen generation run), against 16 ms for a bare
  interpreter.
- `mcp-headers` (called per MCP connection as a header helper) runs a parent and a re-exec'd child interpreter of
  this closure.
- Each worker launch pays two interpreter starts.
- These are control-plane costs, not science hot paths. They are still per-invocation latencies on test fixtures and
  every storage (re)start.

---

### 3. Per-group notes (other modules)

#### 3.1 `surreal_source.py` + `scripts/surrealdb-source/` (ORCH)
Pins `VERSION=3.3.4`, `REVISION=1ac2b1db…`, `ENGINE=1.0.0` (SurrealMX) and `FEATURES="storage-mem,cli,mcp,surrealism"`.
It applies 13 patches in order (`PATCHES`, l.34). Two of them, `mx-1.0-minimal`/`mx-1.0-controls` at 937 lines,
move 3.3.4 from SurrealMX 0.27 to 1.0. Four are gRPC session/ownership/status patches, and five touch the
Surrealism runtime. The module verifies a checksum-pinned DiskANN crate archive, runs `cargo build --release` in the
acquired tree and records a receipt. It is pure build orchestration, so an `xtask` subcommand fits better than a
Python script. The patch set is the real asset and stays either way.

#### 3.2 `surrealism_module.py` (ORCH + DUP)
`install()` (l.470) runs `<surreal> module build --out study.surli crates/pse-study-module`. `inspect_archive`
re-reads the tar and checks a hard-coded `PRODUCTION_EXPORTS={"study::readiness"}` plus qualification exports.
Those export names duplicate the `#[surrealism]` functions in `crates/pse-study-module/src/lib.rs:246–303`.
It then writes `module.json`. Rust reads the same receipt (`canonical_study_module.rs:19 StudyModuleReceipt`, `deny_unknown_fields`).
Disposition: one Rust type owns the receipt, and xtask builds and publishes it.

#### 3.3 `surreal_diagnostics.py` / `surreal_telemetry.py` (NEW-OBS)
`surreal_telemetry.Capture`/`Receiver` (l.22–160) is a **hand-written HTTP/2 server**. It writes raw SETTINGS,
PING and WINDOW_UPDATE frames and a literal HPACK header block (`b"\x88\x0f\x10\x10application/grpc"`), then
walks OTLP protobuf fields with `scripts.grpc_wire.fields`. A tonic `TraceServiceServer` from `opentelemetry-proto`
(0.33.1 is already in `Cargo.lock` via the OTLP exporter; enabling `gen-tonic`+`trace` features is assumed and
should be verified) replaces it. `surreal_diagnostics` is a CLI router over `storage_health`, `server.metric_snapshot`,
`server.mcp_audit_records`, `server.restart_log_configuration` and `exact_parser`.

#### 3.4 `storage_health.py` / `build_storage.py` (NEW-OBS)
storage_health builds a `msgspec` `HealthSnapshot` from `/proc/meminfo`, `statvfs`, cgroup files, `systemctl show`,
`/proc/<pid>` and the server's `/metrics` (`MX_METRIC_FIELDS` from surreal_server). Rust `evidence.rs:388`
already accepts `storage-health-v1` as an evidence source kind, so the document type would naturally move to Rust.
build_storage is a disk-usage survey that still contains legacy cohorts (`plan10`, `plan11`, ...). It is low value
and could stay in Python or become a small xtask command.

#### 3.5 `backup_image.py` (NEW-STORE, DUP)
This is the stopped-metadata-image copy used by `surreal_server._backup`/`resume_backup` (l.8151–8250). It
duplicates the Rust offline copy protocol, so it should be deleted once backup moves to Rust.

#### 3.6 SurrealKit beta cluster (ORCH + DUP), 3,199 LOC + 4,788 test LOC + 24,323 data lines
- `surrealkit_source.build` clones `surrealdb/surrealkit@92ebc3cc` (`.config/surrealkit.toml`). It rewrites the
  upstream `Cargo.toml` by string replacement to point at the workspace's SDK pin
  (`Cargo.toml [workspace.metadata.pse.tool-dependencies] surrealdb`) and runs `cargo build`.
- `surrealkit.py` provisions per-test namespaces (registered in the Python resource registry), stages schema and
  runs the CLI. `surrealkit_maintenance.py` runs rollout `up/start/complete/repair` in watched child processes,
  then calls `xtask canonical-validate-definitions|canonical-activate` (`xtask/src/main.rs:126–137`, implemented in
  `pse_operations::canonical_maintenance`) for settlement.
- **DUP:** `surreal_rollout_producer.beta6_step_checksum` (l.58) reimplements the upstream Rust `rollout.rs::step_checksum`
  field order in Python. `surrealkit_population.rows`/`DECIMALS` re-declares canonical row shapes for 9
  `canonical_*` tables that `pse-model` already generates as typed rows.
- **Stale copy:** `database/canonical-schema/schema/canonical.surql` (2,737 lines, rollout 2026-10-21) **differs from**
  the current generated `crates/pse-operations/src/generated/surreal.surql` (4,509 lines; first diff at line 58).
  The repo therefore holds a second, outdated schema copy plus 6,157-line catalog snapshots twice.
- `docs/dev/surreal-substrate.md#standalone-beta-tooling-fixtures` describes this as a "disposable beta tooling" parity journey.
  The production path is fresh regeneration. Rust still carries a `surrealkit_rollout_pending` branch in
  `canonical.rs::WriteAdmission`, which would go with it. Disposition: DELETE, unless rollouts are promoted to production.

#### 3.7 `evidence_store.py` (WRAP/ORCH + DUP)
All database work is Rust. `pse-evidence` (`xtask/src/bin/pse-evidence.rs` → `development/evidence.rs`, 1,210 LOC)
defines users and DDL (`evidence.rs:1153–1154`, embedding generated `store::evidence::SCHEMA` and
`store::knowledge::SCHEMA`), imports, queries and pages. Python adds four things:
1. `build` runs cargo through `arrow_validation` and publishes the binary via `artifact_publication`.
2. `prepare` mints reader/importer passwords and `config.json` under `STATE/.tooling/evidence`. Rust re-validates
   the same document.
3. `run_native` acquires `surreal_server.tooling_activity` and passes the lease fd to the child.
4. It builds snapshot JSON through `evidence_projection`/`evidence_sources` (outside this slice). Their hand-written
   `EvidenceSnapshot(TypedDict)` mirrors Rust `evidence.rs:333 EvidenceSnapshot`, even though a generated
   `scripts/generated/evidence_snapshot_input.py` exists and is not imported there.

Every query therefore costs one Python interpreter plus one Rust process.

#### 3.8 `knowledge_store.py` (ORCH + DUP)
`corpus()` (l.33–234) builds a version-2 page (sources/excerpts/subjects/edges/productions) from
`workspace_domains`. It enforces bounds `256/256/1024/2048/1024`, which are the same literals as
`xtask/src/development/knowledge.rs:323–330`. It then execs `pse-evidence knowledge-import|query|explain`.
Porting it depends on porting `workspace_domains` (1,137 LOC, other slice).

#### 3.9 `assessment_store.py` (NEW-STORE, Python-only truth)
Python is the only owner of qualification-report roots. It writes `checks.json` via fsync + replace and
content-addressed `details/<sha>.json`, with `VERSION=6` and a v5 "original export". It uses `test_resources`
borrow/read for custody. No Rust type exists, and Rust only sees the evidence-snapshot projection. It is coupled to
`validation.py`, `validation_receipts.py`, `test_resources.py` and `case_measure.py`, so it should move when test
orchestration moves.

#### 3.10 `resource_store.py` / `service_borrowers.py` / `resource_conversion.py` (NEW-STORE + NEW-CONTROL)
`FileStore` is a JSON-file registry with flock locks (`locks/<family>/<key>`), a lock order of invocation, identity,
then resource, access markers and an activation fence (`version 3`, `exact-resource-files`). `service_borrowers`
projects "who borrows this storage service" for the supervisor's stop/backup drain checks.
**The direction is inverted:** Rust fixture code calls Python to register, drain and query fixtures
(`crates/pse-operations/src/testing.rs:321–386`). It spawns `.venv/bin/python -m scripts.test_resources <action>`
with JSON on stdin under a 50–60 s budget **for each fixture registration/drain**, so Rust tests depend on a Python
interpreter and this Python-owned authority.
`resource_conversion.py` is a one-time v2→v3 registry migration that `FileStore.ensure` still fences against.
Per AGENTS.md it should be deleted once complete.

#### 3.11 `artifact_publication.py` (NEW-STORE)
A pydantic `Publication` descriptor plus an immutable binary copy for `extension` and `evidence-executable`. It is
one of **five** implementations of "publish an immutable generation and receipt" in this slice alone:
`surreal_server.publish_generation`, `surreal_source.install`, `surrealism_module.install`,
`surrealkit_source.publish` and this module.

---

### 4. Tests (LOC)

| Cluster | Files | LOC |
|---|---|---|
| Server/source/diagnostics/backup/health | `test_surreal_server` (12,127), `test_surreal_mcp` (1,015), `surreal_fixture_check` (918) + test (369), `surreal_mx_check` (856), `test_storage_health` (719), `test_surreal_source` (561), `test_surrealism_module` (390), `test_backup_resume` (378), `test_surreal_diagnostics` (334), `surreal_scan_budget_probe` (286) + test (136), `surreal_logging_check` (228), `surreal_otlp_check` (147), `test_backup_image` (126), `test_plan30_storage_placement` (123), `surreal_install_equivalence_check` (109), `test_build_storage` (101) | 18,923 |
| SurrealKit beta | `test_surrealkit_maintenance` (1,492), `surrealkit_rollout_fixture_check` (826), `surrealkit_persistent_fixture_check` (736), `test_surrealkit_population` (473), `test_surrealkit` (402), `surrealkit_fixture_check` (330), `test_surreal_rollout_producer` (171), `test_surrealkit_source` (134), `surreal_rollout_producer_fixture_check` (119), `test_surrealkit_persistent_fixture` (105) | 4,788 |
| Stores | `test_resource_store` (738), `test_resource_conversion` (523), `plan30_resources_actual_check` (519), `test_assessment_store` (408), `test_knowledge_store` (340), `test_evidence_store` (309), `test_assessment_boundary` (296), `test_artifact_publication` (213), `test_capability_knowledge` (163), `assessment_fixture` (29) | 3,538 |
| **Total in slice** | 38 files | **27,249** |

Not counted: `test_test_resources.py` (3,201; owner `test_resources.py`, outside this slice, heavy user of
`resource_store`/`service_borrowers`/`surreal_server`), `evidence_native.py` (387) and the evidence
projection/source tests.

Rust consumers outside this slice that drive `surreal_server` from Rust tests: `crates/pse-runtime/tests/worker.rs`
runs `python -c "from scripts import surreal_server as s; s.workers_drained(...)"` and `s.systemctl('kill',...)`
at l.86, 123, 136, 889 and 1625.

---

### 5. Answers to the key questions

#### Q1. What does `surreal_server.py` do, and what duplicates Rust?
See §2.2–2.4. In short, it is a complete **local database operator**: systemd unit/scope supervision with
cgroup/CPU placement, readiness, admission gates, credentials, backup/restore/rebuild/maintenance and recovery
qualification. It also has three ad-hoc DB transports (HTTP `/sql`, `surreal sql` REPL, export/import).
It duplicates Rust in five areas:
1. Canonical table columns and types in hand-written SQL.
2. Process-identity parsing.
3. The verified-copy protocol.
4. Document shapes that Rust deserializes strictly (`credentials.json`, study-module receipt, tooling config).
5. The systemd unit naming convention, which Rust string-matches.

Schema, codecs, payload file I/O, allocation materialization and the authenticated gRPC client are already Rust.
Python reaches them through subprocess + JSON(L).

#### Q2. Could Rust own this directly? Most targeted route

**Option A: embed SurrealDB in process (`surrealdb` crate, `kv-mem`/`kv-surrealkv`/`kv-rocksdb`).**
Not a fit for the canonical store.
- ADR-0214 explicitly rejected it: "Per-process embedded MX creates competing lifecycle authority". Its drivers
  require one supervised authority shared by independent processes: the Python `Runtime` via pse-py, 16 native
  worker lanes, nextest processes, `pse-evidence`, and the server's MCP endpoint. Embedded engines are single-process owners
  of their files.
- The production engine is SurrealDB 3.3.4 patched to SurrealMX 1.0 (`mx-1.0-*.patch`). The SDK's embedded `kv-mem`
  would need the same patches applied to a git-forked crate (`[patch]`). The library doc
  `docs/libraries/SurrealMX.md §10.2` notes that stock 3.3.4 depends on surrealmx 0.27.
- About half the client stack (`canonical_client/*`, `canonical_transport.rs`, `transport_lifetime.rs`; over 5K LOC)
  encodes gRPC connection-bound session and physical-drain semantics that ADR-0214 requires. Embedding would replace
  that transport, not just the supervisor.

**Where embedding *is* targeted:**
- **The disposable evidence/knowledge index.** `pse-evidence` could open its own `surrealdb` `kv-surrealkv` (or `kv-mem` + export)
  database file. That deletes the tooling namespace, the reader/importer credential minting
  (`evidence_store.prepare`), the `tooling_activity` lease and the dependency on a running canonical service.
  This requires that only one process writes the index at a time. The MCP capability server reading it
  concurrently would need checking. Uncertain.
- **Possibly per-test disposable fixture databases.** Today these are namespaces/databases on the shared server,
  registered in the Python resource registry.

**Option B (recommended): a Rust supervisor that owns the external server's lifecycle.**
Keep the patched `surreal` binary as a separate process; this preserves ADR-0214. Replace `surreal_server.py`
(and the `host_admission`/`native_operation` pieces it needs) with a Rust binary, for example `pse-store`/`pse-supervisor`
in `pse-operations` or `pse-runtime` next to `pse-deployment`. That binary is both the CLI and the unit's ExecStart.
Library route, mostly already in the tree:

| Concern | Today (Python) | Rust route |
|---|---|---|
| systemd units/scopes | 35 `systemctl` + 4 `systemd-run` subprocesses, key=value parsing | `zbus` + `zbus_systemd` Manager proxy (`StartTransientUnit` for scopes, `StartUnit`/`StopUnit`/`ResetFailedUnit`, `ActiveState`/`ControlGroup`/`InvocationID` properties). Shelling out to `systemctl` via `tokio::process` is an acceptable first step. Use `sd-notify` with `Type=notify` instead of polling readiness. |
| pidfd, signals, flock, affinity, statvfs | `os.pidfd_open`, `fcntl.flock`, `taskset`, `os.sched_setaffinity` | `rustix` (in tree: `process::pidfd_open/pidfd_send_signal`, `fs::flock`, `fs::statvfs`, `sched_setaffinity` — module path varies by rustix version; `nix::sched` is an alternative, `nix` 0.31 already in the lock). Existing `pse_diagnostics::supervised_child` (pidfd-verified leader). |
| /proc and cgroup facts | `process_observation`, ad-hoc reads | `procfs-core` (in tree) via `pse_diagnostics::process_facts`; cgroup v2 files via `std::fs` |
| Admin SQL, readiness, user/DB DDL, inventories, file-owner drain | HTTP `/sql`, `surreal sql` REPL scraping, `<decimal>` re-casts | Existing `pse_operations` gRPC client (`transport::client` with Root/Database auth; `CanonicalStore`). Typed values remove the Decimal workaround. |
| Payload inventory/backup/verify/restore | JSONL pipe to `pse-canonical-recovery` | Direct in-process call to `canonical_files::offline` |
| Metadata image copy | `backup_image.py` | Reuse `canonical_files::offline::copy` |
| Config/credentials/receipts | Untyped dicts plus Rust projections | One `serde` + `schemars` `ServiceConfiguration` owned in `pse_operations::deployment`; the existing generated `scripts/generated/deployment.py` continues for any remaining Python reader |
| Frozen supervisor generation | Python import-closure probe (29 files, 26K lines) | Copy one binary and record its digest (as for `bin/pse-worker`); delete the `native_deployment.rs` `surreal_server.py` name check |
| Server logs | `BoundedLog` thread + redaction | A small rotating writer task (or the `file-rotate` crate) |
| OTLP capture | Hand-rolled HTTP/2 | `tonic` + `opentelemetry-proto` `TraceServiceServer` |
| Export/import | `surreal export/import` CLI | Keep the CLI via `tokio::process`, or the SDK's `export()`/`import()` on a remote gRPC connection (`surrealdb` `protocol-grpc`, already declared as a tool dependency) |

A further variant is a pse-owned server binary that links the patched upstream server code as a library and adds
the supervisor in process. It is not recommended: it is uncertain whether 3.3.4 exposes its server as a reusable
crate, and it enlarges the patch surface.

The authoritative design currently assigns "Python's environment/systemd realization" to Python
(identity-and-publication §20.6, ADR-0229) and freezes Python import closures for receiver generations.
Option B therefore needs an ADR amendment.

#### Q3. Are the stores separate sources of truth?
- **Evidence/knowledge DBs:** schema and all I/O are Rust (`xtask/src/development/{evidence,knowledge}.rs`, generated
  `store/*.surql`). Python is not a second truth for data. It **is** a second declaration of the tooling
  config/credentials document, of the snapshot type (`evidence_projection.EvidenceSnapshot`) and of the knowledge bounds.
- **Assessment store:** a **Python-only** source of truth (report roots and details). Rust has no type for it.
- **Resource registry (`resource_store` FileStore):** a **Python-only** authority that Rust *depends on*. Rust fixtures
  register and drain through `python -m scripts.test_resources`, and the storage supervisor consults it for borrower
  obligations before stop/backup.
- **Canonical store:** Rust-owned schema. Python reads and writes selected tables directly (file owners/instances,
  interpretations, ROOT users, databases, backup inventories) through SQL strings, outside the Rust codecs.
- **Service state files** (`config.json`, `lifecycle-owner.json`, `service-launch.json`, `server-process.json`,
  `protocol-readiness.json`, ...): Python-authored. Rust reads several of them strictly, so there are two
  declarations per document.

---

### 6. Cross-cutting observations

1. **The same pattern appears in every module: Python does Rust-adjacent work and then shells out to a Rust binary
   with JSON.**
   - `pse-deployment` (allocation; `execution_profiles` memoizes the subprocess result in a module-global dict)
   - `pse-canonical-recovery` (payloads, JSONL)
   - `pse-canonical-init` / `xtask canonical-*` (schema)
   - `pse-evidence` (evidence/knowledge)

   The Python side keeps a parallel model of the inputs and outputs and validates them again.
2. **Wrong-direction dependency.** Rust code and tests call Python:
   - `pse-operations/src/testing.rs::resource_bridge` (per fixture)
   - `pse-runtime/tests/worker.rs` (`python -c 'from scripts import surreal_server …'`)
   - `pse-runtime/src/workflow/native_deployment.rs` (requires a file named `surreal_server.py`)

   Rust also re-implements Python's side conventions (unit-name prefixes, `lifecycle-owner.json` checks).
3. **Text protocols where typed ones exist.** Python talks to the same server over HTTP `/sql` (JSON loses Decimal
   kind), the `surreal sql` REPL (prompt stripping), CLI export/import and a hand-rolled HTTP/2 OTLP sink. Rust already
   has a bounded typed gRPC client for the same server.
4. **Duplicated low-level mechanisms:**
   - process identity ×4 (1 Python, 3 Rust)
   - verified O_TMPFILE copy ×2
   - immutable-generation publication ×5
   - credential/receipt documents ×2 declarations each
   - SurrealQL validation ×2

   The Rust side has its own internal duplication too (`canonical_files::ProcessIdentity` vs `pse_diagnostics::process_facts`).
5. **Python as a resident service process.** `_serve` stays parent of the database for its whole lifetime and must
   be frozen as an immutable 26K-line import closure. That drives `receiver_closure`, `receiver_environment` and
   generation verification code, and Rust admission checks against it. A static Rust binary removes this whole category.
6. **Per-invocation interpreter startup on control paths.** It costs about 120 ms with pyc and about 310 ms without
   (*Measured*, sandbox) for each service start, each `mcp-headers` call (×2), each worker launch (×2 interpreters)
   and each evidence/knowledge command (Python + Rust). None of this is on the numerical hot path. It does tax test
   fixtures and agent tooling latency.
7. **Stale and legacy material that is deletable regardless of language:**
   - `database/canonical-schema/` (stale vs generated schema)
   - the SurrealKit beta cluster
   - `resource_conversion.py`
   - profile-v2 RocksDB/WS retirement branches in `surreal_server.py`
   - `build_storage` legacy plan cohorts

   Together that is about 4K Python LOC, about 5K test LOC and 24K data lines.
8. **Test weight mirrors subprocess plumbing.** About 18.9K test LOC for the server cluster, dominated by mock
   choreography (1,006 `patch.object` in one file). A Rust supervisor using typed systemd/D-Bus and gRPC interfaces
   could test against traits/fakes at a much smaller surface, and the existing qualification controls
   (`surreal_mx_check`, `surreal_fixture_check`) would carry over as integration checks.
9. **Porting order (smallest blast radius first):**
   1. Delete the beta/legacy material in item 7.
   2. Move file-owner reaping, backup inventory and readiness to the Rust gRPC client inside `pse-canonical-recovery`,
      which deletes the HTTP `/sql` and REPL paths.
   3. Fold `backup_image`, `surrealism_module`, `surreal_source` and `artifact_publication` into xtask/Rust.
   4. Move the resource registry to Rust so fixtures stop calling Python.
   5. Port the supervisor (systemd/cgroup/placement) together with the `host_admission`/`native_operation`
      slices, with an ADR amending §20.6/ADR-0229.

---

## C — Host control, admission, native setup, build and cache cluster

Slice: `scripts/` environment, host-control, admission, native-setup, build and cache modules
(≈19.5K lines of Python and shell, plus `.config/agent-capacity.toml`). The snapshot is at
HEAD bd1c4891e (`/home/claude/pse`). I read the code paths named below. Runtime numbers were
measured in this container: Python 3.13, warm `.pyc` held in a scratch pycache prefix, no systemd
user manager and no `.envrc.local`. Any number not measured here is labelled as an estimate.

### 0. Headline

1. **`host_admission.py` does not duplicate `crates/pse-admission`.** They only share a name.
   `pse-admission` is "Owned document intake and native physical source admission"
   (`authoring_driver/`, `physical.rs`). It contains no cgroup, systemd, slice or lane concept
   (grep finds none). The Rust counterparts of host-control concepts sit in other crates and
   form real DUPs (§3.1):
   - `pse-diagnostics::{process_facts, SupervisedChild}`
   - `pse-runtime/src/bin/pse-worker.rs::verify_placement_hierarchy`
   - `pse-operations/src/canonical_files.rs` (scope-owner identity through `systemctl show`)
   - `pse-runtime::workflow::native_deployment(_protocol)` (production profiles)
2. **The hottest Python in the repo is the compiler wrapper, not `pse_env.py`.**
   `build_environment.configure` sets `RUSTC_WRAPPER=scripts/sccache`. That script is
   `exec .venv/bin/python scripts/compiler_cache.py`. `native_cache.compiler_env` also makes it
   the CMake C/C++ launcher (`.config/native-cache.cmake`). So **every rustc unit and every C/C++
   object in KLU, ibex, Uno and PETSc starts a CPython interpreter.** Each start pays:
   - about 80 ms of imports (measured);
   - a SHA-256 of the whole sccache binary (`ensure → digest(binary)`; about 90 ms per 30 MB
     measured here);
   - about 4 `systemctl --user show` subprocesses (`ensure→ready`, then `same()` ×3);
   - about 4 exclusive flocks that also rewrite the host ledger (`registered → host.metadata`);
   - at least 50 ms of unconditional sleep (`finish_group`), plus up to 50 ms of poll latency
     (`monitor.select(0.05)`).

   The lockfile has 913 packages, so this cost recurs about 1,000 times on a clean build. On
   sccache hits it probably dominates the actual compile time (estimate).
3. **Per recipe, `pse-env` costs two interpreters (three for `--native`) and about 12 or more
   systemd round trips** (static trace in §2). There is a parent `pse_env.py`, then `systemd-run --scope`, then a second
   `pse_env.py --bind-allocation`, then `exec`. One `pse_env.py --print` run measured
   **~177 ms against 16 ms for a bare interpreter**. Import alone is about 112 ms across 24
   `scripts.*` modules, including the 7.3K-line `surreal_server` and an `urllib.request`/
   `http.client` chain pulled in through `diagnostic_trace`. A cold start with no `.pyc`
   measured 0.83 s.
4. **Most of the 1.6K-line ledger is inert on the maintainer's machine.**
   `PSE_BUILD_MODE=unbounded` maps compile, functional, wide and exclusive to `build`
   (`pse_env.selected_resource_class`), and `conflict()` returns `None` for unbounded profiles.
   So for ordinary work the ledger, reconcile, drain and slice machinery only supplies:
   - a scope under `psebuild.slice`;
   - exclusion against timing and reference work;
   - ownership and liveness bookkeeping.
5. **Most of the cluster re-derives facts that systemd, the kernel, Cargo, nextest and Rust std
   already hold.**
   - Cgroup paths are rebuilt from slice-naming rules instead of read from the `ControlGroup`
     property.
   - CPU width is recomputed from `cpu.max` and affinity, which `available_parallelism` already
     honours.
   - `NEXTEST_TEST_THREADS` duplicates `test-threads=16` in `.config/nextest.toml`.
   - Build phases are inferred from `/proc` sampling.
   - Cargo configuration discovery is re-implemented.

   The genuinely new control is narrow:
   - cooperative cross-session lane admission, queueing and exclusion;
   - memory-pressure gating at admission;
   - a compiler-cache daemon placed outside the build cgroup;
   - drain-before-release accounting.

   It needs a typed Rust daemon or library, not 20K lines of Python.

### 1. Summary table

LOC is `wc -l`. Class is listed as primary/secondary.

| Module/group | LOC | What it does | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---|---|---|---|---|---|---|
| **Env boundary:** `pse_env.py`, `pse-env`, `build_environment.py`, `systemd_environment.py`, `native-{execution,math,pipeline,solver}-env.sh`, `setup-env` | 1424+11+448+92+56+21 = **2,052** | Composes the env (caller, then `.envrc.local` through a bash subshell, then defaults, then the compiler/sccache/LLVM/target-dir rules). Picks a resource class, calls host admission and wraps the command in `systemd-run --scope`. A second interpreter then binds the allocation. Also emits `--print`/`--explain`. | ORCH / NEW-CONTROL | Cargo already owns jobs (`available_parallelism` honours cgroup quota and affinity) and target-dir. `.config/nextest.toml` owns `test-threads=16`. `pse-worker.rs::verify_placement_hierarchy` already reads the cgroup ancestry. | **PORT-LIB**: a tiny standalone Rust `pse-env` binary, not inside heavy xtask, so it starts in a few ms. Delete `cpu_widths`/`admitted_cargo_jobs` and let Cargo/nextest derive width. | zbus + systemd1 proxies (`zbus_systemd`, *crate name/API to verify*) for `StartTransientUnit` with `PIDs=[self]` then `exec`. rustix (present) for flock, affinity and pidfd. toml/serde (present). | M–L (bootstrap chicken-and-egg, see §4) |
| **Host admission:** `host_admission.py`, `host_metadata.py`, host section of `.config/agent-capacity.toml` | 1620+109+164 = **1,893** | Flock-guarded JSON ledger (`~/.local/state/pse-arrow/host-admission/allocations.json`) of owners with lanes, slots, cores and memory. Per-request `reconcile` calls `drained()`, which runs `systemctl show` per registered unit and reads `cgroup.events`. Creates `pse-allocation-<nonce>.slice` and `set-property`s MemoryMax/CPUQuota/AllowedCPUs. Has a MemAvailable pressure guard, parks resident services and writes admission checkpoints. | NEW-CONTROL / NEW-STORE | Nothing equivalent; `pse-admission` is unrelated. Production profiles are Rust (`native_deployment_protocol.rs::ProductionProfiles`). The `reference` class calls the Rust `pse-deployment` binary through `execution_profiles`. | **PORT-LIB**: Rust host-admission crate, ideally a long-lived user service that holds owners in memory and watches pidfds and `cgroup.events` instead of polling `systemctl show`. | zbus (unit properties and Start/StopUnit), rustix (pidfd_open, flock), procfs-core (present), `inotify` crate or rustix inotify for `cgroup.events` (*verify rustix API*), tokio (present), serde/toml/schemars (present). | L |
| `execution_profiles.py` | **241** | Spawns the Rust `pse-deployment` binary per profile request with JSON over stdin/stdout, validates with msgspec against `scripts/generated/deployment`, and keeps an in-process cache. | WRAP | `pse-runtime` `pse-deployment`, `ProductionProfiles::{read,materialize}` | **DELETE** once admission is Rust: call the library directly | — | S |
| `host_contracts.py` | **13** | Pydantic base for the generated `scripts/generated/native_operations.py` host-operation records | GEN-support | `pse-runtime::host_operations::catalogue()` → `xtask/src/codegen/host_operations.rs` | DELETE alongside the Python capability layer, or KEEP-PY as part of the thin shell | — | S |
| **Native operation lifetime:** `native_operation.py`, `operation_lifetime.py`, `process_observation.py` | 2024+426+204 = **2,654** | Operation records under `~/.cache/.../.operations/*.json` (pins, admissions, observation memo) and foreground handoffs between interpreters. Signal forwarding, cancellation clocks, `killpg`. Composer/terminal-guard acknowledgement protocol. Drain proof via `systemctl show` plus `cgroup.events`. Supplies native env (IPOPT/SCIP/KLU/Uno/PETSc paths, OMP/MKL thread defaults). | NEW-CONTROL / ORCH | `pse-diagnostics::SupervisedChild` (pidfd-verified process group, cancellation eventfd, drain policy) and `process_facts` (procfs-core boot_id, start ticks, pgrp, pid namespace). `pse-operations/canonical_files.rs` does the same scope-owner identity (unit regex, InvocationID, ControlGroup, inode). | **PORT-EXISTING + PORT-LIB**: one Rust supervisor on `SupervisedChild` plus systemd `KillUnit`/`StopUnit` for the scope. Most of the handoff/composer protocol exists only to coordinate several Python processes and becomes unnecessary. | rustix (present), procfs-core (present), zbus, tokio::signal (present in xtask) or signal-hook | L |
| `invocation_disposition.py`, `recovery_operation.py` | 386+145 = **531** | Inspection-first abandonment of a test invocation's resources (via `run_inspection` and `test_resources`). A watchdog that polls a Rust recovery binary's checkpoint file for stall. | ORCH / NEW-CONTROL | The recovery binary writes `deployment.RecoveryCheckpoint` (Rust type, generated to Python). | recovery: **DELETE** by putting the stall watchdog inside the Rust recovery operation. disposition: **PORT-EXISTING** together with the test-resource registry. | — | S / M |
| **Compiler cache:** `compiler_cache.py`, `scripts/sccache` | 716+4 = **720** | RUSTC_WRAPPER and CMake launcher. Each call: resolve and SHA-256 the sccache binary and config, find or start a dedicated `pse-compiler-cache-<nonce>.service` (unit file written to `~/.config/systemd/user`, `daemon-reload`, light host allocation), verify pid, exe, peer cred and ledger registration, spawn the sccache client in a new session, watch the service pidfd, `killpg`, then `sleep(0.05)`. | NEW-CONTROL (on a hot path) | Nothing. sccache itself already offers `SCCACHE_SERVER_UDS`, idle timeout and `SCCACHE_NO_DAEMON`. | **PORT-LIB / DELETE most**: point `RUSTC_WRAPPER` straight at sccache, with the server as a static or templated user service (Slice=, MemoryMax= in the unit). Do the identity checks once at service start. If per-call peer verification must stay, use a Rust shim of about 100 lines (peer cred, then `exec`). | rustix (`getsockopt` peercred, pidfd). zbus for start. *Do not* embed sccache as a library: it has a lib target but no stable API (uncertain). | S–M, **highest payoff** |
| **Native provider preparation:** `native_cache.py`, `native_pipeline_cache.py` | 863+406 = **1,269** | Identity-keyed immutable generations under `~/.cache/.../native`: KLU and ibex via CMake, solver prefix extracted with `docker run … tar`, Uno/PETSc/HiGHS pipeline builds, curl downloads with checksum. Writes `.complete.json` receipts that `pse-backend-native/build.rs` consumes as opaque bytes. | ORCH / NEW-STORE | `pse-backend-native/{build.rs,build_receipt.rs}` reads the receipts. `*-sys` crates (`pse-ipopt-sys`, `pse-petsc-sys`, `pse-uno-sys`, vendored `suitesparse-sys`) own linking. | **PORT-EXISTING** to an `xtask native prepare` subcommand, or move the KLU build into the already-vendored `vendor/suitesparse-sys` build script so Cargo owns freshness | cmake (in lock), flate2/zip (in lock), add `tar`; reqwest (present), sha2/blake3 (present) | M |
| **Provisioning:** `supplier_source.py`, `solver-images.py`, `llvm_system.py` | 223+122+530 = **875** | Refresh vendored supplier deltas and receipts, which `pse-backend-native/build-source.rs` verifies. Single authority over `.github/setup/solver-images.json`. Stage a relocatable LLVM into `/opt` with patchelf and `/etc/environment.d`. | NEW-DEVTOOL / ORCH | `build-source.rs` verifies supplier receipts. `xtask/src/codegen/ipopt.rs` reads `solver-images.json` directly. | **PORT-EXISTING** (xtask) for supplier_source and solver-images. llvm_system is a rare host-provisioning action, so low priority; could stay a script. | serde_json, sha2 (present), flate2 + tar | S–M |
| **Observation:** `activity.py`, `activity_watch.py`, `pressure_observation.py`, `build_observation.py` | 522+261+383+265 = **1,431** | `systemctl list-units pse-* --output=json` plus per-unit show plus `/proc` scan for cargo/rustc. Detached long-poll of capability snapshots. PSI and cgroup counter sampling into OTel metrics. `/proc` descendant sampling that classifies rustc, linker and build-script phases. | NEW-OBS | Rust has OTel and tracing (present). Cargo already reports unit timings (`--timings`). | **PORT-LIB** (activity, pressure). build_observation: **DELETE**, replaced by Cargo timings (JSON timings are nightly/unstable, *verify flag*). activity_watch moves with the capability/MCP layer (rmcp). | zbus (`ListUnitsByPatterns`), procfs or procfs-core, opentelemetry (present) | M |
| **Measurement and probes:** `build_measurements.py`, `build_cache_probe.py`, `cargo_freshness_probe.py`, `tooling_experiments.py` | 1616+140+296+614 = **2,666** | Git-snapshot the worktree and run pinned Cargo builds, measuring RSS, sccache stats and phases. Cache pass-through probe. Standalone Cargo freshness fixture. bwrap-confined tool probes. | NEW-OBS / NEW-DEVTOOL | — | `cargo_freshness_probe`: **DELETE** (no caller; referenced only by plan 54b). The rest is **PORT-EXISTING** (xtask) or reduced to hyperfine plus `cargo --timings` | hyperfine (external CLI), tempfile (present) | M |
| **Cargo adapters and policy:** `cargo_feature_check.py`, `cargo_tools.py`, `native-engine-boundaries.py`, `scripts/cargo-bin/cargo-pse-correctness`, `workspace.rs` | 232+65+142+12+62 = **513** | cargo-hack `CARGO_HACK_CARGO_SRC` shim. Pinned cargo-tool spec listing for bootstrap. N06 dependency/deletion check over `cargo metadata` from stdin. `cargo c`/`cargo t` external subcommand, a 12-line Python shim into `arrow_validation`. Checkout-root discovery (already Rust, `#[path]`-included by xtask and governance). | ORCH / DUP | `xtask/src/dependency_ceilings.rs` evaluates `[workspace.metadata.pse.dependency-ceilings]` over cargo_metadata. `pse-buildinfo::workspace_root` is a second root finder. | native-engine-boundaries: **DELETE** (express it as dependency-ceiling declarations). cargo-feature-check, cargo-tools, cargo-pse-correctness: **PORT-EXISTING** (small xtask bins). workspace.rs: move into xtask and reconcile with buildinfo. | cargo_metadata (present), clap (present) | S |
| **Producer capture:** `producer_cargo.py`, `producer-cargo`, `producer_deployment.py`, `producer_receipts.py`, `installation_context.py`, `installed_artifact.py` | 346+4+729+101+615+95 = **1,890** | Cargo shim between maturin and cargo during strict capture; calls `target/debug/xtask producer-identity`. Capture and validation of runtime, worker and Python producer receipts. SHA-256 file observations. Closed Cargo-config, toolchain and environment context for the custom server installer. | ORCH / DUP | `xtask/src/producer_identity.rs` (7.5K lines, includes `cargo_configuration_files`, the build-env allowlist and `is_cargo_env`), `producer_preparation.rs`, `producer_history.rs`. `pse-buildinfo::identity` (FileObservation, DeploymentAssociation, `verify_receipt_artifact`). | **PORT-EXISTING** into xtask producer modules. Delete the Python Cargo-config walker and the env-var lists. | (present) cargo_metadata, sha2, serde_json | M |
| **Receiver:** `receiver_closure.py`, `receiver_environment.py` | 265+234 = **499** | Derives and probes the immutable receiver's own Python import closure (AST walk plus an isolated-interpreter import probe). Captures and restores the prepared native env for a published receiver (used by `surreal_server`). | NEW-STORE / ORCH | Receiver env types are generated (`scripts.generated.deployment`). | closure: **DELETE** once the receiver/worker is Rust-only. environment: **PORT-EXISTING** with the surreal supervisor. | — | S (follows the surreal_server port) |
| `python_extension.py` | **1,182** | Maturin, cargo and uv orchestration of the pse-py extension. Stage receipts and currency checks. Stub generation via `cargo run -p xtask … python-stubs`. Import admission of the built extension. | ORCH / NEW-STORE | `xtask` python_stubs codegen and `pyo3-introspection` (present). Maturin is itself a Rust CLI. | **PORT-EXISTING** (xtask). Keep a tiny Python import probe (KEEP-PY for the "does it import" check only). | — | M |
| **Readiness:** `doctor.py`, `preflight.py`, `bootstrap.sh` | 745+211+129 = **1,085** | Stdlib-only readiness report (uv, venv sync, tools, sccache, solvers, extension, store). Cheap prerequisite checks before long commands. Installer for uv, cargo-binstall tools and sccache. | NEW-DEVTOOL | — | doctor and preflight: **PORT-EXISTING** (`xtask doctor`; a binary has no venv dependency). bootstrap.sh: KEEP as shell, the one unavoidable pre-Rust entry. | — | S–M |

Cluster total is about 19.5K lines.

**Matching tests: about 26.3K lines, more than the code they test.**
- About 20.8K lines in directly named files. The largest are:
  - `test_native_operation.py` 4,882 (83 tests)
  - `test_host_admission.py` 3,151 (66 tests)
  - `test_build_measurements.py` 1,956
  - `test_pse_env.py` 1,678 (53 tests)
  - `test_producer_deployment.py` 1,128
  - `test_setup.py` 1,012
  - `test_python_extension.py` 883
  - `test_build_environment.py` 633
  - `test_compiler_cache.py` 503
- About 5.5K lines in placement and lifecycle tests: `test_plan30_*` and `plan30_*_check.py` (about 4.3K), `test_build_mode.py` 398, `test_highs_provider_candidate.py` 258, `test_efficiency_native_pins.py` 114.
- No tests exist for `native_cache.py` as a unit, `cargo_freshness_probe.py`, `supplier_source.py` or `native-engine-boundaries.py`.

### 2. Per-invocation cost (question 2)

#### 2.1 Measured

| Measurement (this container, warm `.pyc`) | Result |
|---|---|
| `python3 -c pass` ×10 | 0.163 s, so **~16 ms** each |
| `python3 scripts/pse_env.py --print` ×10 | 1.77 s, so **~177 ms** each (no systemd, no `.envrc.local`) |
| `-X importtime import scripts.pse_env` | **~112 ms** cumulative import. 24 `scripts.*` modules. Largest: `build_environment→diagnostic_trace→urllib.request/http.client` 32 ms, `host_admission` (with `execution_profiles→msgspec`) 29 ms, `surreal_server` 15 ms, `tomllib` 6 ms, plus `dataclasses`/`inspect`/`tarfile`/`zipfile` |
| Same import with no `.pyc` (first run after an edit) | **~825 ms** |
| cProfile of `--print` | `compose()` is called 4×. `telemetry_launch.configure` 4× costs about 29 ms, mostly msgspec `inspect` type introspection |
| `import scripts.compiler_cache` | ~80 ms cumulative; ×10 processes 1.13 s, so **~110 ms per wrapper start** |
| `hashlib.file_digest(30 MB, sha256)` | **~91 ms** (the wrapper does this for the sccache binary on every call; sccache 0.19 musl size not checked here) |

`pse_env.py` imports `surreal_server` at module top (line 51) and `telemetry_launch` in
`base_environment`. A plain `just check-package` therefore loads the canonical-store supervisor and
OTel configuration code even when `--store` is not requested.

#### 2.2 One ordinary workload recipe: static process trace

Example: `just unit-package X …` in unbounded mode (`pse_env.main`).

**Parent `pse_env.py`:**
1. Interpreter #1 starts.
2. `local_keys` runs `bash -c '. .envrc.local; env -0'` if the file exists. `--print` reads it again for every compose.
3. `manager_available` runs `systemctl --user show-environment`.
4. `host.acquire` → `reconcile` → `drained()` runs `systemctl --user show` **for every registered unit of every live owner**. This is O(all units on the machine).
5. `enforce_parent` runs `systemctl --user start psebuild.slice` and reads `memory.max` back up the ancestry.
6. `enforce_allocation` runs `systemctl --user start pse(build)-allocation-<nonce>.slice` and `set-property`, then reads back.
7. `systemd-run --user --scope …` launches the child.

**Child, same interpreter binary:**
8. Interpreter #2 starts and composes the env again.
9. `allocation.bind` runs `systemctl show`.
10. `host.inherit` runs three more times (`require_enclosing_owner`, the `inherit` test, `placement`), each with `systemctl show` and an exclusive ledger lock plus rewrite.
11. `sched_setaffinity`, then `execvpe`.

**After exit, parent again:**
12. `release` → `drained` runs `systemctl show` per unit.
13. `retire_empty_allocation` runs `systemctl show` on the slice and `systemctl stop` on the slice.

**Totals per recipe:**
- 2 Python interpreters (about 0.35 s measured-equivalent);
- about 12 + k `systemctl`/`systemd-run` processes, where k is the number of registered units machine-wide (each about 5–20 ms, *estimate, not measurable here*);
- about 10 flock-and-rewrite cycles of `allocations.json`.

`metadata()` rewrites the ledger on exit even for reads (`host_admission.metadata` → `operation.write_json`).
The fixed overhead is plausibly **0.5–1 s per recipe** (estimate).

Native recipes (`--native`) use **three** interpreters:
- `native()` → `placement(native=True)` → `owned_scope`. The parent also opens an
  `operation.Operation` observer and `prepare_handoff`.
- In the scope, `pse_env.py --bind-allocation … --` execs a third `pse_env.py --no-scope --native=…`.
  That third process stays resident as the `native_operation.run` supervisor for the whole workload.
- Nested native workloads add `native_operation.py` as the scope payload (`_run_child_scope` launches
  `python native_operation.py --capabilities "" -- cmd`).

The agent guide records operational symptoms of this churn. User-manager inotify `ENOSPC` required
raising `max_user_watches` to 1,048,576 during Plan57 qualification (agent-environment.md
"Placement and memory").

#### 2.3 One compile unit: the hot path

`RUSTC_WRAPPER=scripts/sccache`, running `compiler_cache.main` → `ensure` → `compile_command`:
- interpreter plus imports, about 110 ms;
- `digest(binary)` (full SHA-256 of sccache) plus `digest(configuration)`, about 90 ms per 30 MB measured;
- `ready()`: `systemctl show`, `/proc/<pid>/exe` stat, UDS connect with `SO_PEERCRED`, and `registered()` taking an exclusive ledger lock and rewriting it;
- `supervised_compile` calls `same()` twice before and once after; each `same()` is another `ready()`, so 3 more `systemctl show` and 3 more ledger rewrites;
- `Popen` of the sccache client;
- a `select(0.05)` poll loop, adding up to 50 ms of detection latency;
- `finish_group` with `killpg` SIGTERM, then **`time.sleep(0.05)` unconditionally**, then SIGKILL.

Lower bound is about 250–350 ms of wrapper overhead per unit (estimate, built from the measured parts
plus unmeasured systemctl). It applies to:
- about 900 packages × (lib plus build script) per clean build;
- every invalidated workspace crate on an incremental build, which sit on the critical path, so their overhead is not hidden by parallelism;
- every C/C++ object compiled by the KLU, ibex, PETSc and Uno CMake builds via `CMAKE_*_COMPILER_LAUNCHER`.

With 32 jobs, the wrappers also serialize briefly on the shared host-ledger flock.

**Verdict:** the interpreter start in `pse-env` is a meaningful but bounded per-recipe tax. The
compiler wrapper is a **multiplicative** tax on every build, and it is the single best Rust (or
deletion) target in this slice.

### 3. Per-group notes

#### 3.1 Host admission vs Rust (question 1, and the "duplicate?" question)

What `host_admission.py` controls:
- **Ledger.** `allocation_metadata` takes LOCK_EX on `.lock` and holds a JSON map of owners:
  `{boot, pid, start, class, memory, lane, slots, cores, exclusive, deadline, units:{unit:{group,invocation,inode}}}`.
- **Lane and slot policy.** `select()` maps class to `Profile`. Memory-based promotion goes
  light → functional → wide → exclusive. `conflict()` encodes timing exclusivity, exclusive heavy
  work, and slot counts per lane from `.config/agent-capacity.toml`.
- **Admission loop.** `acquire()` runs a 30 s polling loop: `reconcile` → MemAvailable pressure
  guard → `conflict`. It can park resident services for exclusive work.
- **Kernel enforcement.**
  - `enforce_parent` sets `pse.slice MemoryMax` to the max aggregate of live owners.
  - `enforce_allocation` sets per-allocation slice MemoryMax, MemorySwapMax, CPUQuota and AllowedCPUs, then reads `memory.max` back.
  - `cpu_set()` expands physical cores to SMT siblings from sysfs topology.
- **Drain-before-release.** `drained()` treats a unit as drained when any of these hold: the cgroup inode changed, `cgroup.events populated=0`, the unit is not found, or the unit is inactive. The caller must also be dead or released. Unknown means stay charged.
- **Measurement context.** `measurement_context()` scans `/proc` for external RSS hogs.

**Genuinely new, with no OS or tool equivalent:**
- cross-session cooperative admission with queueing;
- timing/reference vs build exclusion;
- the pressure guard;
- parking resident services;
- charging capacity until verified drain.

**Re-derived from what systemd or the kernel already know:**
- cgroup paths: `systemd_environment.slice_group` rebuilds the path from dashed-slice naming instead of reading the `ControlGroup` property;
- unit liveness: via `systemctl show` subprocesses instead of D-Bus property reads or signals;
- CPU and memory ceilings: `pse_env.cpu_ceiling`, `systemd_environment.current_memory_ceiling`/`limits`/`require_unbounded_group`;
- process identity: `process_observation.boot`/`start_identity`/`process_stat`;
- job and test width: `pse_env.cpu_widths`/`admitted_cargo_jobs` override `CARGO_BUILD_JOBS` and `NEXTEST_TEST_THREADS`. Rust `available_parallelism` already honours cgroup quota and affinity (Cargo's default), and `.config/nextest.toml` already declares `test-threads = 16`.

**Rust duplicates already in tree:**
- `crates/pse-runtime/src/bin/pse-worker.rs::verify_placement_hierarchy` reads `memory.max`, `cpu.max` and `cgroup.controllers` up the ancestry. This is the same algorithm as `pse_env.cpu_ceiling` plus `systemd_environment.current_memory_ceiling`.
- `crates/pse-operations/src/canonical_files.rs` (around line 330) finds `pse-native-*`/`pse-surreal-worker-*` in `/proc/self/cgroup`, runs `systemctl --user show --property=InvocationID,ControlGroup`, and stats the cgroup inode. This is the same as `operation_lifetime.scope_owner` with the same unit-name conventions, kept in two languages.
- `crates/pse-diagnostics/src/process_facts.rs` (procfs-core `Stat`, boot_id, start ticks, pgrp, pid namespace) duplicates `process_observation.py`.
- `crates/pse-diagnostics/src/supervised_child.rs` (pidfd-verified group leader, `kill_process_group`, cancellation eventfd, drain policies) covers what `native_operation.run`/`cancel_children` and `compiler_cache.supervised_compile`/`finish_group` do by hand.
- `pse-runtime::workflow::native_deployment_protocol::ProductionProfiles::read` parses the `[production]` section of the same TOML. Python separately parses the host section untyped (`host_admission.policy` re-reads the file on every `select`/`aggregate`/`settings`/`conflict` call) and shells out to Rust for the `reference` profile.
- `pse-runtime/src/budget.rs` (`ResourceBudget`) is the *in-process* DataFusion pool budget. It is a different layer and not a duplicate.

**Second declarations:**
- `host_admission.classify()` hard-codes recipe names (`unit-package` → exclusive, `check-package` → compile …) that the justfile also declares through `--recipe-default-class`.
- `language_server_environment` keys, `producer_cargo.EFFECTIVE_ENVIRONMENT`, `native_operation.CAPABILITY_PATHS`/`AUTHORITATIVE`, `native_cache.INPUT_ENV` and `xtask/src/producer_identity.rs` (the env name list around line 1180, `is_cargo_env`) are 4–5 separate lists of build-affecting environment variables.

**Rust route.** Build a new small crate, for example `pse-host`, plus a `pse-env` binary:
- typed `CapacityPolicy` (serde/toml, schemars) shared with `ProductionProfiles`;
- transient scopes via D-Bus `StartTransientUnit` (scope with `PIDs=[getpid()]`, `Slice`, `MemoryMax`, `MemorySwapMax`, `AllowedCPUs`, `CPUQuotaPerSecUSec`), then `exec` the payload. This removes `systemd-run`, the `--bind-allocation` second interpreter and the separate `set-property` calls;
- unit observation via D-Bus properties;
- drain via `cgroup.events` (inotify) and pidfds;
- process facts via the existing `pse-diagnostics::process_facts`.

Better structurally: a **long-lived user-service admission daemon** in Rust. It would hold owner pidfds and inotify watches and answer admission over a UDS. That turns `reconcile` from "poll every unit through `systemctl show` on every admission" into event-driven state, and removes the "never hold a lock through a workload" constraint that forces the liveness-proof ledger. The rules file's explicit ban on fd-inheriting locks suggests why the simpler held-flock design was rejected. Treat that as a design constraint to preserve, not to silently drop.

Candidate crates:
- `zbus` (confident it exists);
- `zbus_systemd` with the systemd1 feature (I believe it exists with generated Manager/Unit proxies; *verify*). The fallback is hand-written `#[zbus::proxy]` for about 5 methods;
- `cgroups-rs` (exists, v2 support) is optional, because the reads involved are trivial;
- rustix (present: `pidfd_open`, `pidfd_send_signal`, `flock`; *verify* that `sched_setaffinity` and inotify are exposed with the enabled features).

#### 3.2 `pse_env.py` / `build_environment.py` (environment boundary)

What it executes:
- `compose` merges caller, `.envrc.local` (evaluated in a clean `bash -c`) and defaults.
- `build_environment.configure`:
  - LLVM prefix from `/opt/llvm-current`;
  - `CARGO_BUILD_JOBS` from `.cargo/config.toml`;
  - sccache wrapper selection with `SCCACHE_CLIENT_SIDE=1`, `SCCACHE_DIRECT=false`, a frame length from `.config/build.toml` and a per-cache-dir UDS;
  - refusing foreign `CARGO_TARGET_DIR`;
  - frontend and taskdump experiments that read `rustc -vV` and `.cargo/config.toml` target rustflags.
- `refusals` reports replaced values.
- `placement`/`owned_scope` handle admission and scopes.
- `require_store` starts or unparks SurrealDB through `surreal_server`.
- `operation_description` backs `--explain`.

The precedence and refusal semantics are ordinary env composition and port directly to a Rust
binary (clap plus a small `HashMap` composition).

**Bootstrap fragility (tested here).**
- `scripts/pse-env` falls back to `python3`, and `.envrc` runs `/usr/bin/python3 scripts/pse_env.py --print`.
- But `pse_env` transitively imports msgspec (`host_admission → execution_profiles`).
- With a stdlib-only interpreter, `--print` fails with `ModuleNotFoundError: msgspec`. So "bootstrap can prepare a checkout before its pinned environment exists" holds only if system Python happens to have msgspec.

A Rust `pse-env` needs its own bootstrap answer (§4).

#### 3.3 `native_operation.py` / `operation_lifetime.py` (lifetime and cancellation)

`native_operation.Operation` writes `~/.cache/pse-arrow/native/.operations/<uuid>.json` with pid,
start, scope owner, admissions and pins, and exports `PSE_NATIVE_OPERATION`. Nested processes find
their owner by walking `/proc/*/status` PPid chains (`owner_record`). Admissions are memoized under
`record_lock` (flock with signals masked). Generation pins prevent concurrent GC of native cache
generations.

Foreground handoffs (`prepare_handoff`, `bind_handoff`, `fence_*`, `stop_*`, `retire_unlaunched_*`)
and the composer and terminal-guard protocol (`note_composer_cancellation`, `inherited_cancellation`,
`acknowledge_*`, `request_composer_completion`, 93 lines) carry an "original cancellation clock"
across process hops. Those hops are interpreter to `systemd-run` to interpreter to payload.

`run()` (236 lines) is a hand-written supervisor:
- masks signals while installing handlers;
- `Popen(start_new_session=True)`;
- deadline and timeout handling;
- `killpg(SIGKILL)` escalation 0.5 s before the cancellation deadline;
- `on_reaped` drain joins that poll `operation_lifetime.drained` (`systemctl show` plus `cgroup.events`) every 20 ms.

Most of this complexity comes from crossing Python process boundaries and needing exact exit-status
and signal truth. In a single Rust supervisor with `SupervisedChild` (pidfd), D-Bus `KillUnit` on the
scope (`KillMode=control-group` kills atomically) and an inotify wait on `cgroup.events`, the
handoff, fence and composer layers mostly collapse. The env-composition part (`_environment`,
`runtime_selection`, reading the solver Dockerfile `ENV` via `native_cache.runtime_env`) is plain
data and ports trivially.

#### 3.4 `compiler_cache.py`

See §2.3 for the per-call path. The service side (`ensure` → `unit_text` writes a unit file
with `Slice=`, `MemoryMax=`, `CPUAffinity=`, `KillMode=control-group`, then `daemon-reload` and
`start`; `serve` verifies `startup_inputs()` hashes of its own Python sources and execs a frozen
sccache copy) is a legitimate one-time setup. But it is re-verified on **every** compile.

Two simpler routes:
1. A static or templated `pse-sccache.service` plus `RUSTC_WRAPPER=<real sccache>` with
   `SCCACHE_SERVER_UDS`. sccache's own client handles connect and start races.
2. A Rust shim that reads a cached descriptor, does a single `SO_PEERCRED` check, and `exec`s
   sccache. It would not supervise.

Either removes the interpreter, the hashing, the systemctl calls and the sleeps from the compile
path. `startup_inputs()` hashing `host_admission.py`, `native_operation.py` and others is a symptom:
the service's identity depends on the Python control-plane source.

#### 3.5 `native_cache.py` / `native_pipeline_cache.py`

`prepare()` builds an identity dict (tool versions and byte identities via `cmake --version` and
`clang -print-resource-dir`; source hashes; env inputs). It builds into a staged directory, seals it
(chmod), and atomically publishes `<kind>/<digest>` with a `.complete.json` file list. `collect()`
GCs unpinned generations, coordinating with `native_operation.pin`.

- `klu()` runs a CMake build of vendored SuiteSparse KLU.
- `isolation()` uses curl to fetch ibex, then CMake.
- `solver()` streams `docker run … tar` out of the pinned solver image.
- `native_pipeline_cache` builds Uno, PETSc and HiGHS from pinned archives and runs a strings-based
  PETSc verification.

The receipt schema lives only in the Python writer. Rust `build.rs` takes it as opaque bytes via
`build_receipt::materialize`, so the cross-language contract is undeclared.

Ports cleanly to xtask (`cmake` and `flate2` are already in Cargo.lock; add `tar`). A more
Cargo-native option for KLU: the build script of the already-vendored
`vendor/suitesparse-sys` could build only KLU. The `native-math-env.sh` comment says the stock
build enables CHOLMOD/CUDA. Cargo would then own freshness and caching, and an env-var receipt
handshake would no longer be needed.

#### 3.6 Observation modules

- **`activity.py`:** `systemctl list-units 'pse-*' --output=json`, per-unit show and memory, and a `/proc` scan of cargo, rustc, nextest and maturin with cwd in the checkout. It runs outside admission by design. Port with zbus `ListUnitsByPatterns` plus procfs.
- **`pressure_observation.py`:** PSI (`/proc/pressure/*`, `<cgroup>/{cpu,io,memory}.pressure`) plus `cpu.stat`, `memory.events` and placement files, with incarnation (inode) bound deltas and msgspec structs, exported via `telemetry_metrics.host_metrics` (Python OTel). This is real NEW-OBS. The parsing is small and maps to Rust `opentelemetry` (present).
- **`build_observation.py`:** walks `/proc/<pid>/task/<pid>/children` to label rustc, linker and build-script stages. Cargo's own timings cover this.
- **`activity_watch.py`:** long-polls capability snapshots. It is coupled to `scripts/capabilities/*`.

#### 3.7 Producer and receiver group

`producer_cargo.py` is a Cargo stand-in that maturin calls during strict capture. It validates the
child Cargo path, runs Cargo, then calls `target/debug/xtask producer-identity
--observed-cargo-arguments …`. Rust already owns the identity. `installation_context.configuration`
re-walks Cargo config discovery (ancestor `.cargo/config(.toml)`, then `$CARGO_HOME`). This
duplicates `xtask/src/producer_identity.rs::cargo_configuration_files`. `producer_receipts`
(SHA-256 file observations) overlaps `pse-buildinfo::FileObservation` and
`verify_receipt_artifact`.

`receiver_closure.py` exists only because the receiver ships Python code. It AST-walks owned imports
and import-probes them in an isolated interpreter.

#### 3.8 Small items

- `native-engine-boundaries.py` hard-codes forbidden reachability:
  - `pse-engine`, `pse-testkit` and `pse-admission` must not reach catalog/runtime/compiler/operations/math/deltalake/backend/ipopt;
  - testkit edges;
  - removed files.

  `[workspace.metadata.pse.dependency-ceilings]` plus `xtask/src/dependency_ceilings.rs` already
  express and enforce exactly this kind of rule. This is a clear DUP.
- `scripts/workspace.rs` is Rust, included by `#[path]` into `xtask/src/main.rs` and
  `tests/governance/tests/common/mod.rs`. It is a second checkout-root finder alongside
  `pse-buildinfo::identity::workspace_root`, and the two use different predicates.
- `scripts/cargo-bin/cargo-pse-correctness`: the `cargo c`/`cargo t` aliases (`.cargo/config.toml`) dispatch into Python `arrow_validation.main`. That module is outside this slice, but it puts Python in front of every correctness check and test run.

### 4. Cross-cutting observations

1. **Python on hot paths.**
   - Per compile unit: rustc wrapper and CMake launcher (§2.3).
   - Per recipe: two interpreters (§2.2).
   - Per `cargo c`/`t`: an `arrow_validation` interpreter.
   - Per direnv reload and per Claude session: `pse_env --print`.

   Measured warm interpreter-plus-import cost is 110–180 ms per hop; cold is about 0.8 s.
2. **Subprocess-as-API to systemd.** There are 14 `"systemctl"` command literals across `host_admission` (6), `pse_env` (2), `native_operation` (2), `compiler_cache`, `operation_lifetime`, `surreal_server` and `telemetry_service`, plus `systemd-run` in 5 modules. Each parses `key=value` stdout. Rust also does this in `pse-operations/canonical_files.rs`. A single D-Bus client (zbus) replaces all of them with typed property reads and removes process spawn per observation.
3. **Re-derived state.** CPU width, cgroup paths, Cargo configuration, build phases, test threads and process identity are recomputed in Python while Cargo, nextest, systemd, procfs and existing Rust (`process_facts`, `verify_placement_hierarchy`) already hold them. Every re-derivation adds a place where two answers can disagree. Examples: `.config/nextest.toml test-threads=16` vs `pse_env`'s `NEXTEST_TEST_THREADS`, and justfile recipe classes vs `classify()`.
4. **Files plus flock plus JSON as the control plane.** `allocations.json`, `.operations/*.json`, compiler-cache `current.json`/`reservation.json`, `observations/admission/*.json` and native `.complete.json` are all ad-hoc JSON. Writers are Python; some readers are Rust (`build.rs`). None are generated from a Rust schema, except the `deployment` types that `execution_profiles`/`recovery_operation` validate. The ledger is rewritten on every read under LOCK_EX.
5. **Duplicated sources of truth.**

   | Concept | Python | Rust |
   |---|---|---|
   | Scope-owner identity | `operation_lifetime.scope_owner` | `canonical_files.rs` |
   | Cgroup-ancestry limits | `pse_env` + `systemd_environment` | `pse-worker.rs` |
   | Process identity | `process_observation` | `pse-diagnostics::process_facts` |
   | Group supervision | `native_operation.run`, `compiler_cache` | `SupervisedChild` |
   | Cargo config discovery | `installation_context` | `producer_identity.rs` |
   | Dependency-graph rules | `native-engine-boundaries.py` | `dependency_ceilings.rs` |
   | Workspace root | `scripts/workspace.rs` | `pse-buildinfo::workspace_root` |

   Also: the capacity TOML has two parsers, one per section; there are 4–5 build-env-var allowlists; and recipe classes are declared both in the justfile and in `classify()`.
6. **Testing burden tracks lifecycle edge cases.** About 26K test lines for about 19.5K code lines, concentrated in `native_operation` (4.9K) and `host_admission` (3.2K). The volume reflects the cross-process handoff, cancellation-clock and drain protocols that a single Rust supervisor with pidfds would largely not need.
7. **Recommended Rust order, by payoff.**
   1. **Compiler wrapper.** Drop `compiler_cache.py` from the per-compile path: real sccache as `RUSTC_WRAPPER` plus a user service, or a Rust exec-shim. Small and immediate. Needs no other port.
   2. **`pse-env` as a native binary** with D-Bus transient scopes (`StartTransientUnit` with self PID, then `exec`). This removes the second interpreter, `systemd-run` and most systemctl spawns.
   3. **Host admission and native-operation lifetime** as one Rust crate or daemon built on `pse-diagnostics` (`process_facts`, `SupervisedChild`), with typed policy shared with `ProductionProfiles`. Delete `execution_profiles.py`, `process_observation.py` and `systemd_environment.py`.
   4. **xtask subcommands** for `native_cache`/`native_pipeline_cache`, producer capture, `python_extension`, `doctor`/`preflight`, `solver-images`/`supplier_source`. Delete `native-engine-boundaries.py` (move into dependency-ceilings), `cargo_freshness_probe.py` (no caller) and `build_observation.py` (Cargo timings).

   **Bootstrap caveat:** the env boundary configures Cargo, so a Rust `pse-env` must be buildable before it exists. Use plain `cargo build -p pse-env` without the wrapper in `bootstrap.sh`, or install it into a checkout-local bin. Keep it a separate lean crate rather than an xtask bin: xtask carries 43K lines and heavy optional deps.
8. **Uncertainty flags.**
   - `zbus_systemd` crate name, features and generated method names.
   - Rustix feature coverage for `sched_setaffinity` and inotify with the current feature set (`fs, time, thread, process, event`).
   - Exact nightly flag for JSON `cargo --timings`.
   - sccache binary size, which affects the 90 ms hash estimate.
   - Real `systemctl --user show` latency on the workstation; not measurable here, since there is no user manager.

   The per-recipe and per-compile totals above are estimates built from measured interpreter and hash costs plus these unmeasured parts.

---

## D: Test orchestration, validation, evidence, measurement, diagnostics and scientific references

Slice: 51 modules in `scripts/` (24,669 Python lines), plus `scripts/diagnostic_sql/` (3 SQL files, 65
lines) and `scripts/probes/faer-nonfused.rs` (10 lines). Snapshot: `/home/claude/pse` at bd1c4891e.
Method: I read the code paths that execute (entry points, subprocess calls, cross-module imports, and
the Rust callers), not just the docstrings. Then I grepped `crates/`, `xtask/`, `benches/`, `tests/`,
`.config/nextest.toml` and the `justfile` for the same concepts. Nextest capability claims were checked
against the nextest docs (Context7, nexte.st). Anything I could not verify is marked *uncertain*.

Headline: about 93% of this slice (≈22.9K lines) is orchestration, lifecycle and receipt machinery
wrapped around Rust tools: cargo, nextest, criterion, cargo-hack, cargo-mutants, llvm-cov, cargo-fuzz
and the PyO3 diagnostics. It is not Python-ecosystem work. Only about 1.7K lines are true independence
oracles (scipy, teqp, IDAES, Decimal) that should stay Python. Two findings matter most:

1. **Python sits on the Rust test hot path.** Canonical Rust fixtures spawn a Python interpreter per
   fixture action.
2. **The Rust test runner is a Python pipeline that re-derives state nextest already owns.** That
   state is the inventory, the JUnit outcomes, scheduling groups, archives and the cargo argument
   grammar.

---

### 1. Summary table

LOC is from `wc -l`. Classes: primary, then secondary. Effort: S (≤1 week), M (weeks), L (multi-week
with race or qualification risk).

| # | Module/group | LOC | What it does (1–2 lines) | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---|---|---|---|---|---|---|---|
| 1 | `test_resources.py` | 3,248 | Fixture/report lifetime registry: registers isolated SurrealDB databases and namespaces, plus report dirs, per selected test. Records drain, freezes the terminal outcome and reclaims on pass (`REMOVE DATABASE`). Also covers custody relocation of report dirs and orphan recovery. CLI driven by Rust fixtures through stdin JSON. | NEW-CONTROL / NEW-STORE | The Rust fixture caller in `pse-operations/src/testing.rs` (`resource_bridge`). The SurrealDB Root client in `pse_operations::transport::client`. `pse-diagnostics::SupervisedChild`. | PORT-EXISTING: move the registry into `pse-testkit` or `pse-operations::testing` and call it in-process. Python conftest uses it through `pse._native` or a small binary while pytest remains. | rustix (flock), tempfile, serde_json, existing gRPC Root client. Optional redb or rusqlite instead of JSON files. | L |
| 2 | Runner composer: `test_run.py`, `native_tests.py`, `python_tests.py`, `select.py`, `test_invocation.py` | 3,249 | Wraps `cargo nextest list/run` and pytest. It builds via `list --list-type binaries-only`, captures binaries and cargo metadata, lists the inventory and maps requirements. It writes a per-run nextest TOML (JUnit path and test-group overrides), builds the worker and fixture binaries, and sha256-hashes every binary. It runs under a systemd observer that re-enters Python, then parses JUnit and freezes and reclaims resources. | ORCH / NEW-CONTROL | nextest itself (selection, groups, JUnit, `--binaries-metadata` reuse, record/rerun). xtask `producer_preparation.rs` (unit graph, cargo metadata). | PORT-EXISTING into an `xtask test` runner. DELETE `select.py` and the CLI grammar tables in favour of nextest pass-through. | nextest-metadata (list/binary summaries), quick-xml (JUnit), tokio::process, clap. Nextest setup and wrapper scripts (see §4). | L |
| 3 | `test_requirements.py` | 558 | Hard-coded Python table of 93 Rust module-prefix/test-name entries (plus 6 exact fixture/peer tests), mapped to effects/capacity/roles. Projects one nextest override per selected test (`test-group`, `threads-required`) and guards fixture effects. | DUP / ORCH | `.config/nextest.toml` `[test-groups]` (`scientific`, `store`) and filterset overrides already list the same solver modules. | DELETE: replace with static filterset overrides in `nextest.toml` plus a Rust runtime guard on `NEXTEST_TEST_GROUP`. The Python-marker half stays with pytest. | nextest config only | S–M |
| 4 | `test_archives.py`, `experiments.py` | 861 | Wraps `cargo nextest archive` and replay (`--archive-file`, `--workspace-remap`). Adds an `ldd` library-closure digest, staging of input fixtures and scientific capsules, platform and env checks, and a receipt. `experiments.py` is a 40-line dispatcher. | ORCH / NEW-STORE | nextest archive and `archive.include`. | PORT-EXISTING (xtask). Use `archive.include` for staged inputs. Keep the `ldd` and digest check as a small Rust step. | nextest `archive.include`, sha2, goblin or object (ELF DT_NEEDED) as an `ldd` alternative (*uncertain whether worth it*) | S–M |
| 5 | `affected.py` | 412 | `git diff` → owning package → reverse-dependency closure. Emits a nextest `rdeps()` filterset under arrow force-validation, plus advisory notes. | ORCH | `cargo_metadata` is already a workspace dependency, and nextest `rdeps()`. | PORT-LIB | guppy + determinator (changed-path → package set, including Cargo.toml/lock diffs) | S |
| 6 | `arrow_validation.py` | 464 | Front door for every `just` cargo recipe. Runs `cargo metadata --all-features` and 1–2 `cargo <mode> --unit-graph -Z unstable-options`, adds `pkg/force-validate` features for Arrow-consuming roots, verifies leaf `force_validate`, then execs cargo. Holds its own cargo/nextest flag grammar. | ORCH / WRAP | `xtask/src/producer_preparation.rs` already parses the unit graph with `cargo_metadata` (line ~184). | PORT-EXISTING (xtask subcommand or cargo alias), with the resolution cached by manifest and lock hash. | cargo_metadata, serde | S–M |
| 7 | Gate runner: `validation.py`, `validation_scope.py`, `validation_receipts.py`, `validation_result.py` | 2,948 | Keep-going runner over `just <recipe>` gates. It logs each gate and parses JUnit, hashes the whole source tree (`git ls-files` + sha256 per file), reuses or transfers prior passes when inputs are unchanged, and publishes versioned assessment receipts (v6). `validation_result` is the `just result` reader. | ORCH / NEW-STORE | The justfile owns the commands. xtask has a separate source-identity hasher (`producer_identity.rs::reviewed_source_identity`, blake3 FramedHasher). | PORT-EXISTING (`xtask assess`). Unify source identity with the xtask hasher. | tokio::process, serde, blake3 + ignore (both in tree), quick-xml | M |
| 8 | `validation_feature_campaign.py` | 338 | Runs `cargo hack --feature-powerset --depth 2` with a fake `CARGO_HACK_CARGO_SRC` (`scripts/cargo_feature_check.py`) to capture literal argv. It then runs each combination with receipts and resume-from-parent reuse. | ORCH / NEW-STORE | cargo-hack (`--keep-going`, `--partition`) | PORT-EXISTING (fold into the item 7 runner) | cargo-hack; serde | S–M |
| 9 | Tool wrappers: `audit_tools.py`, `assertion_sensitivity.py`, `coverage_correspondence.py`, `boundary_fuzz.py`, `setup_report.py`, `highs_lifecycle_diagnostic.py` | 1,538 | Exit-code and JSON classification for cargo-deny, audit, shear and geiger. Mutant outcomes from cargo-mutants 27.1. Changed-region coverage from the llvm-cov JSON export against a `git diff`. cargo-fuzz campaign receipts. unittest + xmlrunner for the setup suite. One hard-coded HiGHS test under an ASAN or gdb build. | ORCH (coverage_correspondence: NEW-OBS) | All the underlying tools are Rust. | PORT-EXISTING (xtask helpers or just recipes). `setup_report` stays while the Python setup suite exists. | rustsec (*optional*), serde_json, cargo-mutants `--test-tool nextest` | S each |
| 10 | Case observation and measurement: `case_audit.py`, `case_measure.py`, `pipeline_review.py`, `reference_run.py` | 3,346 | Spawns per-case bench processes from `.config/process-cases.json`. Polls Rust diagnostics through PyO3 (`pse.diagnostics.read_checkpoint`, `inspect_image`, `inspect_frozen`, `launch_capture`). Reads criterion `raw.csv` and computes mean and SE. Wraps the managed SurrealDB receiver and stop. Publishes HTML/CSV review packs. `reference_run` sequences the SurrealDB setup/start/init/work/stop lifecycle. | ORCH / WRAP / NEW-OBS | `pse-runtime/src/tooling/*` (27K lines: capture, inspection, frozen_inspection, review_pack/launch/analysis). `pse-diagnostics` (SupervisedChild). benches already write `*-memory.json` (VmHWM) and criterion output. | PORT-EXISTING (Rust driver over pse-runtime tooling, in-process with no PyO3 round trip). Expose catalog cases as nextest tests via libtest-mimic. | libtest-mimic, csv, criterion `estimates.json`, statrs | M |
| 11 | `regression_experiments.py` | 856 | `git bisect run` over one hard-coded predicate test (`pse-admission::authoring_load_package::…`), plus paired-measurement comparison (≥3 pairs, spread checks) over build-measurement receipts. | ORCH / NEW-OBS | (none) | PORT-EXISTING (xtask). Generalise the predicate to any nextest filter. | gix (*optional*) or git CLI; statrs | M |
| 12 | Evidence projection and inspection: `evidence_sources.py`, `evidence_projection.py`, `evidence_detail.py`, `run_inspection.py`, `evidence_comparison_report.py` | 2,206 | Decodes assessment receipts, diagnostic actions, storage-health, host and measurement files into `EvidenceSnapshot` facts. These go to the Rust `xtask evidence` tool, which loads SurrealDB and queries it. Also renders an offline HTML report with plots from a native comparison. | NEW-OBS / DUP | `xtask/src/development/evidence.rs` (`EvidenceFact` :263, `EvidenceSnapshot` :333, validate()) and `evidence_query.rs`/`comparison.rs`. Generated Python models exist at `scripts/generated/evidence_snapshot_input.py`, but the producers don't use them. | PORT-EXISTING: producers into xtask, deleting the hand TypedDicts. The HTML report can stay Python as presentation only, or move to Rust. | serde, plotters or askama (*if ported*) | M |
| 13 | Active diagnostics and traces: `active_diagnostics.py`, `diagnostic_trace.py`, `diagnostic_sql/` | 1,784 (+65 SQL) | Host debugger actions with receipts: py-spy, perf, perf sched/stat, heaptrack, gdb launch/core, rr record/replay, memray. Also a pinned tool installer (download + sha256 + tar) for perfetto `trace_processor` and rr, and runs SQL over Rust-produced perfetto traces. | NEW-OBS / ORCH | `pse-diagnostics` (SupervisedChild, process_facts). pse-runtime `capture/timeline.rs` writes the perfetto traces. | PORT-EXISTING (Rust CLI on pse-diagnostics). Python-target actions (py-spy, memray, python-perf) get deleted as Python processes disappear. Option: run the trace SQL with DataFusion over an Arrow timeline export. | reqwest + sha2 + tar/zstd for the installer, datafusion (*design option*) | M |
| 14 | `faer_arithmetic.py` (+ `probes/faer-nonfused.rs`), `numerical_investigation.py` | 460 | faer: regex scan of `vendor/faer` for FMA or arch bypasses, `rustc --emit=llvm-ir` of the probe, and a walk of the IR call graph for fused ops. numerical: 100-digit `Decimal` closed-form derivatives against the diagnostic samples, plus a matplotlib plot. | NEW-DEVTOOL / ORACLE | the faer arithmetic tests (`pse-backend-native/src/arithmetic_tests.rs`) | faer: PORT-EXISTING (xtask, regex crate). numerical: KEEP-PY as a tiny oracle, or port with symbolica arbitrary precision (in tree; *untested here*). | regex, syn | S |
| 15 | `scientific_inputs.py` | 266 | Adapter: `cargo run -p xtask scientific-inputs` produces a capsule, which it binds to `PSE_SCIENTIFIC_INPUTS`. Holds hard-coded Rust test-owner prefixes and Python test-file lists to decide relevance. | WRAP / DUP | `xtask/src/scientific_inputs.rs` (1,047 lines) owns the capture | DELETE the adapter: a nextest setup script plus a declaration in the item 3 nextest overrides. | nextest setup scripts | S |
| 16 | scipy oracles: `scientific_reference.py`, `pfr_reference.py`, `cstr_controller_reference.py`, `decay_fit_reference.py` | 589 | Independent reduced equations via `solve_ivp` and `least_squares`. They regex-read frozen literals from authored `.pse` fixtures, compare, and publish JSON with pinned numpy/scipy identity. | ORACLE | — | KEEP-PY (independence; isolated venv `build/plan53a-reference`) | — | — |
| 17 | teqp and Decimal oracles: `plan14_reference.py`, `feos_entropy_reference.py`, `pr_stability_reference.py` | 654 | PC-SAFT, PR tangent-plane and caloric references from teqp + Decimal + scipy. Writes Parquet or JSON reference fixtures. | ORACLE | The xtask `feos_reference.rs` FeOS oracle is a second, independent Rust oracle. | KEEP-PY (teqp has no Rust binding that I know of; *uncertain*) | — | — |
| 18 | IDAES oracles: `idaes_nrtl_reference.py`, `idaes_srk_reference.py` | 201 | Freezes NRTL and SRK states via IDAES/Pyomo into Parquet. | ORACLE | — | KEEP-PY (IDAES is Python-only) | — | — |
| 19 | Data generators: `gross2001_bank.py`, `thermodynamic_feeds.py` | 142 | Frozen FeOS JSON → `parameters.parquet` (Decimal sigma scaling). scipy LHS (`rng=23`) → `flash-feeds.parquet`. | WRAP (convenience) / ORACLE-provenance | `feos_reference.rs` reads the same `gross2001.json` and checks the **same SHA constant** | gross2001: PORT-EXISTING into the xtask feos-reference. feeds: KEEP-PY as provenance of a frozen artifact. | arrow/parquet (in tree) | S |
| 20 | `thermodynamic_campaign.py` | 386 | Drives the *product* through the Python API (`pse.Runtime`, Ipopt, 200 feeds) and records outcomes to Parquet. | WRAP / NEW-OBS | pse-runtime itself | PORT-EXISTING (bench or xtask over pse-runtime), unless it is meant as a Python-API journey. | — | S–M |
| 21 | `grpc_wire.py`, `grpc_wire_schema.py` | 163 | `grpc_wire`: hand-rolled protobuf varint decoder used by `surreal_telemetry.py`. `grpc_wire_schema`: **xtask codegen shells out to Python** (`xtask/src/codegen/grpc_wire.rs:50`) to regex-parse prost-generated Rust and emit Rust. | DUP / GEN | prost/tonic are in tree, and the surrealdb-protocol crate. | DELETE: generate inside xtask with `syn` (or prost-reflect over the descriptor). Replace `grpc_wire` with prost once its consumer moves. | syn, prost-reflect | S |

**Totals by disposition (approx.):** KEEP-PY ≈1.7K lines (groups 16–18, numerical_investigation,
thermodynamic_feeds). The rest, ≈22.9K lines, is portable or deletable. Within that, roughly 9K lines
(groups 1–3, 6, 7) form one tightly coupled cluster with the lifecycle modules outside this slice:
`resource_store` (1,005), `service_borrowers` (179), `host_admission` (1,620), `operation_lifetime`
(426), `native_operation` (2,024), `surreal_server` (11,748) and `invocation_disposition` (386). That
cluster has to move as a unit.

---

### 2. Per-group notes (evidence)

#### 1. `test_resources.py`: fixture lifetime registry (genuinely new control, but in the wrong language)

- **Rust → Python per fixture action.** `crates/pse-operations/src/testing.rs::resource_bridge_until`
  builds `Command::new(.venv/bin/python).args(["-m","scripts.test_resources", action])` and pipes
  JSON in and out under a deadline. The actions are `register`, `drain`, `record-drain`,
  `setup-failure` and `status`. That file has 11 `resource_bridge`/`registered_resource` call sites.
  On the Python side, `main()` (line 3055) dispatches 16 actions.
- **Measured** in this sandbox (warm page cache, CPython 3.12, not the user's workstation): `import
  scripts.test_resources` takes 0.09–0.15 s and loads 29 `scripts.*` modules. `surreal_server` (11.7K
  lines) is already pulled in transitively. Add interpreter start (~15 ms) and file-lock/fsync work per
  call. Every canonical fixture makes at least 2 calls (register, then drain or record-drain). A rough
  count gives about 230 test attributes in `pse-operations` server-unit files alone (*estimate*), plus
  the CANONICAL_NATIVE modules of `pse-runtime`.
- **What it does.** `register()` (l.779) validates the identity regexes (`DATABASE`, `NAMESPACE`)
  and checks the current invocation membership and the requirement guard. It unparks or starts the
  SurrealDB service through `surreal_server` and takes `context_admission`, then
  `resource_store.FileStore.register` writes JSON files under flock and fsync. `finalize()` (l.1105)
  maps the frozen terminal outcomes to `disposition` and `pin`. `reclaim()` (l.2941) marks
  `cleanup="removing"` and then calls `remove_owned` → `namespace_sql` → `REMOVE DATABASE/NAMESPACE`.
  That SQL goes through `surreal_server.administrative_results`, which uses **urllib HTTP POST /sql**.
  This duplicates the transport path: Rust reaches the same server through gRPC
  (`pse_operations::transport::client::Root`, already used by `xtask/src/development/evidence.rs`).
  `_custody_*` and `relocate_reports` (l.1424–2147) are a journaled copy protocol for moving retained
  report directories.
- **Process identity.** The fixture is matched to its test by reading `/proc/<ppid>/exe` and
  `/proc/<ppid>/cmdline` and looking for `--exact <test>` (`selected_test`, l.613). The Rust side
  reconstructs the same identity from `std::env::args()` (`selected_fixture_identity`).
- **Architecture constraint.** blueprint §24.x (`operations-and-validation.md`, ADR-0174, still
  proposed) says that "Fixture Drop/exit records borrower/drain facts, never test success". Only the
  terminal composer's reconciled Pass permits cleanup. So the tempting simplification of having the
  Rust fixture decide in `Drop` via `thread::panicking()` is a *design change*, not a port. A faithful
  port keeps the composer and freeze protocol, implemented in Rust.
- **Complexity drivers.** Signals and deadlines are threaded through every call (`deadline`,
  `deadline_supplier`). There are cross-process authentication checks (pid, start-ticks, boot id), a
  JSON-file database with a three-level lock order, the legacy migration and conversion
  (`resource_conversion`, `invocation_disposition`), and the report custody journal. Two languages
  share one protocol through environment variables and stdin JSON.

#### 2. Runner composer: re-derives nextest's own state

Executed sequence for `just unit-package <pkg> <filter>`:
`select.py` → `test_run.run_rust` → (`PSE_NATIVE_OPERATION` set) `observer_rust` →
`native_tests.ordinary_rust_capture`/`_rust_capture` (l.925). The capture step:

1. Runs `arrow_validation.resolve` (`cargo metadata --all-features` + `cargo test --unit-graph`, twice
   when features are added).
2. Runs `cargo nextest list --list-type binaries-only --message-format json` (the build).
3. Runs `cargo nextest list --binaries-metadata … --cargo-metadata …` (the inventory).
4. Computes `test_requirements.rust_catalog` and writes `*-scheduling.toml` (one override per selected
   test).
5. Builds the worker and fixture binaries (`worker_binary`, `prepare_rust_fixtures`, which call cargo
   again).
6. Takes the sha256 of every selected test binary, the worker and the metadata files
   (`native_provenance`).
7. Hashes the rust-product source tree before and after preparation (`rust_source_identity()` →
   `validation.sources`, which reads and sha256s every tracked file under crates/xtask/tests/benches/
   vendor/packages/scripts).

Then `surreal_server.observer` launches `systemd-run … python -m scripts.test_run --observer-child`,
which verifies again and finally runs `cargo nextest run --binaries-metadata …` with a per-run copy
of `nextest.toml` (`test_invocation.report_configuration` edits only `junit.path` with tomlkit). After
the run, `validation.collect_report` parses the JUnit with ElementTree, `test_run.finish` maps the
outcomes, and `test_resources.finalize` and `reclaim` follow.

- The **process nesting** is: just → bash `pse-env` → python `select.py` → systemd-run python
  observer child → `cargo nextest` → test binary → python `test_resources` (per fixture).
- **Hand-written CLI grammars for tool-owned syntax.** Four copies:
  - `arrow_validation.VALUE/FLAG/NEXTEST_VALUE/NEXTEST_FLAG`
  - `test_invocation.CARGO_VALUE/CARGO_FLAG/RUN_VALUE/…/PYTHON_VALUE/PYTHON_FLAG`
  - `assertion_sensitivity.split_arguments`
  - `native_tests.managed_rust_arguments`

  Each tool upgrade (nextest 0.9.146, cargo-mutants 27.1) can silently break them.
- **Environment-variable protocol.** 75 distinct `PSE_*` names are used across the orchestration
  modules (count by grep), among them `PSE_TEST_INVOCATION`, `PSE_NATIVE_PROVENANCE`,
  `PSE_NATIVE_SELECTION`, `PSE_WORKER_BINARY`, `PSE_RUST_OBSERVER_CHILD`,
  `PSE_TEST_EXECUTION_EFFECTS` and `PSE_SCIENTIFIC_INPUTS`.
- **Genuinely new here:** binding the selected catalogue to the fixture registry, the
  freeze/resume-after-signal composer (`execute_selected`, l.184–466), and binary-provenance
  receipts. Everything else is nextest's job.

#### 3. `test_requirements.py`: second source of truth for Rust test layout

- `RUST_OWNERS` lists module prefixes for pse-operations (21 canonical modules), pse-runtime (54
  entries) and pse-backend-native (15 solver modules). `RUST_FIXTURES` and `RUST_PEERS` name 6 exact
  test functions. `rust()` also has 4 more hard-coded test-name special cases inline.
- The **pse-backend-native list duplicates `.config/nextest.toml`**. The first override there,
  `filter = '(package(pse-backend-native) & test(/^(kinsol|ipopt|uno|petsc|root_isolation|dynamics|highs|clarabel_tests|restart_tests|scip_tests|l1_tests|pounce_convex_tests|measurement_tests)::/))…'`,
  sets the slow-timeout. The same set appears again in Python to assign `test-group="scientific"`.
  The nextest.toml comment says groups are "projected … by scripts.test_requirements".
- **Native replacement.** Static `[[profile.default.overrides]]` with the same prefix filtersets,
  plus `test-group` and `threads-required`, is a 1:1 translation. The "undeclared effect" guard
  (`guard()`, `fixture_guard()`) can become a check in the Rust fixture: nextest sets
  `NEXTEST_TEST_GROUP` (`"@global"` when ungrouped) and the slot variables since 0.9.90 (verified in
  the nextest docs). The fixture refuses a canonical effect unless the group is `store` or
  `scientific`.

#### 4. `test_archives.py`

- `archive()` refuses native closures (`require_pure_closure`), runs `cargo nextest archive`, stages
  declared fixture inputs and the scientific capsule into `workspace/`, records the `ldd` closure with
  sha256 (`library_closure`) and the runtime env, and writes `archive.json`. `replay()` re-verifies
  everything, runs `nextest list` and `run` with `--archive-file/--extract-to/--workspace-remap`, and
  registers a fresh invocation.
- Nextest's `profile.<p>.archive.include` (verified) covers extra files relative to the target dir.
  Workspace-relative inputs would still need staging (*uncertain whether `relative-to = "workspace"`
  exists*). Nextest's record and portable recordings (`cargo nextest replay -R run.zip`, `run -R`;
  verified) cover "keep the evidence of a run". They do not cover "re-execute an old build".

#### 5. `affected.py`

`changed_files` (git diff + untracked) → `workspace_packages` (`cargo metadata --no-deps`) → reverse
closure → `rdeps(…)` filterset → `arrow_validation.compose`. Also scientific-input reach via
`scientific_inputs.affected`. This is exactly the guppy/determinator problem: determinator diffs two
PackageGraphs plus a path list. Port it as a small xtask command.

#### 6. `arrow_validation.py`

Every compile and test recipe in the justfile goes through it (`"{{ py }}" -m scripts.arrow_validation
check|clippy|nextest|bench|test|build|run|miri`; 67 justfile lines). The Python itself is
cheap. Its cost is the 2–3 extra cargo invocations per command (*not measured here*: the snapshot has
no `target/` or toolchain run). xtask already deserialises the same `--unit-graph` output with
`cargo_metadata` (`producer_preparation.rs`), so this is a straight port. Cache the `Resolution` keyed
by the (Cargo.toml set, Cargo.lock, args) hash.

#### 7. Gate runner and receipts

- `validation.run_gates` (l.764) runs `["just", recipe, *args]` per `Gate` in
  `validation_scope.GROUPS`, a Python mirror of recipe aggregates such as `ci-fast`, `hygiene` and
  `turn-end`. It writes logs with a follower thread and kills process groups on interrupt. It reuses
  the parent receipt's passing checks when the input identity is unchanged
  (`validation_receipts.reuse_checks`). It publishes through `assessment_store` and registers the
  output dir as an "evidence" resource in `test_resources` (`register_report`).
- `validation.sources()` (l.236): `git ls-files` over `SOURCE_PATHS` (including `vendor`, `docs`,
  `scripts`), then `sha256(mode:bytes)` of every file. It is called at the start and end of
  assessments, feature campaigns, case_measure, coverage and every Rust test preparation. **Second
  source-identity scheme:** xtask computes blake3 framed identities (`reviewed_source_identity`).
- The only value just lacks natively is keep-going plus receipts and reuse. That is a few hundred
  lines in Rust on tokio::process, serde and blake3 + ignore (in tree).

#### 8. `validation_feature_campaign.py`

It captures cargo-hack's literal argv by pointing `CARGO_HACK_CARGO_SRC` at
`scripts/cargo_feature_check.py`, then replays each combination with resume-from-parent. This is a
reasonable design and ports cleanly. A Rust capture shim can play the same `CARGO_HACK_CARGO_SRC` role.

#### 9. Tool wrappers

`audit_tools.finding_status` decodes cargo-deny's 4-bit exit mask and the cargo-audit and cargo-shear
JSON. `assertion_sensitivity` classifies cargo-mutants `outcomes.json` scenarios. Its only logic is
splitting arguments between the build and test phases, and that disappears if cargo-mutants is driven
with `--test-tool nextest` and nextest filter pass-through. `coverage_correspondence` intersects the
llvm-cov JSON export with `git diff -U0` hunks (a NEW-OBS diff-coverage feature, about 200 lines of
real logic). `boundary_fuzz` writes a cargo-fuzz campaign receipt. `highs_lifecycle_diagnostic` runs
one hard-coded nextest test under ASAN/gdb. All are thin ORCH, and all their tools are Rust binaries.

#### 10. Case observation and measurement

- `case_audit.Observer` imports `pse.codec` and `pse.diagnostics` (PyO3). It polls
  `read_checkpoint` → `inspect_image` / `inspect_frozen` every 5–15 s while the case child runs, and
  writes JSON for each window. Those functions live in Rust
  (`pse-runtime/src/tooling/inspection.rs::inspect_image`, `frozen_inspection.rs::inspect_frozen`,
  `capture/launch.rs`). **Python here is a polling loop that crosses PyO3 to call Rust and serialise
  Rust documents to files.**
- `case_measure`: `cargo bench --no-run --message-format=json` locates the binary. It then spawns the
  binary per workload with `PSE_PROCESS_COST_*`, reads the bench's own `{case}-memory.json` (the bench
  measures VmHWM and pools in Rust), and reads criterion's `new/raw.csv` to compute the mean, sample
  SE, min and max. Criterion already emits `estimates.json` with bootstrap CIs, so this re-derives
  weaker statistics. The managed-receiver checks call `surreal_server`.
- **libtest-mimic route.** `.config/process-cases.json` and `preparation-cases.json` are case
  catalogues executed through bespoke Python loops (`case_measure`, `pipeline_review.execute_case`).
  A `harness = false` test target built on libtest-mimic would make each case a nextest test. That
  brings filterset selection, test groups (the 4 GiB observer), slow-timeout, JUnit, record/replay and
  rerun of failures for free. Measurement mode stays in criterion or a dedicated bench binary.

#### 11. `regression_experiments.py`

`bisect()` creates a detached checkout and runs `git bisect run <frozen python controller>`, with a
predicate that is **a single hard-coded test** (`TEST_SOURCE`/`TEST_NAME`). `compare()` checks that
paired measurement receipts declare identical conditions, then computes relative spreads. Port it and
generalise the predicate to a nextest filterset.

#### 12. Evidence projection: a hand-mirrored contract

- The Python `TypedDict EvidenceFact` and `EvidenceSnapshot` (`evidence_projection.py` l.21–47) mirror
  the Rust `xtask/src/development/evidence.rs::EvidenceFact` (l.263) and `EvidenceSnapshot` (l.333)
  field by field. `version: 2` and the coverage values are spelled again in Python (l.217–230;
  `evidence_sources.py` l.218–223).
- xtask **already generates** Python models for the same type (`xtask/src/codegen/private_documents.rs`
  → `scripts/generated/evidence_snapshot_input.py`). No producer imports them (grep finds zero uses
  under `scripts/*.py`). This violates AGENTS prime directive 3 ("one authoritative declaration").
- `evidence_sources.contract_assertions` parses *Python test files* with `ast` to read
  `@pytest.mark.contract_section` decorators. That part is legitimately Python-bound while the Python
  tests exist.
- `evidence_comparison_report` renders HTML and plots from the native comparison, validating against
  the generated JSON schema (`capabilities.native_schema.SchemaSet`). It is a presentation leaf and a
  reasonable KEEP-PY candidate (GUI-facing).

#### 13. Active diagnostics and traces

`ActionRequest` (pydantic) covers 12 actions. Each one resolves a pinned tool (`selected_tool`),
authenticates the target process (pid + start ticks via `process_observation`), runs inside a
`native_operation` child scope, retains bounded output and publishes `action.json`. `py-spy`,
`memray` and `python-perf` exist to diagnose *Python* processes, so their relevance shrinks with the
Python footprint. `diagnostic_trace.install` downloads, verifies and unpacks perfetto
`trace_processor` and rr into `~/.local/share/pse/tools`. `query` runs the 3 SQL files through the
`trace_processor` shell. Rust writes those perfetto traces (`pse-runtime/src/tooling/capture/timeline.rs`,
`tracing-perfetto-sdk-schema`). The `justfile` `diagnostics` recipe also routes the *Rust* diagnostics
CLI through Python (`python -c 'from pse.diagnostics import main'`).

#### 14. faer IR and numerical checks

`faer_arithmetic.source_controls` regex-checks the vendored faer source for `mul_add`, `fma` and
`core::arch` bypasses. `inspect_ir` compiles `probes/faer-nonfused.rs` with `rustc
--emit=llvm-ir` and walks the IR call graph for `llvm.fma`, `fmuladd` and `contract` flags. This is a
pure developer verification tool with no Python dependency beyond `re`, so it is a direct xtask port.
`numerical_investigation` is a 100-digit Decimal closed-form oracle plus a matplotlib plot. It is small
and independent, so keep it.

#### 15. `scientific_inputs.py`

`capture()` = `cargo run -p xtask --features physical-admission,force-validate -- scientific-inputs`.
The rest is relevance logic: `RUST_TEST_OWNERS` (3 pse-runtime prefixes, another copy of Rust test
layout), a hard-coded list of Python test files, and `affected()`. The `run -- <cmd>` subcommand used by
`just conformance` just binds the env var. A nextest setup script covers the Rust half: it is
filter-scoped, runs once per run, and exports env via `$NEXTEST_ENV`. Setup scripts are *experimental*
(`experimental = ["setup-scripts"]`; verified).

#### 16–19. Scientific references (what must stay Python)

- **Must stay Python (ORACLE, independence is the value):**
  - `idaes_nrtl_reference`, `idaes_srk_reference`: IDAES/Pyomo exist only in Python. This is the
    parity harness's sanctioned coupling (AGENTS.md).
  - `plan14_reference`, `pr_stability_reference`, `feos_entropy_reference`: teqp (C++ with Python
    bindings) plus Decimal integrals. I know of no maintained Rust teqp binding (*uncertain*). The repo
    already has an *independent Rust* oracle for PC-SAFT through FeOS (`xtask/src/feos_reference.rs`),
    so two independent oracle ecosystems exist, which is good.
  - `pfr_reference`, `cstr_controller_reference`, `decay_fit_reference` (+ `scientific_reference`
    support): scipy `solve_ivp` (Radau/LSODA/BDF) and `least_squares`. They *could* be Rust (diffsol,
    russell_ode, argmin, levenberg-marquardt) and would remain independent of product code. But scipy's
    long-standing Fortran/C integrators are the stronger "independent reference" story, the scripts are
    tiny, they run manually in isolated venvs (`build/plan53a-reference`, `build/plan14-reference`), and
    they are not on any hot path. KEEP-PY.
  - `numerical_investigation` (Decimal) and `thermodynamic_feeds` (provenance of a frozen scipy LHS
    artifact; bit-reproduction needs scipy==1.18.1): KEEP-PY.
- **Python for convenience only:**
  - `gross2001_bank`: JSON → Parquet with Decimal scaling. `feos_reference.rs` already opens the same
    file and asserts the **same SHA-256 constant**, so fold it in.
  - `thermodynamic_campaign`: drives the product through the Python API. It is product measurement, not
    an oracle.
- **Coupling smell.** `scientific_reference.frozen_number` regex-scrapes expected literals from
  authored `.pse` fixtures (for example `expect root.outlet_temperature==(…)`), and `grpc_wire_schema`
  regex-scrapes Rust. A cleaner contract: the oracles write committed JSON or Parquet, and Rust tests
  consume them (tolerance checks or `insta` snapshots, `insta` being in tree). That removes regex
  coupling in both directions.

#### 21. `grpc_wire*`

`xtask/src/codegen/grpc_wire.rs:50` runs `python scripts/grpc_wire_schema.py <root>`. The script
regex-parses `gen/rust/proto/surrealdb/protocol/**/*.rs` (prost output) and its tonic service file,
then prints Rust `match` tables that land in `crates/pse-operations/src/generated/grpc_wire_schema.rs`.
A Rust generator that parses the Rust source with Python regex is the clearest DELETE in this slice.
Use `syn` in xtask, or better, the protocol's file descriptor through prost-reflect, if
surrealdb-protocol ships one (*uncertain*). `grpc_wire.fields` is a 45-line protobuf decoder used by
`surreal_telemetry.py`, which belongs to another lane; prost replaces it once that consumer moves.

---

### 3. Rust side: what already exists

- **`.config/nextest.toml`** already uses test groups (`scientific` max 2, `store` max 16),
  filterset overrides with slow-timeouts, `default-filter = "not test(managed_primary_)"`, JUnit per
  profile with `store-failure-output`, `local`/`ci`/`flaky-hunt` profiles with retries and
  `flaky-result="fail"`, and (per its comments) nextest's record store. It does not use setup scripts,
  wrapper scripts, `archive.include` or partitioning.
- **`benches/`** (11.2K lines, criterion, `allocation-counter`): `native_process`,
  `modeling_preparation`, `document_admission` and `native_cache` already measure pools and VmHWM and
  write `*-memory.json`. `case_measure` only collects.
- **`crates/pse-testkit`** (1K lines): an engine factory for resource tests and a tracing-capture
  layer. It has **no** fixture registry today. That logic is in Python, bridged from
  `pse-operations/src/testing.rs`.
- **`crates/pse-diagnostics`** (11K lines): `SupervisedChild` (process-group ownership, bounded
  stderr, drain policy), `process_facts`, `Cancellation`, inspection and observation vocabulary. These
  are the natural home for the `native_operation.run_child_scope`/`active_diagnostics` mechanics.
- **`crates/pse-runtime/src/tooling`** (27K lines): capture (cpu, timeline/perfetto, recording,
  launch), inspection, frozen inspection, review pack/launch/analysis/export, taskdump and runtime
  metrics. `case_audit` reaches this only through PyO3.
- **xtask:**
  - `development/evidence.rs` (1,210) and `evidence_query.rs` (1,240): the SurrealDB evidence index
    and queries.
  - `evidence_tools.rs` (499): CLI.
  - `canonical_diagnostics.rs` (109), `canonical_recovery.rs` (1,090).
  - `scientific_inputs.rs` (1,047), `feos_reference.rs` (163).
  - `producer_preparation.rs`: cargo_metadata + unit graph.
  - `producer_identity.rs`: blake3 source identity.

  That covers most primitives the Python reimplements: cargo metadata, unit graph, source hashing, the
  SurrealDB client, the evidence schema and the scientific capsule.
- **`tests/*` crates** (conformance, engine, governance, lifecycle) are plain nextest targets; none
  needs the Python runner semantically. They need it only for the canonical fixture registry and the
  scientific capsule.

---

### 4. Best Rust route (with uncertainty flags)

1. **Static scheduling first (S, low risk).** Move `test_requirements.RUST_OWNERS` into
   `nextest.toml` overrides (`test-group`, `threads-required`). Add a runtime check of
   `NEXTEST_TEST_GROUP` in the Rust canonical fixture. Delete the per-run scheduling TOML and its
   one-override-per-test generation.
2. **Registry in Rust (L, high risk, ADR-0174 semantics).** Port `resource_store` + `test_resources`
   core (register, drain, freeze, finalize, reclaim, custody) into `pse-testkit` or
   `pse-operations::testing`:
   - Use the same on-disk JSON with rustix `flock` and tempfile atomic publish. Optionally redb or
     rusqlite (*a schema change, needs a migration*).
   - Use the existing gRPC `Root` client for `REMOVE DATABASE/NAMESPACE`.
   - Rust fixtures call it in-process, so `resource_bridge` and the per-fixture interpreter go away.
   - Python conftest calls it through a testing-only `pse._native` function or a tiny binary until
     Python tests shrink.
   - Do **not** collapse it into Drop + `panicking()` without a superseding ADR, since the
     architecture assigns sole outcome authority to the terminal composer.
3. **`xtask test` runner (L).** Replace `select.py`, `test_run`, `native_tests` and `test_invocation`:
   - Pass user args opaquely to nextest and read back the resolved selection with
     `cargo nextest list --message-format json`, parsed with **nextest-metadata** (`TestListSummary`,
     `BinaryListSummary`). That removes all four hand-written CLI grammars.
   - `NEXTEST_RUN_ID` correlates the list and run phases (≥0.9.138, verified).
   - Read outcomes from JUnit (quick-xml), or from the experimental libtest-json stream
     (`NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`, verified as experimental), then freeze and reclaim.
   - **Setup scripts** (experimental; verified) can build the worker binary, start or unpark the
     store, and capture the scientific capsule only when matching tests are selected. They export env
     via `NEXTEST_ENV`. As far as I know there is **no teardown hook** (*uncertain*), so reclaim stays
     in the runner.
   - **Wrapper scripts** (`run-wrapper`/`list-wrapper` per filter; verified to exist, stability
     *uncertain*) can place matching tests under the systemd observer. That removes the
     `python -m scripts.test_run --observer-child` re-entry.
4. **Cargo front door (S–M).** Make `arrow_validation` an xtask subcommand (cargo_metadata, cached).
   Make `affected` a guppy + determinator command.
5. **Assessment runner (M).** Port `validation*` + `validation_feature_campaign` + the
   `audit_tools`, `assertion_sensitivity`, `coverage_correspondence` and `boundary_fuzz` classifiers
   into `xtask assess`. Unify source identity on the xtask blake3 framed hasher, parallelised with the
   `ignore` crate. Consider `git ls-files -s` blob IDs for unchanged tracked files.
6. **Cases and measurement (M).** Make the case catalogues libtest-mimic tests. Have a Rust observer
   call pse-runtime inspection directly. Read criterion `estimates.json`. For deterministic regression
   signals, consider iai-callgrind (renamed "Gungraun" upstream; *uncertain*) or divan for
   micro-benchmarks. Criterion baselines (`--save-baseline`/`--baseline`) cover same-checkout
   comparisons, while `regression_experiments.compare` covers cross-commit pairs and can port as-is.
7. **Evidence (M).** Producers move into xtask against the existing Rust structs. Delete the Python
   TypedDicts. Optionally keep the HTML report as Python presentation.
8. **Generation (S).** Make `grpc_wire_schema` → syn or prost-reflect inside xtask.
9. **Oracles stay Python.** Change their coupling to committed data that Rust tests consume
   (tolerance or `insta`), not regex scraping.

cargo-mutants, cargo-hack, cargo-llvm-cov, cargo-fuzz, cargo-deny and cargo-audit all stay as
external tools. Only their Python wrappers go.

---

### 5. Cross-cutting observations

**Duplicated sources of truth**

1. **Rust test layout mirrored in Python:** `test_requirements.RUST_OWNERS/RUST_FIXTURES/RUST_PEERS`,
   `scientific_inputs.RUST_TEST_OWNERS`, `regression_experiments.TEST_NAME`,
   `highs_lifecycle_diagnostic.NORMAL`, and the pse-backend-native module set (also in
   `nextest.toml`). Renaming a Rust test module silently changes scheduling and effect guards.
   `rust_catalog` catches only missing fixture declarations.
2. **`EvidenceFact`/`EvidenceSnapshot`:** hand TypedDict against the Rust struct, while generated
   Python models sit unused.
3. **Cargo, nextest and pytest CLI grammar:** four hand-maintained flag tables.
4. **Unit-graph and cargo-metadata decoding:** `arrow_validation` (msgspec structs) and
   `xtask/producer_preparation.rs` (cargo_metadata + serde).
5. **Source identity:** Python `sha256(mode:bytes)` per file (`validation.sources`) and xtask blake3
   framed identities.
6. **SurrealDB transport:** Python urllib HTTP `/sql` for fixture cleanup, Rust gRPC for everything
   else.
7. **Frozen artifact checks:** gross2001 SHA constant in `gross2001_bank.py` and `feos_reference.rs`.
8. **Protobuf:** `grpc_wire.py` decoder against prost. Rust prost output is parsed by Python regex to
   generate Rust.
9. **Gate aggregates:** `validation_scope.GROUPS` mirrors justfile aggregate recipes (`ci-fast`,
   `hygiene`, …).

**Recurring patterns**

- Every tool follows the same shape: shell out to cargo or a cargo tool, capture JSON or JUnit, re-validate its shape
  defensively in Python (`_invalid_protocol`), hash everything, write a receipt (`schema: …-v1`) and
  print a path. About a dozen receipt schemas live only in Python TypedDicts or dict literals.
- **Lifecycle and signal plumbing** (`deadline`, `deadline_supplier`, `pthread_sigmask`, freeze and
  resume) is the main complexity driver of groups 1, 2 and 7. It is also the part with the most to
  gain from Rust (`tokio::time`, `Cancellation`/`SupervisedChild` already in pse-diagnostics).
- **Self-authentication everywhere:** binaries are sha256'd at preparation and again at execution,
  and the source tree is hashed before and after. That is defensible as policy, but it is done in
  Python, serially.

**Performance-relevant findings**

- *Measured (sandbox):* about 0.1–0.15 s Python import per `test_resources` bridge call. Rust canonical
  fixtures make at least 2 per test. This is CPU inside the test's own slow-timeout budget.
- *Proposed / not measured:* each `just` cargo recipe adds `cargo metadata --all-features` plus 1–2
  `--unit-graph` resolutions. Each Rust test run adds 2 extra `nextest list` passes, a full rust-product
  source hash taken twice, and a sha256 of every selected test binary. The JUnit round trip
  (nextest writes XML, Python parses it with ElementTree) sits on the terminal path.
- Process nesting for a native Rust run is 5–6 interpreters and wrappers deep (§2). The Python layers
  hold signal handlers and deadlines, so failures surface as composer errors rather than nextest
  output.

---

### 6. `scripts/tests` breakdown (whole tree)

These are filename heuristics over the 247 top-level `.py` files (≈125.9K lines), so treat the
cluster boundaries as approximate:

| Cluster | Files | Lines |
|---|---|---|
| SurrealDB service, canonical store, surrealkit, backup and recovery, native-read | 53 | ≈34.6K (`test_surreal_server.py` alone is 12.1K) |
| Capability/MCP/agent/workspace tooling, plan52 acceptance | 62 | ≈29.3K |
| Host admission, native operation, pse-env, build, plan30 placement | 40 | ≈23.4K (`test_native_operation` 4.9K, `test_host_admission` 3.2K) |
| **This slice: test orchestration, fixtures, validation** | 30 | ≈14.8K (`test_test_resources` 3.2K, `test_validation` 2.1K, `test_native_tests` 1.9K, `test_test_run` 1.2K) |
| Telemetry/OTel | 24 | ≈10.6K |
| **This slice: evidence, diagnostics, measurement, case** | 23 | ≈7.7K (`test_case_audit` 1.3K, `test_regression_experiments` 0.6K, `test_evidence_sources` 0.6K) |
| Python extension, producer, codegen | 6 | ≈3.7K |
| **This slice: scientific references and inputs** | 4–6 | ≈0.8K (+0.6K `test_scientific_inputs`/consumers) |
| Docs/ADR/governance | 4 | ≈0.7K |

The tests directly matching modules in this slice total **≈20.8K lines**. Almost all of them test the
lifecycle and receipt machinery rather than science: races, interruption, freeze and resume, custody
and authentication. They would be replaced, not ported, by Rust tests of the Rust registry and runner,
plus nextest's own guarantees. Only `test_scientific_references.py` (87 lines) protects oracle
behaviour.

---

## E1 — FastMCP capability interface and its generated clients

Slice: `scripts/capabilities/**` (64 modules, 16,398 lines), `scripts/capability_cli.py` +
`scripts/capability_cli_runtime.py` (526), `scripts/generated/**/*.py` (4,021 lines, plus about 9K
lines of generated JSON schema/metadata). Snapshot HEAD bd1c4891e, read-only. Method: I read the
code paths. **Nothing was executed.** The snapshot has no `.venv`, so every latency statement
below is a count of hops and transcodings, not a measurement.

### 0. Headline findings

1. **Rust declares about 23% of the callable surface.** The server exposes 96 callable tools, 12
   bootstrap/content tools, 8 workflow prompts, 52 resources and 49 non-callable "native
   route" descriptions. Of the 96 callable tools, 22 come from Rust declarations
   (`crates/pse-runtime/src/host_operations.rs::operations()` contributes 21, and
   `xtask/src/development/host_operations.rs::declaration()` contributes `evidence_compare`). One
   more, `native_contract_schema`, reads Rust-generated schemas. Two (`knowledge_query`/`_explain`)
   end in the Rust `pse-evidence` binary. The other ~71 are handwritten FastMCP bindings over
   *Python* owners in `scripts/*`: 22 proxy other MCP servers (mcpls/LSP and SurrealDB `/mcp`),
   and the rest cover validation, telemetry, workspace, test archives and active diagnostics.
2. **Python reaches native code through subprocess workers, not an in-process import.** The server
   never imports `pse._native`. For each native operation it launches
   `scripts/pse-env [--native=…] -- .venv/bin/python -m scripts.capabilities.<worker>`. That is
   bash, then `pse_env.py` (1,424 lines, which may wrap the command in `systemd-run`), then a worker
   interpreter that imports `pse`.
   - 6 "Dedicated" readers pay this on every call: `project_bundle_inspect`, `diagnostic_*`×3,
     `evidence_inspect` and `invocation_read`.
   - 14 "ExactContext" operations go through a retained child-process pool. It is keyed by a hash
     of resources+physical+documents and speaks length-prefixed JSON frames
     (`native_read_sessions.py`, `native_read_worker.py`, `native_read_protocol.py`). The pool
     exists to keep a `pse.Runtime` and admitted `PhysicalContext`/`ModelingPackage` objects alive
     between calls.
   - `evidence_compare` takes four hops: MCP server → Python worker → temp files → `pse-evidence`
     Rust binary → `EvidenceAnalysis::compare`.
3. **Each native call goes through about 5 JSON encode/decode passes per direction.** The path is
   MCP JSON → pydantic header → `json.dumps` → frame or stdin → pydantic again → `json.dumps` per
   document → msgspec decode → `codec.encode_json` → serde decode in PyO3 → Rust. The result comes
   back the same way. `result_read` turns Arrow into pyarrow IPC, then base64 (+33%), then
   8 KiB string chunks inside JSON.
4. **Dispatch is written twice.** Rust declares `receiver: "pse.ModelingPackage.inspect"` and
   similar strings, but those strings are only displayed. Actual dispatch is a hand-written 21-way
   `if/elif` in `native_worker.py::execute` (lines 361–858). Every receiver exists as a Rust
   function in `pse-runtime` (`workflow/modeling/*`, `workflow/projects.rs`,
   `tooling/inspection.rs`, `tooling/frozen_inspection.rs`, `observation.rs`).
5. **About 3.2K lines exist only to keep a multi-process, hot-editable Python host coherent.**
   This covers loaded-source fencing by AST import walk, admission hand-off files carrying a
   request sha256, schema-set fingerprint checks, compiled-catalogue mismatch checks, extension
   generation/currency/image attachment, and drain/successor logic. The Python code has 13
   `stale_interface` refusal paths. All of this disappears when the server is one Rust binary
   linking `pse-runtime`.
6. **The Rust declaration carries Python-host fields.** `prepare_import`, `ExtensionRequirement`,
   `WorkerFamily`, `ReadEligibility::ExactContext` and `InvocationLifetime` all describe how the
   Python host spawns and retains workers. The header structs (`Source`, `Preparation`, …) derive
   only `Serialize + JsonSchema` with `serde_json::Value` document fields. They are schema-only
   declarations for a pydantic consumer, and Rust never decodes them.
7. **Two Rust operation catalogues are not unified, and MCP serves only the smaller one.**
   - `host_operations::project::ProjectAction` has 49 typed actions, each with schemars and
     `EffectFacts`. The clap binary `pse-project` already dispatches them in Rust.
   - The MCP catalogue only *describes* these 49 as non-callable "native routes"
     (`inventory.py:357`).
8. **Generated Python with no consumers.** Seven `scripts/generated` modules (1,292 lines:
   `evidence_comparison_input`, `evidence_comparison_request_output`, `evidence_snapshot_input`,
   `evidence_snapshot_output`, `knowledge_explain_output`, `knowledge_page_output`,
   `knowledge_query_input`) are imported only by the generation self-check
   `scripts/check_private_contracts.py`. Two more are imported only by tests.
   `host_operation_headers.json` (2,525 lines) has no reader.
9. **Tests outweigh the code.** 54 test/acceptance files under `scripts/tests/` total 24,580
   lines. About 6.4K of those lines test the native subprocess/session path alone.
10. **Prior decisions declined rmcp, under a different driver.** Plan 52p (§"bounded RMCP
    comparison"), Plan 55 ("Do not add another … rmcp/gRPC boundary") and the 2026-10-25 functional
    review all retained FastMCP. 52p's revisit trigger is: *"when an actual resident Rust service
    needs agent-facing control and can retire a complete current adapter path"*. The native adapter
    path below is such a complete path. The owner's new thesis changes the driver that decision was
    weighed against.

### 1. Summary table

| Module/group | LOC | What it does (1–2 lines) | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---|---|---|---|---|---|---|
| **Bootstrap/catalogue/presentation**: `server.py`, `catalogue.py`, `presentation.py`, `model.py`, `inventory.py`, `operations.py`, `cleanup.py`, `__init__/__main__` | 2,135 | FastMCP server composition; 8 bootstrap tools (domains/workflows/describe/select/reset/invoke_read/invoke_action/drain); per-session visibility (progressive reveal); prompts/resources-as-tools. `inventory.py` (1,028) is a hand-written table of 14 domains, 8 workflows, families, guides and native-route descriptions | NEW-IFACE (+NEW-DEVTOOL data) | Only the 21 operation declarations; no MCP | PORT-LIB (rmcp `ServerHandler` with custom `list_tools` + `notify_tool_list_changed`); inventory becomes a static Rust table or TOML | rmcp, schemars, serde | M |
| **Guidance resources** `guidance.py` | 698 | Serves AGENTS/rules/skill files as MCP resources; bounded snapshot capture (128 KiB/file, 64 files, 1 MiB/pkg), identity re-check, supporting-file manifests | NEW-DEVTOOL / NEW-IFACE | — | PORT-LIB (rmcp resources/templates) | rmcp, ignore (present), blake3/sha2 (present) | S–M |
| **Loaded-source fencing** `coherence.py`, `source_basis.py`, `source_modules.py`, `execution_basis.py` | 534 | AST-walks serving sources and their lazy imports to capture a "loaded revision"; writes a per-call `admission.json` hand-off (checkout, route revision, request sha256) that the worker re-verifies | NEW-CONTROL, exists only because the server is live-edited Python | — | **DELETE** (the compiled binary is the revision; embed build identity) | build.rs/`vergen` or `built` (exist; optional) | S |
| **Interface observation and diagnostics** `observation.py`, `input_diagnostics.py`, `diagnostics.py`, `responses.py` | 839 | `_meta["pse.observation"]` correlation, redacted structured stderr, OTel span attributes, safe input-error paths, 120 KiB / 1 MiB JSON bounds | NEW-OBS | tracing + tracing-opentelemetry in workspace; miette diagnostics with §23.2 codes | PORT-EXISTING | tracing, tracing-opentelemetry, serde_path_to_error (exists) | S–M |
| **Native adapter** `native.py`, `native_contracts.py`, `native_schema.py`, `native_worker.py` | 2,000 | Builds 21 `NativeTool`s from generated declarations; composes input schemas by embedding Rust-generated JSON-schema files; worker-side hand-written 21-way dispatch to `pse` facade; error-report redaction plus a failure-class→remedy table; Arrow→base64 copy | WRAP (primary), **DUP** (dispatch, remedy table, defaults resolution, schema envelope) | `pse_runtime::host_operations::{operations, header_schemas}`; receivers in `pse-runtime` `workflow/*`, `tooling/*`; `DiagnosticReport`; `ProductionProfiles` | **DELETE**: rmcp tool per `OperationDeclaration`, typed serde decode, direct receiver call | rmcp, schemars, serde_path_to_error, arrow-ipc (in family), base64 (present) | M |
| **Retained native read sessions** `native_read_sessions.py`, `native_read_worker.py`, `native_read_protocol.py`, `native_read_context.py`, `native_read_image.py`, `native_extension.py` | 1,274 | Child-process pool keyed by context digest; 4-byte length-prefixed pydantic JSON frames; per-request uuid/sha256 correlation; cursor→issuing-worker maps; retire/successor/drain; extension-generation attachment | ORCH + NEW-CONTROL | `Runtime`, `SharedRuntime`, `PhysicalContext`, `ModelingPackage` (the objects being retained) | **DELETE**: in-process `HashMap`/`quick_cache` of `Arc<Runtime>`+contexts keyed by the same digest; cursor map; `CancellationToken` drain | tokio, tokio-util (present), quick_cache (present) | M |
| **Fixed worker transport** `workers.py`, `process.py`, `worker_progress.py` | 538 | 9 worker routes → `pse-env [--native] -- python -m …`; request/response temp files; `progress.json` polling; terminate on cancel | ORCH | `pse_env` placement is Python (other slice) | Native routes DELETE; external routes PORT-EXISTING (`tokio::process`) | tokio | S–M |
| **Readiness and extension preparation** `readiness.py`, `preparation.py`, `preparation_worker.py` | 888 | Passive/explicit checks of Python-extension receipts, import compatibility, currency; maturin stage actions via workers | NEW-OBS / ORCH over `scripts/python_extension.py` | — | **DELETE** for the MCP path (no extension when the server links Rust); extension prep stays as `just py-sync*` (KEEP-PY in its own slice) | — | S |
| **Specialists** `specialists.py` | 1,491 | MCP-client proxy to mcpls (rust-analyzer/pyrefly LSP bridge, 18 `navigation_*`) and SurrealDB's `/mcp` (4 `database_*`); file-stamp identity fences; edit-preview retention (32 previews / 64 MiB) over `semantic_edits` | NEW-IFACE (proxy) + NEW-STORE (preview handles) | — (SurrealDB `/mcp` is SurrealDB's) | PORT-LIB (rmcp client, child-process + streamable-HTTP transports), or drop the proxy and attach mcpls/Surreal directly; previews KEEP-PY until `semantic_edits` ports | rmcp (`client`, `transport-child-process`, `transport-streamable-http-client`) | M |
| **Host read bindings** `read_contracts.py` (62 pydantic classes), `host_queries.py`, `environment/workload/document/coverage/knowledge_queries.py`, `rust_references.py`, `context/concept/producer/entrypoint/schema_bindings.py` | 2,224 | Thin `Tool.from_function` wrappers plus hand-written answer models over Python owners (`workspace_*`, `run_inspection`, `validation_result`, `activity_watch`, `knowledge_store`, …) | NEW-IFACE (thin); answer models partly DUP of owner JSON | Knowledge query/explain: `xtask::development::knowledge` via `pse-evidence`; `native_contract_schema`: generated catalogue | Follows owners: PORT-EXISTING as each owner moves to xtask; KEEP-PY meanwhile | serde, schemars | M–L (coupled) |
| **Integration view** `integration_view.py` | 775 | Ownership-aware change preview combining `affected`, `workspace_sources`, `document_lifecycle`, `test_invocation`, `workspace_tasks` | NEW-DEVTOOL | — | Follows owners (PORT-EXISTING with them) | — | M |
| **Entry-point inventory** `entrypoints.py` | 383 | Runs `just --dump --dump-format json` (10 s/4 MiB bound), classifies recipes into families, resolves `python -m` targets by source | NEW-DEVTOOL | — | PORT-EXISTING (xtask) | serde_json, ignore | S |
| **Actions** `host_actions.py`, `host_worker.py`, `qualification.py`, `qualification_worker.py`, `qualification_contracts.py`, `rust_tests.py`, `rust_test_contracts.py` | 1,138 | Spawn workers that call Python `validation.run_gates`, `test_run`, `test_archives`, `active_diagnostics`, which drive cargo/nextest | ORCH | xtask owns some runners (other slices) | Follows owners; PORT-EXISTING when the validation runner moves to xtask | tokio::process, nextest-metadata (exists; other slice) | M–L |
| **Evidence worker** `evidence_worker.py` | 141 | Writes request/snapshots to temp files, runs `pse-evidence evidence compare-offline`, re-reads stdout | ORCH/WRAP | `xtask::development::evidence_query::EvidenceAnalysis::compare` (the declared receiver) | **DELETE** (in-process call) | — | S |
| **Telemetry tools** `telemetry_queries.py`, `telemetry_contracts.py`, `telemetry_profiles.py`, `telemetry_profile_worker.py` | 774 | Bindings over Python `telemetry_*` (OpenObserve queries, config, watches) + profile capture through `pse` native worker. `telemetry_contracts.SignalSettings` is a hand-redacted mirror of Rust `tooling::telemetry::SignalSettings` | NEW-OBS (bindings); minor DUP | `pse_runtime::tooling::telemetry::{Settings, SignalSettings}`; capture owner native | Capture DELETE (in-process); queries follow owners | reqwest (present) | M |
| **Reference + CLI codegen** `reference.py`, `cli_codegen.py` | 566 | Builds the live FastMCP server, then renders `docs/generated/capabilities/reference.md` (91.5K lines) and the Cyclopts CLI from FastMCP tool schemas | GEN (Python→Python) | — | **DELETE**: reference rendered by Rust from the same catalogue; CLI via clap runtime builder | clap (present) | S |
| **CLI client** `capability_cli.py`, `capability_cli_runtime.py` | 526 | Cyclopts CLI that spawns a fresh STDIO MCP server per invocation, calls one tool, then calls `capabilities_drain`; JSONL session; Codex reconnect | NEW-IFACE | `pse-project`, `pse-evidence` are clap CLIs over the same Rust owners | PORT-LIB (clap; in-process call, rmcp client only for endpoint-bound sessions) | clap, rmcp client | S–M |
| `scripts/generated/capabilities/cli.py` | 1,204 | 96 generated Cyclopts commands + `OPERATIONS` wire-revision table | GEN (from Python FastMCP schemas) | — | **DELETE** | — | S |
| `scripts/generated/native_operations.py` (+`host_operation_headers.json` 2,525) | 320 | Pydantic headers + declaration table generated by `xtask/src/codegen/host_operations.rs` | GEN | `host_operations.rs` | **DELETE** once the server is Rust | — | S |
| `scripts/generated/deployment.py`, `telemetry_settings.py` (+ `deployment_schema/*.json`) | 572 | Generated msgspec/pydantic mirrors used by `surreal_server`, `execution_profiles`, `reference_run`, `telemetry_config`, `telemetry_process` | GEN | `pse_runtime::workflow::ProductionProfiles`, deployment documents, `tooling::telemetry` | DELETE as those consumers port (other slices) | — | S |
| `scripts/generated/evidence_*`, `knowledge_*` (+ `evidence_schema/*.json`) | 1,925 | Generated private evidence/knowledge contracts | GEN | `xtask/src/codegen/private_documents.rs` slugs; xtask evidence/knowledge owners | **DELETE now** for the 7 unconsumed modules (1,292 lines); JSON schemas stay only while `SchemaSet(family="evidence")` / the report validator consume them | — | S |

### 2. Per-group notes and evidence

#### 2.1 Server composition and progressive reveal (`server.py::build`)
- `bindings()` composes 9 factories plus `readiness.bindings`. Each tool is `add_tool`-ed and then
  `server.disable(keys=…)`. `capabilities_select` replaces a single FastMCP visibility rule
  (`save_visibility_rules`), and the `invoke_read`/`invoke_action` fallbacks re-enter
  `server.call_tool`. These are the FastMCP-specific features that rmcp would need re-implemented:
  - per-session visibility,
  - `PromptsAsTools`/`ResourcesAsTools` (`presentation.py`),
  - middleware (`InputDiagnostics`, `observation.install`),
  - `strict_input_validation`.

  With STDIO there is exactly one session per process. Per-session visibility then reduces to a
  `Mutex<HashSet<ToolId>>` plus `peer.notify_tool_list_changed()`. The two invoke fallbacks are a
  ~20-line match.
- `inventory.py` is data: `RETENTION` prose, `_families()` (≈400 lines of `Capability(...)`
  literals), domains, workflows and guides. It does not depend on Python.

#### 2.2 Native adapter: what actually executes
- **Schema.** `native.py::bindings` → `SchemaSet(root).envelope(host_schema, documents, required)`
  loads `docs/generated/schema/catalogue.json`, reads each referenced `*.schema.json`
  (re-checking stat identity and sha256 on every read, `native_schema.py::_captured_source`), and
  splices them into the pydantic header schema under `urn:pse:boundary:<fingerprint>:<slug>:input`.
  In Rust, `schemars::schema_for!(Source<ProjectDraftDocuments>)` produces the same thing at
  compile time if the header structs become generic over their document types and derive
  `Deserialize`.
- **Call.** `NativeTool.run` runs `request.model_validate(arguments, strict=True)`; the header
  validates but document fields are opaque `JsonValue`. It then adds `schema_set_fingerprint` and
  routes to one of three places:
  - `ReadSessions.run` for `exact_context` operations,
  - `run_worker` for `prepare_import` actions,
  - `run_child` (sync `subprocess.run` in a thread) for dedicated readers.
- **Worker** (`native_worker.py::execute`). The worker:
  1. re-validates the header;
  2. checks extension currency and the image (`extension_currency`, `LoadedReadImage.admit`);
  3. compares `pse.SCHEMA_SET_FINGERPRINT`;
  4. compares `codec.encode_json(pse.host_operation_catalogue())` with the generated
     `NATIVE_CATALOGUE`;
  5. activates telemetry;
  6. runs the 21-way `if/elif` decode/dispatch;
  7. builds `pse.EngineSettings` and `pse.Runtime(...)`;
  8. calls the facade method;
  9. returns `codec.document_builtins(observed)`.

  Each of steps 2–4 guards against Python and Rust disagreeing at runtime.
- **DUP: error projection.** `scientific_error()` and `failure_remedy()` (lines 138–344) walk
  `DiagnosticReport` attributes, hash cause messages, redact map keys against schema properties,
  and map Rust failure classes (`runtime.cancelled`, `solve.evaluation_error`,
  `compile.math`, …) to remedy prose. That is a second, host-side interpretation of the §23.2
  taxonomy. In Rust, it belongs on the error type (`miette::Diagnostic::help`) or in one
  redaction function next to `DiagnosticReport`.
- **DUP: defaults.** Rust declares `x-pse-host-default = "production.capabilities.memory_limit_bytes"`.
  The generated pydantic `Resources` resolves that default through
  `scripts/execution_profiles.py::declarations()`, which parses `.config/agent-capacity.toml` with
  `tomllib` + msgspec into a generated mirror of Rust `ProductionProfiles`.
- **`result_read`** (`read_result`) copies batches into a pyarrow IPC stream, base64-encodes it,
  and splits it into 8 KiB strings. The Rust equivalent is `arrow_ipc::writer::StreamWriter` +
  base64 with no Python copy, or an MCP blob/resource content.

#### 2.3 Retained sessions: the latency and duplicated-state hotspot
- `context_identity()` hashes `{resources, physical, documents, route, prerequisites}`.
  `ReadSessions._run` reuses an idle `ReadSession` with that identity, or **spawns a new
  process**. Any document change means a new worker, a new interpreter, a new `import pse` and a
  new `Runtime`.
- `ReadSession.invoke` writes `encoded(ReadRequest)`, which embeds the full `Handoff` admission
  (definition, route revision, request sha256), and awaits `receive(ReadResponse)`.
  `require_request` recomputes the request sha256. Cursor ownership is tracked in `_cursors`, so
  continuation pages return to the worker that issued them.
- `native_read_context.NativeReadContext` holds `runtime`, `_physical` and `_modeling` dicts keyed
  by source sha256. This is an object cache, kept in a child process because the parent must not
  import the extension.
- In a Rust server the whole group becomes a
  `Mutex<HashMap<ContextDigest, Arc<ContextState>>>` with `Runtime`, physical/modeling maps and a
  cursor→context map. The digest computation (`_source_identity`) is the same. "Drain" becomes
  awaiting outstanding tasks plus `Runtime::close`.

#### 2.4 Loaded-source fencing and readiness: Python-only complexity
- `coherence.Interface` captures wire digests of all tools (`execution_basis.wire`) and source
  digests (`source_basis.Sources`, which uses `ast.walk` to follow lazy imports). `execution_basis`
  publishes a 0600 `admission.json` for every worker call, and the worker calls `verify()`.
- `readiness.py` and `native_extension.py` decide whether the *Python extension* is built,
  current and importable.
- None of this has a counterpart in a compiled server. Revision becomes the build identity, and
  "the extension is current" becomes "the binary links `pse-runtime`".

#### 2.5 Specialists: MCP-in-MCP proxy
- `Connection` runs one long-lived `fastmcp.Client` task per backend. `language()` re-stamps the
  mcpls/rust-analyzer/pyrefly binaries and config, and hashes the source file before and after each
  call. `database_call()` rechecks the SurrealDB generation fence before connecting, before each
  call and after it.
- The value-add over attaching mcpls and SurrealDB MCP directly is the source-cut digests, the
  generation fence and the bootstrap reveal. That logic is portable. rmcp's client role plus
  `TokioChildProcess`/streamable-HTTP transports cover the protocol side. *(mcpls is assumed to be
  an external binary; I did not verify its provenance.)*

#### 2.6 Host reads, actions, telemetry: thin bindings over other slices
- For example, `workload_queries.product_run_read` → `run_inspection.product_result`;
  `assessment_read` → `validation_result.read_assessment`; `qualification_worker.execute` →
  `validation.run_gates` (Python gate runner) → cargo/nextest.
- These modules are small. Their disposition is decided by the owners' slices, not by the MCP
  framework. `read_contracts.py` (62 frozen pydantic answer classes) re-types owner outputs; it
  goes away with the owners or turns into serde structs.

#### 2.7 CLI generation
- `cli_codegen.render` builds the whole server, then turns each FastMCP tool's JSON schema into a
  Cyclopts command. Scalar properties become flags, and anything else becomes a JSON string.
- At runtime, every `just capability call X` starts:
  - a Cyclopts interpreter,
  - a server interpreter (`StdioTransport(.venv/bin/python -m scripts.capabilities)`) that builds
    the whole catalogue,
  - the tool's worker,
  - then `capabilities_drain` and teardown.
- Rust already has two clap CLIs over the same owners: `pse-project`, which takes a typed
  `--request` for 49 `ProjectAction`s, and `pse-evidence`.

#### 2.8 Generated Python
- `native_operations.py` comes from `xtask/src/codegen/host_operations.rs::python` and is consumed
  by `native_contracts.py` and one test.
- `evidence_*`/`knowledge_*` come from `xtask/src/codegen/private_documents.rs` slugs.
  `deployment.py`/`telemetry_settings.py` are consumed by Python ops scripts in other slices.
- The unconsumed modules are listed in §0.8. They are loaded only by
  `scripts/check_private_contracts.py`, which checks that the generator's output imports.

#### 2.9 Tests (scripts/tests)
- 54 files, 24,580 lines: `test_capabilit*` (36 files), `mcp_*_acceptance`/assessment (5 files,
  4,671), `native_read_*`, `capability_*_inquiry` and `capability_codex_check`, `test_codex_*`,
  `test_host_operation_generics`, `test_owner_boundary_contracts`.
- The native subprocess/session path alone is about 6,454 lines: `test_capability_native` 1,292,
  `test_native_read_native` 870, `test_native_read_sessions` 636, `native_read_adverse` 615,
  `mcp_native_acceptance` 2,092, plus execution-basis, coherence, context, image and adverse
  helpers.
- Most of these lines test cross-process lifecycle: drain, retired cursors, stale interfaces and
  cancellation shielding. A single-process Rust server makes those scenarios structurally
  impossible.
- Not counted: `test_surreal_mcp` 1,015, `test_semantic_*` 394, evidence/knowledge store tests.

### 3. Answers to the key questions

#### Q1. What is exposed, how much is generated, and where are the parallel contracts?
- **Exposed:**
  - 96 callable tools, plus 12 bootstrap tools: 8 `capabilities_*` and 4 resource/prompt
    adapters;
  - 52 resources (guidance plus `pse://telemetry/schema`) and 2 templates (`pse://capabilities/{id}`,
    `pse://telemetry/selection/…`);
  - 8 `workflow_*` prompts;
  - 49 non-callable "native" family descriptions.
- **Generated from Rust:** 22 tool definitions (name, title, description, effect annotations,
  input header and documents, output document schema). Their *dispatch* is hand-written.
  Everything else is hand-written Python.
- **Parallel contracts:**
  - `read_contracts.py` (62 classes), `telemetry_contracts.py` (≈15),
    `qualification_contracts.py`, `rust_test_contracts.py`, `native_read_protocol.py`
    frame models, `presentation.SelectionReceipt`, `readiness.*`, and `model.Capability`, which
    is the exposure schema;
  - the generated pydantic mirrors of Rust headers (`native_operations.py`) and msgspec mirrors
    (`deployment.py`, `telemetry_settings.py`).
- **Direct mirrors of Rust types:**
  - the header models (generated);
  - `telemetry_contracts.SignalSettings` (hand-redacted);
  - the failure-class remedy table;
  - resolution of `ProductionProfiles` defaults.

#### Q2. How are workers implemented?
Workers are fixed Python modules launched through `scripts/pse-env` → `pse_env.py` (possibly
`systemd-run --scope`) → `.venv/bin/python -m <module>`. There are 9 routes in
`workers.WORKERS`. Transport is one of:
- JSON on stdin/stdout (`run_child`),
- request/response files plus a `progress.json` poll (`process.run_worker`),
- length-prefixed JSON frames over pipes (retained sessions).

Workers import `pse` and the facade, which decodes into msgspec and calls PyO3. State that is
duplicated or re-derived across the boundary:
- extension-generation identity,
- schema-set fingerprint,
- compiled operation catalogue,
- loaded-source revision,
- per-request admission hand-off,
- `Runtime`/context caches held in children.

Per-call overhead is spread across several places:
- **Dedicated readers** pay at least two interpreter starts and the extension import on every
  call.
- **ExactContext operations** pay the same on every new context.
- **CLI calls** pay two more interpreters, a full catalogue build and a drain.
- **Every call** pays about 10 JSON passes plus 2–3 sha256 passes over the request bytes.

#### Q3. Can rmcp replace it, and what stays Python?
**Yes for the native and evidence/knowledge path. Mostly yes overall, with the remainder coupled
to other slices.**

Suggested shape: one Rust binary (a new crate such as `pse-mcp`, or an xtask bin; xtask already
depends on `pse-runtime`, tokio and clap) that:
1. Makes `host_operations` headers generic over their real document types and derives
   `Deserialize`. One serde decode then replaces pydantic → msgspec → serde, and `schemars`
   replaces `native_schema.envelope`.
2. Registers one rmcp tool per `OperationDeclaration`. `EffectFacts` → `ToolAnnotations`;
   `description`/`effects` → tool description; output schema from `output_document`. Dispatch
   goes through a `match` on the operation, or a `receiver: fn(&State, Request) -> Result<Value>`
   field that replaces the `&'static str`. Add the 49 `ProjectAction`s through the same registry,
   which removes the second catalogue's MCP blind spot.
3. Holds `Arc<Runtime>` + physical/modeling contexts in-process, keyed by the existing context
   digest. Uses `tokio_util::CancellationToken` for cancel and drain, and rmcp progress
   notifications instead of `progress.json` polling.
4. Calls `EvidenceAnalysis::compare` and the knowledge query owner directly.
5. Serves guidance files as resources, implements reveal as a mutable visible set plus
   `notify_tool_list_changed`, and keeps `invoke_read`/`invoke_action` as explicit tools.
6. Builds the CLI with **clap's builder API at runtime** from the same static catalogue: scalar
   header fields become flags, document fields become `--<field>-json/-file`. This follows the
   `pse-project --request` precedent. No generated Python file is needed, and the CLI calls
   handlers in-process instead of spawning an MCP server. An rmcp client remains only for
   endpoint-bound edit-preview sessions.

**What genuinely remains Python, or is coupled to other slices:**
- About 71 tools wrapping Python owners (validation runner, test archives, active diagnostics,
  telemetry backend queries, workspace/doc lifecycle, semantic edits/structural migrations). These
  move only when their owners move. Interim option: the Rust server proxies a slimmed FastMCP
  child as an rmcp client, the same way the Python server already proxies mcpls and SurrealDB.
  That gives a single endpoint from day one and lets tools migrate one at a time.
- Python-extension preparation (maturin, stubs, receipts) remains a Python-user concern, but it
  stops being an MCP prerequisite.
- Nothing in this slice is an ORACLE, and nothing is inherently tied to the Python ecosystem. The
  only Python-flavoured dependency is pyrefly, an external binary behind mcpls, for Python
  navigation.

**Governance:** adding a crate and changing the Python boundary contract need an ADR plus a design
review (`.claude/rules/decisions.md`). That ADR should supersede the 52p/55 rmcp disposition.

**Uncertainty:**
- rmcp 3.7.0 and its feature names come from the owner's `docs/libraries/rmcp.md`; I did not
  independently confirm the version.
- `serde_path_to_error`, `jsonschema` (Rust), `vergen`/`built` and `quick_cache` exist, to my
  knowledge.

### 4. Cross-cutting observations

1. **Duplicated sources of truth:**
   - operation dispatch (Rust `receiver` strings vs `native_worker.execute`);
   - header schemas (Rust schema-only structs → generated pydantic → re-validated in parent and
     child);
   - defaults (Rust `x-pse-host-default` names → Python TOML parse of `ProductionProfiles`);
   - the error taxonomy (Rust §23.2 codes → Python remedy table + 77 host refusal codes
     `pse.*.*`);
   - telemetry `SignalSettings` (Rust → generated msgspec → hand-redacted pydantic);
   - the capability catalogue (Rust 21 + xtask 1 + Python inventory 145 entries; `ProjectAction`
     49 described in Python prose);
   - CLI grammar (FastMCP schema → generated Cyclopts, separate from the clap `pse-project` CLI).
2. **Recurring pattern:** hand-written Python fixes up Rust/Python disagreement at runtime.
   Fingerprints, catalogue equality checks, image attachment and stale-interface refusals all exist
   because two independently built artifacts must agree. A single Rust binary turns them into
   compile-time facts.
3. **Recurring pattern:** each tool shells out to `pse-env → python -m worker`, or to a Rust binary
   via temp files (`pse-evidence`), and parses JSON from stdout or a response file. The MCP
   process spends most of its code on process custody (shield/settle/terminate/drain), not on
   operations.
4. **Reverse coupling in Rust.** `OperationDeclaration` fields `prepare_import`, `extension`,
   `worker`, `read_eligibility`, `invocation_lifetime` and `public_binding`, along with
   `receiver: &'static str` naming Python paths, encode the Python host's process model in the Rust
   source of truth. Under a Rust server, most of these become a plain cache policy or go away.
5. **Performance-relevant (unmeasured):**
   - interpreter start plus extension import on every dedicated read, every new read context and
     every CLI call;
   - about 5 transcodings per direction per call;
   - `json.dumps` of whole payloads repeated for size bounds (`intact_result`, `run_child`,
     `bounded`, knowledge 120 KiB check);
   - base64-in-JSON Arrow transport;
   - server startup that builds 96 tools, embeds full schema documents, and AST-scans serving
     sources. The Codex attachment allows 30 s.
6. **Documentation load.** `docs/dev/capability-interface.md` (688 lines) mostly describes
   lifecycle obligations created by the multi-process topology: drain, reattach, stale cursors,
   loaded-source revisions, generation attachment and reconnect. A Rust server would cut most of
   it.
7. **Lowest-risk immediate cuts:**
   - remove the 7 unconsumed generated modules (generator change in
     `xtask/src/codegen/private_documents.rs`) and `host_operation_headers.json`;
   - route `native_evidence_compare` and `knowledge_*` directly to Rust (they already end in
     Rust).

---

## E2: Telemetry cluster and workspace/agent/doc devtools (scripts/)

Slice: 37 files, **14,215 LOC** (telemetry 4,767; workspace/agent/doc/devtool 9,448).
Snapshot: /home/claude/pse (HEAD bd1c4891e), read-only. Matching tests: **15,651 LOC** directly matched (telemetry 11,475; devtools 4,176), plus partial coverage in `test_setup.py`/`test_python_contract_annotations.py` (see section 4).

Answers to the two key questions:

- **Telemetry.** On the Rust side, `pse-runtime` (`crates/pse-runtime/src/tooling/telemetry.rs`, 2,307 lines) already owns the
  OTLP SDK host for traces, logs and metrics, its settings schema and validation, W3C carriers, loss
  observation, 18 native instruments and tokio-metrics. Python adds six things the Rust runtime does not have:
  (1) resolving the `OTEL_*` environment variables, which the architecture assigns to Python;
  (2) a second, Python OTel SDK for Python processes;
  (3) host metrics for work Python performs (cargo builds, assessment gates, PSI pressure);
  (4) lifecycle of a user-systemd **otelcol-contrib 0.163.2 + OpenObserve OSS 1.1.2** service pair, which
  the brief asked to verify and which is confirmed;
  (5) a typed SQL/PromQL query client over OpenObserve;
  (6) four OpenObserve alert rules.
  Items (1) and (3) are duplicated concepts. Items (4) to (6) are genuinely new and have no Rust counterpart.
  All of them are ordinary HTTP, JSON, file and `systemctl` work, so they port cleanly to Rust with crates
  already in the tree (reqwest, serde, schemars, rustix, sha2, procfs-core). Only the Python-SDK part
  disappears instead of being ported.
- **Devtools.** Most of this is doc, ADR and agent governance glue that belongs in xtask. xtask already owns
  the knowledge store and its subject, role and relation vocabularies, codegen provenance and the
  `GENERATED.sha256` writer, but has no Markdown or YAML-front-matter reader (`doc-lint` is a stub).
  Inherently Python-bound pieces:
  - `structural_migrations.py`, a LibCST/ast-grep rewrite of Python source;
  - the two generated-Python import checkers;
  - `reuse_lint.py`, which wraps the Python `reuse` tool;
  - the Codex-runtime glue, which is not product code at all.

---

### 1. Summary table

Classes and dispositions follow the BRIEF vocabulary. Effort: S = days, M = 1–2 weeks, L = more.

#### 1a. Telemetry cluster (4,767 LOC)

| Module/group | LOC | What it does | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---|---|---|---|---|---|---|
| `telemetry_service.py` | 1235 | Installs, configures and owns **two user-systemd units** (`systemctl --user`, `~/.config/systemd/user`, `WantedBy=default.target`): otelcol-contrib and OpenObserve. Writes a private state dir (0700, flock), `service.env` with the generated OpenObserve root password, `collector.yaml` and per-generation unit files. Runs `otelcol validate`, does start/stop/restart/enable/disable, a reconfigure state machine with a rollback receipt, and recovery. Binds MainPID/InvocationID/ControlGroup to /proc identity before acting. | NEW-CONTROL (secondary NEW-OBS) | Nothing for these services. Pattern precedent: `pse-operations/src/canonical_files.rs` already shells `systemctl show --property=InvocationID,ControlGroup`. `/proc` identity: `pse-diagnostics/src/process_facts.rs` (procfs-core), which duplicates the Python `process_observation.py` used here. | PORT-LIB (xtask feature module or ops bin; new crate needs ADR) | rustix (flock, O_NOFOLLOW, fsync; in tree), sha2, serde_json, serde-saphyr (emit YAML, or keep JSON-as-YAML), procfs-core via pse-diagnostics. Systemd: keep the `systemctl` subprocess or use `zbus` + `zbus_systemd` (exists; uncertain on API maturity). | L |
| `telemetry_config.py` | 337 | Resolves `OTEL_*` env, the local `configuration.json` and SDK defaults into the generated private `Settings` wire (msgspec). Validates endpoints, headers, bounds and samplers. Produces the redacted "effective" view. | DUP (secondary GEN consumer) | `tooling::telemetry::Settings` (serde/schemars, the source of `scripts/generated/telemetry_settings.py`). `Settings::validate` re-checks the same rules: ≤32 headers / ≤8192 bytes, http(s) endpoint without userinfo/query/fragment, batch ≤ queue, timeout ranges. | PORT-EXISTING (`Settings::from_env` in pse-runtime; needs an amendment to the "Python owns environment resolution" contract) | reqwest::Url (in tree); opentelemetry-otlp's own env parsing is avoided on purpose | S–M |
| `telemetry_launch.py` | 41 | Called by `pse_env.environment()` on **every** `scripts/pse-env` launch: resolves settings and exports `PSE_NATIVE_TELEMETRY_SETTINGS` JSON for Rust children. | ORCH (DUP glue) | `Host::from_environment` / `initialize_private_child_host` consume that JSON | DELETE once Rust resolves env | — | S |
| `telemetry_process.py` | 645 | Composes the **Python** OTel SDK (TracerProvider/LoggerProvider/MeterProvider, OTLP http/protobuf via `requests`). Includes: a W3C carrier validator and env carrier (`PSE_TELEMETRY_CARRIER`); a log-redaction filter; a `requests.Session` subclass to undo SDK 1.45.1 header overlay; `run_helper` used by 5 capability workers; activation of the Rust host via `pse._telemetry.initialize`. | NEW-OBS (secondary DUP) | Rust `Host::build` does the same composition for native code, including the same header-overlay workaround (`ResolvedClient`). `Carrier::from_fields` duplicates W3C validation; Rust's child carrier uses a different env name (`PSE_NATIVE_TRACE_CONTEXT`). `LossObserver` has drop observation; the Python side reports it as "unavailable". | DELETE most; KEEP-PY ~50 lines only if a thin Python shell must emit its own spans (else expose a span or carrier API from the Rust host via pyo3) | pyo3 span bridge over `tracing` (in tree) | M |
| `telemetry_metrics.py` | 562 | Python `MetricDeclaration` dataclass and 11 host instruments (`pse.build.*`, `pse.assessment.*`, `pse.engineering.*`, `pse.host.pressure`), with finite attribute vocabularies, SDK Views and cumulative temporality. Fed by `build_observation.py`, `validation.py` and `pressure_observation.py`. | NEW-OBS (secondary DUP of declaration schema) | `tooling/metric_declarations.rs`: an identical `MetricDeclaration` struct (+`coverage`) for 18 native instruments, exported as `scripts/generated/native_metric_declarations.json`. Naming conventions differ (`pse.build.records` vs `pse_runtime_executor_workers`). | PORT-EXISTING: add host declarations to the Rust declaration table; instruments follow whichever crate absorbs cargo/assessment orchestration (other slices) | opentelemetry metrics (in tree); procfs-core `pressure` parsing (believed present; verify) | M (coupled) |
| `telemetry_investigation.py` | 728 | pydantic `QueryRequest/QuerySource/QueryAnswer`. Semantic admission of metric estimators against merged native + host declarations. Lowers requests to OpenObserve `_search` SQL or PromQL `query_range`/`query_exemplars`. httpx2 async client with byte and deadline bounds. Fences on backend process identity before and after reads. | NEW-OBS | Nothing: no Rust OpenObserve/PromQL client (grep for `openobserve`/`otelcol` in crates and xtask: 0 hits) | PORT-LIB | reqwest (in tree), serde, schemars (to keep the MCP-facing schema), url | M |
| `telemetry_query.py` | 113 | Synchronous anyio wrappers (`request`, `lookup_trace`) over the investigation client | WRAP | — | DELETE on port | — | S |
| `telemetry_alerts.py` | 553 | Creates/updates four fixed OpenObserve alert rules (collector queue pressure, export failures, SurrealMX poisoned, MX async queue depth), a template and a loopback destination via the OpenObserve REST v2 API. Reads alert history, source freshness and delivered alerts. | NEW-OBS | Nothing | PORT-LIB | reqwest, serde_json | M |
| `telemetry_selection.py` | 252 | Exports copied `QueryAnswer`s as an "investigation" member of a **native** member pack via `pse.diagnostics.export_member_pack`. Paginates with sha-bound base64 cursors. | WRAP (NEW-STORE semantics) | Pack export/read is Rust (`pse.diagnostics`, `tooling/review_pack`) | PORT-EXISTING into the Rust pack owner | serde, base64, sha2 (in tree) | S |
| `telemetry.py` | 301 | `just telemetry` CLI (16 verbs). Health aggregator: service inspect, `/healthz` probes, collector Prometheus self-metrics parsed with `prometheus_client`, re-inspection for staleness. | NEW-IFACE (secondary NEW-OBS) | Nothing | PORT-LIB (clap subcommand) | clap, reqwest; `prometheus-parse` crate (exists; flag) or the OTLP self-metrics the collector already exports | S |

#### 1b. Workspace / agent / doc / devtool cluster (9,448 LOC)

| Module/group | LOC | What it does | Class | Rust already owns | Disposition | Candidate Rust libs | Effort |
|---|---|---|---|---|---|---|---|
| `workspace_context.py` | 516 | `just context` agent-discovery CLI (≈40 flags): recipe search via `just --dump`, symbol "boundary leads" (line grep over catalogue/pse-py/python/xtask), task packets, library packets, retrospective, history recall | NEW-DEVTOOL (NEW-IFACE) | xtask provenance (`docs/generated/source-relationships/**/relationships.json` is xtask-written) | PORT-EXISTING (xtask `context` subcommand) or DELETE overlapping parts (agents grep natively) | clap, `ignore` + `regex` (in tree) | M |
| `workspace_domains.py` + `workspace_domain_contracts.py` | 1137 + 179 | Projects "owner-native concepts" (documents, decisions, contract sections, plan packages, declarations, generators, generated outputs, evidence refs, experiments, research questions) from ADR front matter, section headings, lifecycle scope tables and producer relationships. Hands them to the xtask knowledge store via `knowledge_store.py`. pydantic request models. | NEW-DEVTOOL + **DUP** | xtask `development/knowledge.rs` + `catalog/knowledge.rs` own the same 11 subject kinds (`KnowledgeSubjectKind`), admission policy per kind (`SUBJECTS`), roles and relation kinds, and the SurrealDB store. Python hand-maintains `native_kind` strings mirroring that enum, plus a second hyphenated spelling. | PORT-EXISTING (move projection next to the store; xtask parses docs directly, removing the JSON hand-off) | serde-saphyr (in tree), pulldown-cmark or comrak (tables), toml (in tree) | L |
| `workspace_sources.py` | 1016 | Bounded `git ls-files` membership, source reads with digests and role/scope, a lexical search engine (regex over files), producer-relationship lookup, a "library packet" from Cargo.toml/Cargo.lock/pyproject/uv.lock plus supplied cargo-metadata/rustdoc JSON | NEW-DEVTOOL | `cargo_metadata`, `toml`, `ignore`, `regex` in xtask; xtask writes the relationships it reads | PORT-EXISTING | ignore, `grep-searcher`/`grep-regex` (ripgrep libs, exist), gix (exists; optional vs git subprocess), cargo_metadata | M |
| `workspace_tasks.py` | 346 | Task packet from plan files (question leads, `git log -S/-G` rationale), "alternatives" comparison doc, `git log -L` line history | NEW-DEVTOOL | — | PORT-EXISTING or DELETE (thin wrappers over git) | gix or git subprocess | S |
| `semantic_navigation.py` + `semantic_edits.py` + `semantic_edit_probe.py` | 344 + 392 + 309 | Drives the pinned **mcpls** (Rust LSP→MCP bridge) via a FastMCP client to rust-analyzer and pyrefly. Turns LSP WorkspaceEdits into digest-bound previews, then apply/reconcile with receipts. Probe = real-server fixture qualification. Exposed via the Python capabilities MCP server (`capabilities/specialists.py`). | NEW-DEVTOOL / ORCH | Nothing (mcpls is an external Rust binary) | DELETE (agent runtimes have native LSP/MCP opt-in) **or** PORT-LIB with the capability server if it moves to rmcp | rmcp client (in docs/libraries), lsp-types | M |
| `structural_migrations.py` | 193 | One fixed rule: `X(path.read_text())` → `X(path.read_bytes())` for JSON loaders. ast-grep 0.50.0 candidates, LibCST import/scope resolution, preview via `semantic_edits` | NEW-DEVTOOL (Python-source-bound) | — | KEEP-PY until the Python it edits shrinks, then DELETE (a one-off migration; vanishes with its target) | n/a | — |
| `codex_mcp.py` | 425 | Discovers the Codex app-server Unix socket via /proc ancestry and sends JSON-RPC over websockets (`initialize`, `thread/read`, MCP reload) | NEW-DEVTOOL (agent runtime) | — | KEEP-PY out of tree, or DELETE (not product) | (tokio-tungstenite if ported) | — |
| `codex_session.py` | 206 | Builds a `codex` launch command with checkout env and selected MCP capabilities | NEW-DEVTOOL | — | KEEP-PY out of tree / DELETE | — | — |
| `session_retrospective.py` | 731 | Bounded, resumable scanner over `~/.codex/sessions/**/*.jsonl` (metadata, hashes, locators, recall, excerpts, checkpoints) | NEW-DEVTOOL | — | KEEP-PY out of tree / DELETE | — | — |
| `agent-hooks.py` | 176 | Claude/Codex PreToolUse guard (protected generated paths, authoritative design, accepted ADRs). SessionStart `session-env` renders `pse_env` exports. | NEW-DEVTOOL (secondary DUP) | Codegen roots (xtask `GeneratedTree`/`Language::roots`); the protected-path list is hand-copied here, in AGENTS.md prime directive 2 and in `.claude/settings.json` | KEEP-PY small, but **generate the path list from codegen roots**; or PORT-EXISTING to a prebuilt xtask bin | serde_json | S |
| `agent-config.py` + `check_agent_config.py` + `library_skills.py` | 91 + 431 + 20 | Skill-alias symlink materialisation; lint of instruction paths, `just` recipes, hook wiring and deny rules (stdlib only, "before any environment exists"); 20-line shim into an external `~/.local/share/library-skills/manage.py` | NEW-DEVTOOL | — | PORT-EXISTING (`xtask lint-agents`; cargo exists at bootstrap) or KEEP-PY. Shim: DELETE (call the manager from the recipe). | serde_json, toml, regex (in tree) | S |
| `check_generated.py` | 131 | Recomputes `python/pse/contracts/GENERATED.sha256`; `--write` rewrites it. Used in pre-commit and `.github/workflows/python.yml`. | **DUP** | xtask `codegen.rs` writes that manifest and refuses a second writer ("the Python generator must leave GENERATED.sha256 to the xtask writer"); `just codegen-check` covers drift | DELETE (point pre-commit/CI at `cargo xtask codegen --check`, or a manifest-only fast mode) | — | S |
| `check_private_contracts.py` + `check_python_contracts.py` | 53 + 66 | Called **by xtask codegen** (`codegen.rs:340,365`) to import-validate candidate generated Python (msgspec/pydantic annotation resolution, no `pse`/OTel import) | ORCH (Python-bound checker of GEN) | Generators are Rust | KEEP-PY while generated Python exists; shrinks with it (e.g. `telemetry_settings.py` disappears if Rust resolves env) | n/a | — |
| `check_register.py` | 242 | Lints `docs/adr/register.md` table rows; `--due` runs `$ ` shell checks monthly | NEW-DEVTOOL | — | PORT-EXISTING (xtask) | pulldown-cmark tables, regex | S |
| `adr.py` | 731 | ADR new/lint/index/supersede; front matter via `document_metadata`; git-history immutability and highest-issued checks; section-owner heading index reused by docs/metadata | NEW-DEVTOOL | xtask knowledge reads ADR state only via the Python projection; `doc-lint` is a stub (`main.rs`: "not implemented in phase 0") | PORT-LIB (xtask `adr` subcommands) | serde-saphyr, pulldown-cmark/comrak, gix or git subprocess | M |
| `docs.py` | 477 | Discovers and stages Markdown collections from `docs/site.toml`, then runs **mdBook 0.5.4** and **Pagefind 1.5.2** (both Rust binaries), annotates HTML and validates attachments; also `cargo binstall`s the tools | ORCH | — | PORT-LIB (xtask `docs`), keeping the two binaries as subprocesses or using `mdbook` as a library (API churn at 0.5; flag) | toml, pulldown-cmark, mdbook (lib) | M |
| `document_lifecycle.py` + `document_metadata.py` | 491 + 436 | Strict YAML front matter (ruamel.yaml with node checks: no merge keys, non-string keys or aliases), `docs/lifecycle.toml` policy, Markdown table extraction (markdown-it-py) for plan scope tables, inventory/aggregate/snapshot | NEW-DEVTOOL + **DUP** | `document_metadata.ENUMS["doc_role"]` (8 values) = xtask `catalog/knowledge.rs::DOCUMENT_ROLES` (8 values, hand-written) = generated `KnowledgeDocumentRole` in `scripts/generated/knowledge_query_input.py` (no Python importer found) | PORT-LIB (xtask doc model shared by adr/docs/knowledge) | serde-saphyr (strict duplicate-key handling; verify), comrak or pulldown-cmark, toml | M |
| `github-config.py` | 164 | Read-only `gh api` diff of repo settings against `.github/setup/*.json` | NEW-DEVTOOL | — | KEEP-PY (trivial) or xtask | (octocrab/reqwest) | S |
| `reuse_lint.py` | 44 | Copies Git-eligible files to a temp snapshot, then runs the Python `reuse` CLI | ORCH around a Python tool | — | KEEP-PY (the tool is Python) | — | — |
| `worktree.py` | 101 | `git worktree add`, copy `.envrc.local`, `direnv allow`, `just skills-sync`, optional `uv sync`/`py-sync`, `just doctor` | ORCH | — | KEEP-PY or move to a bash recipe (pure sequencing) | — | S |

---

### 2. Per-group notes

#### 2.1 What the Rust runtime already emits (telemetry baseline)

- `crates/pse-runtime/src/tooling/telemetry.rs` (2,307 lines):
  - `Settings`/`SignalSettings`/`TelemetrySampler` (serde + schemars). The generator relationship is
    declared at `xtask/src/codegen.rs:~430` (`pse_runtime::tooling::telemetry::Settings` →
    `scripts/generated/telemetry_settings.py`).
  - `Settings::from_json` + `validate`.
  - `Host::build` composes `SdkTracerProvider`, `SdkLoggerProvider` and `SdkMeterProvider` with
    `opentelemetry_otlp` http/protobuf exporters and a custom `ResolvedClient`, because SDK 0.33.1 overlays
    env headers.
  - `ObservedBatchProcessor` + `LossObserver`/`SdkLossHealth`; `Carrier` (W3C).
  - `Host::from_environment` reads only `PSE_NATIVE_TELEMETRY_SETTINGS`, the JSON produced by the Python
    resolver. There is no `OTEL_*` env handling in Rust (grep for `OTEL_`: none).
- `tooling/metric_declarations.rs` (525): 18 native instruments → `scripts/generated/native_metric_declarations.json`.
  `operational_metrics.rs` (589) and `runtime_metrics.rs` (tokio-metrics `TaskMonitor`, executor registry gauges).
- `crates/pse-py/src/telemetry.rs`: `_telemetry_initialize/_shutdown/_status`. `python/pse/_telemetry.py` (53)
  is the thin facade the Python helper calls.
- Workspace deps: `opentelemetry`/`_sdk`/`-otlp`/`-http` 0.33.1, `tracing-opentelemetry` 0.34.0,
  `opentelemetry-appender-tracing`, `tokio-metrics` 0.5.2, `tracing-perfetto-sdk-schema` (finite capture path), `procfs-core`.
- The architecture states the split explicitly (`docs/authoritative_design/sections/operations-and-validation.md`,
  "Shared diagnostic telemetry"): "*Python owns environment resolution; immutable encoded settings and native
  validation preserve that single selection.*" Porting resolution to Rust therefore needs a design amendment
  through the decision route.

#### 2.2 What Python adds beyond Rust (per capability)

1. **Env resolution** (`telemetry_config.resolve_process`). This is pure precedence logic over `OTEL_*` plus a
   local `configuration.json`, and the same bounds are then re-validated in Rust `Settings::validate`. The two
   codes check identical limits: headers ≤32 and ≤8192 bytes, URL shape, batch ≤ queue, and timeouts (Python
   reads those bounds back from msgspec `Meta(ge, le)` of the generated module, via `_integer_limits`). This is
   a cross-language contract kept consistent by a generated mirror and two validators. **Rust route:**
   `Settings::from_env(service, env)` in pse-runtime. The Python shell and `pse-env` would call native code or
   simply let the Rust executable resolve. That deletes `telemetry_launch.py`, most of `telemetry_config.py`,
   and the generated `telemetry_settings.py` mirror together with its `check_private_contracts.py` import check.
2. **Python SDK** (`telemetry_process.py`). This exists only because Python processes do work worth tracing:
   the capability server (`capabilities/__main__.py` calls `bootstrap`) and the 5 workers that call
   `run_helper` (`capabilities/native_worker.py`, `native_read_worker.py`, `host_worker.py`,
   `preparation_worker.py`, `qualification_worker.py`); `capabilities/process.py` and
   `native_read_sessions.py` use `helper_environment` for carrier propagation. An embedded Python process therefore runs **two OTel SDKs**, Python 1.45.1
   and Rust 0.33.1, each with its own batch queue and exporter to the same collector. Both carry the same
   header-overlay workaround (`_export_session` vs `ResolvedClient`) and their own W3C carrier validators,
   with different env carriers (`PSE_TELEMETRY_CARRIER` vs `PSE_NATIVE_TRACE_CONTEXT`). Python loss
   observation is self-declared "unavailable" (`_loss_status`), whereas Rust has `LossObserver`. **Rust
   route:** if Python becomes a thin shell, emit its few spans through the Rust host (a pyo3 span/carrier
   entry point) and delete the Python SDK, the logging-redaction filter (`_SafeTelemetryDiagnostics`) and
   the opentelemetry/requests dependencies.
3. **Host metrics** (`telemetry_metrics.py`). These cover cargo child records and durations, assessment gate
   and case populations, engineering stages and Linux PSI pressure. They are produced by Python
   orchestration modules in other slices (`build_observation.py`, `validation.py`, `pressure_observation.py`).
   The declaration *shape* duplicates Rust `MetricDeclaration`, and the query layer merges both sources at
   read time (`telemetry_investigation.metric_declarations()`, renaming `.`→`_`). **Rust route:** move these
   declarations into the Rust declaration table, and the instruments into whichever crate absorbs
   cargo/assessment orchestration. PSI can be read with procfs-core (believed to have a `pressure` module;
   verify) or a 30-line parser. The cgroup `cpu.pressure`, `io.pressure` and `memory.pressure` files use the
   same format.
4. **Service lifecycle** (`telemetry_service.py`, verified). It owns a pinned collector and backend
   (`.config/diagnostic-tools.toml`: otelcol-contrib 0.163.2, openobserve 1.1.2) installed through
   `diagnostic_trace.install`. It writes immutable generations (`generations/<uuid>/collector.yaml`, unit
   files, `service.env` with `ZO_ROOT_USER_PASSWORD` and a Basic auth header). It publishes units into the
   user unit directory, runs `daemon-reload` and polls `systemctl --user show` every 50 ms until active.
   Before any stop it verifies ownership by unit-fragment digest, MainPID/InvocationID/ControlGroup and the
   `/proc` exe dev/inode (`_checked_process`), and checks that the cgroup is empty afterwards. It has a full
   reconfigure → rollback → recover state machine with an on-disk receipt. The size comes from lifecycle
   edge cases (stale PIDs, auto-restart windows, partial pointer publication), not from cross-language
   plumbing. **Rust route:** a straight port with rustix and sha2. The `/proc` binding should reuse
   `pse-diagnostics::process_facts` (procfs-core), which is already a Rust twin of the Python
   `process_observation.py` this module imports. The `systemctl` subprocess can stay (Rust precedent in
   `canonical_files.rs`) or move to D-Bus via zbus.
   *Simplification option (design question, not verified):* OpenObserve ingests OTLP directly. The collector
   adds a file-backed queue, retry and self-metrics. Dropping it would roughly halve the unit and lifecycle
   surface, but loses the durable queue that the alert `queue_pressure`/`export_failures` rules observe.
5. **Query and investigation** (`telemetry_investigation.py`, `telemetry_query.py`, `telemetry_selection.py`).
   A closed vocabulary lowers requests to OpenObserve SQL (`SELECT * FROM "<stream>" WHERE … ORDER BY`) or
   PromQL (`rate`, `increase`, `histogram_quantile(0.95, …)`, a bucket ratio for `fraction_below`). Estimator
   admission is driven by declaration semantics (temporality, monotonicity, buckets). Reads are fenced on
   backend process identity before and after. Export is already delegated to Rust
   (`pse.diagnostics.export_member_pack`, `pack_member`). **Rust route:** a reqwest client plus serde types
   with schemars, so the MCP schema (`schema_description()`) is generated rather than produced by pydantic.
   Admission can read `metric_declarations()` in-process rather than through the JSON file.
6. **Alerts** (`telemetry_alerts.py`). Four fixed rules, two over collector self-metrics and two over SurrealDB
   MX gauges. They use OpenObserve's own scheduler (no host polling) and a loopback HTTP destination into the
   `pse_alerts` stream. **Rust route:** a reqwest port. The rule SQL strings are constants.
7. **CLI and health** (`telemetry.py`). The CLI is a dispatcher. Health combines service inspection, two
   `/healthz` GETs and a Prometheus text scrape (`prometheus_client.parser`) filtered to `otelcol_*` exporter
   and receiver counters.

Consumers outside this slice that would move with it: `scripts/capabilities/telemetry_queries.py` (315),
`telemetry_contracts.py` (264), `telemetry_profiles.py` (104), `telemetry_profile_worker.py` (91),
`server.py` and `host_queries.py` (BackendClient). `surreal_server.py:2670` and `native_tests.py:288` also
forward `PSE_NATIVE_TELEMETRY_SETTINGS`.

#### 2.3 Workspace/knowledge discovery (3,194 LOC)

- `workspace_domains.project()` (`_Projection.fact/add/contribute`, `_decisions`, `_sections`, `_packages`,
  `_productions`) parses Markdown, front matter and lifecycle scope tables, and reads xtask-written
  `docs/generated/source-relationships/**/relationships.json` (`workspace_sources.relationships`). It then
  emits subjects and edges that `knowledge_store.py` sends to `cargo xtask knowledge-*`, a Rust and
  SurrealDB store with its own admission (`catalog/knowledge.rs::SUBJECTS` per-kind policy).
  The data flow is Rust codegen → JSON → Python projection → JSON → Rust store. The Python layer is the
  only Markdown/YAML reader on the path.
- **Vocabulary duplication.**
  - Subject kinds: `workspace_domain_contracts.py` hard-codes `native_kind` strings for 11 kinds that equal
    `KnowledgeSubjectKind` (`xtask/src/generated/model/enums.rs`), plus a second hyphenated presentation
    spelling (`plan-package`, `generated-output`, `contract-section`).
  - Document roles: `document_metadata.ENUMS` hard-codes the 8 roles that `catalog/knowledge.rs::DOCUMENT_ROLES`
    also hard-codes.
  - Generated Python already exists: `scripts/generated/knowledge_query_input.py` defines `KnowledgeSubjectKind`
    and `KnowledgeDocumentRole` StrEnums, but no Python file imports it (grep).
  - The result is three copies of each vocabulary, and only the Rust ones are authoritative.
- `workspace_sources.search` is a hand-rolled regex grep over `git ls-files`, with byte, time and file
  budgets. `library_packet` re-reads Cargo.toml, Cargo.lock and uv.lock. Both port to xtask with crates it
  already has (`ignore`, `regex`, `toml`, `cargo_metadata`).
- `workspace_context.describe` shells `just --dump --dump-format json` and greps recipe docs. `boundary_leads`
  line-greps up to 64 MiB of source per query.

#### 2.4 Semantic navigation and edits (1,238 LOC)

- `semantic_navigation.connection()` writes an mcpls TOML, spawns `mcpls` over a FastMCP `StdioTransport`
  and retries readiness errors (`-32051`) for 45 s. The LSP servers are rust-analyzer and `.venv/bin/pyrefly`.
- `semantic_edits` converts LSP UTF-8/UTF-16 positions, digests every input, previews, validates, applies and
  reconciles partial writes with a receipt.
- `structural_migrations` holds one ast-grep rule plus LibCST import and shadow resolution for a single
  Python-only rewrite. It vanishes with the Python it edits.
- These tools reach agents through the Python capability MCP server (`capabilities/specialists.py`). Their
  fate follows that server. Claude Code and Codex both have native LSP/MCP opt-in (AGENTS.md "Agent
  runtimes"), so a Rust port is optional, not required.

#### 2.5 Agent runtime glue (2,080 LOC including hooks/config)

- `codex_mcp.py`, `codex_session.py` and `session_retrospective.py` are Codex-specific. They talk to the Codex
  app-server over a websocket on a Unix socket found via /proc ancestry, build `codex` command lines, and
  scan `~/.codex/sessions` JSONL. None touches product code. **Disposition:** move them out of the product
  repo, next to the shared library-skills manager that `library_skills.py` already delegates to, or delete
  them. Porting to Rust would gain nothing.
- `agent-hooks.py` runs as a Python process **on every agent edit** (PreToolUse `guard`). It also spawns
  `git ls-files` when `.design-edit` exists. The protected-path list
  (`docs/generated/`, `scripts/generated/`, `xtask/src/generated/`, `python/pse/contracts/`, …) is a hand
  copy of prime directive 2 and of `.claude/settings.json` denies. It should be generated from the codegen
  roots that xtask already knows. `session-env` imports all of `pse_env` (see 3.3).
- `check_agent_config.py` is stdlib-only by design. Cargo is also present at bootstrap, so an
  `xtask lint-agents` is feasible.

#### 2.6 Doc and ADR governance (2,627 LOC incl. the four check_* scripts; +309 for github-config/reuse_lint/worktree)

- `adr.py` (lint, index, supersede, immutability against `origin/main`), `check_register.py` (register table),
  `document_metadata.py` (strict YAML), `document_lifecycle.py` (scope tables) and `docs.py` (mdBook +
  Pagefind staging) together form a small document model: front matter + headings + tables + `lifecycle.toml`
  + `site.toml`.
- xtask currently has **no** Markdown or front-matter reader (`DocLint` prints "not implemented in phase 0").
  The Rust workspace has `serde-saphyr` (used in pse-admission and pse-runtime) and `toml`, but no Markdown
  crate.
- A single xtask `docs` module using comrak or pulldown-cmark, plus serde-saphyr, could serve `adr`,
  `register`, `docs build`, lifecycle aggregates and the knowledge projection. It would also give the
  stubbed `doc-lint` a home.
- Python's markdown-it-py and ruamel.yaml are the only reason the docs bootstrap needs a Python environment
  (`just bootstrap-docs`). mdBook and Pagefind are already Rust binaries.
- `check_generated.py` is a straight DUP of the xtask manifest writer and check.
- The two contract checkers are the opposite case: xtask **calls Python** to import-check generated Python.
  They are legitimately Python-bound and shrink only as generated Python surfaces are removed.

---

### 3. Cross-cutting observations

#### 3.1 Duplicated sources of truth (concrete)

| Concept | Python copy | Rust copy |
|---|---|---|
| Telemetry settings validation | `telemetry_config._headers/_endpoint/_timeout_seconds`, batch≤queue | `tooling::telemetry::Settings::validate` |
| W3C carrier validation + env carrier | `telemetry_process.validated_carrier`, `PSE_TELEMETRY_CARRIER` | `Carrier::from_fields/from_json`, `PSE_NATIVE_TRACE_CONTEXT` |
| OTLP header-overlay workaround | `telemetry_process._export_session` (requests Session) | `ResolvedClient` (`opentelemetry_http::HttpClient`) |
| Metric declaration schema | `telemetry_metrics.MetricDeclaration` (11 host instruments, dotted names) | `metric_declarations::MetricDeclaration` (18 native, snake names) |
| `/proc` process identity | `process_observation.py` (used by `telemetry_service._checked_process`) | `pse-diagnostics::process_facts` (procfs-core) |
| Knowledge subject kinds | `workspace_domain_contracts.*Query.native_kind` | `KnowledgeSubjectKind` (registry enum) + unused generated Python StrEnum |
| Document roles | `document_metadata.ENUMS["doc_role"]` | `catalog/knowledge.rs::DOCUMENT_ROLES` + unused generated `KnowledgeDocumentRole` |
| Generated-contract manifest | `check_generated.py` (`--write` too) | xtask `codegen.rs` manifest writer/check |
| Protected generated paths | `agent-hooks.py::protected` | codegen roots (and AGENTS.md, `.claude/settings.json`) |

#### 3.2 Recurring patterns

- **Rust → JSON → Python → JSON → Rust loops.** The telemetry settings, the knowledge projection and the
  metric declarations each make a round trip like this. The Python stage adds parsing or env precedence that
  Rust could do in-process. Each loop needs a generated mirror plus a Python import-check that xtask invokes
  (`check_private_contracts.py`).
- **Inversions.** xtask shells to Python for `governance` (`scripts.arrow_validation nextest …`) and for codegen
  checks. `crates/pse-runtime/tests/worker.rs` shells to `scripts.surreal_server`. Rust depends on the Python
  layer it is meant to replace.
- **Defensive bounds re-implemented per module.** Bounded reads, deadlines, byte caps, digest fences and
  "unavailable vs absent" outcomes appear in `workspace_sources._command`, `session_retrospective.inspect`,
  `telemetry_investigation.BackendClient.request` and `telemetry_service._read`. In Rust these would be one
  shared helper.
- **Subprocess orchestration of external binaries** (git, just, systemctl, otelcol, mdbook, pagefind,
  ast-grep, mcpls, gh, reuse, codex). Most of these are Rust binaries or have Rust library equivalents. Only
  `reuse`, LibCST and pyrefly are truly Python-ecosystem.

#### 3.3 Performance-relevant findings

- **Every `scripts/pse-env` launch pays for telemetry resolution, even when telemetry is disabled.**
  `pse_env.environment()` lazily imports `telemetry_launch` → `telemetry_config` (msgspec,
  `importlib.metadata`, the generated settings module) and calls `version("pse-arrow")`.
  - *Measured* in this sandbox, not the workstation: system Python 3.13.18, 3 runs, state dir absent.
    `import scripts.telemetry_config` took 48–60 ms; `resolve_process` took ~3.5 ms; `import scripts.pse_env`
    itself took 84–89 ms.
  - So telemetry adds roughly 50 ms (~+55 %) to the per-command environment boundary that every workload
    recipe and every Codex command passes through.
  - A Rust resolver inside the launched binary, or simply checking the state file before importing, removes it.
- **Per-edit interpreter start.** `agent-hooks.py guard` starts a fresh Python for every agent file edit; a
  compiled guard or a generated static deny list avoids it. `session-env` imports the full `pse_env` (≈85 ms
  plus telemetry) once per session.
- **Polling.** `telemetry_service._start` polls `systemctl --user show` every 50 ms for up to 10 s per unit,
  forking a `systemctl` each time. D-Bus `JobRemoved`/`PropertiesChanged` signals via zbus would avoid the
  fork loop. Minor.
- **Two SDKs in one embedded process** (Python + Rust OTel) double batch threads and exporters. This matters
  for footprint, not correctness.
- No Python on a scientific hot path in this slice. Telemetry export is off-path by design.

---

### 4. Tests under scripts/tests/

| Group | Files | LOC | Notes |
|---|---|---|---|
| Telemetry unit tests (`test_telemetry_*.py`) | 12 | 4,340 | 119 `test_` functions; includes `test_telemetry_profiles.py` (272), which targets `capabilities/telemetry_profiles.py` |
| Telemetry manual acceptance/inquiries (`telemetry_*_acceptance.py`, `opentelemetry_*_inquiry.py`, `capability_telemetry_inquiry.py`) | 13 | 6,506 | Effectful: spin up real otelcol/OpenObserve generations ("never discovered by unittest or pytest"); `telemetry_pipeline_acceptance.py` alone is 1,307 |
| `test_capability_telemetry.py` | 1 | 629 | Capability-server telemetry tools |
| **Telemetry total** | 26 | **11,475** | ≈2.4× the 4,767 production LOC |
| Devtool unit tests (`test_workspace_*`, `test_semantic_*`, `test_codex_*`, `test_session_retrospective`, `test_adr`, `test_docs`, `docs_integration`, `test_document_lifecycle`, `test_worktree`) | 14 | 4,176 | 121 `test_` functions; `test_workspace_domains.py` is 1,140 |
| Partial coverage elsewhere | — | (1,012 + 76) | `test_setup.py` exercises agent-hooks/agent-config/check_agent_config/check_register; `test_python_contract_annotations.py` covers `check_python_contracts.py` |
| No test found (grep) | — | — | `check_generated.py`, `check_private_contracts.py` (exercised via codegen), `github-config.py`, `reuse_lint.py`, `library_skills.py`, `semantic_edit_probe.py` (itself a probe) |

Porting implication: the telemetry acceptance harnesses (6.5 K LOC) are service-level and largely
language-agnostic, since they drive real binaries over HTTP. They could be retained as Python oracles against
a Rust implementation during migration, then retired. The unit tests are module-specific and would be
rewritten, not ported (AGENTS.md: do not port tests for a deleted mechanism).

### 5. Suggested ordering (evidence-driven, not a plan)

1. **S, removes DUP and per-command cost:**
   - delete `check_generated.py`;
   - generate the hook's protected-path list;
   - wire `document_metadata`/`workspace_domain_contracts` to the existing generated knowledge enums, or
     better, move those parsers to xtask;
   - add `Settings::from_env` in Rust and delete `telemetry_launch.py`. The last item needs the
     operations-and-validation contract amendment.
2. **M:**
   - port the query, alerts, selection and CLI code to Rust (reqwest);
   - fold host metric declarations into the Rust table.
3. **L:**
   - port `telemetry_service` (or first decide whether the collector stays);
   - build the xtask doc model (front matter + Markdown + lifecycle) to absorb adr/docs/lifecycle/register and
     the knowledge projection.
4. **Out of tree or delete:** Codex glue (1,362 LOC); semantic navigation/edits if the capability MCP server
   is not kept; `structural_migrations.py` once its target Python is gone.
