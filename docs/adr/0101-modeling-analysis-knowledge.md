---
id: ADR-0101
title: Represent analysis and numerical knowledge as facts and annotations
status: proposed
date: 2026-09-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, DP-13, PS-01]
blueprint: [§14.3, §18, §24.1]
review: docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A scientific port needs model-specific Rust or a synthetic kernel contract fails.
verification: Guard/value separation, annotation typing and explicit unsupported-capability controls; subsequent engines and seed retain their own packet acceptance.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/21-modeling-kernel.md]
---

# ADR-0101: Represent analysis and numerical knowledge as facts and annotations

## Context

Analysis, initialization and reporting knowledge should reuse one model rather than author separate steady and dynamic structures. This records Plan 21 decisions A8–A10; R6.

## Scope

The Plan 21 K0–K8 generic analysis and numerical-knowledge boundary. Implementation
authorization, decision acceptance and executed qualification remain distinct.

## Drivers

Data-only extension, complete physical meaning, explicit dependencies and locally testable
semantic operations. Plan 21 owns execution and finding dispositions.

## Options

Separate model generators diverge; arbitrary callbacks hide effects. Explicit facts and typed annotations expose dependencies and preserve one model.

## Outcome

Represent mode and stage as structural facts; starts, nominal values, validity, checks and reports as typed annotations. Cases and tests are data. K0–K3 checks and preserves future constructs, refusing execution until K4–K7 supply their consumers.

### Consequences

Through K8, fixture data dispatches through existing pure, initialized, steady, integrated
and simultaneous operations. Pure fixtures construct no runtime or native solver. Structured
diagnostics survive results, studies and language transport. Existing fitting and result
consumers migrate before legacy model/dynamics deletion; no new fitting algorithm is implied.

K4–K7 resolves continuous domains before finite expansion and executes immutable case,
stage, elastic and continuation overlays through existing native routes. Generic
diagnostics preserve inconclusive outcomes; case-set studies isolate attempts. Authored
concrete fixtures receive automatic shared checks, and Rust/Python use the same engines.

A selected analysis may declare `annotation objective member(minimize|maximize)` on
one scalar physical member. It selects that existing expression without inventing another
evaluator or equation. Competing objectives are refused; indexed objectives must first
be aggregated explicitly. Elastic penalties may accompany a dimensionless objective,
with the penalty sign chosen to discourage violation for either objective sense. A
dimensional objective requires explicit normalization before combination with elastic
penalties. Native routing retains the optimization intent, quantity and sense.

Conformance policies remain separate from scientific declarations. An explicit fixture-ID
map selects optional solver and derivative-inspection overrides within the same complete
run. Unknown IDs and invalid policies refuse before execution; mixed root/optimization
fixtures need no model-specific dispatch or source filtering.

Declared ports own their scalar coordinates and expression members. Generic authored
`connectivity` annotations bound incoming/outgoing connection counts. Every connected port
requires a policy; zero gives direction, one gives exclusivity, and `many` permits fanout.
Packages choose these limits, including different physical and signal policies. Specialization
checks the original occurrences before any strategy-specific graph projection.
 A causal strategy
selects input/output directions and tear policies; it does not supply a second binding
of ports to formulas. Port/connection identities and instance ownership survive equation
expansion. Original source revisions remain immutable across strategy preparation.

Finite authored solve sequences retain native allocation reuse independently of warm-start
selection. A predecessor is eligible only after both native and original-model checks
accept it. Prepared generic checks execute within the admitted sequence worker; they do
not acquire nested worker permits. Result relations distinguish the solve step from any
trajectory sample index. Repeating one authored case cannot collide in published checks,
and interruption retains attempted results while counting unattempted steps explicitly.

K0–K3 does not implement discretization, general implicit solves or the conformance runner. Comprehensive qualification is requested separately.

### Compensating controls

Typed refusal, stable semantic identities, bounded expansion and targeted positive/negative
controls accompany each mechanism. Remove an old path only after its replacement and callers move.

### Confirmation

Guard/value separation, annotation typing and explicit unsupported-capability controls; subsequent engines and seed retain their own packet acceptance. Target review supports architectural reasoning only; executed evidence belongs
to the owning packet, not a retrospective change to this decision's evidence.

## Pros and cons

The generic contract localizes future science changes to packages. Its initial language,
registry and physical-type migration costs are larger than adding another special case.

## More information

[Plan 21](../plans/21-modeling-kernel.md) and its companion documents own implementation
sequencing. [Target review](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md).

## Status history

- 2026-09-26 — proposed; implementation authorized by the maintainer. Acceptance remains on the decision-PR route.
