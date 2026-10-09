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

The [graph compilation, reusable kernels and hashing review](reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md)
assesses further graph preparation, fast hashing and their combined reuse/invalidation role.
It recommends indexed description and tear-policy access plus retained implicit-provider
topology, preserving existing exact preparation reuse and fresh execution effects.
Its [Plan 28 integration](../plans/28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration)
owns the Open finding dispositions and links the proposed corrections at their existing
companions. [28k](../plans/28k-graph-kernels-and-hashing-investigations.md) plans bounded
graph/hash investigations; broader fingerprints and persistent graph products remain
conditional alternatives. Plan publication does not authorize product execution or qualify
the proposed corrections.

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
