# cargo-nextest — Comprehensive Rust Test Runner Reference

Target: cargo-nextest 0.9.150 (stable) · Research date: October 28, 2026 · Ecosystem: Rust / Cargo

cargo-nextest is an advanced Rust test runner that replaces Cargo's conventional test-execution model with process-isolated, parallel execution of individual tests. It provides a substantial orchestration layer around Rust test binaries, including resource-aware scheduling, declarative test selection, retries, timeout enforcement, CI distribution, structured reporting, and execution diagnostics.

Unlike a testing framework, nextest does not primarily define how tests are authored. It operates on compiled Rust test binaries, building on Cargo and the `libtest` harness while supplying more sophisticated execution infrastructure.

The most important architectural distinction is that nextest treats individual tests as independently schedulable operating-system processes, rather than relying on `libtest` to schedule multiple tests within a shared process.

Official documentation: `https://nexte.st/`

Source repository: `https://github.com/nextest-rs/nextest`

Release history: `https://nexte.st/changelog/`

Version note: 0.9.150 was released October 26, 2026. It fixes a security vulnerability affecting experimental recorded-run reruns in versions 0.9.125–0.9.149.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



## 1. Capability map

| Capability family         | Principal mechanisms                                        | Architectural value                                  |
| ------------------------- | ----------------------------------------------------------- | ---------------------------------------------------- |
| Execution engine          | Process-per-test, parallel scheduler, signal handling       | Isolation, failure containment, concurrency          |
| Test discovery            | Cargo metadata, executable test enumeration, binary IDs     | Independent test identification and selection        |
| Test selection            | Filterset DSL, regex, package/binary/platform filters       | Declarative test suites and precise targeting        |
| Resource scheduling       | Weighted threads, priorities, test groups                   | Concurrency constrained by resource requirements     |
| Reliability               | Retries, backoff, flaky classification, failure thresholds  | More controlled handling of intermittent failures    |
| Time management           | Slow-test detection, deadlines, termination policies        | Detect and contain hung or unexpectedly slow tests   |
| Environment orchestration | Setup scripts, environment propagation, wrappers            | Prepare fixtures, services, and execution contexts   |
| Distributed CI            | Partitioning, compiled binary archives, target runners      | Separate compilation from execution across workers   |
| Reporting and diagnostics | JUnit, machine-readable test lists, output policies         | Integration with automation and external analysis    |
| Run history               | Record, rerun, replay, portable recordings, Perfetto traces | Failure investigation and execution introspection    |
| Ecosystem integration     | Coverage, mutation testing, Miri, debuggers, tracers        | Reuse a common execution layer across test workflows |

The central architectural opportunity is to use nextest as a unified execution and control plane for compiled Rust tests, replacing separate machinery for scheduling, retrying, isolating, timing, filtering, and collecting results.

Not every capability has the same stability status: core execution and configuration are established, while run recording and some machine-readable run-output functionality remain experimental.

## 2. Execution architecture and process isolation

### 2.1 Two-phase execution model

Nextest separates test execution into discovery and scheduling:

1. Build and enumerate. It delegates compilation to Cargo, broadly using `cargo test --no-run`, processes Cargo's structured artifact output, and invokes compatible test binaries to enumerate their tests.
2. Execute and aggregate. It selects individual tests, launches each in its own process, captures results, applies retries and timeouts, and produces aggregate reports.

A single test is normally invoked through the harness using its exact test name, rather than launching a test binary once and allowing `libtest` to execute the entire suite internally.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Official architecture: `https://nexte.st/docs/design/how-it-works/`

Why this matters: Rust's conventional test runner provides parallelism within each test binary; nextest can schedule tests across binaries and packages within one global execution pool. This improves load balancing for suites with heterogeneous tests and enables controls that operate at individual-test granularity.

### 2.2 Isolation guarantees and limits

Process-per-test provides several useful properties:

- Memory isolation: global variables, static mutable state, thread-local state, heaps, and process-local state are not shared between tests.
- Failure containment: a process abort or segmentation fault need not terminate unrelated tests; nextest can attribute the abnormal termination to an individual test.
- Independent environment and resources: each test receives its own process environment and can have separately managed standard streams, timeouts, and retry attempts.
- Process lifecycle control: nextest can terminate a hung test and manage associated child-process groups, subject to operating-system differences.

However, process isolation does not isolate external resources: filesystem paths, database instances, fixed ports, system services, and shared operating-system resources remain shared. Nor does it make each test single-threaded; a test can create and coordinate multiple threads within its own process.

Performance tradeoff: nextest substitutes repeated process startup and test-binary initialization for shared-process execution. That typically favors suites containing many nontrivial tests, but may be less efficient for extremely small tests or those with expensive process initialization. The relevant comparison is total wall-clock time, not individual-test execution time alone.

Official rationale: `https://nexte.st/docs/design/why-process-per-test/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 2.3 Harness compatibility

Nextest understands the conventional `libtest` protocol: enumerate test names and run one exact test on request.

A custom harness must therefore support the necessary listing and invocation behavior, including `--list --format terse`, handling ignored tests, and invoking an individual test with `--exact` and `--nocapture`. Custom harnesses that expect only one whole-binary invocation may not work without adaptation.

The `libtest-mimic` and `datatest-stable` ecosystem provides relevant compatibility paths, but nextest does not automatically understand arbitrary executable-specific test protocols.

Important limitation: nextest does not currently run Rust documentation tests (`doctests`). A complete Rust testing workflow may still need a separate doctest execution path.

Source: `https://nexte.st/docs/design/custom-test-harnesses/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



### 2.4 Execution and build controls

| Control                | Semantics                                                                                          |
| ---------------------- | -------------------------------------------------------------------------------------------------- |
| `cargo nextest run`    | Build, discover, and execute selected tests                                                        |
| `cargo nextest list`   | Discover test binaries and individual tests without executing the tests                            |
| `-j`, `--test-threads` | Control nextest's test execution concurrency, distinct from Cargo compilation                      |
| `--build-jobs`         | Control Cargo build parallelism                                                                    |
| `--no-fail-fast`       | Continue scheduling tests after failures                                                           |
| `--max-fail`           | Stop after a specified failure threshold, optionally controlling whether to wait for running tests |
| `--no-tests`           | Choose whether an empty selected suite is an error, warning, or success                            |
| `--run-ignored`        | Include or exclusively run ignored tests                                                           |
| `--no-capture`         | Display output directly, with serial test execution to avoid interleaving                          |

By default, fail-fast stops scheduling new tests after a failure but does not imply immediate cancellation of all tests already running. That distinction matters for cleanup and CI resource usage.

Source: `https://nexte.st/docs/running/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



## 3. Test discovery and declarative selection

### 3.1 Test identity and metadata

Nextest identifies tests in the context of their compiled test binary, not merely by test name.

The surrounding metadata includes package identity, target/binary kind, binary name, host-versus-target platform, executable path, and test attributes such as ignored status. This supports selection at both the Cargo-artifact level and the individual-test level.

`cargo nextest list` exposes this inventory as structured JSON, making it suitable for external test discovery, IDE integrations, and analysis. The `binaries-only` listing mode avoids executing test binaries for individual-test enumeration.

Source: `https://nexte.st/docs/machine-readable/list/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 3.2 Filterset DSL

Nextest's filterset language provides a relational-style selection algebra over test metadata, rather than only substring or regular-expression matching.

| Predicate                            | Selection dimension                                                        |
| ------------------------------------ | -------------------------------------------------------------------------- |
| `test(...)`                          | Individual test names                                                      |
| `package(...)`                       | Cargo packages                                                             |
| `deps(...)`, `rdeps(...)`            | Dependency and reverse-dependency relationships in the package graph       |
| `binary(...)`, `binary_id(...)`      | Compiled test binary names and composite IDs                               |
| `kind(...)`                          | Target kinds such as library, integration test, benchmark, binary, example |
| `platform(host)`, `platform(target)` | Host or target execution platform                                          |
| `group(...)`                         | Tests assigned to named test groups; command-line usage only               |
| `default()`                          | Active default test selection                                              |
| `all()`, `none()`                    | Universal and empty sets                                                   |

Predicates compose through intersection (`&`), union (`|`), set subtraction (`-`), and negation (`!`), with parentheses controlling precedence. Name matching supports substring, exact, glob, and regex syntax.

For example, `package(engine) & test(/^integration::/) - test(slow)` selects integration-named tests in a particular package while excluding names containing `slow`. This is a selection expression, not a source-code annotation mechanism.

The language also supports dependency-aware test targeting: `rdeps(...)` can identify tests in packages depending on a specified package, avoiding the need to implement a separate Cargo workspace dependency traversal for this particular operation. This is package-graph selection, not a general test-impact analysis system that determines which tests exercise changed functions.

Source: `https://nexte.st/docs/filtersets/reference/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 3.3 Filtering composition and discovery optimization

Filtersets are passed using `-E` or `--filterset`. Multiple filterset expressions are combined as a union, while additional positional test-name restrictions interact with the filter selection. The default filter configured in a profile further constrains normal runs unless explicitly bypassed.

Nextest can evaluate package, platform, binary, and target-kind predicates before executing a test binary to list its tests. This permits binary-level filtering even where executing every target-platform binary would be impossible or undesirable.

Semantic distinction: filtering based on test names requires discovering test names; filtering based entirely on package or binary metadata can often reject entire binaries earlier.

Source: `https://nexte.st/docs/filtersets/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



## 4. Declarative configuration and policy resolution

### 4.1 Profiles, inheritance, and configuration layers

Nextest supports a repository-level configuration at `.config/nextest.toml`, conventionally containing a `default` profile, optional named profiles such as `ci`, and shared test-group definitions.

Named profiles can inherit settings from other profiles. This supports progressively specialized execution policies while retaining common defaults.

Additional configuration surfaces include:

- CLI options and environment variables for execution-specific changes.
- Per-test overrides for filtered subsets of tests.
- Tool-specific configuration files for higher-level tools integrating with nextest.
- User configuration for local presentation, pager, interface, and recording preferences, distinct from repository test policy.

The general resolution model prioritizes explicit execution controls over matching per-test rules and inherited profile settings. Within per-test rules, precedence is evaluated per setting, not by selecting one winning override object wholesale.

That last distinction is consequential: one rule can determine retries, another can determine timeout behavior, and another can determine reporting, provided their precedence and match conditions allow it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources: `https://nexte.st/docs/configuration/` and `https://nexte.st/docs/configuration/per-test-overrides/`

### 4.2 Per-test policy overlays

A `[[profile.<name>.overrides]]` rule can select tests through a filterset expression, a host/target platform condition, or both.

Supported settings include:

| Policy dimension | Configurable behavior                                                           |
| ---------------- | ------------------------------------------------------------------------------- |
| Priority         | Relative scheduling priority                                                    |
| Reliability      | Retry policy and whether recovered flaky tests count as failures                |
| Resource use     | `threads-required` and `test-group`                                             |
| Time             | Slow-test thresholds, termination, and leak detection                           |
| Output           | Immediate, final, or suppressed success/failure output                          |
| Reporting        | JUnit output retention, skipped-test treatment, and flaky-result representation |
| Invocation       | Additional test arguments                                                       |
| Benchmarks       | Separate benchmark timeout settings                                             |

