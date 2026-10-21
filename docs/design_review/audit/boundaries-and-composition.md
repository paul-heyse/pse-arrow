# Boundaries and composition

Findings whose structural cause is a responsibility placed in the wrong owner, an
orchestration that re-implements composition per instance, or a library or host boundary
that carries decisions it should not. Baseline and limits are in the
[collection README](README.md). Citations are `path:line` at source commit `a791c533e`.

## <a id="da-012"></a>DA-012 — The run lifecycle supervisor is copied per operation kind, and the copies have diverged

**Principles:** AP-03, AP-01, DP-19, DP-20, DP-16; G9, G5.

**Diagnosis.** No single owner, parameterised by the operation, holds the public run
lifecycle. That lifecycle is: mint the run, open the durable attempt, register, dispatch,
wait for admission, join, then publish as refused or finished.

- **Four inline copies.** Simulation (`crates/pse-runtime/src/workflow/run.rs:949-1171`),
  shooting (`:1209-1388`), fit (`:1444-1628`) and modeling sequences (`:1803-1945`) each
  write the sequence out, differing mainly in the native closure body.
- **Horizon has its own version** (`workflow/horizon/driver.rs:87-271`).
- **Explicit strategies form a second family.** Conic, recycle and block initialization
  return through the math-layer `SolveHandle` (`strategies.rs:129-132`;
  `strategies/conditional.rs:60-80`). They have no `RunResult`, durable attempt, completion
  record or replay. Comments call this deliberate, but §17 and §19 do not state the
  exception.

**The copies already behave differently.** The horizon driver checks the caller's
`CancelSource` only during admission. It then shadows it with a fresh
`crate::CancelSource::new()` (`horizon/driver.rs:142`) that nothing links to the caller
(coordinator-confirmed). The other four forward the caller's cancellation.

