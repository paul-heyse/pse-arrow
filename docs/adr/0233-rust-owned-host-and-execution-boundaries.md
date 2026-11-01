---
id: ADR-0233
title: Own native host control and execution boundaries in Rust
status: proposed
date: 2026-10-28
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, AP-07, DP-15, DP-19]
blueprint: [§3.3, §20.6, §21, §23.2, §24.1, §24.2]
review: docs/design_review/reviews/design_review_rust-owned-host-and-boundaries_2026-10-28.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A named retained workload cannot preserve original physical ownership or canonical admission through the selected native interfaces.
verification: Plan58 B01–B05 targeted cold-bootstrap, actual systemd255 user-manager ownership/drain, environment and native compiler controls; D05 separately establishes transactional migration fencing. Plan59 A07 scoped contract review accepts Proposed design only; A03 verifies typed outcomes/diagnostics, A05/B03 verify authenticated recorded-generation control and actual drain, G verifies analysis context and session lifetime.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/58b-bootstrap-and-host-control.md, docs/plans/58d-canonical-control-and-connectivity.md, docs/plans/59a-cli-foundation.md, docs/plans/59b-runs-and-continuity.md, docs/plans/59g-rust-semantic-intelligence.md]
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
facts. Owned unit capabilities permit stop; borrowed observations do not. For a detached run started
through `pse`, the run ledger's recorded unit description and invocation ID are the stop
authority; a mismatch refuses. Their constructors
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
atomically replaces checkout-local selection. Help, read and explain verbs never rebuild or
rescan source. A top-level new-work `pse` verb that finds the host stale rebuilds it through the
same seed and re-executes the new generation; a failed rebuild refuses new work. Existing-invocation
read/wait/cancel instead authenticate the compatible recorded prepared control generation and
original unit identity without rebuilding current sources. Unknown protocols and forged tool
paths refuse; recorded metadata never grants arbitrary executable authority. Correctness adapter mode, hooks, internal subcommands and
calls nested inside a workload scope never rebuild. Running processes retain their executable. Host errors use concrete thiserror
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
Plan58 B01–B05 supply implementation evidence; Plan58 D05 and H retain their separate acceptance obligations.
This proposed ADR changes no accepted-record status; the prescribed ADR PR route owns that.

## Pros and cons

### Plan59 control and evidence amendment

**Proposed; operator-confirmed R3/R6 and RC01–RC05.** CLI invocation, scientific run/attempt,
process and observation identities are distinct with explicit, possibly missing associations.
Launch acknowledgement and bounded observation wait may succeed while the subject is running;
a completed correctness pass requires actual selection/setup, required records and terminal
reconciliation. Child outcome, original physical settlement, fixture disposal and scientific
acceptance remain separate. Compatible existing-work control does not depend on current sources.

The host presents an outer envelope; prepared product owners retain their request parsing,
schema and scientific interpretation. Native diagnostic causes/facts, Unicode locations and
independent cleanup failures survive projection. Optional observation loss stays visible;
missing required verdict evidence cannot become a pass or be repaired by schema validation.

Reusable owned analyzer sessions serve a concrete developer consumer without a second database
or general coordination platform. They retain configuration/source-generation scope and protect
tools/children until actual settlement. Query cancellation does not stop a shared session;
input changes can cancel older snapshots. Watchers alone establish no atomic filesystem capture.
Source reads need no compiler-private preparation. Generation, probes and edit application are
explicit effects; preview is not a multi-file publication transaction. Plan59 owns implementation
and qualification; formal ADR status remains proposed.

Native ownership removes interpreter crossings and duplicate operational models. The new root
adds a crate and requires integrated caller migration. Unit/cgroup uncertainty remains, and a
Rust type cannot prove an independent process alive. Server readiness/migration must remain a
separate capability rather than inflate the bootstrap closure.

### Plan59 library-integration amendment — 2026-10-30

