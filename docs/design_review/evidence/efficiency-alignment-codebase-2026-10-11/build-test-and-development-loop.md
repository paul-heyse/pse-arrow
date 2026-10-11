# Build, test and development loop: bounded supporting assessment

Inquiry **G** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`G-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

Supporting assessment for the 2026-10-11 efficiency-alignment review. Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f`. Standard: Core 3.4 with Heuristics 1.0, ProcessSimulator 1.5 and the pse-arrow binding. I made no tracked edits and ran no builds, tests or services. The only commands I ran were read-only: `cargo tree`, `cargo build --unit-graph` (which resolves units without building), `jq` over local state files, a parse of the existing JUnit report, and one read-only call to `host_admission.classify` and `host_admission.select`. Local labels are G-01 to G-08.

## 1. Scope examined and coverage limits

**What I read.** I read these files directly:
- Configuration: `Cargo.toml` (members, profiles, pins), `.cargo/config.toml`, `rust-toolchain.toml`, `.config/{hakari,nextest,agent-capacity,build,sccache}.toml` and `crates/pse-workspace-hack/Cargo.toml`.
- Recipes: the justfile's check, unit, test, codegen, py-*, native-* and default recipes.
- Scripts: `pse-env` and `pse_env.py` (compose, placement, main), `host_admission.py` (select, classify, acquire, conflict, metadata), `test_resources.py` (register, resource_metadata, finalize, reclaim, reclaim_reports), `test_run.py`, `select.py` and `arrow_validation.py`.
- More scripts: in `native_cache.py`, the functions prepare, valid, installation_files, collect, klu, tool_identity, solver and seal_generation. In `native_operation.py`, the functions observe, pin, pinned_generations, write_json and Operation. Also `validation_scope.py` (input scopes), parts of `validation.py` (sources, provenance) and `validation_receipts._reuse_guarded`, plus the `surreal_server.py` functions ready, protocol_ready, context_admission and config_for.
- Rust fixtures: `crates/pse-operations/src/testing.rs` (fixture lifecycle and resource bridge), `crates/pse-runtime/src/workflow/{tests.rs,mod.rs}` (`runtime_on`, `canonical_deployment`, `Runtime::from_shared`) and the size of the canonical schema in `crates/pse-operations/src/generated/surreal.surql`.
- Python fixtures: `python/pse/tests/conftest.py::canonical_substrate`.
- xtask and build scripts: `xtask/{Cargo.toml,src/codegen.rs,src/codegen/physical.rs (imports),src/python_tests.rs,src/main.rs}` and the build scripts of `pse-buildinfo`, `pse-backend-native` and the `*-sys` crates (env tracking only).

**What I observed read-only.** The test-resource ledger (`~/.local/state/pse-arrow/test-resources`), the host-admission directory, the native cache (`~/.cache/pse-arrow/native`, including `.operations`) and the maintainer's JUnit report `target/nextest/default/junit.xml` (UUID `be5befc4…`).

**What I did not read.**
- Most of `surreal_server.py` (7.3k lines), `validation.py` beyond provenance, `native_tests.py` beyond capture, `build_measurements.py`, `compiler_cache.py` and `affected.py`.
- The `scripts/tests/` suite (63 files, about 35k lines), `pse-testkit`, `tests/{conformance,lifecycle,engine}`, the bodies of `producer_identity.rs` and the bodies of `*-sys` builds other than their env tracking.
- I checked the Python side only through `canonical_substrate`.

**Library documentation.** Statements about Cargo and nextest behaviour come from the pinned configuration and general Cargo semantics. I did not query Context7, which is a limitation for the G-07 mechanism in particular.

## 2. Complete operations examined

**O1: `just check-package <pkg>`.**
1. `scripts/pse-env` starts Python and composes the environment. Host admission classes the recipe as `compile`: lane `functional`, 1 slot, physical cores 8–11 (8 logical CPUs), `CPUQuota=400%` and 24 GiB. It launches `systemd-run --scope`, which re-execs Python with `--bind-allocation`.
2. `arrow_validation.resolve` runs `cargo metadata --all-features` and one or two `--unit-graph` resolutions, derives force-validate from the actual Arrow closure, then runs `cargo check` with `jobs=8`.
3. On exit, the allocation is released and `test_resources.reclaim_reports()` runs. This takes the exclusive test-resource ledger lock and parses and rewrites the whole ledger.

