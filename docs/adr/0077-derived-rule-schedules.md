---
id: ADR-0077
title: Derive executable rule schedules from checked native plans
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-02, DM-13, DM-28, DM-31, DM-43, DM-59, DM-60]
blueprint: [§6.11, §14.2, §15.3]
review: docs/design_review/reviews/design_review_derived-rule-schedules_2026-09-23.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A supported rule effect cannot be extracted or a complete schedule cannot be derived without a manual order.
verification: Plan 13 computation units; just unit-rust-computation; just family-check; just codegen-rust-contracts-check; just adr-lint.
---

# ADR-0077: Derive executable rule schedules from checked native plans

## Context

Plan 13 W08 removes manually authoritative strata. Native plan preparation currently
checks reads against a caller-supplied stratum, so that order cannot establish its own
completeness. The maintainer explicitly selected derived compiler output over retaining
the reference-row shape and authorized W07–W11 implementation.

## Scope

Amend blueprint §6.11 and §14.2: reference rules declare meaning, not executable strata.
Checked native plans determine effects and a pure complete dependency graph determines
compiled.rule_strata. This is a direct contract replacement, without compatibility fields.
Amend §15.3 to use the iterative petgraph SCC implementation already selected by W06.
No mathematical block/DM interpretation, canonical MathIR hash or commit contract changes.

## Drivers

One scheduling authority, complete negative/nonmonotone/conflict/support dependencies,
runtime-free registry generation, bounded recursion and deterministic interpretation.

## Options

Manual strata duplicate dependencies. Declaration-only conservative scheduling requires
maintaining effects beside SQL. Selected: bind native plans, check declared ports, derive
all effects, then derive and validate the executable schedule before data execution.

## Outcome

Remove stratum from reference declarations. A compiled rule program owns native plans,
pure dependency/schedule values and complete execution limits. Positive SCCs may iterate;
settlement-sensitive edges inside an SCC fail with a witness. The existing relational
executor settles each derived stratum and exposes only complete program results.

### Consequences

Generated reference schemas and every consumer change together. compiled.rule_strata
records program identity, rule identity and deterministic component/stratum ordinals.
Neither schema nor codegen depends on the runtime or generated DTO consumers. Structural
BTD uses iterative kosaraju_scc plus deterministic condensation ordering; it still requires
complete eligible matching and uses no SCC of the raw bipartite graph.

### Compensating controls

Checked declared read sets, subquery traversal, unknown-effect refusal, all-head-writer
closure, conflict/support settlement, maximum rounds, exact input selections and W05 atomic
completion. Exhaustive tiny matching and independent certificate controls precede closure.

### Confirmation

Isolated units and generated contract equality establish only their named claims. W18
remains closed; W19/W20 and all inherited acceptance obligations remain open.

## Pros and cons

The selected boundary removes manual schedules and generated bootstrap cycles. It changes
reference row shapes and requires complete checked-plan effect extraction.

## More information

[Execution packet](../plans/13-w07-w11-execution.md), ADR-0076 and its linked target review.
The deliberate blueprint revision belongs to the named decision/design PR for ADR-0077.

## Status history

- 2026-09-23 — proposed; local implementation authorized. Formal decision/design PR and acceptance remain pending.
