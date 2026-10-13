> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md): the detailed solver target that the principal review summarizes in its findings F09–F13 and rule impact RC10. Standard: Core 3.4, Efficient Architecture 1.0, Process Simulator 1.5. Baseline: `main` at `6c368c88a`. Revised after the omission/preservation (X1) and numerical-methods (X2) challenges.

# Solver target on the post-efficiency codebase

This is a supporting document. The [principal review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md) owns the combined decision, the finding identifiers and the rule impacts. The solver findings are its F09–F13.

This document restates the target of the [2026-10-12 solver-methods review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md), to that review's standard, for the codebase the principal's target produces. It incorporates the [numerical-methods challenge](x2-numerical-methods-challenge.md) (X2). It carries no competing verdict.

The prior diagnosis is current: routing, composition, the walker, bridge stops, recycle defaults, the HiGHS basis and observation are unchanged at `6c368c88a`. The target changes because five premises moved:

1. **Parallel execution** is in scope and is to be designed.
2. **No HSL.** RC05 is withdrawn and the KKT story changes.
3. **The efficiency target supplies four foundations:**
   - a structure plane (principal F06);
   - lanes with result-neutral allocation reuse (F07);
   - shared native resources and placement (F08);
   - one typed attempt record in a journal and record plane (F04, F11).
4. **A determinism-class owner** (principal §6.5) governs every context-dependent choice.
5. **X2 established library facts at the pins that change the method story** (§1).

Everything here is *Proposed* unless labelled. Library facts are *Interface-checked* against pinned sources in R4, R5 and X2. No probes or measurements were run.

## 1. Library facts that shape the target (from X2, against pinned sources)

| # | Fact | Consequence |
|---|---|---|
| A | Ipopt 3.14.20 repeats the linear solver's symbolic analysis on every `OptimizeTNLP`/`ReOptimizeTNLP`, for MUMPS, SPRAL and MKL Pardiso alike. `warm_start_same_structure` keeps only the adapter's index maps and allocations | No symbolic reuse across Ipopt solves. It corrects solver evidence B1 and R4 Part B |
| B | SUNDIALS 7.1.1 repeats KLU symbolic analysis at every `KINSol` and after every `IDAReInit` | Symbolic reuse holds only within one solve or segment. Retaining it needs a project-owned `SUNLinearSolver`, following the precedent in `dynamics/linear.rs`. It corrects B2, B3 and R4 Part B |
| C | Retained KINSOL sessions reuse the previous point's numeric factor (`NoInitSetup`), and the inner-session cache is thread-local | Iteration paths depend on thread history: a DP-11 concern today |
| D | POUNCE batches build a fresh application per instance and share no structure | Batches and lane-retained sessions are alternatives, not complements |
| E | pounce `block_solve` is presolve tooling: unscaled absolute tolerances; aborts on any evaluation failure | Not a qualified block solver |
| F | FERAL arms static-pivot perturbation by default | Inertia is trustworthy only when `n_tiny` = 0 |
| G | KINSOL Newton keeps the last accepted iterate on a callback failure. Fixed-point mode tests the accelerated step, not ‖G(u) − u‖ | Stop fidelity is better than assumed. Recycle history must come from the map callback |
| H | An Ipopt user stop during restoration returns the point where restoration began | Restoration slacks are lost to a stop |
| I | `warm_start_same_structure` does not recompute Ipopt's fixed-variable map | Sessions retained across lb = ub edits must keep today's freeze of bounds at creation |

## 2. What the prior target keeps, and what changes

