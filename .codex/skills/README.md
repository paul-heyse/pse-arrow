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
| [leverage-codebase-audit](leverage-codebase-audit/SKILL.md) | Identify concrete codebase opportunities for library leverage and refresh their living account |
| [leverage-library-assessment](leverage-library-assessment/SKILL.md) | Research focused capability applicability and accumulate reusable library opportunity types |
| [leverage-opportunity-review](leverage-opportunity-review/SKILL.md) | Compare complete options and develop bounded target recommendations |
| [design-principle-audit](design-principle-audit/SKILL.md) | Refresh the broad persisted diagnostic account before a diagnostic review of production or broad system scope |
| [design-principle-audit-tests](design-principle-audit-tests/SKILL.md) | Audit test code and evidence in its separate living account; prerequisite for a test-focused diagnostic review |
| [design-review-audit](design-review-audit/SKILL.md) | Reassess recommendations and omissions; publish a companion and integrated review |
| [plan-completion-audit](plan-completion-audit/SKILL.md) | Assess claimed completion and correct the same owning plan |
| [plan-improvement-review](plan-improvement-review/SKILL.md) | Review alignment, general enhancements and execution improvements across the remaining plan change footprint; publish an assessment for separately instructed plan revision |
| [adr](adr/SKILL.md) | Record or supersede a decision and amend its architectural owner |
| [plan-design-review](plan-design-review/SKILL.md) | Develop an approach to a requested review |
| [design-review](design-review/SKILL.md) | Conduct a scoped review against the declared layered standard |
| [design-review-process-simulator](design-review-process-simulator/SKILL.md) | Add the scientific/process-simulator profile when applicable |
| [functional-extension-review](functional-extension-review/SKILL.md) | Explore valuable product, developer, operator or research capabilities and compare their realization within the operator's scope |
| [plan-functional-extension-review](plan-functional-extension-review/SKILL.md) | Prepare a capability inquiry without requiring a diagnostic audit or predetermined feature list |
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
bounded capabilities and coordination principles, including readiness for the requested result and
delegated decision scope; AGENTS.md owns local operating rules. The coordinator remains accountable
for design coherence, integration and acceptance. The layered
standard remains core 3.4 and process-simulator 1.5, as selected by
`docs/design_review/design_principles/standard.toml`; model adequacy/authority (AP-04/G9),
qualitative execution-fit assessment (AP-07/G9) and independent scientific judgments are preserved.
Explain material operational tradeoffs in plain language under the core's assessment scope;
numerical estimates, runtime cost accounting and cost proof artifacts are not required by it.
Quantitative performance/capacity claims require measurements; semantic correctness obligations
remain. The binding owns cadence. Routine implementation adds no standing domain-model
review or exhaustive flow tracing.

Functional enhancement has a separate entry point: `plan-functional-extension-review` /
`functional-extension-review`. The operator selects the exploration scope; useful capabilities
need no diagnostic finding or existing caller. Design principles shape their realization.
[Opportunity reviews](../../docs/functional_reviews/README.md) can feed the same planning and
execution workflow. Recommendation auditing preserves their value lens and publishes a
[functional companion](../../docs/functional_reviews/audits/README.md) plus the integrated review,
without importing the diagnostic audit prerequisite.

Use [plan-improvement-review](plan-improvement-review/SKILL.md) at requested or explicitly
planned improvement boundaries. Review design-principle alignment, general enhancements and
execution improvements across the remaining change footprint, including substantially modified
supporting components. Value can extend beyond the functional target; coherent integration and
total burden determine fit. Publish a standalone
[assessment](../../docs/plan_improvement_reviews/README.md); plan revision is separately instructed
through existing planning and decision routes. No broad audit prerequisite, separate update skill
or automatic adoption follows; formal review cadence remains unchanged.

## Library orientation and active exception

The [library-leverage framework](../../docs/library_leverage/README.md) connects the three
local leverage skills and their distinct evergreen collections. Start from concrete codebase
opportunities, investigate focused library capabilities and reusable types, then compare complete
options. Base patterns and relevant patterns collected in assessment outputs have equal use;
all stages can discover or refine additional framings. Retain useful types and output patterns
after their matches retire; manual promotion into base guidance is separate maintenance.
Ordinary library use initiates none of these assessments. A comparative recommendation can feed
planning while the binding continues to own any required formal review and decision route.

Use [library overviews](../../docs/libraries/README.md) for optional first-pass awareness,
then Context7 and primary documentation, upstream source/tests or installed sources for
consequential details.
The [native-solver-libraries](native-solver-libraries/SKILL.md) skill remains active and selected
in .config/library-skills.toml; existing setup/readiness/worktree synchronization keeps that one
local discovery link. Central sources are unchanged.

## Inactive central library-skill references

The other former library skills below remain central historical references. They are no longer
selected, synchronized, discovered or required here. Existing plans may still name them; do not
rewrite those plans merely for this transition.

| Former skill | Reference identity |
|---|---|
| rust-code-model | Central library-skill source, inactive here |
| symbolica-faer-oximo | Central library-skill source, inactive here |
| pyomo-and-solvers | Central library-skill source, inactive here |
| rust-reasoning | Central library-skill source, inactive here |
| neo4j-surrealdb | Central library-skill source, inactive here |
| datafusion | Central library-skill source, inactive here |
| ast-grep-ripgrep | Central library-skill source, inactive here |
| datafusion-tracing | Central library-skill source, inactive here |
| salsa | Central library-skill source, inactive here |
| rust-graphs | Central library-skill source, inactive here |

The [living diagnostic audit](../../docs/design_review/audit/README.md) precedes broad diagnostic solution review;
test-only engagements use the [separate test account](../../docs/design_review/test-audit/README.md).
The [test-assessment companion](../../docs/design_review/design_principles/core/test-assessment.md)
interprets the same principles for test code, reuse and independent evidence. Installing these
routes performs no test audit or product qualification.
The operator chooses holistic, library-targeted, principle-focused or combined focus. Requested
design-review recommendation audits write [net changes](../../docs/design_review/reviews/audits/README.md) and
revise the principal review in place; completion audits correct the same owning plan. These are
deliberate review boundaries, not automatic recurring gates. No actual audit is performed by
installing these skills; no root STATUS or separate handoff ledger is introduced.

Formatting remains the root's just turn-end and native-skill synchronization just ready.
These workflows add no hooks; just hygiene remains at scope end.
