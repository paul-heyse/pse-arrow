---
title: "Plan73h: Build boundaries and integrated acceptance"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73h-build-boundaries-and-integrated-acceptance.md
doc_topics: [current-work, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Build boundaries and integrated acceptance

## Target and baseline

**Proposed.** Complete the [Plan73 target](73-symbolica-ecosystem-improvements.md) with credible build/representation choices, assembled scientific consumers and a factual handback. O12 applies to all mathematical deliveries: reduce necessary work and total machinery, rather than infer performance from fewer local lines or a new dependency name.

**Interface-checked.** Baseline `baadc7e32` with parallel changes preserved. The mathematical integration still uses Atom/evaluator APIs and an exact-pinned maintained Symbolica supplier. Direct Numerica imports inside that same owner would not remove its symbolic closure. Current native compilation is opt-in. No compile-unit census, timing experiment or memory measurement was performed in authoring. Existing code and the [build/transfer account](../library_leverage/libraries/symbolica/build-memory-and-transfer.md) support qualitative reasoning only.

A owns shared representation/reuse, B supplier controls/receiving and C execution profiles. H does not create another cache, evaluator abstraction, scheduler or test runner. Existing test selection, source-built fixtures, original-attempt evidence and supervision remain the qualification route. Scope ends with this series' actual deliveries; broader Plan66 science is not silently imported.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| H01 | open | A baseline/consumer map; exact selected dependency/profile declarations | Build, representation and optional-backend decisions closed with actual changed units/lifetimes or reasoned retention; selected changes integrated with consumers. |
| H02 | open | Delivered A–G capabilities and their local acceptance | Assembled production consumers, cold receiving and relevant scientific/reference/static qualification complete against zero target; gaps remain explicit until corrected. |
| H03 | open | H01/H02 and closed conditional decisions | Enduring contracts published, obsolete machinery retired, source coverage reconciled and factual Plan66 handback ready; no existing-plan status edited during this series' authoring. |

## H01 — Change the boundary that causes the cost

Separate five costs when proposing a change: clean Rust compilation, incremental rebuild reach, symbolic preparation, runtime native compilation and repeated numerical execution. Source facts can establish removed conversion or reconstruction, but do not establish the size of a speedup.

The selected default is the existing narrow math integration, runtime-shaped derivative programs, explicit native preparation and shared immutable products with worker-local scratch. Examine manifests and actual consumer dependencies when A/C/D introduce a genuine numerics-only owner. Adopt a new boundary only if it removes symbolic dependency units for that useful owner without duplicating scientific contracts or creating more conversion/coordination. Otherwise close the split inquiry with the observed closure and retain current organization. A new crate uses the existing ADR/review route; no speculative crate split is scheduled merely to import Numerica directly.

Use borrowed `AtomView` traversal where the operation needs only a view. Retaining a small view that pins a large owner can be worse than a compact copy; choose at the consuming lifetime. Polynomial `zero_with_new_context`, `clone_with_context_of` and `compact` are candidate targeted changes for independent worker/context traffic or long-lived products. Require compatible coefficient domains/variable maps and an actual repeated owner; do not rebuild every temporary context. The [representation source account](../library_leverage/libraries/symbolica/build-memory-and-transfer.md#reduce-representation-work-at-its-actual-lifetime) explains these limits.

Close the following finite decisions within H01:

| Candidate | Decision and integration boundary |
| --- | --- |
| Fixed generated hyperduals | Compare C's actual repeated derivative shape with runtime shapes. Select only a stable kernel shape that removes meaningful dynamic work without broad monomorphization. Otherwise retain dynamic layouts and reopen for a named stable kernel. |
| `TermStreamer` | Compare a real large additive analysis, if E exposes one, against factorized in-memory products. Select only when term-local transforms plus explicit global normalization preserve meaning and reduce the actual representation burden. Coupled Helmholtz graphs, GCD/factorization and shared definitions are countercases; absent such an additive workload, close with no adoption and that explicit trigger. |
| Arithmetic backend/features | Retain GMP/MPFR and current exact rounding contracts. A portable-backend move requires an actual supported deployment need and comparison of the exact consumed arithmetic/precision behavior; availability or license is not a reason to switch or reject. |
| JIT/assembly/CUDA | C owns numerical eligibility and the JIT adapter issue. H supplies build/startup/transfer/lifetime consequences. Device execution requires suitable hardware and a named large pure population; it does not accelerate symbolic preparation. No automatic new backend is selected. |
| Python/WASM adapters | No product-boundary expansion is selected. Reopen only for a specific independent reference or preview consumer whose complete custody/deployment contract warrants it. Keep scientific interpretation native. |

Do not impose workstation CPU/memory partitions, compiler-job reductions or a new general capacity framework. Existing operation-specific admission and supervised execution remain meaningful. Quantitative profiling is discretionary only where a serious regression concern changes the decision; otherwise record the qualitative mechanism and its limits without a deferred benchmark obligation.

## H02 — Assemble independent useful consumers

Targeted checks belong with A–G implementation and the corresponding deletion. H02 verifies that individually correct components actually compose through the production operation and result routes. Select existing source-built fixtures and operations rather than a second runner or a unit-only substitute.

Required assembled journeys:

1. A repeated residual/fitting or solver caller reuses qualified named products across changed values and runtime seeds. A guard/layout/provider change invalidates appropriately. Compare independent analytic actions and retain the direct finite weighted-product countercase.
2. A typed stable operation reaches interpreted and selected compiled Value/First/Second products; a repeated multi-output population uses qualified x4 execution. Original failure/point order, valid tails, cancellation and independent scratch survive result reading. Selected log1p/magnitude Value/First/Second products survive their required receiving profile. Profiles outside this selected closure refuse unsupported operations before submission.
3. Checked supplier construction is actually consumed by the application. Too-small allowance, construction interruption and supplier truncation never publish success; the corresponding private predictor/rebinding path is gone.
4. A producer retires before a cold receiver continues numerical execution and a guarded symbolic consumer performs later differentiation/class evidence. Check destination symbols, layouts, receipts, unsupported attributes, bounded framing and original-coordinate results. A sealed-copy/compiler-only control is insufficient.
5. Exact block and numerical-assurance operations return correctly interpreted ordinary/exceptional parameter cases, claim strengths and unsupported certification, through protected results and an independent reader. Preserve existing root/conversion/witness behavior.
6. A caloric source produces derivative/series/identity and indexed artifacts with faithful native presentation. Old artifacts retain original source context after editing; rendering does not rerun science or promote approximation to exactness.
7. F's declared measure yields the independent heater mean, records failed members and replays the selected state/points. G's finite topology population distinguishes complete, interrupted and physically rejected candidates through ordinary admission and scientific execution.

Use focused native integration/component/solver tests for these journeys, plus applicable independent analytical/multiprecision or isolated scientific references. IDAES parity is selected only where its behavior genuinely supplies relevant independent evidence; its full unrelated population is not this series' completion gate. Preserve clean-room boundaries and explicit reference preparation. No test may infer physical validity solely from agreement between two routes using the same transformation.

After all functional scope is integrated, select the affected hygiene/governance, docs/lifecycle, generated-output, native and feature/powerset checks according to repository policy. Use `pse check`/`pse test` for scoped implementation evidence, explicit Arrow validation where applicable, and codegen at declaration changes. Do not rerun full static/integration campaigns per companion. Report actual commands, selections, zero-target failure counts and exclusions. A narrower passing selection cannot qualify unexercised scientific scope.

## H03 — Completion and Plan66 handback

Reconcile all O01–O13 and the coordinator's detailed lead ownership. Every finite investigation has an outcome and its selected implementation/consumer consequence. Do not close a selected capability merely because a supplier API exists, an ADR is accepted or one local test passes. Inspect removal of superseded code, callers, configuration, adapters, fixtures and mechanism-specific tests; preserve independent behavioral verification against the target.

Carry enduring meaning into its authoritative architecture/developer owner through the required route, with revision rows where applicable. Accepted ADRs remain immutable. Track selected supplier changes in reproducible source provenance and exact dependency declarations. Retire resolved research only after surviving knowledge/obligations have owners; Git is the archive, not a new archive directory.

Prepare a compact handback in this series' closing outcome: delivered capability and source/evidence; relevant Plan66 A/C/D/J/K obligation; structural work absorbed here; broader population or acceptance still remaining. This is factual evidence for the maintainer's subsequent reconciliation, not a simultaneous rewrite of Plan66 or duplicate live status. In particular, completing the expectation or expression reader here does not complete all 66j studies or 66k assessments.

## Verification and outcome

**Proposed verification:** H01 source/dependency comparison and actual selected changes; H02 assembled journeys and scope-end qualification; H03 coverage, deletion, enduring-contract and handback reconciliation. No new product/build/performance evidence is claimed by this document. Completion remains open until the named delivery and qualification obligations are actually satisfied. Expected improvements need not acquire measurements unless their magnitude is claimed or a serious regression concern requires them.
