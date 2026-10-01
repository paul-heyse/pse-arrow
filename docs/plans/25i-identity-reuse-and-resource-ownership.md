---
title: "25i: Identity, reuse and resource ownership"
status: in-progress
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s07]
---

# 25i: Identity, reuse and resource ownership

## Context and target

F15/F30 and FU11/FU12 concern incomplete reuse identities, coarse recomputation and ownership
that does not follow actual work/storage. R10 requires fresh revision attribution over shared
mathematics. This plan also owns F27's profile worker failure behavior, F26's hex utilities and
F32's accelerator/supervisor obligations.

Separate mathematical identity, admitted closure, exact source publication, value binding,
result lineage and operational job identity. Share immutable products; bind each revision's
provenance explicitly; keep mutable evaluators, cancellation and budgets attempt-owned.
A cache retains owners but is not itself the proof of allocation or native-work lifetime.

## Decisions and interfaces

- Centralize typed framing in the identity owner, including float canonicalization. Match the
  existing float contract: canonical NaN payloads, distinct signed zero and infinities. Distinct
  semantic hash roles prevent preparation, binding, profile, lineage and operational identities
  from being swapped accidentally.
- Replace JSON-text warm-start/profile hashes with new named frame versions; migrate callers
  directly. A display/source spelling is not a substitute for admitted meaning. G owns any
  persisted column/frame migration; keep no parallel legacy hashing API. Classify every affected
  frame, including raw-bit float users: either its admitted domain leaves the preimage unchanged,
  or introduce a new frame version. Preserve recorded old digests and lineage as immutable facts;
  never globally rehash history under new rules.
- Immutable mathematics and revision provenance have explicit shared owners. Value-only
  rebinding shares unchanged provenance and allocates/charges new binding products only.
  A new revision binds fresh source attribution even when its arithmetic is identical.
- Body admission uses complete semantic dependencies: normalized expression, called functions,
  provider/capability descriptors, physical preconditions and relevant limits. Bindings are
  immutable keyed inputs, not one overwritten slot per root. Salsa handles remain local;
  owned products escape the database.
- Use the existing DataFusion byte-bounded cache mechanism for service-scoped immutable views
  and package retention. Its size callback is a retention measure, not a substitute for unique
  allocation leases. Positive entry overhead is accounted even when a rebound value adds no
  large payload; do not insert a fictitious zero-sized entry or charge shared graphs repeatedly.
- Resource guards move with dispatched native work through completion and required cleanup.
  Caller abandonment requests cancellation; it cannot return CPU admission before work stops.
  Release CPU between genuinely idle staged steps. Step-only foreign allowances last through
  step completion; session-retained state, job/TLS and their allowances last through teardown/join.
- Durable workers share immutable admitted input and programs, not mutable provider workers or
  native attempt state. Declaration replacement preserves explicitly selected accelerator/provider
  inventory and invalidates only products whose dependencies changed.

The selected tools already fit: Salsa for local incremental admission, the existing cache owner
for bounded retention, and the current native supervision/thread model. A generic new scheduler,
persistent Salsa identity or a cache-container-only repair was rejected.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="i1"></a>I1 Canonical framing and roles | Existing identity contract; hashing decision route | Centralize framing, role types and small hex utilities | partial: 25c/25d prerequisite slices |
| <a id="i2"></a>I2 Immutable/binding allocation ownership | Existing prepared-product owner | Share immutable/provenance payload; charge new allocations; retain escaped owners | planned |
| <a id="i3"></a>I3 Complete bounded reuse | I1/I2; H1/H2/H4; A3 | Body-level queries, immutable bindings and service-scoped view/package retention | planned |
| <a id="i4"></a>I4 Completion-owned native admission | Existing staged/session owner | Transfer guards with work and release at actual completion | planned |
| <a id="i5"></a>I5 Worker reuse and failure behavior | I3/I4; F1 | Reuse immutable admission, preserve accelerators and expose worker failures | planned |

I2/I4 are direct target corrections and need not wait for the larger cache redesign. H removes
dead front doors while this plan replaces the actual production reuse path.

### I1 — Framing and small identity utilities

**Implementation vision.** The identity roles form an explicit input map. Source-revision identity includes the exact source
inventory; admitted-closure identity includes selected documents, imports, physical definitions
and provider/capability context; semantic-body identity includes normalized mathematics and its
complete dependencies. Prepared-view identity adds structural roles, coordinate ordering and
compiler settings. Binding identity includes admitted member/value/context pairs. Profile identity
includes effective execution settings. Lineage-request identity names the scientific request;
operational-job identity names the submitted occurrence/idempotency scope. A span edit changes
source attribution, not necessarily body mathematics; a provider-contract change can change
admitted closure even if root text is unchanged. These identities are distinct typed products,
not aliases for one interchangeable ContentHash.

