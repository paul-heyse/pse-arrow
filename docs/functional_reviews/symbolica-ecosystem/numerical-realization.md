# Trustworthy numerical realization and repeated studies

2026-11-09. Inspected `7d89d16f` with concurrent operations/codegen changes preserved.
Applies core 3.4, efficient architecture heuristics 1.0, process-simulator 1.5 and the
pse-arrow binding selected by `standard.toml`. This is a functional-extension-review
assessment: proposed capabilities and comparative recommendations, not implementation
acceptance. Source observations are **Interface-checked**. Recommendations are **Proposed**.
No scientific execution, compile probes, health checks or performance measurements.

## O02 — Make stable operations an intentional mathematical realization

**Recommend pursuing first-class stable operations, initially log1p and a two-argument magnitude,
through the existing typed mathematics owner.** An author should be able to express a small-increment
logarithm or magnitude once, then obtain consistent values and requested derivatives in steady
models, fitting and repeated studies. This improves reliable model expression and removes the
need for individual property packages or backends to invent small-argument and overflow cases.
Do not silently promise that ordinary `log(1+x)` retains an increment already rounded away.

The concrete current foothold is
`pse-math/src/guarded_tests.rs::high_precision_real_values_survive_optimizer_and_jet_profiles`,
which constructs `log(1+x)` and tests Value/First/Second. The example
`pse-backend-native/examples/numerical-investigation.rs` separately uses `ln_1p`. Numerica
3.0.1's f64/F64 `Real::log1p` overrides call native `ln_1p`; multiprecision has its own override.
The maintained Symbolica evaluator's builtin Log instruction calls `Real::log`, so a better
numeric type alone does not recognize the earlier addition as a stable log1p operation.

**Selected realization:** declare the operation and physical/domain/derivative contract once
in the existing primitive vocabulary, lower it to a known Symbolica mathematical head, attach
an explicit derivative and numerical implementations, and keep that identity through evaluator
construction. For log1p, input/output are dimensionless with original `x > -1`; derivative
realizations retain that domain. For magnitude, require physically compatible inputs and retain
non-differentiability at the joint zero at the requested derivative order. Symbolica
`SymbolBuilder::with_derivative_function` and `with_evaluation_info`, and
`EvaluationInfo::{register,with_cpp,with_cpp_generator}`, provide the composition seams.
Register numerical callbacks at supported scalar domains and native source definitions for
selected compiled profiles. Derivative composition remains Symbolica-owned; packages supply
mathematical meaning rather than reimplementing jets.

This adds a small admitted vocabulary and supplier integration, while removing independent
stable-evaluation tricks in consumers. It should remain an explicit selected realization with
scientific numerical checks; a head name alone is not a precision guarantee. The current formal
symbol pool excludes arbitrary callbacks. Introduce known mathematical heads through the
mathematical initialization/registration owner, not by allowing arbitrary per-model global
callbacks or weakening pool validation. The current native compiled eligibility also needs an
explicit extension for these known pure operations.

**Alternatives assessed:** (a) register `log(1+x)` as a symbolic function body: native sharing is
useful but its numerical addition/log sequence still loses tiny increments, so not selected for
stable behavior; retaining it non-inlined also prevents current dualization. (b) evaluator-level
recognition of the old expression: useful optional optimization only after original guard/source
meaning is secured; not the sole author-facing contract. (c) blanket multiprecision: expands work,
retains rounding/input-intent questions, and does not establish each stable operation, so do not
select it. Existing plain functions remain appropriate where cancellation/overflow is not material.

**Delivery evidence:** one authored relation used through interpreted and selected compiled
Value/First/Second products, with independent expectations for tiny signed x, x=0, and the
log domain boundary; magnitude near overflow/underflow and its derivative singularity; original
source errors retained. No numerical benchmark is required to demonstrate the capability.

**Rule impacts:** primitive/function and derivative contract additions require the applicable
kernel/binding ADR route; blueprint §§7.2–7.4 and §14.3 describe the updated supported realization.
This is not permission to enable arbitrary callback symbols or fast math.

## O03 — Prepare demanded derivative products once and evaluate batches as one operation

**Recommend one repeated numerical-product operation using existing product demand and dynamic
shape owners, with scalar and admitted packed realizations.** Engineers running property tables,
parameter/uncertainty populations or many pure residual evaluations should request ordered outputs
and derivative products over ordered points, instead of issuing separate compilation/evaluation
workflows per scalar or output. This is a useful programmatic operation without requiring a UI.

