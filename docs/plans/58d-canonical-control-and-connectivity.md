---
title: "Plan 58d: Canonical execution, schema evolution and connectivity"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [storage, design]
---

# Plan 58d: Canonical execution, schema evolution and connectivity

## Purpose and basis

**Proposed:** implement O07 and the control portion of O09 from the
[principal functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md#O07).
A Rust operator can browse durable work, submit a complete authorized control intent and
reconcile its original outcome using connection, authentication and bounds alone. Reading
headers or requesting stop must not construct mathematical services or prepare a provider.
The [coordinator](58-rust-owned-system.md) owns shared rule decisions, sequencing and overall
acceptance; this document owns D01–D05, including the durable-model and migration foundation selected from improvement-review I02/I03. Creating this draft does not authorize implementation.

The amendment baseline is `c3b7f136d`, on 2026-10-28; production source remains at the reviewed `bd1c4891e` basis. The [improvement review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I02) supplies I02/I03; adoption is planned work, not implemented behavior.
Relevant current contracts are blueprint §20.6 and §23; the selected assessment standard is
Core 3.4 with its efficient-architecture companion and Process Simulator 1.5. Existing
scientific admission, stop intent and physical release remain separate responsibilities.

## Existing foundations and target

`crates/pse-operations/src/canonical_control.rs` already declares bounded browse/cancel/
reconcile documents and `CanonicalStore` operations. Its native cancel path invokes generated
`fn::pse_control_v1` policy as `native-system`; this is privileged worker behavior, not the
record-actor client contract. `deployment.rs::CanonicalConnectionSettings` and
`ServiceIdentity` already separate connection-owned configuration from native execution.
`canonical_client.rs::Client` owns the bounded loopback custom gRPC connection and request
settlement. The Rust SDK is an alternative to assess, not an existing interchangeable owner.

Retain the supervised source-built SurrealDB 3.3.4 with MX 1.0.0, explicit SSI and synchronous
AOL/every-append durability. `scripts/surreal_source.py`, `mx-1.0-controls.patch`,
`canonical-profile.patch` and `api-failure-rollback.patch` in `scripts/surrealdb-source/`
identify the effective server. Stock release defaults and engine-overview labels do not.
Compact client control and privileged scientific workers share canonical command policy;
they do not share administrative credentials or imply equal authority.

The target adds a narrow typed native entry point over this foundation. Direct callers pass
Rust domain requests and receive typed outcomes; external serialization belongs only at the
selected ingress. Do not carry Python JSON/document round trips into in-process calls.
Keep existing constrained `DEFINE API` delivery for actor-facing commands where it supplies
the complete contract, and existing bounded gRPC for workers where useful. Compare those
with exact-release SDK use before adding a protocol. No new Tonic/Prost service, Arrow Flight
control plane, registry, crate or universal connection abstraction is selected by this plan.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [D01](#d01) | planned | Rust operations and connection owners | Control construction and direct typed calls require no mathematical Runtime. |
| [D02](#d02) | planned | Canonical command policy and ingress owners | Actor-bound commands and original receipt reconciliation agree across selected ingress. |
| [D03](#d03) | planned | Canonical study policy and operations owners | Durable model governs bounded progression, preparation handoff, claim and later lineage queries. |
| [D04](#d04) | planned | Control consumer and integration owners | Retained native consumers migrate; replaced frontend orchestration is deleted; H has assembled cases. |
| [D05](#d05) | planned | Canonical schema and maintenance owners | Native ordered migration, version applicability and write fencing work, including open-handle/in-flight semantics. |

<a id="d01"></a>
## D01 — Connection-only native control

Consume [T01's decision route](58-rust-owned-system.md#t01) and
[T02's operation inventory/contracts](58-rust-owned-system.md#t02) before fixing public
shapes. Extend operations/deployment owners so an authenticated native caller
can select its canonical issuer, bounds and declared browse/cancel grants without creating
`workflow::Runtime`, `SourceServices`, compiler preparation, math or solver services.
Reuse connection admission, deadlines and close/drain ownership. Provisioning actors and
installing schema remain explicitly privileged operations, outside ordinary client startup.

For example, an operator opens a selected deployment and requests one compact page of study
headers. The result carries exact cursor predicates/issuer and live-browse meaning; it does
not hydrate payloads or claim a frozen analysis population. A wrong issuer, disabled actor,
missing grant or invalid bound refuses before protected state access. Configuration failure
does not trigger scientific initialization as a fallback. Narrow existing constructors and
move callers together; remove any replaced control-only Runtime construction path.

Targeted controls cover browse with mathematical initialization unavailable, strict selected
configuration, wrong-store continuation, close during an admitted request and independent
caller admission. Compile the affected operations/runtime packages through their scoped
routes; correctness closures retain explicit force-validation where Arrow is consumed.

<a id="d02"></a>
## D02 — Complete commands and truthful reconciliation

Build on D01's usable connection/actor contract. Keep one generated canonical policy owner in
`crates/pse-codegen/src/codegen/surreal_control.rs`, consumed by native operations and the
selected constrained API. An immutable command binds the complete intent to issuing store,
authenticated actor and exact issued operation identity. Identical repetition observes its
original receipt; changed intent under that identity refuses. Restore changes issuer.

The revealing failure is a cancel request whose server effect commits but acknowledgement
is lost. Retain the issued request and reconcile that tuple; receipt absence is unobserved,
not rollback or permission to issue a fresh effect. Preserve original uncertainty and readback
failure rather than returning a guessed success/refusal. A receipt records stop intent,
while the worker/native/host owner establishes actual joining and release. Long scientific
preparation remains outside short revision/fence-checked transactions.

Ingress keeps grant checks before parsing/state/receipt access, raw-byte admission before
strict UTF-8/JSON decoding, opaque string identity and exact integers. API-body actor checks
remain necessary even with endpoint `PERMISSIONS`. Preserve the patched rollback behavior
for evaluation/conversion errors; an intentionally returned refusal response has its stated
transaction meaning. Remove independently maintained frontend retry/transition decisions.
Targeted controls exercise identical/reused intent, actor denial, full-u64 boundaries, stale
fence, lost ACK, missing receipt, failed readback, restore issuer and API rollback/refusal.

<a id="d03"></a>
## D03 — Make canonical state govern durable execution

Definitions/revisions, requested work, dependencies, realized points, assignments, attempts,
state transitions and result references form one queryable execution model. A05 owns native
scientific recording/submission callers and point interpretation; this package owns canonical
submission and transition policy. C owns actual preparation/resource admission and native joins;
B owns database process mechanics; E owns protected payloads. Pure local computation stays direct.

Extend the existing generator `pse-codegen/src/codegen/surreal_studies.rs`, constrained
`pse-study-module` and shared `pse-model::study_policy`. Existing `readiness_scope_page`,
readiness apply, admission, claim, settlement and conclusion are foundations to compose, not
proof the client loops are necessary. `workflow/worker.rs::work_once_scoped` currently pages
studies, advances readiness and fetches candidates; `work_study_candidate` fetches scope,
evaluates policy, prepares and reevaluates before claim. Replace repeated client interpretation
with bounded store-side progression that returns the applicable domain action and exact durable
premises. Reuse domain types; no universal next-action catalogue or new workflow engine.

Prefer ordinary SurrealQL select/recheck/transition composition; retain the readiness module
where shared bounded Rust policy earns its boundary. Preparation and resource admission precede
execution claim. Expensive preparation/solver work stays outside transactions; claim rechecks
prepared seed evidence, dependency revisions, generation and cancellation in the current canonical
snapshot. Invalidated preparation cannot grant a stale claim. A bounded examined page with
continuation is distinct from exhaustive no work; workers advance continuation rather than
mistaking an unproductive page for global idle. Original operation identity reconciles uncertain
acknowledgement. Notifications only wake workers; canonical catch-up handles loss and replacement.

Preserve existing dependency-installation limits: add edges before activation/cancellation,
not live graph mutation. Record realized points and scientifically useful observations at useful
analysis granularity, with actual attempt/result lineage. Do not persist native pointers, task
handles or every implementation event. Later repeated analysis queries this same history.

Before extending Surrealism, check exact 3.3.4 permissions, controller limits, complete outputs,
rollback and intentional-refusal semantics against [guest imports](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealism/src/imports.rs)
and [WIT ABI](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealism/wit/surrealism.wit).
`sql_with_vars`, `run` and writeability metadata are candidates, not authority already granted.
Current readiness forbids arbitrary queries, network and KV; module-local volatile KV does not
share record-transaction rollback. Native science and host release stay outside modules.

Targeted controls cover two competing workers, stale prepared evidence, cancellation before
claim, a bounded empty eligible page with continuation, dependency edits refused after activation,
and replacement worker selection followed by a lineage query. These tests challenge actual
canonical operations, not a second scheduler. Database speedup remains unmeasured.

Embedded deployment, SurrealKV history/checkpoints, payload-format migration and progressive
transport remain conditional on actual consumers. Compare isolation/durability and integration
cost if triggered; none substitutes for D05's required schema evolution. Keep patched MX selected.

<a id="d05"></a>
## D05 — Deliver minimal native schema evolution and write admission

Consume T01's decision authority and the registry-owned schema declaration. Deliver native
ordered predecessor/successor transformations and their application owner during the cutover;
this is required infrastructure, not a deferred interface. Reuse existing generation and
canonical maintenance facilities before selecting a migration library. Do not duplicate schema
meaning or revive the full SurrealKit beta/operator experiment merely to supply migrations.

Construction establishes supported schema/version and issuer premises once and returns the
appropriate typed capability. Dynamic admission/version changes remain real external uncertainty.
Do not repeat complete schema proofs on ordinary operations, and do not treat a once-opened
handle as permanently authorized to write across a migration.

Resolve the open-handle/in-flight contract before dependent implementation: inspect current
`canonical.rs::WriteAdmission`, maintenance and publication paths; establish how new claims
and writes close, how original in-flight attempts settle and physically drain under the old
interpretation, and how stale handles are fenced before successor activation. Favor a quiescent
migration boundary; do not transform schema beneath active scientific writers. A migration
that cannot establish that boundary must remain pending with admission closed. Specify the
permitted drain/publication operations separately from newly refused mutations, version/fence
checks at actual writes, and successor capability reconstruction. Carry this bounded decision
and its targeted race tests through T01 before implementation; no claim of a working fence follows
from current configuration-file checks alone.

Declare the applicable predecessor, close admission, apply ordered transformations, and publish
the successor version only after the transformation succeeds. Interrupted application leaves
explicit pending/closed admission; restart cannot silently reopen incompatible writers or change
original effect identity. A representative adjacent-version disposable dataset demonstrates
correct transformation, wrong-predecessor refusal, interruption and stale-handle/in-flight
behavior. Publication and read compatibility are stated per supported schema, not inferred from
an SDK connection or successful process start.

Owned regenerable development/test deployments may reconstruct from authored inputs/reference
material. Placement is disposable only when established at creation; no reset of existing data
is authorized. Uniquely authored inputs and independent evidence remain protected. Full supported
historical migration, backup/restore and disaster recovery belong to 58h's persistent-deployment
release obligation before non-regenerable/shared operational data, with D's schema owner and B's
service owner supplying that later evidence. Minimal infrastructure above still blocks D completion.

<a id="d04"></a>
## D04 — Migrate control consumers and hand off acceptance

Move retained native study recording/inspection, cancellation and original-command reconciliation callers to D01/D02/D03, with schema applicability from D05,
coordinating Python HTTP client retirement with its owning frontend package. Delete replaced
orchestration, caller-owned retry policy and tests/fixtures for retired machinery in the same
integrated cutover. Include callers, configuration, generators, targeted tests and deletion together; temporary development incompleteness earns no acceptance and needs no compatibility bridge. Preserve tests of retained durable guarantees at their Rust owner. A CLI
is an optional caller; PyO3 remains deferred and MCP is conditional under the coordinator.

**Proposed verification:** targeted owner controls above establish local behavior, with zero
failures as the baseline. [H](58h-qualification-and-publication.md#h01) owns assembled
acceptance of authenticated direct/native and
retained published ingress, restart/lost-ACK reconciliation, denied actors, missed notification
catch-up, durable worker replacement/later lineage, D05 migration and stop-versus-physical-drain. H selects integrated tests and scope-end static checks
once functional scope is complete; protocol examples or local controls do not qualify science.
No tests, builds or production changes ran for this draft; no speedup is claimed.

For release-specific research use the principal review's library appendix: SurrealDB tooling
schema/API request tests are integration aids, not scientific schema authority. Hyper 1.11.1
and Tonic 0.14.6 are the current pins, versus overview 1.12.0/0.14.8; the newer Tonic explicit
cancellation-handle API is not assumed. SDK/protocol selection must preserve request custody,
deadlines and drain rather than claim those duties from transport shutdown alone.
