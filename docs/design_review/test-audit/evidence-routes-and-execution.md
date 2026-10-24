# Evidence routes, isolation and execution

Findings about how tests are selected, gated, isolated and executed, and whether the reported
outcome represents the evidence actually produced. Baseline and limits are in the
[collection README](README.md). Citations are `path:line` at source commit `cbda2efc6`.

## <a id="dta-012"></a>DTA-012 — The managed test route exits with the test tool's status while its reconciled verdict can say failed

**Principles:** AP-05, DP-21; companion §6.

**Claim at risk.** Section 24.1 claims that "one terminal composer reconciles actual selection,
failure, cancellation and missing evidence". The qualification guide adds that a missing
terminal or a selected-but-unreported test cannot become a pass.

**Diagnosis.**
- `execute_selected` (`scripts/test_run.py:152-410`) builds a reconciled verdict. It marks the
  check `failed` for:
  - report errors;
  - selected tests with no result (`not_run`);
  - duplicate or unexpected identities;
  - non-passed Python statuses;
  - fixture cleanup failures (`finish`, `test_run.py:59-149`;
    `scripts/validation_receipts.py:263-291`).
- It then returns the child's exit `code` (`test_run.py:272,410`). Only a signal changes that
  code.
- `scripts/select.py:143,154`, `scripts/python_tests.py:100-106` and
  `scripts/native_tests.py:1413` pass it through unchanged.
- The behavior is pinned by `test_actual_child_status_survives_missing_report_reconciliation`
  (`scripts/tests/test_test_run.py:48-71`). It asserts exit 0 while `terminal["complete"]` is
  false.

**Consequence (conceals missing evidence).** Anyone reading the exit status of
`just unit-package`, `just native-test` or `just py-test` sees success when the report is
missing or stale, a selected test has no result, or fixture cleanup failed. This includes the
agent inner loop, which AGENTS.md routes through `unit-package`. The truthful verdict is only
in three places:
- the `<invocation>.terminal.json` file;
- a `test-resources:` stderr line;
- the output of `just result`.

**Counterevidence.** `.claude/rules/tooling.md` asks commands to keep their own exit status and
signals intact, so this is a deliberate policy. The terminal evidence itself is truthful and
cannot be overwritten by a later report. The finding records the gap between that policy and
the reconciliation claim; it does not select between them.

## <a id="dta-013"></a>DTA-013 — Some committed tests and evidence controls have no route that runs them

**Principles:** AP-05, DP-23, DP-24; companion §4, §6.

**Diagnosis.** Several things exclude evidence from every committed route: manifest flags,
feature gates, `#[ignore]`, discovery rules and missing drivers. No owner reconciles declared
tests with routes.

*Established manifestations:*
- **pse-py tests never compile.** `crates/pse-py/Cargo.toml:28-29` sets `test = false` and
  `doctest = false`. As a result:
  - two `#[test]`s in `crates/pse-py/src/inspection/errors.rs:539,555` are never compiled or
    run, even under `--all-targets`;
  - the file was last edited in `4de4c68d6` (2026-10-22).
- **`compiled-evaluator` tests have no test route.** About 38 `cfg(feature = "compiled-evaluator")`
  sites in `crates/pse-math/src` gate tests and a probe module.
  - The feature is not part of `native-solvers` (`crates/pse-runtime/Cargo.toml:46-58`).
  - No manifest dependency or test recipe enables it; `scripts/native_tests.py:158-166` adds
    only `native-solvers`, `canonical-tests` and `native-acceptance`.
  - The only switch is the bench smoke route (`scripts/case_measure.py:469`), while the
    evaluator remains user-selectable.
- **`native-profiles` link and ABI controls are never selected.** The feature is defined in
  `tests/conformance/Cargo.toml:33` and gates code via `tests/conformance/src/lib.rs:6`, but no
  recipe or script enables it. The `cargo hack` powerset checks without `--tests`.
- **The restart-recovery test has no driver.**
  `retained_canonical_store_recovers_a_new_generation_after_external_server_restart`
  (`crates/pse-operations/src/canonical_execution.rs:1963+`) is `#[ignore]`. It waits for an
  external lifecycle to publish `stopped.json`/`restarted.json`, and no committed driver does
  so. It covers a consequential durability property.
- **Discovery by text misses pytest-only modules.** `scripts/setup_report.py:31-38` discovers
  `scripts/tests/test_*.py` only if the file text contains `unittest.TestCase`. That excludes
  the pytest-only `scripts/tests/test_plan30_context_overlap.py`, which is also outside pytest
  `testpaths` (`pyproject.toml`). Its only driver, `plan30_context_overlap_run.py`, is
  referenced nowhere. It is paired with the ignored Rust test at
  `crates/pse-operations/src/testing.rs:500`.
