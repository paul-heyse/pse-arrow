# Architecture

Start with the [architectural reading guide](sections/reading-guide.md). It routes by task
and explains section ownership, current contract precedence and historical context.

- [Current native contract, §0.5](blueprint.md#05-current-native-process-simulator-contract).
- [Selected foundation amendments, §0.6](blueprint.md#06-selected-data-model-foundation-amendments).
- [Design-change workflow, §24.4](sections/design-change-workflow.md#244-architecture-review-and-design-change-tracking).
- [Current work](../plans/README.md) and [decisions](../adr/README.md).
- [Blueprint and revision history](blueprint.md): retained sections, including displaced historical mechanisms.

The blueprint plus numbered pages under `sections/` are one authoritative collection.
Cite stable identifiers such as `blueprint §14.3`; never renumber when moving a section.
The ADR linter resolves each identifier to its unique owner. Old locations retain anchor links.
The rendered book derives a section directory from those same headings.

ADRs explain decisions, reviews record observations, and plans own execution. Intended design,
observed implementation and historical qualification remain different claims. Source inspection
settles implementation questions; a generated publication index does not.

Amend this collection through the existing decision/design route, with a revision row in the
blueprint and an inline decision reference at the affected section. ADR-0095 records the
maintainer-authorized modularization; formal supersession of ADR-0033/0036 remains pending.
Extract further subsystem sections when substantive work next changes their contracts.
