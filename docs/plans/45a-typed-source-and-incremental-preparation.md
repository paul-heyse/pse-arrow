---
title: "Plan45a: Typed source and incremental preparation"
status: in-progress
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution]
---

# Plan45a: Typed source and incremental preparation

## Target and baseline

This companion develops F01–F03/F05 and I01/I02/I09 from the [review](../design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md). The [coordinator](45-pipeline-execution-optimization.md) owns dispositions and accepted RC01–RC03. All work below is **Proposed**; source at `22ae87c16` establishes the current paths, not a tested replacement.

Local admission currently constructs and drops a generated Arrow declaration batch, serializes canonical edits, and reconstructs local typed payloads. Small edits clone complete ordered maps; granular phase preflight allowances accumulate into retained state before equality/backdating is known. Physical identity remains whole-inventory conservative. Historical misses can acquire fresh separately charged semantic workspaces. Blueprint §14.3/§14.4 and the existing source/compiler owners remain the starting contracts.

The resulting system owns immutable typed revisions and exact indexes directly. Receiving validation, identity and effect boundaries remain explicit. Candidate construction has a bounded temporary lifetime; retained products carry their own conservative ownership. Fine physical reuse includes all resolution premises. Historical acceleration is optional and must not rewind or share a mutable writer with the live lineage. This owner provides admitted source/context products to 45b; 45b must not invent another physical key or source authority.

## A01 — Typed payload carry-through at local admission

**F01/I01; accepted RC01.** Introduce one internal admitted-object input that can carry a validated declaration/fit/document payload, parsed fields, exact spans, logical identity, references and allocation ownership alongside the existing canonical bytes/version. `AdmittedModelRevision` consumes that typed representation rather than immediately decoding its newly created transport payload. Actual imported bytes still use the real decode/admission route. Both input forms feed one local object/index construction owner; they are not separate editable models.

Preserve current identity/preimage bytes for the selected first target. Hash/serialize only as often as that identity or actual transport requires; eliminate the immediate encode/decode/reparse loop and duplicated candidate representation. Avoid claiming all local encoding disappears while identity still consumes it. A genuinely different lazy identity returns under RC01 with its concrete versioned contract.

Migration includes initial `canonical::persist`, typed declaration replacement, `canonical/delta.rs`, complete reload, `canonical/rebind.rs`, receiving/import/replay and export/source-table consumers. Root entrypoints are leads, not a boundary that permits another ingress to retain the replaced round trip. Generated/native predicates still execute under the actual receiving context; typed ownership, matching IDs or a successful old invocation alone cannot grant eligibility. Retain exact reference extraction, absent-name/completeness checks, source order, spans and atomic revision publication.

**Targeted acceptance:** direct typed versus actual byte import gives the same admitted lookup/export/identity; invalid declaration/cross-field state and a different receiver/registry refuse or re-admit correctly; unchanged/default/span/structural edits preserve the expected selected mathematics and refreshed attribution; cancellation leaves the parent usable. Count relevant encoding/decoding/parsing passes with existing instrumentation or narrow test-local counters. Delete replaced immediate local reconstruction and redundant materializations/callers/tests once consumers move; keep actual byte-import decoding.

## A02 — Shared ordered source indexes and changed-object application

**F02/I01; consumes A01's typed changed-object meaning where useful, not a new identity.** Apply exact changed-object operations to structurally shared ordered logical-object and scope/kind/name/reverse-reference indexes. A complete reload first computes its exact changed/deleted set; omission in a delta remains inheritance. Unchanged rows, parsed fields and unaffected buckets retain their immutable owners. Full export and initial construction remain explicit whole operations.

Qualify pinned `rpds` ordered-map sharing against a cold builder before introducing bespoke containers. Its public API hides individual shared nodes: choose a sound finite node/snapshot accounting integration, or another qualified library realization, before production cutover. Do not retain an unbounded chain of ancestor revision leases. A conservative bounded retained-root charge may be an intermediate implementation result, but disclose residual overcount and do not report exact unique-node charging or the broader accounting obligation complete without establishing it. The product must remain usable under its declared ordinary admission envelope.

**Targeted acceptance:** a short source-size ladder with one changed default, expression or span shows changed-path index construction instead of full-map clones; cold load/iteration remains fit. Test rename/deletion, changed references with equal payload, ambiguity, absent names, bulk changes, old retained revisions and repeated release. Compare ordered results with independently specified inventory/lookup expectations. Reject forged logical/version/interpretation combinations and cancellation between candidate construction and publication. Actual retained ownership and removal of map-copy work are separate assertions.

## A03 — Candidate admission versus retained compiler ownership

**F03/I01; accepted RC02.** Replace unconditional accumulation of each phase's candidate allowance into `retained_preflight` with distinct candidate and retained populations. The existing driver checks cancellation and finite candidate construction before work. Successful publication transfers a source-qualified conservative bound for genuinely newly retained containers/products; equality reuse/backdating does not retain another identical construction allowance. Failed/cancelled candidates release temporary grants and leave prior products valid.

Use Salsa 0.28.4's actual heap/storage/LRU capabilities for their reported populations; they do not account for hidden foreign heaps. Shared persistent nodes still reachable through older memos or outputs remain covered. Keep current workspace/default/foreign envelopes and safe generation rotation for unknown or excessive retained state. Neither resetting a counter at request end nor replacing a reservation with observed RSS is an acceptable correction.

**Targeted acceptance:** repeated small default/span/expression edits, with unrelated declarations fixed, observe candidate peak, retained totals, query executions/backdating and workspace rotations. Include a complete no-op revision as an already optimized control, retained old outputs, optional eviction/pressure, failed phase and cancelled publication. Establish that removing history inflation neither undercharges live old products nor suppresses required invalidation. Direct counter/lifetime assertions establish the correction; a stable RSS trace alone does not.

