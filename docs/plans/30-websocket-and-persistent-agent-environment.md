---
title: WebSocket RPC and the persistent agent environment
status: draft
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
of these documents after confirming the rule changes below. Production execution is
not authorized by this authoring handoff.

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
| [WP01](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp01) | S01/S03 | scheduled | 30a A0–A3 | Original-clock, expired replay, uncertain-send/commit and drain controls; no implementation evidence yet. |
| [WP02](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp02) | S01/S06 | scheduled | 30a A1/A2, 30d D2 | Bounded pending/deferred/replay state and native paged result admission, including decoded memory. |
| [WP03](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp03) | S02 | scheduled | 30c C1/C2, 30b B2 | Overlapping Rust/Python tests reuse names without interference; deliberate sharing remains explicit. |
| [WP04](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp04) | S05/S09 | scheduled | 30c C1–C3 | Assertion-after-Drop, interrupted/missing results, retained references and resumable disposal. |
| [WP05](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp05) | S02/S04 | scheduled | 30b B0–B3 | Compatible receiver coexistence, incompatible generation refusal and storage lifetime independence. |
| [WP06](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp06) | S07 | scheduled | 30c C2, 30d D1 | Owner loss/PID reuse with surviving descendants; capacity recovery preserves evidence pins. |
| [WP07](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp07) | S06 | scheduled | 30d D0–D3 | Cross-invocation limits, nested ownership and admission through actual drain. |
| [WP08](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#wp08) | S04/S06 | scheduled | 30d D2/D3 | Explicit residency, contention and process-lifetime attribution; no benefit claimed yet. |
| [AE-25](29-agent-workspace-effectiveness.md#latency-and-concurrency-s1-s5-s7) | Plan 29 S5/S7; review S02 | scheduled | 30c C1/C2 | Transfer the open isolation follow-up from 28e; original observation remains historical. |
| [AE-26](29-agent-workspace-effectiveness.md#latency-and-concurrency-s1-s5-s7) | Plan 29 S5; review S02/S06 | scheduled | 30b B3 | Compare current shared-service behavior only if supported contention triggers the bounded topology investigation; old DDL diagnosis is not current proof. |

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
