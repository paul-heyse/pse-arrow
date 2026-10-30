---
title: An agent-first Rust CLI for pse-arrow
status: proposed
date: 2026-10-29
doc_role: review
doc_topics: [environment, libraries, design, current-work]
---

# An agent-first Rust CLI for pse-arrow

> **Revision (2026-10-29, follow-up).** The first pass ran without a usable clap overview: the
> file then at `docs/libraries/clap.md` duplicated `futures.md`. That gap should have blocked the
> design and was noted only as an evidence limit. With the corrected
> [clap overview](../libraries/clap.md), this follow-up adds three things:
> - an [integrated clap capabilities](#clap-capabilities) contract covering every capability
>   family;
> - [first-principles opportunities](#first-principles) O15–O21, judged by the agent as the tool's
>   user;
> - [environment changes](#environment-followup) O23–O25, plus the operator's new direction that
>   the CLI is the strongly recommended agent interface. It is adopted by being the easiest path
>   and clearly communicated, with no enforcement ([adoption](#adoption-path) and O22).
>
> The follow-up amends O01, O04, O06, O07, O11 and O14 in place, keeps every earlier ID, and
> updates the order of work, proposed direction changes and delivery statements. Its companion-crate checks
> include a scratch probe of rustdoc JSON on the pinned nightly. A fresh design-reviewer assessed
> the delta; its corrections are integrated.
>
> **Enablement pass (same day).** This review serves agent enablement, not compliance. Where a
> current rule or direction costs agents effort without a clear return, the review now proposes
> changing the rule. It no longer designs around it. See
> [Direction changes proposed](#direction-changes): worktrees are allowed without re-review,
> protected documents inform instead of block, any agent may commit its own paths, and a stale
> host rebuilds itself.

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

**Follow-up additions.**
- **clap usage contract.** The [integrated clap capabilities](#clap-capabilities) section is the
  full contract. Its structural choices are one library `Cli` behind thin bins, a federated xtask subtree,
  lazy subtrees, shared argument structs, typed domain value parsers and parser-enforced argument
  groups.
- **First-principles opportunities:**
  - exact pinned-API lookup and scratch probes ([O15](#O15));
  - owner and generator routing ([O16](#O16));
  - fast documentation checks ([O17](#O17));
  - shared-tree commits ([O18](#O18));
  - test discovery ([O19](#O19));
  - continuity across compaction ([O20](#O20));
  - soft claims, as a candidate ([O21](#O21)).
- **Adoption.** `pse` is made the easiest path and clearly communicated, with no enforcement
  hooks ([O22](#O22)).
- **Environment.** A worktree lifecycle ([O23](#O23)), one checkout-scoped home for results, logs
  and scratch ([O24](#O24)), and harness orientation surfaces ([O25](#O25)).

**Operator directions that shaped this review:**
- The Python capability CLI, Python MCP server, operation catalogue and `pse-py` boundary were
  retired today (`f321e9e0f`, `600d4df30`). That retirement is why the operator asked for a
  Rust CLI.
- The retired interfaces are an idea source, not a baseline or a replacement list.
- The target is the best-in-class interface for Codex and Claude. Where current rules conflict,
  the review proposes changing the rule rather than treating it as a limit on the target
  ([Direction changes](#direction-changes)).

**What was inspected:**
- Source baseline `6dc84501d`, inspected 2026-10-29 with an uncommitted working tree.
- The standard selected by `standard.toml`: core 3.4, efficient-architecture heuristics 1.0, test
  assessment 1.2 and process-simulator 1.5, under the pse-arrow binding.
- The supplied overviews, read end to end:
  [serde/serde_json](../libraries/serde_serde-json.md), [schemars](../libraries/schemars.md),
  [tracing](../libraries/rust-tracing.md), [thiserror+miette](../libraries/thiserror_and_miette.md),
  [tokio](../libraries/tokio.md), [indicatif](../libraries/indicateif.md) and
  [figment](../libraries/figment.md).
- [`clap.md`](../libraries/clap.md) was read end to end in the follow-up. At the first pass it
  duplicated `futures.md`, so clap was then researched from Context7, docs.rs and the pinned
  `clap_builder-4.6.7` source.
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

**clap realization (follow-up).** The [integrated clap capabilities](#clap-capabilities) contract
is the full usage specification. Its structural decisions for O01:
- **One library `Cli`, thin bins.** `pse`, the existing `pse-host` and `cargo-pse-correctness`
  stay thin `[[bin]]`s of the one `pse-host` crate, sharing one library `Cli` and error contract.
  They are already built and published as one generation. Multicall was considered and rejected;
  see its row in the clap table.
- **Federated xtask verbs.** xtask, already the generator, exports its own clap schema through
  `just codegen` as a generated JSON file. Each verb records its Cargo features and invocation
  route.
  - **Read at runtime, not compiled in.** `pse` reads that file using the `string` feature and
    reports staleness. If it were compiled in, every xtask flag change would force a host
    re-bootstrap.
  - **Help is free; dispatch builds.** Help, catalog and explain never build anything, per
    ADR-0233. Dispatching a federated verb is an effectful run that builds xtask through the
    correctness adapter.
  - **Planning decision.** The catalog walker lives where both closures already reach without a
    new crate.
- **Lazy subtrees.** `#[command(defer = true)]` keeps them out of cold start.
- **Shared argument structs.** `GlobalArgs`, `SelectionArgs`, `PlacementArgs` and `OutputArgs`
  are flattened everywhere, with `global` and `from_global` where needed.

**Delivery demonstration:** an agent completes the native fixture episode above using only `pse`
verbs, with no inline `PSE_*` assignments, no `/tmp` paths and no `cd` prefix.

<a id="clap-capabilities"></a>
## Integrated clap capabilities

**Basis.** This pass reads the corrected [clap overview](../libraries/clap.md) end to end (§1–§14)
and verifies the consequential APIs in the pinned `clap_builder-4.6.7` and `clap_derive-4.6.7`
sources:
- `Command::defer(fn(Self) -> Self)` and the derive `defer` attribute;
- `Command::multicall`;
- `external_subcommand_value_parser`;
- `get_overridden_usage`;
- the `string` feature;
- `ValueSource::{DefaultValue, EnvVariable, CommandLine}`;
- `unstable-ext` `Command::add`/`Arg::add` with `get::<T>()`.

**How to read it.** This is the design's clap usage contract. Each family of the overview has a
decision:
- *Now:* part of the first `pse` delivery.
- *Later:* adopt once the named need appears.
- *No:* not adopted, for the reason given.

The O-ids show where a capability lands.

### Command model and composition (overview §1–§3)

| Capability | Decision | Use in `pse` |
| --- | --- | --- |
| Derive `Parser`/`Args`/`Subcommand`/`ValueEnum` | Now ([O01](#O01)) | Every host verb is a derived type. Doc comments supply `about`/`long_about`. |
| Reusable `Args` with `#[command(flatten)]` | Now | Four shared argument structs: `GlobalArgs` (`--format`, `--budget`, `-C`, `-v/-q`), `SelectionArgs` (targets, `--changed`, `--workspace`, `-E`), `PlacementArgs` (`--class`, `--native`, `--detach`, `--wait`) and `OutputArgs` (`--fields`). One definition each, identical across verbs. |
| Explicit `ArgGroup`s and `conflicts_with_all` | Now ([O04](#O04)) | The implicit group a derived `Args` struct creates is `multiple(true)` over all its own fields, so it cannot make its members conflict. Selection modes instead use a dedicated sub-struct, or `conflicts_with_all` on `--changed`, against explicit targets (positional `pkg:target` and `-p`, which combine) and `--workspace`. `--workspace --exclude` and `-p … -E` stay legal. `try_parse_from` cases pin these combinations, alongside `debug_assert`. |
| `#[arg(from_global)]` | Now | Nested handlers read the global `--format`/`--budget` without threading them through. |
| `rename_all_env` | Now, with the `env` feature | `PSE_*` names follow one rule. |
| `defer = true` (new in 4.6.7) | Now ([O01](#O01)) | Large subtrees (`fixture`, `docs`, federated xtask verbs) are built lazily, which protects cold start. Catalog and help walks call `build()` first, as the overview's §3.3 caveat requires. |
| Builder augmentation (`augment_subcommands`, `mut_subcommand`, `find_subcommand`) plus the `string` feature | Now ([O01](#O01)) | **Federated tree.** xtask-owned verbs appear in `pse` help and the catalog. Their schema comes from xtask's own clap definitions, exported by `just codegen` into a generated JSON file with drift check, so there is one authority. `pse` reads the file at runtime, reports when it is stale, and builds those subcommands with owned strings. Dispatch is an effectful verb that builds xtask through the correctness adapter. Help, catalog and explain never build. No xtask closure is linked into the host. |
| `Command::debug_assert` | Now | Unit test over the whole tree, including the federated part. |
| `Option<T>` with `require_equals` and `default_missing_value` | Now | `--native` (absent = no providers, bare = `auto`, `=CAPS` = explicit) and `--wait` (bare = `50s`). Derived `Option<Option<T>>` combined with a default is redundant: a bare flag would yield `Some(Some(default))`. |
| `Option<Vec<T>>` | Now | `--fields`: absent means the default projection, present means exactly these fields. |
| `Vec<Vec<T>>` (`unstable-v5`) | No | Occurrence groups are not needed. |
| `#[command(version)]` from Cargo metadata (`cargo` feature) | No | `version`/`long_version` come from `pse-buildinfo`: build identity, host-tools generation and source fingerprint ([O13](#O13)). |

### Parsing semantics, constraints and values (overview §4–§7)

| Capability | Decision | Use in `pse` |
| --- | --- | --- |
| `ArgAction::Count` | Now | `-v`/`-q` adjusts the tracing filter. A ten-line flatten struct; `clap-verbosity-flag` is optional. |
| `args_override_self(true)` and `ArgAction::Append` | Now | For single-valued `Set`/`SetTrue`/`SetFalse` flags, the last occurrence wins instead of producing an error. Set-valued flags (`--native`, `--fields`, `-p`) use `Append`, so repeats combine rather than silently replace. |
| `num_args` and `value_delimiter(',')` | Now | Capability sets, package lists and field lists. |
| `require_equals(true)` on optional-value flags | Now | `--native=CAPS` and `--wait=DUR` never swallow the next positional. Without it, `--native compiler` parses ambiguously. Paired with `default_missing_value` above. |
| `last(true)` pass-through | Now | `pse run -- <argv…>` and `pse test … -- <nextest args>`, keeping `OsString` values. The other §4.3 mechanisms (`trailing_var_arg`, `allow_hyphen_values`, `subcommand_precedence_over_arg`, `value_terminator`) are not used. |
| `ArgGroup` (required, exclusive and multiple), `conflicts_with`, `requires`, `requires_if`, `required_unless_present`, `exclusive` | Now ([O04](#O04)) | The parser enforces combinations before effects. Examples: exactly one selection mode; `--fields` requires a JSON format; `pse wait` needs a run ID unless `--latest`; `pse commands --json` is exclusive. Cross-checks against external state stay in the handler (overview §5.3). |
| `ValueParserFactory` and `TypedValueParser::try_map` | Now | Domain types `PackageSpec`, `TargetSpec` (`pkg:target`), `RunId` (`latest`, prefix, full), `Duration`, `FilterExpr` (non-empty) and `CapabilitySet`. They parse syntax into lightweight IDs. Existence checks against Cargo metadata or the ledger happen in the handler but are reported as clap-style `ValueValidation` errors with did-you-mean suggestions, through `Command::error` and `Error::with_cmd`. |
| `ValueEnum` with `#[value(help, alias, hide)]` | Now | Format, native capability, resource class (owned by `admission.rs`) and run kind. Per-value help flows into help and the catalog. Hidden aliases keep retired spellings parsable. |
| `PossibleValuesParser` built at runtime (`string` feature) | Later | For value sets owned by configuration rather than code, such as fixture names declared in package metadata. |
| `RangedU64ValueParser`, `NonEmptyStringValueParser` | Now | `--budget`, `--tail` and filters. |
| `OsString`/`PathBuf` values, `get_raw` | Now | Paths and pass-through stay OS-native. |
| `env` feature and `Arg::env` | Now | `PSE_FORMAT`, `PSE_BUDGET` and `PSE_CHECKOUT`. Each key has one owner: clap does not declare keys that `environment.rs` owns ([O06](#O06)). |
| `ArgMatches::value_source` | Now ([O06](#O06)) | `explain` and envelopes mark each input as explicit, environment or default. Handlers keep `ArgMatches` alongside the derived struct (`FromArgMatches`) where provenance matters, because `contains_id` does not mean the user supplied a value (overview §7.2). |
| `default_value_if`/`ifs` | Later | Only where one argument's value fixes another's default. |
| Config-file layering in clap | No | clap stays the CLI layer; configuration remains `environment.rs`'s responsibility (overview §7.3). |

### Dispatch, help, errors and generated artifacts (overview §8–§11)

| Capability | Decision | Use in `pse` |
| --- | --- | --- |
| `subcommand_required` and `arg_required_else_help` | Now | Bare `pse` prints compact help. Agents call `pse status` explicitly. |
| `global(true)` | Now | The `GlobalArgs` work in any position. |
| `propagate_version` | Now | Every verb answers `--version` with the same identity. |
| `infer_subcommands`/`infer_long_args` | No | Prefixes become ambiguous when verbs are added, which breaks generated agent commands (overview §8.1). |
| Hidden `alias` | Now, sparingly | Only for the same semantics under a retired name. No `visible_alias`: agents should see one canonical name. `short_flag`/`long_flag` subcommands are not used. |
| `multicall(true)` | No | clap forbids arguments on a multicall root, and dispatch depends on `argv[0]`'s file stem. Hooks exec `pse-host`. Cargo calls `cargo-pse-correctness pse-correctness …`. `bootstrap-host` probes each tool with `--version`. A copied binary would stop dispatching. Thin `[[bin]]`s over one library `Cli` give the same single build, publish and error contract without depending on `argv[0]` (DP-16). |
| External subcommands with `external_subcommand_value_parser` | Later | `pse-<name>` executables on the checkout tool path for maintainer extensions, and git-style alias expansion (the cookbook's reparse pattern) if declared composite commands are wanted. The federated tree covers xtask without this. |
| Help structure: `next_help_heading`, `display_order`, `after_long_help` examples, `verbatim_doc_comment`, `override_usage` for pass-through forms, `hide` | Now ([O06](#O06)) | `-h` stays one line per argument under Selection, Placement and Output headings. `--help` adds worked examples taken from the episodes in this review. `__supervise`, `__finalize` and `agent-hook` are hidden. |
| `render_help`/`render_long_help`/`render_usage` | Now ([O06](#O06)) | Rendered text is embedded in the JSON catalog, so agents get exactly what humans see. |
| `wrap_help`, the `unicode` feature, custom `Styles` | No | The non-TTY fixed width is deterministic, which suits agents. Default styles are used for humans. |
| `suggestions` and `error-context` (defaults) | Now ([O02](#O02)) | `Error::kind`/`context` become the `ErrorRecord`. `KindFormatter` gives the one-line human form. `DisplayHelp`/`DisplayVersion` exit 0. |
| clap_complete (AOT scripts and the dynamic `CompleteEnv` engine) | No | Shell completion serves interactive humans, not agents. The agent need, an argument's valid values, comes from the CLI's own value-domain providers ([O06](#O06)). These are small functions over cached Cargo metadata, the ledger and configuration, exposed in the JSON catalog. Static sets come from `PossibleValue`/`ValueEnum` metadata. |
| clap_mangen | No | Man pages do not serve agents. A Markdown CLI reference generated into `docs/generated/` by `just codegen`, with `codegen-check` drift, serves both agents and the docs site. |
| `try_parse_from`, `Parser::try_parse` | Now | Parser tests, plus the pre-parse format decision ([O02](#O02)). |
| Help and output snapshots | Now | `insta` (already pinned) for `-h`/`--help`/envelope snapshots. |
| Executable example cases (`trycmd`) | Later | Once verbs stabilize, every `after_long_help` example also runs as a case file. |
| `update_from`, REPL with `shlex`, batch mode | No | Agents already chain shell commands. A long-lived interactive parser adds state without a consumer. |
| `ArgMatches` occurrences and indices | No | Not needed. |

### Features and ecosystem (overview §12–§13)

**Workspace clap features:**
- `derive`, `env` and `string`.
- `unstable-ext`, for catalog metadata clap has no getter for: `requires`, examples and
  output-schema IDs. It is bounded by the exact pin (`=4.6.7`); upgrades are deliberate pin moves.
- Not `deprecated`. Enabling it only for linting would give clap a second feature set, which
  ADR-0122's one-feature-set invariant forbids, and enabling it always adds noise. Pin moves read
  clap's changelog instead.
- Feature unification means xtask's clap also receives `env`, `string` and `unstable-ext`, which
  is harmless.
- Not enabled: `wrap_help`, `unicode`, `cargo`, `debug`, `unstable-v5`.

**Ecosystem crates:**
- `clap-cargo` and `argfile` are assessed in the companion note.
- `wild` (Windows globbing), `clio` (stdin/stdout path arguments) and `clap_complete_nushell` are
  not needed.

### Companion crates (checked against the pins)

| Crate | Decision | Notes |
| --- | --- | --- |
| `clap_complete` 4.6.11 | No | Shell completion only. Its candidate engine could be called outside a shell, but it needs `unstable-dynamic`, which adds `clap/unstable-ext` workspace-wide. Agent-facing valid values are simpler as the CLI's own providers ([O06](#O06)). |
| `clap-cargo` 0.19.0 | Now ([O04](#O04)) | `Workspace { package, workspace, all, exclude }` with `partition_packages(&Metadata)`, which honours `workspace_default_members`. Its metadata must not come from `no_deps`. `Features { all_features, no_default_features, features }` with `forward_metadata`. Built on `cargo_metadata ^0.23` (feature `cargo_metadata`) and clap ^4.5.48, both compatible. It gives `pse test`/`check` Cargo's familiar package-selection grammar without bespoke parsing. |
| `insta` 1.49.0 (pinned) | Now | In-process `cmd.render_long_help()` and envelope snapshots. No subprocess needed. |
| `trycmd` 1.2.1 / `snapbox` 1.2.2 | Later | Real-binary cases from `.toml` or Markdown ` ```console ` blocks, with `[ROOT]`/`[CWD]` normalization and `TRYCMD=overwrite`. Adopting them moves the lockfile (snapbox 1.0.0 is installed). Add them once help examples stabilize, so documented examples are executed. |
| `clap-markdown` 0.1.5 / `clap_mangen` 0.3.3 | No | clap-markdown works with clap 4.6 but last released in 2025-05. clap_mangen produces roff only. The catalog walker [O06](#O06) already needs emits both the JSON catalog and the generated Markdown reference from one walk. |
| `argfile` 1.0.0 | No | `@file` expansion covers only `pse`'s own argv. Long nextest filters are better generated by `pse` from intent ([O04](#O04)) than passed in files. |
| `rustdoc-types` 0.61.0 | Now ([O15](#O15)) | `FORMAT_VERSION = 61`. The pinned nightly-2026-09-29 emits `format_version: 61`, *Measured* by a scratch probe that generated JSON for a dependency at its locked version (`cargo rustdoc -p argfile@1.0.0 --lib -Z unstable-options --output-format json`). A toolchain move must bump rustdoc-types with it; readers check `format_version` first. |

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
- **Cancel.** `pse cancel <run>` stops a detached run that the user's ledger records. ADR-0233
  currently gives stop authority only to the exact bus client in `Unit.Refs`, so a later `pse`
  could not cancel. The review proposes that the ledger's record is the authority for detached
  runs ([Direction changes](#direction-changes)).
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
- **Package selection (follow-up).** `clap-cargo` 0.19.0 `Workspace` (`-p`, `--workspace`,
  `--exclude`; `partition_packages` on full metadata) and `Features` give Cargo's own grammar.
  An explicit selection group, or `conflicts_with_all` on `--changed`, makes `--changed`
  exclusive with explicit targets (positional `pkg:target` plus `-p`) and `--workspace`. It keeps
  `--workspace --exclude` and `-p … -E` legal. The implicit group of a derived `Args` struct
  cannot do this, because it is `multiple(true)`.
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
  - **Valid values (follow-up).** The catalog's `valid_values` come from the CLI's own
    value-domain providers: packages and `pkg:target` targets from cached metadata, run IDs from
    the ledger, fixtures and capability sets from their declarations. Static sets come from
    `PossibleValue`/`ValueEnum`. Providers are fast and side-effect-free. `pse complete` is not
    needed: an agent reads the leaf's `valid_values`.
  - **Generated reference (follow-up).** The same walk emits a Markdown CLI reference into
    `docs/generated/` through `just codegen`, with `codegen-check` drift, replacing clap_mangen
    or clap-markdown.
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

**Follow-up:** [O20](#O20) adds `pse handoff`, the same packet for an agent's own continuity across
compaction, wired to both harnesses' compaction and session hooks.

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
   - CLAUDE.md's "load shared contracts rather than copying role behavior" would block this.
     Change that wording, since a generated, checked embedding costs agents nothing and saves
     every worker a startup read ([Direction changes](#direction-changes)).
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
- ~~Replace [`docs/libraries/clap.md`](../libraries/clap.md) with an actual clap overview.~~ The
  operator supplied the corrected overview on 2026-10-29; this follow-up uses it.

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
- Tip for unavoidable script edits: assert the expected match count before writing, so a missed
  needle fails loudly instead of silently changing nothing.
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
- Each generation records a source fingerprint covering the whole host path-dependency closure,
  plus `Cargo.lock`, `rust-toolchain.toml` and `.cargo/config.toml`.
- **A stale host rebuilds itself.** Effectful `pse` verbs rebuild a stale host before running:
  an incremental `scripts/bootstrap-host`, then a re-exec of the new generation. Read-only verbs
  (help, status, explain) only report it. Today an agent working on the host reruns
  `bootstrap-host` by hand (62 times in one day) and sometimes tests a stale binary. ADR-0233's
  "launch never builds" would block this, so the review proposes narrowing that rule to read-only
  verbs ([Direction changes](#direction-changes)).

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

  Plans and ADRs already have scaffolds (`just plan <slug>` and `just adr-new`). `pse new plan`
  and `pse new adr` route to those owners rather than duplicating their templates.

<a id="first-principles"></a>
## First-principles opportunities (reviewer as user)

**Proposed (first-principles; reviewer as user).** The operator asked for expansive ideas about
what would help an agent working here: Claude directly, and Codex by the same reasoning. The
agent is the user and judges whether each works on first use, so none of these depends on
measurement. They come from the work of this review and its follow-up: mapping the surface,
briefing helpers, checking library APIs, editing protected documents, validating links, and
committing in a shared tree.

<a id="O15"></a>
### O15 — Exact pinned-API lookup and scratch probes

**The problem.**
- **Version drift.** Agents constantly need the exact signature and docs of a dependency item at
  the locked version, with the workspace's unified features. Training memory and docs.rs both
  drift from the pins.
- **Today's workarounds.**
  - `rg` through `~/.cargo/registry/src` (169 times a day).
  - Research agents rebuilding toy crates by hand to check one behaviour; two did so in this
    review alone.

**Proposed.**
- **`pse api <crate> <path>`** prints the item's signature, docs, feature gates and source
  location.
  - The data is rustdoc JSON generated once per `(crate, version, features)` for the locked
    version, with
    `cargo rustdoc -p <crate>@<locked version> --lib -Z unstable-options --output-format json`.
  - The JSON is cached in the checkout store ([O24](#O24)), not in the shared `target/doc`.
  - Lookup goes through the `paths` table.
  - A re-export into another crate (for example `arrow` → `arrow-array`) also generates that
    crate's JSON.
  - `pse api <crate> --search <word>` lists matching items.
- **`pse probe new <name> --deps <crate…>`** creates a scratch crate for tiny compile checks.
  - It gets a copied `rust-toolchain.toml`, so rustup selects the pinned nightly (prime
    directive 6).
  - It gets a `Cargo.lock` seeded from the workspace lock, so nothing is re-resolved, and the
    workspace's feature selections for the named crates.
  - `pse probe run` compiles and runs it through the ordinary placement path.
  - Probes are throwaway: `pse probe rm`, or pruned with session scratch.

**Mechanisms** (*Interface-checked*; the format match is *Measured*):
- **rustdoc format.** `rustdoc-types` 0.61.0 (`FORMAT_VERSION = 61`) matches nightly-2026-09-29,
  which a scratch probe confirmed by generating JSON for a locked dependency.
  - Readers check `format_version` first.
  - Bump `rustdoc-types` with the toolchain date. `pse api` reports a format mismatch as one
    clear diagnostic naming the fix.
- **Try first:** `rustdoc-mcp` 0.10.0 (rustdoc-types ^0.61), an existing MCP server for item
  lookup. The trial must confirm that it builds through the checkout's pinned toolchain, not
  `+nightly`. If it works for both harnesses, `pse api` shrinks to producing the locked-version
  JSON it reads.
- `public-api` 0.52.2 is format-incompatible (^0.59).

**Delivery:** an agent answers "what is the exact signature of
`tokio::process::Command::kill_on_drop` here?" with one command, and checks a two-line behaviour
with a probe instead of a hand-built crate.

<a id="O16"></a>
### O16 — Owner and generator lookup; protected-edit lifecycle

**The problem.** In this repository, the place to change something is often not the file the
agent is looking at:
- generated outputs come from generators and registries;
- architecture sections and accepted ADRs are protected and change through decision routes;
- plans own status.

The edit guard blocks correctly, but its message does not say where the change belongs.

`PSE_DESIGN_EDIT` is read from the *hook process's* environment (`design_edit_authorized` in
`agent_hooks.rs`), so setting it inline on a shell command never reaches the guard. The 23 inline
Codex uses in one day therefore accompanied shell writes to protected documents. Those writes
bypass the edit tools the guard covers, which makes a proper route more important.

**Proposed.**
- **`pse owner <path>`** says what owns a path and how to change it:
  - generated path → its generator and the narrowest `xtask codegen --only …`;
  - architecture section → its section ID, the ADR route and the `blueprint.md` revision-row
    duty;
  - accepted ADR → `just adr-supersede`;
  - code → its crate and the plan that owns current work there.
- **Guard messages.** The edit guard's `BLOCKED` message includes the same answer.
- **Inform, don't block, for documents.** Generated paths stay blocked, because an edit there is
  simply lost at the next regeneration. Architecture sections and accepted ADRs would no longer be
  blocked. The guard would add context instead: "this is the authoritative design; add a
  `blueprint.md` revision row", or "accepted ADR: supersede with `just adr-supersede`".
  - Today's `.design-edit` file and `PSE_DESIGN_EDIT=1` launch flag exist only to get past a
    block.
  - Codex worked around the block with shell writes (23 inline `PSE_DESIGN_EDIT=1` uses in a day,
    which never reached the guard).
  - With a nudge in place of the block, `pse design-edit` is unnecessary. The decision routes stay
    the same; agents simply aren't stopped on the way to following them
    ([Direction changes](#direction-changes)).

**Mechanisms.**
- Today the guard's `protected()` is a set of inline prefix predicates, with no mapping to
  generators.
- Generator lookup needs an export from the codegen registry `xtask codegen` already owns: path
  pattern to generator and narrowest regeneration. It is generated like the federated schema
  ([O01](#O01)), so the guard and `pse owner` read one declaration (DP-01).
- Documentation ownership comes from `docs/lifecycle.toml`.

<a id="O17"></a>
### O17 — Fast documentation checks for changed documents

**The problem.** Validating a review's links and anchors took a hand-written shell loop in this
session. `just docs` builds the whole site. The offline link check (lychee) runs only in the
manually dispatched GitHub docs workflow.

**Proposed.** `pse docs check [--changed | <files…>]` checks:
- relative links and fragments, including heading slugs and explicit `<a id>` anchors;
- front matter required by the document kind;
- review `O`-ID anchors;
- README index entries for new reviews and plans.

**Mechanisms.**
- **Links and fragments.** A locally pinned `lychee --offline --include-fragments` (installed like
  the other diagnostic tools) does this rather than a new parser (DP-16).
- **Front matter.** The existing `xtask/src/documentation/metadata.rs` checks.
- **Section anchors.** The publisher injects `#section-N-M` anchors only at publish time
  (`site.rs`), so they are not in the source Markdown. Links that use them are checked against
  the publisher's section map, exported once, or verified after `just docs` builds.
- Results are reported as [O02](#O02) diagnostics with file and line.

<a id="O18"></a>
### O18 — Commits in a shared working tree

**The problem.**
- **Shared tree.** Several agents work in one checkout. This session started with about forty
  files modified by others, and before committing I had to confirm that the README diff was only
  mine.
- **Rules applied by hand.** AGENTS.md says the coordinator owns the shared index, stages only
  owned paths or hunks, and formats only at `just turn-end`.

**Proposed.** `pse commit --paths <…> [--hunk <id>…] -m <message> [--trailer <line>…]`. It never
pushes.
- **Private index.** It builds the commit in a private index (`GIT_INDEX_FILE`), starting from
  HEAD plus exactly the named paths or hunks. Concurrent agents' staging cannot race it. Afterwards
  it refreshes only those entries in the shared index.
- **Hunk IDs** are content hashes printed by `pse changes`. If a hunk changed since it was listed,
  the commit is refused.
- **Formatting.** It formats the named files when they are committed whole, because the whole file
  is going into the commit anyway. In hunk mode it only reports, since rewriting the file would
  touch lines outside the chosen hunks.
- **Drift check.** It runs `codegen-check` scoped to any generator whose inputs are among the
  paths.
- **Attribution** comes from the harness. Trailers are passed with `--trailer` and applied with
  `git interpret-trailers`; `pse` never invents model-specific text.
- **Output.** The envelope reports the staged diffstat.
- **Who may commit.** Any agent may commit its own paths. The private index removes the race that
  made "the coordinator owns the shared index" necessary ([Direction changes](#direction-changes)).

<a id="O19"></a>
### O19 — Test discovery

**The problem.** "Which test covers this, and how do I run just it?" means grepping `#[test]`
names and then reconstructing the target, features and fixtures.

**Proposed.**
- `pse tests <pattern> [--package P]` lists matching tests, each with:
  - its package and target;
  - its required features and native capabilities;
  - its fixtures;
  - the exact `pse test` command.
- The data comes from `nextest list --message-format json`, run through the correctness adapter
  with the same features `pse test` would use. Listing builds the test binaries, so a following
  `pse test` reuses that build. Results are cached per build identity and joined with the
  [O04](#O04) and [O05](#O05) declarations.

<a id="O20"></a>
### O20 — Context continuity across compaction and delegation

**The problem.** Long sessions compact. After compaction an agent has lost which runs are live,
which failures are open, what it was about to do, and which files it changed. Delegation has the
same problem in the other direction.

**Proposed.**
- **`pse handoff`** (an amendment to [O07](#O07)) writes and prints a compact state record:
  - HEAD, dirty paths and an active `.design-edit` reason;
  - live and recent runs with outcomes, and open failures;
  - the plan pointer;
  - a "next" line the agent supplies (`pse handoff --next "…"`).
- **Hooks that use it:**
  - **PreCompact** runs `pse handoff --record`.
  - **After compaction**, the session-start path adds `pse handoff --show` as
    `additionalContext`: Claude's `SessionStart` with source `compact`, and Codex's
    `PostCompact` or `SessionStart`.
  - **Subagent start** injects `pse brief` for the child's paths. Codex has `SubagentStart`;
    confirm the equivalent for the current Claude Code version before relying on it there.
  - **One handler.** Merge this into the existing `agent-hook session-env` handler rather than
    adding a second hook process.
- **Limits.** Keep the record within Codex's default 2,500-token `additionalContext` cap. Codex
  project hooks need a one-time `/hooks` trust, and a changed hook is skipped until trusted again.

<a id="O21"></a>
### O21 — Soft claims for concurrent workers (candidate)

**Proposed as a candidate.**
- `pse claim <paths> [--ttl 2h]` records an advisory claim by the calling session in the
  checkout store.
- `pse changes` and the edit guard warn, but do not block, when an agent edits a path another
  live session has claimed.
- Claims expire with their TTL or when the session ends.

This helps when a coordinator runs several executors in one checkout without perfectly disjoint
scopes. Coordinators already assign disjoint scopes, so adopt it at the first collision.

### What this reviewer found harder than necessary

First-hand experience from the review and its follow-up, mapped to the opportunities that address
it:

| What was hard | What helps |
| --- | --- |
| Learning the command surface took two mapping agents | [O06](#O06) catalog and explain, [O14](#O14) `ref` |
| Reconstructing what Codex ran meant reverse-engineering rollout JSONL, including fork-copied records | [O03](#O03) ledger and `pse history` |
| Every subagent brief restated baseline, pins and conventions | [O07](#O07) `pse brief`, [O08](#O08) `pse dep` |
| Checking an exact library API meant registry greps and hand-built toy crates | [O15](#O15) `pse api`, `pse probe` |
| Protected-document edits meant knowing the route, the authorization file and the revision-row duty | [O16](#O16) `pse owner`, `pse design-edit` |
| Validating a review's links and anchors meant a hand-written shell loop | [O17](#O17) `pse docs check` |
| Committing meant proving the shared tree's diff was mine | [O18](#O18) `pse commit` |
| Placement and front-matter conventions were learned by reading three files | [O14](#O14) `pse new` |
| Bash working directories reset, so commands carried `cd` prefixes | [O01](#O01) `-C` and self-location |
| State is lost when a long session compacts | [O20](#O20) `pse handoff` with compaction hooks |
| The clap overview was a duplicate of `futures.md` | Corrected by the operator; this revision uses it |

<a id="adoption-path"></a>
## Adoption: the easiest path, clearly communicated

**Operator direction (2026-10-29):** `pse` is the strongly recommended agent interface. The
audience is capable agents who judge for themselves whether to use it. Adoption therefore comes
from `pse` being the easiest way to get work done, and from agents knowing it exists and what it
does. It does not come from enforcement.

<a id="O22"></a>
### O22 — Make `pse` the obvious first choice

**Easiest path.**
- Common tasks need no flags to remember.
- `pse` finds its checkout itself.
- Output leads with the answer, and verbs are named after tasks: `test`, `check`, `wait`,
  `result`, `fixture`, `api`, `owner`, `commit`, `worktree`.

If agents keep using a raw form for something, that marks a capability `pse` should do better.
Improve it there.

**Communicate presence and capability where agents already look:**
- **AGENTS.md "Start here".** A short task-to-verb table ("run a test → `pse test`", "wait on a
  build → `pse wait`", "exact dependency API → `pse api`", …) pointing to `pse commands --json`
  and the generated reference ([O06](#O06)).
- **The environment guide.** `docs/dev/agent-environment.md` routes each workflow through `pse`.
- **Role contracts and process skills.** `.agents/roles/*`, `execute-plan` and the review skills
  name the `pse` verb for each step they describe, for example `pse brief` when delegating and
  `pse handoff` before long work.
- **Session start.** The [O25](#O25) status brief ends with one line naming the main verbs.
- **In every result.** Each envelope's `next` field and every diagnostic remedy use `pse` forms
  ([O02](#O02)). Using `pse` once teaches the next step.
- **Recipes.** Recipes for work `pse` covers delegate to it or are retired, so recipe users land
  on it too.

No hooks enforce or nudge adoption. The orientation and handoff hooks in [O20](#O20) and
[O25](#O25) only provide context.

<a id="environment-followup"></a>
## Environment changes for agent effectiveness

These complement O09–O14 and were assessed from first principles plus quick reads of the current
setup.

<a id="O23"></a>
### O23 — Worktree lifecycle

**Today.** `just worktree` forwards to `git worktree add`, and the environment guide
(§Worktrees) lists the rest as manual steps:
- copy the personal `.envrc.local` deliberately;
- review and allow its direnv configuration;
- prepare the native host explicitly;
- start with a cold `target/`.

Worktrees are the sanctioned way to run concurrent agents in isolation. Every manual step is a
place where a new worktree ends up half-prepared.

**Proposed: `pse worktree new <name> [--base REF] [--branch B] [--warm]`.**
1. Run `git worktree add`.
2. Copy `.envrc.local` with mode 600.
3. Run `direnv allow` for the new checkout. It is the same repository; nothing needs
   re-reviewing.
4. Prepare host tools with `scripts/bootstrap-host --from <source-checkout>`, which copies the
   generation when fingerprints match ([O13](#O13)). A copy takes seconds; a rebuild is cold.
5. Select the already-prepared native provider generations from the shared cache.
6. Register the checkout in the store ([O24](#O24)).
7. With `--warm`, start a detached `pse check --workspace` through the shared sccache.

**The fingerprint** covers the whole host path-dependency closure plus `Cargo.lock`,
`rust-toolchain.toml` and `.cargo/config.toml`, not only the host crate.

**Listing and removal.**
- `pse worktree list` shows each worktree's branch, ahead/behind, dirty state, live runs and last
  activity.
- `pse worktree rm <name>` removes the worktree. It warns, and needs `--force`, only when
  uncommitted changes or unmerged commits would be lost. It keeps the worktree's run history.

It never shares `target/` (ADR-0122) and never sets `CARGO_TARGET_DIR`.

<a id="O24"></a>
### O24 — One home for results, logs and scratch

**Today.** Results and logs are scattered:
- `target/nextest/default/junit.xml` is overwritten by every run.
- The nextest record store lives per user. Its `-R latest` races between agents, and the
  correctness adapter refuses `-R`.
- Agents invent `/tmp/plan58-*.log` and `build/*.log` paths (386 in one day).
- Codex has no per-session scratch directory.
- `build/` holds 290 unconventioned files.

The host itself resolves state roots in four places:
- `telemetry.rs` and `telemetry_service.rs` (`$XDG_STATE_HOME/pse/telemetry`);
- `environment.rs` (`$XDG_STATE_HOME/pse-arrow/surreal-functional-v2`);
- `admission.rs` (`$XDG_RUNTIME_DIR/pse-host-admission`).

**Proposed: one resolver, one checkout-scoped store.**
- **One resolver** in `environment.rs` owns every host state path (DP-01) and settles the `pse/`
  versus `pse-arrow/` prefix. Existing consumers move to it.
- **The store:**

```text
$XDG_STATE_HOME/pse-arrow/<checkout-id>/
  runs/<run-id>/{meta.json, events.ndjson, output.log, result.json}   # O03 ledger
  fixtures/{study-module.json, …}                                      # O05 current pointers
  api/<crate>-<version>-<features-hash>.json                           # O15 rustdoc cache
  probes/<name>/                                                       # O15 scratch crates
  scratch/<session-id>/                                                # Codex sessions (Claude uses its harness scratchpad)
  handoff/<session-id>.json                                            # O20
  index.json                                                           # recent runs, claims (O21), worktree link (O23)
```

Rules for the store:
- **Location.** `checkout-id` derives from the checkout path and git common directory. The store
  survives `cargo clean`, and `pse status --all` sees every worktree.
- **Retention.** Keep the last N runs per checkout plus pinned runs. Scratch is pruned when its
  session ends or after a fixed age. Retention is an ordinary setting, not a disk guard.
- **JUnit and nextest records.**
  - The default-profile JUnit stays for humans.
  - `pse` reads per-run libtest JSON from the run directory, never the shared JUnit file.
  - The nextest record store stays enabled for human `cargo nextest replay`, with `max-records`
    and `max-age` aligned to the run store.
- **`/tmp` and `build/`.** `/tmp/*.log` stops being necessary. `build/` keeps only recipe outputs
  with a declared report path.

<a id="O25"></a>
### O25 — Harness surfaces for orientation and visibility

**Proposed.**
- **Session start (Claude and Codex).** Both `SessionStart` paths add `pse status --brief` as
  `additionalContext`: HEAD, dirty count, active plan, live runs, a stale host or `.design-edit`.
  - For Claude, extend the existing `agent-hook session-env` handler.
  - For Codex, add the hook. Today Codex has only the edit guard and a per-command environment
    bridge.
- **Status line (Claude).** `.claude/settings.json` gains a `statusLine` command running
  `pse status --line`. It is read-only and quick, and prints nothing useful but harmless when
  the seed exits 125 because the tools are unprepared.
- **Hook trust.** The session-start and compaction hooks in this item and [O20](#O20) only add
  context. Codex needs a one-time `/hooks` trust for them.
- **No new permissions.** Claude already allows `Bash`, and Codex approvals stay inherited from
  user configuration.
- **Tool servers.** Enable the rust-analyzer MCP for Codex ([O11](#O11)), and trial `rustdoc-mcp`
  for both harnesses ([O15](#O15)).

<a id="not-recommended"></a>
## Not recommended

| Proposal | Why not |
| --- | --- |
| A bespoke structured edit tool | That would be a second editing authority. [O12](#O12) covers the measured risk with existing tools. |
| Progress bars for agents | indicatif already hides itself off a TTY, and `ProgressBar::println` prints nothing when hidden. Agents need terminal states and bounded heartbeats, which [O02](#O02) NDJSON `progress` events provide. Use `tracing-indicatif` 0.3.14 (compatible with the pins) only for the human on a TTY. |
| figment for configuration provenance | `explain` must not run shell code, and the layers already exist in `environment.rs`. figment's useful work is unreleased (last release 2024-05), so adopting it would need a git pin. Revisit when typed, nested configuration arrives. |
| Recreating the retired catalogue, MCP tools or generated CLI | They are an idea source only. Their discovery value reappears as derived [O06](#O06). |
| An RMCP adapter now | No consumer needs it beyond the shell. [O02](#O02) schemas make it cheap later. |
| Shell completion (`clap_complete`, AOT or dynamic) | It serves interactive humans, not agents. Valid values reach agents through the [O06](#O06) catalog instead. |
| Prefix inference (`infer_subcommands`, `infer_long_args`) | Adding a verb or flag silently changes what an abbreviation means, which breaks generated agent commands. |
| Man pages, a REPL or a batch mode | Agents read the generated Markdown reference and chain shell commands. A long-lived interactive parser adds state without a consumer. |
| `@file` argument expansion (`argfile`) | It covers only `pse`'s own argv; `pse` generates long filters from intent ([O04](#O04)). |

## Order of work

The operator intends to plan and execute all of these. The order follows dependencies:

1. **No new CLI needed:**
   - [O11](#O11) and [O09](#O09);
   - [O12](#O12) guidance;
   - [O10](#O10) generated role instructions and a slimmed plan index;
   - the [O25](#O25) status line and rust-analyzer MCP;
   - a `rustdoc-mcp` trial ([O15](#O15)).
2. **[O01](#O01) and [O02](#O02) with the [clap contract](#clap-capabilities):**
   - the `pse` bin and library `Cli` in `pse-host` and the shared argument structs;
   - the envelope and exit classes;
   - `status`, `doctor`, `env --explain`, `schema` and `commands`;
   - self-location and the host ceiling.

   Also [O13](#O13) fingerprints, the [O24](#O24) store skeleton, and [O23](#O23) worktrees, which
   need both.
3. **[O03](#O03):** the corrected property set, admission for detached runs, `wait`, `result`,
   `log`, `cancel` and `history`, composite runs, and output extraction.

   The ledger becomes the stop authority here ([direction change 3](#direction-changes)). [O20](#O20) handoff and its compaction hooks
   follow from the ledger.
4. **[O04](#O04) and [O19](#O19):**
   - host-resident `affected`;
   - `clap-cargo` selection;
   - the leaf capability declaration;
   - the reconciled verdict;
   - test discovery.
5. **[O05](#O05):** `pse fixture` and the receipt pointer, then the bindeps step.
6. **The remaining verbs:**
   - [O06](#O06) catalog, valid values and generated reference;
   - [O08](#O08) `dep`, `outline`/`show`;
   - [O15](#O15) `api`/`probe`;
   - [O16](#O16) `owner` and guard messages;
   - [O17](#O17) `docs check`;
   - [O14](#O14) `ref`/`new`;
   - [O07](#O07) `changes`/`brief`;
   - [O18](#O18) `commit`.
7. **[O22](#O22) communication** lands with each step. The AGENTS.md task-to-verb table, guide
   routes and skill mentions are updated as each verb ships, not at the end.
8. **The `pse-host` → `pse` rename of the hook path**, if still wanted, landing atomically with
   `just setup-test`.

[O21](#O21) waits for the first concurrent-edit collision.

**Interactions:**
- O02 is the contract every verb emits through.
- O03 carries O04's long runs and records O10's delegated workers. Its ledger is the substrate for
  O20 handoff, O21 claims and O24's index.
- O05 completes O04 for integration and database targets.
- O06 is derived from the clap tree, and O19 from O04's declarations.
- O14, O16 and O07 share citation, ownership and rule resolution.
- O22's communication relies on O02 `next` hints, O06's catalog and O25's session brief.

<a id="direction-changes"></a>
## Direction changes proposed

These are the current rules and directions that would cost agents effort under this design. Each
entry proposes the change, in the form "change it, because…". The operator already decided
the first one; the rest go through the ordinary plan-amendment or ADR route as one batch, not as
gates on the work.

1. **The CLI becomes the agent interface.** Plan58 §Purpose and its Outcome say "CLIs are optional
   conveniences", with no command framework, universal catalogue or agent execution engine.
   - Replace that with the operator's 2026-10-29 direction: `pse` is the strongly recommended
     agent interface, adopted by being the easiest path ([O22](#O22)).
   - The catalog is derived from the parser (O06), not authored, so the I09 "universal
     catalogue" concern does not apply.
   - Blueprint §21.7/§24.1's "no mandatory CLI" can stay: nothing is made mandatory.
2. **Owned-run supervision is not build observation.** Plan58, 58b and 58h retired bespoke build
   observation along with its wait and freshness consumers. The environment guide retired watch
   cursors.
   - Keep build-stage telemetry retired.
   - Explicitly allow [O03](#O03)'s terminal-outcome supervision of runs agents start. Polling
     logs by hand is the measured cost it removes.
3. **ADR-0233, three narrowings:**
   - **CLI grammar.** Allow one CLI grammar and a per-run supervisor; the status history says
     neither was selected.
   - **Stop authority.** Let the ledger's record be the stop authority for detached runs, so any
     `pse` in the user's session can cancel them ([O03](#O03)).
   - **"Launch never builds".** Narrow it to read-only verbs, so effectful verbs rebuild a stale
     host instead of running it ([O13](#O13)).
4. **Protected documents inform instead of block.** AGENTS.md "Off-limits" and the edit guard
   block architecture sections and accepted ADRs unless `.design-edit` or `PSE_DESIGN_EDIT=1` is
   set.
   - Keep blocking generated paths, where edits are genuinely lost.
   - For documents, replace the block with context naming the route. Agents then follow it
     without workarounds ([O16](#O16)).
5. **Any agent commits its own paths.** AGENTS.md says the coordinator owns the shared index and
   formatting happens only at `just turn-end`. [O18](#O18)'s private index and whole-file
   formatting remove the reasons for both rules for paths being committed.
6. **Generated role instructions.** Allow generated, checked embedding of `.agents/roles/*` into
   Codex `developer_instructions` ([O10](#O10)). CLAUDE.md's "load shared contracts rather than
   copying role behavior" currently forbids it.
7. **Worktrees are ready on creation.** Replace the environment guide's manual §Worktrees steps
   (supply `.envrc.local` and review/allow direnv "deliberately") with `pse worktree new`, which
   does all of it ([O23](#O23)).
8. **AGENTS.md "Start here" leads with `pse`.** Bare tools stay first-class. The guidance simply
   puts the easiest path first ([O22](#O22)).

Nothing else in this review needs a rule change:
- the new dependencies (tracing-subscriber `json`, clap `env`/`string`/`unstable-ext`, schemars
  `raw_value`, `clap-cargo`, `rustdoc-types`) are ordinary pin moves;
- reusing the `pse-host` crate avoids a new crate.

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
- **Follow-up verbs, each used in ordinary work:**
  - an exact dependency signature answered by `pse api` (or the trialled MCP);
  - a blocked protected edit that names its owner and route;
  - `pse docs check --changed` before committing a document;
  - `pse commit --paths` in a shared tree;
  - `pse tests <pattern>` returning a runnable command;
  - after compaction, the session resuming from the `pse handoff` context it was given;
  - `pse worktree new` producing a ready checkout whose first build hits the shared cache;
  - agents reaching for `pse` first because the start route and results point there.

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
- Follow-up sources:
  - the corrected [clap overview](../libraries/clap.md);
  - pinned `clap_builder-4.6.7`/`clap_derive-4.6.7` sources;
  - `clap-cargo` 0.19.0 (<https://docs.rs/clap-cargo/0.19.0>);
  - `trycmd` 1.2.1 and `snapbox` 1.2.2;
  - `rustdoc-types` 0.61.0, plus a scratch rustdoc-JSON probe on nightly-2026-09-29;
  - `rustdoc-mcp` 0.10.0;
  - Codex hooks documentation (via Context7 `/openai/codex`) and Claude Code hooks
    documentation;
  - `justfile` recipes `worktree`, `plan` and `adr-new`, and `.github/workflows/docs.yml`
    (lychee).
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