| Prior element (2026-10-12 §6) | Status here |
|---|---|
| Solve plan as a pure consumer of `routing::admit`, owning preference, transitions and advice | **Kept.** Features come from the structure plane, with declared lifetimes |
| Typed selection features with lifetimes | **Kept and corrected.** Form facts are keyed by body, pattern facts by pattern, and near-singularity is a value fact |
| Observation families per method family | **Kept.** They are the per-adapter part of the attempt record. Adds `inertia_mode`, FERAL `n_tiny`, and map-callback recycle history |
| Cooperative policy stop | **Generalized** to one `StopRequest` with corrected fidelities and explicit start-transfer rules |
| Compact attempt relation | **Kept and placed** in the journal and record plane |
| History store | **Changed** to a derived aggregate. Censoring is corrected; records are conditioned on plan position and start kind; replay uses the recorded permutation |
| Diagnosis tier and replay snapshot | **Kept.** Replay comes from the journal, so interactive failures are replayable |
| Methods to bind | **Amended:** no HSL; shared native resources; `block_solve` caveat; Uno size rule dropped |
| Segmentation seams | **Designed**, with equivalence conditions (§3.7) |
| Per-block choice over the 36c sequence | **Re-based** on lanes, the structure plane and corrected per-block rules |

## 3. <a id="stage-columns"></a>Numerical stage columns (profile slot 5): target

| Stage | Formulation policy | Derivative source and order | Scaling | Class · capability | Status → outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Admission and plan | Declared forms, guards, MPCC realizations | Order demanded by the plan | Resolved nominals | Class from facts. Profile by preference rules over features. Method choice is context-independent; shared resources decide placement | Typed refusals; plan recorded with rules, features, advice snapshot, placement and determinism class | — |
| Square, per block | As declared per block | Exact Jacobian; difference-quotient Jv where opaque | Global nominals for acceptance; solver-internal scaling may differ | Per block: closed form; Brent after a verified sign change; certified root with branch by the **authored** rule; KINSOL Dense (scaled, recoverable) for small blocks; KINSOL/KLU for large SCCs; TR/LM for near-singular starts; VI only with declared complementarity | Typed per-block causes; cause-typed escalation | Per-block commit on the **global row-wise criterion**. Global verification of the assembled point; simultaneous Newton polish **only if it fails** |
| Recycle (SCC or tear) | Causal maps; G must be a function of x | Opaque map, or exact if equation-oriented | Port conversions | Damped fixed point, then Anderson after a delay, with a project restart rule (KINSOL). PETSc QN/NGMRES placed on a serial or process lane | History from the map callback: ‖G(x) − x‖ per tear, Err/Tol, oscillation. Contraction from damped windows (PS-08) | Original residual validator |
| NLP | ℓ1 route by declared requirement | Exact, or L-BFGS | `nlp_scaling_method=user-scaling` from PS-07 nominals compensates the loss of MC19. SPRAL `ruiz`/`mc64` and MUMPS scaling act on the linear system only | Ipopt with a feature-chosen KKT solver (MUMPS default, call-serialized placement; SPRAL; Pardiso only for convex problems or with `neg_curv_test_*`). POUNCE/FERAL. Uno as a second-opinion profile | Typed. Cooperative stop; a stop during restoration loses the slacks | Original quality; KKT-point analysis; inertia read per `inertia_mode` |
| LP/MILP/QP; cones; MINLP | Proven affine or degree 2; declared cones; factorable | Coefficients | Native | HiGHS with retained basis (simplex); Clarabel; SCIP only under certify or global permission | Typed with certificates and gap | Certificates in original coordinates |
| Dynamics | Index-1, fixed mass | Exact; forward or adjoint | Absolute and relative tolerances | Diffsol or IDAS by plan rule. Shooting windows each own a context and checkpoints | Typed; IDAS step-health getters | Trajectory acceptance |
| Fitting | Steady and transient experiments; the objective is a pure function of θ | Algorithm and derivative mode chosen **jointly**. Gauss–Newton/LM needs forward sensitivities. An adjoint pairs with gradient-only methods when p is large. Events force forward | — | One NLP runner. One IDAS memory per transient experiment, reinitialized with fresh `IDACalcIC`, s₀ and `IDAAdjReInit`. Diffsol for event-crossing fits (problem and linear solver retained) | Typed | Covariance validity stated |

## 4. Target design

### 4.1 The method-choice domain model

