# R2: Getting a plan-scope query into an agent's context (Claude Code and Codex)

**Role:** library-research worker. Read-only for the repository; this file is the only output.
**Scope:** narrowed by the coordinator on 2026-10-09. The direction is light structure in plan
documents plus a query that agents run to fetch the active sections they must update, with what
to consider for each. Hook-based auto-fix and Git hooks are reduced to one line (§4).

**Sources and dates (all read 2026-10-09):**

- Claude Code docs: `code.claude.com/docs/en/hooks`, `/skills` and `/mcp`. The installed CLI is
  **2.1.296**.
- Codex docs: `developers.openai.com/codex/hooks` and `/codex/skills`, which now redirect to
  `learn.chatgpt.com/docs/hooks` and `/docs/build-skills`. The pages carry no version or date.
  The installed CLI is **codex-cli 0.162.0**; I checked strings in its musl binary.
- Repository, read-only: `.claude/settings.json`, `.codex/config.toml`, `.codex/hooks.json`,
  `.mcp.json`, `scripts/agent-hooks.py`, `docs/dev/agent-environment.md`,
  `docs/dev/documentation.md`, ADR-0161, `.codex/skills/*`, and `~/.codex/config.toml`
  (its `[hooks.state]` trust records).

**Evidence labels:**

- *Interface-checked* means read from official docs or the shipped binary.
- *Tested* and *Measured* name the command that was run.
- A missing feature in a doc page or a binary string search is **not** proof of absence. Each
  absence claim below names what was searched.

## 0. Baseline: the query already exists

`scripts/document_lifecycle.py scope --state open` is a read-only JSON projection of the bound
native plan rows (`docs/dev/documentation.md` §Metadata and aggregate scope). Each item carries:
`document`, `binding`, `native_id`, `state`, `raw_status`, `owner`, `source#anchor`,
`source_line` and `native_cells` (for example "Required evidence or settling decision").

That covers the **"what to consider"** half. What is missing is a narrow, per-plan or per-section
rendering. Measurements of the current query:

- **Measured, latency.** `.venv-docs/bin/python -m scripts.document_lifecycle scope --state open`
  took 0.38 s and 0.35 s wall time (two runs, warm). It exited 0.
- **Measured, size.** The output is **175,821 bytes** for 127 open items:
  - 30 items in plan 28;
  - 20 in plan 29;
  - 20 in plan 33;
  - the rest spread over the 28a–28h plans and plan 32.

  A one-line-per-row rendering (path, id, state) is still about **17,500 characters**.
- **Measured, admission stall.** Run through `scripts/pse-env --docs -- …`, the same query blocked
  for **30.07 s** and then exited **125** (`AdmissionError: Admission deadline exhausted:
  exclusive heavy-work owner`). An exclusive native run held the machine at the time.

  A context query used from hooks or skills must therefore **not** go through workload
  admission. The fixed `just activity` observer already uses this exception pattern
  (agent-environment guide §One environment boundary).

## 1. The lowest-effort way for both runtimes to run the query and see its result

The ranking below is by agent effort, reach to both runtimes, and reliability.

### 1. A CLI or `just` command (best base; reaches both)

A shape such as `just plan-scope [<plan-glob>] [--section <anchor>] [--state open]` would print
compact text by default and JSON with `--json`. Output would hold one line per open row, plus
the "consider" cells for that section only.

- **Reach.** Both runtimes always have a shell tool, and Codex shell output is already model
  context. No trust, approval or opt-in step is involved.
- **Freshness.** Results are fresh at the moment of use, because the command runs exactly when
  the agent is about to edit.
- **Token cost.** It is proportional to what was asked for. A single plan's section is typically
  well under 2k characters (an estimate, not a measurement: 4–30 rows per plan).
- **Requirement 1, no admission gate.** It must bypass workload admission: either an observer
  exception like `just activity`, or a direct `.venv-docs` interpreter call (see the Measured
  stall in §0).
- **Requirement 2, a narrow default.** The full open set is 176 kB, so a bare invocation would
  flood the context.

  Whether to default to "plans touched on this branch or in the working tree" is the
  coordinator's call. `git diff --name-only` makes that cheap.

### 2. A skill that points at, or bundles, the command (the routing layer)

This is the layer that makes the agent run the command at the right moment; see §2. One skill
directory serves both runtimes, because `.claude/skills` and `.agents/skills` are both symlinks
to `.codex/skills` (*Tested*: `ls -la`). The script can stay in `scripts/`, and the skill names
the command.

### 3. A SessionStart injection of a compact digest (useful only as a pointer)

**Claude Code (*Interface-checked*, hooks doc):**

