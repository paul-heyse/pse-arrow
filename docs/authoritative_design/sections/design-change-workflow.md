---
title: Design-change workflow
status: current
---

# Design-change workflow

### 24.4 Architecture review and design-change tracking

> Decision: [ADR-0094](../../adr/0094-architecture-first-design-review.md),
> [ADR-0096](../../adr/0096-current-rationale-and-selective-retirement.md),
> [ADR-0129](../../adr/0129-domain-model-review-scope.md),
> [ADR-0149](../../adr/0149-agent-permissions-by-contract.md),
> [ADR-0162](../../adr/0162-workload-fit-review-principles.md).

The [selected design standard](../../design_review/design_principles/standard.toml) governs
architecture review.

Use the [design principles](../../design_review/design_principles/core/design-principles.md) together
with [Heuristics for Efficient Architecture](../../design_review/design_principles/core/efficient-architecture-heuristics.md)
for consequential architectural and implementation choices. Consider relevant execution patterns
before committing to physical organization, interfaces, preparation, assurance and lifecycles;
address material mismatches while the design remains easy to change. The companion supports
AP-07/G9 qualitatively without a new gate, exhaustive checklist, cost models or proof machinery.
During execution, apply it to choices left open or exposed mismatches without restarting settled
reviews.

Its core foundations organize assessment around separation of concerns, contracts, composition,
explicit domain models and scoped semantic authority, constraints and
local reasoning and execution fit for supported workloads. The principles and review skill own the domain-model criterion; assess it
within bounded design/review periods at the binding's cadence. AGENTS.md routes to that process
without imposing a standing modeling mandate. AP-04/G9 retain model adequacy and authoritative
behavior as acceptance criteria. This review policy is Implemented (2026-09-30; execution-fit adoption 2026-10-05).

Reviews start with the functional target, modeled phenomena and owned domain operations,
responsibility boundaries and representative change scenarios. Investigation depth follows the
scoped question; flow tracing is optional where it resolves a concrete uncertainty.
Architectural fitness and behavioral/scientific adequacy are settled independently;
passing one does not establish the other. Mechanism-level investigation follows material
uncertainty. Library eligibility under §3.3.1 remains; integration cost and dependency exposure
are assessed against the architecture. Internal contracts may evolve deliberately without
freezing an immature API, while durable/external compatibility remains explicit.

AP-07 adds qualitative execution-fit assessment through existing G9 in Core 3.4;
ProcessSimulator 1.5 applies it to
edit/re-solve, studies, recycles and dynamics while retaining PS-01–PS-13, PS-G1–PS-G3 and the
PS-09 solver-composition contract. Supported workload premises and relevant growth/skew,
concurrency and interruption scenarios belong in existing scope/scenario discussion. Semantic
model ownership does not dictate physical layout, placement, scheduling or enforcement frequency.
Compare complete necessary work with incidental scans/crossings/materialization, live state,
reuse/invalidation, economical assurance, transaction/visibility and retry units, coordinated
capacity, consumer views and total generated/library-induced obligations. Repeated producer work
is not independent assurance; distinct state/trust failures and required scientific post-checks
remain covered. Explain material tradeoffs and retained benefits in plain language.

Editorial clarification (2026-10-05): assess execution fit qualitatively from relevant operations
and growth/failure scenarios. This design/review consideration does not require numerical
estimates, cost models, estimators, runtime cost accounting, execution-planning machinery,
instrumentation, formal cost proofs or additional proof artifacts. Introduce such mechanisms
only for a separate concrete functional or operational requirement. Quantitative performance or
capacity claims still require measurements; existing semantic correctness obligations remain.
This clarifies the existing assessment scope without a new decision, standard version or
production work.

Static evidence can establish a structurally unfit route; quantitative speed and capacity require
measurement. Safe refusal alone does not establish supported-workload fit. Local passing slices
do not establish assembled fitness, and a material unresolved execution premise prevents scoped
acceptance. This is a bounded architectural judgment, not a new A4/G10, checklist, lint, probe
mandate, benchmark gate or ordinary-implementation review. Historical reviews keep their recorded
versions, scope and evidence. The adoption changes policy and process-skill guidance only; it
claims no product performance or newly qualified numerical behavior. Independent static review
accepted the written policy at scoped strength on 2026-10-05, including retained new-state checks,
necessary global work and drain/freeze before reconciliation. `just docs`, `just lint-agents` and
`just adr-lint` passed at the adoption baseline; the initial stale-index failure cleared after
regeneration. A later whole-tree ADR check failed on concurrently introduced ADR-0163's missing
review and changed index inputs; final docs/agent checks passed. That draft is outside adoption.
Current product qualification stays with its existing plan/packet owners. The policy is Implemented
(2026-10-05).

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
Codex implementation review to Sol/high and run Codex library research on Sol/medium; Claude runs
code mapping on Sonnet/medium, library research, execution, implementation review and testing on
Opus/medium, and design review on Opus/high. Model-allocation quality and native agent behavior remain unmeasured;
no calibration exercise or recurring telemetry is required. The current-work index routes to the active plan or packet
checkpoint for execution and handoff; no root STATUS file or second status owner is introduced.

Reviews record the standard version and evidence scope. Their findings take effect through the
existing decision and plan routes. The owning plan tracks adopted finding dispositions and
links scenario definitions, decisions, packets and execution evidence. Indexes link current
status rather than copying it. Accepted ADR evidence records support at decision time;
historical observations are never silently relabelled as current qualification.

### 24.4.1 Workspace content lifecycle

> Decision: [ADR-0168](../../adr/0168-workspace-content-lifecycle.md).

The maintainer-selected target uses one tooling-owned interpretation of document metadata,
native family meanings and unchanged authored bodies. Collection/bundle defaults and
sparse exceptions identify role, provenance, owner and retention without creating deletion
authority or changing the substantive plan-authoring guidance. Plan content management
starts at retained Plan 28+ documents; obsolete pre-28 plans are removed without salvage,
open-item reconciliation or scope import. Necessary historical references use immutable
originals under ADR-0096.

Multiple plans may remain active and execute concurrently. Each packet/finding keeps its
native status owner. Explicit content bindings derive aggregate scope and cross-plan
dependency/supersession views; they contain no independently editable status or execution
authorization. Similar names do not establish equivalent scope, and implementation is
not automatically qualification or completion. Missing/ambiguous interpretation is visible.

Publication selects assets independently of evidence retention. Documentation bootstrap
provisions a dedicated minimal environment explicitly; normal reads neither synchronize
the product nor download tools. The existing producer resource owner seals artifact roles
to finalized identity and protects retained provenance, references and active borrowers.
Only proven disposable artifacts with truthful reconciled outcome and actual drain may
be reclaimed through exact reservations and identity rechecks. Unknown legacy resources
remain protected; no universal pruning service or routine housekeeping gate is introduced.

Plan 32 owns implementation and scoped acceptance. This target adoption supplies no
scientific qualification and absorbs none of Plan 28's scope.

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
