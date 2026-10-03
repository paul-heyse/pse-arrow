# Mechanism matrix and current-state synthesis

Coordinator synthesis for the solver acceleration and globalization review (principal document:
`docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md`).
It consolidates the wave-1 evidence in this folder into (1) current-state observations,
(2) a mechanism × pipeline matrix, (3) the rules that conflict with the target and their intent,
and (4) candidate contracts. Sections 3–4 are **hypotheses for independent assessment, not
verdicts**. Baseline: see [README](README.md). Evidence pointers name the file in this folder
that carries the source citation; items marked *verified* were re-read by the coordinator.

Classes used below: **L-pinned-used**, **L-pinned-unused** (a pinned library provides it; pse
does not use it), **L-candidate** (an unpinned library provides it), **C** (composition of
pinned-library solves; project owns sequencing/admission), **B** (bespoke numerical code,
no library fit found).

## 1. Current-state observations

| ID | Observation | Evidence |
|---|---|---|
| C1 | One staged executor (`workflow::staged` → `NativeSession::step` → `MathService::execute`) serves single solves, authored sequences, stage+homotopy initialization, studies and the rolling horizon. Block initialization, recycles (`solve_declared_root` drives KINSOL directly), conditional units, nested implicit solves, fitting/profile chains, shooting and the fixed-assignment re-solve run their own loops with a fresh `Retained`. | pipeline-map §composing workflows |
| C2 | Multi-solve policy is duplicated: continuation/step control three times (homotopy grow ×1.5/halve to 1e-6; fit profile chains; horizon predict-or-solve); accept/commit rules in three variants (`commit_block`, `completion.permits_use`, the session keep flag); routing `Requirements` hand-built in five places; start admission restated per workflow. | pipeline-map §seams (d)(e), §duplicated policy |
| C3 | Automatic selection = most-specific problem class, then fixed adapter rank (KINSOL, HiGHS, Ipopt, POUNCE, Clarabel, SCIP); within an adapter, native defaults. No method/setting is chosen from facts (sole exception: POUNCE ℓ1 under authored `penalty(l1)`). The sensitivity-capable preference is applied but not recorded as a reason. | controls-facts-metrics §2; pipeline-map §automatic choices |
| C4 | Facts relevant to strategy exist but never reach selection: BTF/DM blocks, conditioning estimates (KKT/diagnostics only), FBBT/guard/domain facts, compatible retained state, parameter distance between cases, problem size. | controls-facts-metrics §2 |
| C5 | Rich per-attempt observations are recorded (callback counts/time incl. rejected trials, regime crossings, KINSOL counters, Ipopt per-iteration events, POUNCE statistics) and published, but no runtime decision consumes them except retryability-by-termination-category in homotopy. No cross-step work budget for authored sequences and studies. | controls-facts-metrics §3; pipeline-map §(f) |
| C6 | No intermediate-accuracy tier: inner implicit solves use final budgets; homotopy/stage steps use the final policy; KINSOL forcing terms inactive under default KLU; Ipopt `acceptable` absent by default. | controls-facts-metrics §5 |
| C7 | Cross-solve information is limited to previous-solution seeds and native warm starts. `kkt::Advance`/`predict` has one consumer (horizon), one slot, parameter-id compatibility only, Optimize intent only, refuses on any active-set change. Root `square_response` computes `X_p` (dense SVD + LU) but is not retained or used to predict. No tangent/secant state predictor anywhere. Fitting solves start cold each iteration (`warm: None`). | reuse-structure-map Part A; *verified* `predict` consumer `horizon/driver.rs:1252` |
| C8 | Factor/setup reuse across solves is absent: KINSOL `KINSetNoInitSetup(0)` forces a numeric setup every solve (*verified* `kinsol.rs:1084`); FERAL's symbolic analysis is rebuilt per POUNCE solve; Ipopt's C API always runs `OptimizeTNLP` (*verified* `IpStdCInterface.cpp:273`), so same-structure re-optimization is unreachable. | pinned-library-capabilities |
| C9 | Structure (matching, DM, BTF) gates admission at solve time and schedules block initialization (each block recompiled as its own conditional case); it never reorders, partitions or reduces a main solve. Tear selection is a tool step whose result is passed by hand into `RecycleRequest`. Authored nested implicit functions are effectively reduced-space solving (inner KINSOL + implicit-function derivatives) but are authored only and re-solve from the configured start every trial. | reuse-structure-map Part B |
| C10 | Globalization is entirely inside libraries (Ipopt filter line search/restoration, POUNCE, KINSOL line search) plus authored parameter homotopy with a fixed fraction schedule. No derived homotopy, pseudo-transient, trust-region/LM, arclength or escalation exists. | reuse-structure-map absence claims |
| C11 | Library capability unused or untyped: KINSOL `msbsetsub`, residual monitoring, `NoInitSetup(true)`; Ipopt least-squares primal/dual initialization, `start_with_resto`, adaptive `mu_strategy` (raw-reachable only); POUNCE `step_along_path`/`path_direction`, `pounce-qp` parametric homotopy, Schur KKT block, parallel NLP batch with warm starts, partitioned quasi-Newton (reserved), second-opinion ladder (pinned off as "no hidden second solve"); IDAS IC tuning; faer sparse QR; ARKODE/CVODE features in vendored SUNDIALS. sIPOPT and Ipopt's inexact algorithm are not in the build. | pinned-library-capabilities; candidate-libraries |
| C12 | **Defect (source-traced, not executed).** KINSOL fixed-point and Picard modes stop only on negative callback returns (`KINFP`/`KINPicardAA` test `retval < 0`); pse returns `1` for every recoverable trial in all modes (`kinsol.rs:444-451`) and publishes outputs only after success. A recoverable domain/envelope failure in the recycle route therefore leaves KINSOL iterating on a stale map value; an undamped iteration can report native convergence that the original-residual check then refuses, misclassifying an evaluation failure. | *verified* `sundials-sys` `kinsol.c` KINFP; `kinsol.rs:444-470` |
| C13 | Native-free testing covers routing/selection and candidate use; start admission, `Staged` seeding and homotopy/stage/retry policy are testable only behind `solver-kinsol`, because `Staged` → `NativeSession` → `MathService` hard-wires the linked adapter table. | pipeline-map §testability |
| C14 | Documentation divergence: §17.6 describes `Start::Seed(k)` and value-only study seeding; the code has `Start::{Specification, Accepted}` and studies submit the predecessor's full native `WarmStart`. Minor: the session keep flag uses model checks, not the candidate decision, contrary to its doc comment (`math/staged.rs:293-326`, *verified*); Ipopt `warm_start_same_structure` is not reserved although the C API cannot honour it. | pipeline-map; *verified* |

