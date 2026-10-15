---
title: Cancellation shared work and admission
status: in-progress
date: 2026-10-15
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
  - docs/design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/preparation-and-lifetimes.md
  - docs/design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/workflows-results-and-resources.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, preparation, environment, validation]
---

# Plan 41b: Cancellation, shared work and admission

## Purpose and boundary

Make preparation stopping, sharing and resource admission follow the lifetime of the work they govern. This companion of [Plan 41](41-simulation-pipeline-efficiency-and-effectiveness.md) owns packages contributing to review [F06](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f06), [F07](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f07), [F08](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f08) and the bounded O07 pressure investigation. The coordinator owns dispositions and rule decisions. [Preparation/lifetime evidence](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/preparation-and-lifetimes.md) and [workflow/resource evidence](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/workflows-results-and-resources.md) distinguish demonstrated source mechanisms from unmeasured concurrent consequences.

Blueprint §14.3 preserves one mutable compiler writer, immutable prepared products and join-owned resources. §19/§20 govern workflow and durable outcomes; §23.1–§23.2 govern observations and failure meaning. Faster source processing cannot repair disconnected cancellation or a shared producer governed by an unrelated waiter. Conversely, fixing these lifetimes does not establish why the single historical cold request was slow.

**Interface-checked baseline:** source reads on 2026-10-15 over `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the dirty working tree confirm the mechanisms below. **Proposed:** all changes, inquiry outcomes and future verification. Authoring ran no product checks, services or benefit measurements. The selected review standard is Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 through the pse-arrow binding. This plan updates no existing plan status or historical audit outcome.

## Owned lifetime contracts

### Hierarchical private cancellation and study bookkeeping — F06

`CancelSource::child_with_deadline` currently computes the minimum original/enclosing deadline but substitutes a fresh independent token. Make it a true hierarchical child through the existing cancellation abstraction: parent cancellation reaches descendants; a child cannot cancel its parent or siblings; an already-cancelled parent yields a cancelled child. Keep effective caller clocks immutable and minimum-composed. Sharing the parent's exact token would violate child isolation.

Migrate all production child sites together: checked selection, engagement, staged bounded work, candidate-study admission and ephemeral-study coordination. Direct private jobs signal interruption and await actual completion. Preserve each site's typed interruption classification when cancellation/deadline coincide; neither a successful test body nor a cancelled future proves native drain.

The ephemeral study's parent handling must change in the same packet. Its current guards use `!work_cancel.is_cancelled()` to both signal cancellation and repopulate the policy frontier. With hierarchy, that token is already cancelled before the guard executes. Track whether the parent event's bookkeeping has been handled independently from child token state. Apply the frontier refresh exactly once, including pending blocked/waiting points, then poll/drain every issued future and release staged owners through their actual cleanup route. Do not blindly remove the parent bridge from sites that still own classification or join work.

Shared flights retain a separate producer cancellation owner. `pse-columnar::Flights::load_owned` already counts waiters, cancels production when the last leaves and keeps retiring identity until completion. A private child must never become that producer token. One caller's cancellation may release its interest while another receives the shared immutable product. Charges and foreign destruction remain owned until the producer actually joins.

### Independent waiter clocks and shared-producer policy — F07

`plan_frontier_keyed` and `complete_frontier_keyed` correctly share mathematical `BasisKey` identities without caller clocks, but capture the creating caller's deadline for the controlled worker. That producer clock is not authority to expire a later valid waiter.

Select a finite service-owned producer lifetime independent of all caller clocks. Each caller retains its original cancellation/deadline around waiting, and removing the last waiter retires the producer. The policy starts at a declared producer/admission boundary and identifies what its elapsed allowance covers. Producer expiry is a typed producer-policy outcome; it must not be misreported as cancellation or deadline of a still-valid caller. No scientific/content key gains a deadline, diagnostic identity or scheduling policy.

B0 must settle the precise bound, configuration owner and expiry classification before B2. Current `MathPolicy::admission_wait` is explicitly admission-only (30-second default), not an execution timeout; silently reusing it for production would change its contract. Inspect current preparation limits and real consuming deadlines, select a finite consumed policy without imposing unrelated solver limits, and record its default and override behavior. This is required bounded design work, not an arbitrary constant chosen in this document.

If no service-owned policy fits the consumed operation contract, B0 compares two alternatives: execution-clock-compatible flight scheduling separate from pure artifact identity, or a typed producer-expiry retry after the old producer has joined, within the unchanged caller deadline. The first reduces sharing; the second adds retry obligations and must prevent hidden retry multiplication. Record a concrete choice and update affected callers before proceeding. A new rule impact follows the coordinator's operator-decision route. Never renew a caller clock, launch a replacement while old work is active, or turn transient cancellation/refusal into a semantic memo result.

### Cancellable compiler admission before active dispatch — F08

`plan_frontier_controlled` and direct function/parametric/observation preparation enter `job_retained_scoped` before `with_modeling_compiler` waits on the workspace's synchronous mutex. A cancelled waiter can therefore occupy an active native/CPU worker while waiting for unrelated compilation to release exclusion.

The workspace owns cancellable exclusive preparation admission before active dispatch. Transfer the exclusive authority into the worker that mutates the compiler; it must remain exclusive through head comparison, any publication and actual use. Queueing may retain its real request/workspace ownership and bounded population admission, but cannot consume scarce active execution capacity merely to wait for that mutex. If dispatch or cancellation wins after admission, release the permit only when no issued worker still uses it. Producer cancellation remains separate from waiter cancellation for shared work.

Preserve one writer, coherent candidate publication, complete lineage/context, monotonic live head and detached workspaces for historical/branch revisions. An unlocked head precheck followed by later acquisition is not atomic selection/use. A queued historical request must acquire/select its actual authority safely; it cannot rewind the live lane or become an uncharged alternate writer. Detached allocation, poisoned-owner failures and failed publication retain their current resource/infrastructure meanings.

B0 traces direct and keyed preparation, revision admission/publication, workspace creation and detached fallback, including lock order against worker/retention owners. Choose the smallest transfer-safe exclusion boundary supported by existing owners. Async acquisition inside an already charged worker is insufficient; a broad actor/scheduler or concurrent writable Salsa clone is not selected. Pure operations over independently owned immutable products should avoid mutable workspace admission where their actual compiler contract permits, without fabricating a detached product or parallel preparation algorithm.

### Cross-owner idle retention pressure — O07

Existing math reservations already trim idle compiler/native state. `trim_compiler_lanes` preserves borrowed workspaces; the two observed compiler leases were real owners, not proven duplicates. Result readers reserve through other pool paths. First consume [41c](41c-studies-response-artifacts-and-result-resources.md) C3's implemented exact metadata/SDK ownership correction. Then establish whether supported nonempty result reads still contend with genuinely optional idle retention in the actual shared pool.

If that remaining contention exists, coordinate the existing retention/resource owners so a requesting consumer can ask genuinely idle optional holders to release, then retry a bounded reservation through the established accounting path. Decide lock order, retry scope and reentrancy before implementation; no pressure callback may deadlock on a borrower or wait for native destruction under a shared pool lock. Active borrowers, scientific product owners and requests still draining SDK/native work cannot be reclaimed. Useful warm reuse survives until real pressure or explicit owner policy requires release. Dropping an idle handle is not proof every shared allocation was released.

A smaller opaque compiler envelope is a distinct alternative requiring complete accounting knowledge, not sampled RSS or a reservation name. Mark its unobserved extent unknown and preserve a sound bound. If C3 removes the demonstrated problem and no residual supported-work conflict is established, close O07 with a reasoned no-adoption outcome. Do not introduce a manager, increase caps or delete charges merely to satisfy a test.

## Bounded decisions and work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B0 | Source review and existing owner contracts | Settle child cause/bookkeeping behavior, the finite shared-producer policy or justified alternative, and transfer-safe pre-dispatch compiler admission/lock order. Record defaults, expiry meaning, actual affected callers and any new rule-decision prerequisite. B1's already-settled hierarchy slice can proceed independently of producer-policy choice. | Completed — independent 600-second preparation policy and transferred compiler authority selected |
| B1 | B0 child/bookkeeping slice; existing hierarchical cancellation capability | Change child propagation and all production callers, including the simultaneous study frontier handling correction. Test public selection/specialization cancellation, already-cancelled parents, sibling/upward isolation, minimum deadlines, blocked study points and issued-work drain. Remove redundant signal-only bridges only after their classification/bookkeeping consumers have moved. | Completed — public selection/specialization, child isolation and blocked-study drain controls passed |
| B2 | B0 producer-policy decision; B1 for public composed controls | Implement independent immutable waiter clocks and the selected shared-producer lifetime in both frontier/completion flights. Remove first-waiter deadline capture. Exercise short-A/long-B, reversed arrival, explicit caller cancellation, expiry, all leaving and joined retirement; prevent duplicate active replacement and preserve cache/refusal semantics. | Completed — independent caller/producer clocks and compiled Python setting boundary tested |
| B3 | B0 exclusion/lock-order decision; B1 for caller cancellation composition | Move exclusive compiler admission before active job dispatch across keyed and direct preparation/publication consumers. Carry authority through head use and detached handling. A gated A/B/C control must show cancelled/expired B leaves without A finishing or B acquiring an active worker, while A and later coherent head use remain valid. Remove late blocking exclusion paths for migrated work. | Completed — pre-dispatch compiler authority, actual join retention and coherent head/private routes tested |
| B4 | Implemented 41c C3; retained-state/borrower evidence | Settle residual pressure need and select existing-owner coordination only if demonstrated. If adopted, integrate actual result and math consumers, bounded retries, lock order and idle-only release; test borrowed versus idle owners, cancellation suffixes and concurrent requests. Otherwise record supported no-adoption reasoning and its limits. | Planned |
| B5 | B1–B3 actual migrations and targeted controls; B4 investigated and dispositioned | Complete local public/flight/compiler composition and replacement deletion; hand evidence and support limits to 41f. Check no caller owns another's clock and no request releases active foreign resources merely because its waiter ended. | Planned |

B2 and B3 are independently deliverable after their B0 slices; neither depends on all source preparation corrections in 41a. B4 follows actual C3 delivery, not an agreed metadata interface, and is not a prerequisite for unrelated numerical reuse. The shared `math/modeling`, `math/preparation`, job and study coordination surfaces require explicit editing ownership when parallel work starts. A declared interface does not satisfy a migration or drain prerequisite.

[41f](41f-causal-diagnostics-qualification-and-closeout.md) F0 owns any needed diagnostic schema/meaning slice, and F1 producer work participates in B1–B4. Publish actual caller/producer relationships, queue/active boundaries, stop-request/checkpoint/join stages and idle-retirement outcomes from existing safe owners. Shared work is counted once, waiter latency separately; fresh observations do not prove cancellation drain or invent an exclusive-time profile. No new evaluation, blocking collector or scientific key change is permitted.

## Verification and completion

**Proposed targeted verification:** compile affected packages through `just check-package`; use `just unit-package` controls with real owner boundaries and deterministic gates rather than timing races. Pure token, flight, frontier and compiler admission controls require no unrelated canonical store. Actual Arrow-consuming closures retain explicit force-validation. Use real native destruction only in the smallest owned test that settles join/resource release, and select necessary managed/store composition at 41f scope end.

For F06, cancellation must reach real public checked selection and finite specialization, not only a managed receiver. Cover deadline/parent coincidence with preserved typed cause, cancellation before child creation, child cancellation with live siblings, and a study frontier containing blocked points that must be reclassified exactly once. Assert all issued futures are observed/drained and capacity stays owned until actual completion.

For F07, attach two same-key waiters before releasing a gated producer. Let the first expire while the second remains valid; the second obtains the product or an independently justified producer-policy refusal, never the first caller's deadline as its own outcome. Reverse arrival/deadline ordering and cancel explicitly. When all leave, keep producer identity/resources until exit; a new request cannot overlap a retiring producer. Retain existing successful-result reuse and failure-retry controls.

For F08, hold the workspace with A, queue distinct-key B in the same lane, and cancel or expire B. B settles without releasing A and without taking an active CPU/native worker. Include cancellation after exclusive admission but before dispatch, failure after dispatch, live-head publication, historical/branch selection, failed candidate publication and subsequent C use. Preserve exactly one coherent writer and correct detached charges.

For O07, consume nonempty metadata paging after C3, distinguish real live metadata/decode from idle retained lanes, and test inability to reclaim borrowed state. Do not use an empty-demand read that bypasses the disputed operation. A reservation or cleanup failure keeps its typed scope; raising capacity is not evidence of the ownership correction.

Review S02/S04/S10 are the main scenario routes, with S01 preserving revision/value behavior. B1 must precede long dependent simulation reruns; targeted gates can establish behavior without a cold statistical campaign. 41f owns assembled qualification, scope-end hygiene/manual checks and final diagnostic consumer acceptance. Native numerical correctness and original-space checks remain required, while environment/provenance proof and hard comparative performance criteria are excluded permanently.

The native table and Outcome own current delivery. B4 and its dependent B5 remain outside the current A/B enabling execution; B4 still requires implemented C3 and actual residual pressure evidence. Enduring lifetime contracts are recorded at blueprint §14.3. Older plan bodies and statuses retain their existing ownership.

## Outcome — 2026-10-15

**Implemented:** B0 chose one consumed finite preparation policy at `MathPolicy`, surfaced
through the authoritative settings declaration as `math_preparation_limit_ms`. Omission
resolves once at each composite entry; supplied deadlines remain authoritative. Private
children inherit parent stopping and the minimum clock, while child stopping remains
isolated. Study parent bookkeeping has its own once-handled state, independent of a child
already receiving the signal. The obsolete signal-only study-candidate bridge is removed.

Both keyed frontier/completion producers and selected-admission production have finite
service-owned clock origins independent of waiters. Cache hits check the original caller
clock; no automatic retry renews that clock or overlaps a retiring producer. Typed producer
expiry survives shared error ownership and converts to native Time. Canonical caller expiry
also stays Time, and optional root-response classification cannot relabel it Memory or a
scientific neighborhood failure. Pure source admission retains actual caller cancellation
until effect issuance; acknowledged canonical publication retains its original outcome rule.

Compiler work owns one bounded population entry before awaiting cancellable workspace
exclusion, then transfers that same entry into active dispatch and native join. The exclusive
permit and workspace storage stay with the actual join owner, including thread-local teardown.
Private historical/branch preparation releases live authority before charged detached use;
head comparison/publication/use remain under one writer. Source admission is asynchronous
through this owner; raw late blocking compiler lock paths and the old synchronous production
revision route are removed. Direct functions/views/observations/parametric, initialization,
flow and source consumers use the owner, with artifact waits and rebind phases retaining the
same caller clock. No new scheduler, population ticket, host cap or generic resource manager
was introduced.

### Verification and limits

**Tested — native-local composed controls, zero-failure target:** the first accepted-source
batch ran 30 tests: 28 passed and two fixture failures. The complete command below used the
pinned development profile, `CARGO_BUILD_JOBS=2`, explicit Arrow force-validation from the
scope-aware route and the selected resource registry. Compilation was warning-free.

```bash
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw CARGO_BUILD_JOBS=2 \
scripts/pse-env --native=solver,klu,isolation,uno,petsc -- just unit-package pse-runtime \
 'test(compiler_queue_) | test(compiler_authority_) | test(direct_function_preparation_) | test(shared_frontier_and_completion_) | test(shared_preparation_producer_) | test(shared_preparation_expiry_) | test(math::modeling::lane_tests) | test(preparation_limit_preserves_) | test(cancel::tests) | test(indexed_imports_) | test(scoped_local_selection_) | test(held_preparation_reuse_) | test(selected_observation_cache_) | test(parent_cancellation_refreshes_) | test(producer_preparation_expiry_) | test(scoped_deadline_signals_native_worker_) | test(scoped_cancellation_before_deadline_) | test(closed_scope_releases_engagement_) | test(rebind_uses_earliest_driver_clock) | test(public_preparation_parent_cancellation_)' \
 --features native-solvers --execution-effects native-local
