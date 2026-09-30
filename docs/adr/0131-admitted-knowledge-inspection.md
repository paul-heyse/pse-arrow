---
id: ADR-0131
title: Expose immutable admitted knowledge through generated read-only relations
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-05, AP-06, DP-03, DP-09, DP-20]
blueprint: [§6.15.1, §9.10, §22.1]
review: 'not-required: New projection within the accepted package authority; scheduled Plan 23 AUD assesses the boundary before closure.'
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Inspection requires independent scientific interpretation or a second writable data authority.
verification: Synthetic knowledge projection tests preserve identity, quantity, uncertainty and provenance; runtime and Python tests cover bounded readonly inspection and source revision changes.
scenarios: [docs/plans/23-thermodynamic-domain-and-campaign.md#current-execution]
---

# ADR-0131: Expose immutable admitted knowledge through generated read-only relations

## Context

Register R-49 calls for inspection of admitted records rather than reparsing source or
creating an independent database model. Plan 23 includes this read-only capability.

## Scope

Expose generic records, constants, table schemas and cells from the immutable checked
package. Scientific meanings remain authored package declarations. This record remains
proposed until the scheduled Plan 23 boundary assessment.

## Drivers

Consumers need resolved identities, canonical quantities, declared uncertainty and
attributable origins, including rows admitted from binary data documents. Queries must
name the source revision and refuse oversized materializations before allocation.

## Options

Reparsing authored text misses derived values and bulk rows. A writable catalog duplicates
package authority. Project the checked package through an Arrow-free view, with generated
relations at the runtime boundary and DataFusion for read-only relational operations.

## Outcome

The pure kernel supplies borrowed immutable knowledge views. Runtime lowers them through
registry-generated schemas, preserving canonical values, typed references, uncertainty,
origin role, lineage and test-only taint. Inspection does not constitute a production
scientific read and cannot clear taint or mutate admitted data. Explicit selection and
resource limits bound output; a retained projection retains its immutable revision owner.

### Consequences

The registry declares the inspection relations once. Python exposes those generated
contracts without a parallel writable representation.

### Compensating controls

Synthetic tests exercise values, references, nulls, uncertainty and mixed origins. Runtime
checks reserve output memory and reject limits before materialization. Revision tests cover
both source edits and changed data bytes.

### Confirmation

Plan 23 owns targeted acceptance and the final AUD and Q assessments.

## Pros and cons

A single projection serves Rust, relational consumers and Python. Typed generic cells
require explicit lowering for each static value variant.

## More information

ADR-0123, ADR-0125, ADR-0130 and register R-49.

## Status history

- 2026-09-30 — proposed before the inspection implementation under Plan 23 authorization.
