---
id: ADR-0139
title: Select task-dependent delegation and stronger review defaults
status: accepted
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: docs/design_review/reviews/design_review_agent-coordination_2026-09-30.md
evidence: Implemented
supersedes: [ADR-0138]
superseded-by: null
revisit: A concrete coordination failure or a model/runtime change prevents a role from honoring its contract.
verification: Scoped static governance review of root-owned small work, independent evidence gathering, coupled implementation, consequential negative findings, repeated repair failure and focused versus formal review; native settings and cumulative instruction-byte inspection on 2026-09-30. Model-quality trials and product qualification not_run.
standard: null
scenarios: []
---

# ADR-0139: Select task-dependent delegation and stronger review defaults

## Context

The maintainer approved revising agent coordination after reviewing an external perspective
and its research. Broad parallelism, repeated startup tours and implicit model escalation
increase coordination cost and can obscure evidence limits. Blueprint §24.4 owns the workflow.

## Scope

Amend agent workflow governance: shared roles, independent native adapters, canonical local
process skills and task-routing instructions. Scientific and product contracts, hooks and
production-plan qualification are unchanged. This supersedes ADR-0138 while retaining its
shared ownership, skill aliases and separate native adapters.

## Drivers

Keep design, integration and acceptance with the coordinator; preserve local executor discretion.
Let independent evidence work proceed without repeated root orientation, while surfacing conflicts
and consequential uncertainty. A small task must remain locally executable; independent review
must have enough concrete criteria and consumer evidence to form its own judgment.

## Options

1. Retain broad parallelism and existing defaults: simple wording, but weak task-fit and escalation guidance.
2. Require fixed role sequences, costly reruns or new experiments: additional coordination and evidence obligations without an established consumer.
3. Task-dependent delegation and explicit escalation (chosen): selective context isolation and independent judgment, with coordinator-owned reassessment and stronger review defaults.

## Outcome

Delegate when independent coverage, context isolation, distinct capabilities or independent
judgment justify handoff and integration; no mandatory role chain or agent count applies.
Briefs supply settled decisions, exact baseline including relevant dirty changes, permitted effects,
sibling ownership, dependencies and expected evidence. Workers reuse instructions already present,
load relevant owners, and follow discovered dependencies. Negative claims report search coverage
and limitations; triggers prompt reassessment rather than automatic model reruns.

Codex implementation-reviewer uses gpt-6.1-sol/high. Claude code-mapper and library-research use
Sonnet/medium; design-reviewer uses Opus/high. Other defaults remain. Stronger Codex evidence work
uses a fresh built-in default role with explicit Sol/high and the relevant read-only contract;
Claude model overrides retain role effort, and appropriate high-effort design judgment uses the
existing reviewer. Model quality remains unmeasured; no calibration exercise is required.

Shared `.agents/roles/` owns behavior and coordination. Independent `.codex/agents/` and
`.claude/agents/` own native model, effort and tool settings. Claude implementer adapts executor;
Codex uses executor. Astra/high remains the coordinator default, Sol/high the generic fallback;
explicit user runtime choices take precedence. Local process skills remain canonical in
`.codex/skills/`, with `.claude/skills` and `.agents/skills` aliases. `agent-config-sync` only
materializes skill aliases, preserves live shared library links and never modifies native agents.

Conversational planning companions and action skills retain their paired workflows; skills do not
switch runtime modes. Plan creation includes focused foundation assessment. Existing core 3.3,
process-simulator 1.3, AP-04/G9, scientific judgments and the binding's review cadence remain.
Ordinary implementation adds no standing model review or exhaustive tracing. The current-work
index routes baseline and handoff to active plans or packets; no root STATUS or second ledger is added.

### Consequences

Selective delegation depends on coordinator judgment. Evidence workers receive stronger defaults,
but this decision establishes no measured cost or quality gain. Detailed repository and command
reference material moves to existing navigation and qualification guides so loaded instructions
retain operational constraints within the cumulative Codex document budget.

### Compensating controls

Fresh reviewers receive criteria, affected consumers, relevant failure cases and identified tree
baselines. Conflicting evidence, consequential absence claims, unsupported version transfer,
unresolved ownership or contract decisions and a second failed repair are surfaced to the coordinator.
Existing edit protections, generated-code rules, memory caps, pinned commands and test timing apply.
Native definitions remain maintained independently; configuration edits do not change running agents.

### Confirmation

The named static governance review and configuration/instruction inspection establish the scoped
policy argument and operational coherence. They do not establish native behavioral or model-quality
improvements. Functional scope and qualification remain owned by existing production plans.

## Pros and cons

The workflow reduces duplicated orientation and allows proportionate independent judgment while
preserving root acceptance. Task-fit and escalation still require judgment; stronger defaults can
consume more resources. No new hook, framework, hard cap or recurring telemetry is introduced.

## More information

[Blueprint §24.4](../authoritative_design/sections/design-change-workflow.md#section-24-4)
owns the workflow; [.agents/roles/README.md](../../.agents/roles/README.md) owns routing;
[local process skills](../../.codex/skills/README.md) own workflows. ADR-0129 retains bounded
domain-model review. [Current work](../plans/README.md) links existing packet status and handoff.

## Status history

- 2026-09-30 — accepted after the independent scoped governance review returned Accept. Supersedes ADR-0138; its surviving obligations are retained above and in §24.4. Configuration/discovery checks passed; product qualification and model-quality trials not_run.
