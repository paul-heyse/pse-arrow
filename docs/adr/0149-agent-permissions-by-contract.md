---
id: ADR-0149
title: Bound agent effects by role contracts alone
status: accepted
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: not-required: the maintainer waived an independent review; the change only removes native tool restrictions from ADR-0147, whose review accepted instruction-bound scopes
evidence: Implemented
supersedes: [ADR-0147]
superseded-by: null
revisit: A runtime gains per-agent write scoping that could enforce the contracts, Claude Code makes its subagent report-file refusal configurable, a library-research write lands outside its permitted scopes, or a concrete coordination failure or model/runtime change prevents a role from honoring its contract.
verification: Coordinator inspection of contract-only agent permissions, the removal of Claude tool lists and the carried-forward ADR-0147 policy (independent review waived by the maintainer); `just lint-agents`; byte identity of the shared role contracts' changed passages with the copier template v0.3.2 on 2026-10-01; the Claude Code 2.1.287 Write tool source for the subagent report-file refusal. Earlier write smoke (ADR-0147): Coordinator smoke receipt 2026-10-01 in the sibling repository thermo-knowledge, which carries the byte-identical contract: a Claude library-research subagent (`claude -p --permission-mode acceptEdits`) and a Codex library-research role (`codex exec`, danger-full-access) each created a new evidence README and appended to an assigned throwaway shared skill without approval, and the files were removed afterwards; in `claude -p` default mode both writes were refused, as non-interactive default mode refuses any unapproved write. Model-quality trials and product qualification not_run.
standard: null
scenarios: []
---

# ADR-0149: Bound agent effects by role contracts alone

## Context

ADR-0147 let `library-research` write its evidence and an assigned shared library skill by
instruction, granting it Write/Edit through its Claude tool list; the other Claude adapters kept
tool allowlists. On 2026-10-01 a `library-research` subagent's evidence write of `report.md` was
refused by a check built into Claude Code 2.1.287: in any subagent, the Write tool rejects a `.md`
file whose name starts with report, summary, findings or analysis (case-insensitive). The check sits
in the tool's input validation, before permission rules and hooks; no setting disables it, and other
names, Edit and Bash are unaffected. The maintainer asked to remove every native restriction on the
agents and rely on the instructions, across the repositories and the copier template (v0.3.2).

