# Development tooling

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.** The
native host modules that were untracked when the streams read them (`agent_hooks`, `doctor`,
`diagnostic_tools`, `telemetry_service`) were committed unchanged in substance at `f3a05e0be`
during the engagement.

This page holds the causes specific to the developer and host-operation layer:
- `pse-host`;
- `scripts/pse-env` and `scripts/pse-host`;
- the justfile;
- the Python scripts still reached from recipes;
- `xtask` development commands;
- agent hooks.

Causes that cross the product/tooling boundary have their primary homes elsewhere:
- [DA-020](boundaries-and-composition.md#da-020): product crates depend on the mixed host/tool
  root;
- [DA-023](local-reasoning-and-assurance.md#da-023): the architecture sections omit the host
  layer;
- [DTA-063](../test-audit/test-execution-system.md#dta-063): the duplicated Arrow
  force-validation resolver and the coexisting test route families.

The [integrated account](system-architecture.md) places this layer in the whole system.

<a id="scoped-t01-host-foundation-refresh--2026-10-28"></a>**Lineage.** The 2026-10-28 scoped T01 host-foundation refresh, and the full DA-036/037/043 text
at source `682fc3830`, remain recoverable in Git (publication `8fed0a30c`). This refresh merges
DA-036's and DA-037's surviving residue into [DA-057](#da-057), the cause both now instantiate.

## The layer in brief

The native host is a well-built *mechanism*:
- **Typed unit ownership.** Owned and borrowed units are distinct types, an unknown launch is
  never repeated, and release is proven by cgroup population (`crates/pse-host/src/lib.rs:22-55`).
- **Environment precedence** is a pure composition passed to children privately as NUL-separated
  data.
- **Explicit refusals** of unfinished modes, with no Python fallback
  (`crates/pse-host/src/boundary.rs:57-60, 121-123`).
- **A cold seed that publishes atomically**, with an Arrow-, Python- and database-free normal
  closure.
- **Read-only activity** that bypasses admission (`crates/pse-host/src/main.rs:90-108`).
- **Bespoke build observation is gone** (`build_observation.py` was deleted in `8af5ea110`).

What the layer lacks is a single *responsibility* per policy while the cutover is in progress.
The frontdoors have moved to Rust, but the consumers still served by Python carry the entire
Python policy graph:
- `just surreal` runs the 11.6k-line `scripts/surreal_server.py` directly (`justfile:110-114`).
- The assessment, `native-test`, `select.py`, `build-frontend` and `py-*` recipes remain on Python.
- An approximate static import walk from the justfile entrypoints reaches about 86 of the 141
  top-level `scripts/*.py` modules. That walk is a probe, not a census.

## <a id="da-057"></a>DA-057 — Native replacements move entrypoints one consumer at a time, so host policies keep a second live authority or interpreter

**Principles:** DP-01, AP-04, AP-05, DP-16, DP-20; H1, H25.

**Status:** new on 2026-10-29. It absorbs the surviving residue of DA-036 (two admission readers
of one policy file) and DA-037 (the Python reciprocal policy web behind the remaining script
consumers). Both identities stay retired; see [README](README.md#revalidation-outcomes).

**Claim at risk.**
- The guide promises "one native environment and foreground boundary".
- ADR-0233 rejects a thin replacement precisely because it "would retain that second system".
- Plan58's own cutover rules:
  - T02: no compatibility DTO and no Python admission fallback;
  - I01: "integrated owner/callers/configuration/tests/deletion cutovers; no bridge".
- AGENTS.md prime directive 3 requires one authoritative declaration per meaning.

**Structural cause.** Each Plan58 B/F slice replaces a frontdoor:
- `scripts/pse-env` now execs `pse-host boundary`;
- the hooks call `scripts/pse-host agent-hook`;
- the doctor and telemetry-service recipes run natively.

The cutover unit is the consumer, not the responsibility. Every policy that a retained Python
consumer still needs therefore stays implemented, and writable, in Python. Native code also builds
compatibility seams so that it can interoperate during the transition. Those seams are themselves
further declarations of the same facts.

**Manifestations read at source.** Some rows are two writable authorities: the server pin, the
admission ledgers, the service `config.json`. The others are duplicate interpreters of one
declaration: capacity profiles, telemetry resolution and state directory, compiler-cache rule.
`execution_profiles.py` reads Rust-materialized data ("Rust owns materialization"), consistent
with DA-036's resolution, so it counts as an interpreter, not an authority.

| Fact or policy | Native side | Live Python side | Notes |
|---|---|---|---|
| SurrealDB source tuple: version, revision, engine, features, patch list, DiskANN digest | `crates/pse-host/src/server.rs:23-34` | `scripts/surreal_source.py:29-54` | `server.rs:273-283` reproduces "the original recipe's sorted Python JSON spelling" and imports Python-built releases |
| Finite admission ledger | native journal under `XDG_RUNTIME_DIR` (`admission.rs:917-930, 1037`) | `host-admission/allocations.json`, charged by `service_allocation` (`surreal_server.py:5793-5830`) | Native reads the Python ledger only for compiler-cache residents (`admission.rs:1124-1180`; `manager.rs:1499-1509`). Python never sees the native journal |
| Production profiles in `.config/agent-capacity.toml` | runtime `ProductionProfiles` (`native_deployment_protocol.rs:50-120`) and the host's partial `Production` (`admission.rs:117-145, 395-450`) | `host_admission.py`, `execution_profiles.py`, `pse_env.py` | Three typed readers of one file. The product's scientific allocation also lives in a file named for agent capacity |
| Telemetry settings resolution | `pse_host::telemetry::resolve` (`telemetry.rs:148`) | `telemetry_config.resolve_process` (`telemetry_config.py:228`), used by store-launched processes (`telemetry_launch.py:12-27`) | Both produce `PSE_NATIVE_TELEMETRY_SETTINGS` |
| Telemetry state directory | `telemetry.rs:102-118` takes an empty `XDG_STATE_HOME` literally; `telemetry_service.rs:93-100` filters it | `telemetry_config.py:42-48` (literal); `telemetry_service.py:76-87` (filtered) | Four derivations with two empty-value rules. The native writer and resolvers can choose different directories |
| Telemetry service files | Rust writes `configuration.json` / `service.json` | Python reads them (`telemetry_service.py:641-650`) | A cross-language file contract with no schema owner |
| Compiler-cache selection | `environment.rs:181-236` | `build_environment.py:46-124`, through `just build-frontend` (`justfile:1328-1329`) | |
| Doctor | native `pse-host doctor` (`justfile:96-103`) | `doctor.py --extension-kind` for `py-test`/`py-unit` (`justfile:801, 813`) | |
| Canonical service `config.json` | `native_service.rs` writes it for fixtures | `surreal_server.py` / surrealkit write it for the operator store | Readers and their typing: [DA-052](boundaries-and-composition.md#da-052) |

**Consequence.**
- A change to capacity, a compiler-cache rule, telemetry selection or the server pin needs
  coordinated edits in two languages, and drift between them is silent.
- Finite admission accounting is split across two ledgers, and visibility runs one way only.
- Every compatibility seam is one more declaration that a later cutover must find and retire:
  - reading the Python ledger;
  - reproducing Python JSON spelling;
  - importing Python-built releases;
  - sharing state files.
- The test account records the same shape for test routes. There, two Arrow force-validation
  resolvers enforce one invariant ([DTA-063](../test-audit/test-execution-system.md#dta-063)).

**Footprint.** Known: the table above. Suspected: every Python module still reachable from
recipes that encodes host policy.

**Counterevidence and legitimacy.**
- 58b, 58f and 58g declare the remaining consumer cutovers pending.
- ADR-0233 allows intermediate paths to be unavailable without a fallback, and native code
  refuses unsupported selections rather than delegating to Python.
- The telemetry scalar bounds are single-sourced through a generated Python projection.
- Finite startup is gated by a MemAvailable pressure guard (`admission.rs:1319`).
- The legacy-ledger read is read-only and validated.
- The transition is days old.

It is a deliberate state, then. The diagnosis is that the cutover unit chosen makes intermediate
states carry two authorities or interpreters. The compatibility seams are bridges in I01's sense:
reading the Python ledger (`admission.rs:1124-1180`) and reproducing Python JSON spelling
(`server.rs:273-283`). They lengthen that period rather than shorten it.

**Uncertainty.** I did not establish whether native finite admission accounts for the persistent
Python-supervised store through some other path.

## <a id="da-058"></a>DA-058 — The edit guard's enforcement point depends on an explicitly built executable, and its protected-path set is hand-maintained in several places

**Principles:** AP-05, DP-03, DP-01.

**Status:** new on 2026-10-29. It is a manifestation of the native hook cutover (`f3a05e0be`).

**Claim at risk.** AGENTS.md: a native `PreToolUse` hook blocks edits to generated paths,
`external/`, `build/`, `target/`, the authoritative design and accepted ADRs.

**Structural cause.**
- Both runtimes now invoke `scripts/pse-host agent-hook guard` (`.claude/settings.json:70`,
  `.codex/hooks.json:8`).
- `scripts/pse-host` exits 125 when `target/host-tools/current/pse-host` has not been prepared by
  `scripts/bootstrap-host` (`scripts/pse-host:8-11`).
- The guard blocks with exit 2 (`crates/pse-host/src/agent_hooks.rs:166-169`).
- A checkout or worktree without prepared host tools therefore yields a non-blocking hook
  failure. `target/` is per checkout, and `just worktree` does not prepare host tools.
- Protection of the authoritative design and accepted ADRs exists only in the hook
  (`agent_hooks.rs:78-86`). The settings deny list (`.claude/settings.json:18-35`) covers
  generated and external paths only, so AGENTS.md's statement that the settings "deny them again"
  is false for those two classes.
- The generated-path set is written by hand in at least seven places:
  - AGENTS.md prime directive 2;
  - the settings deny list;
  - `agent_hooks.rs:73-76`;
  - the now-unwired `scripts/agent-hooks.py`;
  - `xtask/src/codegen.rs` `retired_outputs` (around `:380-392`);
  - `tests/governance/tests/common/mod.rs` `is_generated`;
  - the `pse-codegen` roots test that mirrors prime directive 2.

  Only the last checks one copy against another.
- `CLAUDE.md:9` still states that both runtimes call `scripts/agent-hooks.py`.

**Consequence.**
- In an unprepared checkout, decision-document protection silently does not apply.
- Adding a generated output means editing several lists that nothing reconciles.
- An agent reading `CLAUDE.md` forms a wrong picture of the enforcement point.

**Counterevidence.**
- The guard fails closed on its own internal errors.
- AGENTS.md says the hook "is not a shell sandbox".
- The deny list still covers generated paths independently of the hook.

**Uncertainty.** Claude Code's hook contract documents exit code 2 as blocking and other
non-zero codes as non-blocking errors. That is external behavior, not checked in-repo. Codex hook
semantics were not checked.

## <a id="da-043"></a>DA-043 — Native boundary crossings revalidate the same inputs per invocation

**Principles:** AP-07, DP-09, DP-10; H12, H14, H18, H19.

**Status:** reopened against Plan56's resolved disposition (CA06–CA08), inherited by the native
replacement and reshaped. The per-call MCP worker this
finding originally described was deleted with the Python MCP (`f321e9e0f`). The tooling-authored
currency key survives only for `pse-py` and Python-test consumers (`scripts/python_extension.py:714, 758`),
which are retiring under 58g G03 and 58f. The cause, per-invocation re-establishment of facts
that did not change, reappears in native entrypoints.

**Claim at risk.** ADR-0233: establish external artifact validity at selection, then carry the
capability rather than re-scanning per read.

**Structural cause.**
- Every `scripts/pse-env` invocation does the following before `foreground` discovers that it is
  nested and simply execs (`crates/pse-host/src/foreground.rs:242-292`):
  - composes the environment (`boundary.rs:197-204`);
  - when `.envrc.local` exists, evaluates it in an isolated bash (`environment.rs:552-600`);
  - loads the capacity policy and `/proc/meminfo` (`boundary.rs:206-228`);
  - for `--native`, takes a SHA-256 of each recorded system tool: cc, c++, cmake, clang, the clang
    resource directory, libclang and sccache (`native.rs:346-449, 764-770`; `providers.rs:161-166`).
- The justfile mentions `scripts/pse-env` 119 times, 79 of them as `[script(…)]` wrappers. Where
  a wrapped body calls `scripts/pse-env` again, the fixed work repeats with nesting. The number of
  truly nested calls was not counted.
- The native project CLI streams its own executable into a digest on every invocation, before it
  builds the runtime (`crates/pse-runtime/src/bin/project/runtime.rs:229-231`). A development
  build of that binary is about 1 GB.
- `pse-worker` hashes the same kind of artifact with a different scheme
  (`FramedHasher(CanonicalPayloadV1).part_reader`, `bin/pse-worker.rs:341-360`, versus
  `EncodingHasher`). Both values feed `OuterAttestation::build`.

**Consequence.**
- Per-command fixed work grows with recipe nesting and with binary size, not with the work
  launched.
- Two producer-identity derivations exist for one kind of fact. Whether any consumer compares
  them was not established.

**Counterevidence.**
- System tools live outside the immutable generation, so a recheck at use guards a real
  uncertainty (an OS upgrade).
- No source tree is scanned per command.

**Uncertainty.** None of this cost is measured, and it may be small relative to the launched
work. This is the least consequential cause on the page.

## Retired identities

- <a id="da-036"></a>**DA-036** — *Finite execution-profile values are restated, and conformance
  settings govern production allocation.*
  - Resolved for authority: `scripts/execution_profiles.py` states "Rust owns materialization",
    and the deployment document is generated.
  - The coexistence of native and Python admission readers over one policy file is now a
    manifestation of [DA-057](#da-057).
  - Full text at source `682fc3830`.
- <a id="da-037"></a>**DA-037** — *Repeated process observations sit inside cyclic tooling
  coupling that includes production supervision.*
  - The native host owns the boundary entry (`scripts/pse-env:7`).
  - The Python reciprocal web (`scripts/pse_env.py`, `native_operation.py` and their consumers
    `native_tests.py`, `evidence_store.py`, `build_storage.py`, `installation_context.py`)
    survives only behind retained consumers.
  - Merged into [DA-057](#da-057). Owning packages are 58b B03/B04 and 58f F01/F02/F05.
  - Full text at source `682fc3830`.