```

The existing terminal result is
`/tmp/pse-friction-review-resources-uMrtOxGw/invocations/e06cd8b286984da29cd04c62f54ccd7b.terminal.json`.
All new hierarchy/public specialization/blocked-study, unequal-clock arrival orders,
last-waiter retirement/no duplicate, independent producer expiry/retry, compiler queue
population/cancellation/cutoff/dispatch refusal, actual TLS join, post-dispatch omission
expiry, caller isolation and earliest-clock rebind controls passed. The live-head/branch
and rejection controls passed; the legacy idle-pressure fixture and the new observation
fixture failed. Both were corrected and accepted by the [41a consumer reruns](41a-source-specialization-and-selected-preparation.md#verification-and-limits),
which also exercise the final canonical Time and root-response classification corrections.
No unresolved test failure remains in these selections. Counts overlap and do not form
whole-series qualification.

**Tested — compile:** native runtime/PyO3 all-target check passed via
`scripts/pse-env --native=solver,klu,isolation,uno,petsc -- just check-package pse-runtime
-p pse-py --features pse-runtime/native-solvers,pse-py/native-solvers`; two initial warnings
were corrected before the warning-free unit build. The final default-feature runtime/PyO3/
benchmark all-target check and native benchmark check are recorded at 41a. Full code generation
passed there. `just py-sync-correctness native` rebuilt and installed the force-validated native extension
and regenerated `python/pse/_native.pyi` from its compiled API, without warnings. Four Python
default/positive override and pure/native conformance-forwarding controls passed, zero failures:

```bash
pse_test_tmp=$(mktemp -d /tmp/pse41-python-settings-XXXXXX)
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
scripts/pse-env --native=solver,klu,isolation,uno,petsc -- .venv/bin/python -m pytest \
 python/pse/tests/test_native_boundary_contracts.py -m unit \
 -k 'preparation_producer_allowance or conformance_forwards_preparation' -q -n 0 \
 --basetemp "$pse_test_tmp"
