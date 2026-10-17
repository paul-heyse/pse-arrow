---
title: Pipeline performance optimization using operation diagnostics
date: 2026-10-17
tier: design
purpose: target
decision: Revise
doc_role: review
doc_topics: [design, numerical-execution, diagnostics, performance]
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  process-simulator: "1.5"
---

# Pipeline performance optimization using operation diagnostics

## 1. Assessment and boundary

**Decision: Revise.** Short production-path observations expose useful optimization opportunities without an extended simulation or a timing tournament. The most consequential are computing transient sensitivities for unconsumed parameter directions, repeating final fitting integrations, and losing structural reuse during active studies under pressure. Sparse implicit verification and repeated KLU symbolic analysis add source-established amplification. Two diagnostic lifecycle defects obstruct causal interpretation of those operations. None requires tighter scientific accuracy or a second solve pipeline.

**Architectural fitness fails in the named execution-fit scenarios.** No incorrect accepted scientific answer was established. Exercised assertions and source inspection support the scientific boundaries described below; they do not qualify every model or backend. Remedies are **Proposed**, including their performance benefits. Measured work counts establish the particular work observed, not an end-to-end percentage improvement.

The target is the best-in-class simulator in the Process Simulator functional target and blueprint §0/§0.2: interactive value changes without structural reconstruction, studies, sensitivities, optimization/fitting, recycles, dynamics from the same definitions, model/solver extension and independently testable responsibilities. The judgment basis is Core 3.4, efficiency heuristics 1.0, Process Simulator 1.5 and the binding selected by `standard.toml`. Existing decisions describe the subject; proposed rule changes appear in §8.

The inspected baseline is `b8c535888075d84bbd9ef54767aacfb283601d5e`, initially clean. Plan43 is complete at that baseline. This review neither reopens its completion ledger nor treats old reviews as unimplemented scope. Product source remained unchanged during this inquiry. Review-support changes add compact selectors and a cooperative observation deadline to the existing runner; two small public-API drivers supply cases without rebuilding the cleaned Rust target directory.

The coordinator conducted the probes and inspected decisive source. Two fresh design reviewers independently examined mathematical/native execution and workflow execution, then assessed conflicting explanations and additional evidence. The library skills supplied capability routes; Context7 and exact vendored/source contracts supplied the relevant library checks. Stale utilization records and capability maps were not used.

## 2. What was run and what the observations establish

The [case notes](../evidence/pipeline-performance-optimization-2026-10-17/case-notes.md) give commands, conditions, repairs and coverage limits. The [case observations](../evidence/pipeline-performance-optimization-2026-10-17/case-observations.json) retain selected invocation documents, native counters, phases, integration records, coverage and raw-output locations. The two drivers use actual Rust production APIs through the installed editable native extension, explicit ephemeral runtimes, native libraries and ordinary engineering accuracy. They do not emulate numerical execution.

The final selection contains **37 observations: 34 passed/completed, two typed derivative-eligibility refusals and one experimental reuse-expectation violation**. The latter retained four scientifically usable, independently checked roots. The immediate-cancellation control intentionally has no usable scientific solution. The largest selected process observation was approximately **4.81 seconds**; elapsed values are exploratory, including driver effects, rather than latency acceptance criteria.

