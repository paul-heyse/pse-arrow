> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, A1 (code-mapper): selection, escalation and dispatch, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

# A1 evidence map: how a prepared problem gets its method today, and what happens between attempts

**Baseline:** HEAD `ff5a4aa2f` plus the Plan 36 working tree, read as it stands. Every claim is *Interface-checked* (I read source) or *Proposed*. I ran nothing. `BN`=`crates/pse-backend-native/src`, `RT`=`crates/pse-runtime/src`, `PM`=`crates/pse-math/src`.

## Summary

- **Leads.** Of (a)–(m), ten are confirmed as stated. Four are confirmed with corrections:
  - (c): observations also steer the POUNCE ladder and the native alternatives, not just continue/stop.
  - (f): the Ipopt → POUNCE → SCIP chain is right, but the KINSOL gap is only partly a gap.
  - (j): Presolve `Auto` does request one pass, and only `LinearBounds` and `Fbbt` are hard-disabled.
  - (k): a weighted HiGHS tear-selection route exists. Recycle just doesn't use it by default.
- **Selection is three fixed things.** Class by facts, then a constant adapter rank, then a fixed-order candidate list with a dynamic append after any failed native attempt. No runtime decision reads `SolveReport.metrics`.
- **The a priori layer ignores problem structure.** `ProblemFacts` has no nnz, block structure, nonlinearity profile, conditioning or stiffness. Structure reaches selection only through `Block` eligibility and the POUNCE-only separator.
- **The in-solve layer is thin.** Typed observations feed a continue/stop gate (`permits_numerical_continuation`) and the POUNCE ladder, which is keyed on the native termination code. Nothing else reorders or tunes. There is no history layer.
- **Escalation exists, but only within one class and only for default settings.** Ipopt → POUNCE (+ladder) → SCIP is the only multi-adapter chain. KINSOL, HiGHS convex-QP and Clarabel have no sibling. The chain is built only for default settings, so an explicit `BackendSettings` selection never escalates.
- **Dead vocabulary.** Of 25 `NumericalMechanism` values, 10 have no non-test dispatch and 3 more are only filters in `admits_mechanism`. Most "declared" mechanisms (Path, Multistart, Derived, Surrogate) have no production constructor outside tests in `crates/`.
- **Prior review.** F01, F04, F05 and C12 are resolved. F02, F03, F07, F08, F13 and others are partial or open (tables below). Prediction (M1, M3) is widely implemented. Mechanisms M9–M11 and M16 are implemented but only reachable by declaration, never by automatic choice.

## 1. Leads (a)–(m)

