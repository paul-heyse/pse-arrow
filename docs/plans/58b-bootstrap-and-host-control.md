---
title: "Plan58b: Bootstrap and host control"
status: draft
date: 2026-10-28
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, environment, validation]
review_sources:
  - docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md
  - docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md
  - docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md
---

# Plan58b: Bootstrap and host control

This companion develops O05 and the host portion of O02/O06. The
[coordinator](58-rust-owned-system.md) owns selection, rule changes and overall acceptance;
this document owns the packages below. Creation authorizes planning only. Plan57 is superseded; its failed/interrupted evidence remains historical under its original conditions.

This revision incorporates [I01/I03–I05/I10–I12](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01). That authoring amendment used `c3b7f136d` and inspected production source `bd1c4891e`; the subsequent checkpoints record delivered native scope. Recommendation disposition belongs to the coordinator.

## Baseline and target

The original functional review inspected `bd1c4891ea7d71178e809b11045dd4fd89602c0f`. At that baseline,
`scripts/pse-env`, `scripts/setup-env`, `scripts/bootstrap.sh`, the compiler-cache launcher
and `scripts/cargo-bin/cargo-pse-correctness` entered first-party Python; rust-only
bootstrap invoked Python tool/configuration helpers. The native cutovers below replace those paths. `xtask/Cargo.toml` has unconditional
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

The [database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O06)
is incorporated for planning on 2026-10-29 at `a6a144bce`. Its opportunity identities are
source-qualified and dispositioned by the coordinator. B owns physical deployment construction,
not canonical state interpretation. Retain the selected source-built SurrealDB 3.3.4/MX1
patch tuple, explicit SSI and synchronous AOL/every-append persistence profile; this is persistent
canonical authority, not a second disposable cache. Neither a KV backend, direct working store,
embedded deployment nor a different host resource policy is selected by this amendment.

## Native manager and physical ownership

