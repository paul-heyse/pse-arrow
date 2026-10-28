# Python subsumption review: Python as a thin shell over an integrated Rust system (2026-10-28)

- **Baseline:** pse-arrow `bd1c4891e` (clean `main`). Scope is the tracked files, excluding `vendor/` and `external/`.
- **Method:**
  - Six parallel source reviews covered every Python module in `python/pse` and `scripts/`. Each review read the code paths, compared them with the Rust owners in `crates/` and `xtask/`, and assigned a class and a disposition.
  - The headline claims were then spot-checked against source. These include line counts, the compiler wrapper, the Rust→Python inversions, the dispatch duplication and the prior rmcp decisions.
  - Libraries were researched from the owner's `docs/libraries/` overviews and from primary documentation.
  - No builds, tests or services were run on the workstation.
  - Timings were taken in a cloud sandbox (CPython 3.13, warm bytecode, no systemd user manager) and are labelled *Measured (sandbox)*. Everything else is *Interface-checked* (read from source) or *Proposed*.
- **Status:** this is evidence for the decision route, not authority. Several targets below conflict with current contracts and need ADRs (§10).
- **Evidence appendix:** `claude/python-subsumption-review-2026-10-28-evidence.md` holds the six slice reports, with file and symbol references.

---

## 1. Bottom line

1. **The Python is not a boundary layer; it has become the system's operating layer.** There are about 160K
   production lines of Python, against 1.74M lines of Rust.
   - The product package (`python/pse`, 59.5K lines) is **86% generated mirror** of Rust types.
   - The other 100.5K lines in `scripts/` *run* the Rust system: a local SurrealDB operator, host admission and
     process lifetime, the compiler-cache wrapper, the test runner and fixture registry, the telemetry backend,
     the assessment/evidence pipeline and the agent-facing MCP server.
2. **By character:**
   - about **41%** mirrors or wraps Rust;
   - about **21%** orchestrates Rust tools (cargo, nextest, criterion, xtask binaries);
   - about **30%** is genuinely new control, observability or interface capability;
   - about **6%** is developer and agent governance tooling;
   - about **1.4%** is inherently Python (scientific oracles, IDAES parity, the numpy/pyarrow consumer edge).
3. **The genuinely new capability is real but narrow, and most of its code volume is cross-boundary
   plumbing.** Examples:
   - freezing and re-verifying a 26K-line Python import closure so that a Python process can be the systemd
     `ExecStart` parent of the database;
   - 3.2K lines that keep a hot-edited Python MCP host coherent with a separately built extension;
   - cancellation-clock hand-offs between interpreter hops;
   - 1,006 `patch.object` calls in one supervisor test file.

   Inside a single Rust process these become compile-time facts or disappear.
4. **Dependencies now point the wrong way.** Rust depends on the Python it should subsume:
   - Rust test fixtures spawn `python -m scripts.test_resources` for every registry action
     (`pse-operations/src/testing.rs`).
   - The runtime admits a storage controller only if it is a file named `surreal_server.py`
     (`pse-runtime/src/workflow/native_deployment.rs`).
   - xtask codegen shells out to a Python regex parser of prost output.
   - Rust tests import `scripts.surreal_server`.
5. **The scientific path is already clean. The Python tax falls on the build, test, agent and control
   loop:**
   - every `rustc` unit and every C/C++ object goes through a Python compiler wrapper;
   - each `just` recipe starts 2–3 interpreters;
   - each canonical Rust test fixture spawns at least 2 interpreters;
   - each MCP native read starts at least 2 interpreters plus `import pse`.

   Subsuming the Python will not speed up a Newton step. It will shorten every compile, test and agent cycle,
   and it removes the machinery that keeps multiple processes and two languages in agreement.
6. **Most of the port lands on Rust that already exists.** Examples are `SupervisedChild`, `process_facts`, the
   gRPC store client, `canonical_files::offline`, `ProductionProfiles`, the telemetry `Settings` and metric
   declarations, and xtask's producer, evidence and knowledge modules. The differentiated libraries fill specific
   gaps:
   - **zbus + zbus_systemd:** systemd control.
   - **rmcp:** MCP.
   - **PyO3 pyclass generation, or pythonize, plus pyo3-opentelemetry:** the Python boundary.
   - **nextest's native features + nextest-metadata + libtest-mimic:** test orchestration.
   - **sd-notify + tokio-graceful-shutdown:** daemon structure.
   - **SurrealKit-as-library and embedded SurrealKV:** narrow, specific database uses.
   - **comrak or pulldown-cmark + serde-saphyr:** documentation tooling.
7. **End state (*Proposed* estimate):** about 3–5K lines of production Python (a shell, oracles and parity),
   down from 160K. Python tests would fall from about 149K lines to roughly 5–10K. Most current tests are
   *replaced* by Rust tests of the Rust owners rather than ported, per AGENTS.md.

---

## 2. Inventory

| Area | Production LOC | Test LOC | Notes |
|---|---:|---:|---|
| `python/pse` product package | 59,465 | 22,082 (+781 `control_tests`, 477 root `conftest.py`) | 51,134 generated (86%): `contracts/`, `generated/`, `_native.pyi` |
| `scripts/` host, dev and agent tooling | 100,495 | 125,931 | 151 top-level modules, `capabilities/` (16.4K), `generated/` (4.0K) |
| **Total Python** | **≈160K** | **≈149K** | Rust: ≈1.74M lines (crates + xtask) |

