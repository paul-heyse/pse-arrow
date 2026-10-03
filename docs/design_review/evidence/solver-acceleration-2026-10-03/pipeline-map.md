# Solver pipeline map: execution spine and seams

Baseline: HEAD `f0b90258` plus the uncommitted Plan 25k working tree, read as-is on
2026-10-03. Changes in the working tree that touch this map: `routing.rs` `decide_assessed`
(keeps one unambiguous structural witness on refused or pending decisions), `solve.rs`
`SolveReport::record_validation_failure` (now also withdraws quality, observation,
qualification and least-infeasible), `quality.rs::attach_nlp`, and new tests in
`workflow/numerics.rs`. `git status` showed 55 entries when this map was finished,
including untracked `docs/adr/0153-certify-nonlinear-regime-selection.md`. The tree may
change while the concurrent session runs. Nothing was built or run.

Paths below are relative to `crates/`. Line numbers are those of the
working tree.

---

## 0. Compact flow (modeling algebraic solve)

```
ModelingPackage::prepare_solve / prepare_analysis[_attempt]      (workflow/modeling/cases.rs:668, engines.rs:305/316)
  └─ resolve_case  (cases.rs:804)
       ├─ prepare (specialize, per structure)          ── Overlay.facts (stage) changes structure here
       ├─ resolve_starts (cases.rs:366)                ── StartSource per input; CaseOverrides{seed,parameters,fixes}
       ├─ inner_registrations (implicit providers)     ── nested KINSOL InnerSolver registered here
       ├─ variable_states + fixes → bound_case (rebind per values, A6)
       └─ parametric program (sensitivity request)
  └─ finish_case (cases.rs:722)
       └─ MathService::prepare_solve (math/solves.rs:904)
            ├─ numerics::resolve → ResolvedNumericalPolicy
            └─ prepare_resolved (solves.rs:947)
                 ├─ discover_class (871)
                 ├─ Normalization / Tolerances / ResolvedAccuracy::resolve (997-999)
                 ├─ LOOP: routing::Requirements::bound_decision (1063-1131)
                 │     Pending → build demanded evidence/artifacts (factorable, cone, derivatives, assembly)
                 │     factorable Unsupported/Contract → refusals[backend] → `continue` (1199-1210)
                 │     Refused or same demands twice → RouteRefused
                 ├─ BackendSettings::for_requirements (l1) + admit_profile (1389-1396)
                 └─ Compatibility stamp (810) → PreparedSolve{route, accuracy, compatibility, explicit_start=None}
       ├─ with_providers (re-registration if the route demands a higher order)
       └─ with_sensitivity (383)
ModelingSolvePreparation  ── optional: with_start / with_primal_start / with_stored_start (workflow/run.rs:584/597/619)

EXECUTION (every modeling step):
workflow::staged::Staged::{run (209) | step (285) | batch (356)}
  └─ Staged::execute (225): Assessment prepared → NativeSession::step
       math/staged.rs NativeSession::step (303) → run (218): CPU permits, session thread, adapter scope (serve 63)
         └─ MathService::execute (solves.rs:1851)
              ├─ admit_step (1962): ReusePolicy::Fresh → retained.clear; StartPolicy → chosen seed;
              │                     seed.validate(layout,backend) else Rejected + clear; StartReceipt
              ├─ run_step (2270): Constant | coefficient_step | recognized_step | factorable_step | callback_step
              │     └─ native runner (pse-backend-native/src/execution/runner.rs):
              │          nlp (93) | roots (264) | coefficients (368) | recognized (646) | cone (703) | factorable (execution/factorable.rs:910)
              │            └─ adapter.execute(retained, Input)  ── BackendExecution (execution.rs:250)
              │                 └─ Retained::session (execution.rs:676) reuse/rebuild decision
              │            └─ recover → record_kkt → qualify (quality.rs:396/509) [+ kkt::derive / Advance keep]
              └─ conclude (2024): failure owner, SeedOrigin, StartReceipt.record
       assess closure (Assessment::assess, modeling/assessment.rs:98) → AssessedPoint
       !accepted → retained.clear()  (math/staged.rs:324)
  └─ ModelingResult::from_assessment (modeling/results.rs:314)
       └─ numerics::complete(Outcome::candidate_use → native_use, checks) → CandidateDecision / accepted
  └─ Staged::record: Record{decision, values, warm}
publication: RunResult::joined(...).finished(...) (workflow/run.rs) → candidate_assessments rows (numerics.rs:298)
```

---

## 1. Stage-by-stage owners

