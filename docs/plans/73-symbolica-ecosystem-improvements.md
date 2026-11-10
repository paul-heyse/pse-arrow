---
title: "Plan73: Symbolica ecosystem improvements"
status: in-progress
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73-symbolica-ecosystem-improvements.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Symbolica ecosystem improvements

## Target, scope and execution authority

**Proposed.** One physically admitted mathematical definition supplies compatible requested exact, derivative, numerical and analytical products. Repeated consumers reuse qualified preparation rather than flattening, differentiating and rebuilding it independently. Stable numerical operations retain their identity through derivatives and execution. Supplier-owned checked operations replace algorithm-private application mirrors. Numerical and symbolic receivers obtain the product needed for their next activity. Scientific users gain exact/numerical diagnosis, analytical inspection, explicit-measure expectation and finite topology exploration through existing product owners.

The maintainer selected this new series to execute **in full before returning to Plan66**. This instruction supersedes the source review's suggestion to leave overlapping implementation ownership in Plan66. This series owns the structural improvements, affected consumers, deletions and required acceptance described here, including overlaps. It does not wait for unfinished Plan66 packages. Existing plans are not amended now. Plan66's broader physical populations and unrelated acceptance remain outside this series; H supplies a factual handback for its later reconciliation.

The input is the [functional review](../functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md), its five supporting documents and all 22 files in the [Symbolica](../library_leverage/libraries/symbolica/README.md), [Numerica](../library_leverage/libraries/numerica/README.md) and [Graphica](../library_leverage/libraries/graphica/README.md) accounts. The review's [correspondence](../functional_reviews/symbolica-ecosystem/correspondence.md) defines the detailed lead coverage; the opportunity titles alone are not the scope. Comprehensive overviews orient further investigation but do not make every described API mandatory adoption.

All affirmative O01–O13 outcomes are selected for implementation planning. Conditional mechanisms receive finite decision work and implementation of the selected branch, or an explicit no-adoption disposition with a reopen condition. Completing an investigation means recording and integrating its consequence; “investigate later if useful” cannot close it. Historical rejected approaches are preserved judgments rather than automatically reopened implementation mandates.

Breaking interfaces, representations and component boundaries are acceptable. The maintainer explicitly confirmed targeted maintained-supplier changes on 2026-11-09. Consumer migration and removal of replaced machinery belong to each delivery. Authoring this draft does not implement the product or claim scientific qualification.

## Reading the implementation scope

Each companion begins with its concrete deliverables and maps them to linked scope IDs. Each ID owns a dedicated implementation section describing the existing components or shared mechanism changed, target dataflow, pivotal library behavior, actual migration/deletion and completion evidence. Tables are navigation and status indexes. They are not substitutes for the implementation descriptions.

A–D primarily change or consolidate existing mathematical preparation, assembly, supplier controls, execution and exact/numerical pathways; they also add the explicitly selected diagnostic capabilities. E–G add selected analytical/research capabilities through existing scientific/operating owners, with their required cross-cutting changes and migration stated separately. H owns bounded build decisions, assembled qualification and the factual handback. An example scientific model is acceptance evidence, not an excuse to leave the actual changed components unspecified.

The current baseline already has reusable bodies, `InstanceBinding`, scalar-family retention and several shared derivative/exact paths. Existing use is preserved and extended; the changed scope is the actual remaining construction, transport, supplier integration and receiving machinery. No package may create a second equivalent mechanism merely to satisfy an opportunity title. Conditional mechanisms remain finite decisions with a named case and implementation consequence.

## Baseline and focused foundation assessment

**Interface-checked.** Source baseline `baadc7e3272d9e6d80f4ea1dbeda2f44f701318b`, with concurrent operations/codegen/schema/study changes present. The source review inspected `7d89d16f`; refreshed focused inspection covered math preparation, derivative products, native export, transfer, compiler physical formals, study sampling/uncertainty and structural graphs. No fresh exhaustive resolved-use census or runtime measurement was obtained. Do not label representative spelling/source reads as proof of absence elsewhere.

The dependency owners select maintained Symbolica3.0.1 with the local source override, Numerica3.0.1 and Graphica3.0.1, with GMP/MPFR and explicit optional native code generation. Manifest/lock/source declarations remain authoritative. Tagged3.0.2 composer/import APIs and companion packages have separate evidence and preservation requirements. A version bump alone does not supply the maintained bounded roots, packing, transfer, provenance or operation-control contracts.

Existing foundations are dependencies, not new projects:

