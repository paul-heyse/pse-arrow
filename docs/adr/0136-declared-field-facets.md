---
id: ADR-0136
title: Declare field comparison purposes and admit selected physical definitions
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, DP-01, DP-03, DP-04, PS-01]
blueprint: [§4.4, §5.3, §8.2]
review: docs/design_review/reviews/design_review_plan25a-contracts_2026-09-30.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A new facet requires a consumer-owned key allowlist, or a valid selected physical definition cannot be admitted without a second unit resolver.
verification: Plan 25a facet-purpose, nested-field, directional admission and selected physical-closure controls; integrated qualification in Plan 25k.
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s09]
---

# ADR-0136: Declare field comparison purposes and admit selected physical definitions

## Context

F34 identifies duplicated metadata classifications and narrower relational unit admission.
FU10 identifies unchecked canonical conversion overflow. A field's observable structure,
value identity, execution identity, storage type and directional admission ask different
questions; collapsing them into one equality would lose meaning.

## Scope

Plan 25a owns schema declarations, generated projections and selected physical admission.
Its authorized H4 contribution migrates their real comparators. Broader H4 validation/dead
path work and I1 identity migration remain with their owners. Existing identity bytes remain
unchanged for unchanged contracts; any actual changed preimage uses a new frame version.

## Drivers

One declaration owns each facet's purpose. Low-level consumers must use it without a
dependency back into the schema generator or an upper runtime. Physical compatibility
must consume the same selected admitted definitions as execution.

## Options

- One universal Arrow equality confuses physical meaning, usage and presentation.
- Independent key lists drift as new facets appear.
- SQL reimplementation of defined-unit algebra creates a second physical authority.
- Selected: schema-declared purposes generated downward, directional predicates retained
  explicitly, and a pure shared relation-to-quantity projection.

## Outcome

The schema model declares domain/materialized spellings, comparison purpose, root/nested
treatment and directional admission behavior. It generates a data-only policy under
pse-columnar; columnar does not depend on schema. Native field projection remains recursive.

Physical observation preserves exact fields. Execution identity omits declared presentation.
Value identity additionally omits declared usage-role/reference facets and normalizes only
the permitted root properties. Logical storage type keeps its separate root storage
question and exact nested fields;
logical type identity additionally omits declared structure presentation recursively.
Unknown metadata is significant by default. Dictionary ordering remains part of the
existing field contract. Actual transfer owner, ordered coordinates and direction are a
declared semantic report facet retained by execution, value and logical type identity.
Directional target admission and missing-metadata restoration
remain distinct operations, not aliases for projected equality.

Move the pure checked-row physical decoder into pse-relations, depending downward on
pse-quantity. Runtime retains selection/execution/resource ownership. Catalog supplies exact
selected relation revisions; the shared builder admits their required physical definitions
and constructs compatibility projections for distinct encountered quantity/unit pairs.
Definitions, including composite-unit dimensions/scales, affine single-factor admission,
unit datum restrictions and typed reference-condition dependencies, come from this closure.
Missing definitions cannot be filled by global state.

Preserve reference/normalized-unit reconciliation and reject conflicting definitions.
Quantity obligation binding consumes the immutable admitted projection and attributes
refusals to occurrences. No quantities-times-units expansion and no conversion-on-read
mutation are introduced. Inputs without quantity occurrences need no physical inventory.

### Consequences

Generation gains a columnar policy root. Existing schema/row-token/checked-value consumers
move together; adding a declaration without moving consumers does not complete the work.
The lower projection removes a runtime-only ownership accident without importing runtime
dependencies into relations or catalog.

### Compensating controls

Tests distinguish every purpose, root/nested names, unknown metadata, dictionary ordering,
established semantic conflicts and lawful restoration. Admitted composite-unit controls
cover missing factors, cycles, bad identities, datum mismatches and incomplete closures.
Numeric admission controls are owned by the common quantity operation.

### Confirmation

Implemented mechanism scope is recorded in blueprint §4.4, §5.3 and §8.2. Plan 25a owns
focused evidence, with actual commands and zero-target results. This decision remains
Proposed; acceptance requires the decision route, and full integration remains Plan 25k
work. No qualification follows from schema consistency or documentation alone.

## Pros and cons

Facet changes stay local and unit compatibility has one mathematical authority. Generator
bootstrap and the initial consumer migration cross several crates once.

## More information

[Plan 25a](../plans/25a-physical-values-and-contextual-contracts.md), F34 and FU10;
blueprint §4.4, §5.3 and §8.2. Current finding status belongs to the Plan 25 coordinator.

## Status history

- 2026-09-30 — proposed before the maintainer-authorized Plan 25a facet/projection implementation.