| Stage | Owner (file:symbol) | Consumes | Decides | Produces | Consumers |
|---|---|---|---|---|---|
| Public entry, modeling | `pse-runtime/src/workflow/modeling/cases.rs:668 ModelingPackage::prepare_solve`; `engines.rs:305 prepare_analysis`; `engines.rs:316 prepare_analysis_attempt` | `ModelingAnalysis` (root, bindings, case, `SolverProfile`, `NumericalInputs`), `CaseOverrides` | nothing numerical; it delegates | `ModelingSolvePreparation` (`cases.rs:79`) | `Staged`, studies, horizon, `ModelingSolvePreparation::start` |
| Case resolution | `cases.rs:804 resolve_case` | specialization, overrides | start values (`resolve_starts` `cases.rs:366`), variable fixedness (`overrides.fixes` mark fixed at `cases.rs:875`), the parametric program for sensitivity (`883-956`), `DomainAnalysis` per intent (`958+`) | `ModelingCaseResolution` (`cases.rs:93`) | `finish_case` |
| Solve preparation | `math/solves.rs:904 MathService::prepare_solve`, `:947 prepare_resolved` | `Preparation` (compiled `CasePlan`, facts, presolve facts, structure), `CaseValues`, providers, `SolverProfile`, resolved numerics | route (via routing), representation artifacts, accuracy, compatibility stamp, backend settings for requirements | `PreparedSolve` (`solves.rs:176`) | `MathService::execute`, `MathService::solve`, lineage identities (`preparation_identity` `:222`, `seed_preparation_identity` `:264`, `request_identity` `:291`) |
| Routing | `pse-backend-native/src/routing.rs:790 admit`, `:884 assess_static`, `:966 decision`, `:970 bound_decision`, `:996 decide_assessed`, `:1174 select`, `:1177 select_assessed`; per-adapter `execution.rs:287 BackendExecution::assess` | `ProblemFacts`, intent, controls, settings, `sensitivity` flag, `Context` (snapshot, structure witness, evidence, refusals) | eligibility per adapter, preferred backend (explicit, or automatic by class then `automatic()` rank), pending evidence or artifacts, refusal | `routing::Decision` (`routing.rs:177`) | `prepare_resolved` loop; diagnostics; fitting (`fitting/modeling.rs:140`); block init (`math/initialization.rs:42`); factorable re-solve (`execution/factorable.rs:~1440`) |
| Staged sequence (workflow half) | `pse-runtime/src/workflow/staged.rs:110 Staged` | `ModelingSolvePreparation`, `Obligations`, `Start`, `Overlay`, deadline | which value seed a step gets (`seed` `:154`), which native predecessor seed is offered (`predecessor` `:166`, `predecessor_at` `:172`), the per-step time cap (`:316-322`) | `Record`s and `StepRecord`s | initialization, studies, horizon, authored sequences, objectives |
| Native session | `pse-runtime/src/math/staged.rs:121 NativeSession`, `:303 step`, `:338 batch`, `:63 serve` | `PreparedSolve`, `Predecessor`, assess closure | adapter scope re-entry (`Scope::admits` `:43`); dropping `Retained` when `assess` returns false (`:324-326`) | `(Outcome, T)` | `Staged`, `MathService::solve`, `MathService::initialize` |
| Step executor | `math/solves.rs:1851 MathService::execute`, `:1962 admit_step`, `:2270 run_step`, `:2024 conclude`, `:1883 execute_batch` | `PreparedSolve`, `Retained` | reuse clear, start choice, representation runner, l1 provenance (`:2360-2371`) | `Outcome::{Native, Constant, Rejected}` (`:623`) | session `assess` closures |
| Representation runners | `pse-backend-native/src/execution/runner.rs` `nlp:93`, `roots:264`, `coefficients:368`, `coefficients_batch:387`, `recognized:646`, `cone:703`; `execution/factorable.rs:910 factorable` | `Step` (`runner.rs:24`: adapter, settings, snapshot, structure, controls, accuracy, execution, tolerances, normalization, compatibility, warm) | structural check, presolve or relaxed-row resolution (`:141-154`), feasibility oracle wrapping (`:155-162`), transport, recovery, qualification, sensitivity derivation | `SolveReport` | `run_step` |
| Adapter | `execution.rs:250 BackendExecution`; adapters under `execution/{ipopt,pounce,kinsol,highs,clarabel,scip,pounce_convex,dynamics}.rs` | `Input` (`execution.rs:152`) | native options from `ResolvedAccuracy` (`solve.rs:94 nlp_options`), reuse of the retained session (`Retained::session` `execution.rs:676`) | `SolveReport` | runners |
| Quality / KKT | `pse-backend-native/src/quality.rs:169 attach_nlp`, `:396 record_kkt`, `:509 qualify`, `:487 least_infeasible`; `runner.rs:561 reobserve` | report, accuracy, tolerances | `Qualification`, quality from a fresh original observation | report fields | `native_use` |
| Completion | `pse-runtime/src/workflow/numerics.rs:141 native_use`, `:376 complete`, `:30 CandidateDecision` (`permits_use :45`, `permits_seed :51`) | `SolveReport`, `NumericalPolicy` (incumbent and closure policy), model checks | `CandidateUse` (Usable, QualifiedUnclosed, SeedOnly, DiagnosticOnly, Unusable), refusals and qualifiers | `Completed` | `ModelingResult.accepted` (`modeling/results.rs:322-340`), `Staged::seed`, block `commit_block` (`math/initialization.rs:1194`), study facts, publication rows (`numerics.rs:298 assessment_row`) |
| Original-model assessment | `workflow/modeling/assessment.rs:52 assessment`, `:98 assess`; `AssessedPoint::accepted :28` | outcome and candidate | model checks, closure, applicability | `AssessedPoint` | `ModelingResult::from_assessment`; `NativeSession::step` keep or clear |

