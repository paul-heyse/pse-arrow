---
title: Mathematical products and evaluator efficiency
status: in-progress
date: 2026-10-17
adrs: [ADR-0193]
review_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, preparation, libraries, validation]
---

# Plan 43b: Mathematical products and evaluator efficiency

## Purpose and foundations

This companion supplies F05/F06, mathematical slices of F07, O01/O03 and Q01 to [Plan43](43-solver-diagnostics-and-performance.md). It owns selected product preparation, arithmetic realization, point-result projections and implicit derivative work. Native policy/factors remain with [43c](43c-native-methods-reuse-and-matrix-boundaries.md), dynamic scientific composition with [43d](43d-dynamics-fitting-and-workflow-effectiveness.md), and admitted lanes/platforms with [43f](43f-execution-resources-and-platform-profiles.md). This document owns B-package progress, not combined finding disposition.

**Interface-checked:** the integrated review, Symbolica 3.0.1/faer 0.24.4 source contracts and current `pse-math` typed/execution/jets/assembly/implicit/library owners plus compiler/runtime artifact consumers. Reuse that evidence; the local skill's Symbolica 3.0.0 baseline alone cannot qualify3.0.1 behavior. **At document authoring — Proposed:** all corrections/qualification. The coordinator records accepted RC03/RC04 and their bounds; any additional consequential identity change follows its concrete decision route before implementation.

The foundation already separates immutable scalar/artifact preparation, selected supports/bindings, mutable worker scratch and fallible providers. Assembly uses packed second partials, narrows axes and serves lower orders from a successful higher order within a demand group. Implicit provider full/point/jet reuse, symbolic LU and factor scratch already exist. Preserve those strengths rather than introducing another cache/optimizer/AD engine. The gaps are intermediate-shape amplification, small arithmetic regions, cross-demand projections and broad implicit products.

## Target packages

<a id="b01"></a>

### B01 — Prepared census, sparse contributions and direct offsets

Attach a compact census to the actual immutable selected product: output/axis/support counts or references to existing support, scalar versus dualized instruction populations, stages/regions, frame/result capacities and actual evaluator implementation. Reuse information already constructed for admission; do not retain a second full support population for telemetry. A publishes these facts and retains them; math does not reinterpret diagnostic collection policy. Capacity, observed allocation, retained bytes and process RSS remain distinct.

Supply actual owner-local runtime facts by template/occurrence, requested/achieved order and demand: calls/completions/failures, cache hit/projection/miss and reason, provider/inner-implicit work and selected aggregate durations. Prepare keys/populations at construction and publish in batches through A01/A04; no default clock per scalar block or unbounded metric-cardinality expansion. Keep scientific charges with their existing owner. Migrate B04/B05/provider/cache producers, terminal/live records and E04 views. Overflow/reset, failed/cancelled upgrade, changed inputs/configuration, repeated occurrences and projection tests must distinguish attempted/completed/reused work without creating zero from missing facts.

Construct sparse addends from required support rather than enumerating absent coordinate pairs when a fitting support traversal exists. Keep candidate contributions, emitted addends and unique nnz distinct. Two independent local coordinates can alias one global diagonal, requiring both ordered cross terms; support-driven construction must not deduplicate required mathematics. Precompute valid output/row/term offsets to avoid callback-time binary searches/maps where current immutable structure already resolves them.

Requested-order frame poisoning and retained trial buffers already exist. Any further selective poisoning/read-before-write analysis must preserve stale branch/provider output detection and failure cleanup. Do not treat all current poison as unnecessary or copy an illustrative0.9MB cost into an acceptance requirement.

Affected owners include `jets.rs`, `execution.rs`, `assembly.rs`, library admission, compiler artifact descriptors and runtime retained-byte/census consumers. Test analytic sparse/empty supports, reordered coordinates, repeated outputs/rows, aliases and duplicate contributions. Independently validate derivatives, then observe eliminated candidate work. Check cancellation/limits and census accounting. Existing alias/stable-refill and support-upgrade controls are useful inputs, not newly executed evidence.

Length-k multi-indices are currently a library input. Reducing cubic metadata needs B03's product/representation decision; B01 does not invent a project derivative algebra. Census alone cannot resolve F05's known amplification.

**Remaining production alignment — 2026-10-17:** matrix terms have prepared result offsets, but `CaseWorker::result`, `objective`, `constraints` and `gradient` in `crates/pse-math/src/assembly.rs` still resolve immutable output ordinals by callback-time binary search; constraints also look up each row ID in the prepared row map. Finish the specified output/row/contribution offsets in these actual consumers, preserving selected lazy demands, reordered outputs, repeated contributions, aliases and scales. Completion removes the redundant structural lookups rather than merely exposing a prepared-offset API. Support-driven sparse construction, census and Jacobian/Hessian offsets remain implemented.

