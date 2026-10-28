---
id: ADR-0233
title: Own native host control and execution boundaries in Rust
status: proposed
date: 2026-10-28
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, AP-07, DP-15, DP-19]
blueprint: [§3.3, §20.6, §21, §24.1, §24.2]
review: docs/design_review/reviews/design_review_rust-owned-host-and-boundaries_2026-10-28.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A named retained workload cannot preserve original physical ownership or canonical admission through the selected native interfaces.
verification: Plan58 B01–B05 targeted cold-bootstrap, actual systemd255 user-manager ownership/drain, environment and native compiler controls; D05 separately establishes transactional migration fencing.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/58b-bootstrap-and-host-control.md, docs/plans/58d-canonical-control-and-connectivity.md]
---

# ADR-0233: Own native host control and execution boundaries in Rust

## Context

Plan58 replaces first-party Python operating and product policy with Rust ownership.
The operator selected integrated cutovers, optional CLIs/MCP, deferred Python bindings,
scientific rather than build observation, typed fixture ownership and balanced durability.
The current host boundary imports Python environment, allocation, lifetime, native setup
and database supervisors. A thin launcher replacement would retain that second system.
The functional/improvement reviews motivate the target but do not supply formal acceptance.

## Scope

Add a lean `pse-host` library/executable for Linux host/environment ownership. Its normal,
build and test closure excludes Python, Arrow/DataFusion, science, database clients and
product-schema generation. Reuse `pse-diagnostics` process facts; no heavy testkit dependency.
Amend the mandatory Python boundary of blueprint §21 toward native first-party operations.
This is a target direction: existing Python scientific callers remain until their owning
Plan58 cutovers; this ADR does not claim them removed. Retain accepted native execution,
Arrow and scientific rationale rather than mass-supersede unrelated records. Proposed
ADR-0164/0214/0228/0229/0232 retain their historical rationale; Plan58 replaces their mandatory
Python-face or old host-controller requirements where applicable. Formal status is unchanged.

## Drivers

Cold construction and diagnosis must not need the system they diagnose. Original workload
exit and physical release must remain truthful across process death, PID reuse, manager signal
loss and partial launch. Canonical data must govern durable execution without owning OS joins.
Useful pure computations and tests must remain independent of a database. Configuration and
fixtures should carry actual capabilities rather than a parallel permissions catalogue.

## Options

A Python launcher proxy or a renamed Rust ledger preserves duplicate policy. Existing xtask
has unconditional product/schema/Arrow dependencies and is unsuitable for cold construction.
A resident admission daemon adds IPC and restart obligations without a named need. Select a
small Rust root plus ordinary OS-manager containment and explicit foreground ownership.
Retain heavy development operations under their actual owners, not inside the host crate.

## Outcome

### Responsibilities and interfaces

`pse-host` owns environment selection, user-manager unit operations and physical process/cgroup
facts. Owned unit capabilities permit stop; borrowed observations do not. Their constructors
bind boot/process/unit/invocation/cgroup identity. Unknown ownership refuses destructive action.
A successful job reply or leader exit alone cannot establish empty descendants. Observe original
unit state and cgroup population before release. Reconnect resolves uncertain launch identity;
it never automatically repeats a launch. Read-only activity bypasses workload admission.

Use exact zbus6.1.0 and zbus_systemd0.26201.0 Tokio client/systemd1 features, qualifying used
methods on systemd255. Move Tokio exactly to1.53.3 without an unrelated lockfile upgrade.
Existing process-group supervision remains useful but cannot prove setsid-descendant drain.
No sd-notify, hierarchical shutdown library or new daemon is required absent an actual consumer.

A shell seed explicitly builds the locked host package with the pinned toolchain, bypassing
Python compiler wrappers for that invocation. It privately copies a completed executable and
atomically replaces checkout-local selection. Ordinary help/read/explain never rebuilds or
rescans source. Running processes retain their executable. Host errors use concrete thiserror
and miette diagnostics; original child exits/signals remain separate from boundary failure.

