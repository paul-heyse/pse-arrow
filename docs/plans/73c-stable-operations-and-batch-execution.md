---
title: "Plan73c: Stable operations and batch execution"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73c-stable-operations-and-batch-execution.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Stable operations and batch execution

## Outcome and baseline

**Proposed.** Deliver O02/O03 in [Plan73](73-symbolica-ecosystem-improvements.md): authors express stable mathematical operations once, request values/derivative products and evaluate ordered independent points with compatible preparation retained. Scalar `log1p` and two-argument magnitude precede an explicit x4 C++ batch profile. Structural overlap, actual consumers, deletion and receiving acceptance belong here before returning to Plan66.

**Interface-checked.** Baseline `baadc7e32`, maintained Symbolica/Numerica3.0.1 with explicit opt-in native codegen. [Dynamic products/layouts](../../crates/pse-math/src/jets.rs) and [Dualizer admission](../../crates/pse-math/src/library/admission.rs) already provide selected/action demands. No new AD engine or global shape registry is needed. The builtin Log calls `Real::log`; using a higher-precision domain alone does not preserve an increment already lost in `log(1+x)`. Numerica scalar `Real::log1p` uses native `ln_1p`.

## Stable operation contract

Add admitted `log1p` and magnitude primitives through the current quantity/function owner. Register deterministic known mathematical heads in a separate fixed initialization owner, preserving callback-free formal/provider-lift pool symbols. Models cannot install callbacks or C++ snippets. Use [SymbolBuilder derivative/evaluation hooks](../../vendor/symbolica/src/atom.rs), `EvaluationInfo::register`, `with_cpp`/`with_cpp_generator`, and [Dualizer::vectorize_function](../../vendor/symbolica/src/evaluate/dual.rs) to retain mathematical identity through Taylor components and selected numerical realization. The [stable-realization research](../library_leverage/libraries/symbolica/numerical-realization.md) explains why registering an inlined `log(1+x)` body is insufficient.

Known heads are the selected initial mechanism. C01 may replace them with supplier builtin support only after a finite comparison establishes a simpler complete derivative/export/receiving route. Do not keep two production meanings or registrations. Stable numeric callback availability alone does not establish pure admission, checked source size or portable receiving.

`log1p` consumes/returns dimensionless quantities and retains original x>−1, requested-order and finite/error behavior. Magnitude requires physically compatible arguments and returns their dimension, using scaled/hypot value evaluation. Joint zero remains nondifferentiable for derivative requests. First/Second formulas must also avoid avoidable extreme-scale overflow/underflow; stable value alone is insufficient. C01 establishes their complete numerical contract with independent high-precision/analytic controls before profile implementation. No global rewrite of existing `log(1+x)` or blanket multiprecision escalation is selected.

Current [scalar native export](../../crates/pse-math/src/native_compiled.rs) admits seven unary builtins and excludes opaque channels. Extend known pure eligibility together with B03 checked export and B05 fixed operation identities for receivers. Verbatim C++ definitions obey conservative FP/source-admission requirements; they do not grant arbitrary source execution. Retain original guards/source receipts outside numerical instruction equivalence. Apply the kernel/binding ADR and architecture route before changing admitted primitives or supported profiles.

## One ordered batch operation

Consume an admitted prepared product, ordered points, demanded outputs/actions and runtime seeds/weights. Produce ordered original-coordinate results with point identity and individual outcomes. Share immutable preparation; each worker owns mutable packing/evaluator/output scratch. Runtime values/seeds do not rebuild compatible shapes; axes/pairs/layout/profile/function semantics qualify preparation.

Start with eligible pure branch-free regions, retaining original per-point domain/derivative admission. Provider effects and lazy branch identity cannot become eager pure arithmetic. Define empty populations, constants/zero parameters, repeated points, uneven tails, partial failures and canceled operation results. Derivative components and independent lanes are different dimensions; preserve Taylor factorial conventions and original axis order.

The selected packed route extends the existing C++ compiler/module owner with supplier `export_cpp::<wide::f64x4>`/`export_simd_str`, `CompiledSimdRealEvaluator` and `BatchEvaluator<f64>::evaluate_batch`. Supplier [backend source](../../vendor/symbolica/src/evaluate/backend.rs) owns scalar-buffer packing, persistent buffers and row/column/lane scattering. The [packed contract](../library_leverage/libraries/symbolica/packed-evaluator-contract.md) documents x4 compiled versus x4/x8 interpreter support and numerical differences. B supplies checked source/control; retain existing process/module custody and explicit no-fast-math/no-contraction/no-host-native ISA settings.

