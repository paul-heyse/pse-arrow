# DR-1 supporting assessment: solver-pipeline composition and domain model

Supporting assessment for the principal review
`docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md`.
It carries bounded judgments for pipeline composition and the strategy domain model only; the
principal review owns the combined scope, the overall decision and finding dispositions. Mechanism
placement (library vs. composition vs. bespoke), per-library fit and per-mechanism numerical
integrity belong to the sibling assessment DR-2; where this document assumes a placement it says so
(section 8).

## 1. Scope, baseline and inspection

| Field | Content |
|---|---|
| Subject | The solver pipeline as a composition of solves: strategy responsibility, start information, derived systems, escalation, retained numerical information, observations/budget/accuracy, user options, local testability |
| Boundary | `pse-runtime::workflow::{staged, modeling::engines, modeling::assessment, numerics, study_execution, horizon::driver, fitting}`, `pse-runtime::math::{staged, solves, initialization}`, `pse-backend-native::{routing, execution, kkt::advance}`; neighbours read for contract meaning only |
| Standard | Core 3.3, process-simulator 1.3, binding `pse-arrow` |
| Tier / purpose | Design tier, **target** purpose (maintainer target of 2026-10-03: all pertinent acceleration/globalization techniques in one pipeline, library methods first, no benchmarking premises) |
| Reviewer | DR-1, independent design reviewer (agent), 2026-10-03; not the author of the inspected code |
| Baseline | HEAD `f0b90258` plus the uncommitted Plan 25k tree, read as-is; a concurrent session was editing. Line numbers below are as read and may drift |
| Executed checks | None. No build, test or probe was run. Claims about current code are *Implemented* (source read) or *Interface-checked*; every recommendation is *Proposed* |

**Inspected directly.** Evidence folder files `mechanism-matrix.md`, `pipeline-map.md`,
`reuse-structure-map.md`, `controls-facts-metrics.md`, `colleague-input.md` (§1, §4–§6),
`mechanisms-globalization-and-decomposition.md` (X1–X6); architecture §15.5.1, §16.1–§16.6,
§17 (all), §18.1–§18.2, §18.6–§18.8, §19.1–§19.3, §13.6 (advanced step), §14.1, §14.3, §14.4;
ADR-0093 Outcome, ADR-0144, ADR-0145 Outcome, ADR-0152, ADR-0153 (head).
Source read: `workflow/staged.rs:1-360` (`Start`, `Overlay`, `Record`, `seed`, `predecessor`,
`execute`, `step`); `workflow/modeling/engines.rs` (`ModelingInitialization`, `retryable`,
`scientific_attempt_retry`, `initialize_model`, `attempt`, `homotopy`, `original`);
`workflow/modeling/assessment.rs:1-110` (`AssessedPoint::accepted`, `Obligations`);
`workflow/numerics.rs:20-140` (`CandidateDecision`); `math/staged.rs` (`serve`, `Scope`,
`NativeSession::step`); `math/solves.rs` (`PreparedSolve`, `SensitivityProgram`, `execute`,
`admit_step`, `conclude`); `pse-backend-native/src/execution.rs:400-720` (`adapter`, `LINKED`,
`Table`, `Retained`); `routing.rs:100-130, 540-620` (`Context.refusals`, `Requirements`);
`kkt/advance.rs:1-200` (`Advance`, `Prediction`, `Fallback`, `predict`);
`horizon/driver.rs:1200-1320` (advanced-step control); `study_execution.rs:335-530`
(continuation seeding, `primal_seed`, `memory_seed`); `fitting/modeling.rs:130-230`;
`fitting/profile.rs:400-445`. A grep located route-request construction sites
(`routing::Requirements {`) and fresh `Retained::default()` sites.

**Not examined.** Dynamics integrator route selection and IDACalcIC failure handling (DC-S09 is
assessed at the contract level only); `workflow/objectives.rs` lexicographic sequencing; the
`pse-py` callers of `MathService::solve`; KINSOL/Ipopt/POUNCE internals (DR-2); the C12 KINSOL
fixed-point defect beyond its consequence for trigger classification (DR-2).

## 2. Integrated assessment

The pipeline already has the right *single-attempt* spine: one step executor
(`MathService::execute`), adapter selection by contextual capability (`routing`), typed native
termination, original-coordinate re-observation and qualification, one candidate-use owner
(`workflow::numerics`), four-part compatibility stamps, typed start receipts, step-scoped overlays
over an immutable specification, and serde-derived settings identity. These are strong and should
be preserved unchanged as the inner contract of any strategy.

What the target needs, and the current design lacks, is a governed account of **what happens
between attempts**: which numerical route leads from the information already held to an
acceptable candidate of the original problem. Today that question is answered implicitly and
separately by each multi-solve workflow. Authored homotopy owns a fixed-shape step controller and
a retryability classification; fit profile chains own another step controller; the horizon owns
the only use of retained-factor prediction; studies own predecessor seeding by native warm start;
block initialization, recycles, conditional units, fitting, shooting and the factorable
fixed-assignment re-solve run their own loops with fresh retained state and hand-built route
requests. The "no fallback" rule holds because the mechanism is absent, not because a contract
governs it. Consequently each acceleration or globalization mechanism the target requires would
have to be inserted into several owners, with independent seeding, acceptance, budget and
recording decisions — the change amplification AP-01/AP-03 exist to prevent.

The core recommendation is to make the inter-attempt route an explicit, *pure* domain decision
with a small set of collaborating contracts rather than one central engine:

- a **solve task** names the original problem, its obligations and its budget; a **strategy plan**
  (pure, recorded, possibly "none") lists admitted rungs, and a pure **transition rule** chooses
  the next rung from typed observations; a generic **sequence driver** executes rungs through the
  unchanged step executor and completion;
- consumers keep their **domain target paths** (which parameter values, samples, pins or study
  points to visit next); the strategy owns only the **numerical transition** from information
  already held to the next target (predict, subdivide, prepare, correct, escalate);
- starts become **start proposals** with typed origin, transport and screening evidence, admitted
  by one start-admission operation; `CandidateUse` remains the permission over attempts and is
  not overloaded;
- **derived systems** become derived realizations of the original prepared view, identified and
  reused through the existing preparation owners and usable only while their original is in force;
- **retained numerical information** becomes a keyed store under complete compatibility keys, with
  a portable tier (sensitivity matrices, previous solutions, tangents) and a native tier
  (sessions, factors), and one prediction operation over it;
- **escalation** becomes a declared, bounded, outcome-typed continuation of a solve task through
  the existing routing assessment, replacing "no fallback after native failure".

Routing, adapters, completion and preparation keep their authority; the strategy composes them.
Section 4 develops each contract with a challenge case.

## 3. Foundation verdicts and gates (current pipeline against the target)

Scenarios are defined in section 3.3.

