---
title: "Plan57g: Host files, process evidence and maintenance"
status: draft
date: 2026-10-27
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, libraries, environment, storage, design, validation]
review_sources:
  - docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md
  - docs/library_leverage/reviews/host-processes-and-durable-files.md
  - docs/library_leverage/reviews/canonical-store-transport.md
  - docs/library_leverage/reviews/mathematics-preparation.md
---

# Plan57g: Host files, process evidence and maintenance

This companion develops publication (CBO-003), shared child supervision (004), process facts
(019), primary observation (020), Python ledger discipline (047) and maintenance output inquiry
(053), while retaining CBO-048's host/service policy. The
[coordinator](57-library-leverage-and-correctness.md) owns aggregate selection and dispositions;
this document owns the pending packages below. Drafting does not authorize host cleanup,
placement changes, maintenance workloads or production implementation.

## Database-boundary review amendment

K/L's [hybrid target](57k-durable-control-and-python-boundary.md) consumes G's existing
host/file owners. Extend G03/G04's consumer map to the selected control ingress, worker
admission and service endpoint configuration. Storage controller and receiver generations
remain distinct; bootstrap/listener authentication, exact program identity, partial-startup
settlement and process-group drain retain their performing owners. A committed database
record is not a kernel liveness or join witness. Use the corrected storage-controller routing;
do not recreate the audit's superseded receiver/storage conflation as a current defect.

L04 owns payload-placement selection; G01 supplies any surviving external publication protocol.
Do not implement a file-helper rewrite that L04's selected atomic database transition eliminates.
Where files remain, preserve sync/no-clobber/intent/attachment and reader custody. Object_store
range/stream primitives are eligible delivery mechanisms but do not supply canonical catalog
transactions or all local durability semantics. A selected change integrates producer, reader,
recovery and protected retention together; K05 consumes its data capability.

Any new HTTP/WS/API ingress must be integrated into the authenticated existing deployment,
readiness/stop/backup and exact owner cleanup routes, not added as an untracked server. J uses
actual partial-startup and drain scenarios. This amendment schedules integration, not host changes.

The first K/L qualification exposed two concrete G03/G04 follow-ups. A native integration
build can replace the mutable allocation helper with a native-linked binary before a pure
storage closure freezes it; qualification recovered by explicitly building the pure helper
with `just deployment-build`. Assess exact helper selection at closure construction rather
than widening the storage environment. Also, direct maintenance-listener drain can enter
`stop` and recursively remove its own marker, making the outer unlink fail after physical
settlement. Keep normal `stop` as the composed teardown owner for the selected K/L fixtures;
G's remaining lifecycle work must reconcile the direct API's ownership. The failed fixture
was recovered through existing dead-owner/reset/stop APIs and independently witnessed drained;
no ownership guard or retained database was deleted. These observations schedule G inquiry,
not a claim that its general lifecycle scope is repaired.


## Foundation assessment

The complete 2026-10-27 G1/G3 review and supporting accounts at product source `4f52fd239`,
plan baseline `93946f572`, additional patterns revision 4, supply inherited evidence. Core 3.4,
Efficient Architecture 1.0, test assessment 1.2 and the pse-arrow binding govern this focused
assessment. Process-simulator 1.5 does not introduce new numerical scope here; retaining native
resource custody and truthful interrupted outcomes supports its consumers. Enduring contracts
remain blueprint §20/§20.6, §21/§23.1 and the environment/substrate guides.

The current foundation is stronger than a universal "replace polling with handles" account
suggests. Python already holds authenticated cgroup identity and reads hierarchical `populated`
for drain; systemd owns units and Python alone owns admission, borrower and ledger policy.
Admission must operate before a Rust extension or virtual environment exists. Ordinary host
work currently selects `PSE_BUILD_MODE=unbounded`; 16-lane/reference fixtures and explicitly
selected finite allocations retain their separate meanings. No package may add a CPU/RAM cap,
serial execution or mandatory managed placement as a side effect of consolidation.

Source inspection confirms the narrower issues: canonical publication still creates a
hard-link/unlink alias window; review-pack `fail_at` deliberately links an acknowledged failed
prefix while retaining staging; `host_metadata` already owns private-file/lock mechanics but
publication varies among scripts; `maintenance_sql` captures complete output with a 120 s
timeout, while copy control logs are captured without a local subprocess timeout and the actual
database is exported through an owned file. Neither observation establishes a runaway command
or the complete enclosing lifetime. These distinctions determine the package boundaries.

