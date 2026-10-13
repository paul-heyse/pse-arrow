> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R1 (library-research), Part B: obligations mapped onto the R1 libraries. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R1 Part B: mapping the W-A/W-B obligations onto durable state, coordination and lifecycle libraries

Inputs: `w-a-interactive-obligations.md` (W-A), `w-b-multicase-obligations.md` (W-B), and my Part A
`r1-durable-state-and-coordination.md` (families F1–F18, allocations A/B). Static reading only.
Baseline `main` at `6c368c88a`. Stage references are `W-A 1.8`, `W-B 1b-9`, and so on.
Ownership grades: **wholesale** (the library discharges the obligation with no project protocol
beyond declaration), **partly** (the library supplies the primitive; a project protocol remains),
**none**. Necessity: **T** = the functional target or a W-B §(a) product feature needs it;
**R** = a current rule imposes it (cited); **H** = harness/tooling only. Interpretation is marked [I].

---

## 1. Obligations in scope

### O1. Durable immutable source revisions and head-CAS (W-A 1.7, 3.5; W-B 1b-3)

- **Ownership.** Partly by SurrealDB: one transaction per edit (F1), UNIQUE identity keys and
  `FOR UPDATE` on the head for CAS (F1/F2), `REFERENCE`/`ENFORCED` for membership-to-object integrity
  (F3). Revision identity hashing and membership-interval semantics stay the project's.
- **Necessity.** T for durable studies and managed workers: a worker in another process rebuilds the
  package from the canonical revision (W-B 1b-4, 1b-8). For the interactive and ephemeral loop it is
  R: §19 "Explicit ephemeral execution … still admits canonical sources"; the scope table "Canonical
  store availability" ("no durable-to-ephemeral fallback; ephemeral execution … still requires
  canonical sources"); `NativeModelingPackage` "modeling sources live only in the canonical store"
  (W-A §(a)). The functional target (interactive edit → re-solve without a structure rebuild) does
  not by itself require a durable revision before solving [I].
- **Consequence.** A per-session or per-package process-local revision (an `Arc` of declaration
  rows, which `ModelingPackage` already half-holds: physical, providers and quantities are in memory)
  would remove the store round trips from the interactive loop. Persisting it then becomes an
  explicit publication step that is needed only when a durable run, study or reproduction record is
  requested. This loses the guarantee that every solve's source is reproducible from the store
  unless the solve is published.

### O2. Protection grants and releases on revisions, premises and physical rows (W-A 1.8, 1.9, 1.11, 3.1, 3.6, 4.2, 4.5; W-B 1a-6, 1b-10)

These cycles are the largest per-solve write share in W-A: two grant/release pairs per solve, a
fresh grant for the root re-read, `SOURCE_LEASE` renewal every 10 s while held, physical-row
re-protection per durable run, and read protections that make every result read a mutation.

- **Ownership.**
  - **Partly, by SurrealDB.**
    - `REFERENCE … ON DELETE REJECT` from a protection or session row to the protected
      revision/result makes reclaim fail in the deleting transaction (F3). That turns a protection
      into one row whose existence is the protection.
    - A `GROUP BY owner` view could hold outstanding-protection counts (F5).
  - **Not covered by SurrealDB.** It has no expiry, TTL or lease semantics (F14). Liveness of the
    protector (heartbeat, expiry) stays the project's.
  - **Partly, by tokio in-process.** A held `Arc` plus a `TaskTracker::token()` per reader gives
    "cannot be reclaimed while referenced" for free inside one process (F17). That holds only if
    reclaim runs in the same process or is excluded while sessions are open.
