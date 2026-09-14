# Plans

A plan records **how work is sequenced and verified**. A decision record
([`../adr/`](../adr/)) records **what was decided and why**. The two
have different lifecycles, which is why they live in different directories.

| Plan | Covers | ADRs | Status |
|---|---|---|---|
| [01 — Repository configuration](01-repository-configuration.md) | Repository identity, layout and migration, the Rust workspace and supply chain, the solver container, Python packaging and tests, the CI job graph, decision records, GitHub setup, the agent environment, the docs site | ADR-0001 – ADR-0038 | in-progress |
| [02 — Blueprint revision 5 contracts](02-blueprint-revision-5-contracts.md) | Design amendment for R4-01–R4-13 and L1–L9; separate Proposed implementation handoff | ADR-0039 – ADR-0048 | done (documentation; runtime handoff Proposed) |
| [03 — Wave 1 foundations](03-wave-1-foundations.md) | Phase-0 foundations and load-bearing interfaces as parallel sub-agent packets | ADR-0039 – ADR-0051 | in-progress |

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