Under AP-01/AP-02/AP-04/AP-06 and DP-19/DP-20, effect ownership stays with the owner holding
the charge. Under AP-07/H18/H21/H23/H25/H26/H27, remove repeated parsing, heartbeat interpreter
starts and publication decisions where their semantics can be preserved. A pidfd proves one
process identity/exit, not descendant drain; a durable receipt is not a live handle; and a
manager-created unit must be controlled through the manager unless that exact unit delegated
the relevant subtree. No universal alive predicate or second resource ledger is introduced.

## Publication and durable Python state

### Named and anonymous no-replace publication

One `pse-operations` helper owns the final named publication step:
`publish_no_replace(staging_dir, staging_name, final_dir, final_name, durability)` using
`rustix::fs::renameat_with(..., RenameFlags::NOREPLACE)` at the inspected 1.1.5 pin. Caller
data synchronization precedes Durable publication; the helper renames then fsyncs the final
directory and the staging directory if different. Visible publication only establishes atomic
no-replace visibility. `EEXIST` includes symlink/directory destinations. Supported filesystem
conditions and no-follow/held-descriptor validation remain explicit; do not substitute a
replace rename when the no-replace capability is absent.

Canonical sealed files, IPC export and study workbench export use Durable; the latter two gain
the directory sync absent today. Capture acknowledgements, review-handoff members and successful
review-pack member publication use Visible with that diagnostic durability limit stated at the
call site. `.pending/<instance>.pending` and `.incomplete` names and prefix-resume comparison
remain domain protocols. Before-rename failure leaves meaningful staging in place. A successful
rename followed by a directory-sync failure leaves the final name visible and staging absent;
return a distinct published-but-durability-uncertain outcome. Retry/recovery authenticates that
final inode/content and completes required synchronization through the original owner; it must
not recreate/overwrite a destination or infer that every error permits prefix resumption.
Catalog acknowledgement remains after established durability. The review-pack failure
path remains a no-clobber hard link because it intentionally exposes a second readable name for
the acknowledged prefix while keeping `.incomplete`. It must not use the successful-rename helper.

Offline anonymous copies keep `O_TMPFILE`+`linkat` and inode/parent synchronization. `xtask`
producer history keeps `persist_noclobber`; compiled content-addressed caches keep intentional
replacement semantics. No RAII random temporary-file owner replaces resumable/interrupted
staging. `cap-std`, object-store filesystem adapters and replace-only atomic writers would
weaken the consumed contract. The existing 5 ms try-flock absolute-deadline wait remains:
a blocking lock could not become promptly cancellable merely by moving it to a blocking thread.
Held-descriptor O_NOFOLLOW, single-link/alias refusal, digest, cancellation/extent checks and
catalog acknowledgement after directory sync remain local canonical obligations.

New rename publication cannot create the legacy two-link crash state. That does not prove
existing stores lack it. Before deleting live `settle_published_alias` or offline `reconcile`,
perform a read-only inventory of all actual stores in the selected operational scope, including
pending/final device+inode+link identity and uncertain/inaccessible stores. Exact legacy aliases
settle only through the original owner under its lock and existing validation. Inventory grants
no deletion authority. Until every selected store is settled or explicitly outside the supported
migration scope, retain legacy-only recovery and its owner contract tests. Delete obsolete
mechanism tests only after that support obligation ends; preserve no-clobber, interrupted-prefix,
resume, digest and crash-order assertions as independent contracts.

### Shared stdlib ledger mechanics

Extend `scripts/host_metadata.py`; do not add a third-party bootstrap dependency. It owns private
same-directory random `O_EXCL|O_NOFOLLOW|O_CLOEXEC` 0600 temp creation, complete write,
`os.replace`, optional data/directory fsync and writer cleanup in `finally`. Its protected-directory
operation validates ancestors and syncs newly created ancestors when the caller requests
durability. A private read holds its O_NOFOLLOW descriptor through owner/mode/type validation.
The caller declares durability: hot admission ledgers and existing pin records keep their
current non-power-loss-durable semantics; durable telemetry or server metadata receives required
directory synchronization. Unknown/malformed state continues to fail closed.