**Preserved strengths (constraints on any remedy).** Original-coordinate acceptance with typed
`CandidateUse` as the single permission owner; resolved numerical policy and `ResolvedAccuracy`
not being a user control; typed `NativeTermination` and independent qualification; the callback
boundary's typed recoverable/terminal distinction; four-part `Compatibility` stamps and typed
`StartReceipt`/`SeedTransformation` lineage; serde-derived settings identity; transactional
overlays that never mutate the specification.

## 2. Mechanism × pipeline matrix

Prerequisites and triggers summarize the mechanism cards (mechanisms-*.md). "Seam" names the
current owner a mechanism would attach to.

| # | Mechanism | Prerequisite facts | In-solve triggers / abandonment | Provider class | Current state | Seam |
|---|---|---|---|---|---|---|
| M1 | NLP parametric sensitivity prediction | Certified KKT point (LICQ, SOSC, strict complementarity); retained factor; parameter change Δp | Predicted active-set change; step-size limit; original-problem evaluation of the prediction | L-pinned-used (`pounce-sens-core` + FERAL backsolver) | Horizon-only; one slot | `kkt::advance`, `Retained`, `Staged` |
| M1b | Active-set-changing path following | As M1 plus piecewise regularity | Breakpoint detection; degenerate crossings | L-pinned-unused (`step_along_path`, `path_direction`, `refine_step_onto_bounds`; `pounce-qp` homotopy) | Absent; `predict` refuses | as M1 |
| M2 | Advanced-step / real-time iteration (limited correction) | Sequence of nearby problems; retained factor | Correction residual; contraction | C over M1 + native solve | Horizon advanced step only | horizon driver → generic sequence |
| M3 | Root tangent/secant predictor | Regular root (`F_x` nonsingular), `F_p`; or two prior solutions | Prediction residual; Newton contraction from predicted point | C (`square_response` math + sparse LU) | `X_p` computed, not retained/used | `square_response`, studies, homotopy |
| M4 | Inexact Newton–Krylov, Eisenstat–Walker forcing, preconditioning, JFNK | Large/sparse system; JVP; preconditioner availability | Linear iterations per Newton step; forcing stagnation | L-pinned-used-manually (KINSOL SPGMR/SPFGMR…, EW choice 1/2); Ipopt inexact not in build | Typed, never auto-chosen; JVP assembles full Jacobian; Jacobi only | KINSOL settings, planner |
| M5 | Jacobian/factor/setup reuse; modified Newton | Same layout/profile; slowly varying Jacobian | Residual-monitoring test; convergence rate drop | L-pinned-unused (KINSOL `NoInitSetup`, `msbsetsub`, res-mon; FERAL symbolic retention; Ipopt same-structure via C++ `ReOptimizeTNLP`) | Setup forced every solve | adapters, `Retained` |
| M6 | Anderson / fixed-point acceleration | Admitted contractive map or justified splitting | Residual growth; history conditioning | L-pinned-used (KINSOL FP+AA, Picard) | Recycle route only; C12 defect | causal maps, NPC |
| M7 | Trust-region / Levenberg–Marquardt local models; bounded regularized LS/QP preparation; SLP/SQP-filter auxiliaries | Evaluable start; Jacobian; bounds | Ratio ρ of actual/predicted reduction; stationary non-root | L-candidate (Ceres LM/dogleg; Uno trust-region SQP/SLP); L-pinned-unused (`pounce-qp`, faer sparse QR); C (POUNCE/Ipopt on ½‖F‖² or lifted QP via HiGHS/Clarabel) | Absent | planner, derived system |
| M8 | Library-native NLP globalization/warm-start options | NLP; prior iterate | Restoration entry; barrier stagnation | L-pinned (Ipopt LS init, `start_with_resto`, adaptive μ; POUNCE second-opinion ladder) | Raw-reachable or pinned off | typed adapter settings, planner |
| M9 | Natural-parameter / pseudo-arclength continuation | Authored parameter with `∂F/∂λ`; C¹ path | Corrector failure at min step; λ̇ sign change (fold); bordered determinant sign (branch); ‖x‖ growth | C + small B core (predictor, step control, bordering); L-candidate partial (`russell_nonlin`; PETSc `SNESNEWTONAL` aborts on NaN) | Authored-endpoint homotopy, linear fraction schedule, no predictor | `engines.rs` homotopy → generic continuation |
| M10 | Constructed homotopies (Newton, fixed-point, affine, bounded/sparse, model) | Evaluable anchor; transversality (rarely verifiable); sparsity-preserving `A` | Path return, path to infinity, domain exit, isolas | C + B (construction ~300–600 lines; Symbolica derives residual and derivatives) | Absent; overlays change values only | derived-system realization |
| M11 | Pseudo-transient continuation (Ψtc, SER) | A justified flow: authored dynamics (strongest), matching pairing, or gradient flow; target attraction | δ stagnation; residual growth at large δ | L-candidate (PETSc `TSPSEUDO`); C (KINSOL on `M(x−x_k)/Δt + F`, or IDAS/Diffsol to steady state) + ~100–250 lines SER | Absent | derived system; dynamics adapters |
| M12 | DAE consistent initialization / reinit | Semi-explicit index 1; identity vector | IDACalcIC failure codes | L-pinned-used (IDACalcIC); IC tuning unexposed; escalation C | Used without tuning/escalation | dynamics adapters |
| M13 | Block-sequential exact solve (BTF/tearing) | Empty DM over/under parts; exact incidence; numerical qualification per block | Block singular at point; block failure from predecessor values → merge/simultaneous | C (+ `pounce-presolve` block solves, L-pinned-used inside NLP presolve) | Initialization only | structural plan → main solve |
| M14 | Reduced-space / nonlinear elimination | `F₁,y` nonsingular over visited region; bound-inactive eliminated vars; inner tolerance tighter than outer | Inner failure → outer evaluation error | C (outer Ipopt/POUNCE, inner KINSOL; existing implicit-function machinery) | Authored implicit functions only | `pse-math::implicit`, derived system |
| M15 | Nonlinear preconditioning / composition (NPC, ASPIN, NASM, NGMRES, block NGS) | Decomposable nonlinear structure | Failure localized to blocks; outer stagnation | L-candidate (PETSc SNES composition; uneven NaN handling); C+B (~600–1,200 lines over KINSOL, limited by C12-type semantics) | Absent beyond recycle FP | planner, structural plan |
| M16 | Multifidelity / surrogate model management | Declared low-fidelity model or surrogate over identical identities; first-order consistency | Ratio ρ ≤ 0 repeatedly; callback-cost share (observation) | L-candidate (egobox construction); B (management ~600–1,000 lines) | Absent | start proposal producer; derived system |
| M17 | Multi-scenario Schur decomposition; parallel batch; multistart | Linking variables; nonsingular blocks | Schur conditioning | L-pinned-unused (`pounce-feral` Schur, `solve_nlp_batch_parallel_warm`); C (multistart) | POUNCE-convex batch only | fitting, studies |
| M18 | Evaluation reduction (inner warm starts, structure-preserving quasi-Newton, constant-derivative flags, forward-mode JVP) | Uniqueness of inner root (for history-dependent starts); derivative structure | Inner iteration counts; callback cost share | L-pinned-used (Ipopt constant flags, L-BFGS); L-pinned-unused (POUNCE partitioned QN); C/B (inner warm start; Symbolica directional derivatives) | Inner solves restart from configured start by design | `pse-math::implicit`, Symbolica evaluators |
| S | Strategy selection, escalation, budget, accuracy tiers | All facts above | All observations above | C (project-owned policy) | Class+rank routing only; no escalation; no cross-step budget | routing, `Staged`, numerics |

