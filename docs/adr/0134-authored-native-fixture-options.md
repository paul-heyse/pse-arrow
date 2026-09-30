---
id: ADR-0134
title: Carry native fixture options through the existing solver controls
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, DP-01, DP-03, DP-15, PS-10]
blueprint: [§6.15.1, §18.10.1]
review: 'not-required: Extension of ADR-0119 fixture execution policy; Plan 23 AUD reviews the consumed boundary.'
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A native option requires scientific interpretation or bypasses protected typed controls.
verification: Authoring roundtrip and primitive-option refusals; runtime precedence; heater global-certification conformance with SCIP presolving disabled.
scenarios: [docs/plans/23-thermodynamic-domain-and-campaign.md#current-execution]
---

# ADR-0134: Carry native fixture options through the existing solver controls

## Context

The unchanged PC-SAFT heater certifies when SCIP's `presolving/maxrounds` is zero.
SCIP's default presolve reports infeasibility contradicted by the independently validated
local candidate. Disabling only the convex nonlinear handler does not resolve it.
The model and its expected certificate remain unchanged; execution policy needs to state
the native setting rather than hide a backend-specific override in orchestration.

## Outcome

Extend ADR-0119's authored fixture policy with named native primitive options. Reuse
the existing cell carrier for Boolean, bounded integer, finite unitless real and text;
other cells and uncertainties are refused. The runtime copies these values into the
existing solver controls, with authored values overriding run defaults. The selected
adapter still validates names, native types and protected controls and records effective
options. No scientific value interpretation or separate solver configuration path exists.

## Scope

Generic fixture execution inputs within ADR-0119; neither model semantics nor adapter
protected controls change.

## Drivers

State the numerical setting that makes the unchanged certification reproducible and keep
backend-specific choices out of orchestration.

## Options

A hidden heater-specific override or a run-wide setting would obscure scope. Named
authored primitives reuse the existing native controls and adapter validation.

## Pros and cons

One existing control path records the effective settings; native option names remain
library-specific and are validated by the selected adapter.

## More information

ADR-0119 and the Plan 23 boundary audit govern this extension.

## Confirmation

Plan 23 owns the targeted tests, scheduled AUD, final qualification and source amendment.
This proposed decision does not claim comprehensive qualification.

## Status history

- 2026-09-30 — proposed before the fixture policy extension.