- Plain stdout on exit 0, or `hookSpecificOutput.additionalContext`, is added to context.
- Matchers are `startup|resume|clear|compact|fork`. Hooks run in the background while the user
  types; the first response waits for them.
- `additionalContext` is capped at **10,000 characters** per string. Anything longer is saved
  to a file, and Claude sees a 2,000-character preview.

**Codex (*Interface-checked*, hooks doc):**

- `SessionStart` exists. Plain stdout is added "as extra developer context", as is JSON
  `hookSpecificOutput.additionalContext`.
- Matchers are `startup|resume|clear|compact`.
- Output is limited to about **2,500 tokens** by default (`additionalContextLimit`).
- Hooks are on by default. Changed hooks must be **re-trusted** (`/hooks`): trust is recorded
  against the hook's hash, and `~/.codex/config.toml` already holds a `pse-arrow …
  pre_tool_use` trust record.

**Fit.**

- The full scope (176 kB) and even the compact rows (17.5k characters) exceed both caps.
- What fits is a digest, under about 1.5k characters: one line per active plan with its open,
  blocked and deferred counts, plus the command to run for detail.
- Freshness is a snapshot at session start, which goes stale during a long session.
  Re-firing on `compact` and `resume` refreshes it at the moments that matter.
- The cost recurs every session. The repository's Claude SessionStart hook already exists
  (`agent-hooks.py session-env`), so this would be one more action on the same script.
- Codex would need a new `SessionStart` entry in `.codex/hooks.json` and one re-trust.

### 4. An MCP tool or resource (works, but is not the lowest effort here)

- **Claude Code** prompts for approval of project `.mcp.json` servers in interactive sessions.
  MCP output warns above 10k tokens and is cut at 25k by default (*Interface-checked*, mcp doc).
- **Codex has `read_mcp_resource` / `list_mcp_resources` handlers.** I found them as strings in
  the binary, for example `core/src/tools/handlers/mcp_resource/read_mcp_resource.rs`
  (*Interface-checked*, binary only).
- **Codex is opt-in for this repository.** `.codex/config.toml` disables the `library-catalog`
  server by default because each thread starts its own server. A Codex agent therefore usually
  will not have the tool.

The verdict: an MCP tool adds a server lifecycle and an opt-in gap for the same JSON the CLI
already prints. Folding a `plan_scope` tool into the existing read-only `library-catalog` server
is possible, but reaches Codex only when that server is enabled.

### 5. An `@`-import of a generated file in CLAUDE.md (not recommended)

- **Claude only.** Codex AGENTS.md has no import syntax, and none appears in the Codex docs read.
- **Always loaded.** It costs tokens every turn.
- **Stale.** It is only correct if something regenerates the file. The regenerating "something"
  is exactly the hook or bundle machinery the maintainer finds cumbersome.

## 2. Can a skill route the agent to the query at the moment it updates a plan, without extra rules?

**Yes, on Claude Code, close to deterministically. On Codex, only probabilistically.**

**Claude Code (*Interface-checked*, skills doc, 2.1.x):**

- The SKILL.md frontmatter field `paths` takes globs. "When set, Claude loads the skill
  automatically only when working with files matching the patterns." A skill with
  `paths: ["docs/plans/**"]` therefore surfaces when the agent reads or edits a plan file,
  with no rule file involved.
- The body can use dynamic injection: `` !`command` `` "runs shell commands before the skill
  content is sent to Claude", and the output replaces the placeholder.
  - A failing command aborts the invocation.
  - The command is checked against permission rules and never prompts.
  - The default timeout is 2 minutes.
  - `$ARGUMENTS` (or named `arguments`) can carry the plan path, for example
    `` !`just plan-scope $ARGUMENTS` ``. The agent then receives the live section rows with the
    skill text, at zero extra steps.
- One caveat: an injected command that fails, for example through admission, aborts the whole
  skill. That is another reason the command must avoid `pse-env` admission.

**Codex (*Interface-checked*, build-skills doc, plus binary strings):**

- Skills are discovered in `.agents/skills` from the working directory up to the repository root.
- Only `name` and `description` are documented frontmatter fields.
- Implicit use happens when "your task matches the skill `description`" (progressive disclosure:
  names and descriptions first, the full SKILL.md once selected).
- The documented invocation policy is `agents/openai.yaml` `allow_implicit_invocation`.
- **Not found** in either the docs page or the binary strings: a path-triggered activation, and
  `!`-style command injection. (The only `disable-model-invocation` string sits inside bundled
  skill-creator prose.) The agent must read the skill and then run the named command, which is
  one extra tool call.

**Lowest-effort routing, both runtimes, no new rule.** Plan edits already flow through
`create-plan` / `plan-creation` and `execute-plan` / `plan-execution`. Two changes would do it:

- Add one sentence to those skill bodies: "before changing a plan's tracked tables or
  checkpoint, run `just plan-scope <plan>`; update rows from its output." Both runtimes then hit
  it whenever they are already in plan work.
- For direct plan edits outside those skills, a small `plan-update` skill could cover both:
  - a description that front-loads the trigger words, for example "Update a plan's packet,
    finding or checkpoint rows…", for Codex's description matching;
  - `paths: ["docs/plans/**"]` plus a `` !`…` `` render, for Claude.

**If routing must be deterministic in Codex too,** the only lever is an inject-only edit hook.

- **Claude:** `PreToolUse` and `PostToolUse` both support `additionalContext`.
- **Codex:** `PostToolUse` fires for `apply_patch`, matchable as `Edit|Write`, and supports
  `hookSpecificOutput.additionalContext`. The Codex doc does not state the PreToolUse context
  fields.
- **The hook** would recognize a `docs/plans/**` path and print that plan's open rows.
- **Cost:** it fires on every plan edit, adds a re-trust in Codex, and contradicts the "no new
  hooks" lean. It modifies nothing, so it cannot collide with the agent's edits.

## 3. Parity gaps (brief)

- **Path-triggered skill:** Claude has `paths` frontmatter. In Codex, none was found in the docs
  or the binary strings.
- **Command injection in a skill:** Claude renders `` !`cmd` ``, and `${CLAUDE_SKILL_DIR}` points
  at bundled scripts. Codex has neither (none found); its agent runs the script itself.
- **SessionStart context:** both runtimes have it (stdout or `additionalContext`). The caps
  differ: 10k characters for Claude, about 2.5k tokens default for Codex. Codex needs hash trust
  on every change.
- **MCP:** Claude asks for approval once per server. In this repository's Codex config, servers
  are opt-in per session.
- **Instruction imports:** Claude CLAUDE.md supports `@` imports. Codex AGENTS.md has none, and
  the repository already raises `project_doc_max_bytes` to 64 KiB.
- **Subagent context:** both runtimes have `SubagentStart` with `additionalContext`. The Claude
  docs show it; the Codex doc lists the event and says subagent hooks use the parent session id.
- **The one lever reaching both identically** is a shell command named in a shared skill. A
  shared hook script is second best, with differing caps and trust.

## 4. Hook-based auto-fix and Git hooks: one line

Neither is clearly worth it here:

- **PostToolUse auto-repair** runs after the edit. `updatedToolOutput` only changes what the
  model sees; a hook that rewrites the file the agent just edited risks stale-read conflicts.
- **Git pre-commit fixers** touch the shared index while several agents keep uncommitted work in
  one checkout. The repository installs no Git hooks today (*Tested*: `.git/hooks` holds only
  `*.sample`).

The only hook with a clean fit is the inject-only edit hook in §2.

## 5. Summary table

| Lever | Runtimes reached | Can modify files | Can inject context | Failure modes | Fit |
|---|---|---|---|---|---|
| `just plan-scope` / CLI | Claude, Codex (shell) | No (read-only) | Yes, as tool output when run | Admission stall if routed through `pse-env` (Measured 30 s, exit 125); flooding if the default is unscoped (176 kB) | **Primary query**; fresh at use |
| Skill naming the command (existing plan skills, plus an optional `plan-update`) | Both, through the shared `.codex/skills` symlinks | No | Claude: yes, via `` !`cmd` `` render; Codex: agent runs the command | Codex implicit selection is probabilistic; a Claude render fails the whole skill if the command fails | **Routing at edit time**, no new rule |
| Claude skill `paths: docs/plans/**` | Claude only | No | Yes, auto-loads on plan files | Description-listing budget; still model-mediated | Strongest Claude trigger |
| SessionStart digest | Claude (existing hook); Codex (new entry plus re-trust) | Hooks could, but this one should not | Yes (Claude 10k-character cap; Codex about 2.5k-token cap) | Stale after edits; recurring token cost; must stay under about 1.5k characters | "What is active" pointer only |
| Inject-only PreToolUse/PostToolUse on plan edits | Claude (Pre and Post); Codex (Post via `apply_patch`) | No (by design) | Yes, per edit | Per-edit noise; Codex hash re-trust; departs from the no-new-hooks lean | Only if routing must be deterministic in Codex |
| MCP tool or resource | Claude (after approval); Codex only when opted in | No | Yes, on call | Server lifecycle; Codex opt-in gap; output caps | Not lowest effort here |
| `@`-import of a generated file | Claude only | No | Yes, always loaded | Stale without regeneration; constant cost | Not recommended |
| PostToolUse auto-repair / Git pre-commit fixers | Both / runtime-neutral | Yes | No / no | Edit races, a shared index with concurrent uncommitted work | Not worth it (§4) |
