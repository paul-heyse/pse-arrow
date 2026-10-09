---
title: Host admission and timing qualification
status: done
date: 2026-10-08
adrs: [ADR-0166]
review_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#4-revealing-scenarios]
---

# 30d: Host admission and timing qualification

## Responsibility and foundation

This companion owns WP07/WP08's host coordination and timing interpretation, and the
series's selected scope-end assembled acceptance. [Plan 30](30-websocket-and-persistent-agent-environment.md#finding-dispositions)
owns finding dispositions. Extend `pse_env` and `surreal_server`; systemd enforces
placement/caps and existing supervised ownership establishes drain. Fixed lanes and
short admission bookkeeping suffice; no general queue, priority service, remote
scheduler or new scientific resource estimator is needed.

**Interface-checked:** per-command ceilings and nextest groups are invocation-local;
the existing reference profile has a finite 160 GiB envelope, but unrelated invocations
are not coordinated. Ceilings, weights and CPU masks do not reserve physical RAM or
exclude arbitrary other projects. The machine-specific design must state that limit.

Read-only `lscpu -e=CPU,CORE,SOCKET,NODE,CACHE` during authoring identified physical
cores 0–7 sharing L3 index 0 and cores 8–15 sharing index 1, with SMT siblings 16–31.
This supplies a useful two-lane placement, not measured cache or timing quality.
Validate the actual topology again when materializing the host profile.

## D0 — Concrete initial profiles

Use these initial **Proposed** profiles under the existing user-manager boundary:

| Mode | Aggregate memory ceiling and partition | Physical CPU ownership |
|---|---|---|
| Functional | 96 GiB: 8 GiB persistent store, two 40 GiB heavy slots, 8 GiB aggregate light work | Physical cores 8–15, including their SMT siblings for placement; each normal heavy slot gets four disjoint physical cores. |
| Timing | 64 GiB: 8 GiB dedicated store, one 56 GiB execution/control slot | Physical cores 0–7, including their SMT siblings; no functional job uses this set. |
| Reference | Unchanged 160 GiB: 16 GiB store, 140 GiB primary, 4 GiB observer; original 128 GiB pool, 16 lanes and 32 population tickets | All sixteen physical cores, exclusive of other cooperating heavy work. |

Normal lanes compose under a 160 GiB aggregate parent. Their nominal remaining
28 GiB on this host is allowance for OS/editor/other activity, not reserved or
guaranteed available memory. Shared storage/memory bandwidth and unmanaged SMT users
can still contend. Reference measurements do not become equivalent to the smaller
timing profile merely because both have an exclusive lane.

Initial command classes are light (2 GiB per command, four slots under the 8 GiB
light parent), compile (24 GiB within one heavy slot), native functional (32 GiB
within one heavy slot), wide functional (80 GiB consuming both heavy slots), timing,
and reference. Known commands select the appropriate class at the environment root;
unknown heavy commands select one bounded heavy slot rather than a light class.

For a managed native functional slot, initially allocate 32 GiB receiver, 4 GiB
observer and 4 GiB control/headroom, with a 24 GiB runtime pool, two case lanes,
four CPU permits, 8 GiB per-worker capacity, four population tickets and one
construction core. The timing slot initially allocates 48 GiB receiver, 4 GiB
observer and 4 GiB control/headroom, with a 40 GiB pool, four case lanes, eight
CPU permits, 8 GiB per-worker capacity, eight population tickets and one construction
core. These are separately declared profiles; their pools are capacities, not RSS.
Their class does not change authored tolerances, point counts or solver requirements.
Tests/campaigns requiring the reference declaration select reference mode unchanged.

D0 checks the materialized runtime/scope relationships before adoption. Adjust a
class when bounded evidence reveals an inadequate cap; update its one owning
declaration and focused consumers. Do not make a supported workload smaller to pass
a resource check or silently rename a changed profile as the reference profile.

## Admission, overrides and actual remaining work

Use a short `flock`-protected local allocation decision with fixed lane/slot records
bound to invocation nonce, PID/start generation and supervised unit/cgroup identity.
The lock protects metadata changes, not the workload lifetime. Existing live cgroups
and admitted service ownership survive an agent crash; an empty stale file does not
permanently exclude work. On restart, reconcile live units before granting capacity.

