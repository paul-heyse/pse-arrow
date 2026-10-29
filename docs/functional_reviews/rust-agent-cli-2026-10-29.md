---
title: An agent-first Rust CLI for pse-arrow
status: proposed
date: 2026-10-29
doc_role: review
doc_topics: [environment, libraries, design, current-work]
---

# An agent-first Rust CLI for pse-arrow

## Judgment and evidence boundary

**Proposed:** give agents one Rust front door, `pse`, built with clap. It routes each verb to
the existing owner: `pse-host` for environment, placement, admission and process ownership; the
correctness adapter for Cargo and nextest; xtask only for heavy development composition. It adds
no second implementation of any of them. The opportunities, in order of measured value:

1. **One output, failure and exit contract.** Every verb is summary-first, bounded and
   versioned, with two exit classes and machine-readable diagnostics ([O02](#O02)).
2. **Owned runs with a file ledger:** launch, wait, result, status and cancel, without a daemon.
   Runs are foreground-owned by default, and detachment is explicit and admitted ([O03](#O03)).
3. **Intent-level `test` and `check` verbs** that derive features, native capabilities,
   force-validation and filters from existing declarations. `pse test` is also where the
   accepted blueprint §24.1 reconciled-pass verdict gets its Rust owner ([O04](#O04)).
4. **Fixture and artifact resolution** that removes hand-plumbed `PSE_NATIVE_FIXTURE_*`
   variables and receipt paths kept in `/tmp`. It starts with fixes that need no new CLI
   ([O05](#O05)).
5. **A grammar agents can discover**, derived from the clap definitions, with `explain` before
   effects and provenance that never runs shell code ([O06](#O06)).

Two smaller opportunities address reading and orientation ([O07](#O07), [O08](#O08)). The
cheapest step for reading is a configuration trial, not new code. Several tempting additions are
explicitly not recommended ([Not recommended](#not-recommended)).

**Operator directions that shaped this review:**
- The Python capability CLI, Python MCP server, operation catalogue and `pse-py` boundary were
  retired today (`f321e9e0f`, `600d4df30`). That retirement is why the operator asked for a
  Rust CLI.
- The retired interfaces are an idea source, not a baseline or a replacement list.
- The target is the best-in-class interface for Codex and Claude. Where current rules conflict,
  they are recorded as rule impacts needing a decision, not treated as limits on the target
  ([Rule impacts](#rule-impacts-and-decisions-needed)).

**What was inspected:**
- Source baseline `6dc84501d`, inspected 2026-10-29 with an uncommitted working tree.
- The standard selected by `standard.toml`: core 3.4, efficient-architecture heuristics 1.0, test
  assessment 1.2 and process-simulator 1.5, under the pse-arrow binding.
- The supplied overviews, read end to end:
  [serde/serde_json](../libraries/serde_serde-json.md), [schemars](../libraries/schemars.md),
  [tracing](../libraries/rust-tracing.md), [thiserror+miette](../libraries/thiserror_and_miette.md),
  [tokio](../libraries/tokio.md), [indicatif](../libraries/indicateif.md) and
  [figment](../libraries/figment.md).
- [`clap.md`](../libraries/clap.md) is byte-identical to `futures.md` and contains no clap
  material. clap was researched from Context7, docs.rs and the pinned `clap_builder-4.6.7`
  source instead.
- Consequential details were checked against pinned crate sources, local tool help and upstream
  documentation: nextest 0.9.146, figment 0.10.19, indicatif 0.18.6 and tracing-indicatif
  0.3.14.

**Review team:**
- **Mapping.** A code-mapper traced the fixture and test-entry contracts.
- **Library research.** Two researchers covered clap, figment and indicatif, and the run
  lifecycle and output stack.
- **Independent assessment.** A fresh design-reviewer assessed the draft against source.
- Its corrections are integrated below. Where it changed a recommendation, the text says so.

**What ran:**
- **Log mining.** Read-only.
- **Two research probes, in scratch locations only:**
  - a toy crate run through cargo and nextest JSON output;
  - transient user-service units started through `busctl`. The failed units were reset
    afterwards.
- No repository build, test, installation or configuration change.

**Evidence labels:**
- Log statistics are *Measured* under the conditions in [Evidence](#what-agents-actually-do-now).
- Probe observations are *Measured* only for the toy conditions stated.
- Library and source behaviour is *Interface-checked* against pinned source or documentation.
- Every target behaviour and benefit is *Proposed*.

## What agents actually do now

### Method and limits

**Measured** by a read-only miner, which reconstructs all of the numbers below from the session
files:
- Its `mine` step extracts every Codex tool call; its `report` step aggregates them.
- Window: 2026-10-28T22:58Z to 2026-10-29T22:58Z.
- Sessions included: Codex rollouts whose cwd is `/home/paul/pse-arrow`.

Processing:
- **Deduplication.** Calls are deduplicated by `call_id`. Forked subagents copy their parent's
  records, and dedup removed 2,559 duplicate cells.
- **Reproducing.** The published probe
  [`probes/rust-agent-cli-log-miner-2026-10-29.py`](probes/rust-agent-cli-log-miner-2026-10-29.py)
  regenerates every number in this section.
- **Pairing.** Results are paired in call order only when a cell returned one JSON envelope per
  exec/poll call. That covers 3,500 of 6,276 commands.
- **Exit codes.** Cells that printed only raw output carry no exit code.
- **Background runs.** These are followed to their final status through later `write_stdin`
  polls.

Earlier month-long counts were inflated by forks and by pre-retirement history. They are not
used.

Scope and caveats:
- Totals: 82 sessions; 6,276 shell commands, 593 polls and 567 `apply_patch` calls.
- Commands by role: root coordinator 2,974, executor 2,470, code-mapper 408, reviewers 322,
  library-research 93, test-agent 9.
- Most root activity is one long Plan58 coordinator session, so the evidence is concentrated
  rather than a census.
- Claude's recent sessions (5 sessions, 756 Bash calls) are review- and delegation-heavy.
  509 of those calls are `cd`-prefixed, and 171 Agent and 400 Edit calls dominate. They add
  little build/test evidence and are used only for composition requirements.

### Where the commands and tokens go

**Measured**:

| Activity | Commands | Failed (paired) | Backgrounded | Output tokens | Share |
| --- | ---: | ---: | ---: | ---: | ---: |
| Reading and search (`sed`, `rg`, `cat`, `nl`, `git show`) | 4,178 | 127 | 9 | 4.80M | 68% |
| Observing runs (`tail`, `ps`, `df`, `systemctl`) | 453 | 11 | 1 | 1.52M | 21% |
| Git inspection (`status`, `diff`) | 261 | 6 | 0 | 0.20M | 3% |
| Build, test and run (cargo, recipes, host) | 594 | 20 | 367 | 0.15M | 2% |
| Polls of running commands | 593 | — | — | 0.15M | 2% |
| Python heredoc edits (`str.replace`/`write_text`; excludes `PSE_DESIGN_EDIT=1`-prefixed) | 502 | 16 | 84 | 0.09M | 1% |
| Other Python, shell writes, git changes, delegation | 288 | 8 | 30 | 0.15M | 2% |

Other measurements from the same window:
- **Truncation.** 841 of 5,687 unique cells had output truncated by the harness, hiding about
  14.6M original tokens.
- **Chaining.** Commands chain a median of 4 operations (p90 10).
- **Orientation re-reading.**
  - `justfile` was read 358 times; one session read it 95 times.
  - Role, rule, skill and plan-index files were read 786 times: `.agents/roles/worker.md` 110,
    `.claude/rules/rust.md` 79, `docs/plans/README.md` 70, `AGENTS.md` 67.
  - 372 (session, file) pairs were re-read five or more times.
- **Change and dependency lookups.** 701 commands re-derived what had changed (`git status
  --short`, `diff --stat`, `--numstat`). 169 searched `~/.cargo/registry` for exact-version
  dependency source.

### Long runs are launched, then watched by hand

**Measured**:
- **Launch.** 491 commands were backgrounded. 271 of them redirected output into ad hoc files,
  and 386 distinct `/tmp` or `build/` log and JSON paths appeared in one day.
- **Outcomes.** Of 347 background runs whose final status the agent saw, 102 (29%) were non-zero.
  For 54 of those failures the poll showed no output, so the agent had to tail or grep the log.
- **Watching.** Commands that start with `tail` used 1.46M output tokens. A further 155 `ps`
  probes looked for `rustc`, `cargo` or `nextest`, and `just activity` ran 94 times.
- **Existing record store unused.** The nextest record store is enabled in the user config
  (`cargo nextest store list`, `cargo nextest replay -R latest`, `store export latest`), but it
  was used zero times. Instead, JUnit at `target/nextest/default/junit.xml` is overwritten on
  every run.

**Episode — watching hygiene (root, 21:53–22:02):**
- `just hygiene` started under `pse-env --native=compiler,solver,klu,isolation,uno,petsc` with
  output to `/tmp/plan58-bg-final-hygiene.log`.
- Over nine minutes the agent issued about fifteen commands of the form
  `tail -16 …hygiene.log; rg -n '^error|warning:|Failed recipes:' …; ps -C rustc …; df -h .`,
  interleaved with unrelated edits.
- Each check returned 300–2,900 tokens. The final answer was in `run-all`'s `Failed recipes:`
  line.

Earlier reviews proposed a stable run handle and transition watching
([codex-effectiveness O07](codex-effectiveness-and-diagnostics-2026-10-22.md#O07),
[agent-effectiveness O15](agent-effectiveness-opportunities-2026-10-23.md#O15)). Their Python
realizations (`just result`, assessment checks and JUnit reconciliation) were retired with the
Python boundary. Plan58 also retired "bespoke build observation and its live
build-stage/wait/freshness consumers" (58b, 58h).

### Driving Rust after the Python retirement

**Measured** for the sub-window since 06:32Z, when the MCP and catalogue were retired. It
contains 429 build/test/run commands.

**Inline variable assignments (194 in total):**

| Variable | Times |
| --- | ---: |
| `PSE_NATIVE_FIXTURE_SERVICE_HELPER` | 47 |
| `PSE_NATIVE_FIXTURE_MODULE_RECEIPT` | 39 |
| `PSE_NATIVE_FIXTURE_MODULE_STATE` | 35 |
| `PSE_WORKER_BINARY` | 14 |
| `PSE_SCIENTIFIC_INPUTS` | 11 |
| `PSE_NATIVE_FIXTURE_PREDECESSOR_MODULE_STATE` | 7 |
| `PSE_NATIVE_STUDY_WORKER` | 7 |
| `PSE_CANONICAL_TOOLS` | 5 |
| `PSE_DIAGNOSTIC_EVIDENCE` | 5 |

**How the work was spread:**
- **Native capability sets.** Eight different `--native=` sets were chosen by hand across the
  day: `compiler,solver,klu` 99 times, an empty `--native` 39, `compiler,klu` 31, and all six
  capabilities 21.
- **Test entry forms.** Tests were run through four entry forms: the correctness adapter's
  nextest directly (107), `just unit-package` (45), `just test-package` (6) and bare
  `cargo test`.
- **Bare `cargo test`.** Across the full window bare `cargo test` appeared 73 times. It has no
  force-validation closure, nextest profile, setup script or JUnit (code-mapper, from
  `.cargo/config.toml` and `crates/pse-host/src/correctness.rs`).
- **Rebuilds.** `scripts/bootstrap-host` ran 62 times, because the host itself was under change.

**Episode — native fixture test (root, 10:57–11:19).** In twenty minutes the agent:
1. Rebuilt the host (`scripts/bootstrap-host > /tmp/plan58-module-bootstrap.log`).
2. Ran `pse-host study-module prepare --tool-root ~/.cache/pse-arrow/plan58-native-module` into
   another log.
3. Saved the receipt path in `/tmp/plan58-native-module-record-path`.
4. Ran the test as:

   ```sh
   IFS= read -r plan58_module_record < /tmp/plan58-native-module-record-path
   PSE_NATIVE_FIXTURE_MODULE_RECEIPT="$plan58_module_record" \
   PSE_NATIVE_FIXTURE_SERVICE_HELPER=/home/paul/pse-arrow/target/debug/pse-canonical-service \
   scripts/pse-env -- just test-package pse-operations \
     --features canonical-tests,native-service,canonical-maintenance,test-support --lib --test native_service …
   ```

5. Tailed four logs, polled `ps` for `cargo|rustc`, launched a `codex exec` worker into a further
   log, and ran `just ready` into yet another file.

Why it works this way (code-mapper):
- The fixture contracts are hard `expect`s in `crates/pse-runtime/tests/*` and
  `crates/pse-operations/tests/native_service.rs`.
- No recipe sets the receipt, worker, scientific-input or evidence variables.
- No recipe covers the `native_service`, `native_study`, `native_repeated_analysis`,
  `native_worker_lifecycle` or `native_scientific_modes` targets.
- `pse-host study-module prepare` prints the receipt JSON but not a current-receipt pointer.
- For `native_service` itself the helper variable is redundant: the test already uses
  `CARGO_BIN_EXE_pse-canonical-service`, so the episode's assignment did nothing (independent
  assessment).

**Delegation.** 20 `codex exec` workers were launched with `--output-last-message
/tmp/…handoff.md - > /tmp/…worker.log 2>&1` and monitored by tailing their logs. One log was
tailed 28 times.

### Edits and reading

**Measured:**
- **Edits.** Half of all edits used ad hoc Python: 502 heredoc `str.replace`/`write_text` edits
  against 567 `apply_patch` calls, of which 18 cells reported a patch error.
- **Reading.** There were 7,085 `sed -n 'a,bp'` reads (median 60 lines, p90 180). They were
  typically preceded by an `rg -n 'fn …|struct …'` that located the range.
- **Unused aids.** rust-analyzer MCP is configured for Codex but disabled in the repository's
  `.codex/config.toml`; it appeared 3 times. `just affected` was used zero times, although it
  already emits a runnable nextest command.

## Current command surface

These facts come from the mapping passes, with file references in each owner:
- **`pse-host`** (about 23k lines) parses arguments by hand in `crates/pse-host/src/main.rs`.
  - Every module has its own `run_cli` loop. The help string omits `boundary` and `agent-hook`.
  - There is no `--format`. Only `doctor --json` carries a version.
  - `pse-env --explain` always prints JSON, whether or not `--json` is given.
  - Errors print as a single Display line with exit 125. Diagnostic codes exist
    (`impl_diagnostic!`, blueprint §23.2) but are never rendered.
  - No tracing subscriber is installed. Each command builds its own tokio runtime.
- **`cargo-pse-correctness`** owns Arrow force-validation and nextest invocation. It exits 2 on
  its own errors.
- **`xtask`** has 34 clap-derive commands with `anyhow` errors.
  - `affected` emits `affected-preview-v1` with a ready command, but without native
    capabilities, fixtures or integration targets.
  - `lifecycle` emits versioned JSON.
- **The `justfile`** has 171 recipes, every line run through `scripts/pse-env`. Parameter
  grammar lives in `just --usage`.
- **Other Rust binaries:** `pse-canonical-test`, `pse-canonical-service`, `pse-worker`,
  `pse-project` and `pse-evidence` use clap but are separately discovered. `pse-canonical-test`
  needs six explicit paths that only `just native-test` and `just canonical-test` compose.
- **Dependencies:** clap 4.6.7 (derive only; no `env` feature), serde_json, schemars 1.2.2,
  miette 7.6.0 (no `fancy`), tracing and tracing-subscriber (env-filter, registry; no `json`)
  and tokio 1.53.3 are pinned. indicatif and figment are absent.

## Target shape

<a id="O01"></a>
### O01 — One `pse` front door over existing owners

**Who benefits and how:** every Codex and Claude role, and the maintainer.
- One discoverable grammar replaces three: hand-rolled `pse-host` verbs, 171 recipes, and xtask
  plus scattered binaries.
- Every verb follows the same output contract ([O02](#O02)).
- Agents stop reading the `justfile` (358 reads a day) to find how to run something.
- Chains such as `scripts/pse-env --native=… -- scripts/cargo-bin/cargo-pse-correctness nextest
  run --no-fail-fast --locked -p … -E '…'` become intent verbs.

**Proposed shape:**
- **The binary.** Add `[[bin]] name = "pse"` to the existing `pse-host` crate and rebuild its
  argument layer as one clap derive tree. Keep the tiny seed and atomic generation publication of
  `scripts/bootstrap-host`.
- **Keep the hook path.** `scripts/pse-host agent-hook` is the edit guard for both harnesses,
  and a broken path breaks every edit session. Any later rename lands atomically with
  `just setup-test`. AGENTS.md forbids a compatibility shim.
- **Cold start.** `pse` keeps the closure required by ADR-0233 and blueprint §3.2: no scientific,
  Arrow/DataFusion, database-client or product-schema dependencies. clap, schemars and
  tracing-subscriber are admissible under that rule.
- **Enforce the closure.** Add a `host` row to `[workspace.metadata.pse.dependency-ceilings]`.
  It declares the closure in a check `family-check` already runs, so it is not a new lint. A CLI
  attracts dependencies, so this matters.
- **Heavy work stays with its owner.** Verbs that need heavy closures run as subprocesses of the
  owner that already has them:
  - the correctness adapter for Cargo and nextest;
  - xtask for codegen, docs and lifecycle work;
  - product binaries for workers and services.
- **Move `affected` into the host.** It needs only `cargo_metadata` (already a host dependency)
  and git. Dispatching it to xtask would pull in xtask's unconditional `pse-schema`,
  `pse-columnar` and `pse-codegen` closure and lose cold-start independence for
  `pse test --changed`.
- **Find the checkout.** `pse` locates the checkout from its own executable path, as the existing
  `executable_checkout_root` does, and accepts `-C <dir>`. Claude's `cd`-prefixed commands
  (509 of 756) and Codex's `cd … &&` chains become unnecessary for `pse` invocations.

Illustrative tree. Names are proposals, not a frozen list:

```text
pse status [--brief]               live and recent runs (all checkouts/agents), readiness, dirty tree
pse test  <pkg|pkg:target|path|--changed> [-E FILTER] [--detach] [--wait=DUR]
pse check <pkg|--changed>
pse run   [--class C] [--native auto|CAPS] [--detach] -- <argv…>
pse wait  <run> [--timeout DUR]    pse result <run> [--failures] [--fields …]
pse log   <run> [--grep RE] [--tail N]    pse cancel <run>    pse history [--session]
pse fixture prepare|show <name>    pse explain <verb …>    pse env [--explain KEY]
pse changes [--base REF]           pse brief [--paths …]   pse ref <ID>   pse new <kind> <slug>
pse dep <crate>                    pse outline <file>      pse show <file>::<item>
pse commands --json                pse schema <verb>       pse doctor
```

**The justfile becomes optional sugar.**
- Retire recipes that only spelled out an invocation `pse` now derives (`unit-package`,
  `check-package`, `native-test`, `worker-test` and their variants).
- Keep recipes that compose genuinely different work (`hygiene`, `turn-end`, `ready`) as thin
  `pse` calls. Their children become composite runs ([O03](#O03)).
- `scripts/pse-env` stays as the exec adapter for bare tools, with its existing exit/signal
  forwarding.

**Alternatives considered:**

| Alternative | Why not chosen |
| --- | --- |
| (a) Keep recipes and improve them | Recipes cannot carry typed output contracts, ledgers or derived requirements. |
| (b) xtask as the front door | ADR-0233 rejects its closure for cold start. |
| (c) An RMCP server instead of a CLI | Both harnesses already have a shell. [O02](#O02) schemas make a later adapter cheap; it stays conditional, as in Plan58. |
| (d) Separate clap CLIs per owner | This is the status quo with better parsing. Agents still have to know which binary, which wrapper and which variables. |

**Enabling mechanisms** (*Interface-checked*, clap 4.6.7):
- `#[derive(Parser, Subcommand, Args, ValueEnum)]`.
- `#[command(flatten)]` for a shared `GlobalArgs` holding `--format`, `--budget` and `-C`.
- `#[arg(last = true)] extra: Vec<OsString>` for explicit pass-through after `--`. Prefer it to
  `trailing_var_arg`, which can mis-claim flags.
- `Command::defer` for lazily built subtrees on cold start.
- `Cli::command().debug_assert()` in a unit test.
- Avoid `infer_subcommands` and `infer_long_args`: prefix matching changes meaning when verbs
  are added.
- Measured parser overhead in argparse-rosetta-rs is about 574 KiB release and a 170 ms
  incremental debug build.

**Delivery demonstration:** an agent completes the native fixture episode above using only `pse`
verbs, with no inline `PSE_*` assignments, no `/tmp` paths and no `cd` prefix.

## CLI opportunities

<a id="O02"></a>
### O02 — One agent-first output, failure and exit contract

**Value.** Today's output is a mix:
- one-line Display errors with exit 125;
- unversioned JSON, with `--explain` ignoring `--json`;
- cargo and nextest text that has to be grepped;
- silent background failures (54 in the window);
- output truncated by the harness (841 cells).

Under this contract, each verb's first lines answer the question asked, and everything else is a
pointer.

**Proposed contract:**
- **Global `--format auto|human|json|ndjson`**, mirrored as `PSE_FORMAT`.
  - `auto` means human on a TTY. Off a TTY it means a compact summary block plus artifact paths.
    Both harnesses run off a TTY.
  - Use one name everywhere. uv's mix of `--format` and `--output-format` is the anti-pattern.
- **A versioned envelope for every result:**

  ```json
  {"format_version": 1, "command": "test", "status": "failed",
   "summary": "pse-host foreground: 1 failed, 10 passed in 12.4s",
   "run": "r-<checkout>-20261029T215317-7f3a",
   "child_exit": 100,
   "failures": [{"kind": "test", "id": "pse-host::foreground::…",
                 "message": "…", "location": "crates/pse-host/tests/foreground.rs:212"}],
   "artifacts": {"log": "…/output.log", "events": "…/events.ndjson"},
   "next": ["pse log r-… --grep foreground_explicit", "pse test pse-host --rerun-failed"]}
  ```

  - `--fields` selects fields, as in `gh --json`.
  - `--budget N` caps the rendered summary, keeping the summary and first failures and replacing
    the rest with counts and paths.
  - Raw output is always one pointer away, never discarded.
- **Streams** are NDJSON with a `type` discriminator (`started`, `progress`, `failure`,
  `finished`) and always end with a `summary` record.
- **Two exit classes.** These reconcile with blueprint §24.1, which keeps `scripts/pse-env`'s
  exit and signal forwarding.
  - **Exec-adapter verbs** (`pse run` in the foreground, `scripts/pse-env`) forward the child's
    exit code, or 128+signal. 125 stays reserved for boundary failure.
  - **Managed verbs** (`test`, `check`, composite recipes) follow a reconciled taxonomy:
    - 0 is a complete reconciled pass ([O04](#O04));
    - 1 means the subject failed;
    - 2 is a usage error;
    - 125 is a boundary or tool failure.

    The child's raw exit always appears in the envelope as `child_exit`.
  - **Distinct statuses, each with its own exit code:** `launched` for an accepted detached run,
    and `running` for a `--wait` timeout that elapsed before the run finished. An agent reading
    exit codes can then never mistake a launch for a pass.
- **Diagnostics.** These carry blueprint §23.2 codes, a `help` remedy and the next command.
  Multiple independent failures are `related` diagnostics, not a cause chain.

**Enabling mechanisms** (*Interface-checked*):
- **schemars 1.2:** `SchemaSettings::draft2020_12().for_serialize()` generates each envelope's
  schema, published by `pse schema <verb>` ([schemars §2.3, §5](../libraries/schemars.md)).
  Version envelopes explicitly (DP-24) rather than relying on schemars' generated structure
  staying identical across upgrades (schemars §12.3).
- **serde_json:**
  - `RawValue` passes selected cargo or nextest records through without re-modelling them. Its
    schemars impl needs schemars' `raw_value` feature, which enables `serde_json/raw_value`;
    neither is enabled in the current pins.
  - `to_writer` streams NDJSON ([serde §5.4, §6.1](../libraries/serde_serde-json.md)).
- **miette 7.6:**
  - Serialize an owned `ErrorRecord` from the `Diagnostic` trait (`code()`, `severity()`,
    `help()`, `url()`, `labels()`, `related()`).
  - Do not emit through `JSONReportHandler`. Its 7.6.0 source formats JSON by hand, leaves `url`
    unescaped, gives labels as byte offsets without line or column, and takes `filename` from
    the first label only. It is a model of the shape, not an emitter.
  - For human output, use `NarratableReportHandler` on a non-TTY stderr and
    `GraphicalReportHandler` (feature `fancy-no-syscall`) on a TTY. Render per audience, not
    through a global hook ([thiserror+miette §4](../libraries/thiserror_and_miette.md)).
  - Never pass diagnostic-carrying errors through `.into_diagnostic()`, which erases their
    metadata.
- **clap parse errors** become structured diagnostics:
  - `Error::kind()` (a non-exhaustive `ErrorKind`) maps to stable codes;
  - `Error::context()` gives `ContextKind::{InvalidArg, SuggestedArg, ValidValue, …}`;
  - `DisplayHelp` and `DisplayVersion` arrive as `Err` with exit 0 and must not become failures;
  - the format is chosen before parsing completes, from `PSE_FORMAT`, TTY detection or an argv
    pre-scan.

**Demonstration:** the hygiene episode becomes one `pse wait` and one `pse result --failures`.
Each returns within its budget and names the failing recipe and its first error with location.

<a id="O03"></a>
### O03 — Owned runs with a file ledger: launch, wait, result, status, cancel, history

**Value.**
- **Watching runs is expensive.** It is the second-largest token sink (21%), and 29% of observed
  long runs fail.
- **A run handle replaces the loop.** "Redirect to `/tmp`, then `tail`, `ps` and `df` in a loop"
  becomes a run with terminal semantics and summary-first failures. This restores, without
  Python and without a daemon, the run-handle value earlier reviews identified.
- **History for resuming work.** The ledger is also each session's authoritative record of what
  ran and how it ended. That helps a coordinator reviewing workers' runs, an agent resuming after
  context compaction, and future reviews, which today have to reverse-engineer harness logs.

**Proposed behaviour:**
- **Foreground-owned by default.**
  - `pse test` and `pse run` use the existing foreground scope, bound to the caller. A harness
    timeout or kill therefore still stops heavy builds; the two editor-wide OOM incidents make
    this the safe default.
  - `--wait=DUR` keeps the run foreground-owned. If DUR elapses, the verb kills nothing and
    reports `running` with the run ID, but only for a run that was detached from the start.
    Otherwise the timeout bounds the run itself.
- **Explicit detachment.**
  - `--detach` launches a transient user service through the shared admission gate in
    `admission.rs`, not only a `Slice` property. It returns `launched` with the run ID.
  - `pse wait <run>` blocks for a bounded time on terminal state. It defaults below the harness
    tool timeouts and reports "still running" compactly. This replaces repeated Codex
    `write_stdin` polls and log tails.
  - For Claude's background tasks and Monitor, `pse wait <run> --format ndjson` prints one line
    per transition.
- **Run directories.** Each run gets `<state>/runs/<run-id>/` holding `meta.json` (command,
  resolved-environment digest, placement, unit, session), `output.log`, `events.ndjson` and
  `result.json`.
  - Run IDs include a checkout identity, because `XDG_STATE_HOME` is shared by all checkouts.
  - The session identity comes from the harness: Claude's `agent-hook session-env` can export it,
    and Codex can supply its thread ID through the per-command bridge.
- **Viewing runs:**
  - `pse status` lists active and recent runs across checkouts and agents with cgroup memory and
    CPU.
  - `pse history --session` lists one session's runs and outcomes.
  - `pse result` reads the extracted outcome, and `pse log` gives bounded, filtered raw output.
- **Composite runs.** `run-all`-style bundles (`hygiene`, `turn-end`, `ready`) give each child its
  own run ID, recorded by the parent. The hygiene answer is then "child `clippy-default` failed:
  first error …".
- **Cancel.** `pse cancel <run>` stops a detached run. Under ADR-0233, stop authority belongs to
  the exact bus client in `Unit.Refs`, and `RefUnit` is never issued. A later `pse` holds only a
  borrowed view, so transferring stop authority through the ledger needs an ADR-0233 decision
  ([Rule impacts](#rule-impacts-and-decisions-needed)).
- **Run kinds.** Cargo, nextest and xtask runs have extractors. Generic commands, including
  `codex exec` workers, record exit, duration, log tail and declared output files such as
  `--output-last-message`.

**Enabling mechanisms:** research probes ran on this machine with systemd 255, nextest 0.9.146
and a toy crate. Code facts come from the independent assessment.
- **Compiler failures** (*Interface-checked*; *Measured* in a toy probe).
  - Run cargo with plain `--message-format=json` and parse it with
    `cargo_metadata::Message::parse_stream` (0.23.1, already a host dependency).
  - Build the one-line `file:line:col: level[code]: message` form from the primary span. Write
    the full `rendered` text to the run directory.
  - Deduplicate on (level, code, primary span, message): the lib and lib-test units each
    reported the same error.
  - Do not use `json-render-diagnostics`, which omits rustc diagnostics from the JSON.
  - Inside `pse test`, compile failures (nextest exit 101) come from nextest's
    `--cargo-message-format json`.
- **Test failures** (*Interface-checked*; *Measured* in a toy probe).
  - Per-test results and inline failure output come from `--message-format libtest-json-plus`
    with `--message-format-version 0.1`. This is experimental and needs
    `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`.
  - That stream and cargo JSON share stdout, with discriminators `type` versus `reason`.
  - Two things must be confirmed during implementation: that the correctness adapter passes both
    through intact, and how the stream is told apart from Cargo's.
  - `nextest-metadata` `NextestExitCode` classifies runs cheaply: 100 test failure, 101 build,
    4 no tests, 105 setup script, 106 incomplete, and 94/96/104 for selection and setup.
- **No per-run JUnit.** The adapter's `reject_reuse` refuses `--tool-config-file`, `--rerun` and
  `-R`. The captured libtest stream in the run directory is already per-run, so per-run JUnit is
  unnecessary.
- **The record store is not the run's source of truth.** It stays a human deep-dive tool:
  - `store list` and `store info` have no JSON output in 0.9.146;
  - the store format changed in 0.9.150;
  - `-R latest` races between agents sharing a checkout.
- **Detached supervision without a daemon** (*Measured* with `busctl` against the user manager).
  - **Service, not scope.** A transient service started through the existing zbus manager
    outlives its caller.
  - **Its own property set:**
    - `Type=exec` with `RemainAfterExit=false`;
    - `ExecStart` running `pse __supervise <id>`;
    - an `ExecStopPostEx` `pse __finalize <id>` that receives `$SERVICE_RESULT`, `$EXIT_CODE` and
      `$EXIT_STATUS`;
    - `StandardOutputFileToAppend`/`StandardErrorFileToAppend` (file output avoids journald rate
      limits);
    - `RuntimeMaxUSec`, `KillMode=control-group`, `WorkingDirectory`, `Environment`, and a
      `Slice` from admission.
  - **Don't reuse `start_service_inner`.** Its set (`RemainAfterExit=true`, `AddRef=true`,
    `CollectMode=inactive-or-failed`) leaves the unit active(exited), so the finalizer would never
    run at completion.
  - **The ledger is the only result record.** A successful unit without `RemainAfterExit` is
    garbage-collected at once.
  - **The supervisor** spawns the child with tokio `process::Command` and `kill_on_drop`, parses
    both pipes in a `JoinSet`, and honours a `CancellationToken` wired to `SIGTERM`.
  - **Waiting.** `Subscribe()` with `JobRemoved`/`PropertiesChanged`, or `pidfd_open` on
    `ExecMainPID`, avoids busy polling. A pidfd only signals completion; the status comes from
    the ledger.
  - **Lost observation.** A vanished unit without a terminal record is reported as *lost
    observation*, never as success. `$INVOCATION_ID` makes a natural run-ID salt, and linger is
    enabled for this user.
  - **Fallback.** Double-fork with a state file is a last resort: it stays in the harness's
    cgroup.
- **Ledger writes** (*Interface-checked*).
  - Terminal files are written by temp file, fsync, then `rustix::fs::renameat_with(…,
    RenameFlags::NOREPLACE)`.
  - Shared indexes reuse admission's non-blocking `flock` pattern on `O_CLOEXEC` descriptors.
  - State transitions are written synchronously.
  - A tracing-subscriber JSON layer (feature `json`) with a per-layer filter can carry verbose
    events into `events.ndjson` while stderr stays compact
    ([tracing §3.4, §3.7](../libraries/rust-tracing.md)).
  - The ledger is authoritative. Export to the retained OpenTelemetry collector service is
    optional and derived (DP-01).
- **Prior art.**
  - Bazel's build event protocol has a terminal event carrying the authoritative exit code, with
    logs by reference.
  - `buck2 log what-failed`/`what-ran` query a persisted invocation log, defaulting to the last
    invocation.
  - `gh run view --json … --log-failed --exit-status` gives field-selected, failure-first access.

**Limits:**
- Bare-tool runs outside `pse` are not ledgered. `status` still shows them through the existing
  activity observation.
- `libtest-json-plus` is experimental. Pin the format version and fall back to exit codes plus
  the log tail.

<a id="O04"></a>
### O04 — Intent-level `test` and `check` that derive their own closure

**Value.** Agents currently choose every part of a test run by hand:
- native capability sets (eight distinct sets in a day);
- features (re-typing `--features canonical-tests,native-service,…`);
- filters (`-E 'test(a) | test(b) | …'`);
- entry forms, including 73 bare `cargo test` runs that skip force-validation.

This also gives blueprint §24.1's accepted reconciled-pass rule its Rust owner. The rule was
implemented in Python before, and JUnit reconciliation was retired with it.

**Proposed behaviour:**
- **Named targets.** `pse test pse-operations:native_service`:
  1. reads the target's `required-features` from Cargo metadata;
  2. derives the native capabilities those features reach;
  3. prepares fixtures ([O05](#O05));
  4. runs nextest through the correctness adapter, with force-validation, under the right
     placement.
- **The reconciled verdict.** Exit 0 only when nextest exits 0 *and* the libtest records cover
  every selected test, ending in a terminal record. Otherwise the result is `incomplete`, with the
  raw exit in `child_exit`.
- **Changed code.** `pse test --changed [--base REF]` uses the host-resident `affected` selection
  ([O01](#O01)). It widens to the integration targets and native packages that `affected`
  currently lists only as exclusions.
- **Explain and rerun.** `--explain` prints the resolved command, features, capabilities and
  fixtures without effects. `--rerun-failed` reruns the failed identities from the run's own
  ledger as an exact filterset; the adapter refuses nextest's `-R`.
- **Filters.** A bare word stays `test(word)`. Full filtersets pass through `-E`.
- **Check.** `pse check` mirrors this with `cargo check --message-format=json` and deduplicated
  compiler-message extraction.

**Enabling mechanisms** (*Interface-checked*):
- `cargo_metadata` `Target::required_features` and the package graph.
- The resolved feature graph the correctness adapter already computes for force-validation.
- nextest filtersets (`rdeps(=pkg)`, `package()`, `test()`; see the
  [nextest overview](../libraries/nextest.md)).
- **One declaration of capabilities.** Which provider each feature needs is currently known only
  to `crates/pse-backend-native/build.rs`, which reads
  `CARGO_FEATURE_IPOPT/UNO/PETSC/SCIP/KINSOL/ROOT_ISOLATION` and provider receipts, and to
  callers' `--native=` choices.
  - Declare the mapping once, on `pse-backend-native`'s features, as package metadata.
  - `pse` derives which features a selection reaches from the resolved graph.
  - `build.rs` reads the same declaration (DP-01).
  - Do not declare it in every consuming crate.

**Demonstration:** `pse test pse-runtime:native_study` and `pse test --changed` complete with a
reconciled typed outcome, without caller-supplied `--native`, `--features` or fixture variables.
`--explain` matches what ran.

<a id="O05"></a>
### O05 — Fixtures and artifacts without environment plumbing

**Value.** This addresses the 194 inline assignments, receipt paths read from `/tmp` files, and
integration targets that no recipe covers.

**Realizations, cheapest first:**
1. **Cargo already provides most of this, on stable Cargo.**
   - `pse-worker` is a binary in `pse-runtime`. `pse-runtime`'s integration tests can use
     `env!("CARGO_BIN_EXE_pse-worker")`, provided the test target enables the bin's required
     feature `product-operators`.
   - That removes `PSE_WORKER_BINARY` and `PSE_NATIVE_STUDY_WORKER`.
   - `crates/pse-operations/tests/native_service.rs` already uses
     `CARGO_BIN_EXE_pse-canonical-service`. The episode's `PSE_NATIVE_FIXTURE_SERVICE_HELPER`
     assignment for that target did nothing, which shows a discovery gap rather than a plumbing
     need.
2. **Receipts get a current pointer.**
   - `pse-host study-module prepare` already caches idempotently. It should also record a
     per-checkout current-receipt pointer, for example `target/pse/fixtures/study-module.json`.
   - `pse fixture prepare|show` wraps it, along with scientific inputs and evidence directories.
   - `pse test` sets the `PSE_NATIVE_FIXTURE_*` variables from these records when a target
     declares that it needs them, in `[package.metadata.pse.test.<target>] fixtures = [...]`.
   - Missing preparation is a typed `fixture.unprepared` diagnostic whose `next` field holds the
     exact prepare command.
   - Historical module-state selectors (`…_MODULE_STATE`, `…_PREDECESSOR_…`) stay explicit
     `pse fixture` arguments.
3. **Database cases reuse `pse-canonical-test`.** `pse test` drives it for these cases, because it
   already owns the fresh-service-per-run lifecycle with `--helper` and `--module-receipt`.
4. **Cargo artifact dependencies come last.** `-Z bindeps` with `artifact = "bin"` in
   `[dev-dependencies]`, giving `CARGO_BIN_FILE_PSE_OPERATIONS_pse-canonical-service`, is needed
   only for the helper that `pse-runtime` tests consume across packages. It has two risks:
   - artifact edges and their features must pass ADR-0170 force-validation through the adapter's
     unit-graph check;
   - a `native-service` feature arriving through a dev artifact interacts with workspace feature
     unification (ADR-0122).

   *Interface-checked* against the Cargo unstable reference only.

**Demonstration:** the native fixture episode becomes `pse test pse-operations:native_service`,
with no `/tmp` file.

<a id="O06"></a>
### O06 — A grammar agents can discover, explain before effects, configuration provenance

**Value.** Agents learn commands by reading the `justfile`, recipe bodies and Rust source, and
reconstruct environments by hand. In this session, two mapping agents spent most of their work
establishing the command surface. One derived catalog and an `explain` on every verb answer that
directly.

**Proposed behaviour:**
- **Catalog.** `pse commands --json` derives the whole tree from the clap definitions. Each leaf
  carries its about text, arguments, possible values, env mirrors, defaults, examples and
  output-schema reference. `pse help --json <path…>` returns one node.
- **Help tiers.** `-h` stays one line per argument; examples go in `after_long_help`.
- **Explain.** `pse explain <verb …>` and a global `--explain` show:
  - underlying commands;
  - placement class;
  - native capabilities;
  - features, fixtures and environment changes.
- **Configuration provenance.** `pse env --explain [KEY]` shows each selected value and its layer
  (repository default, `.envrc.local`, caller environment or command line).
  - It never sources shell. `environment.rs` documents that reading and explaining never evaluate
    `.envrc.local`, and ADR-0233 says help, read and explain never build.
  - So `explain` extends the existing layered resolution in `environment.rs` and reports
    unevaluated `.envrc.local` values as pending (the existing `local_pending`).

**Enabling mechanisms** (*Interface-checked*):
- **Catalog walk over `Cli::command()` after `build()`:**
  - `get_subcommands`, `get_arguments`, `get_possible_values` (from `ValueEnum` with
    `#[value(help)]`), `get_env` (needs clap's `env` feature), `get_default_values`,
    `get_value_hint`, `get_arg_conflicts_with`.
  - About 150–250 lines, serialized with its own schemars schema.
  - clap has no public `requires` getter. Carry that, plus examples and output-schema IDs, through
    `unstable-ext` `Command::add`/`Arg::add`, which is acceptable with an exact pin.
  - `clap_usage`/`usage-lib` is an optional second export, with its `docs` feature off.
- **Provenance:**
  - clap's `ArgMatches::value_source(id)` (`ValueSource::CommandLine`) keeps defaults from
    masquerading as overrides.
  - Each key has one owner: don't declare clap `env =` for keys the environment layer also reads.
- **Until `pse` exists:** AGENTS.md's start route can point at `just --list` and
  `just --usage <recipe>` instead of the recipe bodies.

<a id="O07"></a>
### O07 — Change-scoped orientation and delegation briefs

**Value.**
- 701 commands re-derived the change set, although git inspection is only 3% of output tokens.
- More to the point, briefing a subagent means restating the same facts every time: baseline,
  active plan, applicable rules, pinned versions, conventions. I did this four times in this
  session.

**Proposed behaviour:**
- **`pse changes [--base REF]`** reports, in one bounded answer:
  - changed paths grouped by owning crate;
  - generated and protected paths (from the existing guard tables in `agent_hooks.rs`) with their
    regeneration verb;
  - the affected tests, as a ready `pse test --changed`;
  - the path rules that apply (`.claude/rules/*` by scope).
- **`pse brief [--paths …] [--libraries …]`** renders a context packet for delegation:
  - HEAD and dirty state;
  - active plan pointers;
  - applicable rule files with content hashes;
  - exact pins, features and overview paths for the named libraries;
  - the evidence-label vocabulary;
  - the review and plan conventions that apply.

  A coordinator pastes it into a brief, and a worker who receives it need not re-read what the
  hashes show is unchanged.

**Mechanisms:** git subprocesses, `cargo_metadata` package ownership, the guard path tables, and
the `[workspace.metadata.pse.pins]` reasons.

<a id="O08"></a>
### O08 — Reading: semantic navigation, dependency facts, structural extracts

**Value.** Reading is 68% of output tokens. Most reads are `rg -n 'fn …'` followed by
`sed -n 'a,bp'` guesses, with heavy re-reading. A further 169 commands searched the Cargo
registry for exact dependency source.

**Proposed behaviour:**
- **Enable the configured rust-analyzer MCP for Codex.** In `.codex/config.toml` it is one line:
  `enabled = false`. Use it for definitions, references and implementations ([O11](#O11)).
- **`pse dep <crate>`** prints:
  - the exact pinned version and enabled features;
  - which workspace crates use it;
  - its `[workspace.metadata.pse.pins]` reason;
  - its locked source path;
  - its overview document, if any.

  The research agents in this session each had to rediscover these facts.
- **`pse outline <file>` and `pse show <file>::<Type>::<method>`** give syntactic, server-free
  item extraction with line numbers.
  - `syn` 3.0.5 and `proc-macro2` with `span-locations` are already pinned.
  - Markdown headings reuse the existing `just lib-outline` logic.
  - These complement rust-analyzer, which needs a warm server and answers different questions.

<a id="development-environment"></a>
## Development-environment opportunities beyond the CLI

These serve the same goal, agent effectiveness, and need little or no new CLI code.

<a id="O09"></a>
### O09 — Quiet per-command activation output for Codex

**Measured** (a one-off read-only query over the same deduplicated window): 4,737 `direnv:`
lines (≈156k tokens, 1.1% of visible output) were
printed into Codex command outputs.
- Codex activates per command through the machine bridge `~/.config/codex/bash-env.bash`.
- Each command therefore prints `direnv: unloading`, `loading …/.envrc` and a long `export +…`
  variable list.

**Proposed:**
- Set `DIRENV_LOG_FORMAT=` (empty) in the Codex bridge only. The installed direnv 2.32.1 supports
  it.
- Keep the bridge's own explicit failure path (`exit 125` with a message), so a failed or blocked
  activation still shows.
- If direnv is upgraded to 2.34 or later, `[global] hide_env_diff = true` in
  `~/.config/direnv/direnv.toml` is an alternative. It keeps a one-line load notice and drops
  only the long variable diff.

Both are machine-local changes and need no repository change.

<a id="O10"></a>
### O10 — Deliver instructions without repeated file reads

**Measured** (probe report, plus a one-off query for tokens): role, rule, skill and plan-index
files were read 786 times:
`.agents/roles/worker.md` 110 times, `.claude/rules/rust.md` 79, `docs/plans/README.md` (32 KB)
70, and `AGENTS.md` (30 KB) 67. Commands touching them returned at least 459k tokens, counting
paired outputs only.

The cause is visible in configuration: `.codex/agents/executor.toml`'s `developer_instructions`
tell every worker to read `worker.md` and `executor.md` from disk.

**Proposed:**
1. **Generate the role contracts into each Codex agent's `developer_instructions`** from
   `.agents/roles/*`, using the existing `xtask agent-config`. `agent-config-check` (run by
   `just lint-agents`) keeps them in sync, and `.agents/roles` stays the single authority.
   - This needs a decision. CLAUDE.md says native definitions "load shared contracts rather than
     copying role behavior". A generated embedding is not a hand copy, but the rule's wording
     should be amended.
2. **Slim `docs/plans/README.md`** into a short current-work pointer. Agents read it as
   orientation; it is currently a 32 KB index.
3. **Claude's SessionStart hook** (`agent-hook session-env`) can append a compact
   `pse status --brief`: HEAD, dirty count, active plan, live runs. Each session then starts
   oriented.
4. **Drop stale delegation flags.** The root's `codex exec` launches still pass
   `-c mcp_servers.pse-capabilities.enabled=false`, but that server no longer exists in any
   configuration. Where a native Codex subagent (`.codex/agents/*`) fits the role, prefer it;
   otherwise launch the worker through `pse run --detach` so it has a ledger, wait and result
   ([O03](#O03)).

<a id="O11"></a>
### O11 — Immediate fixes that need no new CLI

Each of these works today and removes friction seen in the logs or in this session:
- `env!("CARGO_BIN_EXE_pse-worker")` in `pse-runtime` integration tests ([O05](#O05)).
- A current-receipt pointer written by `pse-host study-module prepare` ([O05](#O05)).
- Enable the rust-analyzer MCP for Codex ([O08](#O08)).
- A `host` row in `[workspace.metadata.pse.dependency-ceilings]` ([O01](#O01)).
- AGENTS.md "Start here" routes:
  - tests through `cargo t` or the `unit-*` recipes, never bare `cargo test`, which skips the
    force-validation closure (73 runs in the window);
  - discovery through `just --list` and `just --usage`.

  These are one-line instruction changes until `pse test` replaces them.
- Replace [`docs/libraries/clap.md`](../libraries/clap.md) with an actual clap overview. Today it
  duplicates `futures.md`.

<a id="O12"></a>
### O12 — Safer bulk edits with existing tools

**Measured** (a one-off query over the same window): 512 Python heredoc edits, including those
prefixed `PSE_DESIGN_EDIT=1`. 350 touched `.rs` files and 152 touched
several files. 200 had no guard: no `assert`, `count`, `index` or `raise`. Python's
`str.replace` silently does nothing when its needle is absent, so an unguarded edit can "succeed"
without changing anything.

**Proposed** (instructions and tools, no new editor):
- Make `apply_patch`, Claude's `Edit` tool and `MultiEdit` the default.
- For repeated structural Rust changes, use `ast-grep run --pattern … --rewrite … --update-all`.
  It is already installed (`~/.cargo/bin/ast-grep`) and configured (`sgconfig.yml`, used by
  `lint-ast`), and it reports its matches.
- When a script edit is unavoidable, it must assert the expected match count before writing.
- Put this in one line of AGENTS.md's tooling guidance, plus a short example in
  `.claude/rules/tooling.md`.

<a id="O13"></a>
### O13 — Host-tool generations: pruning and staleness

**Observed:** `target/host-tools/` holds twelve `generation.*` directories, and none are pruned.
`scripts/bootstrap-host` ran 62 times in the window while the host itself was being developed.
`scripts/pse-host` runs `current` even when sources have changed since it was built.

**Proposed:**
- `bootstrap-host` keeps the current generation and the previous one, and prunes the rest under
  its existing `flock`.
- Each generation records a source fingerprint of the host crate's inputs.
- `pse doctor` and `pse status` report a stale host as a typed diagnostic whose `next` is
  `scripts/bootstrap-host`.
- Stale tools are never rebuilt implicitly, consistent with ADR-0233.

<a id="O14"></a>
### O14 — Citations and document scaffolds

**Value.** This comes from first principles and from this session. Reviewing and planning here
means resolving stable identifiers constantly:
- ADR numbers and statuses;
- blueprint `§N.M` sections;
- plan packets such as `58b B03`;
- review opportunities such as `…#O07`;
- principle IDs (`DP-21`, `AP-07`).

It also means learning placement conventions by reading README files: file name pattern, front
matter, standard version, index entry and evidence labels. The repository's documentation
principle is that systems pre-decide placement and bookkeeping for capable agents.

**Proposed:**
- **`pse ref <ID>`** resolves an identifier to its file, heading, status and lead paragraph.
  - `pse ref ADR-0233` gives its title, status, decision and supersession.
  - `pse ref §24.1` gives the section file and `#section-24-1` anchor.
  - `pse ref 58b:B03` and `pse ref functional_reviews/…#O07` work the same way.
  - Owners: the existing `xtask adr index` parser, `docs/lifecycle.toml`, and the publisher's
    stable `#section-N-M` anchors (`docs/site.toml`). A prebuilt xtask can read these without
    Cargo, as `docs/dev/documentation.md` notes.
- **`pse new review|plan|probe <slug>`** creates the file with:
  - the pre-decided name (`<subject>-<YYYY-MM-DD>.md`);
  - front matter;
  - the current standard version from `standard.toml`;
  - HEAD as the source baseline;
  - the evidence-label vocabulary;
  - the README index stub.

  ADRs already have `just adr-new`; `pse new adr` routes to it.

### What this reviewer found harder than necessary

The opportunities above already include the items below. This list records them as first-hand
experience from this session rather than log measurement.

| What was hard | What helps |
| --- | --- |
| Learning the command surface took two mapping agents | [O06](#O06) catalog and explain, [O14](#O14) `ref` |
| Reconstructing what Codex ran meant reverse-engineering rollout JSONL, including fork-copied records | [O03](#O03) ledger and `pse history` |
| Every subagent brief restated baseline, pins and conventions | [O07](#O07) `pse brief`, [O08](#O08) `pse dep` |
| Review placement and front-matter conventions were learned by reading three files | [O14](#O14) `pse new` |
| Bash working directories reset, so commands carried `cd` prefixes | [O01](#O01) `-C` and self-location |
| The clap overview was silently a duplicate of `futures.md` | [O11](#O11) |
| Before staging, I had to check whether the shared tree's diff was mine | [O07](#O07) `pse changes` shows generated, protected and owned scopes |

<a id="not-recommended"></a>
## Not recommended

| Proposal | Why not |
| --- | --- |
| A bespoke structured edit tool | That would be a second editing authority. [O12](#O12) covers the measured risk with existing tools. |
| Progress bars for agents | indicatif already hides itself off a TTY, and `ProgressBar::println` prints nothing when hidden. Agents need terminal states and bounded heartbeats, which [O02](#O02) NDJSON `progress` events provide. Use `tracing-indicatif` 0.3.14 (compatible with the pins) only for the human on a TTY. |
| figment for configuration provenance | `explain` must not run shell code, and the layers already exist in `environment.rs`. figment's useful work is unreleased (last release 2024-05), so adopting it would need a git pin. Revisit when typed, nested configuration arrives. |
| Recreating the retired catalogue, MCP tools or generated CLI | They are an idea source only. Their discovery value reappears as derived [O06](#O06). |
| An RMCP adapter now | No consumer needs it beyond the shell. [O02](#O02) schemas make it cheap later. |
| Dynamic shell completion as a priority | `clap_complete`'s `CompleteEnv` helps the human, not agents. Add it late. |

## Order of work

The operator intends to plan and execute all of these. The order follows dependencies:

1. **No new CLI needed:** [O11](#O11), [O09](#O09), [O12](#O12) guidance, and [O10](#O10)
   generated role instructions with a slimmed plan index.
2. **[O01](#O01) and [O02](#O02):** the `pse` binary in `pse-host` with the clap tree, the
   envelope and exit classes, `status`, `doctor`, `env --explain`, `schema`, self-location and the
   host ceiling. Also [O13](#O13).
3. **[O03](#O03):**
   - the corrected property set;
   - admission for detached runs;
   - `wait`, `result`, `log`, `cancel` and `history`;
   - composite runs;
   - libtest and cargo-message extraction.

   Take the ADR-0233 stop-authority decision here.
4. **[O04](#O04):** host-resident `affected`, the leaf capability declaration and the reconciled
   verdict.
5. **[O05](#O05):** `pse fixture` and the receipt pointer, then the bindeps step.
6. **[O06](#O06), [O08](#O08), [O14](#O14) and [O07](#O07):** catalog and explain, `dep`,
   `outline`/`show`, `ref`/`new`, `changes`/`brief`.
7. **The `pse-host` → `pse` rename of the hook path**, if still wanted, landing atomically with
   `just setup-test`.

**Interactions:**
- O02 is the contract every verb emits through.
- O03 carries O04's long runs and records O10's delegated workers.
- O05 completes O04 for integration and database targets.
- O06 is mostly derived once the tree exists.
- O14 and O07 share citation and rule resolution.

## Rule impacts and decisions needed

| Current rule | Conflict | Decision route |
| --- | --- | --- |
| Plan58 §Purpose: "CLIs are optional conveniences… No broad command framework, universal operation catalogue or agent execution engine is selected." The Plan58 Outcome repeats this ("No second scheduler, universal command catalogue, mandatory CLI/MCP…"). | O01–O08 make a CLI the primary agent interface, with one framework and owned runs. | Operator decision, then a Plan58 coordinator amendment. The catalog is derived (O06); supervision is per-run, not a scheduler (O03). |
| ADR-0233 (proposed), status history: "no per-test supervisor or second CLI grammar is selected" | O01 is one CLI grammar; O03 adds `__supervise`/`__finalize` per detached run. | Amend ADR-0233 before acceptance, or a short superseding ADR. |
| ADR-0233 ownership: stop authority is the exact bus client in `Unit.Refs`; `RefUnit` never issued | `pse cancel` from a later process needs authority transferred through the ledger. | The same ADR amendment, with an explicit stop-authority rule for detached runs. |
| Plan58, 58b and 58h: bespoke build observation and its wait/freshness consumers retire; 58b deviations retire telemetry query/watch wrappers; the environment guide's §Detached watches retires invocation-watch cursors | O03 reintroduces wait and result consumers for owned runs. | Operator decision. 58h's acceptance wording already supports this framing: "native compiler/Cargo/nextest diagnostics replace bespoke build observation". Amend 58b and 58h to separate terminal-outcome supervision of owned runs from build-stage telemetry. |
| Plan58 I09: universal catalogue declined | O06 publishes a catalog. | Confirm that a parser-derived catalog satisfies I09. |
| Blueprint §21.7 and §24.1 (ADR-0233 block): no mandatory or full CLI follows | `pse` becomes the documented agent route. Products remain callable without it. | ADR amending §21.7/§24.1 wording with a `blueprint.md` revision row. |
| Blueprint §24.1 P00 / Plan53: managed commands exit 0 only on a complete reconciled pass | Not a conflict. O04 is its Rust owner after the Python reconciliation was retired. | Record O04 as realizing the accepted target. |
| CLAUDE.md: native agent definitions "load shared contracts rather than copying role behavior" | O10 generates role contracts into Codex `developer_instructions`. | Amend the wording to allow generated, checked embedding with a single authority. |
| AGENTS.md: "Recipes are shortcuts, not gates" | Compatible. `pse` is a shortcut too, and bare tools stay first-class. | Update AGENTS.md routing and the docs/dev guides on delivery. |

New dependencies (tracing-subscriber `json`, clap `env`/`unstable-ext`, schemars `raw_value`,
optionally `tracing-indicatif`) need no ADR under the dependency policy. Using the existing
`pse-host` crate avoids the ADR and design review a new crate would need.

## What would demonstrate delivery

The agents are the users and the judges. A capability is delivered when Codex and Claude use it
in ordinary work instead of the hand-built forms in the episodes above, and it does what it
claims at first use:
- **Native fixture test:** runs as one `pse test` with no inline `PSE_*` variables and no `/tmp`
  paths.
- **Detached hygiene run:** waited on once; the failing child and its first error read through
  `pse result --failures` within budget.
- **Long Claude run:** waited on through `pse wait --format ndjson` in a background task or
  Monitor.
- **Subagent brief:** built from `pse brief`.
- **Library facts:** answered by `pse dep`.
- **Citations:** resolved by `pse ref`.
- **New review:** created by `pse new`.

Ordinary implementation tests cover the contracts:
- each verb's envelope validates against its schema;
- `Cli::command().debug_assert()` passes;
- an interrupted supervisor reports lost observation;
- a launch never exits as a pass.

No before/after measurement campaign is part of acceptance.

## Sources

- Logs: `~/.codex/sessions/2026/10/{28,29}/*.jsonl` (Codex rollouts) and
  `~/.claude/projects/-home-paul-pse-arrow/*.jsonl`. The read-only miner is
  [`probes/rust-agent-cli-log-miner-2026-10-29.py`](probes/rust-agent-cli-log-miner-2026-10-29.py).
- Owners: `crates/pse-host/src/{main,boundary,environment,correctness,doctor,agent_hooks}.rs`,
  `crates/pse-host/src/study_module.rs`, `xtask/src/main.rs`, `xtask/src/development/affected.rs`,
  `crates/pse-operations/src/bin/pse-canonical-test.rs`,
  `crates/pse-runtime/tests/{native_study,worker,native_worker_lifecycle}.rs`,
  `crates/pse-operations/tests/native_service.rs`, `justfile`, `.cargo/config.toml`,
  `.config/nextest.toml`, `~/.config/nextest/config.toml`.
- Further owners consulted for the integrated corrections:
  - `crates/pse-host/src/{manager,admission,environment,correctness}.rs`, covering
    `start_service_inner`, the admission gate, `local_pending` and `reject_reuse`;
  - `crates/pse-backend-native/build.rs`, `crates/pse-runtime/Cargo.toml` and
    `xtask/Cargo.toml`;
  - `.codex/agents/*.toml`, `.codex/config.toml`, `~/.config/codex/bash-env.bash` and
    `~/.config/direnv/direnv.toml`;
  - `docs/dev/agent-environment.md`, `docs/adr/0233-rust-owned-host-and-execution-boundaries.md`
    and [Plan53](../plans/53-test-architecture-target.md).
- Plans and decisions: [Plan58](../plans/58-rust-owned-system.md),
  [58b](../plans/58b-bootstrap-and-host-control.md),
  [58f](../plans/58f-development-and-assurance.md),
  [58g](../plans/58g-boundary-retirement-and-optional-consumers.md), ADR-0233, blueprint §3.2,
  §21.7, §23.2, §24.1.
- Earlier reviews:
  - [codex-effectiveness](codex-effectiveness-and-diagnostics-2026-10-22.md)
  - [agent-effectiveness](agent-effectiveness-opportunities-2026-10-23.md)
  - [agent-tool-discovery](agent-tool-discovery-and-access-2026-10-24.md)
  - [mcp-development-effectiveness](mcp-development-effectiveness-2026-10-25.md)
  - [rust-first system](rust-first-system-functional-opportunities-2026-10-28.md)
- Primary library sources:
  - clap 4.6.7: docs.rs and `clap_builder` source.
  - argparse-rosetta-rs: <https://github.com/rosetta-rs/argparse-rosetta-rs>.
  - Cargo JSON output: <https://doc.rust-lang.org/cargo/reference/external-tools.html>.
  - Cargo artifact dependencies: <https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies>.
  - nextest record and replay: <https://nexte.st/docs/features/record-replay/>.
  - nextest machine-readable output: <https://nexte.st/docs/machine-readable/libtest-json/>.
  - nextest exit codes: <https://docs.rs/nextest-metadata/latest/nextest_metadata/enum.NextestExitCode.html>.
  - rustc JSON diagnostics: <https://doc.rust-lang.org/rustc/json.html>.
  - systemd transient units, `CollectMode` and exec environment:
    <https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html>,
    <https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html> and
    `man systemd.exec`.
  - Bazel BEP: <https://bazel.build/remote/bep>.
  - buck2 log commands: <https://buck2.build/docs/users/commands/log>.
  - `gh run view`: <https://cli.github.com/manual/gh_run_view>.
  - figment 0.10.19: docs.rs and the upstream repository.
  - miette 7.6.0: `JSONReportHandler`.
  - schemars 1.2: `SchemaSettings`.
  - direnv quieting (`DIRENV_LOG_FORMAT`, `hide_env_diff` from 2.34.0):
    <https://github.com/direnv/direnv/wiki/Quiet-or-Silence-direnv>.
