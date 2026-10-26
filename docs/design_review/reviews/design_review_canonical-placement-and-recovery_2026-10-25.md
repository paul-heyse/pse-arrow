---
title: Canonical placement and recovery decision review
date: 2026-10-25
tier: design
purpose: target
focus: principle-focused
decision: Accept
evidence: Proposed
standard: core-3.4/process-simulator-1.5
disposition_owner: docs/plans/56-design-principle-alignment.md
---

# Canonical placement and recovery decision review

**Accept at Proposed strength.** Independent reviewer c_contract_review assessed ADR-0228 and
Plan56c C01–C05, selected RC01 and only RC04's product recovery helper. Architectural fitness and
behavioral contract adequacy pass within this boundary; implementation, operational qualification
and performance remain unestablished. This publication preserves that reviewer's judgment.

## Scope and evidence

Core 3.4, heuristics 1.0, Process Simulator 1.5 and the selected pse-arrow binding apply. Baseline
c3b11ba63c0c34c1421beafa7d81ebf849fec54d plus root-owned ADR-0228 and Plan56/56c edits, including
the corrected complete-encoded-body threshold. Concurrent production edits are excluded. This
continues the Plan56 engagement: audit f9b96810a DA-007/033 premises were reconciled with current
source; no broad diagnostic audit was initiated. No writes, builds, tests or benchmarks informed
the judgment.

Inspected owners include canonical schema/generation and terminal projections; operations execution,
results, files, transport, projects and research; runtime result_files; Python payload_files and
supervisor freeze/inventory/copy/restore/fresh-engine; the mixed xtask recovery helper; and blueprint
§20.1–§20.5. The boundary includes locator/body construction, publication, protected reads/release,
backup/restore, exact settlement and fresh v8-to-v9 activation. Solver/formulation behavior, online
backup, historical migration, cross-host files and wider worker/project relocation are excluded.
Plan56 S05/S08/S10 and PSE-S04/PSE-S07 representation/interruption concerns motivate this review.

## Contract and ownership

### Declared meaning and settlement

Declarations own key/vocabulary/association/transition/projection/retention meaning. Mechanical
outputs consume these facts rather than infer them from names/prefixes. Canonical lookup keys
remain distinct from scientific identity. The shared effect domain distinguishes exact ACK,
different immutable request, owner-proven refusal/absence and unresolved effect. Research currently
can obscure the original cause, while project settlement loses failed-readback cause; preserving
both with owner/intent attribution addresses the actual divergence.

Owner-specific atomicity and lifetime remain essential: staged import, finite creation, fenced
attempts, file intents and project references are not one protocol. A missing row is insufficient
proof of absence. Original deadlines, cancellation and fences govern settlement, without renewed
authority or permission to retry science.

### Placement and lifetime

Eligibility is one complete admitted encoded body, at most 65,536 bytes inclusive. A homogeneous
IPC body may contain several batches; descriptors share the body with exact coordinates. Testing
constituent batches cannot qualify an oversized body. Packing/demand boundaries precede placement;
allocation pressure cannot regroup or switch arms.

A flat storage discriminator plus nullable generated fields is valid when construction and decode
enforce exactly one arm and yield the owned typed locator. The locator binds codec, applicable
schema, interpretation, exact digest/extent and member coordinate. The publication owner attaches
inline body, descriptors and ACK atomically under original guards; sealed publication retains its
original intent/descriptor through uncertainty. Neither grants scientific permission.

Actual allocation/transport owners retain inline bytes through RPC and reader drain. Descriptor
discovery remains separate from body hydration. Exact canonical references/read tickets protect
shared bodies through future reads, retirement and failed release; inline collection creates no
file instance. Separate 128 KiB index, 2 MiB append target and actual 4 MiB RPC admission measure
different things; full query/binding encoding remains final transport admission. File-work removal
is a structural benefit, not measured latency/capacity evidence.

### Complete offline operations and activation

Current Python drops per-instance locks before later copy reopens source paths. Moving only the
preflight verifier would preserve that gap. A native operation instead owns each exact lock and
stable descriptor through reconciliation/hash/copy/fsync/final verification, while Python owns
closed admission, service/manager/cgroup freeze and authenticated drain across the whole operation.
Filesystem observations cannot independently prove application quiescence. Restore uses stable
archive descriptors and a fresh destination without source locks excluded from the archive.
Private/no-follow checks and narrowly authenticated final/pending hard-link recovery remain.

The product helper belongs in existing pse-operations, with fixture controls in xtask. Coordinated
v9 cutover closes admission, authenticates drain, stops, regenerates only selected disposable state,
initializes/validates closed and explicitly readmits. Interruptions retain the exact closed target.
Preserve the selected store and prior 56b databases/receipts. Unchanged scientific document bodies
need no incidental version bump.

## Foundations and gates

These verdicts assess the proposed target only.

| Foundation | Verdict | Supporting mechanism |
|---|---|---|
| AP-01 | satisfied | Scientific, canonical, physical and supervisor responsibility boundaries |
| AP-02 | satisfied | Consumed locator/effect meaning and deliberate version evolution |
| AP-03 | satisfied | Shared knowledge with distinct protocols; native operations with Python freeze |
| AP-04 | satisfied | Explicit body/member, placement, ACK/effect and physical/application distinctions |
| AP-05 | satisfied | Closed variants, exact bindings, guards and activation phases |
| AP-06 | satisfied | Declaration/settlement/file owners testable without scientific execution |
| AP-07 | satisfied | Eligible file lifecycle removed, shared bodies and effect-sized guard lifetimes |

G1–G9 pass in this bounded proposed target: declared authority, explicit representation, receiving
admission, publication/recovery effects, interruption/protection, exact scientific fidelity,
truthful evidence, native library mechanisms and satisfied foundations. PS-G1/PS-G2 are not
applicable because no physical/property/formulation contract changes. PS-G3 passes only for
transport preserving original scientific conclusions; numerical adequacy was not reassessed.

## Alternatives and library fit

File-only remains viable but retains tiny-file work. Database-only subjects large IPC to RPC and
loses lazy files. An archive/log or second store adds authority. The two-arm locator is the simplest
examined realization preserving small-output economy and large-file capability. Context7 official
SurrealDB [bytes documentation](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/query-language/language-primitives/data-types/bytes.mdx)
and transaction documentation support native record bytes and atomic grouped writes. Installed
surrealdb-types 3.3.4 Value::Bytes and existing bindings confirm the pinned primitive; these do not
qualify the actual MX/gRPC deployment experimentally.

## Findings, rule impacts and decision

No material open design finding remains. The member/body threshold ambiguity was reconciled
before this decision. Exclusive arm validation, shared-body integrity, charged hydration, original
scientific admission, protected release, dual causes, original authority, native guard lifetime,
mixed recovery and interrupted activation remain implementation acceptance obligations.

RC01 supplements ADR-0187's file-only rule; retaining that rule cannot deliver this selected target.
RC04 changes only the offline helper composition root; wider relocation remains pending. No new
waiver is recommended. Formal ADR acceptance remains separate. **Accept — Proposed target only.**
Plan56 owns dispositions and Plan56c owns implementation/qualification.
