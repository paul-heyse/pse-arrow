---
title: "Plan46a: Provider effects and implicit products"
status: done
date: 2026-10-18
adrs: [ADR-0193]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution]
---

# Plan46a: Provider effects and implicit products

## Target and foundation

Own AF01/AF08/AF13/AF14/AF19 and the provider side of RC01–RC03. The [coordinator](46-execution-speed-and-numerical-efficiency.md) owns rule/finding/inquiry decisions; this companion owns package progress. RC01–RC03 were accepted and this companion was authorized for execution on 2026-10-18.

**Interface-checked:** `pse-math/{assembly,implicit,implicit_selection,jets,execution}.rs` and `pse-kernels` provide suitable stage/provider/IFT owners. Current same-demand group hits bypass provider invocation; cross-demand projection alone excludes providers. Nested implicit selection correctly reruns provider-backed anchors/restrictions when called. Residual workers and factor workspace already exist, but final IFT constructs all parameter columns/pairs and projections allocate. The audit's direct-provider tests do not cover the enclosing hit.

The target carries complete physical values and current effect validity separately from actual demanded numerical products. Preserve original residual/selection/bound-neighborhood and derivative residual checks. Native inner iteration remains library-owned. A provider instance is private to its actual attempt; immutable preparation does not retain attempt cancellation, applicability or a prior provider.

<a id="a01"></a>

## A01 — Enforce current effects through enclosing caches

**AF19; S01; no rule relaxation.** Make every required invocation of provider-bearing selected-partial and weighted/action programs reach its provider owner even when input/order/weight/direction bits match. The simplest safe correction excludes such programs from direct enclosing cache hits; it does not remove pure arithmetic hits or the nested owner's qualified numerical reuse. Scope checks remain before use, and cross-demand projection retains its provider exclusion until an explicit effect capability supports more.

Treat keeping a completed lower-order result and authorizing its use as different operations. A cancelled or limited upgrade may leave completed data resident, but a later live request must still run current provider effects and may refuse. Do not weaken provider semantics to preserve a test's old call count. Migrate actual assembly partial/product callers together; remove the replaced unconditional hit logic for provider programs and update only affected tests.

**Targeted acceptance:** enclose the existing operational-selector provider in real CaseWorker constraints and weighted/actions. With all key bits unchanged, change current anchor, restriction/refusal and configuration; require a fresh visit/current outcome. Include order upgrades, typed refusal then recovery, cancellation, cross-demand exclusion, pure arithmetic hits and original scientific checks. Direct `provider.evaluate` alone is insufficient.

<a id="a02"></a>

## A02 — Bind immutable provider facts and atomic fill completion

**AF08; RC02.** At actual fresh attempt binding, resolve the prepared call to a provider slot and validate its descriptor/request extents. Enforce the descriptor's immutable lifetime within that binding or retain checks at uncontrolled public boundaries; a cached descriptor hash alone is not enforcement. Internal dispatch consumes bound facts instead of rehashing the full physical/implementation/data specification, map-looking up and constructing ordinal sets every call.

Add admitted caller-owned fill storage or an expiring internal completed view, choosing the simplest route that serves actual wrappers and A03 demand. Partial fill/refusal/cancellation never publishes a complete result; old data is not current completion. Keep current envelopes, finiteness, scope/deadline, capability/order, observations and selection effects. Infallible Symbolica external functions cannot replace this fallible provider protocol.

Migrate scoped/enveloped/observed/implicit provider wrappers and stage dispatch, not merely an unused trait method. Remove duplicate immutable checks only where the binding discharges the same premise; retain distinct trust/new-state checks. **Targeted acceptance:** wide counting descriptors, repeated warm calls without descriptor/set allocation, attempted descriptor mutation, malformed public demand, late partial-fill failure, current envelope/effect refusal and recovery.

<a id="a03"></a>

## A03 — Define and compose actual derivative demand

**AF01; RC01; prerequisite for selective execution.** Extend the existing provider/math product model to represent selected input columns, JVP, VJP and weighted second actions separately from full jets and output subsets. Named physical inputs, output/coordinate ordering, directions/cotangents, capability and derivative source/order are explicit. Unsupported actions refuse, never finite-difference or fabricate zero.

Carry that meaning through prepared identity, `ProviderLift`, stage execution, implicit providers, demand preflight and actual root/NLP/dynamic/fitting consumers. The chain rule must consume actions/contracted products where supported, not demand the full local jet after receiving a narrower outer request. All physical inputs/state equations/selection obligations remain. Complete jets remain a legitimate supported demand.

Specify first/second construction and retained extents from the actual representation before effects; the current complete-jet preflight must not reject a newly supported narrow action. Direct matrix/action reuse is coordinated with B05/E02 at existing assembly/product owners. **Targeted acceptance:** demand transport through a real nested provider composition, ordering/scales, unsupported products, complete-demand equivalence and pre-effect refusal at the demanded extent.

