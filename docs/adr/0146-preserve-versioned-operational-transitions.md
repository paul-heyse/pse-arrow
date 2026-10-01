---
id: ADR-0146
title: Preserve operational records through explicit versioned transitions
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, DP-01, DP-04, DP-13, PS-07, PS-10, PS-12]
review: docs/design_review/reviews/design_review_declared-execution-and-qualification_2026-10-01.md
evidence: Implemented
supersedes: []
superseded-by: null
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s04]
blueprint: [§5.3, §20.5, §20.6]
revisit: A supported execution or schema transition cannot retain its declared meaning and evidence.
verification: Plan 25e focused admission, qualification, endpoint, resource-ownership and transition controls.
---

# ADR-0146: Preserve operational records through explicit versioned transitions

## Context

Publishing Plan 25e facts changes operational contracts. ADR-0114's reset-based mismatch recovery would destroy prior scientific records and cannot be the normal upgrade path. Preserve its historical arguments and route this supplement through a decision PR.

## Scope

The required G3 slice admits explicit versioned transitions for changed operational contracts. Wider catalog/control and operations migrations remain separately attributable; this slice does not close Plan 25g. Destructive reset remains an explicitly requested maintenance operation.

## Drivers

Existing records survive supported upgrades. Readers must know which evidence was actually recorded. Failed transitions must be recoverable without accepting ambiguous schema state or silently preparing writes against stale contracts.

## Options

Reset discards records; implicit best-effort alterations conceal source and recovery state. A second persistence path duplicates authority. Select explicit declared transitions, established PostgreSQL transactions and advisory locking, attributable source/target/checksum history, and readiness admission. Use a pinned migration library where its contracts fit rather than build a general migration framework.

## Outcome

Ordinary open validates the exact supported contract read-only. Explicit migrate admits a declared transition from the exact supported predecessor, after quiescing workers and closing affected store generations. Unrecognized source, checksum conflict or incompatible history refuses without mutation. Serialize upgrades under namespace ownership; mark readiness before schema mutation and commit each declared transition with its history atomically. Reopening verifies the target and prepares current statements only after readiness. Resume only from matching committed history. Distinguish ownership/history for operations and catalog/control dependencies.

Preserve historical descriptor and evidence versions. New incumbent, route and endpoint facts absent from old payloads remain unavailable; do not infer them from reason strings, usability flags or native stops. Writers publish current typed contracts only after admission. Unsupported historical scientific projections return typed unsupported/missing evidence instead of fabricating qualification. Schema support identities depend on their owned persisted contracts, not unrelated registry declarations.

### Consequences

Remove reset as the normal mismatch remedy and unrelated global fingerprint dependencies. New fields and table shapes need explicit migration declarations. The current destructive reset API is not used by this implementation.

### Compensating controls

Focused checks cover exact source/target admission, checksum conflict, no mutation on refusal, transactional failure, resumable matching history and preserved legacy evidence. Cross-owner database journeys execute in Plan 25k.

### Confirmation

The bounded author review is Proposed evidence. Plan 25e records completed implementation and focused checks, while Plan 25g retains wider migrations. Decision acceptance remains a separate PR.

## More information

[Plan 25e](../plans/25e-declared-analyses-and-qualification.md), [Plan 25g](../plans/25g-durable-contract-evolution.md), ADR-0114 and ADR-0145.

## Status history

- 2026-10-01 — proposed before transition implementation; predecessor acceptance/status remains immutable locally.
