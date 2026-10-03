---
title: Solver acceleration and globalization
date: 2026-10-03
tier: design
purpose: target
standard: core-3.3
profile: process-simulator-1.3
baseline: f0b902589723a86bc6755ce1c76b45a2ef90224b (plus the uncommitted Plan 25k working tree)
evidence: Interface-checked
decision: Revise
---

# Solver acceleration and globalization

**Revise.** The solver pipeline executes one attempt well. Routing chooses an adapter from
contextual capability, the library owns every iteration, native termination is typed, and an
independent original-coordinate assessment decides candidate use. The pipeline has no owner for
what happens *between* attempts:

- which start to use and where it came from;
- what to retain and predict from;
- which method profile to apply;
- whether to solve an easier related problem first;
- when to escalate and how to spend the work budget.

Each multi-solve workflow decides these things for itself, or not at all. The pinned libraries
already provide much of what the target needs, and the pipeline leaves it unused:

- factor and setup reuse;
- sensitivity path following through active-set changes;
- Schur decomposition;
- typed globalization options;
- declared retry trajectories.

The target is achievable with a modest amount of bespoke numerical code. The new contracts the
pipeline needs are:

- a strategy plan resolved from mathematical facts;
- a declared escalation rule;
- a start-proposal information model;
- derived systems with recorded provenance;
- keyed retained numerical information;
- typed attempt observations with one work budget;
- intermediate accuracy tiers.

Every inner solve and factorization stays a library call. Project code owns the outer sequencing
and a few bounded controllers that no library offers with a fitting failure contract: a path
tracker, a pseudo-time step controller, block-sweep orchestration, a path-following outer loop and
surrogate model management.

The two judgments, kept separate:

