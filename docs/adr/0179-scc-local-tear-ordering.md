---
id: ADR-0179
title: Localize tear ordering while preserving global decisions
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, DP-09, PS-09]
blueprint: [§17.4]
review: docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f21
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Populated tear workloads show local ordering still dominates preparation, and a lazy-cycle alternative has equivalent termination and incumbent evidence.
verification: Exhaustive bounded MILP-versus-independent-DAG feasibility; shared multi-SCC groups, acyclic mandatory/forbidden decisions, self loops, disconnected components and native shared-cost optimum; original incumbent/limit handling retained.
---

# ADR-0179: Localize tear ordering while preserving global decisions

## Context

Plan 36 P-C6/F21 selects SCC-local tear ordering. The existing global ordering formulation introduces constraints for edges that cannot participate in a cycle and uses the whole graph size as big-M.

## Scope

Amend the tear formulation in blueprint §17.4 within the existing physically admitted flow and HiGHS kernel contracts. Flow identity, physical bindings, grouped policies, native quality and the independent whole-graph witness retain their meaning.

## Drivers

Recycles require a policy-valid tear set. Independent components should not enlarge each other's ordering constraints; authored decisions spanning components still represent one shared choice and cost.

## Options

Select one HiGHS model with SCC-local order columns and globally shared group columns. Independent component solves require a separate coupling/merge protocol for shared groups. Lazy cycle generation introduces a callback and termination obligation without current measurements justifying it.

## Outcome

Only intra-SCC occurrences receive order rows, with order range and big-M bounded by that SCC's size. Singleton self loops retain their constraint; acyclic singleton nodes need no order column. Every authored decision column remains, including mandatory or forbidden groups with only acyclic occurrences, and each group cost occurs once. Native incumbent, bound, gap, limit and optimality retain separate meanings; any selected set must pass the existing whole-graph policy and DAG witness.

### Consequences

Remove global inter-SCC rows and whole-graph big-M. Admission retains a conservative structural ceiling including SCC construction scratch, computed without allocating a decomposition before admission. Reduced model size does not establish reduced allocation or latency measurements.

### Compensating controls

Exhaustive bounded projection controls compare all selected-group subsets against the independent graph witness, including shared groups across two cycles. Separate controls preserve acyclic policies and self-loop handling. The native solver still owns incumbent quality and status.

### Confirmation

Proposed: the selected design follows the reconciled review and approved P-C6. Five focused solver-independent projection controls passed with zero failures under the compile-class `just unit-package pse-backend-native 'tears::tests'` route on 2026-10-11. Native optimum and enclosing qualification remain open; this does not accept the ADR or close the packet.

## Pros and cons

The formulation removes irrelevant ordering work without another solver owner. Its admission ceiling remains conservative, and once-per-structure work may have lower consequence than repeated native callbacks.

## More information

[Plan 36c](../plans/36c-mathematical-and-native-execution.md#p-c6--scc-local-tear-constraints-with-global-decision-coupling-preserved) owns implementation, verification and disposition.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of the accepted Plan 36 target.