Platform selection accepts target triples and Rust `cfg(...)` expressions. Cross-compilation makes the distinction between the host platform executing build-time tools and the target platform for the compiled application essential.

Sources: `https://nexte.st/docs/configuration/per-test-overrides/` and `https://nexte.st/docs/configuration/specifying-platforms/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



### 4.3 Version governance

Nextest permits repository configuration to declare a required or recommended nextest version using `nextest-version`. This helps prevent older runner installations from silently applying incompatible or incomplete policy.

The public compatibility surface of `cargo-nextest` covers CLI arguments, configuration format, and documented machine-readable output. Human-oriented terminal output and explicitly experimental interfaces are outside the same stability guarantees.

Source: `https://nexte.st/docs/configuration/minimum-versions/` and `https://nexte.st/docs/stability/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



## 5. Resource-aware scheduling

This is one of nextest's most consequential capabilities beyond ordinary parallel execution.

### 5.1 Weighted test capacity

`threads-required` lets a test consume multiple units from the global execution-capacity budget. A heavyweight integration test might reserve four slots while an ordinary unit test reserves one.

The value is a scheduler weight, not a request to create or allocate actual CPU threads. It cannot enforce operating-system CPU affinity, memory limits, or cgroup quotas.

Special values such as `num-cpus` and `num-test-threads` can reserve machine-relative or runner-relative capacity. Reserving the entire runner capacity effectively creates a global exclusivity constraint for the test.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/configuration/threads-required/`

### 5.2 Named test groups as scoped semaphores

Test groups provide independent concurrency limits for subsets of tests. A group with `max-threads = 1` behaves as a logical mutex; one with a larger limit behaves as a counting semaphore.

The distinction from `threads-required` is fundamental:

- Weighted slots constrain a test relative to all concurrently running tests.
- Test groups constrain a test relative to other members of the same group.
- Composition applies both restrictions to the same test.

This makes groups useful for protecting a shared external dependency, such as a service accepting only a limited number of simultaneous clients, without serializing the entire test suite.

Group limits are local to a nextest invocation. They are not distributed locks across separate CI workers or separate invocations, and they do not protect against unrelated external programs.

Nextest exposes group and global slot identifiers through environment variables, which can support dynamically partitioned resources rather than fixed resource names.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/configuration/test-groups/`

### 5.3 Scheduling priorities

Per-test priorities range from `-100` to `100`; higher-priority tests are scheduled earlier when capacity is available.

This offers a way to move long-running tests earlier in a suite so that the final completion time is less dominated by a late-starting long test. It is a heuristic for improving scheduling efficiency, not a dependency system or ordering guarantee.

Nextest does not currently provide the automatic historical-duration-based prioritization described as a possible future capability in its documentation.

Source: `https://nexte.st/docs/configuration/test-priorities/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Combined architectural value: profiles, filtered overrides, priorities, weighted capacity, and named groups form a declarative resource-scheduling model. For suitable workloads, they can replace bespoke mutex wrappers, manually partitioned test executables, and external test-launch scripts—without changing the test framework itself.

## 6. Reliability, timeouts, and repeated execution

### 6.1 Retries and flaky-test semantics

Nextest can retry failed tests independently. `--retries 2` means a maximum of three attempts, including the initial execution; every attempt has a fresh process context.

Retry policies can be global or scoped through per-test overrides. They support fixed and exponential backoff, optional maximum delay, and jitter for distributing retries against constrained external services. Explicit CLI or environment-level retry counts override configured backoff behavior and per-test retry policies.

A test that fails initially and subsequently passes is classified as flaky, distinct from a first-attempt pass or an ultimately failed test.

Since 0.9.131, `flaky-result = "fail"` allows recovered flaky tests to fail the overall run. Importantly, `flaky-result` does not itself enable retries. The JUnit option `junit.flaky-fail-status` instead changes report representation; it is not a substitute for runner-level failure policy.

Interpretation: retries provide transient-failure recovery and evidence of nondeterminism, but a successful retry is not equivalent to demonstrating deterministic correctness.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/features/retries/`

### 6.2 Slow detection versus termination

Nextest distinguishes a test being slow from a test being forcibly stopped.

By default, a test exceeding 60 seconds receives a slow-test notification; this does not automatically terminate it.

A structured `slow-timeout` policy can specify a reporting period, number of periods before termination, and termination grace period. For example, a 30-second period with `terminate-after = 4` produces a termination threshold of 120 seconds.

The `global-timeout` setting separately constrains the entire test-execution run.

Since 0.9.115, `on-timeout = "pass"` can classify a deliberately time-bounded test as successful when it reaches its timeout, producing a `TMPASS` result. That mechanism is useful for specific bounded-execution contracts, but unsuitable as a general response to hanging tests.

Platform behavior matters: on Unix, nextest uses process groups and normally attempts SIGTERM followed by SIGKILL after the configured grace period. On Windows, timeout termination uses job objects and terminates immediately; the timeout grace period does not have the same effect.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/features/slow-tests/`

### 6.3 Subprocess leak detection

Nextest also monitors whether the output pipes associated with a completed test remain open beyond `leak-timeout`, which defaults to 100 milliseconds.

A lingering subprocess that inherited stdout or stderr can therefore produce a `LEAK` classification. Since 0.9.95, policy can promote this to `LEAK-FAIL`.

This is not comprehensive process-tree leak detection. A child process with redirected output streams may remain alive without triggering the detector. Cleanup and process ownership therefore remain responsibilities of the test and its supporting libraries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/features/leaky-tests/`

### 6.4 Stress testing is different from retrying

`--stress-count=N` executes selected tests repeatedly, while `--stress-duration=DURATION` repeats execution until a specified duration is reached. `--stress-count=infinite` supports open-ended repetition.

The distinction is operationally important:

- Retries execute again after failures, typically to recover from intermittent problems.
- Stress runs execute repeatedly regardless of whether the test passed, to expose nondeterministic or timing-sensitive behavior.

Stress runs retain iteration information and integrate with nextest's output and reporting mechanisms. They do not replace randomized input generation or property-based testing; those capabilities belong to the test implementation or associated frameworks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/features/stress-tests/`

## 7. Test environment orchestration and command interception

### 7.1 Runtime environment reconstruction

Nextest reconstructs an execution environment compatible with Cargo test binaries, including package-relative working directories, relevant Cargo paths, dynamic-library search paths, build-script output directories, and executable locations.

Its additional environment variables expose runner context:

| Environment variable family                                       | Information made available                      |
| ----------------------------------------------------------------- | ----------------------------------------------- |
| `NEXTEST` and `NEXTEST_EXECUTION_MODE`                            | Execution under nextest and its execution model |
| `NEXTEST_RUN_ID`                                                  | Identifier for the overall run                  |
| `NEXTEST_BINARY_ID`, `NEXTEST_TEST_NAME`                          | Executed binary and test identity               |
| `NEXTEST_ATTEMPT`, `NEXTEST_TOTAL_ATTEMPTS`, `NEXTEST_ATTEMPT_ID` | Retry-attempt context                           |
| `NEXTEST_TEST_GROUP`                                              | Assigned test group                             |
| `NEXTEST_TEST_GLOBAL_SLOT`, `NEXTEST_TEST_GROUP_SLOT`             | Allocated scheduling slots                      |
| `NEXTEST_STRESS_CURRENT`, `NEXTEST_STRESS_TOTAL`                  | Stress-iteration context                        |
| `NEXTEST_WORKSPACE_ROOT`, `NEXTEST_BIN_EXE_*`                     | Workspace and compiled-executable locations     |

These variables can support test-specific artifact directories, deterministic resource allocation by slot, attempt-aware diagnostics, and coordination with auxiliary programs. Individual variable availability depends on the applicable execution mode and version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/configuration/env-vars/`

### 7.2 Setup scripts — experimental

Nextest supports declarative setup scripts, enabled with `experimental = ["setup-scripts"]`.

Scripts are defined independently from rules selecting when they apply. Those rules can match test filtersets and host/target platforms.

Notable capabilities include:

- Running setup operations only when relevant tests are selected.
- Running multiple setup scripts serially, with a nonzero exit preventing the test run.
- Providing environment variables to matching tests by writing assignments to the file identified by `NEXTEST_ENV`.
- Defining script-specific environment, command paths, timeouts, and output behavior.
- Representing setup executions in JUnit results.

The scripts' command paths can be resolved relative to the target directory or workspace root, while their working directory remains the workspace root.

Boundary: this is setup orchestration, not a complete fixture lifecycle manager. It does not automatically provide teardown, transactional rollback, or guaranteed destruction of external resources. Since 0.9.148, a setup-script timeout cannot be configured to count as success.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Source: `https://nexte.st/docs/configuration/setup-scripts/`

### 7.3 Wrapper scripts — experimental

Wrapper scripts intercept the invocation of test binaries.

Unlike setup scripts, a wrapper becomes part of the execution command itself, enabling scenarios such as running selected tests under a sandbox, emulator, instrumentation tool, or privileged command.

Two independent operations are supported:

- `list-wrapper`: intercept test enumeration.
- `run-wrapper`: intercept actual test execution.

Wrapper selection can use test filters and platform conditions. A listing wrapper cannot depend on `test()` or `default()` predicates because individual tests have not yet been enumerated.

Wrapper scripts also define how they interact with an existing target runner, including replacing it or executing it inside or outside the wrapper.

Critical caveat: a wrapper must actually invoke the underlying test binary and preserve meaningful failure behavior. An incorrectly constructed wrapper can make an unexecuted test appear successful. Only the first matching wrapper for each relevant stage is applied.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/configuration/wrapper-scripts/`

### 7.4 Target runners versus nextest wrappers

Cargo-compatible target runners are the established mechanism for invoking test binaries on another platform or through an external runtime. They can be configured through Cargo's target runner configuration or the corresponding `CARGO_TARGET_<TRIPLE>_RUNNER` environment variable.

| Dimension                 | Cargo target runner          | Nextest wrapper script                                    |
| ------------------------- | ---------------------------- | --------------------------------------------------------- |
| Stability and portability | Standard Cargo integration   | Nextest-specific and experimental                         |
| Selection granularity     | Target/platform-oriented     | Filtersets and platforms                                  |
| Discovery                 | Applies during listing       | Optional listing wrapper                                  |
| Execution                 | Applies during runs          | Optional execution wrapper                                |
| Composition               | One configured target runner | Selective wrapper rules, with explicit runner interaction |

This distinction matters especially for cross-compilation: a compiled target binary is not necessarily executable on the build host, even during test enumeration.

Source: `https://nexte.st/docs/features/target-runners/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



### 7.5 Extra test-harness arguments

`run-extra-args` allows additional arguments to be forwarded to test harnesses, globally or for matching tests.

This can bridge differences in harness behavior without changing every test invocation. For example, a custom harness may expose a flag for running a selected test on its main thread.

These arguments are harness-specific. Passing an argument accepted by one harness does not imply compatibility with every `libtest`-like executable.

