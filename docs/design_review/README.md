# Design standard and reviews

The [core principles](design_principles/core/design-principles.md) organize review around
seven architectural foundations, including supported-workload execution fit through G9. Use them
together with [Heuristics for Efficient Architecture](design_principles/core/efficient-architecture-heuristics.md)
for consequential choices and material execution mismatches while the design remains easy to change.
The companion guides qualitative judgment without a whole-list checklist or new proof machinery. The [template](design_principles/core/design-review-template.md)
defines the review contract; [standard.toml](design_principles/standard.toml) selects its
versions and profiles. The [repository binding](design_principles/binding/pse-arrow.md)
provides local authority and scenario routes.

Start with the [design-change workflow](../authoritative_design/sections/design-change-workflow.md)
and the affected contract. Reviews are observations, not implementation certificates; adopted
findings have one disposition owner in the relevant plan. A review stays here while a finding
or a pending decision depends on it and then retires to Git history (ADR-0096); retained ADRs
cite retired reviews as `git:<commit>:<path>`.

The [simulation pipeline efficiency and effectiveness review](reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md)
reassesses Plan39's seven diagnostic-audit items across preparation, numerical solving,
workflow composition and result consumption. Its fourteen findings and conditional library
opportunities distinguish avoidable work from numerical strategy and preserve scientific
assessment. The [supporting inquiries](evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/README.md)
include exact library fit and a fresh independent integrated challenge. The decision is Revise;
new recommendations and rule impacts await future plan creation. Original audit status stays
with 39d, and Plans39/40 remain independently active.

The [simulation performance diagnostics review](reviews/design_review_simulation-performance-diagnostics_2026-10-14.md)
assesses live current-work inspection, recent-window progress, causal work attribution and
profiling across preparation, numerical methods, nested workflows and managed execution.
Its [supporting evidence](evidence/simulation-performance-diagnostics-2026-10-14/README.md)
includes context-specific analysis, exact library capabilities, instrumentation boundaries
and bounded workstation profiling probes. [Plan 40](../plans/40-simulation-performance-diagnostics.md)
owns adopted F01–F06, the accepted RC01 route and future implementation/qualification.
The review does not restart benchmarking or alter other active plans' scope or status.

The [implementation friction and Codex effectiveness review](reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md)
examines the final October 14 implementation interval and its operational logs. It traces
startup ownership, local preparation and replay work, producer assurance and capture lifetimes,
and command diagnostics. [Plan 39](../plans/39-implementation-friction-and-codex-effectiveness.md)
owns its proposed corrections and rule decisions, with all remaining Plan 38 acceptance
sequenced last. Plan authoring does not restart or close that campaign. The
[observed episodes](evidence/implementation-friction-and-codex-effectiveness-2026-10-14/observed-episodes.md)
retain the historical evidence boundaries and route to the independent assessment.

The [independent efficiency and solver target audit](reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md)
assesses every finding and principal/nested rule impact in the consolidated review. It supports
local immutable preparation and a common solver-policy seam while correcting existing-capability
and native-library claims, separating topology choices from numerical prerequisites, and adding
native update-grain and retained-policy qualifications. Its [supporting inquiries](evidence/efficiency-and-solver-target-audit-2026-10-12/README.md)
include a fresh independent target challenge and comparative coverage of all Plan 36 packets.
The verdict is Revise; rule decisions and disposition remain for later plan creation.

The [efficiency and solver target review](reviews/design_review_efficiency-and-solver-target_2026-10-12.md)
consolidates three bodies of work after Plan 37:
- the efficiency alignment review, audit and remaining Plan 36 scope;
- the solver methods review;
- the SurrealDB functionality that now exists.

It assesses the whole simulator holistically, from workload obligations and the capabilities of
the libraries already in use. A clean-sheet allocation and two independent challenges informed
its Revise verdict. It records fourteen findings and eleven proposed rule impacts, the central
ones being:
- where the working authority lives: a live model, a local journal and an immutable record;
- how many processes write durable state;
- reuse and structure ownership;
- lane-owned allocation reuse with declared warm edges;
- placement by shared native resources.

