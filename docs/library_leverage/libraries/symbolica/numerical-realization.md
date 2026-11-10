# Stable operations and packed numerical execution

Inspected 2026-11-09 at `f6e91eb37`, including supplied untracked overview additions;
concurrent changes were preserved. **Interface-checked:** maintained Symbolica 3.0.1 source,
locked Numerica 3.0.1 source, representative mathematical consumers. **Proposed:** composed
realizations below. No compile probe, numerical execution or timing experiment was run.
This extends the existing [ecosystem discovery](briefing.md); it does
not select adoption or change Plan66 scope.

## What can actually be delegated together

A stable mathematical operation and a packed numerical domain are related but independent
choices. The supplier already offers three execution routes worth comparing:

| Route | Actual surface and useful delegation | Consequential limit |
| --- | --- | --- |
| Packed interpreter | `ExpressionEvaluator::map_coeff` to a supported packed domain, followed by `try_evaluate`/`evaluate` on packed inputs/outputs. Shares instruction interpretation across lanes without native compilation. | No automatic scalar-buffer packing API was found for the interpreter. Numeric trait/domain support and whole-value branch tests limit interchangeable use. |
| Packed JIT | `ExpressionEvaluator::jit_compile` with `wide::f64x4`, then packed `evaluate` or `batch_evaluate`; `BatchEvaluator<f64>::evaluate_batch` additionally accepts ordinary scalar buffers and packs lanes. | The inspected real-JIT scalar-buffer adapter has a multi-output unpacking hazard described below. JIT availability does not establish the project's conservative FP profile. |
| Compiled SIMD C++ | `export_cpp::<wide::f64x4>` or `export_simd_str`, a supplier loader and `BatchEvaluator<f64>` for `CompiledSimdRealEvaluator`. The current C++ adapter packs row-major scalar inputs, reuses buffers and scatters every output lane correctly. | Source uses xsimd and selected compiler/assembly profiles. The project's present compiled profile is scalar, with explicit admission and process/module ownership; supporting SIMD is a real profile extension. |

This is useful existing capability, beyond a proposal to hand-write a batch loop. A complete
candidate can leave arithmetic and layout conversion with the supplier while retaining physical
admission, source/domain guards, requested-product identity and original operation custody.
That can remove more machinery than merely replacing scalar arithmetic instructions.

Sources: maintained `evaluate/{domain.rs,evaluator.rs,backend.rs,export.rs}`. The relevant
methods are `map_coeff`, `try_evaluate`, `jit_compile`, `JITCompiledEvaluator::{evaluate,batch_evaluate}`,
`BatchEvaluator::evaluate_batch`, `CompiledSimdRealEvaluator`, and `export_simd_str`.

## Stable function identity must survive preparation

The current library evaluator directly dispatches only exp/log/sin/cos/sqrt/abs/conjugation
as builtin unary instructions. It calls `Real::log`, not `Real::log1p`. There is no inspected
native builtin Log1p instruction; writing `log(1+x)` and choosing a more capable coefficient
type does not turn that instruction sequence into `log1p(x)`.

A real current consumer exists: `pse-math/src/guarded_tests.rs::high_precision_real_values_survive_optimizer_and_jet_profiles`
constructs `(&x + 1).log()` and checks Value/First/Second. Its absolute tolerance near small
results does not establish preserved tiny increments. The external example in
`pse-backend-native/examples/numerical-investigation.rs` already uses scalar `ln_1p`.

Two candidate mechanisms have different completeness:

- **Expression-level body registration:** `FunctionMap::add_function_with_options` and
  `EvaluatorBuilder::add_function_with_options` associate a head with a symbolic body.
  `Always` inlines; `Auto` currently means `Always`; `Never` retains a sub-evaluator.
  Registering `log(1+x)` as a body preserves reuse but not stable numerical realization:
  the same rounded addition and logarithm remain. Keeping that body non-inlined also blocks
  subsequent `vectorize` in the inspected implementation.
- **A stable symbolic head with numerical implementations:** `SymbolBuilder::with_derivative_function`
  supplies an explicit mathematical derivative; `with_evaluation_info` attaches
  `EvaluationInfo::new().register(...)` implementations by numeric type. `with_cpp` or
  `with_cpp_generator` attaches exported external-function definitions. This can retain
  original x until a scalar or packed stable operation executes, and let Symbolica compose
  derivatives. It requires a complete operation contract rather than an isolated callback.

