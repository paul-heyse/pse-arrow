# Current work and plans

Plans own execution status and adopted finding dispositions; packets may own their own progress.
Follow the linked owner rather than treating an old `in-progress` field as a resume instruction.

| Work | Status owner and context |
|---|---|
| Build performance | [Plan 15](15-rust-build-performance.md) |
| Data-model architecture and remediation | [Plan 16](16-data-model-architecture.md) and its packet links |
| Architecture-first review process | [Plan 17](17-architecture-first-design-review.md); pilots remain maintainer follow-up |
| Documentation and search | [Plan 18](18-architectural-documentation.md) |

[Plan 14](14-library-owned-process-simulator.md) supplies the implemented baseline; its
[foundation contract](14-math-foundation-contract.md) and [M22 packet](14-m22-execution.md)
retain their scoped design and historical local qualification. Earlier incomplete outcomes
are historical, not automatically inherited obligations.

The rendered book discovers all plans and packets. On GitHub, browse this directory for
history. `docs/site.toml` selects current-work search groups without duplicating progress.
Historical reviews and accepted ADR evidence retain the observations made at their own dates.
Adopted finding dispositions belong to their owning plan; link packet evidence and scenario
definitions instead of maintaining a second backlog or mandatory proof matrix.

## Norms

- **Naming.** `docs/plans/NN-kebab-case.md`, numbered in the order they were
  started. Numbers are never reused.
- **Front matter.** `title`, `status` (`draft` | `in-progress` | `done` |
  `abandoned`), `date`, `adrs` (the decision records the plan implements), and
  `phase` (the delivery phase from blueprint §25).
- **Living until done.** A plan is edited while it is being executed. That is the
  difference from an ADR, which is immutable once accepted.
- **Execution rhythm.** Packets are accepted by targeted unit tests and by deleting
  what they replace, as soon as the replacement is proven and its callers have moved.
  Integration, component, solver, Python and performance journeys, together with
  applicable format/lint/governance/codegen/doc/ADR checks, run once in a
  final qualification stage at the end of the plan. Product obligations remain with their
  product plan; documentation tooling does not require a native or source-proof campaign.
  Checkpoints record state, decisions and next steps, not per-command receipts. See
  AGENTS.md *Execution rhythm* and `.claude/rules/decisions.md`.
- **Outcome.** When the work lands, append `## Outcome (recorded after
  implementation)` with three sub-headings and fill all three:
  - *What was built* — what actually exists now, with the core principles §D evidence
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
