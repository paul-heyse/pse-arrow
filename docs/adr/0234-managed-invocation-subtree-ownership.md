---
id: ADR-0234
title: Unify managed invocation ownership under native service supervisors
status: proposed
date: 2026-10-30
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-15, DP-19]
blueprint: [§21.7, §24.1]
review: "not-required: bounded kernel binding within ADR-0233; independent read-only architecture advice inspected source3b1e97490 and systemd255.4/kernel contracts; not formal review acceptance"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Native attachment, caller-death settlement or nesting cannot preserve the original aggregate allocation under the selected manager interfaces.
verification: Plan59 B01/B03 actual service-entry, origin/supervisor death, nested launch/cancel race, escaped descendant drain, original-root replacement and borrowed finite admission tests; source feasibility alone is insufficient.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/59b-runs-and-continuity.md]
---

# ADR-0234: Unify managed invocation ownership under native service supervisors

## Context

Plan59 managed invocations need a surviving owner for output, result publication and full
descendant settlement. The current raw nested adapter executes its payload in place; adding
an in-process tee would leave nested descendants without an invocation cleanup owner if that
controller receives SIGKILL while the enclosing workload continues.

## Scope

Select the B01/B03 managed invocation realization within ADR-0233. Raw boundary/correctness
protocols keep their existing exec semantics. No daemon, scientific result authority or new
finite capacity cap is introduced.

## Drivers

One implementation should own foreground/detached payloads, keep cancellation effective under
nesting and retain the exact enclosing aggregate allocation. Controller death, child reap,
lost pipes and manager job replies cannot substitute for physical payload settlement.

## Options

Process groups miss escaped descendants. Stopping the enclosing root kills unrelated work.
Sibling scopes with copied limits bypass the shared aggregate budget. A second foreground
guardian would duplicate B03's service supervisor/finalizer. Select that existing planned
service owner for both managed modes, with core-only descendant cgroups where nested payloads
must remain under an established workload root. This introduces a service launch for managed
work; cheap source/help/status reads remain direct. No performance claim is made.

## Outcome

Foreground and detached managed invocations share a transient user-service supervisor and
`ExecStopPost` finalizer. Foreground retains an authenticated origin pidfd/start-identity lease
established before releasing the payload gate; origin death cancels its invocation. Detached
explicitly omits that lease after acknowledged launch. The supervisor stays in its service
cgroup; invoking current `run_owned` unchanged would move it into a scope and violate that
ownership. Placement/admission are supplied by their shared owners.

Keep immutable original systemd root, actual current member and owned invocation subgroup as
distinct capabilities. Create nested work beneath the authenticated current invocation/member,
so cancelling A contains B. Authenticate opened directory/component/inode, boot/namespace and
original unit/invocation identity; a string prefix grants no authority. Use systemd255's
`DelegateControllers=[]` alone for core hierarchy delegation. Keep descendant `subtree_control`
empty and preserve original aggregate controls. A nested service borrows the original
reservation/root; it cannot stop that root or release its reservation.

### Consequences

Service `KillMode=control-group` does not cover a payload moved to an external subgroup.
Supervisor/finalizer must explicitly authenticate, kill, recursively drain and remove that
subgroup, retaining independent failures. Close nested-launch authority under the same short
launch/settlement serialization before drain; no surviving external supervisor may repopulate
a parent after its empty observation. Revalidate parent ownership before attachment and gate
release. Gated unreaped child/PID custody and actual membership validation precede execution.
Nested detachment survives its launching client only within the enclosing owner's lifetime.

### Compensating controls

Qualify actual cross-group attachment and finalizer permissions; do not fall back to sibling
allocation or leader-only settlement. Missing core delegation refuses this managed route.
Original `OwnedUnit` recursive drain remains authoritative for its root. Core-only child
groups lack per-invocation memory-controller accounting: expose root readings as aggregate or
report unavailable. Service control overhead remains outside the payload leaf but under
ordinary host policy; never label it payload-exclusive accounting. Required capture, tool
pins and fixture disposal settle independently of controller/subject outcomes.

### Confirmation

The source argument checks systemd255.4 delegation and kernel cgroup-v2 hierarchy semantics;
it establishes feasibility only. Plan59 B01/B03 own implementation and native permission,
crash, launch-race and finite-admission tests. Formal status stays proposed; no such tests or
scientific architecture review are claimed here.

## Pros and cons

One service owner avoids two foreground/detached supervision implementations and survives the
origin's death. It adds service startup and authenticated external-subgroup cleanup, including
a parent-launch/settlement race that directory FDs alone cannot settle.

## More information

- [Plan59 runs and continuity](../plans/59b-runs-and-continuity.md)
- [ADR-0233](0233-rust-owned-host-and-execution-boundaries.md)
- [systemd255.4 delegation](https://raw.githubusercontent.com/systemd/systemd-stable/v255.4/docs/CGROUP_DELEGATION.md)
- [systemd255.4 scope](https://raw.githubusercontent.com/systemd/systemd-stable/v255.4/src/core/scope.c)
- [systemd255.4 properties](https://raw.githubusercontent.com/systemd/systemd-stable/v255.4/src/core/dbus-cgroup.c)
- [Linux cgroup-v2 contract](https://docs.kernel.org/admin-guide/cgroup-v2.html)

## Status history

- 2026-10-30 — Proposed kernel binding during authorized Plan59 execution after source-grounded
  independent advice identified nested caller-death and launch/settlement races.
