@AGENTS.md

# Claude Code specifics

This is a real file so the shared import survives a Windows checkout without symlinks.

`.claude/settings.json` configures permissions and the edit hook; AGENTS.md
describes which permission layers survive an unprompted session. Both Claude
and Codex call `scripts/agent-hooks.py`, whose pre-edit checks protect generated files and
decided documents. There are no end-of-turn hooks (ADR-0161): run `just turn-end` at the end of a
turn that changed files, `just ready` after an environment change and `just hygiene` at scope end.
The guard does not stand between you and your own runtime directories — memory, scratch
space and runtime configuration are writable, under the scope AGENTS.md describes.
Hook timeouts are seconds. Shell writes remain governed by AGENTS.md; the edit hook
is not a shell sandbox. Do not ask again for actions already authorized by the user.

Path-scoped guidance is in `.claude/rules/`. Shared role behavior is in
`.agents/roles/`; `.claude/agents/` and `.codex/agents/` are independent native adapters.
Use the named Claude roles from the shared routing table: `implementer` is the executor adapter
on Opus/medium, and `design-reviewer` runs on Opus/high. The coordinator owns design, integration and
acceptance; explicit user runtime choices take precedence. Native definitions load shared
contracts rather than copying role behavior between runtimes.

Skills are canonical in `.codex/skills/`, exposed through `.claude/skills` and `.agents/skills`;
library skills there are local-only and gitignored. On Windows, `just agent-config-sync`
materializes skill aliases when symlinks are unavailable; it never regenerates native agents.
`just lint-agents` checks references, aliases and shared contracts; `just setup-test` checks behavior.

Use planning for architecture, pinned-family-major, generation and Python-boundary
changes. Adding or upgrading a third-party dependency is not one of them: no library and no
licence is refused during phases 0-1, and versions float under the lockfiles
(`docs/dev/dependency-policy.md`, ADR-0066, ADR-0159). Apply the ADR
criteria in AGENTS.md: touching those files alone does not make every bug fix an
architecture decision. Use the `adr` and `design-review` skills when
required. Plans belong in `docs/plans/`, never in a runtime's private home directory.
Current contracts and their rationale belong in `docs/authoritative_design/sections/` and
the retained ADRs; when work closes, move its enduring meaning there and retire the plan
and resolved reviews to Git history (ADR-0096).
