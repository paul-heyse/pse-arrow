# Library leverage

**Implemented guidance.** A codebase-first workflow with three distinct
repository skills and evergreen assessment collections. Benefits to discovery and design quality
remain Proposed; installation does not establish an assessed codebase or qualified library fit.

Start with the [framework](library-leverage-framework.md) for responsibilities, handoffs, collection
paths and retention. Read the [base opportunity patterns](patterns.md) with additional patterns
from `assessment-patterns.md` when that output exists. All stages use relevant additions on the
same footing; promotion into base guidance is a manual operator decision.

| Skill | Question it answers | Collection it maintains |
| --- | --- | --- |
| [leverage-codebase-audit](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-codebase-audit/SKILL.md) | Which concrete responsibilities and operations deserve investigation? | [Codebase opportunities](codebase/README.md) |
| [leverage-library-assessment](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-library-assessment/SKILL.md) | Which focused library capabilities could address those opportunities, under what conditions? | [Library assessments](libraries/README.md) |
| [leverage-opportunity-review](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-opportunity-review/SKILL.md) | Which complete option best addresses each opportunity, and why? | [Comparative reviews](reviews/README.md) |

## Use the workflow

Invoke the relevant skill with the intended scope, for example:

- `$leverage-codebase-audit` — identify library-leverage opportunities in a selected operation,
  subsystem or the codebase.
- `$leverage-library-assessment` — assess selected opportunities against relevant capabilities,
  using the opportunity IDs and existing library analysis.
- `$leverage-opportunity-review` — compare complete options for those opportunities and recommend
  bounded target designs.

An end-to-end request can authorize all three stages. Coordinate their substantive handoffs and
iterate where new evidence changes a premise. A focused inquiry needs only the relevant scope;
ordinary implementation and API questions do not initiate this workflow.

## Current accounts

The [codebase opportunity account](codebase/README.md) was rebuilt by a whole-codebase audit on
2026-10-30 at source `c8244cff5`. It holds 142 current opportunities: 31 refreshed from the prior account,
111 new (`CBO-054`–`CBO-164`), with 21 retired on evidence and one anchored pointer kept for each. It added
[additional patterns](assessment-patterns.md) `LOP-X015`–`LOP-X022` (revision 5). Mechanisms are
Interface-checked and value claims Proposed; nothing was run for it.

The [library applicability assessment](libraries/README.md) and [comparative review](reviews/README.md)
predate that baseline. They were built on the first audit at `dbc4c9436`, reassessed against source
`4f52fd239`, and were not refreshed. New opportunities have no assessment yet, and content on retired IDs
is historical. The codebase entry page lists the questions for the next refresh. Record inspected baselines
and limits rather than treating empty collections as assessed, and do not invent patterns to populate a
scaffold.

The shared [vocabulary](references/shared-language.md) and
[collection-writing guidance](references/collection-writing.md) support all three skills. Keep
reusable library types and useful additional patterns when their originating matches retire.
The library collection is distinct from the broad [capability overviews](../libraries/README.md):
its research starts with codebase opportunities and accumulates focused capability knowledge.

The tracked skill source is `.codex/skills`. Existing `.agents/skills` and `.claude/skills`
aliases expose the same sources to Codex and Claude Code. These three local process skills are
tracked through the existing allowlist; `.config/library-skills.toml` continues to select only
the shared native-solver-libraries capability skill. Existing formal review, planning and
implementation routes retain their scope and authority under the
[framework binding](library-leverage-framework.md#repository-binding-and-downstream-use).