| Concept | Content | Lifetime, owner |
|---|---|---|
| `MethodProfile` | Library and version, typed options, and a capability declaration | Authored policy data, versioned |
| Features | Definition, pattern and value features (detail below) | Definition and pattern features are Salsa products; value features are computed at binding |
| Lane context | Lane width, placement, process kind, budget | Per study or solve |
| Determinism class | Bitwise, tolerance or statistical | Declared per study or solve (principal §6.5) |
| Admission (layer 1, hard) | The existing `routing::admit`, unchanged. Shared resources affect placement, not admission, except where a declared determinism class forbids a placement | Pure; memoizable |
| Preference (layer 1, order) | Authored rules with mathematical rationale (§4.2). Each position records the rule that placed it | Pattern lifetime, refined by value features |
| Plan | Steps, each with a profile, options and an observer predicate. Cause-typed transitions `(step, Cause) → (step′, StartTranslation, rationale)`. Finite caps | Per solve; recorded in full |
| `Cause` | Terminal, observer, verification or external (detail below) | Per attempt |
| Advice (layer 3) | Permute within one tier; tune declared-tunable options within declared ranges. The admitted set is unchanged. Never applied to explicit selection | Recorded: snapshot, permutation, tunings, effective plan |

**The capability declaration covers:**
- problem classes;
- derivative orders required;
- bounds, VI and integrality support;
- warm kinds accepted and produced;
- **stop fidelity**;
- **statistics fidelity**: typed, callback-only, text-only or absent;
- **shared native resources and access modes**;
- **width sensitivity**, with inner width fixed per profile;
- **reuse-after-terminal rules**: after which outcomes a session may be reused;
- `inertia_mode`, for KKT profiles.

**Features:**

| Lifetime | Features |
|---|---|
| Definition | per-block algebraic form (keyed by body); smoothness and guards; derivative availability; provider envelopes |
| Pattern | class; DOF; DM parts; BTF blocks and levels; largest SCC; coupling class; sparsity and fill; bounded and integer sets; DAE index; block-angular separators |
| Value | bound activity; FBBT enclosures; start provenance and residual; scaling magnitudes; scaled condition estimate after the first factorization ("near-singular"); value-zero coefficients that contradict the pattern facts |

**`Cause` kinds:**
- **terminal:** from the status map;
- **observer:** stagnation, inertia corrections, divergence, contraction failure;
- **verification:** unscaled residual, bound, domain, closure, **branch**;
- **external:** cancel, budget.

**What consumers stop deciding for themselves.** These defaults become plan outputs:
- dynamics `Auto`;
- Hessian `Auto` and its fitting variant;
- the joint fitting algorithm and derivative mode;
- presolve `Auto`;
- recycle method and acceleration;
- block backend inheritance;
- the KINSOL linear solver;
- the HiGHS method;
- the Ipopt linear solver, chosen by features with a context-independent default.

The roughly twelve hand-built `Requirements` sites become one plan builder per representation.

### 4.2 Preference rules (corrected)

**Per block:**

| Block shape | Rule |
|---|---|
| Affine | Exactly one linear solve. Needs value-lifetime nonsingularity and a relative pivot check at current upstream values; then a residual check or one refinement |
| 1×1, bracketed | Brent, only after a verified sign change on an interval where f is continuous and evaluable. Accept on the re-evaluated scaled residual |
| 1×1 polynomial | Certified isolation, with the branch from the authored phase or branch rule. Bounds only exclude roots. Near a double root (critical point, spinodal) the root count is ill-conditioned, so this is flagged |
| Small dense | KINSOL Dense with scaling and recoverable evaluation errors, or argmin `Newton`/`TrustRegion`. `block_solve` only as a cheap first try with nominal-scaled callbacks and fallback |
| Large smooth SCC | Newton with KLU. Krylov only with a preconditioner over a partition **inside** the SCC (unit or subsystem block Jacobi, ILU) or NASM; BTF preconditioning is vacuous within one SCC |
| Near-singular (a value fact) | TR/LM regularization and rescaling. NLEQ-ERR is affine-covariant damping, not a remedy for near-singularity |
| Two-sided bounds with declared complementarity | VI; NLEQ-ERR only with SSLS. Most two-sided bounds are domain guards, handled by truncation or transforms first |