<a id="a04"></a>

## A04 — Execute selected implicit products

**AF01; consumes working A03 and A01/A02 semantics.** Use the verified complete state solve and sparse state factor to evaluate only requested parameter columns/directions or transpose contractions. Batched RHS and transpose solves remain library-owned. Complete residual derivative obligations and complete local state support can exceed selected returned outputs; no sparse-response assumption is made.

Implement actual first consumers and supported second-action composition without constructing the entire state-by-parameter Hessian as an intermediate. Derive weighted-second action equations at the existing implicit owner and check every required solve; “one direction means one solve” is not assumed for the complete operation. Unsupported richer actions remain explicit until their native/math capability is implemented. No placeholder compatibility path counts as delivery.

**Targeted acceptance:** fixed state count with independently varied total parameters and demanded columns/directions/cotangents; full-demand control; exact coupled analytic implicit model; singular neighborhood, branch/unknown-bound refusal and original checks. Count executed derivative components/RHS/storage before comparing complete latency. Preserve all off-demand physical dependencies and current effects.

<a id="a05"></a>

## A05 — Retain implicit residual, projection and verification buffers

**AF08/AF13; A01 before reuse, A02 for changed fill boundary.** Use residual-worker-owned admitted `TrialEvaluation` storage through the existing into-buffer path, instead of allocating each inner residual/unknown evaluation. After required configuration/anchor/restriction visits, borrow compatible completed numerical products while projecting/filling; avoid cloning a full jet merely to select part of it. Retain row-sum/scale verification arrays with the actual worker/factor operation.

Keep locks/borrow lifetimes explicit, no mutable shared result, current clocks/cancellation and completion flags. Preserve CSC accumulation order and contribution-wise absolute residual scaling; buffer reuse is not permission to omit independent derivative verification. Reset invalidates numerical visibility before storage can be reused.

**Targeted acceptance:** repeated residual and selected cached projection allocation/copy counts, First/Second/full/selected shapes, late refusal followed by success, concurrent independent providers, cancellation/reset and identical derivative residual assessment.

<a id="a06"></a>

## A06 — Associate and reuse exact factor/first products across upgrades

**AF14; RC03.** Associate the verified current state matrix, factor and completed first product with exact point, physical inputs, actual matrix coefficients, provider/selection/configuration, scaling and completion. First→Second demand computes missing residual/second information while retaining compatible factor/first work. The affine inner solver uses the same workspace today: a present NumericLu cannot establish its identity.

Current effects may change matrix identity at identical coordinates. Affine workspace use, reset, failed factor/verification or unknown provider failure invalidate the association. Keep structure-only sharing separate from matrix-specific reuse; never hand KINSOL/KLU factors to the different faer IFT problem. **Targeted acceptance:** count factor/first solve on an unchanged upgrade; change fixed input/configuration/effect, reuse affine workspace, induce factor/verification failure then recover. A retained factor is useful only if its actual completed numerical premise is established.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A01 | Existing provider/effect and CaseWorker contracts | Enclosing effect-safe cache enforcement in partial/product paths; unchanged-key effect/refusal controls | done |
| A02 | RC02 decision; actual wrappers/consumer inventory | Bound descriptor/slot and atomic fill/view in real callers; no stale partial publication | done |
| A03 | RC01 decision; A01 contract; A02 binding design | Demand/lift/admission and consumers agree; pre-effect extent/capability controls | done |
| A04 | Working A03; effect-safe providers; qualified library solves | Selected implicit first/transpose/second products in consumers; RHS/storage/original checks | done |
| A05 | A01; A02 when fill changes; actual worker ownership | Reused residual/projection/check scratch and removed temporary full clones; failure/reset tests | done |
| A06 | RC03 decision; A01; actual matrix/factor association | Compatible upgrade reuses factors/first products; affine/effect/failure invalidation | done |

## Verification and handoff

**Implemented:** A01–A06, immutable provider binding/atomic fill, complete/selected/action
transport, generic chain-rule pullbacks, qualified IFT products and retained factor/first
associations are integrated with compiler, native and runtime consumers. Public dense results
remain an explicit boundary. Factories retain a Value source and the unknown-only native
minimum; actual finite derivative demands prepare before numerical entry. Full consumers
explicitly prepare complete products. Missing preparation refuses without a fallback.

**Tested, 2026-10-18, zero failure baseline:** the following selected correctness commands
passed with zero compile warnings. Root commands used `BASH_ENV=''` and
`PSE_TEST_RESOURCE_STATE=/tmp/pse-plan46-root-resources-20261018`; delegated native commands
used an independent private registry. The supported recipes selected explicit Arrow validation
where the actual closure consumed Arrow; pure closures did not add unrelated relation roots.

