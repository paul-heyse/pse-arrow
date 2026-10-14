---
title: Exact local selection and receiver rebinding
status: draft
date: 2026-10-14
adrs: [ADR-0183]
review_sources: [docs/design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md]
doc_owner: docs/plans/39-implementation-friction-and-codex-effectiveness.md
doc_topics: [architecture, preparation, validation]
---

# Plan 39a: Exact local selection and receiver rebinding

## Purpose, ownership and basis

**Proposed.** This companion supplies the local scientific input corrections for
[Plan 39](39-implementation-friction-and-codex-effectiveness.md): exact operation-shaped
membership access for immutable source revisions, a carried relation between a checked
selection and its actual receiver, and an internal rebinding operation for already owned
admitted source. It covers the review's [F04](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f04)
and [F05](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f05),
including the [product inquiry](../design_review/evidence/implementation-friction-and-codex-effectiveness-2026-10-14/product-inquiry.md)'s
external-import, parsed-reload and repeated-source analogues. The coordinator owns finding
disposition, common decisions and the combined completion boundary. This document alone
owns A1–A5 progress; scheduling a packet does not resolve either finding.

The source basis is `36f15656d`, inspected on 2026-10-14. The review's historical execution
and the Plan 38 checkpoints explain the existing state; they do not qualify this target.
Architecture and rationale remain with blueprint §14.2–§14.4, §19.1–§19.2, §22.2 and
[ADR-0183](../adr/0183-local-model-authority-and-scoped-validity.md). The foundation assessment
uses Core 3.4, Efficient Architecture Heuristics 1.0, ProcessSimulator 1.5 and the repository
binding, focusing on immutable authority, exact reuse, physical work, receiver compatibility
and ownership. It is a scoped planning assessment, not a new enclosing subsystem verdict.

The intended workload is ordinary interactive edit/preparation, including large unrelated
libraries, and explicit replay into a receiving runtime after publication closes. Studies,
recycles, dynamics, fits and horizons consume the same lower contracts. Their numerical
methods, new-state assessments and complete campaigns remain with their owners. No new
library, persistent cache, graph engine, service, generic certificate system or workflow
framework is selected.

All changes and verification below are proposed. This authoring work runs no product
checks, builds, tests or services. Local package verification during execution is limited to
compile checks and targeted tests. After **all** Plan 39 functional work and scheduled
inquiries finish, [Plan 39d](39d-qualification-and-plan38-cutover.md) owns assembled integration,
the remaining Plan 38 work and final qualification. A local A-packet does not start that
campaign early or replace its evidence.

## Current source and foundation assessment

`workflow/modeling/canonical/local.rs::AdmittedModelRevision` owns a complete immutable
logical-object `BTreeMap`, object payloads, parsed declaration products, allocation leases
and actual validation owners. `apply` clones the logical map once for an edit and retains
unchanged object Arcs by comparing actual membership, version, kind, payload and references.
That is a useful preservation boundary. Full initial source ownership and genuine
whole-source admission remain necessary work; the issue is repeating it for narrow
questions.

`LocalSourcePremises::logical` already uses direct lookup. Its `inventory` and `matches`
instead filter the full object map for every Scope, Kind, Name or References question.
The comparison correctly checks that no additional member exists, including for an empty
bucket, but exhausts a complete revision scan to do so. `canonical.rs::select_local` asks
these questions during real lexical selection, imports, members, suppliers and engineering
rule resolution. `checked_selection` repeats them for every cached candidate, and
`modeling/preparation.rs::prepare_selected_basis` immediately checks the same immutable
revision again. F04 therefore affects both cold selection and warm reuse; wrapping the
existing scan in an unowned cache would retain its cause.

`SelectedRequest` and `SelectedAdmission` already distinguish actual physical/provider,
registry and validation context from source dependency content. `CheckedSelection` carries
the admission but does not retain the source owner against which eligibility was just
established. The target should strengthen this existing product rather than create another
selection authority. Request equality and exact source premises continue to govern reuse;
fresh numerical-state domain, residual and closure checks are separate obligations.

`workflow/replay.rs::ScientificReplay::sources` deliberately emits one source position per
modeling step, and constructors retain those indices. `reconstruct_sources` calls
`ModelingPackage::reconstruct_replay_source` for every position. That helper reserves a
whole-source scratch envelope, exports a `ReplaySourceDocument`, and calls its public
reconstruction route, even though the complete immutable source and real providers are
still owned in process. The existing ordinary `rerun_with_cancel` is a useful counterexample:
it retains the original package and recreates case inputs without this transport boundary.
F05 concerns explicit receiver reconstruction, not every rerun.