Its [supporting evidence](evidence/efficiency-and-solver-target-2026-10-12/README.md) holds the
obligations ledgers, library-responsibility surveys, clean-sheet allocations, challenges and the
detailed [solver target](evidence/efficiency-and-solver-target-2026-10-12/solver-target.md). That
target restates the solver review below for the post-efficiency codebase. Disposition awaits plan
creation. The review classifies the remaining Plan 36 packets and maps the earlier findings
without editing their owners.

The [solver methods, selection logic and diagnostics review](reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md)
assesses how the solve pipeline chooses mathematical methods across classes, blocks, recycles and
dynamics, and how solver observations serve switching, diagnosis and measurement. It works from
the maintainer's three-layer selection requirement. Its Revise verdict identifies thirteen findings
and thirteen proposed rule impacts. The [supporting evidence](evidence/solver-methods-and-diagnostics-2026-10-12/README.md)
holds library inventories at the pins, the method landscape and an independent assessment.
Disposition awaits plan creation.

[Plan 37](../plans/37-surrealdb-lifecycle-and-integration.md) records the resolved lifecycle
findings, confirmed rule decisions, bounded capability outcomes and affected acceptance.
Its principal lifecycle review, B1 target review, remaining-target review and supporting
evidence are retired to Git history with immutable references in that owner and ADR-0181/0182.
This acceptance does not close the separate Plan 28 or Plan 36 qualification campaigns.