**O2: `just unit-package pse-runtime <filter>`.** This is the `functional` class, with the same 4-core lane.
1. `select.py` resolves the command. `test_run.run_rust(effects="canonical")` writes an invocation file and a regex-edited copy of `nextest.toml`, runs `cargo nextest list` (which builds), then runs `cargo nextest run` with 16 test threads.
2. Each test that constructs a Runtime does the following:
   - Builds a cold `SharedRuntime`, registry and session factory. In the maintainer's 16-thread run this took 1.09–4.05 s per test (median 1.31 s) before the store refusal.
   - Calls `canonical_fixture_options`. The Rust code then spawns `.venv/bin/python -m scripts.test_resources register` and polls it every 10 ms.
   - In Python, `register` reads `config_for`. While admission is open it calls `surreal_server.ready`: a `/proc/<pid>/fd` scan, an HTTP `/health` request, `systemctl show` and a digest of the credentials file. It may also call `surreal_server.start`. It then enters `context_admission` and does a full-ledger exclusive rewrite.
   - The test connects over WebSocket and installs the complete schema with `store.create()` (479 DEFINE statements, about 107 KB of SurrealQL).
   - The test body runs.
   - On drop, a second Python subprocess handles `drain`: a status read and rewrite, `drain_context` and `record_drain` with another rewrite.
3. The runner composes the JUnit report and calls `finalize` (one rewrite). Then, for each passing fixture, `reclaim` does a status rewrite, `context_admission` and another rewrite, spawns a `surreal sql` process running `REMOVE DATABASE`, and does a final rewrite.
4. The `pse-env` exit then calls `reclaim_reports` (another rewrite).

**O3: `just py-test`.**
1. `pse-env --store` (`require_store`) launches `arrow_validation run --package xtask --bin xtask` with the default `package-fixtures` feature plus derived force-validate. That is 818 units, including Symbolica, DataFusion, aws-lc-sys and egobox.
2. `xtask python-tests` then spawns `.venv/bin/python -m scripts.python_tests`.
3. Each canonical Python test registers in-process (one rewrite), calls `register_context` and `ready`/`start`, runs `Runtime.initialize_database` (the schema install) and then drains.

**O4: `just codegen`.** This is five recipe lines, and each line is a separate admission.
1. `xtask --no-default-features` generates the contracts. That is 409 units, including datafusion-optimizer and datafusion-functions, which arrive through the workspace-hack.
2. The surreal step runs.
3. `pse-env --native=compiler,solver` builds `xtask` with `package-fixtures` (the full `pse-runtime` closure). It then runs the complete generation: physical-fixture admission through `pse_runtime::authoring_driver`, a Python contract-check subprocess and Ipopt bindgen.
4. `cargo hakari generate` and `manage-deps` run.

The writer preserves unchanged bytes and mtimes.

**O5: `scripts/pse-env --native[=caps] -- <cmd>`.** This creates a new `Operation` record. For each family it then:
- computes the identity. For KLU that means `cargo metadata --features …native-solvers`, SHA-256 of the 7,217-file, 77 MB suitesparse vendor tree, and the compiler and cmake binary bytes.
- runs `prepare`, which calls `candidate`. That pins the generation, seals it, runs `valid` (which hashes every installed file) and runs `collect` (which calls `pinned_generations` and so scans every `.operations/*.json` under per-file locks).

The command then runs, and `reclaim_reports` runs on exit.

## 3. Candidate findings

### G-01: Test-resource coordination processes retained history on every fixture and every environment command (High)

**Cause and evidence.**
- `test_resources.resource_metadata` delegates to `host_admission.metadata`. That function takes a blocking exclusive `flock`, parses the whole `allocations.json`, validates it and unconditionally rewrites it through `native_operation.write_json`.
- Every fixture `register` and `drain` passes through this path (`pse_operations::testing::resource_bridge`, which spawns Python, and the Python fixture in `canonical_substrate`).
- `test_run.finish` uses it for `finalize`, and each per-resource `reclaim` uses it about three times plus a `surreal sql` process.
- Every top-level `scripts/pse-env` exit uses it as well: `pse_env.main` calls `test_resources.reclaim_reports()` in its `finally` block. That puts it on every scoped recipe line, including the `default` recipe's `just --list`.

**Observed state (read-only census, 2026-10-11 about 02:45 local).**
- `allocations.json` is 1.7 MB and holds 1,971 owners:
  - 905 are `incomplete`, pinned and drained;
  - 627 are `pass`, unpinned, drained and still `retained`, with no `cleanup_error`;
  - 92 are `pass` and not drained;
  - 101 are `failure` database records;
  - 185 are evidence records.
- `reclaim_reports` considers only `kind == "evidence"`, so passed database records whose reclaim never ran or was skipped are never retried.
- `invocations/` holds 639 files (14 MB) with no pruning path. A search of `scripts/` found no deleter.

**Consequence.** In a canonical test campaign (O2 and O3, with hundreds of fixtures; one reference state alone accumulated 866 ledger records), each fixture pays:
- two Python interpreter starts (Rust path), importing the host-admission, native-operation and `surreal_server` modules;
- two `systemctl` and `/proc` observations;
- at least two serialized full-ledger parse-and-rewrite cycles while it runs;
- after the run, about three more rewrites and one CLI process.

