# Agent environment

This guide explains the checkout environment, native process ownership, preparation and
command selection. `AGENTS.md` owns working rules;
[Plan58](../plans/58-rust-owned-system.md) owns remaining acceptance. A prepared tool or a
focused passing control does not qualify the whole native system or the H campaigns.

## Preparing and activating a checkout

Construct/update the independent native host explicitly with `scripts/bootstrap-host`.
It builds locked host executables, validates the completed pair and atomically selects an
immutable generation under `target/host-tools/current`. Failed publication preserves the
previous selection. Help, activation, reads and launch never rebuild a missing executable.

`just bootstrap` prepares development tools. The checkout's Python environment contains
independent reference and third-party tools only: `pyproject.toml` is the private virtual
project `pse-arrow-reference-tools`, version `0.0.0`, with `package = false`. The default
`dev` group selects quality tools. Numerical oracles use explicit isolated environments;
there is no editable scientific extension, wheel or Python product to install.
`just bootstrap-docs` explicitly installs the declared documentation tools.

Ordinary Python tool commands use the default uv environment and declared `dev` group:

```bash
uv run ruff check scripts
uv run pyrefly check
uv run python -c 'import sys; print(sys.executable)'
```

`bash scripts/setup-env tools` and `just bootstrap-venv` explicitly prepare this checkout's
`.venv`, regardless of inherited Python/project selections. `bash scripts/setup-env reference`
selects only the parity group in `.venv-parity` with Python 3.13; it never replaces the dev
interpreter. `just reference-unit` also uses that isolated parity environment. Other scientific
reference recipes retain their explicit prefixes and interpreters. Bare Python/tool activation
is not an interface requirement; native helpers may retain their own explicit executable paths.

Review `.envrc`, then `direnv allow` in each checkout. Activation composes the native
export owner without syncing packages, running doctor or compiling providers.
`.envrc.local` is the ignored personal override. A checkout without its own environment must
not borrow another's. On this workstation the Codex Bash bridge initializes the machine
baseline once, then refreshes approved direnv settings without rebuilding unchanged activation.
It unsets `BASH_ENV` for workload children, which retain their native or oracle owner's selection.
Existing daemons without the initialized baseline use a safe compatibility path until a fresh
terminal-launched session. Claude Code's global SessionStart adapter sources the same bridge
for each Bash command after shell snapshot loading. Directory refresh reaches this checkout's
native export owner through `.envrc`; the local SessionStart hook no longer captures a fixed
checkout's native exports in `CLAUDE_ENV_FILE`. The local edit guard remains active. Review
changed activation files before allowing them.

Codex and Claude Code supply `UV_LOCKED=1`: ordinary `uv run` checks the lockfile and synchronizes required
packages while retaining extra installed packages. A stale lockfile fails rather than changing
it. Deliberate dependency changes use `UV_LOCKED=0 uv add 'name==x.y.z'`; `just lock-python`
and `just upgrade name` already opt into their intended lock mutation. No global no-sync,
frozen, interpreter or alternate-environment selection is required.

Run commands from this checkout or select another project deliberately. `uv run python
path/to/script.py` retains the selected project; `uv run path/to/script.py` discovers the
project from the script's location in uv 0.12+. `--project` selects project discovery without
changing relative argument paths; `--directory` also changes the working directory. Keep uv's
normal shared cache: use package-specific refresh/reinstall only to diagnose a concrete stale
artifact, rather than clearing the cache or disabling synchronization routinely.

## One native environment and foreground boundary

`scripts/pse-env` is a shell exec adapter over `pse-host boundary`. It starts no Python
controller, canonical server or scientific runtime. All command arguments remain literal data.

```bash
scripts/pse-env -- COMMAND ARGUMENTS
scripts/pse-env --docs -- COMMAND ARGUMENTS
scripts/pse-env --cache off -- COMMAND ARGUMENTS
scripts/pse-env --cache on -- COMMAND ARGUMENTS
scripts/pse-env --explain --json
scripts/pse-env --print
scripts/pse-host activity UNIT.service
```

