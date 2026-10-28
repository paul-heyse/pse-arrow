---
title: "Plan58b: Bootstrap and host control"
status: draft
date: 2026-10-28
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, environment, validation]
review_sources:
  - docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md
  - docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md
---

# Plan58b: Bootstrap and host control

This companion develops O05 and the host portion of O02/O06. The
[coordinator](58-rust-owned-system.md) owns selection, rule changes and overall acceptance;
this document owns the packages below. Creation authorizes planning only. Plan57 is superseded; its failed/interrupted evidence remains historical under its original conditions.

This revision incorporates [I01/I03–I05/I10–I12](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01). The amendment baseline is `c3b7f136d`; reviewed production source remains `bd1c4891e`. Recommendation disposition belongs to the coordinator.

## Baseline and target

The functional review's inspected baseline is `bd1c4891ea7d71178e809b11045dd4fd89602c0f`.
`scripts/pse-env`, `scripts/setup-env`, `scripts/bootstrap.sh`, the compiler-cache launcher
and `scripts/cargo-bin/cargo-pse-correctness` still enter first-party Python. Even rust-only
bootstrap invokes Python tool/configuration helpers. `xtask/Cargo.toml` has unconditional
`pse-codegen`, `pse-columnar`, `pse-schema` and Arrow dependencies; `default = []` does not
make its existing executable a small bootstrap root. These are inspected source facts,
not measurements of cold-build duration. Existing environment and lifetime contracts live
in the [environment guide](../dev/agent-environment.md) and blueprint §24.1/§24.2.

Select a dedicated small Rust host-tool root, independent of Python, scientific mathematics,
Arrow/DataFusion, database clients and the schema/codegen closure. A feature-separated xtask
would require disentangling its unconditional imports and library dependencies while preserving
its scientific development uses. The dedicated root makes bootstrap and failure diagnosis
independently constructible; it adds a crate and therefore needs an ADR and formal design
review before code. Its name and local module layout remain implementation choices. Do not import the current
DataFusion-bearing `pse-testkit` into this root to share fixtures; reuse lightweight physical
primitives or place the minimum necessary primitives below scientific dependencies.

A tiny shell seed locates the pinned Cargo/toolchain and builds only this locked package
into checkout-local Cargo output, then installs a completed executable by atomic replacement
into a checkout-local tool location. The seed implements no allocation, launch or release
policy. Construction uses ordinary Cargo directly because its own host supervisor may be
missing. For that one seed invocation, unset inherited `RUSTC_WRAPPER`/`RUSTC_WORKSPACE_WRAPPER`
that would re-enter checkout Python; a separately verified native cache may be selected explicitly.
An activated old `scripts/sccache` wrapper must not defeat no-venv construction. This is a
narrow bootstrap exception, not a second normal environment policy; it neither imports Python nor calls the tool being constructed. No alternate target
directory, wholesale lock resolution or product build follows. An explicit update reconstructs
the tool; the running executable and its children retain their original generation until exit.
Missing/stale tools produce actionable preparation instructions; ordinary reads never rebuild.

## Native manager and physical ownership