Nested recipes and subprocesses inherit the enclosing owner and original admission
clock. Independent jobs acquire their own slot. Role caps and runtime pools consume
their enclosing allocation; a parent and its children are not charged twice as
independent full profiles. Conversely, separately launched worker/observer units must
be charged to the originating slot even if outside its caller's descendant tree.

D0 materializes outer runner concurrency and per-process allocations inside the
selected class. Independent native processes needing a full receiver profile obtain
their own admitted allocation; inherited owner metadata cannot multiply process-local
permits or memory pools within one slot. Smaller pure tests need not be serialized.

Keep the slot occupied until every owned supervised descendant drains. Parent exit,
caller timeout, PID absence and unlocked metadata are insufficient. Persist failure
and evidence pins through capacity release; resource availability and disposal
eligibility are different operations.

Retain `PSE_MEMORY_MAX` as a requested finite cap. Requests above the current class
select a wider compatible class: up to 40 GiB uses one heavy slot, up to 80 GiB uses
both, and larger functional requests require exclusive heavy-work admission. The
initial exclusive functional allocation is 160 GiB (8 GiB store, 8 GiB light/control,
up to 144 GiB execution), using all sixteen physical cores. Managed execution divides
that 144 GiB into a 136 GiB receiver, 4 GiB observer and 4 GiB launch/control. Its pool
is 128 GiB, with sixteen CPU permits, up to sixteen case lanes, 32 population tickets
and one construction core; per-worker capacity remains 16 GiB unless the consumer
declares another justified profile. A larger requested receiver cap includes the
observer/control roles in the aggregate before admission. This is a named exclusive
functional profile. Reference mode retains its exact partition.

The maintainer has already authorized raising insufficient constraints. A diagnosed
cap failure widens the class/ancestor allocation and reruns the failing identities;
it does not trigger a full suite restart. A larger explicit exclusive request may
raise the aggregate parent after validating role sums, effective ancestor limits and
at least 16 GiB nominal non-agent headroom against actual physical RAM. Record the
changed declared profile; never silently defeat the request with an old ancestor cap.
Unbounded `off`/`infinity` is not supported in coordinated heavy mode. A request beyond
physical/effective capacity receives an explicit refusal and capacity explanation.

Initially require at least 36 GiB `MemAvailable` before admitting new heavy work:
the normal nominal 28 GiB allowance plus 8 GiB startup margin. This **Proposed pressure
guard** is verified in D1/D2 and changes with an explicitly widened host profile;
it is not a promise that all admitted jobs can reach their ceilings simultaneously.
Do not require instantaneous free memory to equal the reference ceiling, and do not
mislabel a snapshot as a reservation. Record admitted resident services and unmanaged
host pressure. Continue existing actual-use observations without adding an estimator.

Inventory existing services before adoption. Externally selected/unknown services
are not silently stopped, absorbed into the lane or ignored as load. For every
exclusive mode, drain competing cooperating heavy work and park only explicitly
owned services whose lifecycle permits it, preserving disk and pins. Any service
left resident is charged inside the selected aggregate envelope. B1 startup/restart
admission prevents a parked service from relaunching into that window. Preserved
external load that prevents the requested conditions produces a bounded wait/refusal.
Wait within the original admission budget; no renewed nested clock or durable job queue.

Expose selected class, parent/role limits, physical-core sets, current owners, pressure
and blocking reason through the existing explain/activity/status surfaces. Light
reading/editing can continue during reference mode. Agent command prefixes and
recipe wrappers compose idempotently rather than allocating again.

## Timing meaning and benefit investigations

Record service generation, receiver/artifact identity, invocation/context, profile,
placement, competing load and each relevant residency state: server process, RocksDB
cache, filesystem cache, receiver, imported extension and compiler/prepared products.
Also state which provisioning/startup/preparation is included in the timed interval.
Fresh process, fresh database and cold filesystem are separate facts.