**KKT linear solver:**
- Chosen from features with one context-independent default.
- Pardiso only for convex problems, or with the curvature test.
- FERAL inertia only when `n_tiny` = 0.
- A context-dependent choice only as a declared study profile, recorded per point.

**NLP:**
- Schur-capable methods for block-angular structure.
- Uno as a second-opinion profile with different globalization. The bridge allows only L-BFGS with convex HiGHS QP subproblems, so no size-based preference.

**Other classes:**
- **LP sequences:** simplex with a retained basis.
- **Dynamics:** one-step versus BDF by steps per segment and event density.
- **Estimation:** joint algorithm and derivative-mode rule (stage columns).

### 4.3 Transitions and escalation

Rows map a typed `Cause` to admitted successors, a start translation and a stated rationale. The corrections from the prior challenge §9 still apply.

- **Class-crossing edges** only when layer 1 admitted the target.
- **SCIP or multistart after a local failure** only under `certify`, a declared global permission, or a bounded diagnostic.
- **Reformulating edges** carry provenance.
- **Request permissions** are inputs.
- **Per-block escalation** is cause-typed:
  - a stronger block method;
  - merging with neighbours;
  - the simultaneous system.

  A bounded-least-squares or VI point is never committed as a root.
- **A divergence-caused successor** starts from the **best-merit iterate**, which the observer tracks, or from the original start. Never from the last iterate.

### 4.4 Observation, the attempt record and the cooperative stop

**Observation families:**
- **Interior-point NLP:** inf_pr, inf_du, μ, δw, δc, α, restoration share, `info_string`, `inertia_mode`. With the curvature test on, δw records curvature-test failures and inertia is not a signal.
- **Newton root:** ‖F‖, step, backtracks, linear iterations.
- **Fixed point:** from the map callback: ‖G(x) − x‖ per tear, Err/Tol, oscillation sign.
- **Integrator step health.**
- **Branch-and-bound.**
- **LP/QP:** iterations by method, basis validity, executed method.
- **KKT factorization:** inertia, `n_tiny`, pivots, fill, κ₁. FERAL reports these typed; MUMPS, SPRAL and Pardiso inside Ipopt are text-only and recorded as absent.

History and feature bins are partitioned by linear solver and inertia mode.

**Tiers:**
- **T0:** always on, and always journaled.
- **T1:** bounded trajectory, with drop accounting.
- **T2:** on failure.
- **T3:** on request.

Layer-2 decisions read the in-memory evidence.

**`StopRequest`.** One contract for cancel, deadline, budget and policy. Each adapter declares its fidelity and **what transfers**.

| Library | Iterate after a cooperative stop | What transfers |
|---|---|---|
| Ipopt | Last accepted filter iterate. **During restoration: the point where restoration began** (slacks lost) | Primal after an in-bounds and domain screen. Multipliers within the family. Barrier state only to an IPM with matching μ |
| POUNCE, Uno | Current iterate | As for Ipopt |
| KINSOL, Newton or line search | **Last accepted iterate** (better than "uncertain") | Primal |
| KINSOL fixed point; PETSc QN, NGMRES, Anderson | Current, possibly accelerated, iterate with G unevaluated | Screened and projected; otherwise fall back to the last iterate whose G succeeded |
| PETSc SNES | User converged or diverged reason; solution vector | Primal; trust-region radius within the same SNES |
| IDAS, Diffsol | Accepted step at `tret`. Resuming needs `IDAReInit` at order 1, so the resumed trajectory differs within tolerance and is recorded as such | State |
| HiGHS | Simplex: the **basis**. IPM without crossover: an interior point. MIP: the incumbent plus the bound | Basis to simplex only |
| SCIP | **Resumable state**, not a start proposal; the incumbent is feasible within tolerance | Incumbent |
| Clarabel | The HSDE iterate scaled by 1/τ is meaningless when τ → 0. Clarabel has no warm start | A primal hint for other families only |