A stable-head route is therefore the more complete hypothesis when preserving an increment is
necessary. It need not introduce a new expression engine. The project would still need an
admitted vocabulary/physical signature, source obligations and supported numerical profiles;
its current formal symbol pool deliberately excludes custom attributes/callbacks, and its
native compiled eligibility is intentionally narrower than the supplier's export capabilities.
This discovery does not authorize bypassing those owners by registering an arbitrary user head.
A documented expression rewrite to an existing stable head could be the recognition phase,
but would require preserving original-domain checks before cancellation or reassociation.

Scalar Numerica f64/F64 implementations override `log1p` with `ln_1p`; multiprecision has a
specific implementation too. Generic `Real::log1p` instead uses an `asinh` identity. Thus an
otherwise well-designed head must choose actual domain implementations: the packed Real
implementations' missing `asinh` makes a generic default unsupported as a universal packed callback.
Stable head identity is not proof that all requested domains implement it stably.

Sources: maintained `atom.rs` (`SymbolBuilder`, `EvaluationInfo`),
`evaluate/function_map.rs`, `evaluate/evaluator.rs::evaluate_impl_no_ops`;
Numerica `domains/float.rs`, `domains/float/{native,multiprecision,simd}.rs`.

## Derivative components and independent lanes compose differently

`Dualizer` transforms an exact evaluator's slots into Taylor-component slots. The selected
shape is ancestor-closed; the expansion carries factorial/Taylor conventions. This is not
SIMD batching. Packed lanes instead hold independent values of each already selected slot.
A useful composed candidate is therefore:

1. Select the mathematical product and original guards.
2. Construct/transform the exact shared evaluator once, including required derivative components.
3. Map coefficients to uniform packed constants and execute independent points in lanes.
4. Map every output/component/row back to original semantic coordinates.

The maintained `Dualizer::vectorize_function` can generate external function component bodies
from symbolic derivatives. It refuses an unresolved derivative (`Symbol::DERIVATIVE`). An
explicit `_v` symbol supplies another external vectorization seam. Hence scalar callback
registration alone does not establish higher-order support; a mathematical derivative callback
and its resulting operation coverage are consequential. Retained non-inlined sub-evaluators
are rejected before this transformation.

This composition preserves library ownership across derivative preparation and batching,
without making each consumer maintain a differentiator or packing vocabulary. It remains a
candidate until a real requested product and complete execution profile establish fit.

Source: maintained `evaluate/dual.rs::{vectorize,Dualizer::vectorize_function}`.

## Packed domains, control flow and batching contracts

The [packed evaluator technical account](packed-evaluator-contract.md) owns the exact domain,
callback, branch, coefficient and adapter constraints. Interpreted x4/x8 support is broader
than inspected native x4 support. Scalar callbacks can be called per lane without becoming SIMD
arithmetic. The interpreter's whole-value branch test is not lazy per-lane selection. Native
scalar-buffer JIT and C++ adapters also have different output handling: the inspected JIT unpack
loop exposes a multi-output bounds hazard, while the separate C++ adapter scatters rows and
columns. These are source-derived conditions, not execution-tested product failures.

## Concrete research result and next discriminator

The strongest complete hypothesis is a stable head whose derivative and numerical definitions
remain visible through **one shared demanded evaluator**, followed by a supplier-owned packed
execution path for repeated provider-free points. This could combine improved numerical
realization, derivative reuse and less bespoke packing/dispatch. It is materially more specific
than treating SIMD and stable functions as unrelated API substitutions.

Start with the existing log-one-plus-increment control, first as scalar stable execution and then
as a multi-output value/First/Second batch. Require original x/domain identity, derivative axis and
Taylor conventions, tiny increments, negative-base integer powers, tails, empty/constant programs,
and finite/error behavior. Compare scalar interpretation with packed interpretation and a supplier
compiled route under the **same** existing no-fast-math/no-FP-contraction, explicit-ISA profile;
upstream examples recommending `-march=native` do not select repository policy. Short batches,
scalarized external functions, unsupported heads or added compilation cost can favor existing
scalar execution. Benefits are Proposed; no speed or compile-time improvement is Measured.

## API artifact coverage

The retained Symbolica API export selected GMP/MPFR without native code generation. Its public
path table does not expose `BatchEvaluator`/`evaluate_batch` in that documentation world. The
native backend contracts above are established by current feature-gated source inspection,
not by pretending those methods appeared in that export or treating absent exported rows as
library absence. Compilation of those native configurations was not performed in this inquiry.