`Runtime::admit_replay_source` is a sound independent boundary for supplied bytes. It
compares native/build, interpretation, actual physical source bodies and executable
provider registrations; checks duplicate identities, references, payload correspondence,
lexical completeness, ingress ordering/data and complete scientific identity; then creates
receiver-owned objects. Its per-declaration and per-data-member linear object searches are
unnecessary even at that trust boundary. `canonical/reload.rs::make_edit` likewise searches
`parsed.encoded_data()` for each changed document after already building a data map. That
bounded analogue warrants a lookup correction, not a claim that it is a dominant hotspot.

The foundation is adequate for exact local authority and explicit durable/local distinction.
Its physical execution is the mismatch: repeated scans and transport materialization hide
usable immutable ownership. Receiver validation and allocation scope are the remaining
material design question. A3 settles those premises before A4 removes checks. Immutable
source validity, receiving invocation validity, executable compatibility and publication
permission must remain distinct; naming them does not require a type or service for each.

## Target: exact revision-owned membership

The admitted revision owns derived access paths matching the existing `LocalInventory`
operations. The logical-object map remains the semantic authority. Index entries refer to
its exact immutable objects; callers cannot edit the indexes independently. The initial
realization uses ordinary ordered maps and immutable bucket owners, accounting for their
keys, containers, member references and retained metadata with existing allocation owners.

| Existing question | Required exact access and result |
|---|---|
| `Scope(scope)` | All objects whose lexical scope equals `scope` |
| `Kind(Some(scope), kind)` | All objects of the requested kind in exactly that scope |
| `Kind(None, kind)` | All objects of that kind across scopes |
| `Name(scope, name)` | All objects having that exact lexical scope and name, including ambiguity |
| `References(scope, name, kind)` | All objects of `kind` with an edge to that exact scoped name |

Results preserve the current deterministic logical-object order and membership semantics.
An absent bucket proves an exact empty inventory; an absent logical key proves exact
absence. Reference membership remains one occurrence of a matching object, as in the
current `contains` predicate: repeated reference edges must not multiply its membership.
Do not collapse ambiguous names or silently deduplicate source declarations. No digest,
version string, root ID or approximate membership replaces exact objects and absence.

Initial admission may build the indexes in one complete source pass. An edit removes old
memberships and inserts new memberships for each actually changed or deleted object,
including both old and new scope/name/kind/reference keys. A payload-only replacement must
replace the bucket's object reference even when its membership keys do not change. An
exact no-op preserves the object and unaffected bucket owners. Moving an object, removing
its final bucket member, adding a nearer lexical name, and changing reverse references
must all be visible. A revision and all derived memberships become usable together only
after successful staging; refusal, allocation failure or cancellation leaves the earlier
revision and its borrowed products intact.

Cold selection reads these exact buckets. Premise comparison reads the same access path,
using immutable owner equality where sufficient and exact ordered object comparison
otherwise. Unchanged bucket identity is an optimization under retained ownership, not a
new durable key. Large genuinely consumed buckets still require their members; growth of
unrelated buckets must not cause every narrow query to scan the complete source. Preserve
cooperative cancellation/deadline checks during substantial comparisons using the existing
operation context rather than adding a new timeout policy.

The logical map and index map containers may still be copied once per revision. That
cost and retained index metadata must be stated honestly. Persistent maps are **not** the
default remedy. Reopen their selection only when representative edit workloads show this
remaining copying to be material; the investigation must compare actual edit work, retained
memory and borrower lifetimes using the same exact controls. Indexing is not blocked on
that measurement and no quantitative improvement is claimed here.

## Target: carry checked selection through preparation

`checked_selection` returns the selected admission together with retained ownership of the
exact admitted source revision and actual request/receiver context for which it was checked.
Cold construction establishes that relation as part of selection. A warm candidate must
first pass request/context eligibility and exact indexed premise comparison; only then can
it carry the relation for the current source. A cache hit does not inherit another call's
active permission. Keep owning handles alive when using identity checks; a bare address
that can outlive its owner is insufficient.

`prepare_selected_basis` consumes this carried relation when source and actual context are
unchanged. It does not immediately repeat the complete premise comparison. If a different
package/revision or context is supplied, it obtains the current checked relation through
the same selection owner, or returns the existing typed mismatch; it cannot silently use
the old relation. Internal encapsulation must prevent callers from manufacturing that
association or substituting a different source behind it. Routine function/type layout is
left to implementation.

