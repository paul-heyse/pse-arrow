---
title: Symbolica mathematical execution
status: in-progress
date: 2026-10-21
adrs: []
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, libraries, numerical-execution, preparation, validation]
---

# Symbolica mathematical execution

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

Current proposed revisions: [the 2026-10-22 reassessment](49-library-native-remediation.md#remaining-scope-reassessment--2026-10-22).
Production execution remains paused; original authoring baselines and receipts retain their dates.

This companion develops the mathematical execution target of
[Plan 49](49-library-native-remediation.md). The coordinator owns source dispositions,
rule decisions, **all package status** and combined qualification. The target below is
**Proposed**, conditional on those decisions; it is complete enough to implement without
another general library-feasibility exercise. Preparation contracts come from
[49c](49c-resolved-model-and-incremental-preparation.md). This document owns arithmetic-region
construction, demanded derivative products, evaluator ownership and their acceptance.

The inspected baseline is HEAD `e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`, with the authorized
dirty [integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md)
and [audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
The entire [Symbolica assessment](../libraries/symbolica.md), original preparation inventory
and probe A/B evidence informed the design. Feature followups used its linked official
documentation and exact installed Symbolica/Numerica 3.0.1 APIs. No probe, product build,
test or formatter was run to author this proposal. Core 3.4, Efficient Architecture 1.0 and
Process Simulator 1.5 guide the choices; blueprint §7 and §14 retain current authority.

## Current mathematical foundation and revised input boundary

**Implemented/source-inspected, 2026-10-22:** regional construction, demand-specific products,
provider/branch composition and interpreted/C++ consumers have focused evidence recorded by
the coordinator. MAT01–MAT04 still retain full consumer/retirement obligations and MAT05 owns
scientific acceptance. I11–I13/I25 preserve their recorded scoped no-additional-adoption choices;
this reassessment does not restart JIT, SIMD/CUDA, exact-algebra or hot-start campaigns.

Review [O05](../functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md#O05)
changes the preparation producer feeding these products, not mathematical authority. Consume
PRE08 immutable resolved entity/region descriptions and complete body premises. Preserve
per-instance validity/source maps even when mathematical fields compare equal. Native
construction, optimizer abort outcomes, mutable evaluators and compilation/drain remain outside
Salsa tracked queries. Existing canonical body reuse stays; the new opportunity concerns
root specialization/projection work. Targeted joins exercise changed membership, physical/
provider premises and source-only attribution without weakening original derivative/guard checks.

PRE09's optional configuration exploration uses capability facts; its logical feasibility
cannot certify a mathematical branch, physical domain, derivatives or numerical solution.
No new Symbolica/Numerica integration was selected by the adjacent capability screening.

## Selected mathematical boundary

Project-owned admission establishes identities, physical contracts, source occurrences,
original-domain obligations, finite membership and provider/branch selection before producing
library atoms. Thereafter Symbolica owns expressions, normalization, exact algebraic operations,
symbolic derivatives, Horner/CSE preparation and numerical evaluator instructions. Numerica
supplies the selected number/jet domains. The project composes those capabilities and retains
host control for ordered `Require`, `Domain`, `Applicability`, lazy `Branch` and fallible
`Provider` stages. It does not introduce another arithmetic interpreter or an optimizer over
a project-owned instruction language.

The execution unit is one **demanded multi-output arithmetic region per `ProductKind`** within
a reusable specialization body. A region spans dependent arithmetic lets where no host stage
intervenes. It ends before a host stage and resumes afterward using that stage's explicit
outputs as region inputs. Unrelated instances do not form a whole-model union-coordinate
evaluator. The compiler supplies canonical local input roles plus per-instance bindings,
validity, consumed physical premises and source mappings; a group hash alone is insufficient.

`PreparedBody::demanded_stages` remains the authority for the computation actually requested.
Build residual/Jacobian demand separately from observation-only terms and contributions.
`SelectedPartials`, `JacobianAction`, `TransposeJacobianAction`, `WeightedHessian` and
`WeightedHessianAction` retain their existing consumed support and runtime seed/weight meaning.
An NLP needs the consumed weighted Hessian, not an automatic full Hessian for every output.
Building all-coordinate dual components because one output needs them is outside this target.

## Original authoring foundations and corrected evidence

At the original 2026-10-21 authoring baseline, `pse-math::guarded::Stage` already separated
arithmetic from host barriers. `execution.rs`
contains `SelectedFirstRegion`, but it currently requires region inputs to be original scalar
inputs. `products.rs` supplies direct symbolic products only for eligible flat arithmetic.
`arithmetic_region` stops when the next block reads an output already produced. Those existing
routes are the extension foundation, not code to discard before a replacement is established.

`PreparedBody::expression` is not a guard-preserving executable representation: flattening can
erase `Domain`/`Applicability` tokens and order-qualified `Require`. R7 therefore operates on
the stage program. The original BT_PR probe A compared flattened obligation-free variants;
177/292/750 operation totals for joint symbolic value/first/second products establish only
a guard-free construction observation. Nothing was evaluated, and the second-order variant
was a full upper Hessian triangle. It neither qualifies this regional target nor predicts its
latency. Most original cost was in non-substitutable provider/branch bodies, which must be
reached through safe arithmetic fusion and chain composition rather than flattened away.

Exact pinned interfaces correct several tempting assumptions in the broad assessment:

| Pinned surface | Design consequence |
|---|---|
| `AtomCore::derivative`, `Atom::evaluator_multiple`; `evaluate/function_map.rs::FunctionMap` | Build requested symbolic products, then optimize related outputs jointly |
| `FunctionMap::add_aliases`, builder `.function_map(...)`, `.direct_translation(true)` | Retain a library-owned arithmetic DAG without repeatedly materializing named let expressions |
| `InliningPolicy::Auto` currently equals `Always` | Choose `Always` explicitly for function definitions; no adaptive inlining policy is assumed |
| Rust `ExpressionEvaluator::vectorize(&Dualizer)` | Keep sparse local Taylor/jet machinery where selected; never infer a Rust `.dualize` call from Python prose |
| `vectorize` refuses external bodies/sub-evaluators | Do not choose `InliningPolicy::Never` as a sharing trick for vectorized products |
| Rust `.export_instructions() -> ExportedInstructions<T>` | Use the existing exported instructions for accounting and native preparation; Python `get_instructions` is a different API |
| `ProgramView` and `.program()` are `pub(super)` | No public immutable interpreted-program/private-scratch split exists in this release; interpreted evaluator clones carry their real retained cost |
| Cargo.lock resolves SymJIT 2.26.0 | A future JIT comparison uses this resolved API and actual numerical semantics, not the assessment's 2.25 compatibility floor |

The precise Rust documentation is [Symbolica 3.0.1](https://docs.rs/symbolica/3.0.1/symbolica/),
with the [evaluation guide](https://symbolica.io/docs/numerical_evaluation.html) and
[AD guide](https://symbolica.io/docs/automatic_differentiation.html) as entry points.
The installed `evaluate/{function_map,dual,instruction,evaluator}.rs` settles the distinctions
above. These are **Interface-checked** facts, not evidence of product integration.

## Realizing regions and derivative demand

Lower admitted arithmetic once to `Atom`/`AtomView` and a `FunctionMap` of local definitions.
Use `add_aliases` for argument-free intermediates that expand in the caller's local scope;
use `add_function_with_options(..., FunctionRegistrationOptions::new().inlining(Always))`
for authored functions with explicit argument scope. Register definitions once per retained
region construction rather than reconstructing them per instance/output. Build all demanded
region outputs together with `Atom::evaluator_multiple(...).function_map(map)` and explicit
optimizer bounds. Select `direct_translation(true)`, already the pinned default, to avoid
the tree route's repeated alias expansion before CSE. The gain comes from the representation
and fused demand, not toggling a currently false flag.

Alias registration affects evaluator construction; it does **not** teach a preceding
`AtomCore::derivative` call what an alias means. Derivative preparation therefore operates on
the resolved local dependency DAG explicitly. Generate local partial expressions for each
definition with Symbolica `derivative` over its actual inputs, name those derivative definitions
in the same library function/alias map, and compose them with the existing Symbolica-generated
jet/action layouts. Numeric seeds, cotangents and weights stay ordinary runtime parameters.
The project supplies dependency/layout wiring; Symbolica/Numerica perform arithmetic and
derivative composition. Do not add a new symbolic rewrite engine or a numerical AD interpreter.

For a region downstream of a provider or branch, differentiate with respect to **region inputs**,
including intermediate outputs. Their derivatives are not zero. If `z=g(x)` is the host/provider
output and `y=f(x,z)`, first order includes `f_x + f_z g_x`; mixed second order includes both
cross terms, `f_zz g_i g_j` and `f_z g_ij`. Feed those local jets/actions into the existing
outer Taylor/reverse composition. The symbolic route must preserve these terms at arbitrary
supported intermediate depth, not merely pass a scalar-input fixture.

Construct selected partials only for reached coordinate support. Construct weighted products
from the requested output sum and demanded support pairs; use directional/adjoint seeds to
avoid materializing an unconsumed dense Jacobian/Hessian. Local derivatives of aliases and
their dependencies participate in joint CSE. Retain existing Taylor/reverse composition for
preflight-ineligible regions; it is a declared route of the target while replacements are
established, not a silent arithmetic fallback. Once a regional replacement is proven for its
eligibility domain, migrate every reached caller and delete the superseded path for that domain.

Preserve each original host stage and its order. A positive/nonzero condition still protects
the original argument after cancellation such as `x/x -> 1`; `Require` runs at its declared
derivative order. Lazy branches evaluate only the selected arm. Do not replace branch stages
with eager library `if` or lift fallible providers into infallible evaluator callbacks. Domain
and applicability evidence retain their lineage, source and actual demanded path.

Fusion retains source checks for eliminated intermediate producers. A later final-output
finiteness check cannot replace an intermediate source check if its disappearance would alter
typed failure behavior. Preserve a check output or split the arithmetic region where required;
the same demanded source/check support informs accounting. This bounds fusion by scientific
and failure semantics without forcing every authored let to be a physical evaluator boundary.

## Deterministic construction and physical ownership

Add a deterministic preflight over the admitted stage/definition DAG before differentiating.
It selects the eligible regional route from product kind, demanded support, derivative order,
primitive/provider contracts and conservative construction/operation/scratch bounds. The
selection and relevant profile/limit premises enter product identity. Do not estimate safety
from `count_operations` alone: the current code notes omitted `Fun` nodes, and construction
can swell before a final optimized census exists. Retain explicit function/alias dependency
and derivative-call bounds and validate actual exported instructions afterward.

Current `selected_partial_calls` is decremented inside differentiation and
`construction::additional` can refuse transient `try_grow`; neither is the missing deterministic
preflight. After selection, cancellation, exhausted runtime capacity or a library error returns
its typed refusal. It cannot pick Taylor, shorten Horner search and retain the result, or select
another backend under the same key. Check cancellation before and after construction, including
successful `.build()` after the optimizer's abort callback fired; discard the truncated result
before any retention/publication. Keep producer cancellation detached from retained evaluator
settings and preserve actual compiler/native worker drain.

Use `map_coeff` at the declared exact-to-binary64 boundary and the current real-arithmetic
profile. Preserve exact integer/rational exponent facts and original physical unit conversion;
do not promote all binary64 literals into guessed rationals or grant new realness assumptions
to simplify bodies. Algebraic equality is over the admitted real domain, not floating-point
bit identity. Original residual/domain/closure qualification remains an independent consumer.

The interpreted route uses the public library evaluator with worker-owned mutable clones;
account for retained instructions as well as numerical scratch where the release clones both.
`ExportedInstructions` supports census and native export, not a project interpreter introduced
to manufacture a nonexistent library ownership API. Immutable admitted source/region descriptions
and instance mappings can still share by `Arc` without claiming interpreted code sharing.

Keep the existing explicitly selected C++ scalar route. `native_compiled.rs::Template` retains
shared code and temporary-directory lifetime; workers call `load_new_function` to obtain
private library-owned evaluator scratch. Preserve `-fno-fast-math`,
`-fno-unsafe-math-optimizations`, `-ffp-contract=off`, generic x86-64, module capacity,
bounded diagnostics and process-group cancellation/drain. Alias/function fusion must preserve
the current native producer boundary: all definitions are inlined, exported products have no
opaque callbacks or sub-evaluators, and the compiler export bound covers the resulting code.
Compiled outputs that do not meet this explicit profile refuse its selection honestly.

JIT is a separate bounded comparison, not the chosen replacement for C++. SymJIT 2.26.0
through `jit_compile(JITCompilationSettings::new().direct_translation(true))` must satisfy
contraction/FMA policy, target reproducibility, owned code/scratch/resource lifetime and typed
cancellation behavior before adoption. Compare total construction plus repeated evaluation
for actual demanded products. A missing policy control or unaccounted lifetime means retain
the selected C++ route; it does not block regional Symbolica construction.

## Broader capability choices

The whole library assessment is a design premise. Its breadth does not make every advertised
family a new product requirement. The following choices place relevant native opportunities
at concrete mathematical owners and preserve their semantic limits.

| Family | Selected use or bounded question and decision consequence |
|---|---|
| Exact integers/rationals and polynomial/rational domains | Preserve existing exact literal/exponent facts. MAT06 assesses `to_polynomial`, `RationalPolynomial` and exact coefficient domains for reached coefficient/factorable extraction, replacing bespoke algebra where equivalent. Cancellation never removes original excluded points. No generic exact solve replaces numerical execution. |
| `SolutionSet`, algebraic roots and extensions | MAT06 examines an existing implicit affine/polynomial selection use only. Any adoption preserves real domain, root embedding, generic versus complete coverage and `coverage_guard`; unsupported/incomplete is distinct from empty. A mere numeric root is not a selected-function certificate. |
| `Transformer`/pattern matching | Prefer reusable native transformations for a repeated mathematical normalization currently implemented locally. MAT06 compares a bounded reached transformation; rule preconditions, unknown properties and termination remain explicit. No unconstrained expand/repeat pass becomes universal preprocessing. |
| `set_coefficient_ring` | MAT06 tests parameter-coefficient grouping on a reached symbolic coefficient workload. Parameters hidden inside coefficients must remain differentiable when demanded; reject the conversion if it obscures consumed dependence or complicates identity more than it removes work. |
| `TermStreamer`/`TermStreamerConfig` | Not selected for ordinary evaluator construction. MAT06 records whether an actual large term-local derivative/transformation can spill with finite `max_mem_bytes`, bounded path/core ownership and cancellation cleanup. `normalize` still sorts/merges globally; it cannot stream arbitrary Horner/factorization work. |
| Numerica precision, duals and balls | Keep selected library-generated jets and exact domains. Broader precision is an explicit profile investigation, not a silent fallback. Numerica 3.0.1 ball certification is limited to supported basic arithmetic; do not claim certified transcendental/provider evaluation or selection proof from that availability. Error-propagating floats are estimates. |
| Graphica and native graph algorithms | Preparation dependency semantics already have an identity graph/petgraph owner. No topology canonicalization/isomorphism requirement has been established here, so Graphica is not selected to replace it. Revisit only when anonymous topology equivalence is actually required; durable model IDs are not graph numbering. |
| Matrices, tensors and Spenso | Retain faer sparse numerical structures. Investigate native exact matrix/coefficient operations only for the reached algebraic use above. Scalar process equations do not currently justify a tensor-network migration; revisit for a concrete indexed contraction whose sparsity/topology would otherwise be lost. |
| `symbolica-ode` | Treat as an example of symbolic RHS preparation plus SUNDIALS integration. It is not selected as a replacement for current qualified DAE/ODE adapters; its early maturity and missing consumed event/initialization/sensitivity contracts require a concrete new use before integration. |
| `symbolica-integrate` and numerical integration | Rule-based antiderivatives can support a bounded authored primitive question, preserving branch/domain/coverage and derivative verification. No such automatic replacement is selected for physical axis integration. VEGAS supplies statistical estimates, not deterministic dynamics or certified enclosure; use only for an explicitly admitted stochastic-integration workflow. |

Follow the assessment's [polynomial](https://symbolica.io/docs/polynomials.html),
[solving](https://symbolica.io/docs/solving.html),
[transformer](https://symbolica.io/docs/transformers.html),
[coefficient](https://symbolica.io/docs/coefficients.html) and
[streaming](https://symbolica.io/docs/streaming.html) links, then exact pinned APIs for any
adopted operation. Companion releases and new dependencies require their own exact selected
version; the overview's ecosystem versions do not declare repository pins. These investigations
are bounded decisions with adopt/no-adopt outputs, not a blanket capability proof prerequisite.

## Packages, dependencies and retirement

| Package | Required input | Delivered behavior and acceptance boundary |
|---|---|---|
| MAT01 | Coordinator RC02 decision; current stage semantics | Demanded region/product description and deterministic preflight; cancellation never changes retained arithmetic |
| MAT02 | MAT01 working preflight; PRE04 resolved body contract for final identity | Symbolica definitions/aliases and joint value regions with original barriers, source checks and instance premises |
| MAT03 | MAT02 regions plus existing jet/action layouts | Demand-specific symbolic local products and correct outer composition across provider/branch/intermediate inputs |
| MAT04 | MAT02/MAT03 working producer and exported census | Complete interpreted/C++ consumer migration, truthful ownership/admission and removal of replaced per-output mechanisms |
| MAT05 | Regional implementation; preparation cutover | Derivative, guard, physical conformance and native-profile acceptance handed to combined qualification |
| MAT06 | Concrete reached use and its local scientific acceptance | Bounded library-family/JIT/formal-slot decisions; optional choices do not delay settled regional work |

**MAT01 — settle route before work.** Extend `jets.rs` demand/layout and `execution.rs`
construction admission with a deterministic preflight product. Unit controls distinguish an
ineligible deterministic shape from transient capacity exhaustion, cancellation and construction
error. Check requested order, product kind and complete support in identity; prove by tests
that a cancelled optimizer result never enters `ScalarCache`/retained artifacts. No native
preparation or mutable evaluator runs inside a new tracked query.

**MAT02 — build the actual library DAG.** Extend `library.rs`/admission to consume the
definition map and joint demanded outputs. Migrate `arithmetic_region` from independent-block
concatenation to dependency-preserving fusion bounded by host stages and source checks.
Coordinate `executable/grouped.rs` producer changes with PRE04. A deep shared-let fixture must
retain one mathematical definition and avoid repeated tree expansion. Preserve sparse outputs,
obligations, multi-output provider demand and per-instance validity/attribution. The old
per-output re-expansion and second normalization disappear when this producer cutover is tested.

**MAT03 — extend products without losing chain terms.** Implement symbolic local partial
definitions and reuse the library-generated composition layouts for every `ProductKind`.
Tests include direct scalar arithmetic, provider-to-arithmetic, branch-to-arithmetic,
intermediate-to-intermediate arithmetic and mixed second derivatives with both direct and
indirect coordinate dependence. Compare actions/adjoints with independently specified matrix
products and weighted Hessians; verify sparse consumed pairs rather than full dense triangles.
Preserve existing Taylor/reverse routes for their declared preflight eligibility domain until
the correct replacements pass; then delete the old route, callers and mechanism-specific
fixtures for the replaced domain. A whole-group Dualizer or `merge` is not the target.

**MAT04 — integrate real execution ownership.** Move selected compiled/interpreted consumers,
assembly and nested implicit preparation onto regions/products. Recount actual exported
instructions, functions, temporary storage, worker clones and code modules. Verify aliases
are fully inlined before current C++ export admission and keep source-attributed nonfinite
failures. Test two workers using one compiled template with independent scratch, cancellation
while compiling, construction refusal and post-cancellation drain. Keep full reservations
until coverage is established; an exported estimate is not native compiler RSS. F04 surrogate
pool worker joins/global-pool reach belong to
[RUN10](49e-runtime-and-native-execution.md) in the runtime companion.

**MAT05 — retain scientific acceptance.** Extend `product_guard_tests`,
`product_contract_tests`, curvature and implicit product tests, and compiler/runtime shared
conformance. Include domain cancellation (`x/x`), derivative-order square-root guards,
unselected invalid branches, closure validity, applicability, provider refusal, missing
derivative capability, shared bodies with different validity and nested implicit derivatives.
Original residual/closure checks must use their original meaning after optimized evaluation.
Re-run the revised probe A only to characterize the actual guarded demand and each product;
log `probe_stage_kinds` or equivalent actual stage inventory and evaluate wider/deeper fixtures.
Any quantitative claim separates preparation time, evaluation time, exported work and retained
state, with recorded profile/defaults and native/thread conditions.

**MAT06 — bounded native opportunities.** Record a concrete reached operation, existing
implementation burden, chosen pinned API, semantic downside and acceptance before an optional
adoption. Function-shaped formals using `Indeterminate::Function` get a focused cross-process
registration-order, `derivative`, `vectorize` and `export_cpp` check before replacing the pool;
until then retain the existing reproducibility control. JIT compares against C++ under the
numerical/ownership conditions above. Exact algebra/coefficient/transformer/streaming inquiries
adopt only if their named operation preserves domain/coverage/support and removes total work
or machinery; otherwise retain the current route with the concrete reason and revisit trigger.
Horner `hot_start` is considered only if the previous scheme is an explicit identity input;
it is incompatible with direct translation. `optimize_stack` is considered if exports show
unreused temporaries. Neither becomes an unqualified clear win or a pivot gate.

## Proposed verification and closure

Targeted implementation checks use `just check-package pse-math`, related compiler checks and
`just unit-package pse-math <filter>` for region/product/guard/curvature/implicit/native-compiled
tests. Use explicit compiled/native test features and `scripts/pse-env` routes appropriate to
the affected consumer; actual Arrow-consuming correctness closures retain force-validation.
No tests/builds/probes were run for this proposal. The failure baseline is zero.

Combined integration/native/Python journeys, derivative-consistency and closure campaigns,
performance characterization and once-at-scope-end static/manual checks are selected by the
coordinator. A local symbolic or C++ pass is **Tested** only for its named mode and conditions,
not full simulator qualification. Reduced repeated expansion, shared CSE and sparse demanded
construction are **Proposed** benefits. Numerical speed and memory improvements remain
unmeasured until actual guarded products are compared end to end.

This design explains R7, DA-027's construction half, DA-028 and the Symbolica-related candidate
inquiries. PRE04 supplies R5's identity half; PRE01 owns R8/F09, and the runtime companion owns
F04. Remaining questions are bounded family/JIT/formal-slot choices and concrete preflight
limits, not whether to keep project expression reconstruction as the long-term architecture.
Move enduring §7/§14 contracts through the coordinator's decision route before retiring this
companion and its resolved source material.
