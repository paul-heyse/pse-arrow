# R3 — Evidence frameworks: rerunnable probes, automatic conditions, staleness, findability

**Role:** library-research worker (shared contract `.agents/roles/library-research.md`). This is
research that offers options with their costs. It does not make an architecture decision.

**Date:** 2026-10-09. **Checkout baseline:** `pse-arrow` HEAD `4c24721e6`, dirty worktree (355
porcelain entries that were already there; none written by this work).

**Sources**
- Context7: `/astral-sh/uv/0.12.22` (scripts guide, 0.5.17 changelog), `/rust-lang/cargo`
  (unstable `script` section), `/treeverse/dvc.org` (status/repro/params), `/websites/quarto`
  (freeze).
- Primary docs, fetched 2026-10-09:
  - doc.rust-lang.org/nightly/cargo/reference/unstable.html#script
  - doc.rust-lang.org/nightly/cargo/reference/config.html (`resolver.lockfile-path`)
  - sacred.readthedocs.io/en/stable/collected_information.html
- Project-status pages: rust-lang project goal "Stabilize cargo-script" (2026, Accepted),
  rust-lang/cargo#12207, rust-lang/rust#148051 (frontmatter stabilization report).
- Registry versions (crates.io and PyPI APIs, 2026-10-09):
  - rust-script 0.36.0 (2025-08-16)
  - insta 1.49.0 (pinned in the workspace)
  - syrupy 6.1.1
  - dvc 3.67.1
  - mlflow 3.17.0
  - aim 3.29.1
  - sacred 0.8.7
  - jupyter-cache 1.0.1
- Repository data points:
  - `scripts/validation_scope.py` (`input_identity`) and `scripts/validation_receipts.py`
    (`_reuse_guarded`)
  - `scripts/evidence-regen.sh`
  - the two `run.sh` harnesses
  - the inventory of `docs/design_review/evidence/`: 17 entries, 132 files, 20 MB

**Conditions for every Tested claim**
- Host: Linux x86_64.
- Rust: rustup toolchain `nightly-2026-09-29` = cargo 1.101.0-nightly (3d7cf6e93 2026-09-25),
  rustc 1.101.0-nightly (c1070d693 2026-09-28), LLVM 23.1.1. Stable 1.99.0 was used for one
  comparison.
- Python: uv 0.12.22 with CPython 3.14.7, plus the repo `.venv`.
- sccache was the rustc wrapper (from `~/.cargo/config.toml`).
- Every probe ran offline in the scratch directory
  a session scratch directory, never inside the checkout. The one exception is a single
  `cargo metadata` run with cwd set to the repo root, which wrote nothing to the repo.
- The two build directories the probes created under `~/.cargo/build/` were deleted afterwards.
- During the first unseeded build the disk filled (`ENOSPC`, root filesystem at 99%, about 23 GB
  free). Disk cost per probe is therefore a real constraint (see Q1).

Evidence labels follow the design principles: *Interface-checked* means read from documentation
or source, *Tested* means a probe was executed (named), and *Measured* means timed.

---

## Q1. Self-describing, rerunnable probes with near-zero setup

### Python: PEP 723 plus `uv run --script`

*Interface-checked* (uv 0.12.22 docs):
- A `# /// script` block declares `requires-python`, `dependencies` and `[tool.uv]`.
  `exclude-newer` can also be set there.
- `uv lock --script probe.py` writes an adjacent `probe.py.lock`. Later `uv run --script`,
  `uv add --script` and `uv remove --script` runs respect it, and `uv export --script` and
  `uv tree --script` work on it.
- A shebang of `#!/usr/bin/env -S uv run --script` makes the file directly executable.

*Tested* (probe `probes/evidence-runner/uv-script/probe.py`, exact pin `packaging==25.0`,
`requires-python = "==3.14.*"`, `exclude-newer = 2026-10-01`):
- `uv lock --script` resolved in 144 ms and produced `probe.py.lock` (lock version 1,
  revision 5). The lock records `exclude-newer`, the manifest requirements, and the hashes of
  the sdist and the wheel.
- `uv run --script --locked probe.py` printed `{"python": "3.14.7", "packaging": "25.0"}`.
- `uv lock --script --check` returned 0.
- **Drift detection works.** After the pin was edited to `24.2`, `uv run --script --locked`
  exited 1 with "The lockfile … needs to be updated, but `--locked` was provided". The message
  misnames the file as `uv.lock`; the behaviour is correct.

