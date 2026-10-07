# Local skills

Use [design principles](../../docs/design_review/design_principles/core/design-principles.md) together
with [Heuristics for Efficient Architecture](../../docs/design_review/design_principles/core/efficient-architecture-heuristics.md)
for consequential architectural and implementation choices. Consider relevant execution patterns
before committing to physical organization, interfaces, preparation, assurance and lifecycles;
address material mismatches while the design remains easy to change. Judgment remains qualitative,
without an exhaustive checklist, cost models or new proof machinery. Execution applies this to
open choices or exposed mismatches without restarting settled reviews.

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
standard remains core 3.4 and process-simulator 1.5, as selected by
`docs/design_review/design_principles/standard.toml`; model adequacy/authority (AP-04/G9),
qualitative execution-fit assessment (AP-07/G9) and independent scientific judgments are preserved.
Explain material operational tradeoffs in plain language under the core's assessment scope;
numerical estimates, runtime cost accounting and cost proof artifacts are not required by it.
Quantitative performance/capacity claims require measurements; semantic correctness obligations
remain. The binding owns cadence. Routine implementation adds no standing domain-model
review or exhaustive flow tracing.

Library capability skills remain shared live sources selected by `.config/library-skills.toml`,
exposed through gitignored links. Follow the skill and Context7 routes in `.agents/roles/worker.md`. Change shared
library skills at their shared source, preserving repository-independent guidance; local process
skills stay tracked here. Formatting is the root's `just turn-end` and skill synchronization
`just ready`; these workflows add no hooks, and run `just hygiene` only at scope end.
