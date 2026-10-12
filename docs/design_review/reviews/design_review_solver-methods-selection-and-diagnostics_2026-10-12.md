---
title: Solver methods, selection logic and diagnostics
date: 2026-10-12
tier: design
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: Process Simulator 1.5
baseline: ff5a4aa2f7da60dff28a72b5b3bd563d75b43ab1 plus the uncommitted Plan 36 working tree (diff fingerprint 45ceea51b0698679)
---

# Solver methods, selection logic and diagnostics

## 1. Synthesis: decision, scope and what to retain

**Decision: Revise.**

The solver pipeline runs a single attempt well:
- libraries own every iteration, factorization and globalization;
- admission is pure and typed;
- native termination maps to typed outcomes;
- an independent original-space assessment decides whether a candidate is used.

What the pipeline does not have is a *model of method choice*. The maintainer's target asks for three layers:
1. a priori rules from mathematical facts;
2. in-solve, observation-driven switching;
3. an advisory history layer that only reorders or tunes what the first two admit.

It also asks for diagnostics and measurement that let the solver pipeline grow without investigative effort. Neither has an architectural home today.

- **Selection.** It models *admissibility* thoroughly and *preference* not at all. A constant rank per adapter picks among admitted adapters. A fixed statement order composes candidates. After a failure the pipeline appends lower-ranked adapters of the same class, whatever the failure was.
- **Structure.** The compiler computes structure: Dulmage–Mendelsohn decomposition, BTF blocks, affine-row proofs, a Schur separator. That structure barely reaches selection. Every BTF block receives the same solver and settings.
- **Observations.** Library statistics that would drive switching, diagnosis and measurement are untyped, unread or discarded. The NLP bridges stop by throwing, which loses the iterate. No attempt carries wall time. Diagnosis runs only on request.
- **Recycles.** Recycle convergence defaults to plain successive substitution and records no iteration history.

Two judgments, kept separate:

