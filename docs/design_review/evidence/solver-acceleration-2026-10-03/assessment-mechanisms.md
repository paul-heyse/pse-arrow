# DR-2 supporting assessment: mechanisms, library fit and numerical integrity

Bounded supporting assessment for the principal review
`docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md`
(design tier, target purpose). It carries placement, numerical-contract and library judgments for
mechanism families M1–M18 and no overall verdict or disposition ledger. Pipeline composition and the
domain model (strategy ownership, start-information model, derived-system identity, escalation
semantics, retained-information model, budgets, options catalogue) belong to DR-1; section 7 lists the
contract assumptions my placements need from DR-1.

## 1. Scope, baseline and inspection

**Standard.** Core 3.3, process-simulator profile 1.3, binding `pse-arrow`
(`docs/design_review/design_principles/standard.toml`). Principles applied: §F, G6, G7, G8, DP-11, DP-12,
PS-06–PS-10, PS-12, PS-G3.

**Baseline.** HEAD `f0b902589` plus the uncommitted Plan 25k working tree, read as-is while a concurrent
session was editing. Pinned library sources: the `native-solver-libraries` corpus (SUNDIALS 7.1.1 via
`sundials-sys` 0.6.2, Ipopt 3.14.20, POUNCE 0.12.0 crates, FERAL 0.18.0).

**Maintainer framing applied.** Repository rules are treated as heuristics; where a rule conflicts with
the target, I give the replacement rule that keeps its intent. Library methods are preferred over bespoke
code, and licences are never a reason to reject a library. No benchmarking: selection rests on
mathematical facts and in-solve observations, and every benefit below is *Proposed*.

**What I inspected myself (decisive source):**
- `crates/pse-backend-native/src/kinsol.rs`:
  - callback return mapping `result()`;
  - the `residual`, `jacobian` and `solve` paths;
  - `KINSetNoInitSetup(0)`;
  - `validate_contract`;
  - `termination()`.
- Vendored `kinsol.c` (SUNDIALS 7.1.1):
  - the `KINSolInit` first evaluation;
  - `KINFullNewton` / `KINLineSearch` recovery and α/β loops;
  - `KINPicardAA` and `KINFP`, including their convergence tests;
  - `sthrsh`/`noInitSetup`;
  - where `AndersonAcc` and damping are applied.
- `crates/pse-backend-native/src/recycle.rs`: `CausalMap::map` and `original_residual`.
- `callback.rs`: `classify`.
- `implicit.rs`: the nested KINSOL session, `setup_interval: 1`.
- `crates/pse-math/src/implicit.rs`: `Options.start` ("no previous trial is used implicitly"), and the faer
  `SymbolicLu` IFT path.
- `crates/pse-compiler/.../executable/implicit.rs`: `ImplicitMeaning` and `SelectionNeighborhood`.
- `kkt/advance.rs`: `predict` and its active-set refusal.
- `kkt.rs`:
  - `KktFactor` is an active-set, μ = 0 factor;
  - its `SensBacksolver` implementation provides `bound_rows` but no release methods;
  - inertia, LICQ and weak-row certification.
- `square_response.rs`: dense `Mat`, SVD and partial-pivot LU.
- `pounce.rs`: `PINNED_OFF` and the backend factory installed per solve.
- POUNCE 0.12.0 source:
  - `pounce-sens-core` `boundcheck::step_along_path`, `SensBacksolver` (release methods);
  - `pounce-algorithm` `second_opinion::second_opinion_rungs`, `batch::solve_nlp_batch_parallel_warm`
    (rayon), `set_kkt_schur(_block)` (transparent fallback);
  - `pounce-restoration` `run_second_opinion_ladder`.
- Ipopt `IpStdCInterface.cpp`: `IpoptSolve` constructs a new `StdInterfaceTNLP` and calls `OptimizeTNLP`.
- Architecture: §17.4–§17.6 and the §18.7 selection paragraph (`numerical-execution.md`); PS-06–PS-10
  text; core G-gates and §F.

**What I took from evidence workers as leads, not re-verified:**
- literature claims in the mechanism cards;
- candidate-library internals: PETSc `tr.c`/`al.c`/`snesngmres.c` NaN aborts, Uno C API and its
  evaluation-error radius shrink, Ceres invalid-step retry, `russell_nonlin` failure semantics, egobox API.
These are labelled where they decide a library choice.

**Evidence labels.**
- *Interface-checked*: I read the signature and contract in pinned source.
- *Implemented*: the source path is traced in pse.
- *Proposed*: a placement or remedy.

Nothing was built or executed, so nothing here is *Tested* or *Measured*.

## 2. Placement table

Classes:
- **L-used**: pinned library, used.
- **L-unused**: pinned library, unused.
- **L-new**: candidate library to adopt.
- **C**: composition of pinned-library solves; pse owns the sequencing and admission.
- **B**: bespoke numerical code, with its scope bounded.

"Owner" names the integration owner (a module path where one exists). For a library item, "Revisit"
names what would reopen the decision.