Timing mode owns its service/context and CPU lane. Functional fixture creation,
disposal and publication use the functional lane; their shared-device contention is
still observable. Require declared isolation conditions or label the measurement
contended. Full reference timing requires the exclusive heavy-work window. An unmanaged
other-project job is neither silently controlled nor evidence of an uncontended run.

Report service-lifetime RSS/cache observations separately from client/case values.
Do not subtract an unexplained baseline and call it case memory, or assign a long-lived
service high-water mark to one case. Separate native build, preparation, numerical
execution, publication and result serving when reporting benefit. No fixed latency
SLA or mandatory whole-product benchmark matrix is added.

The optional benefit investigation compares representative edit/re-solve, study and
reopen journeys with equal scientific checks and declared cold/warm state. Its trigger
is positive D3 functional evidence plus authorized measurement, or a specific existing
time-budget failure needing diagnosis. Existing historical performance receipts keep
their conditions. Do not claim that WebSocket cured the HTTP/2 failure without the
same failing journey's new evidence.

## Packages and scope-end acceptance

| Package | Inputs and delivered behavior | Migration/deletion and focused acceptance | Status |
|---|---|---|---|
| D0 — Host/profile materialization | R0 rules, current topology/effective limits and B0 role identity. Materialize the profiles above and command-class routing. | Verify role sums, allowed physical cores/SMT sets, runtime/ancestor compatibility, reference invariance and widened-cap routes. No machine-wide stress campaign. | Implemented; focused controls and selected D3 handoff Tested |
| D1 — Working cross-invocation admission | D0; existing environment/supervisor scope owners. Implement fixed-slot selection, reentrant nested use, pressure check and liveness reconciliation. | Migrate command prefixes, ordinary Rust/Python/native wrappers, service/receiver launches and measurement roots. Test simultaneous independent callers, original-clock timeout, nested calls, parent death, surviving units/PID reuse and effective-limit refusal. Delete replaced independent per-run admission assumptions and empty-lock exclusion. | Implemented; focused controls and selected D3 handoff Tested |
| D2 — Actual functional/timing composition | D1; working A2/B2/C2. Enforce role placement and timing residency/attribution. | Bounded overlapping representative builds/tests, heavy cap escalation, timing exclusion, reference exclusive admission, service restart and cancellation through actual drain. Pure metadata mocks do not qualify real kernel placement. Delete duplicated placement interpretation and stale budget defaults. | Implemented; focused controls and selected D3 handoff Tested |
| D3 — Selected assembled qualification | All functional A/B/C/D packages and necessary consumer migration/deletion complete. Run the new-scope integrated checks once, with shared terminal/outcome composition. | Real native WS exact reads/writes and unknown acknowledgment; concurrent Rust/Python isolation; compatible receiver coexistence; failed/incomplete pins and referenced cleanup; native interruption/drain; representative edit/re-solve, ordinary/managed study and retained reopen. Scope-end hygiene/manual checks, fix/rerun failures, then documentation handoff. | Implemented; focused controls and selected D3 handoff Tested |

Use owner-local `pse_env`, supervisor, execution-contract and receipt tests first,
plus targeted affected Rust/Python controls with force-validation. Real-placement
controls use owned small inputs and harmless descendants, not memory-exhausting the
machine. At D3 select applicable component/integration/solver/Python journeys and
required static/manual checks; report their actual scope and exclusions against zero.

D3 owns this series's assembled environment evidence. 28e remains the owner of the
existing failing managed identity and the broader substrate/scientific campaign.
Shared evidence is linked with original conditions rather than duplicated or promoted
to whole-simulator qualification. Repair/rerun old failed identities and finish old
interrupted identities; no fresh former full selection or E4/E5 follows from this plan.

The existing managed thousand-point Python identity is the relevant transport-adoption
journey once its new-scope execution is authorized; small queries are not a substitute
for its scientific, activation and publication behavior. Run it under the unchanged
reference profile and attach the actual receipt to 28e's failing-identity owner as well
as this new-scope handoff. That selected rerun is not a rerun of the old full Python suite
and does not establish that HTTP/2 was the original root cause.

## Verification and checkpoint