- `exact_evaluator` already uses aliases, joint multi-output construction and direct instruction-DAG translation; `ScalarCache` shares exact preparations across interpreted/native Value profiles. Extend their actual remaining consumers rather than introducing another optimizer/cache.
- Dynamic derivative layouts, local partial definitions and selected/action demands already exist. Exact request-local objective analysis and coefficient extraction already share useful products. Keep axes, selected pairs and original guards explicit.
- Direct requested-product evaluation is a legitimate distinct realization: a finite weighted square-root result may have an overflowing staged intermediate Second derivative. Delete only construction that its replacement actually supersedes.
- Directed exact-to-binary64 conversion, bounded root isolation and checked f64 numerical transfer already delegate substantial work. Their remaining caller/receiver obligations must not be confused with absent mathematics.
- Current scalar export admits a narrow builtin vocabulary. Stable external heads need registration, derivative, checked export and receiving support together. Current evaluator `map_coeff` does not accept ball types merely because Numerica implements arithmetic for them.
- Existing model operations, study lanes, result custody and independent readers support new capabilities. No CAS service, new workflow engine or second scientific authority is selected.

The focused assessment applies core3.4, efficient architecture heuristics1.0, process-simulator1.5, applicable test guidance and the [repository binding](../design_review/design_principles/standard.toml). It supports this bounded plan; it is not a broad diagnostic audit or a qualification verdict for surrounding subsystems.

The package-detail revision additionally inspected source at `e9f4e7b7c`, preserving concurrent product work. It explicitly reconciles existing instance/body reuse, scalar retention, shared exact objectives and current receiving profiles before describing the remaining changes. Earlier source observations retain their dates; no source inspection is labeled product qualification.

## Combined design and contract ownership

The authority chain is authored physical definitions → resolved ordered inputs/definitions and original guards → requested mathematical product → selected eligible realization → original-coordinate interpretation → protected result and useful reader. Symbolica/Numerica own algebra and differentiation; Graphica supplies structural algorithms. The application owns scientific meaning, admissibility, effects and custody. Native solvers retain iteration and class-specific solution interpretation.

A mathematical product qualifies definitions, coordinate bindings, demand, derivative support, numerical policy and consumed provider premises. Changed values/seeds/weights reuse compatible preparation; changed structure or premises invalidate it. Reuse of preparation is not reuse of scientific results. Immutable prepared backing may be shared; mutable execution scratch belongs to the worker. Exact and approximate products retain distinct claims and lifetimes instead of being forced through one scalarized representation.

A owns the shared-product/binding explanation; B owns supplier operation control and receiving; C owns stable-operation and batch profiles; D owns exact/assurance claims; E owns analytical/indexed artifacts and presentation; F/G own new study/research meaning; H owns build decisions and combined qualification. Shared contract changes are reconciled at those owners before dependent packages integrate, not repeated as competing declarations.

| Document | Responsibility and principal opportunities |
| --- | --- |
| [73a · Shared mathematical preparation](73a-shared-mathematical-preparation.md) | O01, mapped O08, representation/lifetime aspects of O12. |
| [73b · Supplier controls and qualified receiving](73b-supplier-controls-and-qualified-receiving.md) | O06/O07 and supplier compatibility for composition and new profiles. |
| [73c · Stable operations and batch execution](73c-stable-operations-and-batch-execution.md) | O02/O03, from typed operations through scalar and packed consumers. |
| [73d · Exact structure and numerical assurance](73d-exact-structure-and-numerical-assurance.md) | O04/O05, including exact diagnostics, numerical claims and certified-arithmetic bridge. |
| [73e · Analytical products and native presentation](73e-analytical-products-and-native-presentation.md) | O11/O13 and indexed O08; bounded companion decisions. |
| [73f · Explicit-measure expectation studies](73f-explicit-measure-expectation-studies.md) | O10, statistical meaning, adaptive native execution and original result reading. |
| [73g · Finite topology exploration](73g-finite-topology-exploration.md) | O09, finite generation, typed interpretation and scientific screening. |
| [73h · Build boundaries and integrated acceptance](73h-build-boundaries-and-integrated-acceptance.md) | Cross-cutting O12, assembled qualification, enduring contracts and Plan66 handback. |

## Decision routes and preserved rules

The maintainer's selected direction permits substantial structural and supplier changes; it does not waive physical correctness, original guards, explicit effects, scientific evidence strength or admission before uncontrolled construction. No waiver of those rules is requested by this design.

Keep formal/provider-lift pool symbols callback-free. Known mathematical heads have a separate deterministic initialization/registration owner, or supplier builtin support, rather than weakening the pool. C settles the complete known-head realization and B its checked construction/receiving vocabulary. New mathematical primitives and numerical profiles use the applicable kernel/binding ADR and architecture route before dependent implementation. Existing interpreted defaults and conservative floating settings are not silently changed by enabling a compiled feature.

A/B changes to preparation identity or producer/receiver custody, D claim semantics, F authored measure/estimand meaning, and G candidate interpretation are enduring contracts. Each companion identifies its decision route before implementation. An actual D1–D14 alteration, new crate, major dependency family move or rule departure requires its existing review/ADR route; routine dependency changes or use of an API do not. If execution exposes a genuine rule departure, present the specific rule and consequence for operator confirmation before depending on it. Generic “breaking changes allowed” is not a blanket certification waiver.