| Family | Executed coverage | Observation or remaining limit |
| --- | --- | --- |
| C01 lifecycle | Minimal physical/source admission; cold and repeated scalar solving; 16-package reference admission | The reference load admitted 119 files/1,010,520 bytes and 3,829 declarations. Physical admission was 0.74 s and source admission 2.89 s in one profiled observation. This does not reproduce or explain the historical 47-second heater observation. |
| C02 edits/reuse | Same prepared object, accepted primal start, equal preparation, value overlay, value source edit and structural edit | Bound overlay produced the accepted root 3 against the unchanged parent; source-edit and structural-edit cases also completed. Starts and preparation reuse are distinct. |
| C03 mathematical shape | Four/eight-variable diagonal and coupled square systems | Both shapes completed; the small ladder exercises support and callback paths, not a large-model complexity estimate. |
| C04 providers/implicit | Reference source admission and a scalar, branch-selected nested root | Scalar nested execution completed with actual inner/factor/IFT stages. Four/eight-unknown attempts refused unavailable smooth first derivatives; they establish no sparse-IFT scaling measurement. No property-heavy PC-SAFT solve was executed. |
| C05 numerical effectiveness | Nonlinear root, mixed scaling, guarded logarithm, small Ipopt/POUNCE NLP; KINSOL setup interval control | Default scalar root used 16 function/2 Jacobian evaluations; interval 1 used 5/4. An accepted start used 2 function evaluations with default setup. These are work tradeoffs, not a universal method winner. |
| C06 other classes | HiGHS QP and integer-count MILP; Clarabel cone | Original qualification, analytic candidate/objective and result checks passed. Global MINLP/certified searches and optional PETSc/Uno routes were not exercised. |
| C07 recycle | Declared ports, tear selection and accelerated scalar loop | Original fixed-point/qualification checks passed. Large or near-stagnating recycle networks were not run. |
| C08 dynamics | Diffsol/IDAS ODE, 5 versus 25 outputs; index-1 IDAS DAE; event/reset/quadrature journey | Matched ODE sample grids retained 16 internal steps in both backends. DAE and event assertions passed. Final controls use ordinary physical accuracy, including event localization. |
| C09 fitting | One-parameter transient fit, zero versus four additional fixed inputs; Diffsol and IDAS | Predictions and unit response derivatives passed. Complete selected integration records expose trial, upgrade, original-assessment and identifiability work. Shooting/curvature were inspected in source, not executed. |
| C10 studies | Four independent points and continuation; one/two workers; logical pool intervention | All analytic roots passed. Two-worker views changed from 4 at 16 GiB to 2 at 64 GiB; this is additional preparation work, not a measured parallel speedup. Durable/managed studies were not run. |
| C11 results | Repeated selected projections and retained study results | Three equal original-variable projections passed. Durable publication/reopening and large analytics were source-inspected, not measured. |
| C12 interruption/refusal | Immediate owned cancellation; actual typed capability refusals | No usable result was inferred from cancellation/refusal. Long opaque native cancellation latency and failure-recovery stress remain unexamined. |

**Measured interpretation controls.** Ordinary hub-off, ordinary hub-on and selected-detail scalar cases retained the same 16 function/2 Jacobian evaluations on the cold solve. This establishes preserved numerical work for those controls, not zero observer overhead. The once-only observations were sufficient to settle the findings; repeated timing pairs or longer simulations were unnecessary.

Launch CPU capture around reference admission produced raw perf data, decoded symbols and a Samply profile. The returned document reported 299 resolved and nine unresolved frames, explicit unknown clock alignment, sampling-loss uncertainty, source partiality and no related-operation links. Loader, Arrow-schema and serialization/allocation stacks are leads. This capture cannot support an exclusive phase-cost breakdown or a claim that a particular clone dominates source admission.

## 3. Responsibilities and preservation constraints

The existing ownership is useful: authored relations and physical context own meaning; compiler/mathematical preparation owns immutable structure and selected products; native libraries own iteration and factors; workflow owners compose experiments and qualify original results; observations copy facts; result/publication owners govern access and retention. Improve these compositions rather than replacing them with a second scheduler, optimizer or diagnostic authority.

| Distinction | Meaning and owner to preserve |
| --- | --- |
| Values versus derivative directions | Complete physical inputs remain available even when a consumer requests only selected derivatives. The derivative layout names coordinates and conversion scales; absence cannot become numerical zero. |
| Original scientific quantities | Authored units, basis, reference state, bounds, domains, balances and case-owned checks remain under physical/model authority. Ordinary engineering relative fraction is 0.001 in these controls; no global accuracy was tightened. |
| Native versus original convergence | Native residuals/steps use their declared coordinates. Original physical assessment, finite derivative checks and requested-product permission remain independent of native status. |
| Reuse | Immutable structure, mutable scratch, symbolic factors, numeric factors, trial products and accepted starts have different compatibility premises. An allocation checkout does not establish current numerical factors. |
| Dynamic time and samples | Model time is physical time; requested output times need not be native steps. Events, scheduled inputs, initialization, sensitivities and adjoints retain their actual boundaries. |
| Performance facts | Invocation, attempt, producer and generation identify the work. Inclusive stages overlap. Process RSS is not an operation allocation; configured allowances are not resident memory. |
| Progress and outcomes | Started/completed records, retained snapshots, selected raw history, original qualification and caller readiness remain separate. Missing or curtailed collection is not zero work. |

