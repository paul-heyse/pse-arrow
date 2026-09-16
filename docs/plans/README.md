# Plans

A plan records **how work is sequenced and verified**. A decision record
([`../adr/`](../adr/)) records **what was decided and why**. The two
have different lifecycles, which is why they live in different directories.

| Plan | Covers | ADRs | Status |
|---|---|---|---|
| [01 — Repository configuration](01-repository-configuration.md) | Repository identity, layout and migration, the Rust workspace and supply chain, the solver container, Python packaging and tests, the CI job graph, decision records, GitHub setup, the agent environment, the docs site | ADR-0001 – ADR-0038 | in-progress |
| [02 — Blueprint revision 5 contracts](02-blueprint-revision-5-contracts.md) | Design amendment for R4-01–R4-13 and L1–L9; separate Proposed implementation handoff | ADR-0039 – ADR-0048 | done (documentation; runtime handoff Proposed) |
| [03 — Wave 1 foundations](03-wave-1-foundations.md) | Historical foundation implementation and receipts; remaining scope carried into Plan 05 | ADR-0039 – ADR-0059 | abandoned execution strategy; superseded by 05 |
| [04 — Wave 2 semantic compilation](04-wave-2-semantic-compilation.md) | Complete graph/reference/inspection requirements retained by Plan 05; old execution strategy superseded | Contracts through ADR-0065 | abandoned execution strategy; superseded by 05 |
| [05 — Native logical-plan hard pivot](05-native-logical-plan-hard-pivot.md) | Historical implementation and receipts; every unfinished HP00–HP13 outcome carried into Plan 06 | Affected contracts through ADR-0067 | abandoned execution sequence; superseded by 06 |
| [06 — Provider-contract hard pivot](06-provider-contracts-hard-pivot.md) | Historical provider implementation and receipts; useful functions re-scoped to the unified simulator target | Inherited contracts through ADR-0067 | abandoned execution sequence; superseded by 07 |
| [07 — Unified DataFusion and Delta Lake hard pivot](07-unified-datafusion-delta-hard-pivot.md) | Broader simulator inventory and partial native/Delta implementation receipts; current architecture work re-scoped into Plan 08 | ADR-0065, ADR-0066, proposed ADR-0068 | partial implementation; execution sequence replaced by 08 for the newly narrowed scope |
| [08 — Schema-first native data pivot](08-schema-first-native-data-pivot.md) | Complete schema engineering and native DataFusion/Delta replacement of existing functions; legacy deletion; no new simulator functionality | ADR-0065, ADR-0066, proposed ADR-0068 and ADR-0069 | in-progress; initial field/durable boundary tested, recursive registry and later cuts open |

**Current execution sequence:** Plan 08, following
the maintainer's 2026-09-16 scope change. Schema engineering precedes complete replacement
of the existing codebase's data operations, storage and callers with native DataFusion
and Delta. One implementation stream deletes replaced mechanisms with each cut; there
is no compatibility path, historical-object migration or transition period.

Implementation under Plan 08 begins with SP00 and does not add future simulator
functionality. Plan 07 retains that broader unscheduled functional inventory and its
implementation receipts. The current boundary is in [STATUS.md](../../STATUS.md).

## Norms

- **Naming.** `docs/plans/NN-kebab-case.md`, numbered in the order they were
  started. Numbers are never reused.
- **Front matter.** `title`, `status` (`draft` | `in-progress` | `done` |
  `abandoned`), `date`, `adrs` (the decision records the plan implements), and
  `phase` (the delivery phase from blueprint §25).
- **Living until done.** A plan is edited while it is being executed. That is the
  difference from an ADR, which is immutable once accepted.
- **Outcome.** When the work lands, append `## Outcome (recorded after
  implementation)` with three sub-headings and fill all three:
  - *What was built* — what actually exists now, with the charter §D evidence
    label for each claim.
  - *A mistake made and corrected* — at least one. A plan whose outcome records
    no mistake was either not executed or not read honestly.
  - *Deviations from the plan, deliberate* — what was done differently and why.
    A deviation that changed a decision needs an ADR, not a paragraph here.
- **Authority.** *When the code and a plan disagree, the code is what runs.* A
  plan is never cited as the reason something behaves the way it does; the
  blueprint and the ADRs are.
- **Plans live here**, in the repository, not in an agent's scratch directory.
  `just plan` starts one.