**Two rules:**
- The Ipopt and Uno bridges stop through these mechanisms instead of throwing.
- A policy-stopped iterate is a **start proposal**, never a result (PS-10).

### 4.5 History layer (layer 3)

- **Derivation.** A DataFusion aggregate over the journaled and published attempt log. It is partitioned by solver build, environment, linear solver and inertia mode.
- **Keys.** A structure-only key plus coarse feature bins.
- **Conditioning.** Records are conditioned on **plan position and start kind**: cold, warm or predictor. Successors are observed only on harder instances.
- **Censoring.** Only budget exhaustion is right-censored. Terminal failures are outcomes.
- **Scoring.** PAR-k only with equal budgets; otherwise a censoring-aware estimator. Work units are comparable within a method family; across families, wall time within one environment partition.
- **Small samples.** Below a minimum sample per bin, differences are ties and the layer-1 order stands.
- **Reproducibility.**
  - Replay rests on the **recorded permutation and tunings**, not on recomputing the aggregate, whose summation order is not fixed.
  - Near-ties fall back to the layer-1 order by a tolerance rule.
  - A snapshot is pinned for a study's whole life, resumption included. Retention keeps the referenced log prefix or a materialized advice table.
  - An interactive session freezes its own snapshot.

### 4.6 Diagnosis and replay

**A bounded T2 tier** runs at the final or best iterate when a plan exhausts or a limit is hit. It reuses the existing catalogue (25 `FindingKind` rules, conditioning certificates, KKT-point analysis, ℓ1 and deletion-filter explanations), plus:
- KLU `rcond`/`condest` (already computed, not exported);
- FERAL κ₁ and `n_tiny`;
- a bounded SVD;
- evaluation-error and envelope locations;
- IDAS step-limiting states.

It maps native indices to model elements and never changes an outcome. A diagnostic stop waits for Ipopt restoration to exit, so the ℓ1 slacks needed for attribution are not lost.

**Replay** comes from the **journal**: inputs, recorded plan, advice snapshot, start, placement and determinism context. It adds native model export where available: HiGHS `writeModel`, SCIP `SCIPwriteOrigProblem`.

### 4.7 Parallel decomposition (designed, with equivalence conditions)

| Level | Unit | Library basis | Equivalence conditions |
|---|---|---|---|
| Case | Lanes executing **declared chains** or independent points | Per-thread or per-context instances. POUNCE batches as an alternative to lane sessions | Point results independent of lane count within the determinism class. Chains derived from the study definition |
| BTF wavefront | Independent blocks of one level | Block-owned KINSOL sessions (not thread-cached) | Level-synchronous semantics, also when sequential. Fixed commit order. Deterministic per-block budgets |
| Shooting windows | One window per team thread | An IDAS or Diffsol instance per context; adjoint checkpoints owned per window | Static assignment until context migration is settled. Fixed reduction order |
| Fit experiments | One experiment per team thread | Retained per-experiment workers; one IDAS memory per transient experiment | Fixed summation order for the objective, gradient and JᵀJ |
| Finite-difference colour groups | One colour group per task | Library colorings (PETSc, POUNCE); project-owned evaluation | Only where exact derivatives are unavailable |
| Schur | POUNCE Schur KKT, multi-RHS | Library-owned | — |
| Library internal | Inner width **fixed per profile and recorded** | HiGHS `threads`, SCIP `parallel/*`, SPRAL OpenMP, Pardiso MKL, FERAL `parallel`, faer `Par` | Not derived from lane context. Width-1 teams inside parallel structure tasks |
| Process lanes (durable or not) | One child process per lane | PETSc and IBEX (exclusive resources); crash-prone families | They parallelize **independent** PETSc problems. NASM, ASPIN and field-split subdomains of one solve stay serial without MPI |

**Prerequisite.** Thread-local admission and native state must be passed explicitly into structure tasks: `COMPUTE`/`ORIGINAL`, the KINSOL inner-session cache, POUNCE `ADMITTED`, pse-math TLS.

### 4.8 The KKT story without HSL