| Lead | Verdict | Evidence (fact) | Interpretation |
|---|---|---|---|
| (a) `ProblemFacts` has no nnz, block, nonlinearity profile, conditioning or stiffness | **Confirmed** | `PM/facts.rs:25-68` lists every field: `class_status, variables, rows, objective(s), equalities, domains, derivatives, prepared_derivatives, bounds (per-coordinate shape), guarded, coefficients, affine_rows, objective_degree, bound_assumptions, quadratic, native, requirements, convexity`. My grep of `routing.rs`, `strategy.rs` and `facts.rs` for nnz, sparsity, density, stiff and condition found nothing. | Closest proxies are `affine_rows`, `objective_degree` and `convexity`. These are class facts, not performance facts. Structural data (BTF/DM) travels separately, in the routing `Context.structure` (`routing.rs:126`). |
| (b) Class derivation and rank-min selection | **Confirmed** | `problem_classes` at `routing.rs:762-797`. `decide_assessed` (`:1014`) and `select_assessed` (`:1201`) take, per class, `min_by_key(rank)` over adapters whose `automatic_classes` contain the class. Ranks: KINSOL 0 (`execution/kinsol.rs:248`), HiGHS 1, Ipopt 2, POUNCE 3 (`pounce.rs:60`), Clarabel 4, SCIP 5. Explicit-only (`None`): Uno, PETSc, POUNCE-convex, Diffsol, IDAS. | The order differs by class. `automatic_classes` are: KINSOL = SquareRoot + DeclaredFixedPoint; HiGHS = Linear, MixedLinear, ConvexQuadratic; Clarabel = ContinuousCone; Ipopt and POUNCE = SmoothNlp; SCIP = MixedLinear, MixedIntegerQuadratic, MixedIntegerNonlinear, SmoothNlp. The only genuine rank contests are therefore SmoothNlp (Ipopt 2 < POUNCE 3 < SCIP 5) and MixedLinear (HiGHS < SCIP). **Correction:** a sensitivity request first filters to `capability().sensitivities` adapters (`routing.rs:1033, 1257`). Unrecorded as a reason. |
| (b+) A priori rules that already exist | **Added** | `square_root` (`routing.rs:722`) is equalities ∧ rows = variables ∧ no objective ∧ continuous. KINSOL has `general_bounds:false` and `parallel:false` (`execution/kinsol.rs:28,31`). `admit` (`routing.rs:799`) therefore refuses KINSOL on boxed bounds and on `threads != 1`, and the SmoothNlp class then picks Ipopt. Convexity, the PSD certificate and cone recognition choose Linear/ConvexQuadratic/ContinuousCone. A structural requirement (ℓ1) admits only POUNCE and is never automatic (`l1_never_automatic`, `l1_tests.rs:269`). | These are real a priori mathematical-fact rules (bounds, parallelism, convexity), but all are binary eligibility. None is a size- or structure-tuned preference. |
| (c) `next_automatic` fixed-order list | **Confirmed, with refinement** | `RT/math/strategy.rs:76-166`. The first call builds `[automatic_products…, MapsAnderson, ReducedSpace, Block, Direct]` (`solves.rs:687-783`). The walk takes the first un-attempted candidate that `start.permits_entry/recovery` allows. Observations gate only through `permits_numerical_continuation` (`strategy.rs:1922`), which allows Converged/Auxiliary/Limited/Stalled/NumericalFailure. `last.permission` only enters the decision hash (`:229`). | **Added.** After any native report not satisfying the original, the driver calls `automatic_operations(Some(report))` and appends the result (`staged/driver.rs:824-840`). That result is the POUNCE second-opinion profiles plus `automatic_backend_alternatives`. Appended items go after Direct and are deduplicated by binding identity. The default attempt cap is 2 per non-Direct candidate and 1 for Direct (`strategy.rs:112-128`). |
| (c1) Candidate preconditions | **Detail** | `MapsAnderson`: KINSOL ∧ `Root` intent ∧ `NoPriorStart` ∧ threads 1 ∧ default settings ∧ causal supplier (`solves.rs:512-521`). `Block`: `NoPriorStart` ∧ no sensitivity ∧ `ConvexityPolicy::Exact` ∧ presolve not Explicit ∧ Roots/Nlp adapter ∧ initialization schedule with more than 1 block (`blocks.rs:132-171`). `ReducedSpace`: reconstruction accuracy from a supplier that supports reconstruction, with all variables covered, or an NLP adapter (`solves.rs:468-509, 731-771`). | The prepared-rung slot only receives products from `with_automatic_products` (`solves.rs:338`). Its only callers are four tests in `math/surrogate.rs`, a `#[cfg(test)]` module that starts at line 753. |
| (c2) POUNCE second-opinion ladder | **Confirmed** | `BN/pounce/profiles.rs:36-105`. It triggers only on four `ApplicationReturnStatus` codes matched against `report.termination.code`. Terminal-failure, validation-failure or abandoned reports return an empty ladder. Rungs: `feral_scaling=mc64`, `mu_strategy=adaptive`, `start_point_perturbation=1e-2` (replacement start, needs `Auxiliary` recovery), `feral_increase_quality=no`. Library retries are pinned off at `BN/pounce.rs:101`. The ladder runs only when `last.backend == Pounce` (`solves.rs:787-830`). | This is the one genuinely observation-driven option tuner. It reads a numeric status code and `pounce_statistics.quality_escalations`, not metrics. |
| (c3) `automatic_backend_alternatives` | **Confirmed** | `solves.rs:866-935`. Empty unless `selection==Auto` and `backend==Default`. Also empty unless the retained decision is `Ready` and selected equals the current adapter. It takes the first class of the decision that the current adapter owns automatically, then the strictly lower-ranked adapters of that class. `Refused` entries are skipped. A `PendingEvidence` entry (or an absent one) stops the walk. | Because the class is "the first one the current adapter owns", it never crosses class boundaries (SquareRoot → SmoothNlp). |
| (d) BTF blocks use one backend | **Confirmed** | `blocks.rs:8, 222-226, 293, 309`: one `backend` from `original.backend()`, `Route::Native(backend)` for every block. Each block runs with `presolve=Off` and `intent=Initialize` (`:277-279`). A block failure aborts the step and the driver reports a component observation (`:420-431, 485-510`). | There is no per-block adapter, settings or class re-derivation. Blocks solved as `Initialize` then an original-model constant evaluation yields the report. |
| (e) Compiler separator feeds POUNCE only | **Confirmed** | `solves.rs:4629-4634`: `run.adapter.backend()==Pounce ∧ composition.policy==Auto ∧ automatic_separator(...)`, then `with_solve_separator`. `crates/pse-compiler/src/workspace.rs:530-` builds the separator from shared-instance columns plus cross-block dependency columns. | It is a conservative performance projection (docstring). Ipopt never receives it. POUNCE is reached by Auto routing only after an Ipopt failure, or when Ipopt is not linked/eligible. |
| (f) Escalation facts | **Confirmed, nuanced** | KINSOL SquareRoot: `ranked` for SquareRoot contains only KINSOL, so no alternatives (`solves.rs:900-910`). SmoothNlp: Ipopt → [POUNCE, SCIP]. POUNCE → [SCIP] + ladder. Ipopt has no retry ladder. Its only restart machinery is the same-structure sequence (`BN/ipopt/sequence.rs`). | KINSOL "gap" nuance: a square system that KINSOL a priori refuses (boxed bounds, threads>1) is routed to Ipopt via SmoothNlp. A system KINSOL *attempts and fails* has no cross-class successor. HiGHS convex QP, Linear and Clarabel also have no sibling (SCIP isn't automatic for those classes). MixedLinear: HiGHS → SCIP. |
| (g) Mechanism dispatch | **Confirmed (10 + 3)** | See §2 table. | |
| (h) Hessian `Auto` | **Confirmed, plus fitting** | `solves.rs:108-115`: `Exact` if `available>=Second`, else `LimitedMemory`. Production sites in `solves.rs` (3 sites) pass `least_squares:false`; `fitting/modeling.rs:124` passes `true`. **Correction/addition:** fitting resolves its own `Auto` at `fitting/modeling.rs:261-267` to `LimitedMemory` for gradient-only derivatives, else `GaussNewton`. | `Exact` vs `LimitedMemory` is chosen by derivative availability only, not by size or cost. `GaussNewton` is reachable only through fitting, or explicitly. |
| (i) KINSOL and dynamics defaults | **Confirmed** | `BN/settings/kinsol.rs:125-139`: `LineSearch`, `Klu`, `anderson:0`, `damping 1`, `setup_interval 10`, `Preconditioner::None`. Krylov variants and `Eta` are typed but nothing in `RT` selects `Linear::Sp*`. Dynamics `Auto`: `[Diffsol, Idas]`, first candidate not `Refused` (`BN/dynamics.rs:1367-1440`), where refusal comes from `method_refusals`, link state and contract validation. Diffsol defaults are `Bdf` + `FaerLu` (`:50-61`). | No stiffness or size estimate enters the choice. |
| (j) Presolve `Auto` | **Confirmed, corrected** | `BN/presolve.rs:201-207`: `Auto` requests `enabled`, `linear_eq_reduction` and `fbbt`. In `qualify` (`:263-289`), `LinearBounds` and `Fbbt` have `eligible=false` hard-coded with reason `UNTRACKED_BOUND_DUALS` (`:135`). | The effective Auto presolve is `AffineElimination` (when affine facts and tolerances qualify), `RankDiagnostics` and nothing else. `Auto` selects none under ℓ1 (`:102`). `LinearBounds` isn't even requested by `Auto`. |
| (k) Recycle defaults | **Confirmed, corrected** | `RT/workflow/strategies/requests.rs:124-127` sets `anderson:0`, `damping:1.0`; with `tears==None`, `graph.unweighted_heuristic()` (`:88-95`). | A weighted route exists. `RT/math/flows.rs:144-160` serves `TearMethod::{UnweightedHeuristic, Highs}` through `tears::heuristic` / `tears::solve` (HiGHS weighted feedback-edge MILP, `BN/tears.rs:277-331`). It is a separate tool step, not wired into `RecycleRequest::from_model`. |
| (l) Hidden second selection path | **Confirmed** | `solves.rs:2704-2730`: `complete_context` (policy Auto ∧ no sensitivity ∧ no native forms ∧ no requirements ∧ suppliers cover all columns) picks the lowest-ranked linked `Roots` adapter for `Auto`, or the explicit backend. It is used at `:3098-3105` when `decision.state==Refused`. | A refused routing decision does not stop preparation when a supplier covers all columns. The retained `route_decision` still says "refused", while execution uses `Route::Native(<producer backend>)`. |
| (m) Cone failure aborts, factorable re-routes | **Confirmed** | Factorable build failure of type `Unsupported`/`Contract` inserts `refusals[backend]` and `continue`s the routing loop (`solves.rs:2897-2905`). Cone lowering errors propagate through `?` with no refusal entry (`:2995-3035`). | Asymmetric: an unrecognizable cone aborts preparation rather than letting the router try the next class. |

## 2. `NumericalMechanism` (25 values; `pse-model/src/generated/enums.rs:17185-17270`)

"Production" means non-test code in `crates/` and `python/`.

| Value | Status | Pointer |
|---|---|---|
| `direct` | **Automatic** | `solves.rs:781`, `PreparedRung::Original` |
| `maps_anderson` | **Automatic** (KINSOL root, causal supplier) | `solves.rs:711` |
| `reduced_space` | **Automatic** (reconstruction supplier) | `solves.rs:764` |
| `block` | **Automatic** | `solves.rs:776`, `blocks.rs:228` |
| `native_globalization` | **Automatic** (escalation: backend alternative or POUNCE rung) | `solves.rs:813, 847` |
| `continuation` | **Declared-only** (`PreparedRung::Path`) | `solves.rs:370, 691`; no production constructor |
| `multistart` | **Declared-only** | `solves.rs:372, 692`; no production constructor |
| `surrogate` | **Declared-only** | `solves.rs:368, 690`, `surrogate.rs` (constructor in tests) |
| `homotopy` / `pseudo_transient` / `bounded_feasibility` / `least_deviation` | **Declared-only** (`DerivedRequest`) | `derived.rs:654-658`; `prepare_derived` has one caller, the Reduced binding (`solves.rs:1017`). `PseudoTransient` also via PETSc `Product::Flow` (`solves/petsc.rs:339`, explicit-only). |
| `block` / `pseudo_transient` via PETSc | **Explicit-only** | `solves/petsc.rs:339-340` |
| `multifidelity`, `setup_reuse`, `selected_evaluation` | **Filter only** (accepted by `admits_mechanism`, never produced) | `derived.rs:413, 419-421` |
| `kkt_predictor` | **Dead** (outside tests) | only `pse-model/src/strategy.rs:721` (inside the `optional_empty_slice…` test) |
| `activity_path`, `limited_corrector`, `root_response`, `secant_predictor`, `newton_krylov`, `consistent_initialization`, `nonlinear_preconditioner`, `schur`, `batch` | **Dead** | no non-test reference to the variant |

That is 5 automatic, 8 declared or explicit-only with real producers, 3 filter-only and 9 + 1 dead, for a total of 25.

Two caveats:
- **Prediction and path code exists without this label.** `RT/math/prediction.rs` and `BN/kkt/path` implement the behaviours (kkt, root, secant), but they do not record themselves as `kkt_predictor` / `secant_predictor` mechanisms. `Schur` and `Batch` are implemented elsewhere (POUNCE Schur separator; POUNCE-convex batch) without the label.
- **Search coverage:** `MechanismKind::` and `NumericalMechanism::` in `crates/**/*.rs` and `python/**` excluding `generated/`, tests and `*_tests.rs`. Python bindings in `python/pse/contracts` are generated.

## 3. Inputs consumed by resolver and router

**Router (`routing::Requirements`, `routing.rs:561-586, 117-143`):**
- `ProblemFacts`: all fields in (a).
- `intent` and `numerical_psd`.
- `least_squares`.
- `controls`: threads, hessian, `validate()`.
- `settings`: `BackendSettings`, which must honour structural requirements.
- `sensitivity` flag.
- `Context`:
  - snapshot (linkage and build identity);
  - `pending_classes`;
  - `refusals`;
  - `structure` (matching/DM witness; the `structural::mode` policy per adapter);
  - oracle contract;
  - guards;
  - budgets (tolerances, normalization, accuracy);
  - coefficients, PSD certificate, cone and factorable evidence;
  - `prepared` artifacts.

**Strategy walk (`next_automatic`, `strategy.rs:76`):**
- `CompositionRequest`: policy, branch, recovery, limits.
- `StartRules` and `inherited`.
- Candidate descriptors: kind, start, replacement, prepared.
- The `attempted` set.
- `last`: native `Observation`, original-model conclusion, permission (hash only), `awaiting_assessment`.
- Cumulative `WorkObservation`.

**Candidate construction (`solves.rs`):**
- `profile.selection`, `profile.backend` and the retained `route_decision` (selection, state, classes, eligibility).
- The supplier inventory.
- `controls.start` and `controls.threads`.
- Presolve policy and convexity policy.
- Initialization schedule block count.

**Work allowances:** `WorkLimits {attempts, evaluations, iterations, factorizations, proof_steps}` (`pse-model/src/strategy.rs:122`) with a ledger (`strategy.rs:537-670`). Default attempts is a sum over candidates.

**Products:** `ProductState` (`strategy.rs:243-345`) holds actual producer evidence for consumers and trace.

## 4. Where the decision is recorded; do runtime decisions read metrics?

- **`runtime.route_decisions`** (`pse-schema/src/catalog/native_math.rs:322-380`, row built at `routing.rs:232-298`). It holds the intent, selection, requested backend, classes, assessment state, snapshot, evidence and artifact demands, per-adapter eligibility with reason codes and structural mode, the selected route and refusal. It records admissibility. It does not record rank, class-ordering reasons, the sensitivity preference, `complete_context` rescue (lead l) or what set the alternative list.
- **`runtime.solve_strategy_events`** (`pse-schema/src/catalog/native_strategy.rs:157-`). Per-event: mechanism, kind, phase, backend, start origin, `observation`, `transition`, `permission`, `original_conclusion`, attempts/evaluations/iterations/factorizations/proof steps, failures, and `decision_identity`. The `decision_identity` is a hash over request, candidates, chosen action and work (`strategy.rs:168-241`), so it is reproducible but opaque. Nothing says "ranked below X because Y".
- **`SolveReport.metrics`** (`BN/solve.rs:1978`). In non-test code, `RT` reads it only to write rows: `workflow/results.rs:803` pushes every metric to the results table. Test-only reads: `math/initialization.rs:776` (inside a `#[cfg(test)]` block), `study_execution.rs:2247`, `fitting/oracle.rs:2281` (in tests, to my reading). One production in-adapter read exists: `BN/highs.rs:985` (basis validity) feeds HiGHS's own warm basis, not any selection. All runtime decisions read typed `evidence` (e.g. `report.evidence.abandoned`, `evidence.callback.terminal_failure`, `evidence.work`) or the termination category and code (`strategy.rs:1748-1790`, `profiles.rs:43-61`).

## 5. Prior review (2026-10-03): findings F01–F17

Status key: R = resolved, P = partial, O = open, N = not re-verified.

| ID | Status | Pointer / note |
|---|---|---|
| F01 KINSOL FP/Picard recoverable return | **R** | `BN/kinsol.rs:2365-2396` test `map_callbacks_latch_failed_trials_but_newton_can_recover` (read by name; implementation line not re-traced) |
| F02 typed outcome for escalation | **P** | `KIN_STEP_LT_STPTOL` now `Limit` (`kinsol.rs:1646`). `report.evidence.abandoned` consumed (`strategy.rs:1748-1750`). KINSOL per-iteration observations and the globalization-vs-evaluation distinction not verified |
| F03 inert settings accepted | **P** | KINSOL refuses Anderson/damping under non-map strategies (`kinsol.rs:256`). Ipopt `warm_start_same_structure` is reserved (`ipopt/settings.rs:100, 115`) and set only by the retained path |
| F04 keep-flag definitions | **R** | `RT/math/staged.rs:216`: `permits_use()` of the candidate decision |
| F05 §17.6 Seed(k) divergence | **R** | `numerical-execution.md:852` now says it "does not … use a staged `Seed(k)` variant"; code has `Start::{Specification, Accepted}` (`workflow/staged.rs:74`) |
| F06 no owner for between-attempts policy | **P** | `strategy.rs` pure `next_automatic`/`Transition` and `run` exist. Homotopy step control and fit chain policies were not re-audited |
| F07 hand-built `Requirements` | **O** | ~11 production sites: `solves.rs:2753, 3363, 3554`; `derived.rs:1603`; `initialization.rs:75, 512`; `paths.rs:1105, 1800`; `fitting/modeling.rs:119`; `shooting.rs:682`; `modeling/cases.rs:172` (plus `diagnostics.rs:259` from grep) |
| F08 staged primitive coupling | **P** | `strategy::run*` (`:1236-1291`) and `blocks_step` (`blocks.rs:314`). Recycle and conditional loops not rechecked |
| F09 start model | **P** | `StartOrigin` now: specification, explicit, accepted, predicted, auxiliary, partial, modified_specification, surrogate (`native_strategy.rs:104-115`). Whether each origin has a producer was not traced |
| F10 derived systems | **P** | `DerivedRequest` family (`derived.rs:433-`) exists; reachable only via declared composition. No automatic use except `Reduced` |
| F11 retained slot | **O** | `Retained::clear` keeps the advanced-step factor by design (`BN/execution.rs:819-824`). Compatibility key completeness not checked |
| F12 library reuse | **P** | KINSOL `NoInitSetup` follows `setup_usable`, residual monitoring wired (`kinsol.rs:1286-1340`). Ipopt same-structure reuse `ipopt/sequence.rs:592`. POUNCE symbolic survives (`pounce.rs:203`). `msbsetsub` not found |
| F13 budget, accuracy tier, observation | **P** | Strategy-wide work ledger exists (`strategy.rs:537-670`). Finite accuracy refinement exists (`strategy/accuracy_refinement.rs`). No intermediate-accuracy tier for inner solves found |
| F14 method choice within an adapter | **P** | POUNCE ladder resolves it for POUNCE. Ipopt settings are typed (mu strategy, bound_push) but chosen by nothing. `start_with_resto` / LS init not found |
| F15 structure shaping main solve | **P** | Block, ReducedSpace and Schur separator exist. Default tears unweighted (k) |
| F16 NK cost | **N** | `consumes_directional_only` exists (`settings/kinsol.rs:143-150`); cost behaviour not read |
| F17 inner warm starts | **O** | `pse-math/src/implicit.rs:69` still "Deterministic start; no previous trial" |

**Current-state observations C1–C14:** C1 P (shared `run`; blocks/recycle/fitting have own loops). C2 O (F07; step-control not rechecked). C3 **P**: still class+rank and defaults; escalation added. C4 P (blocks and reconstruction now reach selection; conditioning, size, bounds-dual facts still don't). C5 **P**: typed observations drive continue/stop and the POUNCE ladder; callback counts still unconsumed. C6 O. C7 **P**: prediction layer wide (`prediction.rs:407-422, 786-894`, many consumers). C8 P (F12). C9 P (blocks and reduced exist; tears manual). C10 P (declared-only). C11 P (Ipopt options mostly still raw-reachable). C12 **R** (F01). C13 N. C14 **R** (F04, F05).

## 6. Mechanisms M1–M18, S (current status)

Taken from the forked verification pass, which I spot-checked on the F01/F03/F05 points above.

| M | Status | Pointers |
|---|---|---|
| M1 | Implemented | `BN/kkt/advance.rs`; `RT/math/prediction.rs:407-411, 671`; consumers `workflow/horizon/driver.rs:2037`, `modeling/results.rs:258, 445, 727, 811` |
| M1b | Implemented, library-bounded | `BN/kkt/path.rs:1-17, 259` (`step_along_path`); `path_direction`/`refine_step_onto_bounds` use unconfirmed |
| M2 | Partial | horizon only; no generic sequence primitive |
| M3 | Implemented (root + secant) | `prediction.rs:420-422, 786-894`; `results.rs:758, 783, 2176`; automatic calls from studies/homotopy not confirmed |
| M4 | Partial, explicit-only | `settings/kinsol.rs:26-62, 125-150`; no automatic choice |
| M5 | Partial | `kinsol.rs:1286-1340`; `ipopt/sequence.rs:592` |
| M6 | Implemented, manual | `kinsol.rs:1109`; recycle `anderson:0` |
| M7 | Partial | trust-region only in PETSc/Uno (`settings/petsc.rs:13-35`), explicit; no LM / bounded-LS in automatic path |
| M8 | Partial | POUNCE ladder (`profiles.rs`); Ipopt raw/typed options |
| M9 | Declared-only | `paths.rs` (2946 lines; `prepare_path` `:836`, `run_path` `:2273`); `BN/kkt/path/arclength.rs:596` fold event |
| M10 | Declared-only | `derived.rs:654-658` |
| M11 | Declared/explicit-only | `derived.rs:657`; `settings/petsc.rs:52`; `solves/petsc.rs:339` |
| M12 | Partial | `dynamics/idas.rs:1313-1317` typed IC mode; no tuning or escalation found |
| M13 | Implemented (automatic, single backend) | `blocks.rs`, `solves.rs:775` |
| M14 | Implemented (automatic when supplier) | `solves.rs:731-771` |
| M15 | Absent as a mechanism | `nonlinear_preconditioner` dead; only recycle FP and Block |
| M16 | Declared-only | `math/surrogate.rs`; `with_automatic_products` has no non-test caller |
| M17 | Partial | POUNCE Schur via compiler separator only; batch for POUNCE-convex (explicit-only); multistart declared-only |
| M18 | Unchanged | `implicit.rs:69`; constant-derivative flags not checked |
| S | Partial | class+rank routing; fixed-order candidates; observation-gated append; no history layer |

## 7. Gaps against the maintainer requirements

Interpretation, not fact:
- **Layer 1 (a priori):** binary eligibility is thorough (bounds, convexity, cone, parallel, structural). Absent: size, sparsity, structure, conditioning, stiffness, nonlinearity as preference inputs (a). The Hessian, Krylov, presolve and dynamics choices are defaults or availability tests (h, i, j).
- **Layer 2 (in-solve switching):** exists in one adapter (POUNCE ladder). Other observations (callback counts, per-iteration KINSOL/Ipopt events, regime crossings) are recorded and not consumed (§4).
- **Layer 3 (advisory history):** nothing found. The candidate list is purely a function of the problem and the last report. `decision_identity` hashes inputs, so it could key a history table, but nothing reads or writes one. This was a search over `RT/math`, `BN/routing.rs`, `BN/execution`, not an exhaustive repository search.
- **Escalation is class-bounded and default-only (f, c3).**
- **Hidden selection paths:** `complete_context` (l), and the cone/factorable asymmetry (m), mean the recorded decision may not match the executed route.

## 8. Probes I did not run

Each of these settles one material doubt if you choose to run it.
1. **KINSOL cross-class fallback (f).**
   - Question: does a failed KINSOL square root ever reach Ipopt?
   - Command: `just unit-package pse-runtime native_recovery`, then inspect `native_recovery_tests.rs` for a SquareRoot→SmoothNlp case; otherwise, run a boxed-bounds square problem and read `route_decisions.eligibility`.
   - Expected: KINSOL `Ineligible::Bounds`, Ipopt selected a priori; a divergent unbounded square problem yields `Exhausted` with no Ipopt event in `solve_strategy_events`.
2. **`complete_context` divergence (l).**
   - Question: does `route_decisions.state` read `refused` while `solve_strategy_events.backend` is KINSOL?
   - Command: the existing tests in `derived_tests.rs` / `preparation_tests.rs` exercising `selected_suppliers_cover`.
   - Expected: a refused row paired with a native KINSOL event.
3. **Metrics are never consulted.**
   - Command: `ast-grep -p '$X.metrics.get($$$)' crates/pse-runtime/src` and exclude `cfg(test)` modules.
   - Expected: no production hit outside `results.rs` projection.