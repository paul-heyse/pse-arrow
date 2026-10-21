---
id: ADR-0205
title: Retain only completely reset compatible IDAS storage
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-07, PS-07, PS-12]
blueprint: [§18.8, §18.10]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A supported fixed-profile reset lacks complete native state or retained-storage coverage, or measured complete-operation work does not justify retention.
verification: Plan48e E01 compares fresh and retained value/forward requests at engineering accuracy across changed parameters, events, schedules, failures, cancellation and native controls; verifies pointer rebinding, independent preallocation leases, owner-thread destruction and no retained completed Execution.
---

# ADR-0205: Retain only completely reset compatible IDAS storage

## Context

An independent dynamics evaluation currently creates and destroys its IDAS session, including
vectors, linear solver and SUNDIALS context. `IDAReInit` resets integration history but preserves
other native state and callback pointers. Reusing memory requires a complete supported reset
and a storage owner independent of the completed request, rather than extending its lifetime.

## Scope

Amend blueprint §18.8 and §18.10 for bounded IDAS value and forward-sensitivity storage with
an identical complete source, prepared layout and profile. Adjoint checkpoints, backward
problems and second-order directions remain fresh. This is a proposed storage lifecycle;
no numerical method, accuracy policy, default profile or factor-reuse promise changes.

## Drivers

Repeated fitting and fixed-window trials need fresh numerical premises while avoiding repeated
allocation where the library exposes a complete reset. Native objects and opaque providers
must remain on their actual owner thread. Optional retention must not retain an old execution,
consume idle CPU permission, or borrow memory authority from a later request.

## Options

Fresh construction remains the simplest valid route for unsupported configurations or unknown
storage coverage. Reusing a whole `Session` would retain its borrowed oracle and request controls;
using `IDAReInit` alone also leaves options, modes and borrowed parameter pointers unresolved.
A separate owner of neutral native allocations, rebound to a fresh callback context for each
request, localizes the extra lifecycle to the adapter and its actual dynamic worker.

## Outcome

A private owner-affine storage object may retain only native resources, their exact compatibility
identity, immutable admitted layout and an independent resident lease. A fresh context borrows
the current oracle and owns current parameters, callback state, report offsets and execution.
Compatibility binds the full profile, complete semantic coordinate/source and mode maps,
prepared support, native linear route and sensitivity population; equal dimensions do not
establish it. The current integration vector is always rebound and is never a reuse permission.
An incompatible owner is destroyed before ordinary fresh preparation.

Before the first retained allocation, obtain a detached lease proving complete current
prepaid foreign coverage through the existing `WorkAdmission` owner. A narrow optional
coverage query must distinguish a real sufficiently sized allowance from an empty generic
retention token; unknown coverage selects ordinary fresh execution before allocation.
An allowance occupied by a different live retained owner is a separate typed refusal,
not unknown coverage or permission for another fresh population under the same envelope.
The existing query returns a fallible optional lease so this distinction reaches the
adapter before Initial/provider/native effects. Current fitting and window owners establish
their actual finite population's coverage rather than relying on that refusal for scheduling.
The actual storage-aware shooting/fitting submission splits the already admitted foreign
portion of its job reservation before native dispatch and passes that detached lease
explicitly through the callable admission to the current execution. This uses the same
dispatch/join core as fresh jobs; no thread-local resource authority or second reservation
is introduced. Coverage is the actual split extent, so a larger requested foreign allowance
does not qualify. Multiple resident fixed-window workers require distinct foreign envelopes:
the original portion is assigned once, and every additional retained owner is prepaid before
child construction. A concurrently retained population cannot multiply one coverage token.
This also applies to serial fitting experiments whose cached native populations remain
resident, and to the distinct parent and team populations of exact-Hessian fitting. Required
active parallel foreign coverage belongs to actual lanes before child effects. Optional
additional resident coverage is admitted atomically for the complete actual population;
if unavailable, disable retention for that whole population and execute fresh under its
required active coverage. Do not mix one retained claimant with uncharged fresh siblings.
Prepared library-metadata or constructor allowances retain their own purpose and do not
grant native factor residency merely because their byte counts include an opaque envelope.
Parallel profile-likelihood chains are distinct active owners too. Their finite chain
coverage is established before dispatch, and each chain delegates that actual passive
coverage to its nested experiment population. Unsupported or occupied coverage is never
inferred from a concrete token type: the admission contract owns the granted extent.
Insufficient required active coverage narrows the chain team or selects serial execution
before scientific effects; insufficient optional resident coverage selects fresh sessions.
Sharing the actual prepaid allocation lease adds no pool reservation and retains no task
ledger. The lease follows the native owner through teardown and is replaced only after
current-request coverage has been established. Fresh callback/report buffers remain under
their actual current request owner. No new public resource setting or native lane pool is added.

