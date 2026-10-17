---
title: Operation evidence and coherent live diagnostics
status: done
date: 2026-10-17
adrs: [ADR-0192]
review_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, validation]
---

# Plan 43a: Operation evidence and coherent live diagnostics

## Purpose and foundation assessment

This companion delivers F01/F02/F03 and the common evidence contract consumed by F04, the numerical packages and [43e](43e-diagnostic-analysis-artifacts-and-profiling.md). [Plan43](43-solver-diagnostics-and-performance.md) owns rule decisions, combined scope and finding dispositions. This document owns A-package progress. The coordinator records operator acceptance of RC01–RC11 on 2026-10-17, including their bounded investigation limits. The execution checkpoint and Outcome below distinguish implemented scope from acceptance.

**Interface-checked:** the review and current `pse-diagnostics::observation`, backend `CallbackState`/`Progress`/observation, runtime publication/inspection/capture/durable progress, benchmark collector and generated boundary consumers. The coordinator records the baseline. Existing operation identities, typed absence, generations, explicit loss, bounded independent histories and pure inspection are suitable foundations. Retain them rather than create another recorder/work ledger.

The material gaps are specific: nested callback producers publish the same metric key with different generations; fields from one numerical event are projected independently; benchmark work/timing summaries use different populations; native timing availability needs a common truthful projection. History coalescing is honest but insufficient to reconstruct joined iterations. Native extraction stays with [43c](43c-native-methods-reuse-and-matrix-boundaries.md), and source/semantic explanation stays with43e.

## Target contracts

<a id="a01"></a>

### A01 — Producer-scoped facts and activation

Bind native callback boundary, coordinate transport and original oracle to distinct producer roles within their actual operation/attempt. Capture the context at the owner that performs the work, including async/native-child propagation; do not assign a producer later from metric prefixes. Prepared handles or explicit child scopes carry stable meanings without formatting/searching a name on every hot update.

Counts, absolute counters, durations, gauges and unavailable values retain their declared populations/units and reset generations. Native/original/scaled coordinates, norm definitions and inclusive nesting are explicit where interpretation depends on them. One work-charging owner remains authoritative. Reporting two nested producer intervals is allowed; summing them as exclusive time or charged work is not.

Retain phase totals/counts/direct endpoints. Where useful, performing owners accumulate finite duration distributions/extrema locally; log2 buckets are an option, not required machinery. Boundary thread CPU is supplied only for a synchronous interval on the actual owning thread, with explicit clock/scope; migrating async work or parallel children cannot be assigned one thread's exclusive CPU by subtraction. Unsupported/disabled CPU observations remain unavailable. Controlled nested/parallel intervals and distribution reset/overflow tests preserve these meanings; no default expensive clock or wall-minus-thread-CPU wait inference.

Migrate all shared callback consumers, not an Ipopt-only example: original/presolve adapters, foreign NLP/TNLP, other shared native callbacks, final report metrics, live snapshots, durable progress, inspection/timeline and benchmark consumers. C01 supplies actual activation-aware native facts. A01 supplies the common interpretation and public document contract; there is no competing timer implementation here.

At least two actual same-demand nested Jacobian calls and a recoverable failed trial must preserve separate series without alternating generations. The existing different-demand charging test is insufficient. Validate disabled, running, interrupted and unavailable states without measured zero and retain typed causes/panic containment/original qualification.

<a id="a02"></a>

### A02 — Coherent current checkpoint and bounded recent history

Publish fields whose interpretation depends on one event as one coherent producer-issued record. Required meanings are producer/attempt identity, event sequence/generation, actual event time, native method phase, iteration/step meaning, coordinate/norm convention, typed values and availability. Native method families may expose different typed payloads; do not force LP/MIP/conic/dynamic events into one sparse NLP row. Add no scientific permission to this record.

The current checkpoint remains available when optional history retention is zero, subject to explicit collection admission/availability. Retain bounded recent records and selected summaries with their actual timestamps. Preserve first/latest/coalescing as a separate view, not reconstructed iteration tuples. Independent additive counters keep their own generations and coverage.

Migrate backend Progress/event publication, runtime hub/process checkpoints, remote inspection, native child and managed receiver locators, durable progress encoding, timeline capture, generated Rust/Python boundary documents and CLI readers in the same package. Version changed external encodings and regenerate at the source declaration. Cross-process reads remain non-atomic and preserve clock uncertainty; joined event membership is not a claim of synchronized process snapshots.