- **Architectural fitness fails G9 against the target.** All six foundations are violated for the
  representative change "add an acceleration or globalization mechanism" (§[Foundations and
  gates](#foundations-and-gates)).
- **Behavioral adequacy of current supported behavior fails narrowly.**
  - KINSOL's fixed-point and Picard modes can report native success at a point where the map
    could not be evaluated. Independent qualification refuses the candidate, so this is a
    misclassified outcome, not a false acceptance (F01).
  - Typed KINSOL settings that the selected strategy ignores are accepted and recorded as though
    they acted (F03).
  - Retention and continuation use competing acceptance definitions (F04).
  - The retained-predictor reuse key is incomplete (F11).
- **The target mechanisms are unresolved**, because none exists yet.

The target architecture in this review is *Proposed*. Accepting it as a design would not qualify
an implementation.

## Review boundary, baseline and evidence

The review applies [Core 3.3](../design_principles/core/design-principles.md), its
[template](../design_principles/core/design-review-template.md),
[ProcessSimulator 1.3](../design_principles/profiles/process-simulator/principles.md) and the
[repository binding](../design_principles/binding/pse-arrow.md). It is a subsystem
**design-tier, target-purpose** review.

The boundary runs from an admitted, prepared problem through to the published candidate decision:

- preparation and reuse (§14);
- structural analysis and KKT/sensitivity analysis (§15, §15.5.1);
- numerical policy (§16);
- initialization, continuation and starts (§17);
- native class-specific execution and routing (§18).

It includes the consumers that repeat nearby solves: studies (§19.3), fitting (§19.4), the rolling
horizon and dynamic starts (§13.6), flowsheet recycles (§17.4) and nested implicit evaluation
(§18.10). The boundaries of discrete and global routes (SCIP, HiGHS MIP) and of the integrators are
assessed only where the shared contracts touch them. Their internal acceleration is out of depth.

**Baseline.** HEAD `f0b90258` plus the uncommitted Plan 25k working tree, read as-is. The
`git diff` SHA-256 at review start was `1cee9b65…a232` (49 files). A concurrent session was editing
during the review; it touched Plan 25k files and drafted ADR-0153. Source citations refer to the
tree as read, and line numbers may drift.

**Method.**
- **Evidence.** Seven read-only evidence workers produced the
  [evidence folder](../evidence/solver-acceleration-2026-10-03/README.md):
  - three code maps;
  - two library studies, covering the pinned solvers and unpinned candidates;
  - two literature syntheses that turn the colleague's 69-reference bibliography into mechanism
    contract cards.
- **Synthesis.** The coordinator wrote [mechanism-matrix.md](../evidence/solver-acceleration-2026-10-03/mechanism-matrix.md).
  Its observations C1–C14 and hypotheses H1–H10 went to the independent reviewers as hypotheses.
- **Independent assessment.** Two fresh design reviewers worked in parallel:
  - [DR-1](../evidence/solver-acceleration-2026-10-03/assessment-composition.md) assessed
    composition and the domain model;
  - [DR-2](../evidence/solver-acceleration-2026-10-03/assessment-mechanisms.md) assessed mechanism
    placement, library fit and numerical integrity.
- **Reconciliation.** The coordinator re-read the decisive source (citations marked *verified*
  below), reconciled the two assessments and owns this document.

**No product tests, probes, native runs or benchmarks were executed.** At the maintainer's
direction (2026-10-03), no benchmarking is used or premised. No mechanism is admitted "by measured
benefit". Selection rests on mathematical facts and in-solve observations, and every benefit
claimed here is *Proposed*.

**Input.** The external review that prompted this work is preserved as
[colleague-input.md](../evidence/solver-acceleration-2026-10-03/colleague-input.md). Its claims
were treated as leads. Most held:
- the retained KKT predictor has one consumer;
- homotopy is value-only;
- KINSOL forcing terms are inactive under KLU;
- block-triangular structure serves initialization only.

Two of its recommendations are not adopted:
- a separate bounded Levenberg–Marquardt QP preparation stage, which libraries already cover (see
  [Library fit](#library-fit-and-decisions));
- benchmarking native profiles, which the maintainer excluded.

**Disposition owner.** None exists yet. The proposed work owner is a new solver-strategy plan in
`docs/plans/`, created through `plan-creation`. Until that plan adopts the findings, this review
names proposed owners only. Nothing here is claimed as scheduled.

### Functional target and drivers

The maintainer's target, 2026-10-03: deploy **all** pertinent acceleration and globalization
techniques applicable to our solvers, integrated into the one solver pipeline. The contracts,
numerical guarantees and user options for that pipeline should be settled now, so later work can
concentrate on simulator capability. **Prefer library methods to bespoke code wherever possible.**
Repository rules are heuristics: where one blocks the target, state the replacement rule that keeps
its functional intent.

The governing objective, adopted from the external input, is: *minimize total high-cost nonlinear
work and enlarge the reliable convergence region, by exploiting structure, prior-solve information,
controlled approximation and adaptive accuracy, while preserving the original problem, its final
numerical requirements and independent qualification.* The colleague's functional targets, and the
target contracts and mechanisms (defined below) that realize each:

| Target | Meaning | Realized by |
|---|---|---|
| A | Initial-point quality | T3 start proposals, prediction (M1, M1b, M3), start repair, ladder rungs R1–R2 |
| B | Cross-solve information reuse | T5 retained information; M1, M1b, M3, M5 |
| C | Fewer expensive evaluations | M18; M14 reduced space; M16 surrogates; inner warm starts (F17) |
| D | Less factorization and linear algebra | M5 setup reuse, M4 Krylov, M17 Schur, FERAL symbolic retention |
| E | Globalization and basin enlargement | T2 escalation ladder R1–R7 |
| F | Mathematical reduction of the solve | M13–M15, presolve |
| G | Approximation proportional to usefulness | T8 accuracy tiers; M2 one-corrector steps; surrogate seeds |
| H | Seamless automatic selection | T1 strategy plan, from facts and observations |

**Credible variation axes** that justify the seams proposed below:
- a new mechanism family (for example a learned initial guess);
- a new derived-system generation rule;
- a new or replacement solver library (PSE-S02);
- a new multi-solve workload shape (study, fitting, horizon, edit-and-re-solve);
- a new in-solve trigger;
- a change of user-facing strategy policy.

### Scenarios

| ID | Stimulus and conditions | Expected response | Current impact (evidence) |
|---|---|---|---|
| <a id="s01"></a>S01 | NLP parameter sweep whose active set changes mid-sweep | Predict from the retained factor, follow the path through the breakpoint, correct, qualify | Prediction is horizon-only and refuses any active-set change; studies seed from the predecessor's solution only (F11, F12) |
| <a id="s02"></a>S02 | Root (square) feed sweep | Tangent or secant prediction with setup reuse; fold check | `X_p` computed dense, then discarded; KINSOL setup forced every solve (F12) |
| <a id="s03"></a>S03 | Fitting outer loop of nearby solves | Retained information and prediction; Schur over the fitted parameters for multiple experiments | Fit solves start cold (`warm: None`); Schur unused (F11, F12) |
| <a id="s04"></a>S04 | Rolling-horizon advanced step (regression) | Today's behavior preserved; path following replaces a full re-solve at an active-set change | Works; it is the only predictor consumer |
| <a id="s05"></a>S05 | Edit and re-solve | Value edit: prediction or stale-but-valid retained information. Structural edit: semantic primal survives | Previous-solution seed only |
| <a id="s06"></a>S06 | Hard cold start near invalid domains | Evaluability check and repair, then declared escalation; the specification is intact after failure | No escalation; KINSOL domain recovery is partial, and evaluation failure is the reported outcome (F02, F06) |
| <a id="s07"></a>S07 | Large BTF flowsheet with tears and a locally stiff block | Block-sequential solve, elimination or block sweeps with Anderson, then the simultaneous original correction | Structure gates admission and schedules initialization only; recycles drive KINSOL directly (F08, F15) |
| <a id="s08"></a>S08 | Expensive nested property evaluation | Inner warm starts where selection meaning allows; tiered inner accuracy; quasi-Newton when only first order is available | Inner solves restart from the configured start every trial (F17) |
| <a id="s09"></a>S09 | DAE reinitialization after a discontinuity | Tuned `IDACalcIC`, then the algebraic subsystem, then a labelled least-deviation solve | `IDACalcIC` untuned; no escalation |
| <a id="s10"></a>S10 | Add a mechanism (learned guess, new homotopy rule) | One declaration and its implementation; no workflow edits | Every multi-solve workflow would need an edit (F06) |
| <a id="s11"></a>S11 | Add or replace a root-solver library (PSE-S02) | The adapter absorbs it; consumers keep their contracts | Recycle and nested implicit paths bypass the adapter seam (F08) |
| <a id="s12"></a>S12 | Test strategy policy without native solvers (PSE-S05) | Pure planner and transition rule with a scripted executor | Staged and homotopy policy testable only with KINSOL linked (F08) |

## Integrated assessment

**What supports the target.**
- **One attempt** is handled with care. `MathService::execute` admits the step's start and reuse
  policy and records a typed `StartReceipt`. The representation runner calls one adapter. Recovery,
  KKT evidence and `quality::qualify` evaluate the candidate against the original model in original
  coordinates. `workflow/numerics.rs` is the single owner of `CandidateUse`.
- **Accuracy.** `ResolvedAccuracy` is frozen from the resolved policy and is not a user control.
- **Callbacks.** The callback boundary separates recoverable trials from terminal failures by type.
- **Identity.** Compatibility stamps have four parts (layout, profile, data, backend). Settings
  identity is derived from serde.
- **Library delegation.** The retained KKT factor is already an exact active-set, μ = 0 factor:
  more accurate than an interior-point factor, and certified by inertia, LICQ and weak-row
  classification. `pounce-sens-core`, FERAL, KINSOL, IDAS and `pounce-presolve` already own the
  inner mathematics.

These are preservation constraints for every recommendation below.

**Where it stops.** Nothing owns the decisions between attempts.

- **Policy re-decided per workflow (C1, C2; F06).** Continuation step control exists three times:
  - homotopy grow ×1.5 and halve to 1e-6;
  - fit profile chains;
  - horizon predict-or-solve.

  Acceptance for retention and continuation exists in three variants. Routing requests are built by
  hand at about a dozen production sites (coordinator's count). Block initialization, recycles,
  conditional units, fitting, shooting and the fixed-assignment re-solve run their own loops with a
  fresh `Retained`.
- **Selection ignores the facts it has.** Automatic selection is "most specific class, then a fixed
  adapter rank, then native defaults". The pipeline has the facts a strategy would use, and none of
  them reaches selection: BTF blocks, conditioning estimates, domain and FBBT facts, retained-state
  compatibility.
- **Observations are never inputs.** Rich in-solve observations are recorded and published, but
  are never decision inputs.
- **Escalation is absent, not governed.** It is excluded only because nothing implements it.

The information model lacks the concepts the target needs:

- a start that did not come from an accepted attempt of the same problem;
- a derived system with provenance;
- retained information beyond one horizon-specific slot;
- a work budget across attempts;
- an intermediate accuracy tier.

Separately, a cluster of pinned library capabilities sits unused or reachable only as untyped raw
options (F12, F14). This is the cheapest part of the target, because it removes bespoke work
rather than adding it.

**The defects.** These sit inside the typed outcome. They matter beyond their local consequence
because escalation will read the outcome:

- F01: KINSOL fixed-point and Picard modes ignore recoverable returns;
- F02: KINSOL's domain recovery is partial, and a step-tolerance stop is labelled acceptable;
- F03: inert KINSOL settings are accepted.

They are prerequisites, not polish.

## The target pipeline

```text
 original prepared problem  +  resolved numerical policy (frozen)  +  strategy policy (declared)
                       │
         T1 strategy plan  (pure; facts, retained-information inventory, start proposals,
                       │     routing summary, workload context → ordered admitted rungs; "none" valid)
                       ▼
  start preparation:  T5 predict  →  T3 start proposals  →  admission (screen, evaluability, repair,
                       │                                       choose semantic or native transport)
                       ▼
  T7 generic sequence driver  ── one budget ledger (T6) ── typed observations (T6)
     │  rung k: problem = original | T4 derived system ; T9 route request ; T10 method profile ;
     │          T8 accuracy tier ; abandonment criteria
     │  one attempt = one library solve (existing executor; no internal retry)
     │  T2 transition rule:  Done | next rung | subdivide | stop   (typed triggers only)
     ▼
  original-problem correction under the frozen ResolvedAccuracy
                       ▼
  existing original-coordinate qualification → CandidateUse (sole permission owner)
                       ▼
  T5 retention (keyed)  → predictions for the next point / step / sample / edit
```

### Target contracts

Each contract below states what it consumes, what it decides and what it produces. DR-1 §5 has the
full statements with challenge cases, DR-2 §3 the numerical obligations. All are *Proposed*.

- <a id="t1"></a>**T1 Strategy plan.**
  - **Owner.** A pure planner in `pse-runtime::math::strategy`.
  - **Consumes:**
    - problem facts (`ProblemFacts`; a structural summary of DM, BTF and tear candidates;
      derivative availability; declared parameters and the parametric program; authored
      continuation endpoints, dynamics and stages; guards and bounds);
    - a routing summary, which the planner reads but never ranks;
    - the retained-information inventory;
    - the available start proposals;
    - the declared strategy policy;
    - the workload context (single solve, nth point of a path, outer-loop evaluation).
  - **Decides and produces:** an ordered list of admitted rungs. Each rung has:
    - a mechanism;
    - a problem (the original, or a derived-system reference);
    - a method profile;
    - a start;
    - an accuracy tier;
    - a budget share;
    - abandonment criteria.

    Reasons are recorded for every admitted and refused mechanism. One rung with no preparation is
    the common plan, and "none" is valid.
  - **Avoiding a god object.** Each mechanism family contributes one declaration to an exhaustive
    table, the same pattern as `execution::adapter`. The declaration covers applicability over
    facts, the information it requires, its generation rule, the triggers it answers and its
    abandonment observations. The planner orders declarations and holds no mechanism mathematics.
  - **Consumers stop deciding:** homotopy, profile chains, horizon, studies, block initialization,
    recycles, fitting and shooting stop choosing step control, seed transport and failure response.
- <a id="t2"></a>**T2 Transition and escalation rule.** The replacement for no-fallback.
  - **Decides:** `Done | Rung(i, start) | Subdivide(target, fraction) | Stop(reason)`, as a pure
    function of the plan, the attempt trace and the latest typed observation.
  - **Trigger classification.** One classifier next to `NativeTermination` generalizes
    `engines.rs` `retryable()` into three classes: escalate, terminal-scientific and
    terminal-operational. The rule text is in [Replacement rules](#replacement-rules).
- <a id="t3"></a>**T3 Start proposal and start admission.** This generalizes `StartSource`/`Start`;
  it is not a new `CandidateUse` value.
  - **Proposal fields.** A proposal carries:
    - a typed origin: specification, declared guess, accepted, seed-only, stored, predicted (KKT,
      root response, secant, tangent, path), auxiliary (derived system, path value), partial
      (BTF prefix), modified specification, learned, or surrogate;
    - semantic primal values;
    - an optional native payload;
    - validity limits: Δp radius, assumed active set or branch, staleness;
    - screening: finite, inside bounds, inside evaluator domains, residual summary.
  - **Admission.** One admission operation chooses among proposals. It screens them, checks
    evaluability and repairs where needed (projection into the guard or FBBT interior, or a bound
    push). It then chooses the transport, native only when the stamps and transformation path
    permit, and extends `StartReceipt` with the origin and transport. Permission is "start only"
    by type.
  - **Preserved.** `CandidateUse`, `seed_only` and `PreviousAccepted` keep their meaning. The
    horizon's *applied* prediction remains a horizon control policy, not a start permission.
- <a id="t4"></a>**T4 Derived system.** The replacement for value-only continuation.
  - **What it is.** A disposable realization owned by preparation (`pse-compiler` view preparation
    and `pse-math` plan derivation). It is generated from the original prepared problem by an
    enumerated rule:
    - natural-parameter or bordered arclength;
    - Newton, fixed-point (`A = D·P_M`), affine, model or bounded homotopy;
    - pseudo-transient shift;
    - block subsystem;
    - reduced space;
    - nonlinear preconditioner;
    - DAE initialization subsystem;
    - bounded feasibility NLP;
    - surrogate.
  - **Construction.** It is composed over the original evaluator and compiled through Symbolica,
    so derivatives are exact and ADR-0144 selection meaning is preserved.
  - **Identity and reuse.** The structural key frames the original `view_key`, the rule and the
    rule's structural parameters. Anchors, λ and δ are bound values, so a family of steps shares
    one prepared view, as value-only homotopy does today.
  - **Record.** It records the incidence delta, and structure is analysed on the union incidence.
    It also records:
    - the coordinate map;
    - a mechanically checkable terminal identity (`H(·,1) ≡ F`, `λ = λ_target`, block union = full
      system, or none);
    - the budget.
  - **Use.** It is admitted only while the original it names (view, bound values, resolved policy)
    is in force. Its candidates enter the original only as start proposals, except at the terminal
    identity, where the step is an original-problem step.
- <a id="t5"></a>**T5 Keyed retained information and prediction.**
  - **Two tiers:**
    - a native tier, which is worker-owned and never crosses threads: adapter sessions, the
      active-set KKT factor, the root Jacobian factor at x\*, KINSOL setup;
    - a portable tier: semantic solutions, the root response, KKT parametric steps, secant and
      tangent histories, branch and orientation records.
  - **Keys.** Every entry carries:
    - the complete stamp (layout, profile, backend, preparation identity, policy key);
    - the point it was formed at, and its values identity;
    - its accuracy regime and inertia or rank;
    - the decision that permitted retention.

    Validity separates "fits the structure" from "exact at these values".
  - **Prediction.** One `predict(target, store) → StartProposal | Unavailable(reason)` serves
    studies, continuation, fitting, horizon and edit-and-re-solve. It tries, in order:
    - KKT advance;
    - KKT path following;
    - root first-order;
    - secant;
    - zeroth order.
- <a id="t6"></a>**T6 Typed observation and one budget ledger.**
  - **Observation.** Projected once from `SolveReport`. Fields:
    - termination class and candidate kind;
    - original residual and domain summary;
    - trial rejections and regime crossings;
    - restoration entry;
    - line-search, step-tolerance and Krylov failures;
    - iterations, with trajectory or progress evidence;
    - setup and factor reuse;
    - work consumed.
  - **Ledger.** One `WorkBudget` per solve task covers wall time, attempts, iterations and
    evaluations, with preparation, prediction and rejected predictions charged to it. Any
    enclosing deadline clamps it. Exhaustion is terminal-operational.
  - **Abandonment.** A typed abandon latch distinct from cancellation: KINSOL callback `−1`,
    Ipopt and POUNCE intermediate callbacks returning `false`.
- <a id="t7"></a>**T7 Generic sequence driver.** It lives below `workflow`, in `pse-runtime::math`.
  - **Parameters.** It takes a step binder (modeling case, block subsystem, causal map, fit oracle,
    shooting) and a step executor. The executor is `MathService` resolving adapters through an
    injected routing `Table` in production, and a scripted executor in tests.
  - **Responsibilities.** It owns effects: executing rungs, charging the ledger, updating
    retention and recording the trace.
  - **Workflows** keep their domain target paths (which points, pins or samples) and stop owning
    numerical transitions.
- <a id="t8"></a>**T8 Accuracy tiers.**
  - **Final accuracy.** Every original-problem rung uses the frozen `ResolvedAccuracy`, unchanged.
  - **Intermediate tiers.** Derived by declared rung rules and recorded. They are never user input
    and never carried by `Controls`:
    - predictor error radius;
    - path corrector tolerance;
    - inner implicit tolerance as a declared factor tighter than the outer;
    - forcing terms;
    - one-corrector steps on intermediate problems only.
  - **Permission.** An intermediate-tier candidate can never hold result permission.
- <a id="t9"></a>**T9 One route-request constructor per prepared problem** (original, derived,
  block, fit oracle). Post-execution refusals enter `Context.refusals` as typed post-execution
  reasons. A derived problem routes from its own facts: a least-squares rung for a root problem
  routes as an NLP.
- <a id="t10"></a>**T10 Typed method profiles.** Per adapter:
  - **KINSOL:** direct; Krylov with forcing and a block preconditioner; setup reuse with residual
    monitoring; Anderson only where it acts.
  - **Ipopt:** initialization, restoration and barrier options.
  - **POUNCE:** second-opinion rungs, path following, Schur, partitioned quasi-Newton.
  - **Uno:** trust-region filter SQP/SLP.

  The planner selects and records profiles. Once a profile is typed, its raw option keys are
  reserved. A setting that has no effect under its strategy is refused.

**Dependency direction** (DR-1, adopted):
- `pse-model` holds the vocabulary: policy, mechanism kinds, origins, trigger classes.
- `pse-math` holds the facts and derived-system plan derivation.
- `pse-backend-native` holds routing, adapters, trigger classification, observation projection and
  KKT prediction.
- `pse-runtime::math` holds the planner, transition rule, sequence core, retained store, start
  admission, route-request constructor and executor.
- `pse-runtime::workflow` holds the binders and target paths.

**No new workspace crate is needed for the strategy layer.** A Uno binding needs its own `-sys`
crate (see [Authority changes](#authority-changes-and-disposition)).

### Start preparation and the escalation ladder

Start preparation adds no escalation. It takes a prediction (T5), otherwise the accepted
predecessor, otherwise the specification, and checks evaluability and repairs the start before any
rung (DR-2 §6). The rungs are **ordered by the mathematical assumptions each adds, never by
expected speed**. A plan skips any rung whose admission facts fail.

| Rung | Mechanisms | Assumptions added | Enter on (typed) |
|---|---|---|---|
| R0 | Class-selected adapter, fact-chosen method profile | — | — |
| R1 | Same library on a different trajectory: KINSOL profile (setup interval 1, fact-derived max step); Ipopt `start_with_resto`, `least_square_init_*`, adaptive μ; POUNCE `second_opinion_rungs()` as recorded attempts | none | line-search non-convergence or failed backtracking, repeated evaluation failure with trial evidence, restoration failure, local infeasibility, invalid number, iteration limit **with** stagnation evidence |
| R2 | Same equations, different method class: bounded feasibility NLP (Ipopt/POUNCE), POUNCE active-set SQP, **Uno** trust-region SQP/SLP | none, but the stopping meaning can become "locally infeasible / least-squares stationary" | R1 exhausted; evaluation failure with trial evidence |
| R3 | Ordering: BTF block-sequential solve (M13) | exact incidence; per-block numeric regularity at visited points | failures not localized by R1–R2 |
| R4 | Local elimination (M14), block Gauss–Seidel with Anderson (M6, M15) | `F₁,y` nonsingular over the visited region; contractive sweep | failures localized to the same block |
| R5 | Physical-parameter continuation, natural or arclength (M9) | connected path in an authored λ, C¹ in λ | basin failure with an authored parameter |
| R6 | Constructed homotopies (M10) | bounded, transversal path (heuristic label) | R5 inadmissible or failed |
| R7 | Pseudo-transient flows (M11): authored dynamics through IDAS, else KINSOL on the shifted system | attracting equilibrium of the chosen flow | R6 failed; singular Jacobian with an invariant flagged |
| ✱ | Multistart (M17b), surrogate seeds (M16) | seed-only | at any rung, as start variation |

Every rung's terminal output re-enters as a start proposal of its class. The final candidate always
comes from a solve of the unchanged original specification under final accuracy.

**Never escalate on any of these:**
- time, resource or pool limits;
- cancellation, infrastructure failure or panic;
- an invalid model, unsupported capability or contract error;
- a presolve-certified infeasibility re-verified in original coordinates;
- an iteration limit without trajectory evidence of stagnation.

**One reconciled disagreement.** DR-1 placed "iteration limit without a result" in the escalate
class. DR-2 excluded an iteration limit without trajectory evidence. The review adopts DR-2's
refinement. Without evidence that the *method* stalled, an iteration limit is a budget outcome, and
changing method would spend more without addressing the cause. With stagnation evidence (POUNCE's
quality-escalation gate, an Ipopt restoration loop, a contraction ratio ≥ 1), it escalates.

### Mechanism placement

This condenses DR-2 §2, which has the library functions, owners and revisit triggers.

| Family | Placement | Bespoke scope |
|---|---|---|
| M1 NLP sensitivity prediction | Pinned library, used: `pounce-sens-core` over pse's active-set KKT factor; generalized through T5 | screening only |
| M1b Active-set-changing path following | Pinned library, unused: `step_along_path`, `path_direction`, `refine_step_onto_bounds`; `pounce-qp` homotopy for the QP class | outer segment loop, ≈300–600 lines; release methods on `KktFactor` |
| M2 One-corrector intermediate steps | Composition (prediction plus one bounded library iteration), intermediate problems only | none |
| M3 Root tangent and secant | Composition over faer sparse LU at x\*, replacing the dense `square_response` | vector arithmetic |
| M4 Newton–Krylov, forcing, preconditioning | KINSOL, used; admitted only on a fill-estimate fact; block preconditioner from KLU or faer factors | small glue; needs a true directional-derivative program |
| M5 Setup and Jacobian reuse | Pinned, unused: KINSOL `NoInitSetup(TRUE)`, `MaxSubSetupCalls`, residual monitoring. Admitted, low priority: Ipopt `ReOptimizeTNLP` C++ shim. Pending U3: FERAL symbolic retention | ≈100–200 lines of C++ glue for the shim |
| M6 Anderson, fixed point, Picard | KINSOL, used; after F01 | none |
| M7 Bounded local models (LM/trust region) | Ipopt/POUNCE bounded feasibility NLP; **Uno** trust-region SQP/SLP (new) | none: no bespoke LM loop |
| M8 Library-native NLP globalization | Typed Ipopt options; POUNCE second-opinion rungs as declared attempts | none |
| M9, M10 Continuation and constructed homotopies | KINSOL/Ipopt correctors; Symbolica derives H, H_x, H_t | tracker ≈0.5–0.9k lines; construction rules ≈0.3–0.6k lines |
| M11 Pseudo-transient continuation | IDAS/Diffsol on authored dynamics; otherwise KINSOL on the shifted system | SER controller ≈100–250 lines |
| M12 DAE initialization | `IDACalcIC` with its tuning exposed; then the algebraic subsystem; then a labelled least-deviation NLP | none |
| M13–M15 Block, reduced space, nonlinear preconditioning | `pounce-presolve` BTF, existing `pse-math::implicit`, KINSOL FP with Anderson | sweep orchestration and ASPIN JVP ≈600–1,200 lines |
| M16 Surrogates | egobox (new, when implemented) | trust-region model management ≈600–1,000 lines |
| M17 Schur, multistart | POUNCE `set_kkt_schur_block`; multistart through pse workers | partition construction; start generation |
| M18 Evaluation reduction | Ipopt constant flags (used); POUNCE partitioned quasi-Newton and FD Hessian, typed; Symbolica directional derivatives; inner warm starts under the F17 rule | typing only |

### Numerical stage contracts

These are the profile's numerical stage columns for the stages the target adds. Each stage keeps
§16's resolved policy as its sole source of final tolerances.

| Stage | Formulation policy | Derivative source and order | Scaling | Class · solver capability | Status → outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Prediction (M1, M1b, M3, secant) | Validity limits recorded; refuses on active-set change (M1) or walks breakpoints (M1b); fold sign check (M3) | Exact KKT factor at μ = 0 with no regularization; sparse F_x at x\* | Normalized factor (existing congruence) | Backsolves through `pounce-sens-core` / faer | `Unavailable(reason)` typed | No tolerance of its own: the prediction is a seed; screened by residual against the plain warm start |
| Derived-system rung (M5, M7, M9–M15) | Generation rule; terminal identity; union incidence analysed | Symbolica-derived, exact; FD only as a typed source | Original normalization extended to added columns | Routed from the derived problem's own facts (T9) | Derived-system outcome; candidates are auxiliary seeds | T8 intermediate tier, recorded |
| Original correction | Unchanged specification | Existing model derivatives | §16.1 normalization | Existing routing and method profile | Existing typed termination plus T6 observation | Frozen `ResolvedAccuracy`; independent original-coordinate qualification |
| Nested evaluation (M14, M18) | Selection meaning must be start-independent for warm inner starts (F17) | Implicit-function derivatives at the converged inner root | Inner targets from the resolved policy | KINSOL inner session | Inner failure → recoverable trial for Ipopt and KINSOL Newton; abort for FP/Picard (F01) | Inner tolerance a declared factor tighter than outer; qualification re-solves from the canonical start |

**Physical semantics.** The quantities the target introduces:

| Quantity | Dimension and unit | Authority |
|---|---|---|
| Parameter step Δp, validity radius | Parameter's own unit; normalized by the parameter's scale for comparison | Resolved policy targets (§16.1) |
| Path parameter λ | Dimensionless fraction, or the authored parameter's unit for natural continuation | Derived-system record |
| Pseudo-time step δ | Model time unit for authored dynamics; dimensionless for a matching pairing | Derived-system record |
| Intermediate tolerances | Derived from the per-target absolute and relative budgets | T8 |

No new physical basis or reference-state convention is introduced. Every interface stays in original
units and normalized coordinates as §16.1 defines.

**Well-posedness.** Structural analysis still runs before solving (§15.2). A derived system is
analysed on its union incidence before its rung is admitted. A block solved or eliminated needs a
numerical qualification at the point of use (pivot, condition estimate or rank check), because
matching proves only generic rank. Refusals name the rule, block and rows involved.

### User options and numerical contracts catalogue

This is the catalogue the maintainer asked for. It is adopted from DR-1 §7 with DR-2's additions.
Declared options live in Rust serde types with schemars, generated into Python msgspec documents.
No hand-written mirror is introduced.

| Contract or option | Class | Owner | Identity |
|---|---|---|---|
| Strategy mode: `off` (one direct attempt, today's behavior), `automatic` (planned from facts), `explicit` (authored rung list, still admitted) | Declared | `StrategyPolicy` in `SolveSettings` | Lineage, not the session profile stamp |
| Allowed or denied mechanism families | Declared | `StrategyPolicy` | Lineage; a denial is a recorded plan reason |
| Task work budget (wall time, attempts, iterations, evaluations) | Declared | `StrategyPolicy` | Lineage; per-step `Controls` limits stay per-rung caps |
| Backend alternatives under an explicit selection | Declared | `StrategyPolicy` | Lineage; without them, explicit selection fixes the backend |
| Branch policy: `any_qualified_root` or `path_connected` | Declared | `StrategyPolicy` | Lineage; fold and branch crossings recorded (PS-12) |
| Determinism class | Declared | `StrategyPolicy` | Lineage (DP-11) |
| Retention policy and byte cap | Declared | `StrategyPolicy` / `MathPolicy` | Resource only; charged to the job allowance |
| Applied-prediction permission | Declared | Horizon controller | Per sample, unchanged |
| Start policy and reuse policy | Declared (existing) | `Controls` | Existing; proposals add origins |
| Authored continuation endpoints, stages, initialization overrides | Declared (authored, existing) | Model | Existing; they become target paths and stage rungs |
| Step-control constants (initial step, growth, minimum step) | Derived, with a typed expert override | Mechanism declaration | Lineage when overridden |
| Rung ordering, mechanism admission, method-profile choice | Derived | T1 | Plan identity in lineage, with reasons |
| Intermediate accuracy tiers; prediction validity and screening thresholds | Derived | T8, T3, T5 | Recorded |
| Final tolerances, scales, bound relaxation, warm-start options, derivative-constancy flags | Reserved (existing) | `ResolvedAccuracy`, adapters | Existing |
| Raw options owned by a typed method profile | Reserved once typed | Adapters | Remaining raw keys stay an escape hatch inside `Controls::identity` |
| Strategy trace (rungs, routes, starts, problem identities, observations, ledger) | Result contract | T7 | A generated `runtime.*` relation |

## Replacement rules

Each rule below is a heuristic the target outgrows. Each replacement keeps the rule's functional
intent. They are *Proposed* text for the routes in
[Authority changes](#authority-changes-and-disposition).

**RR1. No fallback after native failure** (§18.7, ADR-0152, ADR-0083 "no fallback engine").
- *Intent kept:* deterministic, explainable routing; every attempt recorded; no ungoverned retries
  after resource, cancellation or infrastructure failure; explicit selection honored.
- *Replacement:* "Before execution, selection follows class, rank and contextual eligibility. After
  execution, a solve task may make further attempts only as rungs of its resolved strategy plan,
  chosen deterministically from the plan, the problem facts and the typed numerical outcomes of
  earlier attempts. Each rung's adapter comes from the same contextual routing assessment, with
  earlier attempts entered as typed post-execution refusals. Escalation is triggered only by
  numerical outcomes that leave the problem undecided: line-search or restoration failure, local
  infeasibility or a stationary non-root, recoverable trial failures with trial evidence, a refused
  candidate, or an iteration limit with stagnation evidence. It is never triggered by time,
  resource or pool limits, cancellation, infrastructure failure, panic, an invalid model,
  unsupported capability or a contract error. A re-verified certified infeasibility ends the task.
  All attempts are bounded by the task's work budget, recorded with rung, route, start and problem
  identity, and judged by the same original-problem completion. An explicit selection is never
  substituted: under it, rungs may change method profile, start or derived system, and change
  backend only when the selection names the alternatives."

**RR2. Homotopy changes values only** (§17.5, ADR-0093).
- *Intent kept:* one model authority; transactional, restorable initialization.
- *Replacement:* "Initialization, continuation and escalation may solve derived systems:
  realizations generated from the original prepared problem by an enumerated generation rule. Each
  is recorded with its rule, parameters, anchor provenance, incidence delta, coordinate map and
  terminal identity, and is prepared and reused through the original's preparation owners. A derived
  system is admitted only while the original it names (view, bound values, resolved policy) is in
  force, and never replaces it. Its candidates enter the original problem only as start proposals,
  except at its declared terminal identity, where the step is an original-problem step answering to
  every original obligation. Value-only homotopy over authored endpoints is the parameter-path
  member of this family."

**RR3. Libraries own iteration and globalization** (PS-09 in the profile, §17 intro, ADR-0083).
- *Intent kept:* no reimplemented Newton step, line search or factorization (G8).
- *Replacement* (DR-2 §5, adopted): "Established numerical solvers own the methods they implement:
  the step computation, step acceptance (line search, filter or trust region), barrier or
  active-set updates and factorization inside one solve. The simulator never reimplements such a
  method where a pinned or admissible library provides it with a fitting contract (recoverable
  trials, bounds, cancellation, typed outcomes). The simulator may compose library solves into
  declared outer strategies (prediction, continuation and homotopy tracking, pseudo-transient
  stepping, block and reduced-space decomposition, nonlinear preconditioning, model management and
  escalation) when:
  - (a) every inner solve and factorization is a library call;
  - (b) the outer step-acceptance and step-size rules are declared, bounded and recorded per
    attempt;
  - (c) the reason no library supplies the outer method with a fitting contract is recorded;
  - (d) every outer result enters only as a start proposal of a stated class, or is accepted by the
    original-problem qualification.

  Bespoke linear algebra is limited to small dense bordering, Schur complements of declared small
  dimension and secant updates."
- Audit addition: "Is each outer loop a composition of library solves with recorded rules, or does
  it re-create a library's inner step?"

**RR4. "No hidden second solves"** (POUNCE retry ladder and fallback options pinned off).
- *Intent kept:* truthful attempts.
- *Replacement:* "Library-owned retry trajectories are admitted only as planned rungs, each run and
  recorded as its own attempt. A ladder that hides per-rung outcomes is not called."

**RR5. Nested implicit solves never start from history** (`pse-math/src/implicit.rs`).
- *Intent kept:* an evaluation is a function of its inputs.
- *Replacement* (DR-2 DM-F07): "An inner solve may start from a previous trial's root only when its
  selection meaning is start-independent: `Unique`, or `Branch`/`MinimumScore` with isolation
  discharged. Its inner tolerance must be a declared factor tighter than the outer tolerance, the
  qualification evaluation must re-solve from the canonical start, and the warm source must be
  recorded. `Operational` and `Relation` selections refuse."

**RR6. Metrics are never decision inputs** (`solve.rs` `Evidence` doc).
- *Intent kept:* no decisions from strings or presentation.
- *Replacement:* "String-keyed metrics never drive decisions. Typed attempt observations may, when a
  rung's declared criteria name them."

**RR7. Stopping budgets are not user controls** (§16.6).
- *Kept as is*, with this addition: "Intermediate accuracy tiers are derived from the resolved
  accuracy by declared rung rules, recorded, and never govern an original-problem rung or a result."

**RR8–RR10.** These rewrite three existing descriptions around start proposals:
- §17.6 start model;
- §19.4 fitting starts: the estimate starts from declared guesses unless its strategy records
  another origin;
- §19.3 study continuation edges: transport over an admitted edge may be a prediction.

DR-1 §6 rows R8–R10 have the text.

## Findings

Findings are grouped by cause. "Current defect" means current supported behavior is affected.
"Target gap" means the architecture cannot deliver the target without the correction. Detailed
arguments are in DR-1 §4 and DR-2 §4, cited per row.

| ID | Finding | Kind | Principles · gates · scenarios | Source |
|---|---|---|---|---|
| <a id="f01"></a>F01 | KINSOL fixed-point and Picard modes ignore pse's recoverable callback return. A failed map evaluation leaves the previous output, and an undamped iteration reports `KIN_SUCCESS` where the map could not be evaluated. | Current defect | PS-10 · PS-G3 · S07 | DM-F01; *verified* `kinsol.c` `KINFP`/`KINPicardAA` test `retval < 0`; `kinsol.rs:444-470` |
| <a id="f02"></a>F02 | The typed outcome lacks what escalation needs. KINSOL recovers domain errors only before its α-loop, so globalization failures surface as evaluation failures. `KIN_STEP_LT_STPTOL` is labelled `Acceptable` although it can be a stall. KINSOL observations are cumulative only, and there is no typed abandon latch. | Current defect / prerequisite | PS-10, DP-21 · PS-G3, G2 · S06 | DM-F04, DM-F13, DM-F14; *verified* `kinsol.c` line-search comment; `kinsol.rs:1264` |
| <a id="f03"></a>F03 | Settings that have no effect are accepted. KINSOL Anderson history, delay and damping are accepted under Newton and line search, where KINSOL ignores them; §18.10 says such controls are refused. The record claims an acceleration and the identity changes. Ipopt `warm_start_same_structure` is unreserved although the C API cannot honor it. | Current defect | G7, DP-15 · S11 | DM-F02, DM-F08; *verified* `kinsol.rs:196-262` has no strategy check on `anderson` |
| <a id="f04"></a>F04 | Retention and continuation acceptance have three competing definitions. The session keep flag uses `AssessedPoint::accepted()` (model checks only), contrary to its own doc comment; `CandidateDecision` and `commit_block` are the other two. | Current defect (narrow) | AP-04, DP-01 · G1 | DC-F04; *verified* `math/staged.rs:293-326`, `modeling/assessment.rs:28` |
| <a id="f05"></a>F05 | §17.6 describes `Start::Seed(k)` and value-only study seeding. The code has `Start::{Specification, Accepted}`, and studies submit the predecessor's full native `WarmStart`. | Documentation divergence | G7, DP-24 | DC-F03 |
| <a id="f06"></a>F06 | Policy between attempts has no owner. Continuation step control exists three times. Each workflow chooses its seed transport and failure response. Escalation is excluded only by absence, and the only trigger taxonomy is private to `engines.rs`. | Target gap | AP-01, AP-03, AP-05 · G9 · S06, S10 | DC-F01, DC-F07 |
| <a id="f07"></a>F07 | Route requests are hand-built at about a dozen production sites, each re-stating budgets, structure witness, refusals and demands. | Target gap | AP-03, DP-01 · G9 | DC-F08; coordinator grep |
| <a id="f08"></a>F08 | The staged primitive is tied to modeling and hard-wired to the linked adapter table. Block initialization, recycles, conditional units, fitting and shooting run their own loops. Recycle and nested implicit solves drive KINSOL directly, bypassing the adapter seam. | Target gap | AP-02, AP-06 · G9 · S07, S11, S12 | DC-F02, DC-F11 |
| <a id="f09"></a>F09 | The start model cannot express a predicted, auxiliary, partial, learned or modified-specification start. It has no validity limits and no screening, and the choice between semantic and native transport is made per workflow. | Target gap | AP-04, DP-02, PS-08 · G9 · S01, S05 | DC-F03 |
| <a id="f10"></a>F10 | Derived problems exist without a shared concept: stages, homotopy overlays, blocks, causal maps, factorable export, ℓ1, the feasibility wrapper. Constructed homotopy, pseudo-transient, arclength, least-squares and reduced-space systems are blocked by the value-only rule. The authored homotopy has no predictor and cannot detect folds. | Target gap | AP-04, DP-08, PS-08 · G9 · S06 | DC-F06, DM-F11 |
| <a id="f11"></a>F11 | Retained numerical information has a single slot, one consumer and an incomplete key. `Advance` reuse checks parameter ids only, and `Retained::clear` drops the session but keeps `Advance`. Today's single consumer bounds the consequence; generalization would not. | Current defect (latent) / target gap | DP-09, PS-11 · G6 · S01–S05 | DC-F05; *verified* `execution.rs:633-639`, `kkt/advance.rs:176-187` |
| <a id="f12"></a>F12 | Cross-solve reuse that the pinned libraries provide is unused: `KINSetNoInitSetup(0)` on every solve, `msbsetsub` and residual monitoring unexposed; the KKT predictor refuses active-set changes while `step_along_path` sits unused; root sensitivity is dense and transient; Ipopt's C API cannot re-optimize; POUNCE rebuilds FERAL symbolic analysis on every solve. | Target gap | DP-13, PS-11 · G8 (target) · S01–S03 | DM-F03, DM-F05, DM-F06, DM-F08, DM-F09; *verified* `kinsol.rs:1084`, `IpStdCInterface.cpp:273`, `boundcheck.rs:786/1275/1974` |
| <a id="f13"></a>F13 | There is no strategy-wide budget, no intermediate accuracy tier and no typed observation suitable for decisions. Inner implicit solves run at final accuracy. | Target gap | DP-11, DP-12 · G5 · S08 | DC-F09; C6 |
| <a id="f14"></a>F14 | Method choice inside an adapter cannot be planned or recorded. Globalization options are reachable only as raw passthrough, and POUNCE's declared retry rungs are pinned off. | Target gap | AP-05, PS-09 · G8 · S06 | DC-F10, DM-F10 |
| <a id="f15"></a>F15 | Structure never shapes the main solve. Matching, DM and BTF gate admission and schedule initialization only. Tear sets are passed by hand. No block-sequential solve, reduced space or nonlinear preconditioning exists. | Target gap | PS-05, DP-13 · S07 | C9 |
| <a id="f16"></a>F16 | The Newton–Krylov route has no cost advantage: the JVP assembles the full Jacobian before multiplying, and only Jacobi preconditioning exists. | Target gap | DP-13 · S07 | DM-F12 (U7: not re-verified) |
| <a id="f17"></a>F17 | Nested implicit solves never warm-start from the previous trial. That protects function semantics but forgoes the main evaluation-reduction lever where uniqueness is established. | Target gap | PS-11, DP-09 · S08 | DM-F07 |

**How the findings relate.**

- **Shared cause, outcome information (F01, F02, F03).** The typed outcome, or the typed settings
  record, does not carry true information. The escalation rule (T2) will read exactly these fields,
  so they are prerequisites of RR1, not independent polish.
- **Shared cause, no owner between attempts (F06, F07, F08).** These are manifestations of one
  cause, but each carries a distinct closure obligation:
  - a planner and transition rule (T1, T2);
  - a route-request constructor (T9);
  - a generic driver with an executor seam (T7).
- **One information-model gap, distinct contracts (F09, F10, F11).** T3, T4 and T5 respectively.
  T3 and T5 are prerequisites of T1, because the planner reads proposals and the retained
  inventory.
- **Producers for T5 (F12).** Each item is library leverage that feeds the retained-information
  model.
- **Prerequisites for T10 (F14, F03).** Typed method profiles need inert settings refused first.
- **Mechanism gaps (F15–F17).** They depend on T4 (derived systems) and T8 (accuracy tiers).

**Challenge cases for the main remedies.**

- **T1 on a trivial problem.** The planner must produce a one-rung plan with no preparation work.
- **T2 inside a durable study occurrence.** The occurrence deadline must clamp the ledger. Reaching
  it must stop the task without escalating, and a partial trace must publish as partial.
- **T4 when the user edits a value mid-strategy.** The original identity changes, so derived
  products keyed to the old original lose exact reuse. The last auxiliary point survives only as a
  stale proposal.
- **T3 with the horizon's applied prediction.** It remains a horizon policy and never becomes a
  start permission.
- **M9 at a fold.** Fold detection must record the crossing, and a root reached after it must be
  published as "reached by continuation across a fold". Branch continuity is a declared policy,
  never inferred.

None of these cases defeats the proposed contracts, but each constrains them (DR-1 §5).

## Library fit and decisions

| Decision | Outcome | Reason | Revisit trigger |
|---|---|---|---|
| PETSc (SNES composition and NPC, `TSPSEUDO`, VI bounds, arclength, trust-region Newton) | **Decline now** | The mechanisms only PETSc owns abort on domain errors in the source read (`NEWTONTR`, `NEWTONAL`, `NGMRES`). The ones that fit (`TSPSEUDO`, line-search NASM) are thin loops over solves KINSOL and IDAS already provide with pse's recoverable-trial contract. Adopting it means a second nonlinear stack, an MPIUNI source build and a hand-written `-sys` crate. Rung interfaces stay backend-agnostic, so a PETSc root adapter can be added later. | PETSc trust-region, arclength, NGMRES or NASM treat a function-domain error as step rejection (read from `main`, U5); distributed memory becomes needed; or block failure arises that NGS with Anderson cannot contract |
| **Uno** (trust-region filter SQP/SLP) | **Adopt** as a new NLP adapter for rung R2 | It is the only open trust-region filter SQP/SLP found, and it shrinks the radius on an evaluation error. It supplies a globalization class absent from Ipopt and POUNCE, so no bespoke trust-region loop is needed. | Its C API, determinism or BQPD acquisition fails qualification (U2); then fall back to SLP over HiGHS, and reconsider Ceres |
| Ceres | Decline | Covered by the feasibility NLP and Uno. Normal equations square the condition number of square systems. | Uno declined |
| `russell_nonlin` | Decline | Recoverable trials only through a side hook; no bounds; a second sparse stack. | It gains recoverable trials and pluggable correctors |
| egobox | Adopt when M16 is implemented | Surrogate construction is library-owned; management is a bounded loop of library NLP solves. | A library with trust-region model management |
| Ipopt C++ `ReOptimizeTNLP` shim | Admit, low priority | Integration glue only. It pays off when sequences stay on Ipopt. | The Ipopt C API adds re-optimize |
| sIPOPT | Decline | Redundant with `pounce-sens-core`, and pse's μ = 0 active-set factor is more accurate. | — |
| Ipopt inexact algorithm; ARKODE/CVODE features; faer sparse QR as the main LM factor; POUNCE rayon batch API | Decline | No fact demands them, or they conflict with worker ownership (§18.8). | Per DR-2 §5 |
| POUNCE Schur, partitioned quasi-Newton, FD Hessian (typed), `pounce-qp` homotopy; HiGHS QP hot start | Adopt | Library-owned capability. The Schur hook falls back silently, so whether it was honored must be recorded (DM-F15). | Many linking variables (implicit Schur needed) |

**G8.** The current state passes: no reimplemented Newton step, line search or factorization was
found. The target passes only if each bounded bespoke item records its library-consideration reason
under RR3:
- the tracker;
- the SER controller;
- sweep orchestration;
- the M1b loop;
- model management.

The decisive G8 risk is a bespoke LM or trust-region loop. The review recommends against it.

**Maintainer decision point.** Across the whole target, the bounded bespoke code totals roughly
2.4–4.6k lines, summing the placement table's estimates. All of it is outer controllers over
library solves. Declining PETSc keeps two of those controllers bespoke that PETSc could otherwise
supply, roughly 0.7–1.45k lines: the pseudo-transient step control and the block-sweep
orchestration. The continuation tracker and the homotopy constructions, roughly 0.8–1.5k lines,
stay bespoke even with PETSc, because its arclength solver aborts on domain errors. This is the
main place where "library over bespoke" and the trial-failure contract pull apart. The review's recommendation follows the
contract: a library whose solver aborts on a domain error would make RR1's escalation triggers
unreliable for exactly the hard starts the target addresses. If the maintainer prefers PETSc anyway,
the route is a PETSc root adapter under PSE-S02, restricted to its line-search solver types. The
review should then be reopened for U5.

## Foundations and gates

These verdicts assess the current pipeline against the target.

| Foundation | Verdict | Scenario and evidence |
|---|---|---|
| AP-01 Separation of concerns | **Violated** | S10: adding a mechanism edits every multi-solve workflow, because policy between attempts is spread across them (F06) |
| AP-02 Stable contracts | **Violated** | S11: recycle and nested implicit paths drive KINSOL directly; a root-library replacement must edit them (F08) |
| AP-03 Composition | **Violated** | Continuation ×3, acceptance ×3, route requests hand-built at about a dozen sites (F06, F07, F04) |
| AP-04 Domain model and semantic authority | **Violated** | The model lacks start origins, derived systems, retained information, strategy and escalation (F09–F11), and has a second acceptance authority (F04) |
| AP-05 Explicit structure | **Violated** | Escalation is excluded by absence. Triggers are private. Method choice is raw passthrough. Inert settings are accepted (F06, F14, F03) |
| AP-06 Local reasoning and testability | **Violated** | S12: staged and homotopy policy is testable only with KINSOL linked (F08) |

| Gate | Result | Evidence |
|---|---|---|
| G1 Authority | Fail (narrow) | F04 |
| G2 Semantic fidelity | Fail (narrow) | F02: a step-tolerance stall labelled acceptable |
| G3 Validity | Pass for current behavior | Admission and structural refusal precede native work (§15.2, §17.1) |
| G4 Hidden behavior | Pass | No hidden retries; POUNCE retries are pinned off |
| G5 Consistency and recovery | Pass for current behavior; unresolved for the target | There is no strategy ledger, so T6 must make partial traces explicit (F13) |
| G6 Transformation and reuse | Fail (latent) | F11: `Advance` key incomplete |
| G7 Truthful capability claims | Fail (narrow) | F03 (inert settings recorded as acting), F05 (documentation divergence) |
| G8 Library leverage | Pass for current behavior; conditional for the target | RR3 records; no bespoke LM loop |
| G9 Architectural fitness | **Fail** | All six foundations violated against the target |
| PS-G1 Physical consistency | Not applicable | No physical-model, basis or convention change in scope |
| PS-G2 Well-posedness | Pass for current behavior; target obligation stated | Derived systems must be analysed on union incidence (T4) |
| PS-G3 Numerical integrity | **Fail (narrow)**; unresolved for the target | F01 misclassifies a status (PS-10). The target mechanisms' contracts are the stage obligations stated above |

**Conformance-suite status.** Not applicable: no unit or property model is touched.

## Alternatives

| Alternative | Assessment |
|---|---|
| **A0 Current baseline** (per-workflow policy; globalization inside libraries only) | Deterministic and simple, but it cannot reach the target, and each added mechanism amplifies across owners. It is retained as strategy mode `off`. |
| **A1 Proposed** (T1–T10, the ladder, library-first placement) | Removes per-workflow policy. Testable without native solvers. Extension is by declaration. Costs: new vocabulary and a result relation, migration of about eight consumers, derived-view preparation, and the ADRs below. Benefits are *Proposed*. |
| **A2 Library-owned strategy** (PETSc SNES/TS or LOCA as the owner) | Libraries can supply rungs, but none owns composition across studies, horizon, fitting and original-space qualification. Adopting one as the owner would duplicate the adapter seam and the typed outcomes. Not recommended as owner; PETSc is revisitable as an adapter. |
| **A3 Simplest viable** (a shared `continue_to(target)` helper and a shared trigger classifier, without a plan object) | Fixes part of F06 and the trigger taxonomy. It leaves selection unrecorded, the budget unshared and user policy undeclared, so it cannot deliver target H. It is a reasonable first migration step towards A1. |

**What would reopen the recommendation:**
- plans that rarely exceed one or two rungs, in which case A3 plus method-profile selection
  suffices;
- derived views that cannot share preparation across rungs;
- a supported workload whose escalation trigger falls outside the typed classes, which would change
  the classifier rather than the rule's structure;
- U1 failing, in which case M1b needs an interior-point factor path or the upstream corrector;
- U2 failing, in which case rung R2 loses Uno.

## Verification and uncertainties

| Claim | Label | Basis | Gap or settling evidence |
|---|---|---|---|
| Current-state findings F01–F17 | Interface-checked / Implemented path (source-traced) | Code maps, DR-1, DR-2, coordinator re-reads marked *verified* | Not executed. F01 would be settled by a KINSOL FP fixture with a domain failure inside the map |
| Pinned capabilities exist (`step_along_path`, `NoInitSetup`, Schur hook, second-opinion rungs) | Interface-checked | Pinned source in the native-solver skill corpus | U1: does `step_along_path` fit pse's active-set factor? Settle with a QP fixture across an active-set change. U3, U4: POUNCE same-structure and retry semantics |
| Candidate libraries' failure semantics | Secondhand lead | Evidence worker's source reading of PETSc `main`, Uno and Ceres | U2: a Uno contract probe at plan scope end. U5: a PETSc tagged release |
| Literature-derived prerequisites and triggers | Mixed: read / abstract / known | Mechanism cards with per-claim labels | Many classic sources were read only as abstracts. Placements rest on library contracts and standard results |
| Target contracts T1–T10, ladder, replacement rules | Proposed | DR-1 and DR-2 reasoning, reconciled here | Executed tests per mechanism under the repository's rhythm; no benchmarks |

**Coverage limits.**
- Discrete and global route internals, integrator internals, lexicographic sequencing and direct
  `pse-py` `MathService::solve` callers were not examined in depth.
- U6 (Anderson scaling inside KINSOL) and U7 (assemble-then-multiply JVP) were not re-verified by
  the coordinator.

None of these could change the G9 verdict. U1 and U2 could change two placements (M1b, R2) but not
the contracts.

## Authority changes and disposition

| Change | Route |
|---|---|
| RR1 escalation; supersedes the no-fallback clauses of ADR-0083 and ADR-0152 | New ADR **and** design review; `design:` PR for §18.7 |
| RR2 derived systems; replaces §17.5's value-only rule and touches §14.4 (derived view keys) and §19.1 | Same ADR or a companion ADR with review; `design:` PR |
| RR3 PS-09 clarification (`design:` PR for the §17 intro) | Governance: ADR plus review. The preferred form is a profile text amendment (profile 1.4), because the ambiguous word is in the principle itself; a binding interpretation note is the minimal alternative |
| RR4–RR7 | `design:` PRs for §16.6, §18.3, §18.10 and the code docs, under the RR1 ADR |
| RR8–RR10 | `design:` PRs for §17.6, §19.3, §19.4 and §20.3 (receipt projection) |
| Uno adapter with a new `-sys` workspace crate | Adding a crate needs an ADR and a design review; a new backend binding within the accepted execution decision would otherwise need only a short ADR |
| New registry vocabulary (strategy policy, mechanism kinds, origins, trigger classes, strategy trace relation) | Registry declaration plus `just codegen` within the plan |

**Disposition.**
- **Proposed owner of all findings:** a new solver-strategy plan in `docs/plans/`, created through
  `plan-creation`.
- **Current defects** (F01, F02's labelling part, F03, F04, F05 and the Ipopt reservation) are small.
  The maintainer may place them in the Plan 25k stream or in the new plan's first packet.
- **Ownership chain:** finding → scenario → ADR (RR1–RR3) → packet → evidence. The plan's
  disposition table will own status. This review keeps its observations unchanged.

## Decision

- **Behavioral and semantic adequacy:** revise. F01 (a PS-G3 status misclassification), F02, F03
  (G7) and F04 (G1) are narrow defects in current supported behavior. None publishes a false
  solution, because independent qualification holds. Every target mechanism is unresolved.
- **Architectural fitness:** G9 fails against the target. All six foundations are violated for the
  extension scenario.
- **Overall: Revise.** The proposed target A1 (T1–T10, the ladder, library-first placement and
  replacement rules RR1–RR10) is *Proposed* at the design level. Accepting it would close no
  implementation work.
- **Strongest evidence:** source-traced spine and seams; verified library facts; two independent
  assessments that agree on structure and placement.
- **Main uncertainty:** the U1 and U2 placements, and the maintainer's PETSc preference.

| Priority (by consequence) | Change | Findings and scenarios | Acceptance evidence | Proposed owner |
|---|---|---|---|---|
| 1 | Correct the typed outcome and settings record: FP/Picard recoverable → abort; refuse inert KINSOL settings; step-tolerance stop → stall class; reserve `warm_start_same_structure`; one retention-acceptance definition; correct §17.6 | F01–F05 | Targeted unit tests per defect (FP domain failure → `Evaluation`; inert settings refused) | Plan 25k or the new plan's first packet |
| 2 | Decide RR1–RR3 through ADRs | F06, F10, F14 | Accepted ADRs with review | Maintainer, then the new plan |
| 3 | Information-model contracts: T3 start proposals, T5 retained information (complete keys), T4 derived systems, T6 observation and ledger, T8 accuracy tiers, T9 route-request constructor, T10 method profiles | F07, F09–F11, F13, F14 | Native-free tests of admission, prediction keys and tier derivation | New plan |
| 4 | Strategy core: T1 planner, T2 transition, T7 driver with executor seam; migrate homotopy, profile chains, horizon, studies, block initialization, recycles, fitting and shooting; delete their private loops | F06, F08 | Scripted-executor tests of plans and transitions (S12); the horizon regression (S04) | New plan |
| 5 | Library-first mechanisms by family: prediction and path following (M1, M1b, M3, M2); KINSOL profiles and setup reuse (M4–M6); R1/R2 rungs including the Uno adapter (M7, M8); continuation tracker and homotopy rules (M9, M10); pseudo-transient and DAE initialization (M11, M12); structure as execution (M13–M15); Schur and multistart (M17); evaluation reduction and inner warm starts (M18, RR5); surrogates (M16) | F12, F15–F17 | Per-mechanism contract tests under the repository rhythm; never benchmarks | New plan |

**Order of prerequisites.** Priority 1 before any escalation code, because T2 reads those outcome
fields. RR1 and RR2 before any escalation or derived-system rung lands. T3, T5 and T9 before T1 and
T7, because the planner reads proposals, the retained inventory and route summaries. Within
priority 5 the families are independent once T4, T5, T8 and T10 exist. The user asked for all of
them, so the order reflects prerequisites, not expected benefit.

**Next consequential decision.** The maintainer's choice on PETSc (decline now, per this review, or
adopt as an adapter) and authorization to create the solver-strategy plan and draft the RR1–RR3
ADRs.