*Interface-checked* (`wc -l` on the snapshot).

---

## 3. Tabulation: what the Python executes, and whether it extends Rust or adds capability

LOC is production code only; test LOC is in parentheses where known. "Extension" means the code mirrors,
wraps or re-derives something Rust owns. "New" means the capability has no Rust owner today.

| # | Cluster | LOC | What it actually executes | Extension or new? | Disposition → Rust home | Targeted library route |
|---|---|---:|---|---|---|---|
| 1 | **Generated contract mirror** (`contracts/`, `generated/`, `_native.pyi`) | 51.1K (≈3K mirror tests) | msgspec and attrs copies of Rust serde and registry types: 1,873 document classes from 498 JSON Schemas (21 MB); 522 attrs row classes, of which 2 are used in production; 489 input/output twins, 187 of them byte-identical apart from the name. Each crossing goes serde → JSON → a Python tree walk with callbacks into Rust per versioned struct and per ID field. | **Extension (pure duplicate).** Validation and defaults are written on both sides. | **DELETE.** pse-codegen emits frozen `#[pyclass]` wrappers over the serde types, or pythonize + generated TypedDict stubs. Stubs come from pyo3-introspection, which is already in use. | PyO3 0.29 (in tree), pyo3-introspection (in tree); pythonize 0.29 as the light alternative |
| 2 | **Facades, export gate, project-op glue, codec, governance** | 5.7K | About 43 attrs classes that each forward 1:1 to a native pyclass (137 of 142 functions in `_modeling.py` are one-line forwards). Each of 53 project operations passes through 4 layers (2 Rust, 2 Python). Import-time registry and schema fingerprint checks exist because the mirror can drift. | **Extension (wrapper)** | **DELETE.** Make the native classes public (draft Plan 57i I02/I03). | PyO3 |
| 3 | **Product CLIs and child supervision** (`diagnostics`, `conformance`, `_diagnostic_query`, capsule reader) | 1.65K | argparse CLIs over native functions. Hand field-mapping RunSettings → EngineSettings, three times over. A re-exec'd Python child whose only job is `setrlimit` then `exec trace_processor`. A fourth hand reader of xtask's capsule format. | **Extension (orchestration + duplicate of `SupervisedChild`)** | **PORT-EXISTING.** A clap binary in pse-runtime built on `SupervisedChild`. | clap (in tree) |
| 4 | **Durable control client** (`control.py`, ADR-0232, added 2026-10-28) | 0.57K | httpx2 client to the SurrealDB `DEFINE API` for browse, cancel and reconcile without loading the extension. Handles original deadlines, the cancellation-unresolved identity and receipt matching. | **New interface**, partly duplicating `canonical_control.rs` (same transition, second path) | **Decide** between a tiny second cdylib reusing the `canonical_control` types and keeping it Python | reqwest or hyper (in tree) |
| 5 | **MCP capability server, CLI, generated clients** | 20.9K (24.6K) | FastMCP: 96 tools. 22 come from Rust declarations, about 71 wrap Python owners and 22 proxy mcpls or SurrealDB MCP. Native calls go pse-env → a new Python worker → `import pse`, with about 5 JSON transcodings per direction and a hand-written 21-way `if/elif` dispatch even though Rust declares the receivers. Arrow travels as base64 chunks inside JSON. About 3.2K lines keep the multi-process host coherent. | **New interface.** The native path is wrapper + duplicate. | **PORT-LIB.** One Rust binary with tools generated from a unified, dispatchable catalogue (21 host operations + 49 `ProjectAction`s that are described but not callable today). The CLI is built at runtime from the same catalogue. | **rmcp**, schemars, clap |
| 6 | **SurrealDB local operator** (`surreal_server.py` 11.7K + source, module, diagnostics, health, backup) | 14.7K (≈19K) | A complete database operator covering systemd units and scopes, CPU/memory placement, credentials, readiness, offline backup/restore/rebuild, recovery qualification and borrower-aware stop. Python stays resident as the server's parent. It talks to the server over HTTP `/sql`, a REPL-scraped `surreal sql` and export/import, while Rust already has a typed gRPC client. | **New control (real).** Duplicates row shapes, process identity (×4), verified copy, credentials/receipt documents and unit naming. | **PORT-LIB.** A Rust supervisor binary. Keep the external patched server (ADR-0214 rejects per-process embedding for the canonical store). | **zbus + zbus_systemd**, **sd-notify**, **tokio-graceful-shutdown**, rustix/procfs-core, the existing gRPC client, tonic + opentelemetry-proto |
| 7 | **SurrealKit beta tooling + frozen schema copies** | 3.2K (+4.8K tests, 24K data lines) | Beta rollout parity tooling. The committed `database/canonical-schema` is stale against the generated 4,509-line schema. | **Duplicate / stale** | **DELETE.** If rollouts become production, use the SurrealKit Rust library. | SurrealKit library (1.0.0-beta) |
| 8 | **Durable registries and stores** (`test_resources`, `resource_store`, `service_borrowers`, `assessment_store`, evidence/knowledge wrappers, `artifact_publication`) | 6.6K | Flock-guarded JSON-file registries. Rust fixtures call into them through a Python subprocess. The evidence and knowledge databases are Rust-owned but fronted by Python, which re-declares the config, snapshot and bounds. "Publish an immutable generation" is implemented 5 times. | **Mixed.** The fixture-outcome registry is new control (ADR-0174 composer authority); the rest is wrapper or duplicate. | **PORT-EXISTING.** Registry into `pse-testkit` / `pse-operations::testing`, called in process. Evidence and knowledge into `pse-evidence`. One publication module. | rustix flock, tempfile, blake3 (in tree); `surrealdb` `kv-surrealkv` for the disposable evidence index |
| 9 | **Environment and host admission** (`pse_env`, `host_admission`, `systemd_environment`, `execution_profiles`) | 4.0K | Runs for every recipe: composes the environment, takes cross-session lane admission through a JSON ledger, starts a `systemd-run` scope, then a second interpreter binds the allocation. Most of the ledger is inert under the maintainer's `PSE_BUILD_MODE=unbounded`. | **New control, narrow:** cross-session admission, timing exclusion, pressure gate, drain-before-release. It also re-derives the cgroup paths, CPU width and test width that systemd, Cargo and nextest already know. | **PORT-LIB.** A lean standalone `pse-env` binary plus a Rust admission daemon (pidfds, cgroup-event watches). | **zbus_systemd** (`StartTransientUnit` with the caller's own PID, then `exec`), rustix, procfs PSI |
| 10 | **Operation lifetime and host observation** (`native_operation`, `operation_lifetime`, `process_observation`, `activity*`, `pressure_observation`, `build_observation`, disposition, recovery) | 4.6K | Hand-written supervisors with signal handling and cancellation-clock hand-offs across interpreter → systemd-run → interpreter hops. `/proc` and `systemctl show` polling. | **New control and observability**, heavily duplicating `pse-diagnostics::{SupervisedChild, process_facts}` and `canonical_files.rs` | **PORT-EXISTING** onto pse-diagnostics plus D-Bus `KillUnit`. Delete `build_observation` (Cargo timings covers it). | zbus_systemd; process-wrap (optional) |
| 11 | **Compiler-cache wrapper** (`compiler_cache.py`, `scripts/sccache`) | 0.72K | `RUSTC_WRAPPER` and the CMake launcher. Per compile unit: an interpreter, a SHA-256 of the sccache binary, about 4 `systemctl show` calls, about 4 ledger lock-and-rewrite cycles and at least 50 ms of sleep. | **New control on the hot path** | **DELETE.** Run sccache as a templated user service and set `RUSTC_WRAPPER` to the real binary. An optional exec shim of about 100 lines of Rust could keep a peer check. | sccache's native service mode (configuration only) |
| 12 | **Build, native-provider preparation, producer capture, extension build, readiness** | 10.0K | CMake, curl and docker builds of KLU, ibex, Uno, PETSc and HiGHS into receipts that `build.rs` reads as opaque bytes. Re-implemented Cargo-config discovery. Maturin orchestration. doctor and preflight. | **Orchestration + duplicate** (`producer_identity.rs`, `dependency_ceilings.rs`, `pse-buildinfo`) | **PORT-EXISTING** into xtask subcommands. Build KLU in the vendored `suitesparse-sys` build script so Cargo owns freshness. | cargo_metadata (in tree), cmake/flate2 (in lock), tar |
| 13 | **Test runner and scheduling** (`test_run`, `native_tests`, `select`, `test_invocation`, `test_requirements`, `test_archives`, `affected`, `arrow_validation`) | 5.5K | Wraps nextest with extra `list` passes, a per-run config copy, Python JUnit parsing, a hard-coded Python table of dozens of Rust test-module prefixes and test names (partly repeating `nextest.toml`) and four hand-written cargo/nextest/pytest flag grammars. `arrow_validation` fronts every cargo recipe with 2–3 extra cargo resolutions. | **Orchestration** that re-derives nextest's own state | **PORT-EXISTING.** Move scheduling into `nextest.toml`; build `xtask test` over nextest. | **nextest** test groups and overrides, setup and wrapper scripts, `archive.include`, `rdeps()`; **nextest-metadata** |
| 14 | **Assessment, evidence, measurement, diagnostics** (`validation*`, tool wrappers, `case_*`, `pipeline_review`, `reference_run`, `regression_experiments`, `evidence_*`, `active_diagnostics`, `diagnostic_trace`, `faer_arithmetic`, `grpc_wire*`) | 14.1K | A keep-going gate runner with receipts and reuse. A case observer that polls Rust inspection through PyO3. Criterion `raw.csv` re-statistics. Host debugger actions. Hand-mirrored `EvidenceSnapshot`, while unused generated Python models of the same type exist. | **Mostly orchestration.** New observability in diff coverage, bisect plus paired measurement, and active diagnostics. | **PORT-EXISTING:** `xtask assess`; case catalogues as tests; direct pse-runtime tooling calls; `syn` for the wire tables | **libtest-mimic**, criterion `estimates.json`, quick-xml, blake3 + ignore (in tree) |
| 15 | **Telemetry** (`telemetry_*`) | 4.8K (11.5K) | Resolves `OTEL_*` variables (assigned to Python by design). Runs a second OTel SDK inside Python processes and declares 11 host metrics. Owns a user-systemd otelcol + OpenObserve pair, a SQL/PromQL query client and 4 alert rules. | **Mixed.** Environment resolution, the SDK and the metric schema duplicate Rust; the service lifecycle, query client and alerts are new. | **PORT-EXISTING:** `Settings::from_env` and the metric declaration table. **PORT-LIB:** query, alerts and lifecycle. Delete the Python SDK. | reqwest (in tree), zbus_systemd; **pyo3-opentelemetry** if the shell emits spans |
| 16 | **Docs, ADR, knowledge, workspace tooling** | 5.9K | Front matter, Markdown and lifecycle models feeding xtask's knowledge store (Rust → JSON → Python → JSON → Rust). Vocabularies are hand-copied from Rust enums. xtask's `doc-lint` is a stub. | **Developer tooling + duplicate** | **PORT-EXISTING** into an xtask document model | **comrak** or **pulldown-cmark**, serde-saphyr (in tree), toml_edit, gix |
| 17 | **Agent-runtime glue and semantic navigation** (`codex_*`, `session_retrospective`, `semantic_*`, `structural_migrations`, hooks/config) | 3.5K | Codex app-server glue, a Codex session scanner, and an mcpls LSP bridge served through the Python MCP server. The edit hook starts an interpreter on every agent edit. | **Developer tooling, not product** | **Move out of the repo or delete.** Generate the hook's deny-list from the codegen roots. | — |
| 18 | **Scientific oracles, parity and the consumer edge** (`*_reference.py`, IDAES, teqp, scipy, Decimal; `python/pse/parity`; `_array`, `_transfer`) | 2.25K | Independent references and the IDAES 2.13.0 parity harness. Null-refusing zero-copy numpy. Observation of whether pyarrow extension types survive. | **Inherently Python.** Independence or the Python ecosystem is the point. | **KEEP-PY.** Couple through committed data files, not regex scraping of `.pse` fixtures. | — |

