> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). The briefs given to every inquiry and challenge. `$REVIEW` denotes the working directory in which the inquiries wrote; the files now live in this folder. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# Briefs: consolidated efficiency and solver target review

This file records what each supporting inquiry was asked and what it was not asked. The coordinator (principal author) owns the combined judgment, the finding identifiers and the decision.

- **Scratch working directory** (where workers write): `/tmp/claude-1000/-home-paul-pse-arrow/ec287db0-3345-4c95-b8a8-8582acffa694/scratchpad/review/`. Called `$REVIEW` below.
- **Repository:** `/home/paul/pse-arrow`.

## Common brief (applies to every inquiry)

### Purpose

This is a **DESIGN-tier, TARGET-purpose** design review of the process simulator. It has two coupled outputs:

1. **Efficiency.** The complete set of opportunities to align the codebase with the efficiency principles: AP-07/G9 and *Heuristics for Efficient Architecture* H1–H28. The opportunities must not undermine the other principles.
2. **Solver target.** An updated target for solver method selection, transitions, observation, measurement, diagnosis and library-owned methods. It reflects the codebase *after* the efficiency target and the SurrealDB functionality that now exists.

You are producing a **bounded supporting contribution**. You do not issue a verdict, finding IDs or a decision.

**The maintainer's direction about method, which binds every inquiry:**
- Be holistic and expansive.
- Do **not** assess mechanism-for-mechanism swaps, and do not probe candidate changes.
- Put the detailed effort into what the libraries the project *already uses* can do, and into how the target could lean on them.
- Do not lock onto early candidates.
- Recent reviews got lost in mechanism detail and missed larger opportunities. Avoid that.

### Baseline

`main` at `6c368c88acda548d421b44e8fc4f457c95bcec7e`, clean tree, 2026-10-12. Plan 37 (SurrealDB lifecycle) is done. Plan 36 (efficiency alignment) is stopped mid-acceptance. Plans 28 and 32 are open.

### Standard (it owns the criteria; read what your question needs)

- `docs/design_review/design_principles/standard.toml`, which selects:
  - Core 3.4: `core/design-principles.md`, §1 and AP-01–AP-07;
  - `core/efficient-architecture-heuristics.md`, H1–H28 and the *Interpretation safeguards*;
  - Process Simulator 1.5: `profiles/process-simulator/principles.md`, *Functional target*, PS-01–PS-13;
  - the pse-arrow binding: `binding/pse-arrow.md`, scenarios PSE-S01–S07.

### Functional target

A best-in-class process simulator. An engineer authors physically typed, reusable unit and property models once; composes flowsheets and studies; and solves them (square simulation, optimization, dynamics, parameter estimation). Results must be physically meaningful, diagnosable when they fail, and fast enough for:

- **interactive edit → re-solve**, where a value change must not rebuild structure;
- **studies**: sweeps, sensitivities, optimization, fitting;
- **recycles and tears**;
- **dynamics** from the same definitions;
- **extension**: a new unit, property, reaction model or solver;
- compiling and testing one responsibility at a time.

Blueprint §0 and §0.2 (`docs/authoritative_design/sections/architecture-overview.md`) name the IDAES-PSE capability families.

### Workload premises (qualitative; there is no SLA and no cost model)

- Interactive value-only and structural edit → re-solve.
- A thousand-point value sweep.
- Sixteen independent end-to-end cases on sixteen physical cores, with internal library parallelism. The host has 32 logical CPUs and about 188 GiB, shared with other workloads.
- Larger sparse structures, with many small BTF blocks and some large SCCs.
- Long trajectories with sensitivities.
- Fitting with many observations and parameters.
- Growing retained history.
- Interruption before publication.

### Maintainer requirements for the solver part

- Three-layer method selection:
  1. a priori rules from mathematical facts;
  2. in-solve, observation-driven switching;
  3. an advisory history layer that only reorders or tunes what layers 1 and 2 admit.
- Diagnostics and measurement are product capability.
- Method adoption is never gated on benchmarks; it rests on mathematical rationale.
- New libraries must be free to obtain. Licence type is never an exclusion.
- **No HSL.**
- Parallel execution is **in scope and to be designed**: case-level, structure-level and library-internal.
- The development loop is in scope only where its cost comes from product design.

### Current design is a rule, not a criterion

Read the following to understand the subject. Do not fit your output to them; the review may overturn any of them.

- ADRs, notably:
  - ADR-0164, the unified SurrealDB substrate and durable ordinary execution;
  - ADR-0181/0182, from Plan 37;
  - ADR-0173–0180, from Plan 36.
