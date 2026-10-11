# Coordinator verification and reconciliation

This supporting note records bounded checks for the independent audit. It does not own
the overall judgment or finding dispositions. Baseline: `b01a45d71a5f5ab29991ad193679f5588ac3662f`.
The minor concurrent validation edits were preserved, as requested.

## Baseline and conduct

- Original principal review SHA-256 before audit publication:
  `5c178b566cda3ac59d235cd51d12f88dd0336d8cc4608bc47e4cdd83f124a6a3`.
- `just activity --json` initially reported an active native scope and its store. After the
  maintainer arranged cancellation, a second observation reported no active native scope,
  no store service and no build processes; only the compiler-cache service remained active.
  This is an observation, not workload admission or a claim about unknown processes.
- The maintainer expanded permission to run useful audit experiments after cancellation.
  Any workload still uses its ordinary admission route. No product change is authorized.

## Source checks

**V01 — Scoped file observation is cooperative between chunks (Interface-checked).**
Read `crates/pse-buildinfo/identity.rs::FileObservation::{capture_scoped,verify_scoped}` and
`LocalRuntimeObservation::{capture_scoped,verify_scoped}`, plus
`crates/pse-runtime/src/math/portable.rs::ReplayAdmission::{observe_local_scoped,qualify_receiving}`.
The 65,536-byte read loop invokes the supplied checkpoint before each read and before sealing;
receiving qualification passes its original cancellation/deadline checkpoint through the full
recapture and closing file verification. The convenience `capture` supplies a no-op checkpoint.
Loader-scope acquisition and opaque configuration/initialization calls remain separate limits.
Thus original F05's blanket non-interruptibility statement must be narrowed. The isolated
file-observation cancellation control was also executed, as recorded below; it does not exercise
the full receiving, loader or preparation operation.

**V02 — Cubic lock scope is the coefficient evaluation (Interface-checked).**
Read `crates/pse-math/src/implicit_cubic.rs::CubicRoots::solve`. The chained mutex guard is a
temporary ending after `evaluate(parameters, &mut coefficients)`. Rational conversion,
`isolate_real_roots`, refinement and original residual checks execute afterward. Shared scratch
ownership, lock contention and exact-isolation cost are distinct questions; the code does not
serialize the entire root-isolation operation under that mutex. No data race was established.

**V03 — Expression changes can change dependency references (Interface-checked).**
Read `crates/pse-runtime/src/workflow/modeling/canonical.rs::SourceIndex::references`.
It traverses typed and structural paths, requirements and parsed expression occurrences, including
name/segment resolution. Original F02's proposed restriction to name/parent/kind/import changes
is insufficient: edits to dependency-bearing expressions and other fields also need extraction,
and changes in scope/membership can change how an unchanged spelling resolves.

**V04 — Settled native operation records have a retirement path (Interface-checked).**
Read `scripts/native_operation.py::pinned_generations`. Under each record's exclusion, a
provably drained operation is removed unless a pending registration or live foreground-parent
association still owns its proof. Unknown records conservatively prevent reclamation. Original
F05's claim that settled operation records never retire is false. The remaining directory scan
and acquisition costs are separate questions. No real operation record was deleted by this audit.

**V05 — A zero-byte consumer departure can increase fair allocation (Interface-checked).**
Read `crates/pse-engine/src/resources.rs::ReleaseNotifyingPool` and the pinned
`datafusion-execution-55.1.0/src/memory_pool/pool.rs::FairSpillPool` implementation. Spillable
registration increments `num_spill`; unregister decrements it. The fair limit is the space
remaining after unspillable reservations divided by `num_spill`. Therefore dropping an empty
spillable consumer can make a previously refused growth request admissible without releasing
bytes. Original F10(e)'s bytes-only notification remedy loses a real wakeup. Context7's
`/apache/datafusion` documentation supplied a discovery lead; the pinned local implementation
settled the claim. The existing release-notification test was read, not executed.

