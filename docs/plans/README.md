# Current work and plans

Plans own the execution status of work that is actually active; packets may own their own
progress. The current architecture and its supported scope live in the
[architecture sections](../authoritative_design/README.md), not in plans.

[Plan 25](25-design-remediation.md) coordinates the proposed remediation of the two
codebase domain-alignment reviews. Its 25a–25k plans describe the target contracts,
implementation packets, dependency order and final qualification. The coordinator owns
finding dispositions; the linked plans own packet progress.
[25a is complete](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation),
with focused verification recorded in its Outcome.
[25b](25b-scientific-knowledge-and-applicability.md#execution-checkpoint) is authorized and in
progress; its checkpoint owns the current implementation and handoff. Other lettered plans remain proposed.
The series reserves full integration and qualification for 25k after all functional
work, with focused checks and immediate deletion of replaced mechanisms during the pivot.

[Plan 23](23-thermodynamic-domain-and-campaign.md) (the thermodynamic domain model and
integrated kernel campaign) is **done** (2026-09-30). Its
[Outcome](23-thermodynamic-domain-and-campaign.md#outcome-recorded-after-implementation)
records the completed functional scope, local production execution results and the scope
of completed measurements.
Plan 24 (the thermodynamic knowledge base) and its companion 24a moved with the
`thermo-knowledge/` tree to their own repository,
[paul-heyse/thermo-knowledge](https://github.com/paul-heyse/thermo-knowledge), on 2026-10-01.
[Plan 21](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/plans/21-modeling-kernel.md) (the modeling kernel) is **done**: K0–K8 were implemented;
K9 transferred to Plan 23; ADR-0097–ADR-0101 retain the proposed decision-PR route. Its
companions `21-modeling-kernel-architecture.md` and `21-knowledge-placement.md` remain
target background until Plan 23 supersedes them.
[Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md) (solver capabilities, discrete decisions,
the PostgreSQL operational store and typed data contracts) is **done** (2026-09-29): its
[Outcome](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#outcome-recorded-after-implementation)
records what was built and its Q1 qualification, now the current basis in
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2). Its
plan, packets and target review are retired to Git history; follow-ups it deferred are
register rows R-39–R-42 and R-44–R-48. Its companion `22-solver-capabilities-architecture.md` stays
because ADR-0102–ADR-0121 cite its scenarios S10–S25; it authorizes nothing.
[Plan 20](20-idaes-capability-target.md) and its companions remain capability background;
Plan 21 supersedes their target decomposition.
[Plan 19](19-current-documentation-consolidation.md),
the documentation consolidation, is done. Plans 01–18 are
complete or superseded as workstreams; their enduring meaning has moved to the architecture sections and retained
decisions, and their records are retired to Git history (ADR-0096). A retired plan,
packet or review is not a backlog and creates no obligation. The most recent completed
qualification basis is summarized in
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2).

## Workflow entrypoints

Use the [local process skills](../../.codex/skills/README.md) to prepare or conduct reviews,
create implementation plans, and plan or execute authorized work. This index routes to the
owning plan or active packet checkpoint for current state, decisions and next steps; it does
not duplicate packet status. Handoff updates that owner when the actual tree changes.
The role and workflow rollout is owned by [§24.4](../authoritative_design/sections/design-change-workflow.md)
and ADR-0139 ([governance review: Accept](../design_review/reviews/design_review_agent-coordination_2026-09-30.md), static policy scope); it changes no production plan's lifecycle or qualification status.

## Norms

- **Naming.** `docs/plans/NN-kebab-case.md`, numbered in the order they were started.
  Numbers are never reused: `just plan` allocates the highest retained number plus one,
  so the highest-numbered plan stays until a newer plan exists.
  A coordinated series retains one numeric owner (for example `25-design-remediation.md`)
  and may use alphabetic companion plans (`25a-...md`, `25b-...md`). The numeric owner
  reserves the number; `just plan` does not allocate the alphabetic companions.
- **Front matter.** `title`, `status` (`draft` | `in-progress` | `done` | `abandoned`),
  `date` and `adrs` (the decision records the plan implements). Optional
  `review_sources` and `scenario_sources` link existing definitions.
- **Living until done.** A plan is edited while it is being executed. That is the
  difference from an ADR, which is immutable once accepted.
- **Execution rhythm.** Packets are accepted by targeted tests and by deleting what they
  replace, as soon as the replacement is proven and its callers have moved. Formatting,
  lint and integrated tests run once, after all functional scope is implemented. Checkpoints record state,
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