- **Unreferenced control modules.** These are referenced by no recipe, script, workflow,
  development guide or importer, though several were edited 2026-10-22/23:
  - in `scripts/tests/`: `active_diagnostic_control.py`, `active_mixed_control.py`,
    `grpc_frame_relay.py`, `plan50_database_fixture.py`, `plan50_shell_bridge_check.py`,
    `surreal_logging_check.py`, `surreal_otlp_check.py` and
    `surreal_rollout_producer_fixture_check.py`;
  - `python/pse/tests/canonical_signal_control.py`.

**Consequence.** These tests and controls rot without any execution signal. Their past results
cannot be reproduced without knowledge that is not recorded in the repository. Readers of the
suite overestimate what a full assessment exercises.

**Counterevidence.**
- `just hygiene` type-checks `scripts/` through pyrefly.
- Some ignored tests are declared on-demand measurements with explicit routes:
  - `crates/pse-backend-native/src/measurement_tests.rs:270,400`;
  - `crates/pse-runtime/src/math/portable.rs:2599`, routed by `scripts/validation_scope.py:241`.
- `managed_primary_*` tests, filtered out by default (`.config/nextest.toml:20`), have a route
  (`scripts/native_tests.py:951-956`).
- Retired plans, excluded from this audit's inputs, may record manual invocations.