Root admission remains square original equality matching, while optimizing problems retain genuine degrees of freedom. The two larger implicit probes were correctly refused when first derivatives were unavailable; no unsupported fallback was invented. The discrete pilot was corrected to use the existing physical Count quantity rather than making Scalar integers admissible. These are safeguards to retain.

Representative changes are adding a fixed physical input to a fit, increasing a requested output grid, composing another study point and substituting an eligible native implementation. Their effects should remain with derivative layout, output/assessment, existing lane and integration owners respectively. Scientific model authors should not implement profiling, caching or solver iteration for each new unit.

## 4. Findings and corrective contracts

### F01 — Fitting propagates all parameter sensitivity axes after selecting only free bindings

**Priority: high; AP-07, H5/H9/H10/H15, PS-07/PS-11.** The [fit mapping](../../../crates/pse-runtime/src/workflow/fitting/sparse.rs) selects `free_bindings`, and [integrated experiments](../../../crates/pse-runtime/src/workflow/integrated.rs) gather only those responses. The integration request nevertheless carries only Forward/None. [IDAS](../../../crates/pse-backend-native/src/dynamics/idas.rs) initializes sensitivity vectors for every integration parameter; [Diffsol integration](../../../crates/pse-backend-native/src/dynamics/integrator.rs) likewise derives width from the full parameter vector.

**Measured:** in the final matched linear-accumulation fixtures, the fitted parameter remained approximately 3 and its response approximately 1 with zero or four additional zero-valued fixed inputs. Surviving Diffsol forward experiment counters reported 27 steps in both, but 54 versus 162 linear solves and three versus seven initial callbacks. These are named experiment counters, not a summed whole-fit total; ordinary snapshot partiality remains recorded. Source independently establishes the width amplification in both backends.

**Correction:** distinguish the complete parameter-value layout from an explicit selected seed/direction layout. Propagate that layout through initial-state derivatives, scheduled segments, integration and response gathering. Preserve scales and tied-binding addition. Full ordinary Forward simulation may still request every axis. IDAS already accepts an independently selected `Ns` with a supplied sensitivity callback; retain the native solver and derivative engine. Diffsol's selected-seed realization requires exact interface/design investigation.

**Verification:** the coupled-state/fixed-input/scheduled-input control must keep physical predictions and requested responses while native direction work follows consumed seeds. Never fill omitted axes with zeros. Reduced sensitivity error control may alter accepted native steps; original physical accuracy remains mandatory.

### F02 — Independent final fitting assessment splits values and responses across two integrations

**Priority: high; AP-03/AP-07, H8/H12/H20/H28, PS-10/PS-12.** [FitProblem::observe](../../../crates/pse-runtime/src/workflow/fitting/oracle.rs) correctly creates a fresh final oracle independent of optimizer callbacks, then requests values and subsequently `response_rank`. The response request upgrades the same point by integrating again.

**Measured:** complete `workflow.integration.v1` streams in both final fitting controls contain a value-only `original_assessment` integration followed by a forward `identifiability` integration. Each stream contains eleven actual integrations overall, including ordinary trials and derivative upgrades. The source explains why the final two are separate.

**Correction:** after known response admission succeeds, request one fresh final value-plus-response product and perform original assessment and rank analysis from its completed products. Retain a value-only fallback when derivative evaluation fails, so an independently qualified estimate is not lost with optional responses. Do not reuse optimizer callback caches to satisfy independent assessment.

**Verification:** a successful forward fit should need one fresh final integration, retaining its prediction, response, rank and governing qualification. Derivative/rank refusal must still retain any independently completed admissible value result.

### F03 — Known rank-product refusal is discovered after response integration

**Priority: medium; AP-07, H5/H10/H28.** In the same [response_rank](../../../crates/pse-runtime/src/workflow/fitting/oracle.rs), `evaluate_demand` precedes checking the prepared dense extent against `max_cells` and worker bytes. A product already known to be inadmissible can therefore cause transient forward integration before refusal.

**Evidence:** source-established ordering; the reviewed small rank-admissible fits do not measure this refusal path. Existing sparse-fit coverage provides a suitable candidate-preserving refusal scenario; the new compact benchmark selector uses 12 parameters because sizes four/eight do not exceed its unchanged rank cutoff.

**Correction:** compute dimension/overflow/admission facts before derivative work. Preserve independent value assessment and the unavailable-rank explanation. Do not pre-refuse based on an estimated numerical rank or invent a larger allocation.

