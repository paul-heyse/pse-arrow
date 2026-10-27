# Host processes, containment and durable files

**Group G1 of the [comparative review](README.md), 2026-10-27.** This page covers CBO-003, CBO-004, CBO-019,
CBO-020, CBO-047, the non-SurrealDB parts of CBO-048 and new CBO-053. It is focused comparative advice. It
recommends bounded targets and authorizes no implementation.

| Item | Value |
| --- | --- |
| Reassessment | 2026-10-27 at source `4f52fd239`; consumes the complete original review and updated inputs at `b29fcf802`, base patterns 0.2 and additional patterns revision 3 (refined to 4 here). All named dispositions were reconsidered with representative current source, consumers and inherited evidence; unchanged evidence retains its original conditions. |
| Original source inspected | `main` at `3e9a2525b`. The audit baseline was `dbc4c9436`. Inside this scope the only change since then is a signature change to `canonical::operation_scope`, which does not matter here. Uncommitted changes in other files were not consumed. |
| Accounts consumed | Codebase pages `cross-cutting-mechanisms.md`, `execution-lifecycle-and-observation.md` and `developer-tooling.md`, plus the library accounts, all at `34f9f07ee`, including the dated correction bullets. Base patterns edition 0.2 and additional patterns revision 2, also at `34f9f07ee`. |
| Library realizations | rustix 1.1.5 (workspace pin, `fs`/`process`/`event`), tempfile 3.27.0 and libc 0.2.190 (all installed); tokio 1.53.2 (no `net` feature); procfs-core 0.18.0, jeepney 0.9.0 and zbus_systemd 0.26201.0 from the library accounts; CPython 3.14 stdlib. |
| Host realizations (read-only observation, 2026-10-27) | Kernel 7.0.0-38, systemd 255.4, ext4 home, `pid_max` 4194304, `CLK_TCK` 100. `user.delegate` is set on `user@1000.service`. No `user.invocation_id` xattr was found on user-manager units (one app scope and two `pse-telemetry-*` services). |
| Standard | Core 3.4, read qualitatively. The principles that apply most here are DP-13–DP-16, DP-19, DP-20 and AP-07. ADR-0229 (proposed) is read for its process-observer direction. |
| Evidence | Every target is *Proposed*. The enabling capabilities are *Interface-checked*, and each section names what was inspected. No probe was run. |

## Group summary

**The joint decision.** For each lifetime fact, the question is which kernel- or manager-owned
object should be the evidence, who owns systemd unit lifecycle, and which durable-publication
discipline each language should follow. The reading changes one framing in the inputs.
[LOP-X008](../assessment-patterns.md#lop-x008) (prove a lifetime
fact with an owned handle) is already applied at the unit level. Python admission and supervision
already decide drain from the kernel's inode-bound cgroup `populated` flag, kill whole units
through the manager (`systemctl kill --kill-whom=all`), and act on processes through pidfds
(`process_observation.populated`, `operation_lifetime.drained`, `native_operation`,
`compiler_cache`). The remaining manifestations are narrower:
- two in-process Rust group drains that scan `/proc`;
- a runtime that spawns the 11.6k-line supervisor every heartbeat just to read kernel facts;
- several Rust and Python file-publication helpers whose disciplines have drifted apart.

**Evidence objects (the recommended set).**

| Lifetime fact | Evidence object | Status |
| --- | --- | --- |
| A unit or allocation has drained | cgroup directory inode plus `populated` from `cgroup.events` | Keep. Python already uses it. The Rust runtime adopts it (CBO-020). |
| Acting on a live process (signal, wait) | pidfd, opened race-free for an unreaped child, or opened then verified for a recorded process | Keep and extend existing process use. CBO-020 proposes a primary handoff, with group semantics still preserved. |
| A persisted process is still the same process | (boot ID, pid namespace, PID, start ticks), parsed in one place per language | Keep the rule. Do not adopt the pidfs inode as a persisted identity (CBO-019). |
| An in-process child group has drained | Process-group inventory with pidfd re-check, in one shared type | Consolidate (CBO-004). Delegated child cgroups are the next decision. |
| Manager facts (state, invocation) | `systemctl --user show` in Python; one call per file-session registration in Rust | Keep (CBO-048). |
| A file was published without replacement | Rust: `renameat2(RENAME_NOREPLACE)`, or `O_TMPFILE` plus `linkat` for anonymous copies. Python: one stdlib module | Recommend (CBO-003, CBO-047). |

**Who owns unit lifecycle.** The Python supervisor and admission tooling stays the single owner.
Three facts decide it:
- The host ledger has one writer language.
- Admission must work before the Rust extension exists, because it governs building that
  extension.
- ADR-0229 (proposed) already gives Rust the versioned documents and leaves "Python's
  environment/systemd realization" to Python.

The runtime is a consumer. It receives a handoff of evidence handles (unit, cgroup, inode, main
PID), checks them against the kernel and observes them in-process. It never stops or releases
anything.

**Interactions with other groups.**
- **G2.** CBO-016 lists `observe-primary` polling and CBO-018 lists thread pidfds. G1 supplies the
  evidence leaf, and G2 keeps the waiting cadence. An `AsyncFd` wait would need tokio's `net`
  feature, which is not currently enabled. The synchronous sealed-file publication inside async
  `seal_file` is unchanged here; it is an execution-fit question for G2 and G4.
- **G3.** Its CBO-048 SurrealDB parts are "Retain current", with authenticated protocol readiness
  as the readiness authority. Nothing here changes readiness. CBO-020's handoff only adds a main
  PID to `ensure-primary`'s response.
- **G4.** The sealed payload writer and the export sink (CBO-002) sit beneath CBO-003's
  publication primitive, and their contracts do not change.
- **G5.** The compiler child's supervision moves out of `pse-math` into a shared type.
- **G9.** The Python digest helpers overlap CBO-050's evidence work.

**Reinforcing and competing choices.** Two choices reinforce each other. One Rust cgroup-lifetime
leaf (inode-bound `populated`) serves CBO-020 now and the CBO-004 containment upgrade later. One
Rust process-fact reader (CBO-019) serves CBO-004's inventory, canonical owner identity and
observation targets. Two pairs compete:
- **Per-child transient scopes or delegated child cgroups.** Delegated child cgroups win for Rust
  children, because a transient scope would need registration in the Python-owned ledger.
- **A D-Bus client or the `systemctl` CLI.** The CLI wins while bootstrap must stay stdlib-only.