- Blueprint D1–D14.
- Architecture sections under `docs/authoritative_design/sections/`:
  - §14 preparation;
  - §15–§18 numerical execution;
  - §19 and §21 workflows and Python;
  - §20 persistence.
- The operational guides `docs/dev/operational-store.md` and `docs/dev/surreal-substrate.md`, including the latter's "Capability choices and revisit triggers" I1–I7.
- Plans 28, 32, 36 and 37 under `docs/plans/`.
- Pins in `Cargo.toml` and `pyproject.toml`.

### Prior judgments are leads, not conclusions

**Do not take a position on each prior finding.** The coordinator maps leads after synthesis. The leads:

- `docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md`, its audit `..._audit_2026-10-11.md`, and their evidence under `docs/design_review/evidence/efficiency-alignment-*-2026-10-11/`;
- `docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md` and `docs/design_review/evidence/solver-methods-and-diagnostics-2026-10-12/`;
- the retired Plan 37 review and its evidence, read via `git show e145eb7d2:docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md` and `git show e145eb7d2:docs/design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/<file>`;
- the Plan 36 investigation evidence under `docs/plans/36-investigation-evidence/`.

Prior "what to keep" lists are leads too.

### Guardrails

- Report **facts, capabilities, obligations and allocations**. Keep observed facts separate from interpretation, and label interpretation.
- **No probes, builds, tests, services or measurements**, unless your brief explicitly allows one. Static source reading and cheap static tools are allowed: ripgrep, ast-grep, `git log`/`git show`, `cargo metadata`, and the library skills' own indexes and *existing* probe results.
- Never use `docs/library-utilization.jsonl`, `docs/capability-maps/` or the `library-catalog` MCP. They are stale. Use the library skills, Context7 (load it with `ToolSearch` `select:mcp__context7__resolve-library-id,mcp__context7__query-docs`), pinned upstream sources (`~/.cargo/registry/src/*/`, `vendor/`, and native source caches named in prior evidence) and official documentation. Check that a claim applies to the pinned version.
- Execution-fit reasoning is **qualitative**: no cost models and no numeric estimates presented as findings. Benefits are *Proposed*. Evidence labels follow principles §D: Interface-checked for an inspected interface or source path, *Implemented* for a code path that exists, *Tested* or *Measured* only for executed checks you name.
- An empty search does not establish absence. State the search coverage for any consequential absence claim.

### Permitted effects

- Read anything in the repository.
- Write **only** your assigned output file(s) under `$REVIEW`. Name each file for its content. Do not begin a markdown filename with report, summary, findings or analysis; the harness refuses those.
- Do not edit repository files. Do not commit, and do not run `just` recipes that change state.

### Return

Write the full contribution to your output file. Then return a **compact summary of at most about 100 lines** to the coordinator. The summary gives the key facts and allocations with precise source or version pointers, the material uncertainties, anything unfinished, and every file written.

---

## W-A: interactive work, obligations ledger

`code-mapper`, Opus. Output: `$REVIEW/w-a-interactive-obligations.md`.

Map the **interactive journeys starting from the actual Python call**: `python/pse`, then `crates/pse-py`, then the Rust runtime and downstream. The journeys:

1. The first solve of a flowsheet from authored definitions, end to end, including publication of results.
2. A value-only edit (a parameter, fixed value or start), then re-solve.
3. A structural or expression edit, then re-solve.
4. Reading results back in Python, including inspection queries.

**For each journey**, give a stage-granularity table. Aim for about 6–15 stages per journey. Use no per-function inventory. Columns:

- stage;
- owner (`path::symbol`);
- the scientific work done;
- the **obligations** the stage discharges: durability, coordination or exclusion, reuse or invalidation, assurance or validation, observation or telemetry, capacity or admission, cancellation or drain. Name the mechanism realizing each;
- crossings: Python↔Rust, process, WebSocket RPC to SurrealDB, thread, native FFI. Include transaction lifetimes;
- what repeats per edit or solve, and what that work **grows with** (edit size, package size, history, number of declarations, and so on);
- whether a library does the work (name it) or bespoke code does.

Use the `rust-code-model` and `ast-grep-ripgrep` skills where they help.

**Then answer:**

- **(a) Session state.** What state lives in the Python session and the Rust `Runtime` between edits? Which steps of the edit → re-solve loop **require** the canonical SurrealDB store, and which contract requires it? Cite the section or ADR. Could a live in-process model serve the loop? State only the facts that bear on this question; do not recommend.
- **(b) Engine use.** Where is `pse-engine` (DataFusion) used in these journeys, and for what? Name the operations and what they validate or compute.
- **(c) Repeated or reconstructed work.** Which immutable or validated state is rebuilt or rechecked during these journeys, and at which crossings?

Keep the file to about 250–450 lines.

