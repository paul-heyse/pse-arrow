---
title: Scientific artifacts, history and policy boundaries
status: in-progress
date: 2026-10-12
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md]
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, design, storage]
doc_retention: while-dependent
---

# Plan 37a — Scientific artifacts, history and policy

## Contract and baseline

This companion develops F01–F03/F07 and confirmed RC01–RC03/RC08. [Plan 37](37-surrealdb-lifecycle-and-integration.md) owns decisions, package status, dispositions and overall acceptance. This document owns the scientific design and targeted controls, not another backlog.

**Implemented, source-inspected:** study admission in pse-runtime workflow/study.rs checks bindings and seed roles, while prepare_study_candidate_in_lane acquires explicit starts later. pse-model study.rs owns PointPolicy, StartPolicy, facts and decisions; pse-operations study_policy.rs implements effect-free admission/transition policy. Canonical retention and generated surreal_retention.rs retire whole runs and retain detailed occurrence/operation history. Existing canonical_analyses.rs/surreal_analyses.rs supply a useful protection-to-durable-input handoff.

The [scientific inquiry](../design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/scientific-artifact-lifetimes.md) and independent authoring advice were reconciled against source. **Proposed:** changes below. No product tests or scientific probes were run. OP01 selects a fresh target store: none of this work needs conversion of existing records or historical readers.

## Scientific meaning and ownership

The scientific owner decides what a selection means and what is sufficient to interpret/use it. Canonical storage preserves that product, authorizes new reads and retains its required dependencies. A qualified start, useful final state, selected trajectory, local sensitivity result and restart checkpoint are different products. Promotion preserves original qualification; it cannot improve a partial/failed result or certify an arbitrary byte subset.

Use one small artifact/reference vocabulary with kind-specific scientific admission rather than a new service/table/type for every taxonomy row. Its conceptual contract includes immutable selection identity, issuing store and interpretation, original producer coordinates/assessment, a scientifically admitted content closure and explicit independent obligations. The registry owns boundary vocabulary and persisted fields. Rust scientific owners supply closure; storage does not reconstruct it from table names.

| Product | Owner-supplied sufficient content and preservation |
|---|---|
| Start/seed | Exact original solution/run/attempt/step; closed inventory membership; typed payload and bit-preserving digest; layout/preparation/profile/data/backend; original reuse assessment and optional prediction permission |
| Final state | Original physical coordinates, unit/basis and model/case interpretation; endpoint/coverage and original assessment. A last sample alone is not automatically a complete state. |
| Selected trajectory | Selected coordinates/rows plus required sample, event, mode, candidate assessment and endpoint interpretation. Requested and observed extents remain distinct. |
| Local scientific analysis | Corresponding quantities plus conditioning, assumptions, local validity and withheld reasons; preserve KKT/derivative assessment. A scalar copy cannot claim stronger validity. |
| Persisted graph analysis | Exact selected source/result inputs and graph interpretation under its existing owner; do not conflate this operation with KKT-derived local analysis. |

Actual source starting points are workflow/durable.rs seed admission; workflow/modeling/trajectory.rs::relation_ids; workflow/local_analysis.rs; study output-demand types; and canonical_analyses.rs. Output demand helps select data but does not establish scientific sufficiency.

Provenance means exact attribution and enough interpretation/assessment to use retained content. It does not automatically mean reproducing every operation that generated the content. Downstream reproducibility from retained exact input bits is narrower than complete producer reconstruction. Require transitive original inputs only when the promised product needs them; a selected explicit audit obligation can preserve fuller history.

## A0. Scientific closure and collection decisions

The [completed A0 investigation](37-investigation-evidence/scientific-closure-and-replay.md) supplies source-backed sufficient/insufficient selections, physical alternatives, affected entrypoints and revealing tests. Its selected contracts govern A1–A3:

- Use small owned seed capsules, and artifact-owned bounded IPC copies for selected final-state, trajectory and local-analysis rows. Preserve exact original bits and row identities. Shared producer-block references are not the initial target: copying the admitted scientific selection removes producer collection coupling. Revisit sharing only for measured repeated large overlap with a complete independent block-retention contract.
- Keep persisted graph analyses under their existing owner and narrow their inputs to independent artifacts. Original opaque producer IPC locators are provenance, not active authority after producer release. A2 emits typed artifact-owned block/row/field locators or exact manifest mapping, with complete selected graph endpoints and quantitative evidence.
- Before final-state/trajectory promotion, A2 materializes endpoint mode/integral/input coordinate and unit interpretation while original preparation exists. Local analysis includes original case/binding, statistical assumptions, validity and required upstream input context. Missing interpretation refuses the promised promotion; no consumer guesses array meanings after retirement.
- Reduced replay validates immutable owning store/run/study context before absence or mutation. Preserve operation identities/hash meaning. Caller-asserted replacement context cannot prove original ownership; retain compact operation-to-owner locators where an independently validated immutable binding cannot otherwise survive receipt deletion. Audit-required detail and unknown effects remain protective. The source entrypoint inventory and both-order tests are in the investigation.
- Native references and ENFORCED endpoints express fitting structural links only. Explicit creator/reclaimer conflict, permission, scientific closure and borrower guards remain authoritative. Keep owned bytes and disabled VERSION; neither files/history/compaction nor remote numerical resume is selected.