## A04 — Consumed physical premise closure

**F05/I02; accepted RC03.** Physical resolution/admission issues complete consumed positive and negative premises: units/types/conversions/reference states, selected providers/interpretation, lookup scope, ordered membership, absence and ambiguity. Whole physical context identity remains an interchange/admission identity. Prepared compiler/mathematical products consume the issued closure using explicit versioned current frames; unsupported dependency classification remains conservatively whole-context.

Migrate all boundaries that enforce compatibility together: compiler lane context, tracked inputs, body/artifact requests, selected view identities, preparation frontier physical identity and completion comparison. In particular, changing `solve.rs`'s view key alone while `ModelingPreparationFrontier` and completion still require the whole physical hash is not a completed target. Keep one dependency meaning owned by the physical/compiler resolver, with direct versus derived context distinguished.

**Investigation and implementation:** first run the small current behavior control, then prototype/qualify the complete closure. Add an unrelated declaration; change a consumed conversion/reference state; introduce a resolving declaration after absence; introduce ambiguity and provider interpretation changes. Unrelated additions preserve eligible products; meaning-changing additions/changes invalidate. Implement the sound bounded closure and all consumers if it qualifies. If a missing premise prevents adoption, retain the current rule and record exactly which product/premise remains unresolved at F05; a positive-ID-only cache key is rejected rather than accepted as partial correctness.

## A05 — Historical/study preparation and stable bound-view components

**I09; preserves Plan44's active-view ownership and immutable artifact flights.** Compare current fresh private historical workspaces with bounded retention of an exact historical lane or compact immutable phase products. Key any retained preparation by immutable revision plus complete consumed context. Acquire exclusive authority for each mutable historical lane only after releasing live-lineage authority; never clone a Salsa handle as a snapshot or rewind the current head. Avoid retaining whole ancestral source bundles or creating another scheduler/population ticket.

Inspect bound-view construction for immutable model/profile/key components that are currently rehashed or cloned at each point. Carry a prepared immutable base description where sound, then apply current admitted values, bounds/fixed state and attribution under their existing checks. Existing complete bound keys are preservation controls, not optional omissions. Consumers include local and managed studies, replay/historical preparation and Full/Compact result retention.

**Short contrasts:** alternate current/one historical revision and selections, one/two study workers, ordinary optional retention and forced trim, with old products retained. Count actual workspace generations/construction/reservations, view/request work, queue wait and native work separately. Include incompatible contexts, branch skew, cancellation, concurrent readers and final release. Retention that saves construction but blocks ordinary admission is not automatically superior.

Select and implement the supported bounded reuse/base-description outcome, or keep fresh private construction with a precise supported reason/trigger. This conclusion does not reopen Plan44's completed active-lane fix. Missing historical product retention and consumer-held state are separate from optional cache retention.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A01 | RC01 decision; current receiving/identity contracts | Typed ingress through all actual constructors/edits/rebinds; same predicates/identity; immediate reconstruction removed | in-progress |
| A02 | Exact changed-object contract; sound shared-map allocation model | Persistent authoritative indexes with bounded ownership, cold/edit/absence/adverse tests; full-map copies removed | in-progress |
| A03 | RC02 decision; phase/product lifetime inventory | Separate temporary candidate and retained ownership; edit/backdating/pressure/cancel controls; retained-history inflation removed | in-progress |
| A04 | RC03 decision; exact physical resolution premises | Investigate and implement qualified consumed closure across lane/frontier/body/view checks; invalidating and non-invalidating controls | in-progress |
| A05 | Existing active-view/source contracts; A03 where accounting is consumed; A04 only for finer physical eligibility | Historical/study investigation, selected bounded implementation and complete-context/pressure/drain tests | in-progress |

## Verification and checkpoint

**Planned:** compile touched `pse-runtime`, `pse-compiler` and actual affected source/modeling packages through the scope-aware recipes, then targeted force-validated units for the changed mechanisms. Use existing fixtures for identity, indexed delta, source attribution, phase backdating and active study reuse; add revealing cases where absent. Package tests must exercise actual migrated entrypoints, not only new helpers. Regenerate through the owner if declarations change. Integrated science/static checks belong to 45f after functional scope.

At authoring, the review and source provide interface evidence only; all five packages are open. The first target keeps canonical bytes and reserved defaults. RC03 selects the target; finer physical reuse remains unimplemented until its complete premises qualify and its prerequisite design route is complete. Any hidden-node or physical-closure premise that stays unsettled remains explicit at the coordinator rather than being converted into a completion claim.

Current execution checkpoint: A01/A02 typed construction and persistent source roots are integrated. A03 replaces cumulative candidate allowances with a census of current retained inputs; existing opaque/default allowances remain. The targeted compiler/modeling controls passed 18/18, and the runtime source/spool/historical selection passed 18/18, against zero failures. A05 retains one exclusively borrowed exact historical accelerator when the existing lane budget permits; one-lane pressure and a busy peer keep fresh private construction, without rewinding the live head. Final cross-owner/source/study qualification remains 45f.

A04's bounded investigation retains conservative whole-context invalidation under accepted RC03. Successful PhysicalAdmission/ResolvedInference products do not issue complete negative lookup, candidate-membership, package visibility or checker/provider interpretation premises. In particular, resolve_key absence can change raw versus named representation; adding another matching operation can change ambiguity. The current fact-boundary control passed, but no finer key was adopted. F05 remains explicitly unresolved at the coordinator. Stable bound-view hashing also retains V5: the stable suffix follows structure/context/order in the preimage; a prehashed suffix would change identity, while another owner-retaining cache has no demonstrated material benefit in clone-heavy consumers. A materially costly repeated view-key profile is its revisit trigger.
