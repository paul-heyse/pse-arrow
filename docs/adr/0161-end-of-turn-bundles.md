---
id: ADR-0161
title: No end-of-turn hooks; agents run the turn-end, ready and hygiene bundles
status: accepted
date: 2026-10-05
deciders: [paul-heyse]
level: decision
principles: [AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: "not-required: agent tooling adopted from project-template, where it is render-tested; this repository adds only its step lists and instruction updates"
evidence: Tested
supersedes: [ADR-0143]
superseded-by: null
revisit: Agents repeatedly end turns with unformatted code or a stale ADR index, or an environment change goes unnoticed until a later command fails.
verification: just turn-end and just ready run to "all passed"; the check_hook_wiring tests in scripts/tests/test_setup.py (PreToolUse only); just lint-agents.
standard: null
scenarios: []
---

# ADR-0161: No end-of-turn hooks; agents run the turn-end, ready and hygiene bundles

## Context

ADR-0143 kept a Stop/UserPromptSubmit pipeline (`scripts/after_turn.py`) to run `fmt`, the ADR index,
skill synchronization, readiness and the library catalog. That pipeline needed a background job, a
lock, a report file and operator-only messages to run a handful of recipes that agents can run
themselves. The maintainer decided on 2026-10-05 to remove it, along with its counterparts in
project-template, library-context, corpus-intelligence, arrow-polars-duckdb and thermo-knowledge.

## Scope

This record binds agent tooling: the justfile bundles, the hook wiring in `.claude/settings.json` and
`.codex/hooks.json`, and the instructions that say when each bundle runs. The `agent-hooks.py`
edit guard (PreToolUse) is unchanged.

## Outcome

There are no end-of-turn hooks. Three bundles each run their steps, keep going after a failure and
list the failures:

- `just turn-end` (`adr-index`, `fmt`): the root agent runs it at the end of a turn that changed
  files. Subagents don't.
- `just ready` (`skills-sync`, `doctor`): run it after a dependency, toolchain or skill-selection
  change, or after an environment-shaped failure.
- `just hygiene` (unchanged list): run it once at scope end, then fix what fails.

Dropped from the automatic steps:
- `agent-config-sync`, which only materializes skill aliases where symlinks are unavailable;
  bootstrap runs it.
- `db-status`, because `doctor`'s opstore check covers reachability, version and the schema
  fingerprint.
- `library-catalog`, which stays a manual recipe with no prescribed timing.

### Consequences

- Formatting depends on the agent running `just turn-end`; nothing formats behind it.
- Codex re-trusts the changed `.codex/hooks.json` once.

## Status history

- 2026-10-05 — accepted.