**Verification:** an admitted fit with a known impossible optional rank extent should retain its candidate without an identifiability IVP. Other derivative products genuinely required by optimization remain unaffected. This check is a prerequisite to F02's successful-path fusion.

### F04 — Active study reuse relies on an evictable lookup for preparations still retained by results

**Priority: high for repeated studies; AP-07, H12/H13/H14/H21, PS-11.** [StudyPreparationLane](../../../crates/pse-runtime/src/workflow/study_operations.rs) retains selected admission but no current numerical solver view. [Bound-view preparation](../../../crates/pse-runtime/src/workflow/modeling/views.rs) consults optional shared retention. Pressure recovery can trim that retention and idle compiler lanes. Full [study results](../../../crates/pse-runtime/src/workflow/study_execution.rs) retain [their prepared products](../../../crates/pse-runtime/src/workflow/modeling/results.rs), so losing lookup ownership need not free the payload before an equal-structure point constructs another view.

**Measured:** four analytic results passed in both two-worker repeats. The 16 GiB logical pool constructed four views/ten observation programs; 64 GiB constructed two views/six observation programs and performed two rebinds. Modeling retention ended at five versus 27 entries. Both cache policy limits were 8 GiB; the driver's 4 GiB cache setting describes a different family. Actual process peak RSS was approximately 381 MB in both. These are pressure-sensitive work observations, not an exact eviction chronology or latency improvement. The borrowed `views <= workers` expectation was not a universal existing product contract.

**Correction:** retain and rebind one compatible, explicitly charged current immutable view with the existing active lane. Replace it on complete-key changes; release after drain. Preserve optional shared retention, finite admission, cancellation and per-point checks. Investigate source-qualified constructor allowances separately; RSS alone cannot justify reducing them.

**Verification:** same-structure points should reuse the active view even when optional lookup retention is trimmed, within the declared live-memory envelope. Changed structure/context/policy must rebuild. Cancellation and compact/full result retention must release the appropriate owners without an immortal cache.

### F05 — Sparse implicit derivative verification traverses structurally absent matrix entries

**Priority: medium, increasing with implicit block/response size; AP-07, H9/H10/H15/H28, PS-07.** [Implicit derivatives](../../../crates/pse-math/src/implicit.rs) correctly extract sparse unknown Jacobians and share a factor across first/packed-second RHS solves. `check_solve` then traverses every unknown coordinate for every row/RHS, ignoring the extracted sparse matrix. Diagonal/banded blocks therefore pay quadratic verification slots per RHS.

**Evidence:** source-established; scalar nested execution and separate factor/IFT observations were exercised. The larger multi-unknown fixtures were refused derivative eligibility, so no sparse scaling or latency claim follows from them.

**Correction:** retain the implicit owner's independent backward-error rule while computing matrix action and absolute-contribution scaling from existing sparse support. Compare a deterministic support traversal with faer's pinned sparse/dense multiplication. Keep explicit finiteness checks so skipping absent entries cannot hide a nonfinite derivative. This does not eliminate the separately dense residual jet.

**Verification:** analytic diagonal/banded first/second derivative controls must retain singularity/domain refusal and original tolerances while verifier work follows nonzeros and requested RHS. Large eligible implicit probes are a targeted follow-up, not a prerequisite for recognizing the dense loop.

### F06 — Retained KINSOL applications repeat compatible KLU symbolic analysis

**Priority: conditional on repeated sparse setup cost; AP-07, H12/H13, PS-09/PS-11.** The [KINSOL adapter](../../../crates/pse-backend-native/src/kinsol.rs) checks compatible identities and CSC structure. In the pinned native path, each nontrivial `KINSol` initializes its linear solver; [SUNLinSolInitialize_KLU](../../../vendor/sundials-sys/vendor/src/sunlinsol/klu/sunlinsol_klu.c) sets first-factorization state, and setup frees/recomputes symbolic analysis before numeric factorization. Allocation reuse therefore does not amortize that structural work across applications.

**Evidence:** the native source chain was inspected, including `kinsol.c` and `kinsol_ls.c`. Repeated scalar solves completed, but their retained work does not establish a representative symbolic-analysis cost or a chosen replacement. A zero-setup already-accepted start remains legitimate.