Before ADR-0147, ADR-0139 made `library-research` a read-only evidence role: its shared contract said "Do not edit
repository files", Claude's adapter had no Write/Edit and Codex's declared `sandbox_mode =
"read-only"`. Research that should leave durable evidence (a probe with its README, or a fix to a
shared library skill under `~/.local/share/library-skills/`) was redone by the coordinator or an
executor. Blueprint §24.4 owns the workflow.

Two runtime facts, verified 2026-10-01 against Claude Code 2.1.287 documentation and the codex-cli
0.159.2 source: Claude path permission rules and its Bash sandbox are session-wide (only the tool
list and hooks are per agent), and Codex has dropped `sandbox_mode`/`writable_roots` from custom
agent files since 0.149, so every role inherits the session's sandbox. The declared read-only and
workspace-write modes in `.codex/agents/` enforced nothing.

## Scope

Amends agent workflow governance only: the six Claude adapters' tool lists, the common worker
contract (markdown naming), the roles README and AGENTS.md routing. Restates ADR-0147 unchanged
apart from the native permission settings, so ADR-0147 can be superseded and retired. Scientific and product contracts, hooks,
`scripts/agent-hooks.py` protections and production-plan qualification are unchanged. The same
texts are carried by the copier template (v0.3.2) and the sibling repositories that share these
contracts.

## Drivers

Keep design, integration and acceptance with the coordinator; preserve local executor discretion.
Let evidence work leave its evidence without a second worker, while keeping production code,
design, decisions and plans out of an evidence role's reach. Keep shared library skills
repository-independent. Do not claim an enforcement the runtimes do not provide.

## Options

1. **Keep library-research read-only** — simplest, but evidence work keeps passing through a second
   worker, and the Codex declarations would still claim an enforcement that does not exist.
2. **Enforce write scopes natively** — Codex cannot (role files carry no sandbox settings, and
   roles can only reduce shell, app, plugin, memory and skill capabilities); Claude only through a
   PreToolUse hook, in the adapter's frontmatter or a shared guard keyed on `agent_type`, which
   restricts Write/Edit but cannot stop writes through Bash (`permissionMode` is ignored under the
   bypass parent this repository uses). A hook per repository and its maintenance, for a partial
   guarantee.
3. **Per-role tool allowlists** (ADR-0147) — restrict only Claude, cannot bound shell writes, and
   need a configuration change in every repository whenever a role's work changes.
4. **Contracts only, identical in every repository and the template (chosen).** Native role files
   carry model and effort and nothing that restricts tools or sandbox; the contracts state each
   role's permitted effects.

## Outcome

Within the coordinator's brief, `library-research` writes only: new dated topic folders with their
README in the library-evidence locations AGENTS.md names (here `docs/design_review/evidence/`;
capability-map evidence stays with the coordinator), the source of a shared library skill the brief
assigns (repository-independent, one writer per skill, following that skill's maintenance guide),
and scratch outside the repository. It never edits production code, tests, design documents, ADRs,
plans, pins, configuration, generated paths or another worker's folder, commits nothing unless
assigned and lists every file written. A role contract may grant standing write scopes that a
brief can narrow, never widen. Native role files carry no permission restrictions: Claude adapters
have no `tools` lists, so every role inherits the session's tools; Codex role files carry no
`sandbox_mode`, so every role inherits the session's sandbox and approval settings. The shared
contracts alone bound each role's effects. The worker contract tells workers to name a markdown
file for its content (`README.md`, `<topic>.md`), because the built-in report-file refusal cannot
be switched off.
Stronger Codex evidence work supplies the evidence role's permitted effects instead of read-only
effects. The other evidence roles keep their read-only contracts.

Carried forward from ADR-0147 (and through it ADR-0139) unchanged: delegate when independent coverage, context isolation,
distinct capabilities or independent judgment justify handoff and integration; no mandatory role
chain or agent count applies. Briefs supply settled decisions, exact baseline including relevant
dirty changes, permitted effects, sibling ownership, dependencies and expected evidence. Workers
reuse instructions already present, load relevant owners and follow discovered dependencies.
Negative claims report search coverage and limitations; triggers prompt reassessment rather than
automatic model reruns.

Codex implementation-reviewer uses gpt-6.1-sol/high. Claude code-mapper and library-research use
Sonnet/medium; design-reviewer uses Opus/high. Other defaults remain. Stronger Codex evidence work
uses a fresh built-in default role with explicit Sol/high and the relevant evidence contract;
Claude model overrides retain role effort, and appropriate high-effort design judgment uses the
existing reviewer. Model quality remains unmeasured; no calibration exercise is required.

Shared `.agents/roles/` owns behavior, coordination and permitted effects. Independent
`.codex/agents/` and `.claude/agents/` own native model and effort settings. Claude implementer
adapts executor; Codex uses executor. Astra/high remains the coordinator default, Sol/high the
generic fallback; explicit user runtime choices take precedence. Local process skills remain
canonical in `.codex/skills/`, with `.claude/skills` and `.agents/skills` aliases.
`agent-config-sync` only materializes skill aliases, preserves live shared library links and never
modifies native agents.

Conversational planning companions and action skills retain their paired workflows; skills do not
switch runtime modes. Plan creation includes focused foundation assessment. Existing core 3.3,
process-simulator 1.3, AP-04/G9, scientific judgments and the binding's review cadence remain.
Ordinary implementation adds no standing model review or exhaustive tracing. The current-work
index routes baseline and handoff to active plans or packets; no root STATUS or second ledger is added.

### Consequences

Evidence and skill improvements need no second worker, and no adapter changes when a role's work
changes. Every permission is an instruction, not an enforced boundary, in both runtimes: the session's sandbox (danger-full-access for this
maintainer's Codex) and Claude's session-wide permissions apply to every role, and Bash can write
anywhere. Selective delegation still depends on coordinator judgment; this decision establishes no
measured cost or quality gain. Detailed repository and command reference material stays in the
existing navigation and qualification guides so loaded instructions retain operational constraints
within the cumulative Codex document budget (32 KiB; 31,008 bytes on 2026-10-01). Evidence roles
now hold Write and Edit as well as Bash; their read-only contracts, the return's file list and the
edit-protection hook are what keep them read-only.

### Compensating controls

The return lists every file written; the coordinator reviews it and `git status` before
integration, and for a shared skill the store's own `git status`/`git diff` (weak while the store
has no baseline commit; one writer per skill is otherwise an instruction the coordinators keep).
`scripts/agent-hooks.py` still blocks every role from `docs/authoritative_design/`, accepted ADRs
and generated paths, and `.claude/settings.json` deny rules still hold. Existing edit protections,
generated-code rules, memory caps, pinned commands and test timing apply. Fresh
reviewers receive criteria, affected consumers, relevant failure cases and identified tree
baselines. Conflicting evidence, consequential absence claims, unsupported version transfer,
unresolved ownership or contract decisions and a second failed repair are surfaced to the
coordinator. Native definitions remain maintained independently; configuration edits do not change
running agents.

### Confirmation

The coordinator's static inspection and `just lint-agents` establish the scoped policy argument and
configuration coherence. They do not establish native behavioral or model-quality improvements.
Functional scope and qualification remain owned by existing production plans.

## Pros and cons

Evidence leaves its evidence in one handoff, and the configuration no longer restricts tools in
only one runtime while claiming nothing about the shell. The write boundary depends on the worker honoring its contract and the coordinator
reviewing its return; no new hook, framework or recurring telemetry is introduced.

## More information

[Blueprint §24.4](../authoritative_design/sections/design-change-workflow.md#section-24-4)
owns the workflow; [.agents/roles/README.md](../../.agents/roles/README.md) owns routing and
[the library-research contract](../../.agents/roles/library-research.md) its permitted writes;
[local process skills](../../.codex/skills/README.md) own workflows. ADR-0129 retains bounded
domain-model review. [Current work](../plans/README.md) links existing packet status and handoff.
The carried-forward policy was accepted on the
[library-research write review](../design_review/reviews/design_review_library-research-writes_2026-10-01.md)
and the [2026-09-30 governance review](../design_review/reviews/design_review_agent-coordination_2026-09-30.md).

## Status history

- 2026-10-01 — proposed.
- 2026-10-01 — accepted by the maintainer, who waived an independent review. Supersedes ADR-0147, whose decisions are restated above. `just lint-agents` passed; product qualification and model-quality trials not_run.