Source: `https://nexte.st/docs/configuration/extra-args/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



## 8. Separating test compilation from test execution

### 8.1 Compiled-test archives

`cargo nextest archive` packages compiled test artifacts into a `.tar.zst` archive, allowing `cargo nextest run --archive-file ...` to execute them elsewhere without repeating compilation.

The archive carries Cargo and nextest metadata, test binaries, relevant non-test executables, linked dynamic libraries, and selected build-script output directories. Additional target-directory artifacts can be included using `archive.include`, with recursion-depth and missing-path policies.

Important distinction: the archive contains execution artifacts, not a complete source checkout. The execution environment must have an appropriate corresponding source tree, normally at the same revision. `--workspace-remap` supports a different source-tree location; `--extract-to` controls the extracted artifacts' location.

Nextest itself must be available on the execution machine, but Cargo need not be installed there when using the standalone `cargo-nextest` executable. Binary architecture and runtime-library compatibility remain prerequisites.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/ci-features/archiving/`

### 8.2 Controlling archive contents

An archive can contain a subset of built test binaries by applying `-E` to `cargo nextest archive`. This is distinct from limiting which binaries Cargo compiles in the first place.

`archive.include` supports additional files relative to the Cargo target directory, with configurable traversal depth and behavior for missing paths. It is not a general-purpose arbitrary filesystem packaging facility.

Version 0.9.150 additionally introduces `--zstd-long`, enabling Zstandard long-distance matching with a 128 MiB window to improve compression opportunities for larger archives. No corresponding extra extraction option is required.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



### 8.3 Manual metadata-based execution

The same separation can be constructed without nextest's archive format through:

- `--binaries-metadata`, consuming nextest's JSON binary inventory.
- `--cargo-metadata`, consuming Cargo's workspace metadata.
- `--target-dir-remap`, pointing execution to a relocated target directory.

This is a significant interoperability capability: existing artifact caches, custom packaging systems, and remote build infrastructure can provide compiled test outputs without adopting nextest's exact archive container.

Relocatability still matters. Runtime `CARGO_MANIFEST_DIR` and `NEXTEST_BIN_EXE_*` variables are more appropriate than build-time embedded absolute paths when tests need to move between machines.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



## 9. CI sharding and distributed execution

Nextest supports deterministic selection of test subsets through `--partition`. The two primary strategies have different semantics.

| Strategy    | Mechanism                                           | Consequential property                                                             |
| ----------- | --------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `slice:m/n` | Round-robin division of the filtered test inventory | Even distribution by test count, but changing the inventory can change assignments |
| `hash:m/n`  | Hash of binary ID and test name                     | Assignment remains stable as tests are added, removed, or filtered                 |
| `count:m/n` | Older per-binary counted partitioning               | Deprecated; lacks the main advantages of slice and hash                            |

Hash assignment is stable within a nextest version series, but can produce uneven shards, especially for small suites. Neither strategy inherently balances actual execution time using historical measurements.

Sharding is not distributed orchestration. Nextest decides which tests a worker runs; an external CI system still provisions workers, transfers archives, manages shared infrastructure, and aggregates results.

The strongest composition is:

Single Cargo compilation → compiled-test archive → multiple execution workers → per-worker nextest scheduling and JUnit output.

This separates expensive compilation from scalable execution and allows different workers to execute appropriate host or target-platform tests. It also introduces real operational constraints around identical source revisions, artifact transfer, external resource contention, and result aggregation.

Sources: `https://nexte.st/docs/ci-features/partitioning/` and `https://nexte.st/docs/ci-features/archiving/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



## 10. Reporting, exit status, and machine-readable interfaces

### 10.1 Terminal reporting and output control

Nextest separates three concerns that conventional test runners often conflate:

- Live status reporting: `--status-level` determines which events appear as tests progress; `--final-status-level` independently controls the final summary.
- Captured output visibility: `success-output` and `failure-output` choose whether output is emitted immediately, in the final summary, both at appropriate stages, or never.
- Progress presentation: `--show-progress` selects an automatic display, progress bar, counter, no progress display, or progress-only presentation.

This separation is useful for large suites where displaying every success would obscure failures, while retaining diagnostic output for unsuccessful or retried tests remains important.

Output capture is not equivalent to output retention. For example, suppressing successful test output in the terminal does not necessarily prevent a run recording from retaining it.

Source: `https://nexte.st/docs/reporting/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 10.2 JUnit/XUnit reporting

Nextest produces Jenkins-compatible JUnit XML through a profile's `junit.path` setting, commonly `[profile.ci.junit]`. The generated report is placed under the nextest profile's target-directory storage path.

Each test binary becomes a `<testsuite>`; individual tests become `<testcase>` entries. Reporting can include captured stdout and stderr, retry information, flaky-test results, setup-script outcomes, and optionally skipped tests.

Important policy options:

| Setting                | Function                                                                    |
| ---------------------- | --------------------------------------------------------------------------- |
| `report-name`          | Distinguishes reports during aggregation                                    |
| `store-success-output` | Retains successful test output, disabled by default                         |
| `store-failure-output` | Retains failed test output, enabled by default                              |
| `report-skipped`       | Emits no skipped tests, ignored tests only, or all applicable skipped tests |
| `flaky-fail-status`    | Controls XML representation of a flaky test that is considered failed       |

There are two important semantic caveats. First, `flaky-fail-status = "success"` changes the XML representation, not the actual nextest exit status. Second, `report-skipped = "all"` can introduce duplicated skipped-test records when merging reports from separately partitioned workers, because each worker can report tests assigned elsewhere.

JUnit is the established integration surface for CI dashboards and test-report aggregation, but it should not be treated as a lossless encoding of the entire execution lifecycle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/machine-readable/junit/`

### 10.3 Structured test inventory versus structured execution events

The important machine-readable surfaces are different products of the execution pipeline:

| Surface                                        | Status            | Content and application                                                   |
| ---------------------------------------------- | ----------------- | ------------------------------------------------------------------------- |
| `cargo nextest list --message-format json`     | Supported, stable | Test/binary inventory and filter-match information                        |
| `cargo nextest list --list-type binaries-only` | Supported, stable | Artifact inventory without querying every binary for test names           |
| JUnit XML                                      | Supported, stable | Aggregated test outcomes and CI interoperability                          |
| `libtest-json`                                 | Experimental      | Stream of test-run events broadly modeled on unstable Rust `libtest` JSON |
| `libtest-json-plus`                            | Experimental      | Libtest-style JSON with additional nextest-specific fields                |
| Recorded-run data                              | Experimental      | Persistent run events, outcomes, timing, and captured output              |

The structured test list exposes package and binary IDs, source/target paths, test names, and selection status. The companion `nextest-metadata` Rust crate can deserialize supported list output.

The experimental libtest output requires `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1` and `--message-format libtest-json` or `libtest-json-plus`; its currently documented format version is `0.1`. The official specification is incomplete, so consumers should not assume long-term schema stability.

Integration implication: external tooling can reliably automate test discovery and use JUnit for aggregate results, but richer per-event execution analysis currently involves experimental output or recording interfaces. Scraping human-readable terminal output is not an equivalent stable API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+2



Sources: `https://nexte.st/docs/machine-readable/list/` and `https://nexte.st/docs/machine-readable/libtest-json/`

## 11. Recorded runs, reruns, replay, and execution traces

Experimental feature family — introduced in 0.9.123; important compatibility changes in 0.9.150.

This is one of nextest's more advanced architectural developments: test execution can produce a persistent record that is independently inspected, replayed, or used as the basis for selecting a subsequent run.

### 11.1 Run recording and retention

Recording can be enabled through `NEXTEST_EXPERIMENTAL_RECORD=1` or user configuration containing `[experimental] record = true`, with `[record] enabled = true`.

Recorded information includes test identities, outcomes, durations, and the captured stdout/stderr of both successful and unsuccessful tests. `--no-capture` prevents recording of test output that was not captured.

The local cache is managed through `cargo nextest store` commands:

- `store list` and `store info` inspect run history.
- `store prune` removes old recordings, with a dry-run option.
- `store export` creates a portable recording.
- `store export-chrome-trace` creates a timeline trace.

Default per-workspace retention is a maximum of 100 runs, 1 GB of compressed data, or 30 days, with configurable policies.

This introduces a useful separation between executing a test and reviewing its execution evidence. A test result no longer has to remain accessible only through the original terminal session or CI log.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources: `https://nexte.st/docs/features/record-replay-rerun/` and `https://nexte.st/docs/features/record-replay-rerun/managing-runs/`

### 11.2 Rerunning failed and incomplete work

`cargo nextest run -R latest` uses a recorded run to select tests that previously failed, were not executed because the run was interrupted, or have appeared since the recorded run.

By default, it also inherits the original run's package, target, and feature-selection scope, unless those options are explicitly replaced.

Reruns can be repeated, with nextest tracking outstanding failures across the sequence. An unusual but important status is exit code `5`, `RERUN_TESTS_OUTSTANDING`: the tests executed in the current rerun may all have passed while other previously failing tests remain unresolved.

Fundamental limitation: rerun selection tracks test results and test identity, not code changes. A previously successful test may have become invalid after a source change but still be excluded from a focused rerun. Reruns are therefore a development and failure-investigation acceleration mechanism, not evidence that the complete current codebase passes its tests.

Source: `https://nexte.st/docs/features/record-replay-rerun/rerun/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 11.3 Replay versus rerun

`cargo nextest replay -R latest` reconstructs the presentation of an earlier execution from its recording, without building or re-executing tests. Output display settings can be changed during replay, which can reveal previously suppressed diagnostic information.

Replay normally exits successfully when replay itself succeeds, regardless of the original test outcome. `--exit-code` instead requests the original run's exit code.

This distinction matters for automation: replaying evidence is not revalidating code. A replay command should not accidentally be interpreted as a fresh test run.

Source: `https://nexte.st/docs/features/record-replay-rerun/replay/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 11.4 Portable recordings and CI-to-local diagnosis

`cargo nextest store export latest --archive-file my-run.zip` produces a portable recording. It can be examined with `cargo nextest replay -R my-run.zip` or used for a rerun with `cargo nextest run -R my-run.zip`, subject to version and source/build compatibility.

This enables CI failures to be examined locally without immediately reproducing the entire CI environment. Rerunning from the recording, however, still requires an appropriate build and execution environment.

Security boundary: recordings retain complete captured test output, without automatic scrubbing or redaction. They may contain API keys, credentials, personal data, and other sensitive values. Portable recordings are diagnostic artifacts with confidentiality requirements, not inherently safe public reports.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/features/record-replay-rerun/portable-recordings/`

### 11.5 Perfetto / Chrome execution traces

`cargo nextest store export-chrome-trace latest` converts a recorded run into Chrome Trace Event Format, suitable for examination with Perfetto and SQL-based trace analysis.

The exported timeline contains test execution slices with metadata including binary identity, test name, test duration, and scheduling context. The two grouping modes expose different questions:

- `--group-by binary` organizes tests by their originating binaries, showing distribution and scheduling across test suites.
- `--group-by slot` organizes the timeline by scheduler capacity slots, exposing idle periods, utilization, and how efficiently test processes occupy available capacity.

The trace also supports analysis of slow tests, timing distributions, test groups, and setup-script durations.

This is a more precise diagnostic mechanism than only inspecting the total suite duration. It can reveal whether delays arise from particular tests, poor concurrency utilization, or resource constraints. It does not trace Cargo compilation itself.

Source: `https://nexte.st/docs/features/record-replay-rerun/perfetto-chrome-traces/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 11.6 Critical compatibility and security change in 0.9.150

