# Integrated Symbolica ecosystem functional improvements

2026-11-09. **Proposed:** extend the existing admitted mathematical-product owner into a coherent path from named scientific definitions to requested symbolic, derivative, numerical and analytical products. Use Symbolica, Numerica and Graphica for complete mathematical responsibilities where they fit; remove repeated preparation and superseded consumer machinery when those consumers move. Stable numerical operations and repeated-point execution are the most compelling execution extensions. Selected analytical inspection provides useful additional scientific/developer functionality from the same foundation.

This review implements the agreed functional-extension inquiry across all opportunities in the [Symbolica](../library_leverage/libraries/symbolica/README.md), [Numerica](../library_leverage/libraries/numerica/README.md) and [Graphica](../library_leverage/libraries/graphica/README.md) accounts. It builds on the earlier discovery and its integrated addendum research. It does not restart library discovery or require a diagnostic audit. The operator's priorities are less bespoke code, consolidated operations, execution and preparation efficiency, Rust build efficiency, and a better overall design. Interfaces and representations may change substantially to reach that target; preserving an incumbent stage is not a benefit by itself.

## Baseline, evidence and reading route

The inspected source baseline is `7d89d16f0805b7e169cd9cacc9cdbcd31ae9939e` with concurrent codegen, operations, schema, model and study-module work present. Representative current mathematical/compiler/runtime/structural consumers were inspected; this is not a full resolved-use census of the changing checkout. The supplied accounts retain their original evidence dates. Historical October assessments and November9 discovery artifacts are distinguished from current corroboration, especially around exact roots, supplier provenance and receiving. Discovery's failed resolved-reference capture remains a limitation, not evidence that an API is unused.

**Interface-checked:** selected current integration and supplier contracts, corroborated by the source references in each supporting assessment. **Proposed:** all changed behavior, deletion, integration and delivery scenarios in this review. No new product implementation, compilation, scientific test or performance experiment was performed. Expected reductions in reconstruction, repeated optimization, dispatch or local maintenance are qualitative; no runtime, memory or build improvement is **Measured**. Source tests cited below were inspected, not rerun. Publication checks do not qualify scientific behavior.

The [selected standard](../design_review/design_principles/standard.toml) is core3.4, efficient architecture heuristics1.0, process-simulator1.5 and the pse-arrow binding. The judgment emphasizes authority and explicit meaning (AP-03/04), library leverage and native composition (AP-07), execution/preparation/lifetime fit, and physical/derivative/numerical validity (PS-01/04/06/07/09–12). The standard shapes the proposed design rather than creating a new diagnostic checklist.

The maintained dependency family is Symbolica3.0.1 with the repository source override, Numerica3.0.1 and Graphica3.0.1. Tagged3.0.2 composer/import APIs are candidates with different preservation obligations. Companion integration, tensor-network and rich-document capabilities retain the narrower version/API confidence recorded in their source accounts. This review makes no assumption that a package name, exposed trait or newer release establishes equivalent runtime behavior.

| Supporting assessment | Main question and opportunities |
| --- | --- |
| [Preparation and lifecycle](symbolica-ecosystem/preparation-and-lifecycle.md) | Where should shared transformations, composition, supplier controls, receiving and representation lifetimes live? O01/O06/O07/O12. |
| [Numerical realization and repeated studies](symbolica-ecosystem/numerical-realization.md) | How can requested derivatives, stable operations, batches, numerical claims and expectation studies compose? O02/O03/O05/O10. |
| [Exact structure and certificates](symbolica-ecosystem/exact-structure.md) | Which exact products simplify current consumers, and where are sparse extraction or a specialized witness preferable? O04/O05. |
| [Structure and research products](symbolica-ecosystem/structure-and-research.md) | What do mapped instances, indexed algebra, topology exploration, analytical inspection and presentation enable? O08/O09/O11/O13. |
| [Supplied opportunity correspondence](symbolica-ecosystem/correspondence.md) | How every supplied opportunity family, including historical, conditional and rejected leads, contributes to the judgment. |

## Integrated target and existing strengths