## Opportunity dispositions

| Opportunity | Disposition | Work owner | Evidence or revisit trigger |
| --- | --- | --- | --- |
| O01 | scheduled | 73a; 73b supplier compatibility | Shared demanded preparation and actual caller deletions; finite composition comparison integrated. |
| O02 | scheduled | 73c; 73b control/receiving support | Stable operations and requested derivatives through authored, interpreted, compiled and receiver paths. |
| O03 | scheduled | 73c | Reused dynamic demand and genuine supplier batch execution with original point outcomes. |
| O04 | scheduled | 73d | Shared exact diagnostic products and selected elimination decision; sparse/witness countercases preserved. |
| O05 | scheduled | 73d | Explicit numerical claims and supported certified arithmetic, with existing converter/root authority retained. |
| O06 | scheduled | 73b | Checked supplier operations consumed by callers; corresponding local mirrors/rebinding removed. |
| O07 | scheduled | 73b | Production numerical and symbolic receiving, migrations and old-path retirement. |
| O08 | scheduled | 73a mapped reuse; 73e indexed mathematics | Bindings transported with original premises; native indexed inspection; graph matching and Spenso decisions. |
| O09 | scheduled | 73g | Finite candidate family and ordinary admission/science/reader, including incomplete generation. |
| O10 | scheduled | 73f | Explicit measure/estimand, adaptive estimator, failure semantics and replay. |
| O11 | scheduled | 73e | Source-linked derivative/series/identity outcomes and finite analytical-extension decisions. |
| O12 | scheduled | 73a lifetimes; 73h build decisions | Remove actual repeated representations; close build/streaming/backend decisions without speculative speed claims. |
| O13 | scheduled | 73e | Native presentation through protected readers; finite richer-report decision. |

### Detailed lead ownership and completion

The following table complements the source correspondence rather than replacing its technical caveats. Every lead family has one decision home. Where a decision is negative, retain its reason and observable reopen condition at that home; no separate mutable feature backlog is introduced.

| Source directions | Owner and required treatment |
| --- | --- |
| Visitors, collection, replacement, Transformer, ReplaceBuilder, alias discovery and symbol properties | A: use complete native passes for demonstrated remaining repetition; retain thin callbacks otherwise. Properties describe mathematical heads, not invocation policy. |
| Joint evaluator, merge, composer, inlining, real flags, stack reuse | A/B: named independently consumed chain; compare joint/host/composed routes and integrate chosen ordering/compatibility. |
| Stable functions, numeric callbacks, symbolic derivatives, native definitions | C/B: one supported vocabulary spanning scientific admission, derivatives, control and receiving. |
| Dynamic/fixed duals, factorial conventions, independent lanes, packed branches/powers/callbacks/tails | C: dynamic default, x4 C++ initial packed profile; bounded fixed/JIT/device decisions with numerical eligibility. |
| Rational-polynomial matrix extraction, rank/solve families, fraction-free/sparse/modular/CRT/reconstruction | D: unknown/parameter-aware exact product and elimination decision; avoid automatic dense/global conversion. |
| Schur updates, owned exact division, bulk arithmetic and polynomial/geometric kernels | D: thin consolidation where an actual caller benefits; retain supplier dispatch and LDLᵀ/Gram witness meaning. |
| Solution sets, root coverage, algebraic extensions, generic/exceptional parameters | D/E: diagnostics and faithful analytical results with claim/domain limits; not universal solver replacement. |
| Balls, directed arithmetic, precision/error propagation, conversion, exact roots/cache/refinement | D/B: actual certified-arithmetic bridge, one conversion owner, current bounded roots; do not revive historical fallback ladders. |
| Rational interval points, denominator truncation/rationalization, approximate nsolve | D/E: explicit exact-search/approximate research meaning; no silent coefficient rewrite or false contraction claim. |
| Cooperative abort, private predictors, public size facts, RSS/time controls | B: supplier-owned checked operations and consumed cutover; observations/backstops do not replace preconstruction admission. |
| Atom/state/packing, structured instructions, numerical restore, sealed continuation | B: receiver-specific products, original custody and production qualification; generic import is not sufficient. |
| Borrowing, polynomial contexts, compaction, streaming and parallel transforms | A/H: actual lifetime/representation decision; no blanket streaming, new scheduler or workstation cap. |
| Companion imports, arithmetic backends, fixed specialization and Rust build closure | H: genuine dependency-unit or workload evidence; changing import spelling alone removes no closure. |
| Canonical maps, edge/port incidence, hidden metadata, automorphisms and physical bindings | A/G: mapped templates first; graph matching only for incremental reuse beyond definitions, with owner validation. |
| Native tensors, index groups, Spenso storage/contraction | E: native indexed canonicalization selected; bounded actual-contraction comparison before Spenso adoption. |
| Generation, signatures, inventories, filters, progress/abort/retained maps, roots/bridges, connectivity | G: finite family and completion semantics; no probability/flow interpretation inferred from topology. |
| Continuous/discrete grids, weighted samples, RNG, statistics and merge | F: frozen iteration semantics and declared measure; distinguish DOE, covariance and statistical uncertainty. |
| Combinatorics, borrowed iterators, owned permutations and multiplicity | G/E: embed only where output/count semantics fit; no combinatorial service. |
| Series/order, interpolation/Padé, PSLQ/lattice, integration and special functions | E: analytical products first; finite extension decisions and exact/independent verification. |
| Symbolica-ODE and specialized community physics packages | E: explicit no replacement/adoption on present evidence; reopen for a named process-science outcome requiring their distinct capability. |
| Ring printers, WrappedRingElement, native LaTeX/Typst, rich Typst modules | E: native derived views selected, richer structured report decision; no identity/persistence role for formatting. |
| WASM/Python wrappers and graph DOT/Mermaid helpers | E/G/H: projection/reference conveniences for an actual consumer; no new product language/deployment stack from availability alone. |

