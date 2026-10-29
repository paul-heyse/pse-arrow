# Test execution system

**Interface-checked · refreshed 2026-10-29 · source `7bd341f0bd4a27ad629008abdff56ee2bcf7e9f3` plus
the working tree.** Primary home for the coexistence of route families during the Plan58 cutover
([DTA-063](#dta-063), [DTA-064](#dta-064)) and closure records for requirement interpretation and composition. Other route
causes live in [DTA-015/016](evidence-routes-and-execution.md#dta-016) and
[DTA-050](evidence-routes-and-execution.md#dta-050). Prior diagnoses are preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/test-execution-system.md);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition of
DTA-001 to DTA-058; DTA-063 and DTA-064 have no disposition owner yet.

## <a id="dta-063"></a>DTA-063 — Coexisting route families hold divergent execution meaning during the cutover

**Principles:** AP-04, AP-02, AP-05, DP-01, DP-22, DP-24; companion §6.
**Status:** new, 2026-10-29. Lineage: the native-era counterpart of the single interpretation that
closed [DTA-022](#dta-022); 58f F02/F05 schedule removal of the Python half.

**Claim at risk.** A Rust test run means the same thing (validation closure, scheduling, fixture
authority, result) whichever sanctioned recipe starts it, and AGENTS.md's invariant that every
Arrow-consuming correctness closure activates validation has one implementation.

**Structural cause.** Three Rust test route families are live at this source, each holding its own
meaning:

| Family | Recipes | Arrow closure | Groups | Fixtures and result |
|---|---|---|---|---|
| Native adapter | `test`, `test-package`, `unit-package`, `unit-native-capability-package`, `worker-test`, `cargo t` and most other test, check and clippy recipes (69 adapter invocations in the justfile) | `crates/pse-host/src/correctness.rs:258-347` | none assigned | native private fixtures; nextest exit |
| Python runner | `unit-native-package`, `unit-native-selected` (`justfile:1426-1440`), runnable only after explicit full-profile (`solver,klu,isolation,uno,petsc`) preparation | `scripts/arrow_validation.py:356-420` | projected per run | Python resource registry; reconciled verdict |
| Plain nextest | `affected --run` | Python-composed features | none | nextest exit |

The two closure implementations already disagree on accepted command modes: the native adapter refuses
`miri` and `llvm-cov` (`correctness.rs:135-178`), so `scientific-replay-miri` always refuses and
`coverage` stays on the Python composer. The Python resolver also serves `python_extension` (maturin),
`coverage_correspondence`, `cargo_feature_check`, `evidence_store`, `boundary_fuzz` and `affected`.
The native route launches `cargo-nextest` from `PATH` (`correctness.rs:535`) without the version and
digest check the Python route applies (`scripts/cargo_tools.py:39-52`), while its setup script relies on
nextest features that are experimental at the pinned 0.9.146 (58f). The same scientific claims also run
on both sides: the four `native_local_study` tests mirror `test_studies.py:440-658`, and
`native_conic`/`native_strategies` overlap `test_native_workflow.py`.

The same coexistence holds for other policies with live Python and Rust implementations: environment composition (`crates/pse-host/src/environment.rs:309-475` and `scripts/pse_env.py:172`, still called from `evidence_store`, `installation_context`, `preflight` and `python_extension`) and the `surreal-source-release-v2` receipt (`scripts/surreal_source.py:89`, `pse-host/src/server/release_import.rs:132`). Several environment rules are tested only on the Python side (target-directory selection, removal of inherited nextest thread counts, `CI` disabling caching, solver/LLVM `PATH` prepends, invalid `PSE_BUILD_MODE`), and no test checks that the two compositions agree.

**Consequence.** Two near-identically named recipes, `unit-native-capability-package` and
`unit-native-package`, differ in closure owner, scheduling, fixture registry and verdict. A correction or
new mode in one closure implementation is invisible to the other, and each has its own focused tests
(`crates/pse-host/tests/correctness.rs`; the `scripts/tests` arrow-validation tests). A `PATH` change can
alter runner semantics on the native route without refusal. Duplicated Python claims cost runs and
maintenance without adding a distinct challenge where they exercise only local science.

**Evidence and uncertainty.** Established for the route map and both implementations. That a
`PATH`-resolved nextest differs in practice is suspected, not observed; `bootstrap-rust-tools` installs
the pinned version. Whether the duplicated Python study tests still add durable-route value is open,
and they are unrunnable while `--store` refuses. **Counterevidence:** this is an explicit, bounded
transition; `arrow_validation.py:451-458` labels its retired CLI, and 58f states that unmigrated frontend
builders remain outside accepted native scope.

## <a id="dta-064"></a>DTA-064 — Routes for surviving evidence are composed of the machinery scheduled for deletion

**Principles:** AP-01, AP-06, AP-07, DP-17; companion §5, §8.
**Status:** new, 2026-10-29. Lineage: the collection-time coupling to the supervisor is the coupling
[DTA-026](#dta-026) and [DA-037](../audit/development-tooling.md#da-037) own; what is new is the deletion
schedule that turns that coupling into a launch dependency for surviving evidence.

**Claim at risk.** Tests whose subjects remain valuable (docs and governance tooling, oracle and
scientific-input controls, pure product units until G03) keep running while 58f F02 and 58b/58d/58g
delete the Python host and runner.

**Structural cause.** The routes that reach them are compositions of the retiring owners:
`just setup-test` discovers `scripts/tests/test_*.py` but executes through `test_requirements`,
`test_resources` and `test_run` (`scripts/setup_report.py:13`); `py-unit` runs through
`python_extension`, `native_tests` and `test_run` (`scripts/python_tests.py:10-11`) and republishes the
extension on every run; the product `conftest.py:22` imports `surreal_server`, `test_requirements` and
`test_resources` at collection, even for pure unit modules; `unit-native-package` runs through
`select.py` → `test_run.run_rust` and `arrow_validation`. The native branch of `py-unit` and the
`unit-native-*` recipes run under the full native profile, which refuses until explicitly prepared
(the live-qualified profile is `compiler,klu`), so today these routes are conditionally runnable.

**Consequence.** Deleting `test_requirements.py` or `surreal_server.py`, as F02 schedules, breaks launch
or collection for about 126 tooling modules and about 179 Python unit tests whose subjects survive. Each
retirement step must either rebuild unrelated routes or keep the retiring module alive, and product-unit
evidence depends on the import health of the service supervisor.

**Evidence and uncertainty.** Established for `setup-test` (about 126 `test_*.py` modules among 173
Python files in `scripts/tests`), `py-unit` and `unit-native-package`. The
cutover sets may plan the route rebuild; the plan text names callers and tests but not these routes.
**Counterevidence:** `docs-test` and `surreal-test` are plain `unittest` invocations already independent
of the managed runner. Deletions so far carried their tests: every first-party import in the three test
trees resolves.

## Refresh of earlier records

| Finding | Outcome | Current residual and decisive evidence |
|---|---|---|
| [DTA-022](#dta-022) | Established on the Python route; inherited divergence | Native relies on nextest's own grammar plus a narrow mode allow-list and reuse deny-list; the two route grammars now differ ([DTA-063](#dta-063)). |
| [DTA-024](#dta-024) | Partially reopened on the native route | Test-tool identity is verified only on the Python route ([DTA-063](#dta-063)). |
| [DTA-023](#dta-023) | Partial, unchanged | The dead recipe filter persists (`justfile:253`). |
| [DTA-026](#dta-026) | Established (scoped) | Unchanged; DA-037 owns the shared tooling coupling. |

## <a id="dta-022"></a>DTA-022 — Runner consumers independently interpret selection, profiles and fixture identity

**Refresh:** closure established. `scripts/test_invocation.py::{rust, python}` is the single
interpretation; its `report_configuration` serves both `rust_report_command` and
`validation.native_report_config`. Rust `selected_fixture_identity` and Python `selected_test`
deliberately authenticate each other.

## <a id="dta-023"></a>DTA-023 — Preparation and qualification selections require name-keyed maintenance

**Refresh:** partially resolved. `validate_roles` catches a lost role inside a nonempty selection.
The remaining name-keyed maintenance has primary homes elsewhere: file-name capsule relevance in
[DTA-027](doubles-and-coupling.md#dta-027), and a dead recipe filter (despite the disposition's
"obsolete references removed") in [DTA-050](evidence-routes-and-execution.md#dta-050).

## <a id="dta-024"></a>DTA-024 — Reuse identities omit consequential generator inputs and test-tool identity

**Refresh:** closure established. `cargo_tools.selected_identity` (version and digest) is rechecked in
`native_tests.prepared_rust_inventory`; `validation_scope.INPUT_SCOPES["generation"]` includes packages.

## <a id="dta-026"></a>DTA-026 — Reciprocal lifecycle dependencies and several roots obstruct local test-system reasoning

**Refresh:** closure established for its scoped target. Duplicated meaning moved into
`test_invocation`; performing cycles remain as lazy imports (`native_tests._rust_capture` imports
`test_run`; about twenty-five lazy imports in `native_tests`, twenty in `test_resources`). Plan 53 did
not target module-count cleanup, so this is not reopened. The broad account's
[DA-037](../audit/development-tooling.md#da-037) owns the wider tooling cycle.