RC01–RC03 authorize these target choices. D0 still routes new relation/history/public-boundary decisions before A1–A3 implementation. A0 is complete as a design investigation; it does not establish an implemented artifact or retention mechanism. No producer context requirement is deferred to Q1.

## A1. Retained exact study inputs

New reproducible durable studies with explicit external starts resolve their exact scientific selection before study activation. Current StartPolicy::Explicit identifies intent; the admitted input operation establishes actual availability, compatibility and a durable obligation or owned scientific capsule. Keep role/layout validation under the existing scientific owner.

Admission reads the original protected producer selection, establishes sufficient exact content and transfers protection into durable consumer ownership under the producer's retirement conflict authority. Stage large immutable content outside the short visibility transaction while maintaining its protection. The final activation verifies the admitted closure and installs references atomically; uncertain submitted creation keeps its exact identity/protection until settlement. Never hold a DB transaction across a native solve or numerical validation.

Deduplicate retained input obligations within a study by exact selection, while keeping per-occurrence start provenance and scientific role. Pending dispatch, worker replacement and retry resolve the retained input, not a new producer lookup. Once acquired in process, AdmittedSeed owns its immutable content; do not require renewal of producer permission for every numerical use.

Admission-first preserves the input when the producer's own obligations are released. Retirement-first refuses admission explicitly. Neither case silently becomes a fresh start. A deliberate ephemeral/acquisition-time request can retain honest late-bound availability, but it must not claim durable exact input retention. Do not introduce a second public mode or policy flag without an actual consumer; the new retained path is the durable default, and any retained late-bound route must be explicit and documented.

A conservative producer-run obligation can establish A1 first if necessary. It cannot close F02. Move final acquisition/release callers onto A2's independent artifact owner and delete the coarse prerequisite once replacement controls and all callers are complete.

Consumer scope includes study creation/activation, candidate preparation, durable retry/recovery, seed eligibility discovery/admission, per-point provenance, cancellation/retirement and Rust/Python workflow entrypoints. Generated declaration/codecs must represent input ownership once. Updating only discoverability or a seed ID field is insufficient.

**Proposed targeted controls:** admission/removal in both commit orders; absent/incompatible seed refusal; restart after admission before dispatch; retry after worker loss; shared exact input with independently released studies; cancelled/failed staging with uncertain creation; no fresh fallback; original bits/assessment unchanged. Existing in-process admitted seeds remain usable after producer retirement. Producer DB reads through an expired protection still refuse.

## A2. Independent artifact promotion and retirement

Provide owned operations to promote a scientifically admitted immutable selection, acquire/read it, attach/release an obligation and retire/collect it. Concepts describe required behavior, not mandated API spellings. Promotion must atomically transfer current input protection into artifact ownership and return stable identity/provenance. It does not rerun science or reopen a retired producer.

Consumers retain the artifact's sufficient closure, not an incidental entire-run lifetime. For a small start, a copied capsule may eliminate shared-payload collection complexity. For large trajectories, exact selected immutable blocks and metadata can be referenced; if a block contains both retained and unretained rows, choose a scientifically lossless selected copy or retain the block explicitly and disclose that physical granularity. Do not claim row-level reclamation from block-level retention.

Independent obligations can coexist: retained study input, explicit user/project reference, active analysis, live protected read and deliberate audit. Their release ordering cannot remove content needed by another. Retiring the original run closes producer reads/admission as before, while promoted content is read from its new authority. Returned copied arrays keep their allocation owner; their existence does not authorize future pages from a retired DB object.

Promote sufficient final states, selected trajectories and analyses through their scientific owners. Preserve physical units/basis/conventions, original guards, source/model/case interpretation, event/mode metadata, original partiality and withholding. Generic storage selectors must not assert seed/restart permission because bytes exist. Preserve current per-state numerical checks; immutable content reuse never substitutes for checking a newly computed state.

Update canonical analysis input handoff and any result/seed readers to express the narrow retained selection. Current persisted graph analyses protect whole run/attempt/manifest inputs; migrate that protection to the scientifically sufficient artifact where applicable, without losing actual graph source meaning. Keep whole-run retention as an explicit requested scope when the user really wants it, not the only protection mechanism.

Collection remains bounded, resumable and conflict-registered. Test creation/retirement in both orders, release while a read is live, restart during cleanup and independently released shared borrowers. A collector sees unknown creation/effect ownership as protective. Exact settled release cannot resurrect a retired artifact through operation replay.