| # | Mechanism | Placement | Library / function | Owner | Bespoke scope | Revisit trigger |
|---|---|---|---|---|---|---|
| M1 | NLP parametric sensitivity prediction | L-used, generalized | `pounce-sens-core` `SensApplication::parametric_step` over pse's active-set `KktFactor` (FERAL) | `kkt/advance.rs` → keyed retained-information operation (DR-1) | none beyond screening and seed construction | — |
| M1b | Active-set-changing path following | L-unused + C | `boundcheck::step_along_path`, `path_direction(_with)`, `refine_step_onto_bounds`; `pounce-qp` parametric homotopy for QP class; NLP corrector = warm POUNCE SQP or Ipopt/POUNCE IPM solve | `kkt/` (new release methods on `KktFactor`) | outer segment/corrector loop ≈ 300–600 lines; `SensBacksolver::solve_released*` over a FERAL refactor | upstream `pounce-sensitivity` corrector pinned and fitting the active-set factor |
| M2 | Advanced-step / one-corrector (RTI-like) intermediate steps | C | M1/M1b prediction + one native iteration (KINSOL `mxiter` bounded, POUNCE/Ipopt `max_iter` bounded) on intermediate problems only | generic sequence primitive (DR-1) | none (a policy over bounded library calls) | — |
| M3 | Root tangent / secant predictor | C (faer sparse LU, pinned-used) | faer `SymbolicLu`/`Lu` at x* (the pattern the `pse-math::implicit` IFT path already uses); `CasePlan::parametric` for F_p | a sparse root-response operator shared by `square_response` publication and prediction | secant: vector arithmetic; tangent: none beyond assembly | — |
| M4 | Inexact Newton–Krylov, EW forcing, preconditioning | L-used | KINSOL SPGMR/SPFGMR/…, `KINSetEtaForm`, `KINSetPreconditioner` | `settings/kinsol.rs` method profile | block preconditioner = KLU/faer per BTF block (library factors; small pse glue) | FERAL/KLU symbolic fill estimate makes direct LU infeasible; a directional-derivative program exists |
| M5 | Jacobian/setup reuse, modified Newton | L-unused | KINSOL `KINSetNoInitSetup(SUNTRUE)`, `KINSetMaxSubSetupCalls`, `KINSetResMonParams`; diffsol Jacobian lagging (already exposed) | `kinsol.rs` session | none | — |
| M5′ | Ipopt same-structure re-optimization | L-unused (C API gap) | `IpoptApplication::ReOptimizeTNLP` + `warm_start_same_structure` / `warm_start_entire_iterate` via a C++ shim | `pse-ipopt-sys` + `ipopt.rs` session | shim ≈ 100–200 lines of C++ glue, no numerics | Ipopt C API gains re-optimize; sequences routinely routed to Ipopt rather than POUNCE |
| M5″ | FERAL symbolic reuse across POUNCE solves | L-unused | FERAL `last_pattern_fingerprint`; requires a POUNCE same-structure path | `pounce.rs` | none | verify POUNCE `warm_start_same_structure` behaviour (uncertainty U3) |
| M6 | Anderson / fixed-point / Picard | L-used | KINSOL `KIN_FP`, `KIN_PICARD`, `KINSetMAA/DampingAA/DelayAA/OrthAA` | `kinsol.rs`, `recycle.rs` | none; **fix DM-F01 first** | — |
| M7 | Bounded, domain-aware local models (LM/TR, SLP/SQP-filter) | (a) L-used: Ipopt/POUNCE solving the square system as a bounded feasibility NLP (restoration, `start_with_resto`, `least_square_init_*`); (b) L-new: **Uno** TR filter-SQP/SLP | Ipopt C, POUNCE (feasibility oracle exists); Uno C API (HiGHS for SLP/convex QP, BQPD for nonconvex QP) | NLP adapters; Uno as a new adapter (PSE-S02) | none (no bespoke LM loop) | Uno unavailable or failing H2/H5 in qualification → reconsider Ceres |
| M8 | Library-native NLP globalization and warm-start options | L-used, typed | Ipopt `start_with_resto`, `expect_infeasible_problem`, `least_square_init_*`, `mu_strategy`/`mu_oracle`, `warm_start_*`; POUNCE ladder rungs via `second_opinion_rungs()`, `sqp_globalization` | typed adapter settings | none | — |
| M9 | Natural-parameter / pseudo-arclength continuation | C + small B | correctors: KINSOL on F(·,λ) or on the bordered (n+1) system; Ipopt/POUNCE for constrained; tangent from M3's operator | a continuation tracker replacing `engines.rs` homotopy step control | predictor, step control, orientation, fold indicator, bordering row ≈ 500–900 lines | a linkable tracker meeting H1–H4 (recoverable trials, bounds, cancellation, typed outcome) |
| M10 | Constructed homotopies (Newton, fixed-point with A = D·P_M, affine, model, bounded) | C + B (construction) | Symbolica derives H, H_x, H_t; tracker = M9 | derived-system realization (DR-1) | construction rules ≈ 300–600 lines | as M9 |
| M11 | Pseudo-transient continuation | (a) L-used: IDAS/Diffsol integration of the **authored** dynamic form to steady state; (b) C + B: KINSOL on V(x − x_k)/δ + F(x), one iteration per pseudo-step, SER δ control | IDAS, Diffsol, KINSOL | dynamics adapters; derived system | SER controller ≈ 100–250 lines | PETSc admitted for other reasons (see §5) |
| M12 | DAE consistent initialization / reinit | L-used (tuning L-unused) | `IDACalcIC` (+ `IDASetMaxNumItersIC`, `…StepToleranceIC`, `…LineSearchOffIC` and the other IC controls); Diffsol `set_consistent`; escalation = algebraic subsystem through KINSOL/M13, then a least-deviation NLP (Ipopt) labelled as a modified specification | `dynamics/idas.rs` | none | — |
| M13 | Block-sequential exact solve (BTF / tears) | C | `pounce-presolve` matching/DM/BTF (used), KINSOL/Ipopt per block; KLU BTF internally | structural plan → main-solve strategy | orchestration only | — |
| M14 | Reduced space / nonlinear elimination | C | existing `pse-math::implicit` (inner KINSOL + faer IFT) applied to qualified BTF blocks; outer Ipopt/POUNCE/KINSOL | `pse-math::implicit`, derived system | block→implicit projection, numeric qualification | — |
| M15 | Nonlinear preconditioning (NGS + Anderson, right NPC/ASPIN) | C + B | KINSOL FP+AA as the outer accelerator over block sweeps (after DM-F01); KINSOL Newton on the right-preconditioned residual | strategy layer | sweep orchestration, ASPIN JVP assembly ≈ 600–1,200 lines | PETSc admitted with recoverable-domain semantics in NGMRES/NASM/ASPIN |
| M16 | Multifidelity / surrogate management | L-new (construction) + B (management) | egobox GP/MoE (values, gradients); model homotopy (M10) as the root-problem bridge | start-proposal producer | Eason–Biegler/Alexandrov TR management ≈ 600–1,000 lines | a library with TR model management appears |
| M17a | Multi-experiment / multi-scenario Schur | L-unused | POUNCE `set_kkt_schur_block` / `FeralSchurSolver` (serial, dense S, exact Hessian) | `pounce.rs`, fitting | partition construction from parameter identity | many linking variables (implicit Schur needed) |
| M17b | Multistart and parallel sweeps | C | pse study/worker scheduler composing single solves; **not** `solve_nlp_batch_parallel_warm` (rayon pool, `T: Send + 'static`) | study execution | start generation, clustering | pse oracles become `Send` and a rayon pool is admitted under the job's thread budget |
| M18 | Evaluation reduction | L-used / L-unused / C | Ipopt `*_constant` (used); POUNCE `hessian_approximation=partitioned` (typed, unused); POUNCE coloured FD Hessian (typed as an FD derivative source); Symbolica directional-derivative programs (JVP); nested warm starts under selection rules (DM-F07) | `HessianMode`, `pse-math` | none beyond typing | — |

Declined libraries:
- PETSc (SNES/TS/TAO);
- Ceres;
- `russell_nonlin`;
- sIPOPT;
- Ipopt inexact algorithm (not buildable without Panua Pardiso);
- ARKODE/CVODE feature enablement;
- faer sparse QR as a primary LM factor;
- POUNCE NLP batch-parallel API.

Section 5 gives the reasons and revisit triggers.

## 3. Numerical contracts by family

In every row of every family, success means the original problem is satisfied, in its original
coordinates, under the frozen final `ResolvedAccuracy`. Candidate use (`CandidateUse`) remains the
single permission owner. A mechanism's own success only licenses its output as a **seed** of a stated
class:
- a predicted point;
- an auxiliary-path point;
- a physical-neighbour solution, labelled with its λ;
- an exact partial solution (a BTF prefix);
- a modified-specification solution, with its deviation reported.

### (a) Sensitivity and path prediction: M1, M1b, M2, M3

| | M1 / M1b (NLP) | M3 (roots) | M2 (one-corrector intermediate) |
|---|---|---|---|
| Prerequisite facts | Exact Hessian (`HessianMode::Exact`; quasi-Newton refuses). A certified base point: LICQ and SOSC by inertia, and weak-row classification for SC. pse's `kkt.rs` already certifies all three on the **unperturbed** active-set matrix, a strength to preserve. Parameters are declared columns (`CasePlan::parametric`). Smooth between p₀ and p (no selector, regime or integer change in the facts). Same layout/profile stamp. M1b additionally: a QP solver that accepts a reduced-PD, full-space-indefinite Hessian (`pounce-qp`, not HiGHS/Clarabel). | Regular root: sparse factor of F_x **at x\***, with rank and condition from the factor (FERAL `estimate_condition_1norm` / faer pivots), not the KINSOL held factor (it may be up to `msbset` iterations stale). F_p from the parametric program at first order. Secant: two accepted points on one branch, the same direction, and no fold indicator between them. | The problems form a declared ordered sequence (continuation stages, homotopy, ordered sweep), and every one except the last is marked **intermediate**. |
| In-solve triggers / abandonment | Active-set screen: multiplier sign flips and entering rows/bounds. M1 refuses; M1b walks breakpoints instead. Then:<br>- Schur pin residual / plain-vs-regularized operator choice (POUNCE);<br>- breakpoint count reaching `max_iter`, which means the remainder was taken in one step: record it;<br>- `RefineStop::WorseThanPlain`;<br>- KKT residual at the prediction against the residual of the unpredicted base point at the new p (two evaluations; the prediction must beat a plain warm start or be dropped);<br>- corrector iterations, restoration entry or status change shrink the admissible radius for later predictions. | Prediction evaluable and inside the guard/FBBT box (otherwise shrink Δp). Scaled residual at the prediction ≤ the residual of the plain warm start. First-corrector contraction Θ₀ = ‖Δx¹‖/‖Δx⁰‖: halve Δp at Θ₀ ≥ ½. Corrector line-search failure. Sign change of det F_x (factor sign/inertia) means a fold was passed: stop natural prediction and hand over to arclength (M9). | Contraction ratio ≥ 1 ⇒ abandon to a full solve of that problem. |
| Intermediate accuracy | The prediction has no tolerance of its own: it is a seed. The corrector runs at final accuracy when its problem is final. | As M1. | One corrector iteration per intermediate problem. **The final problem always gets a full solve plus qualification.** |
| Derivative consistency | The factor must be μ = 0 / δ_w = 0. pse's factor is assembled at the candidate without inertia correction; it must stay so, and any regularization must be recorded and must refuse prediction. Hessian of pinned columns at second order. | The tangent is exact only with a fresh F_x at x\*. A stale factor makes the tangent inexact, which is admissible only labelled as such. | Same as M1/M3 for its predictor. |
| Branch / local honesty | The prediction can land in another local minimizer's basin; the corrector's result is "a KKT point near the prediction". Record the distance between the prediction and the corrected point against the first-order error model; a gap far outside that model is flagged as a possible basin change. | Same, plus fold and branch indicators recorded with the result (PS-12). | Same. |
| Observations the pipeline must expose | Active-set changes; breakpoint list (`PathSegment`); Schur pin residual; prediction-versus-base residuals; corrector iteration/restoration counts. | Θ₀ (needs per-iteration step norms; see (b)); det-sign/inertia of the factor; residual at the prediction. | Per-problem contraction ratio. |

