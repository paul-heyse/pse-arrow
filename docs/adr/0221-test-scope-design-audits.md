---
id: ADR-0221
title: Add test-scope design audits and assessment guidance
status: proposed
date: 2026-10-23
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-04, AP-06, AP-07, DP-22, DP-23, PS-13]
blueprint: [§24.4]
review: "not-required: process discussion and edits are exempt from an audit/review artifact under the binding; independent written-process assessment is recorded in Confirmation"
evidence: Implemented
supersedes: []
superseded-by: null
revisit: Actual test audits expose ambiguous oracle independence, conflicting finding ownership or disproportionate assessment machinery.
verification: Inspect the written process against representative reuse, oracle, helper-failure and scope-boundary cases; validate skill discovery, agent references, ADR structure and documentation publication.
standard: "Core 3.4; Efficient Architecture 1.0; ProcessSimulator 1.5; Test assessment 1.0"
scenarios: []
---

# ADR-0221: Add test-scope design audits and assessment guidance

## Context

The operator approved a test-specific counterpart to design-principle-audit. Test code must
be maintainable while providing a credible challenge to production behavior. Reuse can improve
maintenance and consistency while also making implementation and expectations share a mistake.
Blueprint §24.4 needs an explicit route for this separately scoped diagnostic account.

## Scope

Amend §24.4 with a callable test-scope audit, a test-assessment companion to the selected standard,
and a separate living collection. Preserve the core/profile principles and gates, existing audit
history, plan ownership and product qualification. The operator authorizes local implementation;
this record remains proposed under the decision-PR lifecycle. Installing the process performs no
test audit and initiates no product repairs, full-suite campaign or remote publication.

## Drivers

Distinguish legitimate production-type reuse from circular expected-result calculation; share
useful fixture and assertion responsibilities without hiding scenario meaning; assess shared
blind spots as well as failure fan-out. Keep criteria in the standard, audit mechanics in the
skill and baseline/finding ownership in the collection and repository binding.

## Options

Only adding prose to the broad audit would leave test-scope invocation and evidence ownership
implicit. A separate test architecture standard would duplicate principles and risk conflicting
rules. Select a companion interpreting the existing standard for test evidence, consumed by a
small sibling skill and by other reviews when tests are in scope. Separate collection baselines
allow test audits to refresh without claiming that a broad production audit was refreshed.

## Outcome

Use design-principle-audit-tests for an explicitly requested test-scope diagnostic audit. Evaluate
test evidence and support architecture against the selected core, applicable domain profiles and
test-assessment companion. Maintain one living test account under docs/design_review/test-audit/.
Reuse is judged by responsibility and the faults the test can still detect, not by import counts,
duplication counts or a requirement to mirror production architecture.

### Consequences

The standard gains a versioned assessment companion without new principles, gates, quotas or
routine review obligations. The test collection has an independent source baseline and finding
identities. A review about test architecture can consume it without initiating a broad production
audit; mixed reviews identify the relevant accounts and reconcile their applicability.

### Compensating controls

Each finding has one primary home across the broad and test collections. Cross-cutting findings
retain their existing identity and are linked rather than copied. Neither collection owns plan
progress. Known-correct/incorrect controls and other probes are used where they settle material
uncertainty, not as an automatic mutation campaign or recursive test-for-every-helper requirement.

### Confirmation

Independent read-only process assessment on 2026-10-23 inspected source commit
0f0f5a67abf847bdbb14bb691a211a375ca7a1cd plus this installation's changes under the selected
standard and companion. The design-reviewer judged the written process suitable after the
coordinator corrected inherited unconditional broad-audit routing in the shared roles and
blueprint §24.4. No unresolved material process finding remained in that inspected scope.

Five synthetic cases distinguished production-type reuse, policy pass-through from policy
verification, a fixture repairing invalid input, a discarded Boolean comparator, and a narrowly
claimed round trip. This was source-based calibration, not executed product tests or test-audit
findings. The companion, skill, binding and collection retain their distinct ownership.

Discovery, agent, ADR and publishing controls establish the installed process structure only;
they establish no test-suite quality, scientific validity or product qualification. The rollout
reports actual commands and their results separately from this architectural judgment.

## Pros and cons

The sibling skill makes test-specific assurance questions explicit and independently refreshable.
A second collection adds a navigation and ownership obligation; primary-home links and distinct
baselines prevent duplicate diagnoses and false freshness claims.

## More information

- [Workflow owner](../authoritative_design/sections/design-change-workflow.md#section-24-4)
- [Existing audit rationale](0213-audit-led-workflow-and-library-orientation.md)
- [Standard selection](../design_review/design_principles/standard.toml)
- [Repository binding](../design_review/design_principles/binding/pse-arrow.md)

## Status history

- 2026-10-23: Proposed before the operator-authorized local test-audit process implementation.
