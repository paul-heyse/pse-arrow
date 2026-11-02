---
title: "Flowsheet Process Domain Model"
notion_source: "https://app.notion.com/p/40b141b5836981568752c1b6a854ddf0"
notion_last_edited: "2026-11-01T23:33:52.240Z"
---

# Flowsheet Process Domain Model

Source: [Notion](https://app.notion.com/p/40b141b5836981568752c1b6a854ddf0). Imported 2026-11-02.

**Version 1.1** · 2026-11-01 · Adopted domain contracts and implementation refinements; implementation evidence tracked separately.

> The product domain is flowsheet process modeling: composing equipment, material states, physical transfers and operating requirements into reusable models that support simulation, design, optimization, estimation and control. The overall formulation framework supplies the mathematical and execution contracts.

## Relationship to the overall model

This is the process-specific companion to [Unified Simulation Domain Model](../simulation/index.md). Both are live design guidance. Authored definitions and their resolved package closure in the repository remain the single executable authority.

| Responsibility | Owner |
| --- | --- |
| Material systems, state/property/reaction contracts, equipment composition, inventories, physical connections and process specifications | This process-domain design |
| Model / Specification / Query, atomic SimulationProblem, Study orchestration, task profiles, execution capabilities and qualified results | Overall formulation and execution model |
| Executable equations, definitions, instances, interfaces and process views | Existing repository authoring, specialization and runtime owners |
| Implementation and scientific qualification | Source and reproducible execution evidence, recorded per workflow |

A process-facing concept is a contract, role or view over the same underlying authored identities. It does not introduce a second object graph, equation store or equipment-building authority.

## Reading order

1. [Scope and modeling behavior](scope-and-modeling-behavior.md) — Product scope, capabilities and authoring behavior.
2. [Process structure and connectivity](structure-and-connectivity.md) — Definitions, instances, units, ports, streams, connections and multiple graph views.
3. [Materials, states, and physical models](materials-states-and-physical-models.md) — Components, phases, states, properties, reactions, inventories and balances.
4. [Cases, specifications, and studies](cases-specifications-and-studies.md) — Operating cases, specification roles, physical time and steps, atomic tasks and Study orchestration.
5. [Repository mapping and acceptance workflows](repository-mapping-and-acceptance.md) — Pinned repository mapping, implementation boundaries, acceptance workflow and the exchanger example.

## Structured catalogs

| Catalog | Version 1.1 contents | Purpose |
| --- | --- | --- |
| [Process concepts](../catalog/process-concepts.md) | 51 concepts across seven areas | Definitions, ownership, cardinality, relationships, invariants and implementation evidence |
| [IDAES workflow coverage](../reference/acceptance-workflows.md) | 12 prioritized workflows | Required contracts, acceptance checks, inspected source and remaining design cases |

Current revision: [Flowsheet Process Domain Model — v1.1](../rationale/version-records.md#source-40b141b5836981958b9ad5a14519520f). Historical v1 record: [Flowsheet Process Domain Model — v1](../rationale/version-records.md#source-40b141b583698185aee7fac52f7213f2).

## Scope

Included are steady and dynamic flowsheets; lumped, staged and distributed units; multiphase and reacting systems; batch and cyclic operation; heat/work/material coupling; controllers; and engineering analyses over those systems. Detailed local equipment physics can be composed when required. A two-dimensional solid-oxide-cell submodel, for example, can exercise a process integration boundary without making a general CFD engine part of the immediate product.

Full CFD, molecular simulation and electronic-structure engines remain specialized integrations. Their physical assumptions, capabilities and mapping to process observables must be explicit when used.

## Design commitments

1. Equipment composes zero or more balance regions, states, closures and subordinate models. Standard control-volume classes are optional reusable building blocks.
2. Material subject identity, selected characterization, model components, phase membership, composition/structure representation, state coordinates, transported quantities and stored inventories retain distinct contracts. Exact chemistry and empirical aggregate models expose their different evidence limits.
3. Ports expose existing typed quantities; connections declare coupling. Mixing, splitting, translation, pressure loss and storage have explicit physical models.
4. Physical stages and geometry belong to the engineering model. Numerical meshes, discretization and tear choices belong to execution.
5. The same model supports several tasks through explicit case bindings and Query roles, subject to its actual capabilities. Substitution is assessed for the requested outputs, physical conventions, validity, derivatives and effects; compatible ports alone do not prove equivalence.
6. Physical operating procedures, steps, lots and campaigns are distinct from Study orchestration and numerical initialization. Retained inventory, transfers, resets and resource assumptions remain explicit across physical transitions.
7. Stable semantic identities and original physical checks survive lowering, publication and diagnostics.
8. Solver convergence, physical acceptance, output coverage and comparison parity are separate evidence claims. Scientific data, observation meaning and economic/environmental accounting boundaries remain traceable.
9. Exposed residual equations, requested input/output evaluation and stateful external time advance declare different capabilities and lifecycle obligations. Repeated evaluations cannot silently advance external physical state.

## Evidence and change policy

The original v1 source assessment at [pse-arrow revision 9266555](https://github.com/paul-heyse/pse-arrow/tree/926655517f6eac4ab52ae7a817a7b72168afdc25) is preserved in its revision record. The v1.1 refinements and implementation plan use the inspected [revision 19e7b49](https://github.com/paul-heyse/pse-arrow/tree/19e7b49a67df24c1bf73005a6186e145e930d53a). The implementation plan pins IDAES-PSE 2.13.0, examples and compatibility snapshots separately; acceptance runs must retain their actual versions and environments.

No new simulator or parity run was performed for this design update. Coverage statuses distinguish source evidence from proposed cases. A catalog entry is a design contract, not a claim that its complete behavior is already implemented.

When a concrete workflow exposes a gap, first determine whether it needs a new authored scientific model, a stronger process-facing contract, or a genuinely missing generic semantic capability. Update the owning design and the acceptance case together. Preserve previous version records and the reason for the decision.

[Scope and modeling behavior](scope-and-modeling-behavior.md)

[Process structure and connectivity](structure-and-connectivity.md)

[Materials, states, and physical models](materials-states-and-physical-models.md)

[Cases, specifications, and studies](cases-specifications-and-studies.md)

[Repository mapping and acceptance workflows](repository-mapping-and-acceptance.md)

[Process concepts](../catalog/process-concepts.md)

[IDAES workflow coverage](../reference/acceptance-workflows.md)

## For agents

Process concepts: `collection://eec01849-ef45-4415-b032-65e8b0fec763` (Concept, Area, Definition, Owner, Cardinality, Relationships, Invariants, Implementation, Source URL).

IDAES workflow coverage: `collection://31e0a7f8-ef84-45d1-abe2-d979871a98c3` (Workflow, Contracts, Acceptance, Repository evidence, Priority, Status). The catalog's P0–P3 priority labels order its original 12 architecture witnesses. The implementation plan's P0–P7 phases are a separate delivery sequence. Status distinguishes inspected source from a design case; no new execution evidence is implied.

Relationships name concept types. Concrete occurrences retain the existing repository identity and ownership rules. Changes to a contract should update the corresponding narrative, catalog fields, acceptance cases and consequential decision record.

[Implementation design and IDAES equivalence plan](../delivery/index.md)

## Implementation planning

[Implementation design and IDAES equivalence plan](../delivery/index.md) — Adopted high-level plan on 2026-11-01. Applies this domain model to inspected pse-arrow revision `19e7b49a67df24c1bf73005a6186e145e930d53a` and the pinned IDAES 2.13.0 target. P0–P7 retains its scope and qualification owners; P1 now has four bounded delivery packets. The 55-family reference register remains source-inspected evidence, with held/archive entries distinguished.

For implementation, start with [Coding-agent implementation brief](../delivery/coding-agent-brief.md). The [Plan detailing and industry coverage assessment](../rationale/plan-detailing-assessment.md) maps I01–I08 into their live owners. [Industry coverage and expansion strategy](../delivery/industry-coverage-and-expansion.md) defines the broader direction and candidate increments without enlarging the IDAES denominator.
