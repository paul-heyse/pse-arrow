---
title: Active preparation and study reuse
status: in-progress
date: 2026-10-17
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md]
doc_owner: docs/plans/44-pipeline-performance-optimization.md
doc_topics: [current-work, preparation, numerical-execution, diagnostics, validation]
---

# Plan 44c: Active preparation and study reuse

## Purpose and foundations

This companion owns F04 and O03/O06; [Plan44](44-pipeline-performance-optimization.md) owns combined dispositions. **At authoring — Proposed:** all work below; the specified-change checkpoint records subsequent implementation. The source review's 16/64 GiB observations establish pressure-sensitive preparation counts, not a measured speedup, eviction chronology or permission to reduce allowances from RSS.

The relevant owners are `workflow/study_operations.rs::{StudyPreparationLane,PreparedLaneInputs}`, `workflow/modeling/views.rs`, study result retention, `math/retention.rs`, compiler preparation and provider reconstruction. Existing study lanes already retain selected admission and a preparation scope closed after native drain. Shared modeling retention is optional; returned Full results may retain prepared payloads after lookup eviction. Extend that lifetime model instead of adding a second global cache or making results own execution-lane policy.

## C01 — Active lane owns its compatible current numerical view

Retain one current immutable numerical view within each existing active preparation lane and rebind new point values while the complete compatibility premise holds. Consume the actual bound-view key: selected structure/physical context, compiler/order/policy, source attribution and other semantic dependencies. Replace on incompatible change. A named “same structure” shortcut or a partial hash is insufficient.

Preserve optional shared lookup retention for cross-lane reuse. Trimming that lookup cannot discard a view still legitimately owned by the current active lane. Keep one payload accounting owner with legitimate shared references; do not charge every Arc clone or pin unbounded historical views. A result may independently retain its prepared product without extending the lane's mutable scratch or activity. Constructor admission, active payload and retained-result costs remain explicit.

Release lane ownership only after actual native drain; preserve cancellation, closed preparation scopes, point isolation, independent original checks and mutable evaluator/provider scratch exclusivity. No new lane pool/scheduler or immutable cache storing mutable hints is needed. Remove the replaced reliance on global lookup for current-view reuse in the migrated path.

Acceptance: four same-structure independent points under one/two workers with controlled optional-cache trimming; Full and Compact results; changed values versus changed physical context/order/policy/structure; concurrent equivalent requests; cancellation while a point owns native work. Same-lane compatible points reuse their view within admitted live memory. Do not assert `views <= workers` universally across legitimate incompatibility or reuse every rejected candidate.

## C02 — Immutable implicit compilation across value-only preparations

Investigate O03 at `prepare_model_providers`, `reconstruction_factory` and compiler implicit-program creation. Count actual residual/hint compilation and compare fresh value/configuration hints with stable program, order, semantic operational settings and limits. Distinguish separately resolved cases from an already retained study path; do not claim every study point recompiles.

Prefer existing compiler/preparation ownership and exact compatibility to another global cache. Share immutable residual/hint programs only when their full premise is unchanged; value-specific starts, mutable provider state and native workspaces stay per owner. A changed assumption, hint expression or supported derivative order invalidates the relevant structure. Establish the key and charge/lifetime before implementing reuse.

Acceptance uses a branch-selected eligible implicit case with value-only edits, changed hint values versus changed hint programs, order/configuration changes and independent simultaneous executions. Original root/derivative assertions and capability refusals remain. Select a source-backed improvement and remove replaced recompilation logic, or record why current reuse is already adequate; an unmeasured new cache is not an outcome.

## C03 — Attribute preparation and boundary unit costs

The reference-admission CPU capture has unknown clock alignment and source partiality. Loader, Arrow schema, Serde and allocation stacks are leads, not an exclusive bottleneck attribution. Obtain the narrow missing distinction through a phase-aligned selected capture, allocation/call-count control or source-backed repeated-work comparison.

