---
id: ADR-0065
title: Make Arrow and DataFusion the default data and execution foundation
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-03, DM-19, DM-24, DM-28, DM-38, DM-43, DM-44, DM-57, DM-58, DM-59]
blueprint: [§1.1, §1.3, §2, §3.1, §3.3, §5.3, §14.2, §14.3]
review: docs/design_review/reviews/design_review_full-arrow-datafusion-capabilities_2026-09-14.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A capability is excluded solely because it had no initial consumer, or a newly selected engine capability lacks a semantic execution contract
verification: Capability disposition review; pinned library characterization receipt; ADR lint and documentation checks; proposed platform qualification in review section 9
---

# ADR-0065: Make Arrow and DataFusion the default data and execution foundation

## Context

The maintainer explicitly rejected treating the initial bounded recursive UNION ALL
implementation as an architectural ceiling. Investigation found similar exclusions
in blueprint §3.1/§3.3, capability-map recommendations, dependency bans, and the
small selectable optimizer catalog. The [review](../design_review/reviews/design_review_full-arrow-datafusion-capabilities_2026-09-14.md)
compares those decisions with actual consumers and DataFusion 55.1.0 / Arrow 59.3.0.

## Scope

**Current construction target:** ADR-0067 and blueprint revision 37 replace this
record's former local replay, row-copy and phase-limited execution mechanisms.
Plan 05 owns implementation; the domain/identity/lifetime requirements retained
below are implemented through its single native preparation/completion route.

Make Arrow and DataFusion the default foundation for data operations and execution
throughout the system, and amend capability eligibility and selection policy. All Arrow and DataFusion
capabilities, feature flags, extension points and companion crates are available
for use throughout the codebase. This includes facilities currently unselected,
not yet exposed by PSE, or historically rejected. Eligibility is not a claim that
every upstream operation is implemented correctly or already admitted by PSE.

This proposal revises ADR-0013's exhaustive four-role/no-Newton placement ceiling
and the Arrow Flight exclusion in ADR-0026, and supplements the
initial execution profiles of ADR-0039/0048/0053/0062. ADR-0026's units decision
remains applicable. Accepted arguments remain immutable; ADR-0067 records scoped supersession links for the changed placement and admission restrictions. The maintainer authorized
the working-tree policy correction; this record remains proposed and does not
claim an Accept verdict for the unfinished Wave 2 implementation.

## Drivers

- Let the engine implement relational work and its optimization.
- Remove duplicate semantics and manual transformations where existing APIs fit.
- Preserve exact physical meaning, declared effects, coherent revisions and reuse.
- Support measured workstation performance without artificial language ceilings.
- Keep dependency selection proportional to consumers, without permanent bans.

## Options

1. Keep the initial operator/feature allowlist: rejected as an architectural ceiling.
2. Enable every optional crate and feature immediately: unnecessary build, deployment
   and maintenance work without corresponding consumers.
3. Make the entire library surface eligible, select implementations by semantic
   contract and actual use: selected. Existing semantic boundaries determine
   placement; they do not exclude an API family everywhere.

## Outcome

Arrow is the default for typed data and columnar operations. DataFusion is the
default for data transformation, planning and execution across the system. Other
libraries are acceptable when they offer a distinctive advantage. This applies across ingestion/normalization, catalog access, validation
queries, semantic compilation, inference, dependency analysis and incremental
computation, provenance, batch evaluation, case/scenario data operations and results
processing. Blueprint D10 expands these responsibilities; its list is not exhaustive.

Use any Arrow/DataFusion capability that serves a declared operation. Prefer native
expressions, functions, plans, optimizers, kernels, providers and streams before
writing an equivalent general-purpose engine or row transformation in Rust.

Extend the single rule/operation declaration and its mechanical projections where
needed. A registered implementation bound to typed inputs, parameters and outputs
may use native DataFusion plans without reproducing every engine AST variant in a
second closed DSL. SQL may be a frontend to that same declaration/plan route;
Substrait, protobuf and EXPLAIN are derived representations, never separate truth.

Use explicit versioned engine profiles, with the full pinned upstream pipelines as
the starting candidate. Qualify changes against PSE semantics and retain specific,
evidenced exceptions. Reproducibility requires the actual ordered implementations
and configuration, not permanently tiny lists or unexamined evolving defaults.

Effects determine execution placement: async resolution/import, configuration-aware
functions, DML/MERGE in private attempts, remote sources, compatibility profiles and
plugin loading are all eligible. Capture relevant inputs and ownership, then admit
outputs through the existing publication boundary. Pure numerical kernels and
immutable published snapshots retain their existing contracts. D10 establishes the default across data operations and execution rather than
an exhaustive four-role list. Prepared engine expressions or other
library mechanisms may participate in numerical execution when their derivative,
precision, ownership and cost contracts fit; native evaluation remains the current
implementation choice. A solve retains explicit run/effect semantics under any adapter.

### Consequences

The six Arrow/DataFusion companion-crate bans in `deny.toml` are removed;
concurrent ADR-0066 implements their removal within its broader dependency policy. No new
crate, dependency pin or feature activation follows merely from this removal.
IPC compression remains disallowed in canonical identity encoding; its availability
for noncanonical transport is documented. The global Clippy ban is removed and
the governance pattern is scoped to `crates/pse-ids/src/canon`.
PSE's initial algebra and recursion restrictions become explicit implementation
status, with Plan 05 replacing the historical review migration sequence.

### Compensating controls

Preserve the single resolved type universe, active semantic admission, exact keys,
field/quantity contracts, complete dependencies including absence, declared effects,
fallible shared budgets, final-owner lifetimes and atomic publication. Hashes remain
lookup/integrity mechanisms; actual typed equality and validation establish meaning.
Do not infer losslessness from Arrow castability or SQL behavior from API names.

### Confirmation

The [retained probe](../design_review/evidence/full-arrow-datafusion-2026-09-14/README.md)
compares actual values and duplicate counts. It demonstrates broad native mechanisms
and exposes four semantic expectation failures in raw INTERSECT ALL/EXCEPT ALL,
across two partition settings, against a zero baseline. The proposed window/join
lowering passes those examples. This is library characterization, not PSE acceptance.
The review specifies platform differential, lifecycle and performance qualification.

## Pros and cons

The policy increases available implementation choices while reducing incentives for
custom relational code. Its cost is explicit qualification at real boundaries;
capability availability cannot substitute for correctness evidence.

## More information

- [Capability deployment matrix](../design_review/reviews/full-arrow-datafusion-capability-matrix-2026-09-14.md)
- [Wave 2 plan](../plans/04-wave-2-semantic-compilation.md)
- Blueprint §3.1, §3.3, §5.3, §14.2, §14.3; ADR-0026, ADR-0039–0048, ADR-0052–0055.

## Status history

- 2026-09-14 — proposed and expanded to the default data/execution foundation after explicit maintainer direction; working-tree policy
  clarification authorized; formal decision PR and broader runtime qualification pending.

- 2026-09-14 — reconciled with ADR-0067 and Plan 05; prior receipts describe their original code and do not certify the hard-pivot implementation.
