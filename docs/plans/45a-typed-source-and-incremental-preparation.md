---
title: "Plan45a: Typed source and incremental preparation"
status: done
date: 2026-10-18
adrs: [ADR-0183]
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution]
---

# Plan45a: Typed source and incremental preparation

## Target and baseline

This companion develops F01–F03/F05 and I01/I02/I09 from the [review](../design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md). The [coordinator](45-pipeline-execution-optimization.md) owns dispositions and accepted RC01–RC03. At authoring the work below was **Proposed**; source at `22ae87c16` establishes the current paths, not a tested replacement.

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
| A01 | RC01 decision; current receiving/identity contracts | Typed ingress through all actual constructors/edits/rebinds; same predicates/identity; immediate reconstruction removed | done |
| A02 | Exact changed-object contract; sound shared-map allocation model | Persistent authoritative indexes with bounded ownership, cold/edit/absence/adverse tests; full-map copies removed | done |
| A03 | RC02 decision; phase/product lifetime inventory | Separate temporary candidate and retained ownership; edit/backdating/pressure/cancel controls; retained-history inflation removed | done |
| A04 | RC03 decision; exact physical resolution premises | Investigate and implement qualified consumed closure across lane/frontier/body/view checks; invalidating and non-invalidating controls | done |
| A05 | Existing active-view/source contracts; A03 where accounting is consumed; A04 only for finer physical eligibility | Historical/study investigation, selected bounded implementation and complete-context/pressure/drain tests | done |
| A06 | Accepted RC03; prerequisite identity/ownership amendment | Complete resolver-issued physical premises, current-context lane refresh, migrated body/artifact/view/frontier consumers and fresh receiving wrappers; invalidation/non-invalidation/failure/ownership tests | done |

## A06 — Reopened physical-premise implementation

Issue a sealed versioned consumed physical product from actual resolution, including positive/transitive meaning, misses, complete candidate populations, scope, checker interpretation and consumed provider meaning. Unknown classifications retain full-context dependence. Refresh current context and Salsa inputs coherently under existing single-writer ownership; re-admit current source and rebuild current prepared wrappers while sharing compatible immutable mathematics. Migrate every A04 boundary together. Full canonical/inventory identity and receiving permission remain separate. The existing revision-opening API supplies the changed-context journey; no new public API is planned.

Current checkpoint: the independent prerequisite review accepted the bounded target and ADR-0183/blueprint revision 204 preceded production edits. Resolver-owned indexed lookup/candidate capture, audited body physical prepass, atomic context/source publication, current/historical lane selection and receiving-wrapper migration are implemented. Body/artifact, bound-view and frontier/completion consumers use the new consumed meaning. Scientific-witness/direct implicit paths retain explicit complete-context fallback; unknown behavior cannot manufacture narrower retention. Independent implementation review identified and corrected paired context/source admission, redundant old lane owners, sparse retained string charges and fallback provider population mismatches. The targeted resweep below and 45f/F05 combined acceptance passed, including repaired scope-end qualification. A06 is complete.

**Implemented:** captured questions include positive and absent IDs/names/keys, complete candidate buckets before short-circuiting, transitive unit/kind/reference meaning, checked facts, visibility and consumed provider descriptor/output. Indexed sparse projection does not scan unrelated inventory. A sealed built-in checker owner replaces unsound trait-object-vtable address identification. Nested captures, caught failures and administrative framing have explicit synchronous/unwind-safe handling. Materialization audits the preflight; incomplete coverage constructs under complete current context and uses its complete identity. Current provider registrations/configurations retain their existing runtime guard.

New context inputs and incoming source rows publish within one transaction; failure restores the preceding context/head and cancellation leaves it usable. Equal-content/new-owner refresh issues current checked shells without rebuilding eligible mathematics. Historical preparation uses the requested revision's exact owners. Live lane metadata no longer keeps obsolete initial contexts. New sparse registry indexes, declaration strings, captured questions/facts and current-context provider maps retain explicit charges. Ordinary same-owner active-view hits preserve their existing fast path.