**Proposed; A09 refines the earlier RC01–RC04 target under RT03.** Immutable prepared
generations remain retained without automatic collection. Remove collection-exclusive
protection/promotion machinery after separating authentication, provenance and real process/resource
duties. Existing unknown claims/artifacts remain untouched; no PID absence or new protocol
supplies historical settlement/deletion authority. Stale new-work refresh and authenticated
old-generation control remain selected.

Retain original stdout/stderr bytes in independently owned raw files; an optional compact
offset index describes observation order, not global producer order. Capture failures retain
the available original prefix and explicit incomplete/flush outcomes. Required extraction,
optional views and physical child settlement have independent completion obligations.

Producers own bounded indivisible pages and source/context-bound continuations. Directional
Serde/Schemars describe wire representation, while the operation owner enforces contextual
validity (including §21.5 admission) and actual terminal finality. Explain and execution share
the available semantic stage without implicit preparation or invented execution population.
Migrate immature wire consumers together without shims or loss of authored/sole original
evidence. Source-only readiness is independent of semantic sessions. Common source/range/
conflict proposals do not establish resolution, compilation, equivalence or publication.

A08 publishes this bounded clarification within the existing Proposed decision. Focused
governing-target advice is not a new formal acceptance and does not extend the prior A07
review's footprint. A newly changed decision boundary still uses the required review route.
The A08 publication included minimal J01/J02 prerequisites only; that historical evidence does
not qualify later J03–J05 delivery or the K/L/M extension, whose owning plans retain acceptance.

### Plan59 search and inspection amendment — 2026-10-31

**Proposed; operator-confirmed S-RC01–S-RC03 and arrival-order selection.** Consolidate passive
query/read interactions under `search`, `inspect` and immutable `propose` previews. Preserve
useful capabilities while migrating each replaced route and its callers/tests together.
Preparation, acquisition, probes, test construction/execution and lifecycle control remain
explicit effects. Known source/range/position, index/revision and original-evidence locators
can be inspected directly without a preliminary search or implicit provider preparation.

Use one validated Rust request interpretation for flags, structured JSON and saved queries.
Source universe, predicate scope, directed relationship, answer demand and display selection
are separate meanings. Same-line/node/entity conjunction is not same-file conjunction, whose
witnesses may occupy different nodes. Unavailable or unexamined facts remain unknown; negation
does not turn them into established absence. Providers retain native context and scientific
authority. Syntax spelling is not resolution, API documentation is not importability, related
tests are not an executed population, and current semantic observations are not original
compiler evidence.

Default bounded search returns at most 20 results in fast arrival order. Stable ordering and
guarded continuation are explicit choices; arrival-order output offers no reproducible cursor.
Count units and exact/at-least/unknown counts remain stated. Exact totals or stable ordering
may require more examination; a short answer does not claim exhaustive work. Native selection,
matcher stopping and shared parsing should honor demand before unnecessary materialization.

Compact result records reference shared source/provider context rather than repeat it per
capture. Mandatory invocation identity, command/subject outcome, coverage and original fact
meaning survive this representation change. Normal UTF-8 paths are readable; unusual names
retain exact native bytes. Versioned typed result locators carry their addressing and freshness
premises across CLI invocations without a new result database. Lightweight lexical observations
do not claim complete immutable source snapshots. Exact inspection/proposal hydration checks
the recorded premises and either refuses stale selection or explicitly produces a newly
requested observation; it never silently moves a selection to current bytes.

Directional Serde/Schemars describe accepted requests and emitted views; typed constructors
still enforce contextual validity. Native structural rule/query representations remain with
their providers rather than a new application DSL or query service. Configuration combines
typed defaults, optional repository configuration, request inputs and explicitly supplied
flags once; predicate arrays replace rather than concatenate implicitly. The native host
environment owner is unchanged. G11/I06 may investigate named useful native capabilities
without a historical recurrence threshold; selected governing changes precede G12/I07.

Plan59 K01/A10 publish this target before K02. K/L/M and their consumed packages own production,
migration and qualification. This amendment leaves formal ADR status Proposed and establishes
no new runtime test, scientific acceptance or measured speedup.

