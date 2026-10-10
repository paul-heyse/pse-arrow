# R4: Lifecycle tooling survey for agent-maintained Markdown work documents

**Role, sources and dates.** This is a library-research survey (worker and library-research
contracts) for the maintainer's question: is there tooling that improves the lifecycle of
reviews, plans, evidence and instructions by acting programmatically, without asking agents to
perform more compliance steps? It recommends patterns, not an architecture. The coordinator
decides adoption. The baseline is the `main` checkout at 4c24721e6, with uncommitted work
present. Versions, dates, stars and licences come from the npm and PyPI registries and the
GitHub API, all fetched 2026-10-09. Behaviour claims come from:

- official READMEs and docs, through WebFetch and Context7 (`/fission-ai/openspec`,
  `/basicmachines-co/basic-memory`);
- upstream source read through `gh api`: the beads telemetry and metrics packages and FAQ,
  and spec-kit's `templates/commands/converge.md`, `docs/guides/evolving-specs.md` and
  `tasks-template.md`;
- six hands-on trials on this Linux host, run on 2026-10-09 against copies of Plan 33 in
  scratch.

An earlier report (`../D-library-alternatives.md`) already covered document *formats* such as
MyST, Markdoc, sphinx-needs, StrictDoc, Doorstop and DVC. This survey covers only lifecycle
automation.

**Evidence labels.**

- *Tested*: I exercised the behaviour myself, in scratch.
- *Measured*: a number from such a run.
- *Interface-checked*: documentation or source reading only, with no run.

**Trial conditions.**

- Every trial ran in a session scratch directory, with `HOME` redirected into it.
- Binaries were the official GitHub release assets: bd 1.3.1, mdq 0.10.0, mq 0.9.2 and
  iwe 0.26.1.
- npm packages went into a scratch prefix: backlog.md 1.53.0 and @fission-ai/openspec 1.14.1.
  The `@beads/bd` postinstall step was blocked; I used the release binary instead.
- I set `DO_NOT_TRACK=1` and `OPENSPEC_TELEMETRY=0` for Backlog.md and OpenSpec.

**Deviation to flag.** beads ships **default-on, anonymous command-usage metrics**. They are
POSTed to `gastownhall-eventsapi.com`, and the FAQ says each event carries the command name,
the bd version, the OS and an HMAC machine ID, with no issue content, paths or repository data.

- I did not disable them before the first beads commands.
- A detached flush was spawned once, at 22:21:39 local time. I cannot tell whether it
  succeeded.
- I then ran `bd metrics off` and deleted the queued events.
- Neither repository nor issue content was in scope of what the metrics send.

The other tools showed no telemetry. I searched their GitHub code for `telemetry` and
`posthog`; that search is not proof of absence.

---

## 0. What the repository already does (baseline for "beyond what we have")

- **`scripts/document_lifecycle.py`.** Read-only commands `inventory`, `validate`, `scope` and
  `retire-plan`, which read native Markdown tables through markdown-it. `scope --state open`
  is already the "query the active sections" direction. `retire-plan` *lists*
  `references_to_repair`, but it repairs nothing.
- **`scripts/adr.py`.** The ADR index, new and supersede.
- **Search.** Pagefind over the built site, and the `library-catalog` MCP server for library
  facts.
