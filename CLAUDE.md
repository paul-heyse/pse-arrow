@AGENTS.md

# Claude Code specifics

Everything above applies. This file adds only what is Claude-specific.

A real file rather than a symlink to `AGENTS.md`: `windows-2022` is in the CI matrix,
and a symlink does not survive a checkout there without developer mode.

## At the start of a session

The `SessionStart` hook runs `scripts/doctor.py`, so the environment status is already
in context. If it reported a blocking issue, fix that first — a `just test` in a tree
with no solver libraries is a wasted loop with a confusing error.

## What is configured for you

- **`.claude/settings.json`** — read-only `just`, `cargo`, `git`, `gh` and `.venv/bin`
  commands are pre-approved. Anything that mutates history, GitHub, the lockfiles or
  PyPI asks first. Writes under `.git/`, `target/`, `build/`, `external/`,
  `docs/generated/`, `crates/*/src/generated/` and `python/pse/contracts/` are denied.
  `UV_PROJECT_ENVIRONMENT=.venv` is set so `uv` and the justfile agree.
- **`.claude/hooks/`** — `SessionStart` runs the doctor; `PreToolUse` blocks writes to
  the protected paths (including the blueprint and accepted ADRs, unless
  `PSE_DESIGN_EDIT=1` is set); `PostToolUse` formats the file you just edited with the
  pinned `cargo fmt` / `ruff` / `taplo`, so a formatting-only CI failure cannot happen.
  The formatter never fails the tool call — a formatter problem is not a reason to lose
  an edit. There is deliberately no per-edit compile hook: `just check` is fast enough
  to run when you mean it.
- **`.claude/rules/`** — path-scoped guidance that loads only when you touch matching
  files: `rust.md`, `python.md`, `docs.md`, `decisions.md`, `generated.md`, `ci.md`.
- **`.claude/skills`** is a symlink to `.codex/skills`, and **`.codex/agents`** is a
  symlink to `.claude/agents`, so both runtimes read one copy of each. On Windows
  without developer mode git checks those out as text files rather than links; if your
  skills or agents look like one-line files, that is why. `just lint-agents` verifies
  both and checks that every path they name still exists.

## Use plan mode when

The change touches a D1–D14 decision, adds a crate or a dependency family, touches
codegen (`crates/*/src/generated/`, `python/pse/contracts/`, `docs/generated/`, the
Ipopt bindings), or touches the Python boundary (`crates/pse-py/`, `python/pse/`).
Those changes need an ADR and cost more to unwind than to plan.

## Subagents available

- `architecture-executor` — executes a plan document that spans subsystems, with
  judgment when the plan and the tree disagree.
- `precision-integrator` — lands a feature that has to fit exactly into existing
  contracts and cascades across modules.
- `impl-plan-exec-subagent` — executes one handoff packet from a plan, end to end.
- `task-executor` — one discrete, well-specified subtask; good for parallel fan-out.
- `plan-scout` — preflights a plan against the tree, mapping steps to files and
  symbols. Read-only.
- `plan-auditor` — verifies afterwards that every plan step actually landed. Read-only.
- `design-reviewer` — rates a code scope against the design charter.
- `library-leverage-reviewer` — finds custom code that a pinned library already does.
- `architecture-docs-writer` — writes structured architecture documentation only.

## Skills

- `design-review` — a full charter review (DM-01–DM-60, gates G1–G7) of a design
  document or a code scope, written to `docs/design_review/reviews/`.
- `adr` — when a decision needs a record, and how to fill the §H front-matter fields.

## Writing plans

Plans go in `docs/plans/` (`just plan <slug>` creates the stub), never in
`~/.claude/plans`. A plan in a home directory is invisible to Codex and to the next
session; in the repo it is readable by both and reviewable in a pull request.
