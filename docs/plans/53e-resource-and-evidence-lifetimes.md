---
title: "Plan 53e: Resource and evidence lifetimes"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, storage]
---

# Plan 53e: Resource and evidence lifetimes

## Target and preserved protection

**Proposed:** complete [R07](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r07)
with [PI03–PI06](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI03): immutable
collected invocation facts, compact protected evidence reads, independent physical settlement and
selective discovery. The [coordinator](53-test-architecture-target.md#finding-dispositions) owns
progress. Existing FileStore/resource/process/report owners remain authoritative; no event journal,
lease service, general scheduler or parallel outcome store is selected.

**Interface-checked:** current mutable invocation state includes both full selection and resolved
requirements, and terminal evidence copies requirements. Current assessment selection loads the
whole v5 receipt, including its source map, before host bounding. `active_ids(limit)` sorts the
whole active population before slicing. These observations motivate changes in physical work;
small returned output is not proof of a small read. Benefits remain unmeasured.

DTA-041 stays withdrawn: Pending/uncertain registration, legacy-writer fences, exact borrowers,
late effects, terminal freeze and unknown retention have live consumers. DTA-040's aggregate
snapshot/reuse semantics remain valid. Original execution, current applicability, custody,
reclaim eligibility, logical close and physical drain remain separate meanings. Drop/drain is
not Pass. Completed 52k supplies canonical consumers; it is not additional delivery work for E.

## Work packages

<a id="e01"></a>
### E01 — Immutable collected identities and requirements, mutable membership

Use the existing FileStore declaration/state seam. Seal actual collected identities, executable
association and resolved semantic requirements/roles into one immutable descriptor, or bound
immutable parts with one publication meaning. Bind invocation identity, nonce and digest.
Semantic requirements describe demand; actual acquired service/generation bindings stay with
invocation/fixture ownership. Agree exclusion meanings with F01/F02 before sealing their consumers.
Mutable state retains phase, producer/composer/member references, setup observations, projections
and terminal references, without complete selection or requirement maps on each update.

Python requirements may precede collection, and nested collectors may be authenticated subsets.
Preserve an explicit uncollected staging phase, ownership and conflict refusal; collected-empty
is distinct from uncollected. Seal only complete selected correspondence. Publication cannot admit
undeclared effects in a header/state gap. Within an operation, use an indexed validated view of
immutable selection/requirements instead of repeated file reads/scans. Actual parent/executable,
phase and before-effects admission stay fresh.

C01/C02 supply normalized selection. Preserve invocation→identity→resource lock ordering,
original deadlines, Pending/not-started→uncertain→registered, `_settle_members`, context admission
and terminal immutability. Migrate registration, recovery, settlement, diagnostics and frozen
receipt consumers as one cutover under inquiry I09 below. A digest never replaces semantic checks.

**Acceptance:** membership changes rewrite neither full selection nor requirements. Interleave
collection/publication/saves; exercise Python precollection, nested subsets, collected-empty,
stale/mismatched header, duplicate/late registration, undeclared effects, context admission and
terminal freeze. Preserve original diagnostic association. Delete repeated payload construction
and old production layout/callers after the bounded transition and targeted controls pass.

<a id="e02"></a>
### E02 — One outer settlement clock and independent sibling results

Define the original settlement clock at the complete fixture join, then propagate it through
`CanonicalStore::remove_isolated_fixture`, `FixtureLifetime::drain_connections`, Rust
`testing::resource_bridge` and Python drain/drain-context/record-drain. Current task-local drain
clocking does not cover renewed bridge/Python waits. Do not restart a full 50/45-second budget at
a bridge or each sibling. Retain the original cancellation and already possible effects.

Snapshot required owners, preserve cache-eviction/disconnection and other necessary dependencies,
and attempt independent siblings while retaining their individual errors/unknowns. Sequential
attempts with collected errors suffice where no stalled sibling blocks another. Use existing
Tokio `join!` or dynamic all-results composition where independent progress requires it;
fail-fast `try_join!`/task-group sibling cancellation does not discharge this contract. No new
timer, cancellation manager or generic task scheduler is needed.

Logical Python `Runtime.close` releases a caller, not the native receiver. Observer detachment,
helper exit and actual native/client/reader drain remain separate facts. Mark Drained only when
all required physical owners settle; an escaped reader stays protected even when other siblings
finish. Continuing independent cleanup cannot authorize dependent unsafe effects. C04 consumes
cleanup separately from frozen assurance. Independent reclaim targets also retain their own
failure outcomes rather than aborting the remaining eligible attempts.

**Acceptance:** first-sibling failure, stalled sibling/second-sibling progress, original deadline
expiry, repeated cancellation, late effect and retained reader. Assert no renewed clock at bridge
boundaries, complete error detail, continued protection and unchanged frozen assurance. Remove
short-circuit loops after ordering and retention controls pass. Endpoint specialist teardown is a
separate Plan52 inquiry, not this package's implementation or completion dependency.

<a id="e03"></a>
### E03 — Selective service/generation access with established coverage

Maintain narrow service/generation active-access discovery under existing FileStore authority,
including creation borrowers. Do not derive the lookup by enumerating/sorting all active reports
and filtering afterward. Exact identity/membership remains authoritative; markers are discovery
projections, neither leases nor execution permission.

Establish activated writer/index coverage and the legacy/unknown boundary before absence can
support a maintenance decision. Publish protection before authority exposes effects; remove
authoritative membership before marker removal. Stale/missing/corrupt/ambiguous state resolves
through authority or refuses conservatively. An uncovered writer cannot look like service quiet.
Schedule bounded conversion/activation or explicit refusal, not a recurring whole-store audit.

Update projection entries on relevant membership transitions. `_save` currently republishes
active markers on repeated active saves; adding unconditional marker writes to every borrow/
release would replace read amplification with write amplification. Preserve protective ordering,
durability and interrupted-write recovery while removing that repeated work where replaced.

**Acceptance:** relevant-cohort discovery, registration versus maintenance, retained reader,
creation, generation replacement, interrupted marker/authority steps and unknown legacy writer.
F02 consumes this contract but never treats index absence alone as service-quiet proof. Delete
whole-population scans only where the scoped contract safely replaces them; retain necessary
bounded recovery/refusal for unknown state.

<a id="e04"></a>
### E04 — Protected evidence custody and nonempty scratch discovery

Seal reports, authenticate manifests, protect reused origins and retain exact readers regardless
of disposability. Maintain a separate nonempty scratch-candidate projection. All-protected sealed,
pinned, referenced or borrowed reports never enter routine scratch discovery solely because they
are active. A bounded prefix of ineligible reports must not starve actual scratch candidates.

Migrate sealing/candidate publication and reclamation together, using E03's transition-write
reasoning. Candidate presence is not permission to delete; recheck authoritative eligibility before
effects. Original inspection and origin reuse survive. One candidate error does not prevent other
independent eligible candidates from being considered. Cleanup cannot create/rewrite assurance.

**Acceptance:** all-protected/reference-only report absent from candidate discovery; real scratch,
active reference, manifest/path/digest tamper, racing acquisition, interrupted publication and a
failed candidate followed by independent success. Remove obsolete coupling between sealing and
disposal plus full-active-population candidate scans; retain authentic custody and recovery.

<a id="e05"></a>
### E05 — Demand-shaped protected readers and compact atomic progress

Publish one small atomic root with mandatory summary, exact source-cut/selection references and
immutable gate/detail references. Detail is durable and protected before its root pointer. Native
assurance, required/selected/unreported populations, source applicability, detail availability and
presentation completeness remain distinct. Provide selected gate/failure reads, explicit source
detail and explicit full export. Select sections before acquiring detail; ordinary result reads
must not load all source digests or unrelated gate detail, compile, run science or traverse services.

Use one versioned owner-reader boundary across actual producers, readers and custody operations.
Inquiry I09 includes at least these families; follow discovered consumers before cutover:

| Family | Current consumers/owners to migrate |
|---|---|
| Assessment/result reads | `validation_result.read_receipt/read_assessment/selected_result`, canonical assessment bindings and `run_inspection` product resolution |
| Reuse and final outcome | `validation_receipts._reuse_guarded` and original-origin reads, `validation.run_gates`/final summary, `capabilities.qualification_worker` |
| Export, measurement and analysis | `evidence_sources.assessment_snapshot`, Plan51 import/projection/store readers, `case_measure.functional_reference` |
| Publication and custody | `test_resources.finish_report`, report manifests/sealing/reference transfer and `test_archives` report production |
| Invocation representation | `run_inspection` and `evidence_projection.invocation_snapshot` FileStore-version readers |

Classify report meanings rather than force them into one giant wire type. Canonical middleware
remains a final encoded-size safeguard, not the authority deciding which claims survive traversal.
The review's synthetic source-map omission was truthful; acceptance must demonstrate a useful
complete selected answer, not merely suppress that omission flag.

Coordinate 52f F01's native questions and 52g consumers: optional sections and per-section bounds
must be explicit before acquisition. Current `EvidenceAnalysis::ask` limits primary rows, while
contexts/other sections use `LIMIT 10001` without equivalent completeness treatment. Preserve full
semantic denominators and ambiguity checks before display limits. Full exports are legitimate
explicit operations; Arrow IPC serialization does not supply custody or admission. PI09's finite
multi-question extension is optional and separately owned by 52f F03.

**I09 decision product:** inventory selected-header/report readers and origin protection; choose
authenticated one-time conversion where old identity/selection/terminal/detail can be established,
or explicit automatic-reuse refusal with protected original inspection. Activate one production
write/read layout, migrate actual consumers and remove replaced production branches. Unknown old
records remain protected, never empty/pass. If protected originals require an offline conversion,
schedule it before deleting their only inspection route. No permanent dual store or event journal.

**Acceptance:** detail-before-root interruption, failed atomic replacement, corrupt/dangling refs,
large source map with a complete selected gate, omitted/unreported scope, independent section
completeness, exact full diagnostics and full export. Selected reads avoid unrelated detail opens.
Exercise a reused gate whose origin is older: acquire/transfer origin protection before publication
and hydration. Missing detail is unavailable/refused, not empty/pass. Check authenticated migration
or explicit reuse refusal, offline service-down inspection and same-cut native/canonical answers.
Retain parsed native Cargo/unit-graph and actual force-validation checks despite compact roots.

## Sequence and integration

E01/F01/F02 agree semantic requirements/exclusion before immutable consumer migration. E02 can
proceed independently through its actual bridge owners. Settle I09's reader/custody transition early;
E01/E04/E05 migrate their relevant producers and consumers together. E03 supplies F02's access
protection; E04/E03 can share discovery/publication primitives without conflating authority.

One writer coordinates shared FileStore/result-reader changes. 53g and 52g/52h can reference the
same applicable interruption/retained-reader/offline journey while keeping native outcome, host
presentation, custody and physical cleanup acceptance separate. No measured speed/capacity claim
or endpoint teardown delivery is required to close this scope.