### A06 targeted verification

**Tested, baseline zero:** `PSE_TEST_RESOURCE_STATE=<private registry> scripts/pse-env -- just unit-package pse-quantity premises --features fixtures` passed 7/7 resolver controls (64 unrelated tests excluded), including positive/transitive/absent dependencies, candidate ambiguity, scalar/vector literal selection, long sparse names, nested/custom/unwind handling and an independent materialization audit.

`PSE_TEST_RESOURCE_STATE=<private registry> scripts/pse-env -- just unit-package pse-compiler 'test(physical_premises_tests) | test(workspace::modeling::reuse_tests) | test(scientific_witness_tests) | test(physical_potential_tests) | test(implicit)'` passed 54/54 (219 excluded), 2.218 seconds execution, no compile warnings. The package closure is pure. New controls cover unrelated addition preserving mathematical Arcs, same-ID consumed meaning invalidation, failed/cancelled refresh, finite-function cached admission, equal-content/current owners, atomic paired source/context changes and complete scientific-witness fallback with an unused provider. Existing scientific/implicit/reuse controls also passed. Runtime acceptance follows below; assembled acceptance belongs to 45f/F05.

**Tested, runtime force-validation, baseline zero:** `PSE_TEST_RESOURCE_STATE=<private registry> scripts/pse-env -- just unit-package pse-runtime 'test(physical_refresh_tests) | test(shared_body_cache_preserves_unconsumed) | test(checked_selected_admission_separates_provider_configuration) | test(lane_current_view) | test(warm_bound_lookup) | test(solver_view_rebinds) | test(lane_tests)'` passed 22/22 (710 excluded), 1.304 seconds execution, no compile warnings. It covers one live compiler under a held writer, current/new-owner receiving shells and mathematical keys, historical equal-content contexts, no retained initial live-lane ancestors, explicit provider configuration, ordinary view/rebind reuse, pressure/trim and pool release. The correctness closure explicitly enables `pse-runtime/force-validate`. These targeted outcomes qualify A06; combined native/runtime and scope-end acceptance remain 45f/F05.

The first new quantity fixture used an invalid hyphenated physical name; correcting that fixture yielded seven passes. The first compiler resweep had 53 passes and one invalid oracle: failed-candidate rollback may destroy a candidate Salsa generation, so a bare workspace cannot promise identical mathematical Arcs afterward. The corrected test verifies restored exact context/head, body specifications and usable mathematics. Successful unrelated and equal-content refreshes still require actual mathematical sharing. A runtime compile attempted during an API edit did not execute tests; the stable-source retry owns its result. No production check or scientific accuracy was relaxed.

## Verification and checkpoint

**Planned:** compile touched `pse-runtime`, `pse-compiler` and actual affected source/modeling packages through the scope-aware recipes, then targeted force-validated units for the changed mechanisms. Use existing fixtures for identity, indexed delta, source attribution, phase backdating and active study reuse; add revealing cases where absent. Package tests must exercise actual migrated entrypoints, not only new helpers. Regenerate through the owner if declarations change. Integrated science/static checks belong to 45f after functional scope.

At authoring, the review and source provided interface evidence only. The original five bounded package outcomes had targeted acceptance; assembled series qualification remains 45f. The first target keeps canonical bytes and reserved defaults. RC03 selects the target; finer physical reuse was not implemented by A04; A06 above owns the later implemented correction and current acceptance. Any hidden-node or physical-closure premise that stays unsettled remains explicit at the coordinator rather than being converted into a completion claim.

Original bounded checkpoint: A01/A02 typed construction and persistent source roots are integrated. A03 replaces cumulative candidate allowances with a census of current retained inputs; existing opaque/default allowances remain. The targeted compiler/modeling controls passed 18/18, and the runtime source/spool/historical selection passed 18/18, against zero failures. A05 retains one exclusively borrowed exact historical accelerator when the existing lane budget permits; one-lane pressure and a busy peer keep fresh private construction, without rewinding the live head. Final cross-owner/source/study qualification remains 45f.

