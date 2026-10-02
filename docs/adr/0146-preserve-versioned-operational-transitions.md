---
id: ADR-0146
title: Preserve operational records through explicit versioned transitions
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, DP-01, DP-04, DP-13, PS-07, PS-10, PS-12]
review: docs/design_review/reviews/design_review_durable-evolution_2026-10-01.md
evidence: Implemented
supersedes: []
superseded-by: null
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s04]
blueprint: [§4.3, §5.3, §20.2, §20.4, §20.5, §20.6]
revisit: A supported execution or schema transition cannot retain its declared meaning and evidence.
verification: Plan 25e/25f preserving transition controls and Plan 25g directional reader/writer, recorded predicate, explicit transformation and retirement/discovery controls; real database and publication recovery journeys remain Plan 25k.
---

# ADR-0146: Preserve operational records through explicit versioned transitions

## Context

Publishing Plan 25e facts changes operational contracts. ADR-0114's reset-based mismatch recovery would destroy prior scientific records and cannot be the normal upgrade path. Preserve its historical arguments and route this supplement through a decision PR.

## Scope

The required 25e/25f G3 slices admit explicit versioned transitions for changed operational contracts. Plan 25g completes the four distinct operations of recorded interpretation, consumer projection, exact write admission and explicit migration, together with retirement inventory and orphan reconciliation. This supplement replaces ADR-0114 Outcome 23's no-migration/reset recovery only; its other decisions remain in force. Destructive reset remains an explicitly requested maintenance operation.

## Drivers

Existing records survive supported upgrades. Readers must know which evidence was actually recorded. Failed transitions must be recoverable without accepting ambiguous schema state or silently preparing writes against stale contracts.

## Options

Reset discards records; implicit best-effort alterations conceal source and recovery state. A second persistence path duplicates authority. Select explicit declared transitions, established PostgreSQL transactions and advisory locking, attributable source/target/checksum history, and readiness admission. Use a pinned migration library where its contracts fit rather than build a general migration framework.

## Outcome

Ordinary open validates the exact supported contract read-only. Explicit migrate admits a declared transition from the exact supported predecessor, after quiescing workers and closing affected store generations. Unrecognized source, checksum conflict or incompatible history refuses without mutation. Serialize upgrades under namespace ownership; mark readiness before schema mutation and commit each declared transition with its history atomically. Reopening verifies the target and prepares current statements only after readiness. Resume only from matching committed history. Distinguish ownership/history for operations and catalog/control dependencies.

Preserve historical descriptor and evidence versions. New incumbent, route and endpoint facts absent from old payloads remain unavailable; do not infer them from reason strings, usability flags or native stops. Writers publish current typed contracts only after admission. Unsupported historical scientific projections return typed unsupported/missing evidence instead of fabricating qualification. Schema support identities depend on their owned persisted contracts, not unrelated registry declarations.

Verify recorded portable witnesses, their complete support closure and observed fields before consulting a consumer's current declaration. Directional consumer projection consumes only its required meaning; an understood historical enum domain remains closed after registry growth. Read compatibility cannot mint exact write admission. Reconstruct checks from verified recorded domains through the same predicate compiler used by current declarations; executable expression bytes and their codec labels are not durable meaning. Sufficient historical witnesses use this one interpreter, while contradictions or unsupported meaning refuse.

Descriptor version 3 records the canonical profile required-root inventory used in its profile digest. Version 2 lacks that inventory and opens only when its baseline inventory can be independently established and the original digest recomputed. Otherwise it returns a typed missing-proof or migration refusal. The versioned descriptor interpreter may project the new inventory column as null for a verified version-2 declaration; generic consumer admission never supplies missing fields. Historical descriptor identity preimages remain unchanged.

Explicit artifact migration binds exact source selections, target declarations and portable structural/domain/reference operations. Checked native lowering validates target values, keys and reference closure before existing atomic publication commits new immutable members with transformation lineage. Source content remains unchanged on failure. Historical frame preimages remain unchanged; new identity roles use the existing framing owner.

Creation, validation-only opening, migration planning and execution are separate operations. Planning never mutates schema/history. Execution rereads its expected plan under the namespace lock and returns applied transitions and final readiness.

Retirement inventory precedes reset: under the exclusive session lease, export a completed versioned manifest outside the erased namespace and member roots, including prior unresolved obligations. Drop, recreation, inventory import, ResetID/digest and readiness commit in one PostgreSQL transaction. Lost acknowledgement settles against that record before another reset. Reader/export protection survives until expiry or authorized release.

Workspace-root discovery records bounded candidate/report progress, ownership evidence and protection dispositions. The local store's unordered listing stream remains process-local; after a crash a new enumeration generation starts at the root and retains deduplicated candidates. Only a full successful pass establishes enumeration completion, never an atomic storage snapshot. Discovery does not authorize deletion. Explicit selected reclaim rechecks ownership, current publications/intents/readers/retention and the maintenance epoch. Unattributable content remains unresolved; local symlink traversal and escaped roots refuse.

### Consequences

Remove reset as the normal mismatch remedy and unrelated global fingerprint dependencies. New fields and table shapes need explicit migration declarations. Plan 25g replaces the inventory-forgetting reset API and migrates its callers. Existing V1–V5 transitions remain immutable; new persisted obligations receive an appended preserving transition.

### Compensating controls

Focused checks cover exact source/target admission, checksum conflict, no mutation on refusal, transactional failure, resumable matching history and preserved legacy evidence. Cross-owner database journeys execute in Plan 25k.

### Confirmation

The 25e/25f implementation and focused checks retain their original scope. The linked 25g review accepted the target at Proposed evidence before implementation. **Implemented/Tested, 2026-10-01:** recorded-domain/reader/writer, finite/composite migration, descriptor, manifest/inventory and confined-listing controls pass (47 durable controls plus three diagnostic assertion controls, local pinned toolchain and explicit force-validation); complete regeneration and workspace all-target compilation pass. [25g Verification](../plans/25g-durable-contract-evolution.md#verification) owns exact commands, repaired failures and the third-party future-incompatibility warning. PostgreSQL/storage/restart journeys are authored but not_run here. Decision acceptance remains a separate PR; full durable journeys remain 25k.

## Pros and cons

Separate capabilities prevent a readable artifact from becoming writable by implication and preserve old meaning without an old runtime. Portable checks and existing native publication machinery localize evolution. Retirement inventory adds durable state and external export prerequisites; restart-safe scans may repeat listing work after interruption.

## More information

[Plan 25e](../plans/25e-declared-analyses-and-qualification.md), [Plan 25g](../plans/25g-durable-contract-evolution.md), ADR-0114 and ADR-0145.

## Status history

- 2026-10-01 — proposed before transition implementation; predecessor acceptance/status remains immutable locally.
- 2026-10-01 — extended before Plan 25g implementation to cover directional artifact interpretation, portable predicates, explicit publication migration and protected retirement/discovery; the 25e/25f evidence retains its bounded scope.