Existing `jets.rs::ProductKind` already distinguishes SelectedPartials, JacobianAction,
TransposeJacobianAction, WeightedHessian and WeightedHessianAction; `Layout::with_pairs` builds
an ancestor-closed selected shape. `library/admission.rs::bounded_evaluator` already invokes
`HyperDual<Complex<Rational>>` and Symbolica `Dualizer::vectorize`. Do not introduce a new AD
engine or an independent global shape registry. Retain a compatible dynamic shape inside the
existing immutable prepared-product lifetime, using Numerica's shared shape and `variable`/clone
facilities where needed. A fixed generated hyperdual type is a deliberate numerical kernel
specialization only when a stable repeated shape earns the extra Rust monomorphization; it is
not the default representation for runtime-authored models.

**Selected composition:** original admitted demand → exact shared mathematical/derivative
preparation → selected numerical mapping → batch worker → ordered original-coordinate results.
Runtime seeds/weights remain execution inputs, while requested axes, selected pairs, profile and
layout qualify preparation. Consumers share preparation, not mutable scratch. A numerical
operation may pack pure independent arithmetic after original physical/domain admission; guards,
fallible providers and lazy branch identity remain semantically represented. Start with pure
branch-free regions, but make original point identity and individual failure outcomes part of
the operation rather than a reason to prohibit future mixed scientific batches.

The supplier already supplies real code beyond scalar-by-scalar loops:
`export_cpp::<wide::f64x4>` / `export_simd_str`, `CompiledSimdRealEvaluator` and
`BatchEvaluator<f64>::evaluate_batch` own packed execution, input rearrangement, persistent
packing buffers and output scattering. **Prefer extending the existing explicit C++ preparation
route to this supplier packing path**, preserving its process/module/scratch/admission owners
and conservative FP settings. Use scalar execution as an explicit eligible realization for
small or unsupported work, not as silent retry after an effectful failed packed attempt.

Packed interpretation is an eligible lower-startup alternative: `map_coeff` supports x4/x8
`EvaluationDomain`, then `try_evaluate` consumes packed values. However it needs caller packing,
and aggregate `is_fully_zero` branch tests are not lane masks. The Numerica SIMD Real trait has
six unimplemented hyperbolic functions; default `log1p` reaches `asinh`, so O02 must register a
real packed implementation or keep that region scalar. Integer powers use `powf_simd`, requiring
negative-base and FP-profile evidence rather than inheriting scalar integer-power behavior.
The x4/x8 domains can automatically invoke an f64 external callback separately for each lane;
this preserves access, not SIMD acceleration of the callback. Numerica x2 trait support does
not imply a Symbolica x2 `EvaluationDomain`, and inspected native compiled support is x4.

**Do not select the current real JIT scalar-buffer batch adapter as the initial realization.**
It is a worthwhile future alternative, but `backend.rs` currently unpacks
`out.chunks_mut(4*n_out)` from a single four-lane vector; with a full group and two outputs the
source slice extent exceeds four. This is source-derived, not a runtime-tested failure. The
separate compiled C++ SIMD adapter scatters by row/column/lane correctly. JIT also needs explicit
numerical-policy qualification. Do not rebuild a parallel generic batching framework merely to
avoid qualifying the fitting supplier path.

**Delivery evidence:** an actual property-table or pure multi-output scientific population,
including value and an action/selected-partial product, shares preparation across changed values;
check output/axis order, repeated points, zero-parameter and empty populations, uneven tails,
independent worker scratch, per-point domain failures and cancellation with no stale outputs.
Tail padding can execute pure callbacks on inactive zero lanes; choose a valid padding policy or
scalar tail where that would violate operation/domain meaning. Keep independent analytic or exact
oracles, not only scalar-vs-packed self-comparison. Quantitative speedup remains unclaimed.

**Rule impacts:** broaden the current scalar compiled numerical profile in blueprint §14.3;
add demand/batch interfaces to the existing math/runtime operation, not a new service or crate.
Derivative/scalar-domain eligibility is an explicit consumed contract. The required scientific,
numerical-profile and decision routes apply when selected; no `target-cpu=native` or FP-contraction
change is recommended.

## O05 — Offer claim-scoped numerical diagnostics from the same admitted mathematics

**Recommend a bounded numerical-assurance operation with explicitly distinct output claims.**
An engineer or model developer should be able to ask why a value is numerically fragile,
obtain an estimated accuracy at chosen precision, or request a validated enclosure where the
supported expression really permits one. This enables scientific diagnosis and exact-to-numeric
verification without changing ordinary binary64 solve semantics or converting every analysis
into a proof system.

Use the same admitted selected mathematical product, mapped to supported Numerica domains:
ordinary binary64, `DoubleFloat`/multiprecision or `ErrorPropagatingFloat` for approximate diagnostic
realization; `RealBall`/`ComplexBall` and `Float::{from_rational_round,add_round,sub_round,mul_round,div_round}`
for genuinely certified supported arithmetic. Keep result claims distinct: working precision,
propagated error estimate, exact identity, and guaranteed enclosure are not aliases. The current
`rational_binary64.rs` already owns directed MPFR rational conversion; reuse it for exact-floating
boundaries rather than adding another rounding policy. The covariance owner currently establishes
PSD/local-validity/statistical origin; that does not establish a validated enclosure of science.

