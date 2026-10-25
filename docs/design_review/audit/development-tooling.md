# Development tooling

The maintainer put development tooling in full audit scope for this refresh. It covers:
- the FastMCP capability server (`scripts/capabilities/`);
- recipes and `scripts/pse-env` / `scripts/pse_env.py` (host admission, capacity, supervised
  scopes);
- the substrate supervisor;
- `xtask`;
- `pse-runtime::tooling`;
- agent hooks;
- generated developer knowledge;
- evidence tooling outside the [test account](../test-audit/README.md).

This page holds causes specific to that layer. Causes that cross the product/tooling boundary
keep their homes:
- [DA-019](boundaries-and-composition.md#da-019): the deployment contract in tooling;
- [DA-020](boundaries-and-composition.md#da-020): tooling families in product crates;
- [DA-024](local-reasoning-and-assurance.md#da-024): the producer chain without a consumer.

The [integrated account](system-architecture.md#the-development-tooling-layer) discusses the layer
as a whole. Citations are `path::symbol` at `b5bdac9da`. Each *2026-10-25 refresh* note says what
changed since `cbf45ccc9`.

## The layer in brief

The tooling is a second system of comparable weight. At `b5bdac9da` it has:
- about 91k non-test lines of scripts, up from 76k. The growth is Plan 52 telemetry services,
  the Plan 55 capability plane and Plan 54 build tooling.
- a FastMCP capability plane of about 14.6k lines in 57 modules, up from 27 modules;
- about 20k lines of `xtask`;
- about 27k lines in `pse-runtime::tooling`, which now includes product observability
  ([DA-020](boundaries-and-composition.md#da-020));
- native Collector and OpenObserve user services for telemetry.

Inside the layer, composition is often genuinely library-based:
- FastMCP visibility rules, transforms, middleware and `Client`;
- systemd scopes, slices and limits for enforcement;
- `pyo3-introspection` for stubs;
- `just --dump` for recipe truth.

The native schemas in the endpoint are consumed from the generated catalogue, not mirrored.

The defects concentrate in two places:
- the boundary with the product (DA-019 and DA-020);
- general tooling meanings that are restated per owner (below).

## <a id="da-036"></a>DA-036 — Execution-resource and workload meaning has no single authority; a test manifest and agent host policy decide production budgets

**Principles:** AP-04, AP-05, DP-01, DP-20; H21, H25; G1.

**Refresh status:** new. It overlaps [DTA-045](../test-audit/execution-efficiency.md#dta-045) and
[DTA-046](../test-audit/execution-efficiency.md#dta-046), which concern test scheduling and
simultaneous demand. This finding concerns authority.

**2026-10-25 refresh: some literals narrowed; the structure is unchanged, and Plan 54 exercised
it.**
- *Narrowed.* Several fixed numbers became derivations:
  - `setup` takes its server default from `host_admission.settings("functional")["store_gib"]`;
  - `server_environment` derives store thread counts from `host_admission.select(...)`;
  - `build_environment.native_compiler_jobs` derives native-library parallelism from
    `.cargo/config.toml [build] jobs`;
  - fixed `worker_threads(2)` sites were removed.
- *Unchanged.*
  - `reference_execution` and `reference_resources` still hard-code the 12 GiB headroom, the
    4 GiB observer and the 16 GiB server. They duplicate `[reference]`, whose toml header says
    these values are "never redeclared here".
  - The error text "140GiB primary and 160GiB envelope" remains.
  - `pse-worker` qualification still demands 128 GiB, 16, 16 and 32.
  - The production pool still comes from the conformance manifest.
- *A new coupling.*
  - `setup` keeps the literals `49152` MiB total and `32768` MiB worker.
  - The 16 GiB store change (`1787c2029`) fits only because 16 + 32 = 48, and its commit message
    does that arithmetic by hand.
  - Raising `[functional].store_gib` above 16 now makes ordinary setup refuse unless both
    literals change too.
  - That 48 GiB setup envelope also differs from `execution_resources("functional")`, which is
    56 GiB.
  - The coordinator re-read both literals.
- *Static class hints grew.*
  - The `justfile` has 73 `--recipe-default-class` sites, up from 64.
  - `capabilities/workers.py::WORKERS` adds 3 class choices.
  - The 32-logical-CPU default (`dbd959332`) was stated across 22 files.
  - "16 lanes" is stated in at least seven places.

**Diagnosis.** The meaning of "how much memory, how many lanes and which exclusivity a workload
gets" is decided in at least seven places, several of them deciding the same numbers:
1. `.config/agent-capacity.toml`, the agent host policy, defines classes, partitions and
   `[reference] store_gib`, `receiver_headroom_gib` and `observer_gib`.
2. `scripts/surreal_server.py::{reference_execution, reference_resources}` hard-codes the same
   values (16 GiB server, 12 GiB headroom, 4 GiB observer) and enforces `case_lanes == 16`.
   `scripts/host_admission.py` reads the toml for them, and its error text hard-codes
   "140GiB primary and 160GiB envelope".
3. The **production** 16-lane pool (memory, threads, math jobs) is derived from the reference
   conformance campaign's manifest, `packages/reference/conformance.toml [settings]`.
4. `pse-runtime/src/workflow/native_deployment.rs::NativeAllocation::validate` re-imposes the
   relational rules.
5. `xtask/src/bin/pse-worker.rs::{resource_budget, verify_placement}` re-checks cgroup quotas,
   with "sixteen case lanes" in its error text.
6. `scripts/capabilities/native.py::Resources` defaults to 16 GiB, 16 threads and 1 GiB of spill.
7. Recipe classes are chosen statically in three ways:
   - 73 `--recipe-default-class` sites (64 at `cbf45ccc9`);
   - name heuristics in `host_admission.classify`;
   - the maintainer's local `PSE_BUILD_MODE=unbounded` overlay, which remaps them
     (`pse_env.py`).

**Consequence.** For the scenario "move to a host with different RAM or cores, or resize the
16-lane pool":
- The edit spans the toml, the conformance manifest, supervisor constants, Rust validators and
  error text.
- A partial edit silently decouples admission accounting from the systemd `MemoryMax` that the
  supervisor applies.
- Changing a test campaign's threads or memory changes the production worker's pool.
- For callers the class is a remembered convention. AGENTS.md carries a gotcha about not passing
  `--resource-class compile`, and the effective behavior in the maintainer's mode differs from
  the declared default.

**Counterevidence.**
- Admission exists for real reasons: the recorded OOM incidents and concurrent agents.
- Deriving the reference profile from the conformance manifest avoids a second numeric
  declaration.
- Static per-recipe classes are simple, and the unbounded overlay is a deliberate maintainer
  selection.
- What these do not justify: the same numbers living in toml and literals, and test-campaign
  settings acting as the production profile's authority.

**Evidence:** *Implemented* for the duplicated literals and derivations. The `default` →
`functional` classification was read, not executed.

**Improvement direction.**
- One owned declaration of execution profiles and workload classes, from which admission, the
  supervisor, runtime validation, the worker and MCP defaults derive.
- The product's durable profile separated from test-campaign settings.

## <a id="da-037"></a>DA-037 — Process-ownership primitives are re-authored per tooling owner with divergent semantics, inside one cyclic import web that includes the production supervisor

**Principles:** AP-06, DP-01, DP-13, DP-16; H4, H25, H26, H27.

**Refresh status:** new. It is the production-side counterpart of
[DTA-026](../test-audit/test-execution-system.md#dta-026), whose test-side cycles are not
duplicated here.

**2026-10-25 refresh: core unchanged; widened by new owners; one correction.**
- **No liveness primitive changed.** `require_borrowers_drained` still consults test resources,
  and `capabilities/specialists.py` still imports `surreal_server` at module level.
- **Three separately authored owned user-systemd service realizations now exist:**
  - `surreal_server`, for the store unit;
  - `compiler_cache.unit_text` and `registered`;
  - `telemetry_service._unit` and `_checked_process`.

  They differ in restart policy, admission participation and incarnation rule.
- **Other re-authored primitives.**
  - `telemetry_service` has its own `.lifecycle.lock` flock, and its `_systemctl` copies the
    supervisor's DBUS fallback.
  - `codex_mcp.process` parses `/proc/<pid>/stat` for ancestry.
- **A better rule now exists.** `telemetry_service._checked_process` binds systemd
  `InvocationID`, `/proc` start ticks, executable inode and cgroup, which matches the Rust owner
  identity. It is a candidate for the one rule.
- **Correction to the previous account.** S2 and S5 each ran an `ast` import-graph probe that
  counts function-local imports. The runtime strongly connected component had 26 modules at
  `cbf45ccc9` and has 27 now (`doctor` joined). The six modules named below are its core, not
  its extent. The transitive reach of `capabilities.server` grew from 82 to 120 modules, still
  including `surreal_server`, `test_resources` and `host_admission`.
- **Counterevidence.** The new `telemetry_service`, `capability_cli*` and `codex_mcp` modules sit
  outside the cycle, with a reach of 1–2 modules each.
- **Light recipes.** About 24–25 of about 215 recipes are marked light (S5 and the challenger
  counted differently). The Diagnosis's earlier "23 of about 240" is superseded. Light still
  takes an allocation and a `systemd-run`.

**Diagnosis.**
- **"This recorded owner is still alive" has at least four realizations:**

  | Realization | Rule |
  |---|---|
  | `native_operation.start_identity`, `host_admission.caller_alive`, `test_resources.borrower_alive` | Boot id plus `/proc` start tick; zombies count as alive |
  | `surreal_server.reservation_live` | Adds zombie, dead and "unresolved" states |
  | `payload_files.owner_dead` | Adds the PID namespace; raises on uncertainty |
  | `pse-operations/src/canonical_files.rs` (Rust) | Adds invocation and cgroup |

- **Private JSON-plus-`flock` ledgers are re-authored per owner.** They cover host admission,
  `resource_store`, the supervisor's state lock and lifecycle owner, context admission, tooling
  activities, payload instances, native operations, the compiler cache and producer deployment.
- **Drain observation is implemented separately in four modules.**
- **These owners form one strongly connected import component:**
  `pse_env` ↔ `host_admission` ↔ `native_operation` ↔ `surreal_server` ↔ `test_resources` ↔
  `resource_store`.
  - The cycle is broken by in-function imports and a `sys.modules` alias
    (`surreal_server.py` ≈45–50).
  - The production supervisor consults the test-resource registry before stop and drain
    (`require_borrowers_drained`).
  - The MCP endpoint imports the supervisor (`capabilities/specialists.py`).
- **Every top-level recipe line pays the full lifecycle unless explicitly marked `light`**
  (23 of about 240 recipes are). That lifecycle is a ledger lock and admission, parent and
  allocation enforcement, `systemd-run --scope`, then a second interpreter
  (`pse_env.py::placement`, `main`).

**Consequence.**
- **No local reasoning (AP-06).** Changing one owner's lifetime rule means reading the whole
  cycle, which includes the production supervisor and the frozen generations that copy it
  ([DA-019](boundaries-and-composition.md#da-019)).
- **One owner, different answers.** A zombie launcher keeps host capacity in `caller_alive` but
  releases a lifecycle reservation in `reservation_live`. Only some owners authenticate the PID
  namespace. No failure has been demonstrated, but each new owner re-derives the rule.
- **Overhead on cheap commands.** Discovery and light recipes pay allocation, a scope and two
  interpreter starts (H26). This is unmeasured.

**Counterevidence.**
- Lifecycle safety needs coordination: crashes and PID reuse are real.
- Each extra condition protects something concrete.
- Extensive actual-process tests exist.
- `os.pidfd_open` is already used.
- No external library removes the domain conditions.
- The modules' own comments call the cycle "reciprocal ownership", so it is stated intent rather
  than an accident.
- What none of this justifies is re-authoring with drift, and a cycle that couples production
  supervision to test-resource internals.

**Evidence:** *Implemented* for the divergent implementations and the import web. The
per-command overhead and the practical incidence of divergence are hypotheses.

**Improvement direction.**
- One owner-identity and liveness primitive with one stated semantics (namespace, zombie,
  uncertainty, cgroup), shared by the Python owners and matching the Rust owner identity.
- Acyclic layering: environment → admission → service supervision → borrowers registering
  through a declared interface.
- A light path that creates no scope for work that needs none.

## <a id="da-043"></a>DA-043 — Every agent native call pays a full environment, currency, import and runtime lifecycle, and currency is re-derived from a tooling-authored input superset

**Principles:** AP-07, DP-09, DP-10, DP-13, DP-16, DP-20; H2, H12, H13, H14, H18, H26, H28.

**Refresh status:** new at 2026-10-25. S3 (execution fit) and S5 (tooling) reported it
independently. Challenges C1–C5 corrected and strengthened it. Its two halves have different
causes:
- the per-call lifecycle belongs to integrated cause 2: a fixed protocol paid per call rather
  than per workload;
- the currency half is a bespoke reuse key held by the tooling rather than the build tool (DP-09,
  DP-13). It resembles cause 5, and the coordinator does not force it under cause 2.

**Diagnosis.**
1. **Per-call lifecycle.**
   - Both capability dispatch routes spawn a fresh
     `scripts/pse-env [--native=…] -- .venv/bin/python -m scripts.capabilities.native_worker`
     for every call: `process.py::run_worker` and `native.py::run_child`.
   - `pse-env` places each command in a systemd scope.
   - Both routes set `PSE_CAPABILITY_ADMISSION`, and every `native_contracts.Binding` requires
     `python-extension`. So `native_worker.execute` calls
     `python_extension.currency_check_installed` on every call, reads included.
   - The worker then imports `pse` and constructs a `pse.Runtime`, durable when a deployment is
     named. The local working-tree extension is about 1.29 GB.
   - On the CLI route, `scripts/capability_cli_runtime.py::connection` first starts a fresh
     `StdioTransport(python -m scripts.capabilities)` per invocation. That launches the
     120-module endpoint before the native child. This is deliberate: 55a says "One invocation
     owns one fresh Client". The maintainer-selected foreground JSONL session (55d D03) keeps one
     endpoint alive, but the native worker is still spawned per call inside it.
   - With telemetry selected, each helper also bootstraps and shuts down the SDK providers.
   - Some operations repeat their admission per page:
     - `model_declarations` and `model_inspect` re-admit physical and modeling documents;
     - `evidence_compare` re-imports up to 64 snapshots and recomputes the pairing;
     - `result_inspect` reopens its file and protection.
2. **Currency from a superset with a deny-list.**
   - `preparation_inputs` and `currency_check` decide "is the installed extension current?". They
     hash every file in `validation_scope.INPUT_SCOPES["python-extension"]`: all of `crates/`,
     `vendor/` and `packages/`, the lockfiles and seven scripts. S3's and S5's probes counted
     about 10.26k files, about 147 MiB.
   - `preparation_inputs` → `file_identity` also reads and hashes the full bytes of the tool
     binaries: cargo, rustc, uv, cc and c++, pkg-config, clang, mold, maturin, and the toolchain's
     rustc and cargo.
   - When the inputs match, `currency_check` calls `capture()`. That spawns a *second* interpreter
     that imports `pse`, so the extension is imported twice per call. It then runs
     `hashlib.file_digest` over the entire extension binary, about nine times the source hash's
     bytes locally. The coordinator re-read `capture`.
   - That makes five subprocesses per call: `git ls-files`, the sysconfig probe,
     `rustc --print sysroot`, `rustc -Vv` and the `capture` import.
   - The check binds the whole process environment minus a hand-kept deny-list
     (`native_tests.RUST_EXECUTION_ENVIRONMENT`, `PREPARATION_SESSION_ENVIRONMENT`).
   - The `RUST_EXECUTION_ENVIRONMENT` deny-list grew from 20 to 39 names in this window. Each new
     tooling variable made the extension look stale until someone listed it:
     - Plan 52: `PSE_TELEMETRY_CARRIER`, `PSE_NATIVE_TELEMETRY_SETTINGS`, `OTEL_SDK_DISABLED`;
     - Plan 55: `PSE_CAPABILITY_ADMISSION`, `PSE_CAPABILITIES_*`;
     - pytest and xdist keys.
   - Three repair commits on 2026-10-25 were this symptom: `c05dc562b`, `97ce2efb1` and
     `4a706426a`.
   - Plan 54 adopted library reuse for Rust (Cargo `checksum-freshness`, sccache). The Python and
     native layer keeps a second, bespoke reuse key over the same artifact.
3. **Currency semantics differ by prerequisite kind** (challenge C5).
   - Knowledge reads run `target/debug/pse-evidence` (`Prerequisite(kind="executable")`).
     `evidence_store.run_native` checks only that it is a regular executable file. A same-schema
     edit to `pse-operations/src/knowledge.rs` is therefore served by a stale binary, which is the
     failure LIFE04 fixed for the extension.
   - Meanwhile `native_contracts.Binding.requirements` hard-codes `mode="native", validated=True`
     for every binding. That includes `evidence_compare`, which is supplied-only with no build.
     Test-evidence comparison therefore refuses after any `crates/` edit, which is exactly the
     edit-then-compare loop it serves.
   - §24.1 says the offline supplied-only path needs no product extension; the MCP route inherits
     the product gate instead (compare [DA-020](boundaries-and-composition.md#da-020)).

**Consequence.**
- A routine agent read pays env and scope admission, the whole-tree hash and subprocesses, an
  extension import and runtime construction. That fixed cost scales with repository and
  extension size, not with the request. Paging repeats it per page.
- After *any* edit under `crates/` or `vendor/`, every native read refuses
  `pse.native.artifact_not_current` until a rebuild. That includes edits that cannot affect the
  loaded image. For example, `crates/pse-study-module` has no dependents in `Cargo.lock`.
- The next tooling environment variable will invalidate native currency again (H27).
- Magnitude is unmeasured.

**Owner intent: the implementation diverges from its own recorded target.**
- §24.1 ("Native schema compatibility is distinct from artifact currency") requires preparation
  evidence, current supported inputs and "the witnessed image loaded by a fresh worker" to agree
  before a native receiver runs.
- Plan 52n LIFE04 also recorded the failure that motivated it, in which currency was borrowed
  from a different subprocess import. Its target says: "Do not run full preparation-input
  reconstruction, compiler/provider setup or a whole-repository hash for every catalogue/read
  call, or narrow dependencies through an unsound handwritten whitelist. If a sound cheap check is
  unavailable for a mode, disclose that limit."
- The Plan 52 execution-freshness amendment adds: "No universal fresh-child policy, per-read
  global scan or new freshness ledger is introduced."
- 52k MCP06 says not to add a "fresh process for every lightweight read".
- The LIFE04 Outcome (2026-10-24) does not disclose the limit. So the per-call whole-tree
  reconstruction contradicts the plan that introduced it, which strengthens this finding.
- The draft cited ADR-0219 here. That record governs evidence snapshots, not currency (C2).

**Counterevidence.**
- Conservative invalidation gives honest false-stale outcomes, never false-fresh ones.
- Cargo exposes no pure "is fresh" query.
- A fresh process isolates native crashes and signal/drain ownership.
- The parts already proportionate are a strength:
  - the image witness, a stat of device, inode, size and mtime, which now sits beside the full
    digest;
  - the startup-captured source cut, of which the child verifies only the selected operation's
    sources;
  - the light `prep-import` route.
- The contract requires agreement. It does not require a full re-hash per call or a whole-tree
  input superset.

**Evidence:**
- *Implemented* for the per-call route and the currency invocation; the coordinator re-read
  `native_worker.execute` and `currency_check_installed`.
- *Implemented* for the deny-list growth (the coordinator counted 20 and 39 names) and the
  repair history.
- *Implemented* for the second import and the full-binary digest. The CLI route, the
  prerequisite-kind differences and the plan texts are the challenger's reading at `b5bdac9da`.
  The coordinator re-read the LIFE04 sentence.
- *Interface-checked* for the probe sizes; cost is unmeasured.
- Two things would settle it:
  - a cold-call breakdown of one `native_result_read` and one `model_declarations` page (env,
    currency, import, runtime, work);
  - a check of whether the cdylib's rustc dep-info (source files and `# env-dep:` lines) and the
    build scripts' `rerun-if-env-changed` sets cover what currency needs.

**Improvement direction.**
- Currency is derived from the producing build's own dependency record and checked against an
  endpoint-held witness, not recomputed from a superset per call.
- Environment relevance comes from declared dependencies rather than a deny-list.
- Per-call cost is proportional to the request: a reused or pooled native worker within the
  existing drain and isolation contract, and pages served from one admitted preparation.
  - This stays clear of I10 unless compiler products are persisted.
  - It would amend §24.1's "fresh worker" wording and MCP06's per-owner selection.
- Currency semantics are consistent across prerequisite kinds. A supplied-only operation carries
  only the prerequisites it uses.

**Related:** [DA-030](execution-fit.md#da-030) (I10), [DA-036](#da-036),
[DA-015](boundaries-and-composition.md#da-015) (the hand-projected bindings that route here).
The [test account](../test-audit/README.md) holds no finding on the shared Rust test-observer
currency, which uses the same identity (`native_tests.rust_environment_identity`).
