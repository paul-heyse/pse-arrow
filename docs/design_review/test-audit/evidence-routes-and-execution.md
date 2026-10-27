# Evidence routes, activation and outcomes

**Interface-checked · 2026-10-27 · source `87f2eb9050ae24976956628c16e0e7376190be7a`.** This page
holds the primary findings on whether declared tests actually execute, under the prerequisites and
scheduling they need, and whether interleaving claims are forced. See the
[collection baseline](README.md#baseline-and-method) and [integrated account](architecture-and-assurance.md).
Prior diagnoses are preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/evidence-routes-and-execution.md);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition.

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
