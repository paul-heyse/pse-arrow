# Test execution system architecture

Findings about the test execution machinery as software: its module structure, its
composition roots, where test requirements are declared, and how it depends on the test tools
it drives. Baseline and limits are in the [collection README](README.md). Citations are
`path:line` at source commit `cbda2efc6`. All findings on this page come from the second,
principle-directed pass on 2026-10-23.

## <a id="dta-022"></a>DTA-022 — The runner re-derives nextest's resolved state from command lines and TOML text, and each site covers a different subset

**Principles:** DP-13, DP-14, DP-01, AP-05, DP-21; companion §6, §8.

**Claim at risk.** Section 24.1 claims that "one terminal composer reconciles actual selection,
failure, cancellation and missing evidence". The tooling rule says recipes stay thin and leave
selection to nextest.

**Diagnosis.** No stage takes the resolved profile, options or test identity from nextest. Each
stage re-parses what it needs, in four places.

*Profile.*
- `rust_report_command` (`scripts/test_run.py:587-594`) recognizes only `--profile` and
  `--profile=`.
- nextest also accepts `-P` and the `NEXTEST_PROFILE` environment variable, and
  `scripts/arrow_validation.py:143` accepts `-P`.
- With `-P local`, the runner rewrites the default profile's JUnit path while nextest writes
  `target/nextest/local/junit.xml`. The invocation's report then never appears, every selected
  test becomes `not_run` and every fixture stays pinned. Because of
  [DTA-012](evidence-routes-and-execution.md#dta-012), the command still exits 0.

*Configuration rewriting.* Two regular-expression TOML rewriters do the same job differently
(`test_run.py:587-621`; `scripts/validation.py:451-467`). One appends a table when the pattern
misses; the other raises unless it matches exactly once.

*Option grammars.* Several independent grammars disagree:
- `arrow_validation.py:94-178` lacks `--filterset`, nextest's primary long name.
- `scripts/native_tests.py:923-950` and `:858-907` accept different option sets.
- `native_tests.py:910-920` accepts `-r`, which composition refuses.

So valid nextest syntax is accepted at one stage and refused at another.

*Test identity.*
- The Rust fixture reads its own process arguments for `--exact`
  (`crates/pse-operations/src/testing.rs:221-240`).
- The registry reads `/proc/<ppid>/cmdline` and `/proc/<ppid>/exe`
  (`scripts/test_resources.py:423-456`).
- nextest exports `NEXTEST_TEST_NAME`, `NEXTEST_BINARY_ID` and `NEXTEST_RUN_ID` to every test
  process. None is used in the repository.

**Consequence.**
- Missing evidence and pinned resources are hidden in the profile case.
- Valid nextest syntax behaves differently by route.
- A nextest upgrade can change one parsed surface and not the others ([DTA-024](#dta-024)).

**Counterevidence.**
- Composition fails closed on unknown options.
- nextest's per-test JSON output is experimental, so isolating a JUnit path per invocation needs
  a configuration overlay.
- The `/proc` check binds the executable path, which environment variables alone do not.

**Probe.** Run `just unit-package pse-ids <word> -P local`, then inspect the terminal file and the
exit status.

## <a id="dta-023"></a>DTA-023 — Test execution requirements and qualification selections live in name-keyed runner tables away from the tests, and drift is caught inconsistently

**Principles:** DP-01, DP-16, DP-22, DP-24, AP-05; companion §6; the binding's
delete-replaced-code rule.

**Claim at risk.**
- Section 24.1: "Test selection belongs to nextest and pytest; no exact-name acceptance manifest
  exists."
- DP-24: the extension path for a new test is discoverable.

**Diagnosis.** A test's needs are declared in runner tables keyed by exact names, substrings or
module lists, with no check against the actual test inventory. Those needs include its feature
gate, exclusivity, prepared fixture, qualification gate and focused recipe.

*Dead recipe.*
- `python-runner-unit-test` (`justfile:227-231`) selects `test(python_selection_)` in xtask.
- No such test exists at baseline; it was removed with `xtask/src/python_tests.rs` in
  `ff5a4aa2f` (2026-10-11).
- The recipe now exits with nextest's no-tests status, while its description still advertises
  "Focused explicit Python target selection controls".

*A qualification control can be lost silently.*
- The `feature-absence` gate (`scripts/validation_scope.py:278-297`) selects
  `test(feature_absence::) | test(clarabel_tests::clarabel_mkl_pardiso_refused_without_profile)`.
- That control only compiles without `clarabel-pardiso`
  (`crates/pse-backend-native/src/clarabel_tests.rs:577-579`), so this gate is its only route.
- If the control is renamed, the union still matches other tests, nothing errors, and the
  control quietly disappears from qualification.

*Fixture preparation by exact name.* `scripts/native_tests.py:678-693` ties two prepared-fixture
variables to two exact test paths. A rename fails loudly, but bare and `unit-package` runs always
fail those two tests.

*Exact-name gates.* Gates at `validation_scope.py:228` and `:244` fail loudly on rename, but they
contradict the §24.1 claim.

*No map for new tests.* Nothing tells an author which place a new test's requirement belongs
in. The choices are:
- `canonical-tests` cfg plus `*_server_unit` naming;
- the exclusive list;
- `RUST_FIXTURE_TESTS`;
- a `validation_scope` gate;
- a recipe filter.

**Consequence.**
- A lost qualification control can go unnoticed.
- A misleading command remains advertised.
- Maintenance grows with the number of tests.

[DTA-016](evidence-routes-and-execution.md#dta-016) is the scheduling manifestation of this
cause and keeps its own identity.

**Suspected.** Two Ipopt- and POUNCE-backed sensitivity tests sit outside the `scientific`
override's module list (`crates/pse-backend-native/src/kkt/sensitivity/tests.rs:794-799,912-916`).
Whether their solver pools matter was not established.

**Counterevidence.**
- Python declares requirements at the test, through markers and fixture-driven `xdist_group`,
  which survives renames.
- Most tables fail loudly.
- Every other recipe filter and hand-listed module matched a current test at baseline.

## <a id="dta-024"></a>DTA-024 — Cargo test tools float while runner code depends on their version-specific behavior, and evidence reuse does not key on tool versions

**Principles:** DP-15, DP-09, DP-01; companion §6, §8.

**Diagnosis.**
- `scripts/bootstrap.sh:26-35` installs the current release of each cargo tool ("the tool set,
  not its versions").
- At baseline:
  - the installed `cargo-nextest` is 0.9.146, while `docker/solvers/Dockerfile:190` names
    0.9.143;
  - `.config/nextest.toml` sets no `nextest-version` requirement;
  - `cargo-insta` 1.48.0 is installed against `insta = "=1.49.0"`.
- `scripts/cargo_feature_check.py:5-8` depends on "cargo-hack 0.6.45 selects
  CARGO_HACK_CARGO_SRC before CARGO", a source-level behavior the cargo-hack README does not
  document.
- The runner parses nextest's list JSON and option grammar ([DTA-022](#dta-022)).
- The evidence-reuse key covers files, the environment and `rustc -Vv`, but no test-tool
  version (`scripts/validation_scope.py:463-500`; `scripts/validation.py:66-103`).

**Consequence.**
- A tool upgrade can silently change selection, report or feature-check behavior.
- An "unchanged-input" reuse can carry an observation across that upgrade.
- If cargo-hack stopped honoring the variable, the feature-powerset check would quietly lose its
  adapter (suspected).

**Counterevidence.**
- The Python test tools are pinned exactly.
- `doctor` records tool versions.
- Reuse must be requested explicitly.
- The float is deliberate per the bootstrap comment.
- nextest supports `nextest-version = { required = … }`, so a minimum is expressible.

**Uncertainty.** Whether cargo tools count as "toolchain" under the repository's pin policy is
unresolved. The finding is the gap between version-specific dependence and an unbound version,
not the policy itself.

## <a id="dta-026"></a>DTA-026 — The runner, receipt, resource and server modules form one cyclic component with several composition roots for Rust test runs

**Principles:** AP-01, AP-03, AP-05, DP-17; companion §6, §8.

**Claim at risk.** A test run's selection, resource association, verdict and exit status each have
an identifiable owner that can change on its own.

**Diagnosis.** The runner and resource modules are one strongly connected import component. It is
held together by about 75 function-local imports annotated as owner, reciprocal-owner or service
cycles. The edges include:
- `test_run` ↔ `native_tests` (`scripts/test_run.py:527,553,632,750`; `scripts/native_tests.py:1283,1297,1399`);
- `test_run` ↔ `validation`;
- `native_tests` ↔ `validation`;
- `validation_receipts` ↔ `test_resources`;
- `surreal_server` ↔ `test_resources`:
  - the server supervisor imports the test registry at module top
    (`scripts/surreal_server.py:53-61`);
  - its stop/drain path reads untyped registry records;
  - `test_resources` imports `surreal_server` locally seven times.
- `host_admission` ↔ `native_operation` ↔ `pse_env`.

The component spans about 23,800 lines. Product tests depend on it: Python product tests import
it (`python/pse/tests/conftest.py:23`), and Rust fixtures run `scripts.test_resources` as a
subprocess. Rust runs have at least four composition roots:
- `arrow_validation.main`, used by raw recipes and the `cargo t` alias;
- `test_run.run_rust`, used by `select.py` and `native_tests`;
- `affected.py`;
- the `validation.py` groups.

**Consequence (change amplification; obstructs diagnosis).**
- Correcting [DTA-012](evidence-routes-and-execution.md#dta-012) touches `execute_selected`,
  every pass-through route and cleanup semantics owned by `test_resources`.
- A registry record change reaches the server supervisor.
- The route-dependent guarantees in [DTA-015](evidence-routes-and-execution.md#dta-015) follow
  from the multiple roots.

**Counterevidence.** Each cycle edge carries an owner rationale. Several reciprocal dependencies
reflect a genuinely shared lifecycle: the server must not stop while a fixture borrows it. About
20,000 lines of tests cover these scripts.

**Uncertainty.** How much of the component is test-specific versus general workload machinery is
not established.

**Related.** [DA-019](../audit/boundaries-and-composition.md#da-019) concerns Python tooling restating
Rust contracts, a different cause. It was not reexamined.