**Fit for this repo: partial.** Of the 12 committed `.py` evidence probes, the two inspected
(`efficiency-principles…/input-scope-probe.py`, `agent-effectiveness…/probe-interfaces.py`) use
no third-party packages. They import repository internals through
`sys.path.insert(0, parents[4])` and run under `.venv/bin/python` via `scripts/pse-env`.
- PEP 723 cannot cheaply give such a probe the repo. The `pse` package is a maturin extension,
  so an isolated script environment would have to build it.
- PEP 723 is the right form for probes that use **only third-party packages**. It replaces a
  hand-written "how to run" note with a declaration that is checked.
- Repo-internal probes keep the repo venv. Their conditions come from the capture wrapper
  (Q2), not from a header.
- pytest is not a good host for evidence. `conftest.py` hard-fails any test that lacks exactly
  one of unit/component/integration/performance, so evidence would need a fifth category or a
  path outside collection.

### Rust: single-file Cargo scripts

**Stabilization status.**
- *Tested*: `cargo -Zscript` remains **unstable** on the pinned nightly.
  `cargo hello.rs` without the flag fails with "running the file `hello.rs` requires
  `-Zscript`". It fails the same way on stable 1.99.0.
- *Tested*: the **frontmatter language syntax is accepted without a feature gate on stable
  1.99.0 and on the pinned nightly**. `rustc --edition 2024 hello.rs` parsed a `---` fenced
  manifest, and the only error was the unrelated `env!("CARGO_PKG_NAME")`. The language half
  (rust-lang/rust#136889, stabilization report #148051) has therefore landed. The Cargo half
  (#12207) has not; the 2026 project goal "Stabilize cargo-script" is Accepted but not done.
  The repo is nightly-only (ADR-0122), so the flag costs nothing here.
- *Tested*: a bare `---` fence with no `cargo` infostring is accepted.

**Constraints.** *Interface-checked* (cargo unstable docs):
- The embedded manifest may not contain `[workspace]`, `package.workspace`, `[lib]`, `[[bin]]`,
  `package.build` or `package.links`. A script therefore **cannot use `dep.workspace = true`**
  and cannot have a build script.
- The default target directory is per script, under `$CARGO_HOME`.
- The default lockfile lives in the target directory, which makes it ephemeral.
- `cargo <path>` applies the config for `<path>`; `cargo run --manifest-path` uses cwd config.
  rustup chooses the toolchain from cwd, so a script must be run from the repo root to get
  `rust-toolchain.toml` without a forbidden `+toolchain`.

**Path dependencies on workspace crates.** *Tested* (probe `probes/evidence-runner/cargo-script/ids_probe.rs`
with `pse-ids = { path = "/home/paul/pse-arrow/crates/pse-ids" }`):
- It builds and runs. The output was `pse_ids linked: pse_ids::id::ContentHash`.
- The path dependency's own `x.workspace = true` entries resolve through the pse-arrow
  workspace. `pse-diagnostics` came in as a path package.
- Consequence: a probe that depends on a `pse-*` crate inherits the vendored
  `vendor/surrealdb` path and the POUNCE git rev **without restating them**.

**Reusing the workspace lockfile.** *Tested*:
- `--config "resolver.lockfile-path='<scratch>/Cargo.lock'"` works on the pinned nightly.
  *Interface-checked*: the config docs mark this stable with MSRV 1.97+ and say the path must
  end in `Cargo.lock`. The `-Z` and `--lockfile-path` CLI forms are gone, and a file named
  `repo-copy.lock` was rejected.
- When seeded with a **copy** of the repo `Cargo.lock`:
  - every resolved third-party package matched the repo lock;
  - the only absent entry was the script's own root (`ids_probe 0.0.0`).
- **Cargo rewrites the copy, pruning it to the script's graph.** The lockfile path must never
  point at the real repo lock.
- A rerun with `--locked --offline` succeeded in 0.05 s.

**Restating pins is unnecessary.** *Tested*:
- A frontmatter of `arrow = { version = "*", features = ["force_validate"] }`,
  `datafusion = "*"` and `blake3 = "*"`, seeded with the repo lock, resolved to exactly the repo
  pins: arrow 59.3.0, datafusion 55.1.0, blake3 1.8.7, arrow-array 59.3.0, object_store 0.13.2.
- Zero packages were absent from the repo lock.
- This is a wildcard *requirement* paired with a lock that fixes the *version*, so prime
  directive 3 (one declaration per meaning) holds. The run.sh harness's TOML generation from
  `[workspace.dependencies]` becomes unnecessary.

**Fidelity gap.**
- The script is its own resolution root. Workspace feature unification
  (`resolver.feature-unification = "workspace"`) and `pse-workspace-hack` do not apply.
- A probe sees the feature set its own graph asks for, not the production one.
- `force_validate` must be declared in the frontmatter. It is one token, and the old `run.sh`
  carried it as an explicit feature too.
- Where the exact production feature set matters, the high-fidelity alternative is an example
  target or test inside an existing workspace crate. That costs a manifest edit or a fixed
  `examples/` location. Crate additions need an ADR here.

**Cost.** *Measured*:
- `ids_probe` first build: 9.86 s wall, 298 MB max RSS, warm sccache, about 200 MB build
  directory.
- Locked rerun: 0.05 s.
- The default build directory is `~/.cargo/build/<hh>/<hash>/target`, one per script, so
  arrow or DataFusion probes each pay GBs of disk. A fixed shared evidence target directory
  (for example `build/evidence-target`) avoids that. It is one directory shared across
  scripts, not two checkouts sharing one target, so ADR-0122's concern does not apply.

**Is rust-script still needed?** No.
- rust-script 0.36.0 (last release 2025-08-16) exists mainly for stable toolchains and uses its
  own `//! ```cargo` manifest form.
- On a nightly-pinned repo, `-Zscript` is native, uses the eventual stable syntax, and supports
  every Cargo subcommand (`build`, `metadata`, `test`) through `--manifest-path probe.rs`.

**Can scripts replace the bespoke `run.sh` harnesses?**
- **Yes for both `run.sh` files.** Their real jobs were:
  - generate a manifest from workspace deps (replaced by `*` requirements plus the seeded lock);
  - copy the lock and assert no new versions (the seeded lock plus a post-check against
    `cargo metadata`);
  - print the HEAD, lock, probe digests and toolchain (replaced by the shared capture in Q2);
  - enable force-validate (one token in the frontmatter);
  - run offline and locked (runner flags).
- The two files differ only in a temporary-directory prefix and a package name (`diff`: 2
  lines).
- **No for `scripts/evidence-regen.sh`'s rustdoc extraction.** It needs a `[lib]` root and a
  generator script, both of which scripts disallow. Its `probe_*.rs` binaries would fit as
  scripts.

## Q2. Automatic condition capture

### Cheap, no-author-effort captures (*Measured* on this checkout)

| Condition | Command / source | Cost | What it identifies |
|---|---|---|---|
| Rust packages actually in the probe's graph | `cargo -Zscript metadata --manifest-path probe.rs` (seeded lock) | ~0.3 s (repo-wide `cargo metadata --offline --locked` took 0.33 s) | `(name, version, source)` per package. `source` = `registry+…`, `git+…?rev=…#sha`, or `null` for path. Repo graph: 782 registry, 35 path, 9 git. Vendored `surrealdb` and `surrealdb-engine-api` appear as path packages under `vendor/` |
| Source files compiled into the probe | the binary's dep-info file (`<exe>.d`, path from `--message-format=json` `compiler-artifact`) | free after the build | *Tested*: 16 entries, of which 15 are repo files (`pse-ids`, `pse-diagnostics` sources). Registry sources are excluded, so path and vendored trees appear and lock-pinned crates do not |
| Content of path or vendored trees | `git rev-parse HEAD:vendor/surrealdb` (tree id) plus `git status --porcelain -- <paths>` | ~2 ms / ~20 ms | content address of a committed tree, and whether it is dirty |
| Per-file content | `git hash-object --stdin-paths` or BLAKE2/BLAKE3 | ms | blob ids, which also let `git log --find-object` locate the change later |
| Toolchain | `rustc -vV` (commit-hash, LLVM), `uv --version`, `sys.version` | 10 ms | exact compiler identity, which is stronger than reading `rust-toolchain.toml` at rerun time |
| Python modules actually imported | `sys.modules[*].__file__` at exit | free | repo files and site-packages distributions (`importlib.metadata.packages_distributions`) |
| Files read and processes spawned | `sys.addaudithook` on `open` (read modes) and `subprocess.Popen` | negligible | data inputs (`Cargo.lock`, config, fixtures) and external tools invoked |
| Git | `git rev-parse HEAD`, `git status --porcelain` | ms | revision plus dirty state, only as context; staleness is computed from content, not HEAD |

**Inferring the relevant conditions instead of recording everything.**
*Tested* with the scratch wrapper `probes/evidence-runner/capture.py`, about 50 lines. It runs the
existing `input-scope-probe.py` through `runpy` under the repo `.venv`, and wall time was
0.14 s. With no author effort it produced:
- the probe file and `scripts/__init__.py`, `scripts/validation_scope.py` and
  `scripts/native_cache.py`, each with a digest;
- no third-party distributions;
- one subprocess (`git`), which was the wrapper's own call. A real implementation filters
  `__pycache__` files and its own calls.

Sacred (0.8.7) does the same thing as a library (*Interface-checked*). It auto-discovers local
sources by "inspection of the imported modules and comparing them to the local file
structure", and records them with md5, `package==version` dependencies, host info and git
`{url, commit, dirty}`. That is prior art for the exact mechanism. Sacred's observer and run-store
model is the heavy part, so take the idea, not the framework.

**Recommended inference rule (option, not decision):**
- record cheap global context: HEAD, dirty, `rustc -vV`, uv version;
- compute staleness only from the **used set**:
  - Rust: dep-info path files, plus the `(name, version, source, checksum)` of packages in the
    probe's own resolve;
  - Python: imported repo files, files read, distributions of imported modules, and spawned
    executables.
- A whole-`Cargo.lock` digest is too coarse: every unrelated bump would flag every bundle.

**What the used set catches among the observed drifts:**
- SurrealDB moving to the vendored path at the same version: the lock entry's `source` changes
  from registry to `null`/path. Caught.
- POUNCE moving from crates.io to a git fork: the `source` changes to `git+…rev`. Caught.
- The toolchain changing: `rustc -vV` commit-hash. Caught. This is stronger than the `run.sh`
  approach of reading `rust-toolchain.toml` at rerun time.

**Named constants** (the interpretation constant moving v2 to v3, the schema version moving v1
to v3):
- A file digest already flags these, but only as "file changed", which is noisy.
- A cheap no-author option: scan only the used-set files for
  `const [A-Z0-9_]*VERSION[A-Z0-9_]*\s*:\s*\w+\s*=\s*(\S+);` in Rust and
  `^[A-Z0-9_]*VERSION[A-Z0-9_]* = ` in Python, then record name and value. The rg or ast-grep
  cost is milliseconds.
- This yields a two-tier signal: **semantic** (a version constant, a lock `source`/`version`,
  or the toolchain changed) versus **textual** (a used file's digest changed).
- It is heuristic: it covers only constants named `*VERSION*` and only those inside used
  files. For Python, a module-global `*VERSION*` read via `getattr` at exit is exact.
- Not covered: probes that talk to a **server** over HTTP or WebSocket (SurrealDB). The server
  binary appears as a spawned process only if the probe starts it; otherwise the probe must
  query and print the server version. That costs the author one line and is the one place an
  author line is needed.

## Q3. Staleness signals and refresh entry points

| Approach | "May be stale because X" for free? | One-command refresh? | Weight / verdict |
|---|---|---|---|
| **Captured used-set vs current** (Q2 data + ~100-line comparator) | Yes. It names X (file, lock source/version, toolchain, version constant) | Yes: rerun the same probe through the same runner | Light. Recommended core |
| DVC 3.67.1 stages (`dvc.yaml` deps/params/outs, `dvc.lock` md5; `dvc status` prints "changed deps: modified: foo"; `dvc repro` skips unchanged stages) | Yes, but only for **declared** deps, which is author effort unless generated from the capture | Yes (`dvc repro`) | Right model, wrong weight: a large dependency closure, `.dvc/` cache and repo state. The `deps`/`params`/`dvc status` shape is worth copying; installing DVC is not |
| Bazel / Nix input hashing | Yes, hermetically | Yes | Too heavy; would duplicate Cargo and uv resolution |
| `git log <captured_head>.. -- <paths>` | Partly. Fails for dirty-at-capture state and branch switches, and is noisy on unrelated edits | No | Free supplement; prefer blob-id comparison |
| insta 1.49.0 (already pinned) / syrupy 6.1.1 snapshots | Only after a rerun. A diff shows *what* changed in output, not *why* | `cargo insta test --review` / `pytest --snapshot-update` | Good second half: detects output drift once a refresh runs. Redactions handle timings and paths. syrupy would conflict with the conftest category rule unless kept outside collection |
| Quarto freeze (`freeze: auto` = "re-render only when source file changes") / jupyter-cache 1.0.1 | **No.** Keyed on the document or cell source only. Imported code, lock or toolchain changes are invisible, which is exactly the observed failure | Yes | Unsuitable as a staleness signal |
| MLflow 3.17.0 / Aim 3.29.1 / Sacred 0.8.7 | Sacred captures the right inputs. None compares them to current state as a staleness verdict | No | Too heavy (tracking server or run DB, not git-reviewable). Borrow Sacred's capture idea only |

What gives staleness "for free with a one-command refresh":
- record the used-set conditions at run time;
- compare on demand;
- refresh by rerunning the identical command.

Optionally, diff the **decisive output** against the committed one, insta-style, so a refresh
reports either "conclusion unchanged under new conditions" (cheap re-validation, which answers
"leads only" directly) or "output changed: review".

`validation_receipts.py` already embodies the two outcomes: unchanged-input reuse, and a
reviewed transfer with a reason. Its input scopes are **hand-declared path prefixes**
(`INPUT_SCOPES`), whereas evidence can get its scope inferred from the run.

## Q4. Not re-answering the same question

**Observed.**
- Bundles are named for the **review that commissioned them**, not for the question.
- SurrealDB capability material sits in at least 9 differently-named places, for example
  `execution-efficiency-2026-10-05/surrealdb-capabilities.md`,
  `surrealdb-unified-substrate-2026-10-05/`, `plan-28-surrealdb-capabilities-2026-10-06/`,
  `production-execution-efficiency-2026-10-07/surreal/` and `efficiency-principles…`.
  rg finds them all instantly.
- The failure is therefore not *finding* but **trusting**: the later answers call earlier ones
  "leads only" because nobody can tell whether they still hold.
- Findability only pays off once it is paired with a validity status, which is what Q2 and Q3
  provide.

**Options, ordered by authoring effort:**
1. **Status-bearing search, generated on demand (zero authoring).** `evidence-find <words>`
   ranks bundle Markdown by BM25 and prints, per hit:
   - the H1;
   - the first paragraph;
   - `current` / `stale: <X>` / `no capture`, from the Q2 comparison.
   *Tested*: stdlib `sqlite3` FTS5 is available (SQLite 3.53.1 in uv CPython 3.14.7). Indexing
   only `*.md` keeps it tiny, since the 20 MB is mostly JSON. Rebuild per query; do not commit
   the index or a catalog file, because a committed generated catalog is a second declaration
   that drifts.
2. **A scaffold that searches first.** `evidence-new <slug> "<question>"` prints the top hits
   with status before creating the folder. An agent creates the folder anyway, so the search
   adds no step; it is offered, not required. Prior art: the `just plan <slug>` scaffold recipe already
   exists in the justfile.
3. **Question-keyed placement.** The convention would be `evidence/<subject>/<question-slug>/`
   instead of `<review>-<date>/`, with the commissioning review linking to it. This collocates
   repeated answers, at the cost of a naming judgment by the author and a move of existing
   bundles. It is useful but not needed if option 1 exists.
4. **Semantic or embedding search.** It adds a model dependency and yields little over BM25 on
   about 20 bundles. Not worth it now.
5. **Library-capability questions** ("what can SurrealDB 3.3 do") overlap the shared library
   skill (`neo4j-surrealdb`). Repeated re-answering partly reflects that the skill is pinned to
   the registry release while the repo now uses `vendor/surrealdb`. The captured
   `source = path` condition makes that transfer question explicit instead of silent.

## Q5. Smallest common collection framework: options with costs

**Shared shape** (applies to every option):
- One bundle per question.
- The probe source is the only authored executable part.
- One runner:
  - executes the probe under `scripts/pse-env --resource-class <class> --`;
  - captures conditions automatically;
  - writes a small manifest;
  - supports rerun and status.

### Option A — runner plus native script formats (recommended to evaluate first)

**Probe forms.**
- `probe.py`: either a PEP 723 header (third-party-only probes, run via
  `uv run --script --locked`), or plain under the repo venv (repo-internal probes).
- `probe.rs`: `-Zscript` frontmatter with `*` requirements and `force_validate` declared where
  Arrow is involved, or path dependencies on `pse-*` crates.

**Runner** (`scripts/evidence.py`, estimated 200–300 lines, plus thin `just` recipes):
- `run <bundle>`:
  - Rust: copies the repo `Cargo.lock` to `build/evidence/<bundle>/lock/Cargo.lock`, sets
    `resolver.lockfile-path`, `--offline --locked` after the first resolve, and a shared
    `CARGO_TARGET_DIR=build/evidence-target`. It asserts that nothing was resolved outside the
    repo lock (the check `run.sh` did) and captures `cargo metadata` plus dep-info.
  - Python: the audit-hook and `sys.modules` capture (the `capture.py` shape).
  - Writes `conditions.json` (used set, semantic tier, global context) and `output.json`
    (stdout, if JSON).
- `status [bundle]`: compares `conditions.json` with the current tree and prints the "stale
  because" lines.
- `find <words>`: FTS5 plus status.
- `new <slug> <question>`: find, then scaffold.

**Costs.**
- About 300 lines of new bespoke code. It deletes two 90-line `run.sh` files.
- Rust probes lose workspace feature unification (declare features explicitly).
- `-Zscript` is unstable, so syntax churn is possible until #12207 lands. The frontmatter
  syntax itself is stable.
- A shared evidence target directory costs GBs, but only once.

### Option B — the runner, with probes as workspace example or test targets

- Probes become `examples/evidence_*.rs` in one designated existing crate, or `#[ignore]`d
  nextest tests in a group.
- High fidelity: the workspace lock, feature unification, force-validate routes and nextest
  JUnit come free.
- Costs:
  - probe sources live away from the bundle folder (or need `[[example]] path=` edits);
  - examples compile with the crate's dev-deps;
  - a probe that needs a dependency the crate lacks requires a manifest edit.
- Condition capture still needs the runner, though dep-info is available the same way.

### Option C — conventions only, no runner

- Use a PEP 723 / cargo-script header and a README template.
- Zero code, but no staleness signal. It fails the governing test: the observed problem is
  unchanged.

### Option D — adopt DVC

- `dvc.yaml` stages, generated from the capture so authors declare nothing; `dvc status` and
  `dvc repro`.
- Costs: a heavy toolchain, a `.dvc/` cache, and state in the repo. Its lock duplicates what
  `conditions.json` holds. Not recommended.

### Where outputs live (applies to any option)

**Committed in the bundle, the small decisive set:**
- probe source;
- `conditions.json`;
- the decisive output, redacted of timings and absolute paths so a rerun diff is meaningful;
- the README conclusion.

**`build/evidence/<bundle>/<run-stamp>/` (gitignored, already the place for raw campaign
outputs):**
- raw logs and large JSON;
- the seeded lock copy;
- the probe target directory.

Record the raw files' digests in `conditions.json` if a conclusion cites them.

**Current misfits:**
- `native-producer-inputs-2026-10-06/runtime-selected-native-inputs.json` (8.1 MB) and
  `guarded-runtime-native-inputs.json` (6.2 MB) are committed;
- `agent-effectiveness…/retrospective/event-pointers.jsonl.gz` (2.4 MB) is committed.

### Governing-test check

| Option | Removes agent effort | Adds agent effort |
|---|---|---|
| A | Hand-written environment and condition prose; re-investigation once a hit is shown `current`; the run.sh boilerplate | Writing a probe in a standard form, which is already done today |
| B | The same, plus feature fidelity | Probe placement and manifest friction |
| C | Nothing | — |
| D | Some | DVC itself |

Programmatic control: in A, staleness is computed, never declared, and nothing requires an
author to register or tag anything.

## Uncertainties and limits
- Cargo-script behaviour was tested only on the pinned nightly; its syntax and flags can still
  change before stabilization.
- `resolver.lockfile-path` stability was read from the nightly config docs and exercised only
  on the pinned nightly.
- The dep-info result was observed for one small probe. Dep-info for probes with build scripts
  in path dependencies (native `-sys` crates) was not exercised. Those inputs, the C sources and
  env vars that build scripts read, need `cargo:rerun-if` information instead.
- The Python capture was tested on one read-only probe. Probes that use `multiprocessing` or
  spawn Python children would need the hook in each child (for example via
  `sitecustomize`/`PYTHONSTARTUP`).
- The version-constant scan is a naming heuristic.
- DVC, MLflow, Aim and Quarto were assessed from documentation only, not installed.

## Retained artifacts

Retained under `probes/evidence-runner/`: the Python used-set capture wrapper `capture.py` with its
`conditions.json` and `probe-stdout.json`, the PEP 723 probe `uv-script/probe.py` and its
`probe.py.lock`, and the four Rust frontmatter scripts in `cargo-script/`. Not retained: the
`cargo metadata` outputs, the lockfile copies, the build log and the scratch build directories
(deleted after the runs).