**Correction/investigation:** find a supported native linear-solver capability, or a narrowly maintained native integration, that retains symbolic analysis under exact pattern/ordering compatibility and forces fresh numerical factorization for every new application. Compare maintenance/lifecycle cost against observed symbolic work before selecting it. Never merely suppress required initial setup or transfer this factor into an unrelated faer IFT problem.

**Verification:** changed coefficients reuse only compatible symbolic state; changed pattern/ordering rebuilds; fresh numeric setup and failed-factor cleanup remain observable. Preserve existing fresh-application numerical correctness controls. No bespoke nonlinear iteration is proposed.

### F07 — Ephemeral dispatch occurs before its invocation recorder exists

**Priority: high as an analysis prerequisite; AP-05/AP-06/AP-07.** [Ephemeral fit, simulation and shooting start paths](../../../crates/pse-runtime/src/workflow/run.rs) dispatch native work before `supervised_with_cancel` creates the recorder. [Job submission](../../../crates/pse-runtime/src/math/jobs.rs) captures invocation context at construction and transfers that captured absence into workers. Durable dispatch occurs inside the later scope. An outer invocation can instead be captured, giving native and returned work different recorders.

**Measured/source distinction:** final ephemeral fitting documents contain only the outer join and completion intervals, with `omitted_intervals=0`; their selected raw integration streams contain the actual eleven integrations. The current `clock_scope` honestly excludes pre-supervisor work. The defect is broader than that exclusion: later fitting assessment stages also cannot reach this recorder. Zero capacity omissions does not establish performing-owner coverage. Diagnostic operation context and native counters transfer independently and are not generally missing.

**Correction:** establish one invocation context before dispatch and carry it into native work, supervisor and result. Preserve synchronous refusal/admission timing unless deliberately changed. Expanding the clock is a concrete contract change in RC01; retaining a narrow clock instead requires explicit coverage limitations and cannot yield the complete execution comparison target.

**Verification:** equivalent ephemeral/durable fitting and simulation must retain their actual owner stages under the correct invocation, including nested invocations, cancellation and failed final assessment. Do not fabricate a missing interval or move scientific ownership to the recorder.

### F08 — Fitting closes an operation before publishing its final integration record

**Priority: high for trustworthy live/terminal analysis; AP-05/AP-06, H20/H26.** [evaluate_experiment](../../../crates/pse-runtime/src/workflow/fitting/oracle.rs) calls `operation.finish` before `integration.finish`. [Producer publication](../../../crates/pse-diagnostics/src/observation/producer.rs) offers selected raw records before updating retained state, while the retained node rejects updates after closure.

**Measured:** final fitting child snapshots have operation outcome Completed and producer phase `started`, including original-assessment and identifiability children. The explicitly selected complete stream contains both started and completed records for those same integrations. Thus this is an ordering defect, not evidence that numerical work is stuck, a failure to select the family or a need to enlarge all budgets.

**Correction:** publish the integration's terminal record before closing its containing operation. Keep optional writer failure separate from scientific permission. Shooting's inspected window ordering already provides contrary evidence against a universal integration-observer defect.

**Verification:** inspect both retained current/history and selected raw records after a real experiment, including derivative failure and cancellation. Both must retain the final available integration state without accepting updates to an already closed operation.

### F09 — Full trajectory checking repeatedly searches the sample grid for each sample

**Priority: lower for short grids, rising with output density; AP-07, H5/H10/H28, PS-12.** [check_samples_impl](../../../crates/pse-runtime/src/workflow/modeling/dynamics_checks.rs) performs `.position(time)` from the beginning of coverage points for each retained sample. Ordinary Full/no-observer trajectories and final shooting assessment use this path; Compact live checks have a different online path. The ordinal lookup is quadratic even when checks themselves are constant work per sample.

**Evidence:** source-established. The five/25-output controls confirm that output density need not change native work; they do not establish that ordinal lookup dominates these millisecond trajectories.

**Correction:** carry original ordinals through the ordered prefix and merge the independently retained endpoint once. Retain every physical check, correct endpoint input/mode identity, missing-prefix distinctions and original coverage.

**Verification:** Full and Compact paths must agree on original sample identities and checks for sparse/dense outputs, events, partial prefixes and off-grid endpoints. Native step counts should not acquire a sample-density dependency from the correction.

## 5. Further opportunities requiring a concrete choice

