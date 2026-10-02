---
title: Design-change workflow
status: current
---

# Design-change workflow

### 24.4 Architecture review and design-change tracking

> Decision: [ADR-0094](../../adr/0094-architecture-first-design-review.md),
> [ADR-0096](../../adr/0096-current-rationale-and-selective-retirement.md),
> [ADR-0129](../../adr/0129-domain-model-review-scope.md),
> [ADR-0149](../../adr/0149-agent-permissions-by-contract.md).

The [selected design standard](../../design_review/design_principles/standard.toml) governs
architecture review. Its core foundations organize assessment around separation of concerns,
contracts, composition, explicit domain models and scoped semantic authority, constraints and
local reasoning. The principles and review skill own the domain-model criterion; assess it
within bounded design/review periods at the binding's cadence. AGENTS.md routes to that process
without imposing a standing modeling mandate. AP-04/G9 retain model adequacy and authoritative
behavior as acceptance criteria. This review policy is Implemented (2026-09-30).

Reviews start with the functional target, modeled phenomena and owned domain operations,
responsibility boundaries and representative change scenarios. Investigation depth follows the
scoped question; flow tracing is optional where it resolves a concrete uncertainty.
Architectural fitness and behavioral/scientific adequacy are settled independently;
passing one does not establish the other. Mechanism-level investigation follows material
uncertainty. Library eligibility under §3.3.1 remains; integration cost and dependency exposure
are assessed against the architecture. Internal contracts may evolve deliberately without
freezing an immature API, while durable/external compatibility remains explicit.

The [process skills](../../../.codex/skills/README.md) own review planning, plan authoring and
execution. Each has a conversational planning companion and an action skill; companions use an
available planning interface without switching runtime modes or creating another durable ledger.
`create-plan` assesses the affected foundations as part of authoring the target and dependency
sequence. That focused assessment does not certify an enclosing subsystem or initiate recurring
model assessment during ordinary implementation.

The [shared roles](../../../.agents/roles/README.md) own reusable worker behavior, coordination
and each role's permitted effects; the separate native adapters own model and effort defaults
only. Codex roles inherit the session's sandbox and approval settings (Codex ignores sandbox keys
in role files) and Claude roles the session's tools, so no adapter declares either. `library-research` may write new dated folders
in the evidence locations AGENTS.md names and the shared library skill its brief assigns, and
lists every file written; the other evidence roles stay read-only. These scopes are instructions,
not runtime enforcement; the edit-protection hook still applies to every role. The root coordinator retains
design decisions, integration and acceptance; executors choose local details inside their brief.
Delegate when independent work, context isolation, distinct capabilities or independent judgment
justify coordination and integration. Small or tightly coupled tasks may stay with the root.
Briefs name settled decisions, baseline, permitted effects, sibling ownership and expected evidence;
workers load task-relevant authority and follow discovered dependencies. Evidence conflicts,
consequential absence claims, unsupported version transfers, ownership uncertainty and repeated
repair failure are surfaced to the coordinator, who chooses the least sufficient resolution route.
Reviews remain proportional to the binding; fresh reviewers receive concrete criteria and consumers.
Process skills are canonical in `.codex/skills/`, with `.claude/skills` and `.agents/skills` as
aliases. The existing alias synchronizer never generates or modifies native agent definitions.
These workflow/configuration surfaces are Implemented (2026-09-30; evidence write scope 2026-10-01). The selected defaults raise
Codex implementation review to Sol/high and Claude evidence roles to Sonnet/medium, while Claude
design review uses Opus/high. Model-allocation quality and native agent behavior remain unmeasured;
no calibration exercise or recurring telemetry is required. The current-work index routes to the active plan or packet
checkpoint for execution and handoff; no root STATUS file or second status owner is introduced.

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