**Available:**
- MUMPS: call-serialized placement; usable inertia;
- SPRAL: usable inertia; `ruiz`/`mc64` linear-system scaling;
- MKL Pardiso: convex problems only, or with the curvature test;
- FERAL: typed statistics; inertia when `n_tiny` = 0;
- QDLDL or Pardiso inside Clarabel;
- faer.

**Lost:**
- MC19 equilibration. It survives only as linear-system scaling. The NLP-scaling role is compensated only by `nlp_scaling_method=user-scaling` from PS-07 nominals, which makes the **nominal contract load-bearing**.
- MA57 in POUNCE.
- The small-KKT, low-overhead regime.
- **IDAES iteration-path parity.** IDAES 2.13 defaults to MA57/MA27, so parity is solution-level only.

**Cost notes:**
- Per-solve symbolic analysis is repeated for every Ipopt solver (fact A), so SPRAL's analysis cost is paid on every re-solve.
- `dependency_detector=mumps` re-enters the MUMPS mutex even under SPRAL.
- SCIP's nested Ipopt inherits the configured solver.

**Unbuilt options:**
- MUMPS inside PETSc (inertia via `MatGetInertia`);
- SuiteSparse SPQR for rank (Plan 36 R36-01);
- a Uno rebuild against SPRAL (unverified).

## 5. Recycles and PS-08 (F10)

- **History from the map callback.** Record ‖G(x) − x‖ per tear, normalized by Err/Tol, plus the oscillation sign. Under Anderson, KINSOL's norm is the accelerated step (fact G), so it is neither a contraction estimate nor a certificate. The original residual validator stays the acceptance.
- **Contraction diagnosis.** It is meaningful only over damped or plain windows: the Anderson delay window or a probe window. After Anderson starts, diagnose by stagnation over m iterations.
- **Default.** Damped fixed point, then Anderson(m) after a delay, plus a **project-owned restart rule**. On a G failure or a residual increase, re-enter from the last good iterate with the Anderson memory cleared and stronger damping (bounded composition, PS-09). KINSOL 7.1.1 has no such fallback, and Anderson iterates can leave unit domains.
- **Precondition.** G must be a function of x. Units inside G must not warm-start beyond the noise floor (premise 8).
- **Successor by cause and representation:**
  - oscillation → more damping;
  - slow → deeper Anderson, then simultaneous;
  - noise floor → tighter inner tolerances;
  - divergence → Anderson, then simultaneous.

  "Simultaneous" means an SCC Newton step for equation-oriented units. For opaque units it means Broyden on x − G(x), a finite-difference tear Jacobian with Broyden for small tear sets, or JFNK under the prior conditions.
- **Wegstein** stays optional, for reference-tool parity only.
- **Independence.** This lands first; it needs no architecture.

## 6. <a id="methods"></a>Methods to bind, library-first

| Gap | Library owner | Shared resource | Status, caution |
|---|---|---|---|
| Per-block closed form, bracketed, certified, small dense | Guarded closed form; argmin `BrentRoot`; Symbolica certified root; KINSOL Dense | none | Brent unused. `block_solve` only as a scaled first try |
| Two-sided bounds | PETSc VI (declared complementarity); TAO BNTR/TRON | `petsc-process` | Unbound. Serial lane or process lane |
| Damping, arclength, Broyden, NGMRES, Anderson | PETSc NLEQ-ERR (with SSLS), `SNESNEWTONAL`, `SNESQN`, `SNESNGMRES`, `SNESANDERSON` | `petsc-process` | Unbound |
| JFNK on opaque maps | KINSOL difference-quotient Jv; PETSc `-snes_mf` | none; `petsc-process` | Unbound |
| Least squares, LM | TAO BRGN | `petsc-process` | Unbound |
| Recycle acceleration defaults | KINSOL damping and Anderson, with a project restart rule | none | Bound, default off |
| NLP statistics, timing, scaling and robustness options | Ipopt `Statistics()`, `TimingStats()`, `nlp_scaling_method=user-scaling`, `neg_curv_test_*`, warm-start options | `mumps` if selected | Through the C++ bridge |
| KLU symbolic retention across solves | A project-owned `SUNLinearSolver` (precedent: `dynamics/linear.rs`) | none | Proposed; fact B |
| Batched NLP/QP | POUNCE `solve_*_batch_parallel[_warm]` | none | An alternative to lane sessions. Time limit per instance; warm seeds index-aligned |
| Dynamics diagnostics | IDAS getters; Diffsol `order()`/`h` | none | Unbound |
| ODE subsystems | CVODES (feature); ARKODE | none | Disabled |
| LP/QP sequences | HiGHS retained basis; `getRunTime` | `highs-scheduler` (shared) | Basis cleared today |
| Global statistics | SCIP `SCIPcollect*Statistics`, `SCIPwriteOrigProblem` | none | Unbound |
| Rank-revealing sparse QR | SuiteSparse SPQR | none | R36-01; binding blocked |
| Convex MINLP; parametric QP | SHOT or Bonmin; qpOASES or DAQP | — | Candidates; lower materiality |