On each compatible request, initialize the current state/rates and fresh initial sensitivities;
reinitialize IDAS, selected sensitivities, quadratures and quadrature sensitivities as applicable;
reset segment totals, transitions, counter generation and all callback failures. Rebind
`IDASetUserData` to the fresh stable callback box. `IDASetSensParams` retains the parameter
pointer but copies scale and index values into native arrays. This adapter currently uses
analytic sensitivity callbacks without calling it; the retained analytic route restores that
absence/default state rather than introducing an unused borrowed parameter buffer. Any future
parameter-pointer use must rebind it to current stable storage and detach it before retention.
Reapply current tolerances, identities, sign constraints, root functions/directions, initial
step, stop time, iteration limits and IC controls, including their removable/default states.
Root registration first removes the previous population with `IDARootInit(0, None)`,
then registers the current functions and every current direction, including zero/either
directions. A same-count registration alone retains prior native direction and active-root
state and therefore is not a complete independent-request or changed-mode reset.
Recompute consistent initial conditions using the existing policy. A reset that cannot establish
all these premises destroys the owner. Reinitialization still requires native first setup/factor;
no KLU symbolic or numerical factor retention is claimed.

### Consequences

The native allocation owner must be separated from `Session`'s borrowed boxed context. During
idle retention, native user data is detached and there is no oracle, callback box, completed
Execution, result, failure latch or active permit. The fresh callback box remains alive until
all native calls have returned and detachment or native destruction has completed.

IDAS sensitivity and quadrature-sensitivity initialization copy the user-data pointer into
separate module fields; their ReInit entry points do not rebind those fields. A stable native
resource-owned user-data slot therefore holds only a nullable pointer to the current fresh
context. Every callback decodes that slot; null is a fatal refusal. The slot never owns a
context or execution, is nulled while idle, and remains on the native owner's thread through
teardown. Base native user data is detached while idle; internal copied pointers identify
the still-live neutral slot, never a completed context. No old raw-Context callback path remains.
Unsupported adjoint or higher-order calls use the existing fresh route. Retained resources are constructed,
rebound and destroyed on their owning thread; they are not made `Send` by a wrapper.

### Compensating controls

Any callback/native/reset error, cancellation or uncertain cleanup discards the native owner
while its current callback box is still valid. Reports retain actual work and native statuses;
reset is not convergence and grants no new evaluation, iteration or clock allowance. Retention
is optional and bounded by the existing dynamic worker population. Pressure or incompatible
replacement releases the resident lease only after native destruction.

### Confirmation

**Tested, focused:** Plan48e E01's native-local/store-free
`unit-native-capability-package pse-backend-native pse-backend-native/native-solvers
solver,klu,isolation,uno,petsc` selections pass the named retained value/forward, prior-context
cancellation, root replacement, Initial-stop, scheduled DAE/forward-quadrature and fresh
current-adjoint/Second controls at unchanged engineering accuracy. The 26-control selection
and subsequent 8-control selection have zero test failures; the latter has zero compiler
warnings/errors and retains the expected native zero-root diagnostic.

**Tested, actual owner lifetime:** canonical runtime
`fitting_native_foreign_population_covers_every_actual_owner_and_discards_on_cancel` passes
nested twelve-owner uniqueness, required concurrent coverage, atomic optional-pressure fresh
fallback, Exact parent/team populations, immediate native-claim retirement and exact final
job-drain equality with the same immutable preparation alive. Actual IDAS profile-chain and
fixed-window controls also pass. Its forwarding recorder distinguishes accepted constructor
reservation growth, intentionally held by the enclosing live job, from native residency.
No byte tolerance or rebased baseline is used. Plan48e records the exact selection outcomes,
including the separate corrected actual-child curvature-refusal/output-atomicity control,
whose exact canonical rerun passes 1/1 with zero compiler warnings/errors against zero.

These controls qualify the exercised fixed-profile value/forward subset, not full-session,
backward/BBD retention, factor reuse, arbitrary providers or a measured storage speedup.
Formal decision status remains proposed; assembled qualification belongs to Plan48h.

## Pros and cons

Compatible requests can reuse library allocations without carrying scientific history across
independent trials. The native/context split and exhaustive reset add adapter complexity;
ordinary fresh construction remains necessary wherever those obligations are not established.

## More information

- [Plan48e](../plans/48e-dynamics-fitting-and-preconditioning.md)
- Vendored SUNDIALS IDAS 7.1.1: `idas.c` (`IDAReInit`), `idas_io.c`, sensitivity and quadrature reset entry points; `idaa.c` (`IDAAdjReInit`) retains separate backward ownership.

## Status history

- 2026-10-20 — Proposed before production retention; the bounded reset/API design requires coordinator review and focused qualification.

- 2026-10-20 — Updated focused tested evidence and qualification limits from Plan48e; formal status remains proposed.