Precedence is command selection, caller environment, `.envrc.local`, repository defaults.
Explicit empty values remain selections. Personal shell evaluation occurs once per explicit
activation in an isolated shell. `--print` emits a fixed Bash program that receives a private
NUL-delimited mapping, waits for the producer and validates the whole mapping before changing
the parent. The mapping is never evaluated as shell source. Failure leaves parent values
unchanged. Printed programs, explanations and debug views omit private values; source-free
explanation leaves unread personal choices unresolved.

The environment selects existing checkout-local tool paths, compiler inputs and native
sccache. It provisions no virtual environment or provider. Cache `auto` falls back to direct
compilation when optional cache prerequisites or supported modes are absent; required cache
selection refuses incompatible settings. `--cache off` disables wrapping even with an invalid
inherited binary. Successful compiler output remains successful if only the cache service
disappears; genuine compiler failure propagates. The Python per-compiler router is retired.

## Explicit compiler/KLU preparation

Native providers have explicit construction and selection:

```bash
scripts/pse-env -- scripts/pse-host native prepare --capabilities compiler,klu
scripts/pse-env --native=compiler,klu -- cargo t -p pse-runtime --lib --features pse-runtime/solver-kinsol -E 'test(root_response_publication_has_physical_primal_and_no_kkt_fields)'
scripts/pse-env -- scripts/pse-host server prepare
scripts/pse-host server select
just surreal-module
```

Supported capability names are `compiler,solver,klu,isolation,uno,petsc`. Preparation owns
source/tool observation and immutable publication; selection admits only completed compatible
artifacts. `--native=CAPABILITIES` selects them, bare `--native` uses
`PSE_NATIVE_CAPABILITIES` or the full default `solver,klu,isolation,uno,petsc`, and explicit
`--native=` selects native runtime settings without providers. Launch never constructs a
missing provider. Compiler/KLU has a bounded live scientific qualification basis; other
provider controls do not establish every scientific consumer's acceptance.

Native generations live under the selected cache root, with compiler/KLU in `rust-v1/kinsol`.
Failed construction preserves existing selections; automatic generation collection is excluded.
Owned staging is removable only after its work drains. Keep checkout-specific build targets
separate from the shared compiler cache. Supplier provenance and local corrections belong to
[vendor/native-supplier-notes.md](../../vendor/native-supplier-notes.md).

`server prepare` constructs the exact patched source server in authenticated containment.
`server prepare --from-release /absolute/release.json` explicitly imports a matching source
release, verifying the source/patch/profile tuple, original provenance and copied binary/lock
bytes. `server select` reads the completed immutable installation without compiling or
starting it. Server source patches remain maintained inputs. Schema readiness and service
ownership belong to the application, not host preparation.

`study-module prepare` uses the released server CLI for WASI compilation and packaging;
`study-module select --receipt /absolute/module.json` reads already retained bytes.
`NativeModuleSource::Prepared` transfers that capability into a native service's private state.
`--qualification` creates a separate probe identity. With `--long-hold`, preparation builds
an owned source snapshot with the alternate declared module manifest through the selected
released CLI. Production and long-hold construction have passed their selected local build
journeys; this establishes artifact construction only, not H saturation or performance
qualification. Refresh host tools explicitly after source changes.

## Native foreground ownership and supported placement

<a id="placement-and-memory"></a>

Configured `PSE_BUILD_MODE=unbounded` ordinary work shares all 16 physical cores and 32 logical
CPUs, with 32 Cargo jobs, 16 nextest lanes and no new OS memory cap or slot ledger. Recipe class
hints do not impose finite limits. Explicit finite/scientific/reference/timing profiles and
`PSE_MEMORY_MAX` select their declared admission and containment; these selections are supported
by the native owner and must retain their actual limits and exclusion meanings. Profile
support does not establish the remaining broad resident/reference/timing qualification.