This change applies to every checked-selection/basis handoff, including engagement,
conformance/inspection and modeling analysis callers. Retained admissions and bases still
compare the complete dependencies required for cross-revision reuse. Changes to physical
context, executable provider/configuration, registry or actual validation invocation cannot
borrow an unrelated owner's validity. Immutable source facts may survive a value edit while
consumed changed payloads invalidate the relevant products. Each newly solved numerical
state still follows original formulation, bounds/domains and independent scientific
assessment; checked source ownership is not a solve certificate.

## Target: rebind owned source at the receiving context

The internal operation consumes an actual retained `ModelingPackage`/admitted closure and
the chosen receiving runtime. It retains complete exact source, physical and callable
owners, decides which source admission remains valid under that receiver, and returns a
receiver-local package. It does not first clone source bodies into a transport document.
The public supplied-document importer remains independent and performs full validation.

A3 must distinguish at least these premises from current executing source:

| Premise | Proposed reuse/receiver obligation |
|---|---|
| Immutable object bodies, parsed declarations, exact lexical/reference closure and ingress/data identity | Reuse established validity when the retained owners and the rules establishing these facts are unchanged; no reparse or payload rehash solely because publication closed |
| Interpretation and actual validation rules/invocation | Prove compatibility from retained actual owners and complete consumed semantics; re-admit affected properties when rules or invocation-sensitive premises differ |
| Physical quantities, dimensions, basis/reference conventions, visibility and actual physical source bodies | Retain exact compatible admitted physical owners; a same-named or equal-key foreign context alone does not establish compatibility |
| Executable providers, configuration, native build/ABI and registration/registry owners | Preserve real callable owners and actual receiving execution eligibility; changed or unknown premises require actual admission or truthful refusal |
| Allocation/resource owners | Existing payload/index/parsed leases remain held while shared; charge receiver metadata and new validation products to their actual owners; content equality never transfers or discharges another pool's reservation |
| Local receiver head, lineage, origin and engagement | Mint the new independent local receiver/head; clear source durable origin and held preparation/engagement; do not inherit a closed caller or its publication capability |

The selected minimum is a compatible internal rebind with no persistent replay cache.
Known changed validation properties may take a bounded fresh admission route; an unknown
or unsupported compatibility premise may retain full admission or refuse explicitly. A3
must record the exact supported branch and required new checks before A4 is implemented.
It cannot certify the receiver with content hashes or by merely comparing context names.
If source bodies need fresh receiver validation, consume their owned representation
directly where the lower admission permits; external serialization is not the definition
of fresh validation.

Preserve existing callable/physical inputs selected by replay; do not substitute a current
same-named provider. The new package retains original scientific source identity but has
its own receiver identity/head and no canonical source key. It must not reuse held compiler
engagement or mutable workers from the old caller. Subsequent editing branches from the
new local head without advancing the old one. Repeated positions within one receiving
operation may share that newly rebound immutable package. Independent receiving operations
must not accidentally share mutable head/engagement authority.

`ScientificReplay::reconstruct_sources` remains the public Rust composition and keeps its
existing positional output shape. Use an operation-local memo of exact retained source
closure plus its actual physical/provider/admission premises and the receiving context.
For positions `[A, A, B, A]`, return four usable packages in that order, with source index
mapping unchanged, while performing compatible binding of A once and B once. Same hash
but distinct or incompatible admitted owners are not a sufficient memo key. Sharing exact
owners is sufficient; broader content coalescing is unnecessary. Include only successfully
bound products; failure returns no partial success or fabricated origin, and releasing the
operation drops its temporary memo. Returned packages retain their own actual borrowers.

Keep the caller's explicit receiver selection. Closing publication does not close the
owned source, and source validity does not authorize durable effects. The durable runtime
must still refuse when required recording is unavailable; there is no automatic ephemeral
fallback. An explicit local receiver yields ephemeral fresh runs without canonical
run/attempt keys. A3 also inspects existing behavior for an explicitly supplied durable
receiver so the implementation preserves supported semantics or names a necessary refusal;
it must not silently change runtime durability or attach the source's old receipt.

Source reconstruction is shared by modeling steps, simulation, fit, horizon and the
feature-selected shooting route. Ordinary same-runtime `rerun_with_cancel` remains the
existing direct retained-source path. Python `ScientificReplay.rerun` currently selects its
original runtime; this plan does not assume a new Python receiver parameter/API. A concrete
Python consumer needing explicit receiver reconstruction is the trigger for separate
boundary design, not a hidden prerequisite to F05 closure.