**V06 — The seed candidate limit precedes scientific eligibility (Interface-checked).**
Read `crates/pse-operations/src/canonical_execution.rs::result_seed_candidates` and
`crates/pse-runtime/src/workflow/run.rs::ModelingSolvePreparation::with_stored_start`.
The former limits to 64 headers before the higher-level terminal eligibility decision. The
public `StoredStart::Latest` caller fails when the higher-level selector returns none. Inquiry B
traces the intermediate selector; the coordinator also read
`workflow/durable.rs::Operations::latest_seed`: it skips nonterminal attempts only after receiving
that bounded vector. Distinguish this global search from exact-predecessor study selection.
Do not extend the cross-attempt masking counterexample to that exact-predecessor route, or replace
corruption/scientific incompatibility errors with silent fallback.

**V07 — Discovery discards an already decoded seed (Interface-checked).**
Read `workflow/durable.rs::Operations::{seed_document,seed,prediction,latest_seed}`.
`latest_seed` calls `seed` but returns only the solution ID. The public start caller reads it
again. Each `seed_document` separately opens result protection and calls `record`, then reads and
decodes the selected seed bytes. Prediction uses that same full document route. This supports
inquiry B's additional admission-object candidate independently of post-seal finish readback.

**V08 — The configured store uses conflict registration, not exclusive reader locks
(Interface-checked).** Read pinned `surrealdb-kvs-3.3.0/src/api.rs::Transactable::getu` and
`surrealdb-kvs-rocksdb-3.3.0/src/lib.rs::getu`. The latter registers keys through the optimistic
transaction with snapshot read options. The proposed protocol still needs exact write/read
conflict witnesses, absent-record creation and both commit orders. No running-server handshake,
concurrency experiment or alternative-backend qualification was performed. Inquiry B contains
the complete source and deployment context.

**V09 — Shooting value requests enter a derivative-producing evaluation (Interface-checked).**
Read `workflow/shooting.rs::ShootingOracle::evaluate`. Any new point with rows or a terminal
objective and nonzero variable count selects forward sensitivity integration, fills derivative
entries and the sparse Jacobian, and computes terminal gradient components. The current point
cache does reuse a bit-identical trial. Inquiry C checks callback demand and the remedy's
scientific constraints; this source check establishes work performed, not measured wasted time.

**V10 — Surrogate advancement returns a full library-state clone (Interface-checked).**
Read `crates/pse-math/src/surrogate.rs::{Command,RetainedSurrogate::new,advance,state,is_finished}`.
Each successful worker `Advance` clones its `EgorState` into a snapshot retained by the caller.
The local `is_finished` consumes iteration and termination fields, not the full sample history.
Inquiry C traces pinned library field ownership and external consumers before recommending a
compact default response. Existing full-state inspection is a separate demand to preserve or
change explicitly; it cannot be declared absent just because the runtime uses less.

**V11 — Relational inspection has public Rust entrypoints (Interface-checked).**
Read `crates/pse-rules/src/lib.rs` and `invariants.rs::run_invariants`, and
`crates/pse-engine/src/session/native.rs::EngineSession::prepare_sql`. The invariant module is
exported; the public async operation compiles registry queries, executes through the session and
returns typed admitted findings. Public SQL preparation resolves through the native session and
checks its bindings/effects. This defeats an inference from missing Python/runtime facade or
in-repository call sites to no exposed inspection capability. Inquiry D distinguishes actually
unconsumed cache capacity and settings from these real contracts.

**V12 — Test-resource metadata already separates some completed history
(Interface-checked).** Read `scripts/test_resources.py::resource_metadata`. Drained, unpinned,
unreferenced/unborrowed resources with settled disposition leave hot metadata after successful
removal; compacted evidence is written to a completed record and report index. The dirty diff
only changes a separate reclaim admission condition, so this counterevidence is in the named
baseline. A hot-ledger full read/rewrite still exists; the cause must name the records that
remain hot rather than all retained history. No ledger or owned resource was modified by this
source inspection.

**V13 — Original validation-context probe was not executed (Interface-checked).**
Original F23 says that F-P1 confirms the rebuild claim, but the original evidence index V14 says
rebuild frequency is probe-pending and the principal's own proposed-probe list includes F-P1.
No executed F-P1 receipt was supplied. `EngineFactory::validation_context` and weak assembly
ownership support a source diagnosis; they do not turn an inferred whole-run count into a
measurement. The independent audit must correct this evidence attribution.