## W-B: multi-case and long-running work, obligations ledger

`code-mapper`, Opus. Output: `$REVIEW/w-b-multicase-obligations.md`.

Use the same stage-table columns as W-A for these journeys:

1. A thousand-point value sweep, both ephemeral (in-process) and durable (managed workers).
2. Sixteen independent cases run concurrently.
3. Parameter fitting, steady and transient.
4. A dynamics horizon: a long trajectory with sensitivities, and rolling horizon if supported.
5. A recycle or conditional unit with staged initialization.
6. Continuation, homotopy or path following, if supported.

Start from the public entry points: Python and the Rust `Runtime`.

**Then answer:**

- **(a) Product concurrency.** Enumerate every concurrent **writer and reader of shared state in ordinary product use**: managed native workers, supervisors, Python clients, background tasks. Keep this separate from what the test harness and agent tooling create. Record:
  - what each writer or reader shares, and through which mechanism;
  - the crash and interruption domains;
  - which recovery obligations a **product feature** actually consumes (durable studies surviving process loss, multiple clients, and so on), with the contract that requires each.
- **(b) Parallelism and capacity.** Where is parallelism admitted at case level, structure level (blocks, subdomains, segments) and library level (faer `Par`, rayon pools, native threads)? How is capacity charged: pools, permits, leases, reservations? Where do nested pools meet?
- **(c) Cross-point reuse.** What persists across points or iterations: preparation, evaluators, native sessions, starts, factorizations? What is rebuilt per point, and why?

Keep the file to about 300–500 lines.

## W-C: machinery provenance and qualification archaeology

`code-mapper`. Output: `$REVIEW/w-c-provenance-archaeology.md`.

Use history and source facts only, with no judgment.

1. **Provenance timeline.** Cover each lifecycle mechanism family:
   - admission (at least seven mechanisms; see `crates/pse-admission`, `pse-engine/src/session/admission.rs`, `pse-runtime/src/math/strategy/admission.rs`, `pse-backend-native/src/execution*`, `pse-operations/src/canonical_lifetime.rs`, the vendor SDK patch `vendor/surrealdb/PATCH.md`, and `scripts/host_admission.py`);
   - fences;
   - leases;
   - receipts;
   - retention and reclaim;
   - settlement;
   - drain and teardown;
   - cancellation;
   - history release;
   - the 57 `canonical_*` generated runtime relations in `crates/pse-relations/src/generated/runtime/` (declared in `crates/pse-schema`).

   For each, record which plan, ADR or commit introduced it and when, and which later plans modified it. Use `git log --follow`, `git log -S`, ADR front matter and plan text. Give a compact table, plus a short narrative of how the machinery grew from 2026-09-25 to 2026-10-12.
2. **Classification of failures and fixes.** Classify every failed or interrupted acceptance attempt, and every `fix`-type commit, since 2026-09-28 by the responsibility it concerned:
   - scientific or numerical;
   - lifecycle, coordination or retention;
   - transport or connection;
   - test fixture or harness;
   - build, codegen or toolchain;
   - documentation.

   Sources: `git log` messages, and the checkpoint and outcome texts of Plans 28/28e, 30, 33, 34, 35, 36 and 37. Give counts and representative examples with commit hashes.
3. **Churn.** Lines added and removed, by crate and top-level module, since 2026-09-28 (`git log --numstat`). Group by the same categories where the module's role is clear.

Keep the file to about 200–350 lines. Report facts; do not judge.

---

## Library research R1–R5 (common to these five workers)

`library-research`, Opus/high. Each worker is framed by a **responsibility** and uses the library skills as tools.

### Part A, now

1. **Survey the full capability surface first.** Survey the libraries' full relevant capability surface **at the pinned version**, neutrally, **before** looking at current code.
   - Organize by capability family.
   - For each family give:
     - what it does;
     - its contract and guarantees: semantics, isolation, ordering, durability, failure and cancellation behaviour, thread-safety;
     - its limits;
     - whether a newer release changes anything material, naming the version;
     - which responsibility in a process simulator of this kind it could serve.
2. **Current use, briefly.** Give pointers to where the code uses each family today, without judgment. Grep is fine.
3. **Library-shaped allocations.** Sketch one to three complete ways the responsibility could be realized if these libraries owned it **wholesale or largely**. For each sketch give:
   - the obligations the library discharges;
   - the obligations that remain the project's;
   - the obligations that would be lost or weakened;
   - integration, lifecycle, testing and upgrade cost;
   - what existing bespoke machinery it would make unnecessary, at the level of responsibility, not file by file.

### Part B, later

A follow-up message will send you the obligations ledger from W-A and W-B. You will then map those obligations onto your libraries.