- **Necessity.** T only where reclaim or retention can run concurrently with preparation or reading
  in another actor. The only reclaimers in W-B §(a) are Python clients ("history and retention
  operations") and maintenance; no worker reclaims. Everything else is R:
  - §14.3 "Protection covers source and interpretation inputs through off-transaction preparation";
  - §14.4 "a previously qualified complete source receipt may be adopted only after a guarded store
    check verifies a fresh live pin";
  - ADR-0178 (proposed).
  The 1a sweep holds source leases through the store while consuming no run or claim recovery
  (W-B §(a) interpretation).
- **Consequence.**
  - **A session-scoped protection** (one `REFERENCE REJECT` row per held package or runtime,
    created once, removed at release) would remove the per-call grant/release pairs (1.8, 1.9, 1.11)
    and the root re-read's private protect/release. It keeps the cross-process reclaim safety and
    still needs a liveness lease for crash cleanup.
  - **A single-owner or exclusive-reclaim rule** ("reclaim admits only when no runtime holds the
    store") would remove protections from the interactive loop entirely. That weakens concurrent
    multi-client use (ADR-0181) and concurrent maintenance.

### O3. Reuse qualification against the store (W-A 1.9 recheck, 1.10 description publish and ack, 3.8, 3.9; W-A §(c) rows 4–6)

- **Ownership.** None wholesale. SurrealDB can store receipts and answer an indexed recheck (F2, F15).
  Whether a cached selection is still valid is a project semantic. Immutability makes it trivially
  true if the revision cannot change or be reclaimed (revisions are immutable, O1) [I].
- **Necessity.**
  - The recheck exists because reuse "across restart" is governed by persisted dependencies and
    "Salsa is an optional local accelerator" (§14.4 ADR-0164 box). That is R.
  - Within one live process holding an immutable revision under O2 protection, a recheck cannot
    change the answer [I].
  - Description publication under abi3 writes bodies that "this session cannot receive" (W-A §(a)).
    For this session that is R, not T; it serves only another qualified producer or a later restart.
- **Consequence.** Process-local authority for reuse within a session (Salsa plus the in-memory
  caches W-A lists) removes per-preparation store rechecks and publication. Restart reuse would then
  depend on optional publication rather than on every preparation writing.

### O4. Run and attempt registration, lease claim, heartbeat, durable cancellation (W-A 1.14; W-B 1b-9 to 1b-11, 2-B, 3-2, 4a-2)

- **Ownership.**
  - **SurrealDB, partly.** Atomic registration and fenced claims are well served by UNIQUE
    `(run, attempt)` keys, `DEFINE SEQUENCE` fencing tokens (unique and increasing; gaps are harmless
    for fences; F7), and `FOR UPDATE` CAS on the run record (F1).
  - **No library.** Lease expiry, heartbeat cadence and the decision to recover are project-owned
    (no TTL or scheduler, F14).
  - **tokio, wholesale in-process.** Heartbeat task lifetime, cancellation propagation and teardown
    are covered by `TaskTracker`, `CancellationToken` child trees and `DropGuard` (F17).
  - **Durable cancellation flag.** A store row (F1); LIVE SELECT could push it as a wake-up (F6).
- **Necessity.** T for durable studies served by managed workers and for "multiple workers or
  clients on one study" (W-B §(a) recovery table; §19.3, §20.6). For in-process durable `RunHandle`
  runs (W-A 1.14, W-B route B), registration before effect is R (`run.rs` "Every run is supervised
  under its runtime's durability class"; §20.6). The run dies with its process: the lease exists so
  that another actor can recover it after process loss. Restart reconciliation by the same or a new
  runtime reaches the same terminal truth if "recovery never invents scientific success" (W-B §(a))
  [I].
- **Consequence.** For in-process runs, a minimal durable core keeps registration (needed so restart
  finds non-terminal records) and drops the 10 s heartbeat and the 30 s lease. Recovery becomes
  "non-terminal at open" plus the process-liveness evidence the supervisor already owns (PID start
  identity, cgroup). That loses live detection by a second client while the first is still running.

### O5. Study creation, readiness discovery, fenced claims, finalization, waiter polling (W-B 1b-5 to 1b-14; route C)

The coordinator's figure is about 10 store transactions per point. The ledger's per-point
transactions are: point read, header read, seed read, claim, start, mark-started, prediction, renew,
progress appends, chunked publish, terminate, record attempt, and finalization share.

- **Ownership.**
  - **SurrealDB, partly.**
    - Claims: UNIQUE plus sequences plus `FOR UPDATE` (F1/F7).
    - Readiness counters as same-transaction grouped views or sync events (F4/F5). These are hot
      keys under 16 lanes [I].
    - Discovery wake-ups: LIVE SELECT over ws (F6), lost on reconnect, so periodic polling stays as
      the backstop (I2).
    - Incremental readiness reads: `CHANGEFEED` with a cursor; retention-bounded, so a recovery scan
      is still needed.
    - The 100 ms waiter poll (W-B 1b-6) could be a live subscription on the study's terminal
      inventory, with the poll as fallback.
  - **No library.** Predecessor and generation semantics and the pure policy
    (`AdmittedStudy`, Wait/Start/Refuse/Cancel) are scientific scheduling and stay the project's.
- **Necessity.** T: W-B states "Concurrent lanes and processes are made exclusive by the store's
  fenced claims" and that claims are the only cross-process exclusion. Durable study survival is a
  product feature. The number of transactions per point is not fixed by T. Mark-started, prediction
  and renew as separate RPCs, plus a heartbeat every 10 s, are protocol choices (§20.6 "claim,
  start, append, heartbeat, observe, close, seal and finalize register it before their decision";
  ADR-0177 proposed). That is R.
- **Consequence.**
  - Folding claim, start and prediction into one server function call (F8, already the house style)
    and terminal publish, outcome record and readiness update into one transaction would cut the
    per-point round trips by construction [I]. LIVE wake-ups cut idle polling but not the
    correctness poll.
  - What weakens: finer-grained durable intermediate states, which are visible only to diagnostics
    and recovery.

### O6. Result publication (W-A 1.17; W-B 1b-13, 4a-4, 4b-3, 3-7)

- **Ownership.**
  - SurrealDB, partly: transactional append of bytes blocks with a closed manifest (F1/F12). Native
    bytes are fine at block scale. Buckets (F12) are experimental and move backup outside the store.
  - Row locators and indexes are a project schema.
  - Overlaps R3, which owns the result plane.
- **Necessity.** T for durable runs and studies. Ephemeral already skips it.
  - The workload premise "interruption before publication" points the other way for long
    trajectories. W-B 4a observes that results are appended only in `finish`, so interruption loses
    the whole trajectory. Satisfying that premise means more incremental durable writes
    (horizon-style prefixes, W-B 4b-3), not fewer [I].
- **Consequence.** Nothing is removed in R1's scope. A minimal durable core keeps this as its
  central write and makes it the terminal fact that recovery reads.

### O7. Result reads as mutations (W-A 4.2, 4.5)

- **Ownership.** SurrealDB, wholesale in either of two ways.
  - Readers do not need write-protection if retention cannot delete a block while a manifest
    references it: `REFERENCE REJECT` from manifest to blocks, plus a retention protocol that deletes
    manifests first (F3).
  - Alternatively, a plain snapshot read inside one transaction (snapshot isolation, F1) suffices for
    a bounded page.
- **Necessity.** R: `READ_LIFETIME` protections exist so that a multi-page read survives concurrent
  reclaim between pages (§20.6 "A protected reader transfers activity ownership…"). T only if
  concurrent reclaim is allowed during reads (premise P4).
- **Consequence.** Reads become reads: no write transaction, no `retention:` guard `FOR UPDATE`
  (EFF L2-02 hot key). What weakens: a reader that spans pages can observe a manifest whose later
  blocks were reclaimed, unless reclaim is reference-guarded.

### O8. Durable progress streaming (W-A 1.14; W-B 1b-12, 4a-3)

- **Ownership.** SurrealDB partly: an append table with LIVE or CHANGEFEED for consumers (F6).
  tokio wholesale for in-process delivery (`watch` or broadcast, F17). The carrier choice is R3's.
- **Necessity.** T only for observing a run in another process (a managed worker). For in-process
  runs, in-process delivery suffices [I]. Durable progress after a crash is diagnostic, not
  scientific.
- **Consequence.** For in-process runs, removes the 64-event/200 ms RPC stream; it loses post-crash
  progress history.

### O9. Caller, activity and physical connection lifetimes; drain (W-A ACT; ADR-0181, §20.6)

- **Ownership.** tokio-util wholesale for the in-process mechanics:
  - one `TaskTracker` per lifetime layer;
  - `token()` for non-task activities (protected readers, borrowed connections);
  - `close()` followed by `wait()` for drain;
  - `CancellationToken` child trees for caller release.
  The SurrealDB embedded engine would add drop-cancellation of in-flight queries
  (`AbandonOnCancel`, F10). The remote socket's "closure does not prove server abort" stays a
  transport obligation (the vendored patch).
- **Necessity.**
  - Draining before physical close is T (do not drop work mid-publication).
  - The three-layer separation for "multiple Python runtimes in one store" is R: ADR-0181 (proposed),
    W-B §(a). The functional target names no multi-client requirement; the scope page lists
    "multi-writer admission" among unresolved design choices with an implemented protocol
    (`scope-and-open-design.md`).
- **Consequence.** A tracker/token composition replaces the hand-rolled Open/Closing/Closed counters
  in `canonical_lifetime.rs` and the nested ACT scopes. Semantics must be checked: a
  `TaskTracker` does not refuse new work after `close` by itself, so admission refusal needs a check
  of `is_closed` or a closed `Semaphore` (F17).

### O10. Transport: bounded admission, deadlines, unknown-outcome settlement (W-A crossings; `PATCH.md`)

- **Ownership.** Remote: none upstream at 3.3.0 (the patch supplies it). Embedded: wholesale for
  cancellation and deadlines (in-process, drop-aborts; F10); no network unknown-outcome. A crash
  during commit remains, and WAL recovery (`sync=every`) answers it at reopen.
- **Necessity.** T as long as the store is remote and shared by several processes (managed
  workers). The embedded alternative is undermined by W-B facts:
  - a native crash takes its process;
  - managed workers are separate processes in their own systemd scopes;
  - an embedded RocksDB directory opens in one process only (`SB003`).
  An embedded store therefore cannot live in a worker or in a Python process that runs native code
  and also serve other processes. ADR-0164 Options also set embedding aside for build and
  resource-envelope reasons.
- **Consequence.** Keep remote for the cross-process tier. An embedded or no-store tier fits only a
  process that is the sole store user (an ephemeral interactive session) [I].

### O11. Process-level admission and supervision (W-B 1b-2, route C stages 1–3; supervisor, host ledger)

- **Ownership.**
  - Filesystem primitives cover the mechanisms already used: `flock` exclusion released on crash,
    and fsync+rename for JSON publication (F18).
  - systemd and cgroups (outside R1's libraries) cover scope caps and drain proof.
  - No library here offers cross-process capacity accounting.
- **Necessity.** T for managed workers. Host admission is shared with agent tooling (W-B §(a)):
  product use is T, tooling use is H.
- **Consequence.** Not reducible by R1 libraries beyond what is used. R5 owns capacity.

### O12. Recovery after loss (W-B §(a): lease-expiry reconcile, `recover_study_point`, `repair_study_readiness`, `abandon_study_inputs`, `recover_artifact_intent`)

- **Ownership.** SurrealDB partly:
  - atomicity means interrupted transactions leave nothing (F1);
  - ASYNC events could run repeat-safe follow-ups but cannot own recovery (F4);
  - a CHANGEFEED gives a bounded replay window.
  The decision logic ("never invents scientific success") is the project's.
- **Necessity.** T for durable studies and artifact creation (ADR-0182 proposed, which carries R
  parts). Readiness repair exists because readiness is a derived, separately written record [I]. A
  same-transaction grouped view or event (F4/F5) would make readiness consistent by construction and
  remove repair, at the cost of a hot key.

### O13. In-process admission and native join (W-B 0; W-A JOB, SESSION)

- **Ownership.** tokio `Semaphore` (owned permits, `close`) is already used. Native thread join and
  flag cancellation have no library owner (F17 limit). This is R5's.

---

## 2. Refined allocations

### A. SurrealDB-native lifecycle (remote)

- **Supported by:**
  - Fenced claims are the only cross-process exclusion (W-B §(a)). Durable studies, managed workers
    and multi-worker studies are product features. Store-side atomic claims, sequences and
    reference-guarded retention fit exactly that tier.
  - Server functions are already the house style (94 functions), so folding per-point steps (O5) and
    making protections reference rows (O2, O7) are additive.
- **Undermined by:**
  - The heaviest interactive cost is not invariant enforcement. It is protection churn and
    qualification rechecks (O2, O3), plus registration and heartbeat for runs that die with their
    process (O4). Native constraints do not remove those unless protections become typed,
    session-scoped references.
  - Grouped views and sync events would sit on the same hot keys that already serialize writers
    (`retention:` guard, EFF L2-02).
  - Strings are used as identities throughout (I1), so `REFERENCE` needs a typed-identity schema
    change first.
- **Refinement.** Restrict A to the durable tier. Use typed references for the retention
  dependency graph (protections, manifests to blocks, artifacts to obligations), sequences for
  fences, and single-function multi-step transitions per point. Keep polling as the correctness path
  with LIVE as a hint.

### A-E / B-E. Embedded variants

- **Undermined** for every tier that has managed workers (O10): one process per directory, native
  crashes take the process, workers are separate scopes.
- **Survives** only as an optional store for an isolated ephemeral or interactive session that no
  other process touches, which is a second store topology. Or survives as "no store": see B'.

### B. Minimal durable core

- **Supported by:**
  - W-A §(a): the scientific kernels and caches are already in-process. The store's
    interactive-loop contributions are protection, requalification, durability of results and
    cross-process coordination.
  - W-B §(a): the ephemeral sweep consumes none of the run, attempt or claim recovery obligations.
  - In-process runs die with their process.
- **Undermined by:**
  - Durable studies with managed workers (T) need claims, leases and recovery in the store.
  - Source persistence before preparation (O1) and multi-runtime lifetimes (O9) are current rules
    (§19, the scope table, ADR-0181).

### B'. Two-tier refinement (derived from the ledgers; Proposed)

- **Interactive and ephemeral tier.**
  - Process-local revision (`Arc`), Salsa and in-memory caches as the reuse authority.
  - tokio `TaskTracker`/`CancellationToken` lifetimes.
  - No protections and no requalification.
  - The store is touched only to publish (explicit or on durable run) and to read by identity.
  - Reclaim safety by rule: a retention reclaim either excludes open runtimes or meets typed
    `REFERENCE REJECT` rows.
- **Durable tier** (studies, managed workers, durable `RunHandle`s whose recovery another actor
  must adopt):
  - remote SurrealDB with A's refinements;
  - per-point work reduced to fenced claim-and-start, optional progress, and one terminal publish
    transaction carrying outcome and readiness.
- **What remains the project's:** lease expiry and recovery decisions, unknown-outcome settlement
  (the patch or an upstream equivalent), the study policy, native join, and the generator as the
  single schema authority.
- **What is lost:**
  - durable reproducibility of every interactive solve unless it is published;
  - live cross-client visibility of in-flight interactive work;
  - concurrent reclaim during interactive sessions, under the exclusive-reclaim variant.

---

## 3. Repeated or reconstructed work removed by construction

Each item below is Proposed and has an [I] basis.

1. **Root declaration re-read under its own protect/release on every solve, inspect and bound solve
   (W-A §(c)).** An in-process revision (B') or one session-scoped `REFERENCE` protection (A) makes
   the row available without a grant.
2. **Per-call protection grant and release, two pairs per solve (W-A 1.8, 1.9, 1.11; 3.1/3.6).** A
   reference row or `Arc` lifetime replaces cycles with existence: zero writes per call after the
   first.
3. **Selected-admission recheck against the store on every preparation (W-A §(c)).** Immutable
   revision plus held reference: the answer cannot change while held.
4. **Body-description publication the session cannot consume under abi3 (W-A 1.10).** Write only
   when a consumer exists (publication tier).
5. **Result reads as write transactions taking the `retention:` guard (W-A 4.2).** Manifest-to-block
   `REFERENCE REJECT` plus snapshot reads make reads plain reads.
6. **Heartbeat every 10 s and lease for runs that die with their process (W-A 1.14, W-B 3-2,
   4a-2).** `TaskTracker`/token ownership in-process, with restart reconciliation from non-terminal
   records.
7. **Readiness repair (`repair_study_readiness`) and derived readiness records (W-B 1b-7).** A
   same-transaction derived record (view or sync event, or written inside the terminal transaction)
   is consistent by construction. Hot-key trade-off.
8. **Separate per-point RPCs for mark-started, prediction and renew (W-B 1b-11).** One server
   function transition (F8).
9. **100 ms waiter poll and idle discovery polling (W-B 1b-6, 1b-7).** LIVE SELECT wake-ups over the
   existing ws, with polling retained as recovery. This reduces work but adds no correctness.
10. **Store connect and open for ephemeral runtimes (W-A §(a) item 1; W-B 0).** Opening the store
    becomes optional in the interactive tier.

Not removed by R1 libraries: worker recompilation (R2), seed re-decode per durable point (R2/R3),
result re-encoding per `.table()` (R3), new native session per point (R4/R5).

---

## 4. Open premises

| # | Premise | Changes | Evidence that would settle it |
|---|---|---|---|
| P1 | Several Python runtimes must concurrently mutate or reclaim one store | Keeps O9's three-lifetime separation and O2 protections in the interactive tier | Maintainer product decision; ADR-0181 is proposed, and the scope page lists multi-writer admission as unresolved |
| P2 | Ephemeral and interactive solves must persist sources canonically before solving | Decides whether B' applies | Decision on §19 / scope-table "still requires canonical sources"; ADR-0164 (proposed) |
| P3 | Another actor must adopt an in-process durable run after its process dies | Keeps heartbeat and lease for in-process runs (O4) | Product decision: is restart reconciliation sufficient for `RunHandle`? |
| P4 | Retention reclaim may run concurrently with preparation or reads | Keeps per-read and per-preparation protections (O2, O7) instead of exclusive reclaim | Who triggers reclaim (Python retention ops, maintenance) and whether it may wait for open runtimes |
| P5 | Native crash isolation requires separate worker processes | Rules out embedded store variants for the durable tier (O10) | Already observed (W-B §(a)); revisit only if workers move in-process |
| P6 | Typed `REFERENCE REJECT`/`CASCADE` hold under concurrent publish and reclaim on the 3.3.0 RocksDB server | Gates A's reference-row protections | Owned RocksDB controls for both insertion and deletion orders and for permissions (I1 trigger); not run here |
| P7 | LIVE SELECT is usable through the patched ws client | Gates wake-ups (items 9 and O5) | Patch replays setup only, and upstream live queries end on reconnect (`SB101`); needs a reading of the patch's live path plus an interruption experiment (I2 conditions) |
| P8 | Hot-key contention from grouped views or sync events is acceptable at 16 lanes | Decides between derived-in-store readiness and the terminal-transaction write | Measurement under the 16-lane profile; not done |
| P9 | "Interruption before publication" must preserve partial long trajectories | Adds incremental durable writes (prefix publication) for 4a | Workload owner's statement of what must survive (I7 checkpoint conditions) |
| P10 | Restart reuse of described bodies is a product requirement | Keeps publication per preparation (O3) | Whether any consumer of store-received bodies exists under abi3 Python (currently refused, W-A 1.2) |

Files written: this file only.