**Aggregate:**

| Character | LOC | Share | Clusters |
|---|---:|---:|---|
| Mirrors or wraps Rust | ≈65.6K | 41% | 1, 2, and the native-path half of 5 |
| Orchestrates Rust tools | ≈34.4K | 21% | 3, 7, 12, 13, 14 |
| Genuinely new control, observability, interface | ≈48.1K | 30% | 4, the rest of 5, 6, 8, 9, 10, 11, 15 |
| Developer and agent governance | ≈9.4K | 6% | 16, 17 |
| Inherently Python | ≈2.25K | 1.4% | 18 |

---

## 4. Extension vs genuinely new: what must survive the port

The "new" 30% contains real semantics that a port must preserve on purpose. It also contains machinery that
exists only because Python processes sit between Rust processes.

**Semantics to preserve in Rust (do not simplify silently):**

- **Host admission:**
  - cross-session lane admission and queueing;
  - timing and reference exclusion against builds;
  - the memory-pressure gate;
  - parking resident services;
  - charging capacity until drain is verified.
- **Database operator:**
  - credential minting and ROOT rotation;
  - layered readiness;
  - offline backup, restore and rebuild;
  - recovery qualification before restart-on-failure;
  - borrower-aware stop;
  - fenced fresh installation.
- **Fixture registry:** sole terminal-composer authority over reclaim (ADR-0174, still proposed), and
  freeze/resume after a signal.