### Reuse of prior evidence

Reuse prior evidence through its paths or immutable references, and record what changed. Do not redo it.

### Size and return

Keep the Part A file to about 300–500 lines. Return at most about 100 lines.

## R1: durable state, coordination and lifecycle invariants

Output: `$REVIEW/r1-durable-state-and-coordination.md`.

**Libraries:**
- **SurrealDB 3.3.0**, covered **in full**:
  - transactions and isolation semantics, including what `FOR UPDATE`, unique indexes and conflict detection guarantee;
  - record references with `ON DELETE` actions;
  - `DEFINE EVENT`;
  - computed, `VALUE` and `DEFAULT` fields;
  - `DEFINE TABLE ... AS SELECT` views;
  - changefeeds and `LIVE SELECT`;
  - sequences;
  - files and buckets;
  - graph relations and traversal;
  - `DEFINE FUNCTION`;
  - permissions and `ACCESS`;
  - embedded engines (kv-mem, RocksDB, SurrealKV) versus remote WebSocket;
  - import and export;
  - EXPLAIN and the query planner;
  - storing binary payloads at scale;
  - `VERSION` time travel.
- The Rust SDK: which features the pin enables (`protocol-ws` and `rustls` only), and what the embedded engine features would add.
- The vendored request-lifecycle patch (`vendor/surrealdb/PATCH.md`, `vendor/patches.md`).
- SurrealKit 1.0.0-beta.6.
- **tokio 1.53 and tokio-util 0.7.19** structured-concurrency primitives: `TaskTracker`, `CancellationToken`, `JoinSet`, `Semaphore`, and so on.
- Ordinary filesystem primitives for local durability: atomic rename, fsync, advisory locks.

**Skills and tools:** `neo4j-surrealdb`, with its SurrealDB 3.3 indexes and executed probes, and Context7 for tokio.

**Prior evidence:**
- `docs/design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/`;
- `docs/design_review/evidence/surrealdb-unified-substrate-2026-10-05/`;
- the retired `library-capabilities-and-tooling.md` (git show `e145eb7d2:...`, see the common brief);
- `docs/dev/surreal-substrate.md` I1–I7;
- the efficiency evidence file `data-and-infrastructure-library-utilization.md`.

**Responsibility questions** (answer neutrally):
- What durable-state, coordination and lifecycle-invariant obligations could SurrealDB itself enforce? These include retention, reference integrity, cascades, uniqueness, conflict detection, readiness and notification, history, and recovery after crash.
- Under which deployment shape: embedded in-process, or a remote server?
- What can it **not** enforce, and why?
- What do tokio and tokio-util already provide for admission, cancellation, drain and teardown?
- **Library-shaped allocations** should include at least these two:
  - "SurrealDB-native lifecycle", where the store's own constraints, references, events and transactions own the invariants;
  - "minimal durable core": only what must be durable lives in the store, and everything else is process-local.

  In each case, state what remains the project's.

## R2: preparation, reuse and evaluation

Output: `$REVIEW/r2-preparation-reuse-evaluation.md`.

**Libraries:**
- **Salsa 0.28.4**: inputs, tracked and interned items, durability, backdating and early cutoff, LRU, cancellation, accumulators, `specify`, database lifetime.
- **Symbolica 3.0.1 / Numerica**:
  - evaluator construction and optimization (Horner, CSE);
  - compiled, exported (`export_cpp`) and JIT evaluators;
  - SIMD and vectorized evaluation;
  - dual numbers;
  - series;
  - polynomial and root facilities;
  - threading and licence settings.
- **faer 0.24.4**: sparse LU, Cholesky, LDLT and QR; symbolic/numeric split and reuse; supernodal methods; `Par`; dense decompositions.
- **SuiteSparse KLU, BTF, AMD and COLAMD** (`suitesparse_sys` 0.1.4, and KLU via `sundials-sys`): refactor and symbolic reuse.
- **pounce-presolve**: Dulmage–Mendelsohn, BTF, matching, FBBT.
- **FeOS 0.10.1, num-dual and quantity 0.14**: property evaluation, batching and derivatives. Note that FeOS is reference-only today.
- **quick_cache 0.6.24** and **rpds 1.2.1**: persistent structures and structural sharing.
- **petgraph 0.8.3** and **rustworkx-core 0.18.1**: condensation, SCC, matching, partitioning.
- **finitediff**.
- The `rust-reasoning` skill's libraries, **only** insofar as they bear on conditional and applicability admission algebra.

**Skills:** `salsa`, `symbolica-faer-oximo`, `rust-graphs`, `rust-reasoning`.