### (b) Inexact Newton, Krylov, preconditioning, reuse, Anderson: M4–M6

**Prerequisite facts.**
- *M4 (Krylov):* admitted only when a direct factor is unattractive *as a fact*, i.e. the symbolic fill
  estimate of KLU/FERAL exceeds the memory allowance or matrix-free operation is required. A
  preconditioner must be available. Today:
  - only Jacobi exists, which is not a credible default for badly scaled PSE Jacobians;
  - the JVP is assemble-then-multiply (`assembly.rs` `jacobian_product`). That is exact, but it removes
    the reason for being Jacobian-free (DM-F12).
- *Forcing terms:* irrelevant under KLU.
- *M5 (cross-solve reuse):* same layout and profile (session reuse already checks this). Numeric
  staleness is tolerated because KINSOL's residual monitoring (direct solvers) and its
  line-search-failure refresh catch it. Krylov solves turn residual monitoring off, so preconditioner
  age must be bounded explicitly (`msbset`).
- *M6:* a **declared** map G or constant splitting L with a contraction argument:
  - a tear map;
  - a block Gauss–Seidel sweep over BTF;
  - an M-matrix splitting.

  Bounds forbidden: pse refuses them already, correctly.

**In-solve triggers and abandonment.**
- *Krylov:* `KINGetNumLinIters` per nonlinear iteration and `NumLinConvFails` growing means refresh the
  preconditioner, then switch to direct.
- *Reuse:* a line-search failure with a stale setup makes KINSOL refresh internally; only repeated
  failure abandons.
- *Anderson:*
  - a fixed-point residual ratio ≥ 1 repeatedly means damp, then abandon;
  - KINSOL does not expose θ_k or cond(R), so abandonment rests on residual ratios between bounded
    segments;
  - a recoverable map failure must abort the segment (DM-F01) and drive the declared response: a block
    rung, or Newton on the tears.

**Intermediate accuracy.**
- η (EW choice 1 or 2) governs only the linear solves.
- Final acceptance is never relaxed.
- KINSOL's own `fnormtol` stays the final feasibility for final problems, and a separate corrector
  tolerance applies for intermediate ones (H8 / DR-1).

**Derivative consistency.**
- A stale Jacobian is a route choice; the residual is always exact.
- An FD JVP is never selected silently (PS-07). If ever admitted, it is typed as an FD derivative source
  with a noise-class fact: nested iterative properties refuse it.

**Success meaning.** As for every family: the original-residual re-evaluation. That already exists:
`kinsol.rs` re-evaluates `original_residual` for FP.

**Observations to expose.**
- Per-iteration step length and fnorm. KINSOL has no iteration callback wired (SUNDIALS 7 routes info
  through SUNLogger). Two realizations:
  - iteration-bounded `KINSol` segments with `NoInitSetup(TRUE)`, so the factor survives each segment.
    This costs one extra residual evaluation per segment (`KINSolInit` evaluates at uu), so use it only
    for the first two corrector iterations (Θ₀) or at rung boundaries;
  - or derive norms inside pse's residual callback, which sees every trial.
- Setup count (`KINGetNumLinSolvSetups` is not read today).
- Linear iterations and failures.
- A typed abandonment latch: the callback returns −1, as cancellation already does.

### (c) Local models and NLP globalization: M7, M8

**Prerequisite facts.**
- An **evaluable centre**: one evaluation at the proposed start, before submission. A failure there is
  terminal in both KINSOL (`KIN_FIRST_SYSFUNC_ERR`) and Ipopt (invalid number at the start), so
  start-evaluability is a pipeline fact. Its response is start repair:
  - project into the guard-sign interior or the FBBT box;
  - or apply Ipopt `bound_push`;
  - or anchor an M10 homotopy at a repaired point.
- Bounds and domain facts.
- For Uno filter-SQP: C² data and exact or quasi-Newton Hessians. BQPD is needed for nonconvex QP
  subproblems; HiGHS covers SLP and convex QP.
- For the POUNCE SQP route: the colleague's point stands. POUNCE active-set SQP already solves local QP
  models, so a separate bounded-LM QP initializer duplicates its first iterations. I therefore do not
  place a bespoke "lifted LM QP via HiGHS/Clarabel" preparation stage.

**In-solve triggers.**
- KINSOL `KIN_LINESEARCH_NONCONV`, `KIN_LINESEARCH_BCFAIL`, `KIN_REPTD_SYSFUNC_ERR`, and
  `KIN_SYSFUNC_FAIL` with trial-rejection evidence (see DM-F04).
- Ipopt `alg_mod = 1` (restoration), `Restoration_Failed`, `Infeasible_Problem_Detected` (local),
  persistent `regularization_size > 0`, collapsing `alpha_pr`.
- POUNCE `SecondOpinionTrigger::{LocalInfeasibility, InvalidNumber, RestorationFailure,
  IterationLimit-with-quality-escalation}`. This is the library's own fact-and-observation gate,
  `second_opinion_rungs`.
- Uno (per the evidence worker): TR radius collapse; `EVALUATION_ERROR` / `INFEASIBLE_STATIONARY`.

**Intermediate accuracy.** A feasibility-NLP solution of a square system is a seed for the original root
solve unless it is itself qualified against the original residuals. Usually it is, because the
equations are identical.

**Success meaning and honesty.**
- An NLP feasibility stop at a nonzero infeasibility is *local* infeasibility. Publish it as
  "locally infeasible / least-squares stationary", never as "no root" (PS-12).
- Ipopt `Solved_To_Acceptable_Level` stays an acceptable-level stop and goes to qualification.

**Observations.**
- Ipopt per-iteration events, already recorded.
- POUNCE `SolveStatistics` including `quality_escalations`, already recorded.
- These must also be **readable at rung boundaries**, not only published. Today a production reader
  does not exist ("metrics are never an input to a decision").

### (d) Continuation and constructed homotopy: M9, M10

**Prerequisite facts.**
- *M9:* the path parameter is an authored physical parameter. F is C¹ in (x, λ) across the range, so
  no regime or selector switch lies on the path (from presolve and selection facts). F_λ is available
  from `CasePlan::parametric`.
- *M10:*
  - an evaluable anchor;
  - a construction that preserves the pattern of F_x. The Newton and affine forms do; the fixed-point
    form needs A = D·P_M, from the structural matching witness that already exists;
  - for a model homotopy, a recorded low-fidelity residual over identical variable identities.
  - Probability-one claims are never made: process models violate boundedness or smoothness.

**In-solve triggers.**
- Corrector failure at Δs_min.
- λ̇ sign change (fold): natural-parameter tracking stops or switches to arclength.
- Bordered-determinant sign change without a λ̇ change (branch point): record it, and do not claim
  continuity.
- ‖x‖ growth, or a path returning to t = 0.
- Repeated evaluator domain errors at Δs_min, which call for a bounded homotopy or abandonment.
- Arclength budget exhausted.

**Intermediate accuracy.**
- A path tolerance distinct from the final one. Every intermediate point is an *auxiliary-path* point
  (M10), or a *physical-neighbour* solution labelled with λ (M9).
- The terminal point is a seed. Then follows an ordinary solve of the unchanged original specification
  with the final qualification. This keeps §17.5's "only the unchanged original specification may
  commit".

**Derivative consistency.**
- Derived residuals and Jacobians are compiled through Symbolica from the same original equations, so
  they are exact.
- The bordered system's pattern is F_x plus one dense row and one dense column. Its sparse
  factorization goes through KLU/FERAL; the LOCA-style bordering uses the existing factor.

**Branch honesty.**
- A result reached after a recorded fold or branch crossing is published as "a solution reached by
  continuation across a fold". Uniqueness and continuity with the start are not implied.
- The current homotopy cannot detect folds at all (DM-F11).

**Observations.** Corrector iteration count and Θ₀; tangent angle; λ̇ sign; factor determinant sign or
inertia (FERAL reports inertia); domain-error counts per step.

### (e) Pseudo-transient continuation and DAE initialization: M11, M12

**Prerequisite facts (M11).** A justified flow. In decreasing strength:
1. **The authored dynamic form of the same model.**
   - The holdups give M, and the flow is the physical dynamics; IDAS integrates it.
   - Refuse when the target is known to be dynamically unstable.
   - Refuse when closed inventories make the steady state depend on the initial inventory. That is a
     dynamical invariant, so the steady-state equations are singular and need their specification
     from the authored inventory.