## 3. Rules that conflict with the target, and their intent

The maintainer treats these as heuristics to be replaced by rules that keep their intent
(2026-10-03). Intent statements below are the coordinator's reading of the cited authority.

| Rule (authority) | Intent it protects | Conflict |
|---|---|---|
| No backend fallback after native failure; reconsideration only on newly established incompatibility (§18.7, ADR-0152) | Deterministic, explainable routing; every attempt recorded; no ungoverned retries after resource/cancellation/infrastructure failure (G5); no silent capability claims (G7) | Solvability cannot be inferred before solving; escalation to a globalization rung or another method after a typed numerical failure is required |
| Homotopy changes values only; overlays over immutable specification (§17.5, ADR-0093) | One model authority; transactional, restorable initialization (DP-05, PS-08) | Derived equation systems (M7, M10, M11, M14, M15, bordered M9) change rows |
| Libraries own iteration, globalization and factorization; the simulator does not reimplement them (PS-09, §17 intro, ADR-0083) | No bespoke Newton/line search/factorization where a qualified solver provides it (G8) | Path tracking, SER control, trust-region model management and NPC have no linkable library meeting the trial-failure contract; they are compositions of library solves, not reimplementations of a library's inner loop |
| "No hidden second solves" (POUNCE retry ladder and fallback options pinned off) | Truthful, recorded attempts (PS-10) | The ladder is useful when declared as recorded attempts |
| Nested implicit solves never use a previous trial as start (`implicit.rs`) | An evaluation is a function of its inputs (no history-dependent root selection) | Inner warm starts are a major evaluation-reduction lever when root uniqueness is established |
| Stopping budgets are not user controls (§16.6) | One authoritative accuracy | Intermediate accuracy (forcing terms, corrector tolerances, inner tolerances) needs its own contract that never touches final budgets |