Move the reusable framed serializer from the backend into the identity owner; give float parts
their canonical operation. Introduce role-specific hashes where the review found interchangeable
ContentHash arguments and ambiguous request identities. Rename operational identity fields through
G3 when persisted; lineage request identity remains a different concept.

Use the adopted hex codec in the three existing utility roles while preserving lowercase
canonical-literal admission where required. Avoid replacing canonical framing with JSON, CBOR
or a generic serializer that lacks its name/order/float contract.

Focused controls include NaN payload equivalence, signed-zero distinction, infinities,
field/order-sensitive golden vectors, malformed/case-sensitive hex and compile-fail role swaps.
Delete raw float-bit framing conventions, JSON-text hash frames and replaced hand-written codecs.

### I2 — Unique live allocation ownership

**Implementation vision.** The prepared product is factored into shared mathematical structure/artifacts, revision-bound
attribution and bound-value products. Each part retains the owner/lease for its actual allocated
payload. Value rebinding replaces only bound-value products; a source revision replaces attribution
while optionally retaining unchanged mathematics. A cache entry and an escaped result both hold
owners, so eviction removes retention but cannot free live storage or release its reservation.
The accounting operation distinguishes newly allocated bytes from shared-owner references.
An aggregate size may guide cache retention, but is not repeatedly charged as if the whole graph
were newly allocated.

Split immutable programs and provenance from independently owned value state. Rebind operations
must not deep-clone occurrence maps while reserving zero bytes. Charge actual newly owned maps,
vectors and derived values, or share them; do not repeatedly charge the full shared prepared graph.

Focused controls retain multiple rebindings, drop the original, evict the cache and rotate the
workspace while checking live ownership and final release. Exercise both unchanged and rebuilt
bindings with nonempty provenance. The completely unchanged path may return the existing owner.
Delete unaccounted owned copies and overlapping aggregate charges that pretend to represent
unique live storage. Deliberately conservative stack/opaque-library reservations retain their
separate purpose; this does not remove every conservative allowance. No RSS or exhaustion claim
substitutes for these ownership controls.

### I3 — Fine-grained reusable mathematics

**Implementation vision.** The cacheable package product is immutable package admission, not the mutable workflow package
or an attempt. Body queries consume the semantic-body key and admitted dependency closure;
service views consume the compatible structural/profile key and own immutable compiled products.
At use time a revision supplies its source map and a binding supplies values. Mutable evaluators,
foreign workspaces and cancellation are instantiated for the attempt outside the cache.
A warm resident entry can therefore serve two revisions with equivalent mathematics but different
locations. Eviction or refused retention may cause lawful recomputation without changing meaning.
Durable workers resolve this same admission/view path instead of constructing a fresh database
for every identical source closure.

Replace call-local body deduplication with bounded tracked admission on complete semantic inputs.
Separate the revision/source association from body mathematics. Distinct A/B binding sets get
distinct immutable keys so A/B/A reuse is possible without overwriting one tracked input.
Remove the fixed root-count refusal where byte/lifetime admission now provides the bound.

Move the package-local hand-written view LRU into service-scoped retention after owners are
correct. Publish fresh source maps on each bound revision; retain original attribution in old
results. A root file hash is insufficient for imported data, physical context or provider changes.

Focused controls use admission/preparation counters rather than timing: changed bodies and their
dependents invalidate; unrelated bodies reuse; a span-only edit refreshes diagnostics without
arithmetic rebuild; relevant provider/data/context changes invalidate. Include more than the old
root cap under a sufficient byte budget, eviction and denied allocation. Delete replaced singleton
binding inputs, package LRU, call-local memoization and obsolete root-count machinery. Performance
measurements wait for K4.

### I4 — Native work owns admission

**Implementation vision.** Ownership moves in one direction: the caller acquires a step guard; successful dispatch moves it
into the request; the native work/required step cleanup retains it; completion drops it. Before
successful dispatch the caller still owns the guard, so failed send returns capacity normally.
Dropping the waiter signals cancellation but owns no dispatched permit to release. Session job,
stack, retained foreign state and TLS leases remain with session teardown/join; step-only foreign
allowances end with the step. This separation preserves idle intervals without returning capacity
while a native operation is still running.