Shared locked operations retain absolute deadlines and caller-selected poll policy; consolidate
mechanics without changing 10/20/50 ms or blocking semantics merely for uniformity. Lock order
(invocation → identity → resource) and native-operation signal masking remain owner policy.
Keep JSON bytes/digest encoding unchanged: share `canonical_json_digest` only where current
`json.dumps(sort_keys=True)` defaults are byte-identical. Do not normalize separators or encodings
of persisted identities.

Every temp has a per-ledger prefix. Lock-holder sweep is allowed only after all writers for that
ledger/directory are shown to hold the same lock. `operation_lifetime.write_json` also serves
compiler/native caches; their lock coverage cannot be assumed. Without that proof use only
writer `finally` cleanup. With it, the next lock holder can remove authenticated stale sibling
temps left by a killed writer. The review's 22 abandoned temps are inherited snapshot evidence,
not an instruction to delete current host files. The two Python no-clobber artifact publishers
may retain stdlib link/unlink through a shared helper; ctypes renameat2 is unnecessary here.

## Process facts, supervised children and primary observation

### Process facts have an owner, identity keeps its meaning

Add one Rust process-fact reader in `pse-diagnostics` using exactly pinned `procfs-core =0.18.0`,
`default-features = false`. Local file opening and consistency checks surround the library's
I/O-free Stat/Status parsing. Return present, absent, unavailable and changed explicitly. Use
raw `Stat::starttime` ticks, state/pgrp and actual VmHWM/PPid units; never use the clock-derived
`starttime()` convenience method for identity. Context7's procfs descriptions disagree about
VmHWM conversion. Exact 0.18.0 `Status::parse_with_kb` source strips the suffix without multiplying,
and its [Status field contract](https://docs.rs/procfs-core/0.18.0/procfs_core/process/struct.Status.html)
states kibibytes. The reader therefore converts KiB to bytes once with checked multiplication
when a byte-valued caller consumes it; fixed known byte/tick fixtures protect that boundary.
The [procfs account](../library_leverage/libraries/procfs/README.md) names the source
and limitations. Dependency/lockfile resolution is a normal implementation obligation, not a
reason to reopen the selected parser at authoring time.

Persisted identity remains `(boot ID, PID namespace, PID, start ticks)` byte-compatible across
Python and Rust records, including lifecycle-owner and canonical file-owner metadata. Act on a
recorded live process only through a post-open verified pidfd. A PID namespace mismatch is
uncertainty, not proof that the recorded owner died. Python's `process_observation.py` stays its
existing leaf. Effective cgroup limits still require the ancestor walk; procfs does not provide
that sysfs operation. PSI samplers stay as they are. A pidfs inode replacement would require a
metadata migration and decision route without improving the present guarantee, so is not selected.

### Shared process-group supervision

`pse-diagnostics::SupervisedChild` is the shared synchronous owner consumed by
`pse-math::native_compiled` and runtime capture/review launch. Spawn a new process group and hold
a pidfd to the unreaped leader, reserve the PGID with WNOWAIT, and retain output-reader ownership.
An optional bounded stderr reader serves the compiler. Cancellation eventfd and absolute
deadline participate in the existing rustix poll loop. Drain kills the group, inventories members
through the shared reader, opens each pidfd, rereads/verifies identity and exit readiness,
signals/waits and reinventories before finally reaping the leader.

Failure policy is explicit: compiler Park retains prepaid capacity and its temporary output
directory when drain cannot be proved; capture/review Refuse retains the current 3 s diagnostic
drain bound and retries on drop without pretending to hold compiler capacity. The type promises
owned process-group drain, not all descendants: a `setsid` escape is outside that proof. The
enclosing registered host scope remains populated and keeps host capacity charged. The compiler's
deterministic flags/prlimit and the direct-child supervisors for interpreted native child,
sampler and worker control remain unchanged. A command-group/process-wrap `ECHILD` result and
numeric PGID alone are insufficient replacements for this proof.

Delegated child-cgroup containment is a conditional strengthening of the same owner, not a
mandatory placement change. A separate decision must choose whether to keep bare/REPL/editor
workflows or require managed placement. The selected conservative branch preserves those
workflows with process-group fallback. Authenticate delegation using the exact unit's manager
`Delegate` property or an explicit handoff bound to that unit, invocation and cgroup inode.
Ancestor xattrs, ownership and mkdir success do not authenticate it; systemd 255's user manager
does not emit the delegation xattr. Without authenticated delegation, never write systemd-owned
cgroup attributes.

If authorized, enable Delegate on selected scope creation paths, create controller-free child
leaves within the existing charged scope and move only spawned children before exec. Supervisor
stays at the allocation root; `subtree_control` remains empty. Kill via `cgroup.kill`, await
inode-bound `populated 0`, then remove the leaf. Preserve the worker's memory-limit placement
walk and make activity/pressure recursive membership reads consistent. The inherited delegated
probe demonstrates this arrangement and setsid/double-fork drain under its one-run Linux conditions;
it does not authenticate arbitrary current units or make hostile self-migration contained.

### Primary receiver observation without routine interpreter starts

Python keeps ensure-primary, admission, unit lifecycle, receipts/readiness and drain-for-release.
Rust receives a versioned authenticated handoff including unit, InvocationID, cgroup path/inode,
and separately identified main supervisor and actual receiver worker with boot/namespace/start
identity. No PID field ambiguously means both. Pin cgroup identity, open the selected pidfd,
verify process identity and cgroup membership, recheck inode and non-exited readiness. Namespace
mismatch or specifically unsupported evidence retains the existing authenticated observer
fallback; identity mismatch refuses ready status.

Routine heartbeat observation reads held kernel evidence in-process. A pidfd exit is a wake hint:
the supervisor can exit while the receiver or descendants leave the group populated. Only the
authenticated group-lifetime decision establishes drain. Keep changed-owner, drained and
uncertain outcomes distinct until the owner maps them; retain the final canonical inventory
reread before returning study-unavailable. Receiver observation stays at the admitted heartbeat
(normally 10 s), independent of inventory polling (100 ms). A manager failure or unobservable
group grants no release authority. The narrow observer/manager fallback remains until all its
current semantics have a working replacement; `observe-primary` and its owner tests are not
promised deletions. The one systemctl query per file-session registration also remains because
InvocationID is not available in this host's user-unit xattrs.

## Maintenance output is an inquiry before a policy

CBO-053 requires a demand inventory, not a universal output cap or timeout. Trace each
`maintenance_sql` and `copy_current_database` consumer through its enclosing deadline/cleanup.
Separate complete nonce acknowledgements, potentially large complete catalog results and
export/import control logs whose payload is already an owned file. Record required stdout,
stderr, final framing, redaction and retained diagnostic tail independently. Full capture followed
by tail truncation does not bound live memory. Absence of a local copy subprocess timeout does
not prove the whole operation unbounded; long restores may be intentional.

For acknowledgements/catalogs, select either incremental bounded decode under an operation-shaped
grant or a private admitted output file with a separate decode allowance. Never truncate into
an apparently valid nonce or incomplete catalog success. For copy logs, discarding success
output with bounded retained failure tails or incremental drain is credible; drain both pipes
concurrently, including unterminated chunks, so one full pipe cannot deadlock the other. Retain
nonce validation, redaction before publication, exact original deadlines, export device/inode
binding and actual listener/descendant drain in finally. A CLI timeout/direct-child wait alone
does not prove maintenance-group drain. Select among retain-capture, incremental drain and
admitted-file routes only after that inventory; no runaway diagnosis is assumed.

## Packages and dependency readiness

This table alone owns local package state; the coordinator owns CBO dispositions.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| G01 | pending | Plan57g | Named publication preserves no-clobber/staging/durability and accounts for legacy aliases. |
| G02 | pending | Plan57g | Python ledgers consume shared private publication/lock mechanics with proven sweep scope. |
| G03 | open — A–C prerequisite tested; remaining consumers planned | Plan57g | Rust process-fact reader and group supervisor replace duplicated parsing/drain mechanics. |
| G04 | pending | Plan57g | Authenticated primary handoff supplies in-process observation with guarded fallback. |
| G05 | pending | Plan57g | Maintenance demand/lifetime inquiry selects a justified output policy and acceptance. |
| G06 | pending | Plan57g | Selected maintenance output changes integrate and host/file journeys hand off. |

<a id="g01"></a>

### G01 — No-replace publication and legacy crash states

**Prerequisite:** publisher durability and staging meaning classified above; no other companion
must finish. Add the helper, migrate successful named publishers and test EEXIST for ordinary,
symlink and directory names, cancellation with readable incomplete prefix, cross-directory
sync order and catalogue acknowledgement after sync. Preserve review-pack failure hard links
and offline anonymous publication. Read-only census precedes any legacy-recovery retirement;
unknown stores keep that support obligation open. Delete replaced successful hard-link/unlink
pairs and their implementation tests after passing replacement cases. Delete alias recovery
only after its inventory/settlement requirement is satisfied; retain original no-clobber,
resume, held-descriptor and failed-prefix owner contracts. **Targeted acceptance:** local
filesystem tests plus failpoints after rename and each directory sync, including authenticated
retry of a visible but unacknowledged final name; actual store/crash recovery belongs
to assembled qualification, not an assumed implication of syscall choice.

<a id="g02"></a>

### G02 — Private stdlib publication and lock discipline

**Prerequisite:** actual writer/lock/durability/encoding inventory, especially compiler and native
cache use of operation_lifetime. Extend host_metadata and migrate resource_store,
operation_lifetime, surreal_server, telemetry and host-admission callers. Delete their replaced
publishers/lock loops and the duplicated host_admission durable split, keeping durability flags
and caller timing explicit. Sweep only authenticated prefixes under proven common locks.
**Targeted acceptance:** killed writer before replace then next locked writer, simultaneous
writers, partial write, mode/owner/symlink refusal, deadline-expired lock, nested lock order,
native signal masking, durable directory/ancestor sync and byte-identical recorded identities.
Use disposable fixture directories; no current host-state cleanup follows automatically.

The historical 24 stale empty allocation slices also have a bounded G02 inquiry outcome.
Trace creating/retiring exit paths and reproduce the condition on disposable units/ledgers;
determine whether existing owner cleanup covers it or select an owner-local correction.
Reconciliation may retire only an authenticated empty kernel group with no ledger ownership
under the admission owner's lock. Unknown/live/foreign units refuse cleanup. That old snapshot
is neither proof of a current leak nor authorization to sweep the current host.

<a id="g03"></a>

### G03 — Process reader and reusable group-drain owner

**Prerequisite:** caller identity/outcome contract reconciled and independent source-unit cases
for the resolved KiB-to-byte mapping. Add reader, migrate the reviewed stat/status consumers in canonical files,
initializer/offline ownership, observation targets/peak, native child, allocator probe and xtask,
then consolidate CompilerChild and capture/review drain. Preserve live pidfd post-open recheck.
**Targeted acceptance:** comm with spaces/parentheses, maximum ticks, malformed/truncated stat,
VmHWM units, absent/denied/changed process, namespace refusal, member fork during drain, unreaped
leader PGID reservation, stop/deadline, bounded stderr and distinct Park/Refuse policies.
Delete hand parsers and superseded inventory/drain blocks and their implementation-only tests;
retain owner tests for unknown-drain resource custody and diagnostic refusal. Deliver the
working SupervisedChild capability to Plan57b's compiler work without waiting for G04–G06.
The selected A–C execution delivers the shared reader and supervisor with compiler and
capture/review consumers first. The remaining reader migrations listed above stay open here;
this useful prerequisite does not close G03. The shared owner preserves original-clock
checks, independent procfs observer-view and post-open pidfd identity checks, WNOWAIT leader
reservation, re-inventory after group signalling, bounded stderr and Park/Refuse custody.
Process-group drain still does not establish containment of descendants that escape the group.
The selected prerequisite's functional acceptance includes eighteen diagnostics controls in debug
and release, twelve actual capture launcher controls and nine compiler-child controls. The
[57j Outcome](57j-qualification-and-publication.md#selected-a-c-outcome--2026-10-27) owns current
composed evidence and original-owner settlement. This does not qualify the remaining G03 readers
or the G04–G06 host/maintenance scope.

Conditional containment has no selected implementation package until its placement decision;
it cannot hold this useful consolidation hostage or silently replace the retained fallback.

<a id="g04"></a>

### G04 — Kernel-backed primary observation

**Prerequisite:** G03's reader and a versioned supervisor/runtime handoff within the current
ADR-0229 scope, checked before changing its protocol. Add the inode-bound cgroup-lifetime leaf,
extend ensure-primary handoff and use held evidence in ManagedPrimary. Retain narrow manager
fallback for namespace/unsupported cases. **Targeted acceptance:** changed inode/invocation,
PID reuse attempt, supervisor exits with a populated receiver group, receiver exit with an
unrelated remaining descendant, unavailable manager/kernel evidence, namespace guard and final
inventory committed between the first read and observed drain. Preserve no-release-on-observe
and current heartbeat/cancel timing. Remove routine interpreter/JSON overhead only where the
same contract is settled; retain observe-primary and owner tests while fallback remains. Supply
Plan57f study waits with explicit observations, not ownership of unit lifecycle.

<a id="g05"></a>

### G05 — Resolve maintenance output and lifetime demand

**Prerequisite:** read-only access to actual maintenance callers and current controller policy;
no server startup or provider build is an authoring prerequisite. Produce the acknowledgement,
catalog and log classification, required complete output and full enclosing clock/cleanup
analysis. Choose each output route with its allocation owner and failure behavior. If complete
catalog bounds or long-restore lifetime remain unclear, keep that consumer pending and retain
capture; do not choose an arbitrary cap to mark the package done. **Targeted acceptance of the
decision:** every consumed output has an explicit completeness/redaction/lifetime requirement
and selected realization, with uncertainty tied to the dependent consumer. This is an inquiry
deliverable; it makes no tested or measured runaway claim.

<a id="g06"></a>

### G06 — Implement selected output ownership and hand off assembled journeys

**Prerequisite:** G05's settled per-consumer requirements and working G02 private-file primitives
if files are selected. Use stdlib subprocess capabilities; no second supervisor. Replace only
the selected capture paths, removing obsolete complete-memory buffers and tests specific to
that mechanism while retaining nonce/export-identity/deadline/group-drain owner tests.
**Targeted acceptance:** disposable CLI peers emit large simultaneous stdout/stderr,
unterminated chunks, truncated/duplicate/wrong nonce, partial catalog, credential-shaped errors,
cancel/timeout during output and export inode replacement. Assert no falsely complete result,
redaction and original listener/descendant settlement before release. Plan57j later exercises
actual maintenance against disposable owned deployments and the original backup/restore paths.

## Retained service policy, alternatives and rule route

CBO-048 retains Python's stdlib `systemctl show`/`systemd-run`, original release authority,
borrower accounting and kernel population evidence. A D-Bus client would need a cold-start
fallback and create duplicate paths; direct Rust transient-unit creation adds another ledger
writer and moves effects outside the charged unit. Manager-unavailable reports remain possible
and unknown authority remains protective. No replacement admission daemon or universal alive
predicate is selected. The inherited 24 stale empty allocation slices indicate a bounded
reconciliation inquiry; trace exit paths and require empty kernel group plus absent ledger
ownership under the admission owner's lock before proposing removal. No current slice cleanup
is authorized by that old snapshot.

CBO-048's SurrealDB readiness/physical-backup retention is owned in Plan57e. Logical export
cannot replace offline state and payload inventory verification; G05 concerns CLI output
ownership, not that architectural division. Reopen D-Bus observation if bootstrap ceases to be
stdlib-only or a measured manager-call cost has a concrete consumer; reopen event-driven
`POLLPRI` waits for a demonstrated latency need while retaining authoritative reread/fallback.

Consolidation and procfs dependency additions require no new ADR. Mandatory managed placement,
persisted identity changes or changed resource/release contracts would use the coordinator's
explicit operator disposition and existing ADR/design route; none is accepted here. Optional
delegation requires the environment guide and exact launch-owner changes, not an architecture
edit in passing. The U1 conditional upstream plan does not need a rustix convenience wrapper
or stable std pidfd contribution to implement these packages, and authorizes no contact.

## Verification and checkpoint

**Proposed:** G01–G06 and the conditional containment branch. **Interface-checked:** current
publication/maintenance/lock owners and cited library APIs; inherited filesystem/cgroup probe
claims keep their recorded conditions. No build, provider preparation, process probe, host
cleanup or maintenance operation ran during authoring.

During execution use targeted `just check-package`/`just unit-package` for changed Rust packages,
with explicit force-validation and selected filesystem/process filters; Python stdlib mechanics
use the owning script test selections through the supported environment. Targeted cases establish
individual syscall/race/output contracts. Plan57j owns assembled compiler/capture launch, actual
primary worker observation, canonical publication/crash/recovery, bootstrap admission and
disposable maintenance/backup/restore journeys, along with end-of-scope hygiene and publication.
Only those original owners can report actual descendant drain and release. A passing child wait
or 37/37 transport result comparison cannot substitute for these host and maintenance claims.