<a id="b02"></a>

### B02 — Compatible arithmetic regions and library joint optimization

Form bounded multi-output arithmetic regions during derived preparation after selected demand closure. Retain explicit inputs/outputs, occurrence/addend maps and semantic barriers. Symbolica owns CSE/Horner/joint optimization; the host owns fallible provider, branch, guard, applicability, cancellation and failure-location obligations. Prefer this simpler library composition to a parallel host value-numbering engine.

Preserve every relevant authored occurrence through many-to-one physical computation. `independent()` coordinates stay independent even when their values/expressions are numerically equal. A selected objective must not evaluate an unrelated constraint's bad guard. Determine which intermediate finite checks are promised behavior before fusion; algebraic identity alone does not establish equivalent finite-domain/failure behavior.

Use typed builder/execution/library admission as the preparation boundary, retaining authored semantics. If portable admitted-body reconstruction or artifact descriptions change, migrate their compiler/runtime/portable consumers here, not in a later inventory pass. Preparation growth, retained live temporaries and region cancellation granularity are bounded by the actual product owner.

Acceptance requires an actual jointly optimized compatible region plus revealing counterexamples: equal-valued independent arguments, selected/unselected bad guards, provider-dependent branches, nonfinite intermediate/result behavior and cancellation. Check nontrivial values/first/second derivatives independently. Count removed gathers/scatters/regions without claiming universal speed.

<a id="b03"></a>

### B03 — Lower the derivative product actually consumed

Settle Q01's representation choice under the math owner: selected partials, a weighted sparse Hessian and directional curvature are distinct products. Product kind, selected coordinates, support and relevant policy enter compiler-issued preparation/artifact identity and bounds. Weights and directions bind as runtime values where valid rather than causing symbolic recompile for each multiplier change.

Migrate the actual consumer closure together: AlgebraicOracle/CaseWorker weighted NLP curvature; runtime dynamic weighted products and IDAS curvature; existing directional Jacobian consumers; compiler ArtifactRequest construction; runtime/scalar artifact retention; and affected KKT/response/engineering-accuracy consumers. Native NLP still consumes a matrix; an HVP does not replace it. A weighted product does not supply arbitrary per-output Hessians.

Compare current packed seconds, Symbolica differentiation of weighted expressions/gradients, fitting library directional products and support-aware representations. Weights are constant under differentiation. Preserve objective sign, gather/physical scaling, coordinate order, repeated contributions and exact provider derivative chain. A plain scalar library external function is not the repository's fallible scientific provider. Unsupported product/provider combinations refuse explicitly; no silent finite differences or zero derivatives.

Choose a fitting route per supported product/control/provider shape. B02 improves visibility/economics but is not an intrinsic prerequisite. Required lower Taylor coefficients must not be removed merely because the final consumer asks for second derivatives. Establish construction bounds and exactness/policy before selecting a narrower representation. A supported negative result for one candidate selects another fitting route; it does not close known F05 amplification by instrumentation alone.

Acceptance includes independent weighted Hessian/actions with changed weights/directions at the same point, coordinate permutations, aliases, nontrivial providers/branches and unsupported orders. First-order requests keep unused second-order workers cold. Check complete provider/source/failure context and actual constructed versus consumed products.

<a id="b04"></a>

### B04 — Compatible projections across successful lazy demands

Extend the existing CaseWorker point-result owner rather than add a second upper cache. A successful richer result may serve a narrower demand only when exact consumed input bits, fixed bindings, provider/configuration premises, output/coordinate mapping, order, applicability and execution scope make the projection substitutable. `new_x` remains a hint, not the key.

Keep lazy misses: objective-only work must not eagerly compute All to populate a cache. Current higher-to-lower reuse within a group remains intact. Per-output jets may survive changed weights, whereas weighted matrices/actions must include the weights/direction they consumed. Failed/cancelled upgrades publish no partial completion or stale witness. Any retained lower product must remain independently complete under its original premises.

Migrate assembly/result projection, assembled native and dynamic/fitting consumers, rebind/reset and retained-byte accounting. Compiler changes are limited to genuinely changed prepared demand relationships. This package can project current packed products before B03.

Acceptance exercises callback-order permutations, signed zero, fixed input changes at unchanged free x, multiplier/direction changes, failed upgrades/cancellation and valid narrow requests with invalid unrelated providers/guards. Observe avoided evaluations without changing scientific charge semantics.

<a id="b05"></a>

### B05 — Narrow implicit Newton demand and IFT realization

Separate unknown-only first derivative demand for native Newton/JVP from qualified parameter derivative products at the selected root. All parameter values, domains and provider inputs remain bound even when their derivative columns are omitted from Newton. Update compiler implicit realization, math problem/provider and native implicit callbacks together.

