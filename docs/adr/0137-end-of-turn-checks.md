---
id: ADR-0137
title: Run formatting, generators and every non-functional check in end-of-turn hooks with a fixer agent
status: accepted
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: "not-required: agent tooling adopted from project-template v0.2.0, where it is render-tested; this repository adds only its configuration and existing-check updates"
evidence: Tested
supersedes: []
superseded-by: null
revisit: A check's runtime delays the next turn noticeably, a fixer edit changes behaviour or touches a generated path, or Claude Code or Codex changes Stop, UserPromptSubmit or PreToolUse hook semantics.
verification: scripts/tests/test_after_turn.py (13 unittest cases, run by just setup-test); the check_hook_wiring tests in scripts/tests/test_setup.py; just lint-agents; a pipe test of scripts/after_turn.py stop and prompt with the fixer off; live guard tests of a Claude child and a Codex child refusing just --version and running their scoped check.
standard: null
scenarios: []
---

# ADR-0137: Run formatting, generators and every non-functional check in end-of-turn hooks with a fixer agent

## Context

AGENTS.md *Execution rhythm* deferred formatting, linting and integrated testing to the end of a
plan's functional scope, and agents then ran and troubleshot those checks themselves. A
SessionStart hook ran `doctor` into the agent's context at the start of every session. The
maintainer decided on 2026-09-30 to move everything that is not functional testing, and every
action an agent would otherwise start at the beginning of a turn, to the moment the main agent
stops. The next turn follows unchanged. library-context (its ADR-0104), corpus-intelligence
(ADR-0060) and project-template v0.2.0 run the same pipeline, and the template holds the canonical
script that this repository carries verbatim.

## Scope

Binds agent tooling: `scripts/after_turn.py`, `.config/after-turn.toml`, the hook wiring in
`.claude/settings.json` and `.codex/hooks.json`, and the `just hygiene` recipe. Integrated,
component, solver, Python and performance journeys remain a plan's final qualification, as
AGENTS.md describes. `just codegen` stays with agents, because they need regenerated code to
compile and test mid-work.

## Drivers

- Formatting and fixes in the middle of work move code the agent is reasoning about.
- Troubleshooting mechanical findings costs the main model's time.
- A cheaper model can repair them once the turn ends.
- The operator must see what cannot be repaired without the model seeing it.

## Options

1. **Keep the end-of-plan rhythm.** Agents run and fix the static checks at plan close, on the
   main model.
2. **Wake the main agent on a failure** (a blocking Stop decision or an asynchronous rewake). The
   main model fixes the findings, inside the turn it declared finished.
3. **End-of-turn hooks with a cheaper fixer (chosen).** The hooks run the generators and checks. A
   fixer repairs what it can and is allowed only to re-run its own checks. The operator sees the
   rest.

## Outcome

**At stop, synchronously:** `skills-sync`, `agent-config-sync`, `adr-index` and `fmt`. `fmt` now
applies ruff's auto-fixes; `lint-py` stays check-only for `quality`.

**In the background:**
- `doctor` and `db-status` run first, as operator-only readiness steps.
- Then every `just hygiene` dependency runs. These are leaf recipes, not the `validation --group`
  wrappers:
  - static Python and repository lint;
  - the ADR and register lint;
  - both clippy runs and `docs-rust`;
  - the boundary, family and solver-pin checks;
  - the codegen checks;
  - the conformance-fixture check.
- Operator-only: the codegen, family, solver-pin and conformance-fixture checks, and the sync
  steps.
- The fixer handles everything else: `claude -p` on Sonnet 5.5 at the session's effort (high when
  the hook can't see it), or `codex exec` on `gpt-6.1-sol` at medium. It re-runs its checks
  only through `uv run --no-project --python 3.14 python scripts/after_turn.py check <id>`,
  which a PreToolUse guard enforces.
- `library-catalog` runs last and is fire-and-forget.

**At the next prompt:** the UserPromptSubmit hook waits for the job. Leftovers are shown to the
operator only.

**Left manual:**
- the checks that need Docker, the solver image or the network: `python-contracts-check`,
  `codegen-python-check`, `codegen-schemas-check`, `codegen-bindgen-check` and `lint-zizmor`;
- the heavy set: native clippy variants, feature powerset, udeps and audits.

### Consequences

- The SessionStart hook and `agent-hooks.py session` are removed. `check_agent_config.py` now
  requires PreToolUse, Stop and UserPromptSubmit, and accepts `scripts/after_turn.py` as a shared
  hook script.
- The next prompt can wait after Rust edits, while clippy, `docs-rust` and the codegen checks
  rebuild.
- `scripts/after_turn.py` and its test use 3.14 syntax. They are excluded from ruff and pyrefly,
  like the library-catalog scripts, so their bodies stay identical to the template's.

### Compensating controls

- The fixer's shell is confined by the guard, and its edits by `agent-hooks.py guard`, which
  still blocks generated paths and decided documents.
- `.config/after-turn.toml` lists `protected` and operator-only checks.
- `AFTER_TURN_FIXER=off` disables the fixer.

### Confirmation

This is executed behaviour, not reasoning alone: the tests and live runs named in `verification:`.

## More information

- Canonical script: project-template `template/scripts/after_turn.py`.
- Configuration: `.config/after-turn.toml`.
- Agent instructions: AGENTS.md *Execution rhythm* and *Agent runtimes*.

## Status history

- 2026-09-30 — proposed and accepted by the maintainer as agent tooling policy (the operator's
  2026-09-30 plan).
