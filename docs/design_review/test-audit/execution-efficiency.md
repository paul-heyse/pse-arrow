# Execution efficiency of the development feedback cycle

This page covers the explicit efficiency investigation that the skill and companion §8 require,
under AP-06 and AP-07 with the efficient-architecture heuristics. It also holds the findings that
investigation produced. Baseline and limits are in the [collection README](README.md). Citations
are `path:line` at source commit `cbda2efc6`. All content here comes from the second,
principle-directed pass on 2026-10-23.

Evidence labels distinguish three kinds of claim:
- *demonstrated work*, from source reasoning (Interface-checked);
- *suspected bottlenecks*;
- *Measured* durations, all read from runs already recorded in the local nextest record store,
  exported as traces. Every tool version and profile is named. No new runs were made.

## Routes investigated

**Focused Rust test (`just unit-package <pkg> <word>`).**
- *Preparation.*
  - Placement and host admission in a supervised scope come first.
  - Then `scripts/select.py` and `scripts/arrow_validation.py` resolution: `cargo metadata`
    and one or two `--unit-graph` resolutions.
  - `nextest list` follows, building the package's library test binary with the composed
    validation features.
  - A per-invocation nextest configuration copy (`scripts/test_run.py:516-540`) precedes
    `nextest run`.
- *Teardown.* Reconciliation and reclaim.
- *Build.* Inline and sidecar tests live in each crate's `src`, so a one-line test edit
  recompiles that crate's whole library test binary. pse-runtime alone has about 59,600 lines
  of sidecar tests. That is not inherently avoidable, and its cost was not measured.
