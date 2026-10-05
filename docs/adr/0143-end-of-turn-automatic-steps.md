---
id: ADR-0143
title: End-of-turn hooks run only automatic steps; agents run the static checks at scope end
status: superseded
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: "not-required: agent tooling adopted from project-template v0.3.0, where it is render-tested; this repository adds only its configuration and instruction updates"
evidence: Tested
supersedes: [ADR-0137]
superseded-by: ADR-0161
revisit: A static check repeatedly fails late enough at scope end to cost rework, or Claude Code or Codex changes Stop or UserPromptSubmit hook semantics.
verification: scripts/tests/test_after_turn.py (9 unittest cases, run by just setup-test); the check_hook_wiring tests in scripts/tests/test_setup.py; just lint-agents; the project-template v0.3.0 render test, which runs Stop and checks the report and a silent prompt hook.
standard: null
scenarios: []
---

# ADR-0143: End-of-turn hooks run only automatic steps; agents run the static checks at scope end

## Context

ADR-0137 moved formatting, generators and every `just hygiene` check into end-of-turn hooks,
with a headless fixer agent for what failed, confined by a PreToolUse guard. The fixer proved
brittle: it edited code another session was still developing, needed harness-specific trust,
effort and permission plumbing, and the prompt wait for it froze sessions. The maintainer decided
on 2026-10-01 to remove the fixer and, with it, the checks whose findings need targeted fixes (type
errors, clippy, lint, codegen drift). library-context (ADR-0110), corpus-intelligence (ADR-0069),
thermo-knowledge (ADR-0008) and project-template v0.3.0, which holds the canonical script, made
the same change.

## Scope

Binds agent tooling: `scripts/after_turn.py`, `.config/after-turn.toml`, the hook wiring in
`.claude/settings.json` and `.codex/hooks.json`, and when `just hygiene` runs. `just codegen`
stays with agents mid-work, because they need regenerated code to compile and test. The
`agent-hooks.py` edit guard is unchanged.

## Drivers

- Nothing should edit code that its author did not ask for.
- The next prompt must never wait on end-of-turn work.
- Static findings need an owner who knows the change.
- Formatting and lint fixes mid-work still move code the agent is reasoning about.

## Options

1. **Keep the fixer and harden it** (scope it to the stopping session's files, make it
   cancellable). Each hardening adds machinery to a component whose failures are hard to see.
2. **Run every check in the hooks and report findings to the maintainer only.** Nothing edits
   code unasked, but findings have no owner who knows the change, and clippy and `docs-rust` still
   rebuild after every turn.
3. **Surface the findings to the main agent on its next prompt.** The author fixes them, but
   mid-work findings interrupt unfinished turns.
4. **The hooks run only automatic steps; agents run `just hygiene` once at scope end (chosen).**

## Outcome

**At stop, synchronously:** `skills-sync`, `agent-config-sync`, `adr-index` and `fmt` (which
applies ruff's safe auto-fixes; `lint-py` stays check-only).

**In the background:** `doctor` and `db-status` (readiness), then `library-catalog`, which nothing
waits on. The report goes to `.git/after-turn/report.json`.

**At the next prompt:** the UserPromptSubmit hook never waits. It shows failed steps from the last
report to the maintainer, once, never to the model.

**Not in the hooks:** every `just hygiene` dependency (static Python and repository lint, the ADR
and register lint, both clippy runs and `docs-rust`, the boundary, family and solver-pin checks,
the codegen checks and the conformance-fixture check). Once all functional scope in a plan is
implemented, the agent runs `just hygiene` beside the selected journeys, fixes what fails and
re-runs single checks with `just <id>`. The Docker-, solver-image- and network-bound checks and the
heavy set stay manual, as before.

### Consequences

- There is no fixer, no `AFTER_TURN_*` environment and no after-turn PreToolUse guard;
  `check_agent_config.py` still requires a PreToolUse event, which `agent-hooks.py guard` fills.
- Static findings can accumulate during a long plan; fixing them is ordinary scope-end work.
- Codex re-trusts hooks by content hash, so the changed `.codex/hooks.json` needs re-trusting
  once.
- `scripts/after_turn.py` and its test stay excluded from ruff and pyrefly, so their bodies stay
  identical to the template's.

### Compensating controls

- `agent-hooks.py guard` still blocks edits to generated paths and decided documents.
- `just hygiene` at scope end is the gate that the fixer's re-runs used to provide.

### Confirmation

Executed behaviour: the tests named in `verification:`.

## Pros and cons

- Pro: no unrequested edits, no prompt waits, far less hook machinery.
- Con: static findings surface later, at scope end, instead of after each turn.

## More information

- Canonical script: project-template `template/scripts/after_turn.py` (v0.3.0).
- Configuration: `.config/after-turn.toml`.
- Agent instructions: AGENTS.md *Execution rhythm* and *Agent runtimes*.

## Status history

- 2026-10-01 — proposed and accepted by the maintainer as agent tooling policy; supersedes
  ADR-0137.
- 2026-10-05 — superseded by ADR-0161.