### Plan59 selected native-analysis snapshot — 2026-10-31

**Proposed; G11 positive feasibility and A11 publication before G12.** Select an explicitly
prepared immutable native-analysis snapshot for typed HIR/IDE questions not faithfully exposed
by the existing LSP. The native family is isolated behind the existing plain semantic protocol
and managed service/preparation owner; it is not a second independently updated live database.
An active session selects its backend/context explicitly. Native snapshots refuse stale source,
Cargo/toolchain/configuration or supported preparation premises and require explicit reprepare.
Portable locators bind original session identity as well as source and generation; generation
numbers alone may repeat after service replacement.

Use a separately prepared `pse-native-analysis` binary in the existing host package under an
opt-in `native-hir` feature. Ordinary host construction does not compile or link its optional
HIR/IDE dependency family. Reuse immutable prepared-tool authentication, original invocation
custody, the authenticated Unix socket, cooperative CPU completion, cancel and native stop/drain.
The version-one native snapshot connection carries one authenticated request. Its fixed
single-byte cancellation control targets that connection's CPU operation; the typed reply
follows actual CPU settlement and does not stop the session or another query. Unexpected or
oversized control frames refuse. A client retains cancellation and the independent transport
failure when its settlement reply is unobserved. Disconnect proves only client detachment;
the service still retains and settles its CPU work. This native control does not upgrade live
LSP cancellation acknowledgements into physical completion evidence.
The selected compatible native family is 0.0.358; host source syntax0.0.360 remains a distinct
parser and never passes its AST types into the native world. Unicode-version alignment belongs
to exact dependency resolution, not an assumption that compatible semver guarantees it.

The bounded native target exposes actually resolved method identity, original versus adjusted
type information, scoped read/write reference roles and declared generic-constraint facts.
Extract ordinary serialized facts in the owning native worker; do not send HIR/Salsa handles
across the boundary or infer type equality from display strings. Original rustc outcomes remain
separate. Initial snapshot preparation disables build scripts/proc macros; an explicitly requested
unsupported preparation or operation refuses rather than borrowing current LSP facts. Existing
LSP capabilities retain their own backend and context. Public SSR match/capture access remains
unavailable in the selected native release; syntax search plus selected native enrichment remains
the supported path, without manufacturing matches from rewrites. Native preview/application and
blanket trait applicability are not implied by the selected typed operations.

The isolated G11 run demonstrated two distinct same-spelled method targets, one constrained
generic method, four adjusted expressions, read/write roles and a real restricted Cargo/VFS
world. This supports selection, not production qualification. G12 owns actual service, freshness,
native public-query and cancellation acceptance; A11 leaves this ADR formally Proposed.

**Implemented realization:** native request snapshots use an opt-in cancellation group over
the original Salsa storage. The exact library source patches propagate the group through IDE
and Rayon clones while preserving handle-local suppression/reset and ordinary clone behavior.
This preserves the upstream IDE navigation/reference operations; a partial lower-level
replacement lost supported navigation and did not reach cold indexing descendants. The existing
CPU owner signals the group and still waits for physical completion. Independent groups can
reuse the same memo storage after cancellation. This is a realization of the selected lifetime
contract, not a second world or a global revision write.

API/source joins retain original facade and definition identities with compiler-unit/profile,
target, build-input and dependency-byte evidence. Request-owned qualification contexts reuse
the original graphs/routes and retain a final evidence barrier. Missing older or nonlocal byte
evidence cannot be repaired by a matching package name, lock identity or successful native
resolution. Unsupported external descendants remain unknown. Package qualification and formal
ADR acceptance remain separate from implementation and bounded library controls.

## More information

[Plan59](../plans/59-agent-cli.md) owns the subsequent A07/A03/A05/B03/G implementation and
acceptance. Its [scoped contract review](../design_review/reviews/design_review_plan59-host-control-contracts_2026-10-29.md)
accepts the A07 semantic amendment at Proposed evidence; this does not accept future runtime
mechanisms or change this ADR's formal status.

