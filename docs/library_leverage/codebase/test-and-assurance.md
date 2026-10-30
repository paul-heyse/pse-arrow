# Test and assurance

This page covers how the repository establishes its correctness claims: test code and shared test support, fault and concurrency testing, fuzz, mutation and coverage routes, property testing, corpora, benchmarks, and the assurance tools pinned for them. Contracts and qualification meaning are owned by
[§23](../../authoritative_design/sections/operations-and-validation.md#section-23) and
[§24](../../authoritative_design/sections/operations-and-validation.md#section-24) of `operations-and-validation.md`. The separate diagnostic account under `docs/design_review/test-audit` is not used here. The baseline is on the [entry page](README.md). Mechanisms were read statically and are *Interface-checked*; every value claim is *Proposed*, and nothing is *Measured*. Counts are `rg` heuristics over `crates`, `tests`, `benches` and `xtask`, so read them as orders of magnitude.

**System picture.** There are about 6,350 `#[test]` and `#[tokio::test]` attributes in about 940 files, mostly inline `#[cfg(test)]` modules or sibling `_tests.rs` files, with several test files over 5k lines. The practice prefers real owners: a real server, real solvers and production constructors, with `pse-testkit` thin. Nextest is the runner, Criterion lives in `benches/`, `packages/reference` is authored scientific data run by a separate `xtask conformance` runner, and independent Python oracles feed exchange tests. There are 12 `proptest!` blocks in 11 files, `insta` at 5 call sites in 2 crates, and one fuzz target.

Already strong, and not argued against here:

- `pse-testkit::Capture` and `Coverage` make truncated evidence inconclusive instead of a pass.
- Proptest discipline where it is used: fixed seeds, 1,024 cases, committed regressions.
- Most governance tests parse with `syn`, `cargo_metadata`, `toml` and the live registry instead of grepping.
- The real-peer gRPC double (`canonical_grpc_peer`) uses hyper and the supplier's protocol crate, not hand-built frames.
- Reference oracles import nothing from the product, and the parity suite fails instead of skipping.
- The nextest configuration has rich profiles, JUnit output and a flaky-hunt profile that fails rather than launders.

Opportunities cluster where assurance depends on whoever remembered to write a case ([CBO-157](#cbo-157), [CBO-158](#cbo-158), [CBO-159](#cbo-159), [CBO-160](#cbo-160)), where one test identity covers many cases ([CBO-161](#cbo-161)), and where measurement and fixtures are assembled by hand ([CBO-162](#cbo-162), [CBO-163](#cbo-163)). The nextest group declarations that nothing assigns are [CBO-049](developer-tooling.md#cbo-049), and the two test front doors are [CBO-052](developer-tooling.md#cbo-052).

**Coverage on this page.** Read closely: the canonical client fault seam (`canonical_client.rs::execute`, `canonical_client/owned.rs`, `canonical_grpc_peer.rs` start-up and dispatch, the control and execution hook sites), `pse-testkit`, `.config/nextest.toml`, the heads of all 13 governance tests, the benchmark measurement paths (`k4/preparation.rs`, `native_process.rs` and `phases.rs`, `results_analysis::measure_complete`), and the observation hub's `Budget`, `Reservation` and `Metric`. Representative: conformance acceptance journeys (`global_certification`, `vessel_fit`), proptest blocks, test-support inclusion sites. Skimmed or unread: the bodies of the concurrency owners (`stop`, `cancellation`, `transport_lifetime`), host-side crash tests, `canonical_settlement` classification, parser bodies, most test modules (shape and counts only), `benches` `compact`, `review_cases`, `extended` and `restart`, and the `xtask` conformance runner. No test was run, so no claim here is about whether any test passes.

<a id="cbo-157"></a>

## CBO-157 — Durable-effect fault injection is per-site flags in production modules

- **Operation and consumers.** Tests of unknown-outcome, lost-acknowledgement and failed-readback settlement in the canonical store (a central contract, §20.6) and in native lifecycle: execution, artifacts, control, progression, projects, files and pacing.
- **Current mechanism.**
  - About a dozen `tokio::task_local!` flags sit under `cfg(any(test, feature = "test-support"))` or `cfg(all(test, feature = "canonical-tests"))` inside production modules: `CONTROL_ACK_LOST`, `CONTROL_READBACK_FAILED`, `PREPARED_ACK_LOST`, `PREPARED_READBACK_FAILED`, `PROJECT_REPLY_LOSS`, `APPEND_GROUP_REPLY_LOSS`, `ARTIFACT_APPEND_REPLY_LOSS`, `EXECUTION_ACK_FAULT`, `EXECUTION_ACK_READBACK_FAILURE`, `CANCEL_AFTER_ARTIFACT_APPEND_ACK`, `FILE_EFFECT_PAUSE` and `CREATION_OBSERVER`.
  - Each has inline hooks that replace an `Ok` response with `CanonicalError::IncompleteResponse` after a real commit, and `pse_operations::testing` exports a named wrapper per fault. `test-support` cfg sites occupy about 20 files, including struct fields threaded through `canonical_client::Query`.
  - The hooks sit above the client, in the caller's future. Every operation passes `Client::execute`, which runs inside `tokio::spawn` with `REQUEST.scope` set inside the spawned task, so a caller's task-local does not reach it. A fault selected by a test would have to be captured at `Query::into_future` and carried in `RequestContext`. Operation labels exist for about half the call sites (about 154 `protected_query` calls against about 294 `.query` and `control_query` matches).
- **Why it matters.** Exploring a new effect window means a new flag, new cfg sites and a new wrapper, so coverage follows who remembered, and the windows cannot be enumerated to ask "fail at every effect boundary of this operation". These are the paths ordinary runs never exercise. A client-operation seam could cover the reply-loss and readback group, about 8 of the 12 or so named store faults, and let a test run an operation once to record the labels it crossed, then re-run failing at each with one oracle: settled results and receipts equal the no-fault run. The `canonical_execution` reply-loss test asserts this by hand today. A seam would leave a shrunken residue, so "one seam" is too strong (Proposed).
- **Patterns.** LOP-X021, LOP-12, LOP-09. The effect under test is [LOP-X012](../assessment-patterns.md#lop-x012).
- **Evidence.** `canonical_control.rs` head, `testing.rs` head, `canonical_client.rs` (`Query`, `execute`), `canonical_client/owned.rs` (`OwnedHttp2` as a `Service`), `canonical_execution.rs` fault declarations and its comment that controlled faults alter the returned acknowledgement while server effects and recovery reads stay real, `canonical_grpc_peer.rs` head, and the hook sites in `canonical_artifacts` and `canonical_files`. `rg task_local!` found the flags.
- **Footprint.** Only the control and execution hooks were read in full. Host-side crash tests and `canonical_settlement` (whether a transport-level failure reaches `settle_exact` with the same uncertain classification) were not read.
- **Preservation constraints.** Genuine server effects and recovery reads stay real; the fault alters only what the local caller observes and must not leak a known acknowledgement; per-test isolation while sixteen clients share one server; production builds contain nothing; deterministic replay.
- **Counterevidence and uncertainty.**
  - Task-local scoping is well chosen: per-task, no registry.
  - Some faults are not reachable by one seam. Wire-level misbehaviour (unsupported statement frames, sparse kill, compressed response, held detach) stays with the peer, with about 33 uses in `canonical_transport_tests`. `FILE_EFFECT_PAUSE`, filesystem effect failures and `CANCEL_AFTER_ARTIFACT_APPEND_ACK` need named points. Client death after commit and server crash need a process boundary. Native solver and worker faults are a separate domain.
  - A transport-level injector would force connection loss and a new generation, which is a different and more realistic test, not a replacement. Process-global failpoint libraries fit nextest's process-per-test only for the in-process client and add no labels.
  - Not every task-local is a fault: `REQUEST`, `RESPONSE_OWNER`, `RETAINED_REQUEST`, `OPERATION_DEADLINE` and pacing `CURRENT` are production carriers.
- **Status.** Current mechanism. A systematic effect-boundary sweep is Proposed.
- **Research questions.**
  - Can a test enumerate the windows an operation crossed, then replay it failing at each?
  - Does a scoped failpoint facility, a declared effect-boundary registry or a client-layer injector best fit a shared in-process service?
  - What residue of named points remains afterwards?

<a id="cbo-158"></a>

## CBO-158 — Ownership and settlement concurrency has no schedule exploration

- **Operation and consumers.** Hand-built concurrency owners whose correctness is architectural: `pse-diagnostics::{stop, cancellation}`, `pse-operations::transport_lifetime`, `pse-engine::resources::ReleaseNotifyingSemaphore`, `pse-columnar::resource`, the observation hub's `Reservation`, budget and `Weak` links, and single-flight preparation.
- **Current mechanism.** Tests use real threads and tasks, barriers, gates and short sleeps. A manifest search found no `loom`, `shuttle` or `turmoil`, and the workspace pins none. One narrow Miri recipe exists (`scientific-replay-miri`, on `pse-ids`).
- **Why it matters.** "Release only after actual settlement" and "a ready stop wins" are interleaving claims. A test run covers whatever order the scheduler picked, and the failure mode is a rare leak or hang. Controlled-schedule testing of small extracted kernels could establish orderings that integration tests only sample (Proposed).
- **Patterns.** LOP-X021, LOP-12, [LOP-X005](../assessment-patterns.md#lop-x005).
- **Evidence.** `observation.rs` (`Budget`, `Reservation`, `Node`), `Metric`, manifest search, the `justfile` Miri recipe. For `stop.rs`, `cancellation.rs` and `transport_lifetime.rs`, only atomic and condvar counts and the `Cancellation::wait_until` signature were read.
- **Footprint.** The primitives were identified; their bodies were not all read.
- **Preservation constraints.** No production cost, with instrumentation under test cfg only. These owners mix eventfd, pidfd and tokio, so only pure-state kernels are candidates.
- **Counterevidence and uncertainty.** Schedule explorers need their own atomics or sync types swapped in and grow quickly with model size. Many owners cross OS facilities neither can model, so the value depends on separating small pure state machines that may not exist in that form. The maintainer's preference for real owners is strong, so this complements them. Repeated runs with a stress count are a cheaper first step.
- **Status.** Proposed.
- **Research questions.**
  - Is there one extractable kernel (budget reservation, or stop and settle) on which a schedule explorer can be tried?
  - What does a repeated-run stress profile already establish?

<a id="cbo-159"></a>

## CBO-159 — Fuzz, mutation and coverage tools are pinned with no standing route

- **Operation and consumers.** Bounded readers of untrusted or corrupt input: authored `.pse` parsing (`language::parser`, `dsl::parser`), package and YAML admission, frozen documents, review-pack readers, handoff readback, Arrow IPC table reads and `/proc` parsing. Also assessing how much of the test mass constrains behavior.
- **Current mechanism.**
  - One `fuzz_target!` (`xtask/src/bin/frozen-inspection-fuzz.rs`) with a seed generator that has one seed. No `fuzz/` directory exists.
  - `pse-authoring`'s `arbitrary` feature is commented as serving fuzzing harnesses, yet no fuzz target consumes it; only the DSL round-trip proptest does.
  - `cargo-fuzz`, `cargo-mutants` and `cargo-llvm-cov` are pinned in `.config/diagnostic-tools.toml` and installed by `scripts/bootstrap.sh`, but a search of `justfile`, `crates/pse-host/src`, `xtask/src` and `docs/dev` by tool name found no invocation; `docs/dev/validation-assessment.md` mentions them only as optional external tools.
- **Why it matters.** The parsers promise refusal, not panic, and declared finite bounds are a claim that fuzzing tests cheaply. Mutation testing and diff coverage can show which part of the large test mass constrains behavior, supporting safe retirement of redundant tests (Proposed).
- **Patterns.** LOP-X021, LOP-12, LOP-03. Guards to check against input are [LOP-X009](../assessment-patterns.md#lop-x009).
- **Evidence.** The fuzz target and seed binary heads, `xtask/Cargo.toml`, `.config/diagnostic-tools.toml`, the matching line of `validation-assessment.md`, a search of parse entry points.
- **Footprint.** Parser bodies were not read. The host test workflow and `xtask` evidence tooling were read by their own passes, so a route could live in either, and that placement is open.
- **Preservation constraints.** Finite campaigns that do not change qualification status; fuzzed targets must be pure.
- **Counterevidence and uncertainty.** A plan note (`docs/plans/52c-behavioral-and-scientific-verification.md`, read only through a search hit) says automatic review rejects long-running fuzz tasks, so a route may have been removed on purpose. Mutation testing on a workspace this large is expensive and useful mainly by diff.
- **Status.** Tools pinned: current mechanism. Standing routes: Proposed.
- **Research questions.**
  - Can a structured-input fuzz target be built over the existing AST generator for the two parsers?
  - Can mutation testing by diff run with the nextest profile used elsewhere?

<a id="cbo-160"></a>

## CBO-160 — Property testing stops at leaf crates

- **Operation and consumers.** Admission, retention, windowed-counter and comparison logic, and scientific invariance claims such as unit scaling, component permutation and identity hashing under reordering.
- **Current mechanism.**
  - `proptest` is declared in five crates (`pse-authoring`, `pse-ids`, `pse-math`, `pse-quantity`, `pse-runtime`). Blocks cover encodings, floats, DSL round trip, dimension and unit algebra, `implicit_product_tests` and `workflow/uncertainty`. One `.proptest-regressions` file exists.
  - Hand-rolled generators elsewhere: LCG and SplitMix-style streams in `highs/mip_tests`, `scip_tests` and `factorable_tests`.
  - No state-machine testing was seen. `Metric::{delta, elapsed, recent}` has a non-trivial wrap, partial and out-of-order contract and is tested by examples in the 1,300-line hub test module.
- **Why it matters.** Model-based checking suits a bounded ring with an exact cumulative count, reservation accounting and cursor paging. Metamorphic relations are a strong oracle where analytic references are scarce, and they complement the analytic, exhaustive or independently generated references that §24.1 asks for (Proposed).
- **Patterns.** LOP-X021, LOP-12, LOP-03.
- **Evidence.** The `proptest!` file list and config, `Metric`, `implicit_property_tests.rs` head, `RngSeed::Fixed` configurations, the LCG lines by search.
- **Footprint.** Example shapes were sampled, not surveyed, and the hub test module was skimmed by name only.
- **Preservation constraints.** Fixed seeds and committed regressions, already practised; slow native oracles need small case counts.
- **Counterevidence and uncertainty.** Scientific generators need well-posed inputs, which is real work. Hand-rolled streams in native solver tests may be deliberate, to reproduce solver instances without depending on a library's stream stability ([LOP-X006](../assessment-patterns.md#lop-x006)); a pinned proptest stream is already treated as evidence in `implicit_product_tests` ("ChaCha seed 5202").
- **Status.** Proposed.
- **Research questions.**
  - Can a state-machine harness check the hub's metric kernel against a simple model?
  - Which scientific invariances have well-posed generators at small case counts?

<a id="cbo-161"></a>

## CBO-161 — Corpora and tables run inside one test identity; test mass and burden

- **Operation and consumers.** Developers selecting or re-running a failing case, and nextest and `scripts/pse affected` reports. Maintainers working in the modeling-workflow tests.
- **Current mechanism.** Two related manifestations.
  - **Corpora and tables.** `domain_schema::corpus()` reads `tests/fixtures/domain-refusals/*.pse` (headers `// refusal:`) and iterates inside tests. There are 223 `for (...) in [` table loops in 97 test-named files. Reference fixtures run through `just seed-conformance` in a separate `xtask` runner with its own Arrow reports and fixture ceilings, not as nextest tests. No `rstest`, `test-case`, `libtest-mimic` or `datatest` dependency exists.
  - **In-file test mass in the modeling-workflow scope.** About 40 percent of that scope's lines are inline tests. By test names plus samples (not per test): lifecycle tests (`conformance_parallel_*` with `FixtureProbe`, `reuse_tests.rs` cancellation, waiter and producer-expiry cases) guard locally coordinated cancellation and single-flight mechanisms, and 18 `cfg(test)` markers are woven into `conformance.rs` production code. Per-file `package`, `prepare`, `profile`, `case` and `runtime` helpers repeat. Six one-line `explicit_{ipopt,pounce,scip}_root_*` functions form a hand matrix. `#[cfg(test)]` work tuples in `delta.rs` duplicate observation counters on the same path. Authored journeys in `forms_tests.rs`, `global_tests.rs` and `certificate_tests.rs` are driven from Rust while a data-authored route (`.pse` fixtures through `conform`) exists. Tests with an independent analytic oracle, such as `pr_jacobian_tests.rs`, and refusal-meaning tests assert scientific or contract meaning and are not burden.
- **Why it matters.** A failing case stops the rest of a loop, cannot be selected by filter and has no independent timeout or history. A new corpus file adds no visible test. In the test mass, the lifecycle share measures the burden of locally coordinated mechanisms ([CBO-079](execution-lifecycle-and-observation.md#cbo-079) and neighbours) and would shrink only if they did; plumbing and counters could be shared or moved. Effort could go to fixture ownership, table-driven matrices, a fault seam in place of woven probes ([CBO-157](#cbo-157)) and moving structural rule tests down so they run without a runtime (Proposed).
- **Patterns.** LOP-12, LOP-09, LOP-02, LOP-X003 (not fitted).
- **Evidence.** `domain_schema.rs::corpus`, the table-loop count, `justfile` `seed-conformance`, `conformance.toml`, a listing of `tests/conformance/src/acceptance`. For the mass, line counts and test names for `dynamics.rs`, `reuse_tests.rs`, `conformance.rs` and `pr_jacobian_tests.rs`, and bodies of the `forms_tests.rs` head, the `root_response_tests.rs` matrix and the `FixtureProbe` head.
- **Footprint.** The `xtask` conformance runner body was not read, table-loop purposes were not classified, and test groups were classified from names and samples, not per test.
- **Preservation constraints.** The fixture run shares resources (a 128 GiB pool and sixteen permits) by design, and per-process runs would break campaign economics; the parity suite fails and never skips; reports carry Arrow outcome rows; independent expected values must not be regenerated from the implementation; feature-gated solver tests keep their gating; `force_validate` obligations.
- **Counterevidence and uncertainty.**
  - Many table loops are small and name the case in the assertion. Process-per-test makes one test per fixture costly when fixtures share expensive preparation, and corpus-to-test generation costs compile time.
  - The lifecycle tests guard real prior failures, in-file tests use private items so moving them has a cost, and snapshots weaken independence if accepted blindly.
- **Status.** Current mechanism.
- **Research questions.**
  - Can a custom test harness make each corpus file a named test without losing shared preparation?
  - Which table loops warrant per-case identity?
  - Which authored journeys can become authored `test` declarations with expected results?

<a id="cbo-162"></a>

## CBO-162 — Benchmarks use Criterion for scheduling and keep the measurement beside it

- **Operation and consumers.** Maintainers making *Measured* claims about complete scientific operations (§24.3).
- **Current mechanism.**
  - `native_process`, `modeling_preparation` and `native_cache` use `iter_custom` with `sample_size(10)`, flat sampling and a tiny warm-up. Each iteration builds a fresh runtime, runs one workload and writes untyped `serde_json::Value` records and `*-memory.json`, read from case specs in `.config/preparation-cases.json` through about 90 `spec["..."]` lookups across about 29 `bench_function` and `iter_custom` sites.
  - The benchmark metadata records "one JSONL record per executed closure, including warmup; Criterion measured samples separate; membership unavailable", and §23.1 repeats that Criterion-to-invocation membership is unavailable. `benches/benches/audit` adds opt-in phase audits.
  - Phase cost comes from a bench-private `tracing` layer (`phases::Phases`) that selects spans by name (`starts_with("pse.case.")` or `== "pse.delta.write_attempt"`). By search, the second name has no producer in `crates/`; the code treats absent phases as having performed no work, so it reads as zero.
  - Bench targets share `tests/support` modules by `#[path]` ([CBO-163](#cbo-163)).
- **Why it matters.** The evidence a claim needs (pool peak, resident memory, phases, work counts, cleanup) lives outside the tool's sample model, and its samples cannot be tied to the records. Comparison and uncertainty apply to elapsed time only, and the code itself says the statistical mode configures 10 flat samples whose "actual operation count is iterations". A typed case and record schema with a sampler that keeps per-sample membership could join time, memory and work, and deterministic counts could give signal without timing noise (Proposed).
- **Patterns.** LOP-10, LOP-03, LOP-06.
- **Evidence.** `k4/preparation.rs::measure`, the `native_process.rs` metadata block, `native_process/phases.rs`, `results_analysis::measure_complete`, `native_cache.rs` top, `audit/mod.rs` head, `preparation-cases.json` head, §24.3.
- **Footprint.** `compact`, `review_cases`, `extended` and `restart` were read by search only.
- **Preservation constraints.** Actual-sample retention; no timing gates; explicit failed or censored samples; cleanup assertions; untimed compilation; teardown outside the timed interval.
- **Counterevidence and uncertainty.** A custom measurement in Criterion holds one value type and would still not hold the records. Replacing it means owning statistics, though §24.3 reports only a mean and standard error. Criterion may still suit microbenchmark-shaped targets such as `native_consolidation`. Instruction-count tools are slow under native solvers and exclude real parallel effects.
- **Status.** Current mechanism.
- **Research questions.**
  - Would a thin typed sampler serve one-shot long operations better than `iter_custom`?
  - Where do deterministic count tools apply, for preparation kernels only?

<a id="cbo-163"></a>

## CBO-163 — Shared test support is included by path, with repeated fixture assembly

- **Operation and consumers.** Test authors across crates and benches.
- **Current mechanism.**
  - `#[path = "../../../tests/fixtures/scientific-inputs.rs"]` appears in at least 5 crates, `native-worker-environment.rs` in at least 4, and `tests/support/{plan14,workflow_runtime}` in benches and others. Each carries `#![allow(dead_code)]` because each binary consumes a subset.
  - `pse-testkit` has about 11 consumers, and its `capture` module one. `testkit::factory` and `NativeFixture::with_settings` duplicate the same `CpuAdmission` assembly, and the `native_cache` bench assembles its own.
  - About 577 inline `package` and `def` source literals appear in 76 test-named files, 131 of them in `workspace/modeling/tests.rs`, with no source builder.
  - Scientific inputs need an environment-bound capsule (`PSE_SCIENTIFIC_INPUTS`) from a separate prepare step and fail by `expect` when unbound.
  - Ad hoc tracing layer doubles (`CandidateLayer`, `execution_context` `Events`) sit beside `testkit::Capture`.
- **Why it matters.** Path inclusion recompiles shared code per binary and defeats dead-code linting, and repeated inline sources change together when the language or manifest shape changes (Proposed).
- **Patterns.** LOP-09, LOP-02, LOP-12.
- **Evidence.** `pse-testkit/src/{lib,capture}.rs`, `scientific-inputs.rs` head, a `#[path]` search, `k4/support.rs` head, and `tests/conformance/src/acceptance/mod.rs`.
- **Footprint.** Representative. Inline sources are often the point of the test and were not read closely.
- **Preservation constraints.** Production admission and constructors stay in fixtures; capsule semantics (immutable inputs, not the mutable checkout); the testkit must not depend on crates that depend on it.
- **Counterevidence and uncertainty.**
  - That cycle may be why path inclusion exists: a dev-dependency on a crate that depends on the crate under test yields two copies of its types, hence `#[path]` in `src/lib.rs` for `pse-runtime`, `pse-modeling` and `pse-compiler`. This is a real constraint.
  - Readable literal sources are good test documentation, and fixture frameworks do not solve cross-crate type duplication. The smallest useful part is likely consolidating the `factory` duplication and retiring `Capture` if unused elsewhere.
  - The acceptance journeys that include `tests/support/plan14.rs` are real pipeline journeys with analytic checks and show no generic burden.
- **Status.** Current mechanism.
- **Research questions.**
  - Which supports can move into a crate that avoids the two-copies cycle?
  - Would a source builder reduce coupling of inline sources to manifest shape without hiding the tested text?

## Examined and not retained

- **Test time is real wall time (about 90 sleeps in test-named files).** Paused Tokio time does not cover the real SurrealDB server, native threads or `Instant`-based native deadlines, so it likely applies to a small share of pure Tokio owners. The surer part is a shared polling helper in place of private `until` helpers, and that is small.
- **Rendered outputs checked by substring and JSON assertions.** `insta` covers 89 CLI snapshots. Whole-output snapshots would help presentation regressions in review-pack and analysis output, but agents accept snapshots blindly and targeted assertions (exact-value and anti-injection checks) carry contract meaning.
- **No shared numerical comparison vocabulary.** About 1,140 tolerance comparisons across about 256 files, with `pse_quantity::numeric::approx_eq` unused by most. Tolerances are scientific statements that must stay explicit, bit-exact checks stay bit-exact, and the benefit is mostly at new tests.
- **The FFI-callback governance check counts braces.** `tests/governance/tests/ffi_callbacks_catch_unwind.rs` matches `extern "C" fn` by substring and counts braces after a crude per-line comment and string strip. By reading, the substring also matches function-pointer type positions (`kinsol.rs`, `petsc.rs` `type Destroy`, `root_isolation.rs`, `idas.rs`), so the scanned "body" can be an unrelated block; it scans only `pse-backend-native` and `pse-buildinfo`. This was not run and its current result was not observed. It is one check that replaces an existing one, so no opportunity is recorded.
- **Relational realization of review-analysis questions.** `ReviewTableRequest` is documented as not an arbitrary SQL request, and the per-field admission accounting would be rebuilt outside the engine. The review analysis and an exported IPC query answer different questions.
- **Bounded versioned document ingress and child-output capture.** `read_checkpoint_source` parses a local header and then the whole image instead of using `decode_versioned`, and `LogDrain`, `StderrCapture` and `drain_child_pipe` each keep a prefix and drain with different waits. Both are small, and the stop-must-not-hang rule explains the polling in `LogDrain`.
