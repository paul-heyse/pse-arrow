# Current work and plans

Plans own the execution status of work that is actually active; packets may own their own
progress. The current architecture and its supported scope live in the
[architecture sections](../authoritative_design/README.md), not in plans.

No plan is active. [Plan 19](19-current-documentation-consolidation.md), the documentation
consolidation, is done and stays only as the highest-numbered plan. Plans 01–18 are
complete or superseded as workstreams; their enduring meaning has moved to the architecture sections and retained
decisions, and their records are retired to Git history (ADR-0096). A retired plan,
packet or review is not a backlog and creates no obligation. The most recent completed
qualification basis is summarized in
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2).

## Norms

- **Naming.** `docs/plans/NN-kebab-case.md`, numbered in the order they were started.
  Numbers are never reused: `just plan` allocates the highest retained number plus one,
  so the highest-numbered plan stays until a newer plan exists.
- **Front matter.** `title`, `status` (`draft` | `in-progress` | `done` | `abandoned`),
  `date` and `adrs` (the decision records the plan implements). Optional
  `review_sources` and `scenario_sources` link existing definitions.
- **Living until done.** A plan is edited while it is being executed. That is the
  difference from an ADR, which is immutable once accepted.
- **Execution rhythm.** Packets are accepted by targeted tests and by deleting what they
  replace, as soon as the replacement is proven and its callers have moved. Comprehensive
  qualification is requested separately by the maintainer. Checkpoints record state,
  decisions and next steps, not per-command receipts. See AGENTS.md *Execution rhythm*
  and `.claude/rules/decisions.md`.
- **Outcome.** When the work lands, append `## Outcome (recorded after implementation)`
  with three sub-headings and fill all three:
  - *What was built* — what actually exists now, with the core principles §D evidence
    label for each claim.
  - *A mistake made and corrected* — at least one actual correction.
  - *Deviations from the plan, deliberate* — what was done differently and why.
    A deviation that changed a decision needs an ADR, not a paragraph here.
- **Closure.** Move enduring meaning to its contract or rationale owner, remove the plan
  from current work, and retire the completed plan and its resolved reviews once no
  reader, tool or test depends on them. Git history keeps them; no archive tree is kept.
- **Authority.** *When the code and a plan disagree, the code is what runs.* A plan is
  never cited as the reason something behaves the way it does; the architecture
  sections and the ADRs are.
- **Plans live here**, in the repository, not in an agent's scratch directory.
  `just plan` starts one.
