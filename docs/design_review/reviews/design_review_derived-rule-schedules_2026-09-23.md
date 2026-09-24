---
title: Derived rule scheduling contract review
date: 2026-09-23
status: proposed
evidence: Interface-checked
---

# Derived rule scheduling contract review

**Implementation-status addendum (2026-09-23).** This review retains its original
baseline, findings and scoped verdict. Plan 13 is now active; see the
[W19 repair checkpoint](../../plans/13-w19-repair-checkpoint.md) for implemented
changes, current isolated/static evidence and the incomplete functional campaign.
W20 and final G1–G7 acceptance remain open. This pointer does not convert the
original review or its historical measurements into current product qualification.


## 1. Decision and scope

**Proposed:** accept the bounded design in ADR-0077 for implementation, not behavioral
qualification. This supplements the two Rust-computation reviews and approved W07–W11
packet. Review is of specified contracts; runtime correctness remains unqualified.

## 2. Authority and lifecycle

Registry RuleDecl owns SQL, exact ports, head, outcomes and conflict policy. The checked
native plan owns actual operators. Their agreement precedes a pure RuleDependencies graph
and derived compiled.rule_strata. Runtime owns execution and W05 completion. Source IDs
survive reorder/rename; graph indices and schedules are derived, not source identities.

## 3. Contracts

Every read is declared and every selected head includes all producers. Effects include
negative, nonmonotone, support and conflict sensitivity. Unknown effects fail admission.
Settlement-sensitive SCC edges fail with a witness; finite positive SCCs iterate under
explicit bounds. Failure/exhaustion is not an empty or complete result.

## 4. Execution design

Native binding and effect extraction occur outside Salsa. Pure graph analysis derives
strata; native prepared plans iterate inside one complete program/region handoff. Equal
head cardinality is not convergence if support or representatives changed. All source
selections and implementation/settings inputs remain part of request validity.

## 5. Representative journeys

Adding a negative read changes the checked dependency graph and moves its consumer behind
settlement, or produces a prohibited-cycle witness. Adding a second head writer expands
the producer closure. A limit failure exposes no partial completion. Regeneration changes
reference schemas and compiler output schemas from the registry, never by editing output.

## 6. Acceptance gates

| Gate | Design verdict | Enforcement specified; behavioral evidence pending |
|---|---|---|
| G1 | Pass | Registry declarations and checked plans have distinct authority |
| G2 | Pass | Native SQL/operator semantics retained; reference stratum contract explicitly replaced |
| G3 | Pass | Complete reads/effects, producer closure and cycle checks precede execution |
| G4 | Pass | No ambient latest source or effectful Salsa query |
| G5 | Pass | Existing atomic complete-program handoff; failure is not completion |
| G6 | Pass | Complete epoch/source/policy keys and exact support/representative changes |
| G7 | Pass | Interface-checked design only; no integrated correctness or performance claim |

## 7. Findings and applicable principles

DM-01/02 authority, DM-13 constraints, DM-28/31 dependency preservation, DM-43 ownership,
DM-59/60 evidence and RCA §3/§5/§6/§9 are **Satisfied at the specified-design level** by
the boundaries above. Their implementation verdict remains **Unresolved** until named
units land. No normative requirement is waived by calling the design accepted.

## 8. Alternatives

Manual strata duplicate order authority. Conservative declaration-only schedules retain
extra effect declarations alongside SQL. Checked native effects plus pure graph scheduling
use the existing library graph/runtime boundaries without another execution framework.

## 9. Verification

**Proposed:** positive SCC, negative/aggregate/outer-join/subquery/conflict cycles, all
writers, empty/duplicate inputs, representative/support updates, exhaustion and generation
equivalence. W19 separately owns complete cold/reused rule journeys. The W10 graph reduction
uses pinned built-ins plus exhaustive tiny independent-oracle and certificate tests.

## 10. Authority reconciliation

ADR-0077 deliberately changes blueprint §6.11/§14.2 reference strata and §15.3 SCC backend
wording. Iterative kosaraju_scc replaces the algorithm-specific Tarjan wording without
changing SCC meaning. Matching-projected blocks retain their perfect-matching precondition.
No SHOULD deviation, new crate, family upgrade or broad qualification exception is introduced.

## 11. Decision and method

**Accept scoped design for implementation.** Formal ADR status/merge remains the repository
decision process. No product acceptance is awarded. Inspected RuleDecl/RuleSpec, rule_deps,
native validation/rounds and W06 RuleDependencies plus pinned skill contracts; no unexecuted
behavior is called Tested. The implementation packet remains the required completion list.
