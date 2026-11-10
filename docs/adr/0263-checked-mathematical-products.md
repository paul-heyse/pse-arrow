---
id: ADR-0263
title: Use checked mathematical construction and stable operation products
status: proposed
date: 2026-11-10
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, DP-09, DP-13, PS-06, PS-07]
blueprint: [§7.2, §14.3]
review: "not-required: New kernel/primitive contracts within library-owned mathematics; authorized Plan73a/b/c execution follows the integrated functional review and independent actionability assessment."
evidence: Tested
supersedes: []
superseded-by: null
revisit: A new supported operation or backend cannot preserve the closed primitive profile, original domain/derivative meaning or admission before construction growth.
verification: Plan73 supplier before-growth/refusal/cancellation controls, typed stable Value/First/Second independent numerical cases, original guard attribution, private scratch and direct weighted-product preservation.
---

# ADR-0263: Use checked mathematical construction and stable operation products

## Context

The maintainer authorized integrated execution of Plan73a/b/c. Current mathematical products already retain named definitions and scalar preparation. Application copies of supplier expansion/export algorithms and producer-bound optimizer callbacks remain release-coupled. Ordinary `log(1+x)` cannot preserve a small admitted increment once the addition rounds it away.

## Scope

Amend blueprint §7.2 and §14.3 with new selected primitive/kernel contracts within existing library-owned mathematics. Preserve authored physical authority, callback-free formal pool symbols, explicit scientific effects, original guards and preconstruction admission. Durable numerical publication identity and receiving activation require their own decision before that later cutover; this record does not change the canonical hashing/commit contract.

## Drivers

Keep one mathematical authority and make compatible preparation reusable. Delegate algorithm-specific construction bounds and native differentiation to the supplier. Express numerical intent without generic precision escalation or a new AD engine. Existing instance binding and ordered scientific attribution remain application responsibilities.

## Options

Retaining mirrors preserves behavior but couples admission to supplier internals. Post-hoc byte counts or process RSS do not establish admission before growth. Select supplier-owned checked operations with borrowed current-operation control, retaining independent realized reconciliation and source scientific bounds. Select deterministic known mathematical heads for stable operations instead of permitting arbitrary callbacks or replacing every logarithm/norm expression. Native builtin implementation remains eligible only if it simplifies the same complete consumed contract.

## Outcome

### Consequences

Checked supplier operations receive the immutable input, operation settings and borrowed per-call cancellation/admission. They report incremental work and input-live/temporary/output extents before corresponding growth, preserve typed caller refusal, and return a complete product or typed failure. Observed cancellation/truncation cannot publish success. Returned immutable evaluators retain no producer control. Delete covered application formulas/rebinding with actual caller cutover; keep the scoped construction owner.

The reported extents cover supplier-visible mathematical backing: integer and MPFR capacities,
unreduced coefficient/cross-product and rational/complex temporaries, atom/workspace buffers,
optimizer staging and returned products. GMP/MPFR private algorithm scratch remains under the
existing separately admitted conservative foreign allowance in blueprint §14.3 and §18.8.
That allowance stays live through actual native completion and the applicable retained owner.
Its opaque coverage is recorded explicitly; coefficient-height recurrences establish managed
mathematical sizes, without claiming an allocator interception or a proven bound on GMP's
unexposed internal scratch. No arbitrary limb multiplier supplies that missing proof.

The admitted primitives include dimensionless `log1p` with original x > -1, positive `log_ratio(x, reference)` and magnitude over compatible linear/difference quantities. `log_ratio` retains both original arguments: near reference it uses the stable relative increment, elsewhere a qualified logarithmic ratio realization preserves the full positive domain without forcing subtraction to round to -1. Its selected first/second local partials use the original positive arguments; the mixed second partial is zero. The physical registry admits the actual ratio/logarithm kind rather than erasing it to a generic scalar. This operation supports the authored caloric migration where a global `log1p((T-T0)/T0)` replacement would otherwise fail far from reference. Magnitude Value permits joint zero; First/Second refuse it. Deterministic separate known heads and local partial heads preserve stable evaluation through requested order; Symbolica owns chain/Taylor composition. Initial required realizations are scalar interpretation and explicit conservative scalar/x4 C++ profiles. Fixed x4 ISA/ABI selection, source admission and worker ownership precede native use; no host-native target, fast math or contraction is inherited.

### Compensating controls

Preserve original source/domain and derivative-order checks, physical conversions, callback-free formal registration, direct finite weighted products, per-instance failure attribution and private workers. A head name or supplied source snippet does not establish numeric eligibility. Unsupported profiles refuse explicitly.

### Confirmation

Targeted tests must attack before-growth admission, overflow, nested cancellation, interrupted-success refusal and no partial publication. Independent tiny-increment, extreme-scale and derivative cases establish selected primitive meaning. Implementation and test evidence remain in Plan73; no performance benefit or whole-series qualification is claimed by this proposed record.

On 2026-11-10, `scripts/pse test pse-math --lib --features compiled-evaluator` passed
640/640 with zero skips/compiler warnings, baseline zero. The final maintained supplier
checked selection passed 18/18; the unchanged finite 512 MiB shared-DAG and 8 MiB runtime
diamond controls pass after native stage/coefficient bound corrections. Actual authored
caloric, ordered fitting/Study and receiving journeys pass their scoped composite acceptance.
[Plan73b Outcome](../plans/73b-supplier-controls-and-qualified-receiving.md#outcome) owns exact
commands, failed intermediate runs, affected reruns, source reconstruction and profile limits.
This establishes that local implementation scope, with formal decision status and assembled
Plan73h qualification still separate.

## Pros and cons

Supplier-owned operations remove duplicate algorithm knowledge and keep local scientific policy explicit. They require maintaining checked paths through nested builders and source export, reproducible supplier provenance and qualified numeric profiles. Known heads require controlled initialization and explicit support in every selected backend.

## More information

[Plan73 coordinator](../plans/73-symbolica-ecosystem-improvements.md), [shared preparation](../plans/73a-shared-mathematical-preparation.md), [supplier control](../plans/73b-supplier-controls-and-qualified-receiving.md), [stable products](../plans/73c-stable-operations-and-batch-execution.md).

## Status history

- 2026-11-10 — Proposed before new checked-operation and stable-primitive implementation under maintainer-authorized Plan73 execution. Formal ADR acceptance and actual implementation qualification remain separate.
- 2026-11-10 — Refined before the caloric correction to include positive `log_ratio` with original-argument partials. Actual consumer review exposed the far-from-reference subtraction/domain failure and the named temperature-ratio physical contract; Plan73c C02 retains its acceptance obligations.
- 2026-11-10 — Clarified supplier-visible backing versus the existing opaque foreign allowance after inspecting the actual native coefficient/normalizer path. Original before-growth control of visible inputs, outputs and temporaries remains required; no GMP allocator interception or process-RSS guarantee is introduced.
- 2026-11-10 — Authorized A/B/C implementation and scoped acceptance complete; evidence updated to Tested. Status remains proposed; no acceptance PR or whole-series qualification is implied.
