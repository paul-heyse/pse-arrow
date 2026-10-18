---
title: "Plan45: Pipeline execution optimization"
status: in-progress
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
scenario_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution, diagnostics]
---

# Plan45: Pipeline execution optimization

## Purpose and baseline

This standalone series adopts the [post-Plan44 review](../design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md): thirteen findings, ten investigations, ten proposed rule impacts and the supporting source/library/independent assessments. It specifies corrections to ordinary source-to-answer work, the short investigative program needed to select further optimizations, and implementation of supported outcomes within the explicitly approved boundaries below. The maintainer selected **include implementation** during preparation. Explicit operator rule decisions are recorded below; this authoring turn does not authorize product execution.

The functional target is the Process Simulator profile's *Functional target* and blueprint §0/§0.2: interactive edit/re-solve, studies, fitting/optimization, recycles, dynamics from the same definitions, extension and independently testable responsibilities. Core 3.4, efficiency heuristics 1.0 and Process Simulator 1.5 guide consequential choices qualitatively. No universal optimizer, new scheduler, performance SLA or numerical cost model is introduced.

**Interface-checked:** the authoring baseline is `22ae87c16` on main, whose product source is the reviewed `91fd915134` baseline; the newer commit adds the review and evidence. Existing environment/workflow edits are unrelated and preserved. **Tested / Measured, inherited evidence only:** the review ran ten short analytic local simulations and three membership-kernel shapes. Its kernel reduced 8,004,000 sample visits to a 2,000-row index plus 8,000 lookups for one shape. This is neither production graph qualification nor an end-to-end speedup. Plan44's selected acceptance retains its original scope, including its unresolved KLU limitation.

**Proposed:** every product correction, experiment, adoption and qualification in this series. No package is implemented merely because a diagnosis is source-established or an API exists. The current checkpoint and companion package tables own progress; this introduction does not become a second status ledger.

## Combined target and ownership

Local source owners consume admitted typed objects and changed-object operations; canonical encoding occurs where identity/transport actually needs it. Compiler preparation follows consumed premises and separates candidate admission from retained ownership. Mathematical preparation carries one inspected immutable description through its consumers. Native libraries execute demanded products using complete physical values and independently qualified original states. Result selection reaches discovery before broad materialization; promotion and analysis share validated evidence while retaining real ownership/ACK transitions. Existing diagnostics reveal actual work without becoming a scientific authority.

| Companion | Responsibility | Consumed and delivered boundary |
| --- | --- | --- |
| [45a](45a-typed-source-and-incremental-preparation.md) | Typed sources, incremental indexes/accounting, physical dependencies, historical/study preparation | Admitted immutable revisions, exact consumed premises and sound product-held charges |
| [45b](45b-mathematical-program-preparation-and-execution.md) | Arithmetic descriptions, export reuse and qualified composition/JIT | Existing compiler premises into inspected library programs; private execution scratch remains separate |
| [45c](45c-result-selection-artifacts-and-analysis.md) | Selected reads, promotion/analysis, graph and physical publication | Protected immutable selections, final artifact coordinates and bounded transactional effects |
| [45d](45d-demanded-derivatives-and-dynamics.md) | Adjoint demand, route crossover, dynamics and checkpoints | Complete values plus explicit consumed derivative/functional demand; native integration remains library-owned |
| [45e](45e-sparse-native-and-provider-effectiveness.md) | Sparse native preparation, other solver classes, providers and recycles | Class-specific library capabilities, exact reuse premises and independent physical outcomes |
| [45f](45f-short-run-qualification-and-publication.md) | Shared observation support, integrated resweep, qualification and publication | Actual owner facts, package outcomes and the assembled target, without another work ledger |

This coordinator owns finding dispositions, investigation decisions and rule decisions. Companions own package progress and the detailed resulting behavior. A reference such as `45c/C04` names a package; review F04 names a different thing. Implementation responsibility may span consumers without moving semantic authority to that package's author.