Separate Python/native loading, physical/source admission, mathematical preparation, diagnostic serialization, result projection and required original qualification. Compare minimal sources with the actual reference set and a compact property-bearing numerical model; source admission alone cannot stand in for property evaluation. Keep cold/repeated operations separate. Do not subtract unrelated process samples to manufacture exclusive owner time.

For a confirmed repeated conversion/metadata/allocation cause, design and implement the smallest change in the existing authority/transport owner: reuse compatible immutable structure, avoid an unnecessary crossing/materialization or batch an already equivalent operation. Preserve declared schemas, field/extension metadata, physical source identities and independent checks. Use DataFusion/Arrow capabilities through their library skills and current Context7/exact source where pertinent. No representation rewrite follows merely from seeing a clone stack.

Acceptance requires the discriminating call/allocation/work observation, unchanged admitted meaning/results and a short affected-case rerun. If attribution stays unresolved after compact controls, identify the precise missing premise and follow-up experiment; do not publish an elapsed benefit or profile completeness claim.

## C04 — Constructor allowances and study composition

Investigate only the allowances implicated by C01's pressure behavior. Distinguish declared constructor/foreign-library bounds, active operation charges, optional cache policy, retained payload, peak logical pool and actual RSS. The review's 4 GiB engine cache and 8 GiB modeling policy belong to different families. An available-memory intervention is diagnostic, not a target configuration prescription.

Use existing reservation/pool/retention diagnostics; add narrowly missing facts through 44a/A03 when necessary. Evaluate construction peaks and shared-reference accounting from source and compact pressure controls, including lane count, Full/Compact outputs, skewed points and cancellation. A lower allowance requires a sound implementation/library premise, not a single small RSS observation. Retain conservative current allowances if no narrower sound bound exists; C01 must still establish active reuse within its declared envelope.

No host resource guard is bypassed and no shared registry/service is reset for this investigation. Existing owned fixtures and registered cleanup are used if managed coverage needs them. A passing study body with failed join/release is not an assembled pass.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C01 | Current lane/drain and complete bound-view compatibility | Charged current-view ownership/rebind under optional-cache pressure; changed-key and Full/Compact/cancel tests; delete redundant current-view reconstruction | done |
| C02 | Eligible implicit fixture and exact immutable/mutable premise | Compile/reuse investigation and selected correction or supported no-change conclusion; value/hint/order/concurrency tests | open |
| C03 | E01 attribution questions; A01/A03 only for missing joined facts | Attributed preparation/unit-cost cause, selected owner correction or precise unresolved premise; short original-assertion rerun | open |
| C04 | C01 lifetime model; source-qualified current allowances | Sound constructor/accounting conclusion and any selected correction; no RSS-derived guard relaxation | open |

## Verification and checkpoint

[44e](44e-short-run-investigation-and-optimization.md) owns case rounds and measurements, including the cold/repeat/seed distinctions. Targeted C01 lifetime/reuse tests are necessary even when all analytic roots already pass. Benefit reports name work removed and actual conditions; parallel speedup is not inferred from fewer views or a larger pool. Final integration/quality belongs to 44f. All C packages were open at authoring.

## Specified-change checkpoint

**Implemented and targeted Tested:** each active lane owns one compatible current numerical view checked before optional global lookup. The complete bound key, source attribution and original domain admission govern reuse; fresh values are rebound, incompatible keys replace the slot, and payload/metadata references retain their existing shared accounting owners. Native scope drain precedes slot release. The local coordinator future is boxed inside observation wrappers, matching the existing durable route, so the public local path does not embed its large state through nested caller frames. Tests cover four real point preparations with one/two lanes and Full/Compact retention under between-point trimming, key/attribution changes, concurrent warm sharing, metadata denial and cancellation while a real retained native session is held. The cancellation control uses the production study lane's staged execution path with an eligible nonlinear, non-presolved KINSOL model. C02–C04 remain investigations; F04 stays open for C04's separate constructor/accounting question. No RSS-based allowance reduction or measured parallel speedup is claimed.

The coordinator owns command/result totals and the enclosing acceptance boundary. These package results do not close the 44e campaign or 44f qualification.
