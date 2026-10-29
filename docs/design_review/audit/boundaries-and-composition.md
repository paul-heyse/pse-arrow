# Boundaries and composition

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.** Dirty
paths that bear on this page:
- `pse-host` (committed at `f3a05e0be`);
- the Python generator deletion in `pse-codegen`/`xtask`;
- `pse-runtime/src/host_operations/project.rs`, where DA-015's PyO3 method projection is removed.

The [README](README.md#baseline-and-method) lists the full set. Other decisive files were clean.

This page holds the primary causes in which composition happens by convention, or placement
follows reach rather than responsibility:
- product/tooling/host placement ([DA-020](#da-020));
- uniform projection emission ([DA-059](#da-059));
- dormant data-layer machinery ([DA-060](#da-060));
- per-mode composition of native execution ([DA-053](#da-053));
- two study executors ([DA-054](#da-054));
- documents outside the typed regime ([DA-052](#da-052)).

The [integrated account](system-architecture.md) relates them.

## <a id="da-053"></a>DA-053 — Analysis modes compose native admission, routing, execution control and lane custody themselves

**Principles:** AP-01, AP-02, AP-03, AP-05, AP-06, DP-16, DP-17, DP-20, PS-11; H21, H25.

**Status:** new on 2026-10-29 (U2). It is the numerical-execution form of the composition cause.

**Claim at risk.**
- Adding a backend or routing input is "one adapter, one registry value and its capability
  record, with no workflow edits" (`pse-backend-native/src/execution.rs:3-9`).
- §18.8 names `MathService` the single execution owner.
- One supervisor retains cancellation, deadlines, native destruction/join and charges through
  drain.
- PS-11: analysis modes are declared over one model and its shared preparation.

**Structural cause.** The eligibility decision is centralized (`Requirements::decision` →
`eligibility`, `routing.rs:1028-1041`). The composition of a request has no single owner, not even
inside `MathService`. Each composing site hand-builds the same sequence:
1. structural extent reservation;
2. structural assessment;
3. `oracle_facts`;
4. a full `routing::Requirements { … }` literal;
5. `decision` / `route`;
6. a witness lease;
7. `admit_profile`.

There are twelve production literals of this kind:
- **Seven inside `MathService` entry points:**
  - `math/solves.rs:3364, 3987, 4179`;
  - `math/solves/paths.rs:1130, 1864`;
  - `math/solves/derived.rs:1665`;
  - `math/initialization.rs:316`.
- **Five in workflow modes that bypass it:**
  - `workflow/shooting.rs:747-797`;
  - `workflow/fitting/modeling.rs:393-470`;
  - `workflow/modeling/cases.rs:169-190`;
  - `workflow/diagnostics.rs:555`;
  - the second initialization site, `math/initialization.rs:612-700`.

Modes then build the native `Execution` (cancellation, progress, work admission, foreign memory
allowance) themselves:
- `workflow/shooting.rs:1726-1740`;
- `workflow/fitting/oracle.rs:1182-1190`;
- sites in `math/solves.rs`, `math/solves/paths.rs` and `math/initialization.rs`.

About 25 non-test workflow/tooling files import backend-seam internals (`native::execution::*`:
`Step`, `Retained`, `LINKED`, `adapter()`).

**Execution control is ambient and fails open.**
- Part of one attempt's control plane is carried in thread-locals that callers must install in
  the correct nesting:
  - `ORIGINAL` and `COMPUTE` in `pse-backend-native/src/execution.rs`;
  - `construction::OWNER` in `pse-math`;
  - `observation::current()`.
- When no owner is installed, the primitives proceed rather than refuse:
  - `resume_compute` returns `Ok(())` (`execution.rs:134-143`);
  - `try_resume_compute` returns `Acquired` (`:146-157`);
  - `construction::additional` skips accounting, relying on "runtime jobs always install their
    common-pool owner" (`pse-math/src/construction.rs:44-48`).
- Thread custody is re-implemented per mode. Each of the following builds its own `std::thread`
  lanes, channels, stack sizing, `catch_unwind` and `original_scoped(construction::scoped(…))`
  nesting, beside `math/staged.rs`, `math/jobs.rs`, `joined_pool.rs` and `native_child.rs`:
  - shooting window teams (`workflow/shooting/window_team.rs:290-350`);
  - fitting experiment lanes (`fitting/experiments.rs:180-245`);
  - profile chains (`fitting/profile.rs:270-285`).

**Non-algebraic selection is owner-decided, not part of this finding.**
- Dynamics resolves `Method::Auto` through its own candidate list (`dynamics.rs:2228-2300`), and
  the Diffsol/IDAS table entries refuse direct execution. §18.1 assigns dynamic representations to
  §13.
- Fitting's `FitOptimizer::{Automatic, NativeNlp, ResidualLm}` is explicitly decided by proposed
  ADR-0209 ("Native solver selection remains the policy of the NLP alternative").
- Three `Auto` policies with distinct refusal vocabularies are therefore a deliberate design. A
  later review may challenge that against PS-09, but it is not established here as a gap.

**Consequence.**
- A new routing input, budget or representation demand is a coordinated edit across twelve
  production sites.
- Modes hard-code their representation (`prepared: &[Representation(Nlp)]`).
- Admission arithmetic differs subtly between copies: shooting shrinks by `retained_bytes()`,
  while fitting subtracts the witness first.
- Correct cancellation, deadline and memory accounting depends on caller convention, and a lane
  that omits a scope runs silently with direct-library limits.

**Counterevidence and legitimacy.**
- The decision itself is not duplicated.
- The pure `strategy::next_automatic` and `strategy::target::callable` driver is shared by
  fitting and shooting.
- §18.1 assigns dynamic representations to §13.
- Native C callbacks cannot carry context arguments, so thread-locals are a standard route
  through them.
- pse-math must stay usable without a runtime.
- IBEX, SCIP, HiGHS and Uno have process-global state that genuinely needs exclusion.
- ADR-0205 deliberately lets fitting and fixed-window owners prepay their resident populations,
  with "no new … native lane pool".
- Capacity is still coordinated through one pool, so this is not an H21 uncoordinated-pool
  failure.

The fail-open branches may be reached only by direct, test or pure-library callers; no production
path running without an owner was established. Many §18.8 ADRs remain proposed (0154, 0158, 0176,
0185, 0188, 0205, 0215).

## <a id="da-054"></a>DA-054 — Local and durable studies orchestrate the same occurrence semantics in two executors that share only policy predicates

**Principles:** AP-03, DP-01, DP-10, PS-11.

**Status:** new on 2026-10-29 (U3). DA-013 resolved the shared prediction *policy*. This finding
concerns the orchestration around it.

**Claim at risk.**
- ADR-0190: local studies run "without a second admission policy, solver path or storage
  manager".
- §19.3: one study policy is shared across placements.
- PS-11: one model serves every analysis mode.

**Structural cause.**
- `workflow/study_execution.rs` has its own scheduler: frontier, in-flight set, dependents, retry
  and staged-lane retention (`:554-640`).
- The durable path splits the same work across progression, claim and `DurableAttempt`.
- The prediction/continuation flow (entry → proposal → `screen_start` → fallback to seed) is
  written twice (`study_execution.rs:1682-1780`; `study.rs:1335-1360, 1500-1650`).
- Native staged state across dependent points is used by the local executor, `run.rs` and the
  horizon, but not by `worker.rs`, `study.rs` or `study_operations.rs`. Durable lanes retain
  preparation and sources only (`study.rs:2995-3020`).

**Consequence.**
- Retry, seed and prediction changes need two edits.
- Durable studies forgo the native-state reuse that PS-11 values.

**Counterevidence.**
- Database-free local studies are a maintainer premise.
- 58d forbids a new workflow engine.
- §20.6 deliberately keeps native factors process-local, which limits cross-process staged reuse.

Confidence that this is one shared cause, rather than two legitimately distinct schedulers, is
medium.

## <a id="da-052"></a>DA-052 — Durable and service documents outside the exported boundary set re-implement versioning and field interpretation by hand

**Principles:** DP-24, DP-02, DP-01, AP-02, DP-13, DP-14.

**Status:** new on 2026-10-29. U6 and U3 reported it independently.

**Claim at risk.** ADR-0116 and §21.5: Rust-owned documents are typed and versioned, and unknown
versions are never reinterpreted. §0.6: one owner per meaning.

**Structural cause.** `for_each_boundary_document` with `Versioned<T,N>` governs exported
documents only (`pse-runtime/src/boundary_documents.rs:11-60`). Documents that are persisted, or
exchanged between crates and processes, use other means:
- **`StoredCompletion` and `CompletionReceipt`** (durable, inline in canonical receipts) carry
  `pub version: u8` (`workflow/durable.rs:879-881`).
  - The literal `3` is written at about 13 construction sites in `durable.rs` and
    `connected_results.rs`, and a hand check `!= 3` sits at `durable.rs:896`.
- **The canonical service `config.json`** carries write admission, module selection, migration
  predecessor/target, payload incarnation and receiver placement.
  - It is written by the Python supervisor for the operator store and by `native_service.rs` for
    fixtures.
  - At least five Rust readers each declare their own struct or untyped value, bypassing the typed
    `deployment::projection` (`deployment.rs:1-60`):
    - `WriteAdmission`, with `surrealkit_rollout_pending` and `derived_rebuild_pending`
      (`canonical.rs:546-645`);
    - the module-selection structs (`canonical_study_module.rs:181-235`);
    - payload placement (`canonical_files.rs:418-432`);
    - the maintenance reader (`canonical_maintenance.rs:248-260`);
    - `RetainedRecords` (`native_service.rs:229-255`), which reads the service's own typed
      `Configuration` back as `serde_json::Value`, patches keys by string and takes SHA-256 of the
      re-serialized value. `serde_json`'s `preserve_order` is active through workspace feature
      unification, so that digest depends on key order.
  - Every guarded write first pays a synchronous `config.json` read for a gate that is
    deliberately not authoritative (`canonical.rs:562`).
  - A task-local `SETTLEMENT` changes write-admission semantics ambiently
    (`canonical_migration.rs:34-60`).
- **Study-module declarations** are hand-walked as Cargo-metadata JSON and TOML by string index in
  `pse-host/src/study_module.rs:313-320, 688-735`. The same crate uses the typed `cargo_metadata`
  crate in `native.rs:488-509`, and the retiring `scripts/surrealism_module.py` interprets the
  same declaration independently.

**Consequence.**
- A version bump or field change needs coordinated hand edits that neither compiler nor generator
  catches.
- A rename in the Python writer silently diverges from the Rust readers ([DA-057](development-tooling.md#da-057)).
- Service-record digests are representation identities rather than value identities, unlike the
  key-order-independent `pse_ids::document::of`.

**Counterevidence.**
- `pse-host` deliberately avoids `pse-model` and generated schemas (ADR-0233; §21.7), so raw
  versions there are a chosen constraint.
- `StoredCompletion` does carry and check a version; the defect is the lack of a single owner
  for it. Some of the literal sites sit in inline test modules.
- The authoritative write fence is in-transaction (`schema_v1::admit`), and the file gate is
  explicitly a local opportunity.
- The Python writer is a documented 58b/58d transition.

## <a id="da-020"></a>DA-020 — Placement follows build reach rather than responsibility, so product crates carry host and tooling, and foundations carry unrelated reasons for change

**Principles:** AP-01, AP-02, DP-17, DP-16; AP-07 via H13 and H27.

**Status:** retained and relocated on 2026-10-29.
- The old manifestations (evidence and knowledge stores inside product crates, a defeated feature
  gate) are resolved (`dcf90bf18`).
- The cause, placement chosen by what a crate can reach, has moved to the new host root and to
  the lowest common crates. Three unanchored streams (U4, U5, R1) reported it.

**Claim at risk.**
- ADR-0233: a lean `pse-host`, and "retain heavy development operations under their actual
  owners, not inside the host crate".
- Revision 311 (`2232bfa02`) adds to §21.7 that "heavy native developer composition stays under
  xtask, outside the independent host".
- 58f: keep CLI and agent libraries out of ordinary product closures.
- §1.1: foundations and declarations are library-neutral semantic bases.
- §3.2: "semantic and numerical roots remain independent of `pse-operations`".

**Structural cause.**
- **One host crate.** `pse-host` is one library with every module public
  (`crates/pse-host/src/lib.rs:4-16`) and one command bag (`main.rs:55-127`). It holds:
  - B host control: manager, foreground, environment, admission, activity;
  - F developer tools: `agent_hooks`, `doctor`, `cargo-tools`, `diagnostic_tools` (a pinned
    downloader/installer), `telemetry_service` (a persistent Collector/OpenObserve operator) and
    `correctness` (the Cargo/nextest adapter);
  - artifact construction, about 7.6k (roughly 40%) of its about 19k lines including tests:
    `server` and `release_import` build SurrealDB from source or import releases;
    `study_module`; `native` and `providers` build solver images.
- **What decides placement is "must build cold without xtask's closure", not responsibility.**
  `pse-runtime` depends on all of `pse-host` unconditionally (`crates/pse-runtime/Cargo.toml:69`).
  It uses it only for `NativeStudyWorker`'s `HostManager`/`OwnedUnit`/`ServiceSpec`
  (`workflow/native_worker.rs:7-10`). In the normal workspace graph `pse-host`'s dependents are
  exactly `pse-runtime` → `pse-py` (`cargo tree -i pse-host -e normal --workspace`).
- **Product crates own tooling and test seams.**
  - `pse-operations` reaches `pse-host` only through its `native-service` feature, which the
    canonical service and test binaries and `pse-runtime`'s `native-study-tests` enable.
  - `pse-operations::native_fixture` hard-codes the checkout layout
    `target/host-tools/current/cargo-pse-correctness` (`native_fixture.rs:135`).
  - `pse-runtime::tooling` is unconditional (`lib.rs:60-61`) and mixes product observability with
    review-pack/capture tooling.
  - `tooling/review_query.rs:190-214` `include_str!`s `scripts/diagnostic_sql`.
  - `pse-operations`' `test-support` feature changes shutdown drain behavior
    (`native_service.rs:1328-1335`) and exports `testing`/`native_fixture`.
  - The developer binary `pse-canonical-test` lives in the product crate.
- **Lowest-reachable placement in foundations.**
  - `pse-model` holds about 8.3k lines of generated canonical SurrealDB rows (90 `canonical_*`
    modules within its 27k-line `generated/runtime` family) and the hand-written study policy.
    These sit beneath the entire mathematics layer, and no mathematics crate consumes the
    canonical rows. `pse-backend-native` uses three non-canonical runtime rows.
  - `pse-diagnostics` (beneath everything) hosts `SupervisedChild`, procfs process facts,
    telemetry settings (HTTP/URL) and the observation registry. Only the host, the runtime and one
    `pse-math` site use the supervision and telemetry parts.

**Consequence.**
- An edit to the edit guard, doctor, collector configuration or Cargo adapter invalidates
  `pse-runtime`, `pse-py` and runtime dependents. It reaches `pse-operations` and `xtask` only in
  service/test or feature-enabled builds.
- The product library and cdylib compile the host's closure: zbus, `cargo_metadata`, and the
  tar/zip/flate2 added at `f3a05e0be`.
- A canonical storage-row or study-policy change regenerates `pse-model` and rebuilds the whole
  mathematics layer, which does not consume those rows.
- Product needs and cold-bootstrap leanness now constrain each other through one crate.

**Counterevidence.**
- ADR-0233's status history explicitly authorizes provider preparation and the Cargo adapter in
  the host. 58b assigns physical deployment construction to B.
- An edit guard and doctor must build without heavy crates.
- The host's normal closure contains no Arrow, DataFusion, PyO3 or database client.
- The edge direction product → host matches ADR-0233 ("application composition connects it to D").
- §3.2 explicitly places generated canonical rows and study policy in `pse-model`, and the WASM
  study module shares them, a real one-owner benefit.

The ADR's closure rule covers heavy crates entering the host, not product crates depending on the
whole host. Whether doctor, telemetry service and diagnostic tools count as "heavy developer
composition" under revision 311 is the open owner question. The owner intent is incomplete rather
than violated.

**Lineage.** Plan56 recorded DA-020 resolved (E01, Implemented/Tested) for its original
manifestations, and current source confirms that. This refresh reopens the identity against that
disposition because the cause recurs in new owners. Rebuild cost is unmeasured.

## <a id="da-059"></a>DA-059 — The declaration pipeline emits every projection for every relation family, and retiring consumers anchor part of that surface

**Principles:** AP-04, AP-07, DP-01, DP-16, DP-24; H2, H6, H19, H27.

**Status:** new on 2026-10-29 (U7).

**Claim at risk.**
- §4.1/§4.2 and D1: the registry declares each meaning once and generates what consumers need.
- §0.6: projections are derived "per consumer".
- AP-04: a separate layout must have an access benefit.

**Structural cause.**
- Canonical SurrealDB tables, the registry's self-description, scientific result relations and
  authored package relations are all declared as one Arrow-shaped `RelationSpec`.
- The generator loops over every relation (`crates/pse-codegen/src/codegen/rust/mod.rs:80-97`) and
  emits for each one:
  - a `pse-model` row;
  - a `pse-relations` Arrow module (value, row, view, builder, constants, schema, validator);
  - DataFusion invariant SQL;
  - `reference.schema_*` rows;
  - Markdown;
  - an entry in the hydration image.
- No declaration says which projections a family needs. Canonical tables get a placeholder
  `SnapshotClass::Sidecar` (`pse-schema/src/catalog/substrate.rs:1233-1244`), while their actual
  consumer, the Surreal codec, uses the `pse-model` rows directly.
- Of the 90 canonical Arrow modules in `pse-relations` (about 52k generated lines), a
  path-qualified consumer search finds only about seven referenced by production code, all for
  `pse-py` presentation.
- A broader stream search reported 71 of 200 relation modules with no production reference. It
  disagrees with the canonical figure above, so treat it as indicative only. Glob imports such as
  `use pse_relations::generated::runtime::*` (`workflow/results.rs`, `run_outcome.rs`) limit
  both counts.

**Retiring consumers anchor part of the surface:**
- `connected_results::{run_record, attempt_record, result_manifest}`
  (`workflow/connected_results.rs:1288-1318`), and the analysis and artifact headers, rebuild a
  single Surreal-admitted row through a validated Arrow builder, with DataFusion row checks, for
  `pse-py`'s `TableStream`.
- `BoundModel::registry_relation` (`pse-engine/src/session/factory.rs:401-440`) has one production
  consumer, `pse-py/src/inspection/registry.rs:21`.
- §4.2 and §21.5 still name generated Python contracts.
- 58g's retirement table does name the consumers: "stored-result and table-stream projections …
  E04/G03 remove them with actual delivery/review/report consumers". Its G03 package covers
  generator emissions. What no owner declares is which per-family projections the generator
  should emit once those consumers go.

The 497 published JSON Schemas are ADR-0116 boundary documents, not per-family projections, and
are outside this finding. Their in-repo readers are retiring Python contracts and
`scripts/telemetry_investigation.py`.

**Consequence.**
- Any declaration change, including a canonical-only field, regenerates and recompiles
  `pse-relations` (about 199k generated lines, depending on DataFusion's optimizer crates),
  `pse-model` and the hydration image, plus everything downstream.
- The registry's cold derivation covers all 200 relations. ADR-0210's roughly 25 MB hydration
  image then compensates for that, so uniform emission is paid for twice.
- Unless G03's emission change also declares per-family projection needs, removing `pse-py`
  leaves the canonical Arrow views and reflection batches generated without a consumer.
- Presenting one row re-establishes validity that Surreal already established (H19).

**Counterevidence.**
- ADR-0164 deliberately routes canonical tables through the one registry. One declaration
  expanding into projections is DP-06 extension locality.
- Generated code costs nothing to author.
- ADR-0210 is decided with Tested evidence.
- ADR-0116 retains schemars schemas for readers and optional clients.

The issue is the physical unit (every projection for every family), not a broken authority. Build
cost is unmeasured, and string-keyed consumers are bounded only by the searches run.

## <a id="da-060"></a>DA-060 — Data-layer mechanisms and declared invariants outlive their consumers

**Principles:** DP-16, DP-03, AP-02, AP-05, DP-24; H4, H20, H27.

**Status:** new on 2026-10-29 (U7). Both of these have the same origin: the retired rule
workspace, and the deleted catalog and Delta era, left machinery behind after its consumers moved
to admission-owner checks and SurrealDB.

**Declared invariants with no execution point.**
- The registry declares about 260 invariants, rendered as DataFusion SQL, including 105 canonical
  and 67 primary-key uniqueness invariants.
- `pse_rules::invariants::run_invariants` (`crates/pse-rules/src/invariants.rs:92`) has exactly
  one production caller: physical-fixture generation in `xtask/src/codegen/physical.rs:192`, under
  `InvariantScope::Model`. The other scopes are used only in tests.
- In production, key uniqueness is enforced instead by three other realizations, and nothing
  checks that they agree:
  - a hand Arrow row-converter check (`pse-relations/src/result_batch.rs:333-345`);
  - `pse_schema::obligations` templates (`pse-admission/src/physical/plans.rs:22`);
  - Surreal record keys and UNIQUE indexes.
- Canonical SQL invariants address Arrow tables that never exist at run time.
- No production reader of the `derivation_granularity` or `stability` facets was found.

**Engine capabilities with no production caller** (a search for entry points outside
`pse-engine`):
- catalog resolution (`session/resolution.rs`, about 526 lines). Its callers died with
  `pse-catalog` (`06302af77`), yet the codegen roots still list
  `crates/pse-catalog/src/generated` (`pse-codegen/src/codegen/mod.rs:91, 247`);
- finite-round providers (`session/round.rs`);
- DML mutation and commands behind `EngineSession::sql`;
- schema transforms (`session/schema_transform.rs`);
- `RecordedProvider`;
- `pse_schema::compatibility` and `declare_migration` (test-only callers);
- workspace and computation-plan entry points.

Together these are about 6k lines.

**The live providers bypass the engine.** The retained-data providers actually used,
`SnapshotProvider` (`workflow/result_snapshot.rs:1149`) and `CohortProvider`
(`cohort_provider.rs:615`), implement DataFusion `TableProvider` directly rather than through the
engine's provider/operation framework.

**Consequence.**
- The invariant catalogue appears to govern behavior: it appears in the docs, in
  `reference.schema_invariants` and in the hydration image. In practice it governs only fixture
  generation.
- Dormant engine code is compiled and carried through DataFusion upgrades (DP-15).
- §4.1/§4.3 describe migrations and recorded-witness compatibility that §20.5 ("no compatibility
  reader or backfill") and ADR-0182 disown (DP-24).

**Counterevidence.**
- §4.6 explicitly places relational obligations in admission owners and rejects "an imagined
  global SQL enforcement stage". §3.2 states pse-rules' narrow role.
- An extension point can be justified by a roadmap axis, and 58e's composable protected
  relations might want SQL composition. However, 58e names `TableProvider`/`execute_stream`
  directly, not the engine session.
- Whether Surreal assertions cover all 15 canonical foreign keys, and the engine-internal
  reachability of `commands`/`mutation` through the admission path, were only partly traced.

## Retired and moved identities

| ID | Outcome at `f83bac039` | Decisive evidence |
|---|---|---|
| <a id="da-015"></a>DA-015 | Premise largely retired | Python MCP, CLI generator and global catalogue deleted (`f321e9e0f`). Native project operations use typed effects and shared defaults (`pse-runtime/src/host_operations/project.rs`; the dirty tree removes the PyO3 method projection). `pse-py`/`_workflow.py` hand methods remain until 58g G03; revision 311 (`2232bfa02`) schedules `pse-py` removal |
| <a id="da-017"></a>DA-017 | Resolved | Default-settings capability record refined by `capability_for` (`pse-backend-native/src/execution.rs:1137-1155`; `execution/kinsol.rs`). Dynamics attribution goes through `dynamics::backend(method)`. Non-algebraic selection is owner-decided (§18.1, ADR-0209), as noted under [DA-053](#da-053) |
| <a id="da-019"></a>DA-019 | Narrowed; live on a retiring path | The runtime no longer spawns the Python supervisor. Worker and deployment roots are `pse-runtime` binaries (`bee1a2271`). The closure and restore lists are derived. The 11.6k-line Python supervisor is still the operator path for server, backup and restore (`justfile:110-114`; 58b B04, 58h). Coexistence is under [DA-057](development-tooling.md#da-057) |
| <a id="da-033"></a>DA-033 | Resolved | `canonical_settlement.rs` `settle_exact` is shared by about ten owner modules; the Python file protocol is retired (`f6fcc4e09`). Per-owner unsettled variants remain legitimate |
| <a id="da-012"></a>DA-012 | Resolved (2026-10-25) | One run supervisor |
| <a id="da-013"></a>DA-013 | Resolved (2026-10-25) | One pure prediction policy; orchestration duplication is [DA-054](#da-054) |
| <a id="da-014"></a>DA-014 | Closed by decision (I22, RC12) | Not re-examined |
| <a id="da-016"></a>DA-016 | Merged (2026-10-25) | Into DA-017 |
| <a id="da-018"></a>DA-018 | Closed by decision (I23) | Not re-examined |

Earlier full text is at source `682fc3830`.