**Priority by consequence**, kept separate from order:
1. CBO-003. Canonical-store crash window and three diverging Rust publication routes.
2. CBO-020. A per-heartbeat interpreter start during every managed study wait, plus a doubled
   verification seam.
3. CBO-047. Uneven ledger write discipline. The evidence includes 22 abandoned
   `allocations.<pid>.tmp` files in host-admission state.
4. CBO-004. Duplicated drain proof.
5. CBO-019. Parser consolidation.
6. CBO-048. Retained.

**Prerequisite order.** The CBO-019 reader comes before or with the CBO-004 shared type. The
CBO-020 cgroup leaf comes before any containment upgrade. CBO-003 and CBO-047 are independent.
Plan56e owns current deployment and resource-ownership acceptance, so CBO-020's protocol change
should land inside or after that boundary.

**U1–U3.** No choice on this page depends on them. U1 (upstream contributions) would only matter
for conveniences nothing here needs: a rustix `PIDFD_GET_INFO` wrapper, or stable std pidfds. U3
does not reach the host ledger, which is tooling state that must exist before the store starts.

<a id="cbo-003"></a>

## CBO-003 — Durable file publication and the sealed-file byte layer on `std::fs`

### Responsibility and target intent

Publish a completed file under its final name without ever replacing an existing name. Make the
crash order explicit: data synchronized, name published, directory synchronized, and only then
the catalog acknowledgement. Keep the domain protocol where it belongs. The architecture states
what is promised in mechanism-neutral terms:
- "Finish, validate, synchronize and publish without replacement before transactionally
  attaching the exact descriptor" (identity-and-publication, *File publication*).
- For local export, "Interrupted staging remains `.incomplete`; the final destination refuses an
  existing file". This is stated twice, in identity-and-publication and in workflows-and-results.
- The offline recovery helper "uses anonymous temporary files and no-replace publication, and
  synchronizes the inode and parent".

The `hard_link` mechanism itself is incidental. The comment in `canonical_files.rs`
`publish_locked` justifies it only as "an atomic no-replace publication".

**What `HEAD` shows.** Rust publishes in three ways today:
1. **`hard_link` then `remove_file`.** Used by the canonical store (`publish_locked`), IPC export
   (`connected_results.rs`), the study workbench (`study_workbench.rs`), the review pack, capture
   handoff members and capture acknowledgements (`capture/recording.rs`).
2. **`O_TMPFILE` plus `linkat`.** Used by the offline copy (`canonical_files/offline.rs`:
   `anonymous_copy_file`, `linkat(.., AtFlags::SYMLINK_FOLLOW)`).
3. **`tempfile::persist_noclobber`.** Used by `xtask/src/producer_history.rs`. Through tempfile
   3.27.0, this already calls `renameat_with(.., NOREPLACE)`.

The `hard_link` sites diverge in durability:
- The canonical store syncs both directories.
- Export and the workbench sync data but not the directory.
- Capture acknowledgements and handoff members do not call `sync_all`.

The canonical store keeps a deterministic `.pending/<instance>.pending` staging file that can be
resumed. A re-entry compares the written prefix and refuses with `OperationReused`. Because link
then unlink leaves two names for a moment, the store needs two pieces of settlement logic for a
crash in that window: `settle_published_alias` (live) and `reconcile` (offline), each with tests.

### Credible complete options

1. **Retain current.** Keep three routes and per-site durability.
2. **tempfile for the simple publishers.** Use `NamedTempFile::new_in` with
   `Builder::permissions(0o600)`, then `persist_noclobber`, with a local data and directory sync
   around it (`tempfile/named-staging-noclobber-persist`).
3. **One Rust discipline on `renameat2`.** Use `rustix::fs::renameat_with(dirfd, staging, dirfd,
   final, RenameFlags::NOREPLACE)` (`rustix/noreplace-and-anonymous-publication`, installed 1.1.5
   `fs/at.rs`). Staging stays deterministic wherever its name carries meaning. Durability is an
   explicit choice per publisher. Anonymous `O_TMPFILE` plus `linkat` stays as the second form,
   for copies that have nothing to resume.
4. **cap-std, atomic-write-file or `object_store::LocalFileSystem`.** Each was ruled out in its
   account: no no-clobber, replace-only, or no directory fsync and no single-link/alias refusal.

### Comparative argument

Option 2 fails the architecture. `NamedTempFile` removes its staging on drop and unwind, and its
names are random. The export contract requires that interrupted staging *remain* `.incomplete`.
`review_pack.rs` relies on that file to publish a readable acknowledged prefix on failure
(`fail_at` links `<id>.incomplete`). Random names would also erase the canonical store's
resume-by-name check. tempfile fits only publishers without a staging contract, and those are
the diagnostic writers that currently skip `sync_all`. Adopting it there would leave two
disciplines in place.

Option 3 keeps every name that carries meaning and changes only the publication step. One
syscall gives atomic no-clobber naming, `EEXIST` for any existing entry (symlinks included), and
a link count that stays at 1 throughout. With the two-name state gone:
- After a crash, ext4's journalled rename leaves either the staging name or the final name,
  never both.
- The canonical store's resume path already handles a surviving staging file.
- The final-exists path already re-validates bytes.
- New publications need no alias settlement.

The anonymous form stays where the offline helper already uses it, and the architecture names
it. A small shared helper turns each site's durability into a stated choice instead of an
accident. Execution cost does not change: one syscall replaces two.

The domain rules stay local:
- resumable staging and prefix comparison;
- the per-instance `flock`;
- single-link and alias refusal on read;
- framed digest verification;
- byte-limit and cancellation checkpoints;
- catalog acknowledgement after the directory sync.

**The instance lock stays as it is.** It polls `try_lock` every 5 ms with an absolute deadline.
Neither std, `fs4` nor `fd-lock` offers a bounded `flock` wait (`rust-file-locks`), and an
uncontended acquire succeeds on the first try. A blocking lock on a blocking thread could not be
cancelled.

**Reads stay on held descriptors.** `ObjectStore` would lose the `O_NOFOLLOW` open, the
single-link check and verification under a held descriptor.

### Revealing case

An export is cancelled after acknowledged batches. Its `.incomplete` prefix must survive so that
the review pack can publish it as a readable failed member. Option 3 leaves the staging file
untouched on failure and renames it only on success. Option 2 would delete it.

A second case is a crash between `link` and `unlink` in the canonical store. Today recovery must
prove that the leftover `.pending` file has the same device and inode as the sealed file before
it unlinks. Under option 3 that state cannot be created.

### Disposition: Recommend

