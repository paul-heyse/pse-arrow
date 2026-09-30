---
id: ADR-0138
title: Shared workflow roles with independent native adapters
status: accepted
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-06, DP-01, DP-22]
blueprint: [§24.4]
review: "not-required: maintainer-approved agent tooling rollout; scientific and product contracts are unchanged"
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A runtime cannot honor the shared role contract, a workflow creates duplicate status ownership, or alias synchronization modifies a native agent definition.
verification: Targeted disposable-fixture alias synchronization tests in scripts/tests/test_setup.py; source inspection of shared contracts and native adapters, 2026-09-30. Native agent behavioral trials and product qualification not_run.
standard: null
scenarios: []
---

# ADR-0138: Shared workflow roles with independent native adapters

## Context

The maintainer approved adopting the shared review, plan creation and execution workflows.
The previous agent-config generator copied Claude role bodies into Codex definitions, preventing
independent native model and tool defaults. That derivation is retired with this rollout.
Historical accepted decisions retain their original evidence and descriptions.

## Scope

Agent instructions, six shared roles and their common worker contract, native adapters, local
process skills, and the existing alias synchronizer and checker. Product code, hooks, MCP wiring,
scientific review criteria and active production plan status are outside this change.

## Drivers

Behavior has one shared owner, runtime defaults remain native, and workflow status stays in the
existing plan and packet owners. Executors retain bounded local implementation discretion.

## Options

1. Continue generating Codex agents from Claude definitions: one runtime would own both defaults.
2. Duplicate role behavior in both adapters: independently editable contracts would drift.
3. Shared contracts with independent adapters (chosen): explicit runtime routing without behavior
   duplication; the existing script only materializes skill aliases.

## Outcome

`.agents/roles/` owns worker behavior and coordination. `.codex/agents/` and `.claude/agents/`
are maintained native adapters for code mapping, library research, design review, execution,
implementation review and functional testing. Claude `implementer` adapts the executor role;
Codex uses `executor`. The coordinator defaults to Astra/high, with Sol/high as the generic
worker fallback and role-specific settings as declared in the adapters. Explicit user runtime
choices take precedence. Use concurrency as you see fit; strive for parallel execution.

Process skills remain canonical in `.codex/skills/`; `.claude/skills` and `.agents/skills`
remain aliases. `agent-config-sync` only materializes those aliases, including Windows copies,
and preserves shared live library links. It never creates, updates or deletes native agents.
The obsolete generated Codex `implementer` adapter is removed.

Review planning, plan creation and execution each have a conversational planning companion
and an action skill. Creating a plan includes a focused assessment of its foundations, without
certifying an enclosing subsystem. Existing core 3.3, process-simulator 1.3, AP-04/G9 and independent
scientific judgments remain authoritative. Routine implementation adds no standing model review.

### Consequences

Runtime adapters can differ in native defaults while loading the same contracts. Planning skills
do not switch runtime modes. No new hook, ledger, framework or root STATUS file is introduced.

### Compensating controls

The existing checker covers shared role and local process references and native adapters. Existing
protected-edit, migration, generated-code, command and test-timing rules continue to apply.
`docs/plans/README.md` routes to the active packet checkpoint for baseline and handoff.

### Confirmation

The scoped alias tests establish synchronization behavior on disposable trees. Native agent
behavior and model allocation quality remain unmeasured; document/configuration existence is
Implemented evidence only.

## Pros and cons

Shared behavior reduces duplication and permits native allocation. Two adapter formats remain
intentional runtime-specific configuration and require maintenance when runtime schemas change.

## More information

Blueprint §24.4 owns the workflow; `.agents/roles/README.md` owns routing and coordination;
`.codex/skills/README.md` routes process skills. ADR-0129 retains bounded domain-model review policy.

## Status history

- 2026-09-30 — accepted by the maintainer as the workflow rollout, including retirement of native
  agent-definition generation.