**V14 — Force-validation's correctness obligation is narrower than interactive defaults
(Interface-checked).** Read ADR-0170's current `proposed` status, Scope and Outcome, together
with the supplied AGENTS instructions. Validation is explicitly opt-in outside correctness
checks and must not be enabled by the feature unifier in production. An interactive extension
mode need not relax that correctness obligation. A recipe/provenance change must still ensure
that every actual Arrow-consuming correctness run selects a validated extension/closure.

**V15 — Decoder substitution does not establish complete IPC preflight equivalence
(Interface-checked).** Read `workflow/result_blocks.rs::{layout,preflight_result_block}` and
the existing `oversized_node_and_short_validity_buffers_are_refused_before_arrow_constructor`
control, plus pinned `arrow-ipc-59.3.0/src/reader.rs` null-buffer construction. The decoder
constructs a `BooleanBuffer` before full null-count validation. Avoiding intermediate buffer
copies does not by itself discharge the input lengths and layouts needed by constructors.
Inquiry D's StreamDecoder alternative therefore preserves exact application admission unless
its replacement is shown to enforce those same prerequisites. This malformed-input control
was read, not executed during the audit.

**V16 — Added numerical candidates have specific consumers (Interface-checked).**
The coordinator additionally read shooting's four `NlpOracle` callbacks, which all call the
same evaluation owner; runtime surrogate actor branches, which request compact phase/work/
proposal information; and egobox-ego 0.38.1 `solver/egor_state.rs::SurrogateState`, whose derived
clone owns historical `Array2` sample matrices. IBEX wait loops were read with the
`math/jobs.rs` and `math/staged.rs` enclosing `compute_scoped` calls. These support C's separate
consumer-demand and queued-CPU questions. The IBEX remedy must establish that the enclosing
compute owner is quiescent before releasing its allowance; another member of a live native team
cannot lose its accounting merely because one thread blocks.

## Executed control

**Tested — isolated cooperative file-observation cancellation.** Command:

```bash
scripts/pse-env -- just unit-package pse-buildinfo file_observation_cancellation_checks_each_read_chunk_and_never_seals --execution-effects native-local
```

The ordinary checkout admission route selected one library unit control in `pse-buildinfo`,
using the pinned toolchain, locked dependencies, Nextest's default profile and the native-local
execution effect. Its dependency closure is Arrow-free, so the scope-aware selector correctly
added no unrelated Arrow validation root. The test creates a 256 KiB temporary file, cancels on
the fourth checkpoint, asserts `Interrupted` and four polls, then checks a complete observation
against the convenience capture. Result against the zero-failure baseline: **1 passed, 0 failed,
17 unselected/skipped**, exit 0. Nextest run ID:
`3b3c4b54-1d77-48c4-8b92-1066336b9c43`.

The command reused compiled artifacts. Its elapsed output is not a startup, preparation,
throughput or scalability measurement. No store, solver campaign or integrated qualification
was run. All additional probe descriptions in inquiry reports remain proposals unless explicitly
recorded here as executed.

## Cross-inquiry reconciliation supplied to the principal

All five bounded reports are published in this folder. The fresh principal reviewer receives
them and this note, reads decisive source independently, and settles the combined judgment.
The following are compatibility constraints and evidence distinctions, not an assigned verdict.

- **Interactive binding and preparation.** A direct numerical binding can use existing typed
  overlay/rebind semantics without waiting for a whole incremental compiler. A longer-lived
  preparation owner must retain exact revision, dependency, physical, factory/registry and
  resource/protection context. It must not turn a mathematical body hash into canonical source
  eligibility or an expired pin into live authority. Content payload sharing can initially
  retain one complete selection receipt; narrower premises are a separate proof obligation.
- **Protection and publication.** A's engagement and B's conflict-domain correction depend on
  one consistent root/pin creation, renewal, reclamation and cancellation protocol. Both commit
  orders and named absence matter. A cached acknowledgment cannot authorize a new root after
  reclamation. Returning before persistence is optional behavior change; removing acknowledged
  readback does not require it. Deadlines alone do not settle late issued effects or authorize
  tombstone deletion.
