---
title: Design-change workflow
status: current
---

# Design-change workflow

### 24.4 Architecture review and design-change tracking

> Decision: [ADR-0094](../../adr/0094-architecture-first-design-review.md),
> [ADR-0096](../../adr/0096-current-rationale-and-selective-retirement.md) (accepted).

The [selected design standard](../../design_review/design_principles/standard.toml) governs
architecture review. Its core foundations organize assessment around separation of concerns,
contracts, composition, authoritative meaning, explicit constraints and local reasoning.
The core and profile own the detailed requirements; this section does not duplicate them.

Reviews start with the functional target, responsibility boundaries and representative change
scenarios. Architectural fitness and behavioral/scientific adequacy are settled independently;
passing one does not establish the other. Mechanism-level investigation follows material
uncertainty. Library eligibility under §3.3.1 remains; integration cost and dependency exposure
are assessed against the architecture. Internal contracts may evolve deliberately without
freezing an immature API, while durable/external compatibility remains explicit.

Reviews record the standard version and evidence scope. Their findings take effect through the
existing decision and plan routes. The owning plan tracks adopted finding dispositions and
links scenario definitions, decisions, packets and execution evidence. Indexes link current
status rather than copying it. Accepted ADR evidence records support at decision time;
historical observations are never silently relabelled as current qualification.

The [review template](../../design_review/design_principles/core/design-review-template.md) and
[repository binding](../../design_review/design_principles/binding/pse-arrow.md) define the
workflow and follow-up fields. Deterministic dependency/document checks support named facts;
there is no automated architecture score or generated review approval.

Documentation changes follow enduring responsibilities. A function-body change normally needs
no architecture edit. A contract change updates its owner and relevant product tests; a decision
change uses the existing ADR route. Read enough source to settle the claim, and run a focused
probe only when material uncertainty remains. Publishing checks establish document identity,
rendering and links; they do not prove design implementation. No documentation-specific source
seal, exhaustive symbol map or mandatory finding-to-test matrix is required.

When work closes, its enduring meaning moves to the owning section (and, for a consequential
choice, its rationale to an ADR); the plan leaves current work, and the completed plan and its
resolved reviews retire from the working tree once nothing depends on them. A decision whose
rationale no longer explains the current system is retired rather than rewritten. Git history
is the archive; retired material is not a backlog (ADR-0096).