2. **A matching-paired V = D·P_M** with signs from the matched Jacobian entries. This is local evidence
   only; the mechanism is labelled heuristic.
3. **The gradient flow.** Equivalent to least-squares globalization and redundant with M7(a); do not
   place it.

The algebraic part with singular V must be index-1 (a structural fact).

**Triggers.**
- δ stagnation while ‖F‖ plateaus.
- Residual growth at large δ: loss of attraction.
- Repeated step rejection.
- Domain errors, which project or cut.

**Intermediate accuracy and meaning.**
- Pseudo-time iterates are never dynamic-simulation results.
- The terminal iterate is a seed, or the last Newton-limit steps are the original correction.

**M12 facts.**
- Index-1 and identity vector.
- Specification count equals the dynamic DOF.
- After an event, a mode change triggers fresh structural analysis.
- Diffsol `set_consistent` assumes zero-mass-diagonal algebraic variables. Refuse it for a non-diagonal
  mass matrix.

**M12 triggers.** `IDA_LINESEARCH_FAIL`, `IDA_CONV_FAIL`, `IDA_CONSTR_FAIL`, `IDA_FIRST_RES_FAIL`,
`IDA_NO_RECOVERY`. Escalation:
1. retune IDACalcIC (`MaxNumItersIC`, `MaxNumJacsIC`, `StepToleranceIC`), all library controls;
2. then solve the algebraic subsystem given y_d as a square root problem (M13 → KINSOL / Ipopt
   feasibility);
3. then a least-deviation NLP. Its result is a **modified-specification** solution with every moved
   specified value reported. It is never silent.

**Re-initialization after a discontinuity.** The pre-event state supplies the algebraic guess. The
DAE-IC derived system is keyed by mode identity.

### (f) Structural block solving, reduced space, nonlinear preconditioning: M13–M15

**Prerequisite facts.**
- Empty DM over- and under-determined parts.
- Exact incidence, including opaque-call inputs and the union over piecewise branches.
- For every block solved or eliminated, a **numerical qualification at the point of use** (structure is
  generic rank only): pivot magnitude relative to scaling, a condition estimate from the FERAL/faer
  factor, or a rank-revealing check for small blocks.
- *M14* also needs:
  - bound-inactive eliminated variables;
  - inner tolerance tighter than outer, by a stated factor;
  - a selection meaning that makes y(z) a function. For a BTF block that is not authored `implicit`,
    uniqueness evidence is usually unestablished. The admissible claim is then "local IFT branch near
    the current inner root", which is a history-dependent selection and therefore a **solve strategy,
    not an expression meaning**. Its result must be re-qualified on the full original system (it is).
- *M15:* a declared fixed-point map whose fixed points are exactly the roots. For an NGS sweep over BTF
  in topological order this holds by construction.

**Triggers.**
- A block numerically singular at the point: merge it with its neighbours or solve simultaneously.
- A block failing from its predecessor values (there is no feedback): escalate to a simultaneous solve
  of the merged set.
- Tear iteration with residual ratio ≥ 1: Newton on the tears.
- An inner failure at an outer trial is reported to the outer solver as a recoverable evaluation
  failure. This works for Ipopt and for KINSOL Newton. It does **not** work for KINSOL FP/Picard: see
  DM-F01.
- Repeated inner failures in one region: abandon the elimination.

**Intermediate accuracy.**
- Block tolerances are tighter than the full-system tolerance.
- A completed BTF prefix is an *exact partial solution*.
- Final acceptance is the full-system residual in original coordinates.

**Derivative consistency.**
- M14 reduced derivatives are the existing IFT derivatives at the converged inner root.
- The inner residual must be ≪ outer tolerances, or the reduced function is noisy and its derivatives
  inconsistent.

### (g) Multifidelity and surrogates: M16

**Prerequisites.**
- A declared low-fidelity model or surrogate over identical identities, with recorded provenance.
- For optimization: high-fidelity gradients and first-order consistency correction at each TR centre.
- An outer loop of repeated evaluations: a study-shape fact, not a measurement.

**Triggers.** ρ ≤ 0 repeatedly, then abandon to high fidelity. Proposals outside the sample hull. The
callback-cost share is an in-solve *trigger*, never a benchmark gate.

**Meaning.**
- For root problems, a low-fidelity solution is only a seed, or the anchor of a model homotopy (M10).
- For optimization, TR management converges to stationary points of the **original** problem. Its
  claims are "local / stationary".

### (h) Schur, batch, multistart: M17

**Schur.**
- An exact Schur KKT is a linear-algebra realization: the step is monolithic up to rounding and pivots,
  and inertia is additive.
- POUNCE requires the IPM + FERAL + exact-Hessian path, dense S and few linking variables. For
  multi-experiment estimation the linking variables are the fitted parameters θ.
- POUNCE "falls back transparently" when the partition is unsuitable. The pipeline must record whether
  the partition was honoured (G7). A fallback does not change the problem, so it is not a substitution
  of meaning, but an unrecorded capability claim is.

**Multistart.**
- Bounded sampling domain.
- Each start's candidate is qualified independently. "Best accepted" is never "global" unless SCIP
  certifies it (PS-12).

### (i) Evaluation reduction: M18

**Constant-derivative flags.** Already used.

**Partitioned quasi-Newton.**
- Admitted when exact second order is unavailable or declared unaffordable *as a fact*: a provider or
  nested implicit body compiled only to first order.
- It refuses every sensitivity mechanism (M1, M1b, M2).

**FD Hessian of the analytic Jacobian.** Typed as an FD derivative source (PS-07), and never chosen
silently.

**Nested inner warm starts.** These need the DM-F07 rule: the selection meaning must be independent of
the start, and the qualification evaluation is canonical.

## 4. Findings

Severity is my estimate of consequence within this scope; the coordinator disposes.

### DM-F01: KINSOL FP/Picard ignore pse's recoverable callback return; a failed map evaluation can produce native "success"

**Severity.** High for the recycle route (current supported behaviour) and a blocker for M6/M15
composition. PS-10, DP-12. *Implemented* path traced, not executed.

**Cause.**
- `kinsol.rs` `result()` returns `1` for every recoverable callback failure, whatever the strategy.
- On failure, `residual` never writes `out`, so KINSOL keeps the previous `fval`.
- `KINFP` and `KINPicardAA` stop only on `retval < 0` (vendored `kinsol.c`, `KINFP` loop and
  `KINPicardAA` loop). They proceed with the stale `fval`.
- In **undamped fixed point without Anderson**, a failure at x_k gives
  `unew = fval_stale = G(x_{k−1}) = x_k = uu`. Then `delta = 0`, and KINFP returns `KIN_SUCCESS` at
  exactly the point where the map could not be evaluated.
- With damping or Anderson, iteration continues on a wrong map value. For Picard, the convergence test
  uses `F` at the previous iterate.

**Evidence.**
- `kinsol.c` (7.1.1): `KINFP` checks `retval < 0` only; the convergence test is
  `‖unew − uu‖ ≤ tolfac·fnormtol`; `ret_newest` is false by default.
- `recycle.rs` `CausalMap::map` propagates unit errors; `callback::classify` marks domain, validity,
  applicability and range errors as `Failure::Trial`.

**Consequence.**
- No false acceptance: `solve` re-evaluates `original_residual`, which fails or is infeasible, so
  assurance is `None`.
- But the published termination is `KIN_SUCCESS` (category Success) with a validation failure. The
  cause, an evaluation failure, is misattributed. `callback::retryable_evaluation` and any escalation
  keyed on `Termination::Evaluation` see the wrong category.
- Damped or Anderson runs waste the iteration budget on stale data.

**Correction direction.**
- Make the callback return strategy-aware. For `Function::FixedPoint` and `Function::Picard`, a
  recoverable failure returns `−1` while keeping the typed trial-rejection evidence. KINSOL then returns
  `KIN_SYSFUNC_FAIL`, which maps to `Termination::Evaluation`.
- The declared response belongs to the escalation ladder: a block rung, Newton on the tears, or a
  damped restart from the last good iterate.
- Do not build a bespoke step-cutting wrapper inside the map.

**Verification.** A unit test with a map that fails recoverably at iteration k:
- undamped no-AA must yield `Termination::Evaluation`, not `KIN_SUCCESS`;
- a Picard variant likewise.

### DM-F02: Typed KINSOL settings that the selected strategy ignores are accepted silently