Register actual x4 stable implementations where required: SIMD `Real` default `log1p` reaches unsupported hyperbolic functionality, and scalar callback fallback evaluates per lane rather than providing SIMD acceleration. Integer powers need separate negative-base/signed-zero/subnormal checks; trait names do not establish scalar repeated-squaring equivalence.

Use scalar realization explicitly for small/unsupported work, selected before effects rather than retried after a failed packed attempt. Use a valid in-domain padding point or scalar tail; supplier zero padding can call invalid-domain callbacks on inactive lanes. Interpreter whole-vector zero tests are not lane masks; mixed-lane lazy branches are outside initial packed eligibility. Cancellation retains truthful per-point outcomes but no stale complete-success population.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| C01 | open | Joint A01/B01 contract design | Stable primitive/signature/domain/derivative formulas, registration, scalar/x4 profiles and receiving identities settled; builtin alternative closed before representation freezes. |
| C02 | open | C01/A01 | Scalar interpreted stable Value/First/Second and an actual authored scientific consumer implemented with original guards/source reporting. |
| C03 | open | C02 and working B03/B05 | Compiled scalar stable products and portable receiving delivered; replaced consumer numerical tricks removed. |
| C04 | open | A02 demanded preparation and C01 | Ordered batch request/outcomes and private reusable worker scratch delivered, initially explicit scalar realization. |
| C05 | open | C03/C04 and B02/B03 checked operations | x4 C++ batches use supplier packing/scattering, stable realizations and valid tails. |
| C06 | open | C02–C05 and actual repeated property/residual/fitting/study callers | Consumers migrated, compatible preparation reused and duplicate per-point/output preparation or packing removed. |
| C07 | open | C04/C05 eligible workloads; H01 build input | Packed interpreter/JIT/device/fixed-shape decisions closed, selected implementations integrated or reasoned no-adoption outcomes recorded. |

C02/C03 use one authored relation through interpreted and compiled Value/First/Second, tiny signed increments and zero, log boundary/out-of-domain, magnitude extreme scales and joint-zero derivative refusal. Preserve independent expected values/derivatives and original failure reporting. Stable heads must survive the selected actual receiver, not only a local callback test.

C04/C05 cover multi-output value plus selected partial/action, repeated/empty/constant points, output/axis order, uneven tails, independent scratch and individual domain failure/cancellation. Preserve A's finite direct weighted-product countercase. C06 migrates an actual property-table/pure-residual population and relevant fitting/study consumer, not merely a wrapper around freshly prepared scalar calls. Study estimands remain F-owned. Retire replaced construction, dispatch, packing, fixtures and configuration with the migrated caller.

C07 compares packed interpretation when startup and supported vocabulary make it useful; it must reuse the same batch contract. The inspected real JIT scalar-buffer multi-output unpack can exceed a four-lane source vector. Selecting JIT requires source repair/qualification, including empty/zero-parameter layouts and its numerical profile. This concern is source-established, not runtime-reproduced. CUDA requires suitable hardware and a named large pure population whose transfer/compile/device lifetime earns it; it does not speed symbolic preparation. Fixed generated duals require a stable kernel shape and H's monomorphization assessment. Default remains dynamic shapes and CPU profiles. No unused backend estate is mandatory; negative outcomes retain the exact workload/contract condition that would reopen them. Alternate arithmetic/Python/WASM boundary decisions belong to H01.

## Verification

**Proposed:** scoped compilation and focused math/compiler/runtime tests through `pse`, with explicit Arrow validation as needed. Independent analytic/exact/high-precision expectations cover stable values/derivatives, extreme scales, original-domain failures, ordered numerical populations, power/profile behavior, scratch and cancellation. Scalar-vs-packed self-comparison alone cannot establish a shared transformation's correctness. H owns assembled property/fitting/study/native/receiver journeys after functional scope. No tests, compilation or performance measurement occurred in authoring; expected less repeated preparation/dispatch is qualitative.