## External import, reload and portable export

For `Runtime::admit_replay_source`, build one temporary borrowed exact logical-object map
while checking duplicate logical identities. Preserve duplicate lexical identity/reference
refusals. Use that map for declaration, context, ingress, physical-scope and data lookup;
keep the existing complete supplied-body checks and required full passes. Account for the
temporary map within the receiving reconstruction reservation and release it after the
operation. It cannot lend supplied bytes a prior local admission. Self-consistent hashes
with an omitted lexical dependency, malformed body, wrong ingress/data identity or changed
provider must still fail.

For `canonical/reload.rs::make_edit`, derive a temporary exact document-id view of
`parsed.encoded_data()` and reuse it for changed-data membership/payload lookup and content
hash access. Preserve duplicate handling, actual document visibility, inherited data
selection and add/change/remove intent. Inspect the adjacent per-bundle package filter:
use a temporary package-to-document view if it is part of the same repeated lookup, while
retaining complete visibility expansion. Do not decode inherited payloads merely to index
metadata, or turn a missing replacement into deletion. Reload remains source editing,
not an external-trust shortcut.

Portable `document_with_limit` also repeats source export for positional duplicates. A5
includes a bounded decision on the actual materialization and its consumers. The existing
wire contract is retained: all positional source entries and constructor indices, exact
source bodies, complete native/decision/constructor evidence, finite allocation refusal
and actual final encoded size remain. A local operation memo may avoid repeated conversion
only if it saves real work under that unchanged contract and charges all final duplicate
bytes. If the owned wire representation requires all copies, explicitly retain that output
duplication and its accounting; never subtract positional duplicates from final extent or
silently truncate required sources. Record the chosen same-format optimization or retained
duplication and why. A versioned unique-source table/index remap is not selected. Reopen
format design only for a demonstrated portable consumer/size requirement that cannot be
served soundly in the current format, with explicit compatibility/versioning authority.

## Packets and dependencies

| Packet | Prerequisite | Delivered boundary and verification | State |
|---|---|---|---|
| <a id="a1"></a>A1 — Exact immutable inventories | Existing complete admitted source, exact object equality and allocation owners | Revision-owned Scope, scoped/global Kind, Name and reverse-References access; edit staging updates all affected memberships. Cold selection and premise reads migrate; targeted inventory/edit oracle and collision/absence controls pass; remove production full-map per-query filtering. | Planned |
| <a id="a2"></a>A2 — Carried checked relation | A1 exact membership access and existing actual request/context ownership | Cold/warm checked selection binds exact source and receiver context; preparation consumes the carried relation. All selection/basis callers migrate; targeted cross-source/context and warm preparation controls pass; remove immediate duplicate premise checks on unchanged handoff. | Planned |
| <a id="a3"></a>A3 — Receiving-validity decision | Existing ADR-0183 validity/import distinction, actual source/physical/provider/validation/pool owners | Bounded investigation settles the compatibility table, resource lifetime and durable/local receiver branches. Record exact reuse, fresh admission or refusal outcomes and distinguishing controls at this owner; A4 cannot remove checks before these outcomes are settled. | Planned |
| <a id="a4"></a>A4 — Owned-source rebinding | A3 settled receiving contract implemented in the lower operation; A1/A2 when reusing their products | Internal rebind and operation-local exact-source memo used by every `reconstruct_sources` source variant; positional mapping, new local head/no inherited origin, actual leases and fresh science retained. Targeted repeated-source/receiver/refusal/close controls pass; remove retained transport reconstruction and its whole-source scratch path. | Planned |
| <a id="a5"></a>A5 — Indexed import/reload and portable-export decision | Existing independent importer/reload semantics; A3/A4 contract for distinguishing trusted and supplied routes | Temporary exact importer/reload lookups migrate all affected searches with complete validation; targeted malformed/duplicate/data/visibility controls pass. Settle same-format export memo versus retained positional duplication; preserve limits/indices and remove replaced repeated searches/conversion paths only when covered. | Planned |

A1 and A3 can start independently. A2 consumes working exact inventory behavior, not merely
an interface sketch. A4 depends on an actual receiving-validity result; agreement on a
method name is insufficient. A5's importer and reload lookup slices can proceed before A4,
but the final trusted-versus-supplied negative controls and export decision must compose
with its real behavior. The files `canonical.rs`, `local.rs`, `preparation.rs` and
`replay.rs` are shared editing surfaces, not permission for independent conflicting edits.
The coordinator assigns execution ownership and integration order.

