@AGENTS.md

# Claude Code specifics

This is a real file so the shared import survives a Windows checkout without symlinks.

`.claude/settings.json` configures permissions and the lifecycle hooks; AGENTS.md
describes which permission layers survive an unprompted session. Both Claude
and Codex call `scripts/agent-hooks.py`, whose pre-edit checks protect generated files and
decided documents, and `scripts/after_turn.py`, the end-of-turn pipeline (ADR-0137): when the
main agent stops it formats, regenerates and runs every `just hygiene` check, a headless fixer
(Sonnet 5.5 at the session's effort) repairs what it can, and the maintainer sees the rest. It is
silent to the model; subagents stopping don't trigger it. No hook formats edits as you work.
The guard does not stand between you and your own runtime directories — memory, scratch
space and runtime configuration are writable, under the scope AGENTS.md describes.
Hook timeouts are seconds. Shell writes remain governed by AGENTS.md; the edit hook
is not a shell sandbox. Do not ask again for actions already authorized by the user.

Path-scoped guidance is in `.claude/rules/`. The role definitions in
`.claude/agents/` are canonical; `just agent-config-sync` produces Codex's native TOML
roles. Delegate every code change through the `implementer` role, which runs on
Sonnet 5.5: never hand edits to a general-purpose agent or a fork, and never pass a model
override when spawning it. `design-reviewer` runs on Opus; the built-in exploring and
planning agents keep their defaults. Skills are canonical in `.codex/skills/`, exposed through `.claude/skills` and
`.agents/skills`; library skills there are local-only and gitignored. On Windows, run
`just agent-config-sync` to materialize aliases when symlinks are unavailable.
`just lint-agents` checks drift and `just setup-test` checks behavior.

Use planning for architecture, pinned-family-major, generation and Python-boundary
changes. Adding a third-party dependency is not one of them: no library and no licence is
refused during phases 0-1 (`docs/dev/dependency-policy.md`, ADR-0066). Apply the ADR
criteria in AGENTS.md: touching those files alone does not make every bug fix an
architecture decision. Use the `adr` and `design-review` skills when
required. Plans belong in `docs/plans/`, never in a runtime's private home directory.
Current contracts and their rationale belong in `docs/authoritative_design/sections/` and
the retained ADRs; when work closes, move its enduring meaning there and retire the plan
and resolved reviews to Git history (ADR-0096).