A04's original bounded investigation retained conservative whole-context invalidation under accepted RC03. Successful PhysicalAdmission/ResolvedInference products do not issue complete negative lookup, candidate-membership, package visibility or checker/provider interpretation premises. In particular, resolve_key absence can change raw versus named representation; adding another matching operation can change ambiguity. The current fact-boundary control passed, but no finer key was adopted. F05 remained explicitly unresolved at that close; A06 now owns its correction. The original stable bound-view hashing retained V5: the stable suffix follows structure/context/order in the preimage; a prehashed suffix would change identity, while another owner-retaining cache has no demonstrated material benefit in clone-heavy consumers. A materially costly repeated view-key profile is its revisit trigger.

The two actual runtime source-size controls passed at 8/32/128 declarations, including real byte receiving and changed-byte refusal. Equal preparation and independent equal ingress issued no new workspace/frontier/completion; an unrelated deletion likewise issued none. Default/span/expression edits retained the current workspace while issuing one affected frontier/completion. Independent fresh source identities/order and retained original source/bound products remained valid. These are scoped work observations; first-ingress clocks include receiving setup and do not establish a source-size latency ranking. The three-control selection also exposed an unrelated E04 test-shape error; it did not fail either source control.

## Outcome (recorded after implementation)

### What was built

**Implemented / targeted Tested:** Typed source admission, persistent ordered indexes and current retained-input census replace the immediate local decode, full-map copy and cumulative candidate charges. One bounded historical accelerator is exclusively borrowed; busy or inadmissible history uses fresh private construction. Canonical identity, receiving checks and opaque/default allowances remain unchanged. The preceding package verification records targeted conditions; [45f](45f-short-run-qualification-and-publication.md#verification-and-checkpoint) records assembled acceptance and the shared scope-end checks.

### A mistake made and corrected

The first source-size fixture mixed receiving setup with the preparation clock. The final controls separate ingress from equal/edit preparation and compare actual workspace/frontier/completion counts rather than claiming a source-size latency ranking.

### Deviations from the plan, deliberate

At A04's original bounded close, RC03 retained complete-context invalidation because complete consumed premises were unavailable, and V5 remained unchanged. A06 subsequently issues the resolver-owned closure and migrates eligible body/view/frontier consumers; scientific-witness/direct implicit and unknown classes retain explicit complete-context behavior. No extra prehashed suffix or owner-retaining view-key cache was introduced.

## A06 follow-up Outcome

### What was built

**Implemented / Tested:** resolver-issued consumed premises and all compiler/runtime compatibility consumers, coherent incoming source/context publication, fresh current-owner wrappers and exact historical preparation. The targeted verification above records 7 resolver, 54 compiler and 22 runtime passes against baseline zero; 45f records combined native/edit and scope-end acceptance. Versioned frames are published through the real generator. Complete-context and nonreuse fallbacks retain their stated eligibility; neither inventory bytes nor an arbitrary immutable checker assert broader behavior identity.

### A mistake made and corrected

Trait-object vtable addresses were not a sound built-in checker discriminator across code-generation units; the sealed concrete-owner hook corrected false fallback and unnecessary rebuilds. Independent review also found that rechecking old source under candidate physical context rejected a valid paired rename; candidate source/context now publish atomically. Sparse string/provider charges, obsolete lane owners and full-fallback provider membership were corrected at their owners. Tests retain successful refresh sharing while failed-candidate rollback requires restored usable semantics rather than an impossible promise to preserve a destroyed Salsa generation.

### Deviations from the plan, deliberate

Scientific-witness/direct implicit routes use explicit full-context dependence. Uncovered sparse replay constructs fresh current full-context mathematics and does not publish an invented portable/semantic receipt. This is the accepted conservative classification boundary, not unfinished finer reuse scheduled elsewhere. Canonical/source identity, standard accuracies, reserved defaults and other plans remain unchanged.
