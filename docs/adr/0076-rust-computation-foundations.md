---
id: ADR-0076
title: Separate semantic compilation from relational execution and adopt bounded Salsa reuse
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-02, DM-05, DM-13, DM-28, DM-31, DM-39, DM-43, DM-59, DM-60]
blueprint: [§D1, §D6, §D10, §D14, §1.3, §3.2, §3.3, §14.3, §14.4, §15.3, §20.4]
review: docs/design_review/reviews/design_review_rust-computation-target-design_2026-09-23.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A supported semantic dependency cannot be represented by the admitted input contract, or measured graph/query retention exceeds the declared generation budget.
verification: Plan 13 foundation units; just family-check; just codegen-rust-contracts-check; scripts.tests.test_implementation_phase; just adr-lint.
---

# ADR-0076: Separate semantic compilation from relational execution and adopt bounded Salsa reuse

## Context

The two 2026-09-23 Rust computation reviews identify excessive universal engine placement
and repeated semantic compilation. The maintainer authorized Plan 13 W00–W06 implementation.
Prior Plan 10/11 functional and performance qualification remains incomplete.

## Scope

This decision amends the cited blueprint sections. It adopts Plan 13, the dedicated
`pse-codegen` tooling crate, `pse-model` generated semantic values and `pse-columnar`
low-level native adapters. This foundation checkpoint precedes W07–W11 consumer conversion;
it does not withdraw any supported behavior or qualify the complete compiler.

## Drivers

Single semantic authority, complete dependency observation, bounded ownership, deterministic
graph meaning and practical use of maintained library implementations.

## Options

1. Universal DataFusion stages: rejected for finite semantic algorithms and reuse boundaries.
2. Batch-only typed compiler: rejected because repeated edit/compile reuse remains required.
3. Typed semantic compiler with Salsa, library graphs and explicit relational phases: selected.
4. Generator inside xtask: rejected in favor of a directly testable reusable tooling crate.

## Outcome

Registry declarations generate plain values and Arrow adapters. Semantic crates have no normal
Arrow/DataFusion/Delta/Tokio closure. Salsa owns semantic dependency validation; the runtime owns
admission, jobs and asynchronous phases. Delta remains durable authority. Graph projections are
complete, typed, immutable and canonically ordered; qualify rust-igraph matching before W10.

### Consequences

Move the effectful compiler driver to runtime. No compatibility facade or second production
compiler is introduced. Existing pass wrappers retain explicit W07–W11 deletion owners.
Stable artifact identity includes actual implementation inputs; no Salsa/graph handle is durable.

W15 registers `ContainmentInspection@1` as the sole producer of requested ancestor-pair
tables. Ordinary P5/P9 compilation retains complete linear forest intervals and does not
produce those tables. Explicit inspection admits the complete expansion against its row
capacity before emitting values; requesting diagnostics alone does not select expansion.

### Compensating controls

Changed fields only; explicit absence inputs; cancel/drain before setters; fresh worker handles;
whole-phase completion; bounded generation retirement; exact provider bindings; native last-reader
leases. Keep phase-0/1 library eligibility. W18 blocks product qualification until all scope closes.

### Confirmation

The execution inventory records exact case identities and classified receipts. Unit/static/generation
evidence does not award W19/W20 acceptance. Final incremental/fresh equivalence remains required.

## Pros and cons

Library algorithms and incremental boundaries reduce custom mechanisms. Crate extraction and
explicit async coordination add migration work, bounded by direct caller moves and deletion owners.

## More information

[Plan 13](../plans/13-rust-computation-architecture.md),
[foundation execution](../plans/13-w00-w06-execution.md), and register R-01/R-22.
ADR-0075/R-32 Pyomo tear-selection scope is unchanged.

## Status history

- 2026-09-23 — proposed; maintainer authorized local implementation. Formal decision/design PR and acceptance remain pending.
