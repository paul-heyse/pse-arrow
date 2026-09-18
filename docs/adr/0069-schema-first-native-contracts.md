---
id: ADR-0069
title: Make recursive native contracts authoritative for existing data operations
status: proposed
date: 2026-09-16
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-21, DM-22, DM-24, DM-26, DM-31, DM-32, DM-39, DM-41, DM-45, DM-53, DM-59]
blueprint: [§3.3.3, §4, §5, §21, §24]
review: docs/design_review/reviews/design_review_schema-engineering-typed-values_2026-09-16.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An exact-pin test disproves a lossless layout or an existing functional outcome cannot be expressed through the native contracts
verification: just adr-lint; just family-check; just test-package pse-schema; just test-package pse-catalog; docs/plans/08-schema-first-native-data-pivot.md Verification
---

# ADR-0069: Make recursive native contracts authoritative for existing data operations

## Context

Plan 08 narrows the authorized implementation to the architecture of existing functions.
The schema review identifies implicit coupling, untyped nested fields, permissive
positional casts and lost durable metadata. These conventions obstruct the unified
DataFusion/Delta target of ADR-0068.

## Scope

Amend blueprint §§3.3.3, 4, 5, 21 and 24. This record refines ADR-0068 rather than
replacing its unified engine/storage decision. Plan 08 replaces Plan 07's execution
queue. New simulator functions and historical-data compatibility are outside this cut.
Earlier identity byte/envelope prescriptions, including superseded ADR-0050, do not
constrain the new field, signed ordinal or row-key contracts.

## Drivers

One recursive declaration must determine meaning, Arrow fields, durable layouts,
validation and bindings. Native engine capabilities should eliminate generic bespoke
work. Existing domain outcomes, ownership, diagnostics and cancellation remain required.

## Options

1. Keep parallel legacy objects and adapt at every boundary: rejected; duplicates authority.
2. Migrate historical development data: rejected; no user requirement.
3. Replace declarations and callers together, deleting predecessor paths: selected.

## Outcome

Use Arrow Field/DataType as the physical vocabulary, with recursive domain facets.
Generate typed alternatives, quantities, references, collections and defaults. Canonical
PSE ordinals/counts/versions use checked Int64. External Arrow types remain eligible.
Typed row keys frame relation identity, key-contract version and actual ordered primary
key values; payload changes do not change row identity. Selected revision remains explicit.

Generate exact recursive field admission and named reversible layout conversions.
Retain extension metadata only when valid for its physical storage; otherwise store a
logical descriptor and conversion version and restore the execution extension after
checked conversion. Delta fields/properties describe the stored contract independently
of an in-memory registry. Invalid visible values reject admission; filtering them out
is not validation. Masked children of absent parents are not visible values.

Native plans and the common provider hierarchy own current data operations. Delta is
the only durable authority. Coherent expression rows, typed numerical vectors and owned
Arrow streams replace side tables, positional correspondence and general Cell conversion.
Appendix B coverage follows explicit replacement declarations during this design cut:
governance must prove each predecessor side table is absent and its coherent replacement
field exists. The replacement ledger is a test obligation, never a runtime alias or a
deferred-contract exemption. The blueprint amendment remains part of the decision PR.

### Consequences

Delete superseded stores, snapshots, stage/memo machinery, RulePlan/RuleExpr execution,
parallel validators and old Python handles with their caller replacements. Development
data is recreated. No transition flag, migration reader or fallback engine is retained.

### Compensating controls

Exact-pin negative tests cover nested names/nullability/metadata, invalid alternatives,
references, conversion overflow, publication failure and cold opening. Independently
assert current domain outcomes. Shared sessions preserve policy, resources and planners.
Retain no optimizer premise that admission has not established.

### Confirmation

Plan 08 Q01–Q14 and its deletion ledger define acceptance. Current source fixtures,
scalar/Jacobian cases and existing native Ipopt cases remain oracles. Future case,
structural, initialization, Pyomo and NL/SOL work is unscheduled and cannot block or
falsely certify this architectural pivot. G1–G7 remain independently open.

## Pros and cons

Coherent contracts reduce reconstruction and make extensions local. The coupled schema
and caller replacement temporarily breaks compilation; retaining duplicate authority
would defeat the purpose of this design-phase pivot.

## More information

- [Plan 08](../plans/08-schema-first-native-data-pivot.md)
- [Schema review](../design_review/reviews/design_review_schema-engineering-typed-values_2026-09-16.md)
- [Unified target](0068-unified-datafusion-delta.md)

## Status history

- 2026-09-16 — proposed before implementation; maintainer authorized Plan 08. Decision
  acceptance and terminal architecture review remain open.
