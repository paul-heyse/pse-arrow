---
id: ADR-0025
title: Defer LogicalPlan::Extension rule nodes; derivations already answer the attribution question
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-58, DM-46, DM-49]
blueprint: [§14.2]
review: not-required: recorded in the second review's observations as an adopt-candidate, not as a finding
evidence: Proposed
supersedes: []
superseded-by: null
revisit: The §22.4 diff report or agent tooling needs per-rule attribution inside a plan rendering
verification: Register row R-02; the §22.4 diff-report acceptance test when it is written

---

# ADR-0025: Defer LogicalPlan::Extension rule nodes; derivations already answer the attribution question

## Context

Blueprint §14.2 rule 8 defers wrapping each rule body in a `LogicalPlan::Extension` node carrying `rule_id`. The DataFusion capability map measured that such a node survives the optimizer intact (PROBE B), so the deferral is a cost decision, not a feasibility one.

## Scope

Binds whether rule attribution lives in the plan. It does not affect `provenance.derivations` or `rules_fired`, which answer "which rule produced this row" today.

## Drivers

The mechanism costs the 14-method `UserDefinedLogicalNode` contract and an `ExtensionPlanner`; the question it answers is already answered at the data level (charter DM-58).

## Options

Adopt now — rejected: machinery ahead of a demonstrated consumer. Never adopt — rejected: `EXPLAIN`-level attribution is genuinely the better answer once a diff report renders plans.

## Outcome

Not adopted. Derivations and the optimizer observer's `rules_fired` list are the attribution mechanism for phases 0-3.

### Consequences

If the trigger fires, the node must be added for every rule body at once, because partial attribution in an `EXPLAIN` is worse than none.

### Compensating controls

Register row R-02 carries the trigger and a phase-gated review date.

### Confirmation

The observer closures of `Analyzer::execute_and_check` and `Optimizer::optimize` already record which rules fired on each plan, which is recorded in the pass record (ADR-0019).

## Pros and cons

Deferring costs a later retrofit across all rule bodies; adopting now costs a 14-method contract with no consumer.

## More information

Blueprint §14.2 rule 8; the second review's observations; `docs/adr/register.md` row R-02.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