Read injection/contention between events must never publish a mixed row. Cover history-off, disabled/refused collection, resets, nonfinite/overflow, sequence loss, active/terminal eviction and stale retained readers. Dynamic native time versus output time, trial versus accepted state and restoration versus ordinary iteration remain native-owner meanings. Ordinary current progress is not an accepted physical result.

<a id="a03"></a>

### A03 — Explicit finite detailed collection

Select event families, interval and retention policy explicitly. Ordinary live collection stays bounded and store-independent. Detailed studies may retain finite sampled history or request complete coverage of a declared family/interval. A 4096 ring or decimated tail is sampled. A complete route either streams successfully within selected finite output/memory limits or returns an incomplete capture outcome with missing sequence/coverage; it cannot silently relabel a prefix as complete.

Reuse existing Progress, runtime Tap/Streamer, capture and artifact owners. A collector never keeps native execution alive after its owning scientific work drains. Optional writer/queue failure cannot change scientific result permission; a diagnostic study can independently fail its evidence-sufficiency requirement. No implicit backpressure on a numerical callback unless an explicit selected contract establishes its semantics; prefer owner-local fixed records and bounded asynchronous export.

Integrate native histories only through bounded owner-safe paths. POUNCE's upstream collector can grow while executing and its adapter truncates only at return; merely enabling it is not this correction. Do not flatten iteration arrays into thousands of metric keys or shadow the caller's tracing subscriber. C06 supplies specific fitting hooks; A03 needs only that slice, not every backend/profiler.

Acceptance includes burst/oversized records, contention, queue refusal, writer failure, cancellation, output exhaustion, collector failure, normal teardown and hard death. Report acknowledged/unacknowledged or final-prefix limits honestly. A valid result with failed optional telemetry stays valid; a numerically failed run may have sufficient diagnostic evidence.

<a id="a04"></a>

### A04 — Per-invocation joined evidence and endpoints

The actual invocation owner produces a record referencing existing operation, native attempts, strategy/charging identities, effective settings, starts/reuse, requested products, stage intervals, work coverage and outcomes. Instrument preparation, transformation, native execution, recovery, original/workflow qualification, requested analyses, publication/materialization/reopening at their owners. Related native/probe operations contribute references, not independently reconstructed charges.

Keep qualification completion, terminal elapsed, response readiness and each product's availability distinct. Failed qualification has no qualification endpoint. A typed response may be ready with a withheld product. The governing original check may occur after backend quality assessment. Capture failed attempts and required postsolve work even if the native solve was fast.

Migrate `benches/benches/native_process` and helpers, `scripts/case_measure.py`, K4-derived collectors, applicable workflow result/report consumers and E04 export. Write one record per executed invocation, including warmups if executed. Criterion measured samples/statistics remain separate; use verified harness membership or label it unavailable, never guess by order. Preserve once-only smoke/audit semantics and original assertions. No new statistical campaign or provenance comparison is required.

D02/D04 supply fitting/shooting/dynamic product reasons and readiness facts; B supplies census/demand facts. A04 may integrate those incrementally, but it closes only when actual supported consumers are migrated. Do not require the complete invocation record to begin a source-proven local math/IDAS correction.

Acceptance: two invocations with different retries/products retain different work/timing/outcomes; failure/cancellation/withholding preserve missing endpoints; cold/warm/explicit-seed roles remain distinct; unknown totals do not become known after joins. Nested durations remain inclusive. Server statement nanoseconds do not become client elapsed, and process RSS does not become an operation-private peak.