**Severity.** Medium. G7, PS-09 truthful capability, DP-09 identity. *Implemented.*

**Cause.**
- `validate_contract` accepts `anderson > 0`, `anderson_delay` and `orthogonalization` (and `damping < 1`,
  which is never validated) with the `LineSearch`/`Newton` strategies.
- KINSOL applies `AndersonAcc` only in `KINPicardAA` and `KINFP`, and `kin_damping`/`kin_beta` only there
  (vendored `kinsol.c`).
- Conversely, `setup_interval` and `eta` are meaningless for `FixedPoint`.

**Consequence.**
- A requested acceleration has no effect but is recorded in provenance and the profile stamp. That is a
  false capability record.
- Session identity changes on a no-op setting, which forces spurious rebuilds.
- Typed method profiles (H6) inherit the ambiguity.

**Correction direction.** An applicability table per strategy in `validate_contract`: refuse
inapplicable fields, or normalize them out of identity.

**Verification.** Contract tests per strategy.

### DM-F03: No cross-solve numeric-setup reuse in KINSOL; residual-monitor controls not exposed

**Severity.** Medium; it blocks M5 for sequences. *Implemented* (`kinsol.rs` `NoInitSetup(0)`).

**Cause.** `KINSetNoInitSetup(self.mem, 0)` runs on every solve, including retained sessions and nested
implicit sessions. `MaxSubSetupCalls`, `ResMonParams` and `NoResMon` are never set.

**Consequence.** Homotopy and continuation steps, recycle loops, studies and nested trials each pay a
fresh Jacobian evaluation and numeric factorization on their first iteration, although KINSOL's
monitors would handle staleness.

**Correction direction.** A typed method-profile field `setup: Refresh | KeepPrevious`:
- admitted only on a reused session (layout and profile match);
- with residual monitoring on for direct solvers and a bounded preconditioner age for Krylov;
- reuse recorded in the `StartReceipt`-like record, because the converged root may now depend on history
  (DP-11: conforming runs differ within tolerance; for a multi-root problem the branch can differ).

Also enforce KINSOL's documented constraint that `msbset` is a multiple of `msbsetsub`. pse admits any
`setup_interval`, so either normalize or refuse.

**Verification.** Assert the setup count with `KINGetNumLinSolvSetups`, which must also be recorded.

### DM-F04: KINSOL's domain recovery is partial (library fact); pse treats the result only as an evaluation failure

**Severity.** Medium for SC-05. PS-06, PS-10. *Interface-checked* (`kinsol.c` `KINLineSearch`).

**Cause.**
- KINSOL recovers a recoverable failure only while halving the full step: at most `MAX_RECVR = 5`
  halvings, down to 1/32.
- Inside the α/β loops, any nonzero return is `KIN_SYSFUNC_FAIL` ("we assume that any update up to pp
  is feasible").
- When no failure occurred at the full step, `rlmax = mxnewtstep/pnorm` can exceed 1, so the
  β-expansion can step beyond the evaluated point into an invalid region and terminate.
- Domain protection otherwise comes only from sign constraints. Two-sided intervals keep only their sign
  information.

**Consequence.**
- Cold starts near invalid regions end as `Termination::Evaluation` although the cause is
  globalization.
- `KINSetMaxNewtonStep` stays at the default (1000·‖D_u u₀‖) even when FBBT/guard enclosures give a
  tighter, fact-based cap.

**Correction direction.**
- (i) A KINSOL profile whose `max_newton_step` is derived from FBBT/guard distances. That is a fact, not
  a measurement.
- (ii) The escalation ladder treats `KIN_REPTD_SYSFUNC_ERR`, and `KIN_SYSFUNC_FAIL` *with
  trial-rejection evidence and no terminal failure*, as a globalization trigger.
- (iii) The next rung is a bounded feasibility NLP through Ipopt/POUNCE. Ipopt's line search accepts a
  `false` callback without a halving cap, then restoration; the full two-sided bounds are honoured.
  After that comes Uno's TR, which shrinks the radius on an evaluation error.

**Verification.** A unit test of the escalation decision function on typed outcomes (native-free).

### DM-F05: The NLP predictor is single-consumer and refuses active-set changes; the pinned path-following API is unused, and its contract fit is unverified

**Severity.** High for the target (SC-01, SC-04), not a current defect. *Interface-checked*.

**Cause.**
- `kkt/advance.rs` `predict` returns `Fallback::ActiveSet` on any predicted change, and has one consumer
  (horizon).
- `pounce-sens-core` `step_along_path`, `path_direction` and `refine_step_onto_bounds` are uncalled.

**Contract fit (my reading).** pse's `KktFactor` is an **active-set, μ = 0** factor, with active bounds
as explicit rows and bound multipliers as unknowns. It implements `SensBacksolver::{dim, solve,
bound_rows}` but not `supports_release` or the `solve_released*` methods. What follows:
- *Holds* on inactive variables reaching a bound, and *drops* of holds, are Schur rows. They work on any
  factor.
- *Releases* of base-active bounds need `solve_released*`. pse can implement these by refactoring the
  active-set KKT without that bound row (FERAL), which is cheaper than a re-solve and exact.
- The walk's activity split and `weak_rows` semantics are written for interior-point σ = z/s factors.
  With an exact active-set factor:
  - "slack" is exactly zero at an active bound;
  - pse already certifies weak rows (`kkt.rs` weak/strong release logic).

  This should map onto `weak_rows`, but the mapping must be verified, not assumed.
- Inactive *constraint rows* entering need the `rowlimit` view. pse's layout has no slack block, so
  inequality rows need representing as watched rows.

**Consequence.** Studies, continuation and fitting cannot use prediction, and the horizon falls back to a
full solve at every active-set change.

**Correction direction.**
- Generalize `Advance` into the keyed retained-information operation (DR-1).
- Implement the release methods on `KktFactor`.
- Call `step_along_path` for M1b and `pounce-qp` for QP-class sequences.
- Keep `predict`'s screen as the M1 admission test.

**Verification.**
- A QP fixture whose active set changes along Δp: the walked prediction must equal the re-solved
  solution to rounding (exact for QPs).
- An NLP fixture: prediction residual ≤ the residual of the unpredicted base point.

### DM-F06: Root sensitivity is dense and transient, so it cannot serve as a predictor at scale

**Severity.** Medium. *Implemented*.

**Cause.** `square_response.rs` builds a dense `Mat` of F_x, takes a dense SVD for rank and a
partial-pivot LU (O(n³)), and drops the factor. The sparse IFT machinery in `pse-math::implicit`
(faer `SymbolicLu`) already solves `F_y dy/dp = −F_p` sparsely.

**Consequence.** M3 and the M9 tangent have no scalable operator. Root studies (SC-02) seed with
zeroth-order values only.

**Correction direction.**
- One sparse root-response operator: faer sparse LU at x\*, rank and condition from the factor or
  FERAL's estimate, retained under a stamp.
- Consumers: publication (`root_response`), the M3 predictor and the M9 tangent.
- The dense SVD remains only as a small-n diagnostic.

**Verification.** Equivalence of dense and sparse dx/dp on fixtures, and a refusal at a rank-deficient
point.

### DM-F07: Nested implicit solves never start from history; a rule that preserves "evaluation is a function of its inputs"

**Severity.** Medium for SC-07. DP-11, PS-11, G6. *Implemented* (`pse-math/src/implicit.rs`
`Options.start`).

**Assessment.** A history-dependent start preserves functional semantics **only when the selection
meaning makes the root independent of the start within the admitted region**:
- `ImplicitMeaning::Unique`;
- `Branch` or `MinimumScore` regimes whose `SelectionNeighborhood` is `Static`, or `RuntimeIsolation`
  with its obligation discharged at the point.

Even then, the value differs by up to the inner tolerance, so evaluations are equal *within tolerance*,
not bitwise. `Operational` (its meaning names the algorithm and settings) and `Relation` must refuse.

**Replacement rule.** A nested evaluation may start from the previous trial's root (or an IFT-predicted
one, y + (dy/dp)Δp from the derivative already computed) when all of the following hold:
1. the selection meaning is start-independent as above;
2. the inner tolerance is tighter than the outer final tolerances by a declared factor;
3. the evaluation used for **qualification and publication** re-solves from the canonical configured
   start, or re-verifies the selection predicate and isolation at the warm root;
4. the warm source is recorded.

**Related costs.**
- Inner accuracy equals final accuracy today (C6). That is correct for qualification, but trial
  evaluations could use the declared tighter-than-outer contract (DR-1 H8).
- Each evaluation factors twice: KLU inside KINSOL, then faer for the IFT.

