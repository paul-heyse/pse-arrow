---
title: Architectural reading guide
status: current
---

# Architectural reading guide

### 0.1 What this collection is

> Decision: [ADR-0095](../../adr/0095-modular-documentation-and-search.md),
> [ADR-0096](../../adr/0096-current-rationale-and-selective-retirement.md) (both accepted).

The numbered pages in this directory are the authoritative description of the current
system: its contracts, why they are shaped that way, and their limits. Each numbered
section has exactly one owner. Section numbers are stable citation identities: add
subsections rather than renumbering, and keep a one-line retirement pointer when a cited
mechanism is removed. [`blueprint.md`](../blueprint.md) holds only the collection's
revision history and a table mapping the former single-file blueprint's anchors to their
current owners.

A page describes implemented behavior unless it labels a statement *Proposed* or states a
limit. ADRs record why a consequential choice was made; plans own active execution; reviews
are evidence. Superseded designs, completed plans and resolved reviews are not part of the
working corpus; Git history keeps them. Source inspection settles implementation questions.

### 0.3 Reading guide

| Task | Start with | Continue only as relevant |
|---|---|---|
| Understand the system | [Architecture overview](architecture-overview.md): scope, the native contract, layers and D1–D14 | The owning page for the area you change |
| Find what is supported | [Scope and open design §25–§26](scope-and-open-design.md) | The owning page's limits; [qualification basis §24.2](operations-and-validation.md#section-24-2) |
| Model authoring and composition | [Models and composition §10–§12, §22](models-and-composition.md) | [Physical semantics §8–§9](physical-semantics.md) |
| Mathematics and preparation | [Mathematics and compilation §7, §14](mathematics-and-compilation.md) | [Numerical execution §15–§18](numerical-execution.md) |
| Running cases, dynamics, fitting, Python | [Workflows and results §13, §19, §21](workflows-and-results.md) | [Numerical execution](numerical-execution.md) |
| Schemas, identity and storage | [Schema and relations §4, §6](schema-and-relations.md), [identity and publication §5, §20](identity-and-publication.md) | [Generated reference](../../generated/README.md) |
| Crates, libraries and pins | [Workspace and dependencies §3](workspace-and-dependencies.md) | [Capability maps](../../capability-maps/README.md) and the library skills |
| Errors, tests and qualification | [Operations and validation §23–§24](operations-and-validation.md) | [Development guides](../../dev/README.md) |
| Change a contract or review a design | [Design-change workflow §24.4](design-change-workflow.md#section-24-4) | Selected standard, affected section, suppliers/consumers and source |
| Resume or start work | [Current work](../../plans/README.md) | The active plan's packet and its contracts |
| Understand why a decision was made | The owning section, then its linked [ADR](../../adr/README.md) | Git history only when the retained rationale is insufficient |

Read the relevant Markdown directly; site navigation and search are conveniences. Use
module/entrypoint pointers and source inspection for implementation context; do not build a
parallel symbol inventory or proof manifest.

### 0.4 Conventions used in this collection

- Citations name stable identifiers: `blueprint §14.3`, `§D7`, `ADR-0082`, `AP-06`,
  `DP-09`, `PS-10`. A link to a section uses its published anchor, for example
  `sections/mathematics-and-compilation.md#section-14-3`.
- Relations are written `namespace.relation`; namespaces are `authored`, `reference`,
  `normalized`, `runtime` and `provenance`. Column detail belongs to the
  [generated reference](../../generated/README.md), not these pages.
- Equations are plain text; `d/dt` is the time derivative and `Σ_j` a reduction over the
  named domain.
- "Authored" means written by a model author or package; "derived" means produced by
  preparation and never authored; "reference" means shipped package data.
- IDAES names are quoted where the platform preserves them (for example
  `enth_mol_phase`) so that parity can map both ways ([§6.14](schema-and-relations.md#section-6-14)).
- Evidence labels (*Proposed*, *Implemented*, *Tested*, *Measured*) follow the core
  principles §D; *Tested* and *Measured* name the test or measurement and its conditions.