## 4. Candidate contracts (hypotheses)

From the colleague's input, the mechanism cards and the coordinator's synthesis. Each needs an
owner, a consumer list and a challenge case from the reviewers.

- **H1 Solve strategy (plan).** A typed, deterministic plan resolved before execution from
  intent, problem facts, available retained information and policy: an ordered set of
  admitted rungs (predict → prepare/globalize → correct → qualify), each with its mechanism,
  derived-system reference, method profile, budget share and abandonment criteria. Pure and
  testable without native solvers; recorded with reasons. "None" is a valid plan.
- **H2 Escalation rule (replaces no-fallback).** Escalation is declared in the plan,
  deterministic given facts and typed outcomes, bounded by the strategy budget, triggered only
  by typed numerical outcomes (never by resource exhaustion, cancellation, infrastructure
  failure or panic), recorded per attempt, and every rung's candidate answers to the same
  original-problem qualification.
- **H3 Start proposal.** One information model for initial points: provenance kind
  (specification, accepted predecessor, predicted (sensitivity/tangent/secant/path),
  auxiliary-path terminal, stored, surrogate), validity limits (Δp, assumed active set),
  coordinate transport (semantic primal vs native state), permission "start only". Extends
  `StartSource`/`WarmStart`/`StartReceipt` rather than overloading `CandidateUse`.
