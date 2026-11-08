# Local SurrealDB substrate

The canonical store is a Rust-owned application boundary. The host prepares/selects the exact
patched server and study module; an application owns actual launch, schema readiness, connection
lifetimes and physical settlement. Ordinary environment activation never starts a server.
The Python service supervisor, `just surreal` command, allocation-binding protocol and Python
product are retired. This guide describes retained native owners and custody requirements;
it does not claim that every historical supervisor workflow has a native CLI replacement.

The contracts belong to
[identity and publication](../authoritative_design/sections/identity-and-publication.md#section-20),
[workflows and results](../authoritative_design/sections/workflows-and-results.md) and
[operations and validation](../authoritative_design/sections/operations-and-validation.md).
Current acceptance and remaining H work belong to [Plan58](../plans/58-rust-owned-system.md).

## Setup and state ownership

Prepare the native host with `scripts/bootstrap-host`, then explicitly construct/select the
server with `scripts/pse-host server prepare`/`server select`. `just surreal-module` constructs
an immutable production study module; preserve its absolute receipt. Server source patches
and exact release provenance remain maintained inputs. Selecting an artifact establishes no
listener, canonical schema or scientific write admission.

`pse_operations::native_service::NativeCanonicalService` constructs only fresh private state
from `NativeServiceSpec`: a `PreparedServer`, explicitly built helper, prepared module or exact
historical module receipt, and an existing state parent. `NativeModuleSource::RecordedState`
reads that state's original module receipt; it does not adopt or dispose the old state.
The service verifies its own private filesystem, launch identity, endpoint and readiness before
issuing `SchemaReady`. A responding unrelated listener or public state path is insufficient.

Launch is explicit. `shutdown` drains independent canonical connections/maintenance and
HTTP ingresses, then stops the original server under one deadline. It attempts independent
settlements and retains failures. Process stop alone does not prove client/SQL settlement.
`try_into_stopped` transfers only an actually stopped owned state into `StoppedCanonicalState`;
`reopen` and `dispose` consume/recheck their own ownership. A refused transfer returns the
original owner. Drop and a printed path never establish successful disposal.

The private state contains credentials and immutable configuration/module/launch evidence.
Do not hand-edit it, initialize over user data, copy credentials into command lines or infer
adoption from a directory name. Rust fixture callers never require a user's persistent service.
External existing-service mutation is outside this fresh-state owner.

### Project and derived evidence operations

For routine project inspection, prepare a captured product image with
`pse observe prepare` and use its reader ID with `pse project`. `inspect-pack`
reads an existing pack offline. `metadata`, `show`, `branches`, `branch`,
`draft-show` and `study-workbench` connect to an existing deployment selected by
`--state`. They use the configured database unless `--database` selects another
initialized database. Continue bounded pages with the original cursor.

For installed definitions, prepare a Viewer reader with `pse observe prepare --installed-schema`
and use its reader ID with `pse project installed-schema`. The observation authenticates the
expected issuer and compares the normalized catalog and marker on the selected connection.
Configured rollout apply and recovery are deferred until a named native-owned persistent
deployment needs a recurring in-place schema change.

`pse project create` takes the project key and title. `pse project navigate` takes
the original project descriptor and navigation action as JSON on stdin or from
`--intent`. Both require the issuing store observed by a read and an original
`--operation` key. If the outcome is uncertain, `pse project acknowledgement`
looks up that same operation on the same store. An absent acknowledgement leaves
the effect unresolved.

`just project-build` explicitly constructs `pse-project`; `just project --help` exposes the
prepared operator. Select existing state explicitly and use its typed project documents.
Project roots, draft/save/admit/publication intents, source comparison, merge/undo and context
history retain their original authority, semantic source and head-generation/CAS contracts.
A durable save is not scientific admission. Preserve the exact intent across uncertain replies;
settle acknowledged original receipts before inspecting a later mutable head.

`just evidence-build` constructs `pse-evidence`. Its offline readers and independently disposable
index retain separate configuration/credentials and database ownership. A derived index does
not become canonical scientific authority or own original payload lifetime. Import/query cuts
retain source membership, coverage, cursor and freshness; a complete import can still represent
partial source coverage. Rebuilding/dropping this index cannot dispose original scientific
records, test outputs or protection owners. Use the prepared operator's actual help for supported
actions; there is no retired Python test-invocation registry to query.

### Selected logical project exchange

Native project exchange belongs to `ModelingPackage`/Runtime and the
[project workflow](operational-store.md). Explicit selected roots and their original admitted
closure determine export; no host filesystem scan expands that selection. Identity, physical
context, authoring documents and scientific admission remain distinct. Import never substitutes
a later latest source for retained ancestry. Same-object merge conflicts require a choice;
offset-only attribution is not automatically a scientific conflict.

### Exact lifecycle explanations and release

Release authority comes from the original canonical owner and its exact current predicates,
not a copied explanation or diagnostic link. Publication, retain/protect, retirement and payload
collection remain separate effects. Unknown acknowledgements require original-operation
settlement rather than replay under a new identity. Foreign authority, changed source or live
borrowers must remain protective. Logical removal does not prove returned physical disk space.

### Research versions, study workbench and selected evidence

Native research and study APIs retain selected versions, exact source roots, occurrences,
run/attempt identities and protected result manifests. Historical context uses UTC microseconds,
half-open effective intervals and an explicit recorded cut. Corrections dominate only within
their authored interval; a tombstone blocks selection without resurrecting an older value.
Reopening uses the original physical and authored closure, never current latest values.

Discovery returns bounded resident/canonical metadata and separate eligibility. It does not
compile, transfer providers, renew cache recency or reconstruct portable bodies. See
[the operational guide](operational-store.md) and the native research and study caller tests
for actual consumer scopes.

### Disposable development knowledge

`just workspace-sources --selection FILE` builds bounded source-only knowledge through native
xtask. Its explicit selection, coverage and bytes have no scientific admission authority.
The separate evidence index can be disposed by its own operator. Neither facility replaces
a source owner, execution capability or canonical scientific history.

Developer test results live in the performing workflow's working report while they are
needed. Select the current test or campaign to regenerate an expired result. A database
used to investigate selected evidence has its own input selection and disposal boundary.
Retain published scientific results and their authored provenance through the canonical
owner and use its explicit release and recovery operations.

Before creating a named development database, record the question it supports, the
selected sources or rerun inputs, the application that owns it, and its closeout condition.
Use an isolated logical database in an owned fixture for database contract and caller tests.
Keep fresh physical state for tests whose assertions include restart or recovery. Use a
separately owned derived index when a development question needs repeated queries over a
selected source set. Canonical projects and protected scientific results carry their own
publication, recovery and release requirements.

At closeout, retain the selected result and the authored inputs needed to reproduce it.
Close admission and settle readers and in-flight writes through the database's owner.
Use the original service owner for private fixture disposal and the canonical owner for
published results. Account for database bytes, payload files, and worker executable
generations separately. Recheck physical allocation after logical release. If a legacy
store lacks the original owner or a verifiable handoff, record its unresolved scope before
any release is considered.

### Selected-service diagnostics

Read native canonical client/store observations and original diagnostic checkpoints with
explicit state/operation selection. `just activity UNIT.service` observes manager/kernel facts
without starting or repairing the service. `just telemetry inspect` observes the separately
owned optional telemetry installation. Old Python storage-health/MCP command projections are
retired; native APIs and the server's explicitly selected own inspection tools remain.

Server/MX gauges describe their own source incarnation and interval. Version/commit counts can
fall during reclamation; approximate engine bytes are not RSS or reclaimed space. Zero pending
work does not prove SQL settlement or absence of a stale transaction. Read-only inspection
cannot grant initialization, maintenance, release or repair authority.

### Shared study policy module

The prepared study module is a pure policy artifact copied into service-owned private state.
The enclosing canonical transaction owns premises, mutation and immutable acknowledgements;
module policy does not perform SQL/network work or own scientific execution. Its exact receipt,
interpretation and hash are checked with generated declarations. Qualification probe modules
have separate identities and cannot be substituted for production. See
[explicit preparation](agent-environment.md#explicit-compilerklu-preparation).

## Endpoint and allocation

The native bounded canonical transport owns request clocks, authentication, values and physical
response lifetime. `pse_operations::transport::client` uses the pinned native protocol and value
codec; SQL requests remain owned through final statement/transaction completion. Incremental
rows are provisional until finality. Error, cancellation or truncation cannot become successful
stream exhaustion. A client timeout does not prove an already submitted mutation absent.

The transport's SQL requests and bounded outbound/response custody remain Rust-owned.
Finite encoded frames (4 MiB) and aggregate response admission protect different populations.
Protected page/block reads retain exact guards and original clocks; decoded Arrow buffers retain
their allocation independently of connection/read lifetime. Credentials and raw SQL-bearing
errors stay private; public diagnostics retain classification without reflecting secrets.
No Python request loop or SDK is required to preserve these contracts.

The server-side control actor is a distinct narrow permission boundary.
`CanonicalStore::issue_control_actor` explicitly provisions an actor on an already initialized
store with separate browse/cancel grants; existing keys refuse instead of overwriting credentials.
The prepared `pse-canonical-init --issue-control-actor PRIVATE_JSON STATE` consumes a bounded,
private owned versioned document containing key/password/grants. Provisioning is not ordinary
schema installation, and passwords never belong in argv. Server SQL access definitions and
guarded actions own authentication/authorization; a supplied actor label cannot grant authority.
Live browse pages retain predicate/issuer-bound cursors rather than frozen completeness.
Cancellation retains actor/store/operation identity and an immutable decision; reconcile the
original request after uncertain delivery. A cancellation receipt revokes future claims but
does not prove native worker completion or process drain. Tests in
`crates/pse-operations/tests/native_service/http_control.rs` exercise actual native ingress.

Application placement remains explicit under the native host. Ordinary configured unbounded
work has no new memory cap; finite/scientific/reference/timing profiles retain their declared
physical limits. A service and a scientific worker have distinct owners. Placement and database
readiness are not interchangeable capabilities. `--store`/`--bind-allocation` are retired, not
no-op flags. A path or environment marker cannot materialize storage, a worker or finite charge.
Linux user-systemd and actual original kernel containment remain premises; broader deployment
and timing acceptance are separately owned.

## Managed native workers

An application constructs `NativeStudyWorker` from its actual `SchemaReady`, explicitly built
`pse-worker`, selected environment/materialized allocation and worker settings. Retain the
performing owner before `launch`; scientific callers receive its identity-pinned observer via
`Runtime::with_managed_worker`. Launch/observation never builds a worker. The private native
handoff replaces the old Python receiver context. After canonical conclusion, join native lanes
and final transport work, then physically settle the original unit. An observer disappearing or
a worker result becoming terminal does not prove its process forest empty.

## Quiesce, stop and backup

Close new write admission before draining native work. `NativeCanonicalService::shutdown`
settles retained client/ingress owners before the dependent server stop; `stop_process` can
physically stop the original service without database connectivity but does not invent SQL
finality. A manager stop job or direct PID exit is insufficient: original cgroup population
and process incarnation must actually drain.

The retired Python quiesce/backup/inspect-backup commands have no equivalent high-level CLI
here. Retained native canonical file recovery primitives remain usable only under their actual
application freeze/maintenance owner. An offline backup must include exact database/configuration,
credentials, interpretation/module identity, application inventory and required sealed/staging
payload closure. Publish completion last; an interrupted copy is not a ready backup. No live
engine-directory copy or separate WAL/data-file copy establishes a consistent application backup.
A destination must be a new/empty owned path outside live state. Protect backups like credentials.

Original unresolved intents, collecting tombstones and unsealed bytes retain their uncertainty;
copy integrity is not a seal acknowledgement or scientific validity. A missing required file
refuses complete backup. Physical drain, database export completion and full application custody
are separate facts. Do not promote earlier supervisor fixture results to acceptance of a new
Rust application backup composition.

## Payload placement and owner recovery

An admitted complete encoding through 64 KiB inclusive is inline before any file reservation.
Larger results use complete uncompressed IPC files packed within an attempt/relation/schema/
interpretation, targeting 16 MiB with a 32 MiB hard bound and flushing at demanded acknowledgements.
The 2 MiB append target, 128 KiB index and 4 MiB RPC limits have separate purposes. Empty demanded
relations use explicit closure entries, not invented empty payload bodies. Pressure does not
silently change placement. Opaque JSON seed/progress payloads retain their declared encoding.

Catalog acknowledgements bind exact file instance, digest, batch ordinal, codec and schema.
Directory enumeration is not a result snapshot. Manifest tickets protect future unread members
without lease expiry; logical close prevents new admission while admitted continuations remain
protected. Connection close waits for file effects/read drain. Copied Arrow arrays retain their
accounted memory after the reader/runtime closes. Finite creation deadlines do not shorten an
already admitted reader's lifetime.

`pse_operations::canonical_files` owns publication, integrity, locks, protection, collection and
offline primitives. Explicitly build its product-only physical helper when needed:

```bash
scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness build \
  -p pse-operations --bin pse-canonical-recovery --features canonical-maintenance --locked
```

The helper's `--operation`, `--root`, inventory, placement/destination and checkpoint inputs
are physical recovery interfaces, not application freeze/adoption authority. It binds one exact
inventory and rechecks physical bytes on continuation. Canonical framed digests and a backup's
SHA256 copy inventory are distinct layers. Source/file identity, no-replace publication and
instance locks remain held through actual effects. Live/unprovable owners remain protective;
collection serializes with publication/recovery and rechecks state before unlinking. Collected
instances cannot revive. Logical disposal never implies reclaimed-byte timing.

## Restore and validation

Restore requires a fresh owned destination, complete admitted inventory and an explicit
maintenance owner. Never overwrite a live application or infer target admission from copied
bytes. Identity rebinding, installed-definition validation and schema activation are distinct
steps. Pending/unacknowledged operations retain their original state until their native owner
settles them. Restoring files does not resume numerical work or establish historical compatibility.
The physical helper does not recreate the retired Python restore supervisor.

## Explicit design-phase reconstruction and recovery

`SchemaMaintenance` and native migration capabilities retain exact initializer ownership,
predecessor/target interpretation, generated declaration/codec identity and module hash.
`validate_definitions`/`validate_definition_files` produce actual witnesses;
`activate_definitions` independently checks the target before activation. Phase-only witnesses
cannot activate a product. Ordinary open never initializes or repairs a schema. Unknown submitted
SQL requires original finality/settlement; physical child drain alone cannot establish its effect.
Old-state reconstruction needs its explicit owning decision and native caller, not a revived
Python lifecycle command. Frozen forward schema publications remain append-only producer inputs.

## Targeted fixture controls

| Assertion boundary | Fixture and lifetime |
|---|---|
| Generated schema, SurrealQL transactions, and database-level acceptance or refusal | Use the pinned SurrealKit suite with authored setup and per-case logical databases. The native test owns the endpoint and settles and disposes it after the suite. |
| Rust caller, transport, or scientific behavior against an isolated logical database | Select a runner-borrowed fixture. The command owns the service and gives each test an isolated database in the original endpoint. |
| Physical restart, recovery, or service lifecycle | Select a target-owned fixture. The test keeps the original native owner through the observed stop, reader drain, and disposal. |
| Authored projects, published results, and operator recovery | Use the explicitly selected canonical owner. Preserve the original admitted state, exact provenance and any live or recovery holds. |

`scripts/pse test` admits source build/list through the checkout's prepared correctness adapter
and resolves the selected tests' fixture declarations. Borrowed cases delegate that exact captured
compiler context to `NativeTestCommand::from_source_capture` before creating a fresh
`NativeCanonicalService`; execution does not build/list again. Target-owned cases receive prepared
immutable inputs and retain their original service owner. Pure selections prepare no fixture or
unused helper. See [qualification commands](validation-assessment.md#individual-commands) for
selection and explicit module inputs. Source admission belongs to the host performing owner;
the fixture owner validates the private capture and runs the selected context. Caller-supplied
archive/metadata and additional run-time build selectors are rejected.

`FixtureSelection` validates bounded private descriptor bytes, owner/mode, original state device/
inode and disposable nonce. Children borrow isolated UUID databases only after service readiness.
The descriptor gives no shutdown/disposal capability. Normal selected child exit and successful
native settlement are separate disposal premises; signal/timeout and uncertain scientific joins
retain the original state. Real private unit/process, connection finality and custody controls
must run on their actual supported host. The command's existence is not full assembled acceptance.

### Installed external server reuse

Host release import admits an explicitly matching established server source release into a new
immutable generation. It checks original recipe/source/patch/profile and copied lock/binary bytes;
it never fabricates a new compilation receipt. It starts no server and adopts no old state.
A fresh native service can use the admitted artifact while retaining its own listener, credentials,
module/schema and physical ownership. Existing installations and unrelated services remain intact.

## Standalone beta tooling fixtures

Pinned native schema tooling and Rust fixture sources retain their producer-specific interfaces.
The SurrealKit generated-schema suite runs as an explicit selection in the native-service test
target. Explicit installation builds the source revision selected by
[`.config/surrealkit.toml`](../../.config/surrealkit.toml) against the standalone gRPC SDK pin
and the reviewed lock adjustment. It publishes the binary, Cargo lock and release receipt
together in the versioned project data root. Repeating the install verifies and reuses the same
release. Set `PSE_SURREALKIT_BIN` to select an explicit absolute path to an equivalent verified
release. Tests verify the preparation and never build the tool.

The existing server and study-module preparations supply the owned endpoint. The test runs
the current generated schema and authored seed in scoped logical databases. It separately
compares sync and rollout replay using the retained rollout's frozen target, then verifies that
a deliberate schema drift is reported. The fixture settles and disposes the endpoint even
when a suite assertion or setup fails.

```bash
scripts/pse-env -- scripts/pse diagnostic-tools install surrealkit
scripts/pse diagnostic-tools verify surrealkit
scripts/pse test pse-operations:native_service -F pse-operations/native-service \
  --ignored only -E 'test(=surrealkit::surrealkit_generated_contract_uses_owned_service_and_disposes)'
```

Historical Python beta lifecycle/persistent-fixture drivers are retired. Do not run their old
commands or infer a new generic lifecycle adapter. Whole-chain migration qualification needs
exact ordered producer publications, acknowledgements, initializer reservation and independent
activation, with original unknown SQL preserved after interruption. Source-level/phase controls
and older beta fixture passes establish no arbitrary-data migration or new persistent journey.

## Capability choices and revisit triggers

Retain typed attributed/enforced relations, explicit guards, persisted readiness/CAS and
owner-driven cleanup. Notifications, feeds, timers and storage TTLs are not release authority.
Finite protected reads remain the selected transport; consider streaming only for an actual
consumer and compare finality, cancellation, memory and protection through returned buffers.
Bulk payload custody belongs to sealed files; engine history/compaction cannot grant disposal.
Queries/indexes remain generated declarations with exact selection/ordering/cursor semantics;
small output or LIMIT does not prove bounded examined work. EXPLAIN FULL executes work and needs
an effect-safe owner. Optional Studio/collectors/containers do not replace canonical admission.

Local retry, seed transfer, partial trajectories, backups and history are not durable numerical
checkpoints. Resume or distributed/remote execution requires an explicit scientific state and
compatibility contract. Revisit optional mechanisms for a named consumer or measured limitation;
none of these retained choices claims general capability absence or representative performance.

## Disposable tests and retained evidence

| Purpose | Owner and closeout |
|---|---|
| Database contracts | SurrealKit suites specify schema, seeds, actors and assertions. Use isolated logical databases on a test-owned endpoint. The native fixture verifies absence or disposes the endpoint after drain. |
| Application or physical lifecycle tests | Rust owns service, clients and state. Build the smallest authored input that proves the transition. Reopen through the stopped-state capability when the assertion needs persistence. A physical copy belongs to a selected backup or restore assertion. |
| Developer evidence | Read the original working report and inventory while the invocation owns them. Regenerate selected cases from source when the report has been released. Keep any current comparison bounded to its declared selection and lifetime. |
| Durable scientific or operational data | The canonical store and the original publication or deployment owners decide release using their actual holds, borrowers, recovery requirements and settlement. |

The fresh Rust fixture owns only directories it created. A failed assertion can still leave a
settled fixture eligible for disposal. Uncertain joins or state identity retain the original
owner and the resources needed to reconcile it. A process exit, old timestamp or filesystem
size cannot establish that authority. Record physical allocation independently of logical
deletion. [Invocation evidence](agent-environment.md#choosing-tests) ·
[Allocated storage](agent-environment.md#inspecting-allocated-storage)

## Typed scientific inspection

Use an already admitted native Runtime and `ModelingPackage` for scientific source/run/attempt
identities. `source_tables`, `run_record`, `attempt_record`, `result_manifest`, `results` and
`resource_usage` expose their existing typed boundaries. Keep run/attempt distinct, retain exact
keys and respect protection/allocation lifetimes. Source export intentionally materializes one
package inventory, not a database scan. These reads perform no model solve or schema setup.
The [native workflow](native-workflow.md), [operational guide](operational-store.md) and
[diagnostic guide](simulation-diagnostics.md#typed-inspection-and-reader-ownership) explain the
corresponding scientific, retained-data and copied-observation scopes.
