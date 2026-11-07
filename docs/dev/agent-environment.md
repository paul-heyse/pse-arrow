# Agent environment

This guide explains the checkout environment, native process ownership, preparation and
command selection. `AGENTS.md` owns working rules;
[Plan58](../plans/58-rust-owned-system.md) owns remaining acceptance. A prepared tool or a
focused passing control does not qualify the whole native system or the H campaigns.

## Preparing and activating a checkout

Construct/update the independent native host explicitly with `scripts/bootstrap-host`.
It builds locked host executables, validates the completed pair and atomically selects an
immutable generation under `target/host-tools/current`. Failed publication preserves the
previous selection. Help, activation and reads never rebuild. A missing host requires explicit
preparation; an effectful new-work command refreshes an existing stale host before admission.
Existing invocation wait/cancel uses its recorded identities without a current-source rebuild.

If a tool session is outside a unit known to the user manager, foreground admission can
refuse with `NoUnitForPID`. Launch through an ordinary transient user scope with a real
parent shell that waits for the command:

```bash
systemd-run --user --scope --quiet bash -c '"$@" & child=$!; wait "$child"' pse-launch scripts/pse check pse-host --all-targets
```

The waiting parent preserves the original scope while the native boundary moves its controller
into the owned workload. Launching a lone `exec` in a dedicated scope can correctly refuse
because that move would empty the original lifetime unit. This route retains native membership,
containment and settlement checks; it does not substitute a scope marker for authentication.

When an intentional host dependency edit requires a lockfile change, the stale host cannot
rebuild with `--locked` before that change exists. Refresh only the named workspace packages
with pinned-toolchain Cargo directly, for example `cargo update -p pse-host -p xtask`, under
the selected user scope. Inspect the lock diff for the intended dependency edges and version
moves, then run `scripts/bootstrap-host` explicitly and resume ordinary native routes. This
source-lock operation needs no prepared host; it does not execute a product workload or replace
native workload custody. Keep locked host construction and avoid a wholesale re-resolution.

`scripts/pse` is the agent-facing native command. The native environment prepends this checkout's
`scripts/` to `PATH`, exposing that same launcher as `pse` without another wrapper or global
configuration. Without activation, use `scripts/pse` explicitly. The local `pse-cli` skill
(source: `.codex/skills/pse-cli/SKILL.md`, shared through the existing runtime aliases) helps
select capabilities and progressively expand useful detail. `scripts/pse commands` and
`scripts/pse help COMMAND` describe its actual parser. Current invocation commands include
`run [--detach] -- COMMAND`, `wait INVOCATION [--timeout SECONDS]`, `cancel INVOCATION`,
`history`, `result INVOCATION` and bounded `log INVOCATION`. A selector can be an exact ID,
a unique prefix or `latest`, optionally scoped by `--session`. A timed-out wait ends that
observation and leaves the subject running. The original child, payload settlement and service
finalization remain separate facts; a successful control response does not rewrite the child.

Session correlation selects `--session`, then nonempty `PSE_SESSION`, then an existing
`CODEX_THREAD_ID`, then an existing `CLAUDE_SESSION_ID`, and finally the checkout session.
An empty environment value is unavailable; an invalid explicit selection refuses. Exact
invocation IDs retain their identity regardless of the session selection. `latest` resolves
within the selected session and never silently falls back across sessions. Use
`history --all-sessions` for an explicit checkout-wide view. Session correlation adds no hooks
or process ownership authority.

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
not borrow another's. For a tool process that has not inherited activation, use
`direnv exec . scripts/pse-env -- COMMAND` in the already reviewed/allowed checkout. Activation
must precede the outer workload boundary: an inner recipe's environment cannot undo a finite
allocation already selected by an unactivated outer command. This preserves the maintainer's
selected build mode instead of manually redeclaring capacity or changing OS limits.

On this workstation the Codex Bash bridge initializes the machine
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

The independent `scripts/bootstrap-host` seed and its native generation construction
compile directly. Both explicitly disable Cargo wrappers, including wrappers configured in
the user's Cargo home.

Retire one exact checkout-local host generation with
`scripts/pse tools retire-host GENERATION --format json`. Use the returned
`release` token with `--release TOKEN` when the preview has no blockers. The owner
rechecks the current selection, retained selector links, private invocation records
including removed worktrees, and live images. An interrupted release can resume with the
same token and state namespace. Incomplete preparations and unresolved records need
independent attribution.