- **A *Measured* weakness of `scope` on Plan 33** (`.venv-docs/bin/python -m
  scripts.document_lifecycle scope --path 'docs/plans/33*'`):
  - It reports 29 items: 20 open, 9 unknown, 0 complete.
  - The six EI packets are `unknown`, because they have no status column.
  - EFF03, EFF07 and EFF08 are `unknown`, because their prose status ("Integrated under
    ADR-0171; …") does not match the profile.
  - The other "Integrated …" rows count as `open`.

  No external tool fixes this; it is a vocabulary question (§8, pattern 2).

---

## 1. Agent-oriented planning and task tracking in the repository

### Beads (`bd`): gastownhall/beads, formerly steveyegge/beads

- **Version and activity.** 1.3.1 (2026-09-30); MIT; about 27.8k stars; pushed 2026-10-10;
  more than 11k commits.
- **Storage.** A **Dolt** database, embedded by default under `.beads/embeddeddolt/`, which
  `bd init` adds to `.gitignore`. `.beads/issues.jsonl` is an export, "not the source of
  truth". Sync goes through `bd dolt push/pull` to `refs/dolt/data`. Dolt merges at cell
  level. IDs are hashes (`p33-52c`, children `p33-52c.1`). The database carries a schema
  version, and a binary refuses a database migrated by a newer one.
- **Query surface.**
  - CLI commands `ready`, `blocked`, `show`, `prime`, `remember`, `epic status` and
    `epic close-eligible`.
  - `--json` everywhere.
  - The `beads-mcp` 1.3.1 MCP server.
  - Claude, Codex and Cursor hooks, which call `bd prime` automatically.
- **Trial** (*Tested*).
  - Expressing a Plan 33 slice: an epic, EFF00, EFF02, EFF09, EFF10 and EFF11, and F03 as a
    task blocked by EFF09 and EFF11.
  - `bd ready` and `bd blocked` computed the dependency closure correctly.
  - Closing EFF09 dropped it from F03's blockers, and `bd epic status` reported "1/5 children
    closed (20%)".
  - Six parallel `bd create` calls in one checkout all succeeded, serialized by the embedded
    lock (*Measured*: 6/6).
  - `bd init` **made a git commit by itself** (`bd init: initialize beads issue tracking`,
    five files) and wrote `~/.dolt` and `~/.beads`.
- **Compliance asked** (*Measured*). `bd prime` emits 842 words of agent rules:
  - a "SESSION CLOSE PROTOCOL";
  - "Use beads for ALL task tracking";
  - "Do NOT use TodoWrite, TaskCreate, or markdown files for task tracking";
  - "Create beads issue BEFORE writing code";
  - "Do NOT use MEMORY.md files".
- **Fit.** Issues are short records. Status lives in a binary database outside Git diffs, so
  a plan's prose and checkpoints stay in Markdown and its status moves into a second store.
  That is exactly the restated-status problem, made worse.
- **Verdict.** No. The `ready` closure and `epic status` roll-up are the parts worth having,
  and they are computable from what plans already declare.

### Backlog.md: MrLesk/Backlog.md

- **Version and activity.** 1.53.0 (2026-09-24); MIT; about 7k stars; active. Its README
  says "no telemetry".
- **Storage.** One Markdown file per task, with YAML front matter (`id`, `status`,
  `dependencies`, `labels`), under `backlog/tasks/`. Body sections sit between marker
  comments (`<!-- AC:BEGIN -->`, `<!-- SECTION:NOTES:BEGIN -->`). Docs and decisions live in
  sibling folders. `cleanup` moves done tasks to `completed/`.
- **Query surface.**
  - `task list --plain|--json`, with `--ready`, `-s`, `-l` and `--search`.
  - `task <id> --plain`, which renders the dependency tree.
  - `search`, `overview` and `doctor`.
  - An MCP server (`backlog mcp start`), whose workflow guides are exposed as resources.
- **Trial** (*Tested*).
  - Five packets and three findings, with dependencies.
  - `--ready` listed exactly the unblocked items.
  - `task eff-7` drew the full transitive dependency tree with each node's status.
  - A finding's own status did **not** roll up: it stayed "To Do" after its owner packet was
    done.
- **Concurrency** (*Tested*). Two branches each edited EFF-4's notes, closed different tasks
  and created one new task each.
  - Git merged the different-task edits cleanly.
  - The same-section notes conflicted, as expected.
  - **Both new tasks got ID `EFF-9`**, because IDs are sequential. The CLI then refused
    ID-based commands ("ID-based commands are blocked until the collision is repaired") and
    pointed to `backlog doctor`. The collision was detected, but only after the merge.
- **Compliance asked** (*Measured*).
  - Guides of 557 words (overview), 951 (creation), 639 (execution) and 397 (finalization).
  - "Do not edit Backlog task … markdown files directly. Use Backlog commands".
  - One task per session; write a plan and wait for approval.
- **Fit.** It forces a task-card shape, and rich prose lives inside marker-fenced sections.
- **Verdict.** No. Useful as a reference for `--ready`, the dependency-tree view and the
  `doctor` duplicate-ID repair.

### ticket (`tk`): wedow/ticket

- **Version and activity.** 0.3.2 (2026-02-03); MIT; about 900 stars; a single bash script.
- **What it is.** Markdown files with front matter in `.tickets/`, hash-like IDs and
  partial-ID matching.
- **Commands.** `ready`, `blocked`, `dep tree`, `dep cycle`, `query [jq]` and plugins
  (`tk-<cmd>`). Its pitch against beads is "no SQLite to keep in sync, no daemon".
- **Compliance.** One line in AGENTS.md.
- **Verdict.** The lightest shape in this family, and proof that `ready`, `blocked` and
  cycle checks need about 300 lines over Markdown. Still a separate task store, so not
  adopted. (*Interface-checked*)

### git-bug

- **Version and activity.** 0.11.0 (2026-09-22); GPL-3.0; about 10.7k stars.
- **What it is.** Issues stored inside git, adding no files, with CLI, TUI, web UI, GraphQL
  and GitHub, GitLab and Jira bridges.
- **Limits.** Status is open/closed only, with no dependencies, and the content is invisible
  to grep and to agents reading files.
- **Verdict.** No. (*Interface-checked*)

### Task Master: eyaltoledano/claude-task-master

- **Version and activity.** `task-master-ai` 0.43.1 (2026-03-31; no release since); MIT
  with Commons Clause.
- **What it is.**
  - `.taskmaster/` holds task JSON, tags and a PRD.
  - The CLI has `parse-prd`, `next`, `show` and `research`.
  - The MCP server has 36 tools.
  - It needs a provider key, or the Claude Code or Codex CLI as provider.
- **Automation.** LLM decomposition of a PRD into tasks, `next` and complexity analysis.
- **Fit.** It forces a JSON task list.
- **Verdict.** No: it adds an LLM-driven layer and duplicates plan status.
  (*Interface-checked*)

### GitHub Spec Kit (`specify-cli`)

- **Version and activity.** 1.1.3 (2026-10-09); MIT; about 140k stars.
- **What it is.**
  - Per-feature `specs/NNN-*/` with `spec.md`, `plan.md`, `tasks.md`, `research.md`,
    `data-model.md` and `contracts/`.
  - A constitution, plus extensions, presets and hooks in `.specify/`.
  - Commands `/speckit.specify`, `plan`, `tasks`, `implement`, `analyze`, `clarify`,
    `checklist`, `converge` and `taskstoissues`.
- **How it works.** I read the `converge` template: every lifecycle step is a **prompt the
  agent executes**. "Assess the current codebase against the feature's spec, plan, and tasks,
  then append any remaining unbuilt work as new tasks". The CLI only scaffolds and runs
  prerequisite-check scripts.
- **Spec evolution.** `evolving-specs.md` describes "flow-forward", "living" and "flow-back"
  persistence. Each ends with "update any other artifacts that now disagree", done by the
  agent.
- **Verdict.** No. All of its lifecycle work is agent effort; nothing is computed.
  (*Interface-checked*)

### OpenSpec: Fission-AI/OpenSpec

- **Version and activity.** 1.14.1 (2026-10-06); MIT; about 71k stars. Anonymous telemetry
  (command name and version), opt-out.
- **What it is.**
  - `openspec/specs/<cap>/spec.md` is the current truth.
  - Each `openspec/changes/<id>/` holds `proposal.md`, `tasks.md`, an optional `design.md`
    and `specs/` deltas (`## ADDED|MODIFIED|REMOVED|RENAMED Requirements`).
  - `archive` moves the change to `changes/archive/YYYY-MM-DD-<id>/` and **merges the deltas
    into the specs**.
- **Trial** (*Tested*).
  - EFF09 written as a change.
  - `openspec list` showed "2/3 tasks".
  - `status` showed the artifact checklist and the exact next command.
  - `validate --all` passed.
  - `archive -y` created `specs/resource-coordination/spec.md` with "+ 1 added". It
    **archived despite an unchecked task**.
  - `validate --archived` then failed with "1 incomplete task (2/3 completed)". The check
    exists, but it runs separately, not as a gate on archive.
- **Compliance asked.** The SHALL + WHEN/THEN requirement shape, delta headers, and the
  propose → apply → archive sequence through slash commands.
- **Fit.** Specs-as-current-truth is the same idea as this repository's architecture
  sections, and change folders resemble plans. But the requirement grammar is narrower than
  the repository's prose contracts.
- **Verdict.** No as a tool. **"Archive as one command"** (move, merge, validate) is the
  pattern to copy.

### Other spec frameworks (*Interface-checked*)

- **Kiro specs.** A proprietary AWS IDE and CLI. `.kiro/specs/<f>/` holds `requirements.md`
  (EARS), `design.md` and `tasks.md`. The IDE's task runner shows live status and runs
  independent tasks. Verdict: no; the automation is IDE-bound.
- **Conductor** (Gemini CLI extension, 0.4.1, 2026-03-11, Apache-2.0).
  `conductor/tracks/<id>/{spec.md,plan.md,metadata.json}`, with checkbox progress maintained
  by the agent. It warns that it conflicts with built-in plan mode. Verdict: no; it is
  prompt-driven.
- **Agent OS** (3.0.0, 2026-01-20, MIT). Standards discovery and injection plus spec shaping.
  No status tracking. Verdict: no.
- **BMAD method** (6.12.1, 2026-10-04). Agents and workflows, story files and
  `sprint-status.yaml`. A reported v6-alpha bug had code review marking a story done in its
  file while `sprint-status.yaml` still showed the old status. That is this repository's
  restated-status failure inside a framework built to prevent it. Verdict: no.
- **Claude Code built-in Tasks.** Reported, but not verified from official docs.
  - Tasks persist under `~/.claude/tasks/`, with `blockedBy` and `blocks`.
  - `CLAUDE_CODE_TASK_LIST_ID` shares one list across sessions.
  - They live outside the repository, and Codex cannot see them.
  - Verdict: in-session coordination only, not lifecycle.

---

## 2. Agent knowledge and memory over Markdown

The question here: can a tool answer "what do we already know about X" and "what cites this"
with no tagging burden?

### Basic Memory (basicmachines-co/basic-memory)

- **Version and activity.** 0.23.2 (2026-08-25); AGPL-3.0; about 4.1k stars.
- **What it is.**
  - Markdown files plus a SQLite index, with FTS and sqlite-vec hybrid search.
  - MCP tools `search_notes` (with metadata filters), `build_context`, `write_note` and
    `edit_note`.
  - It **indexes ordinary Markdown links as `links_to` relations**, so it can answer "what
    cites this" with no tagging.
  - It can also take optional `- [category] fact` observations and `- relation [[Target]]`
    lines.
- **Blocking issue** (*Interface-checked*, `config_models.py`).
  `ensure_frontmatter_on_sync` defaults to true, and "Canonical permalinks are always added".
  Sync **rewrites front matter in every indexed file**, which would touch ADRs, the
  architecture sections and generated docs.
- **Verdict.** No for repository docs. It is acceptable only for a separate notes folder.

### Obsidian: 1.12 CLI, Bases, Dataview 0.5.70

- The official CLI (from 1.12, February 2026) **drives a running desktop app**, so it is not
  headless. Server use needs Xvfb workarounds.
- Dataview and Bases queries need front-matter or inline fields, which means tagging.
- The app is closed source.
- Verdict: no. (*Interface-checked*)

### Markdown language servers

- **Marksman.** Release 2026-02-08; MIT; about 3.4k stars. Find-references for headings and
  links, rename refactor, wiki-link diagnostics and a TOC code action. A standalone `check`
  CLI is still "planned". It could serve "what cites this" through an LSP client (Claude
  Code has an LSP tool). HTML `<a id>` anchors, which plans use, are probably not resolved.
  (*Interface-checked*)
- **markdown-oxide.** 0.25.12 (2026-06-29); Apache-2.0. A PKM LSP with backlinks.
  (*Interface-checked*)
- **IWE** (iwe-org/iwe). 0.26.1 (2026-10-08); Apache-2.0; about 1.8k stars. A CLI, an LSP
  and an MCP server (`iwec`). Trial results (*Tested*, on a three-file copy with a plan
  index, a review and the Plan 33 copy):
  - `iwe find --referenced-by plans/33-efficiency` correctly listed the index and the review
    as citers. That answers "what cites this" with plain relative links and no tagging.
  - `iwe rename` moved the file and updated both citers. It **dropped every `#fragment`**:
    `33-efficiency.md#finding-dispositions` became `33-efficiency-retired.md`.
  - The rename also **re-serialized the moved file** (*Measured*: 132 diff lines). Flow YAML
    lists became block lists, and every table was re-padded.
  - `iwe delete` turned inbound links into plain text.
  - Verdict: not safe on authored documents. It does prove that rename and retire link
    repair is mechanical.
- **zk.** 0.15.6 (2026-07-26); GPL-3.0. A notes CLI and LSP with a SQLite index and
  `--linked-by`, `--link-to` and `--related` filters. It assumes a zk notebook layout and
  config. Low value over a short script on the existing parser. (*Interface-checked*)

### qmd (tobi/qmd)

- **Version and activity.** 2.8.3 (2026-08-16); MIT; about 30k stars.
- **What it is.** Local search over Markdown collections:
  - BM25 (`search`, needs no model);
  - vector search and hybrid search with query expansion and rerank (`vsearch`, `query`),
    using about 2 GB of GGUF models downloaded from HuggingFace on first use;
  - an MCP server with `query`, `get`, `multi_get`, `status` and `metadata`;
  - path-prefix "context" annotations.
- **Fit.** It answers "what do we already know about X" with no tagging. Search runs locally.
- **Verdict.** It is the strongest candidate for evidence reuse *if* plain-text search misses
  paraphrased questions; the repeated question answered 7 times in 5 days suggests it might.
  Not trialled. Try BM25 first, which ripgrep and Pagefind already approximate.
  (*Interface-checked*)

### Markdown query tools: no migration needed

- **mdq** (yshavit/mdq). 0.10.0 (2026-03-22); Apache-2.0; about 1.75k stars.
- **mq** (harehare/mq). 0.9.2 (2026-09-28); MIT; about 1k stars.
- Both are jq-style selectors over Markdown. Trial results (*Tested*, on the unchanged
  Plan 33 copy):
  - `mdq '# Current checkpoint'` returned the checkpoint section.
  - `mdq '# Implementation packets | :-: /Packet|Progress/ :-: /Planned|In progress/'`
    returned exactly EFF00, EFF10 and EFF11 with their Progress cells.
  - `mdq -o json '# Finding dispositions | :-: /Finding|Disposition|owner/ :-: *'` gave
    structured rows.
  - `mq '.h2'` listed the sections.
- **Derived-view demo** (*Tested*). Joining the two JSON outputs in about 15 lines of Python
  put each finding's disposition next to its owner packets' progress, for example
  `F03 Scheduled {'EFF09': 'Integrated'}`. Nobody has to restate either fact.
- **Limitation** (*Measured*). One mdq invocation on the whole checkpoint section hit an
  output-formatting error ("error while writing char"). Treat these as selector designs to
  borrow. The repository's markdown-it table reader already does the same job.

---

## 3. Docs-to-code freshness

The question here: does any tool flag stale docs, and does any repair them?

| Tool | Detects | Repairs? | Notes |
|---|---|---|---|
| Swimm (commercial SaaS + IDE) | Snippets, tokens and paths in `.swm` docs that changed in code; PR comments | **Yes for trivial changes** ("Auto-sync"); otherwise "Review required" for a human | Only its own doc format, and it needs a hosted app. No. (*Interface-checked*) |
| menard 0.4.0 (2026-04-03, Apache-2.0, a single maintainer, 0 stars) | Section-level links from code paths to doc sections in `.menard/links.toml`; a pre-commit hook blocks when linked code changed and the section did not | No (flags only; JSON for agents) | The right idea (section-scoped, deterministic) at very early maturity. (*Interface-checked*) |
| doc-detective 4.38.1 (2026-08-13, AGPL-3.0) | Runs procedures written in docs (UI and CLI steps) as tests | No | For how-to docs, not lifecycle. (*Interface-checked*) |
| cogapp 3.6.0 (2025-09-21, MIT; nedbat) | `cog --check` reports regenerated blocks that drifted | **Yes**: `cog -r` regenerates the marked blocks in place | Programmatic fix; Python generators inline in the doc. (*Interface-checked*) |
| markdown-magic 4.11.0 (2026-06-29, MIT) | Comment-delimited blocks (`CODE`, `TOC`, custom transforms) | **Yes**: rewrites the blocks | JavaScript counterpart of cog. (*Interface-checked*) |
| embedme 1.22.1 (2022, dormant) | `--verify` checks embedded snippets against source | Yes (re-embed) | Dormant. |
| lychee 0.24.2 (2026-05-01, Apache-2.0), remark-validate-links 13.1.0, markdown-link-check 3.15.0 | Broken links; lychee and remark also check `#fragment` anchors | No | Detection only; mdBook already sees some of this. |

**Conclusion.** Automatic *repair* exists only where the doc content is **generated from a
source**: cog and markdown-magic regeneration, Swimm's snippet auto-sync. Every "stale prose"
detector (menard, Swimm's non-trivial case, LLM drift skills) flags for a human or an agent.

For evidence, the useful form is **fingerprint the inputs and mark the record suspect when
they change**. That is §5's Doorstop pattern and §6's DataLad pattern, and it is
deterministic and cheap.

---

## 4. Decision and record tooling (brief)

- **log4brains** 1.1.0 (2024-12-17, Apache-2.0, dormant). A static ADR site with a timeline
  and statuses.
- **adr-tools** 3.0.0 (2018).
- **MADR** 4.0.0 (2024-09-17). A template only.
- None of them computes lifecycle beyond new and supersede links, which `scripts/adr.py`
  already does.
- **Verdict.** Nothing to add. (*Interface-checked*)

---

## 5. Spec, requirements and traceability tools (lifecycle only)

- **sphinx-needs 8.5.0.** Dynamic functions and `needextend` can *compute* a need's field
  from the needs it links to, which gives a real status roll-up. But it works only inside a
  Sphinx build of RST or MyST, which means changing the publishing owner. No.
- **StrictDoc 0.30.2.** Relations, statuses and a web editor, with no derived lifecycle.
  No.
- **Doorstop 3.2.** **Suspect links**: each link stores a fingerprint of its target, and
  editing the target makes the link "suspect" until `doorstop review` or `clear` re-stamps
  it. That is the cleanest prior art for "is this answer still valid". Adopt the pattern,
  not the tool.

---

## 6. Evidence and experiment reuse (brief)

- **DataLad 1.7.1** (2026-10-06).
  - `datalad run` records the command, inputs and outputs as a machine-readable record in
    the commit.
  - `datalad rerun` re-executes and shows the diff.
  - It is git-annex based and heavy, so not for this repository. The **run record** is the
    pattern: an investigation's probe command plus its declared inputs, stored with the
    answer.
- **Calkit 0.48.2** (2026-10-10, about 60 stars). A DVC-based project framework with stale
  stages. Small and young.
- **Quarto 1.10.19 `freeze: auto`.** Re-executes a computational document only when its
  source changes. **marimo 0.25.1** has Python-file notebooks with persistent caching. Both
  suit executable evidence pages, but adopting them changes the authoring medium.
- **MLflow 3.17.0.** A run registry searchable by tags and parameters; heavy for this use.
- **Sacred 0.8.7** (2024-11). Dormant.
- **qmd** (§2) is the only tool aimed squarely at "has this question been answered", and it
  needs no tagging.

---

## 7. Policy text restated across instruction files (discovered)

- **ruler 0.3.44** (2026-06-30) and **rulesync 29.0.0** (2026-10-08) generate per-agent
  instruction files (AGENTS.md, `.claude/rules`, Codex, Cursor) from one source.
- This repository already has one AGENTS.md, imported by CLAUDE.md, plus shared roles.
- Its remaining duplication is the same policy sentence across rules, skills and guides.
  Neither tool deduplicates *prose* inside a target.
- **Verdict.** No. The mechanical answer is one owner plus links, or a generated include
  block checked by the existing codegen drift check (pattern 6).

---

## Comparison table

Tested here: Beads, Backlog.md, OpenSpec, IWE and mdq/mq. Everything else is
*Interface-checked*.

| Tool | Version (date) | Storage | Agent query surface | Automates | Agent compliance asked | Fits rich prose? | Coexists with current docs? | Maturity | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| Beads `bd` | 1.3.1 (2026-09-30) | Dolt database, gitignored; JSONL export only | CLI `ready`, `blocked`, `prime`, `epic status`; MCP; hooks | Dependency closure, ready and blocked, epic % roll-up, atomic claim, compaction | High: an 842-word `prime`, close protocol, "no markdown task files"; auto-commits on init; default-on metrics | No: short issue records | Only as a second status store | High activity, frequent schema migrations | No; copy `ready` and roll-up |
| Backlog.md | 1.53.0 (2026-09-24) | One `.md` per task, front matter plus marker sections | CLI `--ready`, `--plain`, `--json`, search, doctor; MCP | Ready filter, dependency-tree view, duplicate-ID doctor, archive and cleanup | High: CLI-only edits, about 2.5k words of guides, plan approval | Partly: prose inside marker sections | A separate `backlog/` tree that duplicates plan status | Active, about 7k stars | No; sequential IDs collide across branches |
| ticket `tk` | 0.3.2 (2026-02-03) | `.tickets/*.md` with front matter | `ready`, `blocked`, `dep tree`, `dep cycle`, `query` (jq) | Dependency closure, cycle check | Low: one AGENTS line | Yes (Markdown bodies) | A separate tree | One bash script, about 900 stars | No; reference for scale |
| git-bug | 0.11.0 (2026-09-22) | Git-internal entities | CLI, TUI, web, GraphQL | Offline sync, tracker bridges | Medium | No | Invisible to file reads | Mature, about 10.7k stars | No |
| Task Master | 0.43.1 (2026-03-31) | `.taskmaster/` JSON | CLI `next`; 36 MCP tools | LLM PRD-to-task expansion, `next` | High; needs an LLM provider | No (JSON tasks) | Duplicates plans | Stalled since March | No |
| Spec Kit | 1.1.3 (2026-10-09) | `specs/NNN-*/*.md`, `.specify/` | Slash commands (prompts) | Scaffolding only; analysis and converge are LLM prompts | High: phase sequence | Its own templates | A separate tree | Very popular | No |
| OpenSpec | 1.14.1 (2026-10-06) | `openspec/specs`, `changes`, `archive` Markdown | CLI `list --json`, `status`, `validate`, `archive`; slash commands | Delta merge into specs on archive, structural validation, archived-task check | Medium: SHALL and scenario grammar, delta headers | Partly | A separate tree | Popular, active; opt-out telemetry | No; copy archive-as-command |
| Kiro, Conductor, Agent OS, BMAD | various 2026 | Spec and plan Markdown, YAML status | IDE or slash prompts | Mostly prompt-driven; Kiro IDE task runner | High | Own templates | Separate | Varied | No |
| Basic Memory | 0.23.2 (2026-08-25) | Markdown + SQLite FTS and vectors | MCP search and `build_context` | Link graph from plain links, hybrid search | Low to read, but **sync rewrites front matter** | Yes | **No**: it rewrites files | Active, about 4k stars | No for repository docs |
| Obsidian CLI, Bases, Dataview | 1.12.x / 0.5.70 | Vault Markdown | CLI against a running app | Queries over front matter and inline fields | Tagging needed; GUI app | Yes | Read-only possible | Mature, not headless | No |
| Marksman / markdown-oxide | 2026-02-08 / 0.25.12 | None (LSP) | LSP references, rename, diagnostics | Backlinks, heading rename | None | Yes | Yes | Mature | Optional LSP for "what cites this" |
| IWE | 0.26.1 (2026-10-08) | None (CLI, LSP, MCP over files) | `find --referenced-by`, `rename`, `delete`; MCP | Backlinks; link rewrite on rename and delete | None to query | Yes | Query yes; **writes unsafe**: drops `#fragments`, reformats files | Active, about 1.8k stars | Query only; it proves repair is mechanical |
| qmd | 2.8.3 (2026-08-16) | SQLite index outside the repository | CLI `search`, `query`, `get`; MCP | BM25 + vector + rerank over existing Markdown | None (no tagging) | Yes | Yes (read-only) | About 30k stars | Candidate for evidence reuse (BM25 first) |
| mdq / mq | 0.10.0 / 0.9.2 | None | CLI selectors, JSON | Section and table-row extraction from unchanged docs | None | Yes | Yes | Young but active | Borrow the selector idea; the existing reader suffices |
| Swimm | SaaS | `.swm` docs | IDE, PR app | Snippet auto-sync, outdated flags | Its own doc format | No | No | Commercial | No |
| menard | 0.4.0 (2026-04-03) | `.menard/links.toml` | CLI, pre-commit, JSON | Section-level staleness from git diffs | Maintain the link map; blocking hook | Yes | Yes | Very early | Pattern only |
| cogapp / markdown-magic | 3.6.0 / 4.11.0 | Inline generator blocks | `--check`, `-r` | **Regenerates** drifted blocks | Mark the blocks once | Yes | Yes | Mature | Pattern; fits `just codegen` |
| Doorstop suspect links | 3.2 (2026-07-10) | YAML items | CLI `review`, `clear` | Fingerprint-based suspect flags | Item format | No | No | Mature, niche | Pattern only |
| DataLad run records | 1.7.1 (2026-10-06) | git + git-annex | `run`, `rerun` | Provenance record, re-execution | Run probes through `datalad run` | n/a | Heavy | Mature | Pattern only |
| sphinx-needs / StrictDoc | 8.5.0 / 0.30.2 | RST or MyST / SDoc | Build-time filters / web UI | Computed fields (sphinx-needs) | Format migration | Partly | No | Mature | No |
| log4brains / adr-tools / MADR | 1.1.0 / 3.0.0 / 4.0.0 | ADR Markdown | CLI, static site | New, supersede, timeline | Low | Yes | Redundant | Dormant | No (`adr.py` covers it) |
| ruler / rulesync | 0.3.44 / 29.0.0 | One rules source | CLI generate | Per-agent instruction fan-out | Edit only the source | Yes | Redundant with AGENTS.md import | Active | No |

---

## 8. Patterns worth adopting even if the tool isn't

The common finding: the tools that genuinely remove agent effort do so by **computing views
from what is already written**, or by **performing a whole lifecycle transition as one
command**. The tools that add effort ask agents to keep a second store current through a CLI
or a prompt protocol.

1. **Query the existing Markdown; do not migrate it.**
   - mdq showed (*Tested*) that "open packets" and "current checkpoint" are selectable from
     Plan 33 unchanged.
   - The repository's `document_lifecycle.py scope` is already this mechanism.
   - Extend it with a section selector (`--section "Current checkpoint"`) and a terse
     `--plain` mode. Agents then get "where do I update" with one command and no new format.
2. **Classify status from the leading word, and derive everything else.**
   - Of the 29 items, 9 were `unknown` (*Measured*): 6 EI packets have no status column, and
     3 prose cells did not match.
   - A first-token vocabulary (Planned / In progress / Integrated / Accepted / Retained /
     Done) keeps the prose after it. Read it with that first word; do not enforce it.
   - The tool reports, rather than blocks, cells it cannot classify. The fix is a one-word
     edit the report can propose.
3. **Make roll-ups computed columns, not restated text.**
   - Every tracker that rolls up (beads `epic status`, sphinx-needs dynamic functions)
     computes from links.
   - The mdq demo joined each finding to its owner packets' progress in about 15 lines.
   - A derived view (scope JSON, or a generated index block, pattern 6) can show "F03:
     Scheduled; owner EFF09 Integrated", and flag when every owner is closed.
   - Nobody then marks a plan done while its findings say "scheduled" without seeing it.
4. **Compute `ready` and `blocked` from declared inputs.**
   - beads, Backlog.md and tk all reduce to a dependency closure over IDs; tk does it in a
     bash script.
   - Plan packet tables already name prerequisites ("Required input: … EFF00 …").
   - Parsing packet IDs from that column yields `ready` and `blocked` and cycle detection,
     with no extra field for agents to maintain.
5. **Retirement as one programmatic command with byte-preserving link repair.**
   - OpenSpec's `archive` (move, merge, validate) and IWE's rename and delete (*Tested*)
     show the transition is mechanical.
   - IWE also shows what to avoid: it drops fragments and reformats files.
   - `retire-plan` already lists `references_to_repair`. An `--apply` mode could rewrite
     each inbound link, fragment included, in place:
     - to the surviving owner when it is known;
     - otherwise to a Git permalink of the retired file at the retirement commit.
   - It would then refuse when an unresolved blocker remains, like OpenSpec's
     archived-task check, but run *before* the move.
6. **Generated blocks, with drift checks and a programmatic fix, for every index.**
   - cog and markdown-magic (`--check` / `-r`) match the maintainer's "lint only if the fix
     is programmatic" test.
   - Plan status in `docs/plans/README.md`, finding roll-ups and the few unavoidable policy
     restatements can be generated by the existing `just codegen` path and checked by
     `codegen-check`.
   - Agents never hand-edit an index.
7. **Fingerprinted validity for evidence answers.**
   - Doorstop suspect links, DataLad run records and menard's section links share one shape:
     record what the answer depended on, and flag "suspect" when that changes.
   - For an evidence README, the inputs are a probe command, the lockfile, toolchain and pin
     entries, and the source paths. A command can then list suspect answers after a
     dependency or toolchain move.
   - Refreshing means re-running the recorded command, and deleting is one decision.
   - The repository's scope and reuse-refusal machinery (EFF01) already embodies this
     contract and is a natural owner.
8. **Search-first reuse without tagging.**
   - "What do we already know" fails because nobody searches before probing, not for lack of
     tags.
   - Front matter is already present. A question-shaped `title` on evidence READMEs plus one
     search command over evidence and reviews would cover the repeated question.
   - The command can be BM25 (ripgrep or Pagefind today, qmd if paraphrase recall matters).
     It should return the answer, its date and its suspect state (pattern 7).
9. **Concurrency facts for whichever shape is chosen** (*Measured*, `git merge-file`, Git's
   default merge).
   - Two agents editing **adjacent** table rows (EFF09 and EFF10) conflict, exit 1. Rows one
     apart (EFF09 and EFF07) merge cleanly, exit 0.
   - Sequential IDs collide on parallel branches (Backlog.md `EFF-9` twice); hash IDs
     (beads, tk) avoid that.
   - In the usual single shared checkout these are lost-update risks rather than merge
     conflicts. Exact-match Edit on distinct rows is safe, and worktrees are where row
     adjacency bites.

**Not recommended.**

- Any tool that moves status out of Git-tracked text (beads' Dolt store, git-bug).
- Any tool that requires edits through its CLI only (Backlog.md, beads).
- Any tool whose lifecycle steps are prompts the agent must remember (Spec Kit, Conductor,
  BMAD, Kiro outside its IDE).

Each fails the maintainer's first test: it adds agent effort without clearly removing any.

## Material uncertainties

- Claude Code Tasks details come from third-party guides; I did not verify them in
  Anthropic's docs.
- I did not test Marksman's handling of HTML `<a id>` anchors.
- I did not trial qmd's semantic recall on this corpus. The model download is about 2 GB.
- The mdq output error appeared on one large section; I did not investigate its cause.
- The beads metrics flush outcome is unknown (see the opening deviation note).
- "No telemetry" for Backlog.md, mdq, mq and IWE rests on README claims and a GitHub code
  search, not a network capture.

## Retained artifacts

Trials ran in a session scratch directory with `HOME` redirected. Retained under
`probes/tool-trials/`: the mdq extraction outputs `packets.json` and `findings.json` from the
Plan 33 copy, and `beads-prime.txt`, the beads session primer quoted in the compliance comparison.
Not retained: release binaries and archives, npm installs, the redirected home, the trial Git
repositories, the beads export and the merge-scenario plan copies.