---

## 2. Seams

### (a) Start point, seed and provenance

Two independent mechanisms are documented in §17.6 and implemented as such:

1. **Value start (case values).** `resolve_starts` (`cases.rs:366-548`) builds `CaseValues`
   with typed `StartSource` (`cases.rs:36`). The precedence by insertion order is: model
   default → `overrides.seed` (Predecessor; free variables only, `:401-413`) → case values
   → `overrides.parameters` (Continuation; must be declared parameters, `:431-446`) → case
   members (Binding) → `overrides.fixes` (Fixed) → start annotations for coordinates still
   unset (`:465-547`). These values become the `initial` point of every runner
   (`solves.rs:2493`, `:2647`). The receipt does **not** call this a warm start.
   - `Staged::seed(Start)` (`staged.rs:154`) supplies `overrides.seed` only when the earlier
     record's `CandidateDecision::permits_use()`.
   - `Start` has only `Specification` and `Accepted(usize)` (`staged.rs:41-47`). Docs §17.6
     also describe `Seed(k)` for study points, which does **not** exist in code (doc/code
     divergence).
2. **Native warm start (`WarmStart`, `solve.rs:846`).** It is chosen in `admit_step`
   (`solves.rs:1974-1988`) by `Controls.start: StartPolicy` (`NoPriorStart` | `Explicit` |
   `PreviousAccepted`; registry `NativeStartPolicy`). It is validated against
   `Compatibility.layout` and backend (`WarmStart::validate` `solve.rs:1145`). A mismatch
   gives `Outcome::Rejected` and `retained.clear()` (`solves.rs:1989-2001`).
   - Explicit sources: `PreparedSolve::with_start` / `with_primal_start` (`solves.rs:353/312`),
     wrapped by `ModelingSolvePreparation::with_start` / `with_primal_start` /
     `with_stored_start` (`workflow/run.rs:584/597/619`; the stored seed sets
     `StartSource::Stored` for every column).
   - `PreviousAccepted` sources: `Predecessor{attempt, seed}` (`solves.rs:667`), offered by
     `Staged::predecessor()` (authored sequences, `run.rs:808`), `Staged::predecessor_at`
     (horizon, `driver.rs:1055`) or block init (`initialization.rs:1116-1118`, using
     `PreparedSolve::primal_seed` `solves.rs:563`, the step's own bound values).
   - `Staged::step` always passes `previous: None` (`staged.rs:333-340`). Stage, homotopy
     and objective steps therefore never submit a native predecessor seed. They seed
     values only.
   - **Study continuation** (`study_execution.rs:347-358`) calls `with_start(previous)` with
     the predecessor run's full `native.warm_start` (`primal_seed` `:494`, which despite its
     name returns the whole payload). This is an *Explicit native* seed, which contradicts
     §17.6's "study steps seed values only".
   - Provenance: `StartReceipt` (`solve.rs:1253`) is built in `admit_step` and completed in
     `conclude` (`solves.rs:2043-2045`) with `SeedTransformation::path` (normalization,
     presolve, working set, interior restart; `solve.rs:1209-1249`). The output seed gets
     `SeedOrigin{attempt}` (`solves.rs:2038-2040`). `WarmRestart` (`solve.rs:682`) controls
     interior-point re-centering. `request_identity` hashes the explicit seed content
     (`solves.rs:304-308`).
   - Seeds of nested paths: implicit inner solves use `Options.start`, documented as
     "Deterministic start; no previous trial is used implicitly" (`pse-math/src/implicit.rs:58`).
     Fitting forbids anything but `NoPriorStart` and `Fresh` (`fitting/modeling.rs:218-225`).
     Shooting computes its start by forward integration (`shooting.rs:814 initial_point`),
     or takes a caller vector.