**Prior evidence:**
- the efficiency evidence `numerical-library-utilization.md` and `mathematical-compilation-and-evaluation.md`;
- `docs/plans/36-investigation-evidence/{numerics,preparation,sparse-rank}.md`;
- the solver evidence `structure-and-decomposition-map.md`.

**Responsibility questions:**
- What could carry **one coherent preparation and reuse lifetime**, from an authored edit through structure, evaluator, factorization and native session to study points?
- Could Salsa be the single reuse and invalidation authority?
- What do Symbolica's evaluator capabilities offer for the inner loop and for many identical bodies?
- What structure facts (for example BTF block shapes, affinity, coupling) can these libraries produce cheaply for downstream method choice, and at what lifetime?
- **Library-shaped allocations:** for example, "Salsa owns all preparation and reuse up to the native boundary", and "evaluator-centric preparation with compiled evaluators". State what each makes unnecessary and what each loses.

## R3: observation, results and analytics

Output: `$REVIEW/r3-observation-results-analytics.md`.

**Libraries:**
- **Arrow 59.3**: IPC stream versus file formats, zero-copy decode, C data and stream interfaces, compute kernels, `RowConverter`, extension types.
- **Parquet 59.3**: write and read, statistics, page index, predicate and projection pushdown, row-group pruning, bloom filters, async reader.
- **DataFusion 55.1**:
  - ListingTable and file formats;
  - custom TableProvider and pushdown;
  - aggregations and window functions;
  - streaming execution;
  - MemoryPool and spill;
  - SQL versus `LogicalPlanBuilder`;
  - statistics;
  - extension points;
  - `datafusion-tracing` and `instrumented-object-store`.
- **object_store 0.13.2**.
- **tracing 0.1.44 and tracing-subscriber 0.3.23**: spans, events, layers, structured fields, non-blocking writers. Assess these as a possible carrier for attempt and observation records.
- **statrs**.

**Skills:** `datafusion`, `datafusion-tracing`.

**Prior evidence:**
- the efficiency `relational-and-columnar-layer.md` and `publication-persistence-and-results.md`;
- the solver `diagnostics-and-measurement.md` and `observation-and-persistence-map.md`.

**Responsibility questions:**
- What is the best home for scientific results, attempt and observation records (solver measurement, history layer, diagnostics, telemetry) and analytics (study summaries, convergence-effort regression)? Compare:
  - Arrow IPC bytes inside the store (today);
  - Parquet files with DataFusion;
  - tracing-based capture;
  - store-native records.
- Should a query engine own **admission and validation** of authored or scientific relations at all, versus typed Rust validation plus Salsa? Cover what DataFusion's planner and optimizer actually contribute in `pse-engine` (`crates/pse-engine/src/session/`, about 26k lines; read its module docs).
- Can one observation record serve both efficiency telemetry and solver measurement, history and diagnosis?
- **Library-shaped allocations** should include "Parquet+DataFusion as the results and observation plane" and "tracing as the observation carrier". State what each makes unnecessary and what each loses.

## R4: numerical methods and native sessions

Output: `$REVIEW/r4-numerical-methods-native.md`.

**Libraries:**
- **Ipopt**: C API plus the C++ bridge's statistics, timing, warm start, intermediate callback and options. Linear solvers **without HSL**: MUMPS, SPRAL, Pardiso via MKL, and what the build provides.
- **Uno**: presets including interior point, callbacks.
- **PETSc 3.24 SNES/TS/TAO**: QN, NGMRES, Anderson, NLEQ-ERR line search, VI, NEWTONAL, NASM, composite SNES, field-split, `-snes_mf`, TAO BRGN, BNTR and TRON, convergence history and monitors.
- **SUNDIALS via `sundials-sys` 0.6.2**:
  - KINSOL, including difference-quotient Jv, fixed-point damping and depth, and statistics;
  - IDAS getters and statistics;
  - CVODES and ARKODE.
- **diffsol 0.16.2**: schemes, DiffSL, statistics.
- **The POUNCE family** at the pinned rev:
  - presolve reuse;
  - batched convex;
  - sensitivities;
  - QP homotopy;
  - second-opinion profiles;
  - FERAL and linsol;
  - statistics.
- **HiGHS 1.15** (`highs-sys`): basis retention, parallel and threads, IPM variants, MIP, statistics, model write.
- **SCIP 10.0.2**: statistics, concurrent solving, reoptimization, `writeOrigProblem`.
- **Clarabel 0.11.1**.
- **argmin 0.11**: Brent root and line searches.
- **egobox**.

