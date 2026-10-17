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

**Interface-checked:** the integrated review, Symbolica 3.0.1/faer 0.24.4 source contracts and current `pse-math` typed/execution/jets/assembly/implicit/library owners plus compiler/runtime artifact consumers. Reuse that evidence; the local skill's Symbolica 3.0.0 baseline alone cannot qualify3.0.1 behavior. **Proposed:** all corrections/qualification. The coordinator records accepted RC03/RC04 and their bounds; any additional consequential identity change follows its concrete decision route before implementation.

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

<a id="b06"></a>

### B06 — Qualified optional derivative arithmetic compilation

After RC04 and a settled product/region identity/admission contract, extend the current Symbolica C++ compile/load owner to eligible derivative arithmetic. Dualized scalar evaluator instructions are compiled by the existing library; no second differentiation engine or whole-provider compiler is introduced. B02 is economically helpful but not a universal prerequisite.

Keep explicit conservative FP/ISA flags, actual compiler/link/load lifetime, child cancellation/drain, executable retention, per-worker scratch and unsupported-capability outcomes. Migrate compiler artifact keys/construction estimates, runtime retention and clone/reset consumers. A first-order-only demand must not compile unused second-order programs. Fallible provider/control boundaries remain host obligations.

Compare eligible arithmetic versus surrounding preparation/providers/checks; incremental compilation cost is amortized over actual artifact lifetime and observed demand. Do not claim break-even from an arbitrary N. Failed/cancelled compilation, independent cloned workers, owned child drain and artifact release join analytic derivative/nonfinite checks. RC04 is accepted: deliver the qualified optional capability for its established eligible scope. Optional use is not permission to omit the delivery. Unsupported shapes retain explicit interpreted realization without closing broader corrective obligations; evidence that defeats the accepted route returns for a concrete revised decision rather than silently completing B06 with the current interpreter.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B01 | Existing support/addend/census facts; common A fact slice; RC03 for changed product contract | Support-driven contributions, compact census, owner-local demand/cache/provider counters, prepared offsets and sound clearing; aliases/overflow/reset/limits checked; retire redundant scans/lookups | open |
| B02 | Selected demand and settled region semantic barriers; RC03 | Library-optimized compatible regions with full source/provider/guard/portable migration and independent value/derivative controls | open |
| B03 | Q01 capability/representation decision; RC03 | Product-specific weighted/directional/selected lowering and complete math/compiler/runtime/native consumers | open |
| B04 | Existing complete products and projection premises; RC03 if contract changes | Lazy cross-demand projections with exact point/configuration/weight identity and failure/reset controls; retire duplicate result reuse | open |
| B05 | Admitted implicit contract; RC03; C03 only for optional factor handoff | Unknown-only Newton plus qualified IFT realization and current-factor checks; no unnecessary dense expansion/contraction | open |
| B06 | Accepted RC04; actual product/region identity and bounded compile/load contract | Qualified optional derivative arithmetic compilation for eligible scope, equivalent outputs, own scratch/child drain and complete artifact consumers; unsupported shapes explicit | open |

B01–B06 are not a required serial chain. Shared assembly/execution/compiler/artifact editing requires coordination; C02 can remove transport work before B03, and F01 lane design needs exclusive worker/provider ownership rather than every optimization.

## Verification and checkpoint

**Proposed:** focused mathematical/compiler/provider fixtures plus touched-package compile checks; actual Arrow correctness closures use force-validation. Independent analytic/finite-difference comparisons are verification tools, not production derivative fallbacks. Native adapter and complete dynamic/fitting/optimization journeys are integrated by G02; selected census/work observations establish removed work without a mandatory timing campaign.

The full B packages remain open. Q01 choices and dependent rule decisions remain explicit; the supporting checkpoint below records the narrower A01/A04 observation delivery. Global F05/F06/F07 resolution remains with the coordinator and all contributors.


### Plan43a supporting-scope checkpoint — 2026-10-17

Compact prepared/realized census and separately admitted occurrence/demand/order observations, aggregate fallback, provider/implicit performed-work facts and selected census/demand records are implemented for A01/A04. Support-driven assembly, derivative representations, contraction and other B01–B06 algorithm changes remain open. [43a](43a-operation-evidence-and-coherent-live-diagnostics.md#a-execution-checkpoint) owns the selected execution checkpoint; [Plan43](43-solver-diagnostics-and-performance.md#current-checkpoint) owns finding disposition. [43a’s Outcome](43a-operation-evidence-and-coherent-live-diagnostics.md#outcome) records passing selected acceptance and its exclusions; package table states continue to describe each package’s complete scope, now authorized for execution.
