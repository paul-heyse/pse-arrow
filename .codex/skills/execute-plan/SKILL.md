---
name: execute-plan
description: Execute authorized scope in an existing implementation plan through integration, functional verification and handoff, adapting to current evidence.
---

# Execute a plan

Carry the authorized plan scope through implementation and acceptance. The coordinator uses the current plan,
current-work index and active packet checkpoint relevant to the task, repository instructions and
actual tree to establish what remains; delegated workers use their brief and relevant owners; prior receipts
retain their original date and scope. Inspect dirty-tree ownership before editing and preserve
concurrent work. Resolve material ambiguity without making routine implementation choices into
new approval steps. A proposed decision or unimplemented target is not an established baseline.

The root agent coordinates the work and retains design decisions, integration and acceptance.
Delegate when independent work, context isolation, distinct capabilities or independent judgment
justify coordination and integration; small or tightly coupled work may stay with the root.
The [shared roles](../../../.agents/roles/README.md) offer bounded code mapping,
library research, design review, execution, implementation review and testing through
`code-mapper`, `library-research`, `design-reviewer`, `executor`,
`implementation-reviewer` and `test-agent`. Use the roles that help; neither every role nor a
fixed sequence is required.

Give delegated work the outcome, settled decisions, relevant contracts, exact baseline including
relevant dirty changes, permitted effects, edit ownership and sibling assignments, dependencies
and expected evidence. Workers load task-relevant authorities and may follow discovered dependencies. Assign disjoint edit ownership before
parallel implementation; use separate worktrees only as repository instructions require for
concurrent production edits. Read-only evidence tasks can run alongside implementation when
their inputs are stable. Review agents assess a named baseline independently. Agent conclusions
inform the root's judgment; they do not transfer responsibility for accepting the result.

Implement prerequisite scope before dependent work and integrate completed slices against
their contracts and adjacent consumers. Follow relevant pinned-library skills and documentation
routes when library behavior matters. Adapt as evidence changes: narrow an investigation,
correct a failed assumption, or revise the remaining sequence in its existing owner. Preserve
source-review finding IDs and their single disposition owner rather than adding parallel status
documents or a second durable execution plan.

Use the binding's
[review cadence](../../../docs/design_review/design_principles/binding/pse-arrow.md#reviews-in-this-repository)
for change/conformance and design/target review. Routine code changes do not create a standing
architecture or domain-model review obligation. Reopen architecture when a changed boundary,
decision or concrete evidence warrants it; the focused foundation assessment in
[create-plan](../create-plan/SKILL.md) is not repeated by default. Route decision changes through
[adr](../adr/SKILL.md) and update the architectural owner. Continue independent authorized work
while resolving a prerequisite; seek user input when a material scope or authorization decision
actually requires it.

Validate new functional scope with compile checks and targeted tests or probes. Run integrated
qualification only after all functional scope of the authorized plan is implemented, following
`AGENTS.md`; bounded-package completion does not establish an enclosing stage exit. Repair
failures and rerun the affected checks, broadening verification when a material change warrants
it. Run `just hygiene` once with the integrated qualification and fix what it reports.
Formatting and generators belong to the automatic end-of-turn hook: do not run them here.

Report actual commands and outcomes (`passed`, `failed`, `blocked` with its prerequisite, or
`not_run`). Keep comprehensive evidence in the plan Outcome or qualification report; interim
checkpoints record state, decisions and next steps rather than per-command receipts. Keep
proposed, implemented, tested and measured claims
distinct, date verified claims, and retain composite or partial qualification boundaries. Close
findings only with evidence for their stated obligation. Follow repository commit policy within
the task's authorization, and update the existing plan or active packet checkpoint when the work
changes what is true; keep `docs/plans/README.md` as navigation to its owner. Clean up any
remaining worktrees that are fully merged. Report the resulting
behavior, verification and any remaining authorized work or blockers;
do not treat a partial slice as completion of the requested scope.