**Proposed acceptance:** preserve one seed/final state/analysis and selected trajectory, retire the producer, then demonstrate identical retained content/assessment and collection of unrelated payload. Refuse insufficient selections. Release multiple obligations in different orders and prove final collection after the last one drains. No measured disk-capacity claim follows from logical fixture row counts.

Delete coarse-only consumer retention and replaced selectors/tests with the new caller path. Avoid compatibility re-exports or a parallel production artifact API.

## A3. Releasable history and dominating retirement

Separate explicit scientific/audit history from minimal irreversible effect/identity fences. New default retained history lasts until its owner releases it; release is explicit, never elapsed-age deletion. A deliberately permanent audit obligation remains valid. A terminal run alone does not authorize history deletion.

Before deleting headers/details, ensure run and study retirement dominate every creation, append, activation, claim, settlement and recovery path. Source inspection shows run available/begin_run already checks canonical_result_retirements, but surreal_studies.rs::create currently compares an existing header and checks its supplied run without consulting retirement by study identity. The target must refuse a retired study ID even when reused with a different run. This is a prerequisite for new deletion behavior, not a claim of a current deletion bug.

Define a compact fence/attribution record sufficient to reject resurrection and classify unavailable history. It need not retain every request/diagnostic. Where immutable request equality remains necessary, state the exact retained comparison and collision/interpretation contract; do not replace it with an unexplained hash. Where a dominating permanent identity fence makes detailed replay responses unnecessary, remove that detail.

After authorized release, old calls return an explicit retired/history-unavailable result or the defined minimal settled attribution. They must not reconstruct missing content, return an invented successful original response or make an old identity usable again. Known incomplete/failed scientific outcomes remain truthful. Unknown effects, active grants and unresolved creation windows retain the records needed for exact settlement, even if a caller has requested release.

Register the exact governing guard/identity before destructive effects. A clock expiring is not proof a previously submitted transaction cannot commit. Both collector-first and late-creator-first orders must be fenced. Remove selected study points, policy/facts/outcomes/start/attempt detail, dependencies, readiness/cursors, run/attempt requests and headers only according to the reduced replay contract and actual remaining obligations. Bounded pages bound each effect; total retained detail must follow selected obligations rather than occurrence count forever.

**Proposed targeted controls:** explicit release of a completed exploratory study drains unselected detail; audit-retained history and unknown effects survive; existing live reads/inputs prevent premature cleanup; restart during every collection boundary is safe; replay of the original study ID with a different run refuses; delayed creation/append/recovery cannot resurrect content. Test surviving compact receipts directly, not just disappearance of payload.

## A4. Production policy compile and test boundary

Move the actual effect-free study_policy implementation and its direct unit tests from pse-operations into pse-model beside the existing study semantic types. Use the existing pinned petgraph dependency; preserve diagnostics and behavior. Canonical/runtime adapters consume that implementation. No new crate, copied surrogate or compatibility re-export is selected.

Audit every direct caller and public import, including candidate action, transition/retry, admitted-study graph analysis and tests. Storage observations stay in canonical adapters. Native preparation, scientific seed acquisition and execution stay in workflow/scientific owners. Do not drag transport or Arrow into pse-model to accommodate an adapter test.

Keep schema/transport integration tests with their real SDK/codegen/Arrow dependencies. Move only policy's genuine local tests and remove their old source. A filtered test name in pse-operations cannot establish compilation isolation; verify the new production target's normal and selected dev closure and same-policy calls from adapters.

**Proposed commands:** just check-package pse-model; just unit-package pse-model study_policy; scoped checks/tests of affected pse-operations/pse-runtime callers. Use existing scope-aware routes for Arrow-consuming consumer checks. Compare manifests and actual compile/test closure; report separately from build duration. A reused cache or absence of a running DB is not dependency evidence.

## Target transition, change scenarios and verification

Start from a fresh target store under OP01. Install artifact/history contracts and dominating fences before enabling collection. All production producers/readers/callers move together to the target generated schema; remove replaced legacy history/activation/reader paths instead of supporting old stored data. Source definitions and seed fixtures can regenerate scientific content/evidence. Existing plans are reconciled only by H1 after acceptance.

A new unit/property family extends the scientific closure/interpretation owner and declarations, not generic storage policy. A new solver start format adds its scientific payload/compatibility admission, not an exception in history cleanup. A new study policy changes the pure owner and its tests, then adapters supply observations. These boundaries support the functional target without a universal artifact or workflow framework.

Targeted package controls are **Proposed**, not executed evidence. Q1 owns assembled qualification of fresh studies, retained input reuse, artifact/analysis consumers and actual Rust/Python/native lifetimes. No new numerical algorithm, full unit/property conformance campaign or durable integrator checkpoint is implied. Physical/numerical checks and complete originals remain independent preservation obligations.