### (b) Retained native state and reuse decisions

- Container: `execution::Retained` (`execution.rs:613`). It holds one adapter session
  `(Backend, Box<dyn Any>)` plus an optional `kkt::Advance` factor with its budget charge.
  It lives on the session thread (`math/staged.rs:77`) and is dropped on scope change
  (`serve` `:87-91`).
- Reuse rule: `Retained::session` (`execution.rs:676-706`). It reuses if the backend
  matches, the downcast succeeds and the adapter's `reuse` closure accepts (adapters check
  `Compatibility::same_session` = layout, profile and backend, `solve.rs:659`). Otherwise
  it tears down and rebuilds, or fails under `RequireReuse`. The `profile` hash excludes
  time, iterations, history, reuse and start (`hash_session` `solves.rs:786-809`).
- Callers that drop state: `admit_step` on `Fresh` (`solves.rs:1971`) and on seed
  refusal; `NativeSession::step` when `assess` returns false (`math/staged.rs:324`); the
  panic path (`:261-262`).
- **Seam:** in `Staged::execute`, the `accepted` flag returned to the session is
  `AssessedPoint::accepted()` (`staged.rs:268`, `assessment.rs:28`), which means a
  candidate exists, has the right extent and passes the model checks. It is **not** the
  `CandidateDecision`. The result's `accepted` is `completion.permits_use()`
  (`results.rs:322-340`). Retained native state can therefore survive a step whose native
  outcome was refused (for example IterationLimit → SeedOnly) when the model checks pass.
  The `NativeSession::step` doc comment states the opposite intent ("usable, accepted
  candidate"). Block init uses `candidate_use(...).permits_use()` (`initialization.rs:1129`),
  so the two paths differ. This is an interpretation from reading; it was not run.
- KINSOL reuse keeps allocations and the KLU symbolic analysis, but forces a numeric setup
  refresh on every reused solve (`kinsol.rs:1010-1011`, `KINSetNoInitSetup(...,0)`
  `:1084`). No numeric Jacobian or factor is reused across steps.
- Advanced-step factor: kept by `runner::nlp` when the sensitivity request has `retain`
  (`runner.rs:223-228`, `kkt::derive`), charged in `callback_step` (`solves.rs:2717-2729`),
  and set only by `PreparedSolve::retaining_factor` (`solves.rs:482`). Its only caller is
  the horizon driver (`driver.rs:1072-1076`). It is consumed by `kkt::predict` through
  `Staged::native` (`driver.rs:1248-1257`). `Fallback` reasons are listed at
  `kkt/advance.rs:77`.
- Nested implicit solves keep a per-thread KINSOL session cache keyed by problem identity
  (`implicit.rs:336-352`), bounded by `budget_sessions` (set at session open,
  `math/staged.rs:186-187`).
- Fitting, shooting, conditional units and profile chains each construct
  `Retained::default()` per call (`fitting/oracle.rs:705`, `fitting/profile.rs:290`,
  `shooting.rs:1216`, `initialization.rs:628`). They have no cross-call reuse.

### (c) Modification of the solved problem, and provenance to the original equations

| Modification | Where | Provenance kept |
|---|---|---|
| Stage selection (structure-changing override equations) | `Overlay.facts` (`staged.rs:52`) → `compose` → specialization | `ModelingInitializationStep::Stage`; `stage_sources` attribution (`engines.rs:563-614`); `Obligations::Intermediate` drops final expectations (`assessment.rs:58-62`) |
| Homotopy / continuation parameter values | `Overlay.parameters` → `CaseOverrides.parameters` → `StartSource::Continuation` (`cases.rs:431-446`); values from `continuation_values` (`engines.rs:741`) | `ModelingInitializationStep::Homotopy(fraction)` |
| Temporary fixes (discrete assignment) | `Overlay.fixes` → `CaseOverrides.fixes` → `states.fixed=true` (`cases.rs:875-877`), `StartSource::Fixed` | `report.discrete_assignment` (`engines.rs:255`) |
| Temporary variable states and values | `Overlay.variables`, `Overlay.values`, `Overlay.members` (`staged.rs:56-60`) | composed only into the step's clone (`staged.rs:67-91`) |
| "Values only" check for homotopy | Not enforced by a type. `Initializer::homotopy` builds an `Overlay` with only `parameters` (`engines.rs:684-687`); `resolve_starts` limits parameter overrides to declared parameters; `prepare_resolved` refuses values that differ from compiler presolve or coefficient assumptions and asks for a rebind (`solves.rs:987-993`, `:1007-1015`) | — |
| Library presolve | `runner::nlp` `Pipeline::new` (`runner.rs:163`); `presolve::Policy` default `Auto` (`presolve.rs:21-34`); forced off under l1 (`runner.rs:141-154`) | `report.preprocessing`, `SeedTransformation::Presolve` |
| Normalization (coordinate transport) | `Normalization::from_policy` (`solves.rs:997`); `transport::Roots` and `transport::coefficients` in runners | `SeedTransformation::Normalization(key)`; `recover` back to the original |
| Feasibility objective (Root, Initialize, FeasiblePoint intents on NLP) | `FeasibilityOracle` wrapper (`runner.rs:155-162`) | intent recorded |
| l1 exact penalty realization | `BackendSettings::for_requirements` (`execution.rs:558`), applied at `solves.rs:1389-1395` | `provenance["method.selection"]`, `["method.penalty_scope"]` (`solves.rs:2360-2371`) |
| Factorable export (relaxation, semi lowering, implicit residual export) | `plan.factorable_program` (`solves.rs:1171-1190`); `execution/factorable.rs` | `ExportTransformation` (`factorable.rs:380`); every candidate is re-evaluated against the original (`OriginalCase` `solves.rs:2963`) |
| Fixed-assignment continuous re-solve | `factorable.rs:1329 fixed_assignment`: `Pinned::discrete`, `Unconstrained`, then nested `Requirements::decision(Auto)` (`~1405-1458`) | own layout and profile hashes (`FactorableFixedAssignment*`) |
| Cone lowering / recognized cone | `ConicProblem::from_coefficients`, `conic::lower` (`solves.rs:1235`, `:1327`) | `raise` and certificate verification (`runner.rs:515-526`, `:690-693`) |
| Pinned profile parameter (fitting) | `transform::Pinned::new` (`fitting/profile.rs:268-269`) | the `ProfilePoint` record |
| Structural witness retained around the oracle | `structural::retain_nlp` / `retain_roots` (`runner.rs:107`, `:279`) | `Assessment` on the Decision |

Final acceptance is always re-observed in original coordinates: `quality::attach_nlp`,
`reobserve`, `transport::recover` followed by `qualify`, and model checks in `Assessment`.

### (d) Strategy choices

- **Backend:** `SolverSelection::{Auto, Explicit}` (`solve.rs:42`). Auto picks among
  problem classes, most specific first, the adapter with the lowest `automatic()` rank
  among those whose `automatic_classes` contain the class (`routing.rs:1226-1243`;
  pending-aware variant `:1004-1033`). The l1 POUNCE method follows authored requirements
  (`execution.rs:539-575`). Nested decisions: block init, once per block before execution
  (`initialization.rs:42-124`, intent Initialize); fitting (`fitting/modeling.rs:140-185`,
  `least_squares: true`); fixed-assignment re-solve (Auto, NLP default settings,
  `factorable.rs:~1440`). Recycles hard-code KINSOL (`initialization.rs:703-801`,
  `kinsol::Session::new` direct). Nested implicit solves hard-code KINSOL
  (`implicit.rs:214`).
- **Method:** only from typed `BackendSettings` (`execution.rs:481`). KINSOL `Method`
  defaults are LineSearch, KLU, anderson=0, setup_interval=10, eta default
  (`settings/kinsol.rs:100-140`). Nested implicit KINSOL forces `setup_interval: 1`
  (`implicit.rs:317-321`). There is no automatic Krylov, Anderson or forcing selection.
- **Initialization strategy:** authored or explicit only. `ModelingInitialization`
  (stages, homotopy flag, step policy; `engines.rs:44`); `InitializationProfile` stages
  for blocks; `PreparedRecycle::start` "Execute only the explicitly requested map
  strategy" (`strategies/conditional.rs:188`).
- **Continuation:** fixed-shape adaptive step: initial, growth ×`growth` after acceptance,
  ×0.5 on a retryable failure, and a stop below `minimum_step` (`engines.rs:655-717`).
  The continuation path itself comes from authored `continue … on p from a to b`
  endpoints (`base.compiled().model.continuation`, `engines.rs:413`).
- **Presolve:** `Policy::Auto` resolves to qualified passes, or `Off` under row-relaxing
  methods (`runner.rs:141-154`).

### (e) Failure handling, by layer

| Layer | After a native failure | Who decides |
|---|---|---|
| Adapter / runner | Returns a `SolveReport` with termination, or `Err(ProblemError)`. An error becomes `Outcome::Rejected` (`solves.rs:1873-1875`). There is no retry. | — |
| Step executor | `conclude` only annotates. There is no re-route or retry. | — |
| Session | Drops `Retained` when assess returns false (`math/staged.rs:324`). The sequence continues. | assess closure |
| Route preparation | Only *pre-execution* reconsideration: a factorable preparation failure (Unsupported or Contract) inserts `refusals[backend]` and re-runs the decision loop (`solves.rs:1199-1210`). Identical repeated demands → RouteRefused (`:1143-1145`). | `prepare_resolved` |
| Authored sequence | Stops at the first unaccepted step unless `continue_independent` (`run.rs:820-830`). | `launch` |
| Initialization (authored) | `retryable()` classification (`engines.rs:181-219`, `scientific_attempt_retry` `:222`). A failed stage or first homotopy point stops; a retryable homotopy failure halves the step; the attempt cap is checked (`:519-525`); the deadline is checked via `bounded` (`staged.rs:512`). | `Initializer` |
| Block initialization | The first uncommitted block ends the stage and the schedule; there is no retry (`initialization.rs:1021-1047`). | `Blocks::stages` |
| Conditional unit inside a recycle sweep | A refused result becomes `Err(Contract)` in the causal map (`initialization.rs:656-666`). KINSOL's fixed-point iteration sees an evaluation error. | KINSOL |
| Nested implicit | Non-success → `MathError::Domain` "native inner solve did not converge" (`implicit.rs:355-365`), which surfaces as an outer evaluation error | outer native solver |
| Fitting profile chain | Halves toward the last accepted point and stops below `smallest` (`fitting/profile.rs:428-440`) | `Chains::run` |
| Study | `pse_operations::study_policy::transition`; mathematical failures are marked `RetryFailure::Deterministic` (`study_execution.rs:~400`) and are never retried automatically. Seed unavailability → `FreshFallback` only when `UnavailableSeedPolicy::FreshOnUnavailable` (`pse-operations/src/study_policy.rs:346-357`). | policy crate |
| Horizon advanced step | A refused prediction (`Fallback`) → full solve (`driver.rs:1278-1302`) | driver |

**Where "no backend fallback after native failure" is enforced.** No code path re-routes
after execution. The rule holds because of what is absent, not because of a guard. Stated
in: `solve.rs:40`, `lib.rs:133` (Unavailable), `routing.rs:964` and `:1136`,
`initialization.rs:41`, `strategies/conditional.rs:35,188`, docs §18.7
(`numerical-execution.md:1063`), §17.1 (`:519`, `:584`). Lexicographic staging is
"no fallback" by test (`execution/tests.rs:883`). The only place that does re-decide is the
pre-execution `refusals` loop in `prepare_resolved`.

### (f) Time, iteration and work budgets

- Per step: `Controls.time_limit` and `iterations` (`solve.rs:245-305`, defaults 300 s and
  3000). `Execution::new` starts the clock per attempt (`solves.rs:2002`).
- Sequence deadline: `Staged::step` clamps `time_limit` to the remaining deadline
  (`staged.rs:316-322`). `bounded` covers preparation, execution and assessment
  (`staged.rs:512-555`). `initialize_model` has one wall deadline and `maximum_attempts`
  (`engines.rs:380`, `:519`). Authored sequences (`run.rs:792`) and studies have **no**
  cross-step time budget, only per-step controls.
- Nested work: a conditional unit takes the minimum of its own and the enclosing elapsed
  plus allowance (`initialization.rs:619-627`). Implicit inner solves use their own
  `Options.time_limit` and `iterations` (`implicit.rs:246-250`). Fitting profile chains use
  the fit's remaining time (`fitting/profile.rs:259-262`). The fixed-assignment re-solve
  clones step controls (`factorable.rs:~1411`); whether its time is charged against the
  parent step's started clock was not verified.
- Memory and work: job slots, CPU permits per step, worker budget and charges (`WorkerBudget`,
  `math/staged.rs:205`), the foreign allowance (`solves.rs:2005`), and the advanced-factor
  charge (`solves.rs:2719`).

### (g) `ResolvedAccuracy` flow

Resolved once per prepared step from `ResolvedNumericalPolicy` and the step's tolerances
and normalization (`solves.rs:999`; type `solve.rs:70`; `resolve :150`; `from_policy :170`).
The path is `PreparedSolve.accuracy` → `execution::Step.accuracy` (`solves.rs:2315`) →
runner → `Input.accuracy` → adapter native options (`nlp_options` `solve.rs:94`;
KINSOL scaling through `Budgets`, `runner.rs:296-300`). It is also used by `kkt::Budget.dual`
(`runner.rs:212-215`), `record_kkt` and `qualify` (`runner.rs:218-219`, `:315-316`), the
coefficient MIP bound (`runner.rs:544`), and `objective_accuracy` (`solves.rs:519`). User
`Controls` never carry accuracy (F20). Separate resolutions:

- block steps (`initialization.rs:69`, `solves.rs:~1507`);
- conditional unit (`initialization.rs:483`);
- recycle (caller-supplied, `strategies/conditional.rs:144-145`);
- fitting (`FitProblem.accuracy`);
- **nested implicit KINSOL uses `ResolvedAccuracy::from_policy(&Default::default(), …)`**
  (`implicit.rs:253-262`), the default `NumericalPolicy`, not the enclosing one;
- the SOS bound uses `accuracy.stationarity.max(1e-9)` (`solves.rs:2076`).

---

## 3. Workflows that compose multiple solves

| Workflow | Executor | Policy it re-decides |
|---|---|---|
| Single modeling solve (`ModelingSolvePreparation::start`, `run.rs:662`) | `Staged` (one-step authored sequence) | — |
| Authored sequence (`Runtime::launch`, `run.rs:731`) | `Staged::run` with `predecessor()` | stop-on-unaccepted (`run.rs:823`) |
| Conic request (`PreparedConic::start`, `strategies.rs:71`) | `MathService::solve` (own session, assess always true) | — |
| Authored init: stages + homotopy + original (`engines.rs:363`) | `Staged::step` | homotopy step control, retry classification, attempt cap and deadline, value-only seeding |
| Objectives / lexicographic stages (`workflow/objectives.rs:248, 327, 439`) | `Staged` | own accepted chaining (not inspected in depth) |
| Studies (`study_execution.rs:180`) | `Staged::run` and `Staged::batch` (coefficient batch, `solves.rs:1883`) | seed availability and permission (`memory_seed` `:506`, study_policy), FreshFallback, batching only fresh starts (`:254-282`) |
| Rolling horizon (`horizon/driver.rs`) | `Staged::run` + `Staged::native` | estimator and controller chaining, value and native seeds, advanced-step predict-or-solve (`:1215-1311`) |
| Block initialization (`MathService::initialize`, `initialization.rs:896`) | **Own loop** on `NativeSession::step` (not workflow `Staged`) | per-block route, stage chaining, PreviousAccepted seeding, commit rule (`commit_block`) |
| Recycles (`PreparedRecycle::start` → `solve_declared_root`) | **Own job**; KINSOL `Session` direct (no adapter, no runner) | fixed KINSOL fixed-point (`kinsol::Function::FixedPoint(CausalMap)`) |
| Conditional unit inside a recycle (`evaluate_conditional_unit`, `initialization.rs:577`) | `execution::roots` runner, fresh `Retained` | commit rule |
| Nested implicit solves (`implicit.rs:228`) | **Own** KINSOL session cache, no runner | own settings and accuracy, deterministic start |
| Fitting (`PreparedFit::execute` → `FitProblem::execute`, `fitting/oracle.rs:638`) | **Own** `submit_with` job; `execution::nlp` direct with fresh `Retained` | own route decision; forbids start and reuse |
| Fit profile chains (`fitting/profile.rs:250, 407`) | `execution::nlp` direct, Pinned oracle | own continuation (step, halve, secant bracket) |
| Shooting (`shooting.rs:1137`) | `execution::nlp` direct, fresh `Retained` | own start (forward integration) |
| Factorable fixed-assignment re-solve (`factorable.rs:1329`) | nested `execution::nlp` inside the factorable runner | own Auto routing, default NLP settings |
| Dynamics (trajectory adapters `execution/dynamics.rs`; `workflow/modeling/dynamics.rs`) | integrator path (Representation::Trajectory, excluded from algebraic routing `execution.rs:53`) | consistent initial conditions inside the integrator (IDACalcIC); route selection not traced |

Duplicated policy (observed):

- Continuation and step control exist at least three times: authored homotopy
  (`engines.rs:655`), fit profile chains (`profile.rs:407-440`), and the horizon advance
  (predict, then full solve).
- Commit and acceptance rules exist twice: `commit_block` (native_use plus extent) and
  `ModelingResult.accepted` (complete). The session keep rule is a third (`AssessedPoint`).
- Route decision is constructed in five places with hand-built `Requirements`/`Context`:
  `prepare_resolved`, block `strategies`, fitting, fixed-assignment re-solve, and
  `prepare_conditional_unit` (`initialization.rs:~500`).
- KINSOL-direct paths (recycle, implicit) bypass `BackendExecution` and runners. They
  build their own `Compatibility` stamps with non-standard meaning
  (`profile: accuracy.key()` at `initialization.rs:770`; all fields = `problem.identity`
  at `implicit.rs:329-334`).
- Start policy admission is restated per workflow: fitting (`fitting/modeling.rs:218`),
  block init (`initialization.rs:160`), conditional unit (`:682`).

---

## 4. Testability without native solvers

| Concern | Native-free tests | Notes |
|---|---|---|
| Routing and selection | `routing.rs` tests from `:1377` on (`caller_inventory_limits_automatic_selection :1410`, `sensitivity_requests_route_to_multiplier_adapters :2061`, `explicit_only_classes_never_automatic :1955`, …); `routing/contextual_tests.rs` (fake adapters through `Table::new`, e.g. `:79`, `:118`, `:299`); `execution/tests.rs:149 stub_backend_routes_through_adapter_table`, `:419 published_capabilities_equal_routing_rules`, `:883 …no_fallback` | Pure functions over `Table`. Some use `LINKED` with `test_context`, so results depend on the features linked. |
| Runner and retained reuse | `execution/tests.rs:149` stub adapter runs `roots` twice through `Retained` (cold, then seeded) | Backend-level only. |
| Candidate use and completion | `workflow/numerics.rs` tests (fixture reports, e.g. the new `validation_failure_projects_unavailable_feasibility_and_retains_resource_cause`); `math/initialization.rs:1361 block_commit_is_atomic…` | Pure. |
| Start and seed decisions | `solve.rs` seed validation; `start_tests.rs` is `#[cfg(feature = "solver-kinsol")]` (`strategies.rs:210-212`) | `admit_step` and `Staged::seed` / `predecessor` have no native-free tests. |
| Staged-sequence policy (homotopy, stages, retry) | `staged.rs` native tests and `engines.rs` tests are KINSOL-gated (`staged.rs:607`, `engines.rs:761+`); `initialization_restores_original_specification` (`engines.rs:950`) is not gated, but still needs a linked solver at run time | `Staged` hard-wires `NativeSession` → `MathService` → `execution::LINKED` / `adapter()` (static, `execution.rs:404`, `solves.rs:1040`). There is no injection seam for a stub table above the backend crate. Only `bounded` is tested without natives (`staged.rs:562`). |
| Session lifecycle | `math/staged.rs` admission tests (`:477-511`) | These use a real `MathService`; whether they solve natively was not checked. |

---

## 5. Automatic choices from mathematical facts, and how they are recorded

| Choice | Code | Recorded |
|---|---|---|
| Class-ordered automatic backend | `routing.rs:1226-1243`, `:1004-1033`; `Capability.automatic_classes` + `automatic()` | `Decision{selection, classes, eligibility, selected}`; published capability rows (`execution.rs:114`) |
| Sensitivity-capable preference | `Requirements.sensitivity` → `pick(true)` first (`routing.rs:1240-1243`) | **Not recorded as a reason**: `Decision` has no field for the preference (`routing.rs:177-205`). Only the outcome (`selected`) is visible. |
| Pending class evidence before a lower class | `pending_class_evidence`, `decide_assessed` keeps rank position | `Decision.evidence`, `pending_backend`, `state` |
| Derivative order demand | `ArtifactDemand::Derivatives` (`routing.rs:942-952`) → `prepare_order` (`solves.rs:1348-1360`) | `Decision.artifacts` |
| Presolve pass selection (Auto) | `Pipeline` | `report.preprocessing` passes `requested`/`eligible`/`applied`/`reason`; `Resolution::RelaxedRows` |
| Numerical PSD qualification | `ConvexityPolicy::Numerical` (`solves.rs:1016-1038`) | certificate evidence (request-only) |
| Retained-session reuse | `Retained::session` | `report.evidence.reused_native_state`, metric `reuse.native_model` (for example `kinsol.rs:178-181`) |
| Advanced-step predict vs solve | `kkt::predict` validity checks | `HorizonDecision::{Predicted, Fallback, Solved, Held}` and the `fallback` metric (`driver.rs:1364-1367`) |
| Fixed-assignment re-solve route | nested Auto (`factorable.rs:~1440`) | own hashes; whether the route is surfaced in the report was not verified |
| Block start as seed vs start | `PreviousAccepted` and later stage | `StartReceipt` |

---

## 6. Uncertainties and unresolved edges

- How `ModelingResult.accepted`, `Record.decision` and the session keep flag diverge on
  real outcomes (seam in (b)) was inferred from code; it was not executed.
- The objectives / lexicographic sequence (`workflow/objectives.rs`) and study `operation.start`
  for non-DeclaredCase operations were not traced in depth.
- Dynamics integrator route selection and its consistent-initialization fallbacks were not
  traced.
- Whether the fixed-assignment re-solve's time is charged against the parent step's clock,
  and whether its route is published, was not verified.
- The `MathService::solve` callers in `pse-py` (`workflow.rs:1197/1219` use `with_start`)
  were not traced further.
- Docs §17.6 mention `Start::Seed(k)` and say "study steps seed values only"; the code has
  neither (studies submit an Explicit native `WarmStart`).
- Plan 25k edits are in progress. The `routing.rs` and `solve.rs` hunks quoted above may
  move.