The environment selects existing checkout-local tool paths, compiler inputs and native
sccache. It provisions no virtual environment or provider. Cache `auto` falls back to direct
compilation when optional cache prerequisites or supported modes are absent; required cache
selection refuses incompatible settings. `--cache off` disables wrapping even with an invalid
inherited binary. The direct sccache executable owns cache startup and I/O error handling.
The [build guide](build-performance.md#compiler-cache) describes the selected mode and the
boundaries of its fallback behavior.

## Explicit compiler/KLU preparation

Native providers have explicit construction and selection:

```bash
scripts/pse-env -- scripts/pse-host native prepare --capabilities compiler,klu
scripts/pse-env --native=compiler,klu -- cargo t -p pse-runtime --lib --features pse-runtime/solver-kinsol -E 'test(root_response_publication_has_physical_primal_and_no_kkt_fields)'
scripts/pse-env -- scripts/pse-host server prepare
scripts/pse-host server select
just surreal-module
```

Supported capability names are `compiler,solver,klu,isolation,uno,petsc,coolprop`. Preparation owns
source/tool observation and immutable publication; selection admits only completed compatible
artifacts. `--native=CAPABILITIES` selects them, bare `--native` uses
`PSE_NATIVE_CAPABILITIES` or the full default `solver,klu,isolation,uno,petsc`, and explicit
`--native=` selects native runtime settings without providers. Launch never constructs a
missing provider. Compiler/KLU has a bounded live scientific qualification basis; other
provider controls do not establish every scientific consumer's acceptance.

`coolprop` explicitly constructs the pinned source plus maintained saturation C export,
using checksum-retained CPM dependencies and the selected Python header-generation tool.
Configuration uses local sources only. Selection exports its immutable `COOLPROP_DIR`;
the maintained sys binding consumes that completed image without a bundled fallback or
Cargo-time native construction. The backend build admits the exact source/patch/image
recipe and retains it in the native Water profile. Wet equipment qualification remains
separate from artifact and endpoint ABI controls (ADR-0238).

Native generations live under the selected cache root, with compiler/KLU in `rust-v1/kinsol`.
Explicit preparation snapshots the default CMake compiler-cache bridge into immutable
content-addressed native-cache storage and records that selected path. Completed provider
selection does not depend on a disposable preparing worktree's bridge file. KLU supplies
the selected compiler context before dependent providers; repeat selection accepts its
authenticated installed values while changed cache modes, toolchains or tool bytes still
refuse. Older checkout-bound bridge receipts require explicit preparation before reuse.
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
not placed in argv or the manager environment. Owned transient services preserve their launch
caller's actual soft and hard open-file limits. The launch owner reads them with Rustix
`getrlimit(Resource::Nofile)` and supplies systemd's `LimitNOFILESoft`/`LimitNOFILE` execution
properties, including infinity. Otherwise a user manager's smaller default can cause a
detached Cargo build to fail while the same command succeeds in its caller. This inheritance
does not change host limits, CPU/memory placement or explicit scientific allocation.
Finite reservations remain charged until the
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

`scripts/pse check|test|tests`, `cargo c` and `cargo t` use the prepared native correctness adapter.
Pinned Cargo supplies the actual compiler unit graph, including host/build and selected-root
dev units. The adapter adds necessary validation opt-ins only to consuming roots and verifies
actual Arrow leaves and final root identities. Unknown coverage refuses; pure closures remain
pure. Compiler JSON remains unchanged and graph output stays private.

Nextest owns source-build selection and scheduling. Its invocation-local Cargo adapter sees
actual translated compiler arguments; setup restores real Cargo for test attempts. Reused
archives/binaries, foreign workspaces and unsupported workspace/tool overrides refuse before
runner launch. `scripts/pse tests` lists the actual source-built test population with the same
package, feature, target and filter selectors as `scripts/pse test`. Direct nextest remains a
first-class tool; the CLI owns its retained invocation and captures the original native reports.

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
scripts/pse worktree new pse-arrow-other --branch other --base HEAD
scripts/pse worktree list
```

Creation uses a sibling directory, copies the selected checkout's `.envrc.local` bytes and mode,
and explicitly prepares the new native host with `bootstrap-host --from`. A matching completed
generation is copied; changed source inputs use an explicit local build. Original Git, configuration,
preparation, registration and optional `--warm` launch results remain separate. A failure leaves
the created checkout present with its recovery route. Review/allow its direnv configuration. Every worktree has its own
`target/`. Never set `CARGO_TARGET_DIR` or share targets between checkouts (ADR-0122). The
sccache cache is shared. Remove a worktree only after integrating its changes and preserving
necessary evidence and explicitly settling actual users, using `git worktree remove PATH`.
Git removes the checkout directory and its registration in the same step. Resolve any
checkout-specific state outside that directory through its original resource owners.
Before removal, save the selected row from `pse worktree list --format json` with the
resource owner's retirement selection. The row records the original Git administration and
checkout directory identities, ordinary target, and external state path and identity.
Compare the recorded devices and inodes and, where available, creation times before acting.
Check the backing mounts independently. A recreated worktree can reuse a path and checkout
ID while naming a different Git administration or directory incarnation.
After Git removal, verify that the original registration is gone. If external release is
interrupted, resume with the original resource identities and recheck current references and
holds. A new worktree at the old path or an unavailable backing mount requires the resource
owner to resolve the association before release.
The supported CLI is `worktree new|list`; `pse worktree rm` is explicitly deferred
([register R-56](../adr/register.md)). Automated owner-aware removal refusal and teardown are
unavailable. Administrative removal must preserve readers, children, reusable sessions,
fixture ownership, required artifacts and unresolved settlement; passive inventory is not
a checkout-wide admission or retirement boundary.

`worktree list` observes Git paths, direct commit ancestry, dirty paths and recorded native
invocation users without preparing tools or settling owners. Unknown native facts remain unknown;
neither a terminal result nor this inventory grants removal authority. `--warm` submits the existing
detached workspace check after successful preparation; its acknowledgement is not build completion.

### Inspecting allocated storage

Use `pse worktree list --format json` for checkout and native user observations. Select exact
target and state directories by their checkout or resource receipts. Check each root's mount
with `findmnt --target PATH` and record absent, inaccessible, or unexpected roots separately.
Pass the verified directories on each filesystem to one physical allocation scan:

```bash
du --one-file-system --block-size=1 --summarize -- "$checkout_target" "$checkout_state"
df --block-size=1 --output=source,used,avail,target -- "$checkout_target" "$checkout_state"
```

One `du` invocation counts a shared inode once across its arguments. Use separate output for
apparent size when it matters. A missing root or partial native observation leaves that part
of the census unknown. Use original settlement and current reader, hold, and selection facts
to classify release candidates. Compare measured allocations and filesystem availability
separately after release.

### Closing ad hoc scratch

Put regenerable output for an ad hoc campaign in a private directory below the selected
checkout's `scratch` state root. Give the directory a single producing invocation. Record
its purpose, exact path and directory identity, original invocation, and the source revision
and arguments needed to run it again. Keep authored inputs outside this directory. Use a
managed `pse run` invocation when the command starts subprocesses or needs a durable
settlement observation.

At closeout, retain the selected finding and its rerun selection. Transfer any needed
published result through its own owner. Use `pse result` or `pse wait` on the original
invocation and check its payload settlement. A successful observation that times out
still leaves the original completion unknown. Preview allocated bytes and recheck the
directory identity, backing mount, readers and holds before releasing the exact private
directory. If an acknowledgement, drain or transfer is unresolved, record that fact with
the original owner and leave the material available for recovery. Treat old `/tmp` output
and unclassified developer roots as a separate inventory until their ownership is established.

An authored scratch probe has a narrower output-only release. Use
`scripts/pse probe clean NAME` to preview the allocated Cargo target bytes and exact
directory identity. Pass that identity to `scripts/pse probe clean NAME --release IDENTITY`
after the original native roots drain. The probe owner rechecks readers and identity under
its lease. The authored source, lockfile, and original run records remain available for
regeneration. A replaced directory requires a fresh preview.

## Choosing tests

The Plan59 native commands support `scripts/pse tests PACKAGE -E 'test(name)'`,
`scripts/pse test PACKAGE -E 'test(name)'` and `scripts/pse check PACKAGE`. Listing explicitly
builds and captures the actual selected population. Test/check/list can use `--detach` and
`--session NAME`; `scripts/pse wait ID` reads the original result independently. A zero-match
test selection is a nonpass. Missing fixture lifecycle readiness remains explicit; the broader
fixture/rerun migration is still owned by [Plan59c](../plans/59c-tests-and-fixtures.md).

`--non-incremental` selects Cargo's `build.incremental=false` for that check, test or
listing. Admission and the build carry the same setting, and failed-test rerun uses the
original selection. Unset an inherited `CARGO_INCREMENTAL=1` before using the flag because
that environment value takes precedence in Cargo. Ordinary editing keeps the existing mode.

The original detached owner releases private test and compiler images after native drain and
paired result publication, including when nobody waits. Borrowed runs keep their images until
the exact enclosing owner drains. `wait` can reconcile an explicitly selected settled workflow
and leaves live borrowed scopes under their enclosing owner. The original working report and
inventory remain available after image release.

Use an original working report while it is available. After cleanup, select the named test
or campaign from source and run it again under current admission. Record the resulting
revision, selection and outcome for the decision that requested it. Routine test results
are regenerated when needed. Published scientific results follow their own custody.

`scripts/pse inspect tests ID --limit 20` reads captured whole-test records without rebuilding.
Pass the returned `payload.next` object as JSON to `--cursor`; changed inventory bytes or a
different original invocation refuse continuation. `--page-bytes` bounds serialized whole rows;
an oversized next row reports its required extent without splitting its identity. The reader
still scans the retained inventory and grants no fixture readiness or executable-reuse authority.
New command grammar requires an explicitly prepared image containing that grammar.

`scripts/pse inspect test-selection --from-result REF --lib -E 'test(NAME)'` validates a
working-source result's original root and bytes, resolves its current containing package
from native Cargo metadata, and delegates unchanged target/feature/filter policy to the
test owner. It does not list tests or prepare fixtures. Explicit package/workspace/changed
scopes and original failed-run selection conflict with this source-derived package. The
answer names a current candidate; source search did not capture an original Cargo graph
and cannot establish dependency impact or sufficient coverage. Use `inspect affected` for
the separate conservative changed-source explanation, then explicitly list or execute.

- `scripts/pse check PACKAGE --all-targets` compiles the package's eligible targets without
  running tests; `--lib`, `--test NAME` and `--bin NAME` narrow that scope.
- `scripts/pse test PACKAGE --lib -E 'test(WORD)'` selects actual library test binaries.
  Multiple packages, native filter expressions and explicit feature selectors remain visible.
- `scripts/pse test PACKAGE --lib --features FEATURES -E FILTER` derives the required native
  capabilities from the reached compiler units and selects their already prepared providers.
  Database consumers use their declared Rust fixture owner.
- `scripts/pse inspect affected` previews conservative changed-package reach; inspect its explicit
  widening and coverage limits before
  treating the selection as coverage. Run-only reporting and fixture ownership remain separate.
- `scripts/pse test PACKAGE:TARGET --target-kind KIND --features FEATURES -E FILTER`
  admits and captures selected source tests before resolving their declared fixture inputs.
  Borrowed canonical cases use the existing Rust fixture owner; target-owned cases receive
  immutable prepared inputs and retain their own service ownership. Select the module with
  `PSE_NATIVE_FIXTURE_MODULE_RECEIPT` or the checkout's prepared module selection; see
  [qualification commands](validation-assessment.md#individual-commands).

Neither a missing fixture nor unavailable native preparation becomes a skip or a pass. The
Rust application's fixture capability owns service lifetime; selected children borrow its
private descriptor. It grants no shutdown or persistent-state disposal authority. Rust test
assertions remain in public caller tests under `crates/pse-runtime/tests/native_*.rs` and their
workflow owners. Independent reference Python environments remain explicitly isolated.

Same-package integration helpers consume runtime `CARGO_BIN_EXE_<name>`. Cargo supplies this
variable for direct integration tests; native Nextest supplies it from its non-test-binary
metadata, which the CLI maps to the exact same-build captured executable. Embedding a
compile-time `env!("CARGO_BIN_EXE_...")` path would bypass that mapping. Cross-package helper
inputs remain with their declared fixture owner. Each captured Cargo executable keeps its
original basename inside a unique private artifact directory, preserving native program-name
checks. Before lending an image, the capture owner publishes its existing executable byte/origin
record beside it. Captured correctness tools verify that protected record, retained bytes and the
kernel's executing-image descriptor before recovering the original logical checkout. They do not
infer it from the caller's cwd or an environment marker. The original mutable output may be gone;
foreign-workspace/setup fences still apply. Linked libraries and scientific inputs retain their independent
runtime admission contracts.

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

Supervising runs that an agent starts is a separate, allowed responsibility
([Plan59](../plans/59-agent-cli.md#rule-changes) R2): a run ledger with wait, result and cancel
([59b](../plans/59b-runs-and-continuity.md)). Build-stage telemetry and alerts stay retired.

## Hooks and permissions

`scripts/pse-host agent-hook` owns edit guarding and session activation. Generated routing comes
from the aggregate map emitted by `just codegen`; missing routing preserves generated-directory
and bindings protection. Architecture and accepted ADR edits receive their revision or
supersession workflow as context, with no permission-marker mechanism. `AGENTS.md` retains
their governing policy. Guards are not a shell sandbox. Claude's SessionStart activation uses the same native environment owner;
Codex uses its per-command bridge. Prepare tools explicitly. `just lint-agents` checks wiring;
there are no end-of-turn hooks. Remote-mutating recipes retain their explicit invocation and
confirmation requirements; permissions alone are not instructions to run them.

The shared native `scripts/pse-host agent-hook tooling-advice` handler is wired to matched
Bash tool calls in `.codex/hooks.json` and `.claude/settings.json`. It offers relevant native
capability guidance as `hookSpecificOutput.additionalContext` for `PreToolUse`; it always exits
zero and neither blocks nor rewrites commands. Advice points to the local `pse-cli` skill
rather than inserting its full capability reference. Routine file/text searches and Rust source
reads receive no nudge. Structural-search advice offers guarded edit previews conditionally;
capability availability alone establishes no tool preference.
Changed Codex hook definitions require the native `/hooks` review/trust flow before they become
active; no global configuration, MCP registration or trust bypass is part of this setup.

## Runtime capabilities

### Native operations and optional presentation

<a id="canonical-capability-entry-point"></a>

Use direct native APIs and prepared commands from the [native tool guide](capability-interface.md).
The Python product, MCP presentation and debugger proxy are retired. Native requests retain
cancellation/settlement and canonical write admission independently of an editor or observer.

### Focused command and source context

For capability/use-case selection, use the local `pse-cli` skill and read its
`references/code-intelligence.md` only for the relevant question. This section owns detailed
mechanics; the skill explains when source, semantic, API or compiler evidence is useful.

Use `just --usage RECIPE`, source search and native `scripts/pse inspect affected` for command/reach questions.
`scripts/pse commands --markdown` exports the owning parser without preparing a tool;
`just codegen` publishes its [CLI reference](../generated/agent-cli.md) and checks derive from
that same parser rather than a second command list.
The former `just context` Python projections and history scanners are retired.
`just workspace-sources --selection FILE` supplies bounded source-only knowledge with explicit
coverage through native xtask. Documentation lifecycle/scope queries use
[`xtask lifecycle`](documentation.md#metadata-and-aggregate-scope); they derive status from
native owners rather than create another backlog. Reading a schema or corpus starts no service.

For exact written source, use `scripts/pse inspect outline FILE --budget 20` and pass the returned
`payload.page.next_cursor` unchanged with `--cursor`. The allowance counts nested declarations,
including entries inside modules and impls. A continuation refuses changed working bytes.
Outlines default to navigation only. Add `--detail facts` for selected interpreted source facts
or `--detail mapped` for their decoded-to-source mappings; continuations also bind this detail
selection. `--fields` only projects the response and does not request source construction.
`scripts/pse inspect declaration FILE --at-line 12 --at-column 7` selects the enclosing declaration using
one-based Unicode scalar coordinates; `--range-start` and `--range-end` select a half-open
UTF-8 byte range. `--index` reads the raw staged blob and `--revision REF` resolves a committed
blob without checkout. These reads need neither a prepared analyzer nor a pse-arrow checkout.
Declarations default to `facts`; raw ranges default to navigation. Inspect `detail_coverage`
before treating omitted facts as absent, and request `mapped` only when exact escape/source
correspondence matters.

Use `scripts/pse search --path FILE syntax 'call($ARG)'` for written structural
candidates. Search defaults to the first twenty arrivals; `--all`, `--count` and stable
continuation are explicit demands. Native AST rules and Tree-sitter queries keep their
original subtrees in typed JSON requests. Scoped Boolean clauses distinguish line, file,
node and entity predicates; an unavailable refinement remains unknown rather than false.
`scripts/pse search --explain` shows merged request/configuration provenance without searching.
Request-file paths remain relative to that file. Explain works even when an explicit root is
missing; execution refuses that root before query Begin, with its resolved operand and input
origin. Where safe, the returned recovery action replays the original inputs as passive explain.
Read command failures from stderr and admitted query diagnostics from stdout. In JSON, inspect
`payload.summary` and `payload.diagnostics` as well as top-level failures; in NDJSON, require the
terminal record and consume its completeness/failure fields.
`--result-fields`, `--group-by`, `--limit` and `--budget` select independent result detail,
returned-witness grouping, answer demand and display allowance. NDJSON delivers whole live
records followed by the coverage summary; absent finality means incomplete delivery.

For responsibility questions in reviews, the typed `related` predicate accepts `seeds` (original
result-reference objects), `relations`, `direction`, `depth`, `max_nodes`, `max_edges` and
`seed_match: "any" | "all"`. Its `traversal` source selector admits seeds and supporting
intermediates; the outer request's `sources` select returned answer nodes. For example, an
incoming `field_type`/`parameter_type` question with two policy-type seeds and `seed_match:
"all"` can find operations consuming a shared request type even when that intermediate is
outside the output package. Select the same prepared working native world for all seeds;
index/revision evidence cannot become current native semantics. Scalar `seed`/`relation`
request forms are replaced; `schema 'search request'` advertises the current input shape.

`summary.relationships` retains separate graph scopes for composed clauses: shared declaration
or occurrence addresses, canonical edges, role-qualified occurrence witnesses, per-seed minimum
distances, explicit frontier, examination count and stopping reason. Node expansion preserves
the original semantic session; witness expansion uses its exact guarded source interval.
Two parameter positions can share endpoints while retaining separate witnesses. Depth is an
intentional question boundary; an exhausted node/edge allowance or unsupported provider fact
does not establish absence. Counts concern selected answer nodes, independently of edge
examination and rendered rows. Related NDJSON records describe a settled bounded graph; ordinary
source searches continue to stream. Human graph views include reusable `inspect --result`
actions, and field/budget projection preserves graph identities and coverage.

`scripts/pse search syntax --grammar` reads the linked Rust grammar's native `NODE_TYPES`
without source discovery or analyzer preparation. `--node-kind function_item` selects its
exact native entry, including fields and child constraints; `--grammar bash` exposes the linked
Bash source grammar. `scripts/pse schema 'search syntax grammar'` describes the response envelope,
while the response itself contains grammar data. Grammar inspection is separate from pattern,
rule and Tree-sitter extraction modes. Native Tree-sitter construction failures retain zero-based
query row, UTF-8 byte column and byte offset, rather than source-file coordinates.

For native ast-grep rules, choose strictness and ancestor/descendant search extent explicitly.
For example, `{"rule":{"pattern":{"context":"call($ARG)","strictness":"smart"},
"inside":{"kind":"function_item","stopBy":"end"}}}` searches ancestors up to the
root; `stopBy: "neighbor"` checks only the immediate parent. These are native matching
policies, and syntax matches do not establish resolved binding identity.

`scripts/pse propose syntax FILE --pattern 'call($ARG)' --replacement
'wrapped($ARG)' --select 0` preserves native template captures and original trivia without
source or Git-index writes. Its `--inside`, `--has`, `--exclude`, `--capture` and
`--strictness` options retain native matching policy. An undefined template capture or
selected overlapping edits refuses. Use `scripts/pse inspect changes` to inspect separate
changed Git layers, declared source/generated owners and conservative test reach.

For a hit in a declared generated output, follow the returned `inspect origin` next action. It
binds the original byte range and digest, reads the existing contributor sidecar, exposes the
declared regeneration argv and offers an outline of the known emitter. An unknown authored
declaration location stays unknown. A changed working source refuses the original-hit action;
neither preview nor drill-through edits files, regenerates outputs or prepares a semantic session.

### Offline protobuf investigations

Use `scripts/pse search schema NAME --descriptor FILE` to discover declarations in an
explicitly supplied complete descriptor set. `scripts/pse inspect schema --descriptor FILE
FULL.NAME` follows fields, presence, options and service input/output relationships. Returned
references bind the original descriptor digest; expand them with `inspect --result` rather
than reconstructing an identity from a name. These reads need no analyzer or running service.
The existing `pse schema` command continues to describe CLI output contracts.

`scripts/pse inspect payload --descriptor FILE --message FULL.NAME --input PAYLOAD
--encoding binary` explains an explicitly selected payload without invoking its method.
The other encodings are `json` and `textproto`. Original input identity is separate from
decoded values and re-encoding. Constraint metadata does not imply that a dynamic validator
executed; inspect the reported validation coverage. Text-format parsing uses the installed
`protoc` and retains its diagnostics; parsing is separate from semantic validation.

`scripts/pse inspect schema --descriptor CURRENT --compare-with BASE` reports structural
differences and native Buf compatibility classifications. Intentional breaking changes are
classification evidence. Generated-source associations retain their exact source and generator
qualifications; descriptor names and field numbers alone establish no Rust semantic identity.

### Dependency, API and native documentation investigations

`scripts/pse inspect dependency tokio --why` adds shortest metadata inclusion paths; `--feature-member VALUE`
finds exact authored feature-expansion links. Declaration optionality, default requests and
target expressions are retained independently of Cargo's resolved metadata feature union.
For selected compiler instances, use `scripts/pse inspect dependency-units tokio pse-host --lib
--no-default-features --without-feature diagnostic-taskdump`. The root/target/features grammar
is shared with checks, including multiple roots and negative features. The answer is admitted
compiler-unit context, not compilation, native readiness, a test population or impact proof.

`scripts/pse api tokio --prepare` explicitly acquires a source/context-qualified rustdoc artifact;
lookup does not compile. Then `scripts/pse inspect api tokio --search spawn --budget 3 --examples`
selects authored examples with hidden setup and native expectations. `--declaration` expands
the original version-qualified item when compact signatures omit complex bounds or associated
data. A facade may require explicit preparation of its defining package; absence of that
artifact is unprepared, not an empty API result. `scripts/pse probe --help` exposes separate
authored experiments; printing an example never executes it.

Each acquisition creates a private Cargo target. After the artifact is committed and the
original native root drains, the API owner releases the target and keeps the artifact with
its source and context evidence. For a failed, interrupted, or borrowed acquisition whose
target remains, run `scripts/pse api PACKAGE --retire-build INVOCATION` to preview the exact
target and allocated bytes. Append `--release IDENTITY` from that preview after the enclosing
owner drains. The owner checks the original invocation and target identity under the
acquisition lease. A repeat can report the target already absent. Acquisitions created before
this receipt was recorded need separate owner evidence.

`scripts/pse inspect api PACKAGE --compare-with /original/acquisition/document.json --budget 10`
reads native structural differences against the currently acquired PACKAGE context. Continue
with the returned cursor; different original bytes, extraction context or backend policy refuse.
The structural backend is admitted per input only after a lossless typed decoder round trip;
unsupported newer declarations remain unavailable. Full artifacts are still scanned. Native
automatic/blanket impl omissions and unresolved references remain visible. Structural change
is not a SemVer or behavioral compatibility verdict.

Adding `--compatibility` explicitly runs separately installed native rules under managed
ownership; prepare that optional tool with `scripts/pse diagnostic-tools install
cargo-semver-checks`. It uses both original captured JSON files, without generating a baseline
or updating manifests/locks. Original findings/exit/raw streams and differing extraction
premises are retained. `scripts/pse api pse-ids --check-docs` separately runs scoped native
rustdoc lints through the shared compiler workflow, retaining original rule/source addresses.
Ordinary API queries require neither that check nor an analyzer session.

### Original compiler evidence and explicit investigations

For an original check failure, use `scripts/pse inspect diagnostic INVOCATION` and follow its
addressed `inspect diagnostic` action. `--source` compares original native snippets to current
source; `--raw` retains the exact Cargo JSON representation. Related-span pages keep secondary
locations, child notes, replacements/applicability and macro context. They never compile, and
matching visible lines does not establish a whole original-source identity.
When those visible lines match, a returned `inspect entity` action is separately guarded against
the newly observed whole-source digest. It requests current semantics from an explicitly
prepared session; it does not assign that digest or semantic answer to the original compilation.

Explicit new compiler evidence uses `scripts/pse compiler generate PACKAGE --kind mir --body
FUNCTION --pass ElaborateDrops --question 'remaining field drops'` (library by default, or select
`--bin NAME`). Cargo features and `--target` select the actual admitted unit. THIR, type-size and
mono-item kinds select unit scope; unsupported body/pass narrowing refuses. Native dump generation
disables incremental reuse and requests sccache's no-cache mode within that explicit stage, so
side-effect dumps are produced again. These opaque, pin-specific files are evidence to inspect;
they are not host-inferred loan/liveness facts, an ABI proof, runtime reach or a speed measurement.

For concrete constants/layouts, first create and edit an authored `pse probe`; then use
`scripts/pse compiler probe NAME --question 'concrete constant and layout'`. The existing isolated
probe owner retains its lease, graph, toolchain and authored stdout meaning. Compiler artifact kinds
can also select that probe explicitly. `inspect compiler-artifacts INVOCATION` reads its original context,
native outcome and file index; `inspect compiler-artifact INVOCATION INDEX --length 4096` verifies the historical
whole-file SHA on a checked descriptor before returning a bounded range. Digest validation scans
the complete file and states that cost. Reads never rerun compilation, install components or
prepare a structured compiler driver. Changed original files refuse their digest guard.

For owned compiler observations, explicitly prepare `pse tools prepare compiler-analysis` with
the matching pinned rustc-dev component. `compiler generate PACKAGE --kind facts --body FUNCTION
--fact-focus captures --question QUESTION` selects an exact owning body and its nested bodies;
the same flags work with `compiler probe NAME`. `captures`, `suspension` and `cleanup` select
different native query families before extraction. The ordinary host does not link compiler-private
crates. Native observations remain provisional until full compilation, stream capture and original
input/runtime checks succeed. No file index is published after a failed or interrupted compilation.

`inspect compiler-facts INVOCATION --fact-focus cleanup --max-bytes 16384` passively reads that
original index. It retains whole bodies, reports omissions and addresses the full original context
instead of repeating its entire compiler-unit graph. Capture source types, stored types and saved
coroutine fields are distinct. Move-path may-initialization belongs to analysis-phase drops; later
cleanup edges, callee ABI and termination causes belong to their recorded phases. These observations
are neither general loan facts nor runtime resource-release evidence. Missing acquired families
require explicit new compilation, and an old fact never silently inherits current source context.

Managed probe runs publish their original checkout/invocation, exact command/cwd and control
reference before launch while holding the existing probe lease. `probe rm` takes that same lease,
checks every recorded run against original ledger metadata and requires the original native root
to be drained. A failed calculation can be safely drained; a terminal row or absent PID alone
does not establish that fact. Unknown legacy captures and missing custody remain protected.
An authenticated borrowed enclosing owner can keep the probe protected until that root drains.

### Selected Rust semantics and edit previews

Semantic helpers use generated binary gRPC over HTTP/2 on their existing private Unix socket.
Every initial connection and reconnect authenticates the kernel peer against the original
managed invocation. The server derives client identity from Unix credentials. A reusable typed
client shares that connection; it does not prepare another service. Selector version 4 identifies
the current native fact semantics. Selectors 1–3 remain readable for original native administration, but queries
refuse and require explicit stop/repreparation.

Each admitted operation has a session-retained worker. Losing its handler, response stream or
deadline requests cancellation while the worker retains the actual CPU task through settlement.
Session stop closes admission and drains work before releasing the provider. Input half-close
permits completion; explicit Cancel keeps the receive half through Terminal. Terminal separates
original work outcome, cancellation request and CPU settlement from live-LSP return/detachment.
Transport loss cannot establish that the caller observed settlement. Typed `observed` facts and
`native_types` are usable by ordinary callers; `sections` retains original provider evidence.
Snapshot HIR uses the pinned analyzer's UTF-8 VFS: source-only raw-path access remains available,
while unsupported native raw-path queries refuse explicitly.

Explicitly prepare the native image with `pse tools prepare native-analysis`, then start a reusable
snapshot with `pse code session prepare --backend native-snapshot --project PATH`; project defaults
to this checkout. The LSP backend retains its separately prepared selection. Select features/defaults/platform there. Restricted
preparation disables automatic compiler checking, build scripts and proc macros. Add
`--build-macros` only for explicit correctness-admitted `check --workspace --all-targets`
preparation of those outputs. Queries never enable checking or refresh those compiled inputs.
Native `--test-context` explicitly selects cfg(test); the ordinary native world excludes test-only
declarations. Build products, macro server/dylib and compiler identities remain visible. A missing
or changed required product refuses; cancellation during synchronous expansion can remain pending
until the native work actually returns or the original session is stopped and drained.
Source-only `inspect outline`, `inspect declaration`, `search syntax` and `propose syntax` remain useful without a
session; unavailable typed semantics are not silently replaced with text matching.

Use `scripts/pse inspect entity FILE --line N --column N` for native hover/definition/type
resolution, then select `--view refs`, `calls`, `implementations`, `macro`, `tests`,
`candidates` or `hints` on that same `inspect entity` route. CLI coordinates are one-based Unicode scalar columns; provider locations retain
their actual negotiated encoding. `--if-source SHA` guards a previously observed source.
Returned lazy candidate/hint/assist actions also bind the original generation and native-record
fingerprint; an array index alone cannot select a changed candidate. Related tests are candidates:
their next action goes through `pse tests`/`pse test` selection rather than executing analyzer argv.
Native explain/reference views retain operation roles, implicit targets, adjustments and captures
with available use causes. `--view layout` separates Sized-proven full extents from unsized prefixes
and unsupported children. `--compiler-layout INVOCATION` reads original type-size evidence and
checks recorded source/unit/feature/compiler/target premises; missing cfg/edition/per-type compiler
identity stays unknown and opaque debug output supplies no numerical ABI equality. Native predicates
retain holds/not-proven/invalid/unsupported; failure to prove never supplies a negative proof.

`propose rename`, `propose assists`/`propose assist`, `propose file-rename` and scoped `propose ssr` produce the same checked
proposal used by syntax previews. Inspect exact original/replacement regions, file operations,
placeholders and provider context. These commands apply no edits and make no compile-success or
multi-file publication claim. Unsupported structured native fields stay explicitly unavailable;
hover text remains rendered evidence, not an inferred type/capture model.

`code session status` observes original ownership and readiness; `code session stop` settles the
original service and descendants. Canceling one query leaves the session usable. Changed Cargo,
toolchain or prepared macro inputs require explicit stop and preparation; passive status/stop
remain usable even when current host source cannot compile. Source buffers are synchronized only
for active queries and closed after the last overlapping request; no accumulated source-tree
rescan or second semantic database is maintained.

## Explicit host controls

### Optional native telemetry

The native environment resolves bounded settings through `pse_host::telemetry` without creating
an SDK, exporter or service. The process-owned Rust telemetry host installs providers explicitly.
`OTEL_SDK_DISABLED=true` disables export; explicit signal selections override base endpoint,
headers, protocol and timeout. Supported export uses HTTP/protobuf. Private resolved settings
carry secrets as data; explanations and public status omit them. A scientific Runtime does not
install providers. Export availability never changes numerical admission or results.

The ordinary portable-root test re-executes its own test image and enters the production private
receiver. For the standalone executable's bootstrap, explicitly prepare the native lane and
select the separate test. The optional telemetry journey on that test takes
`PSE_NATIVE_TELEMETRY_ACCEPTANCE=1`, an independent W3C parent in
`PSE_NATIVE_TELEMETRY_ACCEPTANCE_PARENT`, and the resolved native exporter settings.
Repeated preparation reuses the verified lane in the same source and native build context.
Prepare under the same native profile selection used by the test.

```bash
scripts/pse-env --native=compiler,klu,solver,isolation -- scripts/pse-host fixture prepare native-lane
scripts/pse-env --native=compiler,klu,solver,isolation -- \
  scripts/pse test pse-runtime --lib --features pse-runtime/solver-kinsol --ignored only \
  -E 'test(=math::native_child::tests::actual_portable_native_root_standalone_bootstraps_and_parent_screens)'
```

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

New starts require `Type=exec` and correlated native job completion before strict process
inspection. This acknowledges execution, not collector ingestion readiness. A stopped older
`Type=simple` definition requires explicit setup/reload; an already running original remains
inspectable and stoppable after its executable, process, invocation and group authenticate.

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

### Local observation and selected investigation

Explicitly prepare the product reader with `scripts/pse observe prepare`, then use the returned
immutable reader ID. Host help and captured discovery/schema reads do not build a reader.
The product owns the request grammar and numerical interpretation; obtain the accepted contract
from that exact captured executable:

```bash
scripts/pse observe discover --reader READER_ID
scripts/pse observe schema --reader READER_ID request
scripts/pse observe schema --reader READER_ID response
scripts/pse observe query --reader READER_ID --request selection.json
scripts/pse --format ndjson observe follow --reader READER_ID --request follow.json --samples 32 --interval-ms 1000 --timeout 30
scripts/pse observe investigate --reader READER_ID --request investigation.json
```

Each request file contains the product request alone. Discovery describes checkpoint catalogue,
inspection, whole-record pages, follow, retained study/attempt reads and explicit investigation.
Current-process selections require the original process/start identity; retained selections
describe copied evidence. Product-issued cursors bind the selected source and limits. A changed
source can require restarting a page; follow reports gaps and resynchronization explicitly.

Observer invocation IDs, scientific run/attempt IDs, diagnostic operation IDs and native manager
incarnations retain their separate meanings. A successful read can describe failed science,
missing live numerics or partial capture. Inspect original outcome, units, numerical categories,
source clocks and coverage in the product payload. Canonical study/attempt requests select an
existing deployment/database and an explicit observer allocation/allowance; their connection
reader never starts its configured database. Active remote attempts can report
`live_numerics_unavailable` while retained terminal history remains readable.

Optional native association checks the publishing process's full kernel identity and membership
in the recorded original unit. Its CPU, memory and task accounting covers that whole unit and
its descendants; combined native/product reads are non-atomic. Rejected or absent enrichment
leaves the original product evidence readable and grants no stop or result capability.

Follow limits govern observation. Closing its output, stopping the observer or reaching its
allowance leaves the performing owner responsible for its original work. Capture and selected
export require the explicit `investigate` verb; passive queries refuse those effects. Supplied
frozen evidence and selected finalized capture members can be analyzed and exported through
their existing owners. Embedded original paths do not authorize adjacent reads or reconstruction.
Maintained query specifications report `executed: false`; executing an external query requires
a separately selected consumer and bounded tool ownership.

Local checkpoint reads remain available when optional OTLP/Collector/OpenObserve export is
unavailable. Perfetto collection is an explicit investigation. Ordinary shell consumers require
no MCP setup. RMCP remains deferred until an actual client needs typed tools, resources or
subscriptions beyond shell access; any future adapter must preserve native and canonical
execution authority and demonstrate actual host attachment. Current delivery and remaining
acceptance are owned by [Plan59 F](../plans/59f-simulation-observability-and-diagnostics.md).

### Selected performance tooling on this workstation

`just diagnostic-tools TOOL` installs a declared optional pinned tool.
`scripts/pse-host diagnostic-tools verify TOOL` verifies without downloading or executing it. The declaration is
`.config/diagnostic-tools.toml`; supported tools include trace-processor, mcpls, rr and sccache,
plus the telemetry service's Collector/OpenObserve pins. Archive provenance and selected binary
identity are checked; old unreceipted binaries are not silently trusted. CPU capture uses the
native capture owner and an explicitly available perf. Optional tool installation is not capture,
scientific execution or H qualification. Third-party Python allocation tools, where used, belong
to an isolated tool environment and cannot restore the retired Python product.
