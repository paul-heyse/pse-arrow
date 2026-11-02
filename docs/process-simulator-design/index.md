---
title: "Process simulator design"
---

# Process simulator design

The Notion design collection is organized here for repository-based design and implementation planning. Start with the focused handoff, then follow the relevant contracts and detailed references. Source links and original decision/version records are preserved.

The [holistic target design review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md)
assesses this package against current source and the selected design standard. Its proposed
amendments, alternatives, scientific-library evidence and rule impacts are input to subsequent
plan creation; the review does not itself adopt changes or establish scientific qualification.

## Start here

1. [Planning handoff for GPT-6.1 Sol](planning-handoff.md) — the outcome to scope, settled direction, first delivery, consequential questions and expected planning output.
2. [Import and organization strategy](import-strategy.md) — document responsibilities, architecture integration, migration sequence and maintenance.
3. [Delivery program](delivery/index.md) and [coding-agent brief](delivery/coding-agent-brief.md) — P0–P7 and the four first-delivery packets.
4. [Scientific sourcing and catalog](delivery/scientific-sourcing-and-catalog.md) — targeted native Rust intake, existing SurrealDB ownership and compositional equipment.

## Design and reference map

| Area | Read for |
| --- | --- |
| [Simulation domain v3](simulation/index.md) | Model, Specification, Query, atomic problems, Study, execution and qualified results |
| [Process domain v1.1](process/index.md) | Equipment composition, materials, states, connections, cases and operating meaning |
| [Cross-industry contracts](process/cross-industry-contracts.md) | Characterization, populations, lots, procedures, external capabilities and assessment boundaries |
| [Scientific package and equipment delivery](delivery/scientific-packages-and-equipment.md) | Property, reaction, state, equipment and engineering assessment obligations |
| [Industry coverage and expansion](delivery/industry-coverage-and-expansion.md) | Nine industry groups and candidate capability increments |
| [Simulation concept catalog](catalog/simulation-concepts.md) | 100 concepts, grouped by layer, with ownership and relationship detail |
| [Process concept catalog](catalog/process-concepts.md) | 51 concepts, grouped by seven areas, with invariants and implementation leads |
| [Answer kinds](reference/answer-kinds.md), [task classes](reference/task-classes.md), [canonical forms](reference/canonical-forms.md), [kernel kinds](reference/kernel-kinds.md) | Detailed formulation and execution reference records |
| [Acceptance workflows](reference/acceptance-workflows.md) | The original 12 architecture witnesses and their full acceptance criteria |
| [Notebook families](reference/notebook-families.md) | 55 documented families, held/archive dispositions and coverage beyond notebooks |
| [Design decisions](rationale/design-decisions.md) and [version records](rationale/version-records.md) | 24 original decisions, six version records and their rationale |
| [Detailing assessment](rationale/plan-detailing-assessment.md) | I01–I08 and their incorporation map |
| [Source map](source-map.md) and [import verification](import-verification.md) | Exact source-to-file correspondence and import coverage |

Use this collection for continued design work. The imported [Notion index](notion-design-index.md) preserves the source workspace's organization; its editing and database-query directions describe that original workspace. Internal page, concept and database links now lead to repository files.

The existing [architecture](../authoritative_design/README.md), [current work](../plans/README.md), [selected design standard](../design_review/design_principles/standard.toml) and [agent instructions](../../AGENTS.md) retain their established responsibilities. The [strategy](import-strategy.md) explains how to carry the design into those owners as implementation planning proceeds.
