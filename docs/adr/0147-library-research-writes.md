---
id: ADR-0147
title: Let library-research write its evidence and assigned shared skills
status: accepted
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: docs/design_review/reviews/design_review_library-research-writes_2026-10-01.md
evidence: Implemented
supersedes: [ADR-0139]
superseded-by: null
revisit: A Codex release applies sandbox settings from custom agent files again, a runtime gains per-agent write scoping, a library-research write lands outside its permitted scopes, or a concrete coordination failure or model/runtime change prevents a role from honoring its contract.
verification: Scoped static governance review of the library-research write scope, Codex sandbox inheritance and the carried-forward ADR-0139 coordination policy; `just lint-agents`; byte identity of `.agents/roles/library-research.md` with the copier template on 2026-10-01. Coordinator smoke receipt 2026-10-01 in the sibling repository thermo-knowledge, which carries the byte-identical contract: a Claude library-research subagent (`claude -p --permission-mode acceptEdits`) and a Codex library-research role (`codex exec`, danger-full-access) each created a new evidence README and appended to an assigned throwaway shared skill without approval, and the files were removed afterwards; in `claude -p` default mode both writes were refused, as non-interactive default mode refuses any unapproved write. Model-quality trials and product qualification not_run.
standard: null
scenarios: []
---

# ADR-0147: Let library-research write its evidence and assigned shared skills

## Context

ADR-0139 made `library-research` a read-only evidence role: its shared contract said "Do not edit
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

Amends agent workflow governance only: the library-research and common worker contracts, the
roles README, the six Codex adapters' sandbox keys, Claude's library-research tool list and AGENTS.md
routing. Restates ADR-0139's coordination decisions unchanged apart from the evidence-role write
clause, so ADR-0139 can be superseded and retired. Scientific and product contracts, hooks,
`scripts/agent-hooks.py` protections and production-plan qualification are unchanged. The same
texts are carried by the copier template (v0.3.1) and the sibling repositories that share these
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
3. **One instruction, identical in every repository and the template, with the native tool grant
   (chosen).** The contract states the permitted writes; Claude grants Write/Edit; Codex role files
   drop the no-op sandbox key and defer to the contract.

## Outcome

Within the coordinator's brief, `library-research` writes only: new dated topic folders with their
README in the library-evidence locations AGENTS.md names (here `docs/design_review/evidence/`;
capability-map evidence stays with the coordinator), the source of a shared library skill the brief
assigns (repository-independent, one writer per skill, following that skill's maintenance guide),
and scratch outside the repository. It never edits production code, tests, design documents, ADRs,
plans, pins, configuration, generated paths or another worker's folder, commits nothing unless
assigned and lists every file written. A role contract may grant standing write scopes that a
brief can narrow, never widen. Claude's library-research adapter lists Write and Edit; Codex role
files carry no `sandbox_mode`, and Codex roles inherit the session's sandbox and approval settings.
Stronger Codex evidence work supplies the evidence role's permitted effects instead of read-only
effects. The other evidence roles keep their read-only contracts.

Carried forward from ADR-0139 unchanged: delegate when independent coverage, context isolation,
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
`.codex/agents/` and `.claude/agents/` own native model, effort and tool settings. Claude implementer
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

Evidence and skill improvements no longer need a second worker. The write scope is an instruction,
not an enforced boundary, in both runtimes: the session's sandbox (danger-full-access for this
maintainer's Codex) and Claude's session-wide permissions apply to every role, and Bash can write
anywhere. Selective delegation still depends on coordinator judgment; this decision establishes no
measured cost or quality gain. Detailed repository and command reference material stays in the
existing navigation and qualification guides so loaded instructions retain operational constraints
within the cumulative Codex document budget (32 KiB; 30,967 bytes on 2026-10-01).

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

The named static governance review and `just lint-agents` establish the scoped policy argument and
configuration coherence. They do not establish native behavioral or model-quality improvements.
Functional scope and qualification remain owned by existing production plans.

## Pros and cons

Evidence leaves its evidence in one handoff, and the configuration no longer claims a sandbox it
does not apply. The write boundary depends on the worker honoring its contract and the coordinator
reviewing its return; no new hook, framework or recurring telemetry is introduced.

## More information

[Blueprint §24.4](../authoritative_design/sections/design-change-workflow.md#section-24-4)
owns the workflow; [.agents/roles/README.md](../../.agents/roles/README.md) owns routing and
[the library-research contract](../../.agents/roles/library-research.md) its permitted writes;
[local process skills](../../.codex/skills/README.md) own workflows. ADR-0129 retains bounded
domain-model review. [Current work](../plans/README.md) links existing packet status and handoff.
The carried-forward coordination policy was accepted on the
[2026-09-30 governance review](../design_review/reviews/design_review_agent-coordination_2026-09-30.md).

## Status history

- 2026-10-01 — proposed.
- 2026-10-01 — accepted after the independent scoped governance review returned Accept. Supersedes ADR-0139, whose surviving obligations are restated above and in §24.4. Review F01 closed in this change (supersession, blueprint revision row 91, current-work link); F02, F03 and F06 corrected in this record and F05 in AGENTS.md; F04 partly addressed here (store diff named), with a pre-write store check and a baseline store commit left to the maintainer. `just lint-agents` passed; product qualification and model-quality trials not_run.