### Foundation assessment and shared constraints

The review's independently examined AP-01–AP-06 strengths remain useful foundations. The source, math, native, result and artifact owners already represent the relevant meanings; work amplification is not a reason to merge them. Existing local typed validity must replace immediate transport reconstruction without inheriting another receiver's eligibility. Persistent maps need sound retained ownership; public library APIs do not necessarily expose individual shared nodes. Conservatively charging bounded retained roots is preferable to pretending exact hidden-node accounting or retaining an unbounded ancestor chain. Candidate construction and retained products need different lifetimes.

Full snapshots currently mean a complete relation/range inventory. Narrowing discovery requires an explicit selected contract, not a silent reinterpretation of `snapshot()`. Independent artifact ownership and final locators remain genuine transitions even when byte-bound semantic evidence can be shared. Complete parameter values, demanded gradient coordinates and state adjoints are different products. JIT and symbolic-retaining linear owners are eligible alternatives only after their complete numerical/resource/lifecycle contracts are qualified. These conclusions reuse the fresh review and focused source assessment; plan authoring is not another whole-system design review.

Preserve throughout:

- Case-owned physical units, bases, reference states, domains, conservation and original qualification. Ordinary engineering relative accuracy remains **0.001** where selected by the existing cases; analytic, rank and derivative thresholds retain their separate meanings. No tolerance tightening is an optimization.
- Exact source absence/ambiguity/membership dependencies, immutable identity, current receiving predicates and cancellation clocks. Pure runtime value binding, source-default edits, span edits and structural edits remain distinct.
- Complete physical inputs and required state adjoints; absent derivative products are unavailable, never fabricated zero. Native status is not original scientific permission.
- Immutable product versus mutable scratch, optional retention versus active ownership, symbolic versus numerical factors, accepted starts versus preparation reuse, physical output times versus native steps.
- Committed attempt/file coverage, index-versus-IPC checks, original row/unit coordinates, partial/event evidence's weaker meaning, independent artifact retention, lost-ACK recovery and actual owned drain.
- Work counts versus elapsed benefit, inclusive versus exclusive clocks, logical admission versus RSS, complete versus partial diagnostics. No environment/provenance proof or release-wheel installation is revived.

## Rule decisions

**The maintainer explicitly accepted all ten recommendations as written on 2026-10-18.** The rows below record accepted target choices or bounded investigation/adoption routes. Acceptance does not authorize product implementation, assume an unknown winner or approve changes outside the row. Before adopting a conditional realization, its package must establish every stated eligibility/preservation premise. If it cannot, keep the current safe route and record the unresolved finding or next concrete decision; do not call an uncorrected defect resolved.

