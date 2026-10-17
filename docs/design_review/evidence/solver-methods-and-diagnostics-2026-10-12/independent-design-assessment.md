> Supporting evidence for [the solver methods, selection and diagnostics review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Independent assessment by a fresh `design-reviewer` (Opus, xhigh). It received the standard, the functional target, the maintainer's requirements, the raw wave-1 evidence and neutrally phrased questions, but not the principal reviewer's conclusions. Published unchanged; it is a bounded supporting judgment. The principal review reconciles it and owns the combined decision. IR-nn finding identifiers are local to this document.

# Independent design assessment: solver method selection, observation and diagnostics

> **Role.** This is a bounded supporting judgment from the independent design reviewer. The coordinator's principal review owns the combined verdict, finding disposition and publication. Suggested path: `docs/design_review/evidence/solver-methods-and-diagnostics-2026-10-12/independent-design-assessment.md`.
>
> **Standard.** Core 3.4, with Heuristics for Efficient Architecture 1.0 and template 3.4; process-simulator profile 1.5; binding `pse-arrow`. Tier: design. Purpose: target.
>
> **Baseline.** HEAD `ff5a4aa2f` plus the uncommitted Plan 36 working tree (fingerprint `45ceea51b0698679`), read as it stands. Plan 36c is treated as baseline and not reviewed.
>
> **Evidence.** Every claim is *Interface-checked* (source I read) or *Proposed* (remedies), unless it is marked as a worker lead I did not re-read. Nothing was built, run or probed.

## 1. Bounded judgment

**What holds.** The pipeline runs one attempt well, and its admission layer is the strongest part of the decision architecture:

- `routing::Requirements`, `problem_classes`, `admit` and `decide_assessed` form a pure function of compiler facts, per-adapter capability records and an injected `Context`/`Table`.
- It keeps capability refusal, pending evidence, pending artifacts and readiness as distinct typed states, and records them (`Decision::row` → `runtime.route_decisions`).
- The between-attempt walker `strategy::next_automatic` is also pure.
- Candidate use is settled by independent original-coordinate assessment (`OriginalConclusion::satisfied`), never by native status.

The coordinator's remedies should preserve all of these.

**What falls short.** The three-layer requirement needs a *model* of method choice. Today, choice exists as code paths. The model would have to keep three things apart:

- what is admitted;
- in what order, and why;
- which observation leads to which next candidate.

The current design fuses them into three mechanisms that consumers reproduce independently:

1. a constant integer rank on each adapter (`Capability`/`automatic()`), re-applied through `min_by_key` in at least four places;
2. the statement order of `PreparedSolve::automatic_operations` in `pse-runtime/src/math/solves.rs`;
3. an append rule in `automatic_backend_alternatives`, keyed on "the first decision class the current adapter owns".

As a result:

- Layer 3 (advisory history) has no seam to reorder or tune.
- Layer 2 (observation-driven switching) has neither a typed input nor a way for the bridges to hand over an iterate.
- Layer 1 sees class facts but no structural or start-point features. Per-block choice is impossible because every block inherits one backend.

The observation architecture was designed for qualification and work charging, not for decisions, diagnosis or measurement:

- Decision-relevant statistics are either untyped (`SolveReport.metrics`, event `values`) or not bound at all.
- No record joins problem features, the decision made, cost and outcome.
- No attempt carries wall time.
- Diagnosis runs only on request, and native indices are not mapped back to model elements for numerical failures.

**Verdicts (supporting):**

- **Architectural fitness (G9): fails.** AP-01, AP-03, AP-04 and AP-05 are violated for the representative changes. AP-06 is split: admission is locally testable, escalation is not. AP-07 is unresolved, pending one premise.
- **Behavioral adequacy: fails narrowly.**
  - PS-08's recorded recycle iteration history is absent (a MUST).
  - G7: a capability string claims timing that is never read, and the code's failure-driven SCIP escalation diverges from the §18.7 contract text.
  - G2: library-reported iteration counts appear as "unknown" in typed work evidence for four adapters.
  - PS-G3 holds, because publication rests on independent assessment.
- Together these support a **Revise** in the principal review.

**The SCIP disagreement is settled for A1.** SCIP *is* appended as a failure-driven alternative after a failed Ipopt or POUNCE attempt whenever its retained eligibility is `Ready` or `SupportedPendingArtifacts` (§5). B4's mathematical argument against doing this is sound and shapes IR-02.

## 2. Scope, grounding and what I verified

**Subject.** The subject is the decision architecture that takes a prepared mathematical problem to an executed method sequence, plus the observation, diagnostics and measurement architecture. Suppliers are the compiler facts and structural analysis. Consumers are workflows (initialization, blocks, recycles, studies, fitting, shooting, dynamics), persistence and Python. Out of scope: pool and concurrency architecture, and property-model internals. Physical-semantics slots (PS-01–PS-03, PS-G1) are not applicable: the subject consumes no physical quantities beyond prepared numerics.

**Grounding.** I judged only against core 3.4, the profile's *Functional target*, blueprint §0/§0.2, and the maintainer's settled requirements:

- three layers;
- diagnostics and measurement as product capability, with adoption resting on mathematical rationale rather than benchmarks;
- libraries that are free to obtain.

I read ADRs, §15–§18, register rows and Plan 36c as current design only.

**Read and verified myself:**

- `solves.rs`: `automatic_operations`, `automatic_native_route_operations`, `automatic_backend_alternatives`, `profile_for_automatic_backend`, `pounce_recovery_baseline`, and the `complete_context` selection.
- `staged/driver.rs`: the automatic loop and the append after an unsatisfied native report.
- `strategy.rs`: `next_automatic`, `automatic_decision_key`, `observe_native`, `failure`, `permits_numerical_continuation`, `work_native`.
- `routing.rs`: `Context`, `Requirements`, `oracle_facts`, `problem_classes`, `decide_assessed`, `Decision::row`.
- `pse-math/src/facts.rs`: `ProblemFacts`.
- `solves/blocks.rs`: `automatic_block_count`, `prepare_blocks`.
- `execution/scip.rs`: the capability record.
- `solve.rs`: `SolveReport`, `Evidence`, `WorkEvidence`, `CallbackEvidence`, `Event`, `Metric`, `NativeTermination`.
- `callback.rs`: failure-event recording.
- `uno/callback.rs`: the Ipopt iteration dispatch.
- The Ipopt and Uno C++ bridges, at their stop paths.
- `pounce/profiles.rs`: the second-opinion trigger.
- `highs.rs`: `Highs_clearSolver` before each solve.
- `strategies/requests.rs`: recycle defaults.
- `dynamics.rs`: the `Auto` candidate list.
- `results.rs`: metric persistence.
- Greps for typed work setters, `Requirements {` construction sites and production reads of `metrics`.

**Taken as worker leads, not re-read:** A1's census of dead `NumericalMechanism` values; A2's identity and persistence tables (I spot-checked `Decision::row` and `push_native_metrics`); B1–B4's library API inventories (I verified the local binding claims listed above, not upstream headers); A3's block, separator and fitting decomposition details.

## 3. What the decision model has to represent

The functional target asks for solves that are fast enough for edit/re-solve, sweeps and recycles, and diagnosable when they fail. The maintainer's layers turn that into a domain with these phenomena.

**Problem features**, at three lifetimes:

- *structural*, per preparation: class facts, size, sparsity, the BTF/DM block profile, per-block affinity and coupling, a bound-shape census, derivative order and cost;
- *value/start*, per case and start: residual scale, start origin, distance from the last accepted point;
- *trajectory*, for dynamics: algebraic share, event and schedule density, horizon length.

**Methods:** an adapter together with a typed method profile within it. Examples: Ipopt monotone versus adaptive μ, exact versus limited-memory Hessian, KINSOL line search versus Picard versus fixed point, KLU versus Krylov, HiGHS simplex versus IPM, Diffsol BDF versus ESDIRK.

**Decisions:**

- *Admission* (layer 1, binary, with reasons): this exists.
- *Preference* (layer 1, an order with reasons): this exists only as a constant rank.
- *Transition* (layer 2): a typed observation pattern leads to a cooperative stop and a next admitted candidate with a start-transfer rule. This exists only as "any unsatisfied report appends a fixed list".
- *Advisory adjustment* (layer 3): a permutation or tuning inside the admitted set, recorded as an input. This is absent.

**Attempt record:** features key, the plan decision and its reasons, an observation summary, work and time, outcome class.

The distinctions that change behavior are:

- refusal versus failed trajectory (present);
- admitted versus preferred (absent);
- observation classes that call for *different* next methods: local infeasibility, restoration failure, stall, iteration limit, domain/evaluation failure and ill-conditioning are all projected to "continue down the list" (absent);
- whole-problem versus per-block decisions (absent).

Under AP-04 the question is whether this model exists and governs behavior. The rest of this document argues that it largely does not, outside admission.

## 4. Current decomposition (slot 2, compact)

| Responsibility | Owner | Hidden decision or invariant | Consumes / exposes | Local test setup |
|---|---|---|---|---|
| Class derivation, eligibility, rank-min choice | `routing.rs` | Class order; the one eligibility rule; refusal states | `ProblemFacts`, `Capability`, injected `Table`/`Context` → `Decision` | Pure. `test_context(table)`, `policy_*_for_test` (good) |
| Candidate catalogue and order | `PreparedSolve::automatic_operations` (`solves.rs`, 5,410 lines) | Ladder order prepared → Causal → ReducedSpace → Block → Direct; reduced-space adapter by rank | A full `PreparedSolve`, the global `execution::LINKED` | Needs a prepared algebraic source and the linked build |
| Escalation edges | `automatic_backend_alternatives` (`solves.rs`) | "First class the current adapter owns"; lower rank; Ready/PendingArtifacts admitted; only with Auto plus `BackendSettings::Default` | Retained `route_decision`, `LINKED` | Same |
| In-adapter second opinion | `pounce/profiles.rs` | Four native codes → library rungs | `termination.code`, typed `pounce_statistics.quality_escalations` | A `SolveReport` value |
| Walk and stop | `strategy::next_automatic` | First un-attempted candidate permitted by start rules; continuation gate over the observation class | Candidates, attempted set, `AutoObservation`, work | Pure (good) |
| Sub-selectors | Dynamics `Auto` (`[Diffsol, Idas]`), Hessian `Auto` (solves and fitting), presolve `Auto`, recycle request (`anderson: 0`, unweighted tears), block backend inheritance, `complete_context` | Each a local default | Local inputs | Varies |
| Observation carrier | `SolveReport` | `Evidence` typed for qualification, retry and work; `metrics`/`events` untyped, "never read by decisions" | Adapters → runtime | Value type |
| Persistence | `route_decisions`, `solve_strategy_events`, `solve_metrics`, `run_lineage`, `solve_runs` | Separate relations joined by `run_id`/`step` | Arrow relations → Python `TableStream` | — |

Admission runs one way, from facts and capability records to a decision. Everything after admission depends on `PreparedSolve` and the global adapter table. That dependency direction is the root of the AP-06 split (IR-01).

## 5. The SCIP disagreement, settled from source

I read the path end to end. *(Interface-checked.)*

1. `execution/scip.rs` lists `ProblemClass::SmoothNlp` in SCIP's `automatic_classes`, and the record's comment says "smooth NLP behind the local NLP adapters". Its rank is the highest in the table (A1: 5).
2. In `staged/driver.rs`, after every dispatched operation that returns a native report whose original conclusion is not satisfied, the driver calls `step.automatic_operations(report, …)` and appends new candidates, deduplicated by binding identity.
3. With `last = Some(_)`, `automatic_operations` returns the POUNCE second-opinion profiles (only when `report.backend == Pounce`) followed by `automatic_native_route_operations()`, which maps `automatic_backend_alternatives()`.
4. `automatic_backend_alternatives` requires Auto selection, default settings and a `Ready` retained decision that selected the current adapter. It takes the first class in `decision.classes` that the current adapter owns automatically, sorts that class's automatic adapters by rank, and pushes every lower-ranked adapter whose eligibility state is `Ready` or `SupportedPendingArtifacts`. It skips `Refused` entries and stops at `PendingEvidence` or a missing entry.
5. For a continuous optimization problem the first class Ipopt owns is `SmoothNlp`, so the list is `[Pounce, Scip]` whenever SCIP's eligibility is not refused. SCIP's factorable artifact demand typically yields `SupportedPendingArtifacts`, which qualifies.
6. `next_automatic` dispatches the appended candidates. `permits_numerical_continuation` lets Limited (which includes native `Infeasible` and `Unbounded`, per `observe_native`), Stalled and NumericalFailure continue. Each alternative is recorded as `MechanismKind::NativeGlobalization`.

**Conclusion.** SCIP is a failure-driven fallback for SmoothNlp whenever its static eligibility holds. B4's search ("grep for `fallback|escalat` near scip") missed it because no such words appear.

§18.7 says SCIP is selected automatically "wherever the others are ineligible". §18.6 says "a backend change requires named selected alternatives". The code goes further than both: an Auto request names nothing. Whether the contract or the code is right is a policy question. Judged against the standard and the maintainer's "mathematical rationale" requirement, B4's argument holds: a local NLP failure gives no evidence that the problem fits global spatial branch-and-bound, which is exponential in the worst case, needs finite boxes and a factorable structure, and answers a different question. IR-02 takes this up.

## 6. Change scenarios and journeys (slot 4)

| ID | Stimulus | Today's edit / behaviour | Owners touched |
|---|---|---|---|
| S01 | Add a method profile (for example Ipopt adaptive-μ as a first-class profile, or Uno `filtersqp` for small NLPs) | There is no profile concept below the adapter. Options travel as raw `controls.options` (the POUNCE ladder inserts `mu_strategy` as a text option). Making it preferable needs a new rank or a statement-order edit in `automatic_operations` | Adapter settings, `solves.rs` ordering, possibly routing |
| S02 | Add a selection feature (a BTF block profile, or KKT dimension) | Extend `ProblemFacts` and its three producers (`from_plan`, `oracle_facts`, `conic_facts`). Every one of the 12 production `Requirements {…}` sites must supply it (`solves.rs` ×3, `derived.rs`, `initialization.rs` ×2, `paths.rs` ×2, `fitting/modeling.rs`, `shooting.rs`, `modeling/cases.rs`, `workflow/diagnostics.rs`). Then the rule that consumes it has nowhere to live except a new eligibility reason, because preference is a constant | pse-math, routing, 12 call sites |
| S03 | Add an escalation edge (KINSOL fails on a square system → Ipopt feasibility; Ipopt local infeasibility → POUNCE ℓ1) | Today a square problem has classes `[SquareRoot, SmoothNlp]`, but alternatives come only from the first class KINSOL owns, so Ipopt is never offered after KINSOL fails, although layer 1 admitted it. Adding the edge means special-casing `automatic_backend_alternatives` | `solves.rs` |
| S04 | Per-block method (affine block → sparse LU; boxed block → NLP) | `prepare_blocks` uses `Route::Native(original.backend())` for every block. `PreparedInitialization` selects per block but `validate_profile` forces one settings value for all. `oracle_facts` hard-codes `affine_rows = false`. `Block.coupling` has no consumer (A3) | blocks.rs, initialization.rs, routing, compiler |
| S05 | Test selection policy without native solvers (PSE-S05) | Admission: yes (`test_context`). Walk: yes. Catalogue and escalation: no, because they live on `PreparedSolve` and read the global `execution::LINKED`, so whether SCIP escalation happens depends on the build's linked features | — |
| S06 | Upgrade a library (PSE-S06): Uno past `loose_*`, SUNDIALS ≥ 7.3, SCIP 10.1 | Absorbed by the bridge and adapter owners (good). New observation APIs land as untyped metrics unless the evidence types change | Adapter, plus evidence types |
| S07 | A study of N points where the first candidate consistently fails | Each point replays the whole ladder (for example Ipopt → POUNCE rungs → SCIP). No history, no aggregation (`StudyReport` has per-point outcomes only) | — |
| J1 | Edit → re-solve | Warm-start and reuse mechanics are mature (prior review; Plan 36c). The plan is recomputed from scratch and is not recorded as reusable; no observation of the previous solve informs the next choice | — |
| J2 | Recycle that will not converge | KINSOL `KIN_FP` with `anderson: 0` and `damping: 1` (plain substitution) by default, unweighted tear heuristic by default; final totals only, no per-sweep history; no successor after failure; the user sees an exhausted fixed point without a contraction diagnosis | requests.rs, recycle.rs, kinsol.rs |
| J3 | Infeasible or ill-posed problem | Structural refusal before solving is strong (PS-G2). After solving, local infeasibility maps to Limited and continues the same list; the least-infeasible point names violated rows; no failure-conditioned structural or conditioning diagnosis runs | strategy.rs, diagnostics on request only |
| J4 | Study over many cases | See S07. Per-point cost lives only in `solve_metrics` name/value rows and strategy events with no time | — |
| J5 | Dynamic start and event | `Auto` takes the first non-refused of `[Diffsol, Idas]`; no stiffness or event-density feature; IDAS statistics such as order, step, error weights and setups are unbound; `Report.statistics` (JSON values) is not persisted (B3, A2 leads) | dynamics.rs |

## 7. Foundation verdicts (slot 6)

| Foundation | Verdict | Scenario and evidence |
|---|---|---|
| AP-01 Separation of concerns | **Violated** | Preference and transition policy is not hidden behind one owner. It is spread across the effect-binding owner (`automatic_operations`, `automatic_backend_alternatives` in `solves.rs`), the adapter crate (POUNCE ladder, dynamics `Auto` list), routing (rank-min) and workflows (recycle defaults, fitting's own Hessian `Auto`). S03 and S01 each touch the preparation/effect owner, so policy and effect change together. |
| AP-02 Stable contracts | **Satisfied** at the adapter seam, with one gap noted elsewhere | Capability records and bridges absorb library upgrades (S06). The missing cooperative-stop contract is a missing capability (IR-05), assessed under AP-04 and AP-05 rather than as leaked implementation. |
| AP-03 Composition | **Violated** | S02 propagates into 12 hand-built `Requirements` sites (prior F07, still open). S03 and S04 cannot be added at a composition point: edges and per-block routes are statement order and inheritance, not composable entries. Recycle, initialization, fitting and shooting each assemble selection inputs themselves. |
| AP-04 Domain model and authority | **Violated** (adequacy) | The model lacks admitted-versus-preferred, typed transitions, selection features, per-block decisions and history (§3). Authority: the rank fact has one declaration, but the "lowest rank in class" policy is re-expressed in `decide_assessed`/`select_assessed`, `complete_context`, the reduced-space adapter pick and `automatic_backend_alternatives`, each with a different filter (representation `Roots`, `Nlp`, eligibility or none). |
| AP-05 Explicit structure | **Violated** | `route_decisions` records admission but not rank, class-order reasons, the sensitivity pre-filter, the alternative list's origin, or the `complete_context` producer route (which executes while the retained decision says `Refused`). `decision_identity` is a hash of inputs: reproducible but not explanatory. Escalation edges can be discovered only by reading `solves.rs`. |
| AP-06 Local reasoning and testability | **Split: satisfied** for admission and walk; **violated** for catalogue and escalation | S05: `automatic_backend_alternatives` and the reduced-space pick read the global `execution::LINKED`, while routing receives its `Table` by injection. Escalation behavior therefore depends on linked features and a prepared algebraic source. |
| AP-07 Execution fit | **Unresolved** | Within one solve, escalation is bounded by the attempt allowance and the scope deadline, which is sound. S07 has a structural amplification: a study repeats identical failing ladders per point, and an appended SCIP attempt can consume the remaining deadline per point. Whether this is material depends on an unknown premise: how often first candidates fail across realistic studies. The attempt record in IR-06 would answer that. Observation cost (H26) is proportionate today: metrics are written only when the relation is requested, and events are bounded by `Controls.history`. |

**G9: fail.** AP-01, AP-03, AP-04 and AP-05 are violated, and AP-06 is violated for the policy part.

## 8. Gate judgments (slot 6)

| Gate | Judgment | Evidence |
|---|---|---|
| G1 Authority | **Pass**, with a warning | No two independently mutable definitions of one fact were found. The rank constant is single, and `complete_context` is documented as a producer-only context with direct dispatch guarded. The re-expressed rank-min *policy* is the AP-04 concern (IR-01). |
| G2 Semantic fidelity | **Fail (narrow)** | `WorkEvidence` says "absent when the native API supplies none". Yet typed `iterations` is set only by Ipopt, Uno, PETSc and conic. KINSOL (`KINGetNumNonlinSolvIters`), POUNCE (`statistics.iteration_count`), HiGHS and SCIP report iterations natively, but these reach only `metrics`, so the typed ledger reads "unknown" where the library knows. Strategy events label a class-changing SCIP fallback `native_globalization`. IR-04. |
| G3 Validity | **Pass** in scope | Settings, contracts and structural admission are refused before workers. |
| G4 Hidden behaviour | **Pass** | Escalation is recorded in strategy events with backend and mechanism; `complete_context` is documented in source. |
| G5 Consistency and recovery | **Pass** | Throw-based stops veto the candidate, so no partial iterate is published as a solution. |
| G6 Transformation and reuse | **Pass** in scope. Forward condition below | An advisory history layer that reorders or tunes *changes which result is produced*, so it becomes a result-affecting input (IR-06 concern). |
| G7 Truthful capability claims | **Fail (narrow)** | `execution/ipopt.rs` claims "…callback counts and timing", but no Ipopt timing is read; "timing" is only Rust event elapsed time (B1, confirmed by grep). §18.7 states SCIP-only-where-others-ineligible while the code escalates on failure (§5). A1 counts 10 registry mechanisms with no producer; that is a lead, not re-verified. |
| G8 Library leverage | **Pass** for bespoke machinery | I found no reimplemented solver controller. The gaps are *unused* library capability (slot 8), which are target gaps rather than G8 failures. |
| G9 Architectural fitness | **Fail** | §7. |
| PS-G1 | Not applicable | No physical quantities in scope. |
| PS-G2 Well-posedness | **Pass** within inspected scope | Structural analysis precedes solving and carries semantic identities (`routing::Structure`; A3). Not re-traced to diagnostics text. |
| PS-G3 Numerical integrity | **Pass** | `observe_native` projects termination for *decisions only*. Finish requires `OriginalConclusion::satisfied`, and Acceptable never publishes on status alone. |
| PS-08 (MUST, under PS-G3's principles) | **Not met** | The recycle iteration history required by PS-08 is not recorded. `CausalMap` drives `KIN_FP`, `kinsol.rs` pushes no per-iteration events, and `callback.rs` pushes only `*.failure` events. Totals come from post-`KINSol` getters. The default tear method is the unweighted heuristic, while a weighted HiGHS route exists only as a separate tool step (`flows.rs`, A1/A3 lead). IR-08. |
| PS-09 | **Partially met** | The class is derived and selection by capability is recorded (strength). Selection by declared capability for *within-class* method profiles is absent (IR-01, IR-03). |
| PS-10 | **Partially met** | Typed outcome mapping is strong. Failures name model elements for constraint violations (least-infeasible rows), not for numerical failures (IR-07). |

## 9. Findings

### IR-01: Method choice has no explicit model separating admission, preference and transition

**Principles and gates:** AP-04, AP-01, AP-03, AP-05, AP-06, DP-14, DP-15 · G9 · S01–S05.

**Cause.** The admitted set, the order of preference and the transition rules are not represented as data with reasons:

- Preference is an adapter rank constant plus the statement order of `PreparedSolve::automatic_operations`.
- Transition is an append rule in `automatic_backend_alternatives`.
- Sub-selectors (dynamics `[Diffsol, Idas]`, Hessian `Auto` in `solves.rs` and again in `fitting/modeling.rs`, presolve `Auto`, recycle defaults, block backend inheritance, `complete_context`) are local defaults in their owners. Several sit in adapters, contrary to DP-14 ("provider selection and policy do not live in adapters").

**Evidence.** See §4 and §7. *Interface-checked:* `next_automatic` only walks a list built elsewhere, and `automatic_decision_key` hashes it. The rank-min policy is re-expressed in four places with different filters. The global `LINKED` is read in `solves.rs`, while routing receives an injected table.

**Consequences.**

- A history layer has nowhere to reorder.
- A new preference feature cannot express a preference at all; it can only become a new eligibility refusal or a new rank.
- Recorded decisions cannot say why one method preceded another.
- Escalation policy cannot be tested without a prepared problem and the right linked build.

**Correction (Proposed).** A pure *solve plan* operation with no effects and an injected table. It consumes features (IR-03), capability records extended with typed method profiles (IR-01/S01), request constraints and optional advisory evidence. It produces:

- the admitted candidates `(adapter, profile)`, each with an admission reason (reusing `routing::admit` unchanged);
- an order, with the named rule behind each position;
- a transition table (IR-02).

`automatic_operations` becomes a binder that prepares the plan's entries. The 12 `Requirements` constructions collapse into one plan builder per representation. Dynamics, Hessian, presolve, recycle and block sub-selectors become plan outputs. The plan is persisted with the route decision.

**Verification.** Each of S01–S04 is one declaration (a profile or rule) plus focused plan tests that use a scripted table. `automatic_backend_alternatives`, the duplicated `min_by_key(automatic())` sites and the adapter-resident defaults are deleted. A plan unit test exercises SCIP escalation with a test table instead of the linked build.

### IR-02: Transition edges are blind to the observation and bounded by the class that happened to be selected

**Principles and gates:** AP-04, PS-09, DP-15, G7 · S03, J2, J3.

**Cause.** A failed attempt appends the same lower-ranked same-class adapters whatever the failure mode:

- `observe_native` collapses local infeasibility, unboundedness, iteration limit and other stops into `Limited`, and numerical and evaluation errors into `NumericalFailure`.
- `permits_numerical_continuation` then continues all of them identically.
- Edges never cross the selected class: KINSOL failure has no successor although `SmoothNlp` is in the decision's own `classes` and Ipopt was admitted.
- Explicit typed settings disable all native alternatives.
- SCIP's global branch-and-bound is entered after a local NLP failure (§5).

The one observation-specific transition, the POUNCE ladder keyed on four `ApplicationReturnStatus` codes plus `quality_escalations`, is good practice: it lowers library-declared rungs into acting profiles without copying POUNCE's driver. It is confined to one adapter.

**Consequences.**

- Recovery does not follow mathematical rationale. A local-infeasibility verdict calls for feasibility-first, ℓ1 or restoration-oriented methods. A stall calls for globalization or trust region. Ill-conditioning calls for scaling or regularization. Evaluation and domain failure calls for start repair.
- Expensive certification work is spent where it answers the wrong question.
- The behavior diverges from §18.6 and §18.7.

**Correction (Proposed).** Transitions become named rules in the plan's table: an observation class or pattern from the typed families in IR-04 leads to a set of admitted candidates, with a start-transfer rule and the mathematical rationale stated in the rule. A class-crossing edge is admitted only when layer 1 admitted the target (for example SquareRoot ⊂ SmoothNlp feasibility). SCIP after a local failure is removed from the default table. It remains available under `certify`, under a declared global policy, or as a bounded diagnostic sub-solve (B4 §6.3).

**Challenge.** Could this reject valid recoveries? Every edge's target must still be admitted, and explicit requests stay authoritative. A user who wants global fallback declares it. **Verification:** plan tests of rule → candidates over scripted observations; strategy events carry the rule identity.

### IR-03: Layer-1 facts carry no structural or start-point preference features, and per-block decisions are impossible

**Principles and gates:** AP-04 (adequacy), PS-05, PS-09 · S02, S04, J5.

**Cause.**

- `ProblemFacts` holds class facts only: no nnz, block profile, per-block affinity or coupling, bound census, conditioning estimate or start residual.
- The structural products that exist are not joined to blocks: `StructuralAnalysis.blocks[].coupling` has no consumer, and `presolve::Facts.affine` is not attached to `PreparedBlock` (A3).
- `oracle_facts` hard-codes `affine_rows = false`, so every block derived from an oracle classifies as SquareRoot or SmoothNlp.
- `prepare_blocks` routes every block to the original backend. For a square system with boxed bounds that KINSOL refused, every block then runs Ipopt even when the block itself has no boxes.
- Dynamics `Auto` uses no stiffness, event-density or algebraic-share feature.

**Consequence.** Layer 1 can refuse but not prefer. Per-block method choice and structure-shaped main solves (prior F15) remain unreachable. A history layer would key on the wrong things.

**Correction (Proposed).** A derived `SelectionFeatures` product with explicit lifetimes:

- structural features, computed once per preparation from existing analyses (BTF, DM, affinity, coupling, sizes, bound census);
- value/start features, computed per case and start (scaled residual at the start, start origin);
- trajectory features for dynamics.

The plan consumes them for the whole problem and per block. Blocks receive their own plan entries. **Verification:** an affine block reaches the affine sparse-LU realization, and a boxed block in an otherwise unboxed system reaches an NLP adapter, both in plan tests without native solvers.

### IR-04: The typed observation vocabulary cannot carry what layer-2 decisions and diagnosis need

**Principles and gates:** DP-02, DP-21, PS-07, PS-10 · G2 · J2, J3, J5.

**Cause.** `Evidence` is designed for qualification, retry gating and work charging:

- four work counters;
- `CallbackEvidence` with three fields;
- flags.

The rule "decisions read `evidence`, never these keys" is sound by itself, but without promotion it locks decision-relevant data in untyped `metrics` and string-keyed event `values`:

- Ipopt per-iteration values: restoration mode, regularization δw, α_pr/α_du, line-search trials.
- POUNCE: restoration calls and quality escalations (partly read, through the typed library struct).
- KINSOL: β-condition failures, backtracks, function norm.
- Krylov: convergence failures.

Some is not bound at all:

- Ipopt `Statistics()` and `TimingStats()`;
- Uno's per-iteration residuals and multipliers;
- KINSOL per-iteration history;
- IDAS order, step, setups and error weights;
- KLU `rcond`/`condest` (computed in `dynamics/linear.rs`, then discarded);
- SCIP structured statistics;
- the HiGHS run time.

These are B1–B4 leads; I confirmed the local absences named in §2. Typed `iterations` is missing for KINSOL, POUNCE, HiGHS and SCIP (grep).

**Consequence.** No in-solve or between-attempt rule can be written against stable meaning, and diagnosis needs metric-name archaeology.

**Correction (Proposed).** Promote a small set of typed observation *families* keyed to the method family rather than to the library:

- interior-point NLP progress;
- Newton root progress;
- fixed-point contraction;
- integrator step health;
- branch-and-bound progress;
- LP/QP progress.

Each adapter maps its native data into its family; raw `metrics` stay as an unpromoted archive. Typed `WorkEvidence.iterations` is set wherever the library reports iterations.

**Challenge.** Typing could erase library-specific detail. It does not if the raw archive stays. **Verification:** IR-02 rules compile only against family types, and no decision reads `metrics`.

### IR-05: The bridges cannot stop cooperatively and hand over the iterate

**Principles and gates:** PS-09, PS-11, AP-05 · S03, layer 2.

**Cause.**

- Ipopt `intermediate_callback` always returns `true` and stops by `throw Terminal{}`. `finalize_solution` returns early under the terminal latch, so the iterate is lost, and Ipopt creates no `SolveStatistics` on the exception path (B1).
- Uno's termination callback always returns 1 and stops by throwing, which Uno converts to `ALGORITHMIC_ERROR`.
- POUNCE returns `false` only when `state.evaluate` fails.
- KINSOL 7.1.1 has no per-iteration hook. Its `KIN_FP` residual callback sees every iterate, so a contraction history is free to compute (B2).

**Consequence.** Layer 2's natural realization is: observe → stop cooperatively → re-solve on another method from the retained iterate (B1 §4). That cannot keep the iterate or the statistics. Every switch restarts from the submitted start, which wastes the trajectory and loses the evidence for diagnosis.

**Correction (Proposed).**

- A typed *policy stop*, distinct from cancel, deadline and abandon, delivered through each library's cooperative path (Ipopt `false`, Uno `0`, POUNCE `false`).
- The stopped iterate and statistics are retained as an `Auxiliary`/`Partial` start proposal, screened like any other.
- The stop predicate is a pure value passed in controls and evaluated in the callback over the IR-04 family observation. The policy decides *when to abandon a trajectory*, never how to step, so PS-09's library ownership is preserved.

**Verification:** a scripted policy stop on Ipopt yields `User_Requested_Stop`, a retained iterate and non-null statistics. This is probe P2.

### IR-06: There is no attempt record joining features, decision, cost and outcome, and no history key

**Principles and gates:** DP-21, DP-09, DP-11, AP-07, H26 · S07, J4; maintainer requirement 2.

**Cause.**

- The outcome (`solve_runs`), the admission decision (`route_decisions`), the strategy events (with work counts but no time) and the metrics (name/value) are separate relations joined only by `run_id`/`step`.
- `SolveReport`, the strategy `Event` and `canonical_attempts` carry no wall time.
- `preparation_identity` and `profile_identity` fold in full controls and the native build, which is right for exact reuse and wrong as a history key. No coarse structure-and-class key is persisted.
- Studies aggregate nothing. Dynamics statistics are not persisted.
- `environment_identity` has no host or CPU (A2 leads; `push_native_metrics` and `Decision::row` verified).

**Consequences.**

- Layer 3 has nothing to consume.
- Measurement, required product capability, means reconstructing joins over untyped rows.
- S07 amplification cannot be seen.

**Correction (Proposed).** One compact typed attempt relation per attempt:

- features key, from IR-03 structural features;
- plan identity, candidate position and rule;
- observation-family summary;
- work;
- wall and CPU time per phase where the library exposes it, and a total wall time always;
- outcome class.

Study summaries and history statistics are DataFusion aggregations over that relation, not bespoke summarizers.

**Forward condition on the maintainer's layer 3 (a material concern, not an objection).** Reordering or tuning within the admitted set changes *which* local solution a nonconvex problem reaches, and which candidate succeeds first. History is therefore a result-affecting input. Under PS-11 and DP-09 it must be recorded: each attempt cites the history snapshot identity it consulted, and the determinism class becomes "same inputs plus same history snapshot → same plan" (DP-11). Studies should freeze one snapshot per study so that point results do not depend on completion order. **Verification:** a plan computed under a recorded snapshot is reproducible, and removing the snapshot reproduces the layer-1/2 plan.

### IR-07: Failure diagnosis is not triggered by failure and does not reach model terms for numerical failures

**Principles and gates:** PS-10, PS-07, DP-21 · J3; maintainer requirement 2.

**Cause.**

- `diagnose_case`, `diagnose_linear` and `diagnose_jacobian_optimization` run only on request (A2).
- Native evidence that names the cause is reduced or unbound: KLU `singular_col` becomes a message, IDAS `GetEstLocalErrors`/`GetErrWeights` (the states limiting the step) are unbound, and FERAL `n_tiny` and the condition estimate are unbound (B2, B3 leads).
- Constraint-violation diagnostics do name rows; this part is good.

**Consequence.** An engineer facing "numerical failure" must re-run diagnostics by hand and map solver indices to equations.

**Correction (Proposed).** A failure-diagnosis operation that consumes the attempt record plus prepared structure. It runs under a declared policy and budget only when a plan exhausts, and it maps native indices to semantic identities through the existing coordinate maps. Its typed findings are persisted. It is diagnostic only and never changes the outcome (DP-18). **Verification:** an exhausted square solve with a structurally fine but numerically singular block yields a persisted finding naming the block's equations and variables.

### IR-08: Recycle convergence policy and history do not meet PS-08

**Principles and gates:** PS-08 (MUST), DP-12 · J2.

**Evidence.** See §8. The recycle request defaults to `anderson: 0`, `damping: 1.0`, and the unweighted heuristic when no tears are declared (`requests.rs`, verified). No per-iteration history exists.

**Correction (Proposed).**

- Record per-sweep ‖G(x)−x‖, per-tear deltas and a running contraction estimate inside the existing `KIN_FP` residual callback, at no extra evaluations (B2).
- Make tear method, acceleration and limits a recorded policy, chosen by the plan (IR-01) from features such as SCC count and tear cost.
- Add IR-02 transitions for recycles: contraction ρ ≥ 1 leads to Newton or quasi-Newton on the tears (library-owned: KINSOL line search on F = x − G(x), or PETSc QN) or to the simultaneous Direct solve.

Wegstein and dominant-eigenvalue extrapolation are legitimate bounded project composition under PS-09, because no pinned library offers them (B2).

### IR-09: Declared vocabulary and capability claims exceed the implemented routes

**Principles and gates:** G7, DP-16, DP-24.

**Evidence.**

- The Ipopt diagnostics claim (verified).
- The prepared-rung slot's only callers are `#[cfg(test)]` code in `math/surrogate.rs` (verified).
- A1 counts 10 dead and 3 filter-only `NumericalMechanism` values (lead).

**Correction (Proposed).** Correct the claim, or bind Ipopt timing (IR-04). Retire dead mechanism values or give them producers through the plan. Prediction and path code that implements `kkt_predictor`/`secant_predictor` behavior should record itself under those mechanisms.

### IR-10: Library reuse in LP sequences is discarded

**Principles and gates:** PS-11, DP-13 · J1, J2 (tear MILP and reconciliation sequences).

**Evidence.** `Highs_clearSolver` runs before every solve (`highs.rs`, verified), discarding HiGHS's retained basis and factor. Reuse survives only if the runtime round-trips `WarmStart`, and the capability record still claims native coefficient/bound updates. The IPM that actually runs (IPX or HiPO) is not recorded (B4).

**Correction (Proposed).** Clear only when the structure or the explicit start policy requires it. Record the effective method. **Verification:** probe P5.

### How the findings relate

IR-01 is the shared prerequisite *contract*: IR-02, IR-03 (per-block) and layer 3 all need a plan to express themselves. IR-04 is a prerequisite for IR-02, IR-05 and IR-07.

IR-06 depends on IR-01 (plan identity), IR-03 (features key) and IR-04 (observation summary), but its time and outcome fields can land independently. By consequence, the urgent defects are the narrow behavioral ones (IR-08, IR-02's SCIP edge, IR-09), and these do not need IR-01 to be corrected.

The remedies fit together if the plan stays a pure consumer of admission rather than a second admission authority. They would conflict if the observation families (IR-04) were defined per library: transition rules would then multiply per adapter.

## 10. Strengths that must constrain remedies

1. **Pure, injected admission.** `routing::admit`/`decide_assessed` take an injected `Table` and `Context`, keep `Refused`/`PendingEvidence`/`SupportedPendingArtifacts`/`Ready` distinct, and refuse falling through past pending evidence. The plan must call this, not restate it.
2. **A pure walker with an effect boundary.** `next_automatic` plus the injected execute and assess steps in the driver. Keep the pattern; move the content (order, edges) into data.
3. **Candidate use independent of native status.** Finish requires `OriginalConclusion::satisfied`. No layer-2 or layer-3 mechanism may bypass it.
4. **Library-declared rungs lowered to acting profiles.** The POUNCE ladder is the model for adapter-contributed transition rules: the library owns the rungs, the common driver admits and assesses each.
5. **Capability refusal versus failed trajectory**, and terminal classes (cancellation, resource, contract, infrastructure, panic) that stop rather than escalate.
6. **"Absent is unknown, never zero"** in `WorkEvidence` and `Metric::Unavailable`. The IR-04 promotion must preserve it.
7. **Explicit selection is never substituted.** History and transitions apply only inside what the request permits.

## 11. Library fit (template slot 8)

| Capability | Pinned library mechanism | Current use | Recommendation |
|---|---|---|---|
| NLP statistics and timing | Ipopt `IpoptApplication::Statistics()`, `IpoptData::TimingStats()` with `timing_statistics=yes` (C++ headers already compiled by the bridge) | Unbound | Bind in the bridge, into IR-04's interior-point family and IR-06 time. Makes register R-48's premise obsolete |
| Cooperative stop | Ipopt `intermediate_callback → false`; Uno termination callback `0`; POUNCE `false` | Throw-based (Ipopt, Uno) | IR-05 |
| Ipopt warm iterate | `warm_start_entire_iterate`, `warm_start_target_mu` | Unused | Plan profiles for re-solve sequences |
| POUNCE | `SolveStatistics` (bound), per-factor inertia (visible in the local `Lease`), batch, `TnlpPresolveSession`, `set_warm_start_iterate` | Partial | Set typed iterations. Batch is a study-level option for POUNCE-convex already; NLP batch for studies is a plan choice |
| Uno | Per-iteration residuals and multipliers in the termination callback, `notify_acceptable_iterate`, dual warm start | Arguments ignored | Bind into IR-04. Note that upgrading past the pin breaks `loose_*` |
| Root solves | KINSOL counters (bound), β-fails and backtracks; SUNDIALS ≥ 7.3 `KINSetDampingFn`/`KINSetDepthFn`, 7.8 log queue | Totals only | Fixed-point history from the existing callback now. A vendor or system SUNDIALS move is ordinary dependency judgment |
| Root alternatives | PETSc SNES QN/Anderson/NGMRES/VI (`vinewtonrsls` for two-sided boxes on square systems), `SNESSetConvergenceHistory`, TAO BRGN | Only `newtontr` and NASM bound | Candidates for IR-02 edges, since VI fits "square with boxes" better than an NLP reformulation; bind convergence history |
| Dynamics | IDAS `GetIntegratorStats`, order, step, `GetEstLocalErrors`/`GetErrWeights`, setups; Diffsol `order()`, `state().h` | Partly bound; not persisted | IR-04 integrator family, IR-07 limiting-state diagnosis, IR-03 trajectory features (ESDIRK versus BDF by event density) |
| LP/MIP | HiGHS `getRunTime`, retained basis, `writeModel`; IPX/HiPO identity | Basis cleared; run time unbound | IR-10; model export for replay of failed attempts |
| Global | SCIP `SCIPcollect*Statistics` → `SCIP_DATATREE`, `SCIPwriteOrigProblem`, emphasis meta-settings, 10.1 solving phase | Unbound | Typed branch-and-bound family; emphasis as a typed profile (S01) |
| Factor diagnostics | KLU `rcond`/`condest`/`singular_col`; FERAL `n_tiny`, condition estimate | Computed then dropped, or unbound | IR-07 and IR-03 conditioning features |
| HSL | MA27/57/86/97 (and MC19 equilibration in Ipopt); POUNCE `ma57` feature | Excluded (R-34) | Admissible under the maintainer rule if CoinHSL's free licence is obtainable (rule impact IR-RC04) |
| Aggregation | DataFusion over Arrow attempt relations | — | Study and history summaries as queries (IR-06), not bespoke code |

**Bounded project composition (PS-09).** The project legitimately owns:

- the plan (admission is consumed; preference, transition and advisory adjustment are owned);
- the policy-stop predicate (when to abandon, never how to step);
- Wegstein and dominant-eigenvalue recycle extrapolation;
- index-to-model mapping in failure diagnosis;
- the attempt record.

Nothing above reimplements a Newton step, line search, factorization or barrier controller.

## 12. Alternatives considered (slot 9)

| Alternative | Assessment |
|---|---|
| **Current baseline** | Strong admission; policy in statement order. Fails S01–S05 locality. Has no seam for layer 3 |
| **Proposed plan (IR-01) over existing admission** | One pure owner for preference, transition and advisory adjustment. Consumes `routing::admit`, persists reasons, testable with a scripted table. Cost: a new contract and migration of 12 construction sites and the sub-selectors. Revisit if the method-profile catalogue turns out small enough that ranks per profile suffice |
| **Extend ranks: a per-class, per-profile rank table** | Simplest change. Gives an order but no reasons, no observation-specific transitions and no feature-conditioned preference. Layer 3 would mutate ranks, a global mutable authority. Rejected |
| **Library-owned selection** (Uno `auto` preset, HiGHS `choose`, POUNCE `solver_selection`) | Use inside an admitted candidate where the library's rule fits. Uno's thresholds are explicitly uncalibrated, and `choose` always picks simplex. None spans adapters or classes. Complementary, not a replacement |
| **Learned or ML selector** | It would admit methods implicitly, which violates the maintainer's layer-3 constraint, and it needs data the system does not record. Not now. IR-06 keeps the option open as an advisory layer only |

## 13. Rule impacts (slot 11)

| ID | Rule and location | Proposed change | Depends on | If kept |
|---|---|---|---|---|
| IR-RC01 | §18.7 *Selection* ("SCIP … selected automatically … wherever the others are ineligible") and §18.6 ("a backend change requires named selected alternatives"), against the code's failure-driven alternatives | State that transitions are named plan rules with rationale; remove SCIP-after-local-failure from the default table; allow it under `certify`, a declared global policy or a bounded diagnostic | IR-02 | Amend the contract to declare failure-driven same-class escalation, including SCIP, and record its reason |
| IR-RC02 | `SolveReport.metrics` doc, and §18 report text ("metrics … never an input to a decision") | Keep the prohibition on decisions reading raw metrics; add the obligation to promote decision-relevant statistics into typed observation families | IR-04, IR-02, IR-05 | Layer 2 cannot be realized beyond termination codes |
| IR-RC03 | Register R-48 (Ipopt timing excluded until the C API exposes it) | Retire: the bridge is C++ and typed `TimingStats` involves no status parsing | IR-04, IR-06 | No Ipopt per-phase time; G7 claim must be corrected |
| IR-RC04 | Register R-34 (HSL excluded pending a licence route) | Reassess under "free to obtain; licence type is not an exclusion": if CoinHSL's free academic or personal licence is obtainable by the maintainer, admit HSL linear solvers and MC19 | Slot 8 | Ipopt stays with MUMPS, SPRAL and Pardiso, without equilibration scaling |
| IR-RC05 | Register R-47 trigger ("a measured case …") | Replace the measurement trigger with a mathematical-capability trigger (for example diffsol-nl gaining recoverable trials and sign constraints), consistent with "adoption never gated on benchmarks" | — | Exclusion stands with a benchmark-gated trigger |
| IR-RC06 | §17.4 and `RecycleRequest` defaults (`anderson: 0`, unweighted tears); §17 initialization single-settings rule (`validate_profile`: one backend setting for all blocks) | Defaults become plan outputs; per-block routes and settings allowed | IR-03, IR-08 | PS-08 history still achievable; per-block choice remains impossible |
| IR-RC07 | `run_lineage` and reuse-identity contracts (§5, §14, PS-11 application) | Add the advisory-history snapshot identity as a recorded result-affecting input; declare the plan determinism class | IR-06 | Layer 3 makes results depend on unrecorded state (a DP-09/PS-11 MUST gap) |
| IR-RC08 | Measurement as qualification-only evidence (`docs/dev/validation-assessment.md` *Case measurements*; process-wide `ResourceReport`) | Attempt wall time and cost become persisted product data, distinct from qualification evidence | IR-06 | Measurement stays an investigation campaign |
| IR-RC09 | DP-14 application: adapter-resident selection defaults (dynamics `Auto` list in `pse-backend-native`) | Move to the plan; adapters declare profile applicability only | IR-01 | — |

## 14. Coverage limits and probes

**Not examined:**

- Uno, PETSc flow and Clarabel adapters beyond their stop paths and capability strings.
- `prediction.rs` and `paths.rs` internals.
- The fitting and shooting selection paths beyond their `Requirements` sites.
- Python-side summaries.
- Whether each `StartOrigin` has a producer.
- Plan 36c effects on block execution, which is baseline.

None of these would change the G9 verdict. A material missing premise remains for AP-07 (how often first candidates fail in studies). IR-06 is the settling evidence.

**Probes the coordinator may arrange** (each with its question, command and expected observation):

1. **SCIP escalation in practice.**
   - *Question:* does a failing smooth NLP with finite boxes reach SCIP?
   - *Command:* in a `pse-runtime` native unit with `solver-ipopt,solver-pounce,scip`, solve an infeasible boxed factorable NLP under Auto; read `solve_strategy_events`.
   - *Expected:* events for Ipopt, then POUNCE (plus rungs), then a `native_globalization` event with backend `scip`.
2. **Iterate and statistics after a stop.**
   - *Question:* does a cooperative stop keep what a throw loses?
   - *Command:* B1 probe 2. A C++ HS071 probe with `intermediate_callback` throwing, against one returning `false`.
   - *Expected:* NULL statistics and no `finalize_solution` with the throw; `User_Requested_Stop`, statistics and the iterate with `false`.
3. **KINSOL-failure successor.**
   - *Question:* is Ipopt ever offered after a failed square root?
   - *Command:* `just unit-package pse-runtime native_recovery` on a divergent, unbounded square problem; read the events.
   - *Expected:* `Exhausted` with no Ipopt event, although `route_decisions.classes` contains `smooth_nlp` and Ipopt's eligibility is `ready`.
4. **Typed iteration fidelity.**
   - *Question:* are library-reported iterations typed?
   - *Command:* run `kinsol_krylov_variants_solve` and one POUNCE solve; compare `evidence.work.iterations` with the `KINGetNumNonlinSolvIters` and `statistics.iteration_count` metrics.
   - *Expected:* typed `None` beside positive metrics.
5. **HiGHS basis retention.**
   - *Question:* does a basis survive in-place updates without resubmission?
   - *Command:* B4 probe 2. Solve, update one bound, re-solve with and without the exported `WarmStart`; compare `simplex_iteration_count`.
   - *Expected:* near-cold iteration counts without resubmission.

**Decision this informs.** The principal review should treat IR-01 and IR-04 as the prerequisite contracts for the three-layer requirement. It should treat IR-08, IR-02's SCIP edge and IR-09 as narrow behavioral corrections that can land first. It should record IR-RC07 as the condition under which the maintainer's history layer stays consistent with PS-11.

**Relevant files** (all under ``):

- `crates/pse-runtime/src/math/solves.rs`
- `crates/pse-runtime/src/math/staged/driver.rs`
- `crates/pse-runtime/src/math/strategy.rs`
- `crates/pse-runtime/src/math/solves/blocks.rs`
- `crates/pse-backend-native/src/routing.rs`
- `crates/pse-backend-native/src/execution/scip.rs`
- `crates/pse-backend-native/src/execution/ipopt.rs`
- `crates/pse-backend-native/src/solve.rs`
- `crates/pse-backend-native/src/callback.rs`
- `crates/pse-backend-native/src/uno/callback.rs`
- `crates/pse-backend-native/src/pounce/profiles.rs`
- `crates/pse-backend-native/src/highs.rs`
- `crates/pse-backend-native/src/dynamics.rs`
- `crates/pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`
- `crates/pse-uno-sys/bridge/pse_uno.cpp`
- `crates/pse-math/src/facts.rs`
- `crates/pse-runtime/src/workflow/strategies/requests.rs`
- `crates/pse-runtime/src/workflow/results.rs`
- `docs/authoritative_design/sections/numerical-execution.md` (§18.6, §18.7)
- `docs/adr/register.md` (R-34, R-47, R-48)