These are bounded investigations, not established defects or selected numerical winners.

| ID | Question and candidate target | Settling evidence and preservation |
| --- | --- | --- |
| O01 trial product scheduling | The complete simple-fit stream has four same-point derivative upgrades in addition to final work. Could a selected forward value-plus-response trial profile avoid replay when gradients are consumed next? | Compare accepted and rejected line-search trials, small/large selected seed counts and gradient-only/adjoint routes. Eager derivatives can waste work on rejected trials; retain lazy behavior where it wins. Do not implement host-owned optimizer iteration. |
| O02 shooting integral derivatives | When forward sensitivities are already required and controls are few, could native quadrature sensitivities provide integral gradients without another checkpointed forward/backward operation? | IDAS exposes `IDAQuadSensInit`/`IDAGetQuadSens`; integration remains unqualified. Exact derivatives require the explicit derivative callback rather than the null-callback finite-difference default. Check resets, schedules/reinitialization, quadrature error control, initial derivatives and many-control adjoint crossover. Preserve adjoints; record RC02 before changing the documented integral-gradient realization. |
| O03 immutable inner compilation | Does resolving fresh value/configuration hints unnecessarily recompile compatible residual/hint programs in separately prepared implicit cases? | Inspect `prepare_model_providers`, `reconstruction_factory` and compiler implicit compilation with exact program/limit/order keys. Compare value-only cases and retained-study paths. Keep mutable hints and native scratch separate; no universal new cache. |
| O04 provider weighted products | Can a changed multiplier reuse a complete same-point jet under an explicit immutable-provider premise? | Current provider projection barriers are intentional: stateful providers can require re-execution. Preserve guards, failure/context/order/weight semantics. Flat Symbolica weighted lowering cannot be transferred through those barriers without a valid contract. |
| O05 numerical profiles | The scalar root trades 16/2 function/Jacobian evaluations for 5/4 with setup interval 1. Which existing profiles fit cheap versus expensive derivatives and sparse factors? | Short matched roots/recycles with actual setup/evaluation costs and unchanged accuracy/start. Native KINSOL monitoring remains the controller. One scalar case does not justify changing every default or ranking all solvers. |
| O06 preparation/unit cost | Reference admission's CPU capture contains Arrow schema, serialization and allocation stacks. Which work is repeated, and which is necessary first admission? | Obtain a selected phase-aligned capture or narrow allocation/call-count probe before recommending shared metadata/bulk conversion changes. Separate Python loading, diagnostic serialization and scientific admission. Do not infer a hot owner from unresolved or unaligned samples. |

## 6. Integrated target and prerequisite order

Retain one scientific execution pipeline. Consumer requests specify the products and derivative directions they need; preparation exposes immutable reusable structure; existing lanes own mutable state; native libraries own iterative algorithms and factors; independent original assessment governs results. Diagnostics describe those owners from request/dispatch through response and drain, without initiating hidden science.

F07/F08 first remove uncertainty in joined and current explanations; explicit selected streams already allow the present numerical diagnoses. F03's deterministic preflight then enables F02's successful-path final fusion. F01 supplies the derivative-layout boundary needed by fitting and the O02 shooting investigation. F04 preserves prepared views within existing lane lifetimes rather than relying on global cache survival. F05/F09 remove source-established amplification independently. F06 and O01–O06 remain conditional where library fit or workload tradeoffs are not settled.

Consequence-based priority and prerequisite order differ. Selected sensitivities may have greater growth impact than a diagnostic ordering repair, while the repair is cheap and makes later comparisons easier. No measured percentage or fleet-wide winner determines this order. Proposed changes must compose without duplicating caches, redefining scientific permission or creating a second work-charging authority.

## 7. Architectural and scientific judgments