## 7. <a id="rule-impacts"></a>Prior rule impacts RC01–RC13, amended

These are carried as principal [RC10](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md#rc10).

| Prior RC | Status |
|---|---|
| RC01 (transitions as named rules; SCIP only under certify or global permission) | Carried; best-merit start for divergence |
| RC02 (preference in the plan; adapters declare suitability) | Carried, plus stop fidelity, statistics fidelity, shared resources, width sensitivity and reuse-after-terminal rules |
| RC03 (decisions read typed observation families) | Carried, with `inertia_mode` and map-callback recycle history |
| RC04 (retire R-48; Ipopt timing via the C++ bridge) | Carried |
| RC05 (admit HSL) | **Withdrawn.** R-34 stays; parity is solution-level |
| RC06 (R-47 capability trigger) | Carried |
| RC07 (recycle method as plan output; per-block routes) | Carried, with the corrected per-block rules, global acceptance criterion and conditional polish |
| RC08 (history snapshot recorded as a result input) | Carried; replay from the recorded permutation; snapshot pinned through resumption |
| RC09 (attempt time and cost as product data) | Carried, in the journal and record plane |
| RC10 (§15 describes the catalogue) | Carried |
| RC11 (§18.9 matrix and capability strings) | Carried. Adds shared resources and corrects the FERAL wording and the "symbolic analysis retained" claims (facts A, B) |
| RC12 (PETSc admission scope) | Carried. `petsc-process` exclusive placement; no in-solve subdomain parallelism without MPI |
| RC13 (homotopy step control by observed effort) | Carried |

## 8. Prior findings mapped

| 2026-10-12 finding | Here |
|---|---|
| F01, F02, F03, F11 | Principal [F09](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md#f09), §4.1–§4.3, §4.7 |
| F04 | [F10](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md#f10), §5 |
| F05, F06, F07, F10, F12, F13 | [F11](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md#f11), §4.4–§4.5 |
| F08 | [F12](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md#f12), §4.6 |
| F09 | [F13](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md#f13), §6 |

## 9. Change scenarios

| Scenario | Target behaviour |
|---|---|
| Add a method profile | One profile declaration, including resources, stop fidelity, width and reuse rules, plus one preference rule |
| Add a selection feature | One Salsa feature query at its lifetime, plus the rules that read it |
| Add a transition | One transition row with its rationale and start translation |
| Per-block method | The plan applied per block; level-synchronous wavefronts on the lane team |
| Test selection without native solvers (PSE-S05) | Plan tests over scripted features and profiles |
| Upgrade a library (PSE-S06) | The adapter maps new statistics into its typed column; the profile version keys history |
| Study over many cases (PSE-S07) | Lanes reuse allocations. Warm state flows along declared chains. Advice comes from a frozen snapshot. Results are independent of lane count within the class |
| A recycle that will not converge | Map-callback history → damping and Anderson with restart → successor by representation |
| Edit → re-solve | Plan features reused at their lifetimes. The lane re-solves with a recorded session `StartPolicy` |