**Skills and sources:**
- `native-solver-libraries` for the libraries it covers.
- It does **not** cover PETSc, Uno or ARKODE. Use the pinned upstream sources the prior evidence located, for example the PETSc 3.24.0 source cache at `~/.cache/pse-arrow-petsc-source/petsc-3.24.0`, and check prior B2/B3 for the others.
- `pyomo-and-solvers` for IDAES 2.13 reference practice: initialization, scaling, convergence analysis, solver workflow. Read it for behaviour only; this is a clean-room project.

**Prior evidence, used as the baseline:** the solver evidence B1–B5:
- `nlp-solver-capabilities.md`;
- `nonlinear-and-linear-solver-capabilities.md`;
- `dynamics-solver-capabilities.md`;
- `lp-conic-global-solver-capabilities.md`;
- `candidate-libraries.md`;
- `reference-practice-and-selection-literature.md`;
- `method-landscape-and-selection.md`.

Record **deltas only** against B1–B5, plus this brief's framing.

**Responsibility questions.** Frame them by the six uses below:

1. **Selection and diagnosis signals:** statistics, callbacks, inertia, conditioning, iteration histories.
2. **Reuse across study points, recycles and re-solves:** warm start, basis, symbolic analysis, reinitialization, session retention.
3. **Per-block methods** for BTF/SCC sequences: closed form, bracketed, Newton variants, least squares.
4. **Decomposition and parallel execution:** NASM, field split, Schur, multiple shooting, concurrent solves, finite-difference column parallelism. Cover the library-internal threading controls too.
5. **Cooperative policy stop** that retains the iterate and statistics.
6. **The KKT linear-solver story without HSL.**

Also give **library-shaped allocations** for the method-choice and execution responsibility. For example, PETSc's composable SNES and TAO as one framework owning many methods and compositions, compared with the class-specific adapters behind one plan. State what each makes unnecessary and what each loses.

## R5: capacity, parallelism and the Python boundary

Output: `$REVIEW/r5-capacity-parallelism-boundary.md`.

**Libraries:**
- **rayon 1.12**: pools, scopes, `install`, `par_iter`, work stealing, nesting behaviour.
- **faer `Par`**.
- **tokio 1.53**: runtime flavours, `spawn_blocking`, blocking pool sizing, cooperative budgeting, cancellation.
- **tokio-util**.
- **Native thread controls**:
  - Ipopt and MUMPS (OpenMP and BLAS);
  - Pardiso and MKL threads;
  - HiGHS threads;
  - SCIP concurrent solving;
  - the POUNCE rayon pools;
  - FERAL's global thread pool (see register R-41 in `docs/adr/register.md`);
  - Clarabel;
  - Symbolica `max_threads`.
- **pyo3 0.29, pyo3-async-runtimes 0.29 and pyo3-arrow 0.19**: GIL detach and `allow_threads`, free-threaded Python, async integration, zero-copy Arrow, object lifetimes across the boundary, an in-process session model.

**Skills and tools:** Context7 for rayon, tokio and pyo3. `symbolica-faer-oximo` for faer `Par`. `native-solver-libraries` for the native thread controls.

**Prior evidence:**
- `docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md`;
- the efficiency `multi-case-workflows-and-capacity.md`;
- `docs/plans/36-investigation-evidence/coordination.md`;
- `docs/authoritative_design/sections/numerical-execution.md` §18.8.

**Responsibility questions:**
- Across case-level, structure-level and library-internal parallelism, is a **single capacity model** needed, or are bounded independent pools with explicit oversubscription rules adequate? Answer from the libraries' actual nesting and oversubscription behaviour.
- Which admission, cancellation and drain obligations do tokio, tokio-util and rayon already discharge, and which genuinely need project machinery? For the latter, cover native threads that cannot be cancelled, FFI and the GIL.
- **The Python boundary:** what would an in-process, long-lived Python session over a live Rust model need from pyo3? Cover lifetimes, GIL, cancellation and zero-copy.
- **Library-shaped allocations:** for example, "one rayon-centric compute pool with library thread controls derived from it", and "tokio for I/O only, with compute in rayon". State what each makes unnecessary and what each loses.

---

## C1: clean-sheet architect

`design-reviewer`, Opus/xhigh. This is focused advice, not a formal review. Output: `$REVIEW/c1-clean-sheet-allocations.md`.

**Purpose.** Without looking at the current code, design how a best-in-class process simulator with this functional target would allocate its responsibilities across the libraries available. This is the review's main guard against reasoning anchored on existing mechanisms.

### Inputs you may read

