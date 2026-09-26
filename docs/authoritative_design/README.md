# Architecture

Start with the [reading guide](sections/reading-guide.md), then the
[architecture overview](sections/architecture-overview.md). These pages describe the
current system, why it is shaped that way and its limits:

| Page | Owns |
|---|---|
| [Architecture overview](sections/architecture-overview.md) | §0 scope, §0.5 native contract, §0.6 data-model foundation, §1 layers, §2 D1–D14 |
| [Workspace and dependencies](sections/workspace-and-dependencies.md) | §3 crates, library roles, pins and dependency policy |
| [Schema and relations](sections/schema-and-relations.md) | §4 registry and generation, §6 relation families |
| [Identity and publication](sections/identity-and-publication.md) | §5 identity and hashing, §20 publication, retention and compatibility |
| [Mathematics and compilation](sections/mathematics-and-compilation.md) | §7 library-owned mathematics, §14 preparation and reuse |
| [Physical semantics](sections/physical-semantics.md) | §8 physical typing, §9 materials, providers and reactions |
| [Models and composition](sections/models-and-composition.md) | §10 balances, §11 templates, §12 connectivity, §22 authoring |
| [Numerical execution](sections/numerical-execution.md) | §15 structure, §16 numerical policy, §17 initialization, §18 native solving |
| [Workflows and results](sections/workflows-and-results.md) | §13 dynamics, §19 cases and fitting, §21 Python boundary |
| [Operations and validation](sections/operations-and-validation.md) | §23 observability and failures, §24 tests and qualification basis |
| [Design-change workflow](sections/design-change-workflow.md) | §24.4 architecture review and design changes |
| [Scope and open design](sections/scope-and-open-design.md) | §25 supported scope, §26 risks and open choices, capability coverage, glossary |

Cite stable identifiers such as `blueprint §14.3`; never renumber when moving a section.
The ADR linter resolves each identifier to its unique owner, and the rendered book derives
a section directory from the same headings. [`blueprint.md`](blueprint.md) keeps the
revision history and maps former single-file anchors to their owners.

ADRs explain decisions, reviews record observations, and plans own active execution.
Amend this collection through the existing decision/design route, with a revision row in
the blueprint and an inline decision reference at the affected section (ADR-0095,
ADR-0096). A function-body change normally needs no edit here; a contract change updates
its owner.