### A3 investigation outcome and reopening conditions

Inspect `AdmittedModelRevision`'s validation owner/context, `Runtime::admission_context`,
`SelectedRequest`, compiler lineage, `PhysicalContext`, provider registrations and
`package_from_revision` together. Determine which admissions are immutable source facts,
which consume the actual receiver's registry/UDF/configuration/admission scope, and what
is required to keep allocations alive across source publication/caller close and a distinct
receiver pool. Use focused controls only where source does not settle equivalence. The
outcome must name supported compatible owners, changed premises that require new admission,
unknown/incompatible refusal behavior, lifetime charges, and any consumer-specific limit.
Publish the result here before dependent code, with the coordinator informed of contract
consequences. A wholesale replay redesign is not a default outcome.

No F04/F05 rule relaxation is requested by the review. RC01/RC02 belong to the coordinator's
producer work and do not block these corrections. ADR-0183's exact local/import trust and
actual admission scope are constraints to preserve. If A3 exposes a genuine changed
governing contract rather than a refactor within it, identify that exact impact and route
its confirmation/decision before dependent implementation; do not infer permission to
weaken receiving checks. Enduring architecture explanations use the normal design route,
not edits to accepted ADRs.

## Consumer migration, deletion and intermediate states

A1 includes all producers of `AdmittedModelRevision`, including fresh local admission,
edits, canonical reopening and independent replay import. Each must produce complete
derived indexes under the same semantic authority. Remove the per-query production
`objects.values().filter(query.matches)` inventory paths after indexed targeted controls
pass. A deliberately independent scan may remain in tests as an oracle or in genuinely
whole-source operations; it must not remain a second production inventory implementation.

A2 covers `checked_selection`, `checked_selection_scoped`, `prepare_selected_basis`,
engagement, conformance and modeling analysis callers, as well as retained selected/basis
reuse. Migrate callers with the new carried relation and remove the redundant handoff
check in the same change. Keep independent checks for a different revision/context; do
not delete them because one unchanged case now has an identity shortcut.

A4 covers `ModelingPackage::reconstruct_replay_source` and every variant produced by
`ScientificReplay::sources`, plus reconstruction consumers in `replay.rs`, the canonical
publication-close journey and shooting tests. Replace and delete the internal
export/import round trip and its redundant retained-source scratch reservation, callers,
mechanism-specific tests and fixtures once the replacement is exercised. Retain
`replay_source`, `ReplaySourceDocument::reconstruct` and `admit_replay_source` for actual
portable export/import; they are independent trust boundaries. Constructor reconstruction
continues to consume the positional package vector. No compatibility shim or dual
production rebind path remains.

A5 removes replaced per-member vector searches and any conversion work actually replaced
by its same-format decision. Keep semantic validation tests and update their route, while
deleting tests that only enforce the retired transport mechanism. Reconcile operation/doc
comments so internal rebind no longer promises full external re-admission. No stored data
migration, native provider rebuild, replay version bump or Python API expansion is planned.
After A1/A2, ordinary cold/warm preparation is usable while receiver rebind remains pending.
After A3/A4, owned-source transfer is usable while supplied import still independently
validates. These are useful intermediate boundaries, not assembled acceptance.

## Proposed verification and closure

Every execution result records command, mode, scope and actual failures against the zero
baseline. During packets, use `just check-package pse-runtime` and scoped
`just unit-package pse-runtime <filter>` through the repository environment, with the
correct explicit force-validation closure and feature selection. Test names below identify
existing controls to preserve or behavior to add; naming a test here does not claim it ran.
Targeted controls must reach the real owner and consumer rather than only a replacement
helper. Store/native journeys are selected at final qualification by Plan 39d; pure index
and receiver-policy controls should avoid unrelated service startup.

