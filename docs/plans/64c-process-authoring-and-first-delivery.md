---
title: "Plan64c: Process authoring and first delivery"
status: in-progress
date: 2026-11-02
adrs: [ADR-0236]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
---

# Process authoring and first delivery

## Resulting engineering behavior

**Proposed:** one native path lets an engineer inspect selected process structure, edit an admitted definition/case, prepare and run supported operations, and inspect physical outputs or full failures. [64b](64b-native-scientific-intake.md) supplies Packet A's ordinary science; this companion supplies B/C and the native side of D, whose matched reference/closeout owner is [64h](64h-reference-coverage-and-qualification.md). The [coordinator](64-process-simulator-target.md) owns source baseline `1acd8c83c`, rule decisions and F01–F06 dispositions.

The first supported interface is a Rust domain-operation surface using existing `ModelingPackage` and workflow/result owners. A graphical application, new language binding or compulsory CLI wrapper is not needed to establish it. Typed declaration editing is the first edit representation; text documents can still be admitted as source, but editing declarations does not promise regenerated authoritative text. Preserve draft diagnostics without claiming an invalid draft can solve.

## Package state

### Execution requirements from 64a

Consume [64a's operation decisions](64a-scope-and-shared-contracts.md#a03--selected-library-backed-operation-design) at these owners:

| Package | Selected mechanism and boundary |
|---|---|
| C01/C02 | Typed declaration edits and existing flow owners; Salsa derivation is separate from runtime custody/pressure. Use consumed dependencies, not whole-submission invalidation or a second cache. |
| C02 | Symbolica parameterized multi-output/function DAG and demanded Numerica realizations; faer symbolic analysis and numeric factors have distinct dependencies. Preserve solver-owned POUNCE/FERAL factors; reject aborted optimization before retention. |
| C05 | Compiler-bound incidence through POUNCE HK/DM/BTF independently of candidate values, numerical Jacobians and solver eligibility; point diagnostics keep their separate owner. |
| C03/C06 | Completion-driven finite fan-out where useful, explicit admission closure/cancellation settlement and native joins for foreign/TLS teardown. No scheduler rewrite. |
| C04/C05 | Scoped producer facts and registry-generated boundary adapters; retain Buffa/ConnectRPC and protected lineage. No quantity-only permission or caller-authoritative export label. |

C03 preserves Solve versus full Initialization report, including unsuccessful reports. C04/C05 migrate local/durable/inspection/fitting/uncertainty/portability consumers together under proposed ADR-0236. Version changed interpretations explicitly; historical missing scope cannot strengthen permission. C07 covers final initialization, ready-state and conditional ordinary free-variable acceptance without relaxing hard domains or fixed specifications.

C06 completes native journeys and the exact H02/H03 handoff, not matched-reference execution or full A–D acceptance. G08, broader analyses and the collective upgrade retain their owners.

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| C01 | planned | A02/A03 meaning; B01 selected definitions | Selected source/declaration/instantiated process views and typed edits over existing immutable identities |
| C02 | planned | C01; RC03 contract slice | Label/numeric/topology edits preserve or invalidate the appropriate artifacts and match clean admitted reconstruction |
| C03 | planned | B ordinary-input path; admitted initialization/solve | Complete success and intended failure reports survive public native operations and restoration/readback |
| C04 | planned | Protected result/producer completion contract | Export selection derives exact scientific usability; affected readers/writers migrate together |
| C05 | planned | RC04; completion and local-analysis contracts | Exposed product applicability and exact governing scope are consistent across native/recorded consumers |
| C06 | planned | C01–C05 selected behavior; B delivery; H02 matched reference contracts and inputs | Recycle/exchanger native journeys and independent-reference handoff pass complete first-delivery acceptance |
| C07 | planned | 64a A04 ordinary-interval disposition | Quality-owned tolerance semantics integrated into affected ordinary/conditional/initialization acceptance without relaxing hard domains or fixed specifications |

## C01 — Native inspection and editing

Use `modeling.rs::with_declarations` for additions/replacements/removals against an explicit parent revision. Use selected semantic instance/member IDs rather than list ordinals or displayed labels. Return units, state/port contracts, connection occurrences, scientific selections, case roles, diagnostics and their actual inspected scope. Keep drafts, admitted declarations, selected instantiated semantics and numerical artifacts distinguishable.

Use `prepare_flow` for selected topology; do not infer solve order from material-flow direction or require a numerical square solve for a property/structure panel. Preserve parallel edges, isolated units, component identity/order independence and source-attributable incompatible-connection errors. Reuse existing source-delta/canonical publication for recorded edits. No second editable equipment graph or independent readiness state machine is introduced.

Acceptance uses a label rename without semantic identity change, an illegal state/port connection attributed to its original declaration, a reordered component set with equivalent meaning, two parallel connections and an isolate. Invalid edit admission leaves the parent revision intact. Ordinary operation consumers, not only a fixture-specific adapter, use the new surface.

## C02 — Case binding and artifact reuse

Change feed temperature as a case value without changing physical definition or reconstructing compatible symbolic structure. Change fixed/free roles, topology, contractual state slots or scientific data through their actual admitted structural/scientific paths. Rating and design select different specifications over the same exchanger physics; releasing area and specifying outlet temperature is not a mere cosmetic value edit.

Carry exact consumed evaluator/method dependencies through existing prepared owners. A guess is an execution input, not the semantic branch requirement. Reuse starts/factors only while their own premises hold. Compare admitted semantic meaning after incremental and clean construction; equality of incidental output order or source bytes is not the claim. Read existing declared-diagnostic reuse tests as preservation evidence, not newly passing receipts.

## C03 — Preserve full initialization outcomes

`modeling/declared.rs::execute_declared` currently converts an incomplete initialization report to an input error and returns only the last result on success. The process-facing initialization operation uses `initialize_declared` and returns the full typed report or a stable owned report reference, including stage/attempt evidence, candidate availability, original assessment and restored specification. Keep numerical failure distinct from invalid input or unavailable capability.

Retain success, failure and partial-evidence meaning through the selected local and recorded route; inspect existing canonical result representation and extend its owner only for actually missing report fields. No second initializer, rollback engine or retry mechanism is permitted. Migrate each process-facing initialization consumer and its meaningful tests together; convenience solve operations may retain their narrower documented contract.

The failure control must reach the intended initialization stage and preserve the user's immutable specification. A setup error does not satisfy it. A successful multi-attempt initialization must retain earlier attempts and qualifiers, not just the final result. Independently exercise original residual/domain/bounds and closure checks; a native success code is insufficient.

## C04 — Producer-owned export usability

`connected_results.rs::pack_selection` currently copies caller-supplied usability, and validation reconstructs using that same supplied value. Resolve usability from the exact protected run/attempt/member/coverage permission owner when creating and validating an export selection. A caller may request content or apply stricter eligibility; it cannot strengthen unknown or unusable science.

Migrate affected pack-selection/export/readback consumers and durable representations together, using the existing decision route if a durable/public contract changes. Preserve schema, field/row/predicate coordinates, source identity, withheld/unknown states and diagnostic-only access. Retain bounded export staging and interruption/no-final-file behavior. Delete the replaced assertion path after all callers move; do not keep a compatibility entry point that accepts an authoritative caller label.

Acceptance changes only the supplied label and verifies scientific usability cannot improve. Also check missing producer permission, a qualified producer, changed protected coordinates, incomplete output coverage and interrupted output. This correction does not claim an incorrect published physical result was reproduced during review.

## C05 — Scoped products and analysis-only results

First delivery exposes structural diagnostics and the supported products actually requested by the recycle/exchanger workflows; it does not add every sensitivity/covariance family to P1. Inspect all public/recorded consumers before fixing the product identity. Extend `Completion::derive_permission`, `step_permission`, local-analysis and result-inspection responsibilities rather than introduce a service.

A derived fact identifies its actual step/member/reference point, quantity, relevant coverage and analysis assumptions. A valid point-local derivative at an unusable physical candidate remains diagnostically visible but withheld from scientific reuse. An independently accepted step is not erased by a failed sibling, and cannot upgrade that sibling or aggregate coverage. Quantity-only lookup is insufficient when the same quantity occurs at several points.

Structural Analyze success follows the inspected model and analysis evidence; it requires no fabricated physical member or accepted-state predicate. Point-derived sensitivity/covariance/stability still requires its actual point and mathematical prerequisites. Lazy products receive a scoped fact when produced; an earlier immutable invocation can truthfully retain unavailable status. 64g expands product families only with the same contract and migrated consumers.

Acceptance pairs zero-state underdetermined structural inspection with a point-derived request missing its point, plus two same-quantity products at different scopes and one failed independent sibling. Unknown capability remains unavailable; mandatory unsupported evidence refuses rather than silently downgrades.

## C06 — Assemble A–D's native side

Use the actual recycle-flash and exchanger-costing authored definitions in `packages/reference/campaign/models/`. B supplies ordinary eligible records. Preserve the ideal benzene/toluene recycle separately from BT_PR/relational-density exchanger assumptions. Execute default recycle success, its intended initialization failure/restoration and solved-area exchanger design. Include rating with its explicit fixed/free correspondence and one compatible closure substitution without rewriting side balances; changed science has a separate claim.

Retain exchanger unknown-area/pressure-extrapolation permissions and costing currency/unit conventions; matched arithmetic is not a qualified equipment purchase price. Inspect semantic physical outputs and exact retained readback through explicit local/recorded routes. H's reference construction can overlap after scientific correspondence settles; integrated D closeout needs the completed native side.

**Proposed verification:** revealing owner tests and affected compile checks during implementation; assembled native/reference journeys only when their selected scope is integrated. Preserve independent analytic balance expectations and wrong/missing-output comparator controls. A–D completion is bounded first-delivery acceptance, not P0–P7 equivalence. All production packages remain planned at publication.

## C07 — Shared ordinary-interval acceptance

64a's source assessment assigns the DA-062 correction here. Reuse `pse-backend-native::quality` interval/tolerance computation in the affected initialization and conditional/causal consumers, retaining original source IDs and physical-coordinate assessment. Trace each caller to distinguish ordinary free-variable tolerance from hard domain/sign guards and fixed specifications before replacing its comparison. No global relaxation or interval-arithmetic subsystem is selected.

Exercise a finite representable deviation inside ordinary tolerance, one outside, and strict-domain/fixed-value countercontrols across the affected routes. Migrate all selected consumers and delete their replaced ordinary comparisons; keep the stronger checks where their meaning requires them. C03/C06 and E/G/H acceptance consume the correction only for routes implicated by this discrepancy. Passing this slice does not independently close every DA-062 manifestation. This is a scheduled production package, not work executed by 64a.