Select exact `zbus =6.1.0` and `zbus_systemd =0.26201.0`, using only the required systemd1
proxies and a Tokio connection runtime. The [library reference](../libraries/zbus_and_zbus_systemd.md),
especially its runtime, job and transient-unit sections, supplies the capability basis;
[zbus's version-six guide](https://z-galaxy.github.io/zbus/upgrading-to-6.html) supplies the
API migration context. The original assessment identified zbus's Tokio requirement `^1.53.3`
against the then-workspace pin `=1.53.2`; the native implementation checkpoint below records
the narrow exact move to `=1.53.3` and affected callers. Preserve one resolved Tokio runtime.

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

The [current execution handoff](#current-execution-handoff--2026-10-29) records the latest
implemented boundary, closeout checks and remaining work. Earlier checkpoints retain their
original evidence scope.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [B01](#b01) | done | 58b | Reviewed root and independent seed |
| [B02](#b02) | in progress | 58b | Typed user-manager and kernel observation |
| [B03](#b03) | in progress | 58b | Environment/admission/launch callers migrated |
| [B04](#b04) | in progress | 58b | Service/compiler consumers integrated |
| [B05](#b05) | in progress | 58b | Cold-start and host journey acceptance |

## Execution dependencies

The maintainer authorized the entire B01–B05 scope on 2026-10-29, including necessary
C/D/F/G consumer cuts. The following stages describe dependencies, not additional status
owners. Package progress remains in the table above. A prerequisite slice means a working
capability; it does not require completion of its enclosing companion.

| Stage | Delivery | Working prerequisites |
| --- | --- | --- |
| 1 | B02 uncertainty handling, physical observation and direct diagnosis | Existing B01 |
| 2 | B03 environment, placement and admission | Stage 1; C04/F02 owned/borrowed fixture seam |
| 3A | B04 retained provider and compiler consumers | Stage 2; F01/F02 native configuration |
| 3B | B04 native server construction and physical service fixture | Stage 2; exact retained server recipe |
| 4 | D01 connection and D05 readiness/migration slice | Stage 3B fixture; F03 declarations/generation |
| 5 | B04 native service composition and host-consumer retirement | Stages 3A/4; C04 shutdown and necessary F/G consumer cuts |
| 6 | B05 assembled acceptance and closeout | Stages 1–5 implemented and integrated |

Stages 3A and 3B may progress independently. Construction and low-effect policy can develop
before launch/admission integration; consumers receive no acceptance credit until their working
prerequisites exist. D05 consumes B's physical fixture and supplies schema-ready admission to
B04 application composition, avoiding a whole-B04/whole-D05 dependency cycle. Full D03 study
progression and the unrelated F developer surface are not prerequisites for the host owner.

Retain existing zbus/systemd/Tokio facilities. The reviewed sd-notify and hierarchical-shutdown
options remain conditional: this delivery uses Type=exec plus D-owned application readiness,
and explicit retained futures/joins for the flat host controller. Cargo fingerprints, native
sccache and immutable prepared-provider generations retain distinct authority. Current pin
and feature-unification/profile decisions are not changed by overview version differences.

Whole-B03 completion includes the dependent consumer cuts needed to retire the named old
host mechanisms. Moving their remaining imports to another plan is not retirement. Explicit
finite/custom/reference/timing modes and retained compiler/solver/isolation/UNO/PETSc routes
are unfinished scope, not exclusions inferred from their current refusal.

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
cold root. Bind explicit server construction to the effective source/patch/profile identity from
`scripts/surreal_source.py`; carry the established executable/deployment capability rather than
rechecking source on every call. B establishes authentication/physical ownership and delegates
schema interpretation and open-write admission to D. An unavailable database cannot prevent
B/C from stopping and joining owned native work. A server restart, engine snapshot or successful
health request alone cannot establish D05's schema/version fence.

A future engine/topology trigger from [boundary O06](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O06)
requires D's deliberate compatibility/invariant decision and H's selected deployment evidence
before changing B's builder/adapter. It is not per-table MX/KV routing or permission to rebuild,
replace or reset an existing store. Targeted B04 acceptance pairs supported launch with refusal
before schema readiness and cleanup while canonical connectivity is unavailable.

Integrate compiler callers with F02's native sccache configuration and remove the old
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

**Proposed:** remaining package scope and its broader qualification. During implementation, use the applicable
`just check-package <host-package>` and `just unit-package <host-package> <filter>` routes;
pure bootstrap tests must not select unrelated Arrow validation roots. B05 contributes host
journeys to the coordinator's H scope-end qualification, after all functional scope is ready.
[H](58h-qualification-and-publication.md) owns integration, hygiene and manual checks; initial plan authoring performed no build, test or
host mutation. The execution checkpoint below records subsequent implementation. Closure updates the environment/architecture owners through the
decision route and records supported host/API limits. No speed or reliability gain is measured.

## Execution checkpoint — 2026-10-28

T01/T02 remain established under proposed ADR-0233 and the scoped combined formal review;
formal ADR status is unchanged. The initial selected wave covered B02/B03 ordinary foreground ownership
plus necessary C04/F01/F02/G slices. Its first consumer was database-free: D01/D05 readiness,
optional MCP/PyO3 facades and a broader product CLI are not prerequisites or selected additions.

B01 now publishes `pse-host` and `cargo-pse-correctness` together under one immutable generation,
selected atomically through `target/host-tools/current`. A checkout-local explicit-update lock
covers construction/copy/publication/cleanup, without entering ordinary admission or being
inherited by Cargo. Failed/interrupted construction preserves the last completed selection;
post-publication interruption cannot erase its selected pair. Prior published generations stay
available to running commands. Arbitrary concurrent external source/artifact mutation is not a
supported publication baseline. Reads/help/launch never construct tools or hash source trees.

B02/B03 now perform native foreground execution through `scripts/pse-env`, with recipes,
direnv, the SessionStart hook and Cargo aliases migrated. A same-image gated child is spawned
before controller relocation; caller-only and workload scopes share the authenticated prior
slice, workload lifetime binds to caller lifetime, and caller lifetime retains the prior
application-unit dependency. Exec release follows exact population checks. Raw Unix argv/env/cwd,
pipes, controlling terminal, original exits and signals stay local to exec, not D-Bus text.
Nested commands authenticate the actual native workload and exec within it. Markers grant no
ownership. Startup and drain each retain one absolute deadline; independent settlement errors
are combined. A manager stop reply alone never proves descendant release.

Services and scopes acquire one `AddRef` atomically at launch. Reconciliation checks the original
intent/invocation and this exact bus client's `Unit.Refs`, never another recursive `RefUnit`.
Repeated reconciliation adds no references; successful release invalidates duplicate handles'
mutation/observation authority. Manager/proc/cgroup view authentication and same-UID external
restart/reconfiguration limits from the initial prototype remain: systemd255 has no atomic
StopUnit-if-InvocationID API. Existing units are borrowed read-only.

The selected path supports configured unbounded ordinary sharing, existing docs environments
and native cache auto/on/off. It retains common slice ancestry and default task constraints and
refuses displaced finite/custom CPU/memory/cpuset/task limits. Explicit finite/scientific classes,
custom placement, unsupported provider/native profiles, reference/timing admission and store readiness currently
refuse, with no command launch or Python fallback. Already-known unsupported command/caller
selections refuse before personal activation. Personal-file-derived selections necessarily follow
that explicit evaluation. These unfinished modes receive no acceptance credit.

Activation evaluates personal settings once and validates a complete private NUL transfer before
changing parent shell values. Printed activation code and explanation contain no private values;
explanation never evaluates personal code. Native compiler settings select installed sccache
without a Python per-compiler watcher. Optional telemetry uses the one SDK-independent Rust
settings contract and native OTEL/local-state resolver; invalid telemetry cannot block computation.
The existing declared libtest stack is selected natively. 58f owns detailed configuration and
correctness semantics; 58g owns scoped retirements and remaining legacy consumers.

B04 has its native compiler/cache consumer slice; service schema-ready/write-admission remains
D-owned and outside this database-free wave. B05 has targeted real-manager ownership journeys,
not its entire fault matrix. Remaining B02/B05 evidence includes D-Bus acknowledgement-loss,
disconnect/restart/reuse/view-denial controls, kill between individual scope attachments,
creation-specific SIGINT/SIGTERM and exact-budget/failed-sibling fault injection. TTY coverage
establishes descriptor/control-terminal preservation, not a resize/job-control matrix.

## Selected wave verification and outcome — 2026-10-28

**Implemented:** the selected native ordinary frontdoors, coherent tool publication, foreground
ownership, shared telemetry resolver, native Cargo/nextest correctness composition and scoped
build-observation/compiler-wrapper retirement. B02–B05 remain in progress; these components do
not close every package, unmigrated caller or Plan58 as a whole.

**Tested:** focused native owner journeys exercised actual user-manager scopes/services on this
Linux host, including controller SIGKILL, a witnessed blocked exec gate, prior application stop,
a TERM-ignoring setsid descendant, nested membership, raw payload/TTY and 126/127/exit7/signal143.
Repeated service reconciliation followed by one release and 16 concurrent fast-failing services
also pass. The final assembled `scripts/pse-env -- cargo t -p pse-host --lib --test activation --test
bootstrap --test boundary --test correctness --test foreground --no-fail-fast` passes 83 tests
across 6 binaries, zero failures/skips, with 16 default physical lanes and the restore-cargo setup
passing. This is selected local host/component acceptance, not scientific/system qualification.
All failure baselines are zero; no ignored/skipped result is accepted as a pass.

Native direct `pse-columnar --lib` correctness check passes for an actual Arrow/DataFusion closure;
`just check-package pse-runtime` passes after the shared settings extraction. Focused telemetry,
activation, stack, bootstrap interruption/concurrent-update, no-Python-frontdoor and correctness
protocol controls pass. Native Nextest controls challenge inherited discovery markers, private
graph stdout, actual test-attempt Cargo restoration, invalid graph and unsupported reuse/workspace
or configuration overrides. 58f/58g record their corresponding retained controls.

Initial failures exposed guard/restoration bypasses, numeric manager reference accounting and
fixture cleanup accidentally borrowing enclosing test containment; all were corrected before the
named passes. A legacy build-environment module test unexpectedly entered a Docker solver-image
path and was stopped at exit143; it provides no acceptance credit. Its selected pure configuration
class subsequently passes 15 tests after the direct-cache-client expectations and absolute-path
selection were corrected. No broad solver, scientific, database, feature, parity or performance
campaign was resumed. Export delivery and cache-loss behavior have no new live qualification.
Existing canonical stores/services and authored/reference evidence were preserved. Quantitative
benefits remain unmeasured. H owns broad integrated/static/scientific qualification.

`just ready` passes after final dependency/activation integration. Explicit owner generation
refreshed the capability reference/CLI through `scripts.capabilities.reference` and
`scripts.capabilities.cli_codegen`; full native-provider code generation remains unavailable
and was not run. No partial generator publication is allowed before that refusal. `just turn-end` passes the ADR-index/format bundle against zero. Final independent user-manager
readback lists no remaining `pse-host-*` or `pse-foreground-*` units, and no owned test campaign
remains active. Later focused controls also pass for prefix help/refusal-before-activation and
private TOML diagnostic redaction. Scope-end H remains open.

Final normal/build dependency-tree inspection of `pse-host` excludes Arrow/DataFusion,
Symbolica/mathematical runtime, SurrealDB, product schema/codegen and PyO3. The shared telemetry
contract therefore does not reverse the cold-root dependency direction.

## Explicit compiler/KLU consumer checkpoint — 2026-10-28

The selected follow-on B03/B04 scope is the database-free compiler/KLU profile and existing
KINSOL Runtime consumer, with necessary F01/F02/C04/G slices. C01/C02, F03 and D readiness
were not required. This extends the ordinary boundary; it does not close B02–B05 or select
other solvers, helper images, finite/reference/timing admission or database establishment.

Preparation is explicit: `scripts/pse-env -- scripts/pse-host native prepare --capabilities
compiler,klu`. It requires authenticated native containment and delegates construction to
installed CMake and the locked SuiteSparse source. Launch through `--native=compiler,klu`
only selects a completed artifact; missing, changed or corrupt selections refuse before the
consumer starts. Help/explanation neither evaluate personal settings nor prepare providers.
Bare `--native`, other provider sets and custom toolchains remain unsupported.

The operator selected `rust-v1/kinsol` beneath the existing native cache root, immutable
generations and no automatic collection. Legacy generations are preserved. Recipe identities
include source, tool bytes, selected flags/options and the effective compiler-cache bridge;
version-2 completion records validate installed artifacts. Explicit preparation alone discovers
and hashes source. A per-recipe kernel lock serializes construction, and atomic publication
selects only a completed sealed generation. Selection preserves unrelated raw environment
values and can consume its own verified selected environment when commands nest.

The native compiler environment supplies paired SuiteSparse paths and the existing receipt
contract to bundled SUNDIALS/KINSOL, together with matching Clang/resource/libclang selections.
It uses the installed sccache client directly and retains the configured full compiler width.
No Python setup, solver image, database, new crate or compatibility controller was added.
The Python KLU constructor and CLI option are deleted; old artifact readers remain only for
their unmigrated owning consumers. `unit-native-capability-package` now passes Cargo/nextest
arguments directly through the selected native boundary and correctness composer.

### Verification and outcome for the selected compiler/KLU slice

**Implemented:** explicit construction, source-free launch selection, native compiler/cache
composition and the scoped caller/constructor retirement above. Full package and H qualification
remain open. The native provider and boundary received an independent source review; that
review is distinct from test acceptance.

The first live construction completed KLU installation but failed before publication: sealing
the staged root before a cross-parent rename prevented updating its directory parent entry.
The correction keeps only that root writable through the move, seals it at its final location,
then publishes the selection. Corrected preparation succeeds and an explicit repeat selects
the same immutable generation. A direct preparation call outside native containment refuses
before setup effects. No legacy generation or canonical database was changed.

**Tested:** private native sccache 0.19.0 C controls establish one miss, one hit, actual compiler
failure with no object, and valid output after the private cache server's confirmed exit before
compiler release (`scripts/pse-env -- /bin/sh /tmp/plan58-cache-control.sh`). Four controls pass
against zero failures; supporting artifacts are retained in the local ignored
`build/plan58-native-cache-control/`. This is C compilation cache-continuity evidence, not a
Rust/C++ cache-loss campaign or a measured performance benefit.

**Tested:** `scripts/pse-env -- cargo test --locked -p pse-host --lib native --test native`
passes 16 controls against zero failures; its ignored lock-holder helper is invoked explicitly
by the cancellation control, not credited as a separately passing test. The final native
assembly, `scripts/pse-env -- cargo t -p pse-host --lib --test activation --test bootstrap --test
boundary --test correctness --test foreground --test native --no-fail-fast`, passes 99 tests
across seven binaries, zero failures, with that one helper excluded from normal scheduling.
The native Nextest Cargo-restoration setup also passes. Controls cover missing preparation,
private explanation, corruption, changed flags, raw values, repeated selection, atomic pointer
publication and unsupported selections. Independent source review has no remaining material
findings in this bounded provider/boundary baseline.

Live preparation controls (`/bin/sh /tmp/plan58-native-preparation-controls.sh`) pass three
scenarios against zero failures: configure exit47 retained by the boundary failure diagnostic
with no selection published; SIGTERM of the actual gated construction's controller returns143
and independently witnessed configure PID/cgroup drains with no selection; two concurrent cold
prepares configure once and select the same completed generation. Supporting local artifacts
are in `build/plan58-native-preparation-controls/`. These controls use private caches and real
owned workloads; they do not mutate legacy providers or canonical stores.

The first selected Runtime run passed its preparation-loan cancellation control but failed both
scientific tests because their helper built a package and execution services on different
Runtimes. The helper now uses the same Runtime for both, keeping its existing native allowance,
all scientific assertions and the production binding guard. The corrected direct native run
passes all three tests against zero failures:

```sh
scripts/pse-env --native=compiler,klu -- cargo t -p pse-runtime --lib \
  --features pse-runtime/solver-kinsol \
  -E 'test(root_response_publication_has_physical_primal_and_no_kkt_fields) | test(root_response_withheld_at_active_bound_keeps_base_solution) | test(cancelled_serial_preparation_keeps_loan_and_storage_until_child_drain)' \
  --no-fail-fast
```

This exercises authored square-root solution/physical sensitivity publication, retention of
the feasible base solution when a bound prevents sensitivity certification, and the existing
blocking-preparation loan/storage drain control. It is selected local Runtime acceptance, not
whole-product, canonical-store, all-solver, numerical parity or performance qualification.
Unselected Nextest inventory entries are filtered scope, not accepted results. Quantitative
benefits remain unmeasured, and H's joined campaign remains open.

The migrated consumer also passes the same three tests through a nested native boundary:

```sh
scripts/pse-env --native=compiler,klu -- just unit-native-capability-package \
  pse-runtime solver-kinsol compiler,klu \
  'test(root_response_publication_has_physical_primal_and_no_kkt_fields) | test(root_response_withheld_at_active_bound_keeps_base_solution) | test(cancelled_serial_preparation_keeps_loan_and_storage_until_child_drain)'
```

The recipe already supplies `--no-fail-fast`; an initial invocation duplicated that option
and was rejected by native Nextest before running tests. Removing the duplicate preserves
native argument ownership. The successful command above establishes the real migrated caller
and nested selection, rather than only an equivalent hand-built command. Preparation reuse
also succeeds after the final host installation. Only the authenticated staging directory from
the first failed publication was removed after drain; published and legacy generations remain.

The selected compiler/KLU execution wave is complete at this bounded acceptance level.
`just ready` passes after the dependency integration, and the root's `just turn-end` passes
ADR-index generation and formatting against zero failures. Final independent manager readback
has no remaining `pse-host-*` or `pse-foreground-*` units, and the owned test/preparation runs
have finished. B02–B05 retain their broader fault, placement, provider and service obligations;
F/C/G retain their other consumers. The next execution choice must select its actual consumer
and dependencies from those existing owners; no broader campaign or service cutover is started
by this checkpoint.

## Full host execution checkpoint — 2026-10-29

The full B01–B05 execution is authorized, with the dependencies above. B02–B05 remain
in progress; individual source delivery and focused controls do not close these packages.

**Implemented:** `bc299317e` records the execution dependencies. `9f43ec587` retains
original launch identity through acknowledgement/signal failure and replaces the production
10ms manager-drain poll with manager notifications plus original cgroup event observations.
`9a0333398` adds explicit native provider preparation/selection, preserving compiler/KLU and
covering compiler-only, solver image, isolation, UNO and PETSc protocols. Bare `--native`
retains the full profile/caller override; explicit `--native=` selects provider-free runtime
settings. `b95a32dc8` adds the exact patched-server constructor and source-free immutable
selection, plus partial direct activity diagnosis when manager observation is unavailable.
Neither executable selection nor process start establishes schema/write readiness.

**Tested:** `just unit-package pse-host manager::tests` passed18 controls against zero
failures for the committed B02 slice. `scripts/pse-env -- cargo test --locked -p pse-host
native:: --lib` passed25 controls against zero failures after the external-metadata correction.
`scripts/pse-env -- cargo test --locked -p pse-host server --lib --test server` passed14
controls against zero failures (10 server library controls, one incidental activity-name
match, three server integration controls). The targeted activity/boundary/native binaries
passed11 tests; their ignored lock-holder is an explicitly invoked subprocess fixture, not
credited as another passing test. Native-default parser controls also pass3 tests; their
name filter did not execute the integration targets, which were run separately without it.
New B03 manager methods initially produced unused-code warnings while their admission module
was unexported; its subsequent compile check passes without warnings. These are focused
host/protocol results, not whole-package or scientific qualification.

Independent source review found an accepted native Cargo target selecting the wrong server
artifact path, and external isolation metadata still pointing at the original mutable prefix.
The corrections preserve target-specific artifact identity and relocate copied `.pc`/`.cmake`
metadata. The regression controls include a stale host artifact and actual `pkg-config`
consumption after deleting the original isolation prefix. Review also corrected an initial
bare-native default inference and its mixed-option precedence. A fixture spawn intermittently
failed ETXTBSY before entering the boundary; the frontdoor fixture now references the actual
checked-in immutable shell adapters instead of executing freshly copied scripts. Its affected
controls then pass without retries. No previous failure is counted as a pass.

A real explicit compiler-only preparation and selected C compile/run consumer also pass against
zero failures using a private cache under `build/plan58-native-compiler-control/`. The C control
checks actual executable exit7. Other newly added provider source/image builds, linked scientific
consumers, full upstream server compilation and schema-ready service establishment remain
unqualified; command fixtures do not establish them. No existing database or cache generation
was reset, collected or restarted. Performance benefits remain unmeasured.

Current integration implements finite admission and ordinary exclusion using the user manager,
a shared runtime lock/revision and finite-only reservation state, with the original gated child
retained through uncertain attachment. The controller stays in its actual prior parent while
finite workload/service roles share an owned allocation slice; owner references and physical
workload drain remain distinct. B03 remains open for assembled acceptance, resident-service
parking/borrowing and cross-parent ordinary placement. D01/D05 source work supplies native
schema/version fencing and quiescent migration before B04 service composition can open writes;
configuration-file checks do not establish that boundary. Existing host consumers and their
replaced Python mechanisms still require the stage5 cutover; H qualification is not started.

The assembled finite foreground control now passes: an explicitly selected light workload
receives an actual 2GiB kernel envelope, its nested request borrows the same original cgroup
and charge, exit7 is preserved, and physical drain precedes charge removal. The installed
parent's comments differed from the repository declaration; provenance comparison now preserves
the ordered operative directives while ignoring comments. Existing sufficient parent limits
need no mutation. Actual parent changes and isolation still require descendant ownership
checks. A separately owned legacy compiler-cache service contributes only a freshly validated
read-only capacity observation, with no stop, reference or configuration capability.

**Tested:** `just check-package pse-host` passes without warnings after the qualification fix;
`just unit-package pse-host 'test(admission::tests) | test(manager::tests)'` passes33 tests,
zero failures. `scripts/pse-env -- cargo test --locked -p pse-host --test foreground
foreground_explicit_finite_scope_and_nested_borrow_keep_actual_envelope_and_release --
--nocapture` passes its one selected real-manager control, zero failures. Final readback finds
no native finite reservation. These results do not close remaining B03 placement, resident-role
or consumer-retirement obligations. Release reconciliation now retries unrelated admission
revisions within its original deadline. The added controller-death control exposed an empty
original allocation slice remaining manager-active after SIGKILL: the old predicate incorrectly
retained its charge indefinitely. An exact bound slice with proven empty hierarchical kernel
population can now settle without stopping or taking ownership of an unrelated unit;
scope/service rules remain distinct. The corrected TERM/KILL control passes without manually
resetting the journal. Empty slice definitions can remain after abrupt owner death; they do
not imply populated work or a charge.

**Tested:** the scoped manager/admission/exact-server-reopen filter passes35 controls, zero
failures. `scripts/pse-env -- cargo test --locked -p pse-host --test foreground finite --
--nocapture` passes both real finite controls, zero failures. This covers original exit,
nested borrowing, TERM cleanup and post-SIGKILL original physical reclamation. Final native
finite state is empty. The pending unused workspace-dependency warning is retained below;
there are no Rust warnings. Broader placement, service and retirement acceptance remains open.

The explicit server constructor now also admits an existing matching source release through
`server prepare --from-release`, snapshots and hashes the copied binary/lockfile, and retains
the verbatim original release receipt under distinct imported provenance. It neither fabricates
a current build context nor consults the old source during selection. This avoids a redundant
cold rebuild for the native service integration. The actual retained release was imported into
native recipe `a59f06be7998a8f5a2674bdc2cb74f6ae66e4400bf72ad04562f6c0e470f2411`,
generation `01a1864e7feb742ca03747abc71a3e41`; copied bytes match the original receipt and
source-free selection returns that generation. No database started or existing artifact changed.

**Tested:** `scripts/pse-env -- cargo test --locked -p pse-host server:: --lib` passes21
controls and the same route with `--test server` passes3, zero failures. The source builder's
Cargo/toolchain TOML document parsing was corrected after the import exposed the same API
mistake; real table-document regressions now pass. Those final runs reported one unused
workspace-dependency warning from the pending native-service consumer declaration, with no
Rust warnings. Full source compilation remains unperformed. D01/D05's focused implementation,
generation and parser results are recorded at its [own checkpoint](58d-canonical-control-and-connectivity.md#native-host-dependency-execution-checkpoint--2026-10-29);
actual isolated service/migration acceptance remains next.

The actual pinned solver image also prepares successfully through the new native constructor
and selects its sealed generation. A selected solver-only boundary finds the Ipopt/SCIP headers
and resolves the Ipopt shared library's dynamic dependencies without missing libraries.
This establishes real image extraction and runtime selection, not a solver calculation,
numerical parity, cancellation campaign or performance benefit. Other new provider source
builds and their scientific consumers remain unqualified. Existing provider generations and
stores are preserved.

## Current execution handoff — 2026-10-29

The maintainer requested conclusion of current assignments, targeted functional/type/lint
repairs, integration and documentation before further execution. Both implementation workers
and the retirement mapper have concluded. All work was integrated in the existing main
checkout; `git worktree list --porcelain` found no auxiliary worktrees to merge or remove.
No owned campaign remains running. B02–B05 remain **in progress**, not complete.

**Implemented:** `ac3994f21` gives each ordinary operation a private slice containing its
workload and bound sibling services. The controller remains in its original parent; the slice
is bound to its original controller before launch. This adds neither numerical reservations
nor CPU/memory budgets. Recovery and physical settlement attempt the original workload and
whole operation hierarchy under one deadline, retaining every failure. Nested explicit
placement derives the original target from authenticated manager/kernel identities.

Service launch uses systemd 255's typed `ExecStartEx` with `no-env-expand`. The manager setter
preserves specifiers when persisting the property; manual percent escaping would corrupt argv.
Actual controls preserve dollar expressions, percent expressions, empty/quoted/tab/newline and
Unicode arguments, literal environment values and an executable path containing `$HOME`/`%n`.
The initial literal-dollar failure exposed real service expansion and was fixed in production;
the original adversarial fixture was restored for the passing controls.

**Tested:** the correctness-routed `pse-host --test foreground` selection of
`foreground_ordinary_`, `foreground_bound_service_preserves_literal_`,
`foreground_nested_execution_`, `foreground_explicit_finite_`,
`foreground_finite_controller_` and `foreground_preserves_raw_` passes 8 tests, 9 excluded,
zero failures. The final native canonical selection passes 78 tests, 420 excluded, including
all five live fixture/migration cases in one run; [58d](58d-canonical-control-and-connectivity.md#native-host-dependency-execution-checkpoint--2026-10-29)
owns its command, behavior and limits. Independent readback finds all eight original service
units not-found/inactive with MainPID 0 and no successful fixture directory left. The native
finite journal is empty. Earlier failed disposable fixtures/logs remain available for diagnosis;
existing stores, services and prepared generations were not reset or collected.

`f28e0e373` completes the bounded native service prerequisite: exact prepared-server generation,
private module/credential setup, listener ownership, actual ingress/connection join and fresh
current/adjacent-predecessor migration controls. The D01/D05 slice needed for stage 5 is now
working; neither whole D package must finish before a selected retained consumer can move.
Provider construction still has distinct evidence: the real solver image extracts/selects and
resolves its dynamic dependencies, while new isolation/UNO/PETSc builds and scientific uses,
and a full cold upstream server build, remain unqualified.

**Tested — closeout checks:** `just clippy-default`, `just clippy-no-default`, `just docs-rust`,
`just audit-shear`, `just lint-py`, `just typecheck`, `just python-contracts-check` and
`just codegen-all-check` pass against zero failures. Type checking reports 0 diagnostics with 7
existing suppressions; no new typing suppression was added. The additional correctness-routed operations
Clippy check with `native-service,canonical-tests,canonical-maintenance,test-support`, all
targets and `-D warnings` also passes. Python host-tool repairs pass 24 targeted unittest cases.
Typos/license findings were repaired and their owning checks pass. Stale document schemas,
telemetry descriptions and provenance were regenerated by their owner; generated files were
not hand edited. `scripts/bootstrap-host` publishes the final native executable pair explicitly.

The initial `just hygiene --live` attempted 24 steps: 16 passed and 8 were unsuccessful. The
failing source/lint/generation checks were repaired and rerun individually; the aggregate was
not rerun and is not reported green. In particular, the older installed Python extension still
differs from the current registry/API. `just ready` passes but reports that extension warning.
`just python-stubs-check --extension target/debug/lib_native.so` passes against the separately
compiled current API; default installed-extension checking remains stale. No declarations were
rolled back to fit that old binary, and its Python runtime receives no new qualification credit.
The annotation-only contracts recipe no longer selects unrelated native providers or imports
an extension. Root `just turn-end --live` passes after integration and formatting.

**Next work:** continue stage 5 with one retained Rust canonical consumer on the native
service/fixture and typed readiness path, then delete its replaced Python orchestration and
mechanism-specific tests. Preserve scientific assertions, reference oracles and historical
readers. The mapper distinguishes optional MCP retirement from retained reference/test/service,
invocation/conversion and live evidence consumers; removing MCP alone cannot close B03/B04.
Remaining B02–B05 obligations include their actual caller migrations, resident/reference/timing
and broader placement controls, remaining provider acceptance and assembled cold-start/failure
journeys. D05 still owns actual stale-epoch/in-flight race and acknowledgement/restart coverage.
H, full numerical/parity/feature qualification and performance measurement were not run.

## Native physical-owner continuation — 2026-10-29

The stage-5 consumer now runs a real recorded two-point KINSOL study through two independent
worker connections, with bounded control, continuation, protected-result read and physical
replacement. [58c](58c-runtime-lifecycles.md#native-worker-and-publication-checkpoint--2026-10-29)
owns its two passing cases; [58d](58d-canonical-control-and-connectivity.md#native-progression-migration-and-control-checkpoint--2026-10-29)
owns live schema-v2/v3, stale/in-flight fencing, restart and original-ACK controls. Neither
updates an existing user database. A real teardown race required clients/ingresses to drain
before their original server stops; sibling drains remain concurrent and retain all failures.
The stop owner additionally permits an already-absent original MainPID only after its exact
original cgroup is independently empty. Borrowing/general observation remains strict about
PID identity and uncertainty. The owner-minted read-only borrow carries no stop authority.

G02's MCP/catalogue/session retirement is committed; regenerated declarations and the freshly
prepared force-validated development extension agree. The formerly stale installed-extension
telemetry import control now passes in its 31-test retained unittest selection. This does not
qualify the scientific Python facade or live exporters. The Python HTTP control client and
its dedicated orchestration retire in D04 while scientific/cohort assertions are preserved.

B02–B05 remain in progress. Explicit native study-module construction is the next bootstrap
cut so that a fresh native service does not depend on Python to prepare its read-only module.
Remaining host/test/reference/service callers, resident/reference/timing placement and broader
provider/assembled acceptance retain the existing owners. No performance benefit or full
cold-source scientific/provider matrix is claimed.
