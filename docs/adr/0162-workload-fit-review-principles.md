---
id: ADR-0162
title: Assess execution fit through architectural fitness
status: accepted
date: 2026-10-05
deciders: [paul-heyse]
level: decision
principles: [AP-07, DP-03, DP-10, DP-13, DP-16, DP-19, DP-20, DP-23, PS-09, PS-11]
blueprint: [§24.4]
review: "not-required: Maintainer explicitly approved this standard-adoption scope; existing architectural-criteria review supplies rationale and root integration owns independent policy assessment."
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A supported operation exposes an unresolved physical work premise or policy interpretation that prevents a credible conforming realization.
verification: Static policy comparison of the twelve architectural criteria against Core 3.4, template, ProcessSimulator 1.5 and process skills, preserving G1-G9 and PS-G1-PS-G3; scoped documentation and agent checks support links and syntax only.
standard: "Core 3.4 / ProcessSimulator 1.5"
scenarios: [docs/design_review/design_principles/binding/pse-arrow.md#pse-s07]
---

# ADR-0162: Assess execution fit through architectural fitness

## Context

The maintainer approved adoption of the architectural-criteria review's execution-fit direction
across the selected standards. The PSE core already has six architectural foundations and G9;
its physical-work prompts were insufficiently decisive for an assembled supported workload.
The governing owner is [blueprint §24.4](../authoritative_design/sections/design-change-workflow.md).

## Scope

Adopt Core/template 3.4 and ProcessSimulator 1.5 and align local review, plan and ADR guidance.
This amends architectural review policy rather than making a SHOULD exception. Production code,
dependencies, stores, numerical qualification and benchmark claims are outside this adoption.
Historical review snapshots and accepted ADR bodies retain their meaning.

## Drivers

Preserve explicit domain authority, extension locality and all scientific guarantees while
choosing credible physical work for edit/re-solve, studies, recycles and dynamics. Separate
semantic ownership from layout/placement; compare complete-operation growth, live memory,
reuse, independent assurance, transaction/recovery effects and shared capacity. Use existing
scope/scenario discussions, including [PSE-S07](../design_review/design_principles/binding/pse-arrow.md#pse-s07),
without a new registry, review period or mandatory experiment.

## Options

- Retain cost prompts only: local typed/library components can still compose into unjustified
  scans, crossings, rebuilding and resource lifetimes without defeating acceptance.
- Add a separate A4 or G10: creates a parallel acceptance structure although G9 already owns
  architectural fitness; rejected as unnecessary process machinery.
- Add AP-07 through G9 and strengthen existing DP rules: selected. Co-designed native/library
  realizations preserve optimization and owned meaning; a simpler conforming route is compared
  where credible. Concrete techniques remain conditional, not twelve unconditional rules.

## Outcome

Select Core 3.4 / ProcessSimulator 1.5. AP-07 requires execution fit for supported workload
premises through existing G9. Relevant physical work, composed capabilities, locality, live
representations, reuse, assurance, effect-sized transactions/recovery, coordinated resources,
consumer views, exactness/completeness and total machinery are assessed at their existing owners.
Known structural amplification can prevent acceptance before timing; measured benefits still
require measurements. A retained cost needs its concrete benefit.

### Consequences

Designs and plans consider composed preparation, execution, failure and change rather than
repeating only local ownership judgments. Semantic/module boundaries need not become store,
call, transaction or materialization boundaries. Safe refusal does not prove workload fit.
PS-09 retains library-owned numerical iteration, globalization, factors and native management
and bounded scientific composition where no fitting library supplies the consumed contract.

### Compensating controls

Keep G1–G9, PS-01–PS-13 and PS-G1–PS-G3 stable; independent scientific checks remain required.
Reuse immutable assurance only while its premises hold; changed numerical state still receives
PS-10 post-checks. Preserve bounded/discretionary review cadence, optional probes and current
execution/handoff routes. No new hook, checklist, lint or benchmark mandate is introduced.

### Confirmation

Implemented policy, 2026-10-05: selected versions, normative owners and process consumers contain
the adoption. Static comparison maps all twelve source criteria to substantive rule and review
coverage. Root integration owns independent assessment and final document-check receipts.
This establishes written policy adoption, not tested product behavior or measured benefit.

## Pros and cons

The new foundation makes physical execution fit decisive without a second acceptance gate.
It requires scoped judgment of workload premises and composed costs; it does not provide a
universal cost threshold or license to weaken physical correctness.

## More information

- [Selected standard](../design_review/design_principles/standard.toml),
  [governing workflow](../authoritative_design/sections/design-change-workflow.md) and
  [repository binding](../design_review/design_principles/binding/pse-arrow.md).
- Source rationale: library-context architectural-criteria review, 2026-10-05, §§3–5,
  supplied by the coordinator. Its FP-07/A4 proposal is adapted to PSE's AP-07/G9 structure;
  database-specific examples are not PSE requirements.
- Existing workflow and model-review rationale in ADR-0094/0129 remains; this is an additive
  execution-fit adoption, not supersession or an edit of those accepted records.

## Status history

- 2026-10-05 — accepted under the maintainer's explicit implementation authorization;
  policy surfaces Implemented, product assurance unchanged.