**Related.** [DTA-014](#dta-014) covers evidence that runs but silently does nothing.

## <a id="dta-014"></a>DTA-014 — Qualification steps inside ordinary tests, and several controls, return or skip silently when their environment is absent

**Principles:** AP-05, DP-23; companion §6.

**Diagnosis.**
- `qualify_operator` (`crates/pse-operations/src/canonical_explanations.rs:674-684`) returns
  without effect unless both `PSE_DIAGNOSTIC_EVIDENCE` and `PSE_CANONICAL_TOOLS` are set.
  - Its callers are ordinary canonical tests (`canonical_explanations.rs:1542`;
    `canonical_studies.rs:2971`). More env-conditional evidence writes are at
    `canonical_explanations.rs:1225,1543` and `canonical_studies.rs:2953`.
  - The only setters of `PSE_CANONICAL_TOOLS` are the unreferenced
    `scripts/tests/plan50_database_fixture.py:42` and `scripts/tests/canonical_inspection_check.py`.
- Several manual controls use `unittest.skipUnless` for their selection, so a bare run reports
  OK with skips:
  - `scripts/tests/plan30_capacity_modes_check.py:24`;
  - `plan30_placement_check.py:155`;
  - `plan30_resources_actual_check.py:126,248`.
- Other controls state that a missing selection fails and never skips (`evidence_native.py`).
  `setup_report.py` treats skips as failure, but these modules are outside its discovery.

**Consequence (missing evidence reads as success).**
- On every recipe route, the tests that call `qualify_operator` pass with the operator
  qualification step skipped.
- A test name suggests operator-path coverage that is not exercised.

**Counterevidence.** Python product tests contain no `skip`, `skipif` or `xfail`. Parity fails
rather than skips. Child-process tests gated by an environment variable have spawning parents
(e.g. `crates/pse-math/src/guarded_tests.rs:310`), so they are not routeless. They do add a
vacuous pass to ordinary counts.

## <a id="dta-015"></a>DTA-015 — Resource association, Python grouping and the stack policy exist only on managed routes; bare, raw and `affected --run` executions silently lose them

**Principles:** AP-05, AP-06, AP-07; companion §6, §8.

**Claim at risk.** AGENTS.md calls bare tools first-class, with routes carrying "the same
obligation". Section 24.1 says tests own their isolated databases and mutable state.

**Diagnosis.**
- *Resources stay pinned.* Every canonical fixture registers with `disposition: "incomplete"`
  and `pin: True` (`scripts/test_resources.py:543-544`). Only `finalize`, which needs an
  invocation declared through `PSE_TEST_INVOCATION`, can unpin a passed resource
  (`test_resources.py:631-679`). Raw recipes, `just affected --run`, bare `cargo nextest` and
  bare `pytest` therefore leave every canonical database retained on the shared default store
  until someone releases it by hand. Raw recipes here include `test`, `test-package` and
  `functional-test-target` with `canonical-tests`.
- *Python grouping is lost.* `xdist_group("canonical-owner")` (`conftest.py:283-285`) takes
  effect only with `--dist loadgroup`, which only `native_tests.python_command` injects
  (`scripts/native_tests.py:562-571`). Bare `pytest` uses the `addopts` default
  `-n 16 --dist=worksteal` (`pyproject.toml`), so canonical component tests spread across 16
  workers. The reason for the group is not recorded in code.
- *The stack policy is lost.* `just affected --run` replaces itself with
  `scripts/pse-env -- cargo nextest …` (`scripts/affected.py:212-216`). That bypasses
  `arrow_validation.rust_test_environment`, the only place `RUST_MIN_STACK` is set
  (`scripts/arrow_validation.py:30-50`). Commit `b89d08c9f` records native runtime tests that
  needed the configured stack.
- *An exclusion goes unreported.* The `affected` exclusion report
  (`scripts/affected.py:162-184`) names native features, Python, generators, unowned files
  and dynamic reach. It does not name `canonical-tests`, which gates about 270 `cfg` sites in
  62 files, so a change to canonical store code previews and runs with those tests compiled
  out and no notice.

**Consequence.**
- Retained databases accumulate silently.
- Bare and `affected --run` runs differ from recipe runs in concurrency and stack size, so a
  failure or pass may not reproduce across routes.
- A developer can believe store changes were exercised when they were not.

**Counterevidence.**
- Conservative retention is a stated design: unknown resources stay pinned until explicit
  release.
- A missing `PSE_SURREAL_STATE` fails rather than skips.
- `affected` labels itself "dependency reach only".

**Uncertainty.** Two things are suspected but not established:
- start/unpark races among concurrent bare-pytest workers;
- per-worker session runtime memory.

**Related.** [DA-021](../audit/local-reasoning-and-assurance.md#da-021) is a separate,
production-structure finding; it is not reexamined here.

## <a id="dta-016"></a>DTA-016 — Exclusive scheduling for server-wide and timing-sensitive store tests relies on hand-kept exact-name filters with no stale-name detection

**Principles:** AP-05, AP-06; companion §6.

**Diagnosis.**
- `.config/nextest.toml` lists tests that need exclusive use of the shared server or sub-second
  timing as exact `test(=…)` terms (first override, about lines 45-73).
- Unlisted canonical tests get `threads-required = 1` and share the server with up to 15
  others. New `workflow::` and `math::` modules default into a shared group by path.
- nextest does not report filter terms that match nothing.
- The only check, `scripts/tests/test_plan30_routing.py:550-562`, asserts that one name is
  present.
- A static check at the baseline found 1 of 16 exact terms with no matching function:
  `workflow::tests::durability_selection_validates_positive_lease_and_heartbeat_intervals`
  (`.config/nextest.toml:116`). The check extracted every `test(=…)` term and grepped each
  final path segment as a `fn` in `crates/`, `tests/`, `xtask/` and `benches/`.

**Consequence.** Renaming an exclusive test silently moves it into shared execution, where
timing-sensitive assertions can become intermittent.

**Footprint.** *Suspected:* the qualification-pool timing test
`canonical_studies_surrealism_qualification_pool_original_expiry_cannot_start_late`
(`crates/pse-operations/src/canonical_study_module_tests.rs:254-310`). It fills a pool of 4,
relies on a 50 ms sleep and a 100 ms deadline, and is not on the list. Whether that pool is
server-wide or per database was not established.

**Counterevidence.** The `store` group cap of 16 equals the test-thread count, so misclassifying
a test into `store` costs nothing. Each entry on the exclusive list carries an explanatory
comment.

## <a id="dta-017"></a>DTA-017 — The superseded universal force-validate contract survives in declarations, a governance test, CI and a runner constant, while some routes check only the feature name

**Principles:** AP-04, AP-05, DP-01; companion §8.

**Claim at risk.** One authoritative declaration of how correctness routes activate Arrow
validation (ADR-0170: scope-aware, leaf-verified).

**Diagnosis.**
- *Stale declarations.*
  - `Cargo.toml:493-494` declares `validate-features = ["pse-relations/force-validate"]`,
    "passed to every test invocation by the justfile and CI". No runner reads it.
  - `tests/governance/tests/every_crate_registered.rs:163-198` checks only that the list is
    non-empty and names a real feature. Its message says "nothing would switch on Arrow's
    force_validate in CI".
  - The `crates/pse-relations/Cargo.toml` comment still describes the universal mandate.
- *Hard-coded flag.* CI workflows (`.github/workflows/rust.yml:101-109`,
  `rust-scheduled.yml:46,88-92`) and `scripts/native_tests.py:33` hard-code the bare relation
  flag.
- *Name-only routes.* The maturin builds (`scripts/python_extension.py:86-99`; the `parity`
  and `py-measure-force` recipes) and the codegen `cargo run --features …,force-validate`
  recipes do not go through `scripts/arrow_validation.py`.
- *Python gate.* The gate checks only pse-py's own `cfg!(feature = "force-validate")`
  (`crates/pse-py/src/lib.rs:66-70`; `python/pse/tests/conftest.py:52-58`).

**Consequence.**
- Two declarations of the obligation coexist, and a governance test protects the one nothing
  consumes.
- For the name-only routes, leaf activation rests on workspace feature unification rather than
  the composer's verification.

**Bounds.** No Arrow-consuming closure was found running without validation. On workspace scopes
the relation feature reaches every Arrow leaf:
- `arrow/force_validate` reaches `arrow-data`;
- pse-columnar and pse-schema add `arrow-cast/force_validate`;
- `.cargo/config.toml` sets workspace feature unification;
- no workspace build dependency pulls Arrow.

A composer resolution for the maturin closure
(`python -m scripts.arrow_validation build -p pse-py --lib --features force-validate`) would
settle the remaining route.

## <a id="dta-018"></a>DTA-018 — Each Rust canonical test pays a Python bridge round trip and a full schema installation, independent of what it checks

**Principles:** AP-06, AP-07, DP-16; companion §8. **Magnitude unmeasured.**

**Diagnosis.** nextest runs every test in its own process. Each canonical fixture then does
three things:
- *Python bridge.* It spawns `.venv/bin/python -m scripts.test_resources register`, and later
  `drain`, through `resource_bridge` (`crates/pse-operations/src/testing.rs:140-219`). That
  loads `test_resources` and its service-admission imports.
- *Schema installation.* It installs the complete generated schema into a fresh database
  (`testing.rs:348-399`). `crates/pse-operations/src/generated/surreal.surql` is 340,507 bytes
  with 989 `DEFINE` statements.
- *Checkout coupling.* Rust tests take a hard dependency on the checkout's `.venv`.

The managed route adds per-invocation work before the test body:
- `cargo metadata`, plus one or two `--unit-graph` resolutions;
- `nextest list`, then `nextest run` (`scripts/test_run.py:516-540`);
- invocation and config copies.

`py-unit` and `py-test` run `maturin develop` first.

**Consequence.** The cost of feedback for a store-touching responsibility scales with the number
of selected tests times the fixed fixture cost. This is the case whether or not a test's claim
concerns schema installation or registry state. The resources are not unrelated: a fresh
database per test is a deliberate isolation premise.

**Counterevidence.** The machinery buys three things nextest and pytest alone do not:
- retained databases for failed tests;
- drained native descendants and cgroups;
- `not_run` accounting.

[DTA-012](#dta-012) weakens that last benefit at the exit-status level. Some tests target
schema installation itself.

**Uncertainty.** No measurement was taken. Whether the per-test bridge and schema cost is
material relative to test bodies is unknown.

## <a id="dta-019"></a>DTA-019 — Shared fault and counting doubles in pse-testkit have had no consumers since 2026-10-06, and test dependencies are declared beyond their use

**Principles:** DP-16, DP-23, AP-01; companion §2, §3.

**Diagnosis.**
- *Unused doubles.* `FaultStore`/`FaultPlan` (`crates/pse-testkit/src/fault_store.rs`, 228 lines)
  and `CountingStore` (`counting_store.rs`, 186 lines) are referenced only by pse-testkit's own
  self-test (`crates/pse-testkit/src/tests.rs:605`). Their last consumers were removed in
  `06302af77` (2026-10-06). `execution.rs` and `introspection.rs` have no external users
  either.
- *What is used.* Only `factory`, `NativeFixture` and `capture` are imported elsewhere.
- *Declarations without use.*
  - `crates/pse-runtime`, `tests/lifecycle` and `tests/conformance` declare pse-testkit as a
    dev-dependency with no `pse_testkit` references. The declarations mainly forward
    `pse-testkit/force-validate`.
  - `proptest` is declared and unused in `crates/pse-columnar` and `tests/conformance`.

**Consequence.**
- About 400 lines of consequential doubles are maintained and self-tested while protecting no
  claim, contrary to the repository's delete-with-replacement rule.
- Readers of six manifests overestimate shared fault coverage of persistence paths.

**Counterevidence.** The doubles are carefully specified and have known-behavior tests. An
object-store path may exist that should consume them and is under-tested; that was not
established.

**Related.** [DA-024](../audit/local-reasoning-and-assurance.md#da-024) concerns bespoke machinery without a
production consumer. It was not reexamined, and this finding does not refresh it.