The scientific `reference` declaration currently selects a 128 GiB shared pool, 140 GiB worker
envelope and 160 GiB total envelope; interactive request capacity is 128 GiB. These are
ceilings, not reserved resident RAM. Worker/compiler retention derives from the actual pool.
Native application roots and Tokio threads preserve at least the qualified 32 MiB mathematical
stack, including a properly sized application thread; `RUST_MIN_STACK` alone does not resize
the OS main thread. Explicit larger choices remain selections, not performance evidence.

Top-level launch creates a fresh native operation hierarchy bound to the original caller.
A private exec gate releases the child only after physical containment/population is proven.
The user manager stops owned work if its controller dies, including SIGKILL. Nested calls
borrow only after authenticating actual scope identity, dependencies and kernel membership.
Environment markers confer no authority. `--no-scope` requires that authenticated enclosure.
The retired `--store` and `--bind-allocation` protocols refuse; a Rust application constructs
its actual `NativeCanonicalService` or a private test fixture explicitly.

Manager-launched siblings use typed literal argv and private payload handoff; credentials are
not placed in argv or the manager environment. Finite reservations remain charged until the
original kernel group is empty. A stop job or direct child exit is insufficient. Cleanup uses
one drain deadline, attempts independent settlements and retains every failure. Existing
borrowed services remain observation-only. Private installed telemetry units have a separate
narrow authenticated capability. Concurrent external same-UID restart/reconfiguration is
unsupported; systemd has no atomic stop conditioned on invocation identity.

The original command status or `128+signal` propagates; missing/unexecutable commands retain
127/126. Boundary or unresolved cleanup failure returns 125 with its cause and original command
status, without claiming release. Linux user-systemd, usable cgroup ownership and actual
placement are premises. Hosted compile jobs do not establish this local acceptance.

## Native Cargo correctness

`cargo c`, `cargo t` and the check/unit recipes use the prepared native correctness adapter.
Pinned Cargo supplies the actual compiler unit graph, including host/build and selected-root
dev units. The adapter adds necessary validation opt-ins only to consuming roots and verifies
actual Arrow leaves and final root identities. Unknown coverage refuses; pure closures remain
pure. Compiler JSON remains unchanged and graph output stays private.

Nextest owns source-build selection and scheduling. Its invocation-local Cargo adapter sees
actual translated compiler arguments; setup restores real Cargo for test attempts. Reused
archives/binaries, foreign workspaces and unsupported workspace/tool overrides refuse before
runner launch. `PSE_NEXTEST_ACTION=list` selects listing through the same recipe grammar.
No Python selector, generated per-test runner or invocation ledger surrounds generic nextest.

The pinned cargo-hack owns powerset enumeration, ordering and keep-going. Existing
`features-combinations`/`features-no-default` recipes select the correctness executable via
`CARGO_HACK_CARGO_SRC`; it qualifies bare package features from Cargo metadata and preserves
actual target/host graph validation. Diagnostic taskdump builds select their explicit isolated
profile and `tokio_unstable` configuration; ordinary builds retain their original flags.
The diagnostic check is limited to this native Linux x86_64 checkout and target. Cargo's
matching target flags carry the cfg into dependency resolution as well as compilation;
profile-only flags cannot do that. A present encoded/raw global flag channel is extended
literally at its existing precedence, including an explicitly empty value. Foreign workspace
or target selections refuse before diagnostic compilation. This does not reconstruct arbitrary
external build-only flag configurations.

## Builds in one checkout

