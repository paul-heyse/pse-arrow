---
id: ADR-0080
title: Resolve port paths from typed inventories
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-13, DM-26, DM-28, DM-38, DM-43]
blueprint: [§6.7, §14.3]
review: docs/design_review/reviews/design_review_rust-computation-target-design_2026-09-23.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A port binding needs a traversal effect or source-coordinate meaning absent from the admitted typed inventories.
verification: Plan 13 W07 isolated port path units and fixed inventory-read counts; just unit-rust-computation; just codegen-contracts-check; just adr-lint.
---

# ADR-0080: Resolve port paths from typed inventories

## Context

The W07 deletion audit found the P5 port frontier still preparing and executing
several queries per source path position (target-design F04). The operation follows
named children in an already finite prospective instance forest.

## Scope

Replace that traversal with a bounded typed inventory walk. Declare
`inferred.port_walks` as its registry-owned projection, carrying actual target
instance/template, ordered index and domain IDs. P5 retains set-oriented member,
quantity, product and guard joins. This amends blueprint §14.3 within ADR-0076.

## Drivers

Depth-independent native inventory reads, exact ordered coordinates and child
multiplicity, complete source witnesses, finite memory admission and cancellation.

## Options

A repeated native frontier pays preparation cost per step. An unregistered graph
sidecar hides inputs. A typed walk with an explicit registered projection preserves
the relational handoff and removes per-step queries.

## Outcome

Admit complete typed instance, declaration, step and domain inventories once. Resolve
only enabled port roots using exact indexed lookups. Every result retains its actual
source occurrences and read scopes; no fabricated row proves a traversal step.

### Consequences

The registry gains one derived port-path product. Its native producer precedes the
existing port/member joins. W12 determines selected durable publication; the row
contract does not require a new independent cache or graph persistence format.

### Compensating controls

Complete forest admission; missing/ambiguous declaration and incompatible-index
refusal; ordered repeated axes; actual source key association; fallible allocation
before expansion; cancellation checkpoints; isolated chain/branch/empty controls.

### Confirmation

Targeted units establish path semantics and fixed inventory execution counts.
Whole compiler equivalence remains W19 and timing/RSS evidence remains W20.

## Pros and cons

One extra declared projection makes the boundary explicit. It removes query-per-step
cost without introducing a second source identity or an ad hoc physical provider.

## More information

Plan 13 W07, target-design F04, ADR-0076 and ADR-0079. The formal decision/design PR
must include the blueprint revision; this proposed record is not formal acceptance.

## Status history

- 2026-09-23 — proposed before the port-walk relation and producer changes.