All of these serialize on one host-wide lock whose cost grows with retained history rather than with the number of live participants. Every unrelated `pse-env` command, including a compile check, also rewrites the ledger at exit.

This is H25 and H26 (coordination scaled to history, not participants), H8 (item-by-item reclaim) and H9 (whole-ledger scans for exact lookups) under AP-07 and G9. Prior lead 10-09 F03 is therefore *residual and moved*: completed records marked `removed` or `compacted` do leave the hot set, but retained, pinned and unreclaimed history stays hot, and the processing has spread to every environment exit.

**Simpler conforming alternative.**
- Keep live coordination as one small record per resource, keyed by resource ID: create with `O_EXCL`, update under a per-record lock, look up exactly by ID. Enumeration then happens only in an explicit sweep or terminal reconciliation. SQLite in WAL mode (Python stdlib), with indexed rows on state, invocation and disposition, is an equivalent library-owned option.
- Move pinned failure and incomplete diagnostics out of the coordination set into cold per-record files.
- In Rust, write the registration record directly instead of spawning Python, or batch registration per invocation. The runner already owns the selected inventory.
- Batch post-run disposal: one connection and one script that removes N databases.
- Run `reclaim_reports` from runners or an explicit sweep, not on every `pse-env` exit.
- Prune invocation files after terminal reconciliation.

**Preservation constraints.** Keep exact test association (`selected_test` `/proc` checks), the service-generation and namespace checks before removal, drain proof before eligibility, no-recreation semantics, retention of pinned failures, and the rule that "unknown is not evidence of drain". Disposal must still require the runner's terminal outcome.

**Uncertainty.** Per-cycle latency is unmeasured (probe P1). Part of the hot population may be legitimately pinned evidence. The defect is that pinned retention and live coordination share one scanned and rewritten artifact, not that retention exists.

**Evidence label.** The mechanism is Interface-checked. The census is a Measured operational observation under the conditions above. No latency is claimed.

**Verification.** Hold the live fixture count constant while adding retained records: register and drain cost should stay flat. A `pse-env -- true` should not open the resource ledger. Exact-ID lookup and the sweep should keep pinned and unknown records.

### G-02: Every Runtime-constructing test pays a per-test canonical-store lifecycle regardless of its subject (High)

**Cause and evidence.**
- `Runtime::from_shared` requires a `CanonicalDeployment` and eagerly constructs `Durability::Durable(Operations::from_store(...))`.
- The test helper `workflow::tests::runtime_on` always calls `canonical_deployment()`, which calls `canonical_fixture_store()`, before the caller applies `.with_durability(Ephemeral)`.
- I found about 231 call sites of the `runtime()`, `runtime_with` and `runtime_on` helpers across 59 files in pse-runtime, covering math solves, derived and strategies, modeling, fitting and data documents. In the maintainer's default-feature run, all 107 tests that reached it failed at this point.
- Each fixture creates a UUID database and installs the full schema: `canonical_fixture_with_profile` passes `initialize=true` to `CanonicalStore::create`, which runs 479 DEFINE statements (298 fields, 101 indexes, 38 tables, 42 functions). It also runs the G-01 coordination.
- Fixture registration can start the SurrealDB service as a hidden side effect (`test_resources.register` calls `surreal_server.start` when the service is not ready).
- Separately, per-test cold setup before the store is consulted had a median of 1.31 s (min 1.09 s, n=107). This comes from that JUnit report, under a 16-thread bare run on a shared host. Nextest's process-per-test model rebuilds `SharedRuntime`, the registry and DataFusion sessions for every test.

**Consequence.**
- For PSE-S05 and AP-06, policy and algorithm assertions inside pse-runtime cannot run without a compatible supervised server. They inherit fan-out failure (10-11 F01) and per-test DDL, network and Python work.
- Under H12, stable preparation (schema installation, runtime and session construction) is repeated per test instead of per run.
- 10-11 F02 is *confirmed still present and broader than that review stated*: it named three tests, but the coupling is structural, in the `Runtime` constructor.

**Alternatives.** Each preserves the canonical-source rule.
- (a) The runner pre-provisions a pool of initialized fixture databases in one batched operation before fan-out (one prerequisite check, as in 10-11 F01), and fixtures check one out. That moves DDL and registration off each test's critical path and batches disposal.
- (b) Make durable `Operations` construction lazy, or bind it at `with_durability`, so ephemeral construction does not allocate durable machinery.
- (c) Exercise the local owners (`check_deployment`, decoding and strategy policies) directly, as 10-11 F02 proposes.
- (d) If the source-authority rule is reconsidered, a process-local canonical source for tests that do not exercise persistence. This would be a rule impact (§7).

