# cargo-nextest

Realizations considered: installed `cargo-nextest` 0.9.146 (commit 8af696ddc, 2026-09-21),
reported by `scripts/pse-env -- cargo nextest --version`. It is a host tool, not a crate
dependency.
Research date 2026-10-27. Sources:
- Context7 `/websites/nexte_st`:
  - configuration reference;
  - setup-scripts and wrapper-scripts pages (both marked "Experimental", enabled through
    `experimental = ["setup-scripts" | "wrapper-scripts"]`, stable keys since 0.9.98);
  - changelog 0.9.131 (per-command `env`);
  - `threads-required` page;
  - tool-config-file docs;
  - nextest-runner source `config/overrides/group_membership.rs` (`first_override_wins`
    test), `config/elements/threads_required.rs`, `config/core/identifier.rs`,
    `errors.rs` (`InvalidTestGroupsDefinedByTool`) and `config/core/tool_config.rs` tests.
- The docs track current releases. Behavior at exactly 0.9.146 is assumed for items
  documented before that version.

## Types

### cargo-nextest/declarative-test-scheduling
- **Need it serves:** Give tests resource weights and mutual-exclusion groups by selection
  expression, so heavy or shared-resource tests are throttled without an external
  scheduler.
- **Recognizable code shapes:**
  - Tooling that computes per-test resource classes and serializes or limits them outside
    the runner.
  - Per-test generated `filter = "binary_id(..) & test(=..)"` stanzas.
- **Mechanisms and APIs:**
  - `[test-groups.<name>] max-threads = N`.
  - `[[profile.<p>.overrides]]` with `filter = '<filterset>'`, `test-group = "<name>"` and
    `threads-required = N | "num-cpus" | "num-test-threads"`.
    - Overrides also accept `priority`, `slow-timeout`, `retries`, `run-extra-args` and
      `default-filter` (the last requires `platform`).
  - Filterset DSL: `package()`, `binary_id()`, `test(/regex/ | =exact | substring)`,
    `rdeps()`, `deps()`, `kind()`, `platform()` and set operators.
  - Precedence: overrides are evaluated in order, and for each setting the first matching
    override that sets it wins (`first_override_wins`). Settings combine across overrides.
- **Obligations discharged:**
  - Scheduling under weights and groups.
  - Matching tests by package, binary or name pattern without enumerating them.
- **Residual caller obligations:**
  - Declaring the classes.
  - Ordering overrides most-specific first, because longest-prefix semantics must be
    expressed through ordering.
  - Any closed-world check that every selected test has an explicit class. nextest
    defaults unmatched tests to the profile settings.
- **Semantic conditions:**
  - Groups are per-run limits inside one nextest process. They do not coordinate across
    concurrent nextest invocations.
  - `threads-required` greater than test-threads is clamped by nextest. This is
    documentation knowledge and was not re-verified.
- **Version / build conditions:** Test groups since 0.9.48. `priority` since 0.9.91.
- **Limits and negative results:**
  - No concept of effects, fixtures or roles attached to a test beyond environment and
    scheduling.
  - No validation that a declaration matched at least one test. An unmatched override is
    silent.
- **Evidence:** Interface-checked (docs and runner source named above).

### cargo-nextest/layered-tool-configuration
- **Need it serves:** Let a wrapper tool contribute configuration (profiles, overrides,
  groups) for a run without copying and editing the repository's config file.
- **Recognizable code shapes:**
  - A wrapper that parses the repository `nextest.toml`, mutates a copy (adds overrides,
    changes the junit path) and passes `--config-file <copy>`.
- **Mechanisms and APIs:**
  - `--tool-config-file <tool>:<abs-path>`, which can be given several times.
  - Priority order: repository config, then tool configs (earlier is higher), then the
    embedded defaults.
  - Groups defined by a tool must be named `@tool:<tool>:<group>`. The repository config
    may reference tool groups.
  - Tool configs can define profiles and overrides.
  - The `NEXTEST_PROFILE` environment variable and CLI flags (`--profile`,
    `--test-threads`) layer above files.
- **Obligations discharged:**
  - Config merging and precedence.
  - No rewriting of repository files.
- **Residual caller obligations:**
  - Writing the tool file.
  - Accepting that repository settings outrank tool settings. A tool cannot override a
    repository override that sets the same key for the same test.
- **Semantic conditions:**
  - Precedence is per setting, so a tool override's `test-group` applies when no
    repository override sets `test-group` for that test.
  - Paths must be absolute.
- **Version / build conditions:** Tool config files have been available for many releases
  (`tool_config_basic` test). The exact first version was not checked.
- **Limits and negative results:**
  - Not every value can move to a tool file. The junit report path is a profile setting,
    and the repository profile's value outranks a tool's.
- **Evidence:** Interface-checked (CLI help text and runner tests via Context7).

### cargo-nextest/setup-and-wrapper-scripts
- **Need it serves:** Run preparation before matching tests (exporting environment through
  `$NEXTEST_ENV`), or wrap test execution and listing in a command such as an environment
  launcher.
- **Recognizable code shapes:**
  - Shell or Python orchestration that starts fixtures, exports variables, then calls
    `cargo nextest run`.
  - Per-test launcher prefixes.
- **Mechanisms and APIs:**
  - `[scripts.setup.<name>] command = '..' | [..] | {command-line, relative-to, env}`,
    with per-command `env` since 0.9.131.
  - `[scripts.wrapper.<name>]`.
  - `[[profile.<p>.scripts]] filter = .., platform = .., setup = .. | run-wrapper = .. |
    list-wrapper = ..`.
  - A setup script writes `KEY=VALUE` lines to `$NEXTEST_ENV`, and those variables reach
    matching tests.
