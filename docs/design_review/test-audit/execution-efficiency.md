# Efficiency-oriented design principles in the test system

This page applies the standard's efficiency-oriented principles to the test system as a static
architectural review:
- AP-07/G9 execution fit;
- its refinements DP-09, DP-10, DP-16, DP-19 and DP-20;
- [Heuristics for Efficient Architecture](../design_principles/core/efficient-architecture-heuristics.md)
  H1–H28, including its interpretation safeguards.

It also holds the findings that review produced. Baseline, workload premise and limits are in the
[collection README](README.md). Citations are `path:line` at source commit `7902a1f5b`. All
runner, fixture and configuration files cited are unchanged since the account's earlier
baseline `cbda2efc6`.

**Method.** Findings rest on structural causes and consequences established from source,
configuration and library documentation (Context7, then upstream source). No profiling,
benchmark or test execution was started.

**Evidence labels.**
- *Demonstrated* work is Interface-checked.
- *Suspected* bottlenecks are labelled as such.
- *Local observation* means a read-only count or size from the local build or state directories
  on 2026-10-23. It is not a timing or an attribution.
- *Measured* figures come only from runs already recorded, with conditions named. They support,
  but never establish, a diagnosis.

**Safeguards.** Every finding states what a simpler conforming realization must preserve:
isolation, independent expectations, fresh state where it is the claim, and validation and
scientific obligations. No finding assumes caching, sharing, batching or doubles as a default
remedy.