The October 26, 2026 release fixes GHSA-vc25-qg58-w57c, a vulnerability in experimental reruns from untrusted portable recordings.

Versions 0.9.125–0.9.149 stored original Cargo build-scope arguments in a form that could allow a malicious recording to inject additional Cargo options, potentially executing attacker-controlled code during compilation. Version 0.9.150 instead stores structured build-scope data and enforces safer argument construction.

The fix introduces recording-store format 3.0, with a consequential compatibility break:

- Recordings created by 0.9.149 and earlier cannot be replayed or rerun by 0.9.150.
- Older recordings can still be listed and pruned.
- After 0.9.150 updates a recording store, older nextest versions cannot continue writing recordings to it.

This is a deliberate security-driven break in an experimental format, not a general backward-compatibility guarantee. For any workflow accepting portable recordings from other environments, 0.9.150 is the relevant minimum version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/changelog/`

## 12. Integrations with the Rust testing ecosystem

Nextest's value is often greatest when paired with specialized tools that provide the test semantics, instrumentation, or analysis while nextest remains responsible for execution.

### 12.1 LLVM code coverage — `cargo-llvm-cov`

`cargo-llvm-cov` can drive nextest through `cargo llvm-cov nextest`.

The division of responsibilities is explicit:

- cargo-llvm-cov configures coverage instrumentation, manages coverage data, and generates reports.
- nextest builds, selects, schedules, and executes the instrumented tests.
- LLVM tooling processes the underlying coverage data.

The integration works because nextest delegates compilation to Cargo, preserving relevant instrumentation environment variables and compiler settings.

Coverage for doctests requires a separate path because nextest does not execute them. The coverage data from normal tests and doctests can subsequently be merged; the documented doctest-coverage workflow requires nightly Rust.

Source: `https://nexte.st/docs/integrations/test-coverage/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 12.2 Undefined-behavior investigation — Miri

Nextest integrates with Miri using `cargo miri nextest run`.

Miri interprets Rust code to detect classes of undefined behavior, while nextest provides parallel process-level orchestration of independently interpreted tests.

The interaction has important consequences:

- Each test receives a separate Miri execution context.
- Several tests can execute concurrently even though a single Miri interpreter is internally single-threaded.
- The special `default-miri` profile provides separate execution policies for interpreted tests.
- Miri's cross-interpretation support permits execution against certain non-native target semantics.
- Compiled-test archives are not supported in this mode.

Semantic tradeoff: `cargo miri test` can detect a data race involving multiple tests sharing a process. Because nextest isolates those tests into separate processes, `cargo miri nextest run` cannot observe that particular class of cross-test interaction.

This illustrates why process isolation can improve independent test execution yet reduce visibility into bugs that depend on shared process state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/integrations/miri/`

### 12.3 Benchmarks: validation versus measurement

Nextest has two distinct benchmark execution modes.

| Mode                                             | Purpose                                     | Execution characteristics                                            |
| ------------------------------------------------ | ------------------------------------------- | -------------------------------------------------------------------- |
| `cargo nextest run --benches` or `--all-targets` | Validate benchmark implementations as tests | Test-mode execution; Criterion benchmarks perform a single iteration |
| `cargo nextest bench`                            | Measure benchmark performance               | Experimental; discovers benchmark cases and executes them serially   |

Criterion 0.5.0 or newer supports nextest's test-mode integration. Running a benchmark in this mode verifies that the benchmark can execute without failure; it does not produce a valid statistical performance measurement.

The experimental `bench` command, introduced in 0.9.117, supports benchmark harnesses including Criterion and Rust's nightly benchmark harness. It can participate in nextest's setup and wrapper orchestration, with benchmark-specific timeout configuration.

Enable it through `experimental = ["benchmarks"]` or `NEXTEST_EXPERIMENTAL_BENCHMARKS=1`.

The serial benchmark scheduling is deliberate: concurrently executing measurements would introduce interference that undermines many benchmark methodologies. It does not guarantee a noise-free machine or statistically reliable results by itself.

Sources: `https://nexte.st/docs/features/benchmarks/` and `https://nexte.st/docs/integrations/criterion/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



### 12.4 Mutation testing — `cargo-mutants`

`cargo-mutants` integrates through `cargo mutants --test-tool=nextest`, or the equivalent `test_tool = "nextest"` configuration.

Here, cargo-mutants generates source mutations, orchestrates the mutation campaign, and evaluates whether tests detect the changed behavior. Nextest supplies the underlying Rust test execution mechanism.

This permits nextest's scheduling and compatibility policies to be used as part of mutation testing, but does not make nextest responsible for mutation generation, mutation selection, or mutation coverage.

Source: `https://nexte.st/docs/integrations/cargo-mutants/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 12.5 Interactive debugging and system-call tracing

`cargo nextest run` supports `--debugger` and `--tracer` options, each restricted to exactly one selected test.

These modes retain nextest's test discovery and environment reconstruction, but change the process lifecycle to make external instrumentation practical.

| Mechanism           | Debugger                                | Tracer                             |
| ------------------- | --------------------------------------- | ---------------------------------- |
| Typical tools       | GDB, LLDB, WinDbg                       | `strace`, platform syscall tracers |
| Standard input      | Passed through for interactive commands | Null input                         |
| Output capture      | Disabled                                | Disabled                           |
| Timeouts            | Disabled                                | Disabled                           |
| Leak detection      | Disabled                                | Disabled                           |
| Unix process groups | Not created                             | Created                            |
| Intended use        | Interactive inspection                  | Noninteractive instrumentation     |

Examples include `cargo nextest run --debugger "rust-gdb --args" my_test` and `cargo nextest run --tracer "strace -f" my_test`.

This is a useful native alternative to manually discovering executable paths, reconstructing the Cargo runtime environment, and invoking a debugger against a precise harness test.

Source: `https://nexte.st/docs/integrations/debuggers-tracers/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



### 12.6 USDT probes for external observability

Since 0.9.108, nextest exposes User Statically Defined Tracing (USDT) probes for observing runner events without modifying the test suite or recompiling nextest with custom instrumentation.

Supported integrations include Linux x86-64 through `bpftrace`, macOS AArch64 through DTrace, and supported BSD/Solaris-family platforms through DTrace.

Exposed events include run lifecycle, test-attempt lifecycle, and setup-script execution. Probe arguments provide JSON event data, stable-within-execution event identifiers, and selected structured fields such as binary IDs, test names, process IDs, and durations.

Potential applications include:

- Observing process launch/completion and test duration distributions.
- Correlating scheduler events with operating-system resource measurements.
- Investigating resource contention or scheduling bottlenecks.
- Feeding execution events into external telemetry systems.

These probes are not covered by nextest's public API stability guarantees, so integrations must tolerate changes in probe names or argument schemas.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/integrations/usdt/`

## 13. Internal architecture and programmable extension boundaries

### 13.1 Event-driven dispatcher and executor

Nextest's runner is implemented using Tokio and an event-driven architecture organized around two principal components:

- Dispatcher: receives executor, signal, input, and reporting events; maintains the externally visible execution state; and coordinates responses and cancellation.
- Executor: schedules units of work and manages their asynchronous state machines, including test attempts and setup-script executions.

The components communicate using an actor-style message-passing model rather than directly sharing execution state.

Each work unit has dedicated channels for reporting progress and receiving requests. The dispatcher also linearizes significant state transitions so that output, running-test counts, and cancellation behavior observe a coherent sequence of events.

This design matters because it connects several otherwise separate features—weighted scheduling, retries, process lifecycle management, interactive interruption, and reporting—through a common execution state machine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/design/architecture/runner-loop/`

The internal runner is not a general-purpose dependency-DAG workflow engine. The upstream architecture document explicitly favors domain-specific composable operations over a generic execution framework, and describes teardown support as a prospective capability rather than established functionality.

### 13.2 Signal and terminal handling

On Unix, nextest ordinarily creates a separate process group for each running test or setup unit. The terminal sends signals such as SIGINT to nextest's foreground process group, after which nextest coordinates propagation and termination of its running units.

It also supports terminal job control through SIGTSTP and SIGCONT, allowing pause/resume behavior across the run.

Windows instead uses job objects for child-process lifetime management; these do not provide the same graceful termination semantics as Unix signals.

These differences matter for long-lived subprocesses, signal handlers within tests, graceful cleanup, and tests that launch their own process trees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/design/architecture/signal-handling/`

### 13.3 Companion Rust crates

Nextest is principally a Cargo command-line tool, but related crates expose useful portions of its metadata and selection machinery.

| Crate               | Current verified version | Role and stability                                                           |
| ------------------- | ------------------------ | ---------------------------------------------------------------------------- |
| `cargo-nextest`     | 0.9.150                  | Primary CLI application and test orchestrator                                |
| `nextest-metadata`  | 0.15.0                   | Publicly supported typed access to selected machine-readable output          |
| `nextest-filtering` | 0.22.0                   | Filterset parsing, compilation, and evaluation                               |
| `nextest-runner`    | Tracks internal releases | Core implementation crate; internal and subject to frequent breaking changes |

`nextest-metadata` exposes structures such as `TestListSummary`, `BinaryListSummary`, and `NextestExitCode`, together with helpers such as `ListCommand` for obtaining test lists programmatically.

Its compatibility guarantee is asymmetric: a metadata crate version is designed to handle future nextest output within the supported CLI series, but a newer metadata crate may stop supporting output produced by sufficiently old nextest binaries. Version 0.15.0 documents Rust 1.86 as its MSRV and cargo-nextest 0.9.15 as its minimum supported runner.

`nextest-filtering` exposes `Filterset::parse`, `ParseContext`, `EvalContext`, `BinaryQuery`, `TestQuery`, and `KnownGroups`, allowing tools to reuse nextest's own DSL semantics rather than reconstructing them independently.

Notably, `matches_binary` returns `Option<bool>`: `Some(true)` and `Some(false)` mean a binary is decisively accepted or rejected, while `None` means individual-test information is still needed. This expresses the partial-information evaluation model underlying nextest's efficient discovery pipeline.

`nextest-runner` contains the execution engine, but upstream identifies it as an internal implementation crate likely to change incompatibly with releases. Its presence on crates.io should not be interpreted as a stable embeddable test-runner SDK.

Sources:

- `https://docs.rs/nextest-metadata/latest/nextest_metadata/`
- `https://docs.rs/nextest-filtering/latest/nextest_filtering/`
- `https://docs.rs/nextest-filtering/latest/nextest_filtering/struct.Filterset.html`
- `https://nexte.st/docs/stability/`&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Rust+3



## 14. Installation, compiler compatibility, and stability

### 14.1 Distribution and toolchain requirements

Nextest provides prebuilt binaries for supported Linux, macOS, and Windows platforms. It can also be installed using Cargo or supported binary installation tooling.