- **Durable control:** the original deadline, the cancellation-unresolved identity and receipt matching.
- **Telemetry:** backend lifecycle with rollback, the alert rules, and the investigation vocabulary with
  estimator admission.
- **MCP:** progressive tool disclosure, guidance resources and edit-preview fences.
- **Assessment:** keep-going gates with receipt reuse, diff coverage, bisect with paired measurement, and
  authenticated active-diagnostic actions.

**Machinery that disappears inside a single Rust process (most of the code volume):**

- Freezing and verifying Python service code: `publish_generation`, `receiver_closure`, and the Rust check for
  a file named `surreal_server.py`.
- Drift guards between two separately built artifacts:
  - registry and schema fingerprints;
  - catalogue-equality checks;
  - 13 `stale_interface` refusal paths;
  - AST walking to fence loaded sources;
  - `GENERATED.sha256` and `python-contracts-check`;
  - the governance lint;
  - the codec's header walk.
- Cancellation-clock hand-offs and fence/composer acknowledgements between interpreter hops.
- Text protocols:
  - `systemctl show` key=value parsing;
  - stripping `surreal sql` REPL prompts;
  - re-adding `<decimal>` casts lost over JSON;
  - Arrow sent as base64 inside JSON.
- Re-derived state: cgroup paths, CPU and test widths, Cargo configuration, build phases, the Rust test-module
  layout and the cargo/nextest flag grammars.
- Mock-choreography tests of subprocess surfaces, for example 12.1K lines and 1,006 `patch.object` calls in
  `test_surreal_server.py`.

---

## 5. Where Python costs time today