*Lineage.* The second pass (2026-10-23) organized this page as a walk through the feedback
routes. This third pass replaces that walk with a principle-directed review. DTA-029 and DTA-030
keep their identities and are reframed. DTA-018 moved here from
[Evidence routes](evidence-routes-and-execution.md#dta-018); its former home keeps a route.

## Review by heuristic group

**A. Contracts and physical realization (H1–H4).**
- *Deviations:*
  - **H2.** The physical test unit carries obligations that belong to the server generation or
    the invocation. Each test is a nextest process, a database, a full schema installation, a
    bridge subprocess pair and a receiver context ([DTA-018](#dta-018)). Tests whose claims need
    no durable store still acquire one ([DTA-031](#dta-031)).
  - **H1, H4.** The runner's own enumeration and option handling recreate nextest capabilities
    ([DTA-022](test-execution-system.md#dta-022), [DTA-034](#dta-034)).
- *Strengths:*
  - `initialize=false` fixtures keep fresh installation where installation is the claim.
  - The Python suite shares one process runtime per worker.
- *Not material:* placing schema DDL at the server; H3 otherwise.

**B. Necessary work and optimization (H5–H11).**
- *Deviations:*
  - **H5–H6.** Reference-closure tests prepare the union closure of all consumers to assert
    properties of a few units ([DTA-030](#dta-030)).
  - **H8, H9.** Registration rewrites a monolithic invocation document ([DTA-038](#dta-038)).
  - **H10, H11.** Retained state grows without convergence, onto the shared server and the
    production lifecycle path ([DTA-042](#dta-042)).
- *Strengths:*
  - Per-test registry operations never enumerate history.
  - Store tests use small synthetic cases.
  - Seed conformance prepares each package once and checks every fixture in it.
- *Not material:*
  - **H6.** Specialization returns full models where tests assert counts; narrowing would add
    machinery.
  - **H7.** The fixed-point lineage filter is small.
  - Python-to-Rust crossings per element in sampled tests.

**C. Preparation, reuse and change (H12–H14; DP-09, DP-10).**
- *Deviations:*
  - **H12.** Stable preparation is re-established per test or per run although its dependencies
    are unchanged: fixtures ([DTA-018](#dta-018)), the Python extension ([DTA-033](#dta-033)),
    Cargo resolution and enumeration ([DTA-034](#dta-034)), and covered generation
    ([DTA-037](#dta-037)).
  - **H13.** Change propagates at the granularity of build variants, embedded data and packages
    rather than of what a change can affect ([DTA-029](#dta-029), [DTA-032](#dta-032),
    [DTA-035](#dta-035), [DTA-036](#dta-036)).
  - **DP-09.** Generation reuse keys omit generator inputs
    ([DTA-024](test-execution-system.md#dta-024)).
- *Strengths:*
  - Workspace feature unification removes per-`-p` variants.
  - Generators and the stub writer skip identical writes.
  - The compiler workspace has unchanged-revision reuse.
  - Python workers share runtime owners and observation memos.
  - Opt-in reuse re-verifies at the point of reliance.
- *Not material:*
  - Runtime and engine construction (`tests::runtime()` is local and in memory).
  - Process-level statics that pay off only under `cargo test`.

**D. Representations, movement and live state (H15–H17).**
- *Deviations:*
  - **H15, H16.** A monolithic, fsynced invocation document stands in for an append-only
    membership log ([DTA-038](#dta-038)).
  - **H16, H17.** Disposable databases are persisted with fsync on every commit and kept
    resident ([DTA-018](#dta-018), [DTA-042](#dta-042)).
  - **H17.** Per-process budgets are independently maximized ([DTA-046](#dta-046)).
  - **H16.** Receipts copy source maps and failure payloads ([DTA-048](#dta-048)).
- *Strengths:* the ordinary Rust fixture budget is coherent with width (4 GiB × 16).

**E. Assurance without repeated reconstruction (H18–H20).**
- *Deviations:*
  - **H19.** Artifact identity is re-proved by full re-hashing at successive stages of one
    invocation ([DTA-039](#dta-039)), and per fixture on observer routes ([DTA-018](#dta-018)).
  - **H18.** The Python reinstall creates the uncertainty its digests then guard
    ([DTA-033](#dta-033)). End-of-assessment checks cover windows that cannot affect completed
    observations ([DTA-040](#dta-040)).
- *Strengths:*
  - The composer verifies validation leaves per actual scope.
  - One child-side check after admission addresses concurrent rebuilds.
  - Digests are never presented as semantic evidence.
  - `codegen --check` legitimately repeats the producer, because drift is the property.
  - Governance tests each establish a distinct property.

**F. Execution, coordination and recovery (H21–H25; DP-19, DP-20).**
- *Deviations:*
  - **H25.** Disposable test databases receive an evidence-grade durable protocol
    ([DTA-041](#dta-041)).
  - **H24.** Retention is decided per run rather than per test, and teardown stops at the first
    error ([DTA-042](#dta-042)).
  - **H21, H23.** Shared-store and host capacity are coordinated per layer
    ([DTA-043](#dta-043)), and budgets are maximized per process ([DTA-046](#dta-046)).
  - **H22.** Two broad barriers lack a recorded dependency ([DTA-044](#dta-044),
    [DTA-045](#dta-045)).
  - **H23.** A host-shared lock spans durable registration ([DTA-038](#dta-038)).
  - **H24.** Store degradation is recovered per test
    ([DTA-025](evidence-routes-and-execution.md#dta-025)).
- *Strengths:*
  - Runner width is derived from admitted cores.
  - Retries are 0 and BLAS threads are 1.
  - Deadlines are absolute.
  - Effects stay outside metadata locks.
  - Nothing uncertain is deleted, and owner identity uses pid + start + boot.
- *Respected premises:* the maintainer-selected unbounded build mode and 16-lane
  SurrealDB-coordinated parallelism.

**G. Whole-system economics (H26–H28; DP-16).**
- *Deviations:*
  - **H26, H27, H28.** Report retention seals and protects evidence that no producer can make
    disposable ([DTA-047](#dta-047)).
  - **H26.** Receipts are rewritten whole at every checkpoint ([DTA-048](#dta-048)).
- *H27, lifecycle of one test:*
  - A pure Rust test is authored locally. Each run, however, passes through: a supervised
    scope; composer resolution; `nextest list` and `nextest run` (each resolving metadata,
    building and listing); registry invocation; config copy; JUnit parse, freeze and terminal
    publication.
  - A Rust store test adds a feature gate and possible requirement tables at authoring. At run
    time it adds worker resolution, artifact hashing, the observer scope and the per-test
    fixture lifecycle.
  - A Python component test is authored locally (a strength). At run time it adds the
    extension chain and the per-test database.
- *Self-test economics:* about 29,000 lines of unittest code cover the roughly 24,500-line
  runner, resource and server component, much of it patching the module under test. That is
  proportionate to its race risks, but it binds tests to internal decomposition and grows with
  the machinery ([DTA-026](test-execution-system.md#dta-026); DA-024 by title). Removing work
  under DTA-039, DTA-041 and DTA-047 would shrink both. This is narrative, not a separate
  finding.

## A. Contracts and physical realization

### <a id="dta-018"></a>DTA-018 — Each canonical fixture is a complete physical lifecycle that re-establishes, per test, premises shared by every test in the invocation

**Principles:** AP-07, DP-10, DP-16, DP-20; H2 (primary), H3, H8, H12, H16, H19; companion §6, §8.

*Moved from [Evidence routes](evidence-routes-and-execution.md#dta-018) and reframed in the third
pass, from "Each Rust canonical test pays a Python bridge round trip and a full schema
installation, independent of what it checks". The identity is kept.*

**Claim and premise.** §24.1 promises that each test owns an isolated database. That promise is
realized by making the test the physical unit for every store obligation. nextest's
process-per-test model is fixed, so nothing in a test process can amortize the work, and no
run-scoped owner prepares shared store state.

**Per-test lifecycle (demonstrated).**
- *Registration.*
  - A Python bridge subprocess pair (`crates/pse-operations/src/testing.rs:140-219`) loads the
    registry's import closure, about 22,000 lines.
  - Registration re-proves service readiness: health check, `systemctl`, credentials digest,
    and a SHA-256 of the study module plus an archive scan (`scripts/test_resources.py:505-510`;
    `scripts/surreal_server.py:1190-1286`).
  - The service configuration is recomputed several times.
  - On observer routes, every fixture also registers a receiver context and publishes a
    generation (`surreal_server.py:249-320,877-990`).
    - This digests all `scripts/*.py`, the configuration files and the worker binary. The
      worker is content-addressed and identical for every test in the invocation. Local
      observation: `target/debug/pse-worker` is about 1.0 GB.
    - `verify_generation` then re-digests the generation.
- *Schema.*
  - One transaction installs the study-module bucket and 989 `DEFINE` statements (623 fields,
    171 functions, 115 indexes, 79 tables, 1 event; 340,507 bytes).
  - `open()` re-reads the module receipt, hashes the artifact and runs `INFO FOR DB STRUCTURE`
    (`crates/pse-operations/src/canonical.rs:2350-2356`;
    `crates/pse-operations/src/canonical_study_module.rs:92-240,297-312`).
- *Server-side contention and persistence.*
  - Concurrent installations into separate databases conflict. Production carries an
    up-to-8-attempt backoff with jitter for exactly that case (`canonical.rs:1366-1395`).
  - The server persists every commit with `aol=sync&sync=every`
    (`surreal_server.py:7040-7051`), so every DDL commit for a disposable database is fsynced.
- *Teardown.*
  - A drain bridge (`testing.rs:99-137`).
  - Then removal through one `surreal sql` CLI process per database
    (`test_resources.py:1522-1577`), after further configuration reads and fsynced record
    rewrites.
- *Python.*
  - `canonical_substrate` registers in process, but each test still runs `register_context`
    and a full `initialize_database` (`python/pse/tests/conftest.py:84-133`). About 78 test
    functions use it.
  - Program caches are cleared at each teardown.

**Where each obligation belongs.**
- *Per test:* data isolation, outcome-keyed retention and that test's drain.
- *Per server generation or invocation:* schema digest, interpretation, study-module identity,
  payload incarnation, service configuration and the supervisor/worker generation. These are
  exactly what the installed marker records and `open()` re-checks
  (`canonical.rs:1491-1555`).
- *Fresh state is the claim for:* `initialize=false` fixtures, initialization and
  cancellation controls, rebuild and migration, and issuing-store identity.

**Consequence.** The fixed cost of every store test is a multi-process, multi-connection,
fsync-bound protocol. On observer routes it adds whole-file hashing of a gigabyte-scale binary,
plus DDL contention at the shared server. The cost scales with the number of selected store
tests, whatever their claim.

*Measured context, which does not attribute the cost:*
- Run `217bd5e6` (2026-10-23, nextest 0.9.146, default profile) selected 27 tests with
  `pse-runtime/canonical-tests` among its features. Its fastest test took 12.6 s.
- In the managed Python run recorded in `build/plan49de-python-final-20261022/native-python.xml`
  (2026-10-22), canonical items took at least 6.4 s each, against at most 1.1 s for store-free
  items.

**A simpler realization must preserve:** per-test data isolation, fresh installation where
installation is the claim, retention of failed databases, registration before effects, and
drain association.
- *Lead:* establish generation-scoped premises once per invocation, and give each test an
  isolated database whose installed marker matches them.

**Library check.**
- SurrealDB 3.3.4 documents no database clone or template, and `DEFINE` requires a selected
  database, so namespace-per-test cannot share a schema. An empty documentation search does not
  establish absence.
- Wrapping each test in a transaction is not an isolation option, because these tests exercise
  multi-connection transactions and leases.
- nextest setup scripts run once per run, but they are experimental.

**Counterevidence.**
- Fresh-per-test isolation is a deliberate premise.
- Generation copies are made only when identity changes.
- The conflict retry also serves production multi-database deployments.

**Uncertainty.**
- How the time divides between these phases is unmeasured.
- Whether a leased, verified database is as trustworthy as a fresh installation for issuing-store
  identity and per-database module registration is not established.

### <a id="dta-031"></a>DTA-031 — Tests whose claims need no durable store acquire a canonical database anyway

**Principles:** AP-06, AP-07, DP-10; H2, H5, H12 (eager preparation for an unused capability);
PSE-S05; companion §5, §8.

**Diagnosis.**
- *Python.*
  - The `runtime` fixture in `python/pse/tests/test_native_workflow.py:366-371` is
    `canonical_substrate.runtime(inspection_settings, ephemeral=True)`.
  - An ephemeral runtime keeps a deferred store handle that only recording operations open
    (`crates/pse-py/src/workflow.rs:1853-1876,694-717`).
  - A static syntax-tree scan found 17 test functions (about 24 parametrized items) that use
    only this fixture and make no recording call. Each still pays the full `canonical_substrate`
    lifecycle ([DTA-018](#dta-018)).
  - The root `conftest.py:283-285` places each of them in `xdist_group("canonical-owner")`
    ([DTA-044](#dta-044)).
  - The same file already builds a substrate-free ephemeral runtime (`:413-416`), as do
    `test_physical_boundary.py:34-40` and `test_scientific_boundary_adoption.py:29-35`.
- *Rust.*
  - PR Jacobian and recycle tests build `Runtime::from_shared(…, canonical_deployment())`
    (`crates/pse-runtime/src/workflow/modeling/pr_jacobian_tests.rs:105`;
    `recycle_tests.rs:13,121,364`). They are gated on solver features rather than
    `canonical-tests`.
  - Parallel conformance acquires `canonical_deployment()` and then sets
    `.with_durability(Ephemeral)` (`crates/pse-runtime/src/workflow/modeling/conformance.rs:2706-2739`).
  - A local alternative exists: `Runtime::from_shared_local` (`workflow/mod.rs:503-528`), which
    `tests::runtime()` uses.

**Consequence.**
- These tests pay the store lifecycle and require `PSE_SURREAL_STATE`.
- *Suspected:* routes that strip that variable fail them (`scripts/test_run.py:506-514`), an
  undeclared requirement of the kind [DTA-023](test-execution-system.md#dta-023) describes.
- *Measured:* the managed Python run above
  (`build/plan49de-python-final-20261022/native-python.xml`, 2026-10-22; features
  force-validate, native-solvers, solver-highs; JUnit times include setup and teardown).
  - The session took 602.1 s; its 34 canonical-owner items summed to 597.8 s.
  - The 16 ephemeral-only items summed to 186.8 s.
  - `test_native_capability_discovery_and_hard_cut`, whose body is `runtime.capabilities()`,
    took 7.62 s.

**A simpler realization must preserve:**
- the durable-versus-ephemeral contrast test (`test_native_workflow.py:963`);
- per-database isolation where the store is used;
- the production composition, where recording source revisions or products is part of the
  claim (`workflow/mod.rs:463`).

**Uncertainty.**
- Whether local-runtime results are identical for the Rust cases is not established.
- 38 of 79 canonical Python tests make no direct recording call, but durable runtimes record
  implicitly. The wider footprint needs per-test judgment.

**Related.** [DA-021](../audit/local-reasoning-and-assurance.md#da-021) is a production cause and
is distinct. Here a local alternative already exists, so the dependency is a test-side choice.

## B. Necessary work and optimization

### <a id="dta-030"></a>DTA-030 — Reference-closure tests prepare and fully check the union closure of every consumer, then specialize it, to assert properties of a few units

**Principles:** AP-07, DP-10; H5, H6, H12; companion §8.

*Reframed in the third pass from "Broad library-unit feedback is bounded by a few tests that
specialize the full reference closure". The recorded measurement is kept as supporting
evidence. The identity is kept.*

**Diagnosis.**
- `SOURCES` in `tests/fixtures/reference-source-fixture.rs:28-73` lists 44 documents, about
  389 KB. It is the closure "for the changed packages and their imported dependency closure",
  shared by every consumer.
- Each test re-parses the concatenation (`:75-80`). `publish_modeling` then runs
  `check_modeling` over the complete candidate (`crates/pse-compiler/src/workspace/modeling.rs:735`).
- Some files then specialize it under several routes to assert structural counts:
  - `crates/pse-compiler/src/process_composition_tests.rs:36-86`, a three-unit process across
    three routes;
  - `workspace/modeling/conditional.rs:1019-1110`.
- Closure preparation repeats per test with identical text: 10 publications in
  `authored_transfer_tests` and 6 in `physical_potential_tests`.
- nextest's process-per-test model allows no sharing across tests. Grouping cases inside one
  test is the only in-process lever, and `process_composition_tests.rs` already uses it across
  routes. Production already provides in-process reuse (`workspace/modeling.rs:705-722,929`).

**Where breadth is the claim and where it is not.** The actual authored units and their packages
are the claim. Packages outside a test's own import closure are not; that per-test closure was
not computed, so that part is *suspected*.

**Consequence.** Every reference consumer pays for parsing and checking the union. A few pay for
full specializations, which set broad library feedback time.

*Measured* from recorded nextest runs `60f84901` and `b5f4cd35` (2026-10-22, nextest 0.9.146,
default profile, dev build):

| Run | Wall time | Slowest test | Next slowest |
|---|---|---|---|
| `60f84901` | 54.8 s | 53.5 s (`indexed_process_composition_keeps_memoryless_units_algebraic`) | 46.8 s |
| `b5f4cd35` | 66.6 s | 65.1 s | 56.0 s |

- In `b5f4cd35`, admission-only closure tests took about 1.04 s (44 documents) and 0.41 s (18
  documents).
- The 55 tests in closure-consuming files account for 283.9 s of pse-compiler's 294.5 s
  summed test time.

**Counterevidence.** The tests exercise genuine reference science. Specialization latency may be
a real production cost, possibly owned by the broad account's execution-fit findings
([DA-029](../audit/execution-fit.md#da-029), [DA-030](../audit/execution-fit.md#da-030); linked by
title, not reexamined). The source of the closure is the hand assembly in
[DTA-027](doubles-and-coupling.md#dta-027).

## C. Preparation, reuse and change

### <a id="dta-029"></a>DTA-029 — Store-test selection is a compile-time feature on production crates, which also compiles test hooks into the library

**Principles:** AP-07, AP-05, AP-02; H13, H27; companion §6, §8.

**Diagnosis.**
- `canonical-tests = []` (`crates/pse-operations/Cargo.toml:32`) is propagated by pse-runtime
  and xtask. It gates about 270 cfg sites in 62 files.
- Under `cfg(feature = "canonical-tests")` without `test`, it compiles test-only behavior into
  the **library**:
  - a task-local hook that cancels and suspends an artifact write after its append is
    acknowledged (`crates/pse-operations/src/canonical_artifacts.rs:20,508-514`);
  - an extra stderr print on operation failure (`canonical.rs:2457-2466`);
  - further sites in the maintenance, project, selection and study-module owners.
- Recorded runs used ad hoc feature mixes. Run `217bd5e6` combined
  `pse-runtime/canonical-tests,pse-runtime/solver-ipopt,pse-operations/evidence-query,xtask/…`.

**Consequence.**
- Each route compiles its own variant of the largest crates.
- Compiled-out tests are absent from `nextest list` rather than `not_run`
  ([DTA-013](evidence-routes-and-execution.md#dta-013)).
- The pse-operations variant under test is not the production library.

This is one axis of the wider variant split in [DTA-032](#dta-032), and it keeps its own identity.

**Counterevidence.**
- `native-solvers` is a real linking capability, not only a selection switch.
- Runtime selection already exists: `default-filter` for `managed_primary_`, and reasoned
  `#[ignore]`.
- The library hooks serve a cross-crate cancellation-after-acknowledgement claim.

### <a id="dta-032"></a>DTA-032 — Route-chosen features split the shared build into several full workspace variants

**Principles:** AP-07, DP-10, DP-16; H12, H13, H27; companion §8.

**Claim at risk.** "One feature set per dependency, whatever `-p` selects" (ADR-0122). That holds
for package selection but not across routes.

**Diagnosis.** Two mechanisms split the build.

*Test-only features on production crates.*
- `pse-quantity` `fixtures` gates a 10,025-line generated module
  (`crates/pse-quantity/Cargo.toml:32`; `src/lib.rs:56-58,86-87`).
- `pse-relations` `test-support` gates `lib.rs:31-32`.
- `pse-operations` `test-support` gates about 18 sites in `canonical.rs`.
- 16 manifests switch these on through dev-dependencies.
- With `[resolver] feature-unification = "workspace"`, any build that includes dev units turns
  them on for every member, while lib-only builds do not. In pse-py's closure, `cargo tree`
  shows `pse-quantity []` normally and `[fixtures]` with dev units.

*Validation switched per route.*
- `force-validate` is added only by the composer (`scripts/arrow_validation.py`).
- Bare routes leave it off:
  - xtask `python-stubs` (`scripts/python_extension.py:122-131`);
  - the rust-contracts, python, docs, surreal and bindgen codegen checks (`justfile:365-408`);
  - `family-check`;
  - `py-sync`.
- `arrow-data` resolves as `[ffi]` without validation and `[ffi,force_validate]` with it.
  Because `arrow-data` sits at the bottom of the Arrow stack, the whole Arrow, Parquet and
  DataFusion stack and every workspace library differ between the variants.

**Consequence.**
- One `just py-unit` reaches two variants: the validated pse-py closure through maturin, and
  the unvalidated xtask closure.
- Alternating Rust tests (dev units) with Python tests (none) recompiles every workspace
  library twice after a foundational edit.
- `codegen-check` reaches both Arrow variants.
- sccache does not cache incremental workspace crates, so each variant's workspace part is
  rebuilt locally.
- *Local observation:* `target/debug/build/arrow-data` held `{ffi}` and `{ffi,force_validate}`
  units, and pse-relations lib units appeared as `[]`, `[force-validate]` and
  `[force-validate,test-support]`.

**A simpler realization must preserve:**
- force-validate on every correctness closure (ADR-0170);
- the deliberately unvalidated interactive extension;
- workspace unification;
- no test hooks in production libraries.
- *Lead:* run tooling routes in the variant correctness routes already build, and move fixtures
  and test support into separate test-support crates.

**Counterevidence.** Cargo keeps the variants side by side, so they do not thrash. `test-support`
sites need private access.

**Uncertainty.** Rebuild time per variant, and how often routes alternate, are unmeasured.

### <a id="dta-033"></a>DTA-033 — Every focused Python run rebuilds, reinstalls, re-introspects and repeatedly re-hashes a gigabyte-scale extension, whatever the change or selection

**Principles:** AP-07, DP-10, DP-16; H12, H14, H18, H19, H28; companion §8.

**Diagnosis.** `py-unit` and `py-test` always run `python_extension sync auto` first
(`justfile:760-774`; `scripts/python_extension.py:102-137`). The chain is:
- *Doctor probe and `uv sync`.*
- *Build and install.* `maturin develop --features force-validate` (`:86-99`). maturin builds an
  editable wheel unconditionally and installs it unless told not to (maturin
  `src/develop/mod.rs`).
- *Stubs.* `cargo run -p xtask --no-default-features -- python-stubs`, in the unvalidated variant
  ([DTA-032](#dta-032)).
  - It spawns Python to locate the extension (`xtask/src/codegen/python_stubs.rs:84-101`).
  - It reads the whole cdylib into memory for introspection (pyo3-introspection 0.29.2,
    `src/introspection.rs:535`).
  - It runs ruff twice and rewrites the tracked `_native.pyi` when it differs.
- *Hashing.*
  - `capture()` hashes the binary (`python_extension.py:64-71`).
  - The native route hashes it again (`scripts/native_tests.py:1379-1394`).
  - `pytest_sessionstart` hashes it in the controller and in every xdist worker, because workers
    run full sessions (`python/pse/tests/conftest.py:70-81`).
- *Local observation:* `python/pse/_native.abi3.so` is 1,180,465,209 bytes (dev profile).

**Structural cause.** The obligation "first build the explicitly validated correctness extension"
is realized as an unconditional rebuild, reinstall and re-stub. The reinstall itself creates the
replacement hazard that the repeated digests then guard (H18). The property depends only on the
pse-py closure and its features.

**Consequence.**
- Per run: one install, freshness checks of two build closures, one full introspection read and
  (2 + workers) full hashes of the extension.
- This happens even for a pure-Python edit with one selected test.
- A test command writes a tracked generated file.

**A simpler realization must preserve:** no collection against a stale or unvalidated extension,
explicit features, detection of a concurrent rebuild between capture and import, and the
`python-stubs-check`.
- *Lead:* reinstall and re-stub only when Cargo reports the cdylib artifact as not fresh or the
  features differ, and carry identity to workers with a cheap metadata recheck.

**Counterevidence.** Cargo fingerprinting and `uv sync` are near no-ops when nothing changed, and
stubs are written only on change.

**Uncertainty.** Elapsed time is unmeasured. Whether maturin rewrites the `.so` when Cargo was
fresh is not established.

### <a id="dta-034"></a>DTA-034 — The ordinary managed Rust route resolves the workspace several times and builds and enumerates twice, because metadata reuse is tied to the provenance chain

**Principles:** AP-07, DP-16; H4, H12, H19, H28; companion §8.

**Diagnosis.** A focused `just unit-package` run goes through three stages.
- *Composer* (`scripts/select.py:144`): `resolve` runs `cargo metadata --all-features`, the unit
  graph, and the validated unit graph (`scripts/arrow_validation.py:335-369`).
- *Listing.* `run_rust` lists without reuse options (`scripts/test_run.py:529-551`). nextest
  acquires its own metadata, builds and lists the binary.
- *Run.* `nextest run` acquires metadata again, performs a no-op build and lists a second time.
  nextest does this whenever `--cargo-metadata` is absent.

The ordinary path refuses retained metadata unless a prepared, provenance-verified inventory
exists (`test_run.py:536-538`).

The native path does reuse the composer's metadata and binaries list for both listing and run
(`scripts/native_tests.py:1029-1112`). So cheap in-invocation reuse is available only bundled
with the heavy verification of [DTA-039](#dta-039).

**Consequence.** On the agent inner loop that AGENTS.md routes through `unit-package`, each run
pays extra metadata resolutions and an extra fingerprint walk and listing. Magnitude is
unmeasured.

**A simpler realization must preserve:** ADR-0170 leaf verification per scope, the root-identity
check, nextest's ownership of selection, and the pre-run catalog that fixture membership uses.
- *Lead:* pass the composer's metadata and the listing's binaries forward on every route, as the
  native path already does.

**Counterevidence.** The validated unit graph establishes a property nextest cannot supply.

### <a id="dta-035"></a>DTA-035 — Reference science documents are compiled into the largest test binaries, so data-only edits rebuild them

**Principles:** AP-07, DP-10, DP-16; H13; companion §8.

**Diagnosis.**
- Counting the shared `#[path]` fixture, `include_str!`/`include_bytes!` embed 50 of the 85
  reference `.pse` documents in pse-runtime's library test binary, 54 in pse-compiler's and 14
  in pse-modeling's.
- pse-runtime is one `cfg(test)` crate of about 295,000 source lines, compiled at dev
  `opt-level = 2`.
- Equivalent fixtures elsewhere are read at runtime (`crates/pse-runtime/src/workflow/tests.rs:9-47`;
  `tests/support/plan14.rs:43-105`; the conformance runner).

**Consequence.**
- A science-data edit dirties and recompiles those test binaries. Since 2026-09-23, 16 of 52
  commits that touched reference `.pse` files touched no pse-runtime or pse-compiler Rust.
- `affected` selects nothing for such an edit ([DTA-036](#dta-036)).

**Counterevidence.** Embedding binds exact bytes into the binary and fails loudly when a file is
missing.

**Related.** [DTA-027](doubles-and-coupling.md#dta-027) covers the same include lists from the
composition side.

### <a id="dta-036"></a>DTA-036 — `affected` propagates the fact that a package was touched, not which tests a file can reach

**Principles:** H13, DP-07; companion §8. Lower severity.

**Diagnosis.** `scripts/affected.py` maps a file to a package by path prefix (`:92-99`) and selects
reverse dependencies through dev edges too (`:86-110,156`).
- *Over-selection.* An edit to a `cfg(test)`-only file such as `authored_transfer_tests.rs`
  selects 8 packages, including pse-runtime's 899 library tests, none of which can see that
  code.
- *Under-selection.* `tests/fixtures/**` and `packages/reference/**` are reported as unowned
  and select nothing, although they are embedded or read by tests.

Cargo's dependency files already map each test binary to its exact sources and include files.

**Counterevidence.** The exclusions are printed honestly, the tool is opt-in, and package-level
selection is conservative for library edits.

### <a id="dta-037"></a>DTA-037 — `codegen-check` repeats generation that the relations gate already covers

**Principles:** AP-07; H12, H28. Low severity; runs at scope end only.

**Diagnosis.**
- The 10-gate group (`scripts/validation_scope.py:104-115`) runs 8 separate `cargo run -p xtask`
  processes with 4 feature sets (`justfile:365-408`). Each assembles the registry
  (`xtask/src/codegen.rs:50`).
- `--only relations` compares the Rust tree and the physical tree (`:63-68`), a superset of the
  rust-contracts check (`:193`) and the physical check (`:76-81`).
- So within one group run, Rust generation runs twice and physical admission twice. Physical
  admission loads packages, opens an engine session and loads the physical inventory
  (`xtask/src/codegen/physical.rs:44-69`).

**Counterevidence.** Separate gates report independently. `codegen-contracts-check` uses
rust-contracts on its own. The repetitions in `just codegen` are a justified bootstrap.

## D. Representations, movement and live state

### <a id="dta-038"></a>DTA-038 — Every fixture registration rewrites the invocation's whole test selection three times with fsync, under a host-shared server lock

**Principles:** AP-07, DP-10, DP-19, DP-20; H8, H9, H10, H15, H16, H22, H23; companion §8.

**Diagnosis.**
- *Representation.* Invocation membership is one JSON document that embeds the full declaration,
  including every selected test from `nextest list` (`scripts/test_resources.py:296-312`;
  `scripts/resource_store.py:352-371`).
- *Rewrites.* Registration needs only an append. Instead, `FileStore.register` publishes the
  whole state twice (`resource_store.py:444,446`), and the enclosing `invocation_edit` publishes
  it a third time on exit (`:373-389`).
  - Each publish writes a temporary file and fsyncs the file, its directory and the parent
    (`:75-93`).
  - Each registration also parses the document three times.
- *Lock scope.* This all happens inside `surreal_server.context_admission`
  (`test_resources.py:514-557`), which serializes it against every lifecycle change of the
  shared server, and under the invocation's own lock.
  - The `context_admission` docstring says it serializes "only admission metadata".
  - That shared lock also covers context registration, worker admission, reclaim claims and
    server lifecycle (`surreal_server.py:433-466`).
- *Teardown.* `finalize` edits members one at a time (`test_resources.py:600-694`), and removal
  runs one CLI process per database (`:1522-1577`). Namespace removal, by contrast, already
  batches child removals (`:1495-1496`).

**Consequence.**
- Registration work per invocation grows with fixtures × selection size × 3.
- Independent registrations across all sessions, worktrees and Python workers serialize on one
  lock.
- *Local observation:* invocation-state files over 1 MB exist; the largest embeds 4,779
  selected tests with 217 members.
- *Suspected:* registrations queued behind large rewrites exhaust the default 5 s registration
  clock and surface as product failures ([DTA-025](evidence-routes-and-execution.md#dta-025)).

**Claims compared with code.**
- ADR-0174's driver says an identified resource should consume only its actual metadata
  dependencies.
- §24.1 says stable exclusion follows invocation, identity and resource order. A host-shared
  server lock precedes that order.

**A simpler realization must preserve:** crash-safe pending and uncertain publication, freeze
immutability, exact association, and "lifecycle sees every accepted borrower".
- *Lead:* keep the selection only in the immutable declaration, append per-member records, use
  shared rather than exclusive admission for registration, and batch removals.

**Counterevidence.**
- The protocol is crash-safe and fails closed.
- Only metadata work happens under the locks.
- ADR-0174 already chose per-resource files to remove whole-ledger contention.

**Uncertainty.** fsync latency and queue depth are unmeasured.

## E. Assurance without repeated reconstruction

### <a id="dta-039"></a>DTA-039 — Artifact identity is re-proved by full re-hashing at successive stages of one invocation

**Principles:** AP-07, DP-09, DP-23; H18, H19, H20, H26; companion §6, §8.

**Diagnosis.** Validity travels only as `{path → sha256}` inside JSON, so each stage owner
re-proves it by hashing again. No stage gives the artifact an owned identity (a frozen copy,
hardlink or stat identity). The component has several composition roots
([DTA-026](test-execution-system.md#dta-026)), so each stage defends itself independently.

On a native Rust route, the same artifacts are hashed at four points:
- *Capture.* `native_provenance` digests every selected binary and every library `ldd` reports
  (`scripts/native_tests.py:60-81`). `relevant_environment` digests the worker binary again
  (`scripts/validation.py:86-87,128,145`).
- *Parent.* The parent immediately re-digests the fixture bindings (`scripts/test_run.py:658`).
- *Child, after admission.* The child digests the fixtures and the worker
  (`native_tests.py:1274,1281`). `prepared_rust_inventory` then re-digests every recorded file
  and the fixtures again (`:415-436`).
- *Whole-checkout source identity.* Computed three times (`:1018,1156,393`), each pass reading
  every file before projecting to the product scope.

Assessment repeats this. native-test, managed-native and managed-python each resolve,
build-check and digest the same worker.

*Local observation:* a focused one-crate canonical run's `native.json` lists 35 files totalling
1.48 GB, including the 1.0 GB worker and a 0.28 GB test binary. By the paths above, the worker
is hashed about four times and the rest twice per invocation. The Python equivalent is in
[DTA-033](#dta-033), and per-fixture generation hashing in [DTA-018](#dta-018).

**Consequence.** A fixed per-invocation cost that scales with binary size × stages, paid even
when one test is selected. Most of it re-verifies conclusions established moments earlier in the
same process tree.

**A simpler realization must preserve:**
- detection that what executes is what was recorded, across the admission wait and concurrent
  rebuilds by other agents;
- independent full re-proof at reuse (`scripts/validation_receipts.py:181-202`).
- *Lead:* freeze identity once (digest plus stat identity, or a hardlink), check stat identity
  within the invocation, and do full digests only at trust transitions. `scripts/reference_run.py`
  already freezes one artifact by copying it.

**Counterevidence.**
- The digests are identity checks and never semantic evidence, which respects H20 in kind.
- The single child check after admission addresses a real hazard: concurrent builds in a shared
  `target/`.

### <a id="dta-040"></a>DTA-040 — End-of-assessment identity checks can fail observations that finished before the change they detect

**Principles:** H18, H19, DP-23, AP-05. Lower severity.

**Diagnosis.**
- *Source window.* Gate inputs are projected from the snapshot taken at assessment start and
  compared with a snapshot taken after the last gate (`scripts/validation.py:870-872,1096-1118`).
  Any change in the broad scopes during the whole run fails every gate, including gates already
  finished.
- *Binary re-digest.* `classify()` re-digests native binaries after the gate has finished
  (`scripts/validation_receipts.py:41-79`). They were verified before running, and reliance
  happens at reuse, which re-verifies.

**Consequence.** Valid observations fail whenever another agent edits or rebuilds during a long
assessment, which forces full reruns. There is also one more full hashing pass.

**A simpler realization must preserve:** attribution of each observation to the inputs it ran
against, and detection of changes inside its own window.

**Counterevidence.** The current behavior never produces a false pass.

## F. Execution, coordination and recovery

### <a id="dta-041"></a>DTA-041 — Disposable fixture databases receive an evidence-grade durable, fenced, multi-index metadata protocol, paid on every test

**Principles:** AP-07, DP-16, DP-19; H1, H4, H24, H25, H27; companion §6, §8.

*Corrects the earlier "re-examined and not raised" entry.* That entry judged the resource store
under DP-13/DP-16 for library equivalence. This finding judges its proportionality (H25).

**Machinery per passed Rust canonical test.**
- Two Python interpreter launches loading about 22,000 lines (`crates/pse-operations/src/testing.rs:140-219`).
- About 26 fsyncs for registration ([DTA-038](#dta-038)), plus about 6 for drain, about 6 for
  projection (`scripts/test_resources.py:586-695`) and about 10 for the reclaim claim and
  settlement.
- One `surreal sql` CLI process for removal.

**What the protocol guards against.**

| Element | Location | Guards against |
|---|---|---|
| Identity index, identity-pending files and a hash-collision refusal | `scripts/resource_store.py:423-454` | A collision of names that `register` already requires to be fresh opaque UUIDs (`test_resources.py:480-485`) |
| fsync of file, directory and parent on every publish | `resource_store.py:75-93` | Power loss while writing metadata about disposable data |
| Lock files that "survive phase changes and disposal" | `resource_store.py:1-7` | Lock retirement races. *Local observation:* about 3,600 resource, 1,650 identity and 430 invocation lock files |
| Pending/uncertain membership, freeze and member settlement, resumable projection, inspection-first abandonment | `resource_store.py:468-612`; `scripts/invocation_disposition.py` | Interruption between metadata writes for disposable databases |
| A v1 legacy refusal fence and staged migration, re-checked on every store operation through `ensure()` | `resource_store.py:141-178,614-800` | Older worktrees running v1 writers. One worktree exists |

**Actual participants and obligations.**
- Participants: one developer plus agents on one host, a few concurrent build owners and server
  states.
- Failure domains: runner kill or signal, test crash, server restart, and rarely a host reboot.
- Obligations:
  - (a) never remove a database a live session or test uses;
  - (b) retain failed or unknown databases for diagnosis;
  - (c) never turn missing evidence into a pass;
  - (d) let server lifecycle see live borrowers;
  - (e) remove passed state.
- The server already owns an authoritative inventory of existing databases (`INFO FOR NS`,
  used at `test_resources.py:1462-1519`), and owner liveness already exists as pid + start +
  boot id.

**Consequence.** The fixed per-test metadata cost in [DTA-018](#dta-018) is largely this protocol
rather than isolation itself. It buys power-loss, collision and legacy-writer protection for
state nobody needs after a crash. About 3,000 lines of tooling and 2,700 lines of tests maintain
it.

**A simpler realization must preserve (a)–(e).**
- *Lead:* invocation-scoped database or namespace naming, a per-invocation append-only fixture
  log, existing owner liveness, batched removal of authentic passes, and a sweep keyed on the
  server inventory.

**Counterevidence.**
- The same registry manages evidence resources with stronger obligations; those were not
  reassessed here.
- It never deletes anything uncertain.
- ADR-0174 (proposed) rejected a database dependency for this metadata.

**Uncertainty.** The elapsed share is unmeasured.

**Related.** [DA-025](../audit/execution-fit.md#da-025) concerns production persistence sizing, a
distinct cause. It is linked by title only.

### <a id="dta-042"></a>DTA-042 — Test retention does not converge, and accumulated state rides on the shared server and on the production store's lifecycle path

**Principles:** AP-07, DP-19, DP-20; H10, H11, H16, H17, H24, H25; companion §6, §8.

**Diagnosis.**
- *The whole run is the retention unit.*
  - `complete` requires no report error (`scripts/test_run.py:116-118`).
  - When a run is not complete, `finalize` marks every member incomplete and pinned
    (`scripts/test_resources.py:669-679`), including tests with authentic passes.
  - Every fixture is registered pinned and incomplete in the first place (`:543-544`).
- *Teardown removes one database at a time and stops early.*
  - Removal is one CLI process per database with a 90 s timeout (`:43,1522-1577`).
  - The loop sits inside one `try`, so it stops at the first exception, and it is skipped after
    any signal or deadline (`test_run.py:396-406`).
  - A failed removal re-pins a passed database (`test_resources.py:1700`).
- *Deferred collection is tiny.* The reclaim sweep visits at most 64 entries and reclaims at
  most 4 per call (`:1213-1263`), and it is invoked only at the end of validation groups
  (`scripts/validation.py:1137`).
- *Legacy invocations have no automatic exit.* Imported v1 invocations are `unresolved` and
  leave only through manual abandonment (`resource_store.py:772`).
- *Server persistence.* Disposable databases live on the deployment-class in-memory server,
  which syncs every commit and takes periodic full snapshots under a memory limit
  (`scripts/surreal_server.py:7040-7051`).
- *Production coupling.* `require_borrowers_drained` (`surreal_server.py:525-569`) iterates every
  active registry entry, each lookup locked under one 5 s clock. It runs on stop, park, backup
  copy, payload freeze, readmission and module qualification.

**Local observation (read-only, 2026-10-23).**
- 3,655 resource records, 2,916 still active. Of the active records:
  - about 1,750 in 49 legacy `unresolved` invocations, about 720 of them passed but never
    removed;
  - 217 incomplete from one 4,779-test run. That run had one report error, while 2,759 of its
    tests had authentic passes.
- One server state held 389 retained database records; its snapshot file was 338 MB on 2026-10-22.

**Consequence.**
- Retention is effectively monotonic, and the recovery burden is manual.
- Later runs share the server with retained data. *Suspected:* that costs memory and snapshot
  work, and growth could approach the server's memory limit and refuse writes for every
  canonical test (H11).
- The production store's lifecycle operations scale with retained test metadata. *Suspected:*
  the scan can exhaust its clock at some registry size.

**Claims compared with code.** §24.1 says "Bounded maintenance follows active/unresolved paths".
The lifecycle scan is unbounded, and the active set grows.

**A simpler realization must preserve:** retention of failed state for diagnosis, conservative
handling of unknown ownership, and never turning missing evidence into a pass.
- *Lead:* per-test disposition from authentic results, batched removal of every pass,
  age/liveness expiry for unknown state, and lifecycle checks scoped to that server's live
  borrowers.

**Counterevidence.**
- Conservative retention is the stated design.
- The abandonment route exists and is tested.
- A stopped server pays nothing.
- Disk space is maintainer-managed and is not raised here.

**Uncertainty.**
- How the snapshot divides between deployment and test data.
- The current scan time.

**Related.** [DTA-015](evidence-routes-and-execution.md#dta-015) (bare routes) is one inflow and
keeps its own identity.

### <a id="dta-043"></a>DTA-043 — Shared-store and host capacity are coordinated per layer: host admission by recipe name, nextest per run, and the shared server by nobody

**Principles:** AP-07, DP-20; H21, H23; companion §6, §8.

**Diagnosis.**
- *Host admission by recipe name.* `host_admission.classify` chooses a class from command words
  (`scripts/host_admission.py:326-375`). `unit-package`, `unit-libraries` and `native-python`
  declare `--recipe-default-class exclusive` (`justfile:529,536,1429`), whatever they select.
- *nextest per run.* nextest test groups and `threads-required = "num-test-threads"` are
  per-run semaphores (`.config/nextest.toml`).
- *The shared server by nobody.* One SurrealDB server per state is shared by every worktree and
  session (`scripts/worktree.py:68-72` copies `.envrc.local`). It runs with fixed thread,
  memory and session limits (`scripts/surreal_server.py:1325-1333,5752-5754`). No layer lets
  that capacity limit upstream admission.

**Consequence.**
- *Unbounded mode* (the maintainer's selection). Test recipes resolve to `build`, which has 4
  slots (`.config/agent-capacity.toml:21-22`; `scripts/pse_env.py:360-388`).
  - Up to four concurrent runs, each with a 16-wide store group, plus Python workers, share one
    server.
  - An "exclusive" timing-sensitive store test excludes only its own run.
  - *Suspected:* intermittent failures of the sub-second controls under another session's load.
- *Bounded mode* (the repository default when `.envrc.local` is absent).
  - A focused run, even of pure tests, takes the host-exclusive class for resolution, compile,
    run and teardown.
  - It parks parkable stores, which stops and later restarts the shared server
    (`host_admission.py:1751-1775`).
  - Competing sessions are refused after the admission timeout.

**A simpler realization must preserve:** 16-lane SurrealDB-coordinated parallelism and the
unbounded-mode choice.
- *Lead:* a server-scoped exclusivity lease taken by store tests whatever run they belong to,
  and admission derived from the selection rather than the recipe name.

**Counterevidence.**
- Width is derived from the admitted allocation.
- In bounded mode, exclusivity does make per-run exclusion host-wide.

**Related.** [DTA-016](evidence-routes-and-execution.md#dta-016) covers name-list scheduling, a
different cause.

### <a id="dta-044"></a>DTA-044 — All Python canonical component tests run serially on one xdist worker, while their Rust equivalents run 16-wide

**Principles:** AP-07, DP-20; H22; companion §6, §8.

**Diagnosis.**
- Any test whose fixture closure includes `canonical_substrate` is marked
  `xdist_group("canonical-owner")` (`conftest.py:283-285`).
- The runner adds `--dist loadgroup` (`scripts/native_tests.py:560-575`), and xdist runs a group
  on a single worker.
- The isolation premise is the one that lets Rust run these concurrently: a fresh UUID database
  per test (`python/pse/tests/conftest.py:96`; `.config/nextest.toml` store group).
- The group's dependency is not recorded in code; it was introduced in `d0f2c4181`.

**Consequence.** `just py-test`'s wall-time floor is the serial sum of all canonical tests, while
the other workers idle. *Measured context:* in `native-python.xml` above, the canonical-owner
items summed to 597.8 s of a 602.1 s session.

**A simpler realization must preserve:** whatever real dependency motivated the group, once it is
identified. Serialization does also lower load on the shared server ([DTA-043](#dta-043)).

**Uncertainty.** The motivating dependency is unknown.

### <a id="dta-045"></a>DTA-045 — The `scientific` group runs a few hundred independent solver unit tests one at a time, beside memory-heavy journeys

**Principles:** AP-07, DP-20; H21, H22.

**Diagnosis.**
- `scientific = { max-threads = 2 }` with `threads-required = 2` admits one test at a time
  (`.config/nextest.toml:10,89-95`).
- About 390 test attributes in pse-backend-native solver modules, plus every `solve_*` test,
  are assigned to it.
- At `84a1caf17` these ran in `solver = { max-threads = 16 }`, commented "Independent solver
  tests use the sixteen-worker test budget. Native BLAS/OpenMP teams remain one thread each".
  `d0f2c4181` merged them into `scientific`.
- BLAS and OpenMP threads are still pinned to 1.

**Consequence (suspected).** Native-test wall time includes the serial sum of the group, and
solver unit tests queue behind 64–128 GiB journeys.

**A simpler realization must preserve:** single-file execution for memory-heavy journeys and for
genuinely pool-owning solvers (the `pounce` feature pulls in rayon).
- *Lead:* separate groups by resource class.

**Uncertainty.** Which solver units actually own pools is not established.

### <a id="dta-046"></a>DTA-046 — Per-process runtime and per-context receiver budgets are each sized to the whole allocation, then multiplied by runner width

**Principles:** AP-07, DP-20 (coordinate nested pools; exceeding a budget fails explicitly); H17,
H21; the binding's "independently maximized … pools" defect shape. Moderate to low severity.

**Diagnosis.**
- Each Python worker's `inspection_settings` declares a 64 GiB memory limit and a 16 GiB cache
  (`python/pse/tests/conftest.py:263-279`). That equals the whole observer allocation
  (`.config/agent-capacity.toml`), multiplied by the number of workers.
- The only check compares one runtime with the whole allocation
  (`crates/pse-runtime/src/workflow/worker.rs:1638-1642`).
- On observer routes, each canonical fixture registers a receiver context with a 64 GiB pool and
  16 CPU threads (`scripts/surreal_server.py:860-994`). Placement checks one receiver against
  the whole slice (`:2096-2101`).

**Consequence.** In-process budgets cannot fail explicitly before the cgroup limit (bounded mode)
or the host limit (unbounded mode, the class of the recorded OOM incidents).

**Counterevidence.**
- These are ceilings, not reservations.
- The ordinary Rust fixture is coherent: 4 GiB × 16 = 64 GiB (`crates/pse-runtime/src/workflow/tests.rs:127-209`).

**A simpler realization must preserve:** the finite-role contract.
- *Lead:* derive per-worker and per-context budgets from the admitted width.

## G. Whole-system economics

### <a id="dta-047"></a>DTA-047 — Report retention seals, references and sweeps assessment evidence that no producer can ever make disposable

**Principles:** AP-07, DP-16, DP-23; H26, H27, H28; companion §2, §3.

*Corrects the earlier "re-examined and not raised: reference-protected retention".* The new
evidence is that disposal is unreachable.

**Diagnosis.**
- *Every validation run is registered and sealed.* That includes `turn-end`, `ready`, `hygiene`,
  `clippy` and `governance`. Each run:
  - registers a pinned evidence resource (`scripts/validation.py:726`;
    `scripts/test_resources.py:813-852`);
  - seals it by walking and digesting every file (`test_resources.py:1112-1210`);
  - sweeps registry records (`:1213-1263`).
- *Removal can never happen.* It requires sealed `scratch` artifacts (`:1338-1346`), and
  `artifact_roles` never assigns `scratch`. Its own comment says "Current producer outputs are
  all receipts/provenance/consumed evidence" (`validation.py:655-661`). Reclaim therefore only
  marks records "compacted".
- *Group runs stay pinned by construction.* They set `capture=not args.group`
  (`validation.py:1286`).
- *Protection guards nothing.* References and borrows protect against a removal that cannot
  occur, and self-tests exercise the unreachable disposal path.
- *A sibling with no registration at all.* Native invocations write `build/native-tests/<ns>/`,
  consumed only by the same invocation's child, with no reclaim (`scripts/native_tests.py:1444-1447`).

*Local observation:* `build/assessment` held about 980 run directories (272 of them `turn-end`)
and `build/native-tests` 566.

**Consequence.** Per-run auxiliary work and maintained protection machinery for an action no
producer enables. Disk space is maintainer-managed; the point is the dead disposal path.

**A simpler realization must preserve:** never deleting evidence a reuse depends on. Reuse already
re-verifies origin digests and fails loudly (`scripts/validation_receipts.py:174-195`).
- *Lead (remove first):* drop registration, sealing and sweeping for report directories, or let
  producers declare disposable roles. Give native-test outputs the invocation's lifetime.

**Related.** A sibling of [DTA-019](evidence-routes-and-execution.md#dta-019);
[DA-024](../audit/local-reasoning-and-assurance.md#da-024) by title only.

### <a id="dta-048"></a>DTA-048 — Assessment receipts copy per-gate source maps and full failure payloads and rewrite them whole at every checkpoint

**Principles:** AP-07, DP-21; H16, H26; companion §6, §8.

**Diagnosis.**
- *Whole-receipt rewrites.* `checkpoint()` rewrites `checks.json`, `test-findings.json`,
  `failures.json` and `summary.md` (`scripts/validation.py:573-612`). It runs at three to five
  points per gate.
- *Copied source maps.* Every check record's `inputs` holds the projected file map for its scope
  (`:870-872`; `scripts/validation_scope.py:506-529`), duplicating `source_files`. Product
  scopes are about 10,600 entries.
- *Copied failure output.* Captured failure output is copied from JUnit into the XML copy, into
  `results.details`, `failures.json` and `test-findings.json`, and into the frozen invocation
  state and `terminal.json`.
- *Native provenance.* It embeds, per invocation, the source map, the full unit graph and a copy
  of the Cargo metadata (`scripts/native_tests.py:1081-1084,1160-1171`). It is compared only
  for equality.

**Consequence.** Write volume grows with gates² × map size and with failure-output size. A
receipt from before the baseline (version 4, 2026-10-04) illustrates the structure: about 420 MB
each for `checks.json`, `failures.json` and `test-findings.json`, beside an 848 MB XML.

**A simpler realization must preserve:** changed-file diagnosis, crash-safe checkpoints and
`just result`.
- *Lead:* one snapshot plus a per-gate scope digest, failure output referenced by test-case
  identity, and append-only records.

**Counterevidence.** Writes are atomic replaces, and group runs capture no provenance.
