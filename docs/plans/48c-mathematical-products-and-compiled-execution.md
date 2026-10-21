---
title: Mathematical products and compiled execution
status: done
date: 2026-10-20
adrs: [ADR-0193, ADR-0212]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [current-work, preparation, numerical-execution, performance, design]
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
---

# Mathematical products and compiled execution

## Purpose and authority

This companion develops the mathematical preparation and execution portion of
[Plan 48](48-execution-performance-and-numerical-efficiency.md). It consumes AF05–AF08 and
the corrected R1 F01/F02/F03/F12/C02 and R2 P06/P09/P10/P11 recommendations from the
[completed audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md)
and its [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md).
The coordinator owns finding dispositions, inquiry state and rule decisions. Delivery sections
retain the original **Proposed** target; the checkpoint and Outcome record subsequent
implementation and focused evidence. The operator accepted RC01–RC12 as written on 2026-10-20; the
coordinator owns that record and required decision/design routing. RC04 accepts the complete
current-linearization direction; RC06 accepts investigation of realization/platform and
selected default changes. Neither selects an unknown winner or establishes implementation.

The target removes incidental parent scans, dense derivative reconstruction and one compiler
lifecycle per demanded scalar function. It preserves semantic decomposition: authored models
own physical meaning and branch/start permissions, library mathematics own arithmetic and
derivatives, numerical adapters own native iteration and factors, and workflows consume
prepared demand. It does not choose one model-sized program, compile every order, authorize
a new scientific algorithm or turn a scalar artifact cache into a platform project.

## Foundations and preserved contracts

