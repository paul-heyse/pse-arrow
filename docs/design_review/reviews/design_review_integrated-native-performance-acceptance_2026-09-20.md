---
title: Integrated native performance implementation acceptance
date: 2026-09-20
status: in-progress
target: docs/plans/11-integrated-native-performance.md
evidence: Implemented; functional and measurement qualification open
---

# Integrated native performance implementation acceptance

## 1. Decision and scope

This assesses Plan 11 I00–I19 and L01–L18 against the integrated design review and
the Data Model–Based Design Charter. It is implementation acceptance, not a new
architecture proposal. The implementation author is also this reviewer: independent
gate assessment means separate evidence-based judgments for G1–G7, not independent
personnel. The decision remains unresolved while required evidence is incomplete.

Method: inspect the authority and execution seams for registry admission, retained
SessionContext selection, input-dependent reuse, physical ownership, selective
numerical evaluation, exact Delta versions and settlement, and Arrow/Python transfer.
Reconcile V01–V16 and carried Plan 10 cases with executed receipts. Attack changed
dependencies, invalid new values, inactive branches, partial streams, failed commits
and last-reader lifetime through their named controls. A source seal and aggregate
test count do not establish these guarantees on their own.

Scope is this Linux host and its pinned graph. Other operating systems, Python
versions, wheel/sdist packaging and IDAES parity remain outside the campaign.
Existing supported scalar/batch operations and solver cases are included; Hessians
and new simulator capabilities are not. Historical receipts do not qualify current
source. This is a focused review of the load-bearing mechanisms, not a fresh audit
of every implementation expression or every third-party dependency.

## 2. Authority and lifecycle map

| Meaning | Authority and owner | Revision boundary | Derived representation |
|---|---|---|---|
| Relation/field semantics | Registry declarations and admitted arena | Registry owner and complete referenced contracts | Generated bindings and prepared predicates |
| Selected query inputs | Actual native bindings, policies and function assembly | Exact selected owners, including absence | Native plans, admission facts and completed obligations |
| Evaluation semantics | Admitted native expressions and numerical policy | Exact program/input bindings | Immutable instructions and attempt-local workspace slots |
| Durable publication | Delta control row and actual committed member versions | Publication identity, parent and attempt | Exact-version providers, leases and resident results |
| Export ownership | Actual Arrow buffer owners and reservation leases | Last consumer reference | C-stream capsules and consumer arrays |

`GeneratedContracts::new` rejects duplicate/unresolved declarations and cannot mint
an admitted handle (`crates/pse-schema/src/resolved_contract.rs:44`). Selection clones
the actual context state and freezes its native catalog
(`crates/pse-engine/src/session/assembly.rs:97`, `:122`). These derived representations
do not establish separately editable model authority.

## 3. Semantic contracts and invariants

Completed obligation reuse captures relation dependencies and the actual checker
implementation. Unknown or empty declarations force fresh work
(`crates/pse-engine/src/session/obligation.rs:78`, `:111`). Absence is explicit rather
than an empty relation. Mutable/effectful or unqualified providers refuse retained
reuse (`crates/pse-engine/src/session/assembly/derived.rs:84`, `:132`).

The scalar workspace checks finite, dimensionally matching inputs and bitwise point
identity, then invalidates dependent slots (`crates/pse-numerics/src/scalar.rs:355`).
Each requested root checks cancellation; traversal periodically checks it again and
selects only active CASE/Boolean dependencies (`crates/pse-numerics/src/scalar.rs:427`).
Independent numerical expectations are required in addition to scalar/batch agreement.

Typed diagnostics retain aggregate leaf order, multiplicity and original causes.
Disabled, sampled, truncated and interrupted capture are distinct evidence states.
Missing evidence cannot establish complete successful observation.

## 4. Derivation and execution design

Preparation retains immutable assembly capabilities; actual inputs and policy bound
reuse. An attempt supplies live cancellation, permissions and resource admission.
Nested execution inherits its live permit
(`crates/pse-engine/src/session/execution.rs:134`). Freshness examines original
expressions before optimization can erase varying reads; non-immutable functions,
scalar variables and unqualified extensions prevent retained completion
(`crates/pse-engine/src/session/freshness.rs:16`, `:75`).

Foreign Arrow allocations are copied after reserving their extent; proven owners
retain their buffers and reservation lifetime (`crates/pse-ids/src/owned_buffer.rs:267`).
Snapshot pins account for first/last readers; explicit exact-version loads avoid
resolving head (`crates/pse-catalog/src/cache_service/snapshot.rs:225`).

Delta evidence is reserved before execution and finalized with the actual committed
version and complete member identity (`crates/pse-catalog/src/delta/write_evidence.rs:35`).
Settlement distinguishes an unresolved attempt, known commit and interrupted
post-commit maintenance, preserving known effects through observation failure
(`crates/pse-catalog/src/delta/settlement.rs:33`).

