---
id: ADR-0043
title: Declare kernel outcomes and generate only supported bindings
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-08, DM-19, DM-25, DM-28, DM-43, DM-44, DM-47]
blueprint: [§D9, §6.11, §7.6, §18.5, §18.9, §21.4]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0012]
superseded-by: null
revisit: A current kernel needs an execution or outcome policy that the descriptor cannot express without backend-specific semantic invention.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0043: Declare kernel outcomes and generate only supported bindings

## Context

R4-02 and R4-10 show that conditional flags are not execution, null outputs lack mandatory meaning, natural units are absent from the descriptor, and backend capability tables disagree.

## Scope

Amends kernel/result and generated-binding contracts; no arbitrary compiled plugin service is added.

## Drivers

Keep missing inputs, invalid computations and unsupported derivatives distinct across adapters. A declared signature must not be mistaken for an implementation of units, laziness or a backend route.

## Options

Generate every adapter from a signature alone: rejected. Let each backend decide nulls and units: rejected. Generate mechanical adapters from complete declarations and checked implementation bindings: selected.

## Outcome

Declare natural units, null-input handling, domain/failure outcomes, effects, argument evaluation, derivatives and supported binding implementations once. Generate only bindings backed by an implementation and conformance scope. Scalar/UDF failure is a typed error; tolerant batch analysis returns mandatory typed per-row outcomes. Runtime conditionals use actual branch execution, not eager UDF flags.

### Consequences

Kernel authors still write the actual algorithm and supported derivative implementations. An ImplicitRef has no direct DataFusion binding; an explicitly bound implicit KernelCall is a separate capability.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

E2 supplies the eager-UDF counterexample. Plan 02 requires scalar/batch/native/NL/Pyomo value, failure, unit and derivative conformance; absent bindings fail preflight. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Generation removes repeated adapter decisions while allowing specialized algorithms. Complete binding descriptors and actual conformance fixtures cost more than a broad capability checkbox, but make unsupported routes explicit.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0012 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