**Verification.** Fixture: an outer solve with an implicit Unique cubic root. The published result
with and without warm inner starts is identical after canonical qualification. An `Operational`
selection refuses the warm start.

### DM-F08: Ipopt C API cannot re-optimize; `warm_start_same_structure` is unreserved but fails

**Severity.** Low. G7. *Interface-checked* (`IpStdCInterface.cpp`: a new `StdInterfaceTNLP` and
`OptimizeTNLP` on every `IpoptSolve`).

**Correction direction.**
- Reserve `warm_start_same_structure` and `warm_start_entire_iterate`, so a raw setting refuses before
  native execution.
- Treat retained-information support as a per-adapter capability fact for sequence steps (DR-1 / S):
  POUNCE has the SQP working set, IterateSnapshot and sens-core.
- The ReOptimize shim is optional (§5).

### DM-F09: POUNCE solves rebuild FERAL symbolic analysis

**Severity.** Low to medium. *Implemented*/uncertain.

**Cause.** `pounce.rs` installs a fresh `default_backend_factory_with_sink` per solve. The factory
creates a new `FeralSolverInterface` per algorithm build, so `last_pattern_fingerprint` reuse is lost
across solves.

**Correction direction.** Determine whether POUNCE honours `warm_start_same_structure` by keeping its KKT
solver (U3). If not, an upstream request. Do not build a pse-side cache of FERAL internals.

### DM-F10: "No hidden second solve" also forbids declared, library-owned retry trajectories

**Severity.** Medium for SC-05. PS-08, PS-10. *Interface-checked*.

**Cause.** `PINNED_OFF` forces `mu_strategy_fallback` and `dual_divergence_retry` off, and the ladder is
never used. Yet `pounce_algorithm::second_opinion::second_opinion_rungs(availability)` is a **public
pure function**:
- it returns rung definitions (`feral_scaling=mc64`, `mu_strategy=adaptive`,
  `start_point_perturbation=1e-2`, `feral_increase_quality=no`);
- each rung is gated on a typed trigger (local infeasibility, invalid number, restoration failure,
  iteration limit only after a quality escalation).

That is exactly a declared, fact- and observation-gated escalation policy owned by the library.

**Correction direction.**
- Consume `second_opinion_rungs` as rung content. pse runs each rung as a recorded attempt through the
  normal adapter: typed settings, budget charge, receipt.
- Do **not** call `run_second_opinion_ladder`. It reports only rung labels and iteration counts, not
  per-rung terminations and candidates, so its attempts would remain partly hidden.
- `mu_strategy_fallback` and `dual_divergence_retry`: if they are in-solve strategy switches rather than
  re-solves (U4), they are library globalization under PS-09. Admit them as typed options with
  statistics recorded.

### DM-F11: Authored homotopy uses fixed-fraction step control with no predictor or fold diagnosis

**Severity.** Medium. PS-08, PS-12, DP-12. *Implemented* (per pipeline evidence and §17.5).

**Cause.** The authored homotopy steps a linear fraction, grows the step ×1.5, halves on a retryable
failure down to 1e-6, and has no tangent and no fold indicator.

**Consequence.** At a turning point in λ, natural-parameter continuation fails by construction. pse
reports a step-size failure, halving to the minimum, rather than "fold encountered". Attribution is
wrong, and the remedy (arclength) is never reached.

**Correction direction.** The M9 tracker:
- tangent predictor from DM-F06's operator;
- Θ₀- and angle-based step control;
- fold indicator from the λ̇ sign and the factor sign;
- arclength corrector on the bordered system;
- the final original solve unchanged.

The current bespoke schedule is outer composition (not a G8 violation) but inadequate.

### DM-F12: Inexact-Newton machinery has no cost advantage, because the JVP assembles the full Jacobian and only Jacobi preconditioning exists

**Severity.** Low, against the target. *Implemented* (`assembly.rs` `jacobian_product`, per worker; not
re-read by me).

**Correction direction.**
- Symbolica-generated directional-derivative programs, so the derivative source stays exact.
- A block preconditioner from BTF diagonal blocks factored by KLU/faer (library factors).
- Admit Krylov only on the fill or matrix-free fact.

### DM-F13: KINSOL in-solve observations are cumulative-only; no typed abandonment exists

**Severity.** Medium against the target. *Implemented*.

**Correction direction.**
- Per-segment observations through iteration-bounded `KINSol` calls with `NoInitSetup(TRUE)` (factor
  preserved), or residual-callback-derived norms.
- A typed abandonment latch, distinct from cancellation, returned as −1 from the callbacks and as
  `false` from the Ipopt/POUNCE intermediate callbacks. It is recorded as a strategy abandonment, never
  as `User_Requested_Stop` or cancellation.

### DM-F14: `KIN_STEP_LT_STPTOL` maps to `Termination::Acceptable`

**Severity.** Low. PS-10. *Implemented*.

**Cause and consequence.** A small scaled step is a possible stall, not near-acceptance. Qualification
protects the result, but an escalation keyed on category would treat a stall as nearly solved.

**Correction direction.** A stall category, or Numerical for escalation purposes, leaving acceptance to
qualification.

### DM-F15: The POUNCE Schur hook falls back silently

**Severity.** Low; forward-looking. G7. *Interface-checked*.

`set_kkt_schur_block` "falls back to the standard full-space solver transparently". When adopted for
M17a, read and record whether the partition was honoured (FERAL/POUNCE statistics). If it cannot be
observed, request it upstream.

### How the findings relate

- DM-F01, DM-F04 and DM-F14 share a cause: the typed outcome does not carry the information an
  escalation needs. DR-1's escalation contract depends on these corrections, so they are prerequisites,
  not independent polish.
- DM-F03, DM-F06 and DM-F05 are the retained-information producers that DR-1's keyed
  retained-information model consumes.
- DM-F02 is a prerequisite for typed method profiles.
- DM-F07 depends on DR-1's intermediate-accuracy contract.

## 5. Library decisions (§F), G8, PS-G3 and PS-09