| Command and mode | Result and scope |
| --- | --- |
| `just unit-package pse-compiler 'test(implicit_) \| test(kernel_nested_implicit_) \| test(kernel_regime_derivatives_)'` | 23 passed, 0 failed; 251 excluded. Actual compiler/factory demand, dense128, nested child minimum and regime controls. |
| `just unit-package pse-math '<selected modules below>' -p pse-kernels -p pse-backend-native --features pse-math/compiled-evaluator` | 217 passed, 0 failed; 504 excluded; 1.777 seconds test execution. Implicit65, interpreted/actual compiled local pullbacks, bindings/effects, compact results, support/frame, actions, assembly/native callbacks, scalar-cache and sparse/coefficient contracts. |
| `just unit-native-capability-package pse-backend-native pse-backend-native/kinsol,pse-backend-native/root-isolation,pse-backend-native/diffsol klu,isolation 'test(implicit_kinsol_first_order_contract_) \| test(implicit_kinsol_preserves_sparse_support_) \| test(implicit_kinsol_rechecks_roots_)'` | 3 passed, 0 failed; 467 excluded. Native-local KINSOL minimum, sparse support and original root checks. |
| `just unit-native-capability-package pse-runtime pse-runtime/solver-kinsol,pse-runtime/solver-root-isolation,pse-runtime/solver-diffsol klu,isolation 'test(native_child_value_source_) \| test(compiled_ill_conditioned_nested_relation_chain_)'` | 2 passed, 0 failed; 935 excluded. Value/nativeFirst construction, explicit full preparation and ill-conditioned nested reference. |
| `just unit-native-capability-package pse-runtime pse-runtime/solver-diffsol klu 'test(fitting_failed_lane_startup_) \| test(fitting_parallel_experiments_keep_order_) \| test(fitting_parallel_affine_exact_hessian_)'` | 3 passed, 0 failed; 780 excluded. Pre-native lane preparation, ordered parallel results, affine Exact Hessian contract and startup drain. |

The exact root math selection was:

```text
test(implicit::) | test(execution::tests::) | test(products::tests::) |
test(reverse::tests::) | test(assembly_tests::) | test(derivative_tests::) |
test(library::admission::tests::) | test(provider_tests::) | test(assembled_tests::) |
test(product_contract_tests::) | test(product_guard_tests::) | test(sparse::)
```

**Interface-checked:** default backend/runtime all-target compiles and selected native
KINSOL/isolation/Diffsol backend/runtime all-target compiles passed with zero errors/warnings.
Final full `just check-solver-contracts` result is recorded in the coordinator checkpoint.
Independent source review found no remaining production defect in the Value-source/preparation
split or its actual consumers, and confirmed the final lower-action/upper-coefficient correction.

G06 owns whole-series native/Python/static and assembled qualification after the remaining
companions. These checks establish the selected A/B endpoint, not that broader campaign.
No full Maturin installation, long simulation, environment proof or numerical-accuracy change
was performed.

## Outcome

### What was built

**Implemented:** Current provider effects survive enclosing selected/weighted/action caches;
immutable descriptors and slots bind once per actual attempt; outer checks publish only
successful fills. Separate action legs require one coherent owner-issued numerical premise,
including active children. Selected columns, JVP, VJP and weighted second actions use the
complete verified state and library-owned forward/transpose solves without unused parameter
jets. Retained residual/projection/verification storage and exact matrix/factor associations
serve compatible upgrades while affine use, changed effects, failure and reset invalidate them.
Original residual offsets, bounds, selector neighborhoods, cancellation, contribution-wise
solve verification and source/evaluator policy remain enforced.

**Measured, short analytic work controls:** two selected columns use five IFT RHS; a batched
second action uses one forward and two transpose solves in the selected coupled fixture.
Dense 128-parameter narrow products fit a limit that rejects complete Second preparation
before numerical effects. These are scoped operation/storage observations, not an end-to-end
speedup. Structural pair metadata can remain quadratic.

### A mistake made and corrected

The first narrow-product implementation still compiled broad parameter jets at factory creation.
Source inspection and the wide coupled control exposed that hidden cost; Value-source retention
and explicit actual-demand preparation removed it. Independent review also exposed Second-only
guards firing during First transpose, omitted authored offsets, irrelevant rival-factor demands
in certified selection, unchecked assessment gathers and singularity tests satisfied by missing
preparation. Corrections preserve the original scientific obligations, and the singularity
controls now assert their intended scientific error after explicit First preparation. Compiler
fixtures were corrected to expect stable factory programs across scientific capacities;
wide authored sums were balanced to respect the syntax-depth guard without relaxing it.

### Deviations from the plan, deliberate

Operational regime proof may still require broad First derivatives of every alternative;
certified exclusion requires its selected branch and validated proof scope. That scientific
work is distinct from returned numerical demand. No new action-incidence platform, generic
AD engine, cache service or native factor handoff was introduced. Standard engineering
accuracies are unchanged. The companion is retained because active Plan46 consumers and
G06 need its decisions and scoped acceptance.