**Bounded target.**
- **Enabling mechanism.** `rustix::fs::renameat_with(.., RenameFlags::NOREPLACE)` at the installed
  pin, which needs Linux ≥ 3.15 and ext4. The offline helper's existing filesystem refusal
  already covers the anonymous form.
- **One helper in `pse-operations`.** It owns durable file state, and `pse-runtime` already
  depends on it. A possible shape is `publish_no_replace(staging_dir, staging_name, final_dir,
  final_name, Durability)`. With `Durable`, the caller has already synced data, and the helper
  renames and then fsyncs the final directory, plus the staging directory when it differs. With
  `Visible`, it renames only.
- **Durability per publisher.**

  | Durable | Visible (diagnostic artifacts that make no durability claim) |
  | --- | --- |
  | canonical sealed files | capture acknowledgements |
  | IPC export | review-handoff members |
  | study workbench export | review-pack members |

  Each Visible site states its choice.
- **Intended behaviour changes.**
  - No two-name window in the canonical store.
  - Export and workbench gain a directory fsync.
  - Each diagnostic publisher's durability becomes explicit.
- **Unchanged.**
  - `.incomplete` and `.pending` semantics.
  - The offline anonymous copy.
  - xtask's `persist_noclobber`.
  - The `native_compiled` cache's replace-semantics `persist`, which is correct for a
    content-addressed cache.