The explicit per-item outcome and date are recorded here; no target depends on inferred acceptance. The [create-plan skill](../../.codex/skills/create-plan/SKILL.md#confirm-rule-changes-with-the-operator) requires: “Before fixing the target design, present each item to the operator with its consequence and obtain an explicit accept or reject.”

| Item | Recommended decision and bounded consequence | Route before dependent implementation | Decision / date |
| --- | --- | --- | --- |
| RC01 | **Retain current canonical identity first.** Carry typed payloads alongside unchanged canonical bytes to remove immediate reconstruction. Investigate deferred encoding only where the same identity can still be established. A genuinely changed identity/preimage returns for a concrete versioned decision; it is not preapproved. | 45a/A01; ADR-0089, blueprint §14.4/§20. Amend the current explanation through the design route; use an ADR/review before any new hashing contract. | accepted / 2026-10-18 |
| RC02 | **Separate candidate and retained accounting while retaining current reserved defaults and opaque allowances.** Use actual conservative ownership where supported; no RSS-derived reduction, arbitrary reset or guard relaxation. A materially different reservation/default requires a concrete sound proposal. | 45a/A03; blueprint §14.3.2/§14.4. Update the accounting explanation before dependent behavior; existing limits stay until separately justified. | accepted / 2026-10-18 |
| RC03 | **Adopt complete consumed physical premises for product reuse**, including resolution, negative lookup, ambiguity, conversion/reference state and provider meaning. Keep complete inventory identity for interchange. Adopt finer keys only once the closure is sound; otherwise retain conservative invalidation explicitly. | 45a/A04; blueprint §14.4 and affected identity frames. Version changed consumed frames; follow the ADR/design route if the identity contract changes. | accepted / 2026-10-18 |
| RC04 | **Make selected snapshots explicit before discovery**, while preserving the full relation/range meaning of full snapshots. Migrate selected consumers to the selected contract and remove their broad-discovery path. Qualify projected admission separately before dropping columns/checks; no silent narrowing of an existing full snapshot. | 45c/C01–C02/C08; blueprint §20 and ADR-0187 rationale. A changed public or Python contract uses the required ADR/design route; migrate actual consumers together. | accepted / 2026-10-18 |
| RC05 | **Permit investigation and bounded adoption of independently retained sealed shared-file members**, only if exact batch/row locators, immutable bytes, independent retention, ACK/recovery and producer retirement remain sound. Standalone copies remain the target unless that complete alternative qualifies; F07's composition fix proceeds independently. | 45c/C08; ADR-0182/ADR-0187, blueprint §20.4. Before adoption, amend proposed records or supersede accepted ones as their then-current status requires, plus architecture revision. | accepted / 2026-10-18 |
| RC06 | **Use one bounded physical encoding and qualify grouped coherent publication.** Grouping may change revision/ACK granularity only with explicit exact retry/receipt and selective-replay semantics. If those cannot be preserved, retain the per-block edit sequence over the single encoding spool. | 45c/C07; blueprint §20. State the chosen transaction/visibility/retry contract through the decision route before changing recorded granularity. | accepted / 2026-10-18 |
| RC07 | **Add a distinct consumed adjoint parameter-gradient layout.** Preserve complete values, full required state adjoints, schedules/anchors and ordinary full public adjoints. Change recorded framing only if its actual meaning/shape changes; do not reinterpret forward-axis fields. | 45d/D01; blueprint §13.6/§19. Short ADR for a new binding/kernel contract where required; migrate affected generated/public records only through generators. | accepted / 2026-10-18 |
| RC08 | **Qualify a narrow symbolic-retaining native linear binding and eligible Krylov alternatives against stock KLU.** Adopt only a sound, materially useful complete route under exact structure/settings compatibility and fresh numerical setup. No suppression of initialization or retained numerical validity. | 45e/E01; blueprint §18/§23. Short ADR before a new native binding; keep stock if the complete benefit is not established. | accepted / 2026-10-18 |
| RC09 | **Qualify and admit library program composition/JIT only under explicit eligibility**, preserving current conservative floating-point/CPU, derivative, admission, cancellation and private-scratch contracts. Keep interpreted/C++ realization where the alternative does not qualify; do not change scientific accuracy or choose JIT in advance. | 45b/B02–B03; proposed ADR-0191, blueprint §14.1/§14.3 and affected ABI/export/reset identities. Publish the exact admitted realization contract before adoption. | accepted / 2026-10-18 |
| RC10 | **Qualify weighted-functional/checkpoint bindings and narrow upstream API changes where they remove a demonstrated composition gap.** Preserve exact derivatives, sample-side conventions, initial corrections, before-effect route selection and explicit failure. Keep existing quadrature/adjoint/replay routes where alternatives do not qualify. | 45d/D02–D04; blueprint §13.6/§18/§19. Short ADR before a new binding/kernel contract; exact dependency updates use ordinary pin policy. | accepted / 2026-10-18 |

No recommendation was rejected. If a later explicit decision retains a previously proposed changed rule instead, adapt its packages and record the resulting limitation here. RC04 rejection still permits full-reader inventory reuse; RC05 rejection still permits protected promotion composition; RC09 rejection still permits one-pass descriptions; RC08 rejection keeps the confirmed F13 limitation visible. Any additional rule impact exposed by authoring/execution follows the same explicit decision route. Ordinary internal work removal needs no invented ADR.

## Dependency-aware execution

1. Establish 45f/F01's case/observation conventions early, using existing diagnostics. This does not block corrections already distinguishable from source and targeted tests. Complete the accepted prerequisite ADR/design and contract routes before dependent changes; newly exposed rule impacts require their own concrete decision.
2. Implement independent direct corrections: typed source handoff (A01), bounded source-index sharing (A02), candidate/retained bookkeeping (A03), one-pass math descriptions (B01), full-reader frozen inventory reuse (C01), exact evidence indexes (C04), and single IPC preflight (C06). A02 need not wait for a new identity; B01 need not wait for JIT; C04/C06 need not wait for shared artifact files.
3. Establish consumed contracts before dependent consumers: A04 physical closure, C02 explicit selected snapshot, C03 final artifact/evidence handoff, D01 adjoint coordinates. Integrate all actual producers/readers/workflows in the same package; agreeing an interface alone does not make it implemented.
4. Run the bounded I01–I10 investigations at their companion owners alongside ready corrections. Qualify and implement supported outcomes through those same packages and the approved RC rows. Native experiments need not wait for source/result rewrites unless they consume a changed contract. F02 supplies cross-owner discriminators and whole-operation observations; it is not another decision ledger.
5. Reassess newly exposed work on the integrated tree. Remove obsolete code/callers/tests as soon as replacements pass targeted checks and all consumers move. No compatibility shim or second production path is kept as evidence. A newly discovered in-bound cause gets a new N identifier here; changes outside the approved functional/rule boundary return as a concrete decision.
6. After all selected functional changes and required investigation outcomes are integrated, 45f/F03 performs assembled qualification and scope-end quality. F04 publishes enduring contracts and this series' outcome. Longer/different experiments retain explicit questions/triggers rather than an open-ended performance tournament.

Package dependencies are not document-wide barriers. Shared source/runtime/math/dynamics/result files require a single writer or explicitly isolated edits; logical independence does not remove edit conflicts. Use worktrees only for genuine concurrent production edits, integrate before deleting them, and retain ownership of actual processes/resources until drain. No machine-capacity tuning is part of this series.

## Finding dispositions

Source links refer to the review's original F identifiers; its observations remain unchanged. Findings begin open, not resolved by scheduling or by a no-change experiment.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| F01 local transport reconstruction | S01/S03 | open | 45a/A01 | Typed receiving admission removes immediate round trip; identity and cross-context controls |
| F02 full source-index copies | S01 | open | 45a/A02 | Changed-path index construction, cold-load and bounded shared ownership controls |
| F03 retained candidate allowances | S01/S03 | open | 45a/A03 | Edit/backdating/rotation ladder with actual retained old products and sound charges |
| F04 repeated program descriptions | S03/S07 | open | 45b/B01 | One prepared region/export through admission and all consumers |
| F05 broad physical invalidation | S02/S07 | deferred / unresolved under RC03 | Physical/compiler resolver; 45a/A04 owns the current bounded outcome | Retain whole-context invalidation: no resolver-issued negative/membership/scope/checker/provider closure. Revisit when that complete product can be issued and migrated across every compatibility boundary. Unrelated physical changes still invalidate eligible preparation; this is not AP-07/G9 resolution. |
| F06 result inventory discovery | S04 | open | 45c/C01–C02 | Full-reader reuse and selected constructor/first/EOF access work under exact protection |
| F07 repeated promotion/analysis passes | S05 | open | 45c/C03 | Shared protected selection and byte-bound evidence with final locators/ACK transitions |
| F08 repeated evidence scans | S05 | open | 45c/C04 | Production bounded indexes/grouping with complete permission/adverse controls |
| F09 per-edge server lookups | S05 | open | 45c/C05 | Distinct bounded-page server facts, authoritative replay/retirement and query-plan controls |
| F10 duplicate IPC preflight | S04/S05 | open | 45c/C06 | One bounded preflight with malformed/extra-batch/schema/IEEE controls |
| F11 repeated physical encoding/publication | S01/S07 | open | 45c/C07 | Single bounded spool and chosen exact publication/retry semantics |
| F12 unconsumed adjoint coordinates | S02/S03 | open | 45d/D01 | Complete physical inputs with demanded gradient products and independent derivatives |
| F13 repeated KLU symbolic analysis | S06 | open | 45e/E01 | Representative sparse fill/cost and qualified alternative, or explicitly retained limitation |

## Investigation outcomes

This table owns the decision outcome, not companion package progress. An investigation may reach a supported no-change result while a related confirmed finding remains open/deferred. A typed refusal or unavailable prerequisite is not automatically a successful scientific experiment.

| Inquiry | Decision question | State | Decision / work owner | Completion or next trigger |
| --- | --- | --- | --- | --- |
| I01 | Typed incremental ingress and retained ownership | open | 45a/A01–A03 | Current work removed, exact identity/validation and bounded retained/candidate accounting |
| I02 | Consumed physical dependency closure | settled / retain conservative route | 45a/A04 | Actual fact-boundary control passed; exact negative lookup, candidate membership, package visibility and checker/provider premises are not issued. Accepted RC03 explicitly permits retaining the current safe rule; F05 remains unresolved. |
| I03 | Arithmetic composition and conservative JIT | open | 45b/B01–B03 | Qualified realization/eligibility and implemented consumers, or supported retained route |
| I04 | Complete result/artifact/analysis operation | open | 45c/C01–C08 | Selected snapshot, shared handoff, joins, publication and qualified projection/file choices |
| I05 | Consumed derivative and functional-route crossover | open | 45d/D01–D02 | Consumer-specific exact route and gradient/layout controls; no universal count threshold |
| I06 | Repeated sparse native preparation | open | 45e/E01 | Complete alternative comparison at meaningful sparse shape, fresh numerical validity |
| I07 | Dynamics methods, stiffness and linear shape | open | 45d/D03 | Conditional qualified routes at unchanged physics/accuracy, including explicit refusals |
| I08 | Checkpoint and scheduled adjoint lifetimes | open | 45d/D04 | Recording/replay/backward and simultaneous storage distinguished; selected correction |
| I09 | Historical preparation/studies/retained pressure | open | 45a/A05 | Exact historical reuse decision and implementation without rewinding live authority |
| I10 | Other native classes, properties/implicit work and recycles | open | 45e/E02–E04 | Class/provider-specific conclusions and implemented supported changes, not rankings |

### Additional recommendation coverage

The review's unnumbered recommendations are part of the scope: projected registry admission, residual-filter/limit semantics, exact analysis reopening versus new occurrence (C08); cold versus warm source publication (C07); source-bound one-pass program census and conservative JIT comparison (B01–B03); stable view identity components and bounded historical retention (A05); weighted functional contraction, initial/anchor/schedule mapping and checkpoint API gaps (D02/D04); actual class update capabilities, derivative refusals and recycle controls (E02–E04); narrowly missing query/byte/join/export/retention observations, symbolized client/server capture only when decisive, observer interference, returned-failure capture and pagination (F01–F02).

Preserve rather than repeat Plan44's completed selected-forward, fresh final-fit, rank-preflight, active-view, sparse-IFT, invocation/terminal, ordered-trajectory, provider-stateful-barrier, lazy-default and HiGHS update work. This review's F08 is a different analysis join from Plan44's repaired trajectory checker. Existing active plans and their scope are neither superseded nor silently imported.

## Additional in-bound findings

| Identifier | Diagnosis and consequence | Corrective contract / preservation | State and owner |
| --- | --- | --- | --- |
| N01 | Fixed-configuration same-point implicit reuse could skip provider-backed operational anchors and restrictions. A current start change or refusal could therefore be bypassed by an earlier numerical jet. Provider factories issue no purity/retained-premise capability. | The compiled owner identifies provider effects once. Effectful anchors execute before the numerical key is resolved; effectful restrictions execute even when that key/jet is reusable. Pure selector programs retain the original fast path, and refusal clears numerical validity. Complete physical inputs, exact derivatives, original assessment and private worker/cancellation ownership remain. | Implemented and targeted-tested in 45e/E03: the real local Peng–Robinson operational-selector regression covers pure/effectful anchors and restrictions, changed starts, refusal/recovery and release. The nine-control E03/B02 selection passed; assembled acceptance remains 45f. |

## Completion, verification and current checkpoint

The series completes only when every required correction has its targeted acceptance and migrated consumers, every inquiry has its bounded decision plus implemented selected outcome, accepted rule routes have run before dependent changes, and 45f's assembled qualification/publication is complete. A supported no-change choice can settle an alternative; it cannot erase a confirmed unresolved finding. A finding retained/deferred needs its consequence, actual owner, observable trigger and explicit maintainer disposition. A recorded bounded rule decision can supply that disposition only when it unambiguously covers the observed outcome; otherwise present the concrete remaining choice at closeout. Bounded series completion does not imply resolution of the review's AP-07/G9 judgment while a material finding remains unresolved. No packet becomes done merely because its probe was time-limited or an unqualified winner looks faster.

**Planned verification:** package compile/targeted tests during implementation; source-established pass/work removal and before/after work counts with ordinary accuracy; one complete initial observation per short contrast, repeat only when variation changes a decision. 45f owns final Rust/native/Python/durable/managed integration and scope-end static/manual checks appropriate to the changes. No product-wide check has been run to author this plan.

**Current checkpoint (2026-10-18 execution):** execution is authorized across all 28 packages. RC01–RC10 remain accepted. Root owns source/compiler changes and cross-owner contracts; isolated math, result and native executors own ready B01, C01/C04/C06 and D01 work. Proposed ADR-0199 and blueprint revisions 197–198 publish prerequisite adjoint/selected-reader targets before dependent changes. Typed ingress, persistent source indexes, current retained-input accounting, prepared mathematical descriptions, historical accelerator retention and selected adjoint changes are integrated with targeted acceptance. C01/C04/C06 and the once-encoded physical spool have targeted controls; selected discovery/promotion, server facts and the native investigations remain in progress. Proposed ADR-0200 and blueprint revision 199 precede the opt-in KLU candidate. I02 retains conservative invalidation under RC03 and leaves F05 explicitly unresolved. No assembled package/series acceptance is claimed. Targeted compilation/testing precedes dependent integration; final assembled/static acceptance remains F03. Existing environment/workflow edits are preserved.

**Tested, authoring only, baseline zero:** document lifecycle validation passed with all seven new documents and 622 total native scope items checked; source-to-owner coverage and new file links passed; `just turn-end` passed; `just docs` published 488 chapters with scoped search. The initial lifecycle check found four existing uses of an undeclared `performance` topic in the consumed reviews/evidence. Declaring that existing label in the lifecycle vocabulary corrected the registration without editing their bodies. No product tests, new numerical experiments or library qualification ran during authoring, and pre-existing environment/workflow files remained unchanged. These checks establish document readiness, not completion of any product package.

Old plans/reviews and production remain unchanged during authoring. Only this new series and necessary index/lifecycle/publication registration are added. Required ADR/design decisions and contract revisions occur before their dependent implementation, as assigned by each RC row; F04 reconciles those owners with implemented eligibility and final evidence rather than postponing prerequisite contracts. Older-plan reconciliation/retirement is not part of this series. New plans do not inherit the right to delete other owners' data or fixtures.
