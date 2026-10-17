---
title: Simulation performance diagnostics
status: in-progress
date: 2026-10-14
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
  - docs/design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md
doc_topics: [design, preparation, environment, validation]
---

# Plan 40: Simulation performance diagnostics

## Purpose and scope

Make an ordinary simulator operation inspectable before expensive preparation starts, while it runs, and after completion or interruption. An inspector should be able to ask what is currently executing, where time and actual work have gone, what changed in the preceding minute, and which observations support a suspected inefficiency. Context-owned numerical facts and optional profiles deepen that explanation. They do not create a universal convergence score or establish a causal claim that the observations cannot support.

This series adopts all six findings and the supporting investigations from the [simulation performance diagnostics review](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md). It covers preparation and edits, numerical methods and properties, initialization and recycles, sensitivities and postsolve analysis, dynamics and adjoints, studies, fitting and profile likelihood, surrogates, shooting/collocation and horizons, local/managed Rust/Python entry points, publication, reads and drain. It also corrects the independently established affine-proof amplification exposed during the review. The roughly 622-second historical cold interval motivates investigation; its cause and a comparable speedup have not been established.

**Interface-checked baseline:** `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the inspected concurrent working tree. **Proposed:** the target and execution packages in this series. Authoring these documents establishes neither product implementation nor newly Tested/Measured simulator behavior. The source review keeps its original conclusions and evidence. This coordinator alone owns adopted finding dispositions; companions own their native package status.

Plan 40 is distinct from active Plans 28/32 and the completed selected Plan39 qualification. It neither supersedes their owners nor transfers their acceptance. Environment and provenance proof work has been permanently eliminated by the operator; this series must not recreate it as a prerequisite, deferred task or future investigation. Actual operation identity, scientific/model/result lineage and owned-resource drain remain necessary operating contracts. Exact comparative performance benchmarking, repeated cold samples, a hard latency SLA and environmental equivalence are outside scope.

## Resulting system and responsibilities

Ordinary operation start creates lightweight inspectable state before preparation can occupy the caller. Current activity, cumulative facts and bounded recent context are distinct read products. Inspection defaults to the preceding 60 seconds, shortened to actual observed duration; incomplete or lost coverage is explicit. The operation links actual children, dependencies, shared producers and waiters across async tasks, native threads and managed processes. Scientific run and result identities remain separately authoritative.

Producers publish already-available facts at safe ownership points. Native observations retain method, phase, coordinate, unit, clock, activation and reset semantics. A numerical checkpoint is neither an accepted physical state nor a closed scientific result. Readers consume immutable publications independently of an occupied solver or actor. They do not call executing foreign handles or request additional scientific work. Lightweight observation is the ordinary default; opt-out says what was uncollected. Deep capture, sampling, costly plan/detail collection and optional allocation or wait investigations are explicit actions.

Explanation separates observed facts, qualified hypotheses and the next observation that could discriminate them. Count growth, cost per call, repeated construction, rejected trials, factor/refactor fallback, stalled physical time, blocked dependency and shared-producer waiting have different interpretations. Unknown work is not zero; unattributed time is not automatically waiting; inclusive nested work is not summed into a false total. Profiles can localize execution cost while remaining unable to prove algorithmic necessity or scientific convergence.

| Document and owner | Responsibility and delivered boundary |
| --- | --- |
| [40a: Operation context and live history](40a-operation-context-and-live-history.md) | Shared observation meanings, early operation lifecycle, propagation, retention, coverage, independent access and resource/time interpretation. These contracts have one owner here, rather than separate definitions in every adapter. |
| [40b: Context and native diagnostics](40b-context-and-native-diagnostics.md) | Actual producer facts, safe library projections, context-specific scientific interpretation and their consumers. Owns the catalog of capability decisions and context scenario routing. |
| [40c: Inspection, analysis and profiling](40c-inspection-analysis-and-profiling.md) | Typed Rust/Python/CLI reads, context explanations, explicit capture, useful sampling/timelines and bounded tooling decisions. |
| [40d: Affine preparation locality](40d-affine-preparation-locality.md) | Local original-row aggregate proof, qualified substitution choice, dependency-complete reuse and migrated mathematical consumers. |
| [40e: Qualification and closeout](40e-qualification-and-closeout.md) | Assembled diagnostic and locality acceptance, scope-end checks and durable documentation/retirement. Does not absorb other plans' qualification. |

The shared domain is observation of actual simulator operations, not a new workflow scheduler, solver-policy engine or generic telemetry database. Use existing tracing and native/library capabilities where they fit. A new property or experiment should supply its owned detail through the same navigation contract. A replacement solver changes its adapter projection and capability facts, without teaching every formatter its getter semantics. A new placement supplies propagation/read transport while retaining the same coverage and scientific permission distinctions.

## Foundation assessment

Current code already supplies typed callback work, preparation counts, native work observations, engine metrics, caller dispatch propagation in `TaskTracer`, shared-flight cancellation ownership, nonblocking durable taps and closed-result admission. Preserve and compose these contracts. In particular, the numerical observation aggregator already retains inclusive owners and makes totals unknown when components are missing; it is contrary evidence to any recommendation to replace all existing accounting.

The missing foundation is continuous operation lifetime and causal context. `RunHandle` follows expensive preparation, generic progress retains an early prefix, terminal progress deliberately requires closed evidence, and native-child/managed execution lack a complete live read relationship. Raising a cap or opening provisional result manifests does not supply the missing contract. A busy surrogate actor's queued response is also an unsuitable live inspection boundary.

The selected low-dependency direction extends the existing `pse-diagnostics` boundary with a separate observation module; runtime composes retention/access and context owners publish their particular detail. Lower producers already depend on it. `pse-ids` depends on `pse-diagnostics`, so a reverse dependency is forbidden. A0 confirms the concrete public/dependency boundary before implementation; no new crate is presumed. The 40b source assessment also identifies that `CompilerWorkspace::rebuild` replaces the workspace with `Self::new`: B1 must preserve observation attachment across rebuild rather than attach a one-time Salsa callback that disappears on generation change.

Static inspection establishes selected-row affine proof amplification independently of timing: row requests scan global coordinates and perform repeated substitution. The correction must preserve proof of the original aggregate, including cancellation across individually nonlinear contributions, exact binding/alias semantics, guards, fixed-value assumptions and allowance/unknown outcomes. A support index is useful only if complete and correctly mapped; it must not become another full global scan per row or a reduced-row proof substituted for original evidence.

## Rule decisions

| Rule impact | Decision and date | Selected target and route | Dependent work |
| --- | --- | --- | --- |
| [RC01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#rc01) | Accepted by the operator, 2026-10-14: adopt the review target | Expand blueprint §23.1 from optional observation of existing run scope to lightweight current/cumulative/recent operational diagnostics from ordinary start, with explicit opt-out coverage. Deep capture remains optional. Install this accepted change through the existing design-change route with a blueprint revision; use an ADR only if the concrete change meets the repository's ADR criteria. | G0 precedes production changes depending on the new default/lifetime contract. A0 and bounded source inquiries can proceed meanwhile. |

Acceptance selects this target; it does not make it implemented or qualify product behavior. No further rule impact is selected by plan authoring. If an investigation requires a new enduring contract, new binding or other governed change, identify that concrete impact and obtain its decision before the dependent change. Do not invent an ADR for ordinary tooling, tests or refactoring within contracts.

## Coordination packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| G0 | Accepted RC01 and this authored series | Install the accepted observability change at its authoritative owner through the existing decision/design route. Preserve scientific admission, ownership, optional deep capture and the eliminated proof scope. Reconcile any newly discovered rule impact before its dependent implementation; do not edit immutable ADRs in place. | Implemented — blueprint revision 174, §23.1 |

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| [F01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f01) | [Principal S01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s01), S03, S07–S12 | scheduled | [40a A0–A4/A7](40a-operation-context-and-live-history.md#work-packages); 40b producer migration; 40c typed reads | Close only with pre-handle access and actual task/thread/shared/managed propagation integrated and exercised, retaining scientific identity and cancellation ownership. |
| [F02](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f02) | [Principal S12](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s12), S01/S11 | scheduled | [40a A2/A4/A5/A7](40a-operation-context-and-live-history.md#work-packages); 40b B10; 40c C2 | Close with current/cumulative/recent/terminal distinctions, late inspection, explicit loss and opt-out, interruption/writer/death coverage and actual ownership drain. |
| [F03](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f03) | [Principal S03](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s03), S07/S11/S12 | scheduled | [40a A1/A6](40a-operation-context-and-live-history.md#work-packages); 40b scoped producers; 40c C3 | Close with correct units/clocks/reset/window/shared/nested interpretation and preserved unknown totals, not merely populated fields. |
| [F04](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f04) | [Principal S02](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s02)–S10 | scheduled | [40b B0–B11](40b-context-and-native-diagnostics.md#work-packages); 40c C3 | Close with adopted context/library routes resolved and actual producer/consumer projection, scientific-state distinctions and owner-safe access exercised. Unsupported optional detail stays explicit. |
| [F05](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f05) | [Principal S01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s01), S08/S11/S12 | scheduled | [40c C0–C5](40c-inspection-analysis-and-profiling.md#work-packages); 40a/40b consumed contracts | Close with useful typed live/recent explanations and a working deeper profile/capture route, with qualified limits and independent cleanup. An unresolved frame problem is not completion. |
| [F06](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f06) | [Supporting S26](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s26); principal S01/S12 | scheduled | [40d D0–D4](40d-affine-preparation-locality.md#work-packages); 40b B1 observations | Close with sound local original-row proof, settled substitution/support decisions, actual consumer migration and mathematical/locality controls. No speedup or historical-cause claim without corresponding measurement. |

40e supplies assembled acceptance to these dispositions; a completed companion package alone does not resolve a finding with other contributors. No finding is rejected, silently excluded or declared resolved by creating this series.

## Source coverage and investigation ownership

The [evidence index](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/README.md) remains the source of reviewed facts. All its diagnostic opportunities have a route, with conditional/native detail distinguished from required lightweight coverage. The two S-number namespaces are local to their documents and must remain qualified in acceptance reports.

| Principal scenario family | Implementing projection and supporting controls |
| --- | --- |
| [S01 Preparation/edits](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s01) | B1 and D0–D4; supporting S01–S05/S23/S26. Demand versus execution, actual invalidation/reuse and pre-handle work. |
| [S02 Numerical representations](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s02) | B2–B5; supporting S05/S06. Root/NLP/coefficient/conic/discrete and native phase/reset distinctions. |
| [S03 Properties/implicit work](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s03) | B2/B5/B7; supporting S06/S08/S09. Nested derivative/property/inner solve, original versus transformed meaning. |
| [S04 Initialization/recycles/continuation](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s04) | B7; supporting S07/S08. Stage/retry/raw-map trial versus accepted state. |
| [S05 Sensitivities/KKT/uncertainty](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s05) | B5/B7; supporting S09. Actual RHS/factor work and withheld-premise reasons. |
| [S06 Dynamics/IC/events/adjoints](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s06) | B6; supporting S10–S12. Physical/returned/mesh time, setup causes, rejected work and segment reset. |
| [S07 Studies](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s07) | B8/A3/A4; supporting S13–S15. Occurrences, ready/frontier/skew and actual worker scope. |
| [S08 Fitting/profile likelihood](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s08) | B8/B2/B7; supporting S16/S17. Outer trial, experiment, derivative upgrade/cache and chain failure. |
| [S09 Surrogates](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s09) | B9/A4; supporting S18. Fidelity/proposal/original correction and occupied-actor reads. |
| [S10 Shooting/collocation/horizon](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s10) | B9/B6/B7; supporting S19/S20. Outer formulation, windows/actions, role and fallback/hold. |
| [S11 Placement/entry/result journey](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s11) | A3–A5/B10/C2; supporting S21/S23/S25. Rust/Python sync/async, native child/managed transport, actual publication/read work. |
| [S12 Long/interrupted/shared](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s12) | A2–A6/B10/C4; supporting S22/S24/S25. Saturation, independent readers/waiters, writer failure and native drain. |

The [40b capability routes](40b-context-and-native-diagnostics.md#native-and-library-capability-routes) cover Ipopt, KINSOL, PETSc, HiGHS, Clarabel, POUNCE/FERAL, UNO, SCIP, Diffsol and IDAS; sparse faer/KLU/MUMPS/SPRAL/oneMKL; Symbolica/Numerica, Salsa, properties, DataFusion and actual Surreal query statistics. Each opportunity receives an implementation, bounded conditional investigation, unsupported or genuinely inapplicable route with reasons. Not every optional getter or costly timing mode becomes always-on collection. FeOS's reference-oracle role is not promoted into an imaginary live property engine. The exact Symbolica 3.0.1 contract must be checked rather than assuming the skill's 3.0.0 slice settles it.

The [40c investigations](40c-inspection-analysis-and-profiling.md#bounded-mechanism-investigations) cover useful Rust/native sampling, tracing export/open intervals/flows, optional scheduling/allocation/native-heap/wait capabilities and caller-owned subscriber composition. The existing owned attachment probe decoded timestamps but yielded no useful frames. A successful tool exit is insufficient. Choose a working local mechanism or return a concrete decision if the required profile cannot be delivered; do not turn an unavailable-tool message into acceptance. Tokio Console and candidate exporters remain capability options requiring exact-source and bounded functional checks, not selected new infrastructure by name alone.

A0 owns low-layer placement, light retention/cadence/bounds and independent access decisions. B0 owns safe native activation/ABI/getter and context coverage decisions; C1 owns profiling/export choices; D0 owns complete support and exact substitution semantics. Each records its decision in its existing plan explanation and updates dependent packages. Source inspection is primary; use a small probe only when it discriminates a material choice. Library questions follow current library skills and Context7 as applicable, never the stale library-utilization or capability-map files. No second exhaustive capability registry or per-command evidence ledger is required.

## Prerequisites and execution sequence

1. Install G0 before dependent production change; settle A0's common boundary. In parallel, B0, C1's source/tool inquiry and D0 can resolve independent uncertainties. C0 requires settled A1 semantics, but not every producer implementation.
2. Deliver A1/A2's actual low-level publication and early operation state. Compose the first useful slice with B1 preparation producers and C2 reads, including the coverage portion of A5 needed by those reads. This slice is not blocked on every native family, export option or full A5 failure inventory. C0 and A4 consumers co-design the same interface; they are not a circular implementation prerequisite.
3. Propagate through A3 and implement A4 for the actual thread/process/managed boundaries. Integrate B2–B10 and C2/C3 by the context contracts they consume. A5 interruption/writer and A6 cost/resource semantics participate in each relevant slice; B10 completes the dependent publication/drain adoption after those contracts exist. Settled semantics do not substitute for an implemented transport or migrated consumer.
4. Deliver C4's selected deeper capture after its identity/transport needs, independently of unrelated numerical projections. D1/D2/D3 follow their own soundness decisions and integrate the A1/B1 observation slice when consumed. F06 is independent corrective scope and is not a barrier to useful diagnostics. A simultaneous bulk substitution change is conditional on exact semantic qualification; the sound local aggregate correction remains required.
5. A7/B11/C5/D4 finish actual consumer migrations, targeted controls and deletion of replaced code/tests/fixtures. Do not retain compatibility APIs or a second production observer path. Distinct scientific terminal/result contracts survive because they have a separate meaning, not as compatibility shims.
6. Once functional work and bounded choices are settled, 40e runs assembled qualification and scope-end checks once, repairs scoped failures and closes dispositions only with the corresponding evidence. It owns durable documentation transfer and eligible retirement. Other plans' unfinished qualification remains at its existing owner.

Among ready packages, start with pre-handle/current/recent visibility because it assists later investigations. This is an ordering preference rather than another synchronization barrier. Logical independence does not grant shared-file ownership: the coordinator retains shared types/generators/manifests and integrations; delegated executors receive bounded producer/consumer surfaces. Use worktrees only where simultaneous edits genuinely require isolation, then integrate and remove fully merged worktrees.

No default daemon, mandatory remote collector, global subscriber takeover, model-wide copy per callback, new scheduler, root-cause inference engine or event for each arithmetic primitive is required. Retention, queues, retained bytes and optional writer lifetimes are bounded according to their roles. Their settings do not become numerical policy identity or execution limits. Callback publication is nonblocking and avoids formatting/I/O; process death can preserve only the last published prefix. Never query a mutable foreign handle concurrently merely because it exposes a getter.

## Verification and completion

**Proposed:** packages compile touched owners and run meaningful targeted controls during implementation. Pure observation/window/ownership tests use pure local inputs, not canonical storage or native services. Arrow-consuming correctness closures explicitly activate force-validation; generated API changes are made in their declarations and regenerated. Disabled/enabled observation controls preserve scientific contract and actual evaluation requests without demanding identical trajectories where scheduling or elapsed limits can affect outcomes. Record actual collection cost honestly without inventing a performance gate.

**Proposed:** 40e selects assembled local/managed/native/Python journeys by distinct contract, not an exhaustive configuration cross-product. A qualified snapshot test does not establish live foreign-handle safety; a passing Ipopt case does not establish a different callback ABI; typed absence is acceptable for genuinely optional detail but cannot close missing required operation/history/profile capability. Final claims name actual commands, mode, scope, zero-failure baseline and omissions. No measured speedup is promised or required to remove source-established redundant work.

Completion requires all required functional packages integrated, bounded decisions resolved or explicitly changed by the operator, useful live/recent inspection across supported contexts and placements, the qualified deeper-profile route, sound local affine proof, migrated consumers and removed replacements. It also requires scoped assembled acceptance, enduring contracts/guides updated through their proper routes and current finding dispositions reconciled. Creating these plans is only the authoring step.

## Current checkpoint

The post-Plan41 [diagnostic corrective handoff](40e-qualification-and-closeout.md#post-plan41-corrective-prerequisites)
routes new responsibility-summary retention/export and exact receiver association through
completed [Plan42c](42c-responsibility-diagnostics-and-receiver-discovery.md). Plan42 owns the
new review's F06/F07 and accepted RC01/RC02; this series retains its original findings and
40e qualification. Those corrections are prerequisites for the affected summary/managed
acceptance, not completion of E0–E6 or a reason to repeat unaffected profile/locality work.

The operator authorized the remaining 40c C0–C5 and 40d D0–D4 design changes after the
accepted 40a/40b foundation and minimum Rust read slice. Execution includes supporting
producer/locator changes, generated Python/CLI consumers, explanations, explicit capture,
affine locality, targeted acceptance and documentation. 40e's assembled qualification,
hygiene/manual checks, final finding closure and retirement remain separately scoped.

The remaining design-change execution is complete: [40c's Outcome](40c-inspection-analysis-and-profiling.md#outcome-recorded-after-implementation)
owns C0–C5 inspection, generated consumers, explanations and selected CPU/timeline capture;
[40d's Outcome](40d-affine-preparation-locality.md#outcome-recorded-after-implementation)
owns D0–D4 local original-row aggregation/proof and simultaneous substitution. Their focused
acceptance records the joint 101-test selection, final 21 capture controls, 38 pure leaf
controls, 32 Python controls, useful linked Rust/HiGHS profile, default-feature all-target
compile, native extension refresh and generation. Counts overlap and are not summed into a
qualification total. The consumer guide describes the implemented surface and its limits.

Existing dirty work is preserved and all assigned subagents returned; no new worktree was
created. The prior 130-test and managed-worker results below retain their original source/scope.
A current managed-worker refresh was not run; current private-child and remote prefix controls
passed in the targeted selection. No focused result becomes assembled acceptance. E0 can now
select 40e's remaining assembled journeys and scope-end checks. All six finding dispositions
remain scheduled until that acceptance and closure; the series is not complete.

The approved stale Plan39 fixture cleanup is complete, preserving data/evidence and its
unproved outcome. The selected fresh v6 fixture/resource-registry route remains available;
legacy stores and the permanently eliminated environment/provenance proof scope are intact.

## Selected implementation verification

**Tested — 2026-10-15, zero-failure baseline:** the joint targeted selection below passed
130 tests across nine package binaries, with no failures or compiler warnings and 2,325
unselected tests. It uses the pinned toolchain, linked native environment, explicit
Arrow force-validation through the repository selection route, two build jobs and two test
threads. The selected resource registry is `/tmp/pse-friction-review-resources-uMrtOxGw`;
its selection avoids a separately fenced legacy registry and does not migrate it.

The exact invocation was `bash /tmp/pse40-run-targeted.sh`; that temporary script ran:

```bash
export PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw
export CARGO_BUILD_JOBS=2
scripts/pse-env --native -- just unit-package pse-runtime 'test(observation::tests) | test(observation_) | test(live_callback_) | test(rolling_progress_) | test(contended_progress_) | test(raw_map_) | test(unlinked_native_return) | test(native_observations_survive_zero) | test(selected_plugin_statistics_) | test(copied_watch_context_) | test(diffsol_adjoint_live_roles_) | test(native_step_observations_preserve_segment) | test(pounce_convex_batch_matches_single) | test(pounce_convex_changed_qps_) | test(native_failed_fp_and_picard_) | test(retained_allocation_requires_fresh_numeric_) | test(native_report_terminal_callback_cause_precedes_user_wrapper) | test(completed_metrics_observe_actual_) | test(metrics_capture_is_selected_) | test(native_sdk_statistics_) | test(canonical_statistics_) | test(canonical_client_attaches_statistics_) | test(actual_private_child_publishes_) | test(scoped_deadline_signals_native_worker_) | test(scoped_cancellation_before_deadline_) | test(completed_handle_drop_) | test(fitting_value_demand_) | test(terminal_timer_projection_) | test(explicitly_enabled_native_timers_) | test(unchanged_kkt_pattern_) | test(faer_numeric_refresh_) | test(klu_refactor_) | test(actual_native_path_passes_fold_) | test(compact_horizon_bounds_roles_) | test(propagation_preserves_scalar_sum_bits_) | test(checked_upload_clips_binary_) | test(unscaled_conic_updates_) | test(release_refresh_retains_symbolic_) | test(sparse_border_keeps_orientation_) | test(faer_factor_consumes_serial_) | test(exact_candidate_sensitivities_) | test(sensitivity_withheld_when_sosc_) | test(inverse_reduced_hessian_over_solve_columns) | test(fitting_parallel_experiments_keep_order_and_owner_affine_workers) | test(compatible_native_scope_reuses_) | test(native_scope_affinity_) | test(persistent_scope_stacks_survive_)' -p pse-diagnostics -p pse-columnar -p pse-kernels -p pse-math -p pse-compiler -p pse-engine -p pse-operations -p pse-backend-native --features pse-runtime/native-solvers --execution-effects native-local --ignore-default-filter --test-threads 2
```

The selector covers bounded pure history/loss/reset/nonfinite/opt-out behavior; large-future
layout and interruption; actual task/thread/shared-flight context and two-waiter cancellation;
compiler invalidation/reuse; provider/implicit scientific request counts; enabled/disabled
and failed native projections; factor/refactor/reuse; forward/adjoint segment semantics;
continuation and quantity-specific withholding; serial/parallel fitting attribution;
horizon background/prediction permissions; actual private native-child checkpoint/drain;
and original SDK-response statistics without query replay. The full log is
`/tmp/pse40-targeted-tests-final.log`, Nextest run
`ae27a962-e262-4ef8-acfd-63a527d4d21b`. Filtered tests are unexamined by this selection.

**Tested — pure dependency boundary:**
`CARGO_BUILD_JOBS=1 scripts/pse-env --resource-class light -- cargo test -p pse-diagnostics observation --lib`
passed 29 tests, zero failures/warnings, without native services, canonical storage or Arrow.
The default light policy and large `!Unpin` future layout are included. Log:
`/tmp/pse40-core-layout-test.log`.

**Tested — actual managed boundary:**
`PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw CARGO_BUILD_JOBS=2 scripts/pse-env --resource-class exclusive --native -- .venv/bin/python /tmp/pse40-managed-diagnostics-current/run.py`
built the current validated worker, provisioned a fresh owned v6 store and invoked
`cargo nextest run --no-fail-fast -p pse-runtime --locked --test worker --features pse-runtime/native-solvers -E 'test(worker_runs_authored_case_end_to_end)' --ignore-default-filter --test-threads 1`
through `scripts.test_run.observer_rust`. Result: one passed, zero failed, eight unselected;
Nextest run `5c44b5cc-ff26-4b6e-a808-d6dad159356c`, 17.602 seconds of test execution.
The test retained scientific/result/progress assertions and cross-process stored-seed checks,
and verified the actual child's distinct process identity, canonical run/attempt references
and joined-native observation. It removed only that reaped child's matched diagnostic file.
The registered fixture stop completed; databases/evidence remain at
`/tmp/pse40-managed-diagnostics-v6-p5nj2ajg/state`. Log:
`/tmp/pse40-managed-worker-final.log`.

**Interface-checked:** the earlier native
`just check-package pse-runtime --features native-solvers`
passed with zero errors/warnings; the final linked targeted build subsequently compiled the
changed consumers. `scripts/pse-env -- just ready` passed skill sync and doctor after adding
the exact `pin-project-lite =0.2.17` dependency edge; no other resolved package moved.

**Interface-checked — default features:**
`PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw CARGO_BUILD_JOBS=2 scripts/pse-env -- just check-package pse-runtime`
passed the all-target compile with explicit Arrow force-validation, zero errors and warnings.
The recipe already supplies `--all-targets`; the initial duplicated flag was rejected before
compilation and was removed from the invocation. Log: `/tmp/pse40-default-check-final.log`.

**Implemented:** all delegated edits are integrated in the existing main checkout. No
additional worktree remains. The accepted owner contract is blueprint §23.1/revision 174;
enduring Rust usage and bounded/unsupported semantics are in the
[consumer guide](../dev/simulation-diagnostics.md). Companion outcomes remain needed by
40c–40e and are retained with their evidence until the series' closeout/retirement step.

**Tested — turn closure:**
`PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw scripts/pse-env -- just turn-end`
passed both ADR indexing and formatting, baseline zero. The initial invocation without the
selected registry was fenced before checks; no registry recovery/migration was attempted.
Log: `/tmp/pse40-turn-end-final.log`. `git diff --check` also passed. No task-owned agent job or additional
worktree remains; owned managed fixture services are stopped and unrelated dirty work is
preserved.

**Not run:** full hygiene/manual checks, Python/CLI projection, causal explanation/profiling,
affine-locality correction and assembled all-context/scientific qualification. These remain
owned by 40c/40d/40e. No comparative speedup, overhead bound, SLA or environment/provenance
proof is claimed. The focused science and ownership controls establish only the named scope.