**Selected initial functional envelope:** numerical diagnosis of an admitted arithmetic/property
product at declared inputs, plus certified enclosure for the supported exact rational/polynomial
arithmetic subset. Return an explicit unsupported/inconclusive claim for operations whose enclosure
contract is absent. Ball source certifies basic arithmetic and certain complex integer powers,
not inherited transcendental functions; no API trait or high precision can promote those to
certification. Source-domain and guard eligibility still precede numerical diagnosis.

Do not use rationals guessed from decimal approximations to strengthen exact scientific claims.
`truncate_denominator`, `round` and `round_in_interval` are useful optional explanations/candidate
simplifications within this diagnostic operation, with changed-value status retained; a rational
inside a certified interval is not proved identical to the enclosed unknown. PSLQ/lattice candidate
recovery belongs in a later analytic-discovery workflow with independent verification, not as an
automatic science rewrite. Ring-aware formatting is useful for these exact-domain diagnostics,
without adopting formatting as identity or persistence.

**Alternatives assessed:** a global automatic precision escalator hides scope, can repeat work
without a stronger claim, and conflates provider precision, so reject it as the default target.
Specialized current root certificates remain preferred for their own purpose. A separate bespoke
interval evaluator would duplicate native arithmetic while still needing scientific eligibility;
use library-supported domains and refuse unsupported certification instead. The accepted ordinary
factor/response SVD algorithms remain selected; numerical diagnosis does not reopen them.

**Delivery evidence:** one cancellation-sensitive relation, one rational rounding boundary and one
supported enclosure case; show the original binary64 value, chosen diagnostic result and exact
claim meaning. Include an unsupported transcendental enclosure, input uncertainty vs arithmetic
error, signed-zero/subnormal conversion and a violated original-domain predicate. Verify one
independent exact or multiprecision expectation. No universal enclosure guarantee is proposed.

**Rule impacts:** numerical policy/assurance result vocabulary and blueprint §§7,14,16,23 require
explicit target decisions. This recommendation adds a chosen diagnostic capability; it does not
silently change every solver's numerical domain or evidence strength.

## O10 — Add explicit-measure adaptive expectation studies

**Recommend an expectation/integration operation as a distinct study outcome**, composed with
existing model execution and O03's batching. Engineers could estimate mean cost, performance or
reliability indicators under declared uncertain inputs, including finite operating alternatives,
while retaining failures and statistical assumptions. This extends current generated designs and
linearized covariance propagation rather than pretending either already supplies probability
integration.

`workflow/sampling.rs::DesignRequest` supplies physical overlays and generator attribution;
`workflow/uncertainty.rs` propagates covariance using response Jacobians while keeping origin and
local validity. Neither interface specifies the full probability measure required for an adaptive
expectation. The new operation must explicitly name quantity-typed uncertain coordinates,
continuous measure/transforms and finite-category probabilities, selected outputs, original
execution request, estimator/stopping settings and reproducibility policy. A declared initial
family of bounded uniform coordinates plus finite categorical alternatives is useful and fits
existing physical overlay admission; dependent laws require an explicit joint transform/Jacobian
rather than an independence assumption. These are scientific interfaces, not hidden sampling flags.

**Selected realization:** Numerica `ContinuousGrid` plus `DiscreteGrid`/nested `Grid` owns adaptive
sampling and weights, `Sample::get_weight` identifies the mathematical weight, and
`StatisticsAccumulator` owns estimates/iteration statistics. Existing case operations evaluate
points or pure arithmetic batches; the study owner maps the declared measure, retains point
outcomes, schedules training/update barriers and interprets convergence. `MonteCarloRng::new(seed,stream_id)`
and export/import supply stream control. `clone_without_samples`, `is_mergeable` and `merge`
provide a coherent parallel iteration model: workers evaluate against compatible grid snapshots,
merge observations, and adapt at declared iteration boundaries. Do not let workers independently
update a shared evolving distribution and then treat all reported weights as one frozen law.

Results identify the estimand, realized measure, count/weights, estimate and statistical uncertainty,
convergence assessment, failed member population and original case provenance. A failed point is
not silently dropped or replaced; the operation either has an explicitly defined physical failure
outcome/indicator or withholds the expectation under that estimand. Retain replayable points/weights
or sufficient selected RNG/grid state under the existing result custody mechanism. A seed alone
must not imply bitwise reproducibility under changed parallel accumulation order.