An embedded `kv-mem` SurrealDB would avoid the server, but it would add surrealdb-core to test builds and bypass the WebSocket path. It is not obviously cheaper.

**Preservation constraints.** Keep per-test database isolation for tests that assert store state, the late Rust interpretation checks, scientific source admission for ephemeral runs, and drain and retention ownership.

**Uncertainty.** How many of the roughly 107 tests genuinely consume canonical semantics; most compile authored packages, which legitimately need source admission. The per-test cost of `store.create()` is unmeasured (probe P5).

**Evidence label.** Interface-checked. The setup time is Measured with the conditions stated above.

**Verification.**
- A trivial Runtime test's time split into (cold runtime, fixture lifecycle, subject).
- A pooled-fixture prototype keeps isolation assertions and refusal behaviour.

### G-03: Generation and Python-test entry routes compile the whole runtime product for a fraction of it (Medium-High)

**Cause and evidence.**
- **py-test.** `just py-test` runs `arrow_validation run --package xtask --bin xtask -- python-tests`. `xtask::python_tests::run` only spawns `.venv/bin/python -m scripts.python_tests`, while `just py-unit` calls that module directly. The `xtask` default feature `package-fixtures` pulls `pse-runtime`, `pse-backend-native`, `pse-operations` and DataFusion. A unit graph for that command contains 818 units, including Symbolica, DataFusion, `aws-lc-sys` and the egobox family. It is built with derived force-validate, then discarded after one process spawn.
- **Codegen.**
  - `xtask/src/codegen/physical.rs` and the document-schema step use about 5.7k lines of `pse_runtime::{authoring_driver,physical}` plus `SharedRuntime`. Through `package-fixtures` they nevertheless need all of `pse-runtime` (166k handwritten lines) and its SurrealDB, native-backend and math closure.
  - `just codegen` step 3 also runs under `--native=compiler,solver`, which triggers the complete solver verification in G-05 for every regeneration, including registry edits unrelated to Ipopt bindgen.
  - The generator of `pse-quantity`'s physical facts (a leaf crate) depends on the top-level product built against the previous generated output, which is a bootstrapping cycle. The recipe compensates by sequencing "contracts first".

**Consequence.**
- After any Rust edit, `py-test` must re-resolve and re-link, and possibly recompile, a force-validated xtask before Python runs. The extension itself is built separately by `py-sync`.
- A narrow registry change pays a full-product build and native setup.