Transfer each per-step permit in the dispatched request/work owner. Failed dispatch drops the
undispatched guard; successful dispatch retains it through native work and required destructors.
Explicit token cancellation and future abandonment must converge on truthful completion ownership.
Preserve permits' release between steps so intervening preparation can proceed.

Use a deterministic gated worker for caller abandonment, token cancellation, failed send/scope
entry and session reuse. The test must establish that another session cannot acquire capacity
before the first step completes. Delete caller-future ownership of dispatched permits.
Consolidate repeated supervisor mechanics only where they have these same invariants; do not
hide distinct lifecycle responsibilities behind a universal helper.

### I5 — Durable workers, accelerators and profile failures

**Implementation vision.** A durable worker resolves the complete admission key, acquires shared immutable admission/programs,
then creates fresh attempt workers with that attempt's cancellation, start, budget and result
ownership. Replacing declarations carries forward explicitly selected provider/accelerator
configuration and recomputes the dependencies affected by the new source. A profile chain returns
its own typed outcome: completed, scientific refusal, worker panic or scheduling failure, with
attribution and actual parallelism. Only work that never started can be rescheduled after spawn
failure; a panicked chain is not silently repeated as though its first attempt never happened.

Reuse admitted packages by complete closure identity and prepared products through I3. Attempt
workers and cancellation remain separate. When declarations change, retain the selected
accelerator/provider inventory and refresh the affected admission/provenance.

Keep scoped native threads for profile chains. A worker panic becomes a typed chain failure and
is never silently replayed. Spawn failure records reduced parallelism and runs only unstarted
work on the remaining admitted worker capacity; if no worker can run, report an attributable
failure. F1 defines the failure envelope; do not misclassify a panic as scientific trial rejection.

Focused controls exercise shared immutable inputs with isolated cancellation, provider inventory
preservation/invalidation, visible panic, spawn failure and no duplicate chain execution.
Delete unconditional standard-accelerator resets, fresh-per-job preparation where replaced,
swallowed joins and silent replay paths.

## Authority and handoff

Hash/frame changes use an ADR plus design review and blueprint §5.3. Correct §5.1's member
identity owner with the current specialization contract. Update §14.3/§14.4 and §18.8 to describe
production reuse and completion ownership; a new enduring prepared-product contract receives
the appropriate short ADR. No compatibility cache/hash path remains.

H supplies complete admitted context and occurrence attribution. A supplies physical meaning,
D/E semantic capabilities, F occurrence/binding policy and G durable identity transitions.
J consumes typed identities in generated boundaries without exposing Salsa/cache internals.

## Consumed 25c prerequisite slice

**Implemented/Tested, 2026-10-01; scoped focused verification recorded in [25c Verification](25c-process-composition-and-conservation.md#verification):** New state slots frame owner, role and canonical semantic index values under the identity owner's ModelingProcessSlotV1 frame, without changing historical member preimages; corrected numerical coordinate projection uses NumericalProjectionV2 and admitted difference-policy projection uses NumericalDifferenceProjectionV1, with historical frame spellings unchanged; connection overrides retain original occurrence identity while retaining current source attribution. No binder-position species mapping is introduced. The full role-typed framing, historical frame migration, cache/allocation and native-completion work remains open. The maintainer authorized only this required slice and its complete affected consumer migration. This packet remains partial; [25c](25c-process-composition-and-conservation.md) owns the slice evidence.

## Consumed 25d prerequisite slice

**Implemented/Tested, 2026-10-01:** I1's required mathematical-realization/response
slice is integrated. MathFactorableV2, ImplicitConfigurationV2, ModelingImplicitResidualV5 and ModelingImplicitOperationV1 frame canonical exact rationals, admitted selection and numerical versus semantic dependencies, retaining historical frame spellings. Rational/export and response allocations retain declared capacity/owners. Broader role-typed identity migration, reuse, cache ownership and native completion remain open.
[25d Verification](25d-mathematical-realization-and-response.md#verification) owns commands,
conditions, composite results and limits; this does not close the enclosing packets.

## Execution and evidence

The consumed 25c/25d prerequisite slices above are **Implemented**; their focused evidence
is owned by the linked plans. The remaining packet scope and expected benefits are **Proposed**. No full-packet
completion or new broad product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Outcome (recorded after implementation)

### What was built

Full-plan closure remains outstanding. The implemented 25c/25d prerequisite slices and
their remaining boundaries are recorded above; the linked plans own their focused evidence.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
