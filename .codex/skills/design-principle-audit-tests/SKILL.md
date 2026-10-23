---
name: design-principle-audit-tests
description: Audit test code, test evidence and shared test infrastructure against the design principles and test-assessment companion, maintaining a separate living diagnostic account. Use for an explicit test-scope audit or a test-focused diagnostic review prerequisite; ordinary test editing does not initiate an audit.
---

# Audit test design and evidence

Develop a current, evidence-grounded account of structural weaknesses in test code and the
evidence it provides. Assess maintainability together with the test system's ability to challenge
intended behavior. Diagnose causes and consequences without requiring a predetermined remedy,
library preference or test technique.

## Load the criteria and scope

Find the layered standard through repository instructions and standard.toml. Read the core
principles, heuristic companion, applicable domain profiles and repository binding. Load the
test-assessment companion selected by the core's test_assessment entry; it owns the test-specific
criteria. If that companion is unavailable, report the limitation rather than inventing a standard
or claiming a complete test-specific assessment. Loading criteria does not initiate another audit.

Scope follows testing responsibilities, beyond directory names: inline tests, doctests, fixtures,
builders, generators, oracles, assertions, doubles, reference data and relevant collection,
execution configuration and resource lifecycle. Read production contracts and dependency paths
far enough to judge independence, coupling and supported obligations. A production cause can be
reported without authorizing its repair or expanding into a broad production audit.

The binding owns invocation and applicability to subsequent reviews. A test-only engagement
does not require a broad production audit. Mixed scope identifies the applicable accounts and
their limits. Ordinary implementation, test maintenance and process installation do not trigger
this audit or a full test campaign.

## Establish the account

In this repository, maintain [the test audit collection](../../../docs/design_review/test-audit/README.md)
under docs/design_review/test-audit/, following the
[binding](../../../docs/design_review/design_principles/binding/pse-arrow.md#test-code-design-principle-audit).
An empty scaffold establishes no audited baseline, findings or alignment evidence.

Identify the audited source commit, included uncommitted changes, date, applied standard and
companion versions, inspected scope and meaningful limits. Distinguish source from publication
revision. A refresh uses intervening test, production-contract, dependency and configuration
changes as leads, then follows affected responsibilities into unchanged code as needed. Revisit
earlier findings when their premises change. Inherited observations retain their evidence date
and applicability limits; the broad audit's baseline cannot substitute for this account's.

## Investigate evidence and architecture

Choose breadth and depth according to uncertainty. Representative evidence paths can connect
scenario construction, exercised behavior, observation, expected-result derivation and assertion.
Credible refactors, contract changes, new cases and faults can expose coupling or blind spots.
These are inquiry techniques, not a fixed tracing sequence or exhaustive matrix.

Apply the companion's distinctions when judging production reuse, deliberate duplication,
shared helpers, correlated failures, assertion strength, doubles, isolation, domain fidelity and
execution fit. Ask which relevant mistakes the test would still detect. Similar-looking code,
an import, a large fixture, many dependent failures or a passing suite alone establishes no
architectural verdict.

Read decisive sources, including material contrary evidence. Use source reasoning where it
settles the question and targeted probes where consequential uncertainty remains. Perturbations
or known-correct/incorrect controls can be useful; no mandatory mutation campaign, test for every
helper, full-suite execution, benchmark or additional assurance framework follows. Quantitative
claims require measurements under named conditions. Follow repository environment and acceptance
timing for any execution; library questions use its documentation research route.

## Publish diagnosis with clear ownership

Use the shared [finding-writing guidance](../design-principle-audit/references/audit-writing.md).
Explain the claim at risk, structural cause, concrete consequence, known footprint, evidence,
uncertainty and relevant counterevidence. Distinguish established diagnosis from a suspected
wider footprint using the existing evidence vocabulary. A solution lead may help a later reviewer,
but the audit need not select a remedy, priority or implementation package.

Keep one primary home per finding across both audit collections. New test findings use the
binding's DTA-nnn identities; an existing broad finding retains its identity and home and is
linked, not copied or freshly dated. Preserve lineage across moves, splits and merges, and never
reuse retired IDs. Update this account in place; Git preserves prior accounts. Remove resolved
material only when current evidence establishes the correction, with closure evidence recoverable
through the removal commit. Link plans for scheduled disposition and progress rather than
maintaining a second execution ledger.

Use the shared coordination contract when delegation improves coverage or independent judgment.
Read-only contributors return sources, baseline, findings and limits; the coordinator reconciles
and publishes the account. Deliver its inspected baseline, material discoveries and coverage
limits. The account does not certify complete alignment, authorize repairs or automatically
start a solution review.