- The functional target, workload premises and maintainer solver requirements in the "Common brief" above.
- The standard. This covers `standard.toml` and the files it selects: the core principles (§1, AP-01–AP-07), the heuristics H1–H28, the Process Simulator principles (*Functional target*, PS-01–PS-13), and the binding scenarios PSE-S01–S07.
- Blueprint §0 and §0.2 only, in `docs/authoritative_design/sections/architecture-overview.md`. These cover what the simulator is for and the IDAES-PSE capability families. Do not read beyond §0.2.
- The library capability surveys. Read their capability-surface and library-shaped-allocation sections; skip their "current use" sections. The files:
  - `$REVIEW/r1-durable-state-and-coordination.md`
  - `$REVIEW/r2-preparation-reuse-evaluation.md`
  - `$REVIEW/r3-observation-results-analytics.md`
  - `$REVIEW/r4-numerical-methods-native.md`
  - `$REVIEW/r5-capacity-parallelism-boundary.md`
- Optionally, the profile skill `.codex/skills/design-review-process-simulator/SKILL.md`.

### Do not read

- Do not read the source code (`crates/`, `python/`, `scripts/`, `vendor/`).
- Do not read the plans, the ADRs, the other architecture sections, prior reviews or their evidence, or the operational guides.

The point is independence from the current design.

### Produce one or two complete responsibility allocations

**Assumptions first.** State the deployment and concurrency assumptions explicitly, for example a single user, a local workstation, several worker processes, crash and interruption. Mark which allocation choices would change if an assumption changed.

For each allocation, cover the following.

1. **State.** Which state must be durable, shared or queryable, by whom, and at what grain. Which state is process-local and for how long. Cover:
   - authored definitions and revisions;
   - prepared structure;
   - evaluators;
   - native sessions;
   - study points;
   - results;
   - attempt and observation records;
   - method-history records;
   - diagnostics.
2. **Owners.** Which library or project owner holds each responsibility:
   - authored model;
   - preparation and reuse with its invalidation;
   - evaluation;
   - method choice (all three layers);
   - native execution;
   - observation and measurement;
   - results and analytics;
   - persistence and retention;
   - capacity and parallelism;
   - cancellation and recovery;
   - the Python boundary.

   Name the specific library capabilities used. Name the bespoke composition the project must own, and say why no library owns it.
3. **The one reuse authority**, if there is one, and how invalidation propagates from an edit.
4. **The method-choice domain model.** Cover:
   - selection features and their lifetimes;
   - how admission and preference relate;
   - cause-typed transitions;
   - per-block and decomposed execution;
   - the advisory history layer and how its use is recorded;
   - the cooperative stop.
5. **The capacity and parallelism model** across case, structure and library levels.
6. **The failure and recovery model**, proportionate to the stated concurrency assumptions. State which crash or interruption scenarios it must survive, and how.
7. **Extension scenarios.** Show which owners change, and how much, when each of these is added: a unit or property model; a solver or method profile; a study workflow; a result representation; a library upgrade; scale, skew or interruption.
8. **Machinery not needed.** List what the allocation deliberately does **not** need, as general categories such as "no cross-layer lease protocol for X", together with the reason.
9. **Open premises.** List what must be checked against the real code or workloads before adopting the allocation.

Keep it qualitative, with no cost models. The file should run about 300–500 lines. Return at most about 100 lines.

---

## Part B follow-up for R1–R5 (sent after W-A and W-B landed)

Read the two obligations ledgers:
- `$REVIEW/w-a-interactive-obligations.md`
- `$REVIEW/w-b-multicase-obligations.md`

Read both in full, then map the obligations they record onto **your** responsibility and libraries. Write the result to a new file, `$REVIEW/<your-id>-part-b-mapping.md` (for example `r1-part-b-mapping.md`). Do not edit your Part A file.

### What to produce

1. **Obligations in scope.** For each obligation in the ledgers that falls in your responsibility, cover the following. Group obligations by kind; do not enumerate per stage unless needed.
   - **Library ownership.** Could a library you surveyed own it **wholesale**, own it **partly**, or not at all? Name the capability, and the version if relevant.
   - **Necessity.** Is the obligation necessary for the workload? Separate what the functional target needs from what a current rule imposes, citing the rule. Remember that the ledgers distinguish product concurrency from harness and tooling concurrency.
   - **Mapping consequences.** What the mapping would remove, at responsibility level, and what it would lose or weaken.
2. **Refined allocations.** Refine your library-shaped allocations from Part A against the ledgers. State which ledger facts support or undermine each allocation.
3. **Repeated or reconstructed work.** Name the cases in the ledgers that your libraries would remove by construction rather than by more machinery.
4. **Open premises.** Name the premises that would change your mapping, and the evidence that would settle each.

### Rules

All Part A rules still apply: static reading only, no probes, builds or measurements, and write only your new file.

**Additional rule:** never include the user's email address, or any personal identifier, in network requests such as a User-Agent header. Use a generic User-Agent if one is required.

### Size and return

Keep the file to about 150–300 lines. Return at most about 60 lines.