**Alternatives assessed:** egobox DOE remains suitable for space-filling design, not a statistical
law. Fixed-weight Monte Carlo remains a simpler explicit realization for small inexpensive studies
and a useful cross-check. VEGAS is not uniformly better for sharply correlated peaks; require
law-aware channeling/transform or retain a suitable fixed method rather than advertise automatic
variance improvement. Deterministic time/DAE integration and its event sensitivities remain with
native solvers. Do not differentiate the adaptive sampler and call it a pathwise sensitivity;
initial derivative outputs may be requested pointwise, but derivatives of expectation need an
explicit estimator/interchange-of-derivative contract.

**Delivery evidence:** an analytic bounded continuous mean, a categorical mixture, mixed units
and an admitted nonlinear measure transform, with known normalization/weights; one failed-model
point retained; repeatable stream/state replay; independent execution workers and settled
cancellation; a multi-output pure model shares preparation with O03. Report uncertainty as an
estimator's statement, never a deterministic error certificate or physical truth.

**Rule impacts:** authored uncertainty/measure and estimand semantics plus result schema and
operation interpretation are new functional contracts. Follow the existing ADR/design routes;
Plan66 J03 is related context, not automatic adoption or a source of additional scope approval.
No Python product dependency or independent study service is recommended.

## Other execution targets and their selection conditions

CUDA export remains a conditional realization of O03 for a genuinely large pure numerical
population with suitable hardware and enough repeated device work to justify transfer,
compilation and retained-device lifetimes. It does not accelerate the symbolic preparation that
O01 removes. No device availability or complete eligible scientific vocabulary has been established
by this review. Prefer the existing CPU operation owner for the current target; reopen device
execution for a named workload with explicit function, precision, original-domain and cancellation
semantics. Supplier assembly and JIT are likewise distinct numerical profiles, not compiler flags
that inherit the current conservative C++ guarantee. The specific JIT adapter issue above must be
resolved before selecting that batch realization.

## Interactions, priority and explicit non-adoptions

Pursue O02 and O03 as the principal capability target: stable requested products evaluated over
repeated independent points. O02 scalar science can land before a packed profile; O03 may first
use existing pure heads while new stable packed callbacks are qualified. Reuse dynamic shape
preparation at its current owner instead of adding global caches. O05 supplies inspectable numerical
meaning and diagnostics independently; O10 can initially use scalar/fixed sampling then consume
qualified batching without changing its measure/result contract.

All supplied Numerica leads remain accounted for. Fixed hyperdual specialization is conditional,
not the default. Basic ring/integer/finite-field/poly/geometric kernels and exact matrix machinery
are valuable foundations but already delegated or require distinct algebraic outcomes handled by
other review opportunities. Existing general exact-matrix algorithms do not replace LDLᵀ/Gram
certificates. A local `sub_mul_assign` substitution is sensible ordinary integration if it clarifies
an exact update, but denominator-one optimization does not justify a standalone functional project.
Direct Numerica imports are appropriate for genuine numerics-only boundaries, not a reason to
split crates or claim a smaller workspace build before the actual closure changes. Python numeric
wrappers, WASM and alternate arithmetic backends are not selected for these native outcomes;
they remain alternatives for a concrete portability or isolated-reference requirement. Licenses
exclude none of these choices.

Sources: maintained Symbolica `atom.rs`, `evaluate/{domain.rs,evaluator.rs,function_map.rs,dual.rs,backend.rs,export.rs}`;
locked Numerica 3.0.1 `domains/{dual.rs,float.rs,rational.rs}`, `domains/float/{native.rs,multiprecision.rs,simd.rs,interval.rs}`,
`numerical_integration.rs`; current `pse-math/src/{jets.rs,library/admission.rs,native_compiled.rs,guarded_tests.rs,rational_binary64.rs}`;
`pse-runtime/src/workflow/{sampling.rs,uncertainty.rs}` and `src/math/sampling.rs`.
All numerical behavior claims above are source/interface observations; proposed delivered
scientific and functional behavior remains to be qualified in selected implementation scope.

Source routing: [stable function realization](../../library_leverage/libraries/symbolica/numerical-realization.md#stable-function-identity-must-survive-preparation), [packed adapter contracts](../../library_leverage/libraries/symbolica/packed-evaluator-contract.md#supplier-batch-adapters-have-materially-different-current-behavior), [certification domains](../../library_leverage/libraries/numerica/domains-and-certification.md#resolved-certification-and-exact-solving-contracts), [dynamic differentiation](../../library_leverage/libraries/numerica/differentiation-and-integration.md#derivative-preparation-is-a-reusable-structural-product), and [adaptive integration](../../library_leverage/libraries/numerica/differentiation-and-integration.md#adaptive-integration-and-combinatorial-output). [Exact conversion and root qualifications](exact-structure.md#o05--assurance-that-states-the-claim-not-merely-the-number-type) complement O05. Return to [the principal review](../symbolica-ecosystem-functional-improvements-2026-11-09.md).