| Decision | Capability | Candidates and fit | Decision | Reason | Revisit trigger |
|---|---|---|---|---|---|
| **PETSc** (SNES NPC/composition, `TSPSEUDO`, VI bounds, NEWTONAL, NEWTONTR, TAO) | M7, M9, M11, M15 as library methods | Only library with the composition algebra and Ψtc. Per evidence-worker source reads (main, 3.26-dev, a lead): NEWTONTR, NEWTONAL and NGMRES **abort** on NaN/domain error; line-search types recover. Needs an MPIUNI build and a bespoke `-sys` crate (`petsc-rs` needs real MPI); a second sparse stack and options database; per-subdomain SNES/scatter wiring. | **Decline now** | The mechanisms PETSc would uniquely own are the ones whose failure semantics break H2. The ones that fit (TSPSEUDO, NEWTONLS-based NASM) are thin outer loops over library Newton solves that KINSOL/IDAS already provide with pse's typed recoverable-trial contract. A second nonlinear stack would duplicate KINSOL's globalization with different semantics. Keep the rung interfaces backend-agnostic so a PETSc root adapter could later be added under PSE-S02. | PETSc TR/AL/NGMRES/NASM treat `SNESSetFunctionDomainError` as step rejection; or a workload needs distributed memory; or ASPIN/FAS beyond NGS + Anderson is required by a fact (failure localized to blocks that NGS + Anderson cannot contract) |
| **Uno** (TR filter-SQP/SLP) | M7(b): trust-region NLP globalization with evaluation-error radius shrink | Only open TR filter SQP/SLP found. Per worker: C API with sparse COO plus HVP/JVP; non-zero callback return shrinks the TR radius; typed statuses; time limit; termination callback. BQPD for nonconvex QP (binary); HiGHS for SLP/convex QP; OpenMP off. | **Adopt as a new NLP adapter for a declared escalation rung** (*Proposed*) | Provides a globalization class (trust region, active set) absent from Ipopt/POUNCE (line-search filter). It is library-owned, so no bespoke TR loop is needed. | Qualification shows H2 or H6 fails, or BQPD acquisition fails (then SLP/HiGHS only); or POUNCE gains TR-SQP |
| **Ceres** | Bounded LM/dogleg on ½‖F‖² | Good invalid-step retry (docs). Normal-equation solvers square the condition number for square systems; sparse QR absent [mem]; residual-block mapping overhead; Rust crate lacks IterationCallback; Eigen/abseil/glog stack. | **Decline** | Its role is covered by the Ipopt/POUNCE bounded feasibility NLP (restoration ≈ ℓ1 + proximity) and Uno TR. No unique capability. | Uno declined; or large structured least squares (SPARSE_SCHUR) outside POUNCE Schur's reach |
| **`russell_nonlin`** | Continuation | Pseudo-arclength and step control, but H2 only via the secondary-state hook; no bounds; UMFPACK/MUMPS second sparse stack; `Arc<dyn Fn + Send + Sync>` callbacks against worker-owned evaluators. | **Decline** | The tracker must keep KINSOL/Ipopt correctors' recoverable trials, bounds and cancellation. The bespoke core (≈ 0.5–0.9k lines) is outer step control, not an inner Newton. | It gains recoverable trials and pluggable correctors/linear solvers |
| **egobox** | Surrogate construction | GP/MoE with values and gradients, serde; no model management anywhere. | **Adopt when M16 is implemented** | Construction is library-owned; management is a bounded bespoke loop of library NLP solves. | A Rust/C TR model-management library |
| **Ipopt C++ `ReOptimizeTNLP` shim** | M5′ same-structure re-optimization, entire-iterate warm start | C++ glue in `pse-ipopt-sys`. | **Admit, low priority** (*Proposed*) | Integration glue, not numerics. Pays only when sequences stay on Ipopt (linear-solver availability). Otherwise selection should prefer POUNCE for retained-information steps. | Ipopt C API adds re-optimize |
| **sIPOPT** | M1 | Redundant with `pounce-sens-core`. pse's own active-set μ = 0 factor is more accurate than the barrier factor (no O(μ) error). C++/AMPL-only path. | **Decline** | Redundant | — |
| **Ipopt inexact algorithm** | M4 for NLP | Requires Panua Pardiso's iterative mode (not linked). | **Decline for now** | No fact demands it. KINSOL Newton–Krylov covers roots. | KKT symbolic fill exceeds the memory allowance for an admitted NLP |
| **ARKODE/CVODE features** | Ψtc carrier | IDAS (DAE) and Diffsol (ODE with mass) already carry the authored-dynamics flow; SUNNonlinSol adds nothing over KINSOL. | **Decline** | No unique capability | An explicit/IMEX dynamics requirement |
| **POUNCE `solve_nlp_batch_parallel_warm`** | M17b | rayon global pool; `T: TNLP + Send + 'static` | **Decline** (use pse workers) | Conflicts with worker ownership, thread admission and memory caps (§18.8). | pse evaluators become `Send`, and the rayon pool is charged to the job |
| **POUNCE `set_kkt_schur_block`** | M17a | Serial, dense S, exact Hessian, transparent fallback | **Adopt** for multi-experiment estimation, with DM-F15 | Library-owned | Many linking variables |
| **POUNCE partitioned QN / FD Hessian** | M18 | Reserved today | **Adopt, typed** | Library-owned. Typing keeps PS-07 honest. | — |
| **`pounce-qp` parametric homotopy; HiGHS `qp_allow_hot_start`** | M1b for QP class; convex QP sequences | — | **Adopt** | Exact QP paths are library-owned | — |
| **faer sparse QR** | LM/Gauss–Newton factor | Only needed by a bespoke LM loop | **Decline as primary** | No bespoke LM placed | A bespoke Gauss–Newton rung becomes necessary |

**G8, within scope.**
- *Current state: pass.*
  - No reimplemented Newton, line search or factorization was found. The bespoke homotopy schedule and
    the profile secant are outer compositions.
  - `square_response` uses faer (dense where sparse fits). That is a fit issue (DM-F06), not bespoke
    numerics.
- *Target: passes only with the bounded bespoke items recorded through §F:*
  - tracker core;
  - SER controller;
  - NPC orchestration;
  - M1b outer loop;
  - TR model management.

  The decisive G8 risk is a bespoke LM/trust-region loop for M7. I recommend against it: Ipopt/POUNCE
  feasibility NLP plus Uno covers it.

**PS-G3, within scope.**
- *Current supported behaviour:*
  - **PS-10 fails narrowly** on DM-F01: a misattributed outcome on the FP/Picard recycle route. There
    is no false acceptance, because independent qualification holds.
  - DM-F14 is a minor labelling issue.
  - PS-06, PS-07 and PS-08 hold for current mechanisms. Guards become KINSOL sign constraints, there is
    no silent FD, and overlays are transactional.
- *Target mechanisms:* **unresolved** until their derived-system and accuracy contracts exist.
  The obligations in §3 are the PS-G3 content they must meet. In particular:
  - PS-07: exact derived derivatives, and a typed FD source when admitted;
  - PS-12: fold, branch and local-infeasibility honesty;
  - PS-08: a recorded seed class and provenance.

**PS-09 replacement rule.** The current text says "Established numerical solvers own iteration,
globalization and factorization; the simulator does not reimplement them." Proposed replacement:

> Established numerical solvers own the methods they implement: the step computation, step acceptance
> (line search, filter or trust region), barrier or active-set updates and factorization inside one
> solve. The simulator never reimplements such a method where a pinned or admissible library provides
> it with a fitting contract (recoverable trials, bounds, cancellation, typed outcomes). The simulator
> may compose library solves into declared outer strategies: prediction, continuation and homotopy
> tracking, pseudo-transient stepping, block and reduced-space decomposition, nonlinear preconditioning,
> model management and escalation. It may do so when:
> - (a) every inner solve and factorization is a library call;
> - (b) the outer step-acceptance and step-size rules are declared, bounded and recorded per attempt;
> - (c) the §F reason why no library supplies the outer method with a fitting contract is recorded;
> - (d) every outer result enters the pipeline only as a seed of a stated class, or is accepted by the
>   original-problem qualification.
>
> Bespoke linear algebra is limited to small dense operations: bordering, Schur complements of declared
> small dimension, and secant updates of a few vectors.

**Audit addition:** "Is each outer loop a composition of library solves with recorded rules, or does it
re-create a library's inner step?"

## 6. Escalation-ladder content (rungs; DR-1 owns the contract)

**Before any rung: start preparation.** None of the following adds an escalation:
- predict (M1, M1b, M3, secant; admitted by retained-information compatibility and regularity facts);
- else the accepted predecessor;
- else the specification;
- the start's evaluability is checked first, with start repair (projection into the guard/FBBT
  interior) if needed.

The rungs below are ordered by the **mathematical assumptions they add about the problem**, not by
expected speed. A plan skips any rung whose admission facts fail; "no escalation" is a valid plan.

| Rung | Mechanisms | Assumptions added | Admit (facts) | Enter on (typed outcome / observation) | Abandon |
|---|---|---|---|---|---|
| R0 | Class-selected adapter, default profile | — | class and capability | — | any typed failure → next admitted rung |
| R1 | Same library, different trajectory: KINSOL profile (`setup_interval=1`, fact-derived `max_newton_step`); Ipopt `start_with_resto`, `least_square_init_*`, adaptive μ; POUNCE `second_opinion_rungs` (mc64 scaling, adaptive μ, displaced start, no quality escalation) | none (same equations, same method family) | the rung's library gate (`SecondOpinionTrigger`, Ipopt/POUNCE status) | `LINESEARCH_NONCONV/BCFAIL`, `REPTD_SYSFUNC_ERR`, `Restoration_Failed`, local infeasibility, invalid number, iteration limit **with** a quality-escalation observation | same outcome class repeats |
| R2 | Same equations, different method class: square root → bounded feasibility NLP (Ipopt/POUNCE); NLP IPM → POUNCE active-set SQP; → **Uno TR-SQP/SLP** | none mathematically; the stopping meaning changes ("locally infeasible / LS-stationary" possible) | bounds/domain facts; C² for SQP | R1 exhausted; evaluation failure with trial-rejection evidence (DM-F04); restoration failure | local infeasibility repeated; TR radius collapse; Uno `EVALUATION_ERROR` |
| R3 | Ordering only: BTF block-sequential (M13), block-level R0–R2 | exact incidence; per-block numeric regularity at visited points | empty DM over/under parts; numeric qualification per block | failures not localized by R1–R2 | block singular → merge or simultaneous; predecessor-induced block failure |
| R4 | Local elimination / nonlinear preconditioning (M14, NGS + Anderson M6/M15) | F₁,y nonsingular over the visited region; inner uniqueness (or a declared local branch); contractive sweep | qualified partition; bound-inactive eliminated set; declared map | failures localized to the same block across attempts (observation) | repeated inner failure; sweep residual ratio ≥ 1 |
| R5 | Physical-parameter continuation (M9) | a connected solution path in an authored λ; C¹ in λ | authored parameter with exact F_λ; no regime switch on the range | basin failures with an authored parameter available | Δs_min; λ moving away after a fold beyond the arclength budget; domain errors at Δs_min |
| R6 | Constructed homotopies (M10: Newton, fixed-point A = D·P_M, affine, model, bounded) | bounded, transversal path (unverifiable; heuristic label) | evaluable anchor; sparsity-preserving construction; for the model form, a recorded low-fidelity model | R5 inadmissible or failed | path returns to t = 0; ‖x‖ → ∞; budget |
| R7 | Constructed flows Ψtc (M11) | attracting equilibrium of the chosen flow | authored dynamics (IDAS) or a matching pairing (heuristic); index-1 | R6 failed, or singular Jacobian with an invariant flagged | δ stagnation; residual growth at large δ |
| ✱ | Multistart (M17b) | none per start; "best accepted" is local | finite sampling box | at any rung, varies anchors/starts | budget |
| ✱ | Surrogate seed (M16) | seed only | declared low-fidelity model | start preparation, or the R6 model homotopy | ρ ≤ 0 repeatedly |