---

## X1: omission and preservation challenge

`design-reviewer`, Opus/xhigh. Output: `$REVIEW/x1-omission-preservation-challenge.md`.

This is an independent challenge of a **draft**. It is not a parallel full review. Read:
- the standard: `standard.toml`, the core principles and heuristics, Process Simulator 1.5, and the binding;
- the Common brief above;
- the draft principal review, `$REVIEW/principal-draft.md`;
- the draft solver companion, `$REVIEW/solver-companion-draft.md`.

Use the evidence files in `$REVIEW` as needed (W-A, W-B, W-C, R1–R5 with their Part B files, C1). You may read any repository source to check a claim. Form your own judgment; the drafts are proposals to attack, not conclusions to confirm.

Answer three questions.

1. **Omission.** What *large* opportunity, cause or simpler architecture does the draft miss? Look for:
   - workloads or responsibilities the draft does not cover;
   - library capabilities that would remove more work than the draft's remedies;
   - a simpler complete alternative;
   - a cause that is mislabelled.

   Prefer a few material omissions to many small ones. Do not list mechanism-level nits.
2. **Preservation.** Challenge each consequential remedy (F01–F07, F13, and the recommended state boundary) with a *revealing legitimate case*. Could it:
   - reject valid work;
   - erase a distinction;
   - weaken PS-10 independence, AP-04 authority, DP-19 owned workspaces, truthful partiality or PS-11 reproducibility;
   - break crash recovery for real concurrent writers;
   - make an ordinary supported workload infeasible (H11)?

   Name the case, the guarantee at risk, and what the remedy must add to keep it.
3. **Diagnosis accuracy.** Spot-check the three or four most consequential factual claims against source, for example:
   - the per-solve store work;
   - "no product recovery obligation consumed by the interactive or ephemeral path";
   - per-call native sessions;
   - the MUMPS and PETSc concurrency facts.

   Report any claim that is wrong or overstated.

Static reading only: no builds, tests, probes or measurements. Do not put any personal identifier in network requests. Write only your output file. Keep it to about 150–300 lines, and return at most about 80 lines.

## X2: numerical-methods challenge

`design-reviewer`, Opus/xhigh, with a numerical-methods brief. Output: `$REVIEW/x2-numerical-methods-challenge.md`.

Adversarially challenge the **mathematical and numerical content** of:
- the draft solver companion, `$REVIEW/solver-companion-draft.md`;
- the numerical-economics recommendations in the draft principal review, `$REVIEW/principal-draft.md`: F05–F09 and §6.2–§6.3.

Use R2, R4 and R5 in `$REVIEW` (including their Part B files) and the prior solver evidence: `docs/design_review/evidence/solver-methods-and-diagnostics-2026-10-12/`, especially `method-landscape-and-selection.md` and `numerical-methods-challenge.md`.

Examine at least these.

1. **Warm-start reuse across edits and study points on lane-retained sessions.**
   - Its effect on which local solution a nonconvex problem reaches.
   - PS-10 and PS-11.
   - When a warm start, a retained factorization or a KLU refactor is mathematically invalid: a pattern or pivot change, a bound or active-set change, a branch change in an implicit or cubic property.
2. **Per-block plans over BTF.**
   - The suitability rules: affine, Brent, certified root, `block_solve` ≤ 8, Newton/KLU, VI, trust region and NLEQ-ERR.
   - Escalation and final simultaneous polish.
   - Whether per-block solving can return a point the simultaneous system rejects, and how scaling interacts with it.
3. **Wavefront parallel block solving, parallel shooting windows and parallel fit experiments.**
   - Determinism of reductions.
   - Equivalence to the sequential results.
   - Interaction with per-block commit and rollback.
4. **The KKT linear-solver preference without HSL.**
   - SPRAL or Pardiso versus MUMPS under multi-lane.
   - Effects on inertia detection and the Ipopt curvature-test option.
   - Effects on the meaning of δw.
5. **Recycle defaults:** Anderson depth and damping, the contraction diagnosis, and the simultaneous successor.
6. **Transient fits.**
   - `IDAReInit` reuse per experiment, versus correctness of sensitivities and adjoints.
   - The forward-versus-adjoint rule.
7. **The history layer:** PAR-k scoring, censoring, frozen snapshots per study, and their effect on reproducibility.
8. **Cooperative stop fidelity:** whether a policy-stopped iterate is a safe start proposal for each library.

For each item, say whether the draft's statement is correct, needs a condition, or is wrong, and give the corrected statement. Static reading only: no builds, tests, probes or measurements. Do not put any personal identifier in network requests. Write only your output file. Keep it to about 150–300 lines, and return at most about 80 lines.
