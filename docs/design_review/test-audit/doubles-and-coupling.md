# Support ownership, doubles and coupling

**Interface-checked · refreshed 2026-10-29 · source `7bd341f0bd4a27ad629008abdff56ee2bcf7e9f3` plus
the working tree.** This page holds the primary findings on where test support and evidence live, how
tests couple to production, and the seams production carries for tests. See the
[collection baseline](README.md#baseline-and-method) and [integrated account](architecture-and-assurance.md).
Finding bodies keep their 2026-10-27 evidence (source `87f2eb905`) unless the refresh section says
otherwise. Prior diagnoses are preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/doubles-and-coupling.md);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition of
DTA-001 to DTA-058; DTA-059 and DTA-068 to DTA-070 have no disposition owner yet.

## Refresh at `7bd341f0b` — 2026-10-29

Two canonical fixture models now coexist. The native private service
(`pse_operations::NativeCanonicalService`) starts a systemd-bound SurrealDB service per test, connects
through production `CanonicalStore::connect_ready` and settles owners on every path; it serves
`native_study`, `native_repeated_analysis`, `native_worker_lifecycle` and `worker`. The legacy registry
fixture (`pse_operations::testing`) still shells out to `python -m scripts.test_resources` for
register/drain/status (`testing.rs:398-460`), derives identity from argv `--exact` (`:518-536`) and serves
about 30 production-crate test modules plus benches. The [scoped study and database page](study-execution-and-database-boundary.md)
is revalidated here.