These are H5 and AP-06 build-scope violations (the dependency closure exceeds the responsibility) and H27 (the generator's lifecycle burden). DP-16 also applies: the xtask hop is vestigial.

**Simpler alternative.**
- Have `py-test` call `python -m scripts.python_tests` directly after `--store`. No rule impact.
- Move the authoring driver, physical inventory and the small `SharedRuntime` subset they use into a crate below `pse-runtime` that does not depend on `pse-operations` or `pse-backend-native`. Codegen and preparation tests then build only that closure. This is a rule impact because a new crate needs an ADR.
- Split Ipopt bindgen into a step that requests `--native=solver` only when the headers or the bindgen generator change.

**Preservation constraints.** Keep regeneration from one authoritative declaration, production-path physical admission (no test-only validator, per 10-11), byte-preserving writes and `codegen-check` coverage.

**Uncertainty.** Whether xtask artifacts are usually already fresh because maturin and other builds share them. If so, the cost is mostly resolution plus link (probe P6). The crate split depends on how much of `SharedRuntime` the authoring driver actually needs, which I did not inspect.

**Evidence label.** Interface-checked. The unit counts come from `cargo build --unit-graph` (resolution only, no build).

### G-04: Host admission partitions CPU statically, uses hard quotas and classifies work by command shape (Medium-High)

**Cause and evidence.**
- `host_admission.select` gives `compile` and `functional` one slot each on lane `functional` with cores `functional[:4]`. `pse_env.placement` sets `CPUQuota=len(cores)*100%`, which is 400%, while `AllowedCPUs` covers the 8 SMT siblings and `admitted_cargo_jobs` sets `jobs=8`. Rustc jobs therefore exceed the quota.
- `conflict` caps the `functional` lane at 2 slots host-wide, shared by compile, functional and wide.
- The timing lane (physical cores 0–7, half the host) is reserved even when no timing work is admitted.
- `acquire` fails after `admission_seconds = 30`; it does not queue.
- `classify` inspects `command[:4]`. Every justfile recipe line arrives as `bash -euo pipefail -c '<line>'`, so it is classed `functional` (a 40 GiB slot). I confirmed this by calling it read-only: `just --list --unsorted` and `python3 scripts/select.py unit …` both return `functional`.
- Nextest keeps `test-threads = 16` inside that 4-CPU quota. EI06 bounded Cargo jobs but not nextest's width.

**Consequence.**
- In the local-development workload, an edit-compile loop on `pse-runtime` (opt-level 2) uses at most 4 cores' worth of CPU time on a 16-core host even when the host is otherwise idle.
- With three concurrent agents, trivial recipe lines occupy one of only two functional slots, and the third command fails after 30 s.
- Sixteen test processes, each with Tokio, DataFusion and fixture threads, share 4 CPUs. CFS throttling can stretch sub-second lease and deadline controls.

This is H21 (capacity coordinated through fixed lanes rather than the real constraint), H25, and DP-20 (bounding is correct but not work-conserving). The OOM incidents that motivated the system were memory events, and memory caps address them.

**Simpler alternative.**
- Admit by memory budget.
- Share CPU by `CPUWeight` rather than a hard `CPUQuota`.
- Reserve timing cores only while a timing allocation is live; for example, apply `AllowedCPUs` exclusion to functional scopes only during timing.
- Declare a class per recipe (default `light` for non-build lines, `compile` for cargo) instead of parsing `bash -c`.
- Queue with fairness instead of failing at 30 s.
- Derive `NEXTEST_TEST_THREADS` from the admitted width.

**Preservation constraints.** Keep per-command memory caps, aggregate and non-agent headroom, timing isolation during measurement, exclusive admission for full campaigns and drain-before-release.

**Uncertainty.** The maintainer may reserve cores 0–7 for other repositories' workloads, which is not documented here. Wall-clock benefit is unmeasured (probe P7).

**Evidence label.** Interface-checked, plus the executed read-only classification and profile output.

### G-05: Native operations re-hash sealed installations and rescan operation history on every top-level native command (Medium)

**Cause and evidence.** `native_cache.prepare` remembers validity only within one `Operation`, which is one top-level `pse-env --native`. Per family, it does the following:
- `valid` calls `installation_files`, which SHA-256s every installed file. The solver generation is 2.4 GB in 1,125 files; PETSc, isolation, KLU and Uno add more.
- `klu` recomputes its identity: `cargo metadata` plus hashing of 77 MB across 7,217 source files. `tool_identity` hashes the compiler and cmake binaries.
- `collect` calls `pinned_generations`, which locks and parses every `.operations/*.json`. There are currently 1,214 files totalling 35 MB, up from 830 at the 10-09 review, so settled records are not being retired.

Generations are already sealed read-only by `seal_generation`, and "privileged owner mutation is outside this contract". The full hash therefore re-proves an established property (H19, H20) on every native command, including `codegen` and focused native units. This is the same root cause as PE05.

**Positions.**
- PE05 is *residual*: verification moved from per-preparation to per-operation, but it is still a complete hash per command.
- The native half of 10-09 F03 is *residual and moved*: the scan now runs at every preparation's `collect`, not only at retirement, and the population is growing.
- EF06 is *residual*: capability narrowing now exists.

**Simpler alternative.**
- Do the full verification at publication, at admission of an explicit prefix and at an explicit `verify` step.
- Per operation, compare a cheap sealed fingerprint (inode, size, mtime, ctime, mode per recorded file) against the receipt.
- Key the KLU identity on the `Cargo.lock` checksum of `suitesparse_sys` rather than re-hashing the extracted tree.
- Run `collect` only when a new generation is published or on an explicit GC.
- Delete settled operation records at operation exit, after drain proof.

**Preservation constraints.** Keep corruption detection at trust transitions, unknown-record refusal of reclamation, pins for live borrowers and admission of explicit prefixes.

**Uncertainty.** Hashing 2.4 GB from page cache may take only a few seconds, so the magnitude needs probe P3. Why records accumulate (malformed, unsettled or `pinned_generations` returning `None`) is unknown.

**Evidence label.** Interface-checked, with Measured cache and record sizes.

### G-06: Build closures of small roots are widened by redundant unification machinery and misplaced heavy dependencies (Medium-Low)

**Cause and evidence.**
- **The workspace-hack.** The pinned nightly already applies `[resolver] feature-unification = "workspace"`, and the toolchain policy forbids any other Cargo, so the hack (ADR-0122) only adds *crates* beyond what unification provides. `cargo tree --prune pse-workspace-hack` shows how much it adds:

  | Crate | Normal closure with hack | Without hack | Crates added |
  |---|---:|---:|---:|
  | pse-codegen | 261 | 226 | 35 |
  | pse-schema | 259 | 220 | 39 |
  | pse-columnar | 256 | 217 | 39 |

  For pse-codegen, the additions include datafusion-functions, datafusion-optimizer, datafusion-physical-expr, tracing-subscriber and several digest crates. That is why the bootstrap codegen's 409 units include the DataFusion planner. EF04 is *residual*: pse-model is now excluded.
- **pse-math's surrogate stack.** `pse-math` has normal dependencies on egobox-{ego,moe,gp,doe} and argmin, used only by `pse_math::surrogate`, whose sole consumer is `pse-runtime::math::surrogate`. Pruning them removes 78 of pse-math's 262 crates. They also bring a second nalgebra (0.33.3 via statrs and egobox-moe, alongside 0.35.0), a second ndarray (0.16 vs 0.17) and extra rand majors. This confirms the coordinator's L1 lead. `pse-compiler` (293 crates) inherits all of it.
- Under workspace unification, a feature gate on pse-math cannot narrow this: pse-runtime would enable the feature for every selection. Only moving the code (into pse-runtime, or into a crate) narrows the closure.

**Consequence.** Cold and worktree builds, and the critical path for pure-semantic and generator roots, wait on unrelated crates (H5, AP-06). The hack is also a second mechanism for one property (DP-16): its `traversal-excludes` restate the POUNCE git revision, and its `generate` and `manage-deps` steps run on every `just codegen`.

**Alternatives.**
- Retire hakari and rely on the mandated unification, keeping `family-check`. Alternatively, narrow hack membership to actual data-boundary consumers.
- Move the surrogate code into its only consumer, pse-runtime, which needs no rule change, or into its own crate, which needs an ADR.

**Uncertainty.** Whether any supported invocation still runs without `[unstable]` (for example, an external tool). Incremental-loop impact is probably small because dependencies stay cached; the cost falls on cold builds and dependency bumps.

**Evidence label.** Interface-checked from `cargo tree`.

### G-07: Native and non-native routes may invalidate the same build-script unit (Medium, unconfirmed)

**Cause and evidence.**
- `crates/pse-backend-native/build.rs::main` emits `rerun-if-env-changed` for `IPOPT_DIR`, `PKG_CONFIG_PATH` and `PSE_ROOT_ISOLATION_DIR` unconditionally, even with no native features enabled.
- `pse-backend-native` is a non-optional dependency of `pse-runtime`.
- `native_operation._environment` sets `IPOPT_DIR` for the `solver` capability, and the ordinary `pse_env.base_environment` leaves it unset.
- Default-feature closures that contain `pse-backend-native` are built under native environments by `just codegen` step 3 and by `codegen-unit-test`, and under plain environments by ordinary checks and tests.

**Likely consequence.** Alternating these routes re-runs that build-script unit. That would recompile `pse-backend-native` (98k lines), then `pse-runtime` (166k) and its dependents, at each switch. Cargo keeps one fingerprint per unit, so switching back costs the same again. This is H13: environment presence propagates as if relevant.

**Alternative.** Emit the env tracking only inside the native-feature branch, and do not apply native environments to default-feature closures that do not consume them.

**Uncertainty.** The outcome depends on Cargo fingerprint behaviour that I did not execute. Probe P4 settles it.

**Evidence label.** Interface-checked only.

### G-08: A command's exit status and liveness depend on unrelated ledger state (Medium-Low)

**Cause and evidence.**
- In `pse_env.main`, `reclaim_reports()` runs in a `finally` block. Its initial `resource_metadata()` is not guarded, so a `ResourceError` or `AdmissionError` (for example a corrupt or invalid ledger) replaces the command's own status with `FAILURE` (125, "boundary failed").
- `host_admission.metadata` with `deadline=None` waits on a blocking `flock` with no deadline.
- `write_json` replaces files without `fsync`. Leftover zero-byte `allocations.<pid>.tmp` files in `host-admission/` show writes that were interrupted.

**Consequence.** A passing `cargo check` can report a boundary failure, or hang at exit, because of test-resource state. This attributes failures to the wrong owner (DP-21, AP-05), which is part of the functional target.

**Alternative.** Report cleanup errors without changing the command's status, give the lock a bounded deadline, and remove G-01's exit-time work altogether.

**Evidence label.** Interface-checked.

## 4. Opportunities and uncertainties that are not findings

- **Pre-run resolution chain.** Each `unit-package` runs `cargo metadata --all-features`, one or two `--unit-graph` resolutions, `cargo nextest list` and then `run`. That is roughly four resolutions before the first test. The fixture association needs an inventory, but nextest's binaries metadata or archive could supply it. This is small compared with compile time, so it is an opportunity only.
- **Over-broad but correct applicability.** `validation_scope.RUST_INPUTS` (version 3) now covers all of `crates`, `scripts`, `.config`, `vendor` and `docker`, so 10-09 F02 is resolved. The consequence is that receipt reuse almost never applies during active development (H14: machinery whose benefit is mostly limited to unchanged commits). This is correct conservatism and the maintainer's choice, not a defect.
- **The pse-runtime monolith.** It is 166k handwritten lines plus at least 33.6k lines of in-crate tests, so it has two compilation units (lib and lib-test) and one test binary with about 763 tests. Size alone is not a defect (principles §G). Its concrete consequences are G-02 and G-03, and the remedy is the targeted extraction proposed there, not a general split.
- **Retention growth.** `build/assessment` holds 523 runs (4.9 GB), and every bundle captures full provenance (SHA-256 of about 2.6k tracked files, about 74 MB, plus a tar of untracked files). The maintainer manages disk space, and the cost is not on a per-test path.
- **Dev profile.** The workspace builds at opt-level 2 (incremental) and dependencies at opt-level 3. This trades edit-compile latency for numerical test speed, and EI06 retained it deliberately. I have no evidence that would overturn it.

## 5. Strengths and preservation constraints

- `arrow_validation.resolve` derives force-validate from the actual target, host and dev unit graph, refuses changes to the root set and adds no unrelated roots. 10-09 F04 is **resolved**.
- Codegen `write_tree`, Ipopt bindings and Python stubs skip unchanged bytes, so no-op regeneration leaves mtimes intact. EF07 is **resolved**.
- `pse-buildinfo/build.rs` now embeds only the rustc version and profile, so compile-time whole-tree fan-out is gone. PE06 and EF05 are **resolved at build level**; the runtime outer observation belongs to inquiry H.
- Hakari's `final-excludes` keep pure semantic leaves (pse-model, pse-quantity, pse-math, pse-compiler and others) out of the hack.
- An explicit `--execution-effects native-local` route (`unit-native-capability-package`) avoids the worker and observer. 10-09 F01 is **resolved**; the residual is that the general `unit-native-package` defaults to canonical.
- Fixture design separates test outcome, drain and disposal eligibility. Fixtures use isolated UUID databases, refuse on interpretation mismatch, and associate exactly with the selected test.
- Nextest scheduling is effect-based (`store` and `scientific` groups) rather than global serialization. Pure policy modules (`math::strategy`, `study_policy`, `workflow::diagnostics`) stay outside the store group.
- Other build choices are sound: checkout-local targets with shared sccache, `fine-grain-locking`, mold, unwind everywhere and no `target-cpu=native`.
- `build_receipt` writes a writable Cargo output from the sealed receipt. Enh F03 is **resolved**.

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| 10-09 F01 | Resolved, with residual default | `select.py --execution-effects`; `test_run.run_rust` strips canonical variables for native-local; `unit-native-package` still selects canonical |
| 10-09 F02 | Resolved (now over-broad by design) | `validation_scope.RUST_INPUTS`, `PRODUCT_ENVIRONMENT`, `INPUT_SCOPE_VERSION = 3` |
| 10-09 F03 | Residual and moved | G-01 (1,971 hot owners, 1.7 MB; `reclaim_reports` at every `pse-env` exit) and G-05 (scan per `collect`; 1,214 records) |
| 10-09 F04 | Resolved | `arrow_validation.resolve` |
| EI06 | Implemented, residual interaction | `admitted_cargo_jobs` gives 8 jobs against a 400% quota (G-04) |
| PE05 | Residual | `native_cache.prepare`, `valid` and `installation_files` per operation (G-05) |
| PE06 | Resolved at compile level; runtime observation not examined (H) | `pse-buildinfo/build.rs` |
| EF04 | Residual | `pse-codegen`, `pse-schema` and `pse-columnar` widened by the hack (G-06) |
| EF05 | Resolved | as PE06 |
| EF06 | Residual | capability narrowing exists; repeated verification and KLU identity re-hash remain (G-05) |
| EF07 | Resolved | `codegen::write_tree`, `ipopt::run`, `python_stubs` |
| Enh F03 | Resolved | `pse-backend-native/build_receipt.rs` |
| 10-11 F01 | Confirmed still present | No consumer-compatibility preflight; per-fixture refusal after fan-out |
| 10-11 F02 | Confirmed, broader | Structural in `Runtime::from_shared` and `runtime_on` (G-02) |
| 10-11 F03 | Confirmed still present | `.config/nextest.toml` names `fixture_connections_preserve_external_and_deployment_databases`; the test is `testing.rs::fixture_connections_preserve_deployment_database` |
| 10-11 interruption follow-ups | Confirmed present (`test_run.py` unchanged at baseline); not re-examined in depth | `run_rust` and `run_python` use synchronous `subprocess.call` |
| L1 egobox lead (coordinator) | Confirmed | 78 crates pruned; nalgebra 0.33.3 via statrs and egobox-moe (G-06) |

## 7. Candidate rule impacts

- **ADR-0122 (two feature-unification mechanisms).** Proposed change: retire the workspace-hack, or narrow its members to actual boundary consumers. Depends on G-06. If the rule is kept, narrow `final-excludes` further so that pse-codegen, pse-schema and pse-columnar are excluded where they are not DataFusion consumers.
- **Crate addition requires an ADR (AGENTS.md, `.claude/rules/decisions.md`).** Affects G-03 (an authoring and physical preparation crate below pse-runtime) and optionally G-06 (a surrogate crate). If the rule is kept and no ADR is pursued, move the surrogate code into pse-runtime, and remove only the xtask py-test hop and the native requirement from codegen.
- **Host capacity policy (`.config/agent-capacity.toml`; `docs/dev/agent-environment.md` "Functional work uses cores 8–15 … timing uses cores 0–7. Kernel quotas bound CPU time").** Proposed change: memory-based admission, weight-based CPU sharing, timing reservation only while timing work runs, declared per-recipe classes, and queuing. Depends on G-04. If the rule is kept, at least classify recipe lines by declared class and derive nextest threads from the allocation.
- **Retention of pinned, incomplete and unknown records (Plan 33 live-coordination text).** Proposed change: keep retention, but outside the hot coordination artifact. Depends on G-01. No guarantee changes.
- **Mandatory canonical scientific source admission for every Runtime (Plan 28 and blueprint substrate decision).** This applies only if remedy (d) of G-02 is chosen. Remedies (a) to (c) keep the rule.

## 8. Proposed light probes

Note for the coordinator: any command wrapped in `scripts/pse-env` rewrites the test-resource ledger when it exits (G-01). P1 to P3 therefore use the interpreter directly so that they stay read-only.

1. **P1: cost of one ledger cycle.**
   ```bash
   python3 -I -c "import json,os,time; s=open(os.path.expanduser('~/.local/state/pse-arrow/test-resources/allocations.json')).read(); t=time.perf_counter(); [json.dumps(json.loads(s),sort_keys=True) for _ in range(20)]; print((time.perf_counter()-t)/20)"
   ```
   If the cycle takes more than about 20 ms, the serialized lock section, multiplied by five or more cycles per fixture and hundreds of fixtures, is material.
2. **P2: import cost of the bridge.**
   ```bash
   .venv/bin/python -X importtime -c "import scripts.test_resources, scripts.surreal_server" 2>&1 | tail -3
   ```
   The cumulative import time is paid twice per Rust fixture.
3. **P3: cost of native verification.**
   ```bash
   time (find ~/.cache/pse-arrow/native/solver -path '*/generations/*' -type f -print0 | xargs -0 cat | sha256sum)
   ```
   Run it with a warm cache and with a cold cache if possible. More than about 2 s per native command confirms that G-05 is material.
4. **P4: build-script invalidation (G-07; compile class).** Run `cargo check -p pse-backend-native --locked --message-format=json` under plain `scripts/pse-env --`, then under `scripts/pse-env --native=solver --`, then plain again. Filter for `build-script-executed` records or `compiler-artifact` records with `"fresh":false`. A non-fresh `pse-backend-native` on each switch confirms the finding; all fresh refutes it.
5. **P5: per-fixture lifecycle cost.** With a compatible v3 state, run one trivial Runtime-constructing test through `just unit-package pse-runtime 'test(=<name>)'`. Compare its JUnit time with the 1.31 s refusal median. The difference approximates the cost of registration, schema installation, drain and reclaim.
6. **P6: the py-test frontdoor.** After touching one pse-runtime source file, time `scripts/pse-env -- .venv/bin/python -m scripts.arrow_validation build --package xtask --bin xtask --locked` against `.venv/bin/python -m scripts.python_tests --collect-only -q`. The difference shows whether the hop costs a link or a recompile.
7. **P7: CPU quota effect (G-04; compile class, quiet host).** Touch `pse-math`, then run `just check-package pse-runtime` under the default compile class. Repeat with `scripts/pse-env --resource-class wide`. Read `cpu.stat` `nr_throttled` and `throttled_usec` from the scope's cgroup. Material throttling, or a wide-versus-compile wall-time ratio near 2, supports the finding.

Relevant paths: `scripts/{pse_env.py,host_admission.py,test_resources.py,test_run.py,select.py,arrow_validation.py,native_cache.py,native_operation.py,validation_scope.py}`, `crates/pse-operations/src/testing.rs`, `crates/pse-runtime/src/workflow/{mod.rs,tests.rs}`, `crates/pse-backend-native/build.rs`, `xtask/{Cargo.toml,src/python_tests.rs,src/codegen.rs,src/codegen/physical.rs}`, `.config/{nextest,hakari,agent-capacity}.toml`, `justfile`.