The useful authority path remains authored physical definitions → admitted ordered mathematical region and original guards → requested mathematical product → eligible numerical or analytical realization → original-coordinate interpretation and protected result. Definitions, physical conditions and source occurrences remain meaningful across library representations. Library-owned transformations do not acquire authority over empirical validity, provider effects, physical coordinates or scientific conclusions.

The code already has a strong foundation: joint multi-output construction and direct evaluator translation, keyed scalar preparation shared by interpreted/native Value selections, named local partial definitions, selected/action derivative demands, bounded dynamic dualization, directed exact conversion, maintained bounded roots and selected native numerical execution. Symbolica itself already dispatches to Numerica polynomial/arithmetic kernels and Graphica indexed canonicalization. Reimplementing those operations or inserting a generic optimizer/cache/framework would add machinery.

The target develops this existing owner so compatible consumers reuse definitions and demanded products through their useful lifetime. An execution request changes values or seeds without rebuilding compatible structure. A changed guard, function meaning, layout, derivative support or provider premise invalidates the affected preparation. Exact, scalar, packed and analytical views may share a qualified source product without being forced into one representation or numerical claim. Preparation reuse never means treating distinct scientific results as identical.

## Recommended opportunities

<a id="O01"></a>

### O01 — Shared demanded mathematical preparation and selective module composition

**Recommend** completing the shared-product route at actual consuming paths. A developer should define local mathematics once and request selected outputs, partials, `Jv`, transpose actions or weighted Hessian products without creating independent flatten/differentiate/reconstruct/optimize paths. Current `exact_evaluator`, `ScalarCache`, `LocalPartials` and native replacement are the foundation. Preserve named definitions and use native `Transformer`, collection, replacement or alias discovery for useful complete passes; thin current callbacks need no extra abstraction.

For a genuinely reusable pure constitutive module, compare a joint build, retained host-called modules and `EvaluatorComposer`. **Conditionally recommend supplier composition** when independent consumers otherwise repeat symbolic reconstruction and Horner work. A joint evaluator remains better for inseparable work, and retained modules can have better invalidation behavior. `merge` checks parameter count rather than physical meaning; retained non-inline functions can prevent dualization, and real specialization must follow rebuilding. No global fusion or second evaluator DAG/cache is selected.

The delivery case is a repeated definition chain/diamond, selected reordered outputs and independently expected actions through an actual fitting/solver consumer. Changed values reuse preparation; changed guards/layouts invalidate it; cancellation retains no partial success. Remove the migrated consumer's duplicate construction. More detail and alternatives: [shared preparation](symbolica-ecosystem/preparation-and-lifecycle.md#o01--one-shared-preparation-owner-from-definitions-to-demanded-products) and [selective composition](symbolica-ecosystem/preparation-and-lifecycle.md#selective-native-composition-of-independently-reusable-programs).

<a id="O02"></a>

### O02 — Stable operations as explicit mathematical realizations

**Recommend** an admitted stable-operation vocabulary, starting with useful `log1p` and two-argument magnitude cases. An author should express the intended operation once and receive consistent requested values/derivatives in ordinary scientific consumers. Numerica's `Real::log1p` does not recover a tiny increment lost earlier by evaluating `log(1+x)`.

Use known Symbolica heads with explicit symbolic derivatives and `EvaluationInfo` numeric/C++ implementations through the mathematical registration owner. Retain dimension, domain, original guards and derivative singularities. Do not permit arbitrary per-model callbacks through the current formal symbol pool. A symbolic function body alone does not establish a stable numerical realization; blanket multiprecision is a weaker fit than representing the actual operation.

