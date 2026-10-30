---
id: ADR-0234
title: Use synchronous foreground and explicitly detached service ownership
status: proposed
date: 2026-10-30
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-15, DP-19]
blueprint: [§21.7, §24.1]
review: docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Native attachment, caller-death settlement or nesting cannot preserve the original aggregate allocation under the selected manager interfaces.
verification: Plan59 B01/B03 foreground death, detached cancel/wrapper death, raw-byte input, finite aggregate admission, native incarnation and actual drain tests; source feasibility alone is insufficient.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/59b-runs-and-continuity.md]
---

# ADR-0234: Use synchronous foreground and explicitly detached service ownership

## Context

Plan59's operator-confirmed CLI/test/runtime review removes generic foreground reconnection,
independent nested CLI subtree control and automatic host-generation collection. The earlier
universal service controller plus external payload groups paid recovery costs for those features.

## Scope

Plan59 A09 replaces this proposed record's earlier topology. Preserve actual native admission,
raw command bytes, cancellation, child/cgroup settlement and scientific/fixture ownership.
This publication is Proposed, not implementation or formal ADR acceptance.

## Drivers

Use the smallest physical owner matching the requested lifetime. Native test selection and
completed reports remain separate from OS containment and truthful incomplete observations.

## Options

Keeping universal services preserves foreground reconnection at the cost of origin watchers
and external payload recovery. Making every command persistent surprises foreground callers.
Select existing synchronous native foreground ownership and one service for explicit detach.

## Outcome

Foreground uses the native caller-bound owner through child wait, required capture and
settlement. It has no generic crash-written finalizer/reconnection promise. Detached work
uses one transient user service containing a fixed Rust wrapper, its payload and ordinary
descendants. Private UnixCommand transport preserves raw argv/cwd/environment. Descendants
inherit containment before payload effects; external payload relocation is not selected.

Ordinary nested work inherits authenticated enclosing ownership and aggregate allocation.
Independent detach requires admitted placement/lifetime; it cannot escape finite constraints.
Product scientific cancellation and separately owned fixture services retain their real owners.

Systemd owns forced termination. Rust owns cooperative cancellation, child reap, required
stream drain and performing results. Select notify readiness, Restart=no and KillMode=mixed;
configure startup/stop timeouts explicitly. Qualify actual signal, finite sharing, wrapper-death
and drain behavior. Observation timeout does not stop work. A minimal detached ExecStopPost
record can report crash/incomplete, never invent missing child/capture/fixture evidence.

Use actual joined futures or actual graceful-shutdown subsystems, not overlapping task owners.
Async cancellation does not prove process termination. Native report outcome and current native
unit/drain observations remain distinct; no complete pass follows missing required evidence.

Immutable prepared generations are retained without automatic collection. Remove collection-only
claims/guards after separating executable authentication, provenance and actual resource duties.
Existing unknown records/artifacts stay untouched; this is not deletion or settlement authority.

### Consequences

Remove foreground origin watchers and generic nested subgroup/ancestry/external recovery once
callers migrate. Keep raw scopes, actual unit authentication, output and finite ownership.

### Compensating controls

Original operation failure and every independent cleanup failure remain visible. Unknown
settlement remains unknown; missing original process evidence is never reconstructed as success.

### Confirmation

Plan59 B01/B03 targeted foreground death, detached observer exit/cancel, raw-byte, finite,
wrapper crash, native incarnation and broken-source control journeys establish implementation.
Source/API feasibility and this publication do not qualify the replacement topology.

## Pros and cons

Fewer optional physical owners and recovery obligations; intentional loss of generic foreground
continuity and independently cancellable nested CLI subtrees. Explicit detach retains continuity.

## More information

- [CLI/test/runtime opportunities](../functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md)
- [Plan59 execution owners](../plans/59b-runs-and-continuity.md)
- [systemd service semantics](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html)

## Status history

- 2026-10-30 — Proposed kernel binding during authorized Plan59 execution after source-grounded
  independent advice identified nested caller-death and launch/settlement races.
- That earlier advice inspected source `3b1e97490` for the former topology; it does not qualify
  the replacement. The current functional review and operator-confirmed RT02/RT03 supply its
  target basis, with native implementation and formal status separately owned.

- 2026-10-30 — A09 publishes operator-confirmed RT02/RT03, replacing the earlier proposed topology.
