---
title: Host admission and timing qualification
status: draft
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
| D0 — Host/profile materialization | R0 rules, current topology/effective limits and B0 role identity. Materialize the profiles above and command-class routing. | Verify role sums, allowed physical cores/SMT sets, runtime/ancestor compatibility, reference invariance and widened-cap routes. No machine-wide stress campaign. | Planned |
| D1 — Working cross-invocation admission | D0; existing environment/supervisor scope owners. Implement fixed-slot selection, reentrant nested use, pressure check and liveness reconciliation. | Migrate command prefixes, ordinary Rust/Python/native wrappers, service/receiver launches and measurement roots. Test simultaneous independent callers, original-clock timeout, nested calls, parent death, surviving units/PID reuse and effective-limit refusal. Delete replaced independent per-run admission assumptions and empty-lock exclusion. | Planned |
| D2 — Actual functional/timing composition | D1; working A2/B2/C2. Enforce role placement and timing residency/attribution. | Bounded overlapping representative builds/tests, heavy cap escalation, timing exclusion, reference exclusive admission, service restart and cancellation through actual drain. Pure metadata mocks do not qualify real kernel placement. Delete duplicated placement interpretation and stale budget defaults. | Planned |
| D3 — Selected assembled qualification | All functional A/B/C/D packages and necessary consumer migration/deletion complete. Run the new-scope integrated checks once, with shared terminal/outcome composition. | Real native WS exact reads/writes and unknown acknowledgment; concurrent Rust/Python isolation; compatible receiver coexistence; failed/incomplete pins and referenced cleanup; native interruption/drain; representative edit/re-solve, ordinary/managed study and retained reopen. Scope-end hygiene/manual checks, fix/rerun failures, then documentation handoff. | Planned |

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

Only read-only topology inspection and documentation authoring occurred. No resource
cap, unit, CPU placement or test campaign changed. Packages remain planned; Plan 30
owns findings; the next implementation package, once production execution is
authorized, is R0/D0.