| Path | How often | Python cost | Evidence |
|---|---|---|---|
| Compiler wrapper (`RUSTC_WRAPPER` and CMake launcher → `compiler_cache.py`) | Every rustc unit (913 packages in the lock) and every C/C++ object in KLU, ibex, PETSc and Uno | About 110 ms interpreter + imports and about 90 ms SHA-256 of the sccache binary per 30 MB (*Measured, sandbox*). Plus about 4 `systemctl` calls, 4 ledger lock-and-rewrites and at least 50 ms of sleep. Total ≥250 ms per unit (*estimate*). It is on the critical path for invalidated workspace crates. | Slice C §2.3; `build_environment.py` sets `RUSTC_WRAPPER=scripts/sccache` by default outside CI |
| Recipe environment (`scripts/pse-env` → `pse_env.py`) | Every workload recipe (131 justfile uses) | About 177 ms per `pse_env.py` (vs 16 ms for a bare interpreter); 2 interpreters, or 3 with `--native`; about 12 + k `systemctl`/`systemd-run` spawns, where k is the number of registered units machine-wide. About 0.5–1 s per recipe (*estimate*). | Slice C §2.2 |
| Fixture registry bridge | At least 2 per canonical Rust test | 0.1–0.15 s import per call, loading 29 `scripts.*` modules including the 11.7K-line supervisor (*Measured, sandbox*) | Slice D §1 |
| MCP native operations | Every dedicated read, every new context, every CLI call | At least 2 interpreter starts plus `import pse`; about 5 transcodings each way; 2–3 SHA-256 passes per request | Slice E1 §3 |
| `import pse` | Every Python process | `contracts.documents` about 52 ms, `authored` about 95 ms, `enums` about 42 ms; first import about 0.9 s (*Measured, sandbox*) | Slice A §6 |
| Store supervisor | Service start, `mcp-headers` (×2), each worker launch (2 interpreters) | About 120 ms warm / 310 ms cold to import the closure (*Measured, sandbox*) | Slice B §2.6 |
| Agent edit guard | Every agent file edit | A fresh interpreter | Slice E2 §3.3 |
| Numerical solve path | — | **None.** No Python callbacks; numpy only at `_array`. | Slice A §6 |

---

## 6. Target architecture

