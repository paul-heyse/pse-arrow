---
id: ADR-0158
title: Bind observed POUNCE execution with typed configuration and admission
status: proposed
date: 2026-10-04
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-06, DP-19, DP-20, PS-08, PS-09, PS-10]
blueprint: [§18.6, §18.7, §18.8]
review: docs/design_review/reviews/design_review_automatic-pipeline-decisions_2026-10-04.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An upstream release supplies the consumed observation, typed configuration and pre-operation admission contracts.
verification: Plan 25n N5/N6 actual acting profiles, post-classification separator binding, Schur use/fallback/work/storage and terminal-abort controls; Plan 25k owns assembled qualification.
standard: core-3.3 / process-simulator-1.4
scenarios: [docs/design_review/design_principles/binding/pse-arrow.md#pse-s02, docs/design_review/design_principles/binding/pse-arrow.md#pse-s05]
---

# ADR-0158: Bind observed POUNCE execution with typed configuration and admission

## Context

AF-06 identifies unconsumed visible second opinions, partitioned/finite-difference Hessians and
solve Schur bindings. An opaque wrapper cannot supply actual native layout, factor configuration,
failed work or complete allocation lifetimes required by blueprint §§18.6–18.8.

## Scope

Add a narrow source-owned binding within existing library ownership. Preserve ADR-0156's
composition boundary and ADR-0157's separate Uno/PETSc scope. Dependency admission is unchanged.

## Drivers

Native execution must act on admitted typed settings, preserve original guarantees, enforce task
admission and distinguish numerical fallback from terminal abort without reimplementing solvers.

## Options

Invocation counting and raw option strings overclaim effective behavior. Copying Schur arithmetic
adds a competing numerical owner. Select a maintained immutable POUNCE source revision exposing
configuration and observation at the library operations that own them.

## Outcome

Use the maintainer-selected public fork based on characterized POUNCE 0.12 source. The library
supplies actual post-classification x/s/c/d layout and receives the mapped separator before its
algorithm builder consumes it. Inject the same effective typed FERAL profile into Schur factors
and rebuilt main/restoration factories. Hooks are synchronous, worker-local, reset per attempt and
fallible, with callback panic containment. Report actual path/fallback and failed primitive work;
admit work and complete storage before operations/allocations. Retain factor, coupling, separator,
refinement and simultaneous fallback allocations through teardown. Resource/cancel/contract abort
is terminal, never numerical fallback. Library owners retain arithmetic, inertia and globalization.

Lower pure second-opinion descriptions to immutable noncumulative acting profiles. Partitioned
and finite-difference modes consume actual analytic First and conservative support, including
objective support at stationary points, finite probes and explicit approximation observations.
Other adapters act on these modes or refuse them. One typed lowering owns reserved controls.

### Consequences

Publish an addressable immutable source commit and pin the complete resolved family coherently,
including internal edges, preserving QP/convex features and FERAL 0.18.0. Native build identity names
the revision. The same immutable artifact contains a source extension of FERAL 0.18 for an
explicit bounded serial dense factor profile. Its complete linear allowance covers Schur and
simultaneous fallback lifetimes; opaque application storage remains separately deployment-owned.
Profiles outside the proved extent refuse strict linear admission, while ordinary library profiles
retain their supported execution. These are conditional capabilities, not a full native-heap bound.
No registry edits, Cargo patch/replace, unconfirmed-release wait or project Schur engine.

### Compensating controls

Targeted actual native consumers establish use, configuration, failure accounting and retained
storage. Original assessment remains mandatory; native success grants no scientific permission.

### Confirmation

**Proposed:** the exact source/API basis is interface-inspected; N5/N6 controls are planned.
The bounded decision review assesses this binding, not product qualification or ADR acceptance.

## Pros and cons

The seam makes effective operation controllable and observable, at the cost of maintaining a
source revision until equivalent upstream contracts are available.

## More information

[Implementation owner](../plans/25n-automatic-simulation-solve-pipeline.md#n6);
[source review](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-06).

## Status history

- 2026-10-04 — proposed for maintainer-authorized Plan 25n implementation; public source hosting selected during execution planning; acceptance remains the decision route.