**Never escalate on:**
- resource exhaustion, cancellation, infrastructure failure, panic or contract (`Invalid`) errors;
- a presolve-certified infeasibility re-verified in original coordinates;
- an iteration limit without trajectory evidence. The fact-based response is a budget decision, not a
  method change: compare the POUNCE gh#857 gating.

**Every rung's terminal output** re-enters as a seed of its class, and the final candidate always comes
from a solve of the unchanged original specification under final accuracy.

## 7. Uncertainties and contract assumptions needed from DR-1

**Uncertainties.**
- **U1.** `step_along_path`'s activity, `weak_rows` and release semantics on pse's active-set μ = 0
  factor, and inequality-row entry via `rowlimit` without a slack block. Decides DM-F05's remedy
  effort, not its diagnosis. Settling evidence: a QP fixture where the active set changes.
- **U2.** Uno's C API, evaluation-error semantics, determinism and BQPD acquisition were read by the
  evidence worker, not by me. It decides the R2 TR rung. Settle with a native qualification probe at
  scope end.
- **U3.** Whether POUNCE keeps its KKT solver under `warm_start_same_structure` (DM-F09).
- **U4.** Whether `mu_strategy_fallback` and `dual_divergence_retry` are in-solve switches or re-solves
  (DM-F10).
- **U5.** PETSc failure semantics were read from `main`, not a tagged release; TSPSEUDO's handling of an
  inner SNES failure is unverified. This could reopen the PETSc decision only in PETSc's favour for M11.
- **U6.** Whether KINSOL scales the Anderson least-squares by `fscale`. Affects M6 scaling.
- **U7.** The JVP being assemble-then-multiply rests on the worker's reading (`assembly.rs`).
- **U8.** The mechanism literature in the evidence cards is largely abstract-level. Placements here rest
  on library contracts and standard results, not on those papers' specifics.

**Contract assumptions my placements need from DR-1.**
- **A1. Derived-system realization with exact derivatives.** Compiled through the same Symbolica
  pipeline and keyed by the original prepared identity. It covers:
  - homotopy H(x,t), H_x, H_t;
  - bordered (n+1) systems with one dense row and column;
  - shifted systems V(x − x_k)/δ + F;
  - block sub-evaluators (row/column restriction with predecessor values as inputs);
  - the DAE-IC algebraic subsystem.

  It must carry its incidence delta, because BTF/matching must be recomputed on the derived incidence.
- **A2. Keyed retained numerical information**, beyond the single `Retained` slot. Entries:
  - active-set KKT factor;
  - root Jacobian factor at x\*;
  - KINSOL session setup;
  - branch history (p, x) pairs;
  - tangent orientation.

  Each carries a stamp: layout, profile, data at formation, the point formed, the accuracy regime (μ,
  δ_w, δ_c, tolerance), inertia/rank and activity margins. The prediction operation is consumed by
  studies, continuation, fitting, horizon and edit-and-re-solve. Worker-owned; it never crosses threads.
- **A3. Start proposal.**
  - Seed class: predicted, auxiliary-path, physical neighbour labelled with λ, exact partial,
    modified specification, surrogate.
  - Validity limits: Δp radius, assumed active set, branch identity.
  - Transport class: semantic primal versus backend duals, working set or factor.
  - An evaluability pre-check.
- **A4. Typed observations readable at rung boundaries,** plus a typed abandonment latch distinct from
  cancellation, through the KINSOL callbacks (−1) and the Ipopt/POUNCE intermediate callbacks
  (`false`).
- **A5. Escalation triggers on typed outcomes.** These must distinguish:
  - evaluation-with-trial-evidence;
  - local infeasibility;
  - restoration failure;
  - line-search non-convergence;
  - stall (STPTOL);
  - iteration limit with or without trajectory evidence.

  This needs DM-F01, DM-F04 and DM-F14 first.
- **A6. Intermediate accuracy contract.** Corrector or path tolerance; inner implicit tolerance as a
  declared factor tighter than the outer; forcing terms. The final `ResolvedAccuracy` stays frozen.
- **A7. Typed method profiles** with per-strategy applicability validation (DM-F02) and identity, so
  that no-op settings do not alter the stamp.
- **A8. Ordered sequences** that mark each problem intermediate or final, with normalized proximity
  |Δp| from parameter scales.
- **A9. One budget charged across rungs,** including predictor evaluations, screening, refactors for
  releases, and rejected predictions.
- **A10. Nested implicit selection meaning** (`ImplicitMeaning`, `SelectionNeighborhood`) visible to the
  evaluation-start policy (DM-F07).
- **A11. Receipts recording** numeric-setup reuse, retained-factor use, prediction mechanism and
  segments, and ladder rung identity.
- **A12. Adapter capability facts for retained information** (warm, setup reuse, sensitivity, Schur,
  same-structure), so that selection for sequence steps can rest on capability rather than measured
  speed.

## Scenarios (stable IDs)

| ID | Scenario | Mechanisms engaged | Current blocker | Key facts / observations |
|---|---|---|---|---|
| DM-S01 | NLP sweep with an active-set change | M1 → M1b (`step_along_path`) → warm POUNCE SQP corrector; `pounce-qp` for QP class | DM-F05 (refusal; single consumer) | certified inertia/LICQ/weak rows; breakpoints; prediction vs base residual |
| DM-S02 | Root feed sweep | M3 tangent/secant + KINSOL `KeepPrevious` setup; fold check | DM-F06 (dense, no retention), DM-F03 | regular root; Θ₀; det-sign |
| DM-S03 | Fitting outer loop | M17a POUNCE Schur over θ; M18 (partitioned QN when only first-order providers exist); M1 for profile chains | fit solves cold, no Schur | parameter identity as linking set; exact Hessian availability |
| DM-S04 | Rolling-horizon regression | M2 with M1b instead of a full re-solve on active-set change | DM-F05 | active-set breakpoints; one-corrector contraction |
| DM-S05 | Hard cold start near invalid domains | start evaluability + repair; R1 (fact-derived KINSOL max step, Ipopt `start_with_resto`); R2 bounded feasibility NLP / Uno TR; R6 bounded homotopy | DM-F04, DM-F10, DM-F14 | FBBT enclosures, guard signs; trial-rejection evidence |
| DM-S06 | Large BTF flowsheet with tears and a locally stiff block | M13 → per-block R1/R2 → M14 elimination of the stiff block, or NGS + Anderson (M15) | DM-F01 (FP semantics) | per-block numeric qualification; failure localization |
| DM-S07 | Expensive nested property evaluation | DM-F07 warm inner starts; M18 constant flags and partitioned QN; inner tolerance contract | DM-F07, C6 | selection meaning; callback cost share (trigger only) |
| DM-S08 | DAE re-initialization after a discontinuity | M12 IDACalcIC tuning → algebraic-subsystem KINSOL/M13 → least-deviation NLP (labelled) | IC tuning unexposed | mode identity; index and DOF re-analysis |
| DM-S09 | Multi-experiment estimation | M17a Schur (θ linking), M1 covariance via the existing reduced Hessian | Schur hook unused; DM-F15 | few linking variables; exact Hessian |
| DM-S10 | Replacing or adding a root solver library (PSE-S02) | rungs consume a corrector interface (root/NLP adapter), not KINSOL specifics, so a PETSc or Uno adapter slots in | KINSOL-specific semantics (FP returns, `NoInitSetup`) leak into strategy assumptions unless typed as capability facts (A12) | adapter capability: recoverable-trial semantics per strategy, setup reuse, bounds class |