[Plan58](../plans/58-rust-owned-system.md), [58b](../plans/58b-bootstrap-and-host-control.md),
and [hard-cutover recommendations I01–I14](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md)
own planned scope/disposition. The design review links the refreshed scoped audit and independently
assesses the target under the selected standard.

## Status history

### Native local operation and async adapter retirement — 2026-10-28

The authorized next realization uses existing public runtime/source/preparation/run APIs
for a database-free KINSOL operation. Native wait-future detachment, explicit stop, final
immediate-owner departure and original native drain keep their existing distinct meanings.
The maintainer selected retirement of Python asyncio run waiting and its cancellation glue;
shared synchronous adapters and unrelated asynchronous cursor interfaces remain for later
owning cutovers. Actual compiled metadata supplies the regenerated stub. No replacement
Python future adapter, mandatory CLI, database, executor or lifecycle framework is selected.

### Explicit native provider preparation — 2026-10-28

The next authorized realization prepares compiler/KLU explicitly through the lean host,
then selects that completed profile for a database-free KINSOL consumer. Construction runs
inside authenticated native containment. Ordinary launch never builds missing providers.
The maintainer selected a separate `rust-v1/kinsol` area under the existing native cache;
legacy generations remain untouched and automatic collection is excluded. Atomic publication
retains immutable generations and version-two provider receipts consumed by the existing
backend. Existing runtime numerical and cancellation owners remain unchanged; no new crate,
solver-image extraction, database service, Python facade or lifecycle framework is selected.

- 2026-10-28: Proposed for authorized Plan58 T01/T02 and native host execution. No implementation
  or formal ADR acceptance is established by this record.

### Foreground scope and descriptor realization — 2026-10-28

The authorized integrated cutover selects a caller-only scope and a workload scope bound by
BindsTo/After, using actual pidfds. A same-executable gated trampoline is spawned before
controller relocation, then execs the raw workload only after authenticated attachment. Gate
EOF prevents untrusted execution after controller loss during startup; manager dependencies
provide containment after release. Authenticated nested work reuses its actual workload.
Finite or uncertain ancestry is refused before effects; no second admission service is added.

The command-fds0.3.3 library owns descriptor mapping. Its one early inherited-descriptor
initialization requires a documented unsafe call before runtime/owned descriptor creation;
permit that site only in pse-host, with the workspace allowlist reason pointing here. This is
an OS descriptor-ownership binding, not arbitrary unsafe application work. Mapping collisions
and unexpected descriptors refuse. Workload exec closes private gate/control descriptors.
The implementation and actual startup/controller-death races remain targeted proof obligations.

The F02 native Cargo adapter captures actual nextest source-build Cargo selection and validates
its unit graph before compiling. Source-build-only support refuses binary-reuse bypasses before
launch. One static Nextest setup restores real Cargo for tests; no per-test supervisor or
second CLI grammar is selected. This extends the existing responsibility, not a universal
operation catalogue. Existing formal acceptance and ADR status remain scoped/proposed.

Transient service/scope creation sets `AddRef` once atomically. Reconciliation validates
original description/invocation and this exact bus client's membership in `Unit.Refs`;
it never issues `RefUnit`, whose systemd sender references are recursive. Observation and
mutation recheck the original held client reference; successful unref relinquishes authority.
Failed or repeated reconciliation adds no references. Unknown creation retains the original
intent and client lifetime and never replays launch or reference acquisition. Targeted
implementation tests cover repeated reconciliation followed by a single release.


### Agent CLI narrowings — 2026-10-29

