---
id: ADR-0133
title: Consume admitted measurements and publish fitted parameter lineage
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-05, DP-03, DP-09, DP-24, PS-01, PS-12]
blueprint: [§6.10, §6.15.1, §18.8]
review: 'not-required: Register R-50 fires within the accepted data and fitting decisions; Plan 23 AUD assesses the implemented boundary before closure.'
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A fit needs measurement semantics that cannot be expressed as typed admitted attributes and explicit observation selection.
verification: Typed measurement units cover canonical quantities, declared and column uncertainties, measured origins, test-only reads and wrong quantities; solver regression and covariance tests consume the same admitted records; DM6 tests explicit fitted publication and missing fit lineage refusal.
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md#current-execution]
---

# ADR-0133: Consume admitted measurements and publish fitted parameter lineage

## Context

Register R-50 keeps fitting datasets and observations outside typed modeling data. Plan 23
now includes a fit consuming a measured property bank and publishing attributable fitted
parameters, so the deferred trigger has fired.

## Scope

Measurement values and uncertainty have one admitted modeling-record owner. Fit declarations
continue to own parameter and experiment selection, not a second writable measurement model.
This record remains proposed through the scheduled AUD.

## Drivers

A fit must consume canonical physical values from inline or binary banks, preserve their
source and role, reject incompatible conventions and retain measurement identity. Fitted
parameters must enter scientific knowledge only through explicit publication with fit lineage.

## Options

Keeping authored datasets and observations duplicates measurement authority and bypasses
modeling admission. Translating them on demand keeps two production input paths. Select
resolved attributes of admitted records and delete the old input relations and attachment API.
The existing fitting engine continues to own numerical fitting, covariance and qualification.

## Outcome

The registry's `measured` role facet marks data eligible as measurements. Each fit observation
selects a record identity and a value attribute. Standard uncertainty comes from its declared
cell uncertainty, or from one explicitly selected typed standard-deviation attribute for bulk
banks. Supplying both is refused. Relative cell uncertainty is converted to an absolute
standard deviation; a bound has no standard-deviation interpretation and is refused.
Canonical quantities must match the observed output's complete physical contract. Missing
values and uncertainties remain explicit, and included observations require both.

The fit attachment contains fit declarations only. `authored.datasets`,
`authored.observations` and their writable attachment disappear. Source identity covers the
admitted modeling revision, including binary bank bytes; an uncertainty or measured-value
change creates a new fitting source. The fitting-source frame is `pse.modeling.fit-source.v2`; the V1 spelling remains immutable but has no production consumer.

A completed qualified fit may explicitly export typed parameter cells together with its
fit and run identity and source revision. `runtime.fitted_parameter_cells` is the generated export contract; its values retain quantity-type and canonical-unit identities. Publication does not mutate a bank. The caller
selects the target parameter-set schema and attributes, then admits the new declaration.
The `requires_fit` role facet requires a `fit` lineage entry naming the attributed fit
receipt source. Generic provenance retains its source, role and lineage, including taint.
No numerical estimate automatically replaces an authored parameter set.

### Consequences

Generated fit selectors name attributes, and measurement reads use the kernel's existing
provenance boundary. Package data and consumers migrate together. Statistical interpretation
remains explicit; a bound is never silently treated as a standard deviation.

### Compensating controls

Admission validates data types and origins. Fitting validates selected physical contracts,
positive standard deviations and output extents. Explicit export requires fresh original-model
qualification and preserves the producing run and immutable source identity.

### Confirmation

Plan 23 owns targeted numerical and boundary tests, DM6, AUD and final qualification.

## Pros and cons

One admitted data model serves inspection and fitting. Observation selectors must explicitly
name the value and optional uncertainty attribute.

## More information

ADR-0123, ADR-0125, ADR-0130, ADR-0131 and register R-50.

## Status history

- 2026-09-30 — proposed before implementation under Plan 23 authorization.
