# Evidence routes, activation and outcomes

**Interface-checked · refreshed 2026-10-29 · source `7bd341f0bd4a27ad629008abdff56ee2bcf7e9f3` plus
the working tree.** This page holds the primary findings on whether declared tests actually execute,
under the prerequisites and scheduling they need, whether outcomes are truthful, and whether
interleaving claims are forced. See the [collection baseline](README.md#baseline-and-method) and
[integrated account](architecture-and-assurance.md). Finding bodies keep their 2026-10-27 evidence
(source `87f2eb905`) unless the refresh section says otherwise. Prior diagnoses are preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/evidence-routes-and-execution.md);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition of
DTA-001 to DTA-058; DTA-060 has no disposition owner yet. Line citations refer to `f5b436bbf` unless
stated; see the [baseline](README.md#baseline-and-method).

## Refresh at `7bd341f0b` — 2026-10-29

Plan58 moved most Rust recipes (`test`, `test-package`, `unit-package`, `worker-test` and others) to
the native adapter `scripts/cargo-bin/cargo-pse-correctness` (`crates/pse-host/src/correctness.rs`),
which resolves the Arrow-leaf closure once per Cargo invocation and passes arguments straight to
`cargo-nextest`. The Python route (`scripts/select.py` → `test_run.run_rust`) remains for
`unit-native-package` and `unit-native-selected`; every recipe that passes `--store`
(`native-test`, `native-python`, `py-test`, `canonical-test`, the assessment `native-test` gate)
currently refuses before execution (`crates/pse-host/src/boundary.rs:116-118`;
[environment guide](../../dev/agent-environment.md)). The route families are compared in
[DTA-063](test-execution-system.md#dta-063).

### DTA-050 at this source: the native cutover widened it

DTA-050's cause, that no owner establishes that declared tests execute somewhere with their
prerequisites, now governs most replacement evidence:

- **Feature-gated targets are silently absent from ordinary routes.** 16 of the 22 `pse-runtime`
  integration binaries (16 of 20 declared `[[test]]` targets) carry `required-features` (`crates/pse-runtime/Cargo.toml`); Cargo skips such
  targets unless their features are selected, so `just test` and `just test-package pse-runtime` build
  only the ungated ones and report success. Nothing records the omitted inventory; the managed route's
  `not-run` outcome has no native counterpart.
- **The study/worker replacement suites have no aggregate route.** `native_study`,
  `native_repeated_analysis` and `native_worker_lifecycle` require `native-study-tests`; the only
  recipe selecting it runs `--test worker` (`justfile:274`), and `native_tests.py:120-160` does not
  derive it. The library protected-result controls moved from `canonical-tests` to
  `native-study-tests` (`result_snapshot.rs:1392`), removing them from the aggregate that previously
  reached them. The `pse-operations` publication controls are an inline module gated by
  `cfg(all(test, canonical-tests, native-service))` (`canonical_execution.rs:44-46`); they use the native
  fixture, but no recipe selects that feature pair, and the only `pse-operations` `canonical-tests`
  recipe passes `--store` and refuses.
- **The four reference exchanges have only a documented hand route.** They read
  `PSE_{DEGENERACY,COVARIANCE,SENSITIVITY,PID}_REFERENCE_PYTHON`, which no recipe sets
  (`docs/dev/native-workflow.md:436-455` is the only supplier), and `expect()` on absence
  (`reference_degeneracy_exchange.rs:81-82`). The comprehensive native route's
  `native-solvers` closure satisfies their `required-features`, so once `--store` routes run again
  they would compile and panic for a missing prerequisite. `just parity` runs only the preflight
  ([DTA-058](oracles-and-acceptance.md#dta-058)).
- **The managed and legacy canonical populations have no supported route.** The eight
  `managed_primary_` cases (`.config/nextest.toml:32`) and the `canonical-tests` legacy-fixture tests
  in about 30 production-crate files depend on Python controllers reached only through `--store`.
  Plan58 acknowledges this (58c, 58b closeouts). In `pse-runtime/src/workflow` alone about 134
  inline tests sit under `cfg(all(test, feature = "canonical-tests"))` (study, durable, physical-cache,
  connected-results, result-inspection, data-document tests), and their fixture requires an existing
  Python-supervised store through `PSE_SURREAL_STATE` (`pse-operations/src/testing.rs:667-693`), whose
  fresh construction now refuses ([DTA-070](doubles-and-coupling.md#dta-070)). These cover most of the
  roughly 4.5k lines of study, durable and connected-results change since `87f2eb905`. About 92 Python
  component and integration tests are likewise reachable only through the refused `py-test`.
- **The canonical native-service target has no recipe.** The `pse-operations` `native_service`
  target requires `native-service` (`Cargo.toml:115-118`); no recipe routes it and 58d records two
  different hand-written feature sets. Its `test-support`-gated fault and progression blocks
  (`tests/native_service.rs:707,834,945`) are not hidden: under workspace feature unification every
  test-mode build activates `pse-operations/test-support` through `pse-runtime`'s dev-dependency
  (`cargo tree -e normal,dev` shows `[default,native-service,test-support]`; `-e normal` omits it).
- **Dead and retired route references.** `scientific-replay-miri` always refuses because the adapter's
  mode allow-list has no `miri` (`correctness.rs:135-178`; `justfile:263-264`); the `affected`
  preview without preparation prints the retired `python -m scripts.arrow_validation` command;
  `python-runner-unit-test`'s filter matches nothing (`justfile:252-253`).

The earlier manifestations (the `pse-authoring/arbitrary` property test, the solver-gated pure
refusal, receipt-dependent qualification tests) persist. **Counterevidence:** the targets fail loudly,
never skip; the plans scope their evidence honestly as checkpoint runs; `native_design`,
`native_conformance`, `native_source_reload` and `native_conic` are ungated and reached by `just test`.

### Other findings on this page

| Finding | Outcome | Current residual and decisive evidence |
|---|---|---|
| [DTA-015](#dta-015) | Inherited by the native route, widened | `.config/nextest.toml` declares `scientific` and `store` groups but assigns none (`test-group` occurs in no override); assignment exists only in the Python projection `test_requirements.nextest_configuration` (`scripts/test_requirements.py:417-456`). `unit-package` and `worker-test` moved from grouped to ungrouped. The config comment (`nextest.toml:52-54`) describes a projection the native route never performs. Concurrent-demand consequence: [DTA-046](execution-efficiency.md#dta-046). |
| [DTA-016](#dta-016) | Live on the Python route (F02 removal pending) | Dead `canonical_study_module_tests::` prefix and narrowed native prefixes persist; the native route carries no classification at all. |
| [DTA-017](#dta-017) | Partial, live | `scientific_inputs.capture` is still a bare `cargo run` without leaf proof (`scripts/scientific_inputs.py:58-74`) and is the producer named in the 58f reference checkpoint. |
| [DTA-025](#dta-025) | Inherited by native; primary home for setup reporting | Native prerequisites arrive as environment variables checked by `expect` inside the test (`native_study.rs:56-80`, `worker.rs:82-131`, `tests/fixtures/scientific-inputs.rs:30`); the legacy fixture classified setup stages (`pse-operations/src/testing.rs:540-588`), the native fixtures do not. A typed `Refused` from `NativeCanonicalService::launch` outside containment is `unwrap()`ed. `worker-test` checks two variables and derives a third; the module receipt/state selector is unchecked (`justfile:271-273`). 58c records a confirmation run that refused before child launch for an omitted worker variable and was classified by hand. |
| [DTA-057](#dta-057) | Partial, extended | The SCIP cancel case now requires an observed `scip.incumbent`. The kill case keeps a fixed 5 s post-fence window and adds a 31 s sleep because "NativeStudyWorker currently uses the production thirty-second lease" (`worker.rs:1366,1412-1414`), although `pse-worker` accepts `lease_seconds` (`src/bin/pse-worker.rs:52-55`) and the in-process spec does not expose it. `native_study.rs:896-899` asserts non-completion within 50 ms. Canonical lease expiry is likewise reached by polling the server clock or a fixed `sleep(2100ms)` (`canonical_conflict_tests.rs:1411`). By contrast, the "competing workers" control prepares both scopes, then claims in order (`progression_controls.rs:304-331`): a deterministic stale-preparation interleaving, which the companion favours; only a truly concurrent commit race is unexercised. A host negative test kills the child immediately after spawn, then waits 100 ms and asserts absence of an effect it most likely never reached (`pse-host/tests/foreground.rs:474-489`). |
| [DTA-012](#dta-012) | Established on Python; premise retired on native | The native route returns the nextest exit; no second verdict exists to disagree. |
| [DTA-013](#dta-013), [DTA-014](#dta-014), [DTA-019](#dta-019) | Established | No change to their decisive sources. |

## <a id="dta-060"></a>DTA-060 — Native fixture disposal, retention and deadline policy is decided in each test body

**Principles:** AP-05, AP-06, DP-19, DP-20; companion §6, §8; 58f F02/I04.
**Status:** new, 2026-10-29. Related: setup failure reported as test failure stays with
[DTA-025](#dta-025); the general absence of a convergent retirement owner stays with
[DTA-052](execution-efficiency.md#dta-052).

**Claim at risk.** 58f F02 requires that a fixture settle its owned resources "after assertions pass
or fail", retain diagnostics outside the disposable store and avoid success-gated retention.

**Structural cause.** `NativeCanonicalService::run` settles physical owners on every path
(`crates/pse-operations/src/native_service.rs:1116-1171`), a real strength. The rest of a fixture's
outcome policy is left to each test body:

- **Disposal.** Disposal of the private state is a separate caller call (`dispose`, `:487`). Three
  conventions are in use: dispose on both paths (`result_pruning_tests.rs:205-213`,
  `native_repeated_analysis.rs:1766-1782`); dispose only after body and drain succeed
  (`native_worker_lifecycle.rs:148-161`, `worker.rs:308-323`); dispose only after an `.unwrap()`,
  implicitly (`tests/native_service.rs`, `native_study.rs:1203-1238`).
- **Retention.** The retained diagnostic is the whole private service root, database and 0600
  credentials included, under a `state_parent` that each test copy sets to `std::env::temp_dir()`
  (`native_service.rs:616-618`). Nothing references its `pse-canonical-native-` prefix for reclamation.
- **Deadlines.** `native_repeated_analysis.rs:1737` runs under a 600 s internal deadline, but matches no
  nextest override, so the default profile (used by recipes, `justfile:50`) terminates it at 360 s
  (`.config/nextest.toml:35`); `profile.local` has no termination. A runner kill bypasses the
  settle-on-panic path; the service is bound to the invocation scope (`pse-host/src/manager.rs:837-871`),
  not to the test.

**Consequence.** Whether failed state survives depends on which test failed, not on a declared policy.
At inspection, 39 `pse-canonical-native-*` directories from 2026-10-29 were in `/tmp` (an observation,
not a cost measurement). The 600 s journey can be killed by the runner before its own deadline reports.

**Evidence and uncertainty.** Established for the cited files; other native files sampled. Whether the
retained directories came from failures later repaired is unknown. **Counterevidence:** retention of
failed state is deliberate (58c); `BindsTo` containment ends service units with the caller; no `Drop`
claims success; "absence fails, never skips" is intended.

## <a id="dta-050"></a>DTA-050 — No owner establishes that each declared test executes on some route with its prerequisites

**Principles:** AP-05, AP-06, DP-22, DP-23, DP-24; companion §4, §6, §8.
**Status:** new, 2026-10-27. It generalizes the inactive-body diagnosis that
[DTA-013](#dta-013) closed for one population.

**Claim at risk.** A test that exists in the tree is evidence that its behavior is checked somewhere,
and a failing test reports a product failure rather than a missing prerequisite.

**Structural cause.** Whether a test body executes depends on independent conditions held by
different owners: `cfg` features, the requirement catalogue's roles, recipe filters, manual receipts
and route-specific feature graphs. The C02 catalogue checks that *selected* cases have declared
requirements and roles; nothing checks the converse, that every compiled-out or role-dependent body
is executed by some sanctioned route under its prerequisites. Current manifestations:

- **Compiled out everywhere.** `crates/pse-authoring/tests/dsl_roundtrip.rs::arbitrary_ast_render_parse_preserves_structure`
  is under `#[cfg(feature = "arbitrary")]`; no manifest, recipe or campaign enables
  `pse-authoring/arbitrary`, and the feature campaign uses `cargo hack check`, which compiles but
  does not run tests.
- **Prerequisite missing from the ordinary population.** The four
  `canonical_studies::canonical_studies_server_unit::surrealism_qualification::canonical_studies_surrealism_qualification_*`
  tests (from `canonical_study_module_tests.rs`, pulled in by `include!`) compile under
  `canonical-tests`, are classified plain `CANONICAL` through the `canonical_studies_server_unit::`
  prefix in `scripts/test_requirements.py`, and panic at
  `expect("explicit separate qualification receipt required")` on a server without the manually
  selected module-qualification receipt (`docs/dev/surreal-substrate.md`). No role or exclusion
  carries that prerequisite, so a native canonical run reports a setup precondition as a product
  failure. The global-capacity case additionally requires the long-hold profile.
- **Filter selects nothing.** `justfile::python-runner-unit-test` filters `test(python_selection_)`;
  no such test exists. Empty selection fails loudly, but the recipe is dead.
- **Default routes are silently partial.** Static parsing shows roughly a third of `pse-runtime`
  tests and most `pse-backend-native` tests absent from the default feature graph used by
  `just test` and unfeatured `just unit-package`. A filter matching gated and ungated tests runs only
  the ungated ones without warning. `scripts/affected.py` suggests only `pse-runtime/native-solvers`
  (`NATIVE_FEATURES`) and never `canonical-tests`. Some pure-precondition contracts are gated on
  infrastructure: `overspecified_root_is_refused_structurally_naming_members` needs `solver-ipopt`
  although its refusal precedes route selection, and `prune_members` is tested only under
  `canonical-tests`.
- **Plan-only routes.** Drivers whose only route was a completed plan
  ([DTA-051](doubles-and-coupling.md#dta-051)) and the reference library
  ([DTA-049](oracles-and-acceptance.md#dta-049)) have no executing owner.

**Consequence.** Readers believe a property is checked when its body never runs; a full canonical
run fails four tests for an environmental reason; the focused loop gives no signal for whole contract
families; and nothing alerts when a refactor leaves a test unreachable.

**Contrary evidence.** Empty selections fail; `just assessment` runs the linked native graph once
with separate feature-absence controls (ADR-0160); C02 catches lost roles inside nonempty selections;
the guides honestly document manual routes and the default graph's scope. Gating real native and
store dependencies is legitimate. The defect is the missing correspondence, not gating.

## <a id="dta-057"></a>DTA-057 — Cross-process native cancellation journeys infer solver entry and interleavings from elapsed time

**Principles:** AP-06, DP-20, DP-21, H23; companion §4, §6.
**Status:** new, 2026-10-27.

`crates/pse-runtime/tests/worker.rs::wait_study_assignment` waits for the `native_started` fence, whose
own comment says it "records assignment, not native solver entry", then sleeps five seconds.
`cross_process_cancel_stops_scip` asserts only terminal state and elapsed time, so a cancel served
before SCIP entry passes as "stops SCIP". `killed_worker_freezes_truthful_observations_and_retries_only_by_authored_policy`
sleeps three more seconds and requires a `scip.incumbent` event; a kill before the first incumbent
fails spuriously, indistinguishable from lost observations. Similarly,
`canonical_studies_surrealism_qualification_cancelled_apply_settles_original_receipt` labels branches
`cancel-before-fence` and `cancel-after-commit` but cancels at 50 ms in both; which recovery path the
first exercises depends on hold length.

**Contrast and legitimacy.** In-process interleavings are forced well: task-local hooks
(`CANCEL_AFTER_ARTIFACT_APPEND_ACK`, `FILE_EFFECT_PAUSE`), observed-entry gates
(`caller_lifetime_tests.rs::FamilyEntryDrain`) and tests that verify their own interleaving premise.
`study_tests.rs` observes native entry through `entries.json`. No native-entry hook crosses the
worker boundary, but the kill test already reads the progress stream that could supply an observed
condition. The footprint is narrow; the consequence is a discrimination gap on a PS-10 truthful-outcome claim.

## <a id="dta-015"></a>DTA-015 — Sanctioned routes compose different test-specific lifecycle and scheduling guarantees

**Refresh:** partially resolved. Managed affected/native/Python/setup routes now share one
catalogue, requirement set, guard and composer (`scripts/test_requirements.py`, `test_run.py`).
Scheduling groups reach nextest only through the per-run projection
(`native_tests.scheduled_rust_arguments` → `test_requirements.nextest_configuration`);
`.config/nextest.toml` assigns none. `just test`, `test-package`, `affected --run` and the
assessment's default `test` gate therefore run native, store and timing cases without grouping.
Guides state association and cleanup limits of bare routes, not this one. Severity is unresolved:
the default graph excludes most native solver tests, and store `max-threads` equals test-threads.

The Python half has the same shape. `pyproject.toml` addopts select `-n 16 --dist=worksteal`; the root
`conftest.py` attaches `xdist_group` markers from `test_requirements.pytest_group`, but pytest-xdist
honors groups only under `--dist loadgroup` (`xdist/remote.py`). `scripts/native_tests.py` adds
`loadgroup` only on its non-managed route, and the managed route runs `-n 0`. A bare `pytest`, which
AGENTS.md treats as first-class, therefore runs `scientific-owner` cases sixteen-wide.
Shared cause with [DTA-016](#dta-016).

## <a id="dta-016"></a>DTA-016 — Scheduling requirements depend on names without complete correspondence checks

**Refresh:** closure overstated; the structural cause persists in a new form. The contract is
§24.1's "Selected requirements also derive scheduling and pre-effect admission … Timing/native/thread
constraints remain where actually demanded." Renamed canonical tests are caught
(`test_resources.require_effect` → `test_requirements.guard`), but:

- Native demand is keyed by module-path prefixes in `test_requirements.RUST_OWNERS`, and unmatched
  tests default to `LOCAL`. Narrowing `kinsol::`/`ipopt::` to `kinsol::tests`, `ipopt::tests` for
  DTA-045 also de-grouped real native-solve modules such as `ipopt::sequence` (FFI sequence solves)
  and several KINSOL profile/policy modules.
- `rust_catalog(validate_declarations)` checks the two `RUST_FIXTURES`, not `RUST_OWNERS` prefixes;
  Rust has no before-effects guard for native work, so classification fails open. The
  `RUST_OWNERS` entry `"canonical_study_module_tests::"` matches no test path (the file is
  `include!`d under `canonical_studies_server_unit`), direct evidence that prefixes are never checked.
- `.config/nextest.toml` separately classifies `^(kinsol|ipopt|…)::` for timeouts; the two lists
  already differ.
- Exclusivity is granted only to the hand-listed `external-restart` peer. Tests that saturate a
  server-wide Surrealism controller budget (`canonical_studies_surrealism_qualification_global_capacity_survives_cache_eviction`)
  share the store group with fifteen others, risking spurious capacity refusals elsewhere.
- Python demand is inferred from file names (`PYTHON_DURABLE_OWNERS`); only one test uses the
  explicit requirement marker. Moving a canonical test to a new file changes its classification.

The `observed != set(requirements)` comparison in `nextest_configuration` compares the catalogue with
itself. Static first-match nextest overrides could carry the same meaning to every route; exact
applicability on the pinned nextest is unverified.

## <a id="dta-017"></a>DTA-017 — A stale validation declaration is checked while some routes bypass actual leaf verification

**Refresh:** partially resolved. Stale metadata is gone and installer/codegen/managed builds prove
leaves via `arrow_validation.resolve`. `scripts/scientific_inputs.py::capture` still runs a bare
`cargo run -p xtask --features physical-admission,force-validate` without leaf proof, on the focused
path for consumer packages and in `affected` preview.

## <a id="dta-012"></a>DTA-012 — Managed command exit and reconciled assurance verdict can disagree

**Refresh:** closure established. `scripts/test_run.py::execute_selected` returns zero only for a frozen
invocation with complete terminal and `pass` assurance; `validation.compose_selection` reconciles
unreported tests.

## <a id="dta-013"></a>DTA-013 — Some controls are inactive or outside declared campaigns; manual routes are not absent

**Refresh:** closure established for the named population (`pse-py` dormant bodies removed; the
restart peer runs through `scripts/tests/canonical_boundary_run.py`). The general cause recurs
elsewhere as [DTA-050](#dta-050).

## <a id="dta-014"></a>DTA-014 — Optional CLI qualification has no separate executed outcome in enclosing Rust API tests

**Refresh:** closure established (`test_requirements.RUST_PEERS` roles; `native_tests.prepare_rust_fixtures`
refuses a missing `PSE_CANONICAL_TOOLS`).

## <a id="dta-018"></a>DTA-018 — moved

Primary home: [execution efficiency](execution-efficiency.md#dta-018).

## <a id="dta-019"></a>DTA-019 — Some shared test doubles and declared dev dependencies have no external consumer

**Refresh:** closure established. `crates/pse-testkit/src/lib.rs` retains `factory`, `NativeFixture`
and `capture`, each with real consumers.

## <a id="dta-025"></a>DTA-025 — Fixture infrastructure failure is reported through Rust test failure and can fan out

**Refresh:** closure established for fixture setup (`pse-operations/src/testing.rs::fixture_setup_failure_payload`
→ `test_resources.observe_setup_failure`; `test_run.finish` projects `failure_kind`/`assertion_reach`).
Preconditions outside the fixture owner still surface as product panics; see [DTA-050](#dta-050).