Demonstrate one authored relation through interpreted and selected compiled Value/First/Second products, with tiny signed increments, boundary/domain behavior and magnitude overflow/underflow cases checked independently. New function/derivative/kernel contracts need the applicable decision route. [Detailed realization, alternatives and rule impacts](symbolica-ecosystem/numerical-realization.md#o02--make-stable-operations-an-intentional-mathematical-realization).

<a id="O03"></a>

### O03 — One demanded-product batch operation with reusable dynamic shapes

**Recommend** ordered numerical-product evaluation over ordered independent points, retaining preparation and worker-local scratch at their existing owners. This serves property tables, pure residual populations, fitting and studies. Runtime seeds/weights remain inputs; requested axes/pairs, layouts and numerical profiles qualify preparation. Reuse the current dynamic ancestor-closed shape rather than making fixed generated hyperduals or another global registry the default.

Prefer extending the current explicit native C++ route to supplier `CompiledSimdRealEvaluator`/`BatchEvaluator` packing and scattering for eligible pure regions. Scalar execution remains an explicitly selected realization for small or unsupported work. Packed interpretation is a legitimate lower-startup alternative with additional packing/control-flow duties. Independent SIMD lanes and derivative components are different dimensions; whole-vector zero tests are not lane masks, scalar callbacks may remain scalar work, and tail padding can invoke callbacks on invalid zero inputs.

**Withhold the inspected real JIT batch adapter as the initial path:** its source unpack extent can exceed one four-lane vector for multiple outputs. This is a source-established contract concern, not a reproduced runtime failure or a rejection of packed execution. Qualify any repaired JIT route separately. Demonstrate ordered multi-output values plus an action/selected partial, uneven tails, per-point outcomes, independent scratch and cancellation with independent numerical oracles. [Complete alternatives and delivery cases](symbolica-ecosystem/numerical-realization.md#o03--prepare-demanded-derivative-products-once-and-evaluate-batches-as-one-operation).

<a id="O04"></a>

### O04 — Request-shaped exact structure and parameter-aware diagnostics

**Recommend** sharing compatible request-owned exact products and offering bounded exact-block inspection. A modeler could inspect conservation redundancy, inconsistency and parameter degeneracy with exact meaning, rather than interpreting numerical failure as an explanation. Symbolica `system_to_matrix` retains parameter-dependent rational-polynomial coefficients; native matrix/field operations supply the stronger algebraic operation.

Keep current sparse derivative-based affine extraction for selected global-coordinate evidence where it is cheaper and sufficient. **Conditionally recommend selected small-block elimination**, with exceptional parameter cases, original domains and inverse coordinate recovery explicit. Automatic broad symbolic presolve is not the default. Fraction-free or modular techniques become useful when the selected coefficient domain and growth warrant them; no second general algebra engine is needed.

Retain symmetric-pivot LDLᵀ/Gram witness semantics for convexity. Thin `sub_mul_assign` consolidation may simplify exact updates, but its general rational implementation does not justify a fused speed claim. Rank or a general solve cannot replace the consumed certificate. Demonstrate ordinary and exceptional parameter blocks, inconsistency versus underdetermination and a sparse countercase; actual compatible consumers must demonstrate deleted extraction. [Detailed comparison and certificate boundaries](symbolica-ecosystem/exact-structure.md#o04--request-shaped-exact-products-and-explainable-preprocessing).

<a id="O05"></a>

### O05 — Numerical assurance with explicit claim strength

**Recommend** a selected diagnostic operation distinguishing ordinary results, chosen-precision estimates, propagated error estimates, exact identities and supported validated enclosures. Reuse one exact-to-binary64 conversion owner and the current maintained bounded-root path. Existing directed rational conversion and root selection are already substantial leverage.

Map an admitted product to `DoubleFloat`, multiprecision or `ErrorPropagatingFloat` for diagnostics, and use balls/directed operations only within their certified arithmetic envelope. A transcendental trait method is not a validated enclosure contract. Refuse unsupported certification explicitly; do not replace full FBBT or every solver's binary64 semantics. Approximate rational candidates retain changed-value status, and a simple rational inside an enclosure is not proof of identity.

Keep admitted midpoint root refinement unless a new interior-search operation establishes contraction, limb and cancellation properties. `round_in_interval` may return an endpoint and does not guarantee halving. Demonstrate distinct claim meanings, a rounding boundary, supported enclosure, unsupported transcendental request and original-domain refusal. [Numerical diagnostics](symbolica-ecosystem/numerical-realization.md#o05--offer-claim-scoped-numerical-diagnostics-from-the-same-admitted-mathematics) and [root/conversion qualifications](symbolica-ecosystem/exact-structure.md#o05--assurance-that-states-the-claim-not-merely-the-number-type).

<a id="O06"></a>

### O06 — Supplier-owned checked construction and per-operation control

**Recommend** moving algorithm-private vectorization/export bound knowledge and callback rebinding into maintained supplier entry points with operation-specific extents and per-operation cancellation/control. The application continues to own admitted scientific work, resource lifetimes, interpretation and publication. This directly reduces version-coupled bespoke code while preserving useful refusal behavior.

Current preflight predictors, realized-product reconciliation and abort handling fill real API gaps. Retain them until checked supplier replacements cover actual callers. Public size observations inspect built products; RSS/time polling is a backstop rather than pre-allocation admission. No new application budgeting framework, workstation cap or mandatory process-isolation layer is recommended.

Deliver one caller cutover with insufficient allowance, cancellation during construction, successful-but-truncated supplier outcomes and no partial retention. Remove corresponding mirrors/rebinding only when equivalent control is consumed. [Controls, alternatives and rule impacts](symbolica-ecosystem/preparation-and-lifecycle.md#o06--supplier-owned-construction-controls).

<a id="O07"></a>

### O07 — Receiver-qualified transfer of the needed mathematical product

**Recommend** separate qualified derived-product profiles for numerical-only execution and guarded symbolic continuation. A cold receiver should avoid producer construction where the transferred product safely supports its next activity. Existing `F64ProgramView::inspect/restore_with` is a numerical execution foundation; symbolic derivatives/class evidence need sealed normalized stages with original recipe/proof custody and receiving checks.

Typed replay remains a sound alternative when stronger receiving cannot preserve meaning or save meaningful work. General Atom import and numerical instruction import do not by themselves establish bounded symbolic continuation. Retire replaced replay/encodings for migrated consumers after receiving qualification; do not keep a permanent dual-path fallback merely for compatibility.

Delivery includes producer retirement, cold receiving, foreign/reordered symbols, missing receipts, incompatible extents/layouts, refusal before registration and independent original-coordinate results. This overlaps existing Plan66 receiving work without changing its status. [Transfer comparison and receiving cases](symbolica-ecosystem/preparation-and-lifecycle.md#o07--transfer-the-product-the-receiver-actually-needs).

<a id="O08"></a>

### O08 — Mapped instances and native indexed mathematical structure

**Recommend** reusing definition-led mathematical templates with explicit parameter/input/output/derivative-axis transport. Repeated physical instances retain their own source/guard/provider meaning while sharing compatible preparation. This extends O01's existing named-definition authority. It does not deduplicate scientific results or replace semantic model identity with topology.

**Recommend native indexed canonicalization as a bounded analytical product** for genuine dummy-index equivalence. Symbolica's `canonize_tensors` already constructs the Graphica graph and reconstructs mathematics; use it before implementing a separate index walker or motif graph. Physical species, ports and external coordinates cannot be renamed as dummy indices.

**Conditionally retain graph-assisted matching** for independently authored equivalent motifs beyond existing definition reuse. Role-colored incidence vertices can preserve port/edge roles despite Graphica's missing canonical edge map, but equal canonical graphs are candidate correspondence, not physical equivalence proof. A whole-flowsheet canonical cache adds conversion/search/eligibility cost and is not selected. Spenso contraction planning is conditional on an actual structured contraction workload, not equality alone. [Mapped reuse and countercases](symbolica-ecosystem/structure-and-research.md#mapped-mathematical-template-reuse), [indexed operation and demonstration](symbolica-ecosystem/structure-and-research.md#o08--native-indexed-mathematical-canonicalization).

<a id="O09"></a>

### O09 — Finite topology exploration as an opt-in research operation

**Recommend a bounded topology-candidate workflow** for a selected finite process family. Graphica supplies constraint-driven generation and isomorphism deduplication; ordinary typed authoring/admission, structural analysis and class-specific science determine candidate feasibility. This enables a useful research activity without inventing another graph enumerator or universal process synthesizer.

Select explicit equipment/vertex bounds and completed-candidate inventory checks. Signature reuse is not an exact equipment count; `max_loops` alone does not bound degree-two subdivisions. Retained result maps, polling points, bridge/root conventions and lack of named-port order constrain the admitted envelope. Report incomplete enumeration, scientific rejection and unsupported candidates honestly. Graph symmetry counts are neither probabilities nor proof of scientifically identical trials.

A tiny independently known candidate population, open and closed examples, exact inventory, role-sensitive parallel edges, physical-invalid candidates and interrupted partial output demonstrate the capability. A large economics-guided search may instead favor a native mixed-integer superstructure formulation. [Full workflow and alternatives](symbolica-ecosystem/structure-and-research.md#o09--finite-topology-exploration).

<a id="O10"></a>

### O10 — Explicit-measure adaptive expectation studies

**Recommend a distinct expectation/integration study outcome** composed with existing model case execution and O03. A user declares quantity-typed uncertain coordinates, a probability measure/transform, selected outputs, an estimand and stopping/replay policy. Existing DOE generation and local covariance propagation do not already specify that probability law.

Use Numerica continuous/discrete nested grids, sampling weights, stream state and `StatisticsAccumulator` for adaptive integration, with compatible worker snapshots and declared merge/update barriers. Start with a bounded continuous/categorical family; dependent laws need a joint transform. Retain failed members and original provenance. A seed alone does not imply bitwise parallel reproducibility, and sampling uncertainty is not a deterministic scientific enclosure.

Fixed-weight Monte Carlo remains a simpler eligible realization and cross-check. Native ODE/DAE integration remains a different responsibility; differentiating an adaptive sampler does not establish sensitivity of an expectation. Deliver analytic continuous/categorical means, measure normalization, retained failure and replay cases. [Scientific interface, execution composition and delivery](symbolica-ecosystem/numerical-realization.md#o10--add-explicit-measure-adaptive-expectation-studies).

<a id="O11"></a>

### O11 — Analytical scientific inspection from admitted definitions

**Recommend** selected derivatives, remainder-aware local series and exact coefficient/identity checks as source-linked developer/scientific results. A caloric-law developer could inspect whether an enthalpy primitive differentiates to the selected heat-capacity law, preserve reference constants, and inspect behavior near a reference state without reconstructing names and premises in an external notebook.

Use native derivative/series/polynomial operations over the existing qualified product; retain series order objects, physical context, assumptions and unsupported regions. Do not make CAS output a second authoring authority or promote a formal series order to a uniform numerical error certificate. Symbolic integration, Padé/interpolation, PSLQ/lattice candidate recovery and richer special functions are explicit conditional subproducts with their own exactness/domain/backend obligations.

Integration companion compatibility and bounded behavior remain unresolved, so it is not a dependency of the initial analytical operation. Native indexed inspection from O08 and presentation from O13 fit the same consumer. Deliver a caloric derivative/series case, source evolution and unsupported region, with exact identity distinguished from empirical validity or approximation. [Full proposition and analytical alternatives](symbolica-ecosystem/structure-and-research.md#o11--selected-analytical-mathematics-as-a-developer-result).

<a id="O12"></a>

### O12 — Build and memory improvements at actual representation boundaries

**Recommend** keeping the narrow mathematical integration and opt-in native codegen, reducing reconstruction and retained representation work through O01/O03/O07. Separate Rust clean/incremental compilation, symbolic preparation, native source compilation, transfer and repeated evaluation when judging a change. A smaller wrapper or different import spelling does not prove a smaller build.

Direct Numerica use is useful at a genuine numerics-only owner, but the present math crate still depends on Atom/evaluator APIs. No speculative crate split, arithmetic-backend migration or fixed-dual specialization is selected. Borrowed views, factorized representations, thread-local polynomial context ownership and compaction are targeted options where their lifetime fits. `TermStreamer` is conditional on a real large additive operation; spilling and global merging can add work for coupled algebra.

A future boundary proposal must show which dependency units or retained products actually disappear and preserve their consumers. Benchmark only when a serious regression concern makes the result decision-relevant; no generic benchmark campaign is imposed. [Build/memory alternatives and revisit conditions](symbolica-ecosystem/preparation-and-lifecycle.md#o12--coherent-build-memory-and-runtime-boundaries).

<a id="O13"></a>

### O13 — Native mathematical presentation with physical interpretation

**Recommend** native Symbolica printer projections for derived mathematical views, joined to original source/physical labels at the existing reader. Keep DSL round-trip authoring with its own renderer. Native LaTeX/Typst options avoid another mathematical pretty-printer, but neither presentation string is identity, a lossless scientific codec or verification.

Typst native structured modules remain conditional on a richer annotated-report consumer that justifies the dependency and attachment/callback custody. WASM and Python bindings remain deployment/reference alternatives rather than changes to native product science. Demonstrate source/expression correspondence, namespaces, units/datum and honest unsupported fields using the protected original product. [Presentation recommendation and alternatives](symbolica-ecosystem/structure-and-research.md#o13--native-mathematical-presentation).

## Priority, dependencies and selection boundaries

The primary consolidation/execution direction is **O01 + O02 + O03**, with definition-led O08 binding transport where repeated instances need it. These reduce repeated preparation and improve the expression and execution of actual requested science. O02 can establish scalar behavior before packed implementations; O03 can initially use existing eligible pure functions. Neither requires a new topology, expectation or tensor-network workflow.

**O06 and O07 are focused lifecycle simplifications**, especially valuable where current private supplier coupling and receiver replay are substantial. A specific new composition/packed/receiving path must preserve the controls it consumes. This is not a reason to block every mathematical improvement on a universal supplier upgrade. Maintain old admission only while it still supplies a necessary contract; remove it with its verified replacement.

**O04 and O05 broaden explainability and exact/numerical usefulness** while preserving current strong narrow algorithms. Their shared original-product and claim interfaces fit O01; neither requires blanket dense extraction or multiprecision. Thin exact arithmetic changes may belong inside an ordinary selected implementation rather than a standalone feature.

**O11 with O13 is the strongest additional developer/scientific workflow**: useful analytical results with faithful presentation, bounded by real library support. O08 indexed canonicalization can be selected within it. **O10 and O09 are separate useful study/research outcomes** with more new scientific request/result semantics. They can consume the shared foundation, but should be selected for their own value and integration burden. They are not prerequisites for simplifying current execution.

**O12 constrains all these choices.** Prefer fewer conversions, coherent lifetimes and useful native operations before adding crate boundaries, global caches or spill infrastructure. Conditional Spenso, Typst, integration, GPU/JIT, alternate arithmetic backends and graph matching retain concrete reopen conditions in the supporting assessments. Specialized community physics wrappers have no established process-science adoption value from this evidence; licenses exclude none of them.

## Relationship to current work and consequential decisions

[Plan66](../plans/66-integrated-simulation-target.md#symbolica-improvement-dispositions) and [its mathematical owner](../plans/66d-mathematical-representation-and-numerical-execution.md) retain all current scheduling, implementation and acceptance state. Shared products, existing root/conversion integration, numerical receiving, selected derivative demand and native views already overlap that work. Existing implementations are preservation foundations; this review does not reschedule them as new deliveries or turn remaining acceptance into an absent feature.

Selection should identify the actual delta against those owners: a remaining consumer migration/deletion, a changed realization or receiving contract, or a genuinely new diagnostic/study/research outcome. Further planning can integrate selected deltas into the owning plan. No separate library-applicability bureaucracy or duplicate status register is needed. This review itself changes no plan, dependency, primitive vocabulary, scientific authority or production code.

Known mathematical heads and derivative/kernel eligibility, preparation identity, sealed receiving, numerical claims, authored measures/estimands, indexed shape semantics and topology-candidate interpretation may require changes to enduring contracts. Breaking changes are acceptable, with affected consumers and superseded machinery included. Use the existing ADR/design route where a selected change alters an architectural decision or adds a kernel/binding contract. Routine API use and local arithmetic simplification do not themselves justify new governance.

Delivery evidence in each assessment describes what would show a useful capability and preserve its science/lifecycle. It is not a claim those checks have passed. Selected implementation should exercise an actual consumer, independent mathematical expectations, original-domain failures and interruption where material, then remove the machinery it replaces. Quantitative improvement claims require measurement; plausible qualitative simplification does not require inventing a benchmark obligation.

The [correspondence](symbolica-ecosystem/correspondence.md) accounts for all 22 supplied account documents by meaningful lead family, including low-priority and historically rejected directions. There is no adoption quota and no requirement to turn every native API into a product feature. The review is ready for selection and implementation planning without a further discovery pass; narrow contract questions remain attached to the particular optional realization they affect.
