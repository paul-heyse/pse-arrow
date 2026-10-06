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