## 5. Representative journeys

An ordinary relation extension changes registry declarations, regenerates bindings,
and supplies positive/adversarial controls. The fixture generator derives tagged
literals from the declared discriminator field and emits independent outcomes.
A new numerical opcode remains an explicit capability addition with semantic and
derivative controls; unsupported expressions are refused.

A changed provider or policy yields a new selection. Unrelated edits may reuse a
derivation only if its complete selected inputs still match. An Arrow export retains
its final buffer owner after the parent publication/session is dropped. Python's
transfer report follows nested field paths and distinguishes retained meaning,
storage-only representation, metadata loss and mismatch
(`python/pse/_transfer.py:52`, `:69`, `:85`).

A lost commit acknowledgment is reconciled against exact durable state. Cancellation
before execution and after a partial stream remain attempt outcomes; neither means
a complete empty result. Required campaign controls for these journeys remain
unqualified until their actual execution receipts are reconciled.

## 6. Acceptance gates

| Gate | Verdict | Inspected mechanism / required evidence | Required action |
|---|---|---|---|
| G1 Authority | Unresolved | Registry, native selection, exact Delta control; V02/V04/V05/V11 | Reconcile executed cases |
| G2 Semantic fidelity | Unresolved | Native fields, guarded values, durable/Python mappings; V01/V07/V08/V10/V12 | Reconcile executed cases |
| G3 Validity | Unresolved | Producer evidence and new-input admission; V02/V05/V06/V10 | Reconcile executed cases |
| G4 Hidden behavior | Unresolved | Live attempt cancellation/resources/effects; V03/V04/V06/V13 | Reconcile executed cases |
| G5 Consistency and recovery | Unresolved | Known commit versus unresolved settlement; V09/V10 | Reconcile interruption/recovery results |
| G6 Transformation and reuse | Unresolved | Complete selected inputs and selective slot invalidation | Reconcile clean/reused/changed/evicted cases |
| G7 Truthful capability claims | Unresolved | Explicit unsupported states and feature-mode separation | Complete case reconciliation and measurements |

## 7. Principle findings

Applicability: authority/identity (DM-02, DM-11–DM-15, DM-23), boundaries and validity
(DM-06–DM-10, DM-22, DM-24), effects/recovery (DM-04, DM-19–DM-20, DM-28–DM-30), reuse
and cost (DM-31–DM-33, DM-39–DM-45), and extension locality/capability (DM-53–DM-60).
No maturity score offsets an unresolved required gate.

| Finding | Principles | Concrete evidence/gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| Final qualification incomplete | DM-43, DM-44, DM-59 | I18 interrupted; I19 unexecuted | Implementation alone cannot establish final behavior or cost | Finish campaigns and exact case reconciliation | Zero required failures and disclosed measurement conditions |

## 8. Alternatives and architectural leverage

| Alternative | Duplication and extension locality | Risk / maintenance cost | Performance evidence | Decision |
|---|---|---|---|---|
| Prior repeated session/tuple preparation | More independent orchestration around native meaning | Repeated checks and intermediate ownership machinery | Historical source is not a comparable timing baseline | Superseded |
| Retained native assembly and finite contracted ports | Registry/native plans remain authority; indexes/instructions are derived | Requires complete dependencies and exact ownership | Current qualification pending | Implemented target |
| Always rebuild from native plans and revalidate fresh batches | Simpler retained-state lifecycle, same semantic authorities | Less invalidation bookkeeping, more repeated preparation | No general timing advantage asserted | Viable conservative behavior for unqualified inputs |

The registry, generators and native compiler serve current relation families and
engineering consumers. Scalar evaluation, buffer admission, settlement and scheduling
remain ordinary contracted Rust code. This assessment adds no new control system.

## 9. Verification and measurement

The execution checkpoint is [I16–I19 execution](../../plans/11-i16-i19-execution.md).
Attach final source-qualified counts, all 179 carried cases, V01–V16, feature graphs,
hardware/budgets, repetitions and dispersion before closing this review. Separate
pool reservations/peaks, allocator observations and process RSS. A fresh process does
not imply a cold OS page cache. Smoke execution is not timing qualification; historical
partial timings do not establish a universal speed ratio.

## 10. Exceptions and unresolved decisions

R-20 API-reference lint remains explicitly unsupported. ADR-0066 dependency findings
remain advisory; tool execution failures still require correction. ADR-0074 remains
proposed until its required PR lifecycle completes. This review does not silently
accept or edit an immutable decision. No new SHOULD exception is introduced here.

## 11. Decision and implementation changes

| Decision | Reason | Priority / action |
|---|---|---|
| Unresolved | Required functional and measurement evidence is incomplete | Complete I18/I19 before accepting Plan 11 |