**Proposed:** the operator accepted [Plan59](../plans/59-agent-cli.md#rule-changes) R3. One clap
command grammar in the `pse-host` library, exposed as `pse`, `pse-host` and `cargo-pse-correctness`,
and native operation ownership are selected. This narrows the earlier "no second
CLI grammar" and "optional CLIs" wording; its catalog is derived from the parser, not an authored
universal operation catalogue. A09 selects synchronous foreground ownership and a detached transient service containing
its fixed wrapper, payload and ordinary descendants; the run ledger, not the launching bus client, is its stop
authority. Read-only verbs never build; top-level effectful verbs rebuild a stale host and fail on
a failed rebuild. Concurrent agents use separate worktrees, each with its own host tools. Build-stage
telemetry stays retired. This grants no formal ADR acceptance.

### Remaining native development and product retirement target — 2026-10-29

**Proposed:** the authorized B–G completion removes `pse-py` after its last real product,
developer and test consumers migrate or explicitly retire. Remove associated product packaging,
Python-only generated projections/stubs, registration/currency checks, and unused PyO3-family
and boundary dependency declarations together. No replacement ABI or dormant compatibility
facade is required. Isolated independent scientific oracle drivers and third-party tools remain
eligible; this is not a ban on Python executables.

Useful conformance manifests/reports, paired thermodynamic campaign outcomes, uncertainty and
research adoption, protected result reads, calculation observation and developer evidence remain
native capabilities at their existing domain/tooling owners. Heavy development composition stays
outside the independent host closure. Rust-owned metadata/generation and native fixture construction
replace first-party Python policy rather than preserving it behind an executable proxy.

Retire obsolete Python measurement drivers under the maintainer's explicit disposition; retain
scientific scenario requirements and original results for deferred H campaigns. This changes no
numerical tolerances, native completion witness, persisted-data protection or historical evidence.
The package owners must establish actual migrated callers and targeted acceptance before deletion.
Crate/governance and enduring boundary amendments require the scoped design assessment and
architecture revision before dependent implementation; this proposed text grants no formal ADR
acceptance and does not claim that retirement is implemented.

### Plan59 A09 native completed results — 2026-10-30

Operator-confirmed RT01 selects native Nextest exit, actual context/binary/test inventory and
producer-private completed reports with required setup/capture/drain. It removes independent
per-event lifecycle/byte-address verification, not missing-evidence refusal or independent science.
RT04 defers the specialized commit transaction; generic briefing and packet hooks leave required
scope. Ordinary Git is not claimed to provide the deferred shared-index guarantee. These changes
are Proposed; implementation and scoped acceptance remain with Plan59 owners.

### Plan62 native preparation and compiler observations — 2026-11-01

**Proposed; operator-confirmed RC01/RC02 and typed D03 target.** Extend explicit native snapshot
preparation through the existing build_macros selection. Correctness-admitted build outputs,
proc-macro executable/dylib identities and supported environment/source premises qualify readiness.
Restricted preparation remains useful. Macro queries can execute external code lazily; immutable
source does not establish hermetic execution. Query cancellation retains actual CPU ownership and
may await macro return; whole-session stop retains original managed escalation and descendant drain.
Neither a cancellation request nor terminal presentation proves native settlement.

Add an optional, separately prepared compiler-analysis binary inside the existing host package.
Compiler-private linkage stays outside ordinary host/HIR construction. The existing compiler
investigation owner supplies admitted units, exact toolchain/runtime dependencies, explicit
acquisition, original artifacts and foreground signal/child settlement. A stage-local wrapper
selects the actual root compiler invocation, delegates unrelated units, preserves supported wrapper
composition and continues normal compilation after copying early observations. Completed facts
publish only after successful original compilation and capture/settlement; failed outputs remain
partial evidence. No read installs components, builds a tool or starts acquisition.

Project owned phase-qualified captures, coroutine state and selected typed MIR cleanup/unwind facts.
No MIR-text parser, new compiler service, workspace crate or general loan database is selected.
Original artifacts retain independent inspection value. Predicate evaluation status remains distinct
from Boolean refinement truth: NotProven/Invalid/Unsupported do not become false. Grammar and
semantic edits remain guarded previews. Plan62 owns implementation and assembled acceptance;
the prior probes establish only their named interfaces. Formal ADR status is unchanged.