- *Store tests.* For store-touching tests, the route needs a separate compile-time feature
  variant ([DTA-029](#dta-029)). Each test also pays a Python bridge round trip and a full
  schema installation
  ([DTA-018](evidence-routes-and-execution.md#dta-018)).
- *Diagnosis.* Failure diagnosis uses the terminal file and `just result`
  ([DTA-012](evidence-routes-and-execution.md#dta-012)).

**Broad library units (several packages, `--lib`).** Wall time is set by a few tests that
specialize the full reference closure ([DTA-030](#dta-030), *Measured*).

**Focused Python test (`just py-unit -k <expr>`).**
- *Before collection* (`scripts/python_extension.py:102-135`), three commands run under a
  compile-class placement:
  1. `uv sync --locked --no-install-project`;
  2. `maturin develop` with `force-validate`, which builds the pse-py closure, then packages
     and installs a wheel;
  3. `cargo run -p xtask … python-stubs`.

  An extension-kind probe follows. Then pytest starts its xdist workers. Each imports `pse`
  and the root conftest, which loads the native libraries with ctypes. Session invariant
  fixtures run on demand, not automatically.
- *Suspected bottleneck.* How much of the first three steps is a fast no-op when nothing changed
  was not measured. It is the fixed price of every focused Python run, including runs that
  touch only pure-Python code. It is reported as a suspected bottleneck, not a finding.
- *Store tests.* Each component test using `canonical_substrate` initializes a full database
  (DTA-018).

**Broader qualification (`just assessment`).** The comprehensive route runs the shared native
graph once, with separate feature-absence controls and one Python scope (`scripts/validation_scope.py`).
Its composition was read, not timed. No avoidable repetition beyond the per-route costs above
was established.

**Replay.** nextest's record store keeps per-test timings and command lines. That makes later
attribution possible without re-running, and it is how the measurements below were taken.
Ignored and env-gated controls are not replayable from committed routes
([DTA-013](evidence-routes-and-execution.md#dta-013),
[DTA-014](evidence-routes-and-execution.md#dta-014)).

**Not established.** Investigated but left open:
- compile time per edit for a pse-runtime test edit (`scripts/build_measurements.py` probes
  only pse-compiler edits);
- xdist startup cost;
- the per-test split of fixture cost;
- server contention.

## <a id="dta-029"></a>DTA-029 — Store-test selection is a compile-time feature on production crates, which also compiles test hooks into the library

**Principles:** AP-07, AP-05, AP-02; companion §6, §8; heuristics on avoidable compilation.

**Diagnosis.**
- `canonical-tests = []` (`crates/pse-operations/Cargo.toml:32`) is propagated by pse-runtime
  and xtask. It gates about 270 cfg sites in 62 files.
- Under `cfg(feature = "canonical-tests")` without `test`, it compiles test-only behavior into
  the **library**:
  - a task-local hook that cancels and suspends an artifact write after its append is
    acknowledged (`crates/pse-operations/src/canonical_artifacts.rs:20,508-514`);
  - an extra stderr print on operation failure (`canonical.rs:2457-2466`);
  - further sites in the maintenance, project, selection and study-module owners.
- Recorded runs on 2026-10-23 used ad hoc feature mixes. Run `217bd5e6` combined
  `pse-runtime/canonical-tests,pse-runtime/solver-ipopt,pse-operations/evidence-query,xtask/…`.

**Consequence.**
- Each route compiles its own variant of the largest crates.
- A local observation, not a timing or an attribution: the build directory held 34 distinct
  pse-operations unit directories and 33 for pse-runtime (`target/debug/build/<crate>/`). These
  mix libraries, test binaries, feature sets and stale revisions.
- Compiled-out tests do not appear in `nextest list`, so they are absent rather than `not_run`
  ([DTA-013](evidence-routes-and-execution.md#dta-013),
  [DTA-015](evidence-routes-and-execution.md#dta-015)).
- The pse-operations variant under test is not the production library.

**Counterevidence.**
- `native-solvers` is a real linking capability, not only a selection switch.
- Runtime selection already exists: `default-filter` for `managed_primary_` and reasoned
  `#[ignore]`.
- The library hooks serve a cross-crate cancellation-after-acknowledgement claim that a
  `cfg(test)` hook in one crate could not reach.

**Uncertainty.** Compile cost per variant is unmeasured.

## <a id="dta-030"></a>DTA-030 — Broad library-unit feedback is bounded by a few tests that specialize the full reference closure

**Principles:** AP-07, AP-06, AP-05; companion §8.

**Diagnosis (*Measured*).** Two recorded nextest runs, both nextest 0.9.146 with the default
profile and dev build profile on 2026-10-22:

| Run | Packages (`--lib`) | Tests | Wall time | Longest test |
|---|---|---|---|---|
| `60f84901` | pse-compiler, pse-modeling, pse-model, pse-quantity, pse-study-module | 738 | 54.8 s | 53.5 s |
| `b5f4cd35` | pse-math, pse-compiler, pse-runtime, pse-backend-native (`pse-runtime/force-validate`) | 1,960 | 66.6 s | 65.1 s |

- The longest test in both runs was
  `process_composition_tests::indexed_process_composition_keeps_memoryless_units_algebraic`
  (`crates/pse-compiler/src/process_composition_tests.rs:36-86`). It specializes a three-unit
  process over the full reference closure under three routes and asserts structural counts.
- The next were `workspace::modeling::conditional::tests::conditional_unit_recycle_reference_specializes_actual_unit_…`
  (46.8 s and 56.0 s) and
  `process_composition_tests::indexed_balance_operations_keep_all_applicable_material_bases`
  (21.4 s and 28.6 s).
- Summed test time was 252.6 s and 454.9 s.
- So with 16-way parallelism, wall time is set almost entirely by the slowest reference-closure
  specialization, not by suite size.

**Consequence.** Feedback for any broad library change waits about a minute for a handful of
full-closure specializations. That holds even when the change does not touch specialization.
These tests sit outside the `scientific` group and trip the 60 s slow-timeout.

**Counterevidence.** The tests exercise genuine reference science, and specialization latency
may be the real production cost. The source of the closure is the hand assembly in
[DTA-027](doubles-and-coupling.md#dta-027).

**Uncertainty.** The avoidable share is not established. Specialization latency is a production
concern. The broad account's execution-fit findings, for example
[DA-029](../audit/execution-fit.md#da-029) and [DA-030](../audit/execution-fit.md#da-030), may
own that cause; they were linked by title and not reexamined.

**Probe.** Profile the longest test to separate closure setup from specialization, and measure
reuse across its three routes.
