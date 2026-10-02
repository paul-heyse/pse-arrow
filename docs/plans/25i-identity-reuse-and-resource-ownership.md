---
title: "25i: Identity, reuse and resource ownership"
status: done
date: 2026-09-30
adrs: [ADR-0150]
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
| <a id="i1"></a>I1 Canonical framing and roles | Existing identity contract; hashing decision route | Centralize framing, role types and small hex utilities | complete: focused controls passed |
| <a id="i2"></a>I2 Immutable/binding allocation ownership | Existing prepared-product owner | Share immutable/provenance payload; charge new allocations; retain escaped owners | complete: focused controls passed |
| <a id="i3"></a>I3 Complete bounded reuse | I1/I2; H1/H2/H4; A3 | Body-level queries, immutable bindings and service-scoped view/package retention | complete: focused controls passed |
| <a id="i4"></a>I4 Completion-owned native admission | Existing staged/session owner | Transfer guards with work and release at actual completion | complete: 25e |
| <a id="i5"></a>I5 Worker reuse and failure behavior | I3/I4; F1 | Reuse immutable admission, preserve accelerators and expose worker failures | complete: focused controls passed |

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

## Consumed 25e prerequisite slice

**Implemented/Tested, 2026-10-01:** I4 is complete: a successfully dispatched step owns its CPU permit through required native/TLS destruction, cancellation, waiter abandonment and panic cleanup, and returns capacity between steps. Failed dispatch leaves the guard with the caller. Required numerical/declared-admission/dynamic identities are versioned; historical frame spellings remain intact. Broader I1/I2/I3/I5 identity/cache/reuse work remains open.
[25e Verification](25e-declared-analyses-and-qualification.md#verification) owns commands,
conditions, composite results and limits; this does not close the companion plan.

## Consumed 25f prerequisite slice

**Implemented/Tested, 2026-10-01:** The identity owner frames admitted binding content with
StudyBindingV1: canonical member coordinates/values and physical context are distinct from
occurrence identity. Equal binding content can name separate experiments. Immutable operation
admission is shared by the study executors, and focused counted-reuse controls establish the
tested preparation behavior. This is not a cache, allocation or performance measurement.
[25f Verification](25f-studies-diagnostics-and-continuation.md#verification) owns the controls,
conditions and limits. Wider I1 framing/role migration, I2/I3 allocation and bounded reuse,
and I5 worker/accelerator ownership remain open; I4 retains its completed 25e scope.

## Consumed 25g prerequisite slice

**Implemented/Tested, 2026-10-01:** The identity owner supplies ArtifactDescriptorV3,
ArtifactMigrationV1 and ResetManifestV1 frames. Descriptor-v2 preimages are unchanged;
opening requires independently established roots and recomputed profile proof. Explicit
migration dependencies include the transformation digest and exact source publication, so a
changed finite policy cannot recover stale outputs from the same attempt. Source leases remain
owned through native execution and commit. [25g Verification](25g-durable-contract-evolution.md#verification)
owns focused evidence; wider I1 identity-role migration and I2/I3/I5 reuse/resource scope remains open.

## Execution and evidence

The consumed prerequisite slices retain their original evidence above. Remaining I1/I2/I3/I5
production scope is **Implemented, 2026-10-02**, under [ADR-0150](../adr/0150-checked-admission-and-owned-reuse.md).
I1/I2/I3/I5 are **Tested** by the focused controls below and complete; I4 retains its consumed
25e completion. The
[series coordinator](25-design-remediation.md) owns finding dispositions and the architecture.
Integration, formatting, lint and performance qualification remain in [25k](25k-integrated-qualification-and-closure.md).

The production route remains `ModelingPackage` → `prepare_modeling_revision`: complete immutable
Salsa body inputs stay local, service caches retain immutable owned admission/views, and each
revision supplies current attribution. The raw `Inputs`/top-level body route and package-local
fixed 16-entry view LRU are removed. Owner-attached shared fields preserve allocation leases
through binding, provenance, structural, physical and erased aliases; attachment cannot release
an existing owner. Native mutable workers and cancellation remain attempt-local.

The tracked body computation is mathematically pure; its selected service retention attachment
performs narrowly bounded infrastructure cache/accounting effects. Attachment is frozen before
revision publication. Cache hits prove the complete body/physical closure, and the callback may
only preserve admitted meaning while attaching lifetime ownership. Allocation refusal cancels the
query without memoizing a refusal; an unchanged-input retry remains possible. Mutable cache
availability and owners never become semantic inputs or service-global Salsa handles.

### Frame and consumer classification

**Implemented:** canonical floats preserve signed zero and infinities and collapse NaN payloads.
These unrestricted input frames change version; historical spellings and recorded digests remain
immutable. Exact source revisions, admitted closures, bodies, prepared views, bindings, profiles,
lineage requests and operational jobs have distinct types. The package admission closure key is
owner-local and includes exact retained validation factory/registry owners; it is not a persisted
scientific identity.

| Owner/consumer | Current frame | Classification |
|---|---|---|
| Backend settings, resolved accuracy and controls | BackendSettingsV5, NativeAccuracyV4, NativeControlsV3 | Unrestricted typed Serde inputs; canonical float framing |
| Warm-start payload | NativeSeedV3 | Numeric payload, layout/profile/data/backend and auxiliary state; excludes output origin |
| Solver, native sessions, fitting and simulation profiles | SolverProfileV4, SolverSessionV2, SolverConicSessionV2, FitProfileV4, DynamicProfileV8 | Canonical typed effective settings; JSON-text hash paths removed |
| Cone layout and explicit conic request | ConeLayoutV4, ExplicitConicV5 | Public key accepts inputs before finite-domain admission; new versions required |
| Compiler body and selected view | ModelingConsumerBodyV3, CompilerModelingViewV3 | Complete normalized scientific dependency closure; source attribution separate |
| Exact source occurrence and local package admission | ModelingSourceOccurrenceV1, ModelingPackageAdmissionV1 | Revision attribution and validation-context-scoped immutable admission |
| Scientific request and durable submission | SolveRequestV2, DurableJobRequestV3 | LineageRequestHash and OperationalJobHash are separate; G3 owns persisted frame proof |

### Verification

**Interface-checked, 2026-10-02:** `just check-library pse-compiler` passed after the shared-field
migration; its dependency emitted two modeling warnings subsequently repaired by H.
`just check-library pse-runtime` passed after repairing artifact-vector type inference and the
profile dispatcher callback's `Sync` bound. Its one generated-boundary module visibility warning
was subsequently repaired by J. The failure baseline is zero; these are composite focused
compilation results, not an initially clean run or full qualification. Earlier attempts encountered
in-progress prerequisite/generated-contract, lock and adjacent-owner changes before reaching
these owners; their repairs remain with the owning packets.

**Tested, 2026-10-02; zero failure baseline:** focused test-mode controls use the pinned
workspace feature graph and explicit `pse-relations/force-validate`. Symbolica-dependent reruns
activate the checkout with `direnv exec .` and serialize tests with `--test-threads 1`; backend
and runtime selections also use `bash scripts/memory-cap.sh`. The controls exercise production
admission/preparation and bounded dispatch, without running a native scientific solve.

| Command / selection | Result and conditions |
|---|---|
| `just unit-package pse-ids 'test(document) or test(roles)'` | 7/7 passed; typed role/frame proof, canonical NaNs, signed zero, infinities, field/order behavior and frozen vectors |
| `just unit-package pse-ids 'test(id::tests)' --test-threads 1` | 10/10 passed; hex round trips, uppercase adapter normalization, malformed length/digit and source-role utility controls |
| `just unit-package pse-schema 'test(binary_literal_requires_canonical_lowercase_pairs)' --test-threads 1` | 1/1 passed; lowercase paired-byte native literal admission, malformed and noncanonical refusal |
| `cargo test --doc -p pse-ids -p pse-relations --locked --features pse-relations/force-validate RoleBoundary` | 1/1 compile-fail control passed; no scoped doctest recipe fits this role-boundary selection |
| `just unit-package pse-columnar 'test(partition_transfers)'` | 1/1 passed; disjoint lease transfer and final release |
| `just unit-package pse-backend-native 'test(seed_content_identity) or test(identity_covers_every_settings_field)' --test-threads 1` | Composite pass, 2/2; actual warm payload canonicalization, output-origin exclusion, typed special-float snapshot and effective settings |
| `just unit-package pse-runtime 'test(workflow::modeling::reuse_tests) or test(fitting::profile::worker_tests) or test(pushdown_columns_are_the_registry_columns)' --test-threads 1` | Composite pass, 9/9; five admitted-package reuse/allocation controls, three production dispatch controls and repaired runtime-bound SQL validation fixture |

The frozen float vectors were independently derived from explicit length/tag/value preimages
using pinned BLAKE3 1.8.7 portable C, then checked against the canonical Rust serializer. Runtime
controls retain math, binding, provenance, derived, structural, physical, erased and nested-body
aliases across cache eviction and original-product release; final alias release restores the pool
baseline. They use a 512 MiB accounted pool and an explicitly held reservation to force allocation
refusal, then retry the same admitted package. A fresh exact validation-context owner invalidates
package admission; declaration replacement preserves the chosen accelerator owner. Dispatch
controls exercise one worker, a pool, partial starts and zero starts, with a started-chain panic,
exact per-task counts, actual started-worker counts and retained scheduling-string capacity.

The zero target remains in force. Initial backend unit compilation reported two fixture errors
and eight unnecessary-path-qualification warnings; repairs passed the exact selection without source warnings.
Runtime rebind controls initially failed two fixture lookups: a constant-folded parameter and a
fixture-path table were inappropriate for an externally bound runtime symbol. The final fixture
uses an unbound parameter and the authoritative symbol lineage inventory; all nine controls pass.
The final runtime Cargo build emitted one upstream `proc-macro-error2` future-incompatibility
notice, so this is not a warning-free comprehensive gate.

**Tested, composite compiler receipt:** `direnv exec . bash scripts/memory-cap.sh just
unit-package pse-compiler 'test(reuse_tests) or test(modeling::conditional) or
test(checked_compound_guards_preserve_original_arithmetic_occurrences) or
test(workspace::modeling::flow::boundary_unit::) or test(kernel_flow_projection)'
--test-threads 1` passed 13/13, with 218 skipped and no source warnings/errors. Nine controls
belong to this packet and four to J. The retained thermodynamic conditional fixture takes the
actual checked package route. Earlier licensed runs exposed H's response/coordinate-callee
closure omissions; those were corrected, then the complete named selection passed. An earlier
unlicensed execution aborted seven tests through restricted Symbolica thread admission.

These controls count executed body queries, show unchanged arithmetic and fresh source
attribution after a span-only revision, changed versus unrelated body invalidation, immutable
A/B/A requests, eight instances above the former root cap, transient retention refusal retry,
frozen attachment and rejection of a foreign cached body's sealed closure witness. Direct
synthetic path/number lowering preserves original arithmetic guard occurrences.

**Tested, final ownership/dispatch refinement:** `direnv exec . bash scripts/memory-cap.sh just
unit-package pse-runtime 'test(escaped_nested_body_retains_its_admission_owner_after_eviction)
or test(shared_body_cache_invalidates_the_complete_physical_closure) or
test(fitting::profile::worker_tests)' --test-threads 1` passed 6/6, with 227 skipped and no source
warnings/errors. Repeated attachment shares the exact already-owned implicit descriptor allocation;
a private retained allocation witness prevents a replaced public admitted field from claiming
that ownership. Its escaped nested math alias retains the original lease after eviction and
original-product release. A changed physical inventory produces different body/physical witnesses
and no old math alias through the same service cache. The fourth dispatch control classifies a
joined execution-environment panic directly as `ProfileWorkerFailure::Panic`, with two started
workers, zero chain attempts and no replay. An initial attempt was blocked by H's in-progress
Path/String callee join, corrected before this passing selection.

**Tested, dependency invalidation and retained consumers:** `direnv exec . bash
scripts/memory-cap.sh just unit-package pse-compiler
'test(kernel_immutable_function_data_is_visible_differentiable_and_invalidated) or test(kernel_physical_prerequisites_survive_admission_and_context_republication) or test(kernel_admitted_revisions_reuse_checked_state_and_refuse_context_substitution) or test(conditional_unit_recycle_reference_specializes_actual_unit_boundaries) or test(kernel_flow_projection_preserves_ports_isolates_and_explicit_tear_policies)'
--test-threads 1` passed 5/5, with 226 skipped and no source warnings/errors. Changed imported
immutable function data changes the observed value/derivative from 22/11 to 26/13 and matches a
clean admission. Changed applicable physical prerequisites refuse context substitution instead
of reusing a stale admitted revision. The retained seed and flow consumers pass after H's typed
segment resolver correction. After H's final dependency/builtin typed-segment consumer edits,
`direnv exec . bash scripts/memory-cap.sh just unit-package pse-compiler
'test(conditional_unit_recycle_reference_specializes_actual_unit_boundaries) or test(kernel_flow_projection_preserves_ports_isolates_and_explicit_tear_policies)'
--test-threads 1` passed 2/2, with 229 skipped and no source warnings/errors. These are focused
compiled-selection receipts, not integrated product qualification.

**Tested, G3 history scope:** `direnv exec . just unit-package pse-operations
'test(operational_identity_v7) | test(migration_supported_source_preserves_payload_and_catalog) | test(migration_plan25f_frozen_source_and_appended_inventory_target)'`
passed 5/5 against PostgreSQL 18.6 isolated schemas. Fresh V6 and full V1–V6 upgraded histories
transition to V7 while preserving recorded digests/frame provenance; unknown frames do not acquire
current-role proof. The upgraded fixture also checks exact six-NN artifact normalization. G3 owns
this operational-store receipt and the persisted transition.

No new timing, RSS, scaling, scientific solver, Python journey or exhaustion result is claimed.
Integrated qualification remains in 25k.

**Interface-checked, 2026-10-02:** Final cross-plan regeneration and workspace/all-target
compilation passed after the fixture and benchmark consumer cutover. The composite integration
receipt and linked Python boundary checks are owned by
[25j Verification](25j-generated-boundaries-and-library-consolidation.md#verification).
This establishes source integration, not the unexecuted 25k qualification campaign.

## Outcome (recorded after implementation)

### What was built

**Implemented:** typed canonical framing and hash roles, shared allocation ownership, complete
local Salsa admission, service-scoped DataFusion retention, immutable durable package admission,
accelerator preservation and typed profile dispatch outcomes. **Tested:** I1/I2/I3/I5 are
complete under the focused controls above; I4 retains its completed 25e basis. The prerequisite
slices retain their original conditions and evidence. Integrated/static/native/Python journey
qualification remains in 25k; no scaling, RSS or exhaustion measurement was transferred here.

### A mistake made and corrected

The first shared-field migration kept public raw Arcs for binding/provenance and replaced
accounting anchors when attaching an owner. Escaped aliases could therefore retain allocations
after releasing their leases. Owner-attached opaque shared values now preserve prior anchors,
and erased payload aliases retain the same ownership. The same correction applies to plan/body/
program attachment. Production-route controls now pass while retaining these aliases across
cache clearing and workspace release, then restoring the pool baseline on final release. Nested
body wrappers inherit admission ownership once, with a retained exact-allocation witness for
later attachments. Shared semantic-body descriptor owners also prevent duplicate revision charges.

### Deviations from the plan, deliberate

The local admission cache uses exact retained validation-context owners rather than a native
implementation generation alone: policy changes do not necessarily mint that generation.
Local tracked body queries permit narrowly bounded retention/accounting effects while keeping
complete mathematical inputs immutable; the attachment is frozen and transient refusals cancel
rather than poison memoization. Partial profile worker-start failures remain explicit alongside
reduced parallelism even when surviving workers complete every chain. Joined environment panics
retain their typed panic outcome, without interpreting a diagnostic string or replaying a chain.
These choices follow ADR-0150; no compatibility hash/cache
path, global Salsa handle or new generic scheduler was introduced.
