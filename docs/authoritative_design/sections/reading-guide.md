---
title: Architectural reading guide
status: proposed
---

# Architectural reading guide

### 0.1 What this document is

> Decision: ADR-0095 — proposed; implementation authorized by the maintainer.

The blueprint and numbered pages in this directory form the authoritative design collection.
Each numbered section has one owner. Section numbers are stable: add subsections rather than
renumbering. Vacated blueprint locations retain anchors and links, never a second normative copy.
The [blueprint revision history](../blueprint.md#revision-history) owns collection revisions.
ADRs record why decisions were made; plans own execution; reviews are historical evidence.

The current native process-simulator contract is [§0.5](../blueprint.md#05-current-native-process-simulator-contract),
with [§0.6](../blueprint.md#06-selected-data-model-foundation-amendments) covering selected
foundation amendments. Those precedence statements govern displaced historical mechanisms
elsewhere in the blueprint. Search classification does not establish implementation or authority.

### 0.3 Reading guide

| Task | Start with | Continue only as relevant |
|---|---|---|
| Understand the current architecture | [Current contract §0.5](../blueprint.md#05-current-native-process-simulator-contract) and [§0.6 amendments](../blueprint.md#06-selected-data-model-foundation-amendments) | Their cited decisions, then the affected owner in source |
| Resume implementation | [Current work](../../plans/README.md) | The owning plan's packet and its relevant contracts |
| Change a contract or review a design | [Design-change workflow §24.4](design-change-workflow.md#244-architecture-review-and-design-change-tracking) | Selected standard, affected section, suppliers/consumers and actual source |
| Understand physical/numerical requirements | [Process-simulator profile](../../design_review/design_principles/profiles/process-simulator/principles.md) | Relevant contract and scientific tests |
| Choose a library capability | [Capability maps](../../capability-maps/README.md) and the appropriate library skill | Outline first, then the pinned capability and real consumer |
| Investigate a past decision | [ADR index](../../adr/README.md) | Cited review/plan at its recorded scope and date |

Read the relevant Markdown directly. Site navigation and search are conveniences, not a
required agent toolchain. Use module/entrypoint pointers and source inspection for implementation
context; do not build a parallel symbol inventory or proof manifest.

### 0.4 Conventions used in this document

- Relation schemas are written as `namespace.relation @version` followed by columns as `name : ArrowType  [constraints]  -- meaning`. Arrow types use `arrow-schema` names. Semantic extension types are named `pse.<name>` and defined in §4.4.
- Equations are written in plain text; `d/dt` is the time derivative, `Σ_j` is a reduction over the named domain.
- "Authored" means written by a model author or package; "derived" means produced by a compiler pass and never authored; "reference" means shipped library data.
- IDAES names are quoted where the platform deliberately preserves them (for example the property name `enth_mol_phase`) so that parity tests can map both ways.
