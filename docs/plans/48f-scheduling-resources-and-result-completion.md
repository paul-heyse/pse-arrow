---
title: Scheduling, resources and result completion
status: draft
date: 2026-10-20
adrs: []
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

All delivery behavior below is **Proposed**. The State column owns only packet
progress; findings, inquiry choices and rule decisions have their single live owner
in the coordinator.

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| F01 — Native-owner batch partition | Existing resolved native profiles and per-member original admission | Expose conservative numerical-owner compatibility and partition ready batches before dispatch (AF09). Compare compatible and incompatible members, changed inner threads/composition/start permissions, native final checks and member assessments. Remove backend-only grouping as the decision mechanism and tests that assert its incidental fallback. | open |
| F02 — Indexed local ready frontier | Existing pure study policy, predecessor/continuation facts and capacity events | Replace repeated ready-but-blocked candidate visits with ordered ready work and indexed waiting membership (AF10). Verify independent expected order, fan-out, retries, capacity pressure, cancellation and revision changes; count visits for a wide/skewed frontier. Delete replaced local scanning and its private fixtures without replacing canonical readiness. | open |
| F03 — Lane-owned capture and completion | Existing finite lane and selected capture contracts; compatible continuation ownership | Move selected capture/close and mandatory producer completion into each lane (AF11). Verify another lane progresses while capture stalls, failure after the last chunk prevents success, cancellation drains, unique-successor reuse survives and fan-out retains only eligible seeds. Delete coordinator-side capture exhaustion and full-run retention made unnecessary by the settled product. | open |
| F04 — Immutable admission sharing | I28 outcome; available immutable product/validity slice from 48a–48c | If adopted, share completely keyed immutable preparation/in-flight work with private lane permissions, values, seeds and provider workers. Check equal-basis reuse, changed provider/policy/structure and concurrent cancellation without authority leakage. Remove repeated construction and any superseded cache at the same scope; a no-adopt outcome records its observed reason and trigger at the coordinator. | open |
| F05 — Shared bulk and remaining durable crossings | F01 compatibility; I29 outcome; existing claim/fence/ACK contracts | Share dispatch and correct successive-study preparation keys to exclude only nonconsumed presentation; retain demand/receiving dependencies. Remove demonstrated discovery/claim crossings and select push hints only if justified. Check presentation hit/demand miss, per-member lease/start/ACK, claimed-but-unstarted bounds, reconnect/cleanup and repair/refill/warm reuse. Delete replaced scalar loops/subscriptions only as their consumers migrate. | open |
| F06 — Conditional finite commit stage | F03 completion ownership, F05 crossing assessment, observed publication blocking; I30 outcome and accepted RC12 contract route | Adopt separation only with bounded products/bytes and an explicit terminal/ACK state transition. Test backpressure, cancellation with queued products, lease expiry, failed seal, lost ACK and exact uncertainty settlement; preserve actual source/native drain. Otherwise retain joined completion and record a no-adopt outcome with the blocking trigger. Remove the replaced joined path only if the selected route supplies all its obligations. | open |
| F07 — Producer bounds, growth and stack ownership | I31 producer-specific outcome; actual native/resident lifetime and growth premises | Extend unsupported producer bounds and pre-allocation growth; charge actual teams/stacks without omission or duplicate charge. Test overflow, denied growth before allocation, nested teams, pressure while idle and cancellation/destruction. Remove opaque prepayment only where its complete replacement is established; retain conservative coverage elsewhere. | open |
| F08 — Remaining placement and native concurrency | F03 work removal; settled I28/F04 sharing decision and actual relevant products, including a no-adopt outcome; F07 applicable capacity; I32/I33 outcomes and accepted RC10/RC09 contract routes where applicable | Select only justified remaining spawn/pool/worker placement and a concrete native isolated-concurrency route. Verify all context propagation, owner-thread `!Send` teardown, actual joins, reset/failure discard, exclusions and bounded IPC where applicable. Preserve existing lanes/teams; remove only churn or placement machinery actually replaced by an adopted route. | open |
| F09 — Indexed quality-row recovery | Original contract IDs and normalization; existing subset/error meaning | Build/reuse a row-ID-to-ordinal map or establish complete positional correspondence once (AF17). Independently check reordered/subset rows, nonuniform scales, unknown IDs and duplicate/invalid contracts under existing admission. Delete the per-quality-row linear search; preserve physical values, tolerances and error behavior. | open |

F01, F02, F03 and F09 can deliver independently where shared edit surfaces allow.
F04 design can begin from existing products without waiting for every preparation
optimization, but adoption needs their actual validity/ownership contract. F05 need
not wait for F06, and F06 is not a prerequisite to useful bulk dispatch. Resource
ownership work accompanies any adopted concurrency; F07 is not permission to lower
opaque reservations before a bound is available. F08 should examine residual work
after the simpler corrections rather than use a generic pool as their foundation.

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

**Proposed; not implemented.** Product execution and qualification remain unstarted.
Record the delivered behavior and its evidence here after implementation; this
document does not itself authorize production changes.

### A mistake made and corrected

**Proposed; not implemented.** Record an actual implementation mistake and its
correction when execution supplies that evidence.

### Deviations from the plan, deliberate

**Proposed; not implemented.** Record deliberate implementation deviations and their
reasons after execution. Inquiry outcomes, rule decisions and assembled qualification
remain with the coordinator and their named delivery owners.