Retain persistent symbolic LU/scratch and one current numeric factor serving first/second multiple RHS. Compare faer matrix contraction `TᵀHᵢT`, support-driven contraction and directional products for actual demanded width. Avoid dense public expansion when an internal packed/product route suffices; changing summation order requires declared numerical equivalence and derivative residual checks.

Preserve original residual assessment, singularity and inactive-bound neighborhood refusal, branch/selection verification, provider derivative order and derivative-solve residual verification. One independently qualified final-point evaluation may supply several products under an explicit owned contract; neither blindly repeating the same producer nor deleting assessment proves independence.

Basic B05 does not transfer a KINSOL/KLU factor to faer IFT. That requires C03/RC05's actual current-Jacobian capability covering point, layout, scaling and lifetime. Without it, retain separately verified faer factorization. Existing full/point/jet reuse still observes accuracy/configuration/order premises.

Acceptance uses first/second analytic IFT, nontrivial providers, singular/near-bound points, changed parameters/accuracy/configuration, same-point upgrades and cancellation. Record jets/extraction/factor/RHS/backsolve/verification/expansion separately when collection is selected. Distinguish numeric factor reuse within a derivative request from native session allocation reuse.

**Remaining production alignment — 2026-10-17:** `derivatives_at_verified_point` in `crates/pse-math/src/implicit.rs` still combines jet extraction, first/second RHS construction, support contraction, derivative residual verification and public expansion under `implicit.derivatives`. Numeric factor/solve/RHS-column/backsolve facts exist, but they do not distinguish those construction and verification stages. Complete the selected stage facts at the performing owner, including failed/cancelled stages and reused products; keep ordinary observation cheap and preserve independent derivative checks and charging. Unknown-only Newton, canonical IFT axes, support-driven contraction and same-request faer factor reuse remain implemented. This is missing production diagnostic behavior, not a request for additional performance proof.

<a id="b06"></a>

### B06 — Qualified optional derivative arithmetic compilation

After RC04 and a settled product/region identity/admission contract, extend the current Symbolica C++ compile/load owner to eligible derivative arithmetic. Dualized scalar evaluator instructions are compiled by the existing library; no second differentiation engine or whole-provider compiler is introduced. B02 is economically helpful but not a universal prerequisite.

Keep explicit conservative FP/ISA flags, actual compiler/link/load lifetime, child cancellation/drain, executable retention, per-worker scratch and unsupported-capability outcomes. Migrate compiler artifact keys/construction estimates, runtime retention and clone/reset consumers. A first-order-only demand must not compile unused second-order programs. Fallible provider/control boundaries remain host obligations.

Compare eligible arithmetic versus surrounding preparation/providers/checks; incremental compilation cost is amortized over actual artifact lifetime and observed demand. Do not claim break-even from an arbitrary N. Failed/cancelled compilation, independent cloned workers, owned child drain and artifact release join analytic derivative/nonfinite checks. RC04 is accepted: deliver the qualified optional capability for its established eligible scope. Optional use is not permission to omit the delivery. Unsupported shapes retain explicit interpreted realization without closing broader corrective obligations; evidence that defeats the accepted route returns for a concrete revised decision rather than silently completing B06 with the current interpreter.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B01 | Existing support/addend/census facts; common A fact slice; RC03 for changed product contract | Support-driven contributions, compact census, owner-local demand/cache/provider counters, prepared offsets and sound clearing; aliases/overflow/reset/limits checked; retire redundant scans/lookups | in progress |
| B02 | Selected demand and settled region semantic barriers; RC03 | Library-optimized compatible regions with full source/provider/guard/portable migration and independent value/derivative controls | done |
| B03 | Q01 capability/representation decision; RC03 | Product-specific weighted/directional/selected lowering and complete math/compiler/runtime/native consumers | done |
| B04 | Existing complete products and projection premises; RC03 if contract changes | Lazy cross-demand projections with exact point/configuration/weight identity and failure/reset controls; retire duplicate result reuse | done |
| B05 | Admitted implicit contract; RC03; C03 only for optional factor handoff | Unknown-only Newton plus qualified IFT realization and current-factor checks; no unnecessary dense expansion/contraction | in progress |
| B06 | Accepted RC04; actual product/region identity and bounded compile/load contract | Qualified optional derivative arithmetic compilation for eligible scope, equivalent outputs, own scratch/child drain and complete artifact consumers; unsupported shapes explicit | done |

B01–B06 are not a required serial chain. Shared assembly/execution/compiler/artifact editing requires coordination; C02 can remove transport work before B03, and F01 lane design needs exclusive worker/provider ownership rather than every optimization.

## Verification and checkpoint