- **H4 Derived system (replaces value-only continuation).** A disposable realization with
  original prepared identity, enumerated generation rule, anchor, path parameter, affected
  rows/incidence delta, coordinate map, terminal-identity statement and budget, compiled
  through the same Symbolica pipeline; usable only while its original identity is in force;
  its candidates are auxiliary (seed-only) except at the terminal identity.
- **H5 Retained numerical information.** Generalize the single `Retained` slot to keyed
  retained information (factors, sensitivities, tangents, histories) with complete
  compatibility keys (DP-09), and a prediction operation consumed by studies, continuation,
  fitting, horizon and edit-and-re-solve.
- **H6 Method profiles.** Typed, policy-chosen native method profiles (KINSOL direct vs
  Krylov+EW+preconditioner, modified Newton/setup reuse, Anderson; Ipopt initialization,
  restoration and barrier options; POUNCE path following, Schur, batch) instead of raw
  passthrough, so they enter selection, identity and recording.
- **H7 Observation and budget model.** Typed in-solve observations exposed at rung boundaries;
  one work budget per strategy (time, evaluations, factorizations) charged across rungs.
- **H8 Intermediate accuracy.** Per-rung accuracy contracts (prediction error budget, corrector
  tolerance, inner tolerance relative to outer, forcing terms) distinct from the frozen final
  `ResolvedAccuracy`; derivative-consistency obligations for inexact evaluation.
- **H9 One multi-solve primitive.** Homotopy, profile chains, horizon, block initialization,
  recycles, fitting and studies compose the same staged primitive and strategy plan rather than
  re-deciding continuation, acceptance and routing.
- **H10 Library decisions to settle.** PETSc (SNES composition, TSPSEUDO, VI bounds) versus
  composition over KINSOL; Uno (trust-region SQP/SLP); Ceres (LM/dogleg); egobox (surrogates);
  an Ipopt C++ shim for `ReOptimizeTNLP`; enabling sIPOPT in the image (likely redundant with
  `pounce-sens-core`).