Cargo's fine-grain build locking does not establish safe overlap for dirty graphs. Upstream
[Cargo #17508](https://github.com/rust-lang/cargo/issues/17508) records a same-target deadlock;
keep overlapping feature matrices and Clippy under one Cargo driver. Compiler width remains 32.
Independent non-Cargo work can proceed. Isolated concurrent builds require separate checkouts
and targets; do not create cold copies merely to split a matrix.

## Worktrees

```bash
just worktree ../pse-arrow-other -b other HEAD
```

This forwards literal arguments to `git worktree add`. It does not install Python, build a
native host or run doctor. Prepare the new checkout explicitly and supply its personal
`.envrc.local` deliberately; review/allow its direnv configuration. Every worktree has its own
`target/`. Never set `CARGO_TARGET_DIR` or share targets between checkouts (ADR-0122). The
sccache cache is shared. Remove a worktree only after integrating its changes and preserving
necessary evidence, using `git worktree remove PATH`.

## Choosing tests

- `just check-package PACKAGE` compiles all selected package targets without running tests;
  `check-library` and `check-test` narrow that scope.
- `just unit-package PACKAGE WORD_OR_FILTER` selects library units; a bare word becomes
  `test(word)`. Extra Cargo package selectors and filtersets retain native nextest meaning.
- `just unit-native-package PACKAGE FEATURES FILTER` selects prepared native providers and
  direct nextest tests. Database consumers use an explicitly owned canonical fixture.
- `just affected` previews conservative changed-package reach; inspect its exclusions before
  treating the selection as coverage. Run-only reporting and fixture ownership remain separate.
- `just native-test MODULE_RECEIPT ...` builds/lists source tests first, then starts one fresh
  canonical fixture. Put Cargo selectors in repeated `--build-arg` and run filters after `--`;
  see [qualification commands](validation-assessment.md#individual-commands).

Neither a missing fixture nor unavailable native preparation becomes a skip or a pass. The
Rust application's fixture capability owns service lifetime; selected children borrow its
private descriptor. It grants no shutdown or persistent-state disposal authority. Rust test
assertions remain in public caller tests under `crates/pse-runtime/tests/native_*.rs` and their
workflow owners. Independent reference Python environments remain explicitly isolated.

## Selected reproducible experiments

Select a concrete scientific question, case and measurement owner before an expensive run.
The Python experiment, assessment, result, reference-run, archive, coverage-correspondence,
mutation and case-measure controllers are retired. Their required scenario handoffs belong to
Plan58 H; there is no replacement resume ledger or implication that those campaigns ran.
Native benchmarks and direct external tools retain their own interfaces. Report functional
assertions, exact placement, sample intervals and environmental interference separately.

## Bundles, logs and preflight

`just turn-end`, `just ready` and `just hygiene` are ordinary native recipe aggregates, using
Just's keep-going behavior. Read the actual command and exit results; the former Python
assessment directory, JUnit reconciliation and `just result` reader no longer exist. Run
functional checks during implementation and scope-end bundles at the timing in `AGENTS.md`.
No environment marker or preflight bypass certifies readiness.

### Detached watches and pressure observations

`just activity [UNIT]` returns actual current/named manager and independent kernel observations.
Unavailable manager data remains partial with exit 125, not inactive/empty success. The observer's
own membership never substitutes for the selected unit. Old invocation-watch cursors and
pressure/watch CLI flags are retired. Use an explicitly chosen external observer for additional
host pressure measurements and retain its identity and interval.

## Hooks and permissions

`scripts/pse-host agent-hook` owns edit guarding and session activation. Protected paths and the
visible `.design-edit`/`PSE_DESIGN_EDIT=1` exception remain defined in `AGENTS.md`. Guards are not
a shell sandbox. Claude's SessionStart activation uses the same native environment owner;
Codex uses its per-command bridge. Prepare tools explicitly. `just lint-agents` checks wiring;
there are no end-of-turn hooks. Remote-mutating recipes retain their explicit invocation and
confirmation requirements; permissions alone are not instructions to run them.

## Runtime capabilities

### Native operations and optional presentation

<a id="canonical-capability-entry-point"></a>

Use direct native APIs and prepared commands from the [native tool guide](capability-interface.md).
The Python product, MCP presentation and debugger proxy are retired. Native requests retain
cancellation/settlement and canonical write admission independently of an editor or observer.

### Focused command and source context

Use `just --usage RECIPE`, source search and native `just affected` for command/reach questions.
The former `just context` Python projections and history scanners are retired.
`just workspace-sources --selection FILE` supplies bounded source-only knowledge with explicit
coverage through native xtask. Documentation lifecycle/scope queries use
[`xtask lifecycle`](documentation.md#metadata-and-aggregate-scope); they derive status from
native owners rather than create another backlog. Reading a schema or corpus starts no service.

## Explicit host controls

### Optional native telemetry

The native environment resolves bounded settings through `pse_host::telemetry` without creating
an SDK, exporter or service. The process-owned Rust telemetry host installs providers explicitly.
`OTEL_SDK_DISABLED=true` disables export; explicit signal selections override base endpoint,
headers, protocol and timeout. Supported export uses HTTP/protobuf. Private resolved settings
carry secrets as data; explanations and public status omit them. A scientific Runtime does not
install providers. Export availability never changes numerical admission or results.

```bash
just telemetry install
just telemetry setup
just telemetry inspect
just telemetry start
just telemetry stop
just telemetry reconfigure
just telemetry recover-configuration
```

The state path is `PSE_TELEMETRY_STATE`, then `$XDG_STATE_HOME/pse/telemetry`, then
`$HOME/.local/state/pse/telemetry`. Setup owns private credentials, separate Collector queue and
OpenObserve data directories, immutable configuration and public `configuration.json`.
Use `scripts/pse-host telemetry-service --help` for explicit setup listener/organization/stream
options. Setup and reconfiguration preserve existing credentials/headers/data where supported;
inspect never installs, repairs, loads or starts a service. Start/stop authenticate unit fragment,
nonce description, literal execution and executable identity. Independent stop failures are
retained while other authenticated units still settle.

Reconfiguration records its previous/candidate generation and attempts rollback on failure;
explicit recovery selects the recorded previous configuration. Login enable/disable is an
external systemd operation, with no implicit enable side effect. An already login-enabled
deployment refuses reconfiguration before effects until explicitly disabled. Concurrent external
reconfiguration is unsupported. The operator is a local optional convenience, not a new daemon.

Collector persistent queue/retry configuration and internal failure metrics remain configured.
Existing OpenObserve data and alert rules survive through their data directory; the old Python
health/trace lookup/watch management wrappers have been retired. No replacement query/watch CLI
or equivalent backend acceptance is claimed. SDK loss observations distinguish first observed
queue saturation, any available shutdown drop count and actual shutdown outcome. Missing counts
and abrupt-exit loss remain unknown; queue capacity is not occupancy and successful shutdown
does not prove export or ingestion. One-shot children can lose final batches.

### Project and evidence commands

`just project-build` and `just evidence-build` explicitly construct separate native operators;
`just project` and `just evidence` forward to those prepared binaries. The project operator
consumes explicit state and typed documents; the derived evidence index has separate ownership.
Neither can initialize/start an unrelated canonical service during a read. See
[the substrate guide](surreal-substrate.md#project-and-derived-evidence-operations).

### Selected performance tooling on this workstation

`just diagnostic-tools TOOL` installs a declared optional pinned tool.
`scripts/pse-host diagnostic-tools verify TOOL` verifies without downloading or executing it. The declaration is
`.config/diagnostic-tools.toml`; supported tools include trace-processor, mcpls, rr and sccache,
plus the telemetry service's Collector/OpenObserve pins. Archive provenance and selected binary
identity are checked; old unreceipted binaries are not silently trusted. CPU capture uses the
native capture owner and an explicitly available perf. Optional tool installation is not capture,
scientific execution or H qualification. Third-party Python allocation tools, where used, belong
to an isolated tool environment and cannot restore the retired Python product.
