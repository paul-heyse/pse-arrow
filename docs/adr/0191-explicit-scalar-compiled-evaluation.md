---
id: ADR-0191
title: Select scalar compiled evaluation explicitly
status: proposed
date: 2026-10-15
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, DP-09, PS-06]
blueprint: [§14.5, §19.2, §19.3, §21.5, §25]
review: docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc03
evidence: Tested
supersedes: []
superseded-by: null
revisit: A requested provider, derivative order, cancellation boundary or target cannot preserve the selected scalar contract.
verification: Plan 41e E4 scalar compiled versus interpreted and independent arithmetic/domain controls, no-FMA witness, compiler cancellation and owner/scratch teardown.
---

# Select scalar compiled evaluation explicitly

## Context

Plan 41 RC03 is accepted as a bounded capability route. Exact Symbolica 3.0.1 supplies scalar C++ export, while its default compilation enables fast math and native ISA selection. Current guarded arithmetic blocks already separate scientific guards, branches and provider effects from numerical evaluation.

## Scope

Amend blueprint §14.5/§25 to admit explicit scalar compiled arithmetic under the existing mathematical authority. This adds an evaluator binding within the current model and solver pipeline, not a new model, default evaluator, JIT/SIMD guarantee or floating-point relaxation. The proposed record precedes maintainer-authorized implementation; accepted status still follows the decision-PR route.

## Drivers

Repeated value callbacks should be able to execute a prepared arithmetic block without interpreter dispatch. Preparation, source/compiler overlap, executable ownership and independent worker scratch must remain admitted. Existing cancellation and error boundaries must remain effective.

## Options

Retain interpreted evaluation only; enable native-code defaults globally; or select a bounded scalar C++ profile explicitly. Choose the explicit profile. Keep interpreted evaluation as the default and for its declared unsupported orders/regions, without silently broadening compiled eligibility.

## Outcome

An explicit compilation selection enables only qualified scalar value arithmetic. Symbolica owns export and evaluator loading. Generated code uses no inline assembly, fast/unsafe math, native ISA selection or FP contraction. The compilation owner stops and joins its compiler child on cancellation and publishes only successful completed preparation. Prepared products own executable storage; workers own independent admitted scratch. Existing coordinate mappings, guards, nonfinite/error checks and original assessment remain authoritative.

### Consequences

The evaluator choice participates in consumed compilation identity. Study request/operation version 7 and definition version 11 carry the required typed selector; producers and consumers migrate together without an omission-default compatibility path. Scientific replay and lazy variant descriptor version 2 likewise carry directly embedded preparation; their outer/nested envelope versions remain distinct. New artifact/view frames preserve historical frame meanings. Feature availability does not select evaluation. Provider callbacks and higher derivative products remain on their supported interpreted paths unless separately qualified. Compilation and retained working set are real costs; no speedup is assumed.

### Compensating controls

Qualification compares actual production consumers with independent arithmetic, branch/domain/nonfinite and no-FMA controls, plus cancellation, failed preparation, scratch isolation and teardown. Stock scalar batch loops are not SIMD and supply no such claim.

### Confirmation

Tested on Linux on 2026-10-16 through the eight native_scalar production controls, compiler directional identity control, current generated request/replay/variant codecs and public Python admission. Installed Symbolica 3.0.1 supplies the selected export/loading route. Plan 41 owns the bounded outcomes and assembled acceptance limits. Only explicitly selected scalar Value arithmetic is adopted; no global evaluator default, SIMD/JIT, provider/derivative compilation or timing speedup is established. Proposed status is unchanged pending the decision-PR route.

## Pros and cons

A narrow library-owned physical route removes interpreter work where requested. It adds compiler-process and executable lifetimes and does not accelerate unsupported provider/derivative regions or sequential solver iterations by batching them.

## More information

[Plan 41e](../plans/41e-derivative-reuse-and-evaluator-capabilities.md) owns implementation, investigation decisions and acceptance. Plan 41f owns assembled qualification. Environment/provenance proof and statistical timing campaigns are excluded.

## Status history

- 2026-10-15: Proposed after the maintainer accepted RC03 and authorized execution of Plan 41d/41e.