- **Architectural fitness fails G9.**
  - AP-01, AP-03, AP-04 and AP-05 are violated for the representative changes: add a method or profile, add a selection feature, add a transition, add a per-block choice.
  - AP-06 is split. Admission and the strategy walk are locally testable; candidate composition and escalation are not.
  - AP-07 is unresolved, pending premises named in §7.
  - AP-02 is satisfied at the adapter seam. One instance of adapter knowledge in orchestration is noted in [F03](#f03) and reconciled in §13.
- **Behavioural adequacy fails narrowly.**
  - **PS-08 (MUST):** recycle iteration history is not recorded ([F04](#f04)).
  - **G7:** capability and contract text overstate or diverge from execution ([F12](#f12), [F10](#f10)).
  - **G2:** typed iteration work reads "unknown" where the library reports it ([F05](#f05)).
  - PS-G2, PS-G3, G1 and G3–G6 pass on the inspected paths.
  - G8 passes, because nothing is reimplemented. The many unused library capabilities are a library-fit gap (slot 8), not a G8 failure.

### What the design gets right, and must keep

These strengths constrain every remedy below. Both this review and the [independent assessment](../evidence/solver-methods-and-diagnostics-2026-10-12/independent-design-assessment.md) identified them.

1. **Pure, injected admission with distinct typed states.** `routing::admit`/`decide_assessed` keep `Refused`, `PendingEvidence`, `SupportedPendingArtifacts` and `Ready` distinct, never fall past pending evidence, and record per-adapter reasons in `runtime.route_decisions`. A selection model must *consume* this, not restate it.
2. **A pure strategy walker with an effect boundary** (`strategy::next_automatic` plus the injected execute and assess steps). Keep the pattern; move the *content* (order, transitions) into data.
3. **Candidate use independent of native status.** Finish requires `OriginalConclusion::satisfied`. No switching or history mechanism may bypass it.
4. **Library-declared rungs lowered to acting profiles.** The POUNCE second-opinion ladder is the right pattern for adapter-contributed recovery: the library owns the rungs, and the common driver admits and assesses each.
5. **Refusal versus failed trajectory, and terminal classes that stop rather than escalate:** cancellation, resource, contract, infrastructure, panic.
6. **"Absent is unknown, never zero"** in `WorkEvidence` and `Metric::Unavailable`.
7. **Explicit selection is never substituted.**
8. **Transactional block initialization** with independent original assessment, commit and rollback per block (§17.1).
9. **A substantial diagnostics catalogue:** 25 `FindingKind` rules with an `idaes-2.13` threshold profile, an MILP degeneracy hunter, conditioning certificates, KKT-point analysis, and ℓ1 and deletion-filter infeasibility explanations. It is near parity with IDAES and is mostly *unused at the moment of failure*.
10. **Library ownership of numerical machinery.** No Newton step, line search, factorization or barrier controller is reimplemented.

### The causes and the findings

| Cause | Findings |
|---|---|
| **A.** Method choice has no governing model of suitability | [F01](#f01) selection models admissibility, not suitability; [F02](#f02) blocks all receive one method |
| **B.** Recovery between attempts is not driven by diagnosed cause | [F03](#f03) transitions gated on termination category, class-bounded, default-only, global fallback after local failure; [F04](#f04) recycle convergence weakest-by-default, unmonitored, no history (PS-08) |
| **C.** Observations are neither decision-grade nor measurement-grade | [F05](#f05) typed observation too thin, G2; [F06](#f06) library statistics and health signals unread; [F07](#f07) no time, aggregation or history identity; [F08](#f08) diagnosis on request only, no replay; [F12](#f12) selection record and capability claims incomplete or untrue; [F13](#f13) bridges stop by throwing |
| **D.** Library-owned methods and structure left unreachable | [F09](#f09) pinned methods unbound; [F10](#f10) HiGHS discards its basis; [F11](#f11) decomposable structure reaches decomposition only by accident of rank |

**Recommended target (§6).**
- **A solve plan.** A pure owner of preference, transition and advisory adjustment that consumes the existing admission. It works over typed **selection features with declared lifetimes**, is applied to the whole problem and to each block, and records its reasons.
- **Typed observation families** per method family, plus a **cooperative policy stop** in every bridge.
- **A compact typed attempt record** joining features, plan, observation summary, work, time and outcome. It is the basis for measurement, study aggregation, convergence-effort regression and the advisory history layer. That layer must record the history snapshot it used.
- **An automatic, bounded failure-diagnosis tier** reusing the existing catalogue, plus replay snapshots.
- **Binding of library-owned methods already pinned:**
  - PETSc VI, QN, NLEQ-ERR, NGMRES and Anderson, plus TAO BRGN-LM, BNTR and TRON;
  - KINSOL's difference-quotient Jv;
  - argmin `BrentRoot`;
  - Ipopt statistics, timing and options;
  - HiGHS basis retention;
  - SCIP structured statistics;
  - IDAS diagnostics.

  Then free additions: HSL MA27/MC19 and SuiteSparse SPQR.

### Scope and decisions

| Field | Assessment |
|---|---|
| Subject and boundary | The solve pipeline: class derivation, admission, preference, candidate composition and transitions; per-block, recycle, initialization, dynamics, fitting and presolve sub-selection; native adapters and bridges. Also the observation, persistence, diagnostics and measurement architecture. Suppliers: compiler facts and structural analysis. Consumers: workflows, persistence, Python |
| Standard | Core principles and template 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator principles and review additions 1.5, pse-arrow binding |
| Tier / purpose | DESIGN / TARGET |
| Reviewer / date | The coordinator as principal author reviewer, 2026-10-12. An [independent assessment](../evidence/solver-methods-and-diagnostics-2026-10-12/independent-design-assessment.md) by a fresh design reviewer, who did not see the author's conclusions, is reconciled in §13. An adversarial [numerical-methods challenge](../evidence/solver-methods-and-diagnostics-2026-10-12/numerical-methods-challenge.md) corrected the author's mathematical analysis |
| Baseline | `main` at `ff5a4aa2f` plus the uncommitted Plan 36 working tree (165 files, +6461/−1880; diff fingerprint `45ceea51b0698679`). Plan 36c packets are in-progress baseline and are not re-reviewed |
| Architectural fitness | **Fails G9.** AP-01, AP-03, AP-04 and AP-05 violated; AP-06 split; AP-07 unresolved; AP-02 satisfied |
| Behavioural/semantic adequacy | **Revise (narrow).** PS-08 not met; G7 and G2 fail narrowly. PS-G2, PS-G3, G1, G3–G6 and G8 pass on inspected paths. PS-G1 not applicable |
| Overall decision | **Revise** |
| Disposition owner | **Not yet scheduled.** Proposed owner: a **new plan created through `plan-creation`**. It coordinates with, but does not reopen, Plan 36: 36c P-C4 block sequence, 36b P-B6 telemetry. Rule impacts take effect only after operator confirmation in plan creation |

**Supporting documents.** Each carries a bounded judgment; this review owns the combined decision.

- Code maps:
  - [Selection and escalation](../evidence/solver-methods-and-diagnostics-2026-10-12/selection-and-escalation-map.md) (A1). Includes the status of the 2026-10-03 findings F01–F17 and mechanisms M1–M18.
  - [Observation, persistence, identity and replay](../evidence/solver-methods-and-diagnostics-2026-10-12/observation-and-persistence-map.md) (A2).
  - [Structure, decomposition and evaluation capability](../evidence/solver-methods-and-diagnostics-2026-10-12/structure-and-decomposition-map.md) (A3).
- Library capabilities at their pins:
  - [NLP](../evidence/solver-methods-and-diagnostics-2026-10-12/nlp-solver-capabilities.md) (B1);
  - [nonlinear equations, least squares and factorization statistics](../evidence/solver-methods-and-diagnostics-2026-10-12/nonlinear-and-linear-solver-capabilities.md) (B2);
  - [dynamics](../evidence/solver-methods-and-diagnostics-2026-10-12/dynamics-solver-capabilities.md) (B3);
  - [LP, conic, global and auxiliary](../evidence/solver-methods-and-diagnostics-2026-10-12/lp-conic-global-solver-capabilities.md) (B4).
- Candidates and practice:
  - [free candidate libraries](../evidence/solver-methods-and-diagnostics-2026-10-12/candidate-libraries.md) (B5a);
  - [IDAES, simulator practice and the algorithm-selection literature](../evidence/solver-methods-and-diagnostics-2026-10-12/reference-practice-and-selection-literature.md) (B5b).
- Principal analyses:
  - [method landscape and selection](../evidence/solver-methods-and-diagnostics-2026-10-12/method-landscape-and-selection.md), the mathematics of what to choose and on what basis;
  - [observation, diagnostics and measurement](../evidence/solver-methods-and-diagnostics-2026-10-12/diagnostics-and-measurement.md), including the captured-versus-available matrix.
- Independent judgments:
  - [independent design assessment](../evidence/solver-methods-and-diagnostics-2026-10-12/independent-design-assessment.md) (IR-01–IR-10);
  - [numerical-methods challenge](../evidence/solver-methods-and-diagnostics-2026-10-12/numerical-methods-challenge.md).

This review's finding identifiers are F01–F13 and its rule impacts RC01–RC13. IR-nn identifiers are local to the independent assessment.

## 2. Scope, workloads and coverage (slot 1)

**Functional target.** The profile's best-in-class simulator (blueprint §0/§0.2): the edit → re-solve loop, studies, recycles and tears, dynamics from the same definitions, and extension by adding a solver. Results must be diagnosable when they fail and fast enough for interactive loops, sweeps and recycles.

**Maintainer requirements (2026-10-12).** This review assesses how to realize them; it does not overturn them.
- Three-layer selection, with history advisory only.
- Diagnostics and measurement as product capability.
- Method adoption never gated on benchmarks; method benefits rest on mathematical rationale.
- New libraries must be free to obtain; licence type is not an exclusion.

**Workloads considered.**
- **Square flowsheet simulation:** equation-oriented, with BTF block sequences.
- **Recycle (tear) convergence**, sequential-modular style.
- **Initialization.**
- **NLP optimization:** design, NMPC.
- **Estimation:** steady and transient, multi-experiment.
- **Dynamics:** index-1 DAEs with events; shooting and collocation.
- **LP, MILP and QP sequences:** tear MILP, reconciliation, MPC.
- **Global and MINLP.**

The relevant scale: sparse systems with 10² to 10⁶ unknowns, many small BTF blocks plus large SCCs, studies of many cases, and long event-dense horizons.

**Non-goals.**
- Parallelism that comes from code architecture: pools, concurrency, async. Library-internal threading appears only as configuration. Decomposition seams are named as future parallel opportunities, not designed.
- Property-model internals, beyond callback cost attribution.
- Re-review of Plan 36c.
- Library-skill maintenance; skill coverage gaps go to evidence limits.
- Paid solvers, except as reference practice.

**Inspected.**
- Source read by the principal and the independent reviewer:
  - routing, `ProblemFacts`, `solves.rs` (automatic operations, alternatives, `complete_context`, separator attachment), `strategy.rs`, `staged/driver.rs`, `solves/blocks.rs`;
  - `requests.rs`, `recycle.rs`, `kinsol.rs`, `dynamics.rs`, `fitting/modeling.rs`;
  - `solve.rs` (`SolveReport`, `Evidence`), `callback.rs`;
  - the Ipopt and Uno bridges, `highs.rs`, `execution/scip.rs`, `pounce/profiles.rs`;
  - the `FindingKind` catalogue, `results.rs`.
- Library sources and documentation read by workers: B1–B5b, with local binding claims spot-checked.

**Not examined, and why it does not affect the verdict.** Clarabel, Uno and PETSc adapter internals beyond capability strings and stop paths; `prediction.rs` and `paths.rs` internals; Python summaries. None changes the G9 verdict.

No probes were executed. Every claim about current code is *Interface-checked*; remedies and method benefits are *Proposed*.

## 3. How a problem gets its method today (slots 2, 3, 5)

### 3.1 Responsibilities

The independent assessment's §4 has the full decomposition table. In outline:

**Class and admission.**
- `routing::problem_classes` derives the classes. For a root intent on a square continuous system, `SquareRoot` comes first and then `SmoothNlp`. Otherwise exactly one coefficient, discrete or cone class, followed by `SmoothNlp`.
- `routing::admit` gives typed eligibility per adapter.
- `select_assessed` takes the first class with an automatic adapter and the lowest constant `automatic()` rank: KINSOL 0, HiGHS 1, Ipopt 2, POUNCE 3, Clarabel 4, SCIP 5. Uno, PETSc and POUNCE-convex are explicit-only.
- `complete_context` lets a fully supplied producer proceed with the lowest-ranked root adapter while the retained decision reads "refused".

**Candidates and transitions.**
- `PreparedSolve::automatic_operations` composes `[prepared rungs, MapsAnderson, ReducedSpace, Block, Direct]`.
- After an unsatisfied native report, the driver appends the POUNCE second-opinion profiles (when the backend was POUNCE) and `automatic_backend_alternatives`: lower-ranked automatic adapters of the first class the current adapter owns, Auto plus default settings only.
- `next_automatic` dispatches the first unattempted admitted candidate. `permits_numerical_continuation` lets Converged, Auxiliary, Limited, Stalled and NumericalFailure continue.

**Sub-selectors**, each a local default:
- dynamics `Auto` takes `[Diffsol, Idas]` by refusal;
- Hessian `Auto` (exact or L-BFGS), with fitting's own variant;
- fitting's forward or adjoint choice comes from the `FitDerivatives` profile;
- presolve `Auto` is affine elimination;
- KINSOL `LineSearch` + `Klu`;
- recycle `anderson: 0`, damping 1, unweighted tears;
- block backend inheritance;
- HiGHS `choose`, which means simplex.

**Observation.**
- `SolveReport` has typed `evidence` (work counters, callback evidence, flags, typed result evidence) and untyped `metrics`, events, provenance and options.
- Storage: `runtime.solve_metrics` (name/value), `solve_strategy_events` (counts, no time), `route_decisions` (admission), `run_lineage` (hashes), `solve_runs` (outcome).

### 3.2 Well-posedness statement (profile slot 3)

- Variable roles are explicit (§6.8).
- Matching, Dulmage–Mendelsohn and BTF analysis run in compiler preparation. Over- or under-determined structure is refused before any factorization, with semantic identities (§15.2, `pse-structural::incidence`; A3).
- Structural admission is re-checked per adapter mode: Roots, Nlp, NativeFeasibility.
- PS-G2 passes in scope.

What is missing is numerical, not structural. A structurally sound but numerically singular block, or a trajectory failure, reaches the user as a termination category with no model-element localization ([F08](#f08)).

### 3.3 Numerical stage columns (profile slot 5)

| Stage | Formulation policy | Derivative source and order | Scaling | Class · capability | Status → outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Admission and routing | Declared forms, guards, MPCC realizations (ADR-0104) | Prepared order recorded in facts | Resolved nominals (§16.1) | Class from facts; adapter by constant rank | Typed refusal reasons | — |
| Square solve, KINSOL | Sign constraints only; boxes refused | Symbolica exact Jacobian, or directional JVP for Krylov | Policy-derived u/f scales | SquareRoot · KINSOL LineSearch+KLU default | All codes typed; `STEP_LT_STPTOL` → Limit | Original-space quality (§15.4) decides |
| Block sequence | As above, per block | As above | Per-block scales | One backend for all blocks | Component observation; block failure aborts | Per-block commit with independent assessment |
| Recycle | Causal maps; bounded inputs refused | None (opaque maps) | Port conversions | KINSOL FP + Anderson (depth 0 default) | Totals only | Original residual validator |
| NLP (Ipopt, POUNCE, Uno) | ℓ1 route by declared requirement | Exact or L-BFGS (FD and partitioned on POUNCE) | Native gradient scaling toggle | SmoothNlp · Ipopt → POUNCE → SCIP | Typed; restoration and limits distinct | Original quality, KKT-point analysis on request |
| LP/MILP/QP (HiGHS), cones (Clarabel), MINLP/global (SCIP) | Proven affine and degree-2; declared cones; factorable export | Coefficients; factorable | Native | By class | Typed incl. certificates and gap | Certificates verified in original coordinates |
| Dynamics (Diffsol, IDAS) | Index-1, fixed mass | Exact; forward and adjoint | Absolute and relative tolerances | Diffsol then IDAS by refusal | Typed; IDAS message always `None` | Trajectory acceptance |

**Execution fit (qualitative).** Within one solve, escalation is bounded by the attempt allowance and the deadline, which is sound. Across a study, every point replays the same failing ladder, and an appended SCIP attempt can consume the remaining deadline per point. Per-block method uniformity spends full Newton setup on blocks that need one solve. Both are assessed in §7 under AP-07.

### 3.4 Change scenarios and journeys (slot 4)

| Scenario | Today | Target behaviour |
|---|---|---|
| Add a method profile (for example Ipopt adaptive μ with quality-function oracle, or Uno filterSQP for small NLPs) | No profile concept below the adapter; raw options; preference needs a rank or a statement-order edit | One profile declaration plus one plan rule |
| Add a selection feature (block profile, KKT dimension, event density) | Extend `ProblemFacts` and its producers; about 12 hand-built `Requirements` sites; no place for a *preference* rule | One feature producer plus rules that read it |
| Add a transition (KINSOL failure → bounded least squares; local infeasibility → ℓ1 attribution) | Special-case `automatic_backend_alternatives` in `solves.rs` | One transition-table entry |
| Per-block method (affine → LU; 1×1 → bracketed) | Impossible: one backend, one settings value, `oracle_facts` affine rows false | Plan applied per block with block features |
| Test selection without native solvers (PSE-S05) | Admission and walk: yes. Composition and escalation: no; they read the global `execution::LINKED` | Plan tests with a scripted table |
| Upgrade a library (PSE-S06) | Absorbed by adapters; new statistics land untyped | New statistics land in typed families |
| Study over many cases (PSE-S07) | Ladder replayed per point; no aggregation | Advisory history reorders within admitted options; study summaries |
| Recycle that will not converge | Substitution default; totals only; no successor | History, contraction diagnosis, damping and Anderson, then simultaneous |
| Infeasible or ill-posed problem | Structural refusal strong; numerical failures continue the list | Cause-typed transition; on-failure diagnosis names elements |
| Dynamic start and event | Diffsol then IDAS by refusal; BDF default; IDAS diagnostics unbound | Event-density and steps-per-segment feature; step-limiting states on failure |
| Edit → re-solve | Warm-start and reuse mature (36c); plan recomputed blind | Plan reuses structural features; history recalls the last winner |

## 4. Findings by cause (slot 7)

The findings group under four causes:
- **A.** Method choice has no governing model of suitability.
- **B.** Recovery between attempts is not driven by diagnosed cause.
- **C.** Observations are not a decision-grade or measurement-grade product.
- **D.** Library-owned methods and structure are left unreachable.

Each finding states its cause, consequence, correction and verification. Its evidence label applies to the diagnosis; remedies are *Proposed*.

### 4.1 Cause A: method choice has no governing model of suitability

#### <a id="f01"></a>F01: Automatic selection models admissibility but not suitability. Preference is a constant rank, and the choices that matter are spread across owners that cannot see problem structure.

**Cause.**

- *Admissibility is modelled well.* `routing::admit` and the adapter `Capability` records establish, with typed reasons, which adapters can represent and honour a problem: class, bounds, derivatives, forms, sensitivities and threads.
- *Preference is not modelled.*
  - `select_assessed` takes the first problem class with an automatic adapter and the lowest constant `BackendExecution::automatic()` rank in it: KINSOL 0, HiGHS 1, Ipopt 2, POUNCE 3, Clarabel 4, SCIP 5.
  - `ProblemFacts` (`pse-math::facts`) carries counts, domains, bound shapes, affine rows, objective degree, derivative order and convexity. It carries no sparsity, block, nonlinearity, degree-of-freedom ratio, separator, conditioning or stiffness feature (A1 lead (a); *Interface-checked*).
- *The choices that most affect efficiency are made elsewhere, each by a fixed default or by a refusal order:*
  - KINSOL `LineSearch` + `Klu` (`settings/kinsol.rs` defaults);
  - Hessian `Auto`, which is exact if second order is available and limited-memory otherwise (`solves.rs`);
  - fitting's own Hessian `Auto` and its forward/adjoint choice taken from the `FitDerivatives` profile (`fitting/modeling.rs`);
  - dynamics `Auto`, which takes `[Diffsol, Idas]` by refusal with Diffsol `Bdf` + `FaerLu` (`dynamics.rs`);
  - recycle `anderson: 0` and the unweighted tear heuristic (`strategies/requests.rs`);
  - presolve `Auto`, which reduces to affine elimination (`presolve.rs`);
  - HiGHS `choose`, which always means simplex for an LP (B4);
  - Ipopt's linear solver, barrier strategy and scaling, all fixed or explicit (B1).
- *Policy is distributed.* The rank is a per-adapter constant, so the relative order of KINSOL and Ipopt is a global policy expressed in two files. `routing::Requirements` is constructed by hand at about twelve runtime sites (A1, the open 2026-10-03 F07; *Interface-checked* by count).

**Consequence.** The pipeline cannot express the rules a best-in-class simulator uses to choose the most effective method. Examples, all *Proposed*:

- Krylov with BTF-block preconditioning for a large square block, KLU otherwise.
- MA27 or MUMPS against Pardiso by KKT size and threads.
- POUNCE with Schur for block-angular structure.
- Simplex with a retained basis against IPM for LP sequences.
- ESDIRK against BDF by event density.
- Forward against adjoint sensitivities by the parameter-to-output ratio.
- Anderson against Broyden by observed contraction.

Adding one such rule today means editing the owner of that particular default, `ProblemFacts` or the twelve `Requirements` sites, and sometimes the runtime candidate builder: change amplification (AP-01, AP-03). The domain concept "which method suits this problem" has no authoritative representation, so the behaviour is not governed by an adequate model (AP-04). Correct current outputs do not compensate. PS-09 is met for class derivation and capability. Its "explicit policy … and the selection is recorded" is met only for admissibility (see [F12](#f12)).

**Correction (Proposed).** An explicit **method-selection model**, owned by one numerical-policy owner and consumed by every route that chooses a method (algebraic router, block sequence, dynamics, fitting, recycles, presolve):

1. **Admissible set.** The existing typed eligibility: unchanged, still the only gate, still recorded.
2. **Suitability rules.** Declarative rules over typed **features with declared lifetimes** that order the admissible methods and choose method-level settings: linear solver, Krylov or direct, Hessian mode, barrier strategy, scheme, sensitivity mode, acceleration depth.
   - Features and their lifetimes:
     - `Structure`: sizes, nnz, BTF statistics, affine-in-block, separator, degrees of freedom, event count;
     - `BoundSnapshot`;
     - `Values`;
     - `Start`: residual and condition at x₀;
     - `Iterate`: layer 2.
   - Only `Structure` and `BoundSnapshot` features enter compiler facts or prepared identity.
   - Adapters declare **suitability facts and method profiles**, for example "inertia-revealing", "warm-starts working set", "retains basis" or "handles two-sided boxes". They do not declare global ranks.
3. **Advisory history (layer 3).** May reorder only *within* the admissible, rule-ranked set, under a declared tolerance, and the selection record names the history-snapshot identity (§6.4).
4. **Output.** An ordered candidate list with, for each entry, the rule identifiers, feature values and any history reordering that placed it. Consumers stop re-deriving defaults independently.

Explicit selections keep their current meaning: never substituted.

**Verification.**
- Adding a suitability rule touches the policy owner and, at most, one feature producer.
- Adding an adapter declares its admissibility and suitability facts, with no rank arithmetic against other adapters.
- The selection policy is testable without native solvers (PSE-S05), as the pure `next_automatic` already is.
- *Challenge.* Could the model wrongly reject valid work? No: it only orders the admissible set. Could it erase a distinction? It must keep explicit selection and class-boundary semantics separate (see [F03](#f03)).

*Evidence: Interface-checked (A1 leads a–i; routing.rs, execution.rs, solves.rs, settings/kinsol.rs, dynamics.rs, fitting/modeling.rs, requests.rs, presolve.rs).*

#### <a id="f02"></a>F02: Block decomposition is computed but every block receives the same method, so structure never reaches method choice where it matters most

**Cause.**

- The compiler computes matching, Dulmage–Mendelsohn and BTF once per case (`pse-structural::incidence`), and the automatic ladder can run a block sequence (`solves/blocks.rs`).
- But `PreparedBlocks` holds a single `backend` for every block.
- Per-block routing uses `routing::oracle_facts`, which hard-codes `affine_rows = vec![false; …]` and `coefficients: false`. Every well-posed block is therefore class `SquareRoot` → KINSOL `LineSearch` + `Klu`.
- Staged initialization resolves a route per block but with one `SolveSettings` for all blocks.
- The block's coupling class (`AuxiliaryCouplingClass`) has no consumer.
- Row-level affine proofs (`presolve::Facts.affine`) exist but are not projected onto block unknowns.
- No symbolic solve-for-variable, explicit-assignment or equation-level tearing exists. Authored implicit functions do have form-recognizing accelerators (`implicit_affine`, `implicit_cubic`, `Accelerator::admit`) (A3; *Interface-checked*).

**Consequence.**

- In equation-oriented flowsheets, BTF typically yields many 1×1 and small blocks; many are affine in their unknown. The proportion in this repository's flowsheets is unmeasured, and presolve alias elimination reduces it.
  - Each block pays for a full KINSOL attempt: native session, KLU symbolic and numeric factorization, line-search Newton and an independent assessment.
  - An affine block needs one solve plus a residual check.
  - A bracketed scalar block is solved more cheaply by a bracketed method, which converges whenever the function is continuous on the bracket.
  - When branch or absence matters, certified isolation is available. IBEX and the Symbolica certified cubic are already bound.
- This is avoidable work that grows with block count (H9, H28), and a robustness loss in initialization, where block failures stop the schedule.
- Whether the work is *material* under AP-07 depends on two unknowns: the block-size distribution of real flowsheets, and how much of the per-block setup Plan 36c P-C4's retained block-sequence operation already removes. AP-07 therefore stays **unresolved** for this scenario, while the method-choice defect itself (AP-04, AP-03) is established.
- Large SCC blocks get no dimension reduction from tearing.

**Correction (Proposed).** Apply the [F01](#f01) selection model per block with block features:

- size;
- affine-in-block (a projection of the existing affine proofs);
- a recognized scalar form (reusing the existing implicit accelerators: affine elimination, certified cubic);
- a bracket from bounds and guard intervals;
- coupling class.

The methods per block:

| Block | Method |
|---|---|
| 1×1 affine | closed form, guarding \|a\| relative to the scales of b and x (PS-06), then a residual check |
| 1×1 nonlinear with a bracket | pinned argmin `BrentRoot` (continuity required; it returns *a* root) |
| 1×1 where branch or absence matters | IBEX isolation or the Symbolica certified cubic (both bound) |
| 1×1 without a bracket | bound-projected damped Newton |
| Affine block | one sparse LU solve (faer or KLU) |
| Small dense | dense trust region (PETSc `NEWTONTR` dogleg, already bound) or KINSOL Dense |
| Large | KINSOL KLU, or Krylov with the existing BTF block-factor preconditioner (`kinsol/block.rs`) |

Every block keeps its independent original assessment, commit and rollback (§17.1), which are strengths to preserve. They matter because forward substitution in floating point propagates upstream tolerances, and an early multi-root block can commit to a root that makes a later block infeasible.

Inside large SCCs, compare PETSc NASM over a partition (bound, explicit-only) and the existing BTF-block preconditioner before considering equation-level tearing. Tearing is structural pivoting without numerical pivoting and often worsens conditioning (Baharev, Schichl & Neumaier). It is a later, guarded option.

**Verification.** Per-block reports show method diversity; affine blocks report zero Newton iterations; the cost of a 1×1 block no longer includes native session setup. *Challenge:* a nominally affine block whose coefficient crosses zero at the current values. The guard refuses the closed form, and the block falls through to the next admissible method.

*Evidence: Interface-checked (blocks.rs, routing.rs `oracle_facts`, initialization.rs, incidence.rs; A3).*

### 4.2 Cause B: recovery between attempts is not driven by diagnosed cause

#### <a id="f03"></a>F03: Transitions are gated on the termination category and walk a fixed list. Escalation is confined to one class and to default settings, and adapter-specific recovery lives in the runtime.

**Cause.**

- `strategy::next_automatic` dispatches the first unattempted candidate of `[prepared rungs, MapsAnderson, ReducedSpace, Block, Direct]`. Observations decide only *whether* to continue, through `permits_numerical_continuation` over `Converged | Auxiliary | Limited | Stalled | NumericalFailure`. They never decide *which* candidate comes next.
- After a failed native report the driver appends two things:
  - the POUNCE second-opinion profiles, keyed on four termination codes, only when `report.backend == Backend::Pounce`, hard-coded in `solves.rs`;
  - `automatic_backend_alternatives`, which lists lower-ranked adapters of *the first class the current adapter owns*, only for `BackendSettings::Default`.
- Consequently:
  - a failed KINSOL root has no successor. A square problem KINSOL refuses a priori does reach Ipopt, but one it attempts and fails does not.
  - SmoothNlp escalates Ipopt → POUNCE → SCIP. I verified that SCIP's `automatic_classes` include `SmoothNlp`, which settles the disagreement between A1 and B4 in A1's favour.
  - Ipopt has no recovery ladder of its own.
  - An explicitly configured `BackendSettings` never escalates.
- Declared mechanisms with real producers are never chosen automatically: homotopy, pseudo-transient, bounded feasibility, least deviation, continuation, multistart, surrogate. 10 of 25 `NumericalMechanism` values have no non-test dispatch (A1 §2).
- Native reports carry a termination category and code, but no typed *cause* of failure that a transition could act on: ill-conditioning, a non-root local minimum of ‖F‖, bound truncation, restoration failure, slow contraction, or error-test clustering.

**Consequence.**

- The mathematically warranted recovery is not selected (the full mapping is in [method landscape §9](../evidence/solver-methods-and-diagnostics-2026-10-12/method-landscape-and-selection.md#9-escalation-semantics-cause--warranted-next-step)). Examples:
  - a fixed-point homotopy, or Ψtc where a stable flow exists, for a stall at a non-root minimum of ‖F‖ (another merit-function method converges to the same point);
  - rescaling, or a trust region, for an ill-conditioned Jacobian;
  - bounded least squares with an infeasibility report, step truncation, or a declared VI, according to how the bounds arise;
  - a derivative check when Newton converges only linearly;
  - damping, or deeper Anderson, for an oscillating or slow fixed point;
  - ℓ1 attribution for inconsistency;
  - root location and reinitialization for an unlocated discontinuity.
- A global branch-and-bound solve, worst-case exponential, may follow a local failure whose usual causes (initialization, scaling, degeneracy) it does not address (B4 §6.3). It is reached only when SCIP's finite-box and factorable eligibility holds.
- Adding a ladder for Ipopt or KINSOL requires another feature-gated branch in `solves.rs`. Adapter knowledge leaks into orchestration (AP-02) and change amplifies (AP-03).
- The between-attempt policy is not an explicit model (AP-04).

**Correction (Proposed).**

1. **Typed failure causes.** Adapters and the trajectory analysis ([F05](#f05)) produce a typed `FailureCause`/trajectory diagnosis with declared semantics. Library signals are listed in §6.2.
2. **A declarative transition table** owned by the selection model maps (class, cause, budget left, permissions) to admissible next methods. It includes **reformulating transitions** to derived families that already exist under ADR-0155, with provenance recorded, for example square → bounded feasibility or least deviation, square → pseudo-transient, NLP → ℓ1. The 2026-10-03 maintainer direction already accepts these when provenance is recorded.
3. **Recovery profiles become adapter-declared capability.** POUNCE's second-opinion profiles, and new Ipopt profiles, are offered through `BackendExecution`, not branched in the runtime.
4. **Global solves need explicit permission.** SCIP and multistart join a local-failure chain only through an explicit intent or a request permission ("certify", "global allowed"), never by rank fall-through.
5. **Unchanged:** explicit selection never substitutes; cancellation, resource and infrastructure failures stay terminal; one work budget spans the ladder.

**Verification.**
- An Ipopt recovery profile is added without touching `solves.rs`.
- A square-root failure with a "non-root minimum" cause takes a continuation or least-deviation transition, and the trace names the cause and rule.
- *Challenge:* a request whose science forbids reformulation, for example an exact square-root contract with no least-squares candidate. Transition permissions are part of the request and are respected.

*Evidence: Interface-checked (strategy.rs `next_automatic`, `permits_numerical_continuation`; solves.rs `automatic_operations`, `automatic_backend_alternatives`; execution/scip.rs `automatic_classes`; pounce/profiles.rs; A1).*

#### <a id="f04"></a>F04: Recycle convergence defaults to the weakest method, monitors nothing and records no iteration history

**Cause.**

- `RecycleRequest::from_model` sets `anderson: 0`, damping 1 (plain successive substitution) and, without authored tears, the *unweighted* greedy heuristic. The weighted HiGHS MILP route exists in `flows.rs` and `tears.rs` but is a separate tool step.
- KINSOL `KIN_FP` owns the iteration and emits no per-iteration record. KINSOL's info logging is compiled out at `SUNDIALS_LOGGING_LEVEL=2`.
- Recycle history is therefore final totals only. No events exist in `kinsol.rs` or `recycle.rs`; I confirmed this myself.
- Units are opaque causal maps with no Jacobian, and sensitivities are refused inside the sweep (`admit_unit_profile`).
- No policy exists for design specifications converged nested or simultaneous with recycles.
- Conditional units restart from authored guesses every sweep; no recorded-start policy is applied (A3).

**Consequence.**

- **PS-08 (MUST):** "Recycle convergence is a declared policy … with its iteration history recorded." The history is not recorded, so PS-G3 fails for the recycle journey.
- High-recycle-ratio flowsheets converge linearly at rate ρ (the dominant eigenvalue of ∂G/∂x) or diverge. Nothing detects it.
- When a recycle "will not converge", which is the profile's canonical diagnosability journey, the user sees totals and a termination code. They see no contraction estimate, no oscillating tear and no failing unit.

**Correction (Proposed).**

1. Record the per-iteration ‖x − G(x)‖ and per-tear deltas from the existing residual callback. KINSOL calls the map exactly once per `KIN_FP` iteration, so this costs nothing extra (B2 §1.4). Derive the contraction estimate ρ̂ and Wegstein slopes as typed evidence.
2. Default to damped successive substitution or library-owned Anderson acceleration (depth about 3–5, with `KINSetDelayAA`).
   - Damping converges for any real eigenvalue μ < 1 of ∂G/∂x, including oscillatory ones.
   - Anderson can converge even when ρ > 1 (Walker & Ni), and it adds no G evaluations.
   - Select by recorded observations: the geometric-mean contraction over a window, the oscillation sign of Δxₖ₊₁·Δxₖ, the per-tear Err/Tol, the unit-failure rate, and the noise level of G.
   - On oscillation, increase damping. When slow, increase depth. When noisy, tighten inner tolerances. On persistent failure, move to the simultaneous route.
   - For small tear sets, a dense finite-difference tear Jacobian plus Broyden (`SNESQN`) is the classic exact-ish route.
   - Jacobian-free Newton–Krylov on tears (KINSOL's internal difference-quotient Jv, or PETSc `-snes_mf`) works with opaque maps *only* when G is deterministic and smooth and inner tolerances lie far below the difference increment.
   - Bounded Wegstein (the default tear method in Aspen, Pyomo, DWSIM and IDAES practice, B5b) cannot ride on KINSOL FP. It would be an owned loop, mathematically largely subsumed by Anderson. Adopt it only if parity with the reference tools is wanted, as a bounded composition.
3. Use the weighted tear MILP when costs are declared. Tear choice changes the convergence rate ρ itself (Upadhye & Grens), so the default objective should weigh convergence (torn loops, stable tear variables), not only tear count.
4. Make nested versus simultaneous specification convergence a declared policy.
5. Exact Newton-on-tears needs per-unit implicit-function-theorem JVPs from each conditional unit's own root solve. That is a later capability.

**Verification.** A non-converging recycle publishes its iteration history and ρ̂; PS-08's audit question is answered from the stored record.

*Evidence: Interface-checked (requests.rs, recycle.rs, kinsol.rs, flows.rs, tears.rs; B2 §1; A3 §3). Behavioural defect against PS-08.*

### 4.3 Cause C: observations are not a decision-grade or measurement-grade product

#### <a id="f05"></a>F05: Typed observation is too thin for decisions or measurement, and the evidence/metrics split classifies data by intended use rather than by semantics

**Cause.**

- `SolveReport` separates typed `evidence` from `metrics: BTreeMap<String, Metric>`, documented "Decisions read `evidence`, never these keys".
- Typed work is `WorkEvidence`: four optional counters. `iterations` is unset for POUNCE (B1), and several adapters write iterations only as metrics (A2).
- Everything semantically rich is string-keyed: POUNCE statistics flattened, Ipopt iteration events, KINSOL counters, HiGHS info, Clarabel info, SCIP bounds.
- "Iteration" means a Newton step, an IPM iteration, a simplex pivot, a BDF step or a B&B node depending on the adapter, and nothing declares which.
- **G2 (narrow).** `WorkEvidence` documents absence as "the native API supplies none". But typed `iterations` is set only by the Ipopt, Uno, PETSc and conic adapters. KINSOL (`KINGetNumNonlinSolvIters`), POUNCE (`statistics.iteration_count`), HiGHS and SCIP report iterations natively, and the typed ledger still reads "unknown" (independent assessment IR-04; grep).

**Consequence.**

- Layer 2 cannot use stagnation, contraction, regularization or restoration signals without either violating the rule or reading strings at the point of use. PS-10 forbids parsing at the point of use.
- Layer 3 and measurement cannot compare methods across adapters.
- The rule's real protection is against an undeclared ad hoc key becoming a decision input (DP-01). It is achieved by typing, not by forbidding the data.

**Correction (Proposed).**

- A typed **attempt observation** (§6.1) with:
  - common fields of declared semantics: per-phase time, iterations with their meaning, evaluations per demand, factorizations, rejected trials, typed cause;
  - a per-adapter typed statistics sum type;
  - bounded typed trajectories per method family;
  - derived trajectory diagnoses: an observed-convergence-order estimate, contraction, stagnation, restoration share.
- Rule change: decisions read only typed observations with declared semantics, and ad hoc metrics stay diagnostic (see [RC04](#rc04)).
- Adding a library statistic becomes an adapter-local typed change plus generated schema. It must not become a cross-cutting edit; verify this with the "add a library statistic" scenario.

*Evidence: Interface-checked (solve.rs `SolveReport`, `Evidence`, `WorkEvidence`; A2 §1).*

#### <a id="f06"></a>F06: Libraries' statistics and health signals are left unread

**Cause** (the captured-versus-available matrix is in [diagnostics-and-measurement.md](../evidence/solver-methods-and-diagnostics-2026-10-12/diagnostics-and-measurement.md#6-captured-versus-available-matrix)):

| Library | Available but not read |
|---|---|
| Ipopt | `Statistics()` and `TimingStats()`, which the C++ bridge can already reach (B1) |
| Uno | Residuals, which cross the C boundary and are dropped; its per-iteration termination callback is used only to poll |
| PETSc | Per-iteration norms, received and discarded; `SNESSetConvergenceHistory` |
| KINSOL | The fixed-point history, reconstructable for free; KLU `rcond`, `condest`, `singular_col` and `rgrowth`, some computed and discarded |
| IDAS | `IDAGetIntegratorStats`, orders, steps, setups, Jacobian evaluations, linear failures, and `IDAGetErrWeights` × `IDAGetEstLocalErrors` (the step-limiting variables) |
| Diffsol | `order()` and `h` |
| HiGHS | Final runtime and 12 info values |
| SCIP | LP iterations, depth, timing, and the structured `SCIPcollect*Statistics` datatree or JSON |
| FERAL | `n_tiny`, ordering, scaling and refinement information |

Also:
- Error-handler messages: `NativeTermination.message` is always `None` for IDAS.
- The NLP bridges' throw-based stops are a separate defect with its own correction ([F13](#f13)).

**Consequence.**

- Diagnosing a slow or failing solve today requires rerunning with native verbosity and reading journals: the investigative effort the maintainer wants to remove.
- Layer 2 has none of the trajectory signals it needs.
- This is a target gap in library fit (slot 8), not a G8 failure. G8 fails only where generic capability is reimplemented bespoke, and none was found. PS-10 is affected through the lost failure evidence.

**Correction (Proposed).**
- Capture these statistics in the typed per-adapter structures ([F05](#f05)).
- Install a SUNContext error handler to populate termination messages.
- Rebuilding SUNDIALS with logging, or moving to ≥ 7.3/7.8 for the damping, depth and in-process logger callbacks, is optional. Callback reconstruction already covers the fixed-point history.

*Evidence: Interface-checked (B1–B4; the principal reviewer checked the Ipopt bridge's missing `Statistics()` call).*

#### <a id="f07"></a>F07: No timing, aggregation, durable dynamics statistics or history identity exists, so performance cannot be managed or learned from

**Cause** (A2):

- No attempt, rung or phase wall time is stored anywhere. `SolveReport::new` records none, `canonical_attempts` has no start or end, and `solve_strategy_events` carries counts only.
- Preparation spans (`pse.case.*`) have no production subscriber, and native code has no tracing.
- Dynamics `Report.statistics` is in-memory only.
- Studies keep per-point outcomes with no cross-point summary.
- No structure-only identity exists. `preparation_identity` folds in controls, backend and build; `seed_preparation_identity` includes layout and backend.
- Resolved `Controls` are persisted only as a hash, and no host identity is recorded.
- `case-measure` and `benches/` are qualification campaigns, not runtime telemetry.

**Consequence.**
- The maintainer's measurement requirement has no mechanism.
- Regression tracking across releases or library upgrades is impossible.
- The advisory history layer has neither data nor key.
- Engineers cannot see where time goes (callbacks against linear algebra against native iteration against assessment), nor which rung of a ladder wasted the budget.

**Correction (Proposed)**, detailed in §6.3–6.4:

- per-attempt, per-rung and per-phase wall and CPU time, plus admission waiting, in the typed observation;
- typed persisted relations for the attempt observation and ladder, including dynamics statistics;
- study summaries;
- a **structure-only key** plus a feature vector plus environment identity (native build, library versions, host class) for history records;
- persisted resolved `Controls`;
- runtime telemetry kept distinct from `case-measure`, but sharing its identity vocabulary.

*Evidence: Interface-checked (A2 §1–§5, §8–§9).*

#### <a id="f08"></a>F08: A capable diagnostics catalogue runs only on request; trajectory failures are not localized in model terms; failed solves cannot be replayed

**Cause.**

The diagnostics machinery is substantial and must be preserved:

- §15.4 original-space quality.
- A 25-rule `FindingKind` catalogue (`workflow/modeling/diagnostics.rs`) with explicit dispositions and a shipped `idaes-2.13` threshold profile (`packages/reference/diagnostics/`). It covers:
  - structural under- and over-determination;
  - domain evaluation hazards;
  - bounds, values and term mismatch or cancellation;
  - extreme and parallel Jacobian rows, columns and entries;
  - numerical rank, and a condition estimate.
- The HiGHS MILP degeneracy hunter and LP conditioning certificates (`jacobian_diagnostics.rs`).
- KKT-point analysis with inertia and condition.
- The ℓ1 and deletion-filter infeasibility explanations, and SCIP IIS and certification.

I confirmed the catalogue myself. B5b's comparison table finds it near parity with IDAES `DiagnosticsToolbox`, and stricter where structural defects are admission refusals.

The gaps:

1. **Nothing runs automatically after a failed or limited attempt** (A2 §7). A trajectory failure (stall, restoration failure, error-test failure) reports native status and termination only. It names no residual rows, active bounds, near-singular directions or step-limiting states.
2. **No solve-convergence regression analysis.** There is no equivalent of IDAES `IpoptConvergenceAnalysis`: a sweep with per-point iterations, restoration and regularization counts compared against a stored baseline. The per-iteration data it would need is already captured for Ipopt (B5b).
3. **Thresholds have one severity tier.** IDAES separates caution (1e4/1e-4) from warning (1e8/1e-8).
4. **Missing checks:** no "constraint with no free variables" check, and no sparse smallest-k SVD (`MatrixPolicy` is dense-bounded, 65,536 entries in the profile).
5. **No scaling-method profiling.**
6. **Replay is incomplete.** A failed solve cannot be reproduced exactly from storage: the full `Controls`, host identity and model export are missing (A2 §5).
7. **Stale documentation.** §15 still says DiagnosticsToolbox parity "is not a current capability" and that there is "no named … check catalogue". That is documentation drift against the implementation (G7, a narrow fail).

**Consequence.**
- PS-10 ("failures name the units, streams, equations or variables involved") holds for the feasibility checks but not for trajectory failures.
- PS-07's "conditioning diagnostics are model attributes" is met only on request.
- Each failure therefore triggers manual investigation, although the machinery to answer it exists.
- Regressions in solve effort across releases or library upgrades go unnoticed.

**Correction (Proposed).**

1. An automatic, bounded **on-failure diagnosis tier** (§6.3, T2). It reuses the catalogue at the final or best iterate, plus signals the factorization already produced: KLU condest and `singular_col`, FERAL κ₁, the convergence-order diagnosis, evaluation-error and envelope-hit locations per equation and provider, and IDAS step-limiting states. It emits typed findings referencing model elements and never changes the outcome classification.
2. A **convergence-effort regression** facility over the typed attempt observation ([F05](#f05), [F07](#f07)), keyed by structure-only identity and environment.
3. A second severity tier, the no-free-variable check, and a sparse smallest-k SVD path. The SPQR rank route under R36-01 complements the last.
4. A **replay snapshot** on request or by failure policy, under the run's retention.
5. Correct §15.

*Evidence: Interface-checked (`FindingKind`, profile directory, A2 §5–§7, B5b Part 1).*

#### <a id="f12"></a>F12: The selection record holds admissibility but neither preference reasons nor the executed method, and some records diverge from execution

**Cause.**

- `runtime.route_decisions` records per-adapter eligibility. It does not record rank, class-ordering reasons or the sensitivity preference that filters adapters (`routing.rs`).
- `complete_context` lets a refused routing decision proceed with the lowest-ranked root producer while the stored decision still reads "refused". A code comment documents this as a producer context (A1 lead l).
- The executed method is not always the requested one:
  - HiGHS `solver=ipm` silently runs IPX when HiPO's extras library is absent (B4).
  - Ipopt's linear solver and phase are recorded only as options.
- Capability claims overstate:
  - Ipopt's record claims "timing", but no native timing is read (B1).
  - HiGHS claims native coefficient and bound updates with compatible layout, but clears solver state before every solve ([F10](#f10)).
  - §18.9 omits Uno and PETSc, although both are linked adapters.
- Cone-lowering failure aborts preparation while factorable failure re-routes (A1 lead m).

**Consequence.** PS-09's "the selection is recorded" is partial, and G7 (truthful capability claims) and G4 fail narrowly. Engineers and the history layer cannot trust why a method ran or which one did.

**Correction (Proposed).** The selection record from [F01](#f01) carries reasons and history snapshot. The attempt observation carries the *executed* method identity (actual IPM, linear solver, scheme). Correct the capability strings and §18.9. Make the producer-context case explicit in the record. Give cone-lowering failure the same refusal-and-continue route as factorable failure.

*Evidence: Interface-checked (routing.rs `Decision::row`, solves.rs `complete_context`, execution/ipopt.rs, execution/highs.rs; B1, B4).*

#### <a id="f13"></a>F13: The NLP bridges stop by throwing, so a stop decided by policy loses the iterate and the statistics

**Cause.**

- The Ipopt bridge's `intermediate_callback` always returns `true`. It stops only by `throw Terminal{}`, and `finalize_solution` returns early under the terminal latch.
- On that exception path Ipopt creates no `SolveStatistics` (B1 §1.4, §1.7).
- The Uno bridge's termination callback always returns 1 and stops by throwing. Uno converts the throw to `ALGORITHMIC_ERROR`.
- POUNCE returns `false` only when evaluation fails.
- All three libraries have cooperative stops that preserve the iterate:
  - Ipopt: `false` gives `User_Requested_Stop`, still calls `finalize_solution`, and still builds statistics;
  - Uno: returning 0 gives `USER_TERMINATION`;
  - POUNCE: `false` keeps its result.

**Consequence.**
- Layer 2's natural realization is observe → stop cooperatively → re-solve from the retained iterate, with the same structure (`ReOptimizeTNLP`, POUNCE warm start or working set) or with a different method.
- With the current bridges, every in-solve switch restarts from the submitted start and loses the trajectory evidence diagnosis needs. Libraries do not allow option changes mid-solve (B1 §4), so the cooperative stop is the only library-respecting switch point (PS-09).

**Correction (Proposed).**
- Add a typed **policy stop**, distinct from cancellation, deadline and abandonment, delivered through each library's cooperative path.
- Retain the stopped iterate and statistics as an `Auxiliary` start proposal, screened by the existing start rules.
- The stop predicate is a pure value evaluated over the typed observation family ([F05](#f05)). It decides *when* to abandon a trajectory and never *how* to step.
- Cancellation, deadline and resource stops keep their current terminal throw semantics.

**Verification.** A scripted policy stop on Ipopt yields `User_Requested_Stop`, a retained iterate and non-null statistics (independent assessment probe 2).

*Evidence: Interface-checked (`pse_ipopt_sequence.cpp`, `pse_uno.cpp` stop paths; B1; independent assessment IR-05).*

### 4.4 Cause D: library-owned methods and structure are left unreachable

#### <a id="f09"></a>F09: Methods the target needs are available in pinned or bundled libraries but are unbound or unselectable

**Cause** (B1–B5a; *Interface-checked*):

- **Square systems.** The pinned PETSc 3.24 has `SNESVINEWTONRSLS`/`SSLS` (complementarity on square systems, valid only when the formulation declares the equation-to-variable pairing), `SNESQN` (Broyden), `SNESNGMRES`, `SNESANDERSON`, `SNESLINESEARCHNLEQERR` (affine-invariant damping) and `SNESNEWTONAL` (arclength). Only `newtontr`, `nasm` and pseudo-transient are bound.
  - KINSOL's internal difference-quotient Jv is unusable because the adapter requires an analytic Jv. That blocks JFNK on opaque maps.
  - `KINSetMaxSubSetupCalls` and the residual-monitoring parameters are unbound.
- **Least squares and bounded least squares.** PETSc TAO is pinned and wholly unbound: BRGN with LM regularization, POUNDERS, BNTR, TRON. These also serve the "root lies outside the box" case of square systems. No LM route exists.
- **Scalar blocks.** argmin `BrentRoot` is pinned and used only by the surrogate path.
- **NLP.**
  - Ipopt's `mu_oracle`, least-squares initialization, `start_with_resto`, `warm_start_target_mu` and `derivative_test` are raw-only. `MuStrategy` types monotone and adaptive only.
  - HSL is unbound. MA27, MA28 and MC19 are free for personal use through the Coin-HSL Archive, while MA57 and MA97 are free only to academics (B5a). MC19 is also what Ipopt's equilibration scaling needs.
  - Uno is built with HiGHS only. Its interior-point and `auto` presets cannot run, and no Hessian callback is bound.
- **LP and QP.** HiPO is not built, so `solver=ipm` means IPX.
- **Dynamics.** CVODES is bindable by enabling a feature (for ODEs only). ARKODE needs new bindings and cannot take singular mass matrices.
- **SCIP.** Emphasis meta-settings are untyped, and the structured statistics are unbound.
- **Sparse rank.** Rank-revealing sparse QR is unavailable: faer's sparse QR is non-pivoting, and SPQR is blocked by the broken `suitesparse_sys` binding. The Plan 36 R36-01 proposal already covers this (`36-investigation-evidence/sparse-rank.md`).

**Consequence.**
- The selection model ([F01](#f01)), transitions ([F03](#f03)) and recycle escalation ([F04](#f04)) would have too few methods to choose among.
- Two-sided-box square systems and least squares, both common in process models, have no proper method.
- Pressure builds for bespoke machinery that PS-09 forbids when a qualified library exists.
- This is a library-fit target gap (slot 8). G8 itself passes: nothing is reimplemented.

**Correction (Proposed).** Bind these as adapter methods and typed settings, admitted through eligibility and ordered by suitability. Library-first order, from B5a's ranking:

1. PETSc VI, QN, NLEQ-ERR, NGMRES and Anderson, plus TAO BRGN-LM;
2. KINSOL's difference-quotient Jv for maps;
3. argmin `BrentRoot`;
4. typed Ipopt barrier and initialization options;
5. HSL MA27 and MC19 through the archive, as a host-local checksummed input;
6. SPQR under R36-01;
7. CVODES for mass-free ODE subsystems.

SHOT/Bonmin (convex MINLP outer approximation and ESH) and qpOASES/DAQP (parametric active-set QP) are distinct but lower-materiality candidates. Their duplicates and revisit triggers are in B5a.

#### <a id="f10"></a>F10: HiGHS discards its retained basis before every solve, while its capability record claims native in-place updates

**Cause.** `Highs_clearSolver` runs before each solve (`highs.rs`). HiGHS updates its stored basis when the model changes through its own methods, and clearing throws that away. A basis survives only if the caller round-trips the exported `WarmStart` (B4 §1.3).

**Consequence.**
- LP and QP sequences re-factorize from a submitted basis or cold-start: tear MILP relaxations, reconciliation, MPC and studies. Simplex hot start is the strongest warm start in the stack.
- G7: the capability string claims native coefficient and bound updates.

**Correction (Proposed).** Keep solver state across compatible in-place updates under the existing start-permission rules (`NoPriorStart` must still clear it), and record which start was used.

*Evidence: Interface-checked (B4); probe 2 in B4 would settle the magnitude.*

#### <a id="f11"></a>F11: Decomposable structure reaches a decomposition method only by accident of rank

**Cause.**

- The compiler's `automatic_separator` (shared-instance and cross-block columns) is attached only when the selected backend is POUNCE and composition is Auto (`solves.rs`). POUNCE is reached automatically only after Ipopt fails.
- Fitting's multi-experiment NLP and multiple shooting never set a separator, so POUNCE's Schur path is not used for their block-angular structure (A3 §4).
- The BTF coupling class is unconsumed.
- PETSc NASM over BTF blocks is explicit-only.

**Consequence.** Block-angular problems are solved monolithically: multi-period problems, multi-experiment estimation and multiple shooting. Their natural decomposition, which is also the future parallel seam (H22), is unused.

**Correction (Proposed).** Make separator size against block sizes a `Structure` feature in the selection model, so block-angular problems are ordered towards Schur-capable methods a priori. Fitting and shooting oracles should supply their experiment and segment separators. Parallel execution over these seams stays out of scope.

*Evidence: Interface-checked (solves.rs separator attachment; workspace.rs `automatic_separator`; A3 §4).*
## 5. How the findings relate

**Prerequisite contracts.** Two findings are prerequisites for most of the others:

- [F01](#f01), the solve plan and features, is the shared *contract*. [F02](#f02) (per-block choice), [F03](#f03) (transitions), [F11](#f11) (structure-driven decomposition) and the history layer all need a plan to express themselves.
- [F05](#f05) (typed observation families) and [F13](#f13) (cooperative stop) are prerequisites for [F03](#f03)'s cause-typed transitions, [F08](#f08)'s diagnosis and [F07](#f07)'s observation summary.

**Partial independence.** [F07](#f07) depends on [F01](#f01) for the plan identity, on F01's structural features for the feature key, and on [F05](#f05) for the observation summary. Its time and outcome fields can land independently.

**Priority is not the same as prerequisite order.** By consequence, the behavioural defects are the most urgent, and none of them needs F01 first:

- [F04](#f04): recycle history, damping and Anderson defaults (PS-08);
- [F03](#f03)'s SCIP-after-local-failure edge;
- [F12](#f12) and [F10](#f10): truthfulness of records and claims, plus the HiGHS basis;
- [F05](#f05)'s typed iterations (G2).

**The remedies fit together** under four conditions:

- The plan stays a *pure consumer* of admission and never becomes a second admission authority.
- Observation families are defined per *method family* (interior-point NLP, Newton root, fixed point, integrator, branch-and-bound, LP/QP), not per library. Otherwise the transition rules multiply per adapter.
- History records are keyed by a structure-only key plus features, with environment partitioning. They never feed admission.
- Telemetry stays non-interfering, consistent with Plan 36b P-B6. Diagnostic capture never changes or cancels a scientific outcome.

**The interactions that need care:**

- **Per-block method choice ([F02](#f02)) and the 36c retained block sequence (P-C4).** Per-block choice must use P-C4's operation-grain block execution, not reintroduce per-block external crossings.
- **Cooperative stops ([F13](#f13)) and the PS-10 veto.** A policy-stopped iterate is a *start proposal*, never a result.
- **Advisory history ([F07](#f07)) and PS-11.** History changes which candidate succeeds first, and so which local solution a nonconvex problem reaches. It is therefore a result-affecting input and must be recorded ([RC08](#rc08)).

## 6. Target design (recommendations)

All of §6 is *Proposed*. It describes intended behaviour and responsibility boundaries, not work packages.

### 6.1 The solve plan: one owner for preference, transition and advisory adjustment

**What it consumes:**

- the admitted set from the existing `routing::admit`, unchanged;
- **selection features** with declared lifetimes ([method landscape §8](../evidence/solver-methods-and-diagnostics-2026-10-12/method-landscape-and-selection.md#8-feature-model));
- capability records extended with **typed method profiles** and **suitability facts**, for example inertia-revealing, warm-starts the working set, retains the basis, handles declared complementarity;
- request constraints and permissions: explicit selection, reformulation allowed, global or multistart allowed, start rules, budget;
- optionally, an identified **history snapshot**.

**What it decides:**

1. **Order.** A declarative order over admitted (adapter, profile) candidates, using rules keyed to features. Every position records the rule that placed it. Rules come from the [method landscape](../evidence/solver-methods-and-diagnostics-2026-10-12/method-landscape-and-selection.md). For example:
   - per-block methods by size, affinity, bracket and branch requirement;
   - Krylov with BTF preconditioning against KLU;
   - Schur-capable NLP for block-angular structure;
   - KKT linear solver and ordering by size, inertia requirement and structure;
   - simplex with a retained basis against IPM for LP sequences;
   - one-step methods against BDF by steps per segment;
   - forward against adjoint sensitivities by the parameter-to-output ratio;
   - Gauss–Newton with forward sensitivities against L-BFGS with an adjoint gradient.
2. **Transition table.** Rows map a typed failure cause ([F05](#f05)) to the admitted next candidates, a start-transfer rule (a policy-stopped iterate as an `Auxiliary` proposal, [F13](#f13)) and the stated mathematical rationale. A class-crossing edge is allowed only when layer 1 admitted the target, for example `SquareRoot` ⊂ `SmoothNlp`. Reformulating edges to ADR-0155 derived families carry provenance of the original equations and of the modifications.
3. **Advisory adjustment (layer 3).** A permutation within the admitted, rule-ranked set, under declared tolerances, plus tuning of per-attempt budget slices. It needs a minimum effective sample and a confidence margin. Exploration is deterministic, seeded and recorded. It never skips an option layer 1 would try and never applies to explicit selections.

**What it produces.** An ordered plan persisted with the route decision: candidates, rules, feature values, transitions and history snapshot. `automatic_operations` becomes the binder that prepares plan entries. The about 12 `Requirements` constructions collapse into one plan builder per representation.

**What consumers stop doing.** These sub-selectors become plan outputs; consumers no longer re-derive defaults themselves:

- dynamics `Auto`;
- Hessian `Auto` and its fitting variant;
- fitting's forward/adjoint choice;
- presolve `Auto`;
- recycle acceleration and tear method;
- block backend inheritance;
- KINSOL's linear solver;
- the HiGHS method.

**Preserved:**

- admission semantics;
- explicit-selection non-substitution;
- terminal classes;
- one work budget per ladder;
- independent original assessment of every candidate;
- per-block commit and rollback.

**Revealing challenge.** A request whose science forbids reformulation, such as an exact square-root contract with no least-squares candidate. Permissions are request inputs, and the plan cannot add transitions the request forbids.

### 6.2 Observation families and the cooperative policy stop

**Typed observation families per method family:**

| Family | Fields |
|---|---|
| Interior-point NLP progress | inf_pr, inf_du, μ, δw (nonconvexity), δc (LICQ), α, restoration share, `info_string` flags |
| Newton root progress | ‖F‖, step length, backtracks, β-condition failures, linear iterations; Θ only where PETSc NLEQ-ERR provides it |
| Fixed-point contraction | per-iteration ‖x − G(x)‖, per-tear Err/Tol, geometric-mean contraction, oscillation sign, unit-failure rate |
| Integrator step health | order, step size, error-test and convergence failures, setups, step-limiting states |
| Branch-and-bound progress | nodes, bounds, gap, LP iterations, depth |
| LP/QP progress | iterations by method, basis validity, executed method |

**Derived trajectory diagnoses**, computed locally from the bounded in-memory trajectory at the end of each attempt:

- observed convergence order; linear convergence with constant contraction flags a derivative-check need (PS-07);
- stagnation window;
- contraction;
- restoration and regularization shares;
- error-test clustering in time.

These typed values may feed layer-2 decisions. Raw `metrics` stay an unpromoted diagnostic archive ([RC03](#rc03)). Every adapter sets typed `iterations`, with a declared meaning, wherever its library reports them (G2).

**Cooperative policy stop.**

- Distinct from cancellation, deadline and abandonment.
- Delivered through each library's cooperative path: Ipopt `intermediate_callback` returning false, Uno's termination callback returning 0, POUNCE returning false.
- Retains the iterate and statistics as a start proposal.
- Its predicate is a pure value over the family observation. It decides *when* to abandon a trajectory, never *how* to step (PS-09).

### 6.3 Measurement: the attempt record, tiers and analytics

**One compact typed attempt relation** per attempt (an Arrow relation). Columns:

- structure-only key and feature vector;
- plan identity, candidate position and rule;
- executed method identity: actual IPM, linear solver, scheme;
- family observation summary and derived diagnoses;
- typed work;
- **wall and CPU time** per phase where libraries expose it (Ipopt `TimingStats`, POUNCE timing), and total wall time always;
- admission waiting time;
- preparation cache hits and rebuild reasons;
- outcome class and typed cause;
- environment identity: native build, library versions, host class;
- history snapshot identity if one was used.

**Study summaries, convergence-effort regression and history statistics** are DataFusion aggregations over this relation, not bespoke summarizers. Convergence-effort regression is the IDAES `IpoptConvergenceAnalysis` behaviour, over typed records.

**Detail tiers.**

- **T0**, always on: counters and timers.
- **T1**: the bounded trajectory.
- **T2**: on-failure diagnosis (§6.4).
- **T3**: on-request deep analyses.

The tiers stay consistent with Plan 36b P-B6. Per-iteration telemetry is a non-interfering tap with partiality and drop evidence. A guaranteed small channel carries phase, failure and terminal events. Telemetry never back-pressures or cancels a solve.

**History store (layer 3).** The design follows the algorithm-selection literature ([B5b Part 3](../evidence/solver-methods-and-diagnostics-2026-10-12/reference-practice-and-selection-literature.md)):

- two key tiers: an exact structure key for replay, features for generalization;
- lexicographic scoring: verified acceptance first, then PAR-k cost in machine-independent work or wall time labelled with its environment;
- failures treated as right-censored;
- partitioned by solver build;
- cold start falls back to the layer-1 order;
- each study freezes one snapshot, so point results do not depend on completion order.

**Runtime telemetry versus qualification.** Runtime telemetry is distinct from `just case-measure` and the benchmarks, which remain controlled qualification campaigns. The two share identity vocabulary, not storage.

### 6.4 Failure diagnosis and replay

**An automatic, bounded T2 diagnosis** runs at the final or best iterate whenever a plan exhausts or an attempt stops at a limit. It reuses the 25-rule catalogue and signals already computed:

- ranked residual rows and active bounds by model element;
- scaling spread against the declared nominals;
- KLU condest, `numerical_rank` and `singular_col`, FERAL κ₁ and `n_tiny`, or the bounded SVD;
- the convergence-order diagnosis;
- evaluation-error and envelope-hit locations per equation and provider;
- IDAS step-limiting states (`IDAGetErrWeights` × `IDAGetEstLocalErrors`);
- Ipopt restoration slacks for attribution, where exposed.

The diagnosis maps native indices to semantic identities through the existing coordinate maps and emits typed, persisted findings. It never changes the outcome.

**Catalogue additions:**

- a second severity tier;
- a no-free-variable check;
- a sparse smallest-k SVD (complemented by SPQR under Plan 36 R36-01);
- scaling-method profiling.

**Replay snapshot**, on request or by failure policy, under the run's retention:

- full resolved `Controls`;
- granted threads;
- host class;
- the start vector;
- native model export where the library offers one (HiGHS `writeModel`, SCIP `SCIPwriteOrigProblem`).

### 6.5 Methods to bind, library-first

| Gap | Library owner | Status | Caution |
|---|---|---|---|
| Per-block closed form, bracketed scalar, certified branch | Closed form (guarded); argmin `BrentRoot`; IBEX; Symbolica certified cubic | Pinned; Brent unused | Continuity; root branch |
| Two-sided bounds on square systems | Step truncation or transforms (domain guards); TAO BNTR, TRON or BRGN (root outside the box); PETSc VI (declared complementarity) | Pinned, unbound | VI needs a declared equation–variable pairing |
| Error-oriented damping; arclength; Broyden; NGMRES; Anderson | PETSc NLEQ-ERR, `SNESNEWTONAL`, `SNESQN`, `SNESNGMRES`, `SNESANDERSON` | Pinned, unbound | NLEQ-ERR not with VINEWTONRSLS |
| JFNK on opaque maps | KINSOL internal difference-quotient Jv; PETSc `-snes_mf` | Unbound | Inner tolerance far below the difference increment |
| Least squares, LM | TAO BRGN-LM | Pinned, unbound | — |
| Recycle acceleration defaults | KINSOL damping and Anderson (delay, depth) | Bound, default off | Noise floor of G |
| Wegstein / dominant eigenvalue (parity with Aspen, Pyomo, IDAES) | none; owned bounded loop | — | Optional; mathematically subsumed by Anderson |
| NLP diagnostics and timing | Ipopt `Statistics()`, `TimingStats()` | Reachable through the C++ bridge | Needs `timing_statistics=yes` |
| NLP options | Ipopt `mu_oracle`, least-squares initialization, `warm_start_target_mu`, `derivative_test` | Raw-only | Type them as profiles |
| KKT linear solvers | HSL MA27 and MC19 (personal-free archive); MA57 and MA97 (academic only) | Unbound | Host-local checksummed input; not redistributable |
| Rank-revealing sparse QR | SuiteSparse SPQR (+CHOLMOD) | Blocked binding | Plan 36 R36-01 owns the method decision |
| Dynamics diagnostics | IDAS getters; Diffsol `order()` and `h`; SUNContext error handler | Unbound | Logging is compiled out at level 2 |
| ODE subsystems | CVODES (by enabling the feature); ARKODE (needs bindings; nonsingular mass only) | Disabled | — |
| Rosenbrock | PETSc TSROSW | Reachable by type switch | Autonomous systems; Jacobian lagging |
| LP/QP sequences | HiGHS retained basis; executed IPM recorded; `getRunTime` | Basis cleared | — |
| Global statistics | SCIP `SCIPcollect*Statistics`, emphasis, `SCIPwriteOrigProblem` | Unbound | 10.1 adds solving-phase observation |
| Convex MINLP (OA, ESH) | SHOT or Bonmin | Candidate | Lower materiality |
| Parametric active-set QP | qpOASES or DAQP | Candidate | First establish what POUNCE QP homotopy lacks |

### 6.6 Segmentation seams (named, not designed)

The decomposition seams this review identifies as future parallel opportunities (H22):

- per-BTF-block methods;
- NASM inside SCCs;
- a sequential-modular → equation-oriented hybrid;
- Schur decomposition for block-angular NLP and estimation (fitting and shooting must supply separators);
- multiple-shooting segments;
- the columns of a dense finite-difference tear Jacobian.

## 7. Foundations and gates (slot 6)

| Foundation | Verdict | Scenario and evidence |
|---|---|---|
| AP-01 Separation of concerns | **Violated** | Preference and transition policy is spread across the effect-binding owner (`automatic_operations`, `automatic_backend_alternatives`), adapters (rank constants, POUNCE ladder trigger, dynamics `Auto` list), routing (rank minimum) and workflows (recycle defaults, fitting Hessian `Auto`). Adding a method profile or transition changes policy and effect owners together. |
| AP-02 Stable contracts | **Satisfied** at the adapter seam | Capability records and bridges absorb library upgrades. *Author note:* the runtime branch `report.backend == Backend::Pounce` calling `native::pounce::second_opinion_profiles` is adapter knowledge in orchestration (the binding's "consumer branching on backend objects" shape). It is a single instance, corrected by [F03](#f03)'s adapter-declared recovery profiles. The independent reviewer assessed it under AP-01; §13. |
| AP-03 Composition | **Violated** | A new selection feature propagates into about 12 hand-built `Requirements` sites (2026-10-03 F07, still open). Transitions and per-block routes are statement order and inheritance, not composable entries. |
| AP-04 Domain model and authority | **Violated** (adequacy) | The domain of method choice (admitted versus preferred, typed transitions, features with lifetimes, per-block decisions, history) has no adequate model. The rank-minimum policy is re-expressed with different filters in `select_assessed`, `complete_context`, the reduced-space adapter pick and `automatic_backend_alternatives`. |
| AP-05 Explicit structure | **Violated** | `route_decisions` records admission only. It holds no rank or class-order reasons, no sensitivity pre-filter, no alternative-list origin, no `complete_context` producer route and no executed method identity. `decision_identity` is an opaque input hash. Transitions can be discovered only by reading `solves.rs`. |
| AP-06 Local reasoning and testability | **Split** | Admission (`test_context`) and the walk are pure. Composition and escalation read the global `execution::LINKED` and need a prepared source, so escalation depends on the linked build. |
| AP-07 Execution fit | **Unresolved** | Within a solve, budgets and deadlines bound escalation. Two structural amplifications depend on unknown premises: studies replay identical failing ladders per point, which matters only if first candidates fail often; and uniform per-block methods spend Newton setup on blocks needing one solve, which depends on the block distribution and on 36c P-C4's session retention. The attempt record ([F07](#f07)) would settle the first. Observation cost is proportionate today. |
| **G9** | **Fail** | Follows AP-01, AP-03, AP-04, AP-05 and AP-06 (policy part), without averaging |

| Gate | Judgment | Evidence |
|---|---|---|
| G1 Authority | Pass, with a warning | No two independently mutable definitions of one fact. The re-expressed rank policy is the AP-04 concern. |
| G2 Semantic fidelity | **Fail (narrow)** | Typed `iterations` reads unknown where KINSOL, POUNCE, HiGHS and SCIP report it ([F05](#f05)). A class-changing SCIP fallback is labelled `native_globalization`. |
| G3 Validity | Pass | Settings, contracts and structure are refused before workers. |
| G4 Hidden behaviour | Pass | Escalations appear in strategy events. The `complete_context` producer case is documented in source but not in the record ([F12](#f12)). |
| G5 Consistency and recovery | Pass | Throw-based stops veto candidates; nothing partial is published as a solution. |
| G6 Transformation and reuse | Pass, with a forward condition | Advisory history becomes a result-affecting input and must be recorded ([RC08](#rc08)). |
| G7 Truthful capability claims | **Fail (narrow)** | The Ipopt "timing" claim; the HiGHS "native updates" claim against `clearSolver`; §18.7 and §18.6 against failure-driven SCIP escalation; §18.9 omits Uno and PETSc; §15 says diagnostics parity is absent though the catalogue exists; 10 dead mechanism values ([F12](#f12), [F10](#f10), [F08](#f08)). |
| G8 Library leverage | Pass | No generic capability is reimplemented. Unused library capability is a slot-8 gap ([F06](#f06), [F09](#f09)). |
| PS-G1 | Not applicable | No physical quantities in scope. |
| PS-G2 Well-posedness | Pass | Structural analysis precedes solving and names elements. |
| PS-G3 Numerical integrity | Pass | Publication rests on independent assessment. |
| PS-08 (MUST) | **Not met** | Recycle iteration history is not recorded ([F04](#f04)). |
| PS-09 | Partially met | The class is derived and capability recorded; within-class profiles and preference reasons are absent ([F01](#f01), [F12](#f12)). |
| PS-10 | Partially met | Typed outcomes are strong. Numerical failures do not name model elements ([F08](#f08)), and bridge throws lose evidence ([F13](#f13)). |
| PS-07 | Partially met | Conditioning diagnostics exist on request only ([F08](#f08)). |
| PS-11 | Met for warm-start and reuse | History must be recorded once added ([RC08](#rc08)). |

## 8. Library fit and ownership cost (slot 8)

The [captured-versus-available matrix](../evidence/solver-methods-and-diagnostics-2026-10-12/diagnostics-and-measurement.md#6-captured-versus-available-matrix) and §6.5 cover the inventory. The ownership judgments:

- **The project legitimately owns these, as bounded composition (PS-09):**
  - the solve plan (consuming admission, owning preference, transition and advisory adjustment);
  - the policy-stop predicate;
  - the attempt record and analytics queries;
  - index-to-model mapping in failure diagnosis;
  - optionally, Wegstein and dominant-eigenvalue recycle extrapolation, which no pinned library offers. Its main justification is parity with the reference tools, since Anderson subsumes it mathematically.
- **Libraries own** every Newton step, line search, trust region, factorization, barrier controller, inertia correction, Anderson or Broyden update and integrator. None of the recommendations moves those into project code.
- **Library selection rules are complementary, not substitutes.** They do not span adapters or classes, and their thresholds are uncalibrated upstream. They can serve *inside* an admitted candidate where they fit:
  - Uno `auto`, which switches to interior point at about 2000 variables plus constraints;
  - HiGHS `choose`;
  - POUNCE `solver_selection`.
- **Integration cost.**
  - Most gaps are *binding* work on already-pinned libraries: PETSc bridge types and TAO, IDAS getters, the Ipopt C++ statistics, SCIP datatree, KINSOL difference-quotient Jv.
  - New native acquisitions:
    - HSL: host-local and checksummed, non-redistributable;
    - SPQR: an extended SuiteSparse build plus an owned FFI, already proposed under R36-01;
    - CVODES: a feature flag.
  - Upgrading SUNDIALS to 7.3 or later (fixed-point damping and depth callbacks) or 7.8 (in-process logger callbacks) means leaving `sundials-sys` 0.6.2's bundled build. That is optional; callback reconstruction covers the fixed-point history.
  - Upgrading Uno past the pin breaks the bridge's `loose_*` options.

## 9. Alternatives and tradeoffs (slot 9)

| Alternative | Scenarios served and change locality | Machinery | Selection and revisit condition |
|---|---|---|---|
| Current baseline | Admission is local; preference, transition and per-block choice are not | Low | Fails the three-layer requirement |
| **Solve plan over existing admission** (recommended) | All four change scenarios become one declaration each; policy testable without solvers | A new pure contract; migration of about 12 construction sites and the sub-selectors | Revisit if the profile catalogue proves small enough for ranks per profile |
| Per-class, per-profile rank tables | Order without reasons; no cause-typed transitions or features; layer 3 would mutate ranks, a global mutable authority | Lowest | Rejected |
| Library-owned selection (Uno `auto`, HiGHS `choose`) | Inside one adapter only | None | Complementary |
| Learned or ML selector deciding methods | Would admit methods implicitly, contrary to the layer-3 constraint, and needs data not yet recorded | High | Not recommended. The attempt record keeps an advisory variant possible |
| Reference practice (Aspen, Pyomo, OpenModelica) | Per-class defaults; staged, observation-keyed switching; rule tables from the error/tolerance trajectory; first-class nested versus simultaneous specification choice | — | The recommended plan generalizes this practice. Its rules seed the transition table |

## 10. Verification and evidence limits (slot 10)

| Claim | Label | Basis | Gap |
|---|---|---|---|
| Selection is class plus constant rank, with no structural features | Interface-checked | routing.rs, facts.rs, execution adapters (A1; principal; independent) | — |
| Every block gets one backend; `oracle_facts` affine rows false | Interface-checked | blocks.rs, routing.rs (A3; principal) | — |
| SCIP is a failure-driven SmoothNlp fallback | Interface-checked | solves.rs, staged/driver.rs, execution/scip.rs (A1; principal; independent §5) | Independent probe 1 would show it executed |
| Recycle history absent; defaults weakest | Interface-checked | requests.rs, kinsol.rs, recycle.rs (principal) | — |
| Ipopt statistics and timing reachable through the bridge; throws lose them | Interface-checked | Bridge source; Ipopt headers and `IpIpoptApplication.cpp` (B1) | Independent probe 2 |
| Library methods pinned but unbound | Interface-checked | Pinned headers and adapter greps (B2, B5a) | Behaviour of each unbound method in this build: probes in B5a |
| Diagnostics catalogue present, not auto-triggered | Interface-checked | `FindingKind`, profile directory (B5b; principal); A2 §7 | — |
| HiGHS discards its basis | Interface-checked | highs.rs (B4; independent) | Magnitude: B4 probe 2 |
| Method benefits (per-block, Anderson defaults, Schur routing, ESDIRK under events and so on) | Proposed | Mathematical rationale ([method landscape](../evidence/solver-methods-and-diagnostics-2026-10-12/method-landscape-and-selection.md)), corrected by the [challenge](../evidence/solver-methods-and-diagnostics-2026-10-12/numerical-methods-challenge.md) | No measurement. By maintainer direction, adoption rests on mathematical rationale |

**Unresolved premises that could change a recommendation:**

- **How often first candidates fail in realistic studies.** Bears on AP-07 and the priority of layer 3. Settled by the attempt record.
- **The BTF block-size and affinity distribution of real flowsheets.** Bears on the materiality of [F02](#f02). Settled by structural statistics in the attempt record.
- **Whether the maintainer qualifies for HSL academic licensing.** Bears on whether MA57 and MA97 join MA27.
- **Whether parity with the reference tools' Wegstein default is wanted.**

## 11. Rule impacts and disposition (slot 11)

The recommendations would change the rules below. **None takes effect until the operator confirms it in plan creation.** Routes follow the binding:

- a short ADR for a new backend binding or kernel contract within an accepted decision;
- an ADR plus design review for D1–D14, crate and governance changes;
- a `design:` PR with a revision row for blueprint text;
- the register for deferred decisions.

| ID | Rule and where recorded | Proposed change | Findings | If kept |
|---|---|---|---|---|
| <a id="rc01"></a>RC01 | §18.7 *Selection* ("SCIP … automatic … wherever the others are ineligible"), §18.6 ("a backend change requires named selected alternatives"), §18.7 supplement ("Capability reconsideration is limited to newly established … incompatibility"), against the code's failure-driven same-class escalation | Transitions are named plan rules with stated rationale. Class-crossing edges are allowed when layer 1 admitted the target. SCIP and multistart after a local failure only under `certify`, a declared global permission, or a bounded diagnostic | F03 | Amend the contract to declare same-class failure escalation, including SCIP, and record its reason |
| <a id="rc02"></a>RC02 | §18.7 capability model: `automatic()` rank and `automatic_classes` as the preference mechanism; adapter-resident selection defaults (dynamics `Auto` list, DP-14) | Preference moves to the solve plan (rules over features); adapters declare suitability facts and method profiles | F01, F02 | Only ranks; per-block and feature-conditioned choice impossible |
| <a id="rc03"></a>RC03 | The `SolveReport.metrics` contract ("Decisions read `evidence`, never these keys") and §18.6 report text | Decisions read only *typed observation families* with declared semantics. Raw metrics stay diagnostic. Adapters must promote decision-relevant statistics | F05, F03, F13 | Layer 2 limited to termination categories |
| <a id="rc04"></a>RC04 | Register R-48 (Ipopt timing excluded until the C API exposes it) | Retire: the C++ bridge reads typed `TimingStats` without parsing | F06, F07 | No Ipopt phase time; the G7 claim must be corrected |
| <a id="rc05"></a>RC05 | ADR-0108 and register R-34 (HSL excluded pending a licence route) | Admit HSL MA27 and MC19 from the Coin-HSL Archive (free, personal use) as a host-local checksummed native input; MA57 and MA97 if academic eligibility holds | F09 | Ipopt keeps MUMPS, SPRAL and Pardiso, without MC19 equilibration scaling |
| <a id="rc06"></a>RC06 | Register R-47 trigger ("a measured case where the KINSOL session cache dominates") | Replace the benchmark trigger with a capability trigger: diffsol-nl gains recoverable trials, sign constraints and cancellation | — | Exclusion stands on a benchmark-gated trigger |
| <a id="rc07"></a>RC07 | §17.4 (KINSOL owns fixed-point iteration; `RecycleRequest` defaults; tear route) and §17.1 (typed backend settings must belong to every block's route; one settings value) | Recycle method, acceleration, damping and tear objective become recorded plan outputs. Per-block routes and settings allowed. Owned Wegstein loop allowed as bounded composition | F02, F04 | PS-08 history is still achievable; per-block choice remains impossible |
| <a id="rc08"></a>RC08 | Reuse and identity contracts (§5, §14, `run_lineage`, PS-11 application) | Record the advisory-history snapshot identity as a result-affecting input; declare the plan determinism class (same inputs plus same snapshot gives the same plan); freeze one snapshot per study | F01, F07 | Layer 3 would make results depend on unrecorded state (a PS-11/DP-09 MUST gap) |
| <a id="rc09"></a>RC09 | Measurement as qualification-only evidence (`docs/dev/validation-assessment.md` *Case measurements*; process-wide `ResourceReport`) | Attempt wall time, phase cost and outcome become persisted product data, distinct from qualification. No timing gate is introduced | F07 | Measurement remains an investigation campaign |
| <a id="rc10"></a>RC10 | §15 introduction ("IDAES `DiagnosticsToolbox` parity is not a current capability … no named … check catalogue") | Describe the existing catalogue and its remaining gaps | F08 | §15 keeps understating the implementation (G7) |
| <a id="rc11"></a>RC11 | §18.9 capability matrix and capability strings (`execution/ipopt.rs` "timing"; `execution/highs.rs` "native coefficient/bound updates") | Add Uno and PETSc rows; make the strings match executed behaviour | F10, F12 | G7 fails |
| <a id="rc12"></a>RC12 | ADR-0157 PETSc admission scope (NEWTONTR without bounds, NASM domain-safe, TSPSEUDO) | Admit VI (declared complementarity only), QN, NGMRES, Anderson, the NLEQ-ERR line search, NEWTONAL, TAO BRGN, BNTR and TRON, and convergence history, each with stated admission conditions | F09 | Square two-sided boxes and least squares lack library methods |
| <a id="rc13"></a>RC13 | §17.5 homotopy policy ("growth after an accepted step", fixed factor) | Step control adapts to observed solver effort (iteration targets), as in IDAES and OpenModelica | F03 | Geometric growth remains |

No rule change is needed for [F06](#f06)'s statistics capture, [F10](#f10)'s HiGHS fix, [F11](#f11)'s separators, or [F13](#f13)'s cooperative stops. Each is a refactor within existing contracts.

**Disposition.** Not yet scheduled. The proposed owner is a new plan created through `plan-creation`. It records each finding as `finding | scenario | disposition | decision/work owner | evidence or revisit trigger`, presents RC01–RC13 for operator confirmation, and orders work by §5's prerequisites. It coordinates with Plan 36 without reopening it:

- 36c P-C4 owns the block-sequence operation that [F02](#f02) builds on;
- 36b P-B6 owns the telemetry tap that §6.3's tiers respect;
- Plan 36 R36-01 owns the SPQR rank decision.

**Relation to the 2026-10-03 acceleration review.** Its findings F01, F04, F05 and C12 are resolved. F02, F03, F06, F08–F10 and F12–F15 are partial; F07, F11 and F17 are open (status from [A1 §5](../evidence/solver-methods-and-diagnostics-2026-10-12/selection-and-escalation-map.md)). This review's findings subsume the still-open ones that fall in its scope:

- F07 (hand-built `Requirements`) → [F01](#f01);
- F13 (budget, accuracy tier, observation) → [F05](#f05) and [F07](#f07);
- F15 (structure shaping the main solve) → [F02](#f02) and [F11](#f11);
- F14 (method choice within an adapter) → [F01](#f01).

The new plan should take over the disposition of those findings, so that one owner remains.

## 12. Decision and priorities (slot 12)

**Behavioural/semantic adequacy: Revise (narrow).**
- PS-08 is not met: there is no recycle iteration history.
- G7 and G2 fail narrowly.
- PS-G2 and PS-G3 pass on inspected paths, because publication rests on independent original assessment.

**Architectural fitness: Fails G9.** AP-01, AP-03, AP-04 and AP-05 are violated and AP-06 is split for the representative changes the maintainer's target makes routine: methods, features, transitions, per-block choice and history.

**Overall: Revise.**
- **Strongest evidence:** the selection path (class → constant rank → fixed list → same-class append) read end to end by two independent reviewers; the uniform block backend with `oracle_facts` hard-coding affinity away; the absent recycle history; the bridges' throw-based stops.
- **Main uncertainty:** the frequency of first-candidate failure and the BTF block distribution in realistic flowsheets. These affect AP-07 and the priority of per-block work, not the verdict.

**Priorities.** These are ordered by consequence; §5 gives the prerequisite order.

| Priority | Change | Findings | Acceptance evidence | Owner |
|---|---|---|---|---|
| 1 | Recycle iteration history, contraction diagnosis, damping and Anderson defaults, weighted tears | F04 | Stored per-iteration history and contraction on a non-converging recycle; PS-08 audit answered from the record | New plan |
| 1 | Remove SCIP-after-local-failure from the default transitions; correct §18.6/§18.7, §18.9, §15 and capability strings; typed iterations everywhere; HiGHS basis retained | F03 (part), F12, F10, F05 (G2), F08 (§15) | Contract and record agree with execution; probe 1 shows no automatic SCIP after a local failure | New plan |
| 2 | Typed observation families and derived diagnoses; cooperative policy stop in the Ipopt, Uno and POUNCE bridges; library statistics captured | F05, F13, F06 | No decision reads raw metrics; a policy stop retains the iterate and statistics (probe 2) | New plan |
| 2 | The solve plan with features and lifetimes, replacing ranks and sub-selector defaults | F01 | Change scenarios S01–S05 each need one declaration; plan tests run without native solvers | New plan |
| 3 | Cause-typed transition table, including reformulating edges | F03 | Trace names the cause and rule for each transition | New plan |
| 3 | Per-block method choice over the 36c block sequence | F02 | Affine blocks take zero Newton iterations; 1×1 blocks need no native session | New plan, with 36c |
| 3 | Bind pinned library methods (PETSc and TAO, KINSOL difference-quotient Jv, Brent, Ipopt profiles, IDAS getters, SCIP statistics), then HSL MA27/MC19 and SPQR | F09 | Each admitted with stated conditions; capability matrix updated | New plan; R36-01 for SPQR |
| 3 | Attempt record with time, environment and structure key; study summaries; convergence-effort regression | F07 | Queries over one relation answer "where did time go" and "what regressed" | New plan |
| 3 | On-failure diagnosis tier and replay snapshot | F08 | An exhausted solve yields persisted findings naming model elements | New plan |
| 4 | Advisory history layer with a recorded snapshot | F01 (layer 3), F07 | Reproducible plan under a recorded snapshot; removing the snapshot restores the layer-1/2 plan | New plan, after the RC08 confirmation |
| 4 | Separators for fitting and shooting; structure-driven Schur routing | F11 | Block-angular problems routed to Schur-capable methods a priori | New plan |

**Next consequential decision:** operator confirmation of RC01–RC13 in plan creation, especially:

- RC01, which changes failure-driven escalation semantics;
- RC03, which allows typed observations to drive decisions;
- RC08, which makes history a recorded result input;
- RC05, which admits HSL.

**Inputs the maintainer can supply:** HSL academic eligibility, and whether parity with the reference tools' Wegstein default is wanted.

## 13. Reconciliation with the independent assessment and the challenge

The [independent assessment](../evidence/solver-methods-and-diagnostics-2026-10-12/independent-design-assessment.md) was produced without the author's conclusions. Its verdicts and findings agree in substance:

| Independent | This review |
|---|---|
| IR-01 | F01 |
| IR-02 | F03 |
| IR-03 | F01 features, F02, F11 |
| IR-04 | F05, F06 |
| IR-05 | F13 |
| IR-06 | F07 |
| IR-07 | F08 |
| IR-08 | F04 |
| IR-09 | F12 |
| IR-10 | F10 |

Its rule impacts IR-RC01 to IR-RC09 map into RC01–RC09.

**Differences, and how they are resolved:**

1. **G8.** The author's draft judged G8 failed narrowly because of unused library capability. The independent reviewer judged G8 passed, because G8 fails only on bespoke reimplementation (core principles, G8 definition). **Adopted the independent judgment.** Unused capability is reported as a slot-8 library-fit gap.
2. **AP-07.** The author's draft treated uniform per-block methods as an established execution-fit defect. The independent reviewer left AP-07 unresolved. **Adopted *unresolved*.** Materiality depends on the block distribution and on 36c P-C4's session retention; the method-choice defect itself stands under AP-03 and AP-04.
3. **AP-02.** The independent reviewer judged AP-02 satisfied. The author notes one instance of adapter knowledge in orchestration: the runtime's POUNCE-only ladder branch. **Both preserved.** The instance is corrected by F03 regardless, and the G9 verdict is unaffected.
4. **SCIP escalation.** B4 believed SCIP was never a failure-driven fallback; A1, the principal and the independent reviewer traced it in source. **Settled for A1.** B4's mathematical argument against it shapes RC01.
5. **The independent reviewer's forward condition on layer 3** (history as a result-affecting input under PS-11) is adopted as RC08.

The [numerical-methods challenge](../evidence/solver-methods-and-diagnostics-2026-10-12/numerical-methods-challenge.md) corrected the author's mathematical analysis before publication. The [method landscape](../evidence/solver-methods-and-diagnostics-2026-10-12/method-landscape-and-selection.md) and F02, F03 and F04 incorporate the corrections:

- the definition of the Deuflhard monitor;
- the VI pairing condition;
- the conditions under which Ψtc and homotopy converge;
- the fixed-point selection rule;
- the conditions for JFNK on opaque maps;
- feasible path versus reduced SQP;
- δw versus δc;
- the IDAS unlocated-discontinuity diagnosis;
- the convexity feature lifetime;
- the limits of tearing.