**Proposed:** fixed lanes prevent cooperating invocations from each assuming the
whole declared host allowance, preserve actual drain charges and make timing conditions
discoverable. Unmanaged host activity, shared-device interference and configured
ceilings remain explicit limits. D1/D2 settle the initial profile sizes and pressure
guard with bounded evidence before any performance/safety claim.

**Implemented:** the declared profiles, finite host admission, inherited allocations,
service borrowing and independent compiler-cache lifetime now replace independent
per-command assumptions. **Tested:** the bounded controls below establish selected
actual placement and lifetime behavior against a zero failure baseline. They do not
establish scientific composition, performance improvement or machine-wide safety;
D3 is complete for the selected authorized scope; Plan 30 owns finding dispositions and the repaired composite Outcome.

## Implementation checkpoint — 2026-10-09

The single host profile declaration and finite allocation ledger now coordinate lanes, independent callers and inherited owners. Actual kernel controls establish disjoint affinity, finite caps, nested reuse, survivor charges after parent death, bounded refusal and release after drain. Ordinary scientific runner processes share one nextest group; pure selections retain parallelism.

An actual widened-profile control exposed a shared cache daemon surviving inside a
completed compile scope. Its replacement gives the resident daemon an independent
finite 2 GiB light allocation, verifies private socket and process identity, scrubs
caller-specific environment and keeps compiler children in their caller's allocation.
The actual cache lifecycle control records that separation in
`build/plan30-cache-lifecycle-20261009T071814Z/evidence.json`; thirteen focused cache
controls also passed. Service identity and client-group containment address the pinned
client's lazy-start behavior; a transient fork or cache effect remains possible. Cache
storage is preserved. These controls do not measure compiler throughput or latency.

The failed capacity control was repaired and its affected selection passed: actual
80 GiB wide and 56 GiB timing callers overlap with disjoint declared affinities, finite
memory caps and nested allocation reuse. A conflicting unchanged-reference request
refuses after its original 30-second admission budget. The selected actual control
passed in 30.984 seconds, with its result in
`build/plan30-d2-capacity-modes-20261009-3/result.json`. Five focused nested-capacity
controls cover light/compile affinity inheritance; one focused service-borrowing
control covers preserving intentionally stopped services instead of parking/resuming
them. These are placement and lifecycle observations, not performance qualification.

The selected reference startup failure exposed a physical-core versus SMT-affinity
mismatch in storage launch. Execution storage now uses its declared physical role
cores, while codec-only storage retains the host profile's affinity. Explicit reference
demand now starts the actual owning service before observer admission, retaining
restored validation and readiness refusal. A focused prelaunch control and four
focused reference-demand controls passed, and the preserved service reached actual native
WebSocket readiness under unchanged 160 GiB aggregate/16 GiB storage capacities.
`build/plan30-reference-affinity-20261009/passed.json` records the frozen generation
and invocation, subsequent actual drain and the controller evidence-rendering error
recovered from the persisted readiness receipt without repeating startup. This does
not qualify the scientific journeys or recovery policy of that generation.

D3 resolved the selected native/Python failures through affected-only retries. The original managed thousand-point Python identity passed under the unchanged reference allocation. Scope-end integration and documentation handoff are complete; wider Plan 28 campaigns remain paused.

## Outcome (recorded after implementation)

**Implemented:** this packet's target mechanisms and required consumer migration are
complete. **Tested:** the focused controls above and the selected assembled D3 journeys
passed against the zero failure baseline under their recorded conditions. The
[series Outcome](30-websocket-and-persistent-agent-environment.md#outcome-recorded-after-implementation)
owns the repaired composite results, commands, current storage restart proof and
qualification exclusions. Earlier failed receipts retain their original outcome;
there was no restart of the former full Python suite or broader Plan 28 campaign.

A mistake made and corrected, and deliberate deviations, are recorded at that same
series Outcome with their owning repair. Enduring contracts and operation guidance
live in blueprint §20.6/§24.1 and the substrate/environment/validation guides. This
completed handoff remains while retained Plan 28/29 readers depend on it; it is not an
active implementation backlog. Benefit measurement and the conditional topology
investigation retain their explicit authorization and observable triggers.
