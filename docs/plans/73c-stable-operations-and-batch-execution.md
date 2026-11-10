---
title: "Plan73c: Stable operations and batch execution"
status: in-progress
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73c-stable-operations-and-batch-execution.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Stable operations and batch execution

## Outcome and baseline

**Proposed.** Deliver O02/O03 in [Plan73](73-symbolica-ecosystem-improvements.md): authors express stable mathematical operations once, request values/derivative products and evaluate ordered independent points with compatible preparation retained. Scalar `log1p` and two-argument magnitude precede an explicit x4 C++ batch profile. Structural overlap, actual consumers, deletion and receiving acceptance belong here before returning to Plan66.

The implementer's scope is:

- **Required mathematical mechanism and vocabulary:** [C01](#C01) establishes admitted stable primitives, their exact local derivatives, numerical profiles and fixed portable identities. [C02](#C02) implements interpreted Value/First/Second and an authored caloric consumer. This is new source-level scientific capability, not a rewrite of every existing logarithm or norm.
- **Required compiled capability:** [C03](#C03) delivers those same selected products through the existing scalar C++ owner and the qualified cold receiver. [C05](#C05) adds the explicit x4 C++ profile, reusing supplier packing/scattering and a scalar tail.
- **Required cross-cutting execution mechanism:** [C04](#C04) implements one ordered batch request/result and reusable private worker scratch. [C06](#C06) moves actual property/residual and fitting/study populations onto it, deleting redundant point/output preparation and replaced packing/dispatch. The mechanism serves independent scientific outcomes; it is not merely an expectation-study wrapper.
- **Bounded additional-backend decisions:** [C07](#C07) compares packed interpretation, repaired JIT, device execution and fixed-shape specialization against named workload/contract criteria. Selected additions must be delivered; no-adoption decisions retain concrete reopen conditions.
- **Preserved behavior:** original physical/domain/derivative admission, selected/direct actions, source attribution, conservative floating-point settings and truthful individual outcomes remain mandatory across every profile. There is no default global precision escalation or workstation resource cap.

The package sections specify what to implement and remove. Their table is an execution index,
not a substitute for those briefs. All statuses remain open and no old-plan package gates work.

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

Use scalar realization explicitly for small/unsupported work, selected before effects rather than retried after a failed packed attempt. The selected initial x4 profile uses a scalar tail; supplier zero padding can call invalid-domain callbacks on inactive lanes. Interpreter whole-vector zero tests are not lane masks; mixed-lane lazy branches are outside initial packed eligibility. Cancellation retains truthful per-point outcomes but no stale complete-success population.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [C01](#C01) | complete | Joint A01/B01 design slices | Stable primitive/signature/domain/derivative formulas, registration, scalar/x4 profiles and receiving identities settled; builtin alternative closed before representation freezes. |
| [C02](#C02) | complete | C01/A01 contracts; B02 checked known-head derivative slice | Scalar interpreted stable Value/First/Second and an actual authored scientific consumer implemented with original guards/source reporting. |
| [C03](#C03) | in-progress | C02; working B03 scalar export and B05 numerical receiving | Compiled scalar stable products and portable receiving delivered; replaced consumer numerical tricks removed. |
| [C04](#C04) | complete | A02 working demanded-product slice; C01 layout/profile contract | Ordered batch request/outcomes and private reusable worker scratch delivered, initially explicit scalar realization. |
| [C05](#C05) | in-progress | C03/C04; working B02/B03 checked x4 operations | x4 C++ batches use supplier packing/scattering, stable realizations and scalar tails. |
| [C06](#C06) | in-progress | C02–C05 capabilities; actual repeated property/residual/fitting/study callers | Consumers migrated, compatible preparation reused and duplicate per-point/output preparation or packing removed. |
| [C07](#C07) | complete | C04/C05 named workload; H01 dependency/build observations | Packed interpreter/JIT/device/fixed-shape decisions closed, selected implementations integrated or reasoned no-adoption outcomes recorded. |

<a id="C01"></a>

### C01 — Fix stable meanings, derivatives and supported profiles

Deliver the admitted operation declarations and a fixed known-head implementation contract
for dimensionless `log1p(x)` and two-argument `magnitude(x,y)`. The latter consumes compatible
linear/difference quantities with matching complete physical type, basis, subject and reference
conventions, and returns that type's magnitude. Affine physical points are not silently treated
as vectors. Quantity inference owns eligibility through
[quantity operations](../../crates/pse-quantity/src/operation.rs) and
[function signatures](../../crates/pse-quantity/src/functions.rs); the compiler's
[typed request](../../crates/pse-compiler/src/typed_math.rs) and
[BodyBuilder](../../crates/pse-math/src/typed.rs) retain original source checks before algebra.
Add declarations in their authoritative registry/source owners and regenerate during execution;
do not maintain a duplicate runtime primitive registry or hand-edit generated outputs.

The local mathematical contracts are fixed now. For finite real `x > -1`, `f=log1p(x)`,
`f'=1/(1+x)`, `f''=-1/(1+x)^2`. Value preserves tiny signed increments and signed zero through
the actual `ln_1p` numerical operation. `x=-1`, `x<-1` and nonfinite inputs retain explicit
original-domain/numerical failure; extra precision after computing rounded `1+x` is insufficient.
For magnitude, `h=sqrt(x²+y²)`, gradient `(u,v)=(x/h,y/h)` and Hessian
`(1/h)*[[v²,-u*v],[-u*v,u²]]` for `(x,y) != (0,0)`. Value at joint zero is zero; First/Second
refuse joint zero. Stable execution uses `hypot` or scale-normalized evaluation, never the naive
sum of squares. Normalize with `m=max(|x|,|y|)`, `a=x/m`, `b=y/m`, `r=sqrt(a²+b²)`, then
`u=a/r`, `v=b/r`; an overflowing unrequested `m*r` must not turn finite requested gradient
components into zero. Evaluate curvature products with exponent-scaled multiplication/division
(mantissa/exponent decomposition or equivalent supplier-native operation), so a subnormal
intermediate such as `u*u` or `1/m` does not destroy a representable final component. Requested
directional/weighted actions contract these local formulas with original seeds/weights without
forcing all unrequested full-Hessian entries to materialize. Genuine final overflow remains an
honest numerical outcome; these primitives do not promise correct rounding or validated enclosure.

Keep mathematical identity and stable realization together using deterministic known heads,
including internal local-partial heads where expanding a derivative into ordinary arithmetic
would lose the required scaling. Public derivative order is initially through Second; unsupported
higher order refuses. Symbolica owns chain/Taylor composition. Its maintained 3.0.1
[`SymbolBuilder` hooks and `EvaluationInfo`](../../vendor/symbolica/src/atom.rs),
[`Dualizer::vectorize_function`](../../vendor/symbolica/src/evaluate/dual.rs) and
[domain resolution](../../vendor/symbolica/src/evaluate/domain.rs) supply the composition.
Register exact derivative callbacks and actual f64 numerical implementations, plus deterministic
scalar/x4 C++ definitions for the same meanings. A local-partial head carries its own fixed
operation identity, arity/order and supported numerical/export profiles; no user callbacks or
source strings enter this closed set. Do not register these callbacks on the callback-free
formal/provider-lift pool symbols. Initialization checks existing incompatible preclaims.

The required profiles are scalar binary64 interpretation, scalar binary64 C++ and x4 C++ for
pure branch-free regions. Explicit packed arithmetic/head implementations are required; the
generic SIMD `Real::log1p` default is unsupported here. Numerica 3.0.1's native
`domains/float/native.rs::{log1p,hypot}` and the
[source-qualified stable account](../library_leverage/libraries/symbolica/numerical-realization.md)
establish scalar dispatch; other diagnostic domains are admitted only when D supplies their
actual implementation and claim contract. B05 receives a fixed operation/profile version,
not a serialized callback. Compare a supplier builtin route only if it supplies the same complete
typed/derivative/export/receiving contract more simply; close that finite alternative before
representation freezes, retaining one meaning and initialization route.

Readiness is shared A01/B01 contract design, not completion of their dependent implementations.
C01 supplies stable descriptors to B03 and B05 while B02 implements checked derivative growth.
Acceptance for this foundation includes physical-type rejection, fixed initialization and
profile/arity consistency, analytically known gradients/Hessians, tiny increments and scales
where naive square/cube formulas fail. The coordinator's required primitive/kernel/binding
decision precedes product implementation; no blanket recognition rewrite of existing formulas
is included.

<a id="C02"></a>

### C02 — Implement interpreted stable products and authored science

Implement compiler/typed lowering from admitted primitive calls to C01's known heads, original
`x>-1` and magnitude derivative-domain checks, scalar f64 evaluation and selected Value/First/
Second demand. Extend the existing `typed.rs`, `library.rs`,
[local partials/execution](../../crates/pse-math/src/execution.rs),
[typed partials](../../crates/pse-math/src/typed_partial.rs) and
[dynamic layouts](../../crates/pse-math/src/jets.rs); retain A's named definitions and partials
instead of flattening each derivative into a new preparation pipeline. `Dualizer` provides
Taylor components; divide/multiply by the proper factorial conventions at the existing layout
boundary, not by a second AD engine. B02's working known-head checked-vectorization slice is
needed for this path; B02's full caller retirement need not finish first.

Execution exposed a necessary extension, recorded before implementation in proposed ADR-0263
and blueprint §7.2.1: positive `log_ratio(x, reference)` retains both original arguments.
Near reference it evaluates `log1p((x-reference)/reference)`; elsewhere it uses a qualified
positive logarithmic ratio, including a log difference when forming the quotient would
overflow or underflow. Its local first partials are `1/x` and `-1/reference`; diagonal Second
partials are `-1/x²` and `1/reference²`, with zero mixed partial. Fixed local heads preserve
the exponent-scaled realization through Symbolica composition. Both original arguments have
source-attributed positive guards. The actual temperature/temperature operation returns the
existing logarithmic temperature-ratio physical kind rather than coercing it to a generic scalar.

Use [authored caloric correlations](../../packages/reference/methods/models/correlations.pse)
as the actual scientific consumer. Migrate the selected entropy log-ratio term to the explicit
admitted `log_ratio(T,T0)` operation with a stable near-reference realization, preserving positive absolute
temperatures, ratio dimension, datum and applicability interval. Do not assume a subtraction
of already rounded temperatures recovers information never supplied; the improvement preserves
the admitted increment passed to `log1p`. A global relative-increment rewrite is insufficient:
far below reference it can round to -1 while the original ratio is still positive. Verify
near and far original temperatures independently, including finite extreme log ratios.
Keep the original algebraic law and physical
interpretation, and include the [caloric facade](../../packages/reference/methods/models/caloric.pse)
and ordinary property selection. Add an authored small dimensioned constitutive magnitude
consumer with compatible component quantities so magnitude tests cross physical inference and
source attribution rather than only register a callback in a math test.

Acceptance executes one selected relation through Value/First/Second with reordered axes and
original source reports. Use tiny positive/negative increments, signed zero, near `-1`, exact
out-of-domain refusal, unequal/extreme component scales, subnormals and joint-zero Value versus
derivative distinctions. Include a finite derivative/action whose unrequested naive magnitude
or curvature intermediate would overflow/underflow. Independent analytic and chosen high-precision
expectations establish numerical meaning; copying interpreted results into expected fixtures
does not. Preserve useful controls from
[`guarded_tests`](../../crates/pse-math/src/guarded_tests.rs), including the existing
`high_precision_real_values_survive_optimizer_and_jet_profiles` scope, but add revealing tiny
increment assertions rather than relying on a loose absolute tolerance.

Delete the migrated consumer's superseded numerical workaround and duplicate local-partial
setup once replaced; do not delete unrelated `log(T/T0)` laws or generic hypot callers merely
because the new primitive exists. Return working interpreted stable numerical products and
fixed identities to B05. C02 does not require compiled C03 or production cold receiving;
otherwise the supplier/consumer sequence would be circular.

<a id="C03"></a>

### C03 — Execute and receive compiled scalar stable products

Extend the existing [native compiler owner](../../crates/pse-math/src/native_compiled.rs) to
admit C01's closed pure stable heads and generated local-partial definitions through B03's
checked export. The input is the same admitted selected exact/derivative product delivered by
C02, not a separate C++ expression authored by a model. The output is a finalized native
scalar module/template with original coordinate/demand identity and private mutable evaluator
workers. Keep the module's temporary directory/live-library custody and existing publication
boundary; failing one pending function cannot publish a partial successful module.

Use the maintained 3.0.1
[`export_cpp_str::<f64>`](../../vendor/symbolica/src/evaluate/export.rs),
[`CompiledRealEvaluator` loader](../../vendor/symbolica/src/evaluate/backend.rs) and
`EvaluationInfo::{with_cpp,with_cpp_generator}` from
[`atom.rs`](../../vendor/symbolica/src/atom.rs). These APIs insert source definitions verbatim,
so only the fixed checked implementations qualify; arbitrary provider callbacks/snippets remain
outside the profile. C++ uses `log1p`/`hypot` and the same stable local derivative component
realizations, preserving explicit no-fast-math/no-contraction/no-host-native-ISA settings.
Original physical/derivative guards still run in the scientific owner; native purity is not
domain admission or a permission to execute an inactive branch.

Consume B05's completed numerical publication/acquisition contract through the new typed
`MathService` numerical-artifact path, not `portable::reuse_body_current`, which returns a
reconstructed `AdmittedBody`. B05 publishes an interpreted numerical demand/profile product
from working C02; it does not depend on this package. On a producer with admitted source,
ordinary preparation may still use the retained exact scalar/derivative products before mapping
to f64 and compiling. A cold execution-only receiver instead gets the qualified f64 program,
layouts, known-head versions and original guard/source custody, with no exact preparation family.

Select **receiver-local native compilation from the qualified numerical stages**. The current
`native_compiled::Batch::push` already consumes `ExpressionEvaluator<f64>` plus inspected
storage/export facts; exact Symbolica source is not required to emit that numeric program.
Add a math-owned operation on B05's restored numerical-template handle which traverses the
eligible arithmetic stages, supplies their restored f64 evaluators and realized facts to B03's
checked exporter, and builds a new profile-qualified native numerical template. Keep original
guard/control schedule, output/axis/pair layouts, source attribution and scientific receipts
alongside those stages; do not reconstruct them from generated C++. Fixed C01 known-head and
local-partial identities resolve only to the reviewed scalar C++ definitions. Unsupported
instructions, head versions or missing custody refuse before export. Recompilation performs
no differentiation, symbolic Horner rebuild, rational coefficient remapping or conversion of
the restored handle into `CompiledBody`/`PreparedBody`. Add this native-template construction
entry beside the existing producer-side source-bearing entry, sharing the same exporter,
compiler process and module owner rather than maintaining a second compiler implementation.

The local module key qualifies the canonical numerical request and payload digest plus native
compiler/ABI/ISA/FP profile and known-head implementation versions. The receiving operation
owns source text, compiler work, finalized module and private loaded workers through completion;
the old producer's exact evaluator or shared-object path is unnecessary. This package does not
transfer native binaries. Native artifact transfer would need its own qualified contract and
is not an implicit shortcut. The caller selects interpreted versus native execution before
effects; a native construction failure is reported rather than silently replaying source or
retrying another profile. Receiving numerical mathematics grants no later symbolic eligibility.

Migrate the C02 authored caloric and magnitude consumer selections plus affected scalar/native
profile dispatch; delete their alternate inlined `log(1+x)`, naive square-norm implementations
or unsupported-head bypasses where replaced. Accept independent Value/First/Second expectations
on the same tiny/extreme/domain cases, after actual compilation and after cold acquisition.
Include a known-head version mismatch, opaque callback refusal, failed compiler and cancellation
before final module publication with child drain. In the cold case make producer exact/body
preparation unavailable and require B05 acquisition followed by numerical-stage compilation and
the actual observation consumer; this reveals an accidental dependency on `ScalarProducts` or
`AdmittedBody`. Reuse existing native lifecycle tests against the extended owner instead of
creating a separate stable-operation compilation subsystem.

<a id="C04"></a>

### C04 — Deliver the ordered batch contract and scalar baseline

Implement one batch operation at the existing prepared mathematical product/worker boundary,
usable by native and workflow callers. Its input is a qualified prepared product, selected
profile, ordered point identities, point-major input values, requested output/axis/pair/action
layout and per-point runtime directions/cotangents/weights where demanded. Its output is the
same ordered point population with typed per-point success, original-domain/derivative failure,
numerical failure or canceled/not-executed outcome, plus only the demanded numerical cells.
Conceptual `BatchRequest`/`BatchOutcome` names do not require another service or registry.

Fix the scalar buffer contract: for `n` points, `p` admitted inputs and `k` requested product
cells, `inputs[row*p+input]` and `outputs[row*k+cell]` are point-major. `cell` order is the
existing selected output order followed by its explicitly qualified
[`JetLayout`/`ProductKind`](../../crates/pse-math/src/jets.rs) coordinates/pairs or action
entries; it is not SIMD lane order. Point-dependent seeds/weights have their own ordered
stride and semantic map and are runtime data when shape is unchanged. Validate all checked
length products, axis/pair correspondence and empty/zero-input layouts before work. A repeated
point retains a separate identity/outcome. Empty input returns an empty complete population;
a constant zero-parameter program can still produce `n` requested rows.

Use A02's working demanded-product owner and existing
[`PreparedBody`/private worker](../../crates/pse-math/src/execution.rs) lifecycle. Immutable
programs are shared once; mutable evaluator/packing/output scratch belongs to each worker and
is reused across its calls. No worker shares mutable stacks with another. Initially execute
the explicit scalar realization against this same contract. Scalar is also an admitted
selection for small/unsupported packed work; select it before effects rather than retry a
failed packed attempt. Per-point source-domain/derivative admission precedes pure evaluation;
retain source identity on failures and never replace an invalid point with a zero-filled
success. A failed point can coexist with successful independent points; operation cancellation
marks remaining work and prevents a stale complete-success outcome.

Readiness is A02's implemented preparation/worker interface for the selected pure demand, not
all numerical consumers that A02 eventually migrates. Reconcile this early interface with A01
and C01 without requiring finished C05. Migrate the initial repeated pure-property/residual
caller to this scalar batch path so the contract is usable before SIMD. Remove that caller's
duplicate input/output traversal and per-point preparation where the new owner absorbs them;
keep scientific point admission and independent solver iterations outside the pure batch.

Acceptance uses multiple/reordered outputs, selected partials and a direct action, changed
runtime seeds without shape rebuild, mismatched axes, empty/repeated points, constants, a single
failed domain point, cancellation between points and concurrent private workers. Retain A's
finite direct weighted-product countercase; a batch cannot force unrequested intermediate
Second products to overflow. Independent small arithmetic/analytic answers establish row/cell
mapping. Source inspection verifies removal of repeated preparation, without requiring a timing
benchmark to measure that structural change.

<a id="C05"></a>

### C05 — Add x4 C++ execution with native packing and safe tails

Implement the explicit x4 C++ realization of C04 using B02/B03 checked construction/export
and C03's stable primitive/native lifecycle. Inputs and outputs keep C04's point-major layout;
the supplier owns conversion between scalar rows and independent x4 lanes. Taylor components
are already selected program cells before lane packing. Map coefficients uniformly to packed
constants; point-varying inputs/seeds enter input buffers, never per-lane compiled constants.

Use maintained 3.0.1
[`export_cpp::<wide::f64x4>`/`export_simd_str`](../../vendor/symbolica/src/evaluate/export.rs),
[`CompiledSimdRealEvaluator` and `BatchEvaluator<f64>::evaluate_batch`](../../vendor/symbolica/src/evaluate/backend.rs).
That backend retains input/output buffers and scatters scalar rows with
`output_buffer[(row/4)*n_out+column][row%4]`. Extend the current native module owner to retain
profile-qualified x4 templates and private loaded workers; do not create application SIMD
packing loops or an independent compiler/module cache. B03 must cover generated xsimd headers,
stable external derivatives and transient source overlap before exporting.

Admit only pure branch-free regions with supported stable operations and qualified integer-power
behavior. `EvaluationDomain` packed callback fallback can call a scalar callback per lane;
it is not the selected stable x4 implementation. Supply actual x4 stable kernels/export
definitions, including extreme-scale local derivatives, and explicitly qualify the instruction
power semantics for negative bases, signed zero and subnormals. Whole-vector `is_fully_zero`
is not a lane mask; mixed-lane lazy branches and effectful providers remain ineligible. Use
scalar per-point admission first and retain an explicit gather/scatter identity map if failed
points are removed from a pure block. That application map is scientific population handling,
not a duplicate scalar-to-lane adapter.

Select a scalar tail for the last one to three eligible points. Pass only full groups of four
to the supplier packed adapter, eliminating its default zero-padding callbacks on invalid
inactive lanes. C03's compatible scalar module handles the tail under the same original
operation; tail selection is made before executing the batch, not as failure fallback. Segment
long full-group calls at documented cancellation safe points, retaining buffers across segments.
This bounds cooperative polling work without introducing a host CPU partition or serializing
independent workers. Allocate/grow scratch under current ownership before native calls and keep
charges through actual completion/drain.

Acceptance uses populations of 0, 1, 3, 4, 5 and multiple groups with at least two outputs and
selected derivative/action cells, reordered axes, constants/zero inputs, one refused point,
negative-base powers and domain-sensitive heads that would fail on zero padding. Verify every
row/cell against independent expected values and expose the scalar-tail decision. Exercise
repeated worker calls, concurrent workers and cancellation between segments. Delete displaced
application packing/scattering/native source predictors only after these cases pass; preserve
scalar realization and original science checks as required profiles, not legacy fallback code.

<a id="C06"></a>

### C06 — Migrate repeated science and remove redundant preparation

Move actual repeated property/pure-residual populations and a fitting/study consumer onto the
same prepared batch boundary. The first property population is the selected authored caloric
law from C02 evaluated at ordered temperatures/case inputs, with source/domain/applicability
reported per point. Fitting integration uses
[`PreparedExperiments::finish`](../../crates/pse-runtime/src/workflow/fitting/preparation.rs)
and the existing mathematical
[assembly owner](../../crates/pse-math/src/assembly.rs) to reuse admitted residual/response
products across compatible experiment evaluations; it does not batch unrelated solver state,
merge experiment identities or replace fitting iteration.

Use [study execution](../../crates/pse-runtime/src/workflow/study_execution.rs) and
[demand selection](../../crates/pse-runtime/src/workflow/study_demand.rs) for the study-facing
integration. A study point still owns its declared workflow, failure and result. Only compatible
pure mathematical populations within that execution use C04/C05. Provider effects, lazy branches,
different guard/layout/profile/function semantics and scientifically sequential dependencies
remain separate. F consumes this batch capability for expectation integrands but owns measures,
estimands and statistical completion; it is not the sole migration boundary for this package.

At the actual callers, prepare definitions/demand once and bind changing point values and runtime
seeds. Transport original output and both derivative-axis coordinates using A's mapped bindings.
Delete newly superseded per-output/per-point exact reconstruction, local derivative setup,
optimizer/native dispatch and scalar-lane packing loops at those callers. Do not claim every
current point prepare is redundant: different scientific specifications, provider state or
solver starts may legitimately need their own preparation. Retire a helper/configuration/fixture
only when all affected callers have moved; preserve independent ordinary science tests and
explicitly selected scalar execution for unsupported packed regions.

Readiness is working C02–C05 capabilities and the relevant A binding transport, not completion
of F or every old study/fit package. Deliver the caller migration together with its declaration,
result mapping, protected reporting and deletion. Acceptance runs an ordinary property table,
a repeated residual/fit population and an ordered study population with independent expected
science. Include value-only repeat reuse, changed guard/layout invalidation, reordered physical
instances, one failed point and cancellation. Verify compatible repeats do not reconstruct the
identified programs using existing preparation observations/source inspection; no new generic
utilization service or mandatory speed benchmark is required.

<a id="C07"></a>

### C07 — Close additional realization decisions against named workloads

Use C06's actual property/residual population and C04's fixed request/result semantics to settle
the additional backend leads. Deliver either a complete selected realization integrated into
the same preparation/profile/result owner, or a recorded no-adoption decision with the actual
failed criterion and observable reopen trigger. This package does not defer the selected CPU
scalar/x4 target or authorize a framework containing unused backends. H01 supplies dependency
closure/build observations when those affect the choice; C supplies operation-level semantics.

For packed interpretation, compare `ExpressionEvaluator::map_coeff` to supported x4/x8 plus
`try_evaluate`, using the same stable function/profile/branch restrictions. Maintained 3.0.1
[domain](../../vendor/symbolica/src/evaluate/domain.rs) and
[evaluator](../../vendor/symbolica/src/evaluate/evaluator.rs) sources show that interpreter
packing is application work absent a maintained supplier adapter. Adopt only if lower startup
and useful actual vocabulary justify that complete adapter and its ownership; otherwise retain
the existing explicit scalar/x4 C++ selection. A selected adapter should live in the supplier
alongside native packing rather than add a competing consumer-specific loop.

For JIT, inspect/repair the real x4 multi-output scalar-buffer adapter before selection. The
[current backend](../../vendor/symbolica/src/evaluate/backend.rs) and
[exact source account](../library_leverage/libraries/symbolica/packed-evaluator-contract.md)
show a destination chunk of `4*n_out` copied from one four-lane vector, which is unsound for
a full group with multiple outputs; this is source evidence, not a runtime reproduction from
plan authoring. A selected JIT profile must supply correct row/column/lane scattering, empty
and zero-parameter behavior, scalar tails, stable head support and the conservative FP/ISA
contract. Do not adopt it merely because compilation is in-process.

For CUDA/device execution, require suitable prepared hardware and a named large pure population
whose transfer, compilation, resident-buffer and cancellation lifetimes justify the complete
route. Preserve original per-point eligibility and compare its numerical profile independently;
device execution does not accelerate symbolic preparation by itself. For fixed generated duals,
require an actually stable repeated kernel shape and H's monomorphization/dependency assessment;
otherwise retain dynamic demand shapes. Do not multiply fixed types across arbitrary runtime
axes. Alternate arithmetic and Python/WASM deployment decisions remain H-owned.

Any adopted path includes checked construction, active native worker/module or device custody,
producer/receiver profile qualification where used, migrated callers, independent numerical
acceptance and deletion of the superseded realization/adapter. Explicit no-adoption outcomes
name what would reopen them, such as a supported low-startup vocabulary, repaired multi-output
contract, prepared device workload or stable specialization shape. They are not missing required
CPU delivery or unbounded future research tasks. Hand actual supported profiles and evidence to H
for assembled qualification after functional implementation.

## Verification

**Proposed:** scoped compilation and focused math/compiler/runtime tests through `pse`, with explicit Arrow validation as needed. Independent analytic/exact/high-precision expectations cover stable values/derivatives, extreme scales, original-domain failures, ordered numerical populations, power/profile behavior, scratch and cancellation. Scalar-vs-packed self-comparison alone cannot establish a shared transformation's correctness. H owns assembled property/fitting/study/native/receiver journeys after functional scope. No tests, compilation or performance measurement occurred in authoring; expected less repeated preparation/dispatch is qualitative.

## Execution checkpoint — 2026-11-10

Execution of the integrated A/B/C scope began at `d2302f32a` with a clean checkout. The selected first assignments are shared transport consolidation, checked supplier construction and stable primitives. A01/B01/C01 are coordinated prerequisite contracts; A03 proceeds within existing binding semantics. Required new kernel/primitive decisions precede dependent implementation. All other scope remains open until its actual production cutover and targeted evidence; broader Plan73/Plan66 completion is not claimed.


## Scalar and ordered-population checkpoint — 2026-11-10

**Implemented.** C02 now has fixed stable `log1p`, magnitude and positive-argument `log_ratio`
heads with exact selected first/second channels. Source quantity admission, original guard
order and occurrence attribution remain authoritative. The actual authored DIPPR100 entropy
consumer uses `log_ratio(T,T_reference)`: near-reference increments use `ln1p`, while far
positive temperatures retain the original positive-ratio/log-difference semantics. A global
`ln1p((T-T_reference)/T_reference)` replacement was rejected because it loses valid far-range
inputs. ADR-0263 records this correction. Registration is atomic across the fixed head group;
foreign preclaims cannot leave a partially registered operation vocabulary.

**Tested.** The scoped compiled-evaluator stable/batch/shared-function selection passed 18/18;
the two excluded selected-Taylor controls passed with the supplier's 5/5 selection. Actual
authored stable/caloric compiler controls passed 2/2, typed temperature-log-ratio admission
passed 1/1 and isolated registration-preclaim controls passed 2/2. All results have zero
failures against baseline zero. Independent analytic/high-precision expectations cover tiny
increments, extreme/reversed ratios, mixed Second axes and original physical validity.

**Implemented.** C04 supplies a sealed ordered batch layout, private reusable numeric scratch,
point-major admitted results and separate success/failure/canceled/not-executed outcomes.
Actual `ModelingPackage::observe_batch` uses the compiler observation program and runtime
job owner, including multiple physical instances and complete original input validation.
Repeated point IDs remain repeated outcomes; failed cells are inaccessible. The focused
runtime property-population tests passed 2/2, zero failures against baseline zero, in the
default Nextest profile. Required x4 integration remains open under C05/C06.

**Implemented, Tested — adjacent C06 correction.** Study outcome pages now derive a dedicated
wire-depth allowance from the generated native-value depth contract and fixed endpoint
wrappers. Ordinary StudyPage/capture-locator profiles retain their distinct bounds; row,
byte, node, string and blob population limits are unchanged. Four targeted operations
response/bootstrap controls passed 4/4 with zero failures against baseline zero, in the
default Nextest profile. Full `just codegen` publishes the declaration and generated users.
The native Ipopt Study-summary retry remains to be run after host preparation.

C01/C03/C05 remain open for complete compiled/receiving/x4 profile delivery. C06 requires its
remaining actual fitting/study integrations; C07's bounded additional-backend decisions remain
open. This checkpoint does not close the integrated A/B/C scope or H qualification.


## Additional backend decisions — 2026-11-10

**Interface-checked — C07 complete, no additional backend adopted.** Retain the existing
scalar interpreter for low-startup requests and the required explicit scalar/x4 C++ profiles
for repeated pure populations. These are bounded choices for the current property, residual
and fitting/study responsibilities, not claims that another backend could never be faster.
No timing measurement or new benchmark obligation is attached to them.

- **Packed interpretation.** `ExpressionEvaluator::map_coeff` and Numerica's x4/x8 numeric
  domains supply useful arithmetic building blocks, but the current stable registrations are
  binary64 callbacks. The packed `EvaluationDomain` fallback calls those scalar callbacks per
  lane. The interpreter also tests whole-vector zero for its branch instructions, and the
  installed packed integer `Pow` route uses `powf_simd`. A complete alternative would need
  genuine packed head implementations, qualified powers and branch restrictions, checked
  coefficient conversion and a maintained row/lane/tail adapter. The default interpreter
  already serves small tables without compilation; required C++ x4 supplies prepared repeated
  execution. Reopen for a named startup-sensitive repeated pure population whose useful SIMD
  vocabulary and maintained adapter justify that extra realization.
- **JIT.** The current Symbolica x4 multi-output adapter copies one four-lane result into a
  `4*n_out` destination chunk. The actual joint caloric/property vocabulary has multiple
  outputs, so this fails the required scatter contract; empty and zero-input populations have
  separate division/chunk hazards. Symjit's default configuration additionally consults
  `SYMJIT_TOML` or `symjit.toml` and otherwise enables native/fast-math behavior. The existing
  application does not qualify that ambient configuration, checked construction, stable-head
  vocabulary or module lifetime for this route. Symjit is already in the dependency closure;
  dependency cost is not the deciding objection. Reopen after the shared adapter handles
  multiple outputs, empty/constants and scalar tails, with explicit conservative configuration,
  true packed heads and admitted lifetime, for a named cold-compilation need.
- **CUDA.** Prepared execution capability and a useful resident workload are the missing
  premises. Hardware is present: the local query observed an RTX5090 with compute capability
  12.0 and about32GiB, and nvcc13.5. The current application capability preparations have no
  qualified CUDA execution profile, checked device-head exporter or resident-buffer/transfer/
  cancellation receiving owner. The small property and native controls establish numerical
  semantics, not large-population suitability. Reopen for an actual large repeated resident
  property/study population with those explicit device and custody contracts. GPU availability
  alone is insufficient to select this route.
- **Fixed generated duals.** `JetLayout` coordinates, shapes and symmetric pairs are runtime
  selections. Current `HyperDual(shape)` construction generates ordinary scalar numerical
  instructions, which are then mapped to binary64; fixed dual types would primarily change
  preparation, rather than directly accelerate the row worker. Retained products already
  remove repeated compatible preparation. Reopen for an authoritative repeatedly cold-prepared
  fixed kernel shape, with H's monomorphization/dependency assessment, without multiplying
  concrete types over arbitrary fitting/property axes.

The maintained implementation sources are
[packed domains](../../vendor/symbolica/src/evaluate/domain.rs),
[interpreter](../../vendor/symbolica/src/evaluate/evaluator.rs),
[backend adapters](../../vendor/symbolica/src/evaluate/backend.rs), and
[current derivative layouts](../../crates/pse-math/src/jets.rs). The source-backed investigation
also inspected the installed Numerica and Symjit implementation, plus the actual local hardware
and preparation inventory. No JIT, packed-interpreter or GPU execution/performance claim is
made. Required C03/C05 receiving and CPU delivery, and C06's consumer migration, retain their
own acceptance obligations.

## Compiled population and consumer checkpoint — 2026-11-10

**Implemented — C03/C05 prerequisites.** The existing native module owner now supports an
explicit conservative AVX2 x4 profile with a scalar entry in the same module. Supplier
`try_evaluate_batch_with` owns actual row/lane packing, scatter and retained lane buffers;
application scheduling removes scientifically refused rows, preserves their identities and
issues only full groups of four. The compatible scalar entry executes tails of one to three
eligible rows. Long populations have cancellation safe points every 256 rows, preserving the
original operation rather than renewing it per segment. Native batch errors retain their
typed causes. The shared `CheckedBatchExecution` owns the ordinary `NumericFrame` directly,
so restored numerical programs and admitted source programs use one reusable worker mechanism.

One Cargo metadata declaration pins the xsimd14.3.0 archive and digest. The mathematical
build owner authenticates its source extent; the module owner installs and retains the
verified headers through compiler/library drain. Native execution requires no source checkout
or network lookup. Stable operations have actual packed implementations, including scaled
derivatives, ratio endpoints and subnormal logarithm normalization. Generic packed `log` uses
the same corrected subnormal contract. This is a maintained source profile, without modifying
the pinned xsimd archive. Scalar and packed exports are authenticated independently before a
module can publish success.

**Tested.** The true packed/source batch selection passed 7/7 with `compiled-evaluator` and
zero failures against baseline zero. It includes populations0/1/3/4/5/260, reordered multi-output
Second cells, scalar tails, failed-point exclusion and cancellation after a full segment.
The source-backed reusable-worker selection separately passed 7/7: one-byte-short admission
refuses before worker construction; repeated6/3/0/6 calls retain result backing and require
no further construction growth after preclaim. A compiled C++ control of all16 stable heads
plus generic packed logarithm passed 1/1 against independent long-double/high-precision
expectations. These results cover the named CPU realizations, not a timing speedup or every
action profile. True packed direct-action/point-varying-seed coverage remains open in C05.

**Implemented — C06 integration.** Pure fitting residual/response instances now share ordered
cross-case frontier waves through `CrossCasePopulation`. Grouping requires identical actual
program Arcs, demanded order/layout and pure supported stages. A guard failure stops later
instances for that experiment; independent experiments retain their own order, failure and
result. `PreparedExperiments::finish`, serial fitting oracles and the existing assigned
parallel lanes consume this population. Eligible per-case constraint/Jacobian workers are
removed. Provider/branch and distinct weighted-curvature work retain their separate scientific
capabilities. Team placement and CPU width remain authoritative.

**Tested.** The focused math population/source-free/batch selection passed 6/6 in the default
profile, zero failures against baseline zero. It covers five distinct original cases sharing
programs, reordered aliases, two frontier waves, independent values/Jacobians and first-versus-
second-instance failures with no accessible partial successful row. Runtime fitting integration,
an ordered declared fitting Study, the actual authored DIPPR property table and production
canonical/native receiving are under test. Partial start annotations use an explicit internal
source-observation capability because their target variables do not yet have values; public
complete observations retain full input validation. C03/C05/C06 remain in progress until
their remaining actual consumer/receiving and action checks pass. H qualification remains
separate, and no repo-wide health checks are requested by this checkpoint.

**Implemented.** Full `just codegen` completed successfully for all16 targets after the x4
profile/frame changes. Generated engineering crossings, native projections and schemas are
published together with their declarations. This establishes current generated products;
the active numerical/consumer tests retain their own verdicts.

## Authored science, actions and fitting Study acceptance — 2026-11-10

**Implemented, Tested — C03/C06.** The actual authored DIPPR100 heat/entropy table executes
through interpreted and x4 profiles over eight original temperatures, including adjacent
floating-point values around the reference temperature and the original zero-temperature
refusal. Repeated point IDs and repeated calls retain their original rows and reuse preparation.
Independent heat/entropy answers cover both the reference difference and temperature ratio.
Its original scientific domain predicate contains a lazy branch. The packed eligibility gate
was corrected to permit those scalar, effect-only domain/applicability predicates before
pure lane execution, while continuing to refuse numeric output branches and providers.
The actual cold numerical receiver also passed all three execution profiles after producer
exit; 73b owns the receiving evidence and source-authority distinction.

**Implemented, Tested — C05.** Packed direct Jv, transposed-Jacobian, weighted-Hessian and
Hessian-vector actions use ordinary runtime seed/weight inputs, shared scalar/action
binding and actual supplier lane packing. Failed rows retain original IDs and are excluded
before gathering. Selected products do not request an unnecessary Second family, and a zero
weight does not erase the original domain guard. The command
`scripts/pse test pse-math --lib --features compiled-evaluator -E 'test(ordered_x4_action_) | test(ordered_x4_direct_) | test(ordered_x4_scalar_scientific_)'`
passed **8/8**, with zero warnings and zero failures against baseline zero. This includes the
combined four-products/one-operation control, changing seeds and weights, scalar tails,
finite selected curvature, lazy inactive-log predicates and original cancellation before
entry and after a complete 256-row segment. An earlier combined action invocation timed out;
the split controls and the original combined control subsequently passed without increasing
the timeout. No cause or timing improvement is claimed for that observation.

**Implemented, Tested — C06.** An actual declared ordered fitting Study uses the production
fitting/population path with point order90/10/70, independent fitted-parameter and prediction
answers, original run IDs, a structured original child-binding Domain failure, dependent
point disposition and pre-entry cancellation without an attempt/result. The command
`scripts/pse test pse-runtime --lib --features compiled-evaluator,solver-ipopt -E 'test(=workflow::study_operations::study_population_tests::ordered_study_fitting_population_keeps_original_points_failures_and_cancellation)'`
passed **1/1**, zero warnings and zero failures against baseline zero. The fixture explicitly
selects stationarity/complementarity tolerances consistent with its numerical assertions;
the earlier default KKT tolerance did not justify that assertion. The control now belongs to
the actual `solver-ipopt` test module rather than a broader hidden `native-solvers` module.

The four-test runtime selection recorded in 73b passed the authored caloric, cold numerical
and symbolic continuation cases and failed the initial Study spelling. The corrected Study
above is the affected rerun. C03/C05/C06 retain final checked-construction integration and
zero-input constant-call controls before their package state closes. No general workspace
health check, exhaustive solver/backend qualification or measured speedup is claimed.
