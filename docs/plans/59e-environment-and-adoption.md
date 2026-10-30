---
title: "Plan59e: Environment and adoption"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, documentation]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md]
---

# Environment and adoption

Part of [Plan59](59-agent-cli.md). This plan covers the environment around the CLI:
- worktrees that are ready on creation;
- Claude's orientation surfaces;
- guidance that makes `pse` the obvious first choice;
- the plan index;
- the edit guidance.

It also carries the continuing communication work that ships with every verb.

Sources: [O22](../functional_reviews/rust-agent-cli-2026-10-29.md#O22),
[O23](../functional_reviews/rust-agent-cli-2026-10-29.md#O23),
[O25](../functional_reviews/rust-agent-cli-2026-10-29.md#O25) (Claude),
[O10](../functional_reviews/rust-agent-cli-2026-10-29.md#O10) items 2–3,
[O11](../functional_reviews/rust-agent-cli-2026-10-29.md#O11) start routes, and
[O12](../functional_reviews/rust-agent-cli-2026-10-29.md#O12).

Operator directions:
- No direnv changes; the operator's separate environment work owns them.
- No Codex configuration changes; Codex uses the CLI through its shell and the shared AGENTS.md.
- Adoption comes from ease and communication, never hooks.

## Baseline

**Worktrees.** `just worktree` passes its arguments to `git worktree add`. The environment guide
(§Worktrees) leaves the rest manual:
- supply `.envrc.local`;
- prepare the host tools;
- the new checkout starts with a cold `target/` and an unprepared `target/host-tools`.

The sccache cache and native provider caches are already shared across checkouts. Each checkout
has its own `target/` (ADR-0122).

**Claude surfaces.**
- `.claude/settings.json` has the edit-guard `PreToolUse` hook. The operator's concurrent changes
  moved session environment activation to a global adapter.
- There is no status line and no orientation context at session start.

**Guidance.**
- AGENTS.md "Start here" lists `just --list`, `scripts/pse-env --explain` and `just bootstrap`.
- The environment guide, validation guide and documentation guide route through recipes and raw
  commands.
- Role contracts (`.agents/roles/*`) and process skills (`.codex/skills/*`, exposed to Claude)
  describe workflows without a CLI.
- `docs/plans/README.md` is a 32 KB narrative index that agents re-read for orientation, 70 times
  in the review's window.

**Edits.** Half of measured edits were Python `str.replace` heredocs, 200 of 512 without a match
check. `ast-grep` is installed and configured (`sgconfig.yml`, used by `lint-ast`).

## Target

- **Worktrees in one command.** `pse worktree new <name> [--base REF] [--branch B] [--warm]`:
  1. runs `git worktree add`;
  2. copies `.envrc.local` with its current behaviour and mode;
  3. prepares host tools with `scripts/bootstrap-host --from <source>`, which copies a
     fingerprint-matching generation ([59a A05](59a-cli-foundation.md#a05));
  4. relies on the already-shared native provider caches;
  5. registers the checkout in the store ([59a A04](59a-cli-foundation.md#a04));
  6. with `--warm`, starts a detached `pse check --workspace` ([59b B03](59b-runs-and-continuity.md#b03))
     so the first real build hits the shared sccache.

  It does not touch direnv.
- **Listing and removal.**
  - `pse worktree list` shows each worktree's branch, ahead/behind, dirty state, live runs and
    last activity.
  - `pse worktree rm` removes a worktree. It asks for `--force` only when uncommitted changes or
    unmerged commits would be lost, and keeps the worktree's run history in the store.
- **Claude orientation.**
  - The session-start context hook from [59b B05](59b-runs-and-continuity.md#b05) returns
    `pse status --brief` (HEAD, dirty count, active plan, live runs, stale host) and one line
    naming the main verbs. After compaction it returns the handoff instead.
  - A `statusLine` running `pse status --line` shows live runs and the last failure. It is
    read-only and reads only the run directories and a cached staleness result. It never hashes
    sources or runs `git status` on a render, and it is harmless when tools are unprepared.
- **Communication** puts `pse` where agents already look, as each verb ships:
  - AGENTS.md "Start here" becomes a short task-to-verb table ("run a test → `pse test`", "wait
    on a run → `pse wait`", "exact dependency API → `pse api`", …), pointing to
    `pse commands --json` and the generated reference.
  - The environment, validation and documentation guides route their workflows through `pse`.
  - Role contracts and process skills name the verb for each step: `pse brief` when delegating,
    `pse handoff` before long work, `pse new`/`ref`/`docs check` in review and plan skills, and
    `pse commit` when committing.
  - Recipes for covered work delegate to `pse` or are retired with their replacement (59c and
    59d own the retirements).
  - Envelope `next` hints and diagnostic remedies already use `pse` forms ([59a A03](59a-cli-foundation.md#a03)).
- **Plan index.** `docs/plans/README.md` becomes a short current-work pointer: active plans, one
  line each, with status owned by their tables. The narrative history moves to the plans' own
  Outcomes or retires to Git history (ADR-0096).
- **Edit guidance.** One AGENTS.md line and a short `.claude/rules/tooling.md` example:
  - `apply_patch`/`Edit` by default;
  - `ast-grep run --pattern … --rewrite … --update-all` for repeated structural Rust changes;
  - assert the expected match count in any unavoidable script edit.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [E01](#e01) | planned | 59e | `pse worktree new\|list\|rm`; `just worktree` retired; environment guide §Worktrees updated (direnv wording unchanged) |
| [E02](#e02) | planned | 59e | Claude session-start status brief content and `statusLine` |
| [E03](#e03) | planned | 59e (continuing) | AGENTS.md task-to-verb entries pointing to `pse commands`, guide routes, roles/skills mentions and retired-recipe routes, updated as each verb ships |
| [E04](#e04) | planned | 59e | Slim `docs/plans/README.md` current-work pointer |
| [E05](#e05) | complete | 59e | Edit guidance in AGENTS.md and `.claude/rules/tooling.md` |

<a id="e01"></a>
### E01 — Worktrees ready on creation

**Consumes:**
- [59a A04](59a-cli-foundation.md#a04) (store, checkout ID) and [A05](59a-cli-foundation.md#a05)
  (`bootstrap-host --from`, fingerprints).
- [59b B03](59b-runs-and-continuity.md#b03) for `--warm`; until it lands, `--warm` is absent.

**Delivers:** `pse worktree new|list|rm` as described.

**Deletes:**
- The `just worktree` recipe.
- The manual-preparation steps in the environment guide §Worktrees. Its direnv wording stays
  unchanged under the hold.

**Acceptance:**
- `pse worktree new scratch-x` produces a checkout where `scripts/pse status` works immediately,
  with host tools copied rather than rebuilt and `.envrc.local` present with its original mode.
- `pse worktree rm scratch-x` on a clean, merged worktree removes it without prompting, and
  refuses without `--force` when an uncommitted change exists.

<a id="e02"></a>
### E02 — Claude session-start brief and status line

**Consumes:** [59a A06](59a-cli-foundation.md#a06) (`status`) and [59b B05](59b-runs-and-continuity.md#b05)
(context hook).

**Delivers:**
- The `--brief` content: one compact block plus the main-verbs line.
- `pse status --line`.
- The `statusLine` entry in `.claude/settings.json`, coordinated with the operator's concurrent
  edits to that file.

**Acceptance:**
- A new Claude session's first context includes the brief.
- The status line renders from cached state well under its timeout, and prints a neutral line
  when tools are unprepared.

<a id="e03"></a>
### E03 — Communication and routing (continuing)

**Consumes:** each verb as it ships.

**Delivers, per verb, in the same change as the verb:**
- its AGENTS.md task-table row;
- the guide route that replaces the old raw or recipe form;
- the role or skill mention where that workflow is described;
- removal of documentation for retired recipes.

Affected owners:
- AGENTS.md, including "Start here" and the Execution rhythm's named recipes (`check-package`,
  `unit-package`);
- `docs/dev/{agent-environment,validation-assessment,documentation}.md`;
- `docs/authoritative_design/sections/operations-and-validation.md`, the §24.1 Commands
  paragraph, edited after [59d D06](59d-knowledge-and-workflow-verbs.md#d06) makes design
  documents informing;
- `.agents/roles/*.md`;
- `.codex/skills/{execute-plan,plan-creation,create-plan,functional-extension-review,design-review}/SKILL.md`
  and other skills that describe affected workflows;
- active plans' verification sections, including this series' own, where they name a retired
  recipe as the route to run. Historical plan records are left as they were.

These are shared guidance, not Codex configuration.

**Acceptance:**
- The AGENTS.md start route gives a short set of task-to-verb entries and points to
  `pse commands` for the rest. It is not a second, hand-written catalog.
- No current guide, skill or active plan route names a retired recipe.
- `just lint-agents` passes.

<a id="e04"></a>
### E04 — Slim plan index

**Consumes:** nothing; independent.

**Delivers:** `docs/plans/README.md` reduced to active plans with one line each and links to their
status tables. Narrative status that duplicates plan Outcomes is removed, and lifecycle bindings
stay valid.

**Acceptance:** the index fits on one screen, and `xtask lifecycle validate` passes.

<a id="e05"></a>
### E05 — Edit guidance

**Consumes:** nothing; independent.

**Delivers:** the AGENTS.md line and the `.claude/rules/tooling.md` example described above.

**Acceptance:** the guidance is present and `just lint-agents` passes.

## Verification and handoff

- **E01:** targeted `pse-host` tests and one real worktree creation and removal.
- **E02:** one real Claude session start.
- **E03–E05:** documentation and agent-configuration changes, checked with `just lint-agents` and
  the documentation link check.
- **Scope end:** the series' scope-end `just docs`/`just governance` runs cover them.
