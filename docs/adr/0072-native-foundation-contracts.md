---
id: ADR-0072
title: Unify native validation values and diagnostic contracts
status: proposed
date: 2026-09-18
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-15, DM-24, DM-47, DM-54, DM-56, DM-60]
blueprint: [§4.1, §4.2, §5.3, §23.2]
review: docs/design_review/reviews/design_review_contract-foundations_2026-09-18.md#12-native-foundations-scope-extension-n00n05
evidence: Tested
supersedes: []
superseded-by: null
revisit: A new native representation or persistent identity format is introduced.
verification: just unit-contract-foundations; just family-check; just adr-lint; Plan 10 N18 acceptance after N17.
---

# ADR-0072: Unify native validation values and diagnostic contracts

## Context

Plan 10 N03–N05 removes independent Cell validation, diagnostic classification and
value framing. The existing native Delta predicates demonstrate the shared lowering
mechanism; the exact DataFusion 55.1.0 error tree supports retaining original causes.

## Scope

The authorized hard pivot changes foundation ownership and the fingerprint/row-token
formats in blueprint §4.1, §4.2, §5.3 and diagnostic governance in §23.2. The named
follow-up is `design: native foundation contracts`, including a blueprint revision row.
This proposed record does not accept itself or edit historical accepted decisions.
N06–N18 retain their separate implementation and final acceptance obligations.

## Drivers

One semantic declaration, exact native representations, bounded reusable preparation,
and concrete causes across shared engine error wrappers.

## Options

- Keep independent row validators and classifiers: divergent enforcement and lost causes.
- Wrap the old representations: retains competing semantic authority.
- Compile native contracts and retain native causes: selected.

## Outcome

Schema owns native fields and lossless one-value literals. Relations owns immutable
preparation contexts, local predicates, findings and relational obligation templates.
Catalog owns Delta serialization, storage mapping and publication selection. Diagnostics
is a leaf owning detailed codes, coarse classes and causal engine observations. IDs owns
semantic framing; schema fingerprints and row tokens change format once, without readers
for superseded formats. Named-ID derivation and canonical IPC outer framing stay stable.

### Consequences

Delete Cell and datatype mirrors, duplicate semantic checks and catalog classification.
Use one vocabulary declaration with projections for Rust, registry and Python. Explicit
miette Diagnostic implementations are valid evidence of the trait contract; requiring
only derive syntax prevents typed code reuse and is replaced by trait-aware checks.

### Compensating controls

Exact field and literal round trips, hidden-child validation cases, owner invalidation,
framing vectors and typed source downcasts protect the boundaries. Native structural
validation precedes semantic evaluation. Partial/cancelled reports cannot imply validity.

### Confirmation

Focused unit tests use force-validate and a zero-failure baseline. Pure generation and
compilation are permitted now; full integration and performance claims wait for N17/N18.
The execution inventory records completed versus open obligations.

## Pros and cons

Shared native compilation removes independent decisions, but introduces explicit owner
lifetimes and occurrence mapping. These remain implementation details of declared checks.

## More information

[Plan 10](../plans/10-native-contract-consolidation.md), D02/D03/D11 and N00–N05.
The DataFusion and Delta skills pin the source APIs; private Delta DataValidationExec is
not an importable application interface. Miette code attributes stringify paths, so
constant paths alone cannot centralize runtime codes.

## Status history

- 2026-09-18 — proposed before N03–N05 implementation under the user's hard-pivot authorization.
