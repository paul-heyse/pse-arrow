---
title: Scheduling, resources and result completion
status: done
date: 2026-10-20
adrs: [ADR-0211]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, numerical-execution, performance, storage, validation]
review_sources:
  - docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
---

# Scheduling, resources and result completion

## Purpose and boundary

This supporting plan proposes removing avoidable workflow coordination, capture and
admission work while keeping scientific permission, native ownership and acknowledged
publication intact. The [Plan 48 coordinator](48-execution-performance-and-numerical-efficiency.md)
owns the combined design, all finding dispositions, inquiry outcomes and rule decisions.
This document owns the F packet descriptions and packet progress only. Its proposed
mechanisms are not implementation authority. The coordinator's
[rule decisions](48-execution-performance-and-numerical-efficiency.md#rule-decisions)
record the operator's 2026-10-20 acceptance of RC01–RC12; RC09, RC10 and RC12 permit the conditional
investigations below without predetermining their mechanism choices.

The basis is the [recommendations audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md#scheduling-capture-and-resources)
and its [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md#r2-orchestration-resources-and-diagnostics).
AF09, AF10, AF11 and AF17 supply bounded corrections. I28–I33 are questions to settle,
not instructions to install a common pool, subscription service or commit queue.
The focused foundation assessment uses Core 3.4, Efficient Architecture 1.0 and the
applicable Process Simulator 1.5 criteria selected by `standard.toml`. It addresses
AP-02/AP-03/AP-07, DP-09/DP-10/DP-19/DP-20 and PS-10/PS-11 at the affected boundaries;
it does not repeat or extend the audit into whole-system qualification.

## Actual baseline and consumed contracts

The current source gives this work useful foundations:

- `crates/pse-runtime/src/workflow/study_execution.rs` maintains finite pending lane
  futures, continuation retention and separate preparation/execution completions.
  Its grouping uses `batch_backend`, whereas the numerical batch owner in
  `crates/pse-runtime/src/math/solves.rs` has fuller legality information. A backend
  match therefore does not itself establish a legal common native operation.
- The local frontier constructs pending candidates and revisits readiness before
  available capacity. This is the local AF10 surface. Canonical readiness already has
  eligible/pending hints, bounded outgoing notifications, repair and exact claims;
  those mechanisms are a preservation constraint, not a second implementation task.
- `record_execution` awaits `retain_occurrence`, which can await
  `CompactStudyResult::capture`. Coordinator polling can therefore pause while an
  already completed lane's selected source is exhausted. The existing continuation
  owner distinguishes private native reuse for one successor from fan-out seeds.
- `math/jobs.rs` already separates reservations and active acquisition, supports
  growth and charges native stacks. The native service already retains compatible
  lanes, and adapter scopes already retain teams. The audit identifies remaining
  unsupported producer bounds and preparation/member work; it does not establish
  that every request creates a new native thread or pool.
- `workflow/worker.rs` refills productive worker lanes immediately. Its poll sleep
  follows idle discovery. `workflow/durable.rs` owns exact claims, in-process
  heartbeats and immutable result completion. Warm managed-primary reuse exists.
- `crates/pse-backend-native/src/transport.rs::recover` linearly searches the
  original row contract for each returned quality row. Recovery can return subsets
  and reordered rows, so positional identity cannot simply be assumed.

These observations are source-inspected baseline facts, not measured explanations of
the four/sixteen-point timing curves. The audit's claims about existing mechanisms
are retained with their scope; this authoring pass did not run native workloads or
requalify durable recovery.

Plan authoring changed documentation only. No product code, test, dependency, native
deployment or architecture contract was changed, and no build, test or performance
campaign was run. The assessment is limited to the cited audit and focused current
source inspection; it supplies no product-closure evidence.

The enduring contracts remain [native ownership, blueprint §18.8](../authoritative_design/sections/numerical-execution.md#section-18-8),
[studies, blueprint §19.3](../authoritative_design/sections/workflows-and-results.md#section-19-3),
[result assessment, blueprint §19.2](../authoritative_design/sections/workflows-and-results.md#section-19-2),
and [publication, blueprint §20.2](../authoritative_design/sections/identity-and-publication.md#section-20-2).
The durable lifetime and uncertainty contract is owned by
[blueprint §20.6](../authoritative_design/sections/identity-and-publication.md#section-20-6).

## Target and consequential choices

### Compatibility comes from the numerical owner

The workflow should submit already-ready members to a conservative compatibility
operation owned beside native dispatch. That operation describes whether members can
share the same lawful direct batch, accounting for actual profiles, inner threads,
composition, start permissions and adapter constraints. It need not expose native
settings for workflow code to reinterpret. The workflow partitions by that product;
the native owner retains final safety checks and every member's original assessment.

Keeping backend-only grouping preserves a simple caller but pays late fallback and
hides useful intent. Copying compatibility checks into the workflow would introduce
two semantic owners. The target exposes the existing owner's conclusion, allowing an
adapter to add a supported batch restriction without teaching orchestration native
details. Conservative singleton dispatch remains valid when compatibility cannot be
established; it must not manufacture a fresh start or alter a continuation policy.

### Readiness work follows events and capacity

The local executor should keep an ordered ready frontier with indexed membership for
waiting/in-flight occurrences. Terminal predecessor transitions, retry eligibility,
capacity release and cancellation cause relevant transitions; a ready occurrence
blocked only on capacity is not reconsidered on unrelated completions. Ordering,
continuation priority, attempt limits and exact point revisions remain under the
existing pure study policy. The index is an access path over those facts, not a new
definition of readiness or a durable dependency-counter system.

Retain simple policy evaluation at a meaningful event rather than introducing a general
scheduler. For a wide frontier with one skewed lane, visit counts should follow changed
members and actual capacity opportunities. The test oracle must independently specify
the expected issued order/outcomes, so a new queue cannot validate itself.

### A lane completes its own capture and close

Each finite lane should own native execution, required original assessment, selected
capture and required producer completion through its terminal return. It retains the
`RunResult`, selected source, producer, allocation and activity owners while those
steps are live. The coordinator receives a settled occurrence product, mandatory
completion and the minimal authorized continuation product; ordinary bookkeeping
does not then exhaust a source cursor inline.

Independent completed lanes close or enter existing qualified neutral parking.
A unique successor may still inherit the eligible private owner needed for native
reuse. Fan-out keeps the existing immutable seed semantics and bounded consumer
retention; it does not share a mutable native session across successors. Failed,
cancelled or uncertain work cannot park healthy state. No final chunk, empty cursor
or projected table alone proves terminal producer success: errors after the last
nonempty chunk and actual producer drain remain observable before completion.

This change removes an identified coordinator await without requiring all lanes to
be spawned. If synchronous projection still monopolizes polling, I32 examines only
that remaining work. Any selected spawn must carry observation, invocation, canonical,
activity and population context explicitly, retain the original clock/cancellation
authority, preserve every consumed execution/original/provider scope, and join issued
work. Thread-affine `!Send` native objects remain
constructed and destroyed on their owning thread.

### Shared preparation does not share lane authority

I28 may separate a completely keyed immutable admitted basis and its in-flight
producer from the lane that binds changing values, starts and providers. Sharing
immutable structure can remove repeated preparation; sharing an entire lane would
also share mutable permissions and is unsound. The key includes every consumed
structure, interpretation, physical/provider and policy premise. Value and seed
bindings, mutable provider workers, original scopes, run/attempt identities and
lane permissions remain private. If existing keyed preparation already covers the
observed work, extend or consume it rather than adding an overlapping cache.

The [source/preparation plan](48a-source-checking-and-incremental-preparation.md),
[receiving-validation plan](48b-registry-and-receiving-validation.md) and
[mathematical-products plan](48c-mathematical-products-and-compiled-execution.md)
own the relevant immutable products and validity premises. F04 consumes their actual
available slice; an agreed interface is insufficient to claim production reuse.

The preparation key also serves successive compatible durable studies, independently from
sharing concurrent local lane admission. Project out only output-presentation fields not
consumed by preparation; retain numerical/product demand, selected model/body/layout,
provider capabilities and every other consumed premise. Full versus Compact presentation
may reuse an identical prepared basis only where their scientific/product demand is the
same. Each study still establishes current receiving permissions and fresh claim/fence/start
ownership. Test a presentation-only key hit across successive studies, a changed derivative
or numerical demand miss, and equal-looking content under different receiving permissions.
Put this key correction in F05 even if I28 declines additional local sharing.

### Resource and durable lifetimes follow actual work

Capacity reservations are conservative permissions for simultaneously live work,
not predictions of RSS. Extend computed bounds and growth only for actual producer
families with a complete before-allocation contract. Count actual persistent/native
teams, stacks, nested workers, overlap and destruction lifetimes once at their
owner. Unknown foreign allocations remain conservatively prepaid. Waiting for memory
must release unrelated CPU and exclusion capacity; cancellation retains charges
until the real producer/native thread drains.

Local and durable bulk execution can share the numerical operation while retaining
separate occurrence authority. A durable member still has its own preparation,
claim/fence, start, heartbeat, result association and ACK. A bulk interface that hides
the same scalar round trips does not settle I29. Existing readiness hints, bounded
repair, in-process heartbeat, immediate productive refill and warm primary ownership
remain in use. Push notifications, if useful, are hints with explicit cleanup,
resubscription and authoritative fallback, never claim or readiness authority.

I30 permits a separate finite commit stage only after useful compute capacity is
observed waiting for actual publication. The alternative keeps joined completion and
removes demonstrated crossings first. A selected split must bound pending product
count and bytes, retain source/activity/allocation owners through publication, and
backpressure compute. It distinguishes numerical completion, sealed output and
acknowledged durable visibility. Lost ACK settlement uses the original identity;
unknown effects never authorize replaying science. The coordinator's accepted RC12
must have its required contract route available before adopting this separation.

The [native-controls plan](48d-native-controls-starts-and-recycle-policies.md) owns
adapter reset/start semantics; the [dynamics/fitting plan](48e-dynamics-fitting-and-preconditioning.md)
owns trial reuse. F08 consumes those qualified resets and resource scopes instead of
installing universal handle retention. Configured, admitted and actually active widths
remain distinct. Width equal to all cores is not the default conclusion.

## Work packages

The delivery targets began as **Proposed**; the State column records current package
progress; findings, inquiry choices and rule decisions have their single live owner
in the coordinator.

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| F01 — Native-owner batch partition | Existing resolved native profiles and per-member original admission | Expose conservative numerical-owner compatibility and partition ready batches before dispatch (AF09). Compare compatible and incompatible members, changed inner threads/composition/start permissions, native final checks and member assessments. Remove backend-only grouping as the decision mechanism and tests that assert its incidental fallback. | done: implemented; exact native-owner compatibility and actual four-ready POUNCE batch/close controls passed; selected assembled acceptance recorded in48h |
| F02 — Indexed local ready frontier | Existing pure study policy, predecessor/continuation facts and capacity events | Replace repeated ready-but-blocked candidate visits with ordered ready work and indexed waiting membership (AF10). Verify independent expected order, fan-out, retries, capacity pressure, cancellation and revision changes; count visits for a wide/skewed frontier. Delete replaced local scanning and its private fixtures without replacing canonical readiness. | done: implemented; pure frontier and actual 32-point held-cursor order/values/visit-bound control passed; selected assembled acceptance recorded in48h |
| F03 — Lane-owned capture and completion | Existing finite lane and selected capture contracts; compatible continuation ownership | Move selected capture/close and mandatory producer completion into each lane (AF11). Verify another lane progresses while capture stalls, failure after the last chunk prevents success, cancellation drains, unique-successor reuse survives and fan-out retains only eligible seeds. Delete coordinator-side capture exhaustion and full-run retention made unnecessary by the settled product. | done: implemented; focused capture/late-error/cancel/continuation and corrected resource/mandatory-close controls passed; selected assembled acceptance recorded in48h |
| F04 — Immutable admission sharing | I28 outcome; available immutable product/validity slice from 48a–48c | If adopted, share completely keyed immutable preparation/in-flight work with private lane permissions, values, seeds and provider workers. Check equal-basis reuse, changed provider/policy/structure and concurrent cancellation without authority leakage. Remove repeated construction and any superseded cache at the same scope; a no-adopt outcome records its observed reason and trigger at the coordinator. | done: bounded equal/changed-premise inquiry passed; existing immutable sharing retained; coordinator owns I28; selected assembled acceptance recorded in48h |
| F05 — Shared bulk and remaining durable crossings | F01 compatibility; I29 outcome; existing claim/fence/ACK contracts | Share dispatch and correct successive-study preparation keys to exclude only nonconsumed presentation; retain demand/receiving dependencies. Remove demonstrated discovery/claim crossings and select push hints only if justified. Check presentation hit/demand miss, per-member lease/start/ACK, claimed-but-unstarted bounds, reconnect/cleanup and repair/refill/warm reuse. Delete replaced scalar loops/subscriptions only as their consumers migrate. | done: key projection and actual successive admission/receiving controls passed; corrected cold claim probe passes with actual claim-await observations; coordinator owns I29; selected assembled acceptance recorded in48h |
| F06 — Conditional finite commit stage | F03 completion ownership, F05 crossing assessment, observed publication blocking; I30 outcome and accepted RC12 contract route | Adopt separation only with bounded products/bytes and an explicit terminal/ACK state transition. Test backpressure, cancellation with queued products, lease expiry, failed seal, lost ACK and exact uncertainty settlement; preserve actual source/native drain. Otherwise retain joined completion and record a no-adopt outcome with the blocking trigger. Remove the replaced joined path only if the selected route supplies all its obligations. | done: ADR-0211 bounded Running/Publishing implemented; actual same-worker refill/cancel/late-arrival and pure failure controls passed; selected assembled acceptance recorded in48h |
| F07 — Producer bounds, growth and stack ownership | I31 producer-specific outcome; actual native/resident lifetime and growth premises | Extend unsupported producer bounds and pre-allocation growth; charge actual teams/stacks without omission or duplicate charge. Test overflow, denied growth before allocation, nested teams, pressure while idle and cancellation/destruction. Remove opaque prepayment only where its complete replacement is established; retain conservative coverage elsewhere. | done: producer-bound inquiry complete; existing opaque coverage retained where complete bound unavailable; coordinator owns I31; selected assembled acceptance recorded in48h |
| F08 — Remaining placement and native concurrency | F03 work removal; settled I28/F04 sharing decision and actual relevant products, including a no-adopt outcome; F07 applicable capacity; I32/I33 outcomes and accepted RC10/RC09 contract routes where applicable | Select only justified remaining spawn/pool/worker placement and a concrete native isolated-concurrency route. Verify all context propagation, owner-thread `!Send` teardown, actual joins, reset/failure discard, exclusions and bounded IPC where applicable. Preserve existing lanes/teams; remove only churn or placement machinery actually replaced by an adopted route. | done: actual placement/isolation inquiry complete; no new pool/process route adopted; coordinator owns I32/I33; selected assembled acceptance recorded in48h |
| F09 — Indexed quality-row recovery | Original contract IDs and normalization; existing subset/error meaning | Build/reuse a row-ID-to-ordinal map or establish complete positional correspondence once (AF17). Independently check reordered/subset rows, nonuniform scales, unknown IDs and duplicate/invalid contracts under existing admission. Delete the per-quality-row linear search; preserve physical values, tolerances and error behavior. | done: implemented; targeted verification passed; selected assembled acceptance recorded in48h |

F01, F02, F03 and F09 can deliver independently where shared edit surfaces allow.

### F01–F03/F09 execution checkpoint — 2026-10-20

F09 now builds one transient original-row-ID index per quality-bearing coordinate recovery.
It removes each quality row's repeated linear search, preserving reordered/subset output,
nonuniform scales, the unknown-ID error and preceding partial mutation. Existing contract
admission still refuses duplicate IDs; direct malformed input retains the former first-match
behavior. No reusable owner or process-lived row cache was introduced.

F01 moves the existing direct automatic-strategy preflight into the mathematical owner.
The workflow groups only by that owner's conservative complete profile/thread/backend result;
native final admission, method decision receipts and per-member original assessment remain.
F02 separates event-dirty work from ordered capacity-waiting membership, preserving the pure
policy and continuation-first order while avoiding unrelated capacity-blocked visits.
F03 prepares settled selected occurrences in their lane, retaining the source, producer,
scope and native owner through capture. Current coordinator facts decide unique-successor
retention; native close/parking that remains after that decision completes in the same finite
pending-future set before terminal bookkeeping releases its lanes. No new spawned pool or
commit stage was introduced. Compact capture failure remains distinct from original scientific
completion: a failed selected producer refuses detail even when the original completion is usable.

F09's two controls passed in the 15-test focused run recorded below. F01 production extraction
compiled before its new behavior control and the subsequent F02/F03 changes. Their new tests
include real selected-cursor skew, failure after the last chunk, cancellation/drain, current
native compatibility changes and a bounded wide-frontier visit oracle. The native selection reached
all skew behavior assertions and passed late-error, cancellation, current compatibility and
continuation controls. The focused retry also passed the corrected neutral-pool/refusal
fixtures, including actual 32-point skew and four-ready batch completion.
These package changes do not settle F04–F08 or any coordinator-owned inquiry.

The F05 private reuse key now exhaustively projects `StudyOperation` while excluding only
its output presentation. Source, complete operation/settings, preparation controls, version,
canonical horizon assignments and binding target membership remain. Canonical stored study
identity still includes the original full document. Source tests cover a presentation-only
admission hit, consumed compiler/derivative/numerical/source misses and changed receiving-provider
owner refusal. This key correction does not implement durable bulk dispatch or settle I29.
An I28 source probe separately compares actual immutable allocation IDs/preparation counts for
two independent lanes, then changes compiler/policy/provider/source premises. Its scope owners
remain private; no whole-lane sharing or additional cache was adopted. Both actual native controls passed.
F04 design can begin from existing products without waiting for every preparation
optimization, but adoption needs their actual validity/ownership contract. F05 need
not wait for F06, and F06 is not a prerequisite to useful bulk dispatch. Resource
ownership work accompanies any adopted concurrency; F07 is not permission to lower
opaque reservations before a bound is available. F08 should examine residual work
after the simpler corrections rather than use a generic pool as their foundation.

### F04–F08 inquiry checkpoint — 2026-10-20

**Completion audit clarification:** accepted I29 permits scalar dispatch as a finite no-adopt
outcome. F05 shares the numerical policy/dispatch owner and fixes preparation keys; eligible
native bulk remains local. Durable members retain independent task scopes and scalar execution.
No new durable native bulk path or independent native member-control contract is claimed.

The I28 independent-lane probe passed: equal-basis lanes shared the same immutable model allocation, while their private scopes remained distinct. Changed compiler/numerical/provider premises retired only the changed lane, and changed physical source refused. Preparation observations still recorded two views and two observation preparations, so allocation sharing is not evidence that every structural preparation was coalesced.
I29 now has one narrow elapsed phase around the actual canonical `claim_study_point` await;
the former whole-action `managed.worker.claim` phase is corrected to
`managed.worker.work_once`. No consumer of the old phase name was found in the current
repository. Its source probe compares five idle passes with five acknowledged fixed scientific
occurrences, retaining independently owned phase durations instead of subtracting overlapping
preparation/native phases to invent exclusive discovery time. Existing worker refill, readiness
maintenance, recovery and exact lost-ACK controls remain the authority for their behavior.

I30's actual-owner probe passed five ordinary KINSOL/export-through-ACK pairs on one
shared math/pool owner. It then gates a real Table export and a failing Completion export while
another native solve runs. The gate retains the original source, heartbeat and fence; selected
publication cannot become terminal before release. Injected delay alone cannot justify a new
commit stage, and no such stage was introduced. The existing canonical codec claim fixture
supplies transport descriptors, while the source result is an actual independently solved
KINSOL result. This qualifies the export/ACK owner and sibling-progress question, not the full
authored durable dispatch journey. Its ordinary payload is small and cannot rule out heavy-export
blocking elsewhere. Native wait took 1281–2518 μs (median 1471 μs); export through ACK took 774628–923236 μs (median 847503 μs) in the five samples. Publication dominated this small case, but both gated exports allowed another actual native solve to complete on the same math/pool before release. The claim-span probe retains a strict usable-scientific-outcome assertion. Correcting its generic all-fixed helper to the existing successful KINSOL settings exposed a real cold-preparation failure: one exclusive same-service native population loan was consumed concurrently by artifact producers. The other four attempts completed, but the first failed with `concurrent native population loan`. C01 now bounds artifact preparation width to one only inside that actual borrowed population owner, preserving ordinary independent width and the loan refusal. The corrected owner and actual canonical probe now pass. Five ordinary fixed scientific occurrences are usable and uniquely acknowledged. Actual claim timing is recorded separately below; no discovery-by-subtraction estimate is made.

Only POUNCE-convex currently advertises native batch capability. The local mathematical batch
owner shares one cancel/progress/execution scope, and its conservative eligibility refuses a
pre-existing task scope. Durable dispatch instead consumes one claimed `DurableAttempt`, with
its independent heartbeat, run, clock, start and ACK association. Directly reusing the local batch
would therefore merge controls or claim unstarted work. Any later durable bulk target must first
supply independent member control; a discovery/claim cost measurement cannot grant that missing
native capability. Existing study/candidate pages already use bounded pages and readiness hints.

F07 source inspection found existing complete bounds for flat arithmetic producers and installed
growth before known positive excess. Weighted Hessian/Hessian-action/transpose construction and
nonflat control/provider analysis/lowering still explicitly return no complete source bound.
Their conservative envelopes remain. `ConstructionOwner` checks overflow/capacity before growth;
memory waits release tentative CPU/job capacity; actual native join retains stack/population
owners. `scope_stack_bytes` counts distinct actual adapter teams, including simultaneously nested
teams, rather than treating equal widths as one team. Existing pressure, worker-capacity growth, contending-team and cancellation/native-join controls
passed in the runtime selection. The store-free backend selection also passed actual nested-team
stack extents, PETSc process-wait CPU release/deadline preservation and SCIP gate saturation/
cancellation controls. This inspection supplies no lower memory bound or
RSS claim, and no new opaque producer envelope was reduced.
The backend evidence is the shared 28/28 native-local capability selection (`unit-native-capability-package pse-backend-native pse-backend-native/native-solvers solver,klu,isolation,uno,petsc`), zero errors/warnings and 736 unselected; no canonical fixture or isolated process route was exercised.
`PreparedProviderRequest` bounds demanded result cells, but `ProviderFactory::create/create_scoped`
does not expose a complete scratch-construction maximum. A result-cell extent therefore cannot
replace that foreign allowance. Newly built library entries retain their original constructing
job's conservative foreign allowance once per constructor, independently of exact-entry reuse.

F08 inspection follows the remaining real owners: selected cursors yield after delivered chunks;
F03 removes coordinator capture exhaustion without spawning a new pool. Mathematical construction
jobs still create an owning thread and propagate invocation, observation, tracing, runtime and
activity/native context through actual join; retained native lanes/scopes already own qualified
reuse. These facts do not establish a measured preparation-thread bottleneck. A concrete isolated
candidate, two independent PETSc square-root occurrences, is constrained by the serial MPIUNI
process owner. The current managed primary reconstructs recorded studies, but its shared profile
admits one native process and supplies no per-member isolated transport/control/assessment/reap
contract for this candidate. KINSOL/POUNCE remain on their existing eligible lanes. No exclusion,
native reset or deployment contract was widened. Final bounded adoption/no-adoption choices and
their observable reopen triggers remain with the coordinator after the applicable probes.

### F06 reopened target — same-worker publication capacity

The operator's deployment-first direction reopens I30 under ADR-0211. The earlier actual
sibling solve establishes available compute during publication; it does not establish refill
by the same managed worker when all its lanes await ACK. Its source scheduler refills only
when that joined action returns. A finite Running/Publishing state in the existing scheduler
therefore removes a known serialization rather than assuming a generic queue is faster.

The selected target prepays coordination for W scientific plus W publication slots and at
most 2W total lane owners. A worker-only handoff becomes eligible after actual scientific
completion and native owning join; a full publication stage applies cancellation-aware
backpressure. Distinct lanes retain distinct preparation permissions. Every old source,
result allocation, heartbeat, fence, detail scope and required native closure stays owned
through actual ACK/error/drain. Public RunHandle wait, terminal manifests and processed counts
remain terminal-through-ACK. Actual width-one held-export refill, finite claims, late
Completion failure, cancelled waiting-owner settlement and late-arrival refill now pass.
Pure backpressure, lost-channel and prior-failure preservation controls also pass; H02/H03
still own broader deployment/assembled qualification.

The source implementation now extends the existing `serve_owned_lanes` scheduler with
Running/Waiting/Publishing states, a bounded readiness channel and FIFO acknowledgement
of publication capacity. It prepays 2W lane/future owners and coordination bytes before
claiming. The existing `RunResult::finished` reaches this handoff after each scientific
owner's actual joined finish; public completion still waits for the original durable finish.
One pure backpressure/cancel control and an actual width-one worker control with held Table
success, late Completion failure and cancellation are written. Native shutdown precedes
assertions, and the test checks the actual claimed set without assuming claim order.
The first default compile identified an unused test hook outside its canonical capability;
the hooks now follow the actual canonical/Ipopt fixture route.

Independent review returned two further transition gaps before acceptance: an idle discovery
could suppress future polling while an older publisher stayed pending, and an unadmitted
handoff did not react to cancellation. Those scheduler/handoff and required-finish
corrections now distinguish cancellation from lost-ACK cleanup and preserve prior
scientific failures. The actual worker control now keeps its first
publisher gated until a second cancelled attempt has settled with no result/seed payload;
a separate control creates a new occurrence only after held publication and an observed
idle discovery. Pure transition and private failure-preservation controls accompany them.
The pure capacity, late-arrival, lost-channel and prior-failure preservation controls now
pass. Both actual same-worker controls initially failed before claiming work: the fixture
incorrectly forced KINSOL on the original Root two-sided box. The unchanged native
eligibility correctly refused that unsupported representation. The tests now retain the
authored bounds and existing eligible Ipopt default, and their backend-agnostic observers
use the same canonical/Ipopt capability gate. Both actual tests now pass, including all
three original handoff modes and the separate post-idle late arrival. Native shutdown and
complete owned fixture drain precede acceptance; no scheduler rewrite followed from the
unsupported request.

## Scoped completeness handoff — H01

The table covers every recommendation row linked to this companion, including omitted
blanket diagnoses. Shared D/G/H rows retain their other package owners. Package progress
and evidence here do not replace the coordinator's inquiry or finding dispositions.

| Linked coverage rows | Actual producer → consumer | Functional evidence / bounded result |
| --- | --- | --- |
| R1-F08 compatible batch partition; P05 event frontier and batch partition (F01 part) | Mathematical `staged.rs` native compatibility preflight → `solves.rs` and `study_execution.rs` partition, then final native-owner admission | **Tested:** actual complete-owner compatibility and four-ready POUNCE batch controls pass, retaining automatic strategy and method-consumed receipts. Backend-only grouping removed; native safety remains authoritative. |
| R1-F09 local event-ready queue; P05 event frontier and batch partition (F02 part) | `LocalFrontier` event/revision/capacity indexes → local dispatch and current pure policy | **Tested:** two pure frontier controls and 32-point actual held-cursor test pass with ordered usable results, 31 siblings completed and bounded structural policy visits. Local repeated blocked-candidate scanning removed; canonical readiness remains. |
| R1-F10 lane-local Compact capture | `execute_lane`, `study_demand` cursor completion and bounded close futures → selected result plus independent scientific facts and current coordinator bookkeeping | **Tested:** real chunk stall, late completion error, cancellation/drain, Compact release, continuation/fan-out and denied-grant mandatory join controls pass. Coordinator capture exhaustion/full-run retention removed where replaced; full-output Ephemeral remains intentional. |
| R1-F11 indexed original-quality recovery; P17 quality ID map | `transport.rs` one transient ID/ordinal index → original-space quality recovery | **Tested:** reordered subsets/nonuniform scales, unknown ID and duplicate admission controls pass. Per-quality-row linear search removed; first-match and error semantics preserved. |
| R1-C03 presentation-independent preparation key; P05 one shared admission; P05 parallel member pre/post (F04 part) | Exhaustive borrowed `study_operations.rs` key and existing immutable products → successive admission and independent lane preparations | **Tested:** output-only key hit/current receiving and I28 equal/changed-premise controls pass. Same model allocation was shared while scopes remained distinct; two view/observation preparations remain observed. Retain existing immutable sharing; no whole-lane permission/provider sharing. |
| R1-C03 shared local/durable bulk dispatch | Existing local direct batch versus individual durable `DurableAttempt`/heartbeat/start/ACK owners → local dispatcher and `worker.rs` | **Interface-checked:** only POUNCE-convex advertises current bulk eligibility; local batch shares controls and refuses a pre-existing task scope. Durable per-member authority cannot be merged into that owner. No new durable bulk path adopted; I29 actual cold-preparation correction and canonical claim probe now pass. |
| P07 whole-study scans each action; P07 introduce durable dependency counters; P07 500 ms charged per productive point; P07 heartbeat starts a supervisor; P07 warm managed primary missing | Existing bounded candidate pages, readiness hints, repair/refill, lease heartbeat and managed primary → productive worker loop and recovery | **Tested, bounded preservation:** actual refill/readiness/recovery and two lost-ACK controls pass in the 59-test selection. These omitted blanket claims produce no replacement counter, supervisor or per-point sleep mechanism. |
| P07 batch discovery/claims; P07 live/push wakeups | Current authoritative discovery/readiness plus `worker.rs` actual `claim_study_point` await → work-once dispatch | **Tested and Measured:** narrow `claim_acquire` phase and corrected enclosing `work_once` name pass the actual five-idle/five-productive canonical probe after the C01 population-loan correction. Median productive action is 1.300806 s, actual claim await 23.443721 ms and required completion/publication 978.369690 ms. Nested phases overlap; no exclusive discovery estimate, push platform or batched claim adoption follows from this fixture. Reopen selection on representative crossing-bound waits. |
| R1-F10 separate commit/capacity stage; R1-C03 bounded durable commit stage; P07 separate commit/publication; §§4/11 universal compile-once/persist/publish-off-path (F06 share) | Actual native RunResult and canonical Table/Completion export → original source/heartbeat/fence retained through ACK or error | **Tested and Measured:** five ordinary native/export pairs and gated success/failure export allow another actual solve on the same owner before release. Earlier probe retained joined completion; the later same-worker serialization trace now selects ADR-0211 bounded Running/Publishing implementation. Small codec-claim fixture is not a whole authored managed-primary journey or same-worker refill proof. Actual same-worker Table success, late Completion failure, cancelled no-payload settlement and post-idle late-arrival controls now pass, alongside pure lost-ACK/prior-failure preservation. A05/C07 retain persistence ownership; H02/H03 retain assembled qualification. |
| P08 accounting/RSS ratio proves bad bound; P08 use computed bounds; P08 replace opaque prepayment with growth; P08 pool-scoped stack charging | Actual flat/nonflat/provider constructor bounds, `ConstructionOwner` and distinct adapter-team extents → jobs/native allocation, growth, wait and join | **Interface-checked and Tested:** complete flat bounds preserved; uncovered Hessian/action/nonflat/provider scratch retains conservative prepayment. Runtime overflow/growth/pressure/CPU-release/join and backend nested-team stack controls pass. No RSS-derived lower bound. Reopen only a complete producer-bound API or observed producer pressure. |
| P05 single coordinator means serial native solves; P05 spawned lanes; P05 persistent common work-stealing pool; P05 width = cores; P05 parallel member pre/post (F08 part); P17 POUNCE-convex pool per batch | F03 bounded futures/cursor chunk yields and existing context-carrying construction/native owners → local coordinator and actual joined native execution | **Tested, bounded preservation:** 32-point real capture skew and existing worker/context/drain controls support current placement after work removal. No remaining thread-creation/starvation bottleneck was demonstrated; no new pool, spawned lane or cores-equal width. Reopen actual residual starvation or repeated worker churn with useful work waiting. |
| P17 HiGHS reset on every release; R1-RC05 native isolated concurrency | Existing retained lanes/adapter reset and serial MPIUNI PETSc process owner → current native dispatch; managed worker CLI lacks candidate-specific isolated input/control/assessment/reap contract | **Interface-checked and Tested:** existing reset/exclusion/pressure controls retained; PETSc wait and SCIP gate controls pass in the shared 28-test backend selection. No unqualified isolation route or reset relaxation. D04 owns HiGHS policy; coordinator owns I33/RC09. Reopen a concrete reconstructible isolated workload with complete lifecycle/resource contract. |
| P05/P07/P08/P17/P19 numeric gates (F07 share) | Existing consumed allocation/admission and scientific assessment → supported worker/native operations | **Preserved:** no arbitrary width, RSS ratio, percentage or timing gate was introduced. Actual pressure/join tests establish behavior; H02/G02 retain their shared qualification/observation portions. |

B/F source changes have focused functional qualification, without an assembled qualification
claim. I29, the corrected reference-worker envelope and both actual F06 same-worker controls
now pass. This source/consumer review found no further F-local mechanism or legacy-deletion
gap. Adjacent fitting and dynamics qualification remains with its owning package; this handoff does not duplicate its current repair state.
F04/F07/F08 retain their explicit bounded outcomes and reopen conditions; their final inquiry
disposition remains solely at the coordinator. H02/H03 still own authored worker/Python
journeys, full static/codegen qualification and final integration.

### Current canonical qualification boundary

**Tested (partial scope):** the explicit runtime native/canonical feature route with selected
compiler root ran 31 focused controls: 25 passed, 6 failed, 1675 unselected; zero is the failure
baseline. All six A02 compiler controls, G04 owner closure, I29, the prior reference refusal,
foreign lease, fixed-window team, worker reuse and the pure F06 transition/failure controls
passed. Failures are the two actual F06 gates, two fitting comparisons, authored BBD and one
selected-Jacobi accuracy control. Seven unnecessary qualification warnings in the new F06 fixtures were repaired; the
narrow four-control diagnostic rerun compiled with zero errors/warnings. That rerun passed
the corrected authored BBD control and retained three failures: the original selected-Jacobi
accuracy assertion and both ineligible F06 fixture requests. The latter now use their eligible
Ipopt route and passed the later actual same-worker selection below. Raw failed
selection remains `/tmp/pse48-runtime-canonical-f06-lm-controls.log`; the owned functional
canonical fixture was quiesced and stopped after all test owners drained.

**Tested — F06:** the later `just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests' '<focused filter>'
--success-output immediate` selection ran 32 tests: 29 passed, 3 failed, 1394 unselected,
with zero compiler errors/warnings against the zero target. Both
`same_worker_publication_handoff_refills_and_bounds_actual_unacknowledged_attempts` and
`same_worker_publication_refills_a_study_created_after_actual_idle_discovery` passed.
The first exercised held Table success, late Completion failure, exactly bounded pending
claims and cancellation settled without payload admission while the first publisher stayed
held. The second created its new occurrence only after an actual idle discovery and
completed its scientific owner before releasing the old publication. Every issued owner
and native service drained before assertions. That selection's three failures concerned fitting foreign-population coverage, persisted
LM source-selection decoding and selected-Jacobi observation counting. The subsequent
narrow selection passed the latter two and the new LM workspace-eligibility/method controls;
the passive-population cleanup assertion subsequently passed in the affected 12-control
selection after its owner retained the actual immutable preparation and recorded constructor
charges. Both actual F06 controls and the migrated supervisor cancellation control also
passed that selection after the supervisor/scheduler future-construction corrections.
The selection ran 12 tests: 11 passed and one fitting curvature failure-injection assertion
failed, with one deprecated test-helper API warning subsequently repaired by its owner.
The exact-one curvature retry then passed with zero compiler errors/warnings after the
control targeted its actual child execution allowance and asserted unchanged output on
refusal; the failed 12-control selection remains distinct. Raw selection/results remain
`/tmp/pse48-runtime-curvature-final-control.log`,
`/tmp/pse48-runtime-final-twelve-controls.log` and
`/tmp/pse48-runtime-profile-f06-controls.log`; this failed combined selection is not reported
as an enclosing native pass or a whole managed-primary journey.

**Measured, fixture-bounded:** I29 idle action times were 1330, 838, 545, 393 and 422 μs.
Productive action times were 1588561, 1233940, 1311882, 1266208 and 1300806 μs; actual claim
awaits were 52048876, 23886605, 23342749, 23443721 and 19712128 ns; required completion and
publication were 1111210732, 924352703, 992399386, 945887769 and 978369690 ns. This ordinary
local codec/worker fixture establishes neither managed-primary deployment qualification nor
heavy-export behavior, exclusive discovery time, or compute saturation.

## Finite inquiries and decision boundaries

These descriptions explain the coordinator's I28–I33 inquiries. Their decisions and
observable revisit triggers are recorded there, not in a second local register.
Each inquiry can finish with adoption or no adoption. No unknown library capability
or deployment mechanism is mandatory merely because it appears among the options.

### I28 — Which immutable admission is actually repeated?

Compare consuming the existing keyed prepared basis, extending its immutable product,
and retaining separate lane admission. Inspect one equal-basis many-case path and a
changed structure/provider/policy path; distinguish stable preparation from value,
seed and permission work. Source ownership plus construction counts should settle
whether an additional shared producer removes real work without sharing authority.
Stop after those cases establish the complete key and lifetime or expose a missing
premise. Adopt only that bounded sharing slice; otherwise record no adoption and
revisit when repeated immutable construction is observed on another supported path.

### I29 — Which local/durable crossings remain material?

Compare shared native bulk with current scalar dispatch, bounded batched discovery or
claims, and push wakeup hints over the existing fallback. Use a short productive pass,
an idle-to-ready transition and a lost-response/reconnect case. Attribute preparation,
discovery, claim, numerical work and publication separately; do not charge idle sleep
to productive points or supervisor observation to lease heartbeat. Stop when the
remaining crossings and authority/lifetime constraints distinguish a useful route.
Adopt only the demonstrated slice, or record no adoption with a revisit trigger of
observed crossing-bound idle time/contention or delayed readiness despite healthy
authoritative fallback.

### I30 — Does joined publication actually block useful compute?

Compare current joined completion after F03/F05 with a finite separate commit stage.
Use a skewed publication case and a slow/failing store while observing active compute,
pending products and retained bytes. Stop if useful compute is not blocked, or after
one bounded separation design accounts for all owners, backpressure and uncertain
effects. Adoption requires observed blocking, capacity credibility and the accepted
RC12 contract route;
no adoption retains joined truth and reopens only when useful capacity waits on
publication under the supported workload. A queue is never counted as completed work.

### I31 — Which producer can safely replace opaque prepayment?

For each actually uncovered family, compare a complete computed maximum, qualified
growth before allocation, and existing conservative prepayment. Inspect allocation,
retry/overlap, native team and teardown paths; use pressure/denied-growth controls to
expose late reservation or CPU-held waits. Stop per producer when coverage is complete
or an opaque allocation defeats the premise. Adopt only the proven family/lifetime;
otherwise retain its guard and revisit on a qualified allocator/bound API or observed
admission pressure for that producer. RSS alone never supplies that premise.

### I32 — Which remaining work warrants different placement?

Compare the repaired finite futures/native lanes with narrowly spawned synchronous
projection or member pre/post work and with reuse of preparation workers. Examine a
skewed cheap-case path after repeat work is removed. Capture invocation/observation,
canonical/activity/population context, cancellation and thread-affinity implications
before selecting a placement. Stop when residual polling blockage/thread churn is
localized or absent. Adopt the smallest useful route, subject to RC10 where it changes
the retained-lane/reset contract; otherwise retain placement and revisit on observed
coordinator starvation or repeated preparation-worker creation with useful work waiting.

### I33 — Is another native concurrency route useful and supportable?

Compare existing qualified lanes/exclusions with the existing eligible child-process
route and a narrowly extended isolated deployment for one concrete workload. Retain
MPIUNI, scheduler/TPI and other actual exclusion constraints unless the accepted RC09
route changes them. Check reconstructible inputs, consumed native compatibility, finite IPC,
original assessment, memory/CPU envelope, cancellation and actual reap. Stop when the
specific workload can be selected against the existing route or a missing capability
refuses it. Adopt only a qualified useful route; otherwise record no adoption and
revisit when that workload demonstrates exclusion-limited useful concurrency or the
missing reconstructibility/bound capability becomes available. These are consumed
scientific, ABI, layout and lifecycle premises; environment/provenance proof remains
eliminated. No host attestation, proof service or broad installation work is introduced.

Only a timing-dependent choice calls for matched repeated observations, such as five
alternating trials with actual ranges/medians. Start with short supported cases,
source counters and the existing cooperative scientific budget around 60 seconds,
including real drain. No all-core width, RSS ratio, microsecond target or percentage
gate follows from these inquiries.

## Preservation, migration and verification

The physical target remains **0.001** in the selected engineering cases. Scheduling,
recovery maps and capture may change organization, but preserve original-space
assessment, typed termination, start/branch permissions and achieved accuracy. Exact
local work admission and cancellation remain independent of optional telemetry.
Both pre-work stop checks and post-success late-result vetoes remain; panic handling
and failed-work reconciliation are retained. Native destruction remains on its owner
thread, and issued work retains its charges through actual drain.

Migrate Rust local studies and their Python consumers together for changed completion
products; migrate durable worker/export/ACK consumers in any adopted F05/F06 slice.
Update compact report, continuation and quality consumers without introducing a
compatibility execution path. Once each replacement's targeted tests pass and callers
have moved, delete the replaced code, callers, tests and fixtures in that same change.
An inquiry selecting no adoption creates no replacement mechanism or temporary path.

**Proposed verification:** package implementation uses compile checks for touched
packages and meaningful targeted tests, including `just unit-package pse-runtime
study`, focused worker/durable filters, `just unit-package pse-backend-native
transport`, and relevant native lane/reset/callback tests selected against the final
change. These are planned routes, not executed commands. Arrow-consuming correctness
closures retain explicit force-validation; native journeys retain the native environment.
Tests must distinguish issued work from settled work, retained capacity from RSS,
and scientific completion from durable ACK.

The [diagnostics plan](48g-diagnostics-and-review-publication.md) supplies reliable
lifecycle evidence before extra stage instrumentation. The
[qualification/publication plan](48h-qualification-and-design-publication.md) owns
assembled study/solver/Python/durable journeys, matched benefit measurements where a
selection depends on them, final hygiene and enduring design publication. Targeted
passes cannot close cross-plan findings or establish unexercised scientific support.
Every executed command later records its actual mode, scope and failures against the
zero baseline at its evidence owner.

## Outcome (recorded after implementation)

### What was built

**Implemented and Tested, completed adopted scope (2026-10-20):** the package table records
all delivery/inquiry outcomes. [Plan48h](48h-qualification-and-design-publication.md#final-qualification-accounting)
records completed selected assembled acceptance and scope-end checks. The individual
observations below retain their original conditions and partial-run boundaries; they do
not assert a universal speedup or extend unsupported capabilities. There is no outstanding
authorized package work in this companion.

**Implemented and Tested (partial package scope):** F09's transient index and two local
`transport::tests::quality_recovery` controls passed in the default Nextest run
`just unit-package pse-codegen '(package(pse-codegen) & test(typed_local::tests)) | (package(pse-engine) & test(typed_modeling_eligibility)) | (package(pse-admission) & test(typed_modeling)) | (package(pse-backend-native) & test(transport::tests::quality_recovery)) | (package(pse-columnar) & test(flight::tests))' -p pse-engine -p pse-admission -p pse-backend-native -p pse-columnar --execution-effects native-local`:
15/15 selected tests passed, 600 unselected, zero failures against the zero target, with explicit
force-validation closures. `just check-package pse-codegen -p pse-relations -p pse-engine -p pse-admission -p pse-backend-native -p pse-runtime`
passed dev all-target checks with zero errors/warnings at the F09/F01-extraction checkpoint.
**Implemented and Tested (further partial scope):**
`just unit-package pse-runtime '(package(pse-runtime) & (test(frontier_tests) | test(occurrence_execution_tests) | test(preparation_key_excludes_only_output) | test(presentation_only_successive_admission))) | (package(pse-admission) & test(typed_modeling))' -p pse-admission --execution-effects native-local`
passed 7/7 default-profile tests (880 unselected), zero failures against the zero target.
It exercised two frontier controls, the preparation-key projection and four typed-modeling
loader controls. Native-gated occurrence and actual successive admission/receiving tests
were not selected in this mode.
One adjacent mathematical-owner dead-code warning was reported to its owner for correction.
The preceding `just check-package pse-runtime -p pse-admission` passed dev all-target compilation;
three adjacent dynamics visibility warnings were repaired by their owner afterward.
**Implemented and Tested (further partial native scope):** the package-native runtime/operations selection with `pse-runtime/native-solvers,pse-runtime/canonical-tests`, explicit force-validation and a fresh task-owned canonical store ran 59 tests: 52 passed, seven failed against the zero target, 1622 unselected. Both worker and test compilation completed with zero errors/warnings. The exact native-owner batch compatibility, actual presentation-only successive admission/current receiving, I28 changed-premise probe, I30 actual export/ACK and sibling-progress probe, existing worker refill/readiness, two canonical lost-ACK controls and runtime construction/pressure/join controls passed.

The focused runtime/compiler native retry used the same task-owned canonical fixture and explicit runtime native-solvers/canonical-tests features. Its 16 tests ran: 11 passed, five failed against the zero target, 1671 unselected, zero compiler errors/warnings. The three corrected occurrence fixtures passed, including the actual 32-point held-cursor sibling/order/value/visit-bound case, four-ready POUNCE batch/close and prepaid-future refusal. Mandatory close under denied extra grant also passed. The fixtures now distinguish returned CPU from intentionally retained neutral native population, explicitly join the pool before asserting full drain, and clear optional products before saturating the refusal fixture. No production resource refusal was weakened.

The retry also passed the actual compiler preparation-owner phase closure, three simulation-identity controls, opaque-constructor precharge/lifetime, failed-forward upgrade and shooting worker reuse. That earlier retry retained five adjacent/inquiry failures: the actual cold I29 C01 population-loan interaction, conditional-reference worker envelope and three dynamics/selected-Jacobi controls. The later 31-control selection passed I29 and the reference envelope; current fitting/dynamics and F06 boundaries are recorded above. Raw selections/results are retained in `/tmp/pse48-runtime-native-59-controls.log` and `/tmp/pse48-runtime-native-16-controls.log`; neither failed selection is reported as an enclosing native pass.
No integrated scheduling, native-study, durable or full-plan qualification is claimed.

### A mistake made and corrected

Two resource-fixture assertions conflated returned CPU with full destruction of intentionally
parked native owners. The corrected controls first assert CPU release, then join the existing
service pool and assert complete population return. The targeted retry passed without weakening
mandatory close, memory admission or the native population-loan guard.

The first F06 actual fixtures incorrectly forced KINSOL on a two-sided bounded Root,
preventing any claim or publication gate. The corrected fixtures retain all authored
science and the existing eligible Ipopt route; their actual success/failure/cancellation
and late-arrival controls now pass. Native eligibility was never weakened.

### Deviations from the plan, deliberate

F04 retains existing shared immutable products with private lane permissions. F06 reopened
after the same-worker serialization trace and now implements the bounded publication handoff
under ADR-0211; its focused actual and pure transition controls pass. F07/F08 retain conservative uncovered
producer envelopes and existing placement/exclusions rather than introducing unqualified bounds,
a new pool or an isolated deployment. Inquiry state and assembled qualification remain with
the coordinator and their named delivery owners.