The audit inspected commit `1ee81798a9b884f3054b685ee9629e2c8d124498`. Plan authoring additionally
inspected current `pse-math::{assembly,implicit_products,products,native_compiled,typed}` and
compiler conditional/executable preparation. The focused assessment uses Core 3.4, Efficient
Architecture 1.0 and Process Simulator 1.5 and reuses the audit's library/source conclusions.
Current contracts belong to [blueprint §7.1–§7.3](../authoritative_design/sections/mathematics-and-compilation.md#section-7-3)
and [§14.3–§14.5](../authoritative_design/sections/mathematics-and-compilation.md#section-14-3).

`CasePlan` already retains row contributions, requests, support and canonical matrix layouts.
Its conditional capability/allocation paths nevertheless iterate parent instances to rediscover
selected contributors. C01 extends known ownership into reusable access paths rather than
inventing a second semantic block model. Complete allocation/capability obligations and
fixed boundary coordinates survive selecting a smaller physical view.

`implicit_products::demanded_derivatives` distinguishes Value, CompleteJets and SelectedColumns.
Its action path currently builds an n-by-n tangent from unit directions, then extracts the
canonical sparse coefficients and factors them. Current child visits and matrix association
are conservative safety checks. C02 can remove dense scratch without assuming C03's stronger
pre-reconstruction validity is already available. The source's all-coordinate derivative
traversal is deliberate; a zero seed or output subset cannot replace it by assertion.

`products.rs` has a direct weighted-gradient directional route but admits it only when selected
pair support is complete. Sparse support therefore still uses pair construction. C04 makes
that library-owned direct action eligible for sparse cases without removing entry products
needed by other consumers. Existing reverse, multi-output and joint arithmetic paths already
do useful work; C06/C08 must identify remaining work rather than diagnose them as absent.

`native_compiled.rs` currently exports a scalar function per compile lifecycle, shares loaded
code and instantiates private worker scratch through `load_new_function`. Its profile uses
conservative floating-point flags and explicit child/output/resource accounting. Exact
Symbolica 3.0.1 capability inspected by the audit supports multiple named functions in a
shared module. The shared math skill's captured baseline is 3.0.0, so implementation must
reconfirm consequential APIs against actual Cargo.lock/current source instead of silently
transferring the older skill's contracts. C05 changes compilation packing; it does not
itself fuse evaluation or establish SIMD, JIT or device execution.

Ordered finite arithmetic, Require checks, branch selection, applicability and fallible
provider effects are observable. `sqrt(0)` may be Value-valid while First-invalid. Product
reuse needs sufficient derivative demand and complete current premises. Preserve the
`typed::unary_checked` exponent-materialization workaround: normalization can otherwise
discard a consumed exponent dependency. Any replacement needs qualification at actual
Symbolica 3.0.1. Existing fingerprint framing stays byte-identical; arithmetic transformations
follow declared mathematical/numerical and diagnostic equivalence, with bit identity only
where promised. Physical acceptance, including the selected engineering 0.001 endpoint,
and independent original-state assessment remain unchanged.

## Delivery sequence and interactions

C01 improves indexes for block and request consumers. C02/C04/C05 can make useful progress
without waiting for C03, provider contractions or platform choices. C03 first establishes
the complete validity association; only then may an action omit reconstruction. C06 selects
safe extensions within existing ordered arithmetic and instance scopes. C07 separates local
scalar persistence from the compiled-default decision. C08 separates provider contractions
from vector/JIT/device exploration, with independent no-adoption outcomes.

[48a](48a-source-checking-and-incremental-preparation.md) owns source checking and checked
artifacts; [48b](48b-registry-and-receiving-validation.md) owns immutable registry/receiving
preparation. Their hits supply currently eligible source/structure, never current numerical
state validity. Conversely, coefficient/factor or compiled-module identity cannot replace
current receiving admission. Native/workflow companions own algorithm policies, lane
compatibility, cancellation and result completion; these packets supply prepared products
and preserve their clocks, budgets, source attribution and effect boundaries.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C01 — indexed block and request contributors | Existing CasePlan contribution/request/source mappings and actual immutable-request dependency/ownership boundaries | Retain selected contributor, ordinal, boundary and request-consumer indexes; migrate capability/allocation/runtime block consumers, then parallelize eligible independent immutable numerical requests under shared admission, with deterministic failure and joined drain; remove parent scans and replaced serial glue while preserving full obligations. | done: CA01 repaired; selected instance request/support and indexed row eligibility growth/guard controls pass in the 27-test repair selection; 48h records selected qualification |
| C02 — sparse implicit and derived demand | Existing conservative support, canonical layouts and derivative contracts | Assemble sparse coefficients directly or by qualified compressed directions; keep genuine CompleteJets/SelectedColumns distinct; remove dense scratch and repeated unit-column/pattern work where replaced. | done: implemented; focused default and compiled-product controls passed; selected assembled acceptance recorded in48h |
| C03 — complete current linearization | I07 child/zero/subset evidence and accepted RC04 routed into its owner; usable sparse coefficient construction | Establish sufficient current point/parameter/provider/branch/order/scaling association before reconstruction elision; rebind action vectors only under that association and invalidate on changed/unknown premises. | done: implemented complete-current route; focused child, reuse and refusal controls passed; selected assembled acceptance recorded in48h |
| C04 — sparse flat direct HVP | Existing weighted-gradient action and sparse support | Admit direct library HVP construction on sparse support, keep exact entry consumers, and prove guard/provider/source equivalence before removing the complete-support eligibility restriction. | done: implemented; sparse analytic/entry and original guard controls passed; selected assembled acceptance recorded in48h |
| C05 — bounded named compiled modules | Existing admitted scalar exports and child/module/worker owners | Pack co-used already-demanded functions into bounded named modules, keep private scratch and actual child drain, migrate callers and remove per-function compiler lifecycle where replaced. | done: implemented; focused named-module and reverse controls passed as recorded below; selected assembled acceptance recorded in48h |
| C06 — ordered arithmetic, value/order and instance reuse | Current demand/effect contracts and I08 selection | Extend qualified joint work and sufficient-product reuse within ordered regions; preserve instance bindings and errors; retire repeated evaluation/preparation only for migrated scopes. | done: bounded current-pass pure-instance reuse implemented and focused tested; ordered-region expansion declined for current contract; selected assembled acceptance recorded in48h |
| C07 — persistent scalar artifacts and compiled default | C05 scalar module contract; I09 cold/warm decision; accepted RC06 for any selected default/scope change | Separately qualify local disposable scalar artifact persistence and investigate compiled-default policy; preserve compatibility, atomic publication and loaded-code lifetime without treating persistence as vector/JIT/device expansion. | done: implemented and focused tested: 12 scalar controls plus concurrent-publication extension; explicit compiled selection retained; selected assembled acceptance recorded in48h |
| C08 — provider contractions and realization expansion | I10 actual contraction gaps/capabilities; I11 credible workloads and accepted RC06 for expansion | Conditionally deliver qualified provider bulk actions and separately selected vector/JIT/device realizations; preserve demand/barriers/effects and total resource/crossing costs; no expansion is also a valid inquiry result. | done: existing composed actions qualified; I10/I11 settled: retain current contraction and scalar realization scope; selected assembled acceptance recorded in48h |

### C01 — retain construction-time ownership for smaller views

**Completion audit correction, resolved 2026-10-21:**
[CA01](48-execution-performance-and-numerical-efficiency.md#ca01) found remaining parent support
and row-eligibility scans. Support candidates now come from the selected instance's request
groups; `CasePlan::row` consumes the existing row index. The actual compiler boundary uses
that lookup. New growing-unrelated-parent controls and fixed/free-guard regressions pass,
preserving full objective/outside-row/fixed-input obligations without another retained index.

At CasePlan construction, retain the contributor and request-consumer associations already
known from each binding/output/row and canonical column. Conditional preparation receives
selected rows/columns and reads those indexed contributors, including duplicates and distinct
source contributions to one target. Request ownership maps back to every actual consumer,
not merely the first matching request. Boundary-fixed variables remain execution inputs
even when excluded from free-coordinate numerical support.

Move conditional capability/order checks, allocation bounds, selected block construction and
runtime block-boundary preparation together onto these indexes. Unsupported provider or
opaque-region bounds still return the current explicit refusal/unknown bound; an index is
not authority to omit a contributor's capability or memory. Preserve source order and
multiplicity, empty selections, unknown IDs, objective levels and boundary coupling refusal.
Targeted tests compare a small indexed projection with explicit contributor/source expectations
and complete-plan behavior, including many unrelated parent instances and overlapping request
consumers. Verify examined contributors fall with the selected population where the operation
allows it. Remove the replaced parent scans and their helpers after their callers move.

After indexed access is available, prepare genuinely independent immutable numerical
requests concurrently under the existing shared CPU/memory population. This is numerical
request preparation, separate from A06's source-checking phases. The preparation owner
must retain provider/phase dependencies, deterministic source/failure ordering and private
temporary state; work with ordered effects remains ordered. Migrate eligible block/request
callers rather than adding a second preparation scheduler. A skewed pair, a dependency edge,
failure and cancellation must demonstrate legal overlap and joined drain, with the same
admitted products as serial preparation and no duplicate capability/allocation work.

Physical grouping of tiny blocks is a separate numerical selection, not a consequence of
adding indexes. Semantic BTF and per-block policy remain. Any grouped nonlinear route must
qualify actual start/root/branch permission; algebraic equivalence does not promise the same
trajectory or selected root. Do not silently schedule session-per-shape or grouping as an
acceptance shortcut for C01.

### C02 — construct demanded sparse products without dense intermediates

Given a verified current implicit point, parameters, conservative state support and requested
product, construct canonical sparse state coefficients directly or by a qualified compressed
direction scheme. Direct selected entries and conservative CPR/coloring are alternatives;
the deciding factors include actual provider derivative capability, support and total
preparation/evaluation work. No universal coloring policy or exact JVP-count gate is selected.
Canonical scatter preserves row/column order, duplicates/accumulation and normalization.
Derived-family products similarly use known support/maps instead of repeated unit columns
and pattern rediscovery.

Preserve distinct consumers: CompleteJets can require broad dense work; SelectedColumns are
not arbitrary actions; JVP/VJP/HVP consumers need their exact requested outputs and order.
Original residual verification, unknown-bound neighborhood refusal and independent derivative
solve checks remain. Singular/changed systems, unavailable derivatives, cancellation or
provider failure cannot publish partial product buffers. Use existing library factorization
and derivative composition rather than adding project-owned sparse calculus.

Exercise sparse diagonal/banded and coupled examples, selected/permuted outputs/coordinates,
structural zeros, changed parameters/providers/branches, singular matrices and failed trials.
Compare against independently authored analytical derivatives or existing complete products
where those are a valid oracle. Use finite differences only as a test comparison at nontrivial
valid points, never a runtime fallback. Check canonical sparse storage and construction counts
at short revealing sizes. Migrate implicit/derived action consumers and delete replaced dense
scratch/unit-column code and fixtures immediately after targeted acceptance. Until C03 is
available, reconstruct current coefficients before compatible factor reuse as the existing
contract requires.

### C03 — establish validity before skipping reconstruction

A reusable association must identify the current point, parameters, normalization/scaling,
method state and selected regime, plus actual provider context/visits and demanded derivative
sufficiency. Equivalent matrix bits are useful only after construction; dimensions or unchanged
state values alone do not show that a provider/branch/effect is current. New action directions
and weights rebind to an eligible association; they do not become an unbounded cache axis.
Unqualified or unavailable premises force reconstruction or explicit refusal. Affine-workspace
use, reset, failed verification and uncertain mutation invalidate the association as required.

**I07 asks whether a cheaper validity traversal reaches every consequential child.** Compare
the current conservative reconstruction with an explicit complete current-linearization
association and any proposed zero-direction/selected-output traversal. Include zero seeds,
output subsets, numerically cancelled dependencies, changed branches, Value-only versus First
capability and child contexts whose numbers stay unchanged while their premises change.
Use provider-visit evidence and independent expected refusal/effect ordering; an equal final
matrix alone cannot settle the question. Stop when the supported traversal has complete
visitation semantics and invalidation rules, or reject elision for scopes it cannot cover.
Adopt the justified scope or retain reconstruction with C02's sparse construction. Reopen
for new provider/branch/product kinds or evidence of an uncovered consequential visit.
The coordinator owns I07's current result.

Apply accepted RC04 through the decision/design owner before production callers depend on
pre-reconstruction reuse. Targeted tests show a second compatible action reuses coefficients/
factors, changed child premises reject reuse despite equal values, and cancellation/failure
cannot leave an eligible partial association. Include first-to-second upgrades and changed
normalization. Remove the replaced reconstruction work only in qualified consumers; a sound
fresh sparse reconstruction path remains necessary, not a stale compatibility path.

### C04 — make sparse direct weighted actions eligible

For a flat body and selected sparse support, construct the demanded weighted-Hessian action
through library-owned directional weighted-gradient mathematics without first building every
second entry. Preserve the supplied weights, direction, coordinate order and full original
domain/derivative obligations. Keep entry realization for consumers requiring entries and
cases where its actual preparation/evaluation economics favor it; the plan introduces no
unmeasured density crossover estimator.

Extend the existing `products.rs` controls to sparse diagonal, banded and disconnected support,
multiple outputs and selected coordinates, including zeros/cancellation and nonlinear providers
where supported. Compare direct actions with independent analytic contractions and the valid
entry route. A guard failure, nonfinite result or unsupported derivative must keep its exact
source and first refusal. Migrate action preparation to the broadened eligibility and remove
the complete-triangle-only restriction and replaced action-only pair construction where no
consumer still needs it. Do not delete genuine exact-entry programs or relabel approximate
curvature as exact.

### C05 — compile bounded named scalar modules

The operation receives already-demanded scalar evaluator exports and their layouts, groups
functions whose co-use and invalidation make sharing useful, exports stable names and returns
a module-backed function mapping. Loaded code has a retained owner; each worker has private
library scratch. Function-name uniqueness, input/output layout, source attribution and module
identity are checked. An unrelated new product/order is not compiled merely because other
functions share the module.

Packing reduces compiler/linker children without moving arithmetic across guards/providers
or changing its per-call effect scope. Bound export/source, object/shared-image coexistence,
loaded-code retention and worker scratch through existing admission. Parallel remaining
compiler children only when justified, preserving cancellation, output limits and actual
kill/join/drain before charge release. A partially compiled or partly loaded module is not
published. Code remains loaded until the last function/worker finishes, including error paths.

Qualify actual Symbolica 3.0.1 named exports and shared loader semantics against current source.
Targeted tests cover two named functions with different layouts, independent simultaneous
scratch, missing/wrong names, compile failure, cancellation, resource refusal, and dropping
one function while another remains live. Compare each function's valid values/derivatives
and original failures with its existing scalar realization. Migrate actual scalar compilation
callers and remove one-child-per-function lifecycle code for packed consumers. Avoid a second
unbounded module service. Measure total cold preparation and warm evaluation separately;
fewer compiler children are not by themselves a latency claim.

### C06 — reuse within ordered arithmetic and instance scopes

**I08 asks which joint evaluation and cross-order/instance reuse remains useful and sound.**
Compare current joint regions/multi-output optimization with a small further co-used arithmetic
region, sufficient-product reuse and instance batching. Inspect every original materialized
value, finite/Require failure, branch and provider effect between proposed grouping points.
Use cases such as an early nonfinite intermediate, `sqrt(0)` Value versus First, a later
unselected invalid branch and distinct instance normalizations. Stop when each proposed
scope has a declared equivalence/demand contract and a useful removed-work case, or retain
current grouping for that scope. Adopt only qualified regions/products; no-adoption is an
explicit result. Reopen when changed demand, guards or provider capabilities alter co-use or
barriers. Coordinator state records the inquiry outcome.

Implement selected extensions through existing library arithmetic and prepared support.
A previously computed product may satisfy a lower demand only if its complete current
premises and original validity establish that operation; Value does not imply First, and
First-only demand does not prepare Second speculatively. Instance batching retains each
instance's bindings, coordinate/source maps, normalization, obligations, branch/effect scope
and bounded private scratch. Shared mathematical identity is not a shared mutable worker.

Test ordered failures and provider effects, selected demand, attribution, per-instance results
and cancellation as well as ordinary numerical equivalence. Preserve exponent materialization
unless a replacement passes actual-library regression controls for the consumed dependency.
Migrate concrete region/value/instance consumers and delete the replaced repeated work at
that boundary. Do not fuse across a guard/provider barrier by moving checks after arithmetic,
or require bit identity beyond the existing contract. Fingerprint encoding itself stays exact.

### C07 — separate scalar persistence from default selection

A local persistent scalar artifact can accelerate loading the same demanded module without
introducing a new execution platform. Its complete key and load checks cover evaluator/body
and product identity, function/layout mappings, actual ABI/ISA/compiler/FP compatibility and
consumed library/runtime interpretation. It is disposable acceleration, not source authority
or an environment-proof service. Atomic publication and compatible lookup precede loading;
missing/corrupt/incompatible artifacts compile afresh under the original clock and budget.
Partial writers and interrupted loads cannot become hits, and eviction cannot unload code
still held by live workers. Private scratch is reconstructed for each worker.

**I09 asks whether persistence and a compiled default are useful for the actual scalar
workloads.** Evaluate them as separate decisions: compare current interpreted/explicit compiled
execution, packed cold compilation, warm prepared evaluation and compatible local artifact
reload. Use short representative arithmetic/derivative shapes and realistic demanded orders,
with actual selection receipts; a capability flag alone does not establish that compilation
ran. Include load/compile failure and low-reuse cases. Stop when the tested scope supports
adopting or declining persistence and a default policy independently. An uncertain default
remains explicit selection, even if persistent artifacts are adopted. Reopen for changed
cold/warm workload, module invalidation behavior or actual compiler/library compatibility.
The coordinator owns I09 and routes any selected default change through accepted RC06.

Targeted implementation tests cover cache hits, every materially different compatibility miss,
corrupt/partial artifact handling, interrupted publication, concurrent creation/eviction and
loaded owner lifetime. Preserve conservative flags and existing platform limits. Remove
replaced scalar artifact publication/load machinery only after migrated consumers pass; fresh
compilation remains a legitimate miss path. Scalar persistence does not require accepting
SIMD, JIT, GPU or distributed execution, and acceptance of RC06 chooses none of those methods.

### C08 — investigate remaining provider contractions and platform capability

**I10 asks which bodies still incur avoidable per-output provider work.** Existing reverse
and weighted actions already cover some cases. Identify a concrete remaining body/provider
pair and compare current demanded calls with the provider's actual JVP/VJP/weighted-action
or contraction capability. Trace selected outputs, argument maps, derivative sufficiency,
visit/linearization validity and effect multiplicity. Stop when a bounded pair has an exact
consumed contract and demonstrated removed work, or conclude no additional route is warranted.
Adopt that qualified pair or retain current composition. Reopen for a new provider bulk
capability or a demonstrated remaining per-output bottleneck. Do not implement a generic
reverse engine on the premise that all reverse work is absent.

**I11 asks whether a wider realization fits a credible coherent workload.** Compare the
existing packed scalar route with actual eligible SIMD/vector, JIT or provider-free tiled
execution. Device/distributed routes require a concrete workload large/coherent enough to
assess transfers, launches, synchronization, scratch and recovery alongside computation.
Resolve source/demand/barrier fidelity, floating-point profile, library capability and
cancellation/drain for each candidate. Stop after bounded representative evidence selects
one justified scope or no expansion; do not turn a catalog survey into indefinite platform
implementation. Reopen when such a workload or a newly available library capability changes
the deciding total-work premise. The coordinator records I10/I11 independently and routes
selected realization changes through accepted RC06 and the applicable binding decision.

Any adopted provider contraction or realization migrates its specific consumers and preserves
typed unsupported/refusal behavior for the rest. Verify values, exact demanded derivatives,
ordered guard/provider effects, source/instance mapping, resource refusal and interruption.
Remove replaced per-output/crossing machinery only after those consumers move. Broader
platform support is not inferred from a successful scalar module or one contraction example.

## Current implementation checkpoint

The ordinary compiled-stage constructor now collects already-demanded provider-free exports
across its demanded orders and nested stages into modules of at most eight named functions.
This finite packing bound does not fuse arithmetic or execute inactive branches. Every
member retains its existing before-export allowance; shared source/object/library ownership
and actual compiler-child drain use the existing native owner. All symbols and dimensions
are checked before publishing the module, and each worker loads private mutable scratch.
The local profile identity advances to the named-module realization; conservative arithmetic
flags and physical acceptance stay unchanged. Direct single-function products use the same
module builder. Co-used reverse pullbacks now use the same bounded module preparation.

**Tested, 2026-10-20:** `just unit-package pse-math 'test(native_scalar) | test(named_module)'
--features compiled-evaluator` through pse-env passed11 tests,0 failures against zero.
It covers production Value/First/Second packing, exact conservative arithmetic, mixed
function signatures, eight-function rollover, worker/file lifetime, compiler failure and
cancellation/drain. One trivial fixture cast warning was corrected after the pass; the next
focused feature run selected12 tests:11 passed and one fixture incorrectly expected four
exports where the actual joint lowering demands three across two modules. Correcting that
expectation was followed by a passing failed-only `native_scalar_reverse` run. The later11-test
compiled selection below also exercises reverse composition. These are focused controls, not
an invented clean12-test aggregate or a measured latency benefit. The initial C01/C02 default
math slice passed100 tests; the later source checkpoint records the expanded controls.

Independent numerical preparation now has a caller-stop-aware shared-flight primitive in
source. A departing last waiter cancels and joins its actual producer before returning;
another consumer's producer is neither cancelled nor awaited by that departing waiter.
Existing abandoned-future safety remains completion-owned. The ordered artifact batch uses
existing CPU/memory/flight admission, reduces errors in request order, and cancels/drains
issued private work before returning the first error. Batch coordination reserves exact future,
result and captured request-descriptor storage before effects. Initialization prepares the
independent artifact requests across BTF blocks together while numerical execution retains its
causal order. `just unit-package pse-runtime 'test(artifact_batches) | test(artifact_flight) |
test(artifact_)'`, through the pinned pse-env route with `pse-runtime/force-validate`, passed5
tests,0 failures and no warnings against zero. This qualifies the exercised shared-flight and
batch owner, not assembled solver or runtime performance.

### Source implementation checkpoint, 2026-10-20

**Implemented / focused Tested:** the indexed contributor/request population includes
conservative boundary candidates from selected rows and every binding of a selected unknown.
Each visited candidate retains all original contribution, control, provider and objective
obligations; complete consumers retain the complete dependency path. Shared-demand upgrades
visit every actual consumer and preserve each exact support. Runtime initialization consumes
cached source, slot and contribution populations. Sparse IFT action construction writes state
coefficients directly into the original canonical CSC support; cold construction still uses
all state unit actions. Pseudo-time derivatives retain one canonical layout and column scatter
while preserving every required unit action and unsupported-entry refusal.

**Implemented / focused Tested:** sparse flat HVPs now use the existing library-generated
weighted directional gradient without preparing Hessian-entry pairs. Exact-entry consumers
retain their separate product. Current-pass donation is limited to identical immutable requests,
complete normalized gathered bits and sufficient order, with no providers, controls,
applicability evidence or projection witnesses. Every receiver gathers and validates its own
inputs first; scattering and observations remain attached to that receiver. Existing ordered
independent sibling regions remain the arithmetic contract. Dependent substitution would need
an explicit materialized-intermediate and finite/source-order contract that is not supplied
by the current independent-region owner. Reopen for a concrete dependent region with those
original intermediate checks and bounded ownership preserved; marginal timings are not a
reason to withhold an otherwise qualified work-reducing route.

**Implemented / focused Tested:** ADR-0193's RC04 amendment and blueprint revision220 precede
the adopted C03 route. A non-effect candidate first matches the immutable problem/layout,
exact numerical premise, point bits and state-First capability. It then executes the same
all-residual/all-coordinate First JVP with zero seed now, checks original residual budgets
and the unknown-bound neighborhood, and compares the complete ordered child source/key/context
trace. Every current child must supply its seed-independent context. Only that complete match
restores retained canonical coefficients and factor; numerical direction/weight bindings remain
fresh. Missing/mismatched association reconstructs; failed validation, cancellation, reset and
affine workspace use invalidate. State-First reuse grants no unperformed higher-order work.

**Tested:** through the pinned pse-env package routes, the expanded default math selection
(`assembly_tests`, ownership/instance reuse, demanded IFT products, derived products and
`products`) passed115 tests,0 failures against zero. The conditional compiler selection
`test(workspace::modeling::conditional)` passed8 tests,0 failures. The compiled-evaluator
selection of demanded IFT/products/instance reuse passed32 tests,0 failures; the strengthened
`test(zero_state_traversal) | test(sparse_state_actions)` selection separately passed3 tests,
0 failures. The five-state control demonstrates no new state-unit construction on an eligible
hit; changed child context with unchanged state coefficients/residuals reconstructs, and branch,
missing-context, original-residual and cancellation controls retain refusal/invalidation.
`just check-package pse-math --features compiled-evaluator` passed all targets without warnings.
These selected controls do not establish assembled48h qualification or a universal speedup.

**Implemented / focused Tested; E04 handoff:** exact semantic entry preparation is in the
existing CasePlan/ArtifactRequest pipeline under ADR-0203. Complete static First support stays
independent of numerical Value/JVP/selected-entry artifacts. Exact pair order, physical gather
scale, aliases, repeated contributions and structural absence have canonical scatters; original
First guards execute even for an absent derivative. A selected-only worker allocates no full
numeric Jacobian CSC and explicitly refuses its full-Jacobian API; monotonic full preparation
restores that product. `test(assembly::entries)` with `compiled-evaluator` passed2 tests,0
failures; one unnecessary test qualification warning was subsequently removed. The compiler-issued
artifact control passed separately below; the later authored Jacobi/BBD runtime controls
and accuracy-evidence refusal are recorded at 48e/E04–E05. An initial alias fixture placed
NaN in an unread coordinate; it was corrected to invalidate the actual consumed source rather
than inventing a new whole-inventory numerical check. Entry metadata, support narrowing,
operation gathers and artifact descriptors are charged before their new allocations. The
dynamic profile/consumer evidence remains owned by 48e/E04; these focused passes do not close
48h assembled qualification.

**Implemented / focused Tested:** selected source inspection now exposes actual
provider source/key occurrences before numerical compilation. Built-in dynamics can retain
ordinary full preparation for provider demand whose exact selected support is not established
by the immutable owner. The explicit low-level selected route validates actual worker demand;
only providers revisited across entry groups require completed linearization capability, and
each current token must match before any caller output is published. Unknown or drifting current
context refuses atomically; a single group gains no global context restriction. No registration
hint, second worker construction or after-effect full retry was introduced.
`just unit-package pse-math 'test(assembly::entries)' --features compiled-evaluator` passed3 tests,
0 failures and no warnings against zero. The provider control covers stable, unknown, unsupported,
drifting and missing current contexts, preserves single-group eligibility, and retains an ordinary
full-only provider route. Refined scratch/source-metadata bounds are present in this tested source.
Dynamic built-in eligibility and its actual producer/consumer qualification remain48e/E04 work.

**Tested / Measured:** the private C07/I09 and C08/I11 provider-free polynomial/First/Second-
coefficient inquiry now compares interpreted evaluation, the actual two-function packed scalar
module, same-process disposable module reload and actual scalar JIT. The exact compiled-evaluator
selection `test(assembly::entries) | test(realization_probe) | test(zero_state_traversal) |
test(sparse_state_actions) | test(reverse::tests::composed_action)` passed11 tests,0 failures,
without warnings. A separate `just unit-package pse-compiler 'test(selected_entry_artifacts)'`
passed1 test,0 failures: it builds actual compiler-issued artifacts, preserves static First,
requests only the one selected numerical First axis, evaluates the analytic entry and refuses
full Jacobian coverage. This qualifies that exercised issuance, not the dynamic bridge.

The realization probe ran once in a cold Linux x86-64 test process, with Symbolica3.0.1,
SymJIT2.26.0 and16 repeated warm batches. It has2 scalar values and12 library-derived First/
Second coefficients per row. Common symbolic construction took94µs and floating preparation
260µs. The packed two-function module's compile/load took511558µs; JIT construction took429µs.
Atomic same-process image publication took45µs and exact-image/layout validation plus reload
72µs. Partial unpublished writers, corrupted loader input and live loaded code after image
unlink were exercised. These same-process checks do not certify a receiving runtime after restart.

| Rows | Packing / output allocation (µs) | Interpreted warm16 (µs) | Packed warm16 (µs) | Scalar JIT warm16 (µs) | Packed complete (µs) | JIT complete (µs) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 1 / 2 | 1 | 1 | 1 | 511916 | 787 |
| 64 | 1 / 0 | 37 | 21 | 8 | 511934 | 792 |
| 512 | 2 / 4 | 309 | 182 | 59 | 512100 | 848 |

Complete columns include common symbolic/floating construction, realization construction/load,
packing, output allocation and the16 warm batches. Microsecond resolution can report zero for
small allocations; no threshold or statistically established speed ratio follows from this run.
The current interpreted route remains cheaper including preparation for these sampled totals.

Actual JIT settings were direct translation, optimization2, and `fastmath=false`,
`use_simd=false`, `enable_simd512=false`, `use_threads=false`, `parallel_mul=false`. The host
reports AVX/AVX2/FMA. Symbolica3.0.1's JIT wrapper cannot select a
portable compiler type through its options; SymJIT2.26.0 Native chooses host AVX independently
of `use_simd=false`. Explicit `fastmath=false` disables its source-gated FMA fusion, but this
does not grant the production conservative x86-64 ISA contract. Persistence additionally needs
a complete receiving compiler/runtime compatibility interpretation beyond a same-process image
and layout check. Those are concrete decision limits, not presumed library absence.

**Tested; settled bounded I10 result:** the actual existing contraction
pair uses128 inputs and2 provider outputs followed by a nonlinear outer scalar. Two distinct
outer weights produce exactly2 JVP and2 transpose requests, plus2 weighted-second requests for
the Second product, without requesting full partial arrays. Interpreted and compiled local
pullbacks, changed followup contexts, unknown owners and active/inactive branch guards passed
in the11-test selection. The additional transpose consumes the outer cotangent tangent, which
differs from the weights passed to the weighted-second call. The current provider contract does
not return both bindings' contractions in one invocation. No additional route is justified for
this concrete pair; a new provider bulk capability or demonstrated remaining crossing bottleneck
would reopen it. This bounded result makes no absence claim about all providers or future bodies.

The [coordinator inquiry outcomes](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes)
settle I08–I11 within these bounds. I08 adopts current-pass pure sibling reuse and retains
ordered dependent regions. I09 now implements bounded disposable persistence at the complete compiler-input owner under
ADR-0212. The earlier same-process reload inquiry did not settle fresh-process receiving; actual
source/profile keys, integrity checks, current private copies and fresh-process controls are now
part of C07. Explicit compiled selection remains independent of cache adoption. I10 retains the qualified composed
actions; reopen an actual repeated cotangent or contracted product outside the exercised pair.
I11 retains the scalar profile; reopen for a coherent reuse workload with an explicitly admitted
ISA, floating-point and cancellation owner, plus transfer ownership for any device route. The remaining conditional no-adoption decisions retain their stated physical/ownership limits;
they do not replace C07's now-active concrete implementation obligation.

## Verification and handoff

**Proposed verification:** compile touched `pse-math`, `pse-compiler` and runtime consumers
through their package routes, then run meaningful targeted units for the cases above.
Existing derivative/product controls such as
`selected_provider_demand_matches_fixed_coordinate_program_and_authored_partials`,
`demanded_directional_provider_composition_keeps_original_guard_and_first_partials` and
`conditional_block_keeps_actual_guard_provider_execution_inputs` are useful starting points;
new sparse/validity/module mechanisms need their own revealing cases in the same change.
Use `just unit-package <pkg> <filter>` with filters that actually select those tests and
retain explicit force-validation in every Arrow-consuming correctness closure.

Assembled solver/component/Python journeys, relevant performance campaigns and static/manual
checks belong once at coordinator scope end after implemented functional scope. Benefit
measurement distinguishes source/index visits, sparse construction/factor reuse, compiler
children, cold preparation, warm execution and retained memory. No universal density threshold,
batch size, speed ratio, numerical tolerance change or latency SLA is selected. Scientific
comparisons retain independent original assessment and the existing engineering endpoint.

The original authoring task ran no tests, builds, profilers, services or environment qualification.
A package's targeted pass establishes only its exercised behavior; it does not qualify all
numerical routes, industrial model sizes or a conditional platform. Report actual command,
mode, scope and failure count against zero at implementation. Transfer enduring contract
changes through their existing decision/design owners when the coordinator closes the series.

### H01 scoped completeness handoff

R1 F01/F02/F03/F12 and P06/P09 map to the implemented indexed contributor and boundary
consumers, sparse canonical IFT/derived products, complete-current factor association, direct
sparse HVPs, shared artifact batches and bounded named modules. Their focused controls above
cover the replaced producer and actual smaller-view consumers. Cold IFT construction still
uses every state unit direction; selected semantic entries are distinct from full numerical
First coverage. The exact compiler artifact control and three entry/provider controls cover
that distinction. The current typed childless-implicit-owner preflight additionally passed its
one focused control, and the compiled-evaluator all-target math check passed with zero warnings;
the actual authored BBD Value/JVP consumer subsequently passed its canonical runtime control
after an empty-entry layout correction. The Jacobi wide-model receiver reached its exact
selected demand and matched full-KLU independent physical classification. Its corrected
canonical control passed with all 10 native setup evaluations observed as selected, zero
full setups and positive selected-entry work under complete aggregate metric coverage.
The separate ordinary accuracy-evidence refusal control also passed; 48e/E04 owns the
actual selection conditions and enclosing failure counts. Selected First support does
not establish an implicit rate provider's arithmetic uncertainty, and the original-invalid
point remains invalid under both selected and full-KLU routes.

P10 arithmetic, guards, exponent handling, sufficient-order reuse and instance batching map
to C06's existing ordered regions and bounded current-pass pure-instance donation. Dependent
substitution and unchecked intermediate removal were not adopted. P10 provider contractions
and P11 persistence/default/JIT rows have the settled I08–I11 outcomes and concrete reopening
conditions above. No universal color-count, one-model module or new realization guarantee is
inferred from those focused controls.

P11 parallel compiler children has a narrower current implementation boundary: a module
rollover synchronously compiles and links under the original numerical scope, joining each
actual child. C01 overlaps independently admitted immutable artifact preparations; C05 reduces
child count through packing and preserves cancellation/drain. No intra-artifact child fan-out
or ungranted nested CPU population is claimed. Any expansion requires a concrete multi-module
workload and its actual additional CPU/memory owner, rather than reusing the same grant twice.
I09/C05 is settled as no additional intra-artifact parallel adoption under that single-core
owner. Reopen only for a material multi-module benefit with the additional grant; this is
a qualified retained route, not an unimplemented parallelism promise.

The physical-grouping links shared with R1 F03/P06 are settled by48d/D05/I17: retain existing
explicit common direct-case preparation and distinct BTF block owners, with no automatic subset
coalescing policy. The source-qualified grouping inquiry includes original residual checks,
replacement factors and complete decomposition/setup costs. D07/I20 likewise retains the
explicit qualified linear portfolio after complete engineering-accuracy comparisons; no tiny
sample chooses a global backend. Those native inquiry decisions remain owned by48d and the
coordinator. This package handoff does not close48h assembled qualification.

The prior scoped handoff missed C01's remaining support and row-eligibility scans. The
independent completion audit identified that boundary lookup work under CA01; the repair
and actual growth controls now pass. Shared request
demand updates visit every indexed consumer, and native module/cache and provider-entry
callers use their migrated owners; no additional C02–C08 gap was identified. Deliberately retained complete reads,
cold state unit actions and fresh compiler misses are required consumers, not legacy routes.
Current limitations remain actual additional CPU admission for intra-artifact fan-out, ordered
intermediate semantics, and portable ISA/FP permission for JIT expansion. The sampled compiled
module's cold cost exceeds twice interpreted complete work; its explicit-selection policy is
separate from the now-adopted disposable code cache. No remaining choice is declined solely
for a marginal timing difference. The named focused controls above and 48e's actual authored
receiver passes retain their separate qualification boundaries; 48h assembled acceptance
remains distinct.

## Outcome (recorded after implementation)

### What was built

**Implemented and Tested, completed adopted scope (2026-10-21):** the independent
[Completion audit](48-execution-performance-and-numerical-efficiency.md#completion-audit)
identified CA01 in C01; the repair and settling controls now pass with adjacent checks
in the 27-test selection. [48h](48h-qualification-and-design-publication.md#completion-audit-repair-qualification)
records actual repair acceptance and all three full continuing Clippy modes. Earlier
observations retain their original conditions and partial-run boundaries; no outstanding
accepted package obligation remains in this companion.

**Implemented / focused Tested, partial scope:** indexed selected contributors and request
owners, sparse canonical IFT/derived products, complete-current factor qualification, sparse
direct HVPs and bounded pure-instance reuse are implemented as recorded above. Named compiled
module packing and shared-flight preparation are implemented and focused tested above. C07 now implements complete-input disposable scalar code caching under ADR-0212 while retaining
explicit compiled selection and the current scalar profile. The earlier ephemeral-only inquiry
outcome was reopened under the maintainer's work-reduction-first adoption criterion.
Exact semantic entry work is consumed by 48e/E04–E05's authored Jacobi/BBD routes, with
selected setup coverage, no-full-matrix assertions and explicit uncertainty refusal
qualified in their focused runtime controls.
Whole-series assembled qualification remains48h. Finding, inquiry and rule outcomes remain
owned only by the coordinator.

**Implemented / Tested, C07:** `just unit-package pse-math 'test(native_compiled::)'
--features compiled-evaluator --success-output immediate` ran12 tests on2026-10-20:
12 passed, zero errors/warnings against the zero baseline;433 unselected. Three actual fresh
processes assert cold compile, received code without compilation, and valid-checksum invalid-module
fallback, with independently checked scalar values10/17 and private module lifetime. Child
execution markers prevent a zero-test subprocess from becoming a passing observation. Other
controls cover complete changed inputs, dimensions, corruption/partial/oversized images,
nonblocking publication lock, finite eviction, cancellation/deadline and joined compiler cleanup.
The subsequent `just unit-package pse-math
scalar_cache_publication_is_bounded_nonblocking_and_private_copies_survive_eviction
--features compiled-evaluator --success-output immediate` passed1/1 with zero errors/warnings:
two real publishers race eviction and receiving; every received image is complete and exact,
publication remains bounded and the private loaded-copy contract survives. No whole-product,
default-realization or speedup qualification follows from these units.

### A mistake made and corrected

**Implemented / focused Tested:** an early instance fixture compared raw signed zeros although
the existing affine gather normalizes both through its zero offset. Reuse now has independent
controls over the actual gathered bits and over equal-valued opposite-sign inputs, which must
not share arithmetic. The source conversion contract was preserved. A later selected-entry
fixture similarly moved its invalid value from an unread aliased-away coordinate to the actual
consumed source; no unrelated numerical refusal was introduced.

### Deviations from the plan, deliberate

**Implemented:** cold sparse state construction retains all state unit actions and original
coordinate checks; no compressed-coloring or fewer-cold-JVP claim is made. Current ordered
independent sibling grouping is retained rather than expanding dependent substitution without
its materialized-intermediate contract. The bounded persistence/default and realization inquiries
retain the current scope for the concrete reasons and reopening conditions above; a fast scalar
module alone selects no new default or execution platform.