For source installation, the supported command is:

`cargo install cargo-nextest --locked`

Since 0.9.124, installing from crates.io without `--locked` is explicitly unsupported.

Nextest distinguishes the Rust toolchain used to compile nextest from the Rust toolchain used to compile and execute application tests:

- Building cargo-nextest 0.9.150 requires Rust 1.91 or newer.
- Executing tests against older Rust toolchains is supported much more broadly; Rust 1.41 is known to have worked, though that compatibility range is not actively tested.

Consequently, a prebuilt nextest binary can be useful when the project's toolchain is older than nextest's own compilation requirement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources: `https://nexte.st/docs/installation/pre-built-binaries/` and `https://nexte.st/docs/installation/from-source/`

### 14.2 Public API stability

Within the 0.9.x series, nextest treats these interfaces as its public compatibility surface:

1. CLI arguments, commands, options, and flags.
2. Supported machine-readable output formats.
3. Repository configuration format.

Its normal policy is to evolve those interfaces additively. Documented exceptions permit corrections for bugs and security issues.

Human-readable output, stderr presentation, internal crates, USDT probes, and experimental functionality do not share the same guarantees. The experimental recording-store change in 0.9.150 is a concrete example of why those distinctions matter.

Source: `https://nexte.st/docs/stability/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



## 15. Consequential architectural distinctions

The strongest reason to investigate nextest is not merely its potential speed improvement. It provides several control mechanisms that can be combined without introducing bespoke test infrastructure.

| Problem class                      | Native nextest composition                                  | Remaining boundary                                                      |
| ---------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------- |
| Large heterogeneous test suites    | Global scheduling + weighted slots + priorities             | Process startup cost and workload-specific performance                  |
| Shared external fixtures           | Test groups + per-test overrides + setup scripts            | Groups are invocation-local; external resource cleanup remains separate |
| Intermittent failures              | Scoped retries + backoff + flaky classification             | Recovery does not establish determinism                                 |
| Hung integration tests             | Slow warnings + termination policies + global timeout       | Child-process cleanup and operating-system differences                  |
| Targeted tests in large workspaces | Filtersets + dependency predicates + binary-level rejection | Package-graph targeting is not source-level impact analysis             |
| Multi-worker CI                    | Build archives + partitioning + per-worker scheduling       | Artifact transfer, infrastructure and result aggregation                |
| Failure investigation              | Recorded runs + replay/rerun + Perfetto                     | Experimental formats, sensitive output, version compatibility           |
| Test instrumentation               | Target runners + wrapper scripts + debugger/tracer modes    | Different stability and process-lifecycle semantics                     |
| Automated test intelligence        | JSON inventory + `nextest-metadata` + `nextest-filtering`   | Execution-event interfaces are less mature                              |
| Verification beyond ordinary tests | Miri + coverage + mutation testing + benchmark modes        | The specialized analysis belongs to companion tooling                   |

### Overall capability interpretation

Nextest's principal architectural strengths are threefold.

First, execution isolation and scheduling. By turning each test into an independently managed process, nextest can apply scheduling, resource, timeout, and failure policies at a finer granularity than conventional per-binary execution.

Second, declarative test semantics. Filtersets, profiles, test groups, per-test overrides, and platform-specific rules allow complex execution policies to be expressed as structured configuration rather than dispersed custom scripts.

Third, separation of execution from analysis and distribution. Compiled archives, machine-readable inventories, CI partitioning, persistent recordings, replay, and traces expose reusable boundaries for external orchestration and diagnostics.

The most important limitations are equally structural: no native doctest execution, process startup overhead, incomplete isolation of external resources, dependence on compatible test harnesses, no generic distributed resource locking, and experimental status for some of the richest execution-history and interception capabilities.

Bottom line: nextest is best understood as a specialized Rust test execution, scheduling, and observability layer, not simply a faster replacement command for `cargo test`. Its established features can substantially reduce custom test orchestration machinery; its experimental extensions suggest further opportunities, but should be treated separately from its stable execution and configuration interfaces.

The official architecture, configuration, filterset, CI, and integration documentation cited throughout provides the primary technical reference for those distinctions.

# cargo-nextest — Technical Capability Addendum

Target version: cargo-nextest 0.9.151 (stable)\
Research date: November 8, 2026\
Companion document: cargo-nextest — Comprehensive Rust Test Runner Reference (originally targeting 0.9.150)

This addendum extends the original reference with additional capabilities, operational semantics, and integration interfaces. Section references indicate where the material complements the existing guide. Established capabilities are distinguished from experimental functionality, and version-sensitive corrections are documented explicitly.

## 1. Version updates and corrections to the existing reference

### 1.1 Version 0.9.151: scheduler correctness and build reuse

Version 0.9.151, released November 2, 2026, is the latest stable release listed in the upstream changelog as of this research date. It introduces two correctness fixes relevant to existing capabilities.

Test-group scheduling correctness. When using named test groups together with `threads-required`, a test waiting for group capacity could previously be delayed after sufficient global capacity became available. This occurred when capacity was released by a test outside the waiting test's group. If no other test in the group remained active, the run could terminate without ever executing the waiting test.

Version 0.9.151 corrects the wake-up and scheduling behavior so that eligible waiting tests are started once sufficient capacity becomes available.

This is an important distinction from ordinary scheduler efficiency: the earlier defect could affect test-execution completeness, not merely execution order or elapsed time.

Custom-target build reuse. Version 0.9.151 also fixes a panic when executing previously compiled tests through `--archive-file` or `--binaries-metadata` for certain custom targets supplied through `rustc --print=cfg`.

This addresses a compatibility problem in the build-reuse execution path rather than changing the archive format itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.nexte.st\&sz=32)

cargo-nextest



The security fix and recording-store compatibility changes introduced in 0.9.150 remain applicable.

Official changelog: `https://nexte.st/changelog/`

### 1.2 Corrections to defaults and companion versions

Two existing details should be superseded:

| Original guide location          | Corrected information                                                                                                |
| -------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| §6.3 — Subprocess leak detection | The default test `leak-timeout` is 200 ms, not 100 ms. Setup scripts have a separate 100 ms default.                 |
| §13.3 — Companion Rust crates    | `nextest-filtering` 0.23.0 supersedes the documented 0.22.0. The current nextest-runner dependency specifies 0.23.0. |

The leak-timeout correction is significant because the test and setup-script phases have independent configuration defaults.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.nexte.st\&sz=32)

cargo-nextest+3



Sources:

- Repository configuration reference: `https://nexte.st/docs/configuration/reference/`
- Nextest-filtering crate manifest: `https://docs.rs/crate/nextest-filtering/latest/source/Cargo.toml.orig`

## 2. Default-filter semantics and test-selection completeness

Extends original §3.3 — Filtering composition and discovery optimization.

### 2.1 The default filter is an implicit constraint on CLI selection

A configured `default-filter` does more than determine which tests are selected when no explicit filter is supplied.

Beginning with nextest 0.9.77, command-line filterset expressions are evaluated within the effective default-filter set unless the caller explicitly bypasses that constraint.

For example:

```
[profile.default]
default-filter = 'not package(special-tests)'

[profile.ci]
default-filter = 'all()'
```

Under the default profile:

```
cargo nextest run -E 'all()'
```

does not select tests from `special-tests`, even though `all()` matches the entire universe of discoverable tests when considered independently.

The effective selection is conceptually:

\\[ S\_{\text{selected}} = S\_{\text{CLI filtersets}} \cap S\_{\text{default filter}} \\]

Additional restrictions from Cargo build scope, positional name filters, ignored-test handling, partition selection, and rerun selection can narrow the result further.

This behavior means that `all()` is not necessarily a declaration of exhaustive test execution. It is only one component of the effective selection policy.

### 2.2 `--ignore-default-filter` and `default()` have different meanings

The following command bypasses the implicit default-filter constraint:

```
cargo nextest run \
  -E 'all()' \
  --ignore-default-filter
```

The flag does not change the meaning of the `default()` predicate. An expression that explicitly contains `default()` continues to evaluate against the configured default set.

This creates an important distinction:

| Mechanism                 | Effect                                                                                        |
| ------------------------- | --------------------------------------------------------------------------------------------- |
| `default-filter`          | Establishes the profile's default test-selection set                                          |
| `-E 'all()'`              | Selects all tests within the otherwise applicable CLI constraints                             |
| `--ignore-default-filter` | Removes implicit intersection with the configured default filter                              |
| `default()`               | Explicitly refers to the configured default set inside a filterset expression                 |
| `--run-ignored all`       | Includes both normally runnable and `#[ignore]` tests; independent of default-filter handling |

Consequently, bypassing the default filter alone does not automatically include ignored tests, and including ignored tests alone does not bypass the default filter.

Even with both mechanisms enabled, execution remains limited to tests discoverable within the selected Cargo packages, targets, features, and platforms. It does not add unsupported doctest execution.

### 2.3 Configuration expressions are evaluated differently from CLI expressions

Filtersets specified within nextest configuration are not automatically intersected with `default-filter`.

This applies, for example, to filtersets used for per-test overrides or setup-script selection. If such a rule needs to restrict itself to the default set, its filter expression must explicitly include `default()`.

The distinction prevents recursive application of default-selection policy to configuration rules, but also means that an override can match a test that ordinary default-profile execution would exclude.

This is useful when defining policies for test classes normally executed only in CI or specialized environments.

### 2.4 Platform-specific default filters

Nextest can replace a profile's default filter according to the host or target platform.

For example:

```
[profile.default]
default-filter = 'all()'

[[profile.default.overrides]]
platform = 'cfg(windows)'
default-filter = 'not package(unix-integration)'
```

A `default-filter` override must specify `platform` and cannot simultaneously specify a test `filter`. This is a platform-level change to selection policy, not a per-test execution-setting override.

### 2.5 Selection diagnostics

Nextest reports the number of tests and binaries skipped because of the effective default filter.

For programmatic consumers, `nextest-metadata` represents this distinction through `FilterMatch::Mismatch` and the `MismatchReason::DefaultFilter` variant, allowing tools to distinguish tests excluded by the default profile from tests excluded through an explicit filter expression or partition.

This provides the basis for explaining why a discovered test was not selected without independently reimplementing the filterset algebra.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+2



- Test selection: `https://nexte.st/docs/selecting/`
- Filterset semantics: `https://nexte.st/docs/filtersets/`
- Filterset DSL reference: `https://nexte.st/docs/filtersets/reference/`

## 3. Configuration introspection, validation, and hierarchical composition

Extends original §4 — Declarative configuration and policy resolution.

### 3.1 Native configuration-inspection commands

Nextest provides commands for inspecting specific resolved configuration properties without executing tests.

```
# Inspect matching tests and named test-group assignments.
cargo nextest show-config test-groups

# Inspect the installed version against configured requirements.
cargo nextest show-config version

# Display locally bundled configuration references.
cargo nextest help repo-config
cargo nextest help user-config

# Display the filterset language reference.
cargo nextest help filterset
```

`show-config test-groups` accepts the same test-selection options as `cargo nextest list`. It can therefore be scoped to a package, target, filterset, or profile.