```
┌───────────────────────────── Python thin shell (`pse`) ─────────────────────────────┐
│ Native pyclasses re-exported · generated stubs · pyo3-arrow capsules · numpy edge    │
│ async via pyo3-async-runtimes · trace context via pyo3-opentelemetry                 │
│ Parity harness (IDAES/Pyomo) · scientific oracles · future GUI / integrations        │
└──────────────┬──────────────────────────────────────────────────┬────────────────────┘
               │ in-process (cdylib)                               │ out-of-process
┌──────────────▼──────────────── Rust product core ────────────────▼────────────────────┐
│ existing crates (modeling → compiler → math → backend-native → runtime → operations)  │
│ ONE typed, dispatchable operation catalogue  ──►  PyO3 methods · rmcp tools · clap     │
│                                                    (optional tonic/axum for a GUI)    │
└────────────────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────── Rust host plane (new `pse-host`) ─────────────────────────┐
│ pse-env (lean binary) · admission daemon · operation lifetime (SupervisedChild)        │
│ store supervisor (external patched SurrealDB) · telemetry backend lifecycle/query      │
│ fixture/resource registry · systemd via D-Bus · Type=notify services · no Python       │
│ in any ExecStart                                                                       │
└────────────────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────── Dev plane (xtask + nextest) ──────────────────────────┐
│ nextest owns inventory/scheduling/JUnit · `xtask test` / `assess` / evidence producers │
│ native-provider prep · producer capture · docs/ADR/knowledge model                     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

**Principles:**

1. **One catalogue, generated faces.** The Rust operation declarations become typed and dispatchable: headers
   derive `Deserialize`, and the receiver is a function, not a display string. PyO3 methods, MCP tools and CLI
   commands are generated or registered from that single source. This retires the four-layer project-operation
   path, the Python dispatch table and the Cyclopts codegen.
2. **Python never mirrors serde.** Rust types are exposed directly. serde, nutype and `Version<N>` are the only
   validators, and the Rust types own finiteness, which today depends on a Python `_check_finite`.
3. **No Python in a service, a compiler path or a test-fixture path.** Every `ExecStart`, `RUSTC_WRAPPER`, CMake
   launcher and fixture bridge is a Rust binary or the tool itself.
4. **Use the tool's own feature before writing code.** This applies to nextest groups, setup and wrapper
   scripts, sccache's service mode, systemd properties and signals over D-Bus, OpenObserve's own alert
   scheduler, and Cargo timings.
5. **Python stays only where independence or the Python ecosystem is the point.** That covers the oracles,
   parity, the numpy/pyarrow consumer edge, and GUI and integration consumers.

**On the shell and a future GUI.** The shell is `import pse` (PyO3) for in-process scientific work. A GUI or an
external system that should not link the scientific runtime can use the same catalogue over MCP, or over a
tonic/axum service when needed (both are already in the dependency tree). That leaves the choice of GUI
language open, so it does not have to be Python.

---

## 7. Library map: the most targeted Rust route per concern

Status column:
- **verified** = checked this session on docs.rs, the vendor docs or nexte.st;
- **owner doc** = taken from `docs/libraries/`;
- **in tree** = already a workspace dependency.

| Concern (Python today) | Targeted Rust route | Why this rather than hand-written code | Status |
|---|---|---|---|
| Exposing Rust types to Python (51K mirror, 5.7K facades) | **Retarget pse-codegen** to emit frozen `#[pyclass]` wrappers (`struct X(Arc<T>)`, getters, `#[new]` via serde, `__eq__`/`__hash__` from Rust). Use complex pyclass enums for the 64 `serde(tag=…)` unions. Stubs from pyo3-introspection. **Alternative:** `pythonize` (serde ↔ Python objects) + generated TypedDict stubs. **Fallback stub tool:** `pyo3-stub-gen`. | The generator already enumerates all 260 catalogue roots and emits Rust registration, so this replaces Python emission rather than adding a system. pythonize is near-zero code but loses attribute access and immutability; Plan 57i declined it "without measured need". | PyO3 0.29.2 in tree; pythonize 0.29.0 targets PyO3 0.29 (owner doc); pyo3-stub-gen 0.23.1 supports PyO3 <0.30 (verified) |
| Trace context from Python into Rust (Python OTel SDK, 645 lines + deps) | **`pyo3-opentelemetry`** + **`pyo3-tracing-subscriber`** | Purpose-built: attaches the caller's Python OTel context to Rust `tracing` spans through a macro. Python then needs only `opentelemetry-api`, not a second SDK and exporter. | 0.11.1, PyO3 ^0.29, 2026-10-08 (verified) |
| MCP server/client (20.9K) | **`rmcp`** (official Rust MCP SDK): `#[tool]` routers with schemars schemas; `ServerHandler` with dynamic tool lists and list-changed notifications; per-request `CancellationToken`; progress notifications; child-process and streamable-HTTP **client** transports to keep proxying mcpls and SurrealDB MCP | A protocol runtime plus tool schemas derived from the same Rust types. It removes the worker subprocesses, the pydantic→msgspec→serde chain and the coherence layer. | owner doc (3.7.0). Plans 52p/55 declined it under a different driver; 52p's own revisit trigger ("can retire a complete current adapter path") is met by the native path. |
| CLIs (generated Cyclopts 1.2K, capability CLI 0.5K, argparse CLIs) | **clap builder API at runtime** from the catalogue, following the `pse-project --request` precedent | No codegen; calls handlers in-process instead of spawning an MCP server per command | in tree |
| systemd units, scopes and properties (14 `systemctl` call sites in `scripts/*.py`, about 35 calls through the supervisor's wrapper, `systemd-run` in 5 modules, 50 ms poll loops) | **`zbus` + `zbus_systemd`** (systemd1 Manager/Unit proxies generated from systemd's D-Bus definitions): `StartTransientUnit` (a scope with the caller's own PID, then `exec`), Start/Stop/KillUnit, `ControlGroup`/`InvocationID`/`ActiveState` properties, `JobRemoved`/`PropertiesChanged` signals | Typed calls with no process spawn per observation and no text parsing. Signals replace polling. It also removes the second `pse_env.py` interpreter and the `set-property` round trips. | zbus_systemd 0.26201.0, 2026-10-21, zbus ^6 (verified). Method coverage is inferred from generation, not inspected. |
| Service readiness and structure (polled readiness; resident Python `_serve` parent) | **`sd-notify`** (`Type=notify`) + **`tokio-graceful-shutdown`** (subsystem tree, signal handling, shutdown timeouts, error propagation to the exit code) | The standard daemon skeleton for the store supervisor, admission daemon and telemetry service | sd-notify 0.5.0, tokio-graceful-shutdown 0.20.0 (verified) |
| Process groups, pidfds, cgroup events, PSI (`native_operation.run`, `compiler_cache`, `_diagnostic_query`, `pressure_observation`) | **Your `pse-diagnostics::SupervisedChild` + `process_facts`**; rustix (pidfd, flock, inotify on `cgroup.events`); procfs PSI. Add `process-wrap` only if `SupervisedChild` needs session or kill-on-drop composition. | Most of this is already owned in Rust | in tree; process-wrap 10.0.1 (verified); rustix inotify/affinity feature coverage still to verify |
| Compiler cache (720 lines plus a per-unit tax) | **sccache's own service mode** (`SCCACHE_SERVER_UDS`, idle timeout) as a templated user unit with `Slice=`/`MemoryMax=`; `RUSTC_WRAPPER` points at the real binary | Configuration, not code | — |
| Database admin, backup, schema (HTTP `/sql`, REPL scraping, export/import CLI) | **The existing `pse_operations` gRPC client** + `canonical_files::offline`. The **SurrealKit Rust library** (`Sync`, `Rollout`, `seed`, `embed_schema!`) only if rollouts become production. **`surrealdb` `kv-surrealkv` embedded** only for the disposable evidence/knowledge index. Not `surrealdb-server`. | Typed values and one transport. Embedding fits a single-writer index, not the shared canonical store (ADR-0214). | SurrealKit library 1.0.0-beta.1 (verified); `surrealdb-server` "should not be used outside of SurrealDB itself" (verified); embedded engines take an exclusive lock per directory (owner doc) |
| OTLP capture sink (hand-rolled HTTP/2 + HPACK + protobuf walker) | **tonic + `opentelemetry-proto`** `TraceServiceServer` | Both families are already in the lock | crate features to verify |
| Test scheduling and orchestration (5.5K runner + 3.2K registry) | **nextest configuration:** test groups and overrides (replacing `test_requirements`); **setup scripts** (capsule capture, store unpark, worker build, environment via `$NEXTEST_ENV`); **wrapper scripts** (systemd placement); `archive.include`; `rdeps()` filtersets; record/replay. **`nextest-metadata`** reads list and run summaries. | nextest already owns inventory, scheduling and JUnit; stop re-deriving them | Setup and wrapper scripts are **experimental** (since 0.9.98, verified). There is no teardown hook, so reclaim stays runner-owned. |
| Case catalogues (`case_measure`, `pipeline_review` loops) | **`libtest-mimic`** (`harness = false`): each catalogue case becomes a nextest test. Read criterion's `estimates.json` instead of re-computing from `raw.csv`. | Gives filtersets, groups, timeouts, JUnit and rerun for free | nextest documents custom-harness support (verified) |
| Affected-test selection (`affected.py`) | cargo_metadata (in tree) + a `gix` diff → nextest `rdeps()`; guppy/determinator only if rules aware of `Cargo.toml`/`Cargo.lock` changes are needed | Small | determinator 0.12.0 was last released 2023-06-26 (verified), so prefer the simple route |
| Doc, ADR and knowledge model (5.9K; markdown-it-py, ruamel) | **`comrak`** or **`pulldown-cmark`** (tables, front matter) + serde-saphyr (in tree) + `toml_edit`; `gix` for immutability and history checks | Gives xtask's stub `doc-lint` a home and removes Python from the docs bootstrap | established crates |
| Wire-table codegen (Python regex over prost output) | **`syn`** (in tree) or `prost-reflect` | A Rust generator should not regex-parse Rust through Python | in tree |
| Telemetry backend (query 0.84K, alerts 0.55K, service 1.2K) | reqwest (in tree) for OpenObserve `_search` SQL, PromQL and the alerts v2 API; zbus_systemd for the units. Decide whether otelcol stays: OpenObserve ingests OTLP directly, but the collector's queue feeds two alerts. | Plain HTTP/JSON | — |
| Atomic publication and content-addressed receipts (5 implementations) | One Rust module: `tempfile` persist + blake3 (in tree) + `std::fs::File::lock` (stable since Rust 1.89); `cacache` if a general content-addressed store is wanted | Consolidation | — |
| Durable control without loading the native extension (`control.py`) | A second minimal PyO3 cdylib that reuses the `canonical_control` types and refusal predicates over reqwest/hyper, with no scientific crates linked. Or keep Python. | Removes the second validator of one server transition | open decision under ADR-0232 |

**What carries most of the subsumption is Rust you already own:** `SupervisedChild`, `process_facts`,
`canonical_files::offline`, the gRPC client, `ProductionProfiles`, `tooling::telemetry::{Settings, metric_declarations}`,
`producer_identity.rs`, `dependency_ceilings.rs`, `development/{evidence,knowledge}.rs` and `scientific_inputs.rs`.
Several Python modules are second implementations of these. The port is largely a consolidation onto existing
owners, with the libraries above covering the genuine gaps.

---

## 8. What stays Python

| Keeps | Why |
|---|---|
| The `pse` package shell (re-export of the native module, published-image loader, lazy import), about 0.5K handwritten lines plus generated stubs | Python entry point for users, notebooks, a GUI and integrations |
| `_array` (null-refusing, zero-copy numpy) and `_transfer` (observes whether pyarrow extension types survive) | Pyarrow and numpy consumer behaviour cannot be observed from Rust through the C data interface |
| IDAES/Pyomo parity harness and its tests | The sanctioned clean-room coupling; IDAES exists only in Python |
| Scientific oracles (scipy, teqp, Decimal, IDAES), about 1.8K lines | Their value is independence from the product implementation. They should write committed data that Rust tests consume. |
| pytest `conftest.py` (shrinks), the `reuse` lint wrapper, optionally the HTML evidence report | Python-ecosystem tools and presentation |
| `structural_migrations.py` until its target Python is gone | It rewrites Python source |

---

## 9. Sequencing (*Proposed*)

Effort is in the reviewers' units: S = days, M = 1–2 weeks, L = more.

**Wave 0. Delete; no port needed (S overall; about 14K production lines plus about 10K test lines and 24K data
lines).**
- the 7 generated modules nobody consumes;
- the attrs row modules `runtime`, `reference`, `normalized`, `provenance` and `canonical` (6.3K lines; no
  production consumers; tests only);
- `check_generated.py` (duplicates the xtask manifest writer);
- the SurrealKit beta cluster and the stale `database/canonical-schema`;
- `resource_conversion.py`;
- `cargo_freshness_probe.py` (no caller);
- `native-engine-boundaries.py` (express it as dependency ceilings instead);
- `build_observation.py` (use Cargo timings);
- `test_requirements.py` (becomes static `nextest.toml` overrides);
- `grpc_wire_schema.py` (becomes `syn` in xtask);
- `python/stubs`, `IDAES_NAMES`;
- Codex glue: move it out of the repo.

**Wave 1. Take Python off the hot paths (S–M; highest return per line).**
1. Run sccache as a user service with `RUSTC_WRAPPER` set to the real binary, which removes `compiler_cache.py`
   from every compile.
2. Build a lean standalone Rust `pse-env` with zbus_systemd transient scopes and `exec`. It needs a bootstrap
   answer: plain `cargo build -p pse-env` with no wrapper.
3. Move `arrow_validation` into xtask with its resolution cached by manifest and lock hash; `cargo c`/`cargo t`
   dispatch to Rust.
4. Generate the agent-hook deny-list from the codegen roots, or compile the guard.

**Wave 2. Product boundary (M then L; ADR on the Python-boundary contract).**
1. Native public types (Plan 57i I02/I03), which deletes the facades, the export gate and the Python
   project-operation layer (about 5K lines).
2. Retarget pse-codegen to emit pyclass wrappers (or use pythonize), which deletes the document mirror, codec,
   governance, fingerprints and the Python emitters. That is about 50K Python lines and about 3K Rust generator
   lines.
3. Move the CLIs to a clap binary built on `SupervisedChild`.
4. Add `pyo3-opentelemetry`, move `Settings::from_env` into Rust, and delete the Python OTel SDK (this needs
   the operations-and-validation amendment).
5. Decide ADR-0232's control client.

**Wave 3. Host plane in Rust (L; ADRs amending §20.6/ADR-0229 and preserving ADR-0174 semantics).**
- These modules move as one unit because they are tightly coupled: `surreal_server`, `host_admission`,
  `native_operation`/`operation_lifetime`, `test_resources`/`resource_store` and the telemetry service.
- Build a `pse-host` crate containing the admission daemon (pidfds, cgroup events, D-Bus), operation lifetime
  on `SupervisedChild`, the store supervisor (`Type=notify`, gRPC admin, in-process offline payloads), the
  telemetry lifecycle and query client, and the fixture registry in `pse-testkit`.
- This deletes the Rust→Python inversions.

**Wave 4. One catalogue, three faces (M; ADR superseding the 52p/55 rmcp disposition). Can run in parallel with
Wave 3.**
- Unify the 21 host operations and the 49 `ProjectAction`s into one typed, dispatchable catalogue.
- Build an rmcp server, plus a clap CLI from the same catalogue.
- In the interim, the Rust server proxies a slimmed FastMCP child for tools whose owners are still Python,
  the same way the current server already proxies mcpls and SurrealDB.

**Wave 5. Dev plane (M–L).**
- `xtask test` over nextest (nextest-metadata, setup and wrapper scripts).
- `xtask assess` (blake3 source identity, shared with `producer_identity`).
- Evidence producers in xtask; case catalogues as libtest-mimic tests.
- The xtask document model for ADR, docs, lifecycle, register and knowledge.

---

## 10. Risks and decisions for the maintainer

1. **Governance.** These targets reverse current decisions, so they go through the ADR route:
   - **Python-boundary contract:** documents as pyclasses or pythonize instead of the msgspec mirror.
   - **New crates:** `pse-host`, `pse-env` and an MCP binary.
   - **§20.6/ADR-0229:** systemd realization is currently assigned to Python.
   - **Operations-and-validation:** "Python owns environment resolution" for telemetry.
   - **Plans 52p/55:** their rmcp disposition.
   - **ADR-0232:** the Python-only control client.
2. **Preserve semantics; delete only plumbing.** §4 separates the two. Collapsing the fixture registry into
   `Drop` + `thread::panicking()` would be a design change, not a port.
3. **Experimental nextest features.** Setup and wrapper scripts are experimental. Exact pins (ADR-0165) make
   this acceptable, and anything they cannot do (teardown) stays runner-owned.
4. **Bootstrap ordering.** The environment boundary configures Cargo, so the Rust `pse-env` must build without
   itself. Today's bootstrap already fails with a stdlib-only interpreter (`ModuleNotFoundError: msgspec`).
5. **Coverage during transition.** The service-level acceptance harnesses (telemetry 6.5K, MCP 4.7K and the
   SurrealDB checks) are black-box over real binaries. They can serve as oracles against the Rust replacements
   and then retire. Unit tests are replaced, not ported.
6. **Expectations on performance.** The gains are in the build, test, agent and control-plane loops and in
   removing consistency machinery. Solver numerics do not change. The levers from the 2026-10-20 performance
   review (admission, registry, preparation) are Rust-side and independent of this work.
7. **Stop the inflow.** `control.py` arrived today under ADR-0232, which shows new capability still defaults to
   Python. A governance check that refuses new Python modules outside the shell, parity and oracle directories
   would hold the line while the waves land.

---

## 11. Sources

- PyO3 type stubs and introspection: <https://pyo3.rs/v0.30.1/type-stub>; owner doc `docs/libraries/pyo3.md`
- pythonize: <https://docs.rs/crate/pythonize/latest>
- pyo3-stub-gen: <https://docs.rs/pyo3-stub-gen>
- pyo3-opentelemetry: <https://docs.rs/pyo3-opentelemetry>
- rmcp: <https://docs.rs/rmcp/latest/rmcp/>; owner doc `docs/libraries/rmcp.md`
- zbus_systemd: <https://docs.rs/zbus_systemd>, <https://github.com/lucab/zbus_systemd>
- sd-notify: <https://docs.rs/crate/sd-notify/latest>
- tokio-graceful-shutdown: <https://docs.rs/tokio-graceful-shutdown>
- process-wrap: <https://docs.rs/process-wrap>
- nextest setup scripts: <https://nexte.st/book/setup-scripts>
- nextest wrapper scripts: <https://nexte.st/docs/configuration/wrapper-scripts/>
- nextest test groups: <https://nexte.st/book/test-groups>
- nextest custom test harnesses: <https://nexte.st/book/custom-test-harnesses>
- nextest-metadata: <https://docs.rs/nextest-metadata>
- libtest-mimic: <https://docs.rs/libtest-mimic>
- determinator: <https://docs.rs/crate/determinator/latest>
- SurrealKit as a library: <https://surrealdb.com/docs/manage/schema-migration/library.md>
- surrealdb-server: <https://docs.rs/surrealdb-server>
- SurrealDB embedding: <https://surrealdb.com/docs/reference/rust/embedding>; owner doc `docs/libraries/surrealdb.md`
- OpenObserve search API: <https://openobserve.ai/docs/reference/api/search/search/>