| Foundation | Verdict in the examined boundary |
| --- | --- |
| AP-01 separation | Satisfied: semantic, numerical, workflow, observation and publication owners remain identifiable; proposed corrections belong to those owners. |
| AP-02 contracts | Satisfied in the inspected existing full-axis sensitivity and explicitly scoped invocation contracts. Proposed selected-direction and expanded-clock contracts require deliberate evolution, not repair of an undefined existing meaning. No backend-neutral wrapper is prescribed. |
| AP-03 composition | Satisfied structurally; F02/F03 identify unnecessary work in otherwise owned fitting composition. |
| AP-04 semantic model | Satisfied in inspected scientific authorities and typed outcome/product distinctions. Full versus selected derivative layouts must remain explicit as the target evolves. |
| AP-05 explicit structure | Satisfied in the inspected typed ownership and partiality boundaries. F08 is a close-before-publication defect within that structure, rather than evidence that missing observations silently become complete history. |
| AP-06 local reasoning | Violated in F07's invocation propagation: whether native work reaches its recorder depends on dispatch ordering and execution mode. F08 separately limits useful current-state interpretation despite explicit partiality. |
| AP-07 execution fit | Violated by F01–F05/F09 and the active-study pressure scenario; F06 has confirmed repetition with unresolved remedy/importance. |

**Core gates:** G1/G3/G4 pass in the inspected ownership, admission and passive-read boundaries. G2/G5 pass for the truthful scoped/partial observations inspected: F07/F08 limit causal coverage, but no silent semantic drop or partial history presented as committed completeness was established. G6 passes for exercised accepted transformations/reuse, with no acceptance of the proposed new ones. G7 passes for the recorded supported/refused capabilities and explicit measurement limits. G8 passes for inspected numerical delegation; a sparse verifier loop is an inefficient realization, not a reason to reimplement a solver. G9 fails under the foundation verdicts above.

**Profile gates:** PS-G1/PS-G2/PS-G3 pass for the exercised physical fixtures, structural/capability refusals, original assertions and native outcome paths. Complete thermodynamic-model conformance, all conservation families, all derivative orders/backends, long failures and numerical IDAES parity were not requalified. No favorable small-case judgment is promoted into whole-product qualification.

## 8. Rule impacts and follow-up ownership

| ID | Current rule or target | Recommended change | Dependency and alternative if retained |
| --- | --- | --- | --- |
| RC01 | `InvocationEvidenceDocument.clock_scope` currently starts at first supervisor poll and excludes earlier native work; runtime `run.rs`, blueprint §23.1/proposed ADR-0192 diagnostic boundary | Establish one recorder before dispatch and describe that expanded boundary explicitly, including coverage. Migrate affected comparisons/tests together. | F07. Retaining the narrow boundary requires explicit native-stage unavailability; it cannot be represented as equivalent complete execution coverage. Preserve synchronous start/refusal behavior unless separately chosen. |
| RC02 | Blueprint §13.6's current shooting integral-gradient realization uses an adjoint | Permit a qualified selected forward quadrature-sensitivity realization alongside the retained adjoint when consumed products favor it. | O02 only; no implementation or universal winner selected. Retaining the rule keeps the adjoint route and its extra forward/backward work where applicable. |

F01 deliberately evolves the internal all-parameter sensitivity request/result layout into explicit selected directions while preserving ordinary full-Forward semantics; it does not require relaxing an accuracy rule. F02/F03/F04/F05/F06/F08/F09 require no identified governing-rule relaxation. No dependency bump, host configuration change, license exclusion or tighter accuracy is proposed.

All findings and opportunities are **unscheduled** pending subsequent plan creation. This principal review owns the combined judgment; a future adopting plan should own disposition and implementation status. Existing Plan43, Plan40 and other plans retain their independent statuses. No production remedies were applied during this review.

## 9. Next testing rounds and completion

After corrections are authorized, rerun the same short anchors with original assertions. For F01 add a genuinely coupled/scheduled sensitivity control; for F02/F03 compare final integration purposes and refused products; for F04 impose controlled cache pressure with retained Full and Compact outputs; for F05 obtain an eligible sparse multi-unknown implicit fixture; for F06 record actual symbolic/numeric KLU work across retained applications. Check F07/F08 on both ephemeral and durable paths before interpreting combined timelines.

Only then enlarge dimensions, fixed-input count, output density, study width/skew or physical horizon one axis at a time. A longer run must answer a named unresolved question—such as stiffness-dependent solver effectiveness, adjoint crossover, native sparse-factor cost or managed publication tail—that the compact controls cannot settle. Use a short intermediate case first. Timing repetitions belong only to decisions that remain ambiguous after work counts and source analysis; no hard performance SLA or statistical-significance claim is introduced.

The approved initial review is complete with the explicit runtime limits in §2. Supporting runner unit tests and document checks are recorded in the case notes separately from scientific observations. Full Rust rebuild, whole-repository qualification, environment/provenance proof, machine tuning and production optimization were not performed.