Its output associates group limits and selected tests with the override rules responsible for assigning those groups. This is useful for diagnosing unintentionally serialized tests, tests assigned to the wrong resource group, and conflicts between overlapping group-selection rules.

The command provides visibility into the interaction between policy and discovered tests, rather than simply printing raw TOML.

`show-config version` evaluates required and recommended minimum versions. It has distinctive exit semantics:

- `0`: requirements satisfied or checks explicitly bypassed.
- `10`: recommended minimum version not met.
- `92`: required minimum version not met.

Exit code `10` is advisory; it is not equivalent to a failed test run. An automation using `show-config version` must account for that distinction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources:

- Test-group inspection: `https://nexte.st/docs/configuration/test-groups/`
- Version inspection: `https://nexte.st/docs/configuration/minimum-versions/`

### 3.2 Machine-readable configuration schemas

Nextest publishes JSON Schemas for both repository-level and user-level configuration.

The installed executable can emit schemas corresponding to its own version:

```
cargo nextest self schema repo-config
cargo nextest self schema user-config
```

The corresponding public URLs are:

`https://nexte.st/schemas/repo-config.json`

`https://nexte.st/schemas/user-config.json`

The public URLs track the latest release, whereas schemas emitted by the local binary correspond to that binary's version.

This makes it possible to validate configuration against the version actually executing tests, even when a repository intentionally uses an older nextest version.

The two schemas cover different configuration domains:

| Schema        | Scope                                                                                                                     |
| ------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `repo-config` | Profiles, test groups, per-test overrides, setup scripts, execution policies, and version requirements                    |
| `user-config` | Progress presentation, pager behavior, interactive input, recording preferences, and platform-specific personal overrides |

Both support schema-driven completion and validation through compatible editors.

Official documentation identifies the Tombi TOML language server and RustRover as supported consumers. It identifies a crash bug in Taplo as preventing its use for this integration.

Schema validation is stricter than nextest's runtime parser. Unknown configuration keys fail schema validation, while the application normally emits warnings for them.

This distinction matters for forward compatibility: a configuration file may be accepted by a runner but rejected by a strict schema validator, especially when experimental or newly introduced settings are involved.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources:

- Repository schema: `https://nexte.st/docs/configuration/reference/`
- User schema: `https://nexte.st/docs/user-config/reference/`

### 3.3 Tool-specific configuration as a lower-priority policy layer

Tools invoking nextest can provide their own configuration files using:

```
cargo nextest run \
  --tool-config-file my-tool:/absolute/path/to/nextest.toml
```

The argument has the form `TOOL:ABS_PATH`, permitting a higher-level tool to supply named defaults independently of the repository configuration.

The hierarchy is more nuanced than simply loading configuration files in sequence.

For an applicable setting, the main precedence is:

1. Command-line arguments.
2. Environment variables.
3. Matching per-test overrides, evaluated through the selected profile and inheritance chain.
4. Repository configuration for the selected profile.
5. Tool-specific configuration for that profile.
6. Corresponding repository and tool-specific settings inherited from parent profiles.
7. Nextest's embedded defaults.

Within a particular profile level, repository configuration takes precedence over tool-specific configuration. If multiple tool configuration files are supplied, earlier files take precedence over later ones.

This architecture lets external tools establish execution defaults while preserving the repository's authority over its own policies.

Minimum-version requirements are an exception. Nextest considers requirements from all repository and tool-specific configuration files, instead of allowing a lower-priority requirement to be masked by a higher-priority value. The strongest applicable minimum requirement must therefore be met.

This is significant for integrations that depend on functionality introduced in specific nextest versions: the integration can declare its own minimum independently of the repository's configuration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources:

- Configuration hierarchy: `https://nexte.st/docs/configuration/`
- Minimum-version requirements: `https://nexte.st/docs/configuration/minimum-versions/`

## 4. Build-directory remapping and compiler-message interfaces

Extends original §8.3 and §10.3 — Manual metadata-based execution and machine-readable interfaces.

### 4.1 Separate Cargo target and build directories

Cargo's `build.build-dir` configuration allows intermediate build artifacts to reside separately from the target directory containing final artifacts.

Nextest supports this distinction through three independent relocation controls:

| Option               | Relocated resource                                                                  |
| -------------------- | ----------------------------------------------------------------------------------- |
| `--workspace-remap`  | Workspace and source-tree location                                                  |
| `--target-dir-remap` | Target directory containing compiled artifacts                                      |
| `--build-dir-remap`  | Separate build directory containing intermediate outputs and build-script artifacts |

`--build-dir-remap` becomes relevant when compiled tests are executed using reusable build metadata or relocated artifacts.

If it is not provided, nextest falls back to the target-directory remapping.

The distinction matters because a test executable and its runtime dependencies are not necessarily located in the same artifact tree. A test may rely on paths generated by build scripts, non-test binaries, dynamic libraries, or other build outputs.

For example, a compilation environment might produce final binaries under one persistent artifact directory and intermediate build products under a separate cache volume. A remote execution environment must reconstruct both paths correctly.

Nextest's remapping facilities allow those locations to be represented independently instead of requiring a custom mechanism to rewrite every path embedded in the build metadata.

### 4.2 Dynamic-library resolution across Cargo artifact layouts

Nextest reconstructs Cargo-compatible dynamic-library search paths from compiled artifact metadata.

Depending on the platform, the effective search path is supplied through `PATH`, `DYLD_FALLBACK_LIBRARY_PATH`, or `LD_LIBRARY_PATH`.

The reconstructed paths can include the final artifact directory, dependency directory, selected build-script linkage paths, and Rust sysroot library directories.

Two distinctions are important.

First, Cargo's target directory and build directory need not be identical. Under a split-directory configuration, final executables and dynamic libraries may reside outside the tree containing intermediate compiler outputs.

Second, Cargo's build-directory layout v2 changes the traditional expectation that dynamically linked artifacts reside under a separate `deps` directory. Under that layout, the relevant libraries reside directly in the artifact directory.

Consequently, any external system reconstructing a test execution environment from compiled metadata must account for the Cargo artifact layout as well as the host/target platform distinction.

This is a reason to preserve nextest's structured build metadata rather than infer library paths solely from conventional `target/debug/deps` locations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+2



Sources:

- Archiving and build reuse: `https://nexte.st/docs/ci-features/archiving/`
- Runtime environment reconstruction: `https://nexte.st/docs/configuration/env-vars/`

### 4.3 Compilation-only operation

The `--no-run` option allows nextest to invoke Cargo compilation without proceeding to test execution:

```
cargo nextest run --no-run
```

This separates compilation from the remainder of the execution workflow.

It is distinct from obtaining a test inventory through `cargo nextest list`, because compiling the test targets does not itself provide the per-test enumeration and selection information required for execution scheduling.

The separation is useful for build-validation workflows and external tools that need to control when test execution occurs independently from compilation.

### 4.4 Cargo compiler messages versus nextest execution messages

Nextest exposes two distinct mechanisms for controlling structured output.

Cargo compilation diagnostics

```
cargo nextest run \
  --no-run \
  --cargo-message-format=json
```

Introduced in 0.9.123, `--cargo-message-format` accepts Cargo-compatible message-format options, including `human`, `short`, `json`, and diagnostic-rendering variants.

It controls how messages from Cargo's build phase are reported, including forwarding structured JSON messages to standard output.

Nextest test-execution events

```
NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 \
  cargo nextest run \
  --message-format libtest-json-plus
```

This is a separate interface for execution events. The `libtest-json` and `libtest-json-plus` formats remain experimental.

The two interfaces have different producers and different schemas.

| Interface                                  | Information represented                                   | Stability                              |
| ------------------------------------------ | --------------------------------------------------------- | -------------------------------------- |
| `--cargo-message-format=json`              | Cargo compilation artifacts and diagnostics               | Established Cargo-compatible interface |
| `cargo nextest list --message-format json` | Discovered binaries, tests, and selection metadata        | Stable nextest interface               |
| `--message-format libtest-json-plus`       | Test execution events and nextest-specific event metadata | Experimental                           |

A system consuming multiple phases should distinguish compilation, discovery, and execution records rather than treating them as one uniform event stream.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



`https://nexte.st/docs/running/`

## 5. Structured exit statuses and programmatic metadata

Extends original §10.3 and §13.3.

### 5.1 `NextestExitCode` as a typed failure taxonomy

The `nextest-metadata` crate exposes `NextestExitCode`, a collection of documented exit-code constants describing expected outcomes and failure classes.

The primary distinction is between a failure in the tested program and a failure in the infrastructure used to compile, discover, or execute that program.

| Exit code | Constant                           | Meaning                                                     |
| --------- | ---------------------------------- | ----------------------------------------------------------- |
| `0`       | `OK`                               | Successful execution                                        |
| `1`       | —                                  | Unexpected or unclassified error                            |
| `4`       | `NO_TESTS_RUN`                     | No tests selected, under a policy treating this as an error |
| `5`       | `RERUN_TESTS_OUTSTANDING`          | Executed rerun tests passed, but unresolved tests remain    |
| `10`      | `RECOMMENDED_VERSION_NOT_MET`      | Advisory minimum-version check failure                      |
| `70`      | `DOUBLE_SPAWN_ERROR`               | Failure during nextest process re-execution                 |
| `80`      | `UPDATE_AVAILABLE`                 | Update detected by `self update --check`                    |
| `81`      | `UPDATE_DOWNGRADE_NOT_PERFORMED`   | Requested downgrade not performed                           |
| `82`      | `UPDATE_CANCELED`                  | User canceled the update                                    |
| `90`      | `UPDATE_ERROR`                     | Error downloading an update                                 |
| `92`      | `REQUIRED_VERSION_NOT_MET`         | Minimum required nextest version not satisfied              |
| `93`      | `SELF_UPDATE_UNAVAILABLE`          | Self-update not available in the installed build            |
| `94`      | `INVALID_FILTERSET`                | Invalid filterset expression                                |
| `95`      | `EXPERIMENTAL_FEATURE_NOT_ENABLED` | Experimental functionality requested without activation     |
| `96`      | `SETUP_ERROR`                      | Error configuring or preparing the invocation               |
| `100`     | `TEST_RUN_FAILED`                  | One or more tests failed                                    |
| `101`     | `BUILD_FAILED`                     | Cargo test compilation failed                               |
| `102`     | `CARGO_METADATA_FAILED`            | Cargo metadata acquisition failed                           |
| `103`     | `ARCHIVE_CREATION_FAILED`          | Compiled-test archive creation failed                       |
| `104`     | `TEST_LIST_CREATION_FAILED`        | Test discovery or list construction failed                  |
| `105`     | `SETUP_SCRIPT_FAILED`              | Setup-script execution failed                               |
| `106`     | `INCOMPLETE_RUN`                   | Replay requested for an incomplete recording                |
| `110`     | `WRITE_OUTPUT_ERROR`               | Failure writing to stdout or stderr                         |

These constants are part of a publicly documented Rust metadata interface, rather than conventions inferred from terminal output.

The taxonomy enables external orchestration to make several consequential distinctions.

A compilation failure (`101`) is not a test failure (`100`). A failure to construct the test inventory (`104`) means the selected tests may not have been executed at all. A setup-script failure (`105`) indicates that the execution environment was not prepared successfully.

