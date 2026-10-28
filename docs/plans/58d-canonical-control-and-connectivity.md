---
title: "Plan 58d: Canonical control and connectivity"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [storage, design]
---

# Plan 58d: Canonical control and connectivity

## Purpose and basis

**Proposed:** implement O07 and the control portion of O09 from the
[principal functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md#O07).
A Rust operator can browse durable work, submit a complete authorized control intent and
reconcile its original outcome using connection, authentication and bounds alone. Reading
headers or requesting stop must not construct mathematical services or prepare a provider.
The [coordinator](58-rust-owned-system.md) owns shared rule decisions, sequencing and overall
acceptance; this document owns D01–D04. Creating this draft does not authorize implementation.

The inspected source baseline is `0659343f123fefa2371ac7d1659e3af7f75f9949`, on 2026-10-28.
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
| [D03](#d03) | planned | Database integration owner | Placement questions have bounded decisions without changing canonical deployment by assumption. |
| [D04](#d04) | planned | Control consumer and integration owners | Retained native consumers migrate; replaced frontend orchestration is deleted; H has assembled cases. |

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
## D03 — Bound database placement questions

Retain the constrained readiness module in `crates/pse-study-module/`. Assess expansion only
for a named data-local decision whose complete input/output, transaction, permission and
resource contract improves on the current generated function. `sql_with_vars`, `run`, typed
exports and writeability metadata are candidates from
[Surrealism §4 and §8](../libraries/surrealism.md#4-native-database-and-inter-module-integration),
qualified against [3.3.4 guest imports](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealism/src/imports.rs)
and its [WIT ABI](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealism/wit/surrealism.wit).
Current readiness forbids arbitrary queries, network and KV; retain its source-build controller,
memory and deadline restrictions. Module-local volatile KV is not record-transaction rollback.
Neither native solvers nor host release moves into modules as part of this packet.

Embedded deployment, SurrealKV `TreeBuilder`/history/checkpoint use, payload migration and
progressive `stream_items` delivery require a concrete consumer or operational problem first.
If triggered, compare retained MX SSI and backup/recovery against the candidate's actual
isolation, durability, adapter and migration cost. Otherwise record non-selection here and
continue D04. A transport candidate must demonstrate actor policy and original ACK settlement;
notifications remain hints with authoritative catch-up after reconnect or lag.
Any selected module expansion adds targeted transaction-error/intentional-refusal, permission,
complete-output and deadline/controller/memory controls before dependent consumers migrate.

<a id="d04"></a>
## D04 — Migrate control consumers and hand off acceptance

Move the retained native study inspection, cancellation and recovery callers to D01/D02,
coordinating Python HTTP client retirement with its owning frontend package. Delete replaced
orchestration, caller-owned retry policy and tests/fixtures for retired machinery in the same
proven replacement change. Preserve tests of durable guarantees at their Rust owner. A CLI
is an optional caller; PyO3 remains deferred and MCP is conditional under the coordinator.

**Proposed verification:** targeted owner controls above establish local behavior, with zero
failures as the baseline. [H](58h-qualification-and-publication.md#h01) owns assembled
acceptance of authenticated direct/native and
retained published ingress, restart/lost-ACK reconciliation, denied actors, missed notification
catch-up and stop-versus-physical-drain. H selects integrated tests and scope-end static checks
once functional scope is complete; protocol examples or local controls do not qualify science.
No tests, builds or production changes ran for this draft; no speedup is claimed.

For release-specific research use the principal review's library appendix: SurrealDB tooling
schema/API request tests are integration aids, not scientific schema authority. Hyper 1.11.1
and Tonic 0.14.6 are the current pins, versus overview 1.12.0/0.14.8; the newer Tonic explicit
cancellation-handle API is not assumed. SDK/protocol selection must preserve request custody,
deadlines and drain rather than claim those duties from transport shutdown alone.
