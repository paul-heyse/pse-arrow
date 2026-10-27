# Comparative library leverage reviews

**Account state: first comparative review, 2026-10-27.** This review covers every live
opportunity in the [codebase account](../codebase/README.md) (`CBO-001`–`CBO-052`) across nine
group pages. It is focused comparative advice under the
[opportunity review skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-opportunity-review/SKILL.md).
It recommends bounded targets and leaves choices unresolved where the evidence does not settle
them. It does not authorize implementation, accept scope or replace a formal review due under the
existing cadence. Every target is *Proposed*; enabling evidence is labelled where it is used.

| Item | Value |
| --- | --- |
| Consumed accounts | Codebase account `c2f8b485a` (source baseline `dbc4c9436`) with its dated library-assessment corrections; library assessment `34f9f07ee`; [additional patterns](../assessment-patterns.md) revision 2, now revision 3; base [patterns](../patterns.md) edition 0.2 |
| Source inspected | `main` at `3e9a2525b` by the group reviewers, `9cb8e4f4a` by the probes, published at `87f2eb905`. The commits in between touch canonical transport, staging and Plan56 repairs; each page records any change it saw. Concurrent uncommitted product edits were not used as evidence. |
| Standard | Core 3.4 (`design-principles.md`, `efficient-architecture-heuristics.md`), process-simulator profile 1.5 and the pse-arrow binding, via `standard.toml`, applied qualitatively |
| Method | Nine parallel group reviews, five coordinator-run probes, a maintainer checkpoint, then two independent challenges ([omission and preservation; numerical and solver methods](#challenges)) |
| Premises | Accepted: this host's kernel 7.0 and systemd 255 floors (pidfd and pidfs, `cgroup.kill`, `cgroup.events`, `Delegate=yes`, the D-Bus API). Standing: SurrealDB canonical for runtime and scientific data with 16-lane parallelism and no daemon; FMA-free determinism; panic=unwind; typed errors; one Arrow family; no library or licence refused; no compromise that regresses another part. Unresolved: [U1–U3](#unresolved-premises-and-their-branches). |

## Groups

| Group | Page | Opportunities |
| --- | --- | --- |
| G1 | [Host processes, containment and durable files](host-processes-and-durable-files.md) | 003, 004, 019, 020, 047, 048 (non-SurrealDB) |
| G2 | [Execution lifecycle, cancellation and observation](execution-lifecycle-and-observation.md) | 010 (retry policy), 011, 013, 014, 015, 016 (in-process), 017, 018 |
| G3 | [Canonical store transport and SurrealQL](canonical-store-transport.md) | 009, 010 (SurrealQL), 016 (store), 046 (SurrealQL), 048 (SurrealDB) |
| G4 | [Result data paths and prepaid accounting](result-data-and-prepaid-accounting.md) | 001 (joint judgment), 002, 005, 006, 007, 008, 012 |
| G5 | [Mathematics preparation and structure](mathematics-preparation.md) | 021–025, with the mathematics input to 001 |
| G6 | [Native solver integration](native-solver-integration.md) | 026–032, with the native input to 001 |
| G7 | [Numerical workflows](numerical-workflows.md) | 033–036 |
| G8 | [Declarations, registry, Python boundary and diagnostics](declarations-registry-and-python-boundary.md) | 037–045, 046 (templating) |
| G9 | [Developer tooling](developer-tooling.md) | 049–052 |

## Synthesis

Most of the codebase's library use is already sound: of the 58 judged items (six CBOs are split
across groups or mechanisms), 26 keep their current mechanism or have their premise rejected
after inspection. The recommendations that remain are few, local and mostly
consolidations. Four of them are also **defect repairs** that the comparison exposed and that
should not wait for the leverage work around them:

1. **A replay document can abort the process** (CBO-005, *Tested*). arrow-ipc 59.3.0's
   `StreamReader` allocates whatever body length a message claims before reading it, and a
   user-supplied reproducer, bundle or pack reaches it without authentication. A 408-byte stream
   claiming a 1 PiB body ends in SIGABRT, which `panic = "unwind"` cannot contain, so a Python
   caller loses the interpreter ([probe](../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)).
2. **The cubic root ladder can lose a root** (CBO-025, *Tested*). Symbolica 3.0.1's root
   isolation can return an interval that strictly contains a second root. The local ladder then
   panics in debug and, in release, silently returns the double root twice: 20 of 8,556
   double-root cubics lost their simple root
   ([probe](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)).
3. **SUNDIALS patch content is outside solver identity** (CBO-028, *Interface-checked*).
   Native-build identity frames the bundled version but not the local numerical patch, so a
   patch change at an unchanged version moves no profile identity.
4. **One failure reports two diagnostic codes** (CBO-045, *Interface-checked*). `CanonicalError`
   declares miette code `runtime::infrastructure` while its typed mapping gives `Codec` the code
   `ValidationInvariant`. Python shows the first in the exception message and the second in
   `report.code`.

The numerical challenge added a fifth finding of the same weight, which is a premise conflict
rather than a single defect:

5. **faer's dense kernels use FMA chosen at run time** (*Interface-checked*). faer 0.24.4 enables
   `pulp/x86-v3` unconditionally, and pulp's run-time V3 path uses `_mm256_fmadd_pd`. The
   workspace's FMA-free rule (no `target-cpu=native`) does not reach this route, and native-build
   identity frames only compile-time facts, so faer-backed results can vary by CPU class under an
   unchanged identity. Removing it costs dense-kernel speed or needs an upstream change, so it is
   recorded as [remedy-open](native-solver-integration.md#faer-kernels-and-the-fma-free-premise)
   with U1 and U2 branches; the choice is the maintainer's.

The same challenge found that Numerica 3.0.1 `Rational::to_f64` rounds twice for large
numerators or denominators, which affects today's cubic ladder and coefficient conversion; the
CBO-025 target below avoids it.

### Headline recommendations, by consequence

1. **One IPC admission at the trust boundaries** ([CBO-005](result-data-and-prepaid-accounting.md#cbo-005)).
   Build one `pse-columnar` admission on arrow-ipc 59.3.0 `FileDecoder` and `StreamDecoder` with
   caller-reserved, 64-byte-aligned block buffers attached through the existing
   `AllocationScope::attach_buffer`. Keep every local check 59.3.0 still needs (buffer spans
   panic in both decoders). Extend it to the reproducer, review-pack, portable-payload and
   physical-object paths first, because that part removes the abort. The double footer read and
   the ×4 and ×2 grants disappear.
2. **Square-free isolation and exact bisection for cubic roots** ([CBO-025](mathematics-preparation.md#cbo-025)).
   Isolate on `u = p / gcd(p, p′)`, refine with a local exact bisection (derivative-side rule at
   root endpoints, cancellation every step), certify the returned double by the exact sign of u at
   its two rounding half-way points, convert exactly rather than through `Rational::to_f64`, and
   refuse typed only at a step cap derived from the bracket and the binary64 exponent range, so
   roots near the subnormal range are still returned. Delete `refine_root_interval`, the endpoint screen and
   the `IsolatedRoot` fallback, and advance the solver identity. This also stops the cubic path
   from filling Symbolica's process-global root cache ([LOP-X014](../assessment-patterns.md#lop-x014)).
3. **A repository-owned SurrealDB client on the published protocol** ([CBO-009](canonical-store-transport.md#cbo-009)).
   Replace the vendored SDK delta with a `pse-operations` module over `surrealdb-protocol` 0.13.1
   `SurrealDbServiceClient` and `surrealdb-types` 3.3.4 `proto`, pinning the four SDK result
   rules the [direct-client probe](../../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md)
   needed for 37/37 parity. The two lifecycle tiers become one owner and the vendored SDK leaves
   the workspace. Do it instead of the next SDK rebase, or earlier if a transport change is needed.
4. **Registry products as a guarded postcard blob** ([CBO-040](declarations-registry-and-python-boundary.md#cbo-040)).
   The [build-coupling probe](../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md)
   showed that adding a model field breaks `just codegen` with no routine path and that the
   generated module is about 72% of `pse-schema`'s build. Carry the products as one postcard 1.1.3
   blob with a shape-digest guard; the 87k-line literal module and its emitter disappear. This
   amends proposed ADR-0210 before acceptance.
5. **Prepaid accounting by kind of size information, one owner per residual bound**
   ([CBO-001](result-data-and-prepaid-accounting.md#cbo-001)). Use the strongest size information
   each library offers, and give every remaining library-internal bound one named owner that
   records its derivation and pinned version. The decode charge becomes the exact block span; the
   FERAL Markowitz bound moves into the maintainer-owned POUNCE fork; one conservative B-tree
   helper replaces four inconsistent formulas. Whether a derive should produce `HeapUsage` stays
   unresolved until a census of the hand implementations.

Next in consequence, each small and local: make `impl_diagnostic!` the single diagnostic
authority ([CBO-045](declarations-registry-and-python-boundary.md#cbo-045)); exhaustive
destructuring in about 20 identity frames ([CBO-039](declarations-registry-and-python-boundary.md#cbo-039));
one no-clobber publication helper on `renameat2(RENAME_NOREPLACE)` ([CBO-003](host-processes-and-durable-files.md#cbo-003));
the runtime observing the primary through cgroup `populated` plus a pidfd instead of spawning an
interpreter per heartbeat ([CBO-020](host-processes-and-durable-files.md#cbo-020)); and the
SUNDIALS 7.9 move after its identity prerequisite ([CBO-028](native-solver-integration.md#cbo-028)).

### Prerequisite order, kept separate from priority

- **SUNDIALS identity before SUNDIALS numerics.** The native-build identity must cover vendored
  patch content before the 7.9 move or any hunk removal. The fix may be a bug fix under the
  existing hashing contract or need a hashing-contract decision; that is for the owner to judge.
  Whatever resolves faer's run-time FMA dispatch extends the same identity question.
- **ADR-0210 amendment before the registry blob.** It is a proposed ADR, so the amendment goes
  through the decision route before acceptance.
- **The Python wait helper before native pyclasses.** CBO-017's wait helper lives in the same
  `#[pymethods]` that CBO-044 would make public, so it lands first and CBO-044 carries it.
  CBO-044 itself needs an ADR and a design review (Python-boundary contract).
- **The supervised child before the compiler-child move.** G1's `SupervisedChild` in `pse-diagnostics`
  absorbs `pse-math`'s compiler child; G5 consumes it rather than refactoring in place.
- **One port of the query sites.** CBO-010's guarded-script envelope and CBO-009 B's client
  touch the same ~355 query sites; design the envelope as the future client's script type and
  schedule them together rather than rewriting the sites twice.
- **The IPC admission carries the exact decode charge.** CBO-001's decode part is delivered by
  CBO-005's module, not separately.

## Interactions between groups

- **One request seam for the canonical store.** G2's `retry_conflicts` owner and `ExecutionContext`
  wrap whatever request builder G3 lands on. Under CBO-009 B, three task-local carriers become
  typed request options on the repository-owned client; the attribution context then feeds those
  options, while authority carriers (deadline, activity, admission scopes) stay explicit. G9's
  xtask evidence tools use the same transport and move with it. Nothing in G2 depends on which G3
  option is chosen.
- **Retry and guarded scripts.** CBO-010 is split: G3's typed guarded-script envelope owns
  framing, guard variant, bind set and result slot; G2's retry owner owns pacing, caps and the
  settle-before-retry classification ([LOP-X012](../assessment-patterns.md#lop-x012)). Neither
  needs a library retry combinator.
- **Waiting and notification.** CBO-016 is retained on both sides. The bounded profile refuses
  LIVE, so a store subscription would be a new repository-owned request class; if one is ever
  added it enters G2's waits as a hint before the same authoritative re-read, with polling as the
  backstop.
- **Prepaid accounting.** CBO-001 joins G4's Arrow charges, G5's B-tree formulas and Symbolica
  mirrors (retained, because they already reconcile against the built product), and G6's FERAL
  bound and admission gaps (`conditioning::jacobian_condition` and Diffsol `FaerLu` scratch).
  Replacing G6's dense opaque factor allowances with structure-derived bounds stays unresolved
  until checked against real Jacobians, because a cubic charge on whole-Jacobian diagnostics would
  refuse work that runs today.
- **Shared helpers.** G1's publication helper serves the store, export and review pack; G6's
  `QuiescentGate` sits on G2's `WakeSignal`/`Cancellation` primitives; G1's `SupervisedChild`
  serves `pse-math` and the capture launcher.
- **Supervision.** CBO-048 is retained in both G1 and G3. The supervisor script stays the single
  owner of unit lifecycle; G1's CBO-020 only hands the runtime evidence handles
  (cgroup and pidfd) with the `ensure-primary` result.
- **Trust placement in both directions.** CBO-005 adds a guard where untrusted bytes enter; CBO-041
  removes a guard that is re-run on bytes the trusted producer just emitted in the same process.
  Both follow the same refinement of [LOP-X009](../assessment-patterns.md#lop-x009).

## Unresolved premises and their branches

- **U1: are upstream contributions an acceptable recommended option?** No recommendation depends
  on it; each page names what it would add, still not available until released:
  - Symbolica: abort and vectorize limits (CBO-021), a fallible refiner (CBO-025);
  - `surrealdb-types`: a public `proto::SurrealError → Error` conversion, replacing the hidden
    `Error::from_wire` (CBO-009);
  - arrow-rs: span errors instead of assertions in `read_buffer`, footer-length checks, verifier
    options on the decoders (CBO-005);
  - Diffsol, `sundials-sys` and Uno option forwarding (CBO-027, CBO-028);
  - msgspec required tags, pyo3 and schemars items (CBO-041, CBO-044, CBO-042), all marginal.

  Whether changes to the maintainer-owned POUNCE fork count as upstream contributions is an
  [open question](#open-questions); G6 treats them as local integration, following the fork's
  existing practice.
- **U2: replay of recorded values or regeneration from a seed?** It matters only for CBO-033 and,
  on reopening, CBO-034. Under replay, the design producer's realized points are recorded and key
  identity, as identities already do. Under regeneration, the producer must also be bit-portable,
  which egobox-doe's Classic LHS under a seeded `Xoshiro256Plus` supports, but downstream fit
  statistics still vary by CPU class because faer selects SIMD kernels at runtime
  ([LOP-X006](../assessment-patterns.md#lop-x006) revision 3). The evidence leans toward replay.
- **U3: must the development evidence index stay on SurrealDB?** CBO-050's recommendation
  (generated msgspec types in place of hand TypedDicts) holds either way. Only the store choice
  depends on U3; under "no", the index could leave the canonical transport and shrink G3's
  migration footprint.

## Open questions

These came up during the review. None blocks a recommendation, and each page states its default.

- Is it intended that the HiGHS process gate stays held while blocking on `resume_compute`
  (CBO-030)?
- Is it intended that window-team lanes run without the caller's tracing span (CBO-014)?
- Study admission renews leases every lease/3 inline, pausing guarded work, rather than at the
  admitted heartbeat. Is the inline pause intended (CBO-015)?
- Is `begin_stage_inner`'s missing retry jitter intentional (CBO-010)?
- Does work in the maintainer-owned POUNCE fork sit outside U1 (CBO-001 native input)?
- How should faer's run-time FMA dispatch be resolved: accept CPU-class variation with the ISA
  level recorded, patch pulp to scalar at a speed cost, or seek an upstream override (U1)?
- Should the defects above be raised through the bug and plan routes now, independently of
  this review?

## Disposition of every opportunity

| CBO | Group | Disposition | Target or reason |
| --- | --- | --- | --- |
| [001](result-data-and-prepaid-accounting.md#cbo-001) | G4 (G5, G6 inputs) | Recommend; derive sub-question unresolved | Classify charges by kind of size information; one owner per residual bound; exact decode spans |
| [002](result-data-and-prepaid-accounting.md#cbo-002) | G4 | Recommend | One `pse-ids` sink with a typed `LimitReached`; delete the dead trial-encode path |
| [003](host-processes-and-durable-files.md#cbo-003) | G1 | Recommend | `rustix::fs::renameat_with(NOREPLACE)` publication helper with explicit durability; staging contracts kept |
| [004](host-processes-and-durable-files.md#cbo-004) | G1 | Recommend | One `SupervisedChild`; delegated-leaf containment as an explicitly signalled mode |
| [005](result-data-and-prepaid-accounting.md#cbo-005) | G4 | Recommend | One IPC admission on `FileDecoder`/`StreamDecoder` at every trust boundary |
| [006](result-data-and-prepaid-accounting.md#cbo-006) | G4 | Recommend | Column-wise indexing with `arrow_ord::partition` and `BooleanBuffer`; same `f64::min`/`max` fold |
| [007](result-data-and-prepaid-accounting.md#cbo-007) | G4 | Recommend; planner premise rejected | One `OutputPredicate`; dense windows already run in SurrealQL |
| [008](result-data-and-prepaid-accounting.md#cbo-008) | G4 | Retain current | Heterogeneous open-keyed events; overflow joins CBO-002 |
| [009](canonical-store-transport.md#cbo-009) | G3 | Recommend | Repository-owned client on `surrealdb-protocol` 0.13.1; vendored SDK leaves |
| [010](execution-lifecycle-and-observation.md#cbo-010) | G2, G3 | Recommend (both parts) | Local `retry_conflicts` owner; typed guarded-script envelope |
| [011](execution-lifecycle-and-observation.md#cbo-011) | G2 | Retain current | Completion-owned flights are the core; delete the unused `cache_service::flight` wrapper |
| [012](result-data-and-prepaid-accounting.md#cbo-012) | G4 | Retain current | Field-local visible-value checks differ from cross-row plans |
| [013](execution-lifecycle-and-observation.md#cbo-013) | G2 | Recommend | Owner-typed lifecycle events; sinks stay independent |
| [014](execution-lifecycle-and-observation.md#cbo-014) | G2 | Recommend | `ExecutionContext` for attribution carriers only |
| [015](execution-lifecycle-and-observation.md#cbo-015) | G2 | Recommend | Typed stop reason, drain helper, one `LeaseRenewal` |
| [016](execution-lifecycle-and-observation.md#cbo-016) | G2, G3 | Retain current | Waits are cross-process; the bounded profile refuses LIVE |
| [017](execution-lifecycle-and-observation.md#cbo-017) | G2 | Recommend | One interruptible Python wait helper |
| [018](execution-lifecycle-and-observation.md#cbo-018) | G2 | Recommend | One `NativeThread` owner; thread pidfds deferred |
| [019](host-processes-and-durable-files.md#cbo-019) | G1 | Recommend (minor) | procfs-core 0.18 reader in `pse-diagnostics`; start-tick identity retained |
| [020](host-processes-and-durable-files.md#cbo-020) | G1 | Recommend | Cgroup `populated` plus main-PID pidfd from the handoff; no per-heartbeat spawn |
| [021](mathematics-preparation.md#cbo-021) | G5 | Retain current, add a pin hold | Mirrors already reconcile with typed refusal |
| [022](mathematics-preparation.md#cbo-022) | G5 | Premise rejected | Four layers are four lifetimes |
| [023](mathematics-preparation.md#cbo-023) | G5 | Recommend (narrow) | One checked dense-decomposition function in `pse-structural` |
| [024](mathematics-preparation.md#cbo-024) | G5 | Premise rejected | One factorable projection; per-consumer rules are semantics |
| [025](mathematics-preparation.md#cbo-025) | G5 | LDLᵀ: Retain current. Cubic: Recommend | Square-free isolation and exact bisection; identity advanced |
| [026](native-solver-integration.md#cbo-026) | G6 | Recommend | One shared faer-LU helper; faer's run-time FMA dispatch is a separate [remedy-open](native-solver-integration.md#faer-kernels-and-the-fma-free-premise) finding |
| [027](native-solver-integration.md#cbo-027) | G6 | Retain current, two local fixes | Typed terminal cause; outdated `adjoint.rs` doc |
| [028](native-solver-integration.md#cbo-028) | G6 | SUNDIALS: Recommend after the identity fix. Uno: Retain current | SUNDIALS 7.9.0 (at least 7.5) |
| [029](native-solver-integration.md#cbo-029) | G6 | Premise rejected | One translation chain, not several |
| [030](native-solver-integration.md#cbo-030) | G6 | Recommend | `QuiescentGate` helper; hand KINSOL LRU kept |
| [031](native-solver-integration.md#cbo-031) | G6 | Premise rejected | One hub form with forward maps |
| [032](native-solver-integration.md#cbo-032) | G6 | Premise rejected; retire | The named facilities are in use |
| [033](numerical-workflows.md#cbo-033) | G7 | Recommend | Seeded design producer on egobox-doe 0.35.1; U2 branches |
| [034](numerical-workflows.md#cbo-034) | G7 | Retain current | A new category; first-order scope is the contract |
| [035](numerical-workflows.md#cbo-035) | G7 | Retain current | Removing LM removes no linear-algebra family |
| [036](numerical-workflows.md#cbo-036) | G7 | Retain current | A parity completion that supersedes ADR-0118 item 2; decision work, not leverage |
| [037](declarations-registry-and-python-boundary.md#cbo-037) | G8 | Retain current | Context-sensitive grammar; two optional hardenings |
| [038](declarations-registry-and-python-boundary.md#cbo-038) | G8 | Recommend (minor) | miette rendering at the source-owning boundary |
| [039](declarations-registry-and-python-boundary.md#cbo-039) | G8 | Recommend | Exhaustive destructuring in about 20 struct frames |
| [040](declarations-registry-and-python-boundary.md#cbo-040) | G8 | Recommend | Postcard blob with a shape-digest guard; amends proposed ADR-0210 |
| [041](declarations-registry-and-python-boundary.md#cbo-041) | G8 | Recommend | Private trusted Rust→Python decode; one header check |
| [042](declarations-registry-and-python-boundary.md#cbo-042) | G8 | Retain current | Strict flatten and naming are not library-supplied |
| [043](declarations-registry-and-python-boundary.md#cbo-043) | G8 | Retain current | Two contracts, not one meaning twice |
| [044](declarations-registry-and-python-boundary.md#cbo-044) | G8 | Recommend (needs ADR and design review) | Native pyclasses become the public types |
| [045](declarations-registry-and-python-boundary.md#cbo-045) | G8 | Recommend | `impl_diagnostic!` is the authority; migrate the 10 derive sites |
| [046](declarations-registry-and-python-boundary.md#cbo-046) | G3, G8 | SurrealQL: Retain current. Templating: Recommend | Qualifier context passed to emitters; no template engine |
| [047](host-processes-and-durable-files.md#cbo-047) | G1 | Recommend | One stdlib ledger and publication module |
| [048](host-processes-and-durable-files.md#cbo-048) | G1, G3 | Retain current | Manager facts, cgroup evidence and ledger already combine |
| [049](developer-tooling.md#cbo-049) | G9 | Retain current | One declaration with derived per-run outputs |
| [050](developer-tooling.md#cbo-050) | G9 | Recommend | Generated msgspec types replace hand TypedDicts; store choice follows U3 |
| [051](developer-tooling.md#cbo-051) | G9 | Retain current | SurrealDB 3.3.4 exports OTLP only over gRPC |
| [052](developer-tooling.md#cbo-052) | G9 | Premise rejected | Layers with distinct owners |

## Probes

All five ran on this host on 2026-10-27, against disposable fixtures; none touched the canonical
store or the managed primary.

| Probe | Decided | Outcome |
| --- | --- | --- |
| [Delegated cgroup containment](../../design_review/evidence/library-leverage-delegated-cgroup-containment-2026-10-27/README.md) | CBO-004, CBO-020 | Works in a `Delegate=yes` user scope: leaf creation, pre-exec move, `cgroup.kill` of escaped descendants, `POLLPRI` `populated 0`, scope limits charged. No `user.delegate` xattr under the user manager, so delegation must be signalled explicitly; `+memory` is unavailable with the supervisor in the root. |
| [Hostile IPC stream](../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md) | CBO-005 | `StreamReader` aborts on a claimed body length; `StreamDecoder` returns typed errors; both panic on out-of-body spans. The replay route is unauthenticated before decode. |
| [Symbolica root containment](../../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md) | CBO-025 | Containment reproduced; debug panics, release loses roots; the fallback and root cache are the common path. |
| [SurrealDB direct client](../../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md) | CBO-009 | 37/37 parity with four SDK rules reproduced; one hidden API dependency. |
| [Registry build coupling](../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md) | CBO-040 | A model field breaks `just codegen`; the generated module is about 72% of `pse-schema`'s build. |

The assessment's earlier probes ([egobox-doe determinism](../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md),
[msgspec tagged decode](../../design_review/evidence/library-leverage-msgspec-tagged-decode-2026-10-27/README.md),
[Symbolica root refinement](../../design_review/evidence/library-leverage-symbolica-root-refinement-2026-10-27/README.md))
were reused by G7, G8 and G5.

## Challenges

Two independent design-reviewer challenges read the group pages without the coordinator's
framing. Neither changed a Recommend into Retain; both changed what several recommendations
may promise. Revised passages are marked on the pages.

- **Omission and preservation** (all nine pages; 15 findings, all applied). Bounded targets
  changed for: CBO-018 (a join observer per owner, so staged-session joins stay out of
  `native_joined()`); CBO-004 (`SupervisedChild` moves to `pse-diagnostics`, which already holds
  rustix, `Cancellation` and `WakeSignal`); CBO-040 (both sides of the shape digest derived from
  the types, hash-ordered members such as `logical_storage` carried canonically, and a
  two-run byte-identity check, else remedy-open for that member); CBO-009 (truthful dispatch
  uncertainty retained, a pin hold for the hidden `Error::from_wire`, the interactive-transaction
  test path and xtask's wider SDK use); CBO-026 (Diffsol scratch charged against the existing
  allowance, not a smaller budget); CBO-005 (alignment handling for embedded streams);
  CBO-015 (the new deadline arm in two drains made explicit). Clarifications for CBO-003, 010,
  020, 033/035, 028, 044 and 047, and the `AsyncFd` decision assigned to G2.
- **Numerical and solver methods** (G4–G7; 9 findings, all applied). CBO-025's stopping rule
  replaced by an exact half-way-point certificate with exact conversion and a derived step cap;
  the faer FMA finding recorded as remedy-open; signed-zero handling made explicit for CBO-006;
  KLU's growth rule recognised as not a bound (CBO-001); CBO-033's mapping, level and maximin
  details; CBO-036's parmest example corrected and its optional design made dimensionally
  consistent; and `SymbolicSingular` on a validated pattern classified as a structural breach
  (CBO-026).

## Further decision routes identified

None is initiated by this review.

- **ADR and formal design review:** CBO-044 (Python-boundary contract); CBO-009 B if the owner
  judges removing the vendored SDK to alter a D1–D14 transport decision (no crate is added or
  removed: the module lives in `pse-operations`).
- **Amendment of a proposed ADR:** ADR-0210 for CBO-040.
- **Supersession of an accepted ADR:** ADR-0118 item 2, if CBO-036 is ever taken up.
- **Hashing-contract judgment:** the SUNDIALS identity gap (CBO-028).
- **Architecture amendment:** the stale "Retained scientific rows" text in blueprint §19.2
  (CBO-005); a revision row in `blueprint.md`.
- **Functional-extension review, if wanted:** fix-relax for the advanced step (CBO-032), parser
  error recovery (CBO-037), nonlinear uncertainty propagation (CBO-034).
- **Dependency holds:** a `symbolica` pin hold (CBO-021) beside the pin in
  `[workspace.metadata.pse.pins]`; ordinary dependency moves (procfs-core, postcard, egobox-doe to
  a normal dependency, `rand_xoshiro`, SUNDIALS) need no ADR.

## Patterns

The review refined every additional pattern and added four, as
[additional patterns revision 3](../assessment-patterns.md):
[LOP-X011](../assessment-patterns.md#lop-x011) (a publication contract keeps interrupted work
products), [LOP-X012](../assessment-patterns.md#lop-x012) (a retry's classification requires
settling an uncertain effect), [LOP-X013](../assessment-patterns.md#lop-x013) (a library
kernel's value semantics differ from the domain's) and
[LOP-X014](../assessment-patterns.md#lop-x014) (library-held process-global retention outlives
caller ownership). Base-framework notes for LOP-02, LOP-03, LOP-09 and LOP-12 are recorded there
for optional manual promotion.

## Limits

- Each page records what its reviewer did not read; the largest gaps are most of
  `surreal_server.py`, the 3.7k-line declaration parser, `durable.rs` and `connected_results.rs`
  beyond the paths judged, and the Python wrapper modules beyond samples.
- No quantitative benefit is claimed except where a probe measured it. Over-admission figures are
  multipliers read from source.
- Settling checks named but not run: a classification of the roughly 770 `HeapUsage`
  implementations into derivable and hand-owned (CBO-001 derive),
  structure-derived factor bounds on real Jacobians (CBO-001 native), a QP-initialization check
  on rebased Uno (CBO-028), an inventory of legacy published aliases (CBO-003), result-type
  alignment (CBO-005 inline payloads), and stub quality for native pyclasses (CBO-044).
- The codebase account keeps CBO-032 until its next refresh, which can retire it on this review's
  evidence.

## Maintaining this account

Refresh judgments in place when their decisive premises change, and keep each inherited
judgment's evidence date. Plans own scheduling, progress and closure evidence; link them from
here rather than copying status. When an opportunity retires, carry surviving mechanism
rationale and obligations to their owners before removing its section.
