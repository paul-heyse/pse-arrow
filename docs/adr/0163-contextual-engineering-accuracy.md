---
id: ADR-0163
title: Bind numerical work to contextual engineering accuracy
status: proposed
date: 2026-10-05
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, PS-01, PS-10]
blueprint: [§16.1, §16.2, §16.5, §16.6, §13.3, §13.6, §19.2]
review: docs/design_review/reviews/design_review_contextual-accuracy-contracts_2026-10-05.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An admitted engineering goal needs a producer or observation outside the Plan 27 support matrix.
verification: Plan 27 scenarios S01-S06 and targeted policy, amplification, allocation and classification controls; Plan 25k integrated qualification and selected measurements.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#scenarios-that-distinguish-the-alternatives]
---

# ADR-0163: Bind numerical work to contextual engineering accuracy

## Context

Conditioning scales and physical residual allowances do not establish numerical error in a consumed engineering output. Static defaults alone cannot express useful scale-sensitive accuracy or whether uncertainty changes a declared decision. Plan 27 implements the contextual-accuracy review's CA-F01–CA-F04 correction.

## Scope

Amend the named numerical, identity, generated boundary and completion contracts. No new mathematical IR, numerical iteration algorithm, crate or universal error certificate is introduced. The historical source review retains its standard snapshot; this proposal uses Core 3.4/ProcessSimulator 1.5.

## Drivers

Ordinary work should avoid analytically irrelevant precision. Conditioning, original physical acceptance, output resolution and decision stability remain separate. Goal dependencies determine additional work, with actual evidence strength and one finite execution account. Review scenarios S01–S06 distinguish scale/datum behavior, explicit overrides, amplification, decision margins and unavailable evidence.

## Options

Static physical defaults remain useful fallback. Conditioning nominals confuse engineering meaning at affine datums and cancellation. Universal certification or paired solves impose unjustified no-goal cost. Select independent admitted engineering scales and optional goals, composed with existing native producers and finite strategy.

## Outcome

Resolve inherited engineering budgets as `max(F,r*S)`, with shared physical allowance F, shared fraction r and independently admitted engineering magnitude S. Freeze context/provenance before execution. Explicit overrides retain `absolute+relative*nominal`. Missing meaningful context uses a recorded canonical fallback unless strict engineering completeness is requested.

Physical closure declarations retain the distinction between a marked shared-rule reference and an explicit tolerance. A connected closure resolves each endpoint's full physical quantity, unit and independently admitted characteristic before taking the tighter endpoint budget. Both endpoint obligations and their provenance enter the frozen numerical interpretation. An identical numeric literal remains explicit; no value comparison establishes inheritance. This physical agreement check is independent of any certificate for downstream output error.

Registry-owned goals protect selected scalar outputs or the original optimum objective at supported observations. At least resolution or quantitative criterion is required. Inclusive bounds/bands express specifications independently of numerical resolution. Estimated is default; Certified requires actual admitted certification. Criterion-only goals require sufficient evidence to classify their decision without an unrelated output-digit cap.

Execution selectively refines actual reducible contributors under existing grants. Reserve fixed error contributions before allocation. Compatible goals share preparation/factors and dynamic comparator runs; ordinary no-goal work acquires none of these solely for accuracy. Keep sparse dependency views and bounded live study workers. A resolved violation stops refinement; unavailable capability, nonprogress, precision limits and exhausted grants stop unresolved. Native terminal outcomes and original acceptance retain precedence.

Completion consumes one immutable assessment. Assess permits resolved violation subject to existing obligations; RequireSatisfied additionally refuses it. Unresolved goals retain diagnostics and refuse the requested goal-qualified result. No goals means NotRequested. Generated Rust/Python/durable boundaries derive the same semantics; version changed identities, preserve supported old no-goal bytes and refuse unsupported versions.

### Consequences

Producer support differs by class: local correction is Estimated, objective gaps concern the optimum objective, paired trajectories supply empirical evidence. Factors, evaluations, proof work and reruns consume the existing account. Root-selection certificates remain independent mandatory obligations.

### Compensating controls

Admission rejects conflicts, invalid conversions and unsupported observations. Frozen acceptance identities stay separate from changing work demands. Analytic physical/amplification/classification controls and the assembled campaign verify behavior. No universal theoretical-order or speed claim is made.

### Confirmation

Proposed: this target and workload reasoning. The independent contracts review assesses the proposal; Plan 27 owns implementation and Plan 25k owns findings, qualification and measurements. ADR acceptance is separate from implementation acceptance.

## Pros and cons

The target directs work to useful outputs without imposing no-goal estimation. It adds binding/evidence integration and can honestly report unresolved when producers cannot establish the requested meaning.

## More information

[Plan 27](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/27-contextual-engineering-accuracy.md), [qualification and finding owner](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions), [source review](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md).

## Status history

- 2026-10-05 — proposed; maintainer authorized implementation. Publication and ADR acceptance remain separate from local execution.
