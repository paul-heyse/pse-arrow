# Local skills

Process skills are maintained in `.codex/skills/`. The `.claude/skills` and `.agents/skills`
aliases expose the same sources to both runtimes. `just agent-config-sync` only materializes
aliases on Windows when directory symlinks are unavailable and preserves live library links.
It never generates or overwrites native agent adapters.

| Process skill | Use |
|---|---|
| [adr](adr/SKILL.md) | Record or supersede a decision and amend its architectural owner |
| [plan-design-review](plan-design-review/SKILL.md) | Develop an approach to a requested review |
| [design-review](design-review/SKILL.md) | Conduct a scoped review against the declared layered standard |
| [design-review-process-simulator](design-review-process-simulator/SKILL.md) | Add the scientific/process-simulator profile when applicable |
| [plan-creation](plan-creation/SKILL.md) | Develop the approach to authoring an implementation plan |
| [create-plan](create-plan/SKILL.md) | Write the design and execution plan, including focused assessment of affected foundations |
| [plan-execution](plan-execution/SKILL.md) | Develop an approach to executing an approved plan |
| [execute-plan](execute-plan/SKILL.md) | Execute authorized scope through integration, functional checks and checkpoint handoff |

Invoke `/plan $plan-design-review <goal>`, `/plan $plan-creation <reference and goal>` or
`/plan $plan-execution <plan and scope>` when the runtime supports Plan mode. Leave Plan mode
for the corresponding action skill; invoke an action directly when its authorized approach is
already clear. Skills work in ordinary conversation and do not themselves switch modes.
The [suggested plan structure](create-plan/references/plan-structure.md) is adaptable to local
plan naming, front matter and packet conventions in [the current-work index](../../docs/plans/README.md).
That index routes to the active plan or packet checkpoint for baseline and handoff; no separate
root STATUS file or handoff skill is required.

All workflow pairs use the [shared agent roles](../../.agents/roles/README.md). Roles supply
bounded capabilities; the coordinator retains design, integration and acceptance. The layered
standard remains core 3.3 and process-simulator 1.3, as selected by
`docs/design_review/design_principles/standard.toml`; AP-04/G9 and independent scientific judgments
are preserved. The binding owns cadence. Routine implementation adds no standing domain-model
review or exhaustive flow tracing.

Library capability skills remain shared live sources selected by `.config/library-skills.toml`,
exposed through gitignored links. Follow AGENTS.md's capability and Context7 routes. Change shared
library skills at their shared source, preserving repository-independent guidance; local process
skills stay tracked here. Formatting is the root's `just turn-end` and skill synchronization
`just ready`; these workflows add no hooks, and run `just hygiene` only at scope end.