Dropping the `RunHandle` would still cancel the run (`Lease::drop`, `run.rs:52-67`). But a
local study never drops it. Lanes `start(...).await?.wait().await`
(`study_execution.rs:1366-1370`), and on parent cancellation the study cancels its work
source and keeps draining the futures it has issued (`:802-821`). This was verified in the
challenge review. So cancelling a local study does not stop a running horizon point
(`study_operations.rs:1143-1166`). Horizon replay (`replay.rs:798-810`) has the same shape;
its caller was not traced. This contradicts §19.2 ("caller cancellation stays connected
through drain") and §13.6.

Dispatch over declared procedures is also re-implemented per consumer, each with a
different subset:
- `execute_declared` handles Solve and Initialize (`modeling/declared.rs:502-545`).
- Conformance handles all five procedures (`modeling/conformance.rs:1386-1600`).
- Studies handle Solve only, plus separately named operations (`bindings.rs:219-229`).

**Consequence.**
- A lifecycle change (progress, cancellation, clock, a durable hook) must be repeated in
  five copies.
- A new analysis kind means copying a supervisor.

**Counterevidence.**
- Completion (`run.rs:362-387`), durable registration and the
  `admit` / `submission` / `supervised_with_cancel` helpers are shared; only the skeleton
  is copied.
- Exhaustive matches make the per-kind sites findable.
- No test cancels a horizon's caller after start: `horizon_tests.rs:2405` covers only
  `handle.cancel()`.

**Evidence strength.** *Implemented*. The cancellation divergence is established from
source and is not *Tested*.

**Related.** [DA-009](structure-and-invariants.md#da-009).

## <a id="da-013"></a>DA-013 — Predicted- and continuation-start policy is implemented twice, once for local and once for durable studies

**Principles:** DP-01, AP-03, PS-08; G1.

**Diagnosis.** The scientific policy for predicted starts lives in the runtime twice,
outside the pure occurrence policy in `pse-model::study_policy`. Both copies contain:
- the eligibility rule (`recovery ∋ Predicted` and start policy ≠ Explicit);
- the proposal flow;
- a silent fallback, under which an `Unsupported`, `Incompatible` or `Numerical` boundary
  class means "no prediction".

The two copies order the steps differently. Local studies try prediction first and fall
back to the seed. Durable studies apply the seed and then overlay the prediction.

**Evidence.**
- Local: `crates/pse-runtime/src/workflow/study_execution.rs:1667-1776` (fallback at `:1739-1748`).
- Durable: `study.rs:1109-1118` (eligibility) and `:1266-1412` (fallback at `:1401-1408`).

**Consequence.** Changing eligibility or fallback in one place makes local and durable
studies of the same `StudyDefinition` choose different starts.

**Counterevidence.**
- The two executors differ legitimately in mechanics: only local studies batch through
  `Staged::batch`.
- The occurrence policy itself is pure and unit-tested.

**Evidence strength.** *Implemented*.

## <a id="da-014"></a>DA-014 — Study points bind values only for steady Solve cases; dynamic and fit sweeps have no value-only path

**Principles:** PS-11, AP-03 (PSE-S03), G7 (declared-design claim).

**Diagnosis.** Study overlays bind per-point values only for `DeclaredCase` operations with
the `Solve` procedure. That path is well served by a lane-held view plus value rebind.
Every other kind refuses values or cannot be studied:
- Simulation, Fit and Horizon operations reject any non-empty assignment with
  `StudyOperationUnsupported` (`crates/pse-runtime/src/workflow/bindings.rs:146-154`,
  `:254-265`; `study_operations.rs:576-590`).
- Initialize, Shooting and Check procedures are not study operations at all.

A dynamic or fit sweep therefore needs structural source variants, a new revision per
point. The value-only primitive does exist: `IntegratedExperiment::bind` binds parameter
values without re-preparing (`integrated.rs:63-71`). Neither studies nor the public
simulation API (Rust or `crates/pse-py/src/workflow/modeling.rs:466-490`) expose it.

§13.5 promises "`PreparedSimulation::rebind` prepares new immutable case bindings", but no
such type exists. The §25 supported-scope table does not record the limit.

**Consequence.** The functional target's "studies over many cases" and "dynamics from the
same model definitions" combine only through source variants. Whether those reuse
preparation depends on caches this audit did not examine.

**Counterevidence.**
- The steady path rebinds values on a lane-held view, with preparation counters and a
  scaling test (`modeling/views.rs:104-125`; `study_execution.rs:3421-3478`).
- The refusal is typed and happens before any effects.

**Evidence strength.** *Implemented* for the refusals and the absent API. Repeated
preparation in dynamic or fit sweeps is unresolved.

## <a id="da-015"></a>DA-015 — Host boundaries re-compose runtime-owned operations, choose defaults and bypass the runtime's composition and reuse

**Principles:** AP-01, AP-02, DP-14, DP-17, DP-10 (H12), AP-06; G9.

**Diagnosis.** `pse-py` describes itself as a "Mechanical public workflow projection"
(`crates/pse-py/src/workflow.rs:3`), and §1.1, §3.2 and §21 declare it a convenience layer
over the runtime. In practice it is where several runtime operations are assembled.

- **Operation chains.** Thirteen `ModelingPackage` methods sequence
  `declared_execution(...)` with their operation in the binding (`workflow/modeling.rs`).
  Eleven of them hard-code the default compiler `Profile` and `NumericalInputs` (for example
  `:355-357`, `:581-583`, `:1008-1010`; coordinator-confirmed). Only `prepare_solve`,
  `solve_bound`, `solve_case` and `conform` accept `PreparationSettings`. A case solvable
  only with raised allowances (for example assembly contributions,
  `crates/pse-math/src/assembly.rs:59-63`) therefore cannot be diagnosed, initialized,
  inspected or flow-prepared through Python under those allowances. The binding also adds
  small policies of its own:
  - a deadline rule (`modeling.rs:404`);
  - capability refusals based on cargo features;
  - a recycle route hard-coded as `Route::Native(Backend::Kinsol)` rather than read from the
    prepared strategy (`workflow/strategies.rs:71-78`);
  - a synthesized final progress event (`:456-463`).
- **Document admission.** `pse-py` composes document admission itself
  (`pse_admission::…::load_package_documents_owned` → `OwnedDocumentSet::try_from_bundles` →
  runtime; `workflow.rs:977-1012`; `workflow/modeling.rs:59-104`). The runtime has an
  equivalent private composition (`physical_from_sources`, `package_from_sources`;
  `worker.rs:2382`, `:2406`). The two routes are not like for like:
  - The Python route is ingress-and-publish (`crates/pse-runtime/src/workflow/modeling.rs:412-455`),
    where re-validating new external input is a legitimate trust transition (DP-03, H18).
  - The worker route reloads persisted sources through the package-admission cache
    (`worker.rs:2444`).

  The defect here is that host-side code composes admission at all. That repeated
  ingress of identical bytes misses a reuse opportunity is a hypothesis.
- **Composition root and deployment.** `pse-py` builds the process composition root
  (`inspection/runtime.rs:23-70`). It calls `pse-operations` directly to initialize, connect
  and qualify deployment (`workflow.rs:692-760`, `:859-883`). `pse-worker` re-implements
  the same deployment sequence (`xtask/src/bin/support/deployment.rs:30-110`).
- **Invariants enforced only in the adapter.** `ModelingLimits` refuses zero limits in the
  binding (`workflow/modeling.rs:152-158`). The same limits admitted through
  `PreparationSettings.limits` are generated with `ge=0`, and `pse_modeling::Limits` has no
  validated constructor (`crates/pse-modeling/src/specialize.rs:57-82`). Likewise,
  `pse-worker` alone enforces "heartbeat shorter than lease"
  (`xtask/src/bin/pse-worker.rs:404-411`) and restates the `LeasePolicy` defaults, while the
  runtime checks only that values are non-zero
  (`crates/pse-runtime/src/workflow/durable.rs:1428`).
- **Untestable placement.** All this logic sits in a crate declared `test = false`
  (`crates/pse-py/Cargo.toml:27-28`), so it can be exercised only through Python against a
  built extension.

**Consequence.**
- Behaviour differs by host and by operation: different allowances per operation, and
  invariants enforced only at one host.
- A third host would need a third copy of runtime assembly.
- Python-facing route reports can drift from the runtime's selection.
- Rust callers re-compose the same chains (`crates/pse-runtime/src/workflow/diagnostic_request.rs:188-208`).

**Counterevidence.**
- Each underlying operation is a real runtime method; the binding sequences them and does
  not reimplement them.
- The defaults are the owners' own `Default` values.
- Per-executable composition roots are normal, and ADR-0164 deliberately has each host make
  its own deployment assertion.
- Type-exposing edges to backend-native, engine and modeling are permitted by §1.1. The
  admission and operations edges are the ones that carry behaviour.

**Evidence strength.** *Implemented*. The allowance divergence is source-established and
its end-to-end effect is not *Tested*. The reuse difference is a hypothesis.

**Related.** [DA-021](local-reasoning-and-assurance.md#da-021), [DA-019](#da-019).

## <a id="da-016"></a>DA-016 — Solver identity and composition knowledge lives in runtime orchestration outside the adapter seam

**Principles:** AP-02, AP-03, PS-09, DP-15, DP-01, AP-05; PSE-S02.

**Diagnosis.** The backend-execution seam claims that adding a backend takes "one adapter,
one registry value and its capability record, with no workflow edits"
(`crates/pse-backend-native/src/execution.rs:3-9`). §18.7 says neither runner nor adapter
selection names a backend. That holds for algebraic routing, which names no backend in
production. It does not hold for several compositions and recoveries.

- **Declared fixed-point and causal maps have no representation in the seam.**
  `NativeRepresentation` has no fixed-point variant, and `routing::problem_classes` never
  produces `DeclaredFixedPoint` (`routing.rs:762-797`), although KINSOL's record lists it
  (`execution/kinsol.rs:25-26`). Runtime code therefore constructs `kinsol::Session`
  directly and calls `quality::qualify` itself (`crates/pse-runtime/src/math/initialization.rs:1112-1271`,
  coordinator-confirmed at `:1271`; `workflow/strategies/conditional.rs:912`, `:953`,
  `:1554-1570`). This repeats steps `execution::roots` owns (`execution/runner.rs:305-358`).
- **POUNCE's second-opinion recovery ladder is wired in by name in two runtime owners,**
  each with its own settings extraction (`math/solves.rs:712`, `:1127-1150`, `:1278-1290`;
  `math/strategy/target.rs:97-115`, `:398-430`). POUNCE Schur separators are gated the same
  way (`solves.rs:5269`).
- **Duplicated demand fact.** The runtime re-derives KINSOL's JVP demand
  (`solves.rs:3893-3894`), which the adapter already owns (`execution/kinsol.rs:212-229`).
- **The arclength corrector is pinned to Ipopt with limited-memory Hessians**
  (`math/solves/paths.rs:977`, `:988`, `:1684`, `:1890`; coordinator-confirmed). Because the
  suitability is not a capability, POUNCE and Uno (same `Nlp` representation) cannot take
  part.
- **Fit LM route.** The fitting residual LM route is selected before backend routing by a
  hand-written conjunction over `SolverProfile` fields
  (`workflow/fitting/oracle/residual.rs:9-35`, coordinator-confirmed; `:37-57`). It lists
  fields by name instead of destructuring the struct, so a new profile field is not forced
  into the predicate, and `convexity` and `numerics` are not examined.
- **PETSc.** NASM and pseudo-time are refused by the adapter (`execution/petsc.rs:74-76`)
  and run only as a runtime strategy rung (`math/solves/petsc.rs`;
  `workflow/modeling/petsc_pipeline.rs`).
- **Python boundary.** `pse-py` re-asserts KINSOL for recycles
  ([DA-015](#da-015)).

**Consequence.**
- Serving declared recycles with another library, or giving another NLP backend a recovery
  ladder or the arclength role, means editing runtime initialization, conditional
  strategies, solves, path control and the binding, not adding an adapter and record.
- The interpretations held in two places (the POUNCE ladder; KINSOL JVP demand) can drift.
- An obligation added to the roots runner would not reach the direct KINSOL paths.

**Footprint.** About 50 production matches on concrete `Backend::` variants across 20
runtime files. A few of those files are test fixtures. Diagnostics routes are suspected to
follow the same pattern.

**Counterevidence (material).**
- ADR-0209 deliberately makes LM a fitting-owned method rather than a backend. Selection is
  explicit and recorded, with refusal before effects.
- PS-09 permits bounded domain composition of admitted operations.
- §0.5 itself assigns fixed-point maps to KINSOL.
- Every named restriction is a typed refusal before execution, so outcomes stay truthful;
  the defect is locality and authority, not silent fallback.
- Some specificity is inherent: only POUNCE exposes a library second-opinion ladder.

**Evidence strength.** *Implemented*.

**Related.** [DA-017](#da-017).

## <a id="da-017"></a>DA-017 — The published capability inventory and the declared solver matrix do not match what is admitted and reachable

**Principles:** G7, DP-15, DP-22, DP-24, PS-09.

**Diagnosis.** Capability is one static record per backend
(`crates/pse-backend-native/src/execution.rs:380-402`, published through
`Table::published`). A static record cannot express capabilities that depend on method or
profile, and nothing reconciles the published or declared views with what actually runs.

- **Uno.** The record advertises `ExactHessianOrLimitedMemory`
  (`execution/uno.rs:20`), but `admit_settings` refuses any Hessian mode except
  `LimitedMemory` (`execution/uno.rs:73-81`; coordinator-confirmed). The registry vocabulary
  has no limited-memory-only value.
- **KINSOL.** The published row lists `declared_fixed_point` as automatic, but routing
  never derives that class ([DA-016](#da-016)). `published_capabilities_equal_routing_rules`
  (`execution/tests.rs:512`) cannot catch a class that routing never produces.
- **PETSc.** The Python contract offers `nonlinear_additive_schwarz` and `pseudo_transient`
  (`python/pse/contracts/enums.py:1791-1796`). The adapter refuses both, and their only
  route is a Rust API used by tests. No Python route was found in `python/pse` or
  `crates/pse-py`.
- **Architecture text.** §0.5's class → library table, §1.3, §3.3 and the §18.9 matrix
  omit Uno and PETSc. Yet `native-solvers` links both, `runtime.solver_capabilities`
  publishes their rows, and §3.2 and §1.1 omit `pse-uno-sys` and `pse-petsc-sys`. ADR-0157,
  which introduces them, is `proposed` with `Proposed` evidence.

**Consequence.** Python authors, diagnostics and agents that read the inventory or the
architecture are told about capabilities without routes, and linked adapters have no
declared contract.

**Counterevidence.**
- Refusals are typed and happen before work, so G7's silent-fallback clause holds.
- The record-driven inventory with its equivalence test is a sound mechanism; the gap is
  what it can express.

**Evidence strength.** *Implemented*.

## <a id="da-018"></a>DA-018 — The Uno integration carries project numerics in a patched copy of the library, with a dense, admission-capped SQP route

**Principles:** DP-13, DP-15, AP-07, PS-09; PSE-S06, PSE-S07.

**Diagnosis.** `docker/solvers/uno-pse.patch` (374 lines; declared at `Cargo.toml:519-531`)
modifies Uno's HiGHS subproblem solver and its no-relaxation strategy. It adds:
- thread-local scope and accounting state;
- tolerance plumbing, and removal of HiGHS QP regularization;
- a feasibility-LP seed before every QP;
- dense materialization of the L-BFGS Hessian through n Hessian-vector products per new
  iterate.

ADR-0157 calls this a "scoped binding". It is in fact a fork of library internals that
carries semantics, and the bridge depends on it (`crates/pse-uno-sys/bridge/pse_uno.cpp:19`).

Memory is quadratic in n. The patch's allowance projection is `count*64 + n*16` with
`count = n(n+1)/2`. Under the default 64 MiB foreign allowance
(`crates/pse-runtime/src/math.rs:100`), that refuses SQP above roughly 1,450 variables. This
is source arithmetic, not a measurement. Neither the record nor the architecture declares
a size scope. The status mapping uses bare integers rather than named binding constants
(`crates/pse-backend-native/src/uno.rs:114-131`).

**Consequence.**
- Each Uno upgrade must rebase a patch that changes behaviour.
- Upstream tests do not qualify the patched solver.
- Whether Uno is supported for flowsheet-size NLPs is an open AP-07 question.

**Counterevidence.**
- Uno is explicit-only (`automatic() == None`).
- Refusals are typed.
- ADR-0157 states its reasons (HiGHS QP needs an explicit Hessian; avoiding a 1e-4
  truncation), so G8 does not fail outright.
- The patch digest is part of the native build identity.

**Evidence strength.** *Implemented* for the structure and the admission bound. The status
literals have not been checked against Uno's C API.

## <a id="da-019"></a>DA-019 — Python operator and developer tooling restates Rust-owned contracts, and one script is a second writer of canonical state

**Principles:** DP-01, AP-02, AP-06, DP-24, DP-14; G1.

**Diagnosis.** Several Python tools re-express contracts that a Rust owner or the registry
already declares, without consuming the generated contracts:

- **Canonical store writer.** `scripts/surreal_server.py` (8,874 lines) builds SurrealQL
  with f-strings that read and mutate canonical tables directly, for example during
  authority rotation, storage rehoming, admission freezing and protection checks
  (`:5905`, `:6565-6600`, `:7266-7300`, `:7924-7929`). It imports no `pse.contracts`; the
  generated `python/pse/contracts/runtime.py:373` already models `gc_frozen`. Transition
  rules thus exist in two languages. A substrate change passes regeneration and Rust
  compilation, then fails only in rarely exercised restore and rotation paths.
- **Conformance CLI.** `python/pse/conformance.py` declares an unversioned manifest schema
  for `packages/reference/conformance.toml` (`:49-108`). It restates the derivative-check
  defaults twice (`:71-83`, `:378-383`). The same defaults are owned by
  `ConformanceControls::default()` (`crates/pse-runtime/src/workflow/modeling/documents.rs:431-443`)
  and by the generated document defaults. It also hand-spells subsets of `NativeSolveIntent`
  and `PresolvePolicyKind`. Its package-document suffix filter (`:172-181`) is repeated in
  the parity support, tests and `scripts/thermodynamic_campaign.py:32`. It has a store-free
  `pure` mode (`:247-261`); only the full mode opens a durable store (`:263`).
- **Diagnostics launch tooling.** `python/pse/_diagnostic_launch.py:141-335` and
  `_diagnostic_query.py:29-94` implement process supervision, outcome classification and
  acknowledgement attribution. This runs beside the Rust `capture::launch` owner
  (`crates/pse-runtime/src/capture/launch.rs:54-58`) and restates its environment protocol
  (`_diagnostic_launch.py:221` versus `capture/recording.rs:19`). The tooling builds
  versioned Rust documents with hard-coded versions and writes review outputs without the
  Rust export completion rule (`python/pse/diagnostics.py:907-945`, `:1010-1030`).
  `scripts/case_audit.py:593-700` reimplements the same protocol.

**Consequence.** A change in the Rust owner (default, vocabulary, field, protocol) is not
seen by the Python copy. The divergence surfaces late, in operator or developer paths.

**Counterevidence.**
- The supervisor paths are offline and admission-closed, and they assert gate premises
  in-transaction.
- The diagnostics tooling never cancels science and records its own limits.

**Evidence strength.** *Implemented* / *Interface-checked*, from source reading.

## <a id="da-020"></a>DA-020 — A performance-investigation toolkit lives in the production orchestration root and the foundation diagnostics crate

**Principles:** AP-01, DP-17, DP-21 ("without letting instrumentation dominate cost"), H26, H27.

**Diagnosis.**
- **In the runtime.** `pse-runtime` exports capture, frozen inspection, inspection
  comparison, invocation, review analysis, review launch, review pack and review query
  modules (`crates/pse-runtime/src/lib.rs:37-55`; coordinator-confirmed). Together they are
  about 23,000 lines, including inline tests (challenge-review count). They include launching `prlimit` and `/bin/sh` to profile
  (`capture/cpu.rs:214`; `capture/launch.rs:367`) and producing offline review packs.
- **In the foundation crate.** `pse-diagnostics`, declared as the foundation-layer
  "diagnostic vocabulary" (§1.1), hosts a bespoke observation hub of about 6,300 lines
  (`observation.rs`, `producer.rs`, `inspection/performance.rs`) beside `tracing`.
- **Coupled to scientific readers.** Scientific result readers construct review-pack
  selection types (`workflow/connected_results.rs:243-249`, `:517-553`;
  `crates/pse-py/src/documents.rs:138-159`).

**Consequence.**
- Changes to developer tooling rebuild the orchestration root and the Python extension, and
  widen what the runtime ships.
- A foundation crate owns a telemetry runtime used widely across product crates (on the
  order of a thousand `pse_diagnostics::observation` mentions).

**Counterevidence.**
- §23.1 states a reason for keeping frozen inspection and review analysis close to the live
  owners: they reuse "the live owners' pure interpreter" through "one Rust interpreter"
  (`docs/authoritative_design/sections/operations-and-validation.md:35-38`, `:50`). The
  defensible diagnosis is therefore narrower: a separate crate could keep that sharing
  without placing the toolkit in the orchestration root or the foundation layer.
- Capture and packs are opt-in and versioned.
- `tracing` is adopted.
- Observation is documented as never changing outcomes.
- The ordinary-run cost has not been established.

**Evidence strength.** *Interface-checked* for placement and coupling. Whether separation
preserves the shared interpreter at acceptable cost is unresolved. Cost is a hypothesis.