```

The first Python invocation stopped before collection because the shared
`/tmp/pytest-of-paul` directory belongs to another user. Its contents and ownership were
left intact. `just ready` passed, and selecting this run's fresh owned temporary directory
allowed all four assertions to run. No substrate or absent scientific asset was acquired.
B0–B3 are complete; B4/B5 and full 41f acceptance remain open.

**Interface-checked:** independent review exposed missing clocks after model preparation,
a warm held-model hit, value rebind and initialization artifact waits. These are corrected
at their consuming entry and the same clock is carried through their phases. The actual
function preparation dispatch gate confirms stop signalling at expiry while the caller
awaits native join, the workspace/population/bytes remain held and its own token stays live.
Final readback found no remaining material A/B finding in its inspected scope. General
artifact-producer lifetime policy, B4 idle-pressure coordination and full-series assembled/
managed/store/Python qualification are outside this enabling delivery; direct artifact
waits are bounded without importing a new cache policy.

### A mistake made and corrected

Initially the compiler permit was held only by the worker closure, which could release
before thread-local destruction. It now belongs to the existing native join owner, and an
actual TLS destructor test observes that retention. Review also found downstream preparation
phases forwarding an omitted clock; composite/direct entries and shared artifact waits now
retain one finite clock, with explicit typed expiry tests.

The prior 64 MiB idle-pressure fixture admitted source synchronously. Its replacement uses
real stack/session/foreign, source-candidate and historical workspace owners, which cannot
all fit there. The test pool is now 128 MiB, still fills its actual capacity and still must
evict the idle lane to satisfy the snapshot demand. Production allowances and pressure
assertions were preserved. Other fixture/identity corrections are recorded at 41a.

### Deviations made deliberately

No required B1–B3 correction was dropped. The preparation policy was named independently
of `admission_wait`; no caller clock enters scientific identity. Review-driven clock
normalization extends through the actual selected preparation consumers so an omitted
clock is not reintroduced after a phase boundary. B4 remains dependent on C3's actual
metadata correction and evidence, and B5 remains open with it. No legacy cold campaign,
statistical timing criterion or eliminated environment/provenance proof was introduced.