Select exact `zbus =6.1.0` and `zbus_systemd =0.26201.0`, using only the required systemd1
proxies and a Tokio connection runtime. The [library reference](../libraries/zbus_and_zbus_systemd.md),
especially its runtime, job and transient-unit sections, supplies the capability basis;
[zbus's version-six guide](https://z-galaxy.github.io/zbus/upgrading-to-6.html) supplies the
API migration context. Confirm the resolved release manifests before integration: zbus's
Tokio requirement `^1.53.3` conflicts with today's workspace `=1.53.2`. Plan the narrow exact
Tokio move to `=1.53.3` with affected callers and lock diff, rather than two runtime versions.

Connect to the actual **user** manager through `Connection::session()` or an explicitly
selected user-bus address; never silently target PID 1. Compose `ManagerProxy`, unit/service/
scope properties and `StartTransientUnit`'s typed `a(sv)` properties. Services let the manager
spawn owned workloads; scopes attach explicitly owned external processes, preserving partial
spawn/attach failure cleanup. Retain borrowed scope/service distinctions and allocation policy.
Systemd owns process containment; Rust owns necessary admission and original-result interpretation.
The baseline is an owned foreground operation with manager containment, not a resident admission
daemon. A daemon requires a named persistent coordination workload and a demonstrated reduction
in total machinery before selection. In configured unbounded development mode, delete unnecessary
slot records and pressure-policy branches rather than translate them. Explicit scientific demands,
selected reference/timing exclusions and operator finite selections retain their meaning; no new
CPU/RAM partitions or serial execution are introduced.

Create the `JobRemoved` receiver and call `Subscribe()` before submitting a job. Match its
returned job path, distinguish accepted/queued from completed/failed/canceled, and inspect the
selected unit's final state. Reconnect or signal loss requires fresh authoritative observation;
it never authorizes replay of an uncertain launch. The local host runs systemd 255 while the
bindings describe newer APIs: qualify every used method/property and authorization on 255's
user manager. Do not select newer multi-job APIs merely because the bindings expose them.

Bind boot ID, PID/start identity, manager invocation/unit identity and original cgroup identity
before consumers trust ownership. A successful StopUnit job, absent observer, pidfd exit or
transport EOF is insufficient for descendant release: observe the bound kernel cgroup's
hierarchical population and authenticate identity changes. Unavailable evidence prevents a claim of physical release and produces a useful refusal.
A finite scientific charge remains until its actual release; ordinary unbounded work does not
require recreating a general allocation ledger. Host liveness has no new canonical database store; consolidate
helper-only records but preserve distinct scientific admission, borrowed ownership, durable
study receipts and immutable artifact identity where their effects require them.

Read-only activity/ownership diagnosis remains directly executable when admission, Python,
compiler setup or the database is unavailable. Report partial observations and unknowns rather
than enter workload admission to explain why workload admission failed. Keep configured
unbounded host sharing, ordinary compiler/test parallelism and explicit finite selections.

`sd-notify` is conditional on a selected service needing application readiness beyond `Type=exec`.
If selected, `NotifyState::Ready` follows actual readiness and manager-observed acceptance;
absent `NOTIFY_SOCKET` returning success proves nothing. The supplied [0.5.0 reference](../libraries/sd-notify.md)
records abstract-socket handling and missing barrier limitations; qualify the actual endpoint
or choose a source-backed alternative. No mandatory notification/watchdog/socket-activation
dependency is scheduled. Likewise [tokio-graceful-shutdown](../libraries/tokio-graceful-shutdown.md)
is optional when real subsystem hierarchy warrants `Toplevel`, subtree failure routing and
explicit joins. Flat tasks can use existing tokens/tracking. Neither timeout nor task abort
proves a blocking native call or operating-system process drained.

## Service, fixture and observation boundaries

58b owns deployment process mechanics and the selected lifecycle/credential establishment.
[58d](58d-canonical-control-and-connectivity.md) owns schema/version readiness, migrations and
write admission. Starting a process does not establish schema applicability. Do not port the
complete Python database operator or historical recovery machinery to satisfy service launch.

A fixture gets real typed owned or borrowed capabilities with private construction and explicit
shutdown. Test names, nextest groups and an effect registry grant no authority. Private per-run
resources are preferred; shared services have an isolated borrow/stop adapter. The native owner
settles disposable resources after either scientific success or failure, preserving useful
diagnostics outside the store. Cleanup does not establish scientific success, and `Drop` is not
a successful test outcome. Preserve uncertain ownership instead of stopping unrelated services.

Replace controller filename/unit-name drift conventions with actual artifact/manager capabilities.
Select native tool configuration once at explicit setup/build establishment. Use pinned sccache
0.19 native executable/service behavior rather than a Rust translation of per-compiler Python
hashing, service-identity polling, ledger updates and process-group settlement. The operator
selected on 2026-10-28 that cache-service loss alone does not invalidate a valid compiler result;
58f supplies the native tool configuration and targeted controls. Real compiler failures, child
exit and containment remain meaningful. `SCCACHE_NO_DAEMON` is not a never-start guarantee;
selected logging/distribution modes need exact-source qualification before integration.

Retire bespoke build observation and its live build-stage/wait/freshness consumers and alerts.
Retain scientific calculation/progress and useful diagnostics. Implement the useful native
`OTEL_*`/local settings resolver below scientific dependencies: existing Rust
`Host::from_environment` consumes serialized settings and is not that resolver. Optional telemetry
does not gate local computation. Assessment/pressure observations need individual retained-value
dispositions; collector removal is a separate decision because queue/failure alerts depend on it.
No second SDK or monitoring stack follows from this cutover.

Executable selection uses atomic replacement of a completed selection; immutable payloads in
58e and canonical transaction publication in 58d retain their own visibility semantics. Delete
frozen-Python generation/source-hash coherence machinery when its consumer goes; do not create
a universal publication protocol.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [B01](#b01) | planned | 58b | Reviewed root and independent seed |
| [B02](#b02) | planned | 58b | Typed user-manager and kernel observation |
| [B03](#b03) | planned | 58b | Environment/admission/launch callers migrated |
| [B04](#b04) | planned | 58b | Service/compiler consumers integrated |
| [B05](#b05) | planned | 58b | Cold-start and host journey acceptance |

<a id="b01"></a>
### B01 — Establish the independent construction root

After [T01's decision prerequisites](58-rust-owned-system.md#t01), implement the root and explicit
seed/install/update path. Inspect its resolved normal/build dependency closure and prove it
does not reach excluded product/Python owners. Targeted tests cover missing tool, failed build,
failed publication and interrupted update preserving the last completed selection, including an
inherited old Python compiler wrapper with no venv. A cold
checkout with no venv must construct and run help/diagnosis without database or science setup.

<a id="b02"></a>
### B02 — Delegate manager operations and retain physical observation

B01 supplies executable independence. Implement typed manager requests/job observation and
kernel identity/drain readers with exact dependency pins and the Tokio move above. Targeted
tests cover completion-before-reply, unrelated job signals, disconnect/unknown outcome, PID or
unit reuse, absent cgroup and populated descendants. Use recorded protocol/kernel inputs for
policy tests and a real disposable user-manager unit for the host255 capability check.

<a id="b03"></a>
### B03 — Move environment, admission and launch as one ownership slice

B02 supplies real containment/observation. Deliver the retained environment, launch and release
outcomes through the lean host owner, migrating recipes, direnv and direct callers together.
Replace `pse_env`, `host_admission`, `operation_lifetime` and `systemd_environment` responsibilities
according to the target above; do not mechanically preserve every old branch or ledger.
Preserve original child exit, retained cleanup failures, necessary borrowing and original absolute
stop deadlines. Establish the typed fixture capability seam with F02 and C04. Targeted controls
exercise partial launch, interrupted caller, borrowed owner, failed sibling teardown, uncertainty
and populated descendants. Delete replaced Python mechanisms/configuration/tests in the same
cutover. Temporary incompleteness receives no acceptance credit and needs no fallback launcher.

<a id="b04"></a>
### B04 — Integrate service and compiler launch consumers

B03 supplies physical ownership and the environment contract. Integrate required database/service
launch with D's schema-ready/write-admission capability, without pulling schema clients into the
cold root. Integrate compiler callers with F02's native sccache configuration and remove the old
per-compiler wrapper. Choose the native server/client startup and compiler placement using pinned
source; isolate any genuinely missing retained capability instead of inventing another daemon.
Targeted controls cover compiler success/failure, cache loss with valid output, selected native
modes, manager refusal, service teardown without database availability and failure before schema
readiness. Establish external artifact validity at selection, then carry the capability without
per-read source scans. Remove obsolete helper-image/frozen-Python publication checks and all
corresponding launch writers, recipes and wrapper tests with their replacement. Third-party
subprocess tools remain eligible.

<a id="b05"></a>
### B05 — Demonstrate cold-start and owned-resource settlement

Use targeted native host fixtures for B01–B04: no Python, missing/stale tool, unavailable user bus,
interrupted launch, borrowed child, service restart and surviving descendant. Independently
inspect selected `/proc`/cgroup identity and manager state to challenge the controller's own
claim. Original read-only diagnosis must still explain each refusal. Retire obsolete bridge
records/fixtures after all writers move; do not delete retained user data or unrelated scopes.

## Verification and handoff

**Proposed:** all packages and their tests. During future implementation, use the applicable
`just check-package <host-package>` and `just unit-package <host-package> <filter>` routes;
pure bootstrap tests must not select unrelated Arrow validation roots. B05 contributes host
journeys to the coordinator's H scope-end qualification, after all functional scope is ready.
[H](58h-qualification-and-publication.md) owns integration, hygiene and manual checks; no build, test or host mutation was run
to author this document. Closure updates the environment/architecture owners through the
decision route and records supported host/API limits. No speed or reliability gain is measured.
