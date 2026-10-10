---
id: ADR-0132
title: Compose temporal state from one analysis-owned axis
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-05, DP-03, DP-09, DP-20, DP-24, PS-08]
blueprint: [§6.15.1, §9.10, §18.6]
review: 'not-required: Kernel composition within the accepted continuous-domain decision; Plan 23 AUD reviews the implemented boundary before closure.'
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Temporal composition needs independent unit science or unrelated time axes in one analysis.
verification: Synthetic temporal composition tests exercise one axis, shared scalar parameters, distinct spatial replicas, native integration and simultaneous discretization; CT-S05 exercises the same CSTR definition.
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md#current-execution]
---

# ADR-0132: Compose temporal state from one analysis-owned axis

## Context

Plan 23 requires analysis-owned temporal composition. The CSTR campaign currently authors
whole-unit replicas and supplies singleton time sets manually, repeating analysis mechanics
at each child declaration.

## Scope

Add generic composition of an authored child hierarchy over a physical time axis. The unit
still owns its balances and derivative laws; the analysis owns the axis and its realization.
This record remains proposed until the scheduled AUD.

## Drivers

One unit definition must serve steady, integrated and simultaneous analyses. Changing the
time mesh must lift its numerical states and ports without duplicating scalar numerical
parameter authority or collapsing independent spatial replicas.

## Options

Keeping manually authored replicas leaves temporal composition with every caller. Rewriting
unit science in Rust violates the accepted boundary. A generic analysis declaration lowers
temporal child coordinates and supplies the unit's declared instantaneous-time argument,
using the existing continuous-domain and native integration mechanisms.

## Outcome

`evolve <policy> on <child> using <axis> bind <argument>;` selects an analysis-owned time
axis for a child. Admission derives its temporal coordinate ahead of explicitly authored
spatial coordinates. Instantiation supplies the selected coordinate as the singleton
value of the declared set argument. Numerical states and ports are temporal; scalar
numerical parameters share one owner across time, including descendants. Independent
spatial child coordinates retain distinct owners. Conflicting parameter initial values,
competing policies, non-time axes and a second temporal axis are refused.

A stationary realization selects the lower endpoint without integration or finite
mesh equations. Dynamic laws and endpoint constraints remain explicitly guarded by the
analysis route. Existing integrated and simultaneous realizations consume the same unit.

The source revision uses `ModelingSourceRevisionV4` over the structured declaration rows,
data-document identities and hashes, physical inventory and resolved physical bindings.
The new frame separates the temporal declaration schema from previously stored revisions;
there is no migration or compatibility interpretation.

### Consequences

Temporal instantiation is a mechanical lowering, not an additional scientific definition.
The checked child coordinate schema is derived from its owning policy. The original source
and inspection metadata retain the policy. Scalar parameter defaults that vary with time
are refused; schedules use the existing analysis input mechanism.

### Compensating controls

Bounded specialization limits govern temporal expansion. Tests distinguish shared temporal
parameters from independent spatial parameters and exercise coordinate and policy refusals.

### Confirmation

Plan 23 owns targeted tests, CT-S05, the final qualification and AUD.

## Pros and cons

The declaration localizes time realization in the analysis. Internal temporal instances
remain explicit for solver lowering and diagnostics, increasing specialization size with
the selected mesh.

## More information

ADR-0123, ADR-0125 and Plan 23 CT-S05.

## Status history

- 2026-09-30 — proposed before implementation under Plan 23 authorization.
