---
id: ADR-0142
title: Compose independent process states and independently assess temporal conservation
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-06, DP-01, DP-03, PS-01, PS-04, PS-08]
blueprint: [§7.7, §10, §12, §13, §17.4, §22.2]
review: docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f02
evidence: Tested
supersedes: []
superseded-by: null
revisit: A new state formulation, physical translator or temporal realization requires meaning not expressible by independent coordinates and original inventory/flux/transfer expressions.
verification: Plan 25c focused state independence, semantic species correspondence, indexed balances, conditional-unit admission and independent temporal conservation controls with force-validation; assembled native journeys in Plan 25k.
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s02]
---

# ADR-0142: Compose independent process states and independently assess temporal conservation

## Context

Scalar material wiring duplicates stream identity and cannot state transported agreement. Local storage and an independently assessed inventory balance must survive both temporal realizations and authored event transfers. Plan 25c resolves F02/F33 and FU04/FU05 and contributes physical evidence to F16.

## Scope

Amend blueprint §7.7/§10/§12/§13/§17.4/§22.2 within the generic modeling kernel, including the required retained process-occurrence and context-complete document-reuse prerequisites. Scientific state definitions, pressure policies, reaction sources and translators remain authored knowledge. Qualification and actual endpoint permission remain Plan 25e responsibilities.

## Drivers

A species or inlet addition changes indexed declarations rather than flowsheet wiring. A memoryless unit remains algebraic in dynamics. Topology cannot manufacture causal unit execution, and solved derivatives cannot independently prove conservation.

## Options

Retaining scalar equation families or expanding them with a macro preserves competing stream, tear and cancellation identities. Select one state/connection occurrence with semantic-indexed coordinates and transport obligations. Retain the existing graph, nonlinear root and integration libraries rather than implement new algorithms.

## Outcome

Check process expressions once with declaration, field-role and ordinal attribution, retaining lexical/physical dependencies and indexed obligations. Reuse document interpretation only under the complete consumed identity/package context, matching clean admission of final sources. The wider grammar and general specialization migration remain their own work.

Declare independent state coordinates, dependent reconstruction and transported observations. New state slots use the identity owner's `ModelingProcessSlotV1` frame with their actual owner, declared role and canonical semantic indices; existing member frames and historical preimages remain unchanged. Admit compatible direct connections or an explicit translator; lower one occurrence into state bindings, transport obligations and topology. Indexed boundaries share balance formulation, while local storage alone adds inventory and initial-condition obligations. Conditional unit solves require owned inputs, residuals, outputs and admitted structure/capability before recycle iteration. Derived inputs compile an actual typed subtraction residual; point observations remain points and affine residuals are differences. Their numerical magnitude policy and provenance are preserved without affine offsets or selecting defaults again. Numerical projection uses the identity owner’s NumericalProjectionV2 and NumericalDifferenceProjectionV1 frames; historical frame spellings remain unchanged. Simultaneous initialization is an explicit alternative.

Temporal conservation declares inventory, original signed flux/source, tolerance and allowed event transfers together. Assess inventory change minus independently accumulated original flux and transfers. Inventory and transfer expressions are evaluated in their actual mode and boundary state. Original coordinate initial conditions remain required checks when a composite inventory introduces a derived stock. Transfer paths bind to the actual guard member in the owning instance rather than a fixture name string. Terminal events perform no reset; their endpoint retains the terminating mode and active scheduled input segment. Event-bearing simultaneous requests refuse under this scope.

### Consequences

The semantic producer, mathematical lowering, graph/recycle projection and generated consumers migrate together. Scalar signal ports retain their existing role. Replaced scalar material families, copied balances and bespoke closure implementations are removed after focused replacement controls pass.

### Compensating controls

Semantic species correspondence, independent state bindings, derived transport agreement, local initial conditions, external-coupling refusal, restoration and independent flux/reset closure have focused positive and refusal controls. Result policy cannot erase a Closure observation or turn missing evidence into NotRequired.

### Confirmation

**Implemented/Tested, 2026-10-01:** Maintainer-authorized C1–C5 implementation and its focused force-validated controls are recorded in [Plan 25c Verification](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25c-process-composition-and-conservation.md#verification): authoring 1, quantity 1, modeling 22, numerical projection 2, compiler 8 composite, native backend 18 plus derivative-demand 1, and runtime 20 composite controls passed on local Linux with the documented licensed/native conditions. Full generation passed. Final workspace compilation remains the first restart step; full series qualification remains Plan 25k. This record remains proposed pending its decision PR.

## Pros and cons

One process occurrence composes physical and execution consumers; the necessary migration crosses authored, compiler, native and result boundaries.

## More information

[Plan 25c](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25c-process-composition-and-conservation.md), blueprint §10/§12/§13/§17.4, and the Plan 25 coordinator's finding dispositions.

## Status history

- 2026-10-01 — proposed before maintainer-authorized Plan 25c implementation.