| Foundation | Scenario and evidence | Verdict |
|---|---|---|
| AP-01 Separation of concerns | DC-S10: adding a predictor or a constructed homotopy requires edits in `engines.rs` (homotopy loop), `horizon/driver.rs` (predict-or-solve), `study_execution.rs` (seeding), `fitting/profile.rs` (step control), `math/initialization.rs` (block loop); continuation step control exists three times, acceptance-for-continuation three times, start admission per workflow (DC-F01, DC-F08) | **violated** |
| AP-02 Stable contracts, replaceable implementations | DC-S11: single-solve routes satisfy PSE-S02 through the adapter table; the recycle causal map and nested implicit solves drive KINSOL directly (`initialization.rs:703-801`, `implicit.rs:214`) with non-standard `Compatibility` meanings, so replacing or adding a root solver changes those owners (DC-F11). The advanced-step `Advance` is consumed only through horizon-specific plumbing (DC-F05) | **violated** (root-solver replacement path) |
| AP-03 Composition over entanglement | DC-S03, DC-S07: `Staged` is modeling-bound (`ModelingPackage::prepare_analysis_attempt`, `Assessment`), so block initialization, recycles, conditional units, fitting, shooting and fixed-assignment re-solve each run their own loop with `Retained::default()` (DC-F02); a new strategy is a copied workflow | **violated** |
| AP-04 Domain model and semantic authority | Model adequacy: no concepts for solve task vs attempt, strategy/rung, derived system, start proposal with non-attempt origin, escalation trigger class, keyed retained information, intermediate accuracy (DC-F03, DC-F05, DC-F06, DC-F07, DC-F09). Authority: retained-state survival is decided by `AssessedPoint::accepted()` (`workflow/staged.rs` `execute`), not by `CandidateDecision`, contrary to the session's own doc comment (DC-F04); §17.6 documents `Start::Seed(k)` and value-only study seeding that the code does not have | **violated** |
| AP-05 Explicit structure and constraints | No-fallback holds by absence (pipeline-map §(e)); the homotopy schedule, retry classification and budgets are statement order inside `Initializer`; authored sequences and studies have no cross-step budget; selection reasons (sensitivity preference) are not recorded (DC-F07, DC-F09) | **violated** (for the target's strategy, escalation and budget) |
| AP-06 Local reasoning and testability | DC-S12: routing policy is pure over an injectable `Table` (satisfied there); start admission, `Staged` seeding and homotopy/retry policy run only through `NativeSession` → `MathService` → static `execution::adapter`/`LINKED` and are KINSOL-gated in tests (DC-F02) | **violated** |

| Gate | Result | Evidence / scope reason |
|---|---|---|
| G1 Authority | **fail** (narrow) | Three definitions of "accepted enough to keep or continue": session keep (`AssessedPoint::accepted`), completion (`CandidateDecision::permits_use`), block commit (`commit_block`); the first can disagree with the second (DC-F04). Source-traced, not executed |
| G4 Hidden behaviour | **pass** (current); target obligation | No hidden retries: executor has none, POUNCE retries pinned off. The target's escalation must remain declared and recorded (rule R1) to keep this gate passing |
| G5 Consistency and recovery | **pass** (current scope) | Overlays are step-scoped (`Overlay::compose` clones the original); failed steps seed nothing; limits are typed. Unresolved for the target until a strategy budget exists (DC-F09) |
| G6 Transformation and reuse | **fail** | `Advance` is reused under a key of parameter ids only, with no layout/profile/policy/value stamp, and `Retained::clear` keeps it (`execution.rs` `Retained`); retained sessions survive refused candidates (DC-F04, DC-F05). Under the target, derived systems and predictions add reuse paths that need complete keys (DC-F06) |
| G7 Truthful capability claims | **fail** (documentation) | §17.6 claims `Start::Seed(k)` and "stage, homotopy and study steps seed values only"; studies submit an explicit native `WarmStart` (`study_execution.rs` continuation → `with_start(previous)`) (DC-F03) |
| G9 Architectural fitness | **fail** | Follows AP-01, AP-02, AP-03, AP-04, AP-05, AP-06 violations above |
| PS-G3 (PS-08, PS-10, PS-11) | **unresolved** in this scope | PS-08: authored initialization is declared and restorable (strength); recycle policy is declared. PS-11: the retained factor and prediction that decide an advanced-step sample are not framed into lineage (DC-F05). PS-10: the C12 KINSOL fixed-point recoverable-trial defect, if confirmed by DR-2, misclassifies an evaluation failure and fails this gate |

### 3.3 Scenarios

| ID | Stimulus and conditions | Desired response | Current observation |
|---|---|---|---|
| DC-S01 | NLP parameter sweep; the active set changes mid-sweep | Each point predicted from the nearest certified neighbour when valid; at the active-set change the prediction refuses with its reason and a weaker start (or path-following rung) is used; every point answers to completion | Studies seed zeroth-order native warm starts only; KKT prediction is horizon-only |
| DC-S02 | Square root feed sweep; possible fold | First-order root prediction `x + X_p Δp` from the published response, subdivision on failure, branch continuity recorded | `X_p` is published (`evidence.root_response`) but no consumer predicts with it |
| DC-S03 | Fitting outer loop of nearby solves (profile chains, re-solves) | Chains use the shared transition (predict/subdivide/correct); the fit estimate's dependency on declared guesses stays explicit | Own step controller; fresh `Retained`; starts refused |
| DC-S04 | Rolling horizon advanced step (regression) | Unchanged behaviour: background solve retains factor, prediction applied as the sample's moves under horizon policy, refused prediction falls back to a full solve, recorded | Implemented and tested (`advanced_step_matches_full_resolve`) |
| DC-S05 | Edit and re-solve on one flowsheet | Value edit rebinds the view; prior solution or first-order prediction is proposed; derived/retained products of the old original are not reused as exact | Rebind is strong (§14.4); start is spec defaults unless the caller wires a seed |
| DC-S06 | Hard cold start near invalid property domains | Plan escalates on typed numerical outcomes through admitted rungs; never on resource/cancel/infrastructure; specification intact after failure; all attempts recorded | No escalation; homotopy only on authored endpoints |
| DC-S07 | Large BTF flowsheet with tears | Structure-derived block rungs and recycle maps compose with the same driver; tear choice explicit; block partial solutions become a start proposal | Block init and recycles are separate own loops |
| DC-S08 | Expensive nested property evaluation | Evaluation-level reuse stays inside evaluation contracts; strategy never makes evaluation history-dependent | Inner solves restart by design; inner accuracy uses a default policy |
| DC-S09 | DAE re-initialization after a discontinuity | Integrator owns consistent IC; on typed IC failure an algebraic-subsystem rung may prepare a start, recorded | Not traced (scope limit) |
| DC-S10 | Add a mechanism (learned initial guess; new homotopy type) | One mechanism declaration plus its implementation; planner and consumers unchanged | Requires edits in each consuming workflow |
| DC-S11 | Replace or add a root solver library (PSE-S02) | Adapter table and routing absorb it, including recycles and nested solves | Two KINSOL-direct owners must change |
| DC-S12 | Test the strategy planner without native solvers (PSE-S05) | Plan and transition testable as pure functions; the driver with a stub executor | Only routing is native-free |

## 4. Findings

Finding IDs are stable for this supporting assessment; the principal review may renumber.

### <a id="dc-f01"></a>DC-F01 — Inter-attempt numerical policy has no owner and is re-decided per workflow

- **Cause.** The decision "what to do next given the attempts so far" — continuation step control,
  predecessor seeding, response to failure, retry classification — is implemented inside each
  multi-solve consumer. Authored homotopy: `Initializer::homotopy` (grow ×`growth`, halve on
  `retryable()`, stop below `minimum_step`) and `retryable()`/`scientific_attempt_retry` in
  `engines.rs`. Profile chains: halve toward last accepted point (`fitting/profile.rs:428-440`).
  Horizon: predict-or-solve with `Fallback` (`horizon/driver.rs:1236-1300`). Studies:
  zeroth-order native seed from the predecessor (`study_execution.rs` continuation). Each also
  decides its own seed transport: homotopy steps seed semantic values only (`Staged::step` passes
  `previous: None`, so no native duals/barrier ever reach a homotopy step), studies submit only a
  native `WarmStart`.
- **Principles / gates / scenarios.** AP-01, AP-03, PS-08, DP-06; G9; DC-S01, DC-S03, DC-S10.
- **Evidence.** Implemented (source read: `engines.rs` `homotopy`, `retryable`; `staged.rs`
  `step`; `horizon/driver.rs` `control`; `study_execution.rs:335-360`).
- **Consequence.** Every mechanism in the target (prediction for studies, root tangents,
  constructed homotopies, Ψtc, escalation) must be inserted into several owners, each choosing
  its own seeding, acceptance, budget and recording; behaviours already diverge (homotopy forgoes
  native warm starts; studies never subdivide a failed transition; fitting profile steps never
  predict).
- **Correction.** Separate the consumer-owned **target path** (authored endpoints, sweep points,
  pins chosen by the profile's secant-to-threshold search, horizon samples) from the
  strategy-owned **transition** between the information held and the next target (DC-C1–DC-C3).
  Consumers stop deciding step control, seed transport and failure response.
- **Verification.** Adding a predictor rung (DC-S10) changes one mechanism declaration and its
  implementation, and the homotopy, profile, study and horizon owners are untouched; a grep for
  per-workflow step-halving and retry classification finds one owner.

### <a id="dc-f02"></a>DC-F02 — The staged primitive is modeling-bound and hard-wired to linked native execution

- **Cause.** `workflow::staged::Staged` combines generic sequencing (records, `seed`,
  `predecessor`, deadlines) with modeling-specific binding (`ModelingPackage::
  prepare_analysis_attempt`, `Assessment`), and executes through `NativeSession` →
  `MathService::execute`, which resolves adapters through the static `execution::adapter`/
  `LINKED` (`solves.rs:1040, 1063, 2304`). Consumers whose step binding is not a modeling case
  therefore cannot use it: block initialization runs its own loop on `NativeSession`
  (`initialization.rs`), recycles drive a KINSOL session directly, and conditional units,
  fitting, profile chains and shooting call runners with `Retained::default()`
  (`initialization.rs:628`, `fitting/oracle.rs:705`, `fitting/profile.rs:290`,
  `shooting.rs:1216`).
- **Principles / gates / scenarios.** AP-03, AP-06, DP-17; G9; DC-S03, DC-S07, DC-S12,
  PSE-S05.
- **Evidence.** Implemented (source read and grep). Routing, by contrast, already accepts
  `table: &Table` and has stub-adapter tests (`routing/contextual_tests.rs`, pipeline-map §4).
- **Consequence.** No shared multi-solve composition below the modeling workflow; strategy
  policy cannot be exercised without a linked KINSOL; reuse across nearby fitting/shooting solves
  is structurally impossible.
- **Correction.** Split the sequence core (records, start resolution, retained-information
  access, budget ledger, transition loop) from **step binders** (modeling case, block subsystem,
  causal map, fit oracle, shooting) and from a narrow **step executor** seam (production:
  `NativeSession`/`MathService` resolving adapters through an injected `Table`; tests: a scripted
  executor returning `SolveReport`s) (DC-C3).
- **Verification.** Block initialization and a fit profile chain run on the same sequence core;
  a strategy test constructs the core with a stub executor and no native feature.

### <a id="dc-f03"></a>DC-F03 — The start information model cannot express a start whose origin is not an accepted attempt of the same problem

- **Cause.** Starts are represented in four places with partial meanings: `StartSource`
  (per-coordinate value provenance, `cases.rs:36`), staged `Start::{Specification,
  Accepted(k)}` (`staged.rs:41-47`), native `WarmStart` (layout+backend-stamped payload) chosen
  by `StartPolicy`, and `StartReceipt`/`SeedTransformation` (record of what was submitted).
  Permission to seed is borrowed from `CandidateDecision::permits_use`/`permits_seed`, which is a
  verdict about a completed attempt. There is no representation for a predicted point (KKT
  advance, root tangent, secant), an auxiliary-path terminal, a block-sequence partial solution or
  a surrogate optimum as a *start*, no typed screening evidence (finite, in bounds, in evaluator
  domain, original residual), and the choice between semantic-primal and native-state transport
  is made per workflow. §17.6 describes `Start::Seed(k)` and value-only study seeding; neither
  exists.
- **Principles / gates / scenarios.** AP-04, DP-02, DP-05, PS-08, PS-11; G7 (documentation);
  DC-S01, DC-S02, DC-S05, DC-S06.
- **Evidence.** Implemented (source read); doc divergence verified against
  `study_execution.rs:344-357`.
- **Consequence.** Predictions or auxiliary points could only be introduced by overloading
  `CandidateUse` (making a non-attempt look like a seed-only candidate) or by ad hoc value
  overrides with `StartSource::Predecessor`, erasing the distinction between "a qualified
  neighbour's solution" and "an unevaluated linearization". Out-of-domain starts are discovered
  only as native evaluation failures.
- **Correction.** One **start proposal** concept generalizing `StartSource`/`Start` (DC-C4), with
  one start-admission operation that screens, chooses transport and records the receipt.
  `CandidateUse` stays the permission over attempts; result permission is never derived from a
  proposal.
- **Verification.** A horizon prediction, a study predecessor seed and a homotopy-terminal
  proposal flow through one admission operation; the receipt names origin kind and transport;
  §17.6 describes the code.

### <a id="dc-f04"></a>DC-F04 — Retention and continuation acceptance have competing definitions

- **Cause.** `NativeSession::step` keeps retained native state when its `assess` closure returns
  true; `Staged::execute` returns `AssessedPoint::accepted()` (candidate present, no error, model
  checks satisfied), not the `CandidateDecision`. The session doc comment says "a step that does
  not end with a usable, accepted candidate drops the retained native state". Block
  initialization uses `candidate_use(..).permits_use()` plus extent (`commit_block`), and
  `Retained::clear()` deliberately keeps a kept `Advance`.
- **Principles / gates / scenarios.** DP-01, DP-09, AP-04; G1, G6; DC-S01, DC-S04.
- **Evidence.** Implemented (read `workflow/staged.rs` `execute`, `assessment.rs`
  `AssessedPoint::accepted`, `math/staged.rs` `step`, `execution.rs` `Retained::clear`). The
  divergence on real outcomes (e.g. `IterationLimit` → `SeedOnly` with passing checks) is
  source-traced, not executed.
- **Consequence.** Native session state (Ipopt problem, POUNCE app, HiGHS basis) from a refused
  native outcome can serve the next step; once strategies chain many rungs, the reuse decision
  silently disagrees with the completion decision that governs seeding.
- **Correction.** The retained-information owner decides retention from the attempt's
  `CandidateDecision` and the entry kind (DC-C6); `AssessedPoint` becomes an input to completion
  only.
- **Verification.** One function decides retention; a native-free test of the sequence core
  shows a `SeedOnly` step drops native-tier state that requires result permission.

### <a id="dc-f05"></a>DC-F05 — Retained numerical information is single-slot, consumer-specific and incompletely keyed

- **Cause.** `Retained` holds one adapter session and one `Advance`. `Advance` is keyed by
  parameter ids and order only (`kkt/advance.rs` `predict`); it has no layout, profile, resolved
  policy, preparation or value-point stamp, survives `clear()`, and is reachable only through
  `PreparedSolve::retaining_factor` and the horizon's `Staged::native` closure. The root response
  `X_p` is computed, published (`evidence.root_response`) and dropped, never offered as a
  predictor. No secant/tangent state exists. Edits to fixed values that are not declared
  parameters have no parametric program (`CasePlan::parametric` covers declared parameters).
- **Principles / gates / scenarios.** DP-09, PS-11 (MUST for reuse dependencies), AP-04, AP-02;
  G6; DC-S01, DC-S02, DC-S04, DC-S05.
- **Evidence.** Implemented (source read `execution.rs` `Retained`, `kkt/advance.rs`,
  `horizon/driver.rs:1236-1258`); reuse-structure map Part A.
- **Consequence.** The cheapest high-value mechanism in the target (first-order prediction for
  nearby solves) is unavailable to studies, continuation, fitting and edit-and-re-solve; the one
  consumer relies on its own discipline (single use, `usable` check) for validity; the factor
  that decided an applied advanced-step sample is not a declared lineage dependency.
- **Correction.** A keyed retained-information store with a portable and a native tier and one
  prediction operation (DC-C6).
- **Verification.** A study point and the horizon both obtain predictions through the same
  operation; a layout or policy change makes a native entry unavailable (typed reason), while a
  value change marks it stale rather than exact; lineage frames the entry used.

### <a id="dc-f06"></a>DC-F06 — Derived problems exist but have no common concept, identity or admission rule

- **Cause.** The pipeline already solves many problems derived from the original: stage
  specifications (`Overlay.facts`), homotopy overlays (`Overlay.parameters`), block conditional
  cases (`CasePlan::conditional`), causal maps, the feasibility objective wrapper
  (`FeasibilityOracle`), the ℓ1 realization, factorable exports (`ExportTransformation`), the
  fixed-assignment re-solve, presolve, pinned profile parameters. Each carries bespoke provenance
  and identity (pipeline-map §(c)). §17.5 restricts continuation to value replacement ("homotopy
  steps therefore change values only"), so constructed homotopies, pseudo-transient augmentation,
  bordered arclength systems, regularized least-squares/QP models and reduced-space problems have
  no admitted route. Completion has no explicit notion of *which problem* a candidate answers to:
  an intermediate homotopy step's `Usable` means usable for its overlay specification.
- **Principles / gates / scenarios.** AP-04, DP-01, DP-05, DP-08, PS-06, PS-08; G6, G9;
  DC-S06, DC-S07, DC-S10.
- **Evidence.** Implemented (overlay and initialization source read); doc §17.5, §19.1.
- **Consequence.** Each new auxiliary formulation would invent its own identity, cache key and
  provenance; there is no rule preventing an auxiliary success from being mistaken for an
  original success other than per-workflow care.
- **Correction.** A **derived system** contract owned by preparation (DC-C5), the replacement
  rule R2, and an explicit "answers to" scope on attempt records so only original-problem
  attempts can carry original result permission.
- **Verification.** A Newton homotopy and a pseudo-transient step are prepared, keyed, rebound
  and recorded through the same contract as the existing stage and homotopy steps; existing
  `ExportTransformation` and block cases fit the same record without loss.

### <a id="dc-f07"></a>DC-F07 — Escalation is excluded by absence; the only trigger taxonomy is workflow-local

- **Cause.** No code re-routes after execution (pipeline-map §(e)); the rule is stated in §18.7,
  ADR-0152 and comments. The one classification of which outcomes justify another attempt is
  `ModelingInitializationAttempt::retryable()` and `scientific_attempt_retry` in `engines.rs`,
  private to authored initialization. Routing already has the mechanism a declared escalation
  needs (`Context.refusals`, used pre-execution for factorable refusals).
- **Principles / gates / scenarios.** AP-05, DP-12, DP-15, PS-09, PS-10; G4 (target), G7;
  DC-S06.
- **Evidence.** Implemented (source read `engines.rs`, `routing.rs:114-121`).
- **Consequence.** The target's globalization ladder cannot be built without either violating
  the stated rule or hiding retries; if added per workflow, trigger classification will diverge.
- **Correction.** Replacement rule R1; one trigger classification owned beside
  `NativeTermination`/`BoundaryClass`; escalation decided by the plan's transition (DC-C2) and
  re-routing through `Context.refusals` with a post-execution reason.
- **Verification.** A native-free test shows that `TimeLimit`, `ResourceExhausted`,
  `Cancelled`, `Panic`, infrastructure and contract failures never yield another rung, a
  certified infeasibility ends the task, and an exhausted budget yields a typed resource outcome.

### <a id="dc-f08"></a>DC-F08 — Route requests are constructed by hand in many production owners

- **Cause.** `routing::Requirements { .. Context { .. } }` is built in at least eight production
  sites (`solves.rs:1063, 1509, 1679`; `initialization.rs:71, 484`; `cases.rs:169`;
  `fitting/modeling.rs:139`; `shooting.rs:636`; `execution/factorable.rs:1444`), each choosing
  budgets, refusals, structure, prepared demands and settings. The fixed-assignment re-solve uses
  default NLP settings; block initialization applies one profile to every block.
- **Principles / gates / scenarios.** AP-01, AP-02, DP-01; G9; DC-S07, DC-S11.
- **Evidence.** Interface-checked (grep plus reads of the fitting and solves sites; test-module
  sites excluded by position). The exact count may move with the concurrent edits.
- **Consequence.** Every derived or auxiliary rung would add a site; contextual routing
  conditions (ADR-0152) can be honoured in one site and missed in another.
- **Correction.** One route-request constructor per prepared problem (original, block, derived,
  fit oracle) owned with the problem's preparation (DC-C9); strategy rungs call it.
- **Verification.** Production `Requirements` construction is confined to that constructor and
  routing's own tests.

### <a id="dc-f09"></a>DC-F09 — No strategy budget, no intermediate accuracy contract, no typed decision-grade observation

- **Cause.** Budgets are per step (`Controls.time_limit`, `iterations`) plus authored
  initialization's deadline and attempt cap; authored sequences and studies have no cross-step
  budget. Accuracy is the frozen final `ResolvedAccuracy` for every step, including homotopy and
  stage steps; there is no derived intermediate tier. Observations are rich but declared
  non-decisional ("metrics remain observations and are never an input to a decision",
  `solve.rs` `Evidence` doc), and some are only in truncated event streams (Ipopt restoration
  entries).
- **Principles / gates / scenarios.** AP-05, DP-11, DP-12, DP-20, PS-08; G5 (target); DC-S06,
  DC-S08.
- **Evidence.** Implemented (controls-facts-metrics §1, §3, §5; `engines.rs` policy).
- **Consequence.** Escalation, continuation and preparation work cannot be bounded together;
  in-solve triggers would have to read string metrics or be re-derived per consumer.
- **Correction.** A typed attempt observation projected once from `SolveReport`, one budget
  ledger per solve task, and derived intermediate accuracy tiers (DC-C7, DC-C8).
- **Verification.** Transition rules consume only typed observation fields; a strategy's total
  work is charged to one ledger and its exhaustion is a typed outcome.

### <a id="dc-f10"></a>DC-F10 — Method choice inside an adapter is not plannable or recorded

- **Cause.** Within an adapter, Auto leaves native defaults; the only fact-driven setting is
  POUNCE ℓ1 under authored `penalty(l1)`. Globalization-relevant options (Ipopt `resto_*`,
  `mu_*` other than strategy/init, POUNCE `mu_strategy`) are reachable only through raw option
  passthrough, outside typed identity semantics and outside any selection reason. The
  sensitivity-capable route preference is applied but not recorded as a reason.
- **Principles / gates / scenarios.** AP-02, AP-05, DP-15, DP-21, PS-09; G7; DC-S06.
- **Evidence.** Interface-checked (controls-facts-metrics §1.2, §2.1). Library-specific option
  fitness is DR-2's.
- **Consequence.** A strategy cannot choose, record or explain a method profile; user raw
  options can silently contradict a planned profile.
- **Correction.** Typed method profiles chosen by the plan and reserved from raw passthrough
  once typed (DC-C10); selection reasons recorded in the plan.
- **Verification.** Each planned profile appears in the plan record with its reason; the raw
  keys it owns are refused.

### <a id="dc-f11"></a>DC-F11 — KINSOL-direct paths bypass the adapter seam

- **Cause.** The recycle causal map (`initialization.rs:703-801`, `kinsol::Session::new`) and
  nested implicit solves (`implicit.rs:214`) construct KINSOL sessions without
  `BackendExecution` or runners, with their own `Compatibility` meanings (`profile:
  accuracy.key()`; all fields = problem identity).
- **Principles / gates / scenarios.** AP-02, DP-04, DP-17; G9; DC-S07, DC-S11.
- **Evidence.** Interface-checked (pipeline-map §3 and §(d)); not re-read in full here.
- **Consequence.** A second root-solver library or a strategy rung over the recycle map cannot
  reach these paths through the adapter table; stamps mean different things in different owners.
- **Correction.** The recycle map becomes a representation consumed through the adapter seam
  (KINSOL `Function::FixedPoint` already is an adapter-level representation); the nested implicit
  solver remains an evaluation-internal solver behind its `InnerSolver` contract but adopts the
  standard stamp meanings. Placement details are DR-2's.
- **Verification.** Adding a stub root adapter makes it eligible for the recycle map through
  routing; stamps have one meaning.

## 5. Recommended target contracts

All *Proposed*. "Owner" names the module that holds the authority; "stops interpreting" names
what consumers no longer decide.

### DC-C1 Solve task and strategy plan (answers Q1)

- **Consumes.** The original problem reference (prepared solve identity, bound values identity,
  resolved numerical policy key, intent, obligations); problem facts (`ProblemFacts`; structural
  summary: DM square/over/under, BTF block count and sizes, tear candidates; derivative
  availability; declared parameters with parametric-program availability; authored continuation
  endpoints; authored dynamics/holdup availability; declared stages; guards and bounds); the
  routing decision summary for the original (eligible adapters per class/representation, never
  re-derived); the retained-information inventory (keys and kinds only); the start proposals
  available; the resolved strategy policy (section 6); the workload context (single solve; nth
  point of a path with distance to its predecessor; outer-loop evaluation).
- **Decides.** An ordered set of admitted **rungs**. A rung is: mechanism kind; the problem it
  solves (original, or a derived system reference with its generation rule); a method-profile
  reference; the start proposal it consumes; an accuracy tier; a budget share; abandonment
  criteria over typed observations; escalation edges keyed by trigger class. Ordering follows the
  assumption ladder (X3: ordering → local elimination → physical parameter paths → constructed
  paths → constructed flows), never measured speed. "One rung: correct the original from the best
  admitted start" is the common plan; "none" (no preparation) is valid.
- **Produces.** `StrategyPlan` (serde, schemars, identity-framed), recorded with reasons per
  admitted and rejected mechanism (e.g. "Ψtc not admitted: no authored dynamics or justified
  pairing").
- **Owner and direction.** `pse-runtime::math::strategy` (pure functions). Depends on
  `pse-model` vocabulary (generated registry enums for mechanism kinds, trigger classes, origins),
  `pse-math` facts and `pse-backend-native` routing/capability types as data. Workflows depend on
  it; it depends on no workflow and constructs no evaluator, session or adapter.
- **Relation to neighbours.** Routing remains the only eligibility authority: the planner reads
  the routing summary and requests a routing decision per rung problem through DC-C9; it never
  ranks adapters. The step executor remains one attempt = one native solve with no internal
  retry. Completion remains the only permission owner. The staged primitive becomes the generic
  driver (DC-C3).
- **Consumers stop interpreting.** Homotopy, profile chains, horizon, studies, block
  initialization stop choosing step control, seed transport and failure response.
- **Preserved guarantees.** Original-coordinate acceptance; frozen `ResolvedAccuracy` on every
  original-problem rung; overlays step-scoped; no undeclared attempt.
- **Avoiding a god object.** Each mechanism family contributes a declaration (applicability over
  facts, information required, derived-system rule, triggers answered, abandonment observations)
  in one exhaustive table, like `execution::adapter`. The planner orders declarations; it holds no
  mechanism mathematics. A learned initial guess is one declaration producing a start proposal
  (origin `Learned`, screening required); a new homotopy type is one generation rule plus one
  declaration.
- **Challenge case (DC-S06).** Cold start near an invalid property domain: plan = [correct from
  spec start] → on `Evaluation`/restoration failure → [block-sequential preparation if DM square
  and BTF > 1] → [bounded constructed homotopy from a screened in-domain anchor] → [Ψtc only if
  authored dynamics exist] → each followed by an original correction. Must not escalate on the
  enclosing deadline, must stop on a certified infeasibility, must end within budget, and must
  leave the specification untouched (every rung is an overlay or derived system). A trivial
  problem must get a one-rung plan with no preparation work.

### DC-C2 Transition and escalation rule (answers Q4)

- **Consumes.** The plan, the task's attempt trace so far, the latest typed attempt observation
  (DC-C7) and its `CandidateDecision`.
- **Decides.** `Next = Done(attempt) | Rung(i, start) | Subdivide(target, fraction) |
  Stop(reason)`. Trigger classification is one function beside `NativeTermination`/
  `BoundaryClass` in `pse-backend-native` (generalizing `retryable()`), with three classes:
  *escalate* (local infeasibility, stationary non-root, iteration limit without a result,
  restoration failure, numerical/singular failure, recoverable trial failures, refused candidate),
  *terminal-scientific* (certified infeasibility, unboundedness, invalid model, unsupported,
  contract), *terminal-operational* (time/resource/pool limits, cancellation, infrastructure,
  panic).
- **Produces.** The next action plus its reason, appended to the trace.
- **Owner.** `pse-runtime::math::strategy` (pure); classification in `pse-backend-native`.
- **Consumers stop interpreting.** `engines.rs` retry classification and per-workflow halving.
- **Preserved.** Explicit backend selection is never substituted (rule R1); determinism given
  facts and outcomes.
- **Challenge case (DC-S01).** At the active-set change the KKT prediction refuses
  (`Fallback::ActiveSet`): the transition must choose the next rung (predecessor seed, or a
  path-following rung if admitted), not stop the sweep, and must record the refusal; an iteration
  limit at a later point must escalate, while a deadline reached by the enclosing study must not.

### DC-C3 Generic sequence driver with step binders and an executor seam (answers Q1, Q7)

- **Consumes.** A solve task, its plan, a **step binder** (maps a rung's problem and start
  proposal to a prepared step and its assessment: modeling case, block subsystem, causal map, fit
  oracle, shooting), and a **step executor** (`execute(step, predecessor) -> Outcome`).
- **Decides.** Nothing numerical beyond DC-C2's verdict; it owns effects: executing rungs,
  charging the ledger, retention updates through DC-C6, recording.
- **Produces.** The task result: the original-problem attempt that holds result permission (if
  any), the full attempt trace with each attempt's "answers to" scope, and the start proposals
  generated.
- **Owner.** Generic core in `pse-runtime::math` (below workflow, so block initialization,
  recycles, fitting and shooting can use it); the modeling binder stays in `workflow::staged`.
- **Executor seam.** Production: `NativeSession` with `MathService` resolving adapters through an
  injected `Table` (the routing seam already exists) instead of `execution::adapter()` static
  calls; tests: a scripted executor returning fixture `SolveReport`s.
- **Preserved.** Block commits remain "exact partial solutions" (only independently qualified
  coordinates commit); their committed values become a start proposal for the original, never an
  original result.
- **Challenge case (DC-S07).** A large BTF flowsheet: block rungs with per-block routes, a recycle
  rung over a declared causal map with an explicit tear set, then the simultaneous original
  correction from the composed proposal — one trace, one budget, no own loops.

### DC-C4 Start proposal and start admission (answers Q2)

**Verdict on Q2.** The existing types express *value provenance* (`StartSource`), *native payload
fit* (`WarmStart` with `layout`/`backend`), *what was submitted* (`StartReceipt`) and *permission
derived from a completed attempt* (`CandidateDecision::permits_seed`). They do not express a start
whose origin is not an attempt of this problem, its validity limits, its screening, or the choice
of transport. A new concept is warranted, but as a **generalization of `StartSource`/`Start`**, not
a parallel subsystem and not a new `CandidateUse` value.

- **Shape.** `StartProposal { origin, primal: SemanticId → value (original units), native:
  Option<WarmStart>, validity: Option<ValidityLimits>, screening: Option<StartScreening> }`.
  `origin ∈ {Specification, Declared(annotation/guess), Accepted(attempt), SeedOnly(attempt),
  Stored(solution), Predicted{basis: KktAdvance | RootResponse | Secant | Tangent, from, Δp},
  Auxiliary{derived system, path value}, PartialSolution{block sequence}, Learned{model},
  Surrogate{model}}`. `ValidityLimits` = parameter step, assumed active set or branch, staleness.
  `StartScreening` = finite; within bounds; within evaluator domains (guards/envelopes evaluated
  at the point); original residual summary. Permission is "start only" by type: no path turns a
  proposal into a result.
- **Consequential distinctions.** Equation-infeasible (normal for a start) vs out-of-domain
  (evaluation would fail; requires projection, a different anchor or a bounded rung) vs qualified
  (it came from an attempt with result permission). Semantic-primal transport (by `SemanticId`,
  survives layout and backend changes, drops absent coordinates explicitly) vs native-state
  transport (duals, barrier, basis, working set; requires `layout`+`backend` and, where the
  receipt's transformation path differs, refusal). Validity limits are evidence for ranking and
  for the horizon's applied-prediction policy; they never gate seed use.
- **Admission operation.** One owner (generalizing `admit_step` and `resolve_starts`): choose
  among proposals by origin rank and validity, screen, choose transport (native only when stamps
  and transformation path permit, otherwise semantic), and record a `StartReceipt` extended with
  origin kind and transport.
- **Consumers stop interpreting.** `Staged::seed`/`predecessor`, `memory_seed`, study
  `with_start(previous)` and fitting's start refusal become calls to admission.
- **Preserved.** `CandidateUse` remains the only permission over attempts; `seed_only` keeps its
  meaning (a completed attempt's lawful seed); `PreviousAccepted` still only reuses results.
- **Challenge case (DC-S04).** The advanced-step horizon applies a prediction as the sample's
  moves without correction. This is a domain policy of advanced-step NMPC, not a start: the
  start model must not grant it. The horizon keeps `HorizonDecision::Predicted` as its own typed
  decision, consuming the prediction and its validity evidence, and the prediction source is
  framed into the sample's lineage. A second challenge (DC-S02 with a fold): a root prediction may
  land on another branch; completion accepts any root that passes the checks, so branch
  continuity must be a declared policy (section 6), recorded from the transition, never inferred.

### DC-C5 Derived system (answers Q3)

- **Ownership.** Preparation (`pse-compiler` view preparation and `pse-math` plan derivation),
  not the strategy and not workflows. A derived system is a derived artifact (DP-01): rebuildable
  from the original view, its generation rule and value inputs; never authored, never edited.
- **Identity and reuse key.** Structural key = frame(original `view_key`, generation rule kind,
  structural rule parameters (selected rows/parameters, regularization or mass-operator source,
  partition), derivative order, evaluator profile). Value inputs (anchor point, `λ`, `δ`/`Δt`,
  previous iterate, regularization weight) are bound values, so a family of homotopy or Ψtc steps
  shares one prepared derived view and rebinds per step, exactly as today's value-only homotopy
  shares one view (§14.4). Retention uses the existing byte-bounded service cache (one reuse
  mechanism per scope).
- **Record (lineage).** Original identity (view key, bound-values identity, resolved policy key);
  rule and parameters; anchors with their own start-proposal provenance; path parameter value;
  incidence delta (the union incidence is what structural analysis of the derived system uses);
  coordinate map (identity for residual-level constructions; lifting map for reduced spaces;
  extra columns for bordered systems); **terminal identity** (a mechanically checkable statement
  that the derived system equals the original at a declared terminal value, or "none"); budget.
  Existing `ExportTransformation`, block conditional cases, stage selections and the feasibility
  wrapper fit this record.
- **Construction rule.** A generation rule composes over the original evaluator (e.g.
  `H(x,λ) = F(x) − (1−λ)F(x₀)`, `F(x) + M(x−x_k)/Δt`) so selected-function meaning (ADR-0144)
  is preserved by construction; a rule that rewrites rows containing an operational selection
  must refuse unless it re-admits the selection evidence.
- **Use rule.** Admitted only while its original is in force (identity match at each rung).
  Its attempts answer to the derived system; their candidates enter the original only as start
  proposals (`origin: Auxiliary`), except at a declared terminal identity, where the step *is* an
  original-problem step and answers to every original obligation.
- **Challenge case (DC-S05).** A user edits a value mid-strategy: the original identity changes,
  so derived products keyed on the old original are not reused as exact; the last auxiliary
  point may survive only as a stale proposal. Second challenge: a bordered arclength system adds
  a column (`λ` free): its layout differs, so only semantic-primal transport into the original is
  lawful.

### DC-C6 Retained numerical information and prediction (answers Q5)

- **Shape.** A keyed store with two tiers. *Native tier* (session-thread, `!Send`): adapter
  sessions, KKT factors (`Advance`), retained Jacobian/LU factors where an adapter can honour them.
  *Portable tier* (`Send`, may live in results and cross workers or durable storage): previous
  solutions in semantic coordinates, root response `X_p` (already published), KKT parametric
  steps, secant/tangent vectors and step histories.
- **Key.** `kind` + original or derived problem identity (`layout`, `profile`, `backend` for
  native entries; preparation identity and resolved-policy key for all) + the point it was taken
  at (attempt and the values identity) + parameter coordinates + the decision that permitted
  retention. Validity distinguishes *fits* (structure, profile) from *exact at* (values): a
  value change makes an entry stale, which remains lawful for prediction or modified-Newton
  reuse but not for an applied prediction.
- **Prediction operation.** `predict(target values, store) -> StartProposal | Unavailable(reason)`
  chooses by availability and declared validity: KKT advance (NLP, active set held), first-order
  root response, secant from two path points, zeroth order. Consumers: studies, continuation,
  fitting chains, horizon, edit-and-re-solve.
- **Owner.** Store in `pse-runtime::math` (session-owned for the native tier); the KKT backsolve
  stays in `pse-backend-native::kkt`; the root first-order step is ordinary composition over the
  published response.
- **Preserved.** Allowance charging of retained factors; mutable native state never crosses a
  worker; four-part stamps.
- **Challenge case (DC-S05).** An edit to a fixed value that is not a declared parameter has no
  parametric program, so `predict` returns `Unavailable(no_parametric_program)` and admission
  falls to the previous solution; a structural edit invalidates every native entry while
  semantic-primal proposals survive for coordinates whose identities persist.

### DC-C7 Typed attempt observation and one budget ledger (answers Q6)

- **Observation.** Projected once from `SolveReport` in `pse-backend-native`: termination
  category, candidate presence and kind, original residual and domain summary at the candidate or
  best iterate, trial rejections, regime crossings, terminal evaluation failure, restoration
  entered, iteration count, line-search failure or minimum step, Krylov failures, work consumed.
  Transition rules read only these typed fields, declared in each rung's criteria; string-keyed
  metrics stay presentation (rule R7).
- **Ledger.** One `WorkBudget` per solve task (wall time, attempts, native iterations,
  evaluations; preparation and compilation charged too) partitioned into rung shares and clamped
  by any enclosing deadline (as `Staged::step` clamps today). Exhaustion is a typed
  terminal-operational outcome; it never escalates.
- **Challenge case.** A strategy inside a study occurrence inside a durable job: the occurrence's
  deadline clamps the ledger; a partially executed plan publishes its trace as partial, never as
  complete.

### DC-C8 Accuracy tiers (answers Q6)

- **Rule.** `ResolvedAccuracy` stays frozen and is used unchanged by every original-problem rung.
  Intermediate tiers (prediction error budget, corrector tolerance on a path, inner tolerance of
  a reduced-space evaluation relative to the outer, Krylov forcing) are *derived* from it by the
  rung's declared rule and recorded; they are never user input and never carried by `Controls`.
  An intermediate-tier candidate can never hold original result permission.
- **Challenge case (DC-S08).** Looser inner tolerances in nested property evaluation change the
  derivatives the outer solver sees; the tier rule must state derivative-consistency obligations
  or refuse. Evaluation-level inexactness is outside the strategy (DR-2).

### DC-C9 One route-request constructor per prepared problem

- **Consumes / produces.** A prepared problem (original, derived, block, fit oracle) plus profile
  → `routing::Requirements` with its `Context`; post-execution refusals (rule R1) enter
  `Context.refusals` with a typed post-execution reason.
- **Consumers stop interpreting.** Budgets, structure witness, refusals and prepared demands
  (DC-F08).
- **Challenge case.** A regularized least-squares rung for a Root problem must route as an NLP
  with `least_squares` and Optimize intent from the derived problem's own facts, not the
  original's.

### DC-C10 Method profiles (contract shape only; library content is DR-2's)

Typed per-adapter profiles (KINSOL direct/Krylov+forcing+preconditioner, setup reuse, Anderson;
Ipopt initialization/restoration/barrier; POUNCE path following, Schur, batch) that the plan
selects from facts and records with reasons; once typed, their raw option keys are reserved.

### Dependency direction (proposed)

```text
pse-model (vocabulary: policy, mechanism kinds, origins, trigger classes)
   ↑
pse-math (facts, derived-system plan derivation)      pse-backend-native (routing, adapters,
   ↑                                                    trigger classification, observation,
   |                                                    kkt predict)
   └──────────────────────┬────────────────────────────┘
                          ↑
pse-runtime::math (strategy planner + transition [pure]; sequence core; retained store;
                   start admission; route-request constructor; MathService executor)
                          ↑
pse-runtime::workflow (binders and target paths: modeling, studies, horizon, fitting, shooting)
```

No new crate is needed; a new crate would require an ADR and buys nothing here.

## 6. Required authority changes and replacement rules

| ID | Current rule (authority) | Intent preserved | Replacement rule text (proposed) | Route |
|---|---|---|---|---|
| R1 | No backend fallback after native failure; reconsideration only on newly established scientific/representation incompatibility (§18.7, ADR-0152, ADR-0083 "no fallback engine") | Deterministic, explainable routing; no ungoverned retries after resource, cancellation or infrastructure failure; no hidden attempts; explicit selection honoured | "Before execution, selection follows class, rank and contextual eligibility. After execution, a solve task may make further attempts only as rungs of its resolved strategy plan, chosen deterministically from the plan, the problem facts and the typed numerical outcomes of earlier attempts. Each rung's adapter comes from the same contextual routing assessment, with earlier attempts entered as typed post-execution refusals. Escalation is triggered only by numerical outcomes that leave the problem undecided (local infeasibility or a stationary non-root, an iteration limit without a result, restoration or numerical failure, recoverable trial failures, a refused candidate); never by time, resource or pool limits, cancellation, infrastructure failure, panic, an invalid model, unsupported capability or a contract error. A certified infeasibility ends the task. All attempts are bounded by the task's work budget, recorded with rung, route, start and problem identity, and judged by the same original-problem completion. An explicit selection is never substituted: under it, rungs may change method profile, start or derived system, and change backend only when the selection names the alternatives." | New ADR superseding the affected clauses of ADR-0083/ADR-0152, with design review (alters D-level execution decision); `design:` PR for §18.7 |
| R2 | Homotopy steps change values only (§17.5, ADR-0093) | One model authority; transactional, restorable initialization | "Initialization and continuation may solve derived systems: realizations generated from the original prepared problem by an enumerated generation rule, recorded with rule, parameters, anchor provenance, incidence delta, coordinate map and terminal identity, and prepared and reused through the original's preparation owners. A derived system is admitted only while the original it names (view, bound values, resolved policy) is in force and never replaces it. Its candidates enter the original problem only as start proposals, except at its declared terminal identity, where the step is an original-problem step answering to every original obligation. Authored stages remain alternative specifications; value-only homotopy over authored endpoints is the parameter-path member of this family." | Same ADR or a companion ADR with review; `design:` PR for §17.5, §14.4 (derived view keys), §19.1 (overlay references a derived system) |
| R3 | Stopping budgets are not user controls (§16.6) | One authoritative final accuracy | Keep, and add: "Intermediate accuracy tiers are derived from the resolved accuracy by declared rung rules, recorded, and never govern an original-problem rung or a result." | `design:` PR §16.6 under R1's ADR |
| R4 | Libraries own every iteration (§17 intro, ADR-0083; PS-09 in the profile) | No bespoke Newton, line search or factorization where a qualified library provides it | The profile principle is not changed. Binding/blueprint interpretation: "Libraries own each solve's inner iteration, globalization and factorization. The project may compose library solves under declared strategies (sequencing, continuation, derived systems, escalation) and may own small path or pseudo-time controllers where no qualified library offers them with the required trial-failure contract, each with a stated library-consideration note." | `design:` PR §17 intro; binding note. Placement per mechanism is DR-2's |
| R5 | POUNCE hidden second solves pinned off | Truthful, recorded attempts | "Native internal retries may be enabled only as a planned method profile whose attempts the adapter surfaces as typed evidence." | `design:` PR §18.3 under R1's ADR; fit is DR-2's |
| R6 | Nested implicit solves never use a previous trial as start (`pse-math/src/implicit.rs`) | An evaluation is a function of its inputs | Out of strategy scope: evaluation-level reuse must keep evaluation a function of its inputs, or declare a uniqueness condition. DR-2 owns the replacement | DR-2 |
| R7 | "Metrics remain observations and are never an input to a decision" (`solve.rs` `Evidence` doc) | No decisions from strings or presentation | "String-keyed metrics never drive decisions. Typed attempt observations may, when a strategy rung's declared criteria name them." | R1's ADR; code doc |
| R8 | §17.6 start model (`Seed(k)`, "study steps seed values only") | Typed starts, no inference from labels | Replace with the start-proposal model (DC-C4); record origin and transport in the receipt; correct the study description | `design:` PR §17.6, §20.3 receipt projection |
| R9 | Fitting starts refused (§17.6, `fitting/modeling.rs:218-225`) | A fit's estimate depends only on declared guesses | "The fit estimate starts from declared guesses unless its strategy records another start origin in lineage; nested and repeated fit solves (profile chains, re-solves) use the shared transition and prediction." | Plan scope plus `design:` PR §19.4 |
| R10 | Study continuation edges seed only a predecessor's solution (ADR-0148, §19.3) | Explicit, recorded dependencies; no undeclared fallback | Keep edges and permissions; the transport over an admitted edge may be a prediction from the predecessor's retained information, recorded as such | `design:` PR §19.3 |

## 7. User-options and numerical-contracts catalogue

Classification: **Declared** (user-facing typed policy, identity-bearing), **Derived** (computed
from facts and resolved policy; recorded, never user input), **Reserved** (refused if supplied raw).
All declared options live in Rust serde types with schemars, generated into Python msgspec
documents (the `SolveSettings` route); no hand-written Python mirror.

| Contract / option | Class | Owner | Identity / lineage | Notes |
|---|---|---|---|---|
| Strategy mode: `off` (one direct attempt), `automatic` (planned from facts), `explicit` (authored rung list, still admitted) | Declared | `StrategyPolicy` in `SolveSettings` | Lineage request identity; not in the session `profile` stamp (like start and reuse) so rungs share retained sessions | `off` reproduces today |
| Mechanism families allowed/denied (prediction, derived homotopy, pseudo-transient, LM/trust-region auxiliary, block-sequential, reduced-space, Anderson/inexact profiles, multistart) | Declared | `StrategyPolicy` | Lineage | Denial is a recorded plan reason |
| Work budget (wall time, attempts, iterations, evaluations) for the whole task | Declared | `StrategyPolicy` | Lineage; not preparation | Per-step `Controls` limits remain per-rung caps |
| Escalation depth and backend alternatives under explicit selection | Declared | `StrategyPolicy` | Lineage | Without alternatives, explicit selection fixes the backend |
| Branch policy: `any_qualified_root` / `path_connected` (requires a path rung; fold crossings recorded) | Declared | `StrategyPolicy` | Lineage | Different starts can select different roots; never inferred |
| Determinism class (sequential deterministic; parallel multistart within tolerance) | Declared | `StrategyPolicy` | Lineage | DP-11 |
| Retained-information policy (allow factor retention; byte cap) | Declared | `StrategyPolicy` / `MathPolicy` | Not identity (resource), recorded | Charged to job allowance |
| Applied-prediction permission (advanced-step horizon only) | Declared | Horizon controller | Lineage per sample | Unchanged domain policy; not a start permission |
| Start policy, reuse policy | Declared (existing) | `Controls` | Existing | `PreviousAccepted`/`Explicit` keep meaning; proposals add origins |
| Authored continuation endpoints, stages, initialization overrides | Declared (existing, authored) | Model / `InitializationOverrides` | Existing | Become target paths and stage rungs |
| Step-control constants (initial step, growth, minimum step) | Derived with typed expert override | Mechanism declaration | Lineage when overridden | Today user-facing in `ModelingInitialization` |
| Rung ordering, mechanism admission, method-profile choice | Derived | Planner | Plan identity in lineage, with reasons | Never from benchmarks |
| Intermediate accuracy tiers | Derived | DC-C8 | Recorded per rung | Never `Controls` |
| Prediction validity checks, screening thresholds | Derived | DC-C4/DC-C6 | Recorded | From resolved budgets |
| Final tolerances, scales, bound relaxation, warm-start options, derivative constancy flags | Reserved (existing) | `ResolvedAccuracy`, adapters | Existing | Unchanged |
| Raw native options owned by a typed method profile (e.g. Ipopt `start_with_resto`, `least_square_init_*`, restoration and barrier options) | Reserved once typed | Adapters | Raw options remain in `Controls::identity` | Remaining raw keys stay an escape hatch |
| Strategy trace (rungs, routes, starts, problem identities, observations, ledger) | Result contract | Sequence core | Generated `runtime.*` relation from the registry | Exposed to Python as typed results |

## 8. Alternatives and what would reopen the recommendation

| Alternative | Assessment |
|---|---|
| **A0 Current baseline** (per-workflow policies; library-internal globalization only) | Deterministic and simple, but cannot reach the target; each added mechanism amplifies across owners (DC-F01). Retained as the `off` mode |
| **A1 Proposed** (pure plan and transition, generic driver, start proposals, derived systems, keyed retained information, declared escalation) | Removes per-workflow policy; testable natively-free; extension by declaration. Costs: new vocabulary and result relation, migration of six consumers, derived-view preparation, an ADR. Benefits are *Proposed* |
| **A2 Library-owned strategy** (e.g. PETSc SNES composition/LOCA as the strategy owner) | Libraries can supply rungs (DR-2), but none owns multi-solve composition across studies, horizon, fitting and original-space qualification; adopting one as owner would duplicate the adapter seam and typed outcomes. Not recommended as owner |
| **A3 Simplest viable** (keep workflows; extract a shared `continue_to(target)` helper and a shared trigger classifier; no plan object) | Fixes DC-F01 partially and DC-F07's taxonomy; cheapest first increment. Leaves selection unrecorded, no shared budget, no declared user policy, so it does not reach "seamless automatic selection" with recorded reasons. Reasonable as the first migration step toward A1 |

**Reopen if:** (a) DR-2 places most mechanisms inside libraries as method profiles, so plans
rarely exceed one or two rungs — then the planner may collapse into method-profile selection at
preparation plus A3's helper; (b) derived systems cannot share prepared views across rungs
(every rung recompiles), making derived-system preparation cost dominate — then library-native
forms (e.g. a library pseudo-transient integrator) may own them, with DC-C5 kept as the record
contract; (c) a supported workload requires escalation on a trigger outside the typed classes —
then the classification, not the rule's structure, changes.

## 9. Uncertainties and placement assumptions for DR-2 reconciliation

- **Assumed placements** (my contracts are agnostic, but examples assume): KKT prediction is
  pinned-library (`pounce-sens-core` via `kkt::advance`); the root first-order predictor is
  composition over the published `square_response`; constructed homotopies and Ψtc are derived
  systems compiled through Symbolica and solved by existing adapters (composition plus small
  controllers); block-sequential solving is composition over existing block cases; Anderson,
  inexact Newton, setup reuse and Ipopt/POUNCE initialization/restoration options are method
  profiles inside adapters; reduced-space solving builds on `pse-math::implicit`. If DR-2 places
  NPC or pseudo-transient inside a library (PETSc), those become method-profile rungs rather than
  derived systems; DC-C5 still governs their record.
- **Composition over the original evaluator** (DC-C5 construction rule) is assumed sufficient for
  residual-level homotopies and Ψtc derivatives; bordered and reduced-space systems need
  structural changes whose compile cost and derivative orders DR-2 should confirm.
- **C12** (KINSOL fixed-point/Picard recoverable returns) determines whether recoverable trial
  failures on the recycle route are classified correctly as escalate-class triggers; DC-C2
  depends on that correction.
- **Ipopt `ReOptimizeTNLP`** availability changes whether an Ipopt native-tier entry can be a
  reusable factor; DC-C6 treats it as optional.
- **Nested implicit inner warm starts and inner accuracy** (`ResolvedAccuracy::from_policy` with
  a default policy in `implicit.rs`) are evaluation-level and excluded from the strategy; DR-2
  owns their contract.
- **Unexamined in this scope:** dynamics IC failure handling (DC-S09), lexicographic sequencing,
  `pse-py` direct `MathService::solve` callers. The count of route-request sites (DC-F08) and the
  session-keep divergence (DC-F04) are source-traced against a tree under concurrent edit and
  were not executed.
- **Disposition owner.** None exists; a new solver-strategy plan under `docs/plans/` would own
  these findings once adopted. Prerequisite order: DC-C4 and DC-C6 (information model) and DC-C9
  enable DC-C1–DC-C3; R1/R2 must be decided before any escalation or derived-system rung lands.