Similarly, exit code `5` represents incomplete rerun-chain validation despite successful execution of the tests in the current invocation.

An exit code of `0` indicates success under the applicable execution policy. It does not establish that every theoretically possible test was included, because selection scope, default filters, ignored-test handling, and Cargo target selection remain independent constraints.

The documented constants can therefore support more precise CI reporting, retry classification, and failure attribution than treating all nonzero statuses as equivalent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

Rust



Source: `https://nexte.st/rustdoc/nextest_metadata/enum.NextestExitCode`

### 5.2 `FilterMatch` and explicit mismatch reasons

The `nextest-metadata` crate also exposes structured information explaining why discovered tests were included or excluded.

The relevant types are:

```
pub enum FilterMatch {
    Matches,
    Mismatch {
        reason: MismatchReason,
    },
}
```

`MismatchReason` distinguishes multiple exclusion mechanisms:

| Variant              | Interpretation                                         |
| -------------------- | ------------------------------------------------------ |
| `NotBenchmark`       | Test excluded when selecting benchmarks                |
| `Ignored`            | Test excluded by ignored-test handling                 |
| `String`             | Test failed positional or substring-name filtering     |
| `Expression`         | Test failed filterset-expression matching              |
| `Partition`          | Test belongs to another selected partition             |
| `RerunAlreadyPassed` | Test previously passed in the applicable rerun history |
| `DefaultFilter`      | Test excluded by the configured default filter         |

This is more expressive than a simple selected/skipped Boolean.

For example, a test skipped because it belongs to a different CI partition should be interpreted differently from a test excluded because it does not satisfy the selected test expression.

The former may be scheduled elsewhere in a distributed execution campaign. The latter may be outside the intended logical test set entirely.

The `MismatchReason` enum is marked `#[non_exhaustive]`. Consumers must permit additional variants in future versions rather than assuming that the current set is closed.

The documented reason also reflects nextest's mismatch-classification semantics, rather than necessarily enumerating every possible condition that could have excluded a test.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

Rust+1



Sources:

- `FilterMatch`: `https://nexte.st/rustdoc/nextest_metadata/enum.FilterMatch`
- `MismatchReason`: `https://nexte.st/rustdoc/nextest_metadata/enum.MismatchReason`

### 5.3 `RustBuildMetaSummary` and build-environment reconstruction

`nextest-metadata::RustBuildMetaSummary` provides structured metadata for compiled Rust test artifacts.

Its fields include:

| Field                     | Information                                    |
| ------------------------- | ---------------------------------------------- |
| `target_directory`        | Cargo target directory                         |
| `build_directory`         | Optional separate Cargo build directory        |
| `base_output_directories` | Base directories containing build artifacts    |
| `non_test_binaries`       | Executables available to integration tests     |
| `build_script_out_dirs`   | Build-script output directories                |
| `build_script_info`       | Additional build-script metadata               |
| `linked_paths`            | Linked artifact search paths                   |
| `platforms`               | Structured host/target platform metadata       |
| `target_platforms`        | Target-platform information                    |
| `target_platform`         | Legacy optional target-platform representation |

These fields complement test-list metadata by describing the execution environment that produced the compiled test binaries.

This matters for tooling that separates compilation from execution: a test binary's path alone is insufficient to reconstruct every runtime dependency or environment variable required to execute it correctly.

The metadata structures make those requirements accessible without reconstructing Cargo's build behavior from filesystem conventions.

A useful architectural distinction is that `nextest-metadata` exposes a supported representation of the build and discovery outputs, while `nextest-runner` remains an internal implementation crate without the same embedding-stability guarantee.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

Rust



Source: `https://nexte.st/rustdoc/nextest_metadata/struct.RustBuildMetaSummary`

## 6. Runtime environment compatibility and environment-variable safety

Extends original §2.2 and §7.1.

### 6.1 Cargo-compatible executable paths

Since version 0.9.130, nextest provides `CARGO_BIN_EXE_<name>` at runtime for integration tests and benchmarks, matching Cargo's runtime executable-path convention.

This complements `NEXTEST_BIN_EXE_<name>`.

Both expose the absolute path to a compiled binary target, subject to the binary being built and available.

For example, an integration test can locate a companion executable through:

```
use std::env;

let executable = env::var("CARGO_BIN_EXE_my_program")
    .expect("expected compiled executable path");
```

The exact variable name follows the Cargo binary target's name, so a binary containing hyphens can produce an environment variable whose name also contains hyphens.

Nextest additionally exposes underscore-normalized variants of the `NEXTEST_BIN_EXE_*` variables, which can be more convenient for shells and tools that cannot reliably handle hyphens in variable names.

When compiled artifacts are relocated, nextest updates these runtime paths to refer to the relocated target directory.

An important distinction is that the runtime environment is reconstructed after compilation. Code using Rust's compile-time `env!` macro obtains the value embedded when the binary was compiled, which may refer to the original build environment. Code using `std::env::var` reads the runtime value and can therefore observe remapped paths.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/configuration/env-vars/`

### 6.2 Build-script outputs and Cargo package metadata

Nextest recreates several additional Cargo-related runtime values:

- `OUT_DIR`, when the test package has a build script.
- `CARGO_MANIFEST_DIR`, reflecting workspace remapping when applicable.
- `CARGO_PKG_*` variables derived from the package manifest.
- Environment variables defined through Cargo's configuration.
- Variables exported from build scripts through `cargo::rustc-env`.

This is an important compatibility mechanism for integration tests that use compiled resources, generated files, or package metadata.

However, Cargo discourages relying on `cargo::rustc-env` variables as a general runtime configuration mechanism. Their availability through nextest should not be confused with a guarantee that they constitute an appropriate or portable runtime configuration interface.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



`https://nexte.st/docs/configuration/env-vars/`

### 6.3 Runtime metadata during the test-listing phase

Beginning with version 0.9.138, several environment variables became available during test enumeration, not only when test cases are executed.

These include:

| Variable                      | Listing-phase purpose                                         |
| ----------------------------- | ------------------------------------------------------------- |
| `NEXTEST_RUN_ID`              | Correlates test discovery with the subsequent execution phase |
| `NEXTEST_VERSION`             | Exposes the current runner version                            |
| `NEXTEST_REQUIRED_VERSION`    | Exposes the configured required minimum version               |
| `NEXTEST_RECOMMENDED_VERSION` | Exposes the recommended minimum version                       |
| `NEXTEST_BINARY_ID`           | Identifies the binary being enumerated                        |
| `NEXTEST_WORKSPACE_ROOT`      | Provides the applicable workspace location                    |

Within one `cargo nextest run` invocation, test enumeration and execution share the same run ID.

This allows custom harnesses and target runners to correlate their discovery-phase operations with the subsequent execution phase.

A target runner that performs remote enumeration can therefore associate the enumeration request with the same logical run as the actual test invocation.

These variables do not imply that all per-test runtime context is available during enumeration. For example, a test's retry attempt and execution slot are properties of an actual execution attempt rather than of binary discovery.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



`https://nexte.st/docs/configuration/env-vars/`

### 6.4 Preserving dynamic-linker environment variables

Nextest exposes `NEXTEST_LD_*` and `NEXTEST_DYLD_*` variables that replicate relevant `LD_*` and `DYLD_*` environment values.

For example, `LD_PRELOAD` can have a corresponding `NEXTEST_LD_PRELOAD` value.

These names provide a mechanism for preserving dynamic-linker-related configuration in environments where operating-system behavior may sanitize the original variables.

This is particularly relevant on macOS, where System Integrity Protection can remove selected `DYLD_*` environment variables from subprocess environments.

The mirrored variables are exposed on all supported platforms for consistency.

They are not a general guarantee that dynamic-linker settings are restored automatically to their original effect. A wrapper or other consumer may need to interpret the preserved values and reconstruct the appropriate launch environment.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



`https://nexte.st/docs/configuration/env-vars/`

### 6.5 Process isolation and Rust's unsafe environment mutation APIs

Rust's `std::env::set_var` and `std::env::remove_var` can be unsafe when multiple threads access the process environment concurrently.

On Linux and other Unix platforms where environment mutation is not generally thread-safe, this creates difficulties for conventional test execution when multiple tests share a process.

Nextest's process-per-test architecture changes the analysis substantially.

A standard `#[test]` executed under nextest normally has a dedicated test process containing a main harness thread and a separate test thread.

Although that process is technically multithreaded, current `libtest` behavior does not access the environment from the main harness thread while the test thread is executing.

Consequently, mutating the environment at the beginning of a standard test can satisfy the relevant safety requirements, provided the test has not created additional threads that may access the environment concurrently.

This is a conditional property of the execution arrangement, not a guarantee that `set_var` is always safe inside nextest tests.

Custom test harnesses may perform additional initialization or create threads before individual tests execute. Tests using procedural-macro wrappers, including asynchronous test frameworks, can introduce additional runtime behavior.

The upstream documentation specifically considers:

- Ordinary `#[test]` functions.
- `libtest-mimic` and harnesses built on top of it.
- `datatest-stable`.
- `#[tokio::test]`, including the multithreaded runtime flavor.

For Tokio tests, upstream documents circumstances in which beginning-of-test environment mutation remains safe because runtime worker threads are not concurrently accessing the environment. That conclusion depends on the runtime and test code maintaining those conditions.

The important architectural advantage is that process isolation removes concurrency between different tests at the environment-variable level. It does not remove concurrency introduced within an individual test process.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



`https://nexte.st/docs/configuration/env-vars/`

## 7. Setup-script capture, diagnostics, and JUnit representation

Extends original §7.2 — Setup scripts.

Feature status: Experimental.

### 7.1 Separate capture policies for setup scripts

Setup scripts support independent standard-output and standard-error capture controls:

```
[scripts.setup.prepare-environment]
command = 'scripts/prepare.sh'

capture-stdout = true
capture-stderr = true

slow-timeout = {
    period = "30s",
    terminate-after = 2
}

leak-timeout = "500ms"
```

Both `capture-stdout` and `capture-stderr` default to `false`.

The default timeout policies also differ from those used for ordinary tests.

| Setting          | Setup-script default |
| ---------------- | -------------------- |
| `slow-timeout`   | Unset                |
| `leak-timeout`   | `100ms`              |
| `capture-stdout` | `false`              |
| `capture-stderr` | `false`              |

A setup-script timeout cannot be configured as a successful outcome through `on-timeout = "pass"`. This preserves a distinction between intentionally time-bounded test behavior and incomplete environment preparation.

### 7.2 Captured setup output in JUnit

If JUnit reporting is enabled, setup scripts are represented as synthetic test suites.

Each setup script has its own `<testsuite>` with a name beginning `@setup-script:` and a corresponding test case.

When output capture is enabled, captured stdout and stderr can be represented through `<system-out>` and `<system-err>` elements.

Setup scripts have independent JUnit retention settings:

```
[scripts.setup.prepare-environment]
command = 'scripts/prepare.sh'

capture-stdout = true
capture-stderr = true

junit.store-success-output = true
junit.store-failure-output = true
```

