# Single-solve native route: bounded supporting assessment

Inquiry **C** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`C-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

Baseline: `main` at b01a45d71a5f5ab29991ad193679f5588ac3662f. Standard: core 3.4, process simulator 1.5, pse-arrow binding. This is a read-only, focused supporting assessment. I ran no builds, tests or services. Local labels are C-nn. Diagnoses are source-traced (*Interface-checked*). Remedies are *Proposed*. Nothing is *Tested* or *Measured*.

## 1. Scope examined and coverage limits

**Read in depth:**
- `pse-backend-native`: `tnlp.rs`, `uno/callback.rs` (the `foreign_nlp` bridge used by Ipopt), `callback.rs`, `assembled.rs` (`AlgebraicOracle`), `execution.rs` (`Retained`, `scoped`), `execution/runner.rs` (`nlp`, `roots`), `presolve/pipeline.rs` (`Pipeline::new`, `finish`), `quality.rs::attach_nlp`, `kkt.rs` (`attach`, `analyse`), `conditioning.rs`, `structural.rs`, `tears.rs`, `recycle.rs`, `square_response.rs` with `actions.rs`, `ipopt/sequence.rs`, `kinsol.rs` callbacks, `kinsol/block.rs`, `dynamics/idas.rs` (callbacks, `sample`, linear solver), `dynamics/linear.rs`, `solve.rs` (`Execution`, `Progress`).
- `pse-runtime`: `math/solves.rs` (`execute`, `admit_step`, `run_step`, `callback_step`, `root_response`, `with_sensitivity`), `math/strategy.rs` (`transition`, `run_inner`), `math/solves/multistart.rs`, `math/flows.rs`, `math/initialization.rs` (`stages`, `attempt`, `evaluate_conditional_unit`), `math/staged.rs::serve`, `workflow/strategies/conditional.rs` (`UnitWorker`), `workflow/dynamics.rs::DynamicWorker`, `workflow/durable.rs` (`Tap`, `Streamer`), `workflow/modeling/assessment.rs`.
- `pse-structural`: `incidence.rs::analyze`, `flowsheet.rs` (`admit`, `witness`, `unweighted_heuristic`), the head of `initialization.rs`.
- `pse-math`: `assembly.rs` (`CaseWorker` evaluate, constraints and jacobian) and `binding.rs` (`CaseValues`, `values_selected`), only as far as needed to see what each callback does.

**Not examined or only skimmed:**
- `root_isolation.rs` (IBEX), `implicit.rs`, `derived.rs` (backend), `petsc*`, `scip.rs`, `highs.rs`, `uno.rs`, `conic*`, `execution/factorable.rs`, `pounce.rs` internals, `kkt/path/*` (arclength continuation), `kkt/sensitivity*`, `engineering_accuracy*`, `transport.rs`.
- `math/solves/{paths,derived,petsc,blocks,causal}.rs`, `strategy/{admission,target,path_control}.rs`, IDAS adjoint routes, and diffsol routes beyond `linear.rs`.
- I did not compare continuation step control for duplication.

Evaluator internals (Symbolica instancing and its cost) belong to inquiry B. Study-level lifetimes and pools belong to D, and durable persistence belongs to E. I followed those areas only to the seams named below.

## 2. Complete operations examined

**(a) One NLP or Root step on the session thread.**
`staged.rs::serve` keeps native scopes and one `Retained` per session, and therefore per run. Each step then runs as follows:
1. `MathService::execute` → `admit_step`, which handles seed validation, `Execution::within` and the receipt.
2. `run_step` → `callback_step`:
   - It builds a fresh `CaseWorker` through `case_worker` → `CaseAssembly::worker_scoped`. This instantiates one evaluator worker per program and clones the Jacobian and Hessian `AssemblyMatrix`.
   - It builds `AlgebraicOracle::new(worker, values.clone())` with the compiler structural witness, presolve facts and normalization.
3. `execution::nlp`:
   - `structural::check` validates the compiler witness linearly.
   - A new `presolve::Pipeline` wraps POUNCE `PresolveTnlp`.
   - `adapter.execute(retained, …)`. Ipopt keeps its TNLP when its signature matches (`ipopt/sequence.rs::Sequence::solve`). KINSOL reuses its setup through `KINSetNoInitSetup`. POUNCE keeps FERAL symbolic state through `pounce/retained.rs`.
4. `Pipeline::finish` evaluates `outer.eval_g` and `finalize_solution`, then `recover_fixed_bound_reactions`, then `quality::attach_nlp` (original constraints, objective and KKT stationarity on the unpresolved oracle). It ends with `kkt::attach`, a fresh FERAL LDLᵀ inertia factorization for every Optimize step (`Analysis::for_intent`).
5. `quality::qualify`, followed by the runtime `Assessment::assess`, which evaluates a separate observation program for closure and expectation checks.
6. Optionally `root_response` / `fresh_root_predictor`.

Live state per step: the attempt worker and oracle, the presolve wrapper stack, the native session in `Retained`, the KKT factor while it is consumed, and the report.

**(b) Per callback.**
The native caller passes x. The callback runs through one of two wrappers:
- `foreign_nlp` (Ipopt): a dimension check, a scratch `Vec`, the oracle call, a finiteness scan and a copy.
- `tnlp::Adapter` (POUNCE): `normalization.physical_point(x)` allocates, a scratch `Vec` is allocated, each entry is scaled with a checked product or ratio, then a finiteness scan and a copy.

Both wrappers run inside `CallbackState::evaluate`, which does `catch_unwind`, admit/observe work charging, two `stopped()` checks and two `BTreeMap<String,_>` entries keyed by `demand.into()`.

The oracle side then does the following:
1. `AlgebraicOracle::trial` inserts every x[i] into `CaseValues.scalars: BTreeMap<SemanticId,f64>`.
2. `CaseWorker::evaluate`, for each instance:
   - `values_selected` builds a `BTreeSet` of formals, looks each one up in the map and applies its conversion.
   - It collects a `Vec<f64>` and a `Vec<u64>` of bits, compares the bits with the memo, and evaluates.
3. `constraints` and `jacobian`: each contribution or term does a `rows[&SemanticId]` lookup (`BTreeMap`) and/or a `binary_search` over outputs. The result is a fresh `Vec` that is copied into the callback scratch.

**(c) Between attempts.**
`math/strategy.rs::run_inner`, `transition` and `Ledger` own the mechanism ladder. Multistart (`solves/multistart.rs`) prepares a seed on the original `PreparedSolve`. Each dispatch is a full step as in (a): new worker, new presolve pipeline, native attempt, post-solve analysis.

**(d) Staged initialization.**
`initialization.rs::stages`: for each stage, and for each BTF or merged block (`pse_structural::initialization::Plan`), `attempt` runs:
1. `rebind_within` or `bind_block` (a job);
2. `prepare_conditional`;
3. `within_task`;
4. `session.step`, a full step as in (a);
5. `commit_block`.

Several full `CaseValues` clones are made per block.

**(e) Recycle (sequential modular).**
`recycle.rs::CausalMap::sweep` is the KINSOL fixed-point map with Anderson acceleration. For each unit, `UnitWorker::evaluate` clones `program.values`, the whole case's values. A conditional unit additionally runs `MathService::evaluate_conditional_unit`, which per call:
- builds a new worker;
- builds `AlgebraicOracle::new(worker, values.clone())`;
- computes `values.identity()` (BLAKE3 over every scalar);
- creates `Retained::default()`;
- runs `execution::roots` with `warm: None`;
- runs `commit_block`.

**(f) Tear selection.**
`FlowGraph::admit` (petgraph, with a binary search per edge) feeds `tears::compile`, which builds one MILP over every node and connection with big-M = total node count. `tears::solve` runs it on HiGHS, and `FlowGraph::witness` checks the result with a petgraph toposort. The alternative is `unweighted_heuristic` (`greedy_feedback_arc_set`). This runs as an explicit job (`flows.rs::select_tears`) and is not on the re-solve path.

**(g) Structural analysis.**
`CaseIncidence::analyze` runs pounce-presolve `hopcroft_karp`, `DulmageMendelsohnPartition` and `BlockTriangularForm`, then rustworkx `lexicographical_topological_sort`, on a dedicated stack thread. The compiler computes it once (`pse-compiler/src/workspace.rs`) and the steps carry it as a witness.

**(h) Dynamics.**
In `idas.rs`, one IDAS memory owns KLU through SUNDIALS, `IDAReInit`/`IDASensReInit` and `IDACalcIC`. The callbacks behave as follows:
- `residual` copies y and y', calls `DynamicWorker::evaluate` (which writes t, state and parameters into a `BTreeMap` through `assign`, then calls `CaseWorker::constraints`), and stores a cloned evaluation in the bits memo.
- `jacobian` maps each nonzero with a `binary_search`.
- `sample` reads every forward sensitivity vector and stores n×np state sensitivities for every sample.

**(i) Progress.**
Every `Progress::push` event goes first to the tap, which is unbounded by `controls.history`. Those events include each Ipopt or POUNCE `intermediate_callback` and each failed trial (`CallbackState::evaluate`). The tap is `durable.rs::Tap::observe`, which sends to a bounded mpsc (capacity ≤ 512). `Streamer` turns each batch into JSON and calls `append_result_batch`. When the channel is full, the native thread blocks for up to `heartbeat`. On overflow the run is cancelled (`Streamer::finish`: "durable progress admission failed; scientific run cancelled").

## 3. Candidate findings

### C-01: Solver coordinates are re-bound through semantic-identity maps on every callback

**Cause.**
- `assembled.rs::AlgebraicOracle::trial` writes every free coordinate into `CaseValues.scalars: BTreeMap<SemanticId(16 B), f64>` on each objective, gradient, constraint, Jacobian and Hessian callback.
- `pse-math assembly.rs::CaseWorker::evaluate` then rebuilds each instance's inputs through `binding.rs::values_selected`. That function allocates a `BTreeSet` to re-check for duplicate formals, does one `BTreeMap` lookup per formal, applies `CanonicalConversionPlan::apply` per value, and collects into a `Vec`. A second `Vec<u64>` of bits is built for the memo comparison.
- `CaseWorker::constraints` resolves `self.assembly.rows[&id]` (a `BTreeMap<SemanticId, GlobalRow>`) per contribution. `jacobian` and `hessian` do a `binary_search` in `result()` per term. Every result is returned as a fresh `Vec` and copied into callback scratch.

The same pattern appears in dynamics, where `workflow/dynamics.rs::DynamicWorker::assign` inserts t, state and parameters into a `BTreeMap` for each residual or Jacobian evaluation. It also appears in `UnitWorker::evaluate`.

Every one of these facts is fixed once the bound case exists: the coordinate→slot binding, the duplicate-formal check, the unit conversions, the row indices and the output positions.

**Consequence.** The cost is O(n log V + Σ formals·log V) of map traffic plus several allocations on every evaluation, on every native route (Ipopt, POUNCE, KINSOL, IDAS, diffsol). It applies to larger sparse structures (with many instances and wide states) and to long trajectories. This is DP-10 ("keep wide provenance out of inner loops") and H12/H15: the semantic authority's representation is reshaped in the innermost loop. Whether the binding work rivals Symbolica evaluation is **not measured**. For models with many small instances it is plausibly material.

**Simpler conforming alternative.** When `CasePlan`/`CaseAssembly` are bound, derive a dense, operation-shaped binding once per bound case (H15):
- each instance input formal → a column index into x, or an index into a dense fixed-value vector, with its conversion resolved to scale and offset;
- contributions and terms → resolved row indices and output positions.

The oracle then passes x straight through. A value rebind refills only the fixed vector. `CaseValues` stays the semantic authority at the boundary (candidate publication, assessment, diagnostics hydrate by identity). Evaluation results can be written into caller buffers rather than returned as `Vec`. Library alternatives: none needed. This is a representation change inside the owner, and Symbolica evaluators already take dense slices.

**Preservation constraints.**
- Per-instance failure context (`MathError::Instance`).
- The per-instance same-input memo, which makes post-solve re-evaluation at the candidate cheap.
- Finite checks.
- Atomic publication (a failed trial never publishes a partial buffer).
- DP-19 attempt-owned scratch.
- Conversion semantics. If any conversion is not affine, keep it as a precompiled closure.

**Uncertainty.** Relative cost (probe P1). Ownership is a seam: B owns the `CaseWorker` internals, and C owns the oracle and callback bridge.

**Label.** Diagnosis *Interface-checked*, remedy *Proposed*.

### C-02: Conditional causal units rebuild and cold-start a complete native attempt on every fixed-point evaluation

**Cause.** For every KINSOL fixed-point map evaluation (`recycle.rs::CausalMap::sweep`), and for every unit:
- `conditional.rs::UnitWorker::evaluate` clones `program.values`, which is `resolved.model.values.clone()`, the complete case value map (see `UnitProgram` construction).
- For a conditional unit, `initialization.rs::MathService::evaluate_conditional_unit` then:
  - instantiates a new `CaseWorker` (`self.worker(...)` → `worker_scoped`: evaluator workers for all programs plus matrix clones);
  - clones the full value map again into `AlgebraicOracle::new`;
  - hashes the whole value map with BLAKE3 for `Compatibility.data` (`CaseValues::identity`);
  - creates `Retained::default()`, which means a new native session, KINSOL memory and KLU symbolic analysis;
  - solves with `warm: None` from the original start values;
  - runs the transport, quality and qualification steps;
  - drops everything.

**Consequence.** Recycle journey (PS-08 "recycle convergence"): the work is K fixed-point iterations × U conditional units × (worker instancing + O(V) clone and hash + native setup + a cold Newton solve). V is the whole flowsheet. Near convergence each unit's inner solve could take 0–1 Newton steps from its previous solution, but it restarts from the authored guess.

This is H12 (stable preparation repeated per value), H10 (U·V amplification per sweep) and PS-11 (reuse of factor and session). It also continues prior solver-acceleration F08/F17: nested solves run their own lifecycle and never warm-start.

**Simpler conforming alternative.**
- Make the conditional unit's solver state live as long as the `CausalMap` that already owns the units (`units: Vec<Box<dyn CausalUnit>>`). That state is one attempt-owned worker and oracle (`AlgebraicOracle::update_values` already exists), one `Retained` session (KINSOL setup reuse through `KINSetNoInitSetup` is already supported), and a unit-local value binding instead of whole-case clones.
- Compute compatibility over the unit's selected inputs only.
- Optionally seed from the unit's previous committed solution as a *declared* start (PS-08), recorded as a dependency.

**Preservation constraints.**
- The fixed-point map must stay a deterministic function of the tear values. Warm starts could select a different root branch, so session and factor reuse is unconditional, while warm starts apply only where uniqueness or regularity is established, or are screened as a proposal under the same branch policy.
- Typed refusal and teardown on every path.
- Charges released per evaluation.
- `commit_block` qualification.

**Uncertainty.** How common conditional units are in authored flowsheets. Iteration counts are unmeasured (probe P2).

**Label.** Diagnosis *Interface-checked*, remedy *Proposed*.

### C-03: Root parameter response is a dense O(n³) analysis and repeats the sparse predictor's factorization

**Cause.** `solves.rs::root_response` densifies the sparse Jacobian into `faer::Mat::zeros(n, n)` and `(n, np)`. `square_response::response` then:
- computes a full dense SVD (`spectrum`, `Par::Seq`) for numerical rank;
- solves with dense partial-pivot LU (`solve_regular`);
- checks the backward error densely.

`workspace_bytes` reserves 3·n² plus SVD scratch and refuses with `Withheld::Memory` beyond the allowance. When the predictor is retained (`retaining_factor`), `fresh_root_predictor` evaluates the same Jacobian at the same point again and factors it with sparse faer LU (`square_response/actions.rs::SparseFactor::prepare`). The same `F_x X_p = -F_p` relation therefore gets two factorizations, one sparse and one dense.

**Consequence.** For larger sparse structures, a supported sensitivity request on a Root solve is either serial O(n³) work or an honest refusal. H11 applies: refusal does not establish feasibility, and the blueprint (§15.5.1) presents the Root response as a supported capability. When both products are requested, H14/H20 apply as well: the second factorization adds no independent property beyond the SVD rank. This confirms the L1 flag and prior solver-acceleration F12 ("root sensitivity is dense and transient"). That finding is residual: a sparse predictor now exists alongside the dense path.

**Simpler conforming alternative.**
- Make one sparse factorization serve both the response and the predictor.
- For the rank certificate:
  - use SuiteSparseQR's rank-revealing sparse QR. It is available at the pin as `suitesparse_sys` 0.1.4 feature `spqr`, which is not enabled today; the workspace enables only `klu, amd, colamd, btf`.
  - Alternatively, keep the dense SVD as an explicit, size-bounded certification mode.
  - Structural nonsingularity already comes from the carried matching witness (`SquareScope::admit` → `structural::check`).
- Solve multiple right-hand sides with the sparse factor (faer supports this), and check the backward error sparsely.

**Preservation constraints.**
- The published rank, cutoff, backward-error and neighbourhood evidence (§15.5.1).
- Withholding independent of the base root.
- Physical coordinate back-mapping.

**Uncertainty.** SPQR's rank estimate is a heuristic tolerance on R, not singular values, so the evidence semantics would change (candidate rule impact RC-1).

**Label.** *Interface-checked* (the algorithmic order follows from source), remedy *Proposed*.

### C-04: Forward dynamic sensitivities copy and retain full state-sensitivity matrices at every sample regardless of demand

**Cause.** When forward sensitivity is on, `idas.rs::Session::sample` reads all np sensitivity vectors (n each) and stores `state_sensitivities` (n×np) on every sample. The dense product `chained` gives the output sensitivities. In runtime source (tests excluded), the only consumers of `state_sensitivities` are:
- shooting continuity rows, which read only the window's last sample (`shooting.rs` `Row::Continuity`, `end(k)`), and which then `clear()` the field on every stitched sample;
- report byte accounting (`dynamics.rs` retained bytes).

Fitting (`fitting/modeling.rs`, `FitDerivatives::Responses` → `Forward`) and trajectory publication (`modeling/trajectory.rs`) consume only `output_sensitivities`.

**Consequence.** Long trajectories and fitting with many parameters hold samples × n × np live floats, plus copies, that no consumer reads (H6, H17). The result-memory charge grows with them, and they can be refused by the budget.

**Simpler conforming alternative.** Pass sample demand into the integrator profile: output sensitivities at requested samples, and state sensitivities only at window ends or at explicitly requested samples. IDAS can interpolate sensitivities at any time (`IDAGetSens`/`IDAGetSensDky`), so nothing is lost.

**Preservation constraints.** Shooting continuity derivatives, adjoint replay inputs, and the conservation and quadrature outputs.

**Label.** *Interface-checked*, remedy *Proposed*. Seam with D (fitting, shooting).

### C-05: The weighted tear MILP covers the whole flowsheet with a global big-M and does not use SCC decomposition

**Cause.** `tears::compile` creates:
- one order variable per node, with bounds `[0, n−1]`;
- one binary per decision;
- one row `o_from − o_to − n·x_d ≤ −1` per connection.

It does this for *every* node and connection, including connections between strongly connected components, which can never lie on a cycle. `n` is the global node count. `FlowGraph::components()` (petgraph `kosaraju_scc`) exists but is not used for tear selection.

**Consequence.** Recycle journey on larger flowsheets with several recycle loops: the MTZ-style ordering relaxation with M = |V| is weak and the branch-and-bound tree is global. Independent recycle systems are solved jointly (H9, H5). A time limit then truncates to "no usable incumbent" (H11). This happens per structure, not per re-solve.

**Simpler conforming alternative.**
- Decompose by SCC. Only intra-SCC connections enter the MILP, and each SCC gets M = |SCC|. Decisions grouped across SCCs couple their components; merge such components.
- Solve each component separately, or as independent blocks in one model.
- Trivial SCCs (single nodes without self-loops) need no tear.
- Optionally use a cycle-constraint formulation with lazy generation.

All of these stay within HiGHS and petgraph.

**Preservation constraints.** Mandatory and forbidden policies, decision grouping, the independent `witness` DAG check, and recording of the native optimality or limit status.

**Uncertainty.** The magnitude depends on HiGHS presolve and on realistic loop counts. This is *Proposed* (probe P4 is optional).

### C-06: Staged initialization runs one complete attempt pipeline per BTF block

**Cause.** `initialization.rs::stages` iterates `prepared.blocks`, which are BTF blocks from the DM/BTF witness merged only by control cycles (`pse_structural::initialization::Plan`). For each block, `attempt` runs:
1. a rebind job;
2. `prepare_conditional` (possibly `prepare_order`/`assemble`);
3. `within_task`;
4. a strategy driver;
5. a session `step` (through the mpsc channel to the session thread);
6. `admit_step`;
7. a new `CaseWorker` and `AlgebraicOracle`;
8. a structural check and transport;
9. a native solve whose session usually cannot be reused because the next block has a different structure;
10. the original observation and qualification;
11. `commit_block`.

Several whole-`CaseValues` clones are made per block (`values.clone()` passed into the rebind, `prepare_conditional` and the oracle). A typical flowsheet BTF has many 1×1 or small blocks.

**Consequence.** For the "many small blocks" and initialization journeys, fixed per-block orchestration dominates. Work scales as blocks × (job and async crossings + worker instancing + O(V) clones + native setup). This is H2: the physical unit is chosen by semantic distinction, and each declared block gets its own lifecycle.

**Simpler conforming alternative.** Keep blocks as semantic records and per-block outcomes, but execute a sequence of blocks inside one admitted native operation on the session thread:
- one original-case worker;
- a dense coordinate binding (C-01);
- per-block library root solves (KINSOL/Newton), or direct evaluation for explicit 1×1 assignments;
- commits applied in place.

Larger or failing blocks escalate to the full strategy step. The `kinsol::block` BTF preconditioner already shows structure-shaped execution inside one native solve.

**Preservation constraints.**
- PS-08: each block's commit is qualified, temporary overlays are restored, and a failed block commits nothing.
- Declared strategy per block.
- Cancellation between blocks.

**Uncertainty.** Real block-count distributions, and whether `prepare_order` results are memoized across stages (I did not trace the memo).

**Label.** *Interface-checked*, remedy *Proposed*.

### C-07: Every native iteration event is streamed durably, which back-pressures the solver thread and cancels the run on overflow

**Cause.**
- `tnlp.rs::Adapter::intermediate_callback` (POUNCE) and the corresponding `uno/callback.rs::iteration` bridge (Ipopt/Uno) push one `Event` per solver iteration. The event is a `BTreeMap<String, Metric>` with about 11 owned keys.
- `CallbackState::evaluate` pushes an event for every failed trial evaluation.
- `solve.rs::Progress::push` calls the tap *before* applying its retention `limit`.
- `durable.rs::Tap::observe` sends into a bounded channel (`policy.batch` ≤ 512). If the channel is full on a non-Tokio thread, the native solve blocks for up to `heartbeat`. If the send still fails, the tap cancels the run, and `Streamer::finish` returns "scientific run cancelled".
- The writer serializes JSON and calls `append_result_batch`, which costs `ensure_writes` + 2 RPCs. Mutations share one per-problem write guard (see the operation map).

**Consequence.** The solver's inner loop is coupled to the durable store's latency and its per-problem serialization (H23, H26). Under the sixteen-case workload, or a long Ipopt run with restoration trials, store contention can stall native iterations. A diagnostic stream's overflow can abort a scientifically sound solve (H1: progress completeness is being enforced as if it were the result). The bounded in-memory history shows that partial progress is already acceptable locally.

**Simpler conforming alternative.**
- Coalesce or sample per-iteration telemetry at the tap. Keep the latest iteration state and dropped counts, and publish a summary at a bounded rate.
- Send failure, phase and terminal events through a guaranteed small channel.
- Treat overflow as recorded progress partiality, not cancellation.
- Use `&'static str` keys for per-iteration metrics.

**Preservation constraints.** The terminal outcome record, the ordering of the events that are kept, cancellation responsiveness, and the claim fence before writes.

**Rule impact.** RC-2.

**Label.** *Interface-checked*, remedy *Proposed*. Seam with E (persistence) and H (observation volume).

### C-08 (low): Factor reuse gaps in setups that are already amortized

**Cause.**
- `dynamics/linear.rs::Klu::set_linearisation` frees and calls `klu_l_factor` on every linearisation, using the retained symbolic analysis. It does not use `klu_l_refactor`, which keeps the numeric object and pivot order; SUNDIALS' own KLU module refactors and checks rcond.
- `kinsol/block.rs::setup` rebuilds `Vec<Vec<(row,value)>>`, a FERAL `SparseColMatrix`, and calls `SparseLu::factor_markowitz` for each block on every preconditioner setup. FERAL offers `SparseLu::factor(a, &SparseLuSymbolic, params)` for ordering reuse.
- `kkt.rs::analyse` → `conditioning::factor` creates a new `feral::Solver` per Optimize step, even though `factor_into` keeps the symbolic analysis for an unchanged pattern.

**Consequence.** These are modest constant-factor repetitions (H12) per Jacobian setup, not per callback. They confirm the L1 flags at low priority.

**Alternative.** Refactor with an rcond or pivot-growth guard and fall back to a full factorization. Retain the block orderings. Keep a session-scoped KKT solver keyed by pattern.

**Preservation constraints.** Typed singular and nonfinite refusal (the purpose of `linear.rs`), and certified inertia.

**Label.** *Interface-checked*, remedy *Proposed*.

## 4. Opportunities and uncertainties that are not findings

- **Callback-boundary allocations** (`tnlp.rs` `physical_point`, scratch `Vec`s, `CallbackState` `String` keys). These are small per-callback constants. The scratch buffers implement atomic publication, which is a correctness feature. They fold naturally into C-01's remedy (caller buffers, attempt-owned scratch). They are not material alone.
- **Presolve per attempt.** `Pipeline::new` runs POUNCE presolve (and the FBBT admission analysis when facts allow) on every step: multistart starts, strategy rungs and continuation correctors over an unchanged structure and fixed values. Much of it may be start-independent and reusable (H12). However, some passes (warm duals, LICQ) touch the start, and the cost share is unknown. This is an uncertainty (probe P3).
- **Native session lifetime is the run.** Across sweep points (D's area), each point opens a fresh session thread, so KLU, MUMPS and FERAL symbolic analysis and Ipopt TNLP construction repeat per point. Symbolic work is usually small next to numeric factorizations, so this is a seam with D and not a finding here.
- **Per-step `structural::check`.** This is linear witness validation using `BTreeMap`s of rows and columns. It is proportionate (H19 carried witness plus a premise check). Adequate.
- **KKT second-order analysis per Optimize step.** It establishes properties the solver does not claim: active-set inertia, LICQ and second-order sufficiency (PS-10/PS-12). Keep it. The only opportunity is symbolic reuse (C-08).
- **IDAS `jacobian` callback.** It maps each nonzero by `binary_search`. A precomputed scatter map would remove the log factor, but IDAS rarely re-evaluates the Jacobian. Minor.
- **Ipopt `Signature` includes bound bits**, so bound-changing sweeps rebuild the TNLP. Ipopt's C API cannot re-optimize in any case. Minor.
- **The IBEX process-global mutex** (`root_isolation.rs`) and the PETSc and HiGHS process locks are library constraints. Their scheduling belongs to D (parallel F03).

## 5. Strengths and preservation constraints

- **Library-owned structural analysis.** Matching, DM and BTF come from pounce-presolve, and topological order from rustworkx, all inside a checked semantic boundary (`incidence.rs::analyze`). The compiler witness is carried and only revalidated linearly (`structural.rs::check`), which is good H19 practice. Kept.
- **PS-09 conformance in the recycle and tear machinery.**
  - Tear convergence is a KINSOL fixed-point/Anderson map (`recycle.rs`: "no recycle iteration algorithm lives here").
  - Tear selection is HiGHS with an independent petgraph DAG witness.
  - IDAS owns integration, IC, roots, reinitialization and sensitivities.
  - KKT inertia comes from FERAL.
- **Native state retention within a session.**
  - `Retained::session` with reuse and refusal semantics.
  - Ipopt `Sequence` signature reuse.
  - KINSOL `KINSetNoInitSetup` reuse with stale-setup monitoring.
  - The POUNCE FERAL lease pool keeps symbolic state.
  - IDAS `SampleElimination` per sample.
- **Independent post-solve assurance (PS-10) without wasted re-evaluation.**
  - The original unpresolved oracle observes through `CountedNlp`.
  - A separate observation program checks closure and expectations.
  - The `CaseWorker` same-input memo makes repeated evaluation at the candidate cheap.
  - `defer_kkt` shares the KKT factor for output accuracy instead of refactoring.
- **A typed strategy owner between attempts** (`strategy.rs`: `transition`, `Ledger`, `run_inner`), separate from native adapters.
- **Callback containment.** `catch_unwind`, typed latching, atomic output publication and admission charging per callback. Any C-01 remedy must keep these.
- **Team-stack reservation before scope entry** (`staged.rs::serve`).
- **The KINSOL Jacobian callback** copies the canonical CSC pattern directly. Efficient.

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| Enh F04 (IDAS refactors one sample matrix) | Resolved | `idas.rs` jump path builds one `SampleElimination::new` per sample for all directions |
| Parallel F03 (native process coexistence/exclusion) | Residual; scheduling not examined (D) | `petsc.rs` `PROCESS`, `highs_lifecycle.rs` `LIFECYCLE`, `root_isolation.rs` `IBEX` still process-global |
| Parallel F04 (POUNCE team stack reservation) | Resolved | `staged.rs::serve` reserves `execution::scope_stack_bytes` as `math:native-scope-stacks` before `scoped` |
| Solver-acceleration F06 ("policy between attempts has no owner") | Resolved for ownership; continuation duplication not examined | `strategy.rs::run_inner`/`transition`/`Ledger`; multistart is "one seed, not a private retry loop" |
| Solver-acceleration F08 (recycle/conditional units run own loops) | Residual | Conditional units use the adapter seam (`execution::roots`) but build a private attempt per evaluation (C-02) |
| Solver-acceleration F12 (unused cross-solve reuse; dense root sensitivity) | Residual | KINSOL setup reuse and FERAL symbolic retention now present; Root response still dense, duplicated by the sparse predictor (C-03) |
| Solver-acceleration F15 (structure never shapes main solve) | Residual | `kinsol/block.rs` BTF block-factor preconditioner exists; block-sequential execution only via per-block attempts (C-06) |
| Solver-acceleration F16 (JVP assembles full Jacobian) | Partly resolved | `CaseWorker::jacobian_product` uses directional programs when `has_directional_actions`; falls back to assembly otherwise |
| Solver-acceleration F17 (nested implicit solves never warm-start) | Not examined for implicit (B); confirmed for conditional units | `evaluate_conditional_unit` `warm: None` |
| Graph F02/N12 (flow admission scans decisions per edge) | Resolved | `FlowGraph::admit` uses `binary_search_by_key` per edge; linear `find` only on the error path |
| Integrated-solve-pipeline 2026-10-03 | Not examined beyond the overlaps above | — |
| L1 flags (square_response dense; `Klu` `klu_l_factor`; `kinsol::block` Markowitz) | Confirmed (C-03 material; C-08 low) | As cited |

## 7. Candidate rule impacts

- **RC-1.** Rule: §15.5.1 (numerical-execution, "Root response … explicit rank/cutoff/backward-error/neighborhood evidence").
  - *Change:* allow rank evidence from a sparse rank-revealing factorization (SPQR tolerance on R) or from structural matching plus a sparse backward error, with dense SVD as an explicit, bounded certification mode.
  - *Depends:* C-03.
  - *If kept:* a single sparse factor still serves the predictor and the solve. The dense SVD remains, size-bounded and labelled as the rank certificate, and large n is withheld honestly.
- **RC-2.** Rule: durable progress completeness. Today overflow cancels the run (`durable.rs::Streamer::finish`). This is related to §19.2's "bounded progress ingestion" and the earlier ADR-0112/0114 verification name `progress_stream_complete_under_volume`. Where it is now authoritatively recorded should be confirmed.
  - *Change:* progress may be sampled or coalesced with recorded drop counts. Overflow marks the progress log partial instead of cancelling the scientific run.
  - *Depends:* C-07.
  - *If kept:* move per-iteration telemetry off the guaranteed stream (summaries only), so that the guaranteed stream's volume is bounded by phases and failures, not iterations.
- **RC-3 (conditional).** If a recorded rule forbids nested or inner solves from warm-starting to protect function semantics (prior F17), C-02's warm-start part needs an explicit uniqueness- or branch-screened exception. Its session and factor reuse part needs no rule change.

## 8. Proposed light probes

The coordinator arranges these under `scripts/pse-env --resource-class light --native`. P1 and P2 require test binaries. If they are not already built, treat them as heavier and defer them.

- **P1 (C-01 materiality).**
  - *Question:* what fraction of callback time is binding and map traffic rather than evaluation?
  - *Command:*
    ```
    scripts/pse-env --resource-class light --native -- perf record -F 997 -g -o <scratch>/c-p1.data -- cargo nextest run -p pse-runtime -E 'test(reference_flash_completed_blocks_discharge_reporting_allowance)'
    ```
    Use the coordinator's choice of the largest existing NLP or Root flowsheet test. Then run `perf report -i <scratch>/c-p1.data --no-children --sort symbol | head -80`.
  - *Discriminating outcome:* `BTreeMap` insert/search, `values_selected`, `AlgebraicOracle::trial`, `CaseWorker::constraints` bookkeeping and allocator symbols together at ≥10% of samples within callbacks makes C-01 material. ≤3% downgrades it to an opportunity.
- **P2 (C-02).**
  - *Command:* the same perf method on `automatic_workflow_causal_dispatch_reconstructs_nonport_original_state` (in `workflow/strategies/conditional/tests.rs`), or on a conditional-unit recycle test if one exists.
  - *Discriminating outcome:* visible time in `CaseAssembly::worker_scoped`, `CaseValues::identity`, `BTreeMap` clone and KINSOL creation (`KINCreate`/`SUNLinSol_KLU`) relative to the unit Newton iterations.
- **P3 (presolve per attempt).** In a multistart test, compare the time in `Pipeline::new` and POUNCE presolve symbols with the native solve symbols. If presolve is ≥20% per start, a start-independent presolve product is worth designing.
- **P4 (optional, C-05).** Static only: count SCCs and intra- versus inter-SCC connections for the largest authored flowsheet fixture. A large share of inter-SCC rows in the current MILP supports the decomposition remedy.