| Finding | Outcome | Current residual and decisive evidence |
|---|---|---|
| [DTA-055](#dta-055) | Live; recurs in the native replacement | The shipped fixture drain (`canonical.rs:2453-2472`, `workflow/mod.rs:619-640`, `pse-py/src/workflow.rs:772-820`) is unchanged. `NativeCanonicalService` adds `#[cfg(feature = "test-support")]` fields and shutdown branches (`native_service.rs:434,558,729,1234,1310-1317`); because `pse-runtime`'s dev-dependency enables `test-support` and workspace feature unification applies it to every test-mode build (`cargo tree -e normal,dev` shows `[default,native-service,test-support]`; `-e normal` omits it), every test exercises a different shutdown composition than a production build, and non-test builds never contain these seams. Its only consumers so far are fixtures. The migration owner adds more: `Query` carries a held-transaction field and dispatch branch for tests (`canonical_client.rs:763,995-1005`); the private `advance_v2_checkpoint(discard_acknowledgement: bool)` carries an ungated parameter whose only `true` caller is a test hook; `install_pre_epoch_fixture`/`install_schema_v2_fixture` sit in the production migration module gated only by `native-service` (`canonical_migration.rs:191-254`); assertion bodies (`canonical_studies/progression_controls.rs`) are a public library module under `test-support`. The scoped `ManagedStudyFixture` hybrid remains (`study_tests.rs:3521-3534,4350-4385`). |
| [DTA-051](#dta-051) | Partial; new footprint | MCP drivers retired with G02; non-collected `scripts/tests` modules fell from 63 to 47. The Python parity candidate harness outlived its last consumer: `python/pse/parity/tests/support.py` (228 lines, constructs `pse.Runtime`), the `parity_context` fixture and `python/pse/tests/test_parity_context.py` remain after all four families moved native, and `just parity` still builds the full native extension to run five preflight assertions. Scenario-builder duplication recurs natively in [DTA-059](#dta-059). About 21 `scripts/tests` check and acceptance files (about 5.9k lines: `telemetry_*_acceptance`, `grpc_frame_*`, `plan50_*`, `plan52_*`, `active_*_control`, several `surreal_*_check`) have no recipe, script, guide or live-plan consumer, yet `telemetry_semantics_acceptance.py` was still edited in `ef1ce1956`. |
| [DTA-027](#dta-027) | Partial, extended | `RUST_TEST_OWNERS["pse-runtime"]` gained `"reference_pid_exchange::"` (`scripts/scientific_inputs.py:32`), matched by `startswith` against the keys of nextest's `testcases` map (`:96-105`); nextest keys integration-binary cases by their in-binary path (`native_pi_loop_matches_petsc_…`, `reference_pid_exchange.rs:180`) and holds the binary only in `binary-id`, so the entry never matches. `native_dynamic_workflows` reads the capsule (`:13-14,76`) and is not listed. The native route derives no demand; `PSE_SCIENTIFIC_INPUTS` is bound by hand. |
| [DTA-028](#dta-028) | Partial; wider footprint; primary home for untyped refusals | New production footprint: `pse-host`'s `HostError::Ownership(String)` absorbs invalid input, unsupported capability, ownership and infrastructure preconditions under one diagnostic code (`crates/pse-host/src/lib.rs:21-62`); test-side consequences are in [DTA-067](oracles-and-acceptance.md#dta-067). Modeling/compiler still mixes 26 `to_string().contains(` with 17 `invalid_reason(`. Runtime `contract()` maps to `WorkflowError::Input(String)` (`workflow/mod.rs:555-557`), and the native binaries migrated from Python carry about 22 wording-substring negative assertions (for example `native_scientific_modes.rs:257,342`, `native_diagnostics.rs:302,722`). |
| [DTA-011](#dta-011) | Resolved for the native fixture; legacy residual live | The native fixture connects through production `connect_ready` with no `fixture_lifetime` (`native_service.rs:1267-1282`). Legacy `own_fixture` routing remains for the legacy population, whose Python complement has no supported route. |
| [DTA-009](#dta-009), [DTA-010](#dta-010) | Partial, unchanged | No commit touched `test_observation.rs` or either residual-body helper after `87f2eb905`. |

## <a id="dta-059"></a>DTA-059 — Native integration binaries restate fixture policy that drifts from production construction

**Principles:** AP-01, AP-03, AP-06, AP-07, DP-16, DP-17, H12; companion §2, §3, §5.
**Status:** new, 2026-10-29. Lineage: the native counterpart of the scenario-builder residual in
[DTA-051](#dta-051) and the construction divergence of [DTA-010](#dta-010).

**Claim at risk.** 58f F02: fixtures are constructed "through the actual native owner", so native
journeys exercise production construction policy, and case-defining setup changes in one place.

**Structural cause.** Each `tests/*.rs` file is its own crate. Support is shared only by `#[path]`
mounting; `reference_support/{mod,context}.rs` is mounted by about twelve binaries, while others carry
private copies, and per-family supports are reused under `allow(dead_code, unused_imports)`
(`native_scientific_modes.rs:14-36`). No owner above `SharedRuntime::build` and
`NativeCanonicalService::new` encapsulates budget selection, service-spec selection or package
construction:

- **Runtime budget.** Five near-verbatim literal budgets (4 GiB pool, 16 threads, cache disabled,
  1 GiB worker/workspace bytes, `jobs` set in only some) in `reference_support/context.rs:22-55`,
  `local_operation.rs:42-59`, `native_study.rs:127-146`, `native_repeated_analysis.rs:145-163` and
  `native_workspace.rs:57-74`; `worker.rs:399-423,617-631` and `tests/support/workflow_runtime.rs`
  differ again. Production derives these from the pool (`MathPolicy::for_pool`,
  `src/bin/pse-worker.rs:137-142`).
- **Service spec.** Six copies of environment-to-spec selection with differing precedence and messages
  (`tests/native_service.rs:19-42`, `native_study.rs:55-94`, `worker.rs:80-121`,
  `native_worker_lifecycle.rs:31-58`, `result_pruning_tests.rs:36-65`,
  `canonical_publication_controls.rs:17-44`). The `PSE_NATIVE_FIXTURE_*` / `PSE_WORKER_BINARY` /
  `PSE_NATIVE_STUDY_WORKER` protocol has no single declaration.
- **Minimal model package.** An unchecked text splice,
  `.replace("dependencies = []", ...)`, appears in twelve files (integration tests, a bench, and three
  inline test modules under `src/`: `workflow/modeling.rs`, `workflow/reproducer.rs`,
  `bin/project/runtime.rs`), while
  `native_project_sources_support/mod.rs:27-66` derives the dependency from the primitives header
  instead.

**Consequence.** A production budget or derivation change does not reach in-process journeys, which keep
passing under a configuration production no longer uses. A primitives version or manifest-format change
silently no-ops in the spliced copies while the typed copy adapts. Each new journey copies about 40
lines of service and budget setup, and each binary repeats construction and link work (static
observation only).

**Evidence and uncertainty.** Established for the cited sites; about 15k lines of native tests were
sampled, not read. It is not established that any journey's scientific claim depends on the literal
budget. **Counterevidence:** every copy uses public constructors and none reaches private internals;
`worker.rs:100-112` reads production `ProductionProfiles`; some local repetition aids readability
(companion §2). A lower-confidence related observation: `pse_testkit::factory` builds `EngineFactory`
without the cache service that production `session_factory` attaches, so its consumers run cache-absent
branches; their admission and rule claims probably do not depend on them.

## <a id="dta-068"></a>DTA-068 — Host provider preparation is established by fakes, refusals and a lock test that calls no product code

**Principles:** AP-02, AP-06, DP-10, DP-22, DP-23; companion §1, §5, §6.
**Status:** new, 2026-10-29.

**Claim at risk.** 58b: explicit provider preparation (the active `compiler,klu` profile) is
content-keyed and reused, serialized per recipe, cancellation-safe, and never publishes a partial
generation.

**Structural cause.** KLU `prepare` (`crates/pse-host/src/native.rs:568-640`) is one effectful procedure
(source digest, recipe key, `locks/<recipe>`, generation scan, build, publication) with no seam between
identity/plan and tool execution. Tests either fake every tool or exercise refusals:

- `tests/native.rs:58-88` (`native_cancellation_releases_kernel_lock_without_publishing`) spawns its own
  `native_lock_holder` that `flock`s a temp file, kills it and asserts the lock is free; no pse-host
  function is called, and its "without publishing" check inspects a directory nothing writes to;
- the success path of KLU `prepare` has only refusal tests; selection tests begin from a hand-published
  generation whose recipe key is computed by the production hash (`native.rs` tests), and only a
  `CFLAGS` change is checked as a key input (`providers.rs:1223-1238`);
- solver, isolation, Uno and PETSc recipes run shell fakes and assert argv substrings
  (`providers.rs:1582-1641,1702-1806`), appropriate for argv as a contract but silent on usable artifacts.

Two fixture entry points are ordinary tests that pass vacuously when their variable is unset:
`ordinary_bound_service_fixture` (`tests/foreground.rs:225-233`) and the `PSE_CORRECTNESS_REAL_CARGO`
check (`tests/correctness.rs:156-166`), which would also pass if the adapter stopped propagating the
variable it exists to guard. `tests/native.rs` marks its own fixture `#[ignore]`, showing the
alternative is known.

**Consequence.** No test shows that the recipe key captures the material inputs or that concurrent
preparations serialize, and no test interrupts a real build; a regression surfaces
only in an operator's `native prepare`. Pass counts include no-op passes.

**Evidence and uncertainty.** Established for the cited tests. Operator preparation runs may be the
intended integration evidence; no route records them. **Counterevidence:** atomic `current` pointer
publication is tested under concurrency against the real function; corrupt and writable payloads are
refused; `native_failed_publication_keeps_previous_selection` (`src/native.rs:1009-1028`) shows a failed
publication keeps the previous selection through the real `publish_generation`/`select`; isolation runs
real `pkg-config`; bootstrap publication is tested with an injected interruption.

## <a id="dta-069"></a>DTA-069 — Real-effect host tests bind to the operator's live admission journal and capacity policy

**Principles:** AP-06, AP-02, DP-20; companion §1, §5, §6, §8.
**Status:** new, 2026-10-29.

**Claim at risk.** Host admission and foreground tests check admission rules in isolation and do not
perturb, or depend on, concurrent workloads.

**Structural cause.**

- `tests/foreground.rs:530-579,687-765` run `boundary --resource-class light` against the live user
  manager and read and assert `$XDG_RUNTIME_DIR/pse-host-admission/finite.json`, the journal production
  admission uses (`src/admission.rs:926-933`); `:581-685` creates a slice under the live `app.slice`.
  Nothing schedules these tests apart from other host workloads. Inline admission tests, by contrast,
  use a private `XDG_RUNTIME_DIR`.
- Inline selection tests compile in the operator's `.config/agent-capacity.toml` with `include_str!`
  (`admission.rs:1837-1839`) and assert
  its current numbers (2 GiB light, 24/64/128 GiB classes, allocations 152/168, reference 140/160,
  observer 144, production 72; about `:2088-2181`); one expectation comes from the same table under test.
  `foreground.rs:559` asserts the kernel `memory.max` equals `2 << 30`.

**Consequence.** Outcomes depend on concurrent host state: spurious refusals within the admission
window, or test charges delaying a concurrent timing or exclusive admission; tests write and reclaim
production journal rows. Each policy retune (the file changes often) breaks selection-logic tests even
when logic is unchanged, while the rules themselves are never checked against values the test controls.

**Evidence and uncertainty.** Shared-state cause established; interference suspected, not observed.
The journal location derives from `XDG_RUNTIME_DIR`, which also serves bus discovery, so separating them
may need a production override (unverified). **Counterevidence:** reading the real policy catches an
unparsable or contradictory file, a separate valuable claim; real containment journeys need the real
manager.

## <a id="dta-070"></a>DTA-070 — Python orchestration tests certify flows whose real first step now refuses, against test-authored configuration

**Principles:** AP-02, AP-04, AP-06, DP-01, DP-23, H20; companion §1, §4, §5.
**Status:** new, 2026-10-29 (source `ef1ce1956`, which committed the refusal).

**Claim at risk.** The lifecycle, fixture and recovery orchestrations exercised by their unit suites are
executable, and the remaining `surreal_server` readers are tested against configurations the system
produces.

**Structural cause.** Fresh Python construction now refuses before effects
(`scripts/surreal_server.py:2214`, `surrealism_module.refuse_legacy_construction`). The real check scripts
still construct through it (`surreal_fixture_check.py:760`, `canonical_recovery_check.py:545`,
`canonical_boundary_run.py:356`, `surreal_logging_check.py:121`,
`surrealkit_persistent_fixture_check.py:141`), while their unit suites replace it with
`patch.object(..., "setup")` (`test_surreal_fixture_check.py:278`, `test_canonical_recovery_check.py:499`,
`test_plan30_lifecycle.py:918`, `test_canonical_boundary_run.py:789,855`). 29 patches use `create=True`,
disabling `mock`'s guard against renamed or deleted collaborators. The commit also removed the production
writer of the Python profile-v3 `config.json`; `test_surreal_server.py:2313-2350` now hand-authors it,
and seven test files write the literal `"profile_version": 3`. A typed producer of the core exists
(`ServiceIdentity { profile_version: Version<3> }`, `pse-operations/src/deployment.rs:17`, written by
`native_service.rs:197-210`); the Python-only superset (`owner`, `instance_id`, `server{…}`, resident and
logging fields) has none.

**Consequence.** `just surreal-fixture-test` and `just canonical-recovery-test` can no longer construct
their subject while the companion suites stay green; the doubles certify a sequence whose first real
step refuses. Python readers are tested only against test-maintained documents whose Python-only fields have no
producer, and no test checks them against the native writer, so drift between copies, from the native
writer or from real existing stores is undetectable.

**Evidence and uncertainty.** Established by reading the code path; recipes were not run. These readers
may be intended only for existing stores until cutover, in which case the schema gap is transitional.
**Counterevidence:** the refusal itself is directly tested (`test_surrealism_module.py:246`), as the
environment guide's refuse-before-effects contract requires.

## <a id="dta-051"></a>DTA-051 — Test evidence and support are placed by creation history rather than enduring responsibility

**Principles:** AP-01, AP-03, AP-05, DP-17, DP-22, DP-24, H26, H27; companion §2, §3, §6.
**Status:** new, 2026-10-27.

**Claim at risk.** A retained control is current, runnable and protects a stated claim; tests and
support can be found, changed and retired by the responsibility they serve. The repository's own
rules require it: AGENTS.md forbids retaining a path "as evidence", ADR-0160 asks contracts to describe
responsibility boundaries rather than historical tests, and ADR-0096 retires completed plans.

**Structural cause.** A control's route, claim and owner are recorded in the plan or packet that
commissioned it, and support is created inside whichever plan driver or test module needed it first.
The retirement preflight in `docs/dev/documentation.md` finds executables that consume a plan
*document*, not executables whose only *route* is that plan. Retiring a plan therefore silently
orphans its drivers, and enduring tests acquire imports from plan-era modules.

**Footprint (representative).**

- `scripts/tests` holds 63 non-collected driver modules (about 29k lines). Eleven have no route in any
  recipe, guide, architecture text or plan history, including `mcp_native_acceptance.py` (about 2,100
  lines), `mcp_isolation_acceptance.py`, `active_mixed_control.py`, `active_diagnostic_control.py`,
  `plan52_test_evidence_check.py`, `surreal_otlp_check.py`, `surreal_logging_check.py` and
  `grpc_frame_probe.py`/`grpc_frame_relay.py` (hand-coded HTTP/2 framing). About 25 more are routed
  only by completed plans (ten `telemetry_*_acceptance.py`, eight `plan52_*.py`, inquiry drivers).
- Enduring collected tests import plan drivers: `test_telemetry_metrics` from
  `telemetry_operational_acceptance`, `test_surreal_diagnostics` from `surreal_mx_check`,
  `test_surrealkit_maintenance` from `surrealkit_rollout_fixture_check`. Twenty-five tooling test
  modules import from other `test_*` modules; `surreal_server` lifecycle behavior is spread across
  about twenty modules, some plan-named (`test_plan30_lifecycle`, about 2,000 lines).
- Product tests and support carry plan identities: `tests/support/plan14.rs` (used by conformance
  acceptance and `benches/native_process.rs`), `tests/fixtures/plan14/`,
  `python/pse/tests/test_plan14_acceptance.py`, fifteen Rust tests named `plan38b_*`, `plan39_*`,
  `plan47f_*`, `plan30_*`, `PSE_PLAN30_*` variables in an enduring guide, and the
  `just plan53a-reference` recipe.
- Python scenario builders live in the test modules that first needed them:
  `test_native_workflow.{package_documents, declaration, physical_context}`,
  `test_modeling_kernel.{physical, on_primitives}` and private `_package` helpers are imported across
  about twenty files. The primitives dependency (package `5a5a…` at 1.0.0) is restated eight times
  and spliced with `manifest.replace("dependencies = []", PRIMITIVES)`, which silently does nothing if
  the fixture text drifts. Shared `physical_input_fixture.py` exists but is partly adopted.

**Consequence.** Refactors keep route-less drivers type-correct without running them (commits
`d7d0169af` and `159d63169` edited fourteen and fifteen drivers, including route-less ones), so readers
can believe they are current. Retiring 52p–52s would orphan ten more. Owners must find tests by plan
number; product-test refactors ripple into drivers with no route; a drifted manifest silently
produces a different scenario.

**Strongest legitimacy argument.** A manual route in a guide is real (the plan30 drivers in
`agent-environment.md`, capability inquiry drivers kept for the R-55 trigger); the execution rhythm
forbids running them mid-plan; sharing an independent oracle between acceptance and its unit test is
good practice. These justify routed drivers and shared oracles, not route-less drivers or plan
identity as the organizing key.

## <a id="dta-055"></a>DTA-055 — Production types carry test choreography and fixture seams across several vocabularies

**Principles:** AP-01, AP-02, DP-18, DP-21; companion §5.
**Status:** new, 2026-10-27; related to closed [DTA-011](#dta-011).

**Structural cause.** Deterministic interleaving control and fixture lifetime are threaded through
production struct fields, parameters and call-site arguments under four overlapping vocabularies:
`cfg(test)`, `test-support`, `canonical-tests` and conjunctions with solver features. Static counts show
hundreds of `cfg(test)`-family items outside test modules in `pse-runtime`, and over a hundred in
`pse-operations`. Examples: `workflow/shooting/window_team.rs` gates and buffer identities;
`workflow/fitting/experiments.rs` (`gate`, `wave_gate`, `fixed_affinity_for_test`);
`workflow/physical_cache.rs` admission pauses and counters; `math/staged.rs` with a `cfg(test)`
parameter and a `cfg(not(test))` twin; `workflow/durable.rs::FinishExportGate`. Test-only counters
(`result_files.rs::PREFLIGHT_CALLS`, `canonical/local.rs::CONSTRUCTION_DECODES`) sit beside the existing
production observation channel. A `canonical-tests` worker variant has a different `serve` signature.

Two seams change exercised behavior or the shipped surface:

- Under `test-support`, `CanonicalStore::fixture_lifetime` routes protected release to the fixture's
  executor (`FixtureLifetime::cleanup_executor`) and changes drop order, so Rust canonical tests do not
  exercise the production executor path; the complement comes from Python's support-absent closure
  and a few `fixture_lifetime: None` transport tests.
- `Runtime::fixture_connection_owners` and `CanonicalStore::drain_fixture_connection_before` are
  ungated `#[doc(hidden)] pub` items, surfaced as `pse-py::NativeRuntime::_drain_fixture_connections`
  in the production extension and guarded only by a `canonical_test_` database-name prefix.

**Consequence.** Readers of core concurrency code must filter test choreography; seam combinations
compile only in particular feature sets; the shipped API includes fixture operations guarded by
convention.

**Legitimacy and limits.** Deterministic cancellation and drain interleavings back truthful-outcome
claims and are hard to obtain otherwise. Instrumentation does not reach the shipped extension
(`test-support` is dev-only; `pse-py` has no test feature), apart from the doc-hidden drain. The
purpose is legitimate; the cost is the scattering, the duplicated counter channel and the shipped
fixture surface. A related lead: `scripts/tests/test_surreal_server.py` (about 12k lines, about 1,000
patch sites) mostly patches the supervisor module's own functions; its likely cause is the
production supervisor's internal structure, which the broad account treats under
[DA-037](../audit/development-tooling.md#da-037). Not raised separately here.

## <a id="dta-027"></a>DTA-027 — Reference-source dependencies are maintained in several test assembly lists

**Refresh:** partially resolved. The 19-family catalogue derives the closure from production
capture once (`xtask/src/scientific_inputs.rs`), a real improvement. Residuals:

- **Demand side is hand-declared in three places**: the catalogue's `consumers` crate list,
  `scripts/scientific_inputs.py::RUST_TEST_OWNERS` (private `pse-runtime` test-module prefixes) and
  the Python test-file name list in `prepare_python` (`test_studies.py`, `test_plan14_acceptance.py`, …). Readers (`tests/fixtures/scientific-inputs.rs::{package_documents, file_bytes}`,
  `python/pse/tests/_scientific_inputs.py`) accept any captured path regardless of declared family.
  A new `pse-runtime` test outside the listed prefixes that calls `actual_reference_recycle_fixture`
  gets no preparation and panics with a message that looks environmental.
- **Live readers bypass the capsule**: `pse-modeling/src/domain_schema.rs::schema`,
  `pr_jacobian_tests.rs`, `math/solves/derived_tests.rs`, `pse-admission/tests/quantity_relations.rs`
  (a non-declared consumer) and `benches/benches/modeling_preparation.rs` read `packages/reference`
  directly, without exact-byte identity or archive replay.
- **A text-based adaptation outside the catalogue**: `tests/fixtures/reference-source-fixture.rs::{reference_family_sources, rows}`
  injects an import with `replacen("package bt_ideal {", …)` and renames lineage
  `idaes_thermo.rpp4_pressure_oracles`; neither checks that it applied. Reformatting `bt-ideal.pse`
  breaks every bounded-reference consumer with an unrelated admission error; renaming the oracle
  dataset silently removes `pressure_bindings`, which no test asserts.

## <a id="dta-009"></a>DTA-009 — Reuse observations depend on private query names and some lack positive controls

**Refresh:** partially resolved. `pse-compiler/src/workspace/modeling/test_observation.rs::{Query, QueryObservations::count}`
centralizes spellings and refuses missing/truncated traces, but matching is a substring search over
salsa's `Debug` event output and only `Query::Parsed` has a positive control. `EffectiveMembers` and
`Projection` assertions are equalities that a rename satisfies with zero on both sides, so
`relocating_unchanged_declarations_preserves_type_and_member_memos` can pass vacuously for member memos.

## <a id="dta-010"></a>DTA-010 — Implicit fixture construction duplicates preparation mechanics across crates

**Refresh:** partially resolved. `SourceBody::prepare` absorbed the common finish step, but the two
residual-body copies remain and still differ: `pse-math/src/implicit.rs::residual_bodies_for_test`
uses optimization parameters and evaluation limits, `pse-backend-native/src/implicit.rs::test_residual_bodies`
uses defaults, and both differ from `pse-runtime/src/math/native_child.rs::compile_problem`.

## <a id="dta-028"></a>DTA-028 — Free-text modeling refusal contracts couple some negative tests to wording

**Refresh:** mechanism established, closure scope overstated. Source-owned `ModelingContractReason`
covers the migrated distinctions (about eighteen `invalid_reason` sites) while the large remainder of
`invalid(...)` sites construct `Unclassified`, and about a hundred `to_string().contains(` assertions
remain in modeling/compiler tests. Read as resolved for migrated distinctions only.

## <a id="dta-008"></a>DTA-008 — Static routing policy tests are not evidence of a defective test-only composition

Withdrawn 2026-10-23; never reused. Complementary static and contextual readiness tests remain.

## <a id="dta-011"></a>DTA-011 — Fixture lifetime ownership changes the canonical-store path exercised by some tests

**Refresh:** closure established as scoped: `canonical-tests` selects participation, `test-support`
selects instrumentation, and support-absent Python reader/drop journeys exercise the complementary
production path. The wider seam pattern is [DTA-055](#dta-055).
