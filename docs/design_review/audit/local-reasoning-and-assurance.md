# Local reasoning and assurance

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.**

This page holds two primary findings:
- [DA-021](#da-021): the breadth and internal entanglement of the orchestration crate;
- [DA-023](#da-023): divergence between current-contract text and code.

Two related causes have their homes elsewhere:
- failure classification: [DA-039](structure-and-invariants.md#da-039);
- identity derivation: [DA-050](structure-and-invariants.md#da-050).

The [integrated account](system-architecture.md) relates them to the composition cause.

## <a id="da-021"></a>DA-021 — The orchestration crate is one cyclic, broadly visible ownership space holding independently changing responsibilities

**Principles:** AP-01, AP-05, AP-06, DP-17; AP-07 via H27.

**Status:** reopened and widened on 2026-10-29 against Plan56's resolved disposition (B04). The
original "domain values carry the composition root" manifestation is narrowed:
- Modeling packages no longer hold a `Runtime` (`workflow/modeling.rs:141-158`).
- `workflow/artifacts.rs:21, 302` still do.

The flat-ownership half has grown, and independent unanchored streams on structure (U4) and
contracts (U6) rediscovered it.

**Claim at risk.**
- §1.1 and §3.2 name `pse-runtime` the composition root that joins data, mathematics and
  canonical persistence.
- Plan58 targets "independently constructible scientific, control and retained-data
  capabilities".
- AP-06 asks that a change be reasoned about from its owner.

**Structural cause.** One crate owns all of the following, each changing for different reasons.
It has about 343k hand-written lines, of which about 259k are non-test production source.
- scientific methods: fitting with Levenberg–Marquardt, covariance and profiles; dynamics
  accuracy and events; shooting; horizon; multistart; KKT, certified and engineering accuracy;
  surrogates;
- numerical candidate qualification;
- durable and worker execution against SurrealDB;
- project portability, result export and IPC;
- product telemetry exporters: OTLP, reqwest and Perfetto are unconditional dependencies
  (`crates/pse-runtime/Cargo.toml:85-100`);
- developer review and capture tooling;
- three operator binaries.

Its internal module boundaries do not hide decisions:
- **Bidirectional imports.**
  - 18 non-test `math` files import `crate::workflow`; for example, `math/staged.rs:225` holds a
    `workflow::numerics::CandidateDecision`.
  - 86 non-test `workflow` files import `crate::math`.
  - `tooling` and `workflow`/`math` import each other: 12 and 7 files.
- **Re-exports and visibility.**
  - `workflow/mod.rs` re-exports about 480 names through 67 `pub use` statements.
  - 108 top-level workflow files carry 372 `pub(in crate::workflow)` items.
  - The `Runtime` fields `shared`, `model`, `canonical` and `durability` are `pub(crate)`
    (`workflow/mod.rs:577-590`) and are reached directly from about 54 workflow files.
  - The `Durable`/`Ephemeral` mode is matched at about 69 sites.
- **Ambient lifetime.** The canonical-client lifetime reaches native math task spawns as a
  task-local (`pse_operations::canonical::inherit_activity` at `math/jobs.rs:913` and
  `math/staged.rs:1021`, among others).
- **Test seams in production source.** About 136 `canonical-tests` / `native-study-tests` `cfg`
  sites sit in 34 non-test files. Feature-gated public API exists, for example
  `native_worker.rs:524` `with_qualification_native_entry`.

**Consequence.**
- Changing a fitting or dynamics method recompiles the crate that also holds durable workers,
  export, telemetry and binaries, and everything downstream of it (`pse-py`, `xtask`, the
  benches and the `tests-*` crates). The reverse holds as well.
- A library consumer of local pure computation compiles the telemetry exporter closure.
- A maintainer changing one module must read across the math ↔ workflow ↔ tooling cycles.
- Extracting a capability so that it is "independently constructible", as Plan58 intends, first
  requires untangling them.
- The breadth also makes the inline re-minting of identities ([DA-050](structure-and-invariants.md#da-050))
  and the per-mode composition of native admission ([DA-053](boundaries-and-composition.md#da-053))
  easy to write and hard to notice.

**Footprint.** Known across the crate. Suspected: the internal cycles are the principal obstacle
to any later split. That is a hypothesis.

**Counterevidence and legitimacy.**
- The sections deliberately assign fitting, dynamics, numerics and the effect boundary to
  `pse-runtime`, which keeps `pse-compiler` pure.
- A database-free local construction exists (`Runtime::from_shared_local`).
- AP-01 itself says crate splitting is not modularity.

The judgment here is that the responsibilities change for different reasons and that the
bidirectional imports show the module boundaries are not hiding decisions. No particular split
is implied, and build cost is unmeasured.

## <a id="da-023"></a>DA-023 — Current-contract text no longer describes the post-Plan58 structure it claims to own

**Principles:** AP-04, AP-05, DP-24.

**Status:** reopened and widened on 2026-10-29 against Plan56's resolved disposition (P01). The
original named items (an obsolete IR description, a removed rebind API, retired scripts) are
reconciled. The Plan58 cutover has produced a new and wider divergence. The sections are
off-limits to audit edits, so this records diagnosis only.

**Claim at risk.**
- The reading guide says a numbered page "describes implemented behavior unless it labels a
  statement *Proposed* or states a limit".
- `workspace-and-dependencies` says its table "records current implementation until those
  integrated cuts land".

**Divergences read at source:**
- **The host layer is missing from the inventories.** §21.7 describes a Proposed independent host
  root, and revision 311 (`2232bfa02`) adds ADR-0233 notes to the §3 preamble and §21.7. But the
  crate `pse-host` is named in neither the §1.1 layer table nor the §3.2 crate table, so the
  layering rule cannot place it.
  - It is a normal dependency of `pse-runtime` (`Cargo.toml:69`) and, through `native-service`, of
    `pse-operations`.
  - §1.1's Data layer "depends on Declaration and foundations", yet `pse-operations` also depends
    on `pse-host` and `pse-buildinfo`.
- **Other crates are missing.**
  - `pse-study-module`, `pse-petsc-sys` and `pse-uno-sys` appear in neither table.
  - `pse-admission` is listed in §3.2 but is absent from the §1.1 layer table.
- **One edge is wrong.** §1.1 says the Mathematics layer depends on `pse-buildinfo`; no
  mathematics crate does.
- **Backends are missing from §0.5.** The PETSc and Uno backends (features of
  `pse-backend-native`) appear only in §18, not in the §0.5 class table or the §3.2 backend
  description.
- **Ownership statements lag the native host.**
  - §23.1: "Python owns environment resolution". Environment composition is now
    `pse-host::environment`, and `scripts/pse-env` execs it.
  - §3.2: "Python retains service, manager/cgroup and maintenance authority". `pse-host::manager`
    owns user-manager units for native workers and fixtures, while the Python supervisor still
    owns the operator store, so the statement is now half true. §20.6's "Python owns freeze …
    maintenance" remains true for backup and restore.
- **Stale source pointers.**
  - §14.2 names `crates/pse-runtime/src/authoring_driver/p1.rs`, which no longer exists.
  - §5.1 locates `assign_ids` in `pse-runtime::authoring_driver::document`, but it lives in
    `pse-admission/src/authoring_driver/document/edit.rs`.
- **`pse-diagnostics` is understated.** §3.2 describes it as "shared diagnostic vocabulary and
  lossless native causes". It also owns:
  - the observation/producer framework used by 13 crates;
  - Linux `supervised_child` and `process_facts`;
  - telemetry settings;
  - cancellation and stop races (`pse-diagnostics/src/lib.rs:9-31`).
- **§3.2 contradicts itself on worker roots.** It gives `xtask` "current worker composition" in
  the non-product table while saying worker roots belong to `pse-runtime` binaries.
- **Agent instructions disagree with the code.**
  - `CLAUDE.md` still names `scripts/agent-hooks.py` as the hook both runtimes call.
  - AGENTS.md says `.claude/settings.json` "denies them again", but the deny list omits the
    authoritative design and accepted ADRs.
  - Both are detailed under [DA-058](development-tooling.md#da-058).

**Consequence.**
- A maintainer or agent reasoning from the owner sections forms a wrong picture of where host
  control lives, what the Data layer may depend on, which backends exist and who owns
  environment and service authority.
- The layering rule cannot be used to judge a new edge.

**Counterevidence.**
- §3 and §21.7 explicitly declare a transition. §21.5's Python-generation text is a declared
  retiring account and is not counted here.
- Plan58 schedules the architecture amendment (RC07).
- ADR-0233 is proposed.

The host crate has nevertheless landed and is consumed by product crates, so the preamble's
promise to record current implementation is unmet.

## <a id="da-024"></a>DA-024 — Portable producer/replay qualification is reachable only from the retiring Python boundary

**Principles:** AP-02, DP-21.

**Status:** live on a retiring path at 2026-10-29. The broad diagnosis was already not
established at `682fc3830`.
- `workflow/canonical.rs:103-160` (`open_observed`, producer qualification) has one production
  caller, `pse-py/src/workflow.rs:752`.
- The native roots record only outer attestation, with no strict receipt or local-runtime
  admission: `bin/pse-worker.rs:353-360` and `bin/project/runtime.rs:232-243` both use
  `CanonicalDeployment::new(…, None)`.

**Open consequence.** When 58g G03 removes `pse-py`, as revision 311 (`2232bfa02`) schedules,
the public portable API has no first-party consumer. No Plan58 package names whether native roots
need it. The two native roots also derive their producer digest differently
([DA-043](development-tooling.md#da-043)).

**In flux.** Uncommitted edits to `pse-runtime/src/math/portable.rs` and
`pse-buildinfo/src/identity.rs`, made after the baseline was fixed, remove the Python producer
role (`ExpectedProducerTarget::PYTHON` → `WORKER`). This finding's premise may retire with that
change.

## Retired identities

- <a id="da-041"></a>**DA-041** — *Edited vendored trees have no truthful patch record.*
  - Resolved: `vendor/deltas/*.patch` and `vendor/deltas/readme.md` give a reconstructable
    record (`4867afe89c0aceb50e984636eaf11f7fdd10f0df`, `dede74bc2`).
  - The SurrealDB SDK is de-vendored at exact `=3.3.4`.
  - Full text at source `682fc3830`.