- **Obligations discharged:**
  - Ordering setup before matching tests.
  - Environment propagation.
  - Per-filter launcher wrapping.
- **Residual caller obligations:**
  - Teardown: none is documented, so fixture cleanup is the caller's.
  - Any tie between fixture lifetime and the run's outcome.
  - Failure semantics beyond "a setup failure fails the run".
- **Semantic conditions:**
  - Both features are experimental. The upstream warning says incorrect wrapper scripts
    "can cause your tests to silently stop working".
- **Version / build conditions:**
  - `experimental = [...]` opt-in.
  - Setup-script configuration moved from `[script.*]` to `[scripts.setup.*]` in 0.9.98;
    the old form is deprecated.
- **Limits and negative results:** No teardown or post-run hook in the documented
  interface. Search scope: setup-scripts page and configuration reference via Context7.
- **Evidence:** Interface-checked (docs).

### cargo-nextest/list-archive-record (already-adopted family, recorded for completeness)
- **Need it serves:** Machine-readable selection, portable built-test archives and recorded
  run history.
- **Mechanisms and APIs:**
  - `cargo nextest list --message-format json`, which gives `rust-suites`, `testcases` and
    `filter-match`.
  - `cargo nextest archive` with `run --archive-file`.
  - Record and replay: `[record]` user config and `cargo nextest store list|export`,
    `replay -R`.
- **Obligations discharged:** Inventory, archive packaging and run history.
- **Residual caller obligations:** Interpreting the JSON, and archive provenance policy.
- **Semantic conditions:** Record and replay need the experimental `record` opt-in in user
  config.
- **Evidence:** Interface-checked (docs; current local user config enables record).

## Current applicability

### [CBO-049](../../../library_leverage/codebase/developer-tooling.md#cbo-049) — Test selection, requirements and runner configuration maintained beside nextest
- **Types:** `cargo-nextest/declarative-test-scheduling`,
  `cargo-nextest/layered-tool-configuration`, `cargo-nextest/setup-and-wrapper-scripts`,
  `cargo-nextest/list-archive-record`.
- **Realization:** Installed tool 0.9.146. No version move is needed.
- **Conclusion:** Fits with conditions for the scheduling projection only. The rest of the
  catalogue (effects, fixtures, roles, fixture-outcome reconciliation) has no nextest
  equivalent, and setup and wrapper scripts are ruled out for fixture lifetime.
- **What it would discharge / what remains local:**
  - Scheduling, option (a): the owner declarations already key on package plus test-name
    prefix, plus a few exact-name exceptions. They map directly onto
    `package(x) & test(/^prefix/)` overrides with `test-group` and `threads-required`.
    - Written statically into the repository config, ordered most-specific first, they would
      replace the per-run generation of one exact-match override per selected test, and the
      tomlkit rewrite of the repository config.
    - The static file then becomes a second declaration of the same owner facts (LOP-02).
      Generating it once through the existing codegen path, or making it the authority,
      avoids that.
  - Scheduling, option (b): keep the per-run computation but emit an `@tool:pse:`-scoped
    tool config file through `--tool-config-file` instead of mutating a copy of the
    repository config.
    - The wrappers already treat tool configs as lower-priority inputs.
    - This removes the "copy and edit repository TOML" mechanism, but not the projection.
  - The junit path rewrite in the test-invocation module is a repository-profile setting.
    It cannot move to a lower-priority tool config. It could move to `--profile` with a
    dedicated profile, or stay.
  - Remains local:
    - The closed-world check that the requirement catalogue equals the selected catalogue.
      nextest silently defaults unmatched tests.
    - Effect guards, fixture and peer roles, required-role validation, force-validation per
      unit-graph closure and the managed-primary partition.
    - Fixture lifetime reconciliation.
- **Prerequisites, configuration or extension:**
  - (a) Ordered static overrides plus a generator, or a single authority.
  - (b) A tool-config writer.
  - Groups referenced by the repository must exist. A tool group must be `@tool:`-prefixed.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Never manufacture a passing result:* unaffected.
  - *Fixture lifetime tied to the reconciled outcome:* setup scripts have no documented
    teardown and are experimental, with a silent-failure warning for wrappers. Ruled out
    for fixtures.
  - *Managed-primary partitions:* already expressed by `default-filter` and
    `--ignore-default-filter` in the repository config.
  - Static prefix overrides lose the "selected catalogue equals declarations" check unless
    it is kept in tooling.
- **Unsettled premises and what would settle them:**
  - Whether the exact per-test projection is intentional, to fail closed when a test
    changes owner, or incidental.
  - Whether concurrent agent nextest processes need cross-process group limits, which
    nextest groups cannot provide and the host admission layer handles. The test
    requirements module's design notes would settle intent.
- **Evidence:** Interface-checked (docs and runner source); application Proposed.

### [CBO-052](../../../library_leverage/codebase/developer-tooling.md#cbo-052) — Several command surfaces without a shared declaration (the 17 recipes that call the nextest wrapper)
- **Types:** `cargo-nextest/declarative-test-scheduling`, through profiles.
- **Realization:** Installed 0.9.146.
- **Conclusion:** Ruled out as a consolidation route. Each recipe is a named selection
  preset combining cargo build arguments (`-p`, `--features`, `--bin`, `--lib`), a
  filterset and a native environment mode. nextest profiles can carry `default-filter` but
  not cargo package or feature selection.
- **What it would discharge / what remains local:** Only the filterset part could move into
  per-preset profiles. Build selection and environment admission stay in recipes.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** Admission through the
  environment launcher must stay outside nextest.
- **Unsettled premises and what would settle them:** None.
- **Evidence:** Interface-checked; application Proposed.
