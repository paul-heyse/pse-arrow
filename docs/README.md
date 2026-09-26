# pse-arrow

A process systems engineering core in Rust, with typed process definitions, library-owned
mathematics and native solvers. Arrow, DataFusion and Delta provide data-boundary, relational
and storage capabilities. See the [relationship to IDAES](relationship-to-idaes.md).

| I want to… | Start here |
|---|---|
| Understand the architecture | [Architecture and reading guide](authoritative_design/README.md) |
| Know what is supported | [Scope and open design](authoritative_design/sections/scope-and-open-design.md) |
| Find active work | [Current work](plans/README.md) |
| Review or change a design | [Design-change workflow](authoritative_design/sections/design-change-workflow.md) |
| Research a pinned library | [Capability maps](capability-maps/README.md) |
| Understand a decision | [Decision records](adr/README.md) |
| Work on documentation | [Publishing documentation](dev/documentation.md) |

Search defaults to **Current**: the architecture, development guides, the design standard and
active work. **Reference** includes library maps, generated schemas, proposed decisions and the
blueprint's former-anchor table. **History** holds only a few deliberately retained records;
completed plans and resolved reviews are retired to Git history and are not published.
**Everything** searches all published chapters. Scope describes reading purpose, not
qualification. The chapter footer identifies the collection and any recorded lifecycle status.

Markdown is the canonical source for readers and agents. Follow a task route, read the relevant
contract, and inspect the implementation needed for the change. Qualification results keep
their original conditions; only named tests and measurements support Tested and Measured claims.
