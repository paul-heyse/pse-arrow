---
id: ADR-0256
title: Rebuild the canonical catalog on SurrealDB 4
status: proposed
date: 2026-11-07
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, DP-15, PS-09]
blueprint: [§3.3, §19.3, §20, §20.5, §20.6]
review: docs/design_review/reviews/design_review_surrealdb-v4-canonical-cutover_2026-11-07.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: The pinned v4 candidate fails to preserve required durability, settlement, or selected evidence recovery.
verification: Rehearse fresh native activation, protected scientific readback, selected history rerun, physical settlement and owner-issued release before selecting the durable catalog.
standard: Core-3.4 / Efficient-Architecture-1.0 / Process-Simulator-1.5
scenarios: [PSE-S04, PSE-S05, PSE-S07]
---

# ADR-0256: Rebuild the canonical catalog on SurrealDB 4

## Context

ADR-0214 records the qualified patched 3.3.4 foundation. Plans66 and 67
need efficient protected reads, scoped developer history, and a testable release
lifecycle. The design-phase catalog can regenerate its authored and derived data.

## Scope

Move the source-built server, value and protocol family, parser, and study-policy
extension to exact v4 releases. This advances ADR-0214's concrete release while
preserving its native authority, durability, and bounded transport contracts.

## Drivers

One deployed schema and transport generation makes proof of release and recovery
local to an actual owner. Historical originals with active readers need explicit
custody even when ordinary derived state can be reproduced.

## Options

A fresh catalog permits v4 schema and engine choices immediately. In-place
conversion carries old layout and is experimental and irreversible in 4.0.
A parallel production path would duplicate admission, testing, and settlement.

## Outcome

Select SurrealDB 4.0.0 and persistent SurrealMX 1.1.0. Rebuild the generated
catalog and pure study-policy extension. Preserve only named evidence, by
portable native custody or a one-time bounded import where the reader requires it.

### Consequences

The selected generation uses one v4 writer, schema, and extension. Reproducible
runs can be regenerated under new identities. Plan66 owns scientific readback
and Plan67 owns selected developer-history and resource retirement.

### Compensating controls

Rehearse in a fresh owned state directory. Keep the frozen v3 generation through
the pre-write rollback point. Recover forward from v4 once it admits writes.
Retire patches only after their behavior is covered on the replacement.

### Confirmation

Require original-evidence readback, protected scientific and developer receiving
journeys, typed conflicts, durable reopen, and physical cleanup. The tagged
source establishes interface fit. The owning plans track executed qualification.

## Pros and cons

Fresh reconstruction removes the need for a continuing legacy path and lets the
new schema reflect the v4 target. It requires explicit evidence classification
and verified regeneration before release of the frozen generation.

## More information

[Functional review](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md) ·
[Plan66](../plans/66-integrated-simulation-target.md) ·
[Plan67](../plans/67-storage-lifecycle-and-build-reuse.md)

## Status history

- 2026-11-07 — proposed.