**Implemented — production-completeness correction, 2026-10-17:** joined scientific-product availability now uses each modeling step’s governing original completion and the shooting result’s original permission. Native sensitivity/root-response/reduced-Hessian evidence and partial trajectories remain retained diagnostic data after refusal. A later failed step cannot revoke an earlier accepted step’s products. [43d/D04](43d-dynamics-fitting-and-workflow-effectiveness.md#d04) owns that permission; A04 consumes it through the existing joined Rust/Python/query projection. Terminal elapsed and caller readiness remain independent.

<a id="a05"></a>

### A05 — Observational control effects and identity

Settle O08 by classifying each collection, history, native timer/collector, system profile and numerical-probe setting by actual effects. Current `Controls::identity` serializes history together with numerical controls. Inspect all key/compatibility consumers before removing any field. A numerical probe or callback that can stop/change work is not passive retention simply because it is called diagnostics.

`PSE_DIAGNOSTICS=off` is first-registry optional hub collection, engine ObservationPolicy::Off concerns operator metrics, and history 0 concerns retained progress. Mandatory scientific work/accounting and baseline local callback bookkeeping remain. Define propagation to native/async/managed-child owners and visible reporting without claiming a physically zero-overhead baseline.

Separate genuinely observational retention from mathematical preparation identity only after RC10 and effect analysis establish that boundary. Preserve actual mode in execution/comparison records even when it does not invalidate prepared math. Time overhead can alter deadline-limited execution; do not promise identical trajectories under every diagnostic mode. Any identity/wire change is versioned and all affected construction/reuse/readback consumers migrate together. If unsound, retain conservative identity and explain the remaining invalidation; a supported no-separation result can settle this investigation.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A01 | Existing callback/work authority; RC01 for changed external contracts | Explicit producer scopes/prepared facts across all shared callback consumers; actual same-demand failure/control cases; remove old ambiguous publications | done |
| A02 | A01 producer contract; RC01 | Coherent current/recent records and versioned complete consumer migration; no mixed event under contention/history-off | done |
| A03 | A02 record meaning; RC02; C06's owner-safe extraction slice only where needed | Finite sampled/complete-selected collection with honest outcomes and bounded native retention; retire end-only truncation/flattening in migrated paths | done |
| A04 | Early invocation/endpoint contract and existing work identities; RC01 where needed | Joined records across harness/workflow/analysis consumers with failures/withholding and correct sample populations; remove aggregate-only interpretation | done |
| A05 | Current control/key consumers; RC10 before identity change | Effect-classification decision plus qualified identity separation or supported retained-identity branch; mode propagation and consumer migration | done |

A05 may start before A01/A02. A04's consumers depend on particular producer facts, not every detailed collector. Logical independence still requires shared schema/callback/publication edit coordination.

## Verification and checkpoint

**Proposed:** low-level collection/interpretation tests use injected clocks/contention/loss and need no solver or store. Native extraction tests are scoped separately; regenerate changed documents/contracts through their owners. Compile touched packages and run focused tests with force-validation where Arrow is consumed. G02 qualifies active and terminal Rust/Python/managed consumer integration after functional completion; G03 assesses selected observer-cost questions without a fixed SLA.

Current state: A01–A05 are complete, including the D04-owned governing scientific-product availability correction. [43g’s remediation Outcome](43g-qualification-and-design-publication.md#production-completeness-remediation-2026-10-17) records its affected acceptance; earlier foundation acceptance below retains its original scope.


### A05 effect classification and selected branch — 2026-10-17

**Interface-checked:** `Controls::identity`, execution/comparison profile keys, report allowance, native-session hashing and mathematical preparation consumers. Retain the full control identity. History changes the report-memory allowance and execution exposure, so removing it from that identity would erase a real admission difference. Native-session compatibility already omits passive history; compiled mathematical artifacts use their own narrower meanings. No identity split, new compatibility shim or relaxed native admission is needed.

| Setting | Actual effect and retained boundary |
| --- | --- |
| `PSE_DIAGNOSTICS=off` | First-registry optional hub collection; mandatory science, accounting and baseline callback bookkeeping remain |
| Engine `ObservationPolicy::Off` | Operator instrumentation policy; distinct from native progress and scientific checks |
| Progress history zero | Recent retention disabled; independent compact current event and source-head/loss still available |
| Selected source families/stride/window/budgets | Explicit future-owner propagation, bounded optional output; sampled versus complete independent from science |
| Selected synchronous callback CPU | Actual-thread inclusive CPU only; no async/parallel attribution or wall-minus-CPU wait inference |
| Native timers/collectors | Actual activation/return state, including not-requested/running/interrupted/unavailable; no disabled measured zero |
| Numerical probes and stopping controls | Scientific/work effects retained in original method, demand and admission identities; not passive retention |

Detailed selection reaches future Rust async/native owners and explicitly budgeted managed children. Each managed group receives at most one separately declared allowance divided across bounded attempts including retries; children retain the original deadline. Missing/refused artifacts stay explicit and current-attempt inspection does not assert atomic whole-study completeness. Optional output failures never change scientific permission. Added timing can affect deadline-limited trajectories; no zero-overhead or identical-trajectory claim is made.

<a id="a-execution-checkpoint"></a>

### Execution checkpoint — 2026-10-17

Historical foundation checkpoint: A01–A05 and observation-only B01/B05/C01/C06/D02/D04/E01/E04 support were integrated and accepted for the then-selected scope. The current checkpoint above supersedes its overall completion interpretation. Actual ordinary-stack local Python and managed-child journeys pass, including original scientific assertions and enclosing cleanup. Replaced ambiguous publications, end-only POUNCE flattening and aggregate service-counter joins are removed. All three owned worktrees were integrated and removed. The Outcome records repaired static checks and the full-book input limitation. The then-incomplete feature sweep subsequently completed in G04’s recorded composite acceptance. Plan43 owns the final finding dispositions; this historical checkpoint does not supersede the completed current state.

## Outcome

### Delivery and boundaries

**Implemented — delivered foundation and production-completeness correction:** performing-owner producer roles and prepared typed facts; coherent native/current/recent records; bounded selected JSONL capture and finite page readers; explicit synchronous thread CPU; actual managed child selection and canonical StudyHandle/StudyReport readback; per-invocation workflow/harness evidence with distinct endpoints and populations. Process checkpoints, inspection and progress documents are versioned, and generated Rust/Python schemas and native stubs are regenerated through their owners. Durable progress, inspection, timeline, CLI, reports and benchmark consumers use the same source meanings.

The selected enabling slices add mathematical census/demand observations, native activation-aware facts and safe selected extraction, fitting/shooting requested-product reasons and artifact/inspection transport. B–G’s subsequent numerical, platform, profiler and assembled scope is recorded in those companions and 43g; this foundation Outcome retains its original acceptance boundary. Conservative full Controls identity is retained for the real report-memory admission effect; existing narrower native-session and mathematical compatibility remain valid. No numerical permission or original scientific assertion is weakened. ADR-0192 remains formally proposed.

Ambiguous shared callback publications, end-only POUNCE history flattening and aggregate before/after service-counter joins are deleted. Three owned worktrees were integrated before removal. Owned qualification services were quiesced, drained and stopped; fixture data and evidence remain retained. Enduring diagnostic meanings are published under blueprint §23.1/§21.1 and the simulation diagnostics guide. This companion remains because Plan43’s retained Outcome and proposed ADR verification consume it.

### Mistakes corrected

Independent review caught a close/drain race that could turn an early selected stop into normal completion. The first closing reason now survives late finish and delayed draining; actual writer failure remains independently incomplete. Initial/interior/final source-head coverage, disabled-mode allocation, refused-producer mapping, real clocks under ordinary refusal and compatible duration/count populations have revealing controls.

The original local Python study crashed during selected consumer acceptance. Source symbolication identified a multi-megabyte inline async child in the study lane. Non-inlined inert future construction and exact pre-allocation mathematical-budget charging remove that caller-frame amplification while preserving prepared members on refusal/unpolled drop and cleanup on cancellation. The regression uses an ordinary 8 MiB caller stack; no stack limit or worker policy is changed. A harness compile error also exposed an attempt to serialize an entire canonical revision row; the invocation records its canonical key instead.

### Deliberate deviations

Use once-only cold/warm small harness controls and original scientific assertions, without a statistical campaign, timing SLA or gain claim. Explicit unavailable native work is preserved for presolved cases. Selected evidence sufficiency is separate from scientific usability. The ordinary local Python acceptance uses the checked linked extension and service-free ephemeral runtime; it does not acquire an unrelated store fixture. Environment/provenance proof remains eliminated.

Preserve the supplied reports/review/evidence unchanged. Full-book publication is blocked by R1's missing title/H1; scoped publication excludes only that malformed input in memory, without changing publisher configuration or creating an adapter. Formal ADR acceptance and older-plan reconciliation remain separate.

### Verification — 2026-10-17

All passing checks below use a zero-failure baseline. Arrow-consuming correctness closures explicitly select force-validation; linked tests use the local solver/klu/isolation/uno/petsc capability environment. Checks qualify the selected scope and exercised journeys, not every simulation context or the full Plan43 target.

| Evidence | Command and selected scope | Result |
| --- | --- | --- |
| **Tested** | `just unit-package pse-diagnostics 'test(observation) \| test(inspection)'` | 62 passed; zero failures |
| **Tested** | `just unit-package pse-math 'test(demand_observations) \| test(compact_census) \| test(evaluator_census) \| test(demand_collection) \| test(selected_demand_rows) \| test(demand_counter_overflow) \| test(repeated_occurrences_share) \| test(exclusive_case_scratch) \| test(demanded_directional_case_actions) \| test(observation_distinguishes_actual_inner_attempt)'` | 11 passed; zero failures |
| **Tested** | `just unit-native-capability-package pse-backend-native native-solvers solver,klu,isolation,uno,petsc` with callback/observation/progress/POUNCE/final-validation/conic selector | 120 passed; zero failures |
| **Tested** | Linked `just unit-native-capability-package pse-runtime pse-runtime/native-solvers solver,klu,isolation,uno,petsc` with capture/invocation/inspection/current/durable/timeline and fitting/shooting/horizon selectors | 34 passed; zero failures |
| **Tested** | Same linked runtime recipe with ordinary-stack/real-member grant-drop-refusal/cancellation/distinct-binding/POUNCE-batch selectors | 5 passed; zero failures |
| **Tested** | `cargo test -p pse-diagnostics --doc --locked` through `scripts/pse-env` | 1 compile-fail doctest passed; CPU interval cannot migrate threads |
| **Tested** | `.venv/bin/python -m unittest scripts.tests.test_case_measure -v` through `scripts/pse-env` | 10 passed; zero failures |
| **Tested** | Checked linked correctness extension, `scripts.python_tests --unit-only --maxfail=0 --continue-on-collection-errors python/pse/tests/test_diagnostics.py -q` | 15 passed; zero failures |
| **Tested** | `just native-python build/acceptance/p43a-managed-final-20261017 --managed-primary-route python/pse/tests/test_studies.py::test_managed_selected_detail_retains_actual_worker_owners_and_complete_prefixes` with a fresh base directory and no worker restart | 1 passed; zero failures; enclosing quiesce/stop/release succeeded |
| **Tested** | `just bench-case-smoke build/acceptance/p43a-small-v6-20261017 --case cold-small-1 --case warm-small-1` with the owned v6 reference fixture | Both once-only cases passed, one completed invocation each; original assertions retained |


| **Tested** | Checked linked correctness extension, ordinary main-thread `pytest -n0 -m component python/pse/tests/test_studies.py::test_local_study_runs_without_store_and_retains_source_after_package_release`, fresh base directory, no worker restart | 1 passed in 2.96 s; zero failures; original source-retention assertions retained |
| **Tested** | `just hygiene`, followed by `python3 -m scripts.validation --group clippy --live` after repairing its findings | All hygiene steps passed in composed evidence; repaired full default and no-default workspace/all-target Clippy passed with `--keep-going` and zero warnings/errors |
| **Tested** | `just lint-solver-contracts` in the linked environment | Full native workspace/all-target Clippy passed with `--keep-going`, force-validation and zero warnings/errors |
| **Tested** | `just governance`; `just turn-end` | Both passed; governance's 25 tests and all selected generation/family checks had zero failures |
| **Tested** | Existing publisher staging/mdBook/annotation/Pagefind with only malformed R1 excluded in memory | 459 chapters published; selected local-link check had zero broken links. Full `just docs` remains blocked by the unchanged supplied R1 title/H1 defect |
| **Tested** | Lifecycle metadata/native-scope interpretation and `git diff --check` | Zero interpretation/whitespace errors for the authored scope |
| **Not run to completion** | Full feature powerset and no-default sweep | Two owned attempts interrupted for resource sequencing; no-default not run. Owned processes drained and allocations released. Complete sweep is assigned to G04 after B–F integration |

**Measured:** the cold-small control observed 47.338 s for the complete invocation, including 41.712 s source admission, 2.787 s preparation/start, 1.738 s workflow join, 1.009 s runtime admission and 0.091 s teardown. These direct intervals explain this exercised invocation; they are neither native solver elapsed nor a percentage improvement. Criterion measured-sample membership and absent native work remain explicitly unavailable.