[Plan 35](../plans/35-surrealdb-lifecycle-and-integration-remediation.md) owns the resolved
SurrealDB lifecycle/integration findings, accepted RC01/RC02 and bounded capability decisions.
Its [acceptance evidence](../plans/35-surrealdb-lifecycle-and-integration-remediation.md#acceptance-evidence)
records affected implementation and actual recovery controls. The resolved source review is
retired to Git history with immutable references in that owner; Plan 28 retains its separate
comprehensive campaign qualification.

The [unified state-management and persistence review](reviews/design_review_unified-state-management-and-persistence_2026-10-10.md)
assesses active memory, caches, identification, canonical SurrealDB operations and host/native
lifecycles together, including the recent campaign failures. Its Revise verdict identifies
nine findings and distinguishes source-backed defects, test-oracle mismatches and unresolved
causes. [Plan 34](../plans/34-unified-state-management-and-persistence.md) owns the integrated
target, resolved finding dispositions, investigation decisions and affected qualification independently of Plan 28;
its [Outcome](../plans/34-unified-state-management-and-persistence.md#outcome-recorded-after-implementation)
records corrections, actual acceptance and exclusions. The review retains its original observation scope.
[Plan 28e](../plans/28e-rebuild-retirement-and-qualification.md) retains the existing campaign's
qualification status.

The [codebase efficiency-principles review](reviews/design_review_efficiency-principles-codebase_2026-10-09.md)
assesses build/test turnaround and simulator runtime against all seven architectural
foundations, with particular attention to execution fit. It distinguishes concrete work
amplification and qualification-applicability defects from unmeasured opportunities, and
preserves the recent preparation, native-lifetime and concurrency corrections. Its
[supporting inquiries and projection probe](evidence/efficiency-principles-codebase-2026-10-09/README.md)
document the inspected baseline and evidence limits.
[Plan 33](../plans/33-efficiency-principles-remediation.md) now independently owns the
adopted findings, six investigation avenues and affected qualification. Its confirmed
design-phase directions replace the review's proposed preservation of retired analysis
lineage and obsolete internal history. Its completed
[Outcome](../plans/33-efficiency-principles-remediation.md#outcome-recorded-after-implementation)
owns implemented remedies, affected acceptance and exclusions; the review retains its original scope.

The [workspace content-lifecycle review](reviews/design_review_workspace-content-lifecycle_2026-10-09.md)
assesses document relevance, publication, agent discovery and repo-linked storage.
It proposes typed lifecycle metadata, explicit asset delivery and producer-owned disposal,
with a bounded disposition inventory and concrete retirement candidates. The [scoped target assessment](reviews/design_review_workspace-content-lifecycle-target_2026-10-09.md)
accepts the revised target at Proposed strength. [Plan 32](../plans/32-workspace-content-lifecycle.md)
owns implementation and finding disposition; target acceptance is not implementation acceptance.

The [structured documentation and evidence lifecycle review](reviews/design_review_structured-documentation-lifecycle_2026-10-09.md)
assesses the agent effort spent on the mechanics around reviews, plans, evidence and guidance.
It does not assess the content itself, whose flexible format it keeps. It recommends removing
the external lifecycle overlay and the restated publication lists. It adds an on-demand plan
query that returns the sections to update with their paired skill guidance, and an optional
evidence runner that records what each probe used and reports whether prior answers still hold.
Its rule impacts and proposed owner await plan creation; the
[supporting inquiries, exhibits and probes](evidence/structured-documentation-lifecycle-2026-10-09/README.md)
record their baseline and limits.

The [graph compilation, reusable kernels and hashing review](reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md)
assesses further graph preparation, fast hashing and their combined reuse/invalidation role.
It recommends indexed description and tear-policy access plus retained implicit-provider
topology, preserving existing exact preparation reuse and fresh execution effects.
Its [Plan 28 integration](../plans/28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration)
owns finding dispositions and links correction owners.
[28k's followup outcome](../plans/28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09)
records implementation and bounded followup inquiries, including the typed-checker proposal and
retained portable reconstruction. Broader durable fast fingerprints and persistent runtimes
remain unselected; inquiry evidence does not qualify the enclosing Plan 28 campaign.

The [FlowProjectionV2 review](reviews/design_review_flow-projection-v2_2026-10-09.md)
accepts ADR-0167's bounded tagged/count-framed target. The maintainer selected original
Graph/hash RC04; implementation and consumer acceptance remain at Plan 28k. This supplies
explicit collection/parent boundaries without claiming complete physical-registry equivalence.

The [agent effectiveness enhancement review](reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md)
assesses the personal Codex environment, recent activity, command interfaces,
persistent SurrealDB inspection and optional Rust navigation. Its findings and
candidate enhancement decisions and resolved findings belong to
[Plan 31](../plans/31-agent-effectiveness-enhancements.md), including the accepted
instruction changes, native MCP choice and exclusion of semantic Rust search. Its
[supporting evidence](evidence/agent-effectiveness-enhancements-2026-10-09/README.md)
distinguishes focused interface controls from product qualification. The completed plan
records implementation and qualification; these newly authored inputs remain for the
uncommitted handoff until an immutable Git archive exists, rather than as active work.

The [unified SurrealDB simulation-substrate follow-up](reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md)
examines canonical problem, dependency, run and result storage with durable runs by default,
targeted compilation and connected pre/post-simulation queries. It reassesses every finding
and the architectural recommendation of the earlier efficiency review.
Current accepted changes, work sequence and dispositions for both reviews belong to
[Plan 28](../plans/28-surrealdb-unified-substrate.md). Its confirmed clean rebuild/native-query
decisions supersede the review's legacy-preservation and optional SQL transition assumptions.

The earlier [execution-efficiency and SurrealDB review](reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md)
assesses build turnaround, model preparation and execution, source and study lifecycles, and
database alternatives against Core 3.4 and the efficiency heuristics. Its recommendations
are review evidence, not an implementation authorization.

`evidence/` keeps supporting investigations and pinned Arrow/DataFusion characterization
probes that the [capability maps](../capability-maps/README.md) cite for library behavior.
