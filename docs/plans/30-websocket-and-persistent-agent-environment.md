---
title: WebSocket RPC and the persistent agent environment
status: done
date: 2026-10-08
adrs: [ADR-0164, ADR-0166]
review_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#4-revealing-scenarios]
---

# 30: WebSocket RPC and the persistent agent environment

## Purpose and authorization boundary

Provide dependable native WebSocket application RPC and a persistent local working
environment for concurrent Codex sessions. Storage services outlive the session that
borrows them; independently admitted receiver artifacts and isolated mutable test
contexts allow source changes without unnecessary service interruption. Final test
outcomes govern disposal. Shared host admission coordinates builds, functional tests,
native execution and timing work.

The [review](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md)
judged the current architecture **Revise** and its unresolved candidate **Not Accept**.
This series develops that candidate; authoring does not revise the historical verdict,
accept an ADR or establish a deployed replacement. The maintainer authorized creation
of these documents after confirming the rule changes below. The maintainer authorized full production execution on 2026-10-09. The execution
baseline is clean main at `d92fa8ee1110508e50f049bc681765e2903cbd8b`; the earlier
authoring observations retain their original scope.

Plan 30 is the sole current disposition owner for WP01–WP08 and the transferred
AE-25/AE-26 follow-ups. Companions own their package progress. [Plan 28](28-surrealdb-unified-substrate.md)
retains its scientific and substrate findings; [28e](28e-rebuild-retirement-and-qualification.md#preparation-acceptance-and-investigation-handoff)
retains the existing failure-only continuation and broader qualification obligations.
Creating this series does not restart E4/E5 or the former full Python selection.

## Baseline and foundation assessment

**Interface-checked:** dirty main at `2410880efc5ab20e623006fde78e624044d3358d`,
including the implemented Plan 28 I/J and consumer changes. Source and installed
artifact identities remain distinct. The [supporting evidence](../design_review/evidence/websocket-and-persistent-agent-environment-2026-10-08.md)
records exact SurrealDB/SDK 3.3.0 limits and bounded host observations. Those observations
are not a capacity qualification of this proposed environment.

The machine has sixteen physical cores, thirty-two logical CPUs and approximately
188 GiB usable RAM. Existing supervised services already use disk-backed RocksDB.
Several running servers and saved profiles have different owners/generations; none
is implicitly obsolete or available for deletion. A resident service, persistent disk
state and cross-session admission are separate properties.

Useful foundations are the native exact-value codecs, statement completion checks,
guarded canonical publication, immutable acknowledgments, protected readers and
explicit scientific retention; immutable artifact admission and native drain; the
existing supervisor/systemd scopes; and runner selection/terminal reconciliation.
These stay authoritative. Rust owns scientific meaning, class-specific native
libraries execute it, and short database transactions retain its effects.

The necessary corrections are at their composition boundaries:

- A caller timeout currently does not settle all queued or submitted work. The checked
  WS SDK requires a bounded route-lifecycle correction, not merely a URL change.
- Rust fixture Drop and Python fixture exit precede the final test outcome. Disposal
  must consume the runner's existing terminal result after drain.
- Ambient Python database selection and whole-profile receiver readmission obstruct
  independent test/session ownership.
- Per-invocation groups and process caps do not coordinate the host. Fixed capacity
  lanes reuse the environment/supervisor boundary without a general job broker.

AE-26's earlier schema-conflict evidence used a different initializer and resource
profile. Current initialization already retries definite conflicts. Its successful
concurrent study counterexample also survives. Do not schedule a second missing-retry
fix or infer universal server serialization from the older measurement.

## Confirmed rule changes

All items are **Accept, 2026-10-08**. RC01/RC02/RC03/RC05 were explicitly confirmed
during plan preparation; RC04/RC06 carry forward the earlier explicit successful-data
cleanup and failure/incomplete pinning decision. These are operator decisions, not
accepted ADR status. R0 records the required decision/design route before dependent work.

| Review item | Accepted consequence | Route and dependent work |
|---|---|---|
| [RC01](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#rc01) | Replace gRPC application RPC with native binary WebSocket. Supported provisioning/control-plane operations remain explicit. | Amend proposed ADR-0164 and blueprint §20.6; 30a owns the working transport and all application consumers. |
| [RC02](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#rc02) | Retain the exact primary/observer reference lane while supporting separately bounded functional receiver generations. | Amend proposed ADR-0166 and its deployment explanation; 30b/30d consume 28h's preserved reference identity and allocation. |
| [RC03](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#rc03) | Replace independent ceilings as the admission policy with coordinated host admission and reserved timing capacity. | Environment/tooling policy and blueprint §20.6/§24.1 where governed; 30d replaces the prospective Plan 29 aggregate policy. |
| [RC04](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#rc04) | Final test disposition plus completed drain governs disposal; failures and incomplete outcomes stay pinned. | Test-ownership design amendment to blueprint §24.1 and validation/environment guides; 30c migrates runners and fixtures together. |
| [RC05](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#rc05) | Compatible disk services survive independent worker/receiver readmission. | ADR-0164/ADR-0166 deployment amendments and substrate/environment guides; 30b owns generation separation. |
| [RC06](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#rc06) | Automatically remove only proven successful disposable resources; preserve failure/incomplete pins and referenced evidence. | Prospective fixture/evidence policy through 30c. Existing stores, archives, worktrees and unknown materials retain their current preservation rules. |

If either proposed ADR has become accepted before R0, supersede it rather than editing
its reasoning. Additional governed Python/metadata/commit changes discovered while
implementing these contracts take their required ADR/design route; dependency patching
alone does not require an ADR. Architecture amendments include a blueprint revision.

## Target, shared contracts and document ownership

| Document | Responsibility and boundary |
|---|---|
| [30a — Native WebSocket RPC and operation lifetimes](30a-native-websocket-rpc-and-operation-lifetimes.md) | Native wire integration, original clocks, submission classification, bounded retained work, complete results and effect settlement. |
| [30b — Persistent services and receiver generations](30b-persistent-services-and-receiver-generations.md) | Disk/service generation, authenticated endpoint, interpretation admission, receiver artifact generations and owned lifecycle. |
| [30c — Test ownership and evidence retention](30c-test-ownership-and-evidence-retention.md) | Invocation/test resource association, mutable isolation, final-outcome disposal, recovery and reference-protected evidence release. |
| [30d — Host admission and timing qualification](30d-host-admission-and-timing-qualification.md) | Host capacity lanes, nested admission, cgroup drain, declared timing conditions and series-level assembled acceptance. |

The composition root binds a service generation, immutable receiver generation,
invocation/test context and capacity owner. It passes an already selected handle to
Rust/Python consumers. Scientific APIs retain their existing meaning; consumers stop
independently interpreting ambient configuration or switching a shared connection's
database. Conceptual products need not each become a service, crate or serialized registry.

The contracts exchanged are:

| Product/operation | Sole owning decision | Consumer obligation |
|---|---|---|
| Bound canonical context | 30b selects service/interpretation; 30c selects the owned mutable database; 30a binds the connection immutably | Opening checks compatibility and never installs an unknown schema. |
| RPC operation lifetime | 30a owns original deadline, cancellation, dispatch state and remaining work | A timed-out caller does not release pending work or authorize a mutation retry. |
| Receiver generation | 30b owns immutable executable/configuration association | 28i validity and 28j fresh effects are consumed under actual receiving premises. |
| Invocation resources | 30c associates opaque resource IDs with the runner's selected test identity | A resource declaration is not another test selector or pass/fail authority. |
| Capacity admission | 30d owns the host lane/slot and actual supervised descendants | Nested work borrows the same owner; release follows actual drain. |
| Disposal/release | 30c consumes reconciled terminal outcome, drain and retained references | Success of a scientific operation alone does not establish success of its test. |

Short operation IDs settle storage effects; service/receiver generations establish
compatibility; test invocation identity controls evidence disposition. None substitutes
for a scientific run/attempt identity. Replacing the transport must preserve full-domain
values, native bytes, typed errors, exact coordinates, guards, current attribution,
closed staging, partial outcomes and independently assessed scientific completion.

## Dependencies and implementation order

| Package/boundary | Required delivered capability | What becomes possible |
|---|---|---|
| R0 — Decision and foundation settlement | Accepted rule routes; 30a A0's checked SDK/timeout contract; 30b B0's generation model; 30c C0's runner ownership; 30d D0's concrete profile materialization | Dependent production changes have a specific supported target. Missing SDK or clock premises block the dependent RPC claims, not independent design work. |
| A1/B1/C1/D1 early contracts | Working native RPC/context binding, service selection, durable fixture registration and host admission | Consumers can migrate to the single target with genuine owners, rather than compatibility shims. |
| A2/B2/C2/D2 integration | Complete bounded reads/writes, separately admitted receivers, final-result cleanup and actual overlapping workload placement | Rust/Python, ordinary and managed studies, recovery, readers and agent commands use the same composed lifetimes. |
| A3/B3/C3/D3 scope-end handoff | All affected consumers migrated, replacements deleted, focused controls positive | D3 runs selected assembled qualification once; 28e consumes only applicable evidence at its existing owner. |

R0 is a design/authority package, not permission to claim acceptance of the reviewed
candidate. Resolve material choices in the companion's named prerequisite; retain
supported limits explicitly. If a resulting target materially changes the reviewed
architecture, obtain the binding's bounded review of that change. Do not restart settled
scientific reviews or impose a whole-codebase audit.

Several early contracts can develop independently, but the root owns shared schema,
manifests, configuration identity and integration. The supervisor/environment modules
are shared editing surfaces across B/C/D, not evidence that those responsibilities should
be merged. Necessary caller migration belongs with the change, not a later inventory sweep.

R0 supplies the initial confirmed authority route to A0/B0/C0/D0, then concludes when
their material contract decisions and required design amendments are settled. Their
source/design investigations do not require a working A1/B1/C1/D1 implementation.
B1's service readiness consumes working A1; A1 needs only B0's bound-context contract,
not completed residency. C1/D1 similarly exchange early identity/admission contracts
before later B2/C2/D2 integration. These slices prevent an apparent whole-document cycle.

## Finding dispositions

Definitions and scenarios remain at their source reviews/plans. This table owns current
status; companion package tables do not repeat finding status.

| Finding reference | Scenario reference | Disposition | Decision/work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [WP01](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp01) | S01/S03 | resolved | 30a A0–A3 | Native WS lifecycle and failed-only real-server controls are positive at 30a; D3 selected scientific handoff passed; see Outcome. |
| [WP02](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp02) | S01/S06 | resolved | 30a A1/A2, 30d D2 | Bounded pending/deferred/replay state and native paged result admission, including decoded memory. |
| [WP03](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp03) | S02 | resolved | 30c C1/C2, 30b B2 | Overlapping Rust/Python tests reuse names without interference; deliberate sharing remains explicit. |
| [WP04](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp04) | S05/S09 | resolved | 30c C1–C3 | Assertion-after-Drop, interrupted/missing results, retained references and resumable disposal. |
| [WP05](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp05) | S02/S04 | resolved | 30b B0–B3 | Compatible receiver coexistence, incompatible generation refusal and storage lifetime independence. |
| [WP06](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp06) | S07 | resolved | 30c C2, 30d D1 | Owner loss/PID reuse with surviving descendants; capacity recovery preserves evidence pins. |
| [WP07](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp07) | S06 | resolved | 30d D0–D3 | Cross-invocation limits, nested ownership and admission through actual drain. |
| [WP08](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp08) | S04/S06 | resolved | 30d D2/D3 | Implemented residency/attribution and dedicated timing placement; actual 8+56 GiB control passed. Optional benefit measurement remains unclaimed at 30d. |
| [AE-25](29-agent-workspace-effectiveness.md#latency-and-concurrency-s1-s5-s7) | Plan 29 S5/S7; review S02 | resolved | 30c C1/C2 | Transfer the open isolation follow-up from 28e; original observation remains historical. |
| [AE-26](29-agent-workspace-effectiveness.md#latency-and-concurrency-s1-s5-s7) | Plan 29 S5; review S02/S06 | deferred | 30b B3 | Shared service retained on the selected positive overlap/scientific controls. Reopen B3 only for diagnosed current catalog/commit contention obstructing admitted work within its existing budgets; compare two services at equal allocation/scope. Historical DDL observations do not trigger it. |

S08's domain-extension scenario is a preservation constraint across every package:
adding a model uses existing semantic admission and execution operations; it introduces
no model-specific service, transport or agent scheduler policy.

## Verification and completion

**Proposed:** package-local compile and targeted tests establish each new mechanism and
its refusal cases. Delete its replaced production path, callers and obsolete fixtures
once those checks pass. Correctness tests retain explicit force-validation. The new
application path has no automatic gRPC fallback, dual write or legacy importer.

**Proposed:** D3 owns the selected assembled Rust/Python, managed receiver, interruption,
retained-read and recovery journeys after all functional packages are complete. Run
required scope-end hygiene/manual checks at that point, fix failures and rerun the
failing recipes. This is new-scope qualification; the original Plan 28 Python continuation
still repairs/reruns failed identities and completes interrupted identities only.

Preserve ordinary engineering solver stopping budgets, independent physical/domain/
closure checks and the maintainer's ±10% historical-result assessment. Historical IDAES
values are not authoritative enough to tighten current solver stopping criteria.
Report command, mode, scope, zero failure baseline and actual result. Old receipts keep
their original source/artifact/profile and are not new passes of the replacement.

**Proposed:** timing controls demonstrate correct classification/admission. Optional
speed or memory benefit comparisons require equal scientific scope, explicit residency
and comparable service/profile conditions after positive functional evidence. They do
not trigger the paused E4/E5 campaign automatically.

Completion requires working composed contracts, all affected consumers migrated,
replacements removed, positive selected scope-end evidence and evidence-supported
dispositions. Enduring meaning moves to the architecture and operator guides through
the decision route; retire the completed series/review when no retained reader depends
on it. Native algorithms, remote/distributed deployment, arbitrary-host guarantees and
power-loss durability are outside this series's qualification.

## Authoring checkpoint

The five plan documents are authored and the existing plan routes are reconciled.
All six rule impacts have operator decisions; target mechanisms and acceptance are
Proposed. No SDK patch, service generation, test disposal or host-admission change has
been implemented by this authoring work. Existing services and materials are preserved.

Next, when production scope is authorized, execute R0 with A0/B0/C0/D0. The original
managed failing test remains with 28e; this checkpoint neither closes it nor schedules
another full campaign.

## Implementation checkpoint — 2026-10-09

R0's bounded contract assessment accepts the proposed runtime design in
[the concrete-contract review](../design_review/reviews/design_review_plan-30-runtime-contracts_2026-10-09.md).
The proposed ADRs and architectural owners describe the operator-confirmed contracts;
their decision status remains proposed. The single application RPC path is bounded
native WebSocket. Persistent disk/service generations, separately admitted receiver
artifacts, runner-owned disposable contexts and cross-invocation host admission are
implemented with their focused controls.

Actual controls establish finite kernel placement, overlapping isolated Rust/Python
contexts, final-outcome retention and recovery, independent receiver generations,
restart exhaustion, offline backup/reopen, bounded submitted RPC state and exact paged
reads. The shared compiler cache has its own finite resident allocation. Scope and
conditions remain at the companion owners; these are not full scientific qualification
or uncontended performance claims.

D3 selected scientific qualification is complete. The selected managed Rust launch
first stopped before testing because the captured worker was not bound before reference
admission. That launcher was corrected. Actual testing then exposed a reference-store
affinity/readiness mismatch and an inadequate default libtest thread stack; both are
repaired without changing reference scientific scope or solver stopping budgets. The
next managed attempt reached primary launch and exposed KLU preparation resolving
development metadata from an immutable runtime closure. Prepared-prefix admission
was corrected and its focused controls passed. The ordinary Rust study and six
ordinary Python scientific journeys passed across their selected invocations. One
Python cancellation journey exposed concurrent effect-free settlement against stale
immediate premises; the guarded policy-refresh repair is complete: the original failed cancellation identity passed against the rebuilt extension.
The three previously failed managed Rust identities passed on the retained current native artifacts. The original thousand-point managed Python identity passed under the unchanged reference profile. Scope-end checks completed with failed-leaf repairs. Independent storage restart after launch-caller exit also passed; the Outcome owns final evidence and exclusions.

Existing stores, failed/incomplete resources, receipts and frozen worktrees remain
preserved. Broader Plan 28 E4/E5 and the former full Python selection remain paused.

## Outcome (recorded after implementation)

### What was built

**Implemented:** one native binary WebSocket application path with original operation
clocks, bounded submitted correlations and immutable context selection; separately
admitted disk/service and receiver generations; runner-owned isolated disposable
contexts; final-outcome retention and reference-protected reclamation; finite host
admission and independent persistent storage/compiler-cache allocations. The exact
3.3.0 SDK and engine API are selected by direct paths and exact versions, with their
original license retained. Scientific meaning and native solver stopping budgets
remain with their existing owners. Optional benefit measurements and the wider Plan
28 campaigns have not been started.

**Tested, zero failure baseline:** the companion evidence establishes the focused
transport, generation, retention and actual kernel placement controls. D3 assembled
evidence now includes the ordinary Rust study (1 pass, nextest
`87ec0b50-b2b7-44e9-945e-48afb463d830`), seven ordinary Python scientific journeys
across their original selection and failed-only retries, and the three managed Rust
journeys from retained current artifacts (3 passes in 107.737 seconds, nextest
`1e50d804-f850-4876-8d15-6d862083ddb5`). The Python cancellation retry is
`just native-python build/plan30-d3-python-cancel-current-20261009
python/pse/tests/test_studies.py::test_durable_study_cancel_and_its_refusals -n 0`,
under the native exclusive observer: 1 pass, 0 failures in 16.70 seconds. These are
repaired composite selections; the producer-deployment-specific identity was
legitimately deselected without its separate deployment capture and is not a pass.
The original thousand-point managed Python identity passed: 1 pass, 0 failures in 1,832.49 seconds, with one selected identity and a passed terminal result without report errors. Command: `scripts/pse-env --resource-class reference --native --store -- just native-python build/plan30-d3-python-thousand-current-20261009 --managed-primary-route 'python/pse/tests/test_studies.py::test_flash_sweep_prepares_structure_once[managed-durable-science]' -n 0`. Its XML, native provenance, selected inventory and reconciled terminal result retain that directory. All 1,000 occurrences retain their result and physical assertions; this is no proof that HTTP/2 caused the original failure.

**Tested:** current functional and unchanged-reference service generations passed
explicit process-kill/reopen recovery qualification and native WebSocket readiness.
Actual dedicated timing placement passed with an 8 GiB store and separate 56 GiB
caller, including the actual kernel memory cap, disjoint charged owners and native
WebSocket readiness (`build/plan30-d2-timing-storage-current-20261009/passed.json`).
These are lifecycle/placement observations, not speed comparisons or power-loss
survival claims. The final caller-loss automatic-restart control passed (`build/plan30-b-storage-restart-independent-20261009/passed.json`): the actual loaded/activating restart delay retained its original charged owner through reconciliation after the launch caller died; the new unit invocation reused the same independent 8 GiB storage allocation and returned the original acknowledged bytes. The service remains resident and native WebSocket ready.

**Tested, repaired composite static evidence:** `just hygiene` originally reported six
failed leaves. The failed typos, license, Python lint, type, default Clippy and
no-default Clippy leaves were repaired and passed; its other leaves passed, including
all generated-output equivalence checks. `just governance` ran 102 tests: 101 passed
and one failed. The unchanged failed root override prohibition passed after selecting
the exact SDK paths directly (nextest `742342bd-0255-4310-94d8-5545822a8fe4`).
`just ready`, `just docs`, and the compile-class retries of `just lint-native-contracts`
and `just lint-native-data` passed. The first native lint attempt was wrongly routed
through a 2 GiB light cap and was killed; it is not a positive result.

**Interface-checked:** the scoped feature matrix has 12 valid depth-two cases and five
absence/default cases passing across operations, runtime, Python and xtask. The first
`cargo hack` command also generated 28 invalid cross-package feature requests, which
failed before compilation because its pinned unknown-feature filtering did not apply
to that inclusion list. Those are harness selection errors, not successful checks or
product defects. Supported cases are retained in
`build/plan30-scope-end-powerset-20261009/result.json` and
`build/plan30-scope-end-feature-absence-20261009/result.json`. No whole-workspace
feature powerset, complete simulator qualification or performance campaign is claimed.

### A mistake made and corrected

Treating storage readiness as science-placement readiness caused the actual timing
store to reach its startup deadline. Storage now validates its own allocation, finite
caps, affinity and exact unit/invocation/cgroup/PID binding. A qualified restart retains
that charge through the restart delay, even while the old cgroup is empty. It only
rebinds after the exact predecessor drains; stale or malformed ownership refuses.
The actual service recovery probe also initially assumed the base canonical database
existed and that every provisioning statement returned NULL. Its owned UUID recovery
context now checks the released 3.3.0 DEFINE/USE acknowledgments exactly.

Other assembled failures exposed default libtest stack insufficiency for a large async
publication frame, runtime KLU preparation using development metadata in an immutable
receiver closure, and cancellation settlement using stale predecessor premises.
The launcher uses the declared test stack, prepared KLU prefixes are admitted exactly,
and effect-free settlement refreshes policy only after a definite guarded premise
conflict. Unknown effects do not gain a retry. Failed/incomplete evidence remains pinned.

### Deviations from the plan, deliberate

The setup/replay allowance was settled at 64 bounded entries instead of the initial 16-entry proposal; complete acknowledged selection checkpoints fold only across adjacent compatible setup, retaining intervening command order. Pending and deferred routes remain bounded at 34. Shared functional storage remains selected. The two-service pool investigation keeps
its observable current-contention trigger; historical measurements do not establish
that trigger. Cache lifetime separation was necessary to preserve actual caller drain
and resource attribution. Tests retain the unchanged 160 GiB reference profile and
ordinary engineering stopping budgets. Historical final results use the maintainer's
separate +/-10% assessment; it does not control solver stopping.

Existing stores and failed/incomplete materials are preserved. The two remaining
frozen worktrees contain substantial preserved uncommitted work and are therefore retained. This highest-numbered parent
and its companion handoff remain available while Plan 28/29 readers depend on them;
retirement follows ADR-0096 after those references move. Proposed ADR status has not
been promoted by implementation or test success.