**Interface-checked — production completeness, 2026-10-17:** B02/B03/B04/B06 match their selected target routes. B01 remains incomplete for ordinary callback offsets and B05 for the specified IFT stage diagnostics; the native packet table reopens both. No new numerical/profile choice is required. **Tested, earlier 2026-10-17:** the Outcome below retains acceptance of the implemented scope and supported inquiry branches. [43g’s Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns assembled/scoped acceptance, repaired composite boundaries, scope-end checks and exclusions. The packet table is the native progress owner; formal proposed ADR status does not assert or replace implementation acceptance.

### Contract clarification from independent review — 2026-10-17

RC03's selected product boundary preserves all original materialized values and ordered source/domain/derivative-eligibility guards, and finite checks on the complete requested product. It does not require constructing unrequested intermediate derivative jets or certify their availability. At positive x=1e-300, materializing u=sqrt(x) and requesting the Hessian of u²+x² can yield the valid contracted result2 even though an unused separate u″ exceeds binary64. Actual production BodyBuilder square roots retain Nonnegative Value and Positive First-order guards through lowering; x=0 remains ineligible for derivative products. The raw PreparedBody constructor used by tests is crate-private/test-only and does not infer missing guards. E02's explicit Value-check versus selected-product scopes retain this distinction. Independent review accepted this concrete interpretation after tracing the production guard path; analytic/interpreted/compiled controls remain required before acceptance.

Independent review also found that implicit final derivative programs could supply permuted formal axes even though IFT assumes unknowns then parameters. The constructor now requires canonical complete derivative axes (empty for Value); analytic first/second and permutation-rejection controls qualify the correction. This is a correctness repair, not a relaxation of verification or an alternative production path.

## Outcome (recorded after implementation)

### What was built

**Implemented — delivered scope; B01/B05 have the remaining production clauses above:** B01–B06 use support-driven sparse contributions and prepared matrix offsets, bounded joint optimization of compatible siblings, explicitly demanded selected/weighted/directional products, and lazy complete-product projection. Exact consumed weights/directions, point/fixed inputs, provider/applicability and configuration premises govern reuse. Unknown-only Newton and canonical unknown-then-parameter IFT programs are separate; current faer factors serve multiple IFT right-hand sides without claiming a KINSOL/KLU factor handoff. Eligible derivative arithmetic uses the optional owned Symbolica C++ compile/load route with conservative flags, independent admitted scratch and cold unused Second products. Runtime/Python deployment roots forward the same opt-in compiled-evaluator feature and report actual extension availability. Compiler/runtime/native/portable/artifact consumers use the same contract; no legacy broad-body alias or second derivative engine remains.

**Tested, 2026-10-17:** support/assembly controls (14 B05 and 12 assembly cases), 17 projection/product/math controls, nine compiled-derivative controls and two first-only/branch controls passed through the selected `just unit-package pse-math` routes with actual `compiled-evaluator`/native compiler capability where required. The final `just unit-package pse-math` compiled contract selection passed 9/9: independent implicit u=2p+p² at p=3 gives first8/second2, canonical-axis refusal, analytic/interpreted/compiled products, ordered guards and full singular-mode checks. G02's native runtime selection passed 76/76, including consumed-accuracy/refinement, derivative actions, fitting/shooting and original-corrector qualification. The compiler all-unit run passed 261/262; its single demand-ceiling fixture was corrected and the affected selection then passed 1/1. [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns commands, integrated/static results and exclusions.

**Interface-checked:** Q01 selects existing library product-specific lowering, support and bounded region optimization. Provider cross-demand projection stays ineligible without an established validity premise; pointer identity supplies none. Full materialized source values/ordered guards and finite complete requested products remain required. Unrequested intermediate jets are not separately constructed or certified. No all-context speedup or unexercised scientific-model claim is made.

### A mistake made and corrected

Independent review exposed permuted final implicit derivative axes despite IFT's canonical indexing assumption. Construction now rejects those axes and an independent nonlinear analytic control checks first/second derivatives. Actual Maturin execution disproved a metadata-only transitive-feature build assumption; explicit forwarding through existing dependencies now makes the optional route selectable without an artificial direct dependency. A compiler fixture also assumed every local demand inherits a case's Second ceiling; it now checks each projected support's real ceiling while still requiring the requested case ceiling and unchanged first/second support restrictions.

### Deviations from the plan, deliberate

Retain library-owned faer IFT factor/RHS reuse rather than an unqualified native factor handoff. Direct weighted/action completion preserves requested-product validity without fabricating unrequested intermediate derivative obligations. Existing opaque construction admission governs symbolic weighted construction; no invented full-square HVP bound or mandatory timing campaign was introduced. Formal ADR-0193 remains proposed; target authorization, implementation and test evidence are separate.