| Scenario / obligation | Proposed distinguishing evidence and expected result |
|---|---|
| Review S01: cold selection and warm preparation over a narrow root plus a large unrelated inventory | Compare exact inventories with a simple independent full-scan oracle; inspect the actual cold, candidate-hit and basis handoff paths. Unrelated buckets are not visited for every exact query, and the immediate same-source handoff does not repeat the check. A targeted test may use local test-only access counters; no permanent metrics framework is required. |
| Review S06: membership/invalidation | Add/delete/replace, scope/name moves, scoped/global kind changes, reverse-reference changes, empty/final-member buckets, absent logical/name observations, nearer shadows, imports and ambiguity preserve deterministic exact results. Retain `local_exact_premises_reject_payload_collision_and_new_shadow` and `complete_namespace_inventory_resolves_nearer_names_without_leaf_or_import_probes`; equal correspondence keys with changed bytes still reject stale reuse. |
| Carried relation and changed context | Same retained source/context uses the carried relation; unrelated edit can reuse only after exact premises pass. Supplying a different source or physical/provider/registry/validation context cannot consume the old relation. Value edits invalidate actual consumed payloads while new numerical states still receive fresh original checks. |
| Resource lifetime and interruption | Old revisions/buckets/parsed products remain valid and charged while borrowed through edits or cache eviction. Failed staging leaves the predecessor usable. A large consumed bucket comparison honors the enclosing cancel/deadline; temporary rebind/import views release, escaped products remain owned and no reservation transfers by content equality. |
| Review S05: publication close and explicit receiver | Preserve `solve_durable_replay_after_publication_close_requires_explicit_local_receiver`: durable start refuses before new native dispatch; explicit local receiver retains exact source/callables/original solve identity, avoids reparsing unchanged admitted declarations, creates a new run with original independent root/residual checks, explicit ephemeral durability and no canonical keys. This is logical publication close, not physical server-loss/restart evidence. |
| Repeated source positions and independent receiver heads | `[A, A, B, A]` binds each compatible retained closure once and returns four positions with unchanged constructor indices. Distinct incompatible same-hash owners do not coalesce. Separate receiving calls get independent local heads; edits and failures cannot advance the original source or another receiver. |
| Receiving context refusal/re-admission | Changed executable/configured provider, native/interpretation premise, physical convention/source, validation owner/rules or unsupported resource lifetime obtains the exact new admission required by A3 or a truthful refusal. No old receipt, acquisition permission, publication capability or held mutable engagement is inherited. |
| External supplied body remains untrusted | Valid transport reconstruction still agrees with independently specified source/constructor meaning. Malformed/duplicate logical or lexical identities, duplicate edges, missing lexical references despite internally consistent hashes, altered payload/data/ingress ordering/context/physical sources or provider registration still refuse through full external validation. |
| Parsed reload analogue | Exact changed-data lookup preserves document visibility and intended additions/replacements/removals; unchanged inherited payloads remain unopened. Missing/duplicate members retain admission behavior. Compare the resulting admitted source meaning with fresh ingress, not just with the new index's own output. |
| Portable export decision | Multi-step duplicate sources preserve the exact positional document and all constructor indices; requested/final size limits reject incomplete export without truncation. A same-format memo must retain accounting for duplicate final bytes. Otherwise record deliberately retained duplication and the versioned-format revisit trigger. |
| Review S07: new workflow or preparation mechanism | Existing modeling, simulation/fit/horizon and selected shooting consumers use the same selection/rebind contracts. A new workflow supplies its own constructor/policy and fresh numerical checks; it does not implement its own lexical scanning or source transport admission. No claim is made that this local evidence qualifies every workflow campaign. |

F04's local closure requires A1/A2, the exact negative controls and a source trace of cold
and warm preparation proving the repeated query-by-complete-revision and immediate recheck
are removed. F05's local closure requires the A3 decision, A4 and A5's applicable importer/
positional controls, with source tracing that compatible owned rebinding preserves validity
without transport re-admission. The coordinator records the finding dispositions only from
that evidence and Plan 39d's required assembled result. There is no quantitative latency,
throughput, memory or convergence claim; an end-to-end representative interaction campaign
is required only if such a benefit is claimed, using exact unchanged/edited/large unrelated
source cases and actual receiver/export lifetimes.

## Current checkpoint and handoff

**Proposed:** A1–A5 are planned. No functional changes, executed product verification or
finding closure are claimed. The next executable work is A1 exact inventory ownership and
A3 receiving-premise investigation, under the coordinator's execution authorization and
shared-file assignment. Once their outcomes are available, proceed through the dependencies
above; update this packet table instead of creating another status ledger. At completion,
retain the source findings' exact closure limits, move enduring contracts to their existing
owners through the normal decision/design route, and hand local evidence to Plan 39d for
the combined Plan 39 qualification and Plan 38 cutover. The final Outcome must record actual
evidence, a mistake corrected and deliberate deviations; this draft supplies none of those
execution claims.