- **Residual work.**
  - Rework the canonical tests whose meaning changes, such as
    `link_before_unlink_crash_settles_only_exact_original_alias` and the offline two-link
    reconcile tests.
  - Keep `review_pack.rs`'s failure path (`fail_at`) as a no-clobber *link*, not a rename: it
    publishes the failed member's acknowledged prefix under a second name and deliberately keeps
    the `.incomplete` staging file ([LOP-X011](../assessment-patterns.md#lop-x011)). *(Revised after the omission and preservation challenge.)*
- **Machinery that disappears.**
  - Six `hard_link` and `remove_file` pairs.
  - Per-site sync decisions.
  - `settle_published_alias` and offline `reconcile`, once existing stores are shown to hold no
    legacy two-link alias (a read-only census of existing stores, not run in this pass). Until then that logic stays for legacy data only.
- **Machinery added.** One helper and a durability enum.
- **Transition.** Local. No interface or format change.

**Rule impacts.** None. The work is an implementation change inside the stated "publish without
replacement" contract. No ADR is needed and no architecture text changes.

**Evidence.**
- *Interface-checked:* rustix 1.1.5 `renameat_with` and `RenameFlags::NOREPLACE`; the tempfile
  3.27.0 `persist` route (`file/imp/unix.rs`); the architecture contract text; the publication
  sites listed above.
- *Proposed:* the target.

<a id="cbo-047"></a>

## CBO-047 — File-ledger primitives re-derived per script

### Responsibility and target intent

The Python tooling ledgers carry capacity, ownership and pin state across processes without a
daemon, so they must be correct under concurrency and crashes. The constraints are:
- private (`0600`, owner-checked) replace-publication and reads;
- `flock` locks with absolute deadlines, taken in a fixed order;
- fail-closed reads;
- stdlib only, because the launcher can run under system Python during bootstrap.

**What the scripts show.** `scripts/host_metadata.py` already calls itself "private host metadata
mechanics without admission or workload policy". It holds `lock_until`, `protected`,
`validate_private_file` and `control_run`. Publication, however, has diverged:
- `resource_store.publish` is the strongest. It uses an `O_EXCL|O_NOFOLLOW` temp, data fsync,
  `os.replace`, a directory fsync, ancestor fsync of newly created directories, and cleanup in
  `finally`.
- `surreal_server.write_json` syncs the file and the directory but checks no ancestors.
- `telemetry_service._write` does not sync the directory.
- `operation_lifetime.write_json` uses a PID-suffixed `O_TRUNC` temp, never fsyncs and never
  cleans up. It writes the native-operation pin records, and by default it also writes the host
  admission ledger: `host_admission.metadata(durable=False)`.

The host-admission state directory holds **22 zero-length `allocations.<pid>.tmp` files dated
2026-10-10** (read-only listing on 2026-10-27). They are abandoned staging from killed writers.
Lock loops are also spelled separately, with 10, 20 and 50 ms polls or blocking `flock`, in
`surreal_server.state_lock`, `native_operation.record_lock`, `telemetry_service._lock` and
`host_metadata.lock_until`.

### Credible complete options

1. **Retain current.**
2. **A lock or atomic-write library.** filelock, portalocker, fasteners, flufl.lock or
   atomicwrites. All are ruled out in their accounts:
   - filelock's default Unix fallback uses a soft lock with a warning; explicit native
     configuration fails closed and its descriptor API preserves caller ownership, but it
     still lacks shared `flock`, a composed absolute deadline and the stdlib-only bootstrap route;
   - fasteners uses `lockf`, which does not conflict with `flock`;
   - flufl.lock implements leases;
   - atomicwrites is unmaintained and has no `O_NOFOLLOW` or mode checks.

   None meets the stdlib-only cold-start rule.
3. **One in-repo stdlib module.** Extend `host_metadata.py`, the existing owner.
4. **Python `renameat2` through `ctypes`.** This would give no-clobber parity with Rust.

### Comparative argument

The libraries either weaken fail-closed `flock` semantics or add nothing the stdlib lacks, so
option 3 is the library-first answer here (DP-13 with a stated reason). The stdlib already
composes each primitive in a few lines. The defect is not missing capability. It is the repeated,
independent decision about the same mechanism (DP-16), and the evidence is the uneven fsync and
cleanup.

Making all ledger writes durable is *not* the target. The admission ledger is written on every
`pse-env` command start and stop. Its non-durable default appears deliberate, and the
architecture disclaims power-loss durability. Today's non-fsync'd state already degrades into a
protective refusal: an unparsable ledger fails closed. What should become uniform is:
- an explicit durability argument;
- `O_EXCL` random temp names in the same directory;
- cleanup in `finally`;
- for locked ledgers, removal of stale sibling temps by the lock holder. Temps are created only
  under the ledger's `flock`, so any temp the holder finds belongs to a dead writer.

Option 4 brings `ctypes` FFI into cold-start tooling. It would serve only two no-clobber artifact
publishers (`backup_image.py`, `diagnostic_trace.py`), whose stdlib `os.link` plus unlink route is
adequate.

### Revealing case

A `pse-env` command is SIGKILLed between creating its temp and calling `os.replace`. Today it
leaves `allocations.<pid>.tmp` permanently. Under the target, `finally` cannot run either, but the
next ledger holder removes the stale sibling while it holds the lock, so the directory converges.
The durability choice for the hot ledger is unchanged, so no latency is added to every command.

### Disposition: Recommend

**Bounded target.** One stdlib module, `scripts/host_metadata.py` extended. Possible API names:

| Function | What it does |
| --- | --- |
| `publish_private(path, data, *, durable)` | `O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC`, `0600` random temp in the same directory; write; with `durable`, fsync and then a directory fsync after `os.replace`; unlink in `finally`. |
| `protected_directory(path)` | The existing ancestor check, plus fsync of newly created ancestors, taken from `resource_store`. |
| `read_private(path)` | `O_NOFOLLOW` open plus `validate_private_file`. |
| `locked(path, kind, deadline)` | Opens the lock file privately and uses the existing `lock_until`. |
| `sweep_stale_temps(directory, prefix)` | Removes stale temps carrying this ledger's prefix. Called only under the ledger lock, and valid only where every writer of that directory holds the lock. |
| `link_no_clobber(staging, final)` | For the two artifact publishers. |

`canonical_json_digest(value)` is shared only by copies that are byte-identical
(`json.dumps(value, sort_keys=True)` with default separators). Every other persisted identity
keeps its exact encoding, because a change would alter recorded identities.

- **Lock coverage before the sweep.** *(Revised after the omission and preservation challenge.)* `operation_lifetime.write_json` is shared with
  `compiler_cache.py` and `native_cache.py`, and whether those writers hold the ledger lock was not
  shown. Random temp names therefore carry a per-ledger prefix, and the sweep is enabled per
  directory only after its writers are confirmed to hold the lock; elsewhere temps stay
  writer-cleaned in `finally` only.
- **Retained as owner policy.**
  - Lock order (invocation, then identity, then resource).
  - The callers' absolute deadlines.
  - `native_operation`'s signal masking around its record lock.
  - The durability choice per ledger.
  - Fail-closed handling of unknown state.
- **Machinery that disappears.**
  - The publish helpers in `operation_lifetime`, `surreal_server`, `telemetry_service` and
    `resource_store`. Their call sites move to the module.
  - The `durable` split in `host_admission.metadata`, which becomes one call with an explicit
    flag.
  - The separate lock loops.
- **Machinery added.** About six functions in an existing module.
- **Intended behaviour changes.**
  - Abandoned temps converge.
  - Pin records keep their current non-durable choice, now stated.
  - Telemetry gains a directory fsync if it declares itself durable.
- **Transition.** Mechanical, per script. On-disk formats are unchanged.

**Rule impacts.** None. This is tooling.

**Evidence.**
- *Interface-checked:* the modules read; the library account sources; the host-admission
  directory listing.
- *Proposed:* the target.

<a id="cbo-004"></a>

## CBO-004 — Child-process and process-group supervision implemented per consumer

### Responsibility and target intent

Start an external process, bound its output and observe cancellation and the deadline. Before
releasing in-process resources, prove that its whole process group has drained. An unproved
drain must never release an owner that holds admitted capacity.

**What `HEAD` shows.** Two sites carry the group-drain proof, with the same design written twice:
- `pse-math/src/native_compiled.rs`: `CompilerChild`, `group_members`, `proc_identity`,
  `wait_exit`.
- `pse-runtime/src/tooling/capture/launch.rs`: `LaunchedCommand::drain_and_reap`,
  `group_members`, `member_identity`, `spawn_owned_command`, also used by
  `review_launch/execution.rs`.

The shared design is:
- keep the leader unreaped (`WNOWAIT`) so that its PGID stays reserved;
- `killpg`;
- scan `/proc` for members, reading stat, then `pidfd_open`, then re-reading stat and checking
  the pidfd is not yet exit-readable;
- `pidfd_send_signal` and wait on each member;
- re-inventory until no members remain.

Only the failure policy differs, and legitimately:
- **The compiler parks forever** (`retain_unknown_drain`). It holds prepaid capacity and a
  temporary output directory.
- **Capture refuses after 3 s** and retries on drop. It is diagnostic and holds no capacity.

Several supervisors kill only their direct child, and their contracts do not claim group drain:
- `native_child.rs` `OwnedChild` runs the repository's own binary with the `Interpreted`
  evaluator, so it has no descendants by construction;
- the `capture/cpu.rs` sampler `Tool`;
- the `worker.rs` supervisor control through tokio `kill_on_drop`.

Python already uses unit-level containment. Work runs in scopes and is killed with `systemctl
kill --kill-whom=all`, and drain is decided by the inode-bound `populated` flag. So
system-level drain is already kernel-proved for everything inside a pse scope.

### Credible complete options

1. **Retain current.** Two copies.
2. **Consolidate into one supervised-child type.** It keeps the pgid and pidfd proof and makes
   the failure policy a parameter.
3. **Delegated child cgroups.** Place each child in an owned sub-cgroup of the supervisor's own
   delegated cgroup, using a `pre_exec` write to `cgroup.procs` or `clone3(CLONE_INTO_CGROUP)`.
   Kill with `cgroup.kill` and wait for `populated 0` (`linux-cgroup-v2/containment-kill-and-drain`,
   `linux-pidfd/spawn-into-cgroup`). Two conditions apply:
   - It needs `Delegate=yes` on the scopes the repository creates.
   - It needs a fallback for undelegated contexts.
4. **A transient scope per child,** via `systemd-run` or zbus `StartTransientUnit` with `PIDFDs`.
5. **process-wrap or command-group.** Ruled out: `waitpid(-pgid)` stops at `ECHILD`, and a numeric
   PGID can be reused.

### Comparative argument

**Option 4 is ruled out by accounting.** A transient scope created from Rust lands outside the
supervisor's registered unit. The capacity rules ("released only after actual descendant drain",
units registered before launch) live in the Python-owned ledger, so each Rust child scope would
need a ledger registration by a second writer language. A delegated sub-cgroup stays *inside*
the already-registered scope. The scope's hierarchical `populated` flag then covers it with no
ledger change.

LOP-X008 needs this countercondition: the evidence object must nest inside the owner that holds
the resource charge.

**Option 3 is the stronger proof, but cannot be the only mechanism without regressing.** Its
advantages:
- It covers members that leave the group with `setsid` or `setpgid`.
- `cgroup.kill` handles fork races.
- The PGID-reservation trick and the `/proc` scans disappear.

Its limit is coverage. Undelegated contexts are real and supported:
- bare `cargo t` and `cargo test`, documented as first-class;
- the Python extension imported in an editor or REPL;
- this agent, which runs in an `app-cursor-*.scope`.

systemd forbids writing into cgroups it created for undelegated units ("Never *write* to any of
the attributes of a cgroup systemd created for you", CGROUP_DELEGATION, v255). Those contexts
would therefore keep the pgid route as a fallback, and two drain proofs would remain. Refusing
compiled evaluators or capture launches there would make currently valid bare workflows fail.
That is ruled out unless the maintainer deliberately makes managed placement the prerequisite.

**The gain from option 3 is mostly in-process.** Unit-level containment already keeps admission
capacity charged while any escaped process lives in the scope.

**Option 2 removes the duplication and delivers a clear, low-risk win.** It keeps the proof that
the repository already trusts. It gives a stricter rule one home: today a fix at one site does
not reach the other, which is the CBO's own complaint. It also prepares the slot into which
option 3 can later be added as a containment mode.

### Revealing case

A capture launch runs a command that daemonizes with `setsid`. The consolidated type, like
today's two copies, returns once the original group is empty while the daemon runs on. The
enclosing pse scope stays `populated`, so host capacity remains charged, and the system-level
guarantee holds. The in-process "drained" claim is narrower than its wording. The type's contract
must say so: "the owned process group drained". Only containment (option 3) would widen it.

### Disposition: Recommend (consolidation), with containment as the next decision

**Bounded target.**
- **The type.** One synchronous `SupervisedChild` in `pse-diagnostics`, beside the CBO-019
  process-fact reader. That crate already depends on `rustix` and owns `Cancellation` and
  `WakeSignal`, and both `pse-math` and `pse-runtime` depend on it. *(Revised after the omission and preservation challenge.)* The first pass
  placed it in `pse-kernels`, but that crate's "worker ownership" means provider workers
  (worker-local controls) and it holds no process code.
- **Spawn.** `process_group(0)`, then a pidfd on the unreaped leader, obtained race-free from
  `Child`.
- **Output.** An optional bounded stderr capture thread, taken from `CompilerChild`.
- **Waiting.** Cancellation through an eventfd plus the deadline, in one `rustix::event::poll` set
  (from `wait_exit`).
- **Drain.** The leader stays reserved. Then `killpg`, an inventory through the CBO-019 process-fact
  reader, the pidfd re-check and wait per member, re-inventory until empty, and finally the
  leader is reaped.
- **Failure policy.** `Park` for the compiler, `Refuse { deadline }` for capture and review
  launch.
- **Consumers.** `native_compiled` and `capture::launch` (with `review_launch`).
- **Unchanged.**
  - The direct-child supervisors.
  - Python's per-site `killpg` on its own unreaped children.
  - The compiler's deterministic flags and `prlimit` wrapper.
- **Machinery that disappears.** One of the two inventory, identity and drain implementations,
  and the duplicated stat parsers.
- **Transition.** Local to two crates. G5 should know that supervision leaves `pse-math`.

**Next decision (not part of the target).** Add a delegated-cgroup containment mode to the same
type. Authenticate delegation from this exact unit's manager `Delegate` property or an explicit
launch handoff bound to that unit and cgroup. The retained probe shows that the systemd 255 user
manager omits `user.delegate` on its units; an ancestor xattr or writable cgroup directory does
not establish delegation of the selected unit.

**What it would require:**
- `Delegate=yes` on the scopes the repository creates. These are in `pse_env.py` (two lists),
  `native_operation.py` and `surreal_server.py` (worker and observer scopes).
- Supervisors stay in the scope root, and only children move into leaf sub-cgroups with no
  controllers enabled. Memory is then charged to the scope. The `pse-worker` placement walk,
  which reads its own `memory.max`, keeps working.
- `activity.py` and `pressure_observation.py` move to recursive `cgroup.procs` reads.

**Settling evidence:**
- Probe P1.
- The maintainer's answer to one question. Should supervised children require delegated
  placement? If yes, the pgid route can be deleted and a single mechanism remains. If no, the
  mode is an optional strengthening beside the fallback.

**Rule impacts.**
- Consolidation into an existing crate needs no ADR.
- `Delegate=yes` is a tooling and placement change. Its operator-facing description belongs in
  `docs/dev/agent-environment.md`.
- Making managed placement mandatory for supervised children would change behaviour for bare
  runs, and would need an architecture amendment by the design-edit route.

**Evidence.**
- *Interface-checked:* both drain sites; the three direct-child supervisors; rustix 1.1.5 process
  APIs; systemd v255 CGROUP_DELEGATION (via Context7: `Delegate=` on scopes and services,
  `user.delegate`, the write prohibition, the no-processes-in-inner-nodes guidance); host xattr
  and `subtree_control` reads.
- *Proposed:* the target.

<a id="cbo-019"></a>

## CBO-019 — Linux process, cgroup and pressure facts parsed by hand in Rust and Python

### Responsibility and target intent

Read process identity, peak RSS, cgroup limits and pressure with fail-closed semantics, keeping
absent, unavailable and changed distinct. Persist identities that stay sound against PID reuse.

**Python.** Python already has one leaf, `scripts/process_observation.py`, used by eight modules.

**Rust.** Rust parses `/proc/<pid>/stat` at about eight sites and `/proc/self/status` at four:
- `canonical_files.rs` `ProcessIdentity::current`;
- `canonical.rs` `require_initializer`;
- `canonical_files/offline.rs` `owner_dead`;
- `observation_target.rs`;
- `observation.rs`;
- `peak.rs`;
- `native_child.rs` (PPid);
- `allocator_probe.rs`;
- the two drain scanners;
- `xtask/src/producer_preparation.rs`.

**The cgroup limit walk.** The ancestor walk in `pse-worker.rs` (`verify_placement_hierarchy`) is
the native mechanism. No kernel file reports the effective limit.

### Credible complete options

1. **Retain current.**
2. **procfs-core 0.18** (`default-features = false`) for Rust parsing (`procfs/typed-proc-readers`).
   Behind one local reader that owns the absent, unavailable and changed vocabulary and the
   identity composition.
3. **One local Rust parser helper.**
4. **pidfs inode identity.** `fstat(pidfd).st_ino` paired with the boot ID would replace start
   ticks in persisted records in both languages (`linux-pidfd/handle-based-process-identity`).
5. **psutil, cgroups-rs or `procfs` for the walk.** Ruled out in their accounts.

### Comparative argument

The user-level preference and DP-13 favour an established parser over a 13th hand split, and
procfs-core is I/O-free (`Stat` implements `FromRead`, *Interface-checked* via Context7
`/eminence/procfs`). It is a modest, clear win only if it carries the shared reader. The reader,
not the crate, is what removes the repeated decisions. Option 3 would achieve the same
consolidation without the dependency. The choice between them is close, and under the stated
library preference it goes to option 2.

**Option 4 changes no guarantee here.** A PID would have to wrap past `pid_max` 4194304 within one
10 ms tick (`CLK_TCK` 100) and inside the same boot and namespace before start ticks became
ambiguous. That is not a credible event on this host. The migration it would need is wide:
- the canonical file-owner rows in SurrealDB (`boot`, `pid_namespace`, `pid`, `start`), which are
  catalog metadata;
- `lifecycle-owner.json`, written by Python and read by Rust `require_initializer`;
- the host-admission owner records;
- native-operation composer records;
- `server-process.json`.

systemd 255 does not expose pidfd inode IDs (they arrive in 258), so manager cross-checks would
stay PID-based. LOP-X008's real value, holding a handle when acting on a process, is already
obtained through pidfds in both languages. The persisted key is a different evidence object, the
"record checked after the subject is gone" case that LOP-X008 itself flags.

**Unchanged:**
- **The limit walk.** It is the native mechanism. One caution: the walk assumes the process sits
  in its allocation cgroup, which constrains CBO-004's containment branch.
- **The PSI samplers.** No consumer acts on thresholds.
- **The Python leaf.** psutil is ruled out: its float, clock-derived `create_time` and its lack of
  cgroup and PSI support.

### Revealing case

`offline::owner_dead` checks a file owner recorded by *another* process, possibly in another PID
namespace. With pidfs identity, `pidfd_open(pid)` in the wrong namespace would open an unrelated
process. Its inode would differ, so the check would report the owner as dead. Today's explicit
namespace refusal ("cannot be authenticated") is therefore still required. The inode does not
remove the namespace fact, only the parse.

### Disposition: Recommend (bounded and minor), with the identity rule retained

**Bounded target.**
- **One Rust process-fact reader** on procfs-core 0.18 (`default-features = false`). It reads with
  std, parses `stat` (state, pgrp, the `starttime` field) and `status` (VmHWM, PPid), and maps
  read and parse outcomes onto present, absent, unavailable and changed. It lives in
  `pse-diagnostics`, which `pse-operations`, `pse-runtime` and `pse-math` already depend on, and
  which already reads `/proc/self/status`. xtask uses procfs-core directly or through the same
  reader.
- **Retained.**
  - The (boot, namespace, PID, start-tick) identity rule.
  - pidfd for every live signal to a recorded process, with post-open verification (already the
    practice in `native_operation.py` and the drain scanners).
  - The cgroup walk.
  - The PSI samplers.
  - Python's `process_observation.py`.
- **Machinery that disappears.** About 12 hand parsers and their overflow and malformed-input
  tests.
- **Added.** One dependency and one reader.

**Rule impacts.**
- procfs-core is an ordinary dependency: no ADR.
- Adopting pidfs identity for canonical file owners *would* change catalog metadata. That is a
  metadata and commit-contract change needing an ADR and a design review. It is not recommended.

**Evidence.**
- *Interface-checked:* the sites listed; procfs docs; host `pid_max` and `CLK_TCK`; systemd NEWS
  facts from the library accounts.
- *Proposed:* the target.

<a id="cbo-020"></a>

## CBO-020 — Primary-receiver supervision split between the Rust runtime and a Python script

### Responsibility and target intent

The runtime must start or reuse the deployment's one primary receiver under host admission. While
it waits for a study, it must learn whether that receiver's lifetime has ended. Observation never
grants release authority (`ManagedPrimary::observe`).

**What `HEAD` shows.** `ensure-primary` is launched once per study run. It hands back a
`PrimaryOwner`: {unit, cgroup, invocation} plus `scope_inode`. After that, `wait_study_inventory`
(`study_execution.rs`) spawns `python3 surreal_server.py observe-primary` at every heartbeat. That
script re-derives the drain phase with `operation_lifetime.drained`, which reads the kernel
cgroup inode and `populated` first. A populated group returns before `systemctl show`; manager
LoadState, InvocationID, ControlGroup and ActiveState are consulted only on remaining paths.
Inventory polling is normally 100 ms, but receiver observation follows the admitted heartbeat,
normally 10 s; these are distinct cadences.

The runtime then re-checks the JSON identity. The runtime is the only consumer of `observe-primary`
(repository-wide `rg`). It collapses `Drained` and `OwnerChanged` into "unavailable", and treats a
failed observation as `Live`.

### Credible complete options

1. **Retain current.**
2. **The runtime owns unit lifecycle through zbus 6.1 and zbus_systemd 0.26201** (`StartTransientUnit`,
   `JobRemoved`).
3. **Observe in-process.** Lifecycle stays with the script, and the runtime watches the handed-off
   kernel evidence itself: the inode-bound cgroup plus a pidfd on the primary's main process.
4. **The runtime calls the manager for observation only,** through zbus.

### Comparative argument

**Option 2 moves policy to the wrong owner.** Admission, the ledger, receipts and
service-borrower accounting would need a second writer language. Python admission must also keep
working before any Rust artifact exists. ADR-0229 (proposed) sets the opposite direction: Rust
owns the documents, and Python realizes them in systemd.

**Option 4 retains a manager dependency** for cases not settled by held kernel evidence. It
remains a credible narrow fallback; removing it requires preservation of its remaining meaning.

**Option 3 uses the evidence this consumer actually needs.** That is whether the bound receiver
lifetime still exists. Two kernel objects answer it, and the runtime can hold both:
- **The cgroup.** Its directory is identified by inode, and its `populated` flag covers
  descendants.
- **A pidfd on an authenticated process.** It gives a race-free exit signal for that process,
  not for all descendants. systemd's main process is `native_operation.py`, which launches the
  receiver worker; `_primary_process` separately authenticates the marker's `pse-worker` PID.
  Supervisor exit must not silently replace the current receiver-group drain decision.

What option 3 can remove once the complete observation contract is preserved:
- the interpreter start at every heartbeat (CBO-016's "spawning an interpreter to read state");
- the doubled identity comparison;
- unnecessary manager reads when held kernel facts suffice. A narrow fallback remains for
  unresolved observations; uncertainty still grants no release authority.

ADR-0229's "dependency-leaf process observer [that] reports authenticated facts and uncertainty",
with no "universal alive predicate", is exactly this shape: the owner composes its own decision
from shared leaf facts.

### Revealing case

The supervisor exits while a worker descendant remains. A main-process pidfd becomes readable,
but the cgroup is still populated. Calling this group drain would change today's semantics.
Treat the process event as a wake hint and retain the authoritative inventory re-read unless a
broader receiver-unavailable condition is explicitly selected and qualified.

A second case: the manager is unreachable for a few seconds. Today `observe-primary` fails and
the wait continues. Under the target the kernel read continues, and no release authority is
gained in either case.

### Disposition: Recommend

**Bounded target.**
- **Handoff.** Distinguish the main supervisor PID from the authenticated receiver worker.
  Bind the chosen process to boot ID, PID namespace and start ticks captured during the
  authenticated handoff, together with the unit, invocation and cgroup identity.
- **Authentication at handoff.**
  1. Stat `/sys/fs/cgroup<group>` and require its `st_ino` to equal `scope_inode`.
  2. Open the selected process pidfd through rustix and verify its recorded start/boot identity.
  3. Require the `0::` line of `/proc/<pid>/cgroup` to equal or descend from `group`, through the
     CBO-019 reader.
  4. Recheck unchanged cgroup identity and require the pidfd not yet readable.
  5. Require the runtime to share the manager's PID namespace (compare `/proc/self/ns/pid` with
     the handoff's recorded namespace), because a PID from another namespace names a different
     process. On a mismatch, fall back to today's observer rather than refuse, so a runtime in a
     different namespace keeps working. No such context was found, so the fallback is a guard,
     not a supported mode. *(Revised after the omission and preservation challenge.)*

  On an identity mismatch, refuse ("managed primary is not ready"). For a namespace mismatch
  or specifically unsupported kernel-evidence case, use the authenticated observer fallback;
  do not convert uncertainty into drain or release authority.
- **Observation.** Preserve the current group-lifetime decision, explicit changed-owner and
  uncertain outcomes, and final authoritative inventory re-read. Process exit is a wake hint,
  not proof of group drain. Keep a narrow manager-observation fallback where kernel evidence
  cannot settle the current contract. `Unavailable` still means keep waiting, not release.
- **Cadence.** Unchanged: a zero-timeout `poll` at the heartbeat. Whether any wait becomes an
  `AsyncFd` event wait (which needs tokio `net`) is owned by G2 under CBO-016; this target does
  not need it, and neither do CBO-004 or CBO-018.
- **Shared leaf.** One Rust cgroup-lifetime leaf (inode-bound `populated`) that CBO-004's
  containment branch can reuse.
- **Eligible for retirement.** Ordinary observation subprocess/JSON overhead when the new
  handoff and kernel evidence settle the same decision. Do not promise deletion of
  `observe-primary` or its owner-level contract tests until the fallback need is resolved.
- **Retained.**
  - The script's `ensure-primary`: admission, `systemd-run` transient service, protocol readiness
    (G3), receipts, and drain for release.
  - The runtime's "no release on observation" rule.
  - `canonical_files` file-owner registration, which makes one `systemctl show` call per session.
    The invocation is not available as an xattr on this host's user-manager units.
- **Not adopted.** zbus and zbus_systemd.
- **Transition.** Extend the authenticated handoff, qualify process-versus-group semantics and
  fallback behavior, then remove replaced paths. Plan56 is complete and authorizes no new
  work; selected implementation needs its current planning owner.

**Rule impacts.** The change concerns the runtime and supervisor control protocol. That is not the
PyO3 Python boundary and not a D1–D14 decision. Check the scope of ADR-0229 (proposed) before
implementation. No new ADR is expected.

**Evidence.**
- *Interface-checked:* `worker.rs`, `study_execution.rs`, `surreal_server.py` (`observe_primary`,
  `primary_dispatch_owner`, `primary_observation`, the primary launch), `operation_lifetime.drained`,
  ADR-0229, rustix pidfd APIs.
- *Measured* in the [delegated cgroup probe](../../design_review/evidence/library-leverage-delegated-cgroup-containment-2026-10-27/README.md): `POLLPRI` on `cgroup.events` reports
  `populated 0` after a kill, zombies do not hold `populated`, and pidfds become readable on exit.
- *Proposed:* the target.

<a id="cbo-048"></a>

## CBO-048 — Host workload admission and service supervision rebuilt above systemd (non-SurrealDB parts)

### Responsibility and target intent

Bound concurrent workloads across independent processes without a daemon, using 16 lanes, the
explicit `unbounded` mode, release that is never inferred from a missing caller, and drain
reports that remain possible when the manager is unavailable. Honor the selected placement,
including the explicitly unbounded host mode, rather than assuming every workload is in a capped
systemd user scope.

### Premise check

The CBO's framing, "re-derives [systemd] state by polling `/proc` and parsing `systemctl` output",
is only partly true. The sampled lifecycle paths compose authoritative facts:
- **Manager facts** come from `systemctl --user show --property=…`: LoadState, ActiveState,
  InvocationID, ControlGroup, MemoryMax, MainPID.
- **Kernel facts** come from cgroup inode identity and `populated`.
- **Ledger policy** comes from lanes, owners and units.

See `host_admission` `retire`/`drained`, `operation_lifetime.drained`,
`surreal_server.worker_busy` and `_stop`. Units are created with `systemd-run --scope`, which
moves itself into the scope before exec, so containment starts with the first instruction. The
`/proc` reads are caller-liveness checks (start ticks), not drain inventories.

### Credible complete options

1. **Retain current.**
2. **jeepney 0.9.0 for manager facts and `JobRemoved` waits** (`python-dbus-clients/pure-python-blocking-client`).
3. **`StartTransientUnit` with `PIDFDs` instead of `systemd-run`.**
4. **Event-driven drain waits** with `select.poll(POLLPRI)` on `cgroup.events`.

### Comparative argument

- **Option 2.** The launcher, including host admission, falls back to system Python "before
  [the] environment exists". So the stdlib `systemctl` route must stay, and jeepney would create
  a second route for the same facts, or a vendored client. The gain is the cost of a process
  spawn, not a guarantee: `systemctl show --property` is a stable scripting interface, and
  `systemctl stop` already waits for its job.
- **Option 3** would rebuild `systemd-run`'s fork, attach, wait and exec sequencing locally.
- **Option 4** is a latency refinement. CBO-016's constraint, that correctness never depends on
  notification delivery, keeps a bounded polling fallback in place anyway.

The ledger itself is required by the premises, since systemd has no cross-client semaphore. So
the complete case is strongest for the current composition. The worthwhile fixes are its
file-ledger discipline (CBO-047) and, if chosen, `Delegate=yes` (CBO-004's next decision).

### Revealing case

During bootstrap, `.venv` does not exist yet and `pse-env` admits under system Python. A D-Bus
client that is not in the stdlib would be unavailable at the moment admission matters most. A
`systemctl` fallback would then be the real authority, and the D-Bus path a duplicate.

### Disposition: Retain current

The "rebuilt above systemd" premise is partly rejected, with the evidence given above.

**Residual observation for the admission owner** (cause not traced). 24 empty per-allocation
slices persisted for days: 18 of 19 under `pse.slice/pse-allocation.slice` and 6 of 6 under
`psebuild.slice/...`, from a read-only listing on 2026-10-27. `retire_empty_allocation` exists,
but some exit paths evidently skip it. A reconcile pass that stops empty allocation slices
missing from the ledger would converge them. This is a minor lifecycle fix, not a leverage
choice.

The SurrealDB parts belong to G3, whose page retains authenticated protocol readiness. This
page relies on nothing in them beyond the `ensure-primary` handoff.

**Rule impacts.** None.

**Evidence.**
- *Interface-checked:* the functions named above; the `pse_env.py` placement code; host cgroup
  listings.
- *Proposed:* retention. Most of `surreal_server.py`'s 274 functions were not read; see
  the limits at the end of this page.

## Pattern notes

- **[LOP-X008](../assessment-patterns.md#lop-x008) refined (revision 3).** The evidence object must sit inside the
  owner that already holds the subject's resource charge; a persisted record checked later by
  another process is not a handle; objects created by a manager are read freely but acted on
  through the manager or an owned delegated sub-object. Investigate the coarser level first, and
  look for consumers that call the effect owner's tool only to read facts they could observe
  themselves (CBO-020).
- **New [LOP-X011](../assessment-patterns.md#lop-x011): a publication contract keeps interrupted work products.**
  CBO-003's `.incomplete` and `.pending` contracts are why RAII temporary-file helpers are ruled
  out for exports while a lower-level `renameat2(RENAME_NOREPLACE)` publication step fits.

## Limits

- **`surreal_server.py`.** About 20 of its 274 functions were read: the `systemctl` helper,
  `write_json`, `state_lock`, unit and worker naming, `worker_scope_command`, the head of
  `_worker`, `group_populated`, `worker_observation`, `settled_worker`, `workers_drained`,
  `primary_observation`, `primary_ready`, `primary_dispatch_owner`, `observe_primary`,
  `observer_scope_command`, the head of the primary launch, and `_stop`. Backup, restore,
  generation publication, telemetry and maintenance were not read. The CBO-048 retention covers
  the lifecycle paths sampled, not all 36 `systemctl` uses.
- **Other modules not read, or read only in part.**
  - `test_resources.py` and `service_borrowers.py` were not read.
  - In `compiler_cache.py`, only the pidfd and supervision section was read; the unit-text path
    was not.
  - `resource_store.FileStore` was read only up to its lock.
  - `telemetry_service.py` was read beyond `_write` and `_lock` only in part.
  - `pse_env.py` was read beyond placement and the manager probe only in part.
  - `native_operation.py` was read only for `record_lock`, `admit`, `pin`, cancellation and the
    scope launch.
- **Kernel facts inherited from the library accounts or existing repository premises and not
  re-fetched:**
  - `cgroup.kill` fork and migration handling;
  - `populated` poll and inotify notification;
  - uniqueness of cgroup directory inodes per boot;
  - zombies are excluded from `populated`. This is the repository's own comment in `_stop`.
- **The systemd no-processes-in-inner-nodes point is reasoning, not a test.** It is guidance in
  the documentation. Whether the kernel enforces it without controllers in `subtree_control` is
  my reasoning. The [delegated cgroup probe](../../design_review/evidence/library-leverage-delegated-cgroup-containment-2026-10-27/README.md)
  then *Measured* it: the kernel accepted the pre-exec move while the scope's `subtree_control` was
  empty, and refused `+memory` (EBUSY) with the supervisor in the scope root.
- **Dependencies.** procfs-core 0.18 was not resolved against the workspace. Nothing was built or
  tested.
- **Discipline sites.** The tooling-only Python helpers in `docs.py`, `capabilities/*.py` and
  `assessment_store.py` were counted but not each read.
- **Host observations.** These are single read-only snapshots taken on 2026-10-27. The cause of
  the stale allocation slices and temps was not traced.

<a id="cbo-053"></a>

## CBO-053 — Maintenance CLI output ownership

**New opportunity from reassessment; Interface-checked at `4f52fd239`, targets Proposed.**
Expanded supervisor reading found that `maintenance_sql` captures complete stdout/stderr
before bounding retained failure tails. `copy_current_database` also captures CLI output with
no local timeout, while passing the actual database export through an owned file. This does not
demonstrate a runaway command or establish that its complete enclosing lifetime is unbounded.
The [codebase account](../codebase/developer-tooling.md#cbo-053) owns that observation.

**Complete options.** Retain capture if real output is small and its lifetime policy is
intentional; drain both pipes incrementally while retaining operation-shaped bounds; or route
complete structured output to an admitted private file and decode it under a separate grant.
Discarding success output plus bounded failure tails is also credible for import/export control
logs, but not for nonce acknowledgements or catalog results. Existing
[stdlib subprocess facilities](../libraries/python-subprocess/README.md) support these routes
without another supervisor or a new third-party dependency.

**Judgment: investigate and preserve the alternatives; realization unresolved.** Tail truncation
after complete capture does not bound live output memory. It is worth separating acknowledgement,
catalog and copy-log demand, but their acceptable extents and complete lifetime policy were not
established. Catalogs can be legitimately large, and a long restore may be intentional. Prefer
demand-shaped ownership over a universal cap or timeout. Any selected route retains nonce checks,
redaction before publication, original deadline meaning, export identity and actual maintenance
listener/descendant drain. A direct child wait or CLI timeout is not group-drain evidence.

**Settling evidence.** Inspect actual success/failure output requirements and enclosing clocks;
then exercise large stdout/stderr, unterminated chunks, acknowledgement truncation, cancellation
and cleanup with disposable CLI peers. No such workload was run here. This inquiry does not
reopen the retained systemd/physical-backup division or authorize maintenance implementation.