Environment precedence stays command, caller, personal local selection, repository defaults.
Explicit empty values and secrets retain meaning. Child environments are constructed explicitly;
no process-global mutation after worker threads start. Configured unbounded ordinary work keeps
all existing physical/logical parallelism; explicit finite scientific demands and selected
reference/timing exclusions remain. Unnecessary ordinary slot/pressure ledgers retire.

C owns actual native work and joins. D owns schema applicability, canonical transitions,
authentication/write admission and migrations. B establishes the service process, not readiness.
D construction establishes supported version/issuer premises; actual writes recheck mutable
canonical fences rather than trust a permanently valid client handle. Migration closes new
claims/mutations, permits narrowly scoped original-attempt settlement under the old interpretation,
then drains writers/native work before transforming. Unknown drain leaves admission closed.
Successor activation fences stale handles and reconstructs capabilities. The host crate imports
no DB types: application composition connects it to D. D05 owns working migration infrastructure
and its races; a configuration file check is not transactional fencing.

F selects native compiler/cache tools once per setup/build. Native sccache client-side compilation
keeps compilers in the workload; qualify actual pinned logging/distribution behavior. Remove the
Python per-compiler identity watcher, hashes and ledger. Cache-service death alone does not
invalidate a successful native tool/compiler result; actual errors remain errors, without a new
blind retry layer. Bespoke build-stage/freshness/lock telemetry and derivative signals retire.
Retain scientific progress, diagnostics and optional native telemetry configuration. Collector
removal remains a separate decision; no equivalence to its queue/failure signals is claimed.

Fixtures receive real private-construction owned/borrowed capabilities. Test names/nextest groups
confer no authority. Disposable fixtures clean up on failed assertions too; preserve useful
diagnostics outside the store, and never infer test success from Drop. External authentication,
process death and shared-service interference remain actual runtime uncertainty. New fixture
placement can be disposable; this authorizes no existing-store deletion. Minimal native schema
migration infrastructure remains required now, with full historical migration/backup/disaster
recovery qualification before non-regenerable or shared operational deployment.

### Consequences

Cutovers contain native owner, real callers/configuration/tests and obsolete mechanism deletion.
Intermediate development paths may be unavailable with no acceptance credit. No compatibility
DTO, Python fallback, FastMCP proxy, universal operation catalogue or mandatory foreign ABI is
introduced. Other Plan58 packages retain their unimplemented scope. Plans53/54 retain original
evidence and unrelated remaining work; changed host/fixture/cache assumptions route here.

### Compensating controls

Use targeted construction, protocol and kernel-fact tests plus real uniquely owned disposable
user-manager journeys. Independently inspect physical state. Pure host correctness tests select
no unrelated Arrow roots; actual Arrow-consuming closures keep force-validation. No global CPU
caps, serial independent work, data resets or numerical speed claims follow.

### Confirmation

The scoped combined formal review must establish architectural acceptance before dependent code.
B01–B05 supply implementation evidence; D05 and H retain their separate acceptance obligations.
This proposed ADR changes no accepted-record status; the prescribed ADR PR route owns that.

## Pros and cons

Native ownership removes interpreter crossings and duplicate operational models. The new root
adds a crate and requires integrated caller migration. Unit/cgroup uncertainty remains, and a
Rust type cannot prove an independent process alive. Server readiness/migration must remain a
separate capability rather than inflate the bootstrap closure.

## More information

[Plan58](../plans/58-rust-owned-system.md), [58b](../plans/58b-bootstrap-and-host-control.md),
and [hard-cutover recommendations I01–I14](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md)
own planned scope/disposition. The design review links the refreshed scoped audit and independently
assesses the target under the selected standard.

## Status history

- 2026-10-28: Proposed for authorized Plan58 T01/T02 and native host execution. No implementation
  or formal ADR acceptance is established by this record.