- **Validation lifetime.** A's retained preparation context and D's validation owner refer to
  actual immutable implementations. Function-name inventories do not prove same-named UDFs
  equivalent. Keep per-registry/factory ownership bounded, avoid cycles, and preserve independent
  checks on new numerical states. Untrusted IPC constructor premises remain distinct from
  repeated semantic validation on an unchanged admitted value.
- **Results and seeds.** B's result-index correction needs new exact key semantics or row
  locators, not reuse of floating-point time bounds. Close/seal must declare empty sets and
  coverage. A stored completion cache or admitted seed object needs an explicit snapshot/pin/
  retirement contract. Repeated seed transport and incomplete latest-seed discovery are separate
  causes. A horizon streaming correction includes both Loop and Staged owners; other callers'
  arbitrary historical references remain an explicit contract.
- **Resources and numerical work.** Queueing native jobs is not safe until retained sessions,
  nested jobs and phase transitions cannot hold each other's required permits. C's IBEX waiter
  may yield CPU only when its actual compute owner is quiescent; resident/foreign memory remains
  charged through drain. Matrix/factor reuse requires identical numerical premises or a solver's
  declared refresh policy. Structural matching and a small backward error do not establish
  numerical rank. Carrying immutable same-trial validity does not remove new-arithmetic checks.
- **Development dependencies.** E names concrete composition tests whose checked ownership
  responsibility does not read the store. `Runtime::from_shared` receiving a canonical handle
  is not a hidden service launch, and lazy `Operations` does not remove source admission.
  Correct pure-owner test setup and unnecessary runner build hops independently of larger crate
  decisions. Keep current hot/cold ledger separation and real pending obligations. Both native
  and non-native newly scoped wrapper exits run sweeping; inherited/no-scope paths differ.
  The principal caught and corrected an overly narrow coordinator summary of this branch scope.
- **Rule impacts versus routes.** A new crate may require an ADR under current policy; following
  that route is not a recommendation to repeal the policy. Likewise interactive extension mode
  can change without relaxing correctness validation. Sequential transfer of emptied scratch
  may satisfy attempt ownership without waiving DP-19. Plans, policies and current implementation
  choices do not waive a standard MUST if retained.

The coordinator additionally read E's named tests
`workflow_composition_preserves_custom_same_owner_factory_and_partition_demand` and
`workflow_composition_refuses_foreign_consumed_owners_and_width`, and the native wrapper's
new-placement `finally` path. Inquiry E preserves the complete assertion/owner reasoning.

**Candidate handling.** B's incomplete discovery and repeated seed admission, and C's IBEX
waiting, surrogate snapshots and shooting derivative demand have separately identified causes
and consumers. D's C-stream signal bridge has a concrete different call path but no controlled
interruption observation. E's disabled-capability build-script environment sensitivity is
source-backed; wider downstream rebuild and materiality remain unmeasured. These limitations
must travel with any promotion, narrowing or non-finding decision. No new performance campaign
or infrastructure experiment is required merely to make the audit appear more comprehensive.

**Priority and prerequisites.** Consequence-based urgency is separate from contract order.
The target's value route, amplification in ordinary studies, and development responsibility
dependencies can be urgent without a timing measurement. Assuring identity, protection and
publication semantics may precede particular optimizations even when that contract work has
no separately demonstrated latency cost. Optional new query APIs, broad compiler restructuring,
host-utilization tuning and numerical-method substitutions are not prerequisites for every
narrow correction. The principal decides priority; future plan creation chooses disposition
and work order after rule-impact decisions.

The aggregate investigation does not certify physical/numerical adequacy, solver ABI behavior,
all extension combinations, long-run resource growth, current store query plans, every Python
signal path or performance at a claimed scale. No numeric latency or memory improvement is
inferred from a source count. Original positives and negatives are both subject to the same
evidence standard.

## Publication check

The published principal matches the independent reviewer's final report. All 29 original
finding positions, 19 original rule-impact positions, five new findings, conditional RC20,
foundation/profile gates and nine representative scenarios are present. All 33 local Markdown
links across the principal and evidence resolve, including their anchors. The original review's
SHA-256 remains identical to the baseline above. This is document integrity checking, not
additional product qualification. Only the new audit and its evidence files were authored;
repository-wide auto-formatting was not run over the concurrent source edits.