## Dependency order and complete-operation delivery

Begin with A's actual remaining consumer/contract map and B's supplier control/receiving decisions, coordinating C's stable vocabulary and D's arithmetic bridge. These are shared design inputs, not a requirement to finish all of A/B before other work starts. Establish the minimum working checked product before a dependent caller cutover; an agreed interface alone is not an implemented prerequisite.

C can land stable scalar behavior before packed execution. D can deliver exact inspection independently of new compiled heads. E's initial caloric derivative/series result can consume existing admitted products while later indexed/companion extensions proceed. F starts with ordinary scalar execution and consumes qualified C batching afterward. G uses existing typed model/science capabilities and template reuse where available. Neither research workflow blocks simplifying current mathematical execution.

Every delivery includes the actual operation/client/result changes needed to use it before Plan66 resumes. Do not defer readers, protected receiving, source/physical projection or supplier API adoption to an unrelated “integration later” phase. H assembles evidence across delivered paths; it does not repair missing consumer ownership by declaring foundation tests sufficient.

### Plan66 overlap and later handback

| Existing context | Work taken into this series | Boundary of the later handback |
| --- | --- | --- |
| 66d D01/D02/D05 | Remaining shared mathematical products, requested derivatives, stable/batch realization, duplicate caller retirement and their acceptance. | Do not claim every remaining G/H/I/J physical demand or family qualified. |
| 66c C04 and 66d receiving | Selected production numerical and symbolic receiving, custody/admission, actual cold consumers and encoding/replay retirement. | Do not claim unrelated execution-context/service populations complete. |
| 66a scientific operation and product meaning | New analysis, assurance, expectation and candidate request/result semantics and the consumers needed here. | No blanket completion of all experiment/member permission populations. |
| 66j uncertainty/study context | Explicit-measure expectation and its scientific/operating consumer. | Existing fitting, covariance, control and wider configured populations retain their scope. |
| 66k native views | Expression/source/physical result projection, protected readers and mathematical rendering needed here. | No blanket completion of costing/environment/utility or all native views. |

H records delivered capabilities, source changes, tests and residual wider obligations. Plan66 is reconciled only after this series executes, under the maintainer's later instruction. Its unchanged status tables are not a competing execution instruction for the work expressly assigned here.

## Verification and completion

**Proposed verification.** Each companion names targeted success/refusal/interruption scenarios and replaced machinery. Use scoped `pse check` and `pse test`, with explicit force-validation for Arrow-consuming correctness closures. Regenerate when declarations/generators change. Run relevant assembled integration, scientific/reference, docs and static qualification at scope end as selected in H; do not impose a full campaign per companion. No generic benefit benchmark is required. Expected simplification is qualitative until measured.

Completion requires all selected behavior and consumers delivered, bounded decisions closed with their consequences integrated, replaced machinery deleted, required receiving and assembled qualification performed, and enduring contracts published through their decision routes. A passed local test, accepted ADR or new supplier API does not establish whole-series completion. H owns the factual Plan66 handback. No production test, build, health check or efficiency measurement was performed in authoring this draft.

## Execution checkpoint — 2026-11-10

Execution of the integrated A/B/C scope began at `d2302f32a` with a clean checkout. The selected first assignments are shared transport consolidation, checked supplier construction and stable primitives. A01/B01/C01 are coordinated prerequisite contracts; A03 proceeds within existing binding semantics. Required new kernel/primitive decisions precede dependent implementation. All other scope remains open until its actual production cutover and targeted evidence; broader Plan73/Plan66 completion is not claimed.