Both JUnit output-retention settings default to `true` for setup scripts, unlike ordinary tests, where successful output is not retained by default.

The distinction is that capture determines whether output is available, while JUnit retention determines whether captured output is included in the resulting report.

JUnit also records setup-script command information and environment variables exported through the setup mechanism.

For systems inspecting test infrastructure failures, these records provide a distinct source of diagnostic evidence from the application test cases themselves.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/configuration/setup-scripts/`

## 8. Interactive diagnostics and user-specific execution controls

Extends original §10.1 — Terminal reporting and output control.

### 8.1 Interactive inspection of active tests

Nextest has a built-in terminal input handler that accepts keyboard commands while a test run is executing.

| Input  | Effect                                                                    |
| ------ | ------------------------------------------------------------------------- |
| `t`    | Prints status and available captured output for currently running tests   |
| Enter  | Prints a progress snapshot or current execution-state line                |
| Ctrl-T | Requests status through `SIGINFO` on supported platforms, including macOS |

The input handler makes long-running tests observable without terminating them or attaching a debugger.

Its behavior is implemented through terminal input events coordinated with nextest's event-driven dispatcher.

On Unix, nextest changes selected terminal settings to allow character-by-character input without echoing ordinary keystrokes. It preserves conventional signal handling for controls such as Ctrl-C and Ctrl-Z.

The input handler operates only in appropriate interactive terminal contexts. It can be disabled through:

```
cargo nextest run --no-input-handler
```

or through user configuration:

```
[ui]
input-handler = false
```

This is relevant when nextest is executed under another interactive program that needs to retain control of terminal input.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/design/architecture/input-handling/`

### 8.2 User-level UI configuration

User configuration provides persistent settings for presentation behavior independently of the repository's execution policy.

For example:

```
[ui]
show-progress = "bar"
max-progress-running = 16
output-indent = true
input-handler = true
```

`max-progress-running` controls how many currently executing tests are individually displayed in the progress interface.

It accepts a positive integer, `0` to hide the individual running-test list, or `"infinite"` to display all running tests.

This does not change the scheduler's concurrency limit. It changes only how much of the scheduler's active state is visible.

### 8.3 Platform-specific user configuration

Since version 0.9.120, user configuration supports platform-specific overrides.

```
[ui]
show-progress = "auto"
max-progress-running = 16

[[overrides]]
platform = "cfg(windows)"
ui.max-progress-running = 4

[[overrides]]
platform = "cfg(target_os = \"macos\")"
ui.show-progress = "bar"
```

Overrides are evaluated in declaration order, with the first matching override supplying each individual setting.

An important distinction is that these user-level platform conditions are matched against the platform on which nextest itself was built.

They are not the same as per-test platform overrides, which can distinguish the host and target platforms associated with compiled tests.

The user-configuration precedence is:

CLI argument → environment variable → matching user override → base user configuration → embedded default.

This makes user presentation configurable independently from repository test policy.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/user-config/reference/`

### 8.4 Pager integration

Nextest supports both external and built-in pagers.

Paging is available for commands such as:

- `cargo nextest list`
- `cargo nextest replay`
- `cargo nextest store list`
- `cargo nextest show-config test-groups`
- Help output

On Unix, the default pager is based on `less`; Windows uses a built-in pager.

Paging is disabled automatically when standard output is noninteractive or a machine-readable output format is requested.

The user configuration supports string, array, and structured command specifications:

```
[ui]
pager = ["less", "-FRX"]
paginate = "auto"
```

The built-in pager can be selected through `pager = ":builtin"` and customized using `[ui.streampager]` settings for wrapping, screen behavior, and ruler presentation.

A significant integration detail is that nextest deliberately ignores the generic `PAGER` environment variable. Its pager behavior is controlled through nextest-specific configuration and CLI options.

This avoids external pager preferences unexpectedly changing nextest's interactive output behavior.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



`https://nexte.st/docs/user-config/pager/`

## 9. Native editor and IDE integrations

Extends original §12 — Integrations with the Rust testing ecosystem.

### 9.1 VS Code and rust-analyzer

Nextest supports direct integration with VS Code through rust-analyzer's test runnable override facility.

The relevant setting is:

`rust-analyzer.runnables.test.overrideCommand`

This replaces the command rust-analyzer uses to launch tests from its editor interface.

The documented configuration uses argument placeholders such as `${package}`, `${target_arg}`, `${target}`, `${test_name}`, `${exact}`, and `${include_ignored}` to preserve the selected test's Cargo context.

For example, the configured command begins:

```
{
  "rust-analyzer.runnables.test.overrideCommand": [
    "cargo",
    "nextest",
    "run",
    "--package",
    "${package}",
    "${target_arg}",
    "${target}",
    "--",
    "${test_name}",
    "${exact}",
    "${include_ignored}"
  ]
}
```

This lets editor-initiated test runs use nextest's execution infrastructure while retaining rust-analyzer's source-level test discovery and command construction.

The current integration documentation identifies rust-analyzer 0.3.2862 or newer as a requirement.

### 9.2 Debugging in VS Code with CodeLLDB

Running tests through rust-analyzer and debugging them through an IDE are related but distinct operations.

An ordinary VS Code debugging session does not automatically reconstruct nextest-specific environment variables or values produced by nextest setup scripts.

For tests that depend on those values, nextest's debugger integration can be combined with CodeLLDB.

The documented approach uses the `codelldb-launch` tool and CodeLLDB's RPC server to launch a test through nextest while allowing the IDE to control the debugging session.

This preserves nextest's responsibility for selecting the test and constructing the execution environment.

The distinction matters when the behavior being debugged depends on setup-generated configuration, dynamically located executables, or other runtime state specific to nextest.

### 9.3 RustRover native integration

RustRover 2026.1 and newer support selecting nextest as the Cargo test runner.

The IDE can invoke nextest through its normal test execution interface.

RustRover also supports automatic configuration validation and completion for nextest's repository and user configuration through published JSON Schemas.

However, upstream currently documents that RustRover does not support debugging tests with nextest's execution environment configured.

Thus, native test execution integration is not equivalent to complete debugger-environment integration.

These capabilities belong to the IDE and its integrations with nextest; they are not separate Rust testing frameworks supplied by nextest itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest+1



Sources:

- VS Code: `https://nexte.st/docs/integrations/vscode/`
- RustRover: `https://nexte.st/docs/integrations/rustrover/`
- Debuggers and tracers: `https://nexte.st/docs/integrations/debuggers-tracers/`

## 10. Self-update and distribution controls

Extends original §14.1 — Distribution and toolchain requirements.

### 10.1 Native self-update

Nextest provides a built-in update mechanism:

```
cargo nextest self update
```

For compatible prebuilt installations, this retrieves and installs a newer cargo-nextest binary.

The updater supports explicit versions, version ranges, and prerelease channels.

```
# Check whether a newer version is available.
cargo nextest self update --check

# Request a specific version.
cargo nextest self update --version 0.9.151

# Include beta releases.
cargo nextest self update --beta

# Include release candidates, but not betas.
cargo nextest self update --rc
```

The `--beta` and `--rc` capabilities were introduced in version 0.9.120.

If no appropriate newer prerelease exists, the corresponding channel operation can select the latest stable release.

With `--check`, nextest exits with code `80` if an update is available and `0` if none is available.

### 10.2 Distribution-specific behavior

Self-update is not necessarily available in every distribution of nextest.

Source installations can be updated through:

```
cargo install cargo-nextest --locked
```

Operating-system package installations generally use their distributor's package manager.

For distribution builds, nextest provides the `default-no-update` Cargo feature, which enables its ordinary default features without including self-update support.

The upstream-supported build pattern is:

```
cargo install cargo-nextest \
  --locked \
  --no-default-features \
  --features default-no-update
```

This distinction allows distributors to retain control over binary updates rather than allowing the installed application to replace itself independently.

It also explains why the availability of `cargo nextest self update` depends on how the executable was built or installed, not solely on its version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://nexte.st\&sz=32)

cargo-nextest



Source: `https://nexte.st/docs/installation/updating/`

## 11. Architectural implications of the additional capabilities

The additions above reveal several further properties of nextest's architecture that are useful when evaluating its role as an execution and integration layer.

### 11.1 Configuration is both executable policy and inspectable data

Default filters, scoped overrides, configuration schemas, and native inspection commands form a declarative policy system with identifiable resolution semantics.

The value extends beyond expressing policy in TOML. Tooling can validate configurations, identify test-selection exclusions, inspect group assignments, enforce version requirements, and reuse the filterset parser.

This reduces the amount of custom logic required to explain why a test was selected, skipped, or assigned a particular execution constraint.

A limitation remains: the documented inspection commands expose specific aspects of effective configuration, not a universal structured dump of every resolved per-test execution property. Tooling requiring more comprehensive introspection may still need to combine configuration interpretation with test metadata.

### 11.2 Build reuse depends on preserving the execution environment, not only the executable

Separate target/build-directory remapping, runtime Cargo environment variables, linked-library paths, and `RustBuildMetaSummary` show that nextest treats a compiled test as more than an executable path.

A test can depend on build-script outputs, non-test executables, package metadata, dynamic libraries, and source-tree locations.

The native metadata and remapping mechanisms make it possible to preserve that context across compilation and execution environments without requiring an external system to rediscover all of Cargo's behavior.

They do not eliminate platform compatibility, ABI, source-revision, or external-resource dependencies.

### 11.3 Execution outcomes have multiple independent dimensions

A test execution campaign can be successful or unsuccessful along several axes:

| Dimension                  | Relevant mechanism                                             |
| -------------------------- | -------------------------------------------------------------- |
| Build validity             | Cargo build diagnostics and build exit codes                   |
| Discovery completeness     | Structured test inventories and discovery errors               |
| Selection completeness     | Default filters, mismatch reasons, partitions, and rerun scope |
| Environment preparation    | Setup-script results                                           |
| Test correctness           | Test outcomes, flaky classifications, and timeouts             |
| Campaign completeness      | Recorded-run and rerun-chain state                             |
| Infrastructure correctness | Typed error statuses and artifact-reuse failures               |

These dimensions explain why a single process exit status or an aggregate JUnit report may be insufficient for sophisticated test-analysis tooling.

Nextest exposes several independent interfaces that can be combined to distinguish them, although the richer execution-event and recording interfaces remain experimental.

### 11.4 Process isolation changes correctness assumptions as well as scheduling

Process-per-test execution removes shared in-process state between separate tests, but it also changes the assumptions governing environment mutation, background threads, instrumentation, and custom harnesses.

These effects can make tests easier to isolate and reproduce, while simultaneously hiding interactions that would have occurred under a shared-process test runner.

The environment-variable safety analysis is a concrete example: process isolation reduces one source of concurrency, but does not automatically establish safety for all code executing within the individual test process.

The resulting architectural opportunity is to use nextest's native isolation and environment reconstruction where those semantics are appropriate, without assuming they reproduce every behavior of Cargo's standard shared-process test execution model.

Addendum status: All capabilities described here are documented for released nextest versions through 0.9.151, except where explicitly identified as experimental. The source URLs are direct upstream documentation or Rust API references. No tests or runtime measurements were performed as part of this research.