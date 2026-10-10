---
title: Unified state management and persistence
status: done
date: 2026-10-10
adrs: []
review_sources: [../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md]
scenario_sources: [../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#4-representative-journeys-and-change-locality]
---

# Plan 34: Unified state management and persistence

## Purpose, ownership and authorization

This independent plan adopts the nine findings and nine investigation avenues in the
[unified state-management review](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md).
It owns the proposed target, implementation and investigation packets, finding dispositions
and affected qualification. It covers active memory, optional retention, source acquisition,
deployment composition, durable cancellation, host lifecycle and the acceptance mechanisms
that distinguish their behaviors. SurrealDB remains the canonical durable substrate; this
is not a project to move every state representation into the database.

The maintainer authorized creation, detailed execution preparation and production execution
of this plan on 2026-10-10. Execution is complete; the acceptance evidence and Outcome below,
rather than document adoption or authorization, establish this plan's scoped completion.
[Plan 28](28-surrealdb-unified-substrate.md#current-checkpoint) retains its existing scope,
and [28e](28e-rebuild-retirement-and-qualification.md#checkpoint-and-next-step) owns the full
E3/E4/E5 campaign. Neither this plan's backlog nor its completion is folded into Plan 28.
Completed Plan 30 and Plan 33 Outcomes keep their original evidence and exclusions.

The source baseline is `main` at `a7a0d55e28aa5d94801c65d45dcb8b8c8d49b7a0`, with the
uncommitted review and its index entry. Reconcile current source and concurrent ownership
before execution. The retained campaign discussed by the review is incomplete historical
evidence, not a final failure count or qualification of this baseline. Later campaign status
belongs at 28e, not in this document.

Core 3.4, Efficiency Heuristics 1.0 and Process Simulator 1.5 apply through the
[selected standard](../design_review/design_principles/standard.toml) and
[binding](../design_review/design_principles/binding/pse-arrow.md). The review's F01–F09 and
S01–S11 are the source identities; references below do not create another scenario registry.

## Design direction and decision boundary

Build the target directly and delete the replaced mechanism, callers, tests and fixtures
once the replacement's targeted controls pass and callers have moved. Preserve authored
models, physical definitions, external reference inputs and actual live obligations.
Immature generated runs and derived state can be recreated when their contracts change;
they do not justify legacy readers, compatibility APIs or a second production path.
Coordinate any necessary cutover with actual users and store owners. This plan does not
authorize blanket resets, unrelated cleanup or deletion of concurrent work.

Retired analyses ultimately disappear entirely after protection and authentic drain permit
deletion. Permanent tombstones, retained obsolete lineage and historical internal scientific
goldens are not targets. Current analytical expectations, original scientific assessment,
conservation and applicable pinned external IDAES/Pyomo comparisons remain valid acceptance
bases. A new first-principles thermodynamic validation framework remains outside this scope.

Adopt a correct applicable improvement expected to improve execution fit unless it does not
work or substantially impairs performance. No minimum measured speedup is required. Small,
inconclusive, noisy or slightly adverse timing is insufficient for rejection, particularly
under concurrent repository workloads. Quantitative benefit claims still require comparable
measurements of the same complete operation. Hashing a key is not complete preparation.

### Rule changes

The source review selects **no rule impacts**. This plan selects none and introduces no
SHOULD exception. The target enforces existing ownership, exactness, resource and lifecycle
contracts; it does not change D1–D14, durable hashing, scientific usability or host support.
`adrs: []` means no new decision record is selected at authoring, not that later work is
exempt from the decision rules.

If implementation exposes a necessary change to public boundary meaning, persisted metadata,
a kernel/binding contract or another governed rule, record its actual impact and follow
[the decision route](../authoritative_design/sections/design-change-workflow.md#section-24-4)
before dependent changes. Present any new rule replacement to the maintainer for explicit
acceptance or rejection. A typed internal classification of an existing writer refusal is
not permission to weaken the refusal or silently change the external error/wire contract.
Independent packets can continue while such a genuinely consequential choice is unresolved.

## Assessed foundations and target responsibilities

The foundations are useful but their joins need correction. Immutable models and complete
selected dependencies provide scientific authority; mathematical products and native
workspaces provide execution. Canonical operations own durable visibility and exact attempt
settlement. Pool leases own actual allocations; cache membership owns only optional references.
Service observation supplies evidence of a current host incarnation, not scientific identity.
These responsibilities should remain distinct while composing into complete operations.

Relevant enduring owners are [mathematics and compilation](../authoritative_design/sections/mathematics-and-compilation.md)
(blueprint §7/§14), [numerical execution](../authoritative_design/sections/numerical-execution.md)
(§15–§18), [workflows and results](../authoritative_design/sections/workflows-and-results.md)
(§13/§19/§21), [identity and publication](../authoritative_design/sections/identity-and-publication.md)
(§5/§20), and [operations and validation](../authoritative_design/sections/operations-and-validation.md)
(§23/§24). These pages retain contract authority. The review and this plan describe diagnosed
divergences and the proposed correction, rather than silently amending those owners.

Keep semantic body identity, portable producer/artifact identity, active binding ownership,
attempt identity and host incarnation distinct. Role/version-framed durable identities retain
their current complete preimages; local fast hashes remain bucket hints backed by full
equality. Allocation addresses retain their actual local owners and are not portable keys.
Pure equivalence, producer qualification, exact receiving validity and fresh publication
acknowledgement are separate permissions. Cache presence confers none of the latter three;
memory eviction does not retire durable roots or withdraw a live attempt's authority.

Focused source assessment during authoring confirms three integration requirements. Workflow
composition must check actual consumed resource owners while preserving supported custom
factories. Pressure reclamation must coordinate retention publication during an admission
wait, otherwise another builder can immediately retain the capacity again. Cancellation
settlement must feed acknowledged stored completion into both durable records and study facts;
rewriting only a terminal state leaves an incomplete export looking scientifically usable.
These requirements are included below, with no new test or measurement claim.

The preferred physical design reuses existing component allocations, native cache operations,
grouped acquisition, completion owners and exact-attempt reconciliation. It avoids a universal
state manager, another allocator, a bespoke cache inventory/LRU, persisted active native handles
or database transactions surrounding numerical construction. A simpler reduced-retention
policy remains a valid control; replacing SurrealDB is not selected by these findings.

### One checked deployment at workflow assembly

SM01 makes `workflow::Runtime::from_shared` fallible with a typed construction error.
It verifies the factory's actual allocation pool, native runtime pool, cache-service extension
and pool, CPU-admission ownership and deployment width, and consumed spill/cache/store-registry
services against `SharedRuntime`. Equal capacities or a caller-supplied hash do not establish
affiliation. Compare consequential owner identity; exact equality of the outer `RuntimeEnv`
Arc can unnecessarily reject a legitimate wrapper that retains the same services.

Preserve custom planners, functions, rules, observation and query-scoped partition choices,
including `with_target_partitions`. Do not silently overwrite a foreign supplied factory.
Broader direct `EngineFactory` construction outside workflow composition remains useful.
Replace arbitrary live-operation injection with `DurabilitySelection`: ephemeral, or durable
with worker and lease policy. Construct durable operations from this runtime's canonical store
and shared pool; validate positive lease and heartbeat before installation. Keep `Durability`
as the readback of the installed owner. Use the existing typed workflow input error for
construction refusals. Migrate Python assembly, worker fixtures, workflow,
study and modeling tests and every other caller with the constructor change. Remove the
unchecked route rather than adding a compatibility constructor.

An aligned custom factory succeeds; a factory with a foreign pool or spill owner refuses
before workflow registration. Combined budget evidence becomes interpretable only after this
boundary is enforced. Pure component ownership tests need not wait for the public migration.

### Live components and disposable retention

SM02 replaces changed-binding ancestry in `math/products.rs::own_rebind`/`own_binding`.
The new product retains the stable structure's actual component owners and its current
binding's allocations, rather than the preceding complete `ProductOwner`. Shared allocations
must not attach the previous binding indirectly through `SharedAllocation::with_owner`.
Legitimate escaped aliases and explicitly retained report/history products keep their own
charges until their final owners disappear. The correction is selective parent ownership,
not deleting all parents or reporting freed memory while it is still reachable.

Latest-only repeated changed bindings, including `initialization::Blocks::attempt`, should
retain stable components plus the latest binding, while intentionally held older aliases
retain their distinct allocations. Identical scientific results and current attribution must
hold for fresh and rebound paths. This structural correction is required independently of
its measured latency; it also precedes conclusions about failed cache reclamation.

SM03 makes optional `MathService` retention yield during admission to the existing pool.
The two independently limited cache families can otherwise retain capacity needed to build a
miss. On `ResourcesExhausted`, an eligible requester registers a scoped pressure demand,
trims optional retention references and retries its existing reservation under the original
deadline. The demand lasts through asynchronous waiting until success, cancellation or expiry.
While demand exists, affected caches bypass new optional retention; existing hits, active
products, flights and returned aliases remain valid. This closes the refill race without
turning retention policy into a second capacity authority.

Use the pinned native cache's synchronous LRU limit trimming where it fits, with owner-local
temporary reduction and restoration of the configured retention ceiling. Retry actual pool
admission progressively; reachable cache bytes are not unique bytes released. Avoid
`list_entries` under pressure: it clones the entire inventory, retains values and allocates
precisely when capacity is scarce. Do not infer cache-only ownership from naive Arc counts.
Dropping a retention reference is safe even if a live alias prevents it releasing capacity.

Serialize trim/publication using the existing retention owners, without holding a lock across
pool allocation or awaits. Pressure is not semantic invalidation: it must not advance clear
generations, cancel flights or invoke global `NativeCacheService::invalidate` or
`clear_program_cache`. Explicit clear keeps its separate late-fill fence. Admission, active
CPU/TLS/native ownership and actual join remain unchanged; original clocks do not restart.

Migrate `MathService::reserve`, `reserve_entry` and job admission together. Portable scratch
already delegates to `reserve`; audit remaining direct reservations in source revision,
retained conversion, composition and constructor growth. Route eligible allocations through
the same bounded reclamation attempt or explain their already-admitted capacity. Synchronous
constructors acquire no new async wait, and hard workspace limits stay hard limits. A reduced
or disabled optional retention policy is a simpler operating control if native trimming cannot
fit the complete contract; no arbitrary new resource cap is selected.

### Protected, operation-shaped source acquisition

SM04 gives source protection a completion owner that survives outer-future abandonment and
submitted work until cleanup. `SourceRevision::open`, inventory and selected reads must
acquire, transfer and release the exact protection safely. Explicit successful finish awaits
release; final-owner drop supplies a cleanup backstop through the existing executor pattern.
Server expiry remains essential after process loss or failed acknowledgement.

Cover the gap between protection acquisition and owner construction, cloned followers,
cancellation during submission/read/decode and cleanup failure. One follower's departure
must not release another's live protection. A copied immutable payload owning its bytes is
distinct from authority to perform later protected reads. Reuse `ResultRead`'s final-owner
pattern where its full contract fits, without a universal lease framework or assuming outer
future cancellation proves remote work drained.

SM05 preserves selected and bulk intent in modeling inventory and conformance acquisition.
Selected operation resolves the required logical/kind selection and dependency closure; it
does not inventory an entire package merely to reach one fixture. Complete coverage still
enumerates all required declarations, then uses existing `selected_objects` grouping instead
of one `selected_object` call per member. Preserve absent-name and membership dependencies,
stable required ordering, non-test/error distinctions, precharged decoding, bounded blocks,
closing digest/manifest checks and physical validation. These are caller migrations into an
existing mechanism, not a new store/query abstraction.

The phase probe in SI03 separates retrieval, preparation, admission and actual native entry.
Avoid attributing a 20-second entry timeout to retrieval without that evidence. SM04's
completion-owned protection is the prerequisite for migrated acquisition paths that depend
on it; selected algorithm development can proceed independently.

### Cancellation settlement after scientific authority is revoked

SM06 retains immediate old-writer refusal on acknowledged study cancellation and actual
native join. It makes operational settlement reachable when that revocation interrupts
stream/table export. Factor existing closed-attempt reconciliation/sealing into the normal
termination and recovery owners, or adapt the existing recovery operation if that is smaller
and preserves actual identity and diagnostics. Do not add a second cancellation persistence
implementation or rerun science to repair storage.

There are three consequential outcomes. A complete export preserves the admitted scientific
completion and exact manifest, with truthful existing cancellation-race semantics. An
incomplete export with established cancellation freezes/reconciles the exact attempt and
acknowledges Cancelled lifecycle with scientific completion unavailable. Unrelated transport
failure or unacknowledged settlement remains an error. A cancelled header alone cannot
justify swallowing every persistence failure; rendered-message matching is not classification.

Before suppressing expected writer revocation, establish reliable typed refusal information
at the existing canonical writer boundary. Preserve its original cause and operation receipt.
If current typed errors are insufficient, add the narrow internal admission/refusal result
needed for this existing distinction; follow the decision boundary above if external meaning
would change. This slice is a prerequisite to consuming a refusal as expected cancellation.

`DurableAttempt::record` must consume the actual acknowledged stored completion, returned
from settlement or obtained through exact-record readback. `record_study_attempt` must derive
scientific facts from that completion's availability rather than provisional in-memory native
results. Ordinary and recovery projection share this distinction. An incomplete prefix may
remain observable but grants no successful scientific result, seed, activation or usability.
Using `Operations::recover` unchanged is insufficient if it substitutes worker-loss identity
or diagnostics for the actual normal attempt. Preserve already acknowledged terminals and
complete exports, immutable lost-acknowledgement recovery and exact attempt/point closure.

### Recovery meaning, readiness and trustworthy controls

SM07 replaces raw recovery dictionary equality with explicit consumed equivalence. Immutable
receiver bytes/content identity, exact profile and external interpreter qualification remain
equal. Installation association is validated under the exact permitted source-to-restored-root
mapping; emitted absolute paths are not immutable content identity. Current service invocation,
materialization, readiness and admission are re-established, not copied as proof of liveness.
Check all consequential receiver fields, manifest hashes, permissions and mapping. Do not
strip whichever keys fail. Controls accept deliberate relocation and reject changed bytes,
profile, interpreter or permissions, followed by an actual fresh restore/reopen/solve journey.

SM08 replaces Boolean host-readiness collapse at the decision boundary. Distinguish compatible
ready, owned startup in progress, actionable absence, observation unavailable and authenticated
mismatch. Unknown observations fail closed or receive bounded re-observation under the original
clock; they do not authorize restart. Compatible concurrent borrowers join the existing startup
owner's completion. Preserve PID start, listener inode, systemd invocation/cgroup, allocation,
profile and maintenance exclusion. No new service-wide reservation may be justified solely by
an unavailable observation; authentic mutation still requires the current lifecycle owner.

SM09 repairs test premises alongside the corrections that need them. A supervisor deadline
before provider entry need not produce a provider-expiry subtype. A fixture-entry watchdog
must report its actual pending phase and completed error, rather than hide failures behind
joining every unfinished task. Dropping callers does not drop cache-owned prepared bases.
A one-microsecond claim can correctly expire before commit; committed-then-expired recovery
needs an acknowledged claim and controlled expiry. Dispatch authorization, actual native
entry and actual join are different observations. Use deterministic controls or the existing
owned clock seam where appropriate; do not lengthen healthy timeouts, reduce the sixteen-case
workload, relax drain, add routine resets or weaken original scientific tolerances.

## Implementation packets and dependencies

The packet table owns progress. SM09 supplies early test slices to other packets rather than
blocking them behind a complete testing project. SM06, SM07 and the relevant SM09 slices are
early priorities because cancellation and recovery evidence are currently unreliable. SM01
precedes combined deployment-budget acceptance; SM02 precedes reclamation attribution; SM04
precedes dependent acquisition migration. Shared files require coordination, not invented
logical dependencies. The root integrates shared declarations, generators, consumers and
acceptance; executors receive bounded edit ownership and preserve concurrent changes.
SI08/SI09 choose their affected journeys before final integration; their assembled evidence
is delivered within SM11, rather than required before SM11 can start.

| Packet | Required input | Delivery and targeted acceptance | Progress |
|---|---|---|---|
| SM01 — Checked workflow composition | Existing `SharedRuntime`, factory and durability services; S05 | Implement the fallible composition boundary, including durability injection; migrate all Rust/Python/worker callers and delete unchecked assembly. Default/custom same-owner and partition-wrapper controls succeed; foreign pool/cache/CPU/spill/store owners refuse before registration. | Done |
| SM02 — Component and binding ownership | Existing allocation owners; SI01; S01 | Replace obsolete whole-binding ancestry throughout construction/rebind consumers. Latest-only repeated edits and initialization plateau at required live components; escaped old aliases retain honest charges. Fresh/rebound values and attribution agree; delete superseded ownership and its tests. | Done |
| SM03 — Pressure-aware retention | SM01 for composed budgets; SM02 before attribution conclusions; SI02; S02 | Integrate scoped retention-yield demand, native trimming and original reservation retry across both math caches and reservation callers. Test cache-only admission, concurrent refill, genuinely live saturation, escaped aliases, cancellation and original deadline; preserve explicit clear's late-fill fence. | Done |
| SM04 — Completion-owned source protection | Existing protected acquisition and retained executor; S04 | Transfer cleanup to the exact operation/final owner; migrate revision, inventory and selected consumers. Test acquisition/ack windows, dropped futures, surviving followers, submitted-work drain, explicit finish and expiry backstop; remove manual release-only lifetime paths. | Done |
| SM05 — Selected and grouped retrieval | SM04's protection slice for migrated paths; SI03; S03/S10 | Migrate modeling declaration/fit inventory and conformance selected/full modes into sufficient grouped acquisition. Test absence/membership, kind/error/order, complete coverage, digest checks and bounded decode with small/large/skewed fixtures; remove per-member retrieval and unconditional selected inventory. | Done |
| SM06 — Truthful cancellation settlement | Typed expected-refusal slice and acknowledged completion projection defined above; SM09 cancellation controls; S11/S07 | Integrate exact normal/recovery settlement, durable record and study facts. Test cancellation before entry, during each export phase, after join, repeated requests, lost acks, concurrent unrelated transport errors, complete-export races and existing terminals. Assert real drain, unavailable incomplete science and no science rerun; delete unreachable duplicate settlement logic. | Done |
| SM07 — Recovery equivalence | Current emitted receiver/restore contracts; SM09 recovery controls; S06 | Correct recovery oracle with content, mapped association, qualification and new-incarnation checks. Targeted pure controls distinguish legitimate relocation from changed bytes/profile/interpreter/permissions; select actual restore/reopen/solve for scope-end qualification. Remove raw equality and permissive ad hoc key omission. | Done |
| SM08 — Readiness and startup ownership | Existing authenticated lifecycle/admission owners; SI08; S08 | Implement typed readiness decisions and compatible startup joining. Test vanished descriptor versus unavailable observation, mismatch, concurrent registration/start, failed resume and interrupted owner; preserve real reservation and descendant-drain refusal. Delete Boolean-to-start fallback. | Done |
| SM09 — Phase- and owner-correct acceptance | Existing local test seams and each packet's consumed phase; S01/S02/S06/S07/S11 | Correct derivative, fixture entry, caller-drop and claim-expiry premises; add actual entered/joined and acknowledged-result distinctions where consumed. Deterministic positive and negative controls preserve original production clocks/workloads. Delete stale assertions and historical internal goldens; do not turn diagnostics into a new instrumentation platform. | Done |
| SM10 — Investigated integration corrections | SI04–SI08 implementation decisions and the core contracts above | Implement justified access-path, pacing, eligibility or maintenance-fence corrections at existing owners, with affected consumers and decisive targeted controls. A justified no-change conclusion is valid; unexamined query syntax is not a defect. No compulsory store/index rewrite, new global recovery service or unselected guarantee. | Done |
| SM11 — Integration, qualification and handoff | Functional packets complete; inquiry implementation decisions and corrected acceptance premises available | Select affected integration/solver/Python/recovery/maintenance journeys and scoped measurements, completing SI08/SI09 assembled evidence here; run scope-end checks, independent bounded review and repair. Reconcile dispositions and enduring documentation, then retire eligible material through its lifecycle. Do not acquire Plan 28's full campaign as a completion requirement. | Done |

Every functional packet supplies compile checks and targeted unit controls in the same change.
Only after proven replacement and consumer migration is the old mechanism deleted. If a
generator/registry declaration changes, fix the generator and regenerate; never hand-edit
generated queries, relations or Python contracts. Do not retain a second path as evidence.

## Bounded investigation packets

These rows cover every avenue in review §10. They investigate specific uncertainty, rather
than requiring a benchmark before correcting source-established amplification. Each concludes
with evidence, decision and integration owner: implement the warranted in-scope correction,
record a justified no-change decision, or route a separately authorized extension with an
observable trigger and owner. Deferral cannot excuse an unresolved core MUST obligation.

| Packet | Question, evidence and decision boundary | Integration and acceptance | Progress |
|---|---|---|---|
| SI01 — Rebind retention | Compare latest-only actual initialization/edit consumers with deliberately retained old aliases; inspect component reachability and pool charges, not RSS alone. The parent cause is already established; scale and exact retained owners remain to characterize. | SM02 owns correction and controls; reuse them as evidence rather than create another probe. Preserve scientific values and escaped aliases. | Done |
| SI02 — Reclaimable occupancy | Separate cache retention references, genuine live products, flight ownership, scratch and accounting under one verified pool/original clock. Include the four-GiB refusal fixture if reproducible; do not assume its sole cause. | SM03 selects native trim/bypass policy; SM02 precedes final attribution. Actual successful reservation establishes reclaimed capacity; failure under genuine live saturation remains truthful. | Done |
| SI03 — Pre-entry conformance work | Observe sufficient source acquisition, preparation, admission, actual entry and completion/error in the failing journey. A watchdog alone cannot identify dispatch failure or retrieval cost. | SM05 removes established retrieval amplification; SM09 corrects the oracle. New independent failure gets its actual owner without being relabelled as F03. | Done |
| SI04 — Native query access plans | Inspect exact SurrealDB 3.3 plans and examined/returned work for growing/skewed membership, kind/inverse-edge selection, absent products and `product_discovery`. Preserve negative/membership and conflict semantics. | Canonical query owner via SM10 selects justified indexes, grouping or compact views; migrate generated declarations through codegen. No store replacement or graph traversal cutover without complete operation evidence and its decision route. | Done |
| SI05 — Per-problem pacing | Examine original-clock queuing and concurrent staging/read operations with exact emitted guards and transaction behavior. Distinguish process-local pacing from distributed authority. | Canonical operation owner via SM10 narrows pacing only if phantom/conflict safety and complete guarded operation survive. A visible mutex alone does not justify partitioning the guards. | Done |
| SI06 — Candidate rejection and hydration | Establish candidate counts, dependency failure position, decoded bytes before eligibility and fresh/reused equivalence under relevant edits and skew. | Product discovery/reconstruction owner via SM10 moves sufficient cheap rejection before expensive hydration where correct. Preserve exact absence/membership checks, interpretations and complete replay admission. | Done |
| SI07 — Maintenance late-fill promise | Reconcile `NativeCacheService::invalidate`'s pending-load promise with fixed-generation `Namespaced` views and unfenced `Envelope::put`. Determine whether an actual external maintenance owner drains all loads. The inspected caller is test-only; no production stale-science claim is established. | Native cache/maintenance owner via SM10 either demonstrates the consumed fence or corrects admission/generation handling and all affected callers. Control old pending fill, new view and active values. This is distinct from ordinary SM03 pressure eviction and current math/physical clear fences. | Done |
| SI08 — Interrupted authority and retirement | Use corrected recovery meaning, exact operation receipts and actual invocation/descendant drain to exercise interrupted restart, restore and retirement with a fresh authority incarnation. Socket closure, copied PID or empty cgroup is insufficient. | SM08/SM10/SM11 share existing lifecycle and retirement owners. Reconcile roots/protection and fully delete retired analyses once safe. Power-loss, remote deployment and a new global recovery protocol are outside scope. | Done |
| SI09 — Scientific reuse meaning | Use current scoped fresh-versus-reused, provider and original-assessment journeys, with independently supplied physical conditions/tolerances and pinned external reference where applicable. | Mathematical/workflow owners via SM11 confirm affected reuse/completion remains truthful. Invalid historical internal results are not goldens; no new thermodynamic validation framework or full solver rewrite is required. | Done |

Research exact pinned library capabilities when implementing a selected mechanism. Use the
relevant capability skills and current documentation for API questions; plan prose is not a
verified library API contract. Inspect actual backend plans and emitted transactions where
they determine fit or correctness. Measurements retain mode, workload, resource ownership,
source and concurrent-load conditions; they cannot turn an inquiry into broad qualification.

## Finding dispositions

This is the single current disposition owner for the state-management review. All nine
findings are resolved by the scoped corrections and acceptance linked below.
Finding IDs are qualified as State F01–F09 to distinguish the unrelated Plan 28 families.

| Finding | Scenario | Disposition | Decision/work owner | Evidence or completion condition |
|---|---|---|---|---|
| [State F01](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f01) | S01 | Resolved | SM02 / math product and allocation owners; SI01 | **Tested:** mathematical ownership/pressure selection and actual `Blocks::attempt` control in [functional acceptance](#functional-and-assembled-controls); latest-only plateau, held aliases and fresh/rebound values pass. |
| [State F02](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f02) | S02 | Resolved | SM03 / math retention and admission owners; SI02 | **Tested:** pressure admission, concurrent refill, live saturation, cancellation/original clocks and clear fencing in [functional acceptance](#functional-and-assembled-controls). Historical four-GiB sole-cause attribution is not claimed. |
| [State F03](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f03) | S03/S10 | Resolved | SM05 / canonical modeling and conformance consumers; SI03 | **Tested:** 145-declaration selected inventory, grouped/paged operations and actual native conformance in [functional acceptance](#functional-and-assembled-controls); absence/membership/order and complete coverage pass. No historical timeout-cause claim. |
| [State F04](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f04) | S04 | Resolved | SM04 / source protection and completion owners | **Tested:** source abandonment/ack/follower controls and immediate final-owner drain, plus public close with an escaped canonical reader in [functional acceptance](#functional-and-assembled-controls). Explicit finish and process-loss expiry remain intact. |
| [State F05](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f05) | S05 | Resolved | SM01 / workflow assembly and all constructor consumers | **Tested:** three composition controls and migrated native/Python/worker consumers in [functional acceptance](#functional-and-assembled-controls); actual foreign owners refuse and supported same-owner custom factories remain usable. |
| [State F06](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f06) | S06 | Resolved | SM07 / canonical recovery oracle and receiver owner | **Tested:** actual reference SIGKILL/reopen/restore/rebuild/native solve and exact retained results, plus schema/permission/incarnation negative controls in [functional acceptance](#functional-and-assembled-controls). Legitimate relocation and directory hardening accept; qualification/content changes or permission widening refuse. |
| [State F07](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f07) | S01/S02/S07 | Resolved | SM09 / derivative, conformance, retention and claim controls | **Tested:** corrected derivative, actual initialization, sixteen-case conformance, caller/final-owner drain and acknowledged expiry controls in [functional acceptance](#functional-and-assembled-controls), preserving original clocks, workloads and science. |
| [State F08](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f08) | S08 | Resolved | SM08 / host readiness and lifecycle owner; SI08 | **Tested:** 250 tooling/recovery controls and actual frozen-supervisor process recovery in [functional acceptance](#functional-and-assembled-controls); unavailable observation, lifetime ambiguity and stale replacement snapshots retain refusal. |
| [State F09](../design_review/reviews/design_review_unified-state-management-and-persistence_2026-10-10.md#f09) | S11/S07 | Resolved | SM06 / durable attempt, study projection and canonical settlement | **Tested:** actual native export-phase matrix, normal/recovered study projections, committed-write lost-ack controls and cross-process kill/cancel/retry in [functional acceptance](#functional-and-assembled-controls). Incomplete science stays unavailable; complete assessment and unrelated-error precedence remain truthful. |

S09's domain-extension boundary is a preservation condition across these packets: new physical
concepts belong in semantic owners, and changing solver capability does not create a second
scientific policy in storage. S10 is also exercised by SI04–SI08. Inquiry closure does not
silently resolve a finding; update its disposition only with the required correction evidence.

## Existing-plan coordination and qualification handoff

The subsequent [SurrealDB capabilities, lifecycle and integrations review](../design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md)
has its own corrective owner in [Plan 35](35-surrealdb-lifecycle-and-integration-remediation.md).
The maintainer accepted that review's RC01/RC02 on 2026-10-10: narrow local read pacing
while retaining distributed safety, and move native deployment interpretation to composition.
Plan 35's [rule-change route](35-surrealdb-lifecycle-and-integration-remediation.md#confirmed-rule-changes)
governs those prospective changes. SI05's retained-pacing decision and mixed-operation
acceptance below describe the historical implementation; they are not a veto on the new
target or evidence that it is already implemented. This plan remains completed, with its
original findings, acceptance and Outcome intact. New review F01–F06 and Q01–Q08 are not
reopened SM/SI packets. The stopped campaign follows the current 28/28e checkpoints.

Plan 28's functional mechanisms and N4/T5/L4/E3/E4/E5 acceptance stay with their existing
owners. Plan 34 corrects prerequisites only where those invocations consume the affected
contracts. In particular, actual recovery consumes SM07 and its SM09 oracle slice; durable
cancellation consumes SM06 and its phase/result controls; interpreted preparation/conformance
failures consume the applicable SM09 slices. Allocation/reuse, selected acquisition and
concurrent startup journeys consume SM01–SM05/SM08 as affected by their actual paths.
These are evidence dependencies, not a requirement that every inquiry finish before any
unrelated Plan 28 work can progress.

Before relaunching an invalidated covering invocation, reconcile its exact affected prerequisites
and current source/profile/tool closures at 28e. Earlier unconditional restart instructions
are superseded by that dependency-aware handoff. Do not stop or mutate an existing service,
campaign or concurrent worker merely because this plan was authored. Current capture results
must be read from their owner, including interruption and terminal accounting.

One exact matching invocation may supply evidence to both plans, with command, mode, source,
store/resource context and actual outcome linked at each owner. A targeted Plan 34 pass does
not close Plan 28 E3; a historical setup pass does not establish present scientific cancellation.
Plan 34 requires its affected assembled journeys, not the entirety of Plan 28's full campaign.
Newly exposed unrelated failures are routed candidly to their owner, not hidden or silently
absorbed into this scope.

## Verification and completion

**Tested:** functional and assembled acceptance is recorded in [Acceptance evidence](#acceptance-evidence).
The following verification approach governed execution: during functional implementation, use `just check-package <pkg>`
and targeted `just unit-package <pkg> <filter>` through `scripts/pse-env`, with explicit
`pse-relations/force-validate` on every Rust test invocation. Use the corresponding focused
tooling/Python controls for non-Rust changes. Each packet's behavioral distinctions above
define its meaningful positive and negative cases; do not write tests that merely reproduce
the chosen implementation. Regeneration is part of any affected registry/generator change.

At functional scope end SM11 selects affected integration, component, native solver and
Python journeys for actual initialization/rebinding, pressure/reuse, protected selected/full
preparation, durable cancellation, concurrent lifecycle and restore/retirement. Retain original
physical/provider conditions, actual sixteen-case concurrency where relevant, native/TLS
join and current external reference scope. Include cross-process and real emitted-transaction
controls where pure mocks cannot establish authority or exact settlement. Keep local functional
acceptance, assembled scientific qualification and measured benefit distinct.

Measure complete affected operations only where benefit/capacity remains consequential:
latest-only retained state, warm cache miss admission, selected/full acquisition, candidate
hydration and native access plans if selected. Existing counters/pool ownership and scoped
timings should suffice; no mandatory runtime cost model, new observability platform or minimum
speedup threshold is introduced. A substantial regression requires diagnosis under comparable
conditions; noisy timing is reported with its limits.

After functional completion run required scope-end `just hygiene` and applicable governance,
documentation, native and powerset checks under the repository execution policy. Coordinate
shared invocations with 28e rather than launching a second full campaign. Fix failures in scope
and rerun invalidated covering checks; report every actual command, mode, scope, zero-failure
baseline and result. Independent bounded assembled review assesses this target and its consumer
integration, not an unexercised whole-product certification.

Completion requires all core packets delivered with displaced paths removed, all nine inquiries
decided with justified integration/disposition, and F01–F09 resolved by their required evidence
or explicitly routed through a genuine decision rather than marked resolved by scheduling.
Move enduring contracts and workflow meaning to their architecture/dev owners through the
proper route, with any required blueprint revision and ADR adoption. Update inbound current-work
links, preserve unrelated outcomes and retire eligible completed plans/reviews under ADR-0096.
The [Outcome](#outcome-recorded-after-implementation) records what was built, mistakes corrected,
deliberate deviations and the evidence/exclusions. The owning checkpoint distinguishes its
implemented changes from qualification that has not yet finished.

## Current checkpoint

All eleven implementation packets, nine bounded investigations and nine finding dispositions
are complete. Production corrections and caller migrations are integrated, and displaced paths
are removed. The affected math, canonical operations, native execution, managed-primary,
cross-process and public Python controls pass after the bounded repairs recorded below.
Original scientific conditions, workloads and clocks remain intact.

Actual reference SIGKILL/reopen, backup/fresh-authority restore, fresh solve and authored/physical
rebuild pass, including full schemas and exact retained value bits. Scope-end hygiene repairs,
governance, native lints and the complete declared feature powerset pass. Enduring contracts
and operational guidance have their existing owners; the current-work index routes this
completed plan through its Outcome. Independent review requires no further production repair.

Math, durable and lifecycle executor worktrees are integrated and retired with their owned
evidence preserved; unrelated worktrees and persistent state are retained. No authorized
functional scope remains in Plan 34. Conditional future investigation triggers remain with
their identified query/operation owners. Plan 28's full E3/E4/E5 campaign remains at its
existing owners and is not qualified or closed by this plan.

## Acceptance evidence

This section records SM11's integrated evidence for the completed authorized scope.
The failure baseline is **zero** for every command below. Rust correctness routes explicitly
enable `pse-relations/force-validate`; native routes use the pinned linked solver environment.
All runs use `scripts/pse-env`, the pinned toolchain and this checkout's environment. Source
is `a7a0d55e28aa` plus the integrated changes in this plan, dated 2026-10-10. Concurrent sibling
repository builds were active; wall-clock times do not establish isolated performance.

### Functional and assembled controls

**Tested — mathematical ownership and pressure:** the integrated math worktree's native-local
`scripts/pse-env --resource-class compile --native=solver,klu -- just unit-package pse-runtime
'test(/^math::/) & (test(latest_only_changed_rebinds) | test(changed_unused_coefficients) |
test(pressure_tests) | test(admission_) | test(constructor_) | test(retained_result_) |
test(event_export_) | test(strategy_export_windows) | test(cancellation_before_entry) |
test(scoped_preparation_deadline) | test(task_preparation_waiter) |
test(initialization_latest_block) | test(first_block_binding))'
--features solver-kinsol --execution-effects native-local`
covered 32 controls (32 passed, zero failed, 2.614 s; Nextest
`cb9e74e0-4399-4faa-86b4-365e20b267d2`). The controls use a 64-MiB shared pool and two
32-MiB optional caches, real reservation failures/successes, concurrent refill, cancellation,
original deadlines, latest-only binding changes and intentionally escaped aliases. They
establish reclaimed admission and exact live charges, not RSS savings or sole attribution
of the historical four-GiB refusal. The actual native `Blocks::attempt` control separately
performs 36 binding edits, 24 latest-only and 12 with held old coefficients, checks unchanged
fresh/rebound KINSOL values and attribution, stable preparation reuse, charge plateaus and
final release to zero.

**Tested — maintenance:** native-local
`just unit-package pse-engine 'test(cache_service::tests::)'` passed all nine controls
(zero failed, 0.010 s). Old views cannot publish after explicit maintenance; new views work,
and escaped active values remain valid. Ordinary retention pressure does not use this fence.

**Tested — protected canonical operations:** functional SurrealDB 3.3.0 at the task's
explicit current-v3 store, `just unit-package pse-operations <filter>
--features pse-operations/canonical-tests`, passed the following bounded selections:

- Protection/writer-refusal/study-cancellation/frame-validation/interpretation/mixed-reader/
  scoped-page/result-retirement/analysis-retirement selection: 12 passed, zero failed,
  31.151 s. This includes exact issuing-store cleanup, abandoned acquisition, follower
  election and sixteen same-problem readers with authentic guarded staging.
- Scoped/grouped/lexical/selected/multipage/result-retirement/held-analysis selection:
  seven passed, zero failed, 24.299 s; Nextest `9aad075b-9691-4874-89d9-04e5f9020df9`.
- `test(inverse_supplier_set) | test(exact_premises_unrelated_changes_and_native_inverse_selection)`:
  two passed, zero failed, 9.015 s; Nextest `3efe8133-ebc8-4485-baa2-6674e8f0624f`.
  The actual 130-member protected inverse inventory pages 64/64/2/0 and preserves revision,
  kind, absence and exact membership semantics.
- `test(later_candidate_page_and_expired_selection_refuse_false_reuse) |
  test(bounded_qualification_rejects_grown_wide_scope_and_merges_only_matching_delta)`:
  two passed, zero failed, 42.111 s; Nextest `cbb0b9ad-b2c1-4344-a46d-9c83b1b007b9`.
- `test(study_committed_claim_start_and_summary_lost_ack_settle_exact_identity) |
  test(study_start_lost_ack_cannot_redispatch_after_cancellation) |
  test(study_creation_issued_activation_lost_ack_settles_and_cancels_exact_identity) |
  test(canonical_execution_exact_batches_private_closure_and_settlement)`:
  four passed, zero failed, 8.413 s; Nextest `aca1023f-2f1d-432c-a2e3-d655c0215c43`.
  The first three inject lost acknowledgement after real committed settlement-boundary writes,
  then reconcile the exact claim/start/summary/creation identity without redispatching cancelled
  science. The fourth checks definitely acknowledged exact replay and immutable closure;
  it is not a lost-response injection.

**Tested — default workflow composition and acquisition:** functional
`just unit-package pse-runtime <composition/selected/durable filter>
--features pse-runtime/canonical-tests` ran 12 controls: 11 passed, one failed
(`01e55117-59a3-4aa8-90fa-6193f5197e69`, 61.528 s). The failure attempted Ipopt without
its linked feature; its test premise is corrected to require `solver-ipopt`, and the actual
linked run passes it. The three composition controls positively establish same-owner custom
factories/partition wrappers, actual foreign-owner refusals and durable interval/deployment
selection. The sparse selected inventory covers 145 declarations, required ordering,
absence/membership and cancellation. Generic document preparation and acknowledged claim
expiry controls also pass; an unlinked default selection is not native acceptance.

**Tested — native conformance and actual initialization:** exclusive/full-native
`just unit-package pse-runtime <conformance/actual-blocks filter>
--features pse-runtime/canonical-tests,pse-runtime/solver-kinsol,pse-runtime/solver-ipopt`
passed three controls (zero failed, 56.717 s; Nextest
`96d145c9-5731-4ba1-ae0f-5f0ab4ea9bd0`). These are
`conformance_parallel_cancellation_drains_admitted_workers`,
`conformance_parallel_sixteen_native_workers` and
`actual_block_attempts_plateau_and_escaped_coefficients_keep_their_charge`.
The conformance cases retain actual sixteen-case native entry, original watchdogs,
scientific tolerances and join/drain. They do not identify the historical pre-entry timeout's
exact cause. The corrected export matrix separately passed in 15.231 s
(`2911da5e-c4e6-47a6-8640-affe0e6f6ed7`).

**Tested — final durable export/settlement integration:** exclusive/full-native
`just native-test -p pse-runtime --lib --test worker -E <durable-export/portable/worker filter>
--no-fail-fast` ran ten controls (`60da5f8f-f8a5-4514-95a8-1a5b40e69fe2`, 119.223 s):
seven passed, three worker cases failed only at protected-reader fixture cleanup. The seven
passing controls include portable body reconstruction, failed export prefix, pre-entry
refusal, heartbeat-through-export/reconciliation, complete-export cancellation, cancelled
prefix/late-heartbeat reopening and the finish matrix. The matrix gates actual progress,
table, seed and completion publication; checks exact acknowledged prefixes and manifests;
and reads both normal and recovery study-point projections. Incomplete science remains
unavailable, complete science preserves the original assessment, and unrelated errors retain
their precedence. A definitely acknowledged seed replay is not a lost-response control.
The repaired exclusive/full-native rerun used
`just native-test -p pse-runtime --lib --test worker -E
'test(killed_worker_freezes_truthful_observations_and_retries_only_by_authored_policy) |
test(cross_process_cancel_stops_scip) |
test(worker_replays_same_authored_case_in_a_fresh_default_process) |
test(physical_admission_cache_unit)' --no-fail-fast`:
ten passed, zero failed, 128.500 s; Nextest `93e812c2-cd2a-434f-88eb-5d7e83f4fdfd`.
This covers all three worker journeys and all seven physical-cache controls, including
immediate final-owner drop/drain and refusal while either escaped physical or row owners live.
Independent bounded review found no material issue in the final cleanup delta.

**Tested — managed-primary studies:** original-reference/full-native, explicit current
worker, `python -m scripts.native_tests rust --managed-primary-route -p pse-runtime --lib
--ignore-default-filter -E 'test(managed_primary_)' --no-fail-fast`, passed all four controls
(zero failed, 154.204 s; Nextest `e768b0b8-28ee-4451-a370-8edabd9af20c`). They exercise actual
sixteen-owner entry, admitted cancellation and tail refusal, exact partial history and escaped
Arrow retirement, and reopened continuation with its original correction. Post-run authentic
context drain also completed. This is affected Plan 34 acceptance, not full Plan 28 E3/E4/E5.

**Tested — host lifecycle and recovery oracle:** light
`python -m unittest scripts.tests.test_surreal_server scripts.tests.test_host_admission
scripts.tests.test_pse_env scripts.tests.test_canonical_recovery_check` passed all 246 controls
(zero failed, 18.126 s). These include compatible startup joining, unknown/mismatched
observations, unchanged original clocks, actual unreaped zombies, replacement allocations,
stale borrowed snapshots, exact receiving-content relocation, interpreter/permission refusal
and intermediate-directory/manifest symlink refusal. Actual
`scripts/surreal_server.py qualify-recovery --state <task-current-v3>` passed on final frozen
supervisor generation `b928801fc8864762a3a9756bd59aa098ccea02058be21d86ce3b3e648188eb8b`:
acknowledged writes, abandoned response, SIGKILL and reopen. This is actual process recovery;
scientific backup/restore/rebuild remains separately required below.

**Tested — actual scientific recovery:** original-reference/full-native
`scripts/pse-env --resource-class reference --native=solver,klu,isolation,uno,petsc --
just canonical-recovery-test --scientific
--profile-state /home/paul/.local/state/pse-arrow/surreal-plan28-reference-20261010
--directory /home/paul/pse-arrow/build/sm34-scientific-recovery-qualified-20261010`
passed the complete journey, exit zero. It retains the original 128 GiB pool, sixteen CPU
lanes and reference host/server envelope. Public API seed, SIGKILL/reopen, offline backup,
fresh namespace/database/credentials, gated restore, fresh solve and authored/physical rebuild
all pass. The dimensional `x == 2 m` fixture preserves its original physical inputs and
acceptance; current acknowledged manifests and float/binary bits survive reopen/restore.
Stale credentials and creation authority refuse; fresh authority activates. Final owned
services stop through authentic drain. Artifacts, explicit inputs and current-run receipts
remain in the named directory; failed predecessor fixtures remain identified by their own
directories. This does not establish power-loss or remote recovery.

The final light `python -m unittest scripts.tests.test_surreal_server
scripts.tests.test_host_admission scripts.tests.test_pse_env
scripts.tests.test_canonical_recovery_check` passes **250 controls, zero failures, 12.291 s**.
The additional recovery controls distinguish metadata map order from full schema values,
preserve signed-zero/NaN payload and binary bits, allow permission narrowing while refusing
widening/lost access, and distinguish a fresh maintenance unit file from copied qualification
or reopened admission. The earlier 246-control result retains its original source scope.

**Interface-checked — independent final review:** the bounded read-only review of the late
lifecycle and acceptance-oracle corrections found no material contract violation or weakened
scientific acceptance. It checked actual lifecycle reservation ownership, permission narrowing,
fresh materialization versus qualification, full schemas and exact value bits, original
physical inputs/tolerances and visible cleanup failures. This review does not substitute for
the executed journeys or the separately completed feature matrix below.

**Tested — Python workflows and public close:** exclusive/full-native on the isolated task
store, `just native-python build/sm34-python-final-20261010
python/pse/tests/test_native_workflow.py python/pse/tests/test_native_caches.py
python/pse/tests/test_canonical_results.py -q` ran 31 controls: 30 passed and one new
fixture-premise failure, zero teardown errors, 285.53 s. Native workflow/fitting, cache,
canonical results and the nested cross-process loader journey passed. The initial negative
close control incorrectly assumed document loading populated physical retention; preparation
alone also failed that premise (one passed, one failed, 17.20 s). The corrected public
selection, `just native-python build/sm34-python-owner-final-20261010
python/pse/tests/test_canonical_results.py -k canonical_close -q`, passed both controls,
zero failures, 29.90 s. An actual durable solve populates retention. Public close succeeds
after caller departure without manual clear; an unread canonical result reader refuses close,
fetches its exact server-backed scalar rows after refusal, still refuses before EOF, and
permits retry after its departure. Ordinary fixture teardown also repeats close successfully.
This is combined affected coverage, not a claim that the original 31-case command passed.

### Scope-end checks

**Tested:** `just hygiene` ran its complete scope-end bundle at
`build/assessment/20261010T164716.122616Z-hygiene-3565427-7fea15/summary.md`.
Four leaves initially failed: Python lint (19 findings), Python type checking (13 diagnostics),
default Clippy and no-default Clippy (four nested-if findings plus a test-only unused helper).
All were repaired; each failing leaf was rerun and passed. Final Python type checking reports
zero diagnostics with its two configured suppressions; neither a warning allowance nor a
quality baseline was introduced. The remaining original hygiene leaves passed.

**Tested:** full-native consumer Clippy for `pse-runtime`, `pse-py`, `pse-benches` and `xtask`,
all targets, locked, with runtime canonical/native, Python native and xtask canonical features,
passed `-D warnings` after replacing the worker test's unreachable-lint violation. An initial
compiler-cache observation failure caused `just ready` to run; ready passed, and retry reached
and repaired the actual source finding. The environment incident's exact cause is not claimed.

**Tested:** `just governance` passed all declared gates, including generator equivalence,
family checks and actual installed-extension contract/stub checks:
`build/assessment/20261010T165500.843619Z-governance-3818134-6a0492/summary.md`.
`just docs` passed rendering/indexing (310 chapters, 51,124 words before final closeout edits).
The affected final source checks, manual native checks and complete feature powerset are
recorded below. Final `scripts/pse-env --docs -- just docs` passes after the closure edits,
publishing 310 chapters and indexing 51,385 words. The read-only lifecycle validation reports
zero errors, and this plan's native scope projection reports 29 complete items with zero open,
blocked, deferred or unknown items. Retirement preflight retains this owner for its six inbound
readers and unreleased source meaning, consistent with the retention explanation below.

**Tested — final cleanup checks:** `just lint-py` and `just typecheck` pass after the Python
fixture/collector/recovery cleanup repairs (zero lint findings; zero type diagnostics,
two configured suppressions). `just py-sync-native` rebuilds and installs the actual linked
extension, and `python-stubs` passes against its compiled API. The manual
`just lint-native-contracts` and `just lint-native-data` recipes pass with `-D warnings`
(23.44 s and 6.82 s). `just turn-end` passes after these source edits:
`build/assessment/20261010T171539.014233Z-turn-end-4110631-89691b/summary.md`.
The full-native consumer Clippy command above was rerun after the physical-owner regression
control and worker cleanup edits: zero failures, `-D warnings`, 33.06 s.
After the public-close implementation it passed again with `-D warnings`, 5.51 s.
Final recovery/Python edits pass `just typecheck` (zero diagnostics, two configured
suppressions) and `just lint-py`; one new invalid-receipt exception-type lint was corrected
and its failing leaf rerun passes. `just turn-end` passes:
`build/assessment/20261010T180408.974997Z-turn-end-520305-071313/summary.md`.

**Interrupted, superseded:** the first `just features-powerset` assessment was deliberately interrupted
before changing the public close implementation, at combination 170 of 355. It reported
`features-combinations: interrupted`, `features-no-default: not_run`, exit 143:
`build/assessment/20261010T172106.453084Z-features-powerset-4144222-f9c05e/summary.md`.
No check failure was established by that passing prefix, and it is not powerset qualification.
The final source is covered by the fresh completed assessment below.

**Interface-checked — final feature matrix:** `scripts/pse-env --resource-class compile --
just features-powerset` passes both declared leaves, zero failures against zero:
355 workspace feature combinations at depth two (1,869.2 s) and 33 workspace package checks
without default features (25.5 s). The pinned linked native environment, locked dependencies
and explicit force-validation adapter remain in use. The accumulated logs contain no compiler
errors or warnings. This is compilation coverage of the declared matrix, not behavioral
qualification of every possible feature combination:
`build/assessment/20261010T180426.916080Z-features-powerset-521586-bb7694/summary.md`.

### Measurements and inquiry decisions

**Measured:** [native access-plan evidence](../design_review/evidence/unified-state-management-2026-10-10/README.md)
records released SurrealDB 3.3.0 isolated fixtures and concurrent repository load. Explicit
scope equality with `(problem, scope, key)` reduces the 4,096-member rare-scope iterator from
4,096 examined rows to one; the same-index statement comparison is 7.256 ms versus 394 µs.
The [inverse supplier comparison](../design_review/evidence/unified-state-management-2026-10-10/native-inverse-semijoin.md)
preserves complete returned rows across 48 queries. One LET replaces repeated edge lookups:
the rare unscoped statement falls from 10.422 s and 2,449,448 inner index rows to 2.024 ms
supplier construction plus 9.338 ms membership selection. An absent-kind case increases
from approximately 17 to 31 ms; the technically applicable correction is adopted under the
maintainer's criteria. These are individual statement measurements, not complete preparation
speedups, isolated performance qualification or a high-degree memory characterization.

SI01/SI02 adopt component anchors and coordinated optional-retention yielding, supported by
actual initialization, exact pool charges, successful miss admission and retained-alias
controls. The historical four-GiB failure's sole cause remains unestablished. SI03 adopts
sufficient grouped retrieval and phase-correct conformance controls; passing the original
watchdog does not retroactively diagnose the original timeout.

SI04 adopts the scoped predicate/index and one native inverse supplier intermediate. The
managed server allocation owns that intermediate; client windows remain 64 rows. No unused
binary-request index is retained: the pinned planner still scans the problem/producer prefix
for `NativeLiteral::Bytes`. The canonical product-discovery owner should revisit that access
path when the pinned planner changes. The canonical inventory/query owner should revisit
high-degree supplier materialization if complete paged-operation measurements reveal
consequential server memory or latency cost; no arbitrary supplier cap or persisted graph
view is introduced.

SI05 retains short per-problem pacing because distributed reads and staging still conflict
through the same authentic retention guard. The sixteen-reader mixed-operation control
passes. Revisit at the canonical operation owner if guard topology changes or complete-operation
evidence identifies consequential queueing; a visible mutex alone is insufficient evidence.

SI06 adopts early sufficient candidate rejection **after complete immutable blob hydration
and frame validation**, before later source inventory and scientific-payload copying.
Interpretation and logical/name dependency checks use bounded 256/64 windows. Later-page,
wide/skewed, absence/membership and corrupt-frame controls preserve exact reconstruction.
This is not avoidance of initial blob hydration, measured reduced total decoded bytes or an
end-to-end preparation speedup. SI07 adopts facade generation fencing, with old-load/new-view/
active-alias controls; no prior production stale-science defect is claimed.

SI08/SI09 combine authentic interrupted authority/retirement with original-assessment,
fresh/reused and cross-process native science. Their affected scientific restore/rebuild
and Python acceptance is recorded above. Historical internal runs are not scientific goldens; current-run
acknowledged bytes establish persistence preservation, while authored physical expectations
and original tolerances establish current numerical acceptance.

## Outcome recorded after implementation

### What was built and removed

**Implemented:** workflow assembly checks the actual consumed deployment owners and rejects
foreign composition before registration. Every Rust, Python, worker, fixture and benchmark
caller uses the fallible construction boundary and deployment-owned durability selection.
Stable mathematical components and current binding allocations replace obsolete whole-binding
ancestry; trace exports charge their actual owner. Both optional mathematical cache families
cooperate with the existing pool under one scoped pressure demand, native trimming and the
original admission clock. Pressure preserves active values and flights without semantic
invalidation or an additional allocator/capacity authority.

**Implemented:** canonical source protections have issuing-store completion ownership before
submission, correct final-follower election, acknowledged explicit finish and tracked final
cleanup. Physical products retain those owners directly. Public Python close evicts its own
optional retention before guarded disconnect; genuine escaped protected readers still refuse
close and remain usable until their departure. Selected acquisition preserves demand,
and complete inventory uses bounded grouped reads. Exact scoped access and a native inverse
supplier intermediate remove established query amplification. Portable candidate rejection
preserves whole-frame validation and complete dependency/interpretation semantics while
avoiding later work for ineligible candidates. The native maintenance facade fences late fills
from old views while escaped values remain valid.

**Implemented:** typed exact-attempt cancellation refusal reaches one acknowledged settlement
path after actual native join. Incomplete export is scientifically unavailable; complete export
preserves the original scientific assessment. Durable records and study facts read the same
acknowledged completion. Heartbeat ownership covers export/reconciliation and joins before
final sealing; unrelated failures preserve their precedence. Typed host readiness and compatible
startup joining retain authentic allocation/incarnation exclusion, exact PID/start and descendant
drain. Recovery permits only exact content-preserving relocation and independently verifies
the receiving association, permissions, interpreter and new service incarnation.

Removed routes include unchecked workflow/durability injection, changed-binding ancestry,
manual release-only source lifetimes and the untracked physical-owner release wrapper,
per-member declaration/fit inventory retrieval and unconditional selected-conformance inventory, duplicate/unreachable
cancellation settlement, Boolean readiness-to-start fallback, raw recovery-dictionary equality
and its obsolete test, and invalid caller-drop/claim-expiry assumptions. No compatibility API,
second production path, obsolete internal scientific golden or permanent retired-analysis
lineage was added. Fully integrated executor worktrees were retired after scoped provenance
and borrower checks; unrelated worktrees, stored science and concurrent changes were preserved.

Enduring contracts are handed to blueprint §5.4/§14.3/§14.4/§19/§20.6 in their existing
architecture owners, with visible design-edit authorization and revision 145; operational
mechanics belong to [the substrate guide](../dev/surreal-substrate.md).
No new ADR, selected rule replacement or SHOULD exception was required. Existing proposed
ADR statuses are unchanged. This plan's final acceptance and dispositions remain here;
Plan 28 consumes the applicable handoff without inheriting this plan's scope.

Plan 34 is retained as the highest-numbered plan under ADR-0096, not as an active backlog
after closure. Its source review and bounded query evidence still serve this disposition
owner and existing Plan 28 readers. Those newly authored records have no committed archive
copy yet, so deleting their only source would not constitute retirement to Git history.
Their later retirement follows the existing reader/provenance route; no archive directory
or parallel status ledger is introduced.

### Mistakes made and corrected

The first broad native selection ran 20 tests: 17 passed, two failed and one timed out
against zero (`f1e8c1ec-8c09-4b19-8ba2-c011d8f4634b`, 356.012 s).
One conformance controller could fail its final poll after all sixteen workers had completed;
it now joins the remaining assertions only after all entered/finished/released premises hold.
The export matrix repeatedly installed six complete schemas under one test's original clock;
it now shares one isolated database with six distinct case/run/claim/study identities. The
pre-entry watchdog also failed once under concurrent preparation; its phase includes source,
compiler-flight and publication work. Its clock and workload remain unchanged, and the
corrected isolated controls pass without claiming a causal diagnosis of that failure.

The first actual `Blocks::attempt` fixture omitted the variable's authored start/bounds,
causing one failure before the ownership premise. Supplying the intended physical boundary
made the actual native control meaningful; production ownership and scientific tolerances
were not weakened. The first default workflow selection accidentally included an unlinked
Ipopt case; feature-correct selection now requires the linked backend and passes its real
cancellation-after-summary-close journey.

Three cross-process tests passed their scientific, cancellation/retry and process-drain
assertions but failed final fixture cleanup. Caller drop did not imply optional physical
cache eviction, and an older asynchronous release wrapper delayed tracked-cleanup registration.
The cache owners are now explicitly evicted after caller departure, the wrapper is deleted,
and ten affected worker/physical controls pass. The Python run similarly exposed four
teardown errors plus one nested collector failure (28 passed, one failed, four errors,
316.37 s). Its fixture now evicts all borrower caches before closing any, and its child
reproduces the parent's exact grouped identity without removing invocation ownership.
The standalone recovery child also departs its local result/table/package/physical owners
before cache eviction and disconnect, including partial-construction/error paths; cleanup
failure remains visible without replacing a primary scientific error. Independent review
also caught the public close regression behind that fixture symptom: disconnect alone could
wait indefinitely on the runtime's own optional cache pins. Public close now evicts those
pins first, preserving the existing protected-owner refusal and repeat-close behavior.
The first public-close selection passed 30 of 31 controls without teardown errors; the
negative control had not actually populated physical retention before asserting its presence;
preparation alone did not establish that premise either. Source tracing also established that
a package does not retain continuous protection between its reads. The corrected control
completes a real solve, retains an unread canonical result reader, and fetches its server-backed
block after refused close before dropping the reader and retrying. Decoded Arrow buffers own
their allocation independently and are not misrepresented as durable read protection.
The first scientific recovery selection failed at seed import because the probe's physical
dependency contained only 30 hexadecimal characters. Its identity now matches the unchanged
32-character physical-primitives fixture; original source, physics and tolerances remain intact.
The corrected seed then passed, but its caller retained the superseded raw metadata lock
around lifecycle operations. Entering the lifecycle reservation attempted the same non-reentrant
file lock. The exact owned invocation was interrupted, its final cleanup completed, and the
four stop/reconfigure/start/backup wrappers now use the existing reentrant lifecycle owner;
the metadata-only config write retains its short lock. No production clock or drain changed.
The next real reopen exposed another oracle mismatch: the manifest digest, decoded typed
tables and schemas (including metadata values) were equal, but IPC bytes differed because
metadata map insertion order changed across processes. Transport encoding is not scientific
content. The correction preserves complete schema metadata and exact float/binary bits while
removing metadata map order from the comparison premise; signed-zero and changed-content
negative controls remain required.
With that correction, actual scientific reopen passed. Restore then exposed an incidental
directory-mode assumption: the publisher's intermediate directories were `0775` beneath
the private installation root, while restore intentionally reconstructs them as `0700`.
The recovery oracle must accept this permission narrowing, while rejecting added permissions,
lost owner access, changed ownership, changed file modes and symlink/association changes.
The next fresh restore exposed a separate phase mismatch: credential rotation materializes
the new installation's systemd unit under closed maintenance admission. A unit file is not
readiness or restart qualification. The oracle now distinguishes this legitimate effect
from copied qualification/open admission and still requires distinct instance/namespace/database.

Actual lifecycle qualification and independent review exposed joins that pure cgroup checks
missed: selected-store admission must not eagerly resume unrelated parked services; an empty
cgroup does not prove that a recorded PID/start lifetime has disappeared; host reconciliation
must not bypass that lifetime check; and a stale borrowed snapshot must not mutate a replacement
allocation. The affected owners now verify those facts before effects. Original admission/stop
clocks and strict uncertain-owner refusal remain intact. A transient compiler-cache observation
failure triggered the prescribed ready check and retry; its exact cause is not established.
Scope-end lint/type failures were repaired against zero, rather than recorded as tolerable noise.

### Deliberate deviations and limits

The query investigations adopted applicable corrections despite small/noisy or slightly adverse
individual timings. An ineffective binary-request index was rejected because it did not improve
the pinned access path; per-problem pacing is retained because the distributed guard remains
the real authority. Neither decision is an unexamined deferral of a core finding. Inverse
supplier materialization remains subject to managed server capacity, with an explicit
high-degree measurement trigger at its existing query owner.

SI06's sufficient early rejection occurs after complete blob hydration, rather than claiming
an unimplemented pre-hydration path. This preserves the authenticated frame contract and
avoids speculative metadata duplication. No universal state manager, custom cache inventory,
Arc-count ownership inference, transport-fault platform, new resource cap, relaxed timeout or
new thermodynamic validation framework was introduced. Current-run retained-byte checks prove
persistence preservation; they do not make immature historical internal runs numerical goldens.

Scoped qualification does not certify all product/scientific functionality, the full Plan 28
E3/E4/E5 campaign, remote deployment, power-loss recovery, the historical four-GiB failure's
sole cause, the historical watchdog's exact cause, high-degree peak server memory, reduced total
candidate decode bytes or an end-to-end preparation speedup. No formal proof is claimed.
