# AGENTS.md

Instructions for any coding agent working in this repository. Canonical for both
Claude Code (via `CLAUDE.md`, which imports this file) and Codex (which reads this
file directly).

## What this repository is

`pse-arrow` is an Arrow-native process systems engineering core in Rust: typed
relations as the only model authority, a relational math IR, DataFusion-based
inference, native NLP solving, and a generated Pyomo backend.

It is a **clean-room re-implementation** of core IDAES-PSE capabilities, parity-tested
against `idaes-pse==2.12.0`. **Not affiliated with IDAES.** Read `external/idaes-pse`
for *behaviour*; never copy its code, docstrings or comments. The parity harness is the
only sanctioned coupling, and the enumerations preserved by name are listed in the
blueprint (§6.14). See `docs/relationship-to-idaes.md`.

Crates are `pse-*` under `crates/`. The Python package is imported as `pse` and lives in
`python/pse`; the PyPI distribution is `pse-arrow`.

## Start here, every session

```bash
just doctor        # is this working copy able to do work?
just bootstrap     # if it complains -- idempotent, safe to re-run
just --list        # the command surface
```

`just --list` is the contract. Prefer a recipe over an ad hoc command: recipes own the
feature flags, profiles, report paths and tool paths, so those can change without you
re-learning them. If no recipe fits, say so rather than improvising a long command line.

`direnv` activates the environment on `cd` (run `direnv allow` once). It never installs
or downloads — `just bootstrap` does that, visibly.

For Python/native development, run `just py-sync` to refresh the editable extension
using the dev profile, then `just py-test` and `just quality`. Use
`just parity-container` when solver-backed parity is needed. Full wheel/sdist builds
are manual (`just wheels-check <ref>`) or part of a release; ordinary PRs do not wait
for distribution builds. CI uses editable development builds for Python and parity.
These local recipes are available on demand, not prerequisites for committing or
pushing. Git hooks run static checks only: no native compilation, Python tests,
parity or distribution builds. GitHub runs the required suites before merging.

## Prime directives

1. **The baseline is zero.** No quality baselines exist and none will be introduced. A
   lint finding, a failing test, a warning: the target is none, not "no worse than before".
2. **Never edit a generated directory. Fix the generator, then `just codegen`.** The
   generated paths are `docs/generated/`, `crates/*/src/generated/`,
   `crates/pse-ipopt-sys/src/bindings.rs`, and `python/pse/contracts/`. `just codegen-check`
   is what CI runs; a hand edit there is a red diff, not a fix.
3. **One authoritative declaration per meaning.** Version pins live once in `Cargo.toml`
   (`[workspace.dependencies]`) and once in `pyproject.toml`. A schema is declared in the
   registry, never inferred. If you find yourself writing a fact down twice, one of the
   two is wrong and nothing will tell you which.
4. **Label every claim with the evidence vocabulary** (charter §D): *Proposed*,
   *Interface-checked*, *Implemented*, *Tested*, *Measured*, *Formally established*. The
   labels are mandatory in PR descriptions, ADR `evidence:` fields and plan Verification
   sections. `Tested` and `Measured` must name the test or benchmark and its conditions.
5. **An ADR comes before the change**, not after, when the change alters a D1–D14
   decision, adds or removes a crate, or adds, drops or majors a dependency family. See
   "When an ADR is required" below.
6. **Use the tool from `.venv` and the pinned toolchain, never `$PATH`.** A stale global
   `ruff` or a nightly `cargo` silently produces a different result than CI.
7. **Report a failure count with its baseline and the command.** Never report that tests
   pass without naming the command, the mode, and the baseline.

## Repository map

| Path | What it is | How to treat it |
|---|---|---|
| `crates/` | The 23 `pse-*` crates plus `pse-ipopt-sys`, `pse-buildinfo` | Ours; see `.claude/rules/rust.md` |
| `xtask/` | Everything that needs Rust APIs, JSON or cross-platform behaviour | Logic lives here, the justfile is the surface |
| `tests/` | Workspace test crates: `governance`, `engine`, `conformance`, `lifecycle`, `structural` | `tests/golden/` is data, not a crate |
| `benches/` | Criterion benchmarks (`pse-benches`) | No timing gate in CI; `just bench-smoke` only runs them |
| `python/pse/` | The Python package (import name `pse`) | `python/pse/contracts/` is GENERATED |
| `docs/authoritative_design/` | **The blueprint.** Off-limits to edits | Read constantly, write never |
| `docs/adr/` | Decision records, immutable once accepted | `just adr-new`; index via `just adr-index` |
| `docs/plans/` | Implementation plans, living until done | `just plan <slug>` |
| `docs/capability-maps/` | Pinned third-party API maps + their evidence | `just lib-outline <file>` first; they are large |
| `docs/design_review/` | The design charter and the reviews written against it | The `design-review` skill's output contract |
| `docs/generated/` | `pse-schema` output | Never edit |
| `external/` | Pinned read-only checkouts (`just fetch-external`) | **Not source.** Gitignored, never edited, never copied from |
| `build/`, `target/` | Build output | **Not source.** Regenerable |
| `docker/solvers/` | The Ipopt 3.14 + MUMPS + ASL recipe | Changing it changes CI's solver image |

## Where authority lives

Do not restate these; cite them.

- **`docs/authoritative_design/blueprint.md`** — the authoritative architecture. Section
  numbers are stable citation targets: cite `blueprint §14.3`, never a line number.
- **`docs/adr/README.md`** (generated index) and the ADRs themselves — *why* a decision
  was made. The blueprint says what is true; ADRs say why; reviews are evidence, not authority.
- **`docs/adr/register.md`** — every deferred decision with its trigger, check and next
  review date. `just register-check` runs the ones that are due.
- **`Cargo.toml` header comment** — why the arrow/datafusion/object_store/pyo3 pins are
  what they are, and why `=` pins alone are not sufficient.
- **`pyproject.toml`** — every Python library `==`-pinned; tool versions under
  `[dependency-groups]`, never installed ad hoc.
- **`docs/capability-maps/`** — what the pinned libraries actually expose, with evidence.
  `just lib-outline docs/capability-maps/arrow-rust.md` before reading one.

## Invariants

- **One type universe.** Exactly one resolved version of `arrow`, `parquet`,
  `object_store` and `datafusion`. Two majors make `downcast_ref` return `None` with no
  compile error — a silent failure that looks like a logic bug. `just family-check`.
- **`force_validate` is a feature, not a profile.** Every test invocation passes
  `--features pse-relations/force-validate` explicitly. `just test` does this for you.
- **`panic = "unwind"`** in every profile. PyO3 turns unwinds into Python exceptions and
  the Ipopt callbacks `catch_unwind`; `abort` would take the interpreter down.
- **Never `target-cpu=native`.** It lets LLVM contract `a*b + c` into an FMA and changes
  floating-point results between your machine and CI.
- **Every `pub enum *Error` derives `thiserror::Error` and `miette::Diagnostic`** with a
  blueprint §23.2 code. No `anyhow` in `crates/*`.
- **`typing.Any` is banned in Python**, as is `from __future__ import annotations` under
  `python/pse` (PEP 563 breaks the import-time `Any` check in `pse.governance`).
- **Exactly one of `unit`/`component`/`integration`/`performance`** per Python test;
  `conftest.py` hard-fails at collection otherwise.
- **The parity suite fails, it never skips.** A missing solver or a wrong IDAES version
  is a failure, not a skip.

## Gotchas that have already cost time here

Each of these is a real incident, not a hypothetical.

- **Pinning the umbrella crate does not pin the family.** `arrow = "=59.3.0"` binds one
  direct dependency; `arrow-schema`, `arrow-array` and friends can still resolve to a
  different version through a transitive path, and `cargo tree -d` cannot see it because
  each package name appears once. `just family-check` exists for exactly this. Context:
  `docs/capability-maps/evidence/README.md`.
- **A rustdoc extraction whose scratch project was deleted made every `[rustdoc:]` marker
  in the capability maps unreproducible.** The evidence lockfiles are committed for that
  reason; regenerating evidence without them proves nothing. `just evidence-regen` rebuilds
  from the committed locks.
- **PyPI `pse` is taken.** The distribution is `pse-arrow`; the import name stays `pse`.
  Do not "fix" one to match the other.
- **Tooling that hard-coded `/home/paul/...` broke on the first move.** Resolve the repo
  root from `git rev-parse --show-toplevel` (shell) or `Path(__file__).parents[N]` (Python).
  Never write an absolute home path into a committed file.
- **GitHub Actions has no YAML anchors** — duplicate the block and say why in a comment.
  **`yaml.safe_load` accepts duplicate keys silently** (last wins), so a plain parse check
  will not catch a second `if:` that disabled the first. **`windows-*` runners default to
  PowerShell**; set `shell: bash` or the script dies with a `ParserError`.
- **`SessionConfig::set_str` panics on an invalid key and `Field::extension_type()` panics
  on a missing or invalid extension.** Both are banned in `clippy.toml`; use the typed
  config path and `try_extension_type`.
- **uv workspaces enforce a single `requires-python`** (the intersection across members),
  which is why the IDAES parity set is a *dependency group* with an environment marker
  rather than a workspace member. Do not "simplify" it into one.

## Verifying work — what each command actually proves

| Command | Proves | Does not prove |
|---|---|---|
| `just ci-fast` | the workspace formats, compiles, lints clean and its tests and doctests pass | nothing about Python, features, policy or docs |
| `just test` | Rust tests pass with Arrow `force_validate` on | nothing about doctests, other profiles, or release-only paths |
| `just codegen-check` | the generated trees have no unstaged/staged/untracked changes (phase 0) | no regeneration equivalence while generators remain deferred |
| `just family-check` | one resolved version per dependency family, equal to the pins | nothing about whether that version behaves as documented |
| `just governance` | the workspace-level invariants hold (pins, crates registered, MSRV, unsafe allowlist, error taxonomy) | nothing about runtime behaviour |
| `just quality` | Python format/lint/types/import boundaries and repo config are clean | that the code works |
| `just policy` | no known advisory, no banned or duplicate crate, no disallowed licence | nothing about code you wrote |
| `just parity` | the exercised parity checks pass against `idaes-pse==2.12.0` | nothing about cases not exercised, or other IDAES versions |
| `just docs` | the book builds and its internal links resolve | nothing about whether the prose is true |
| `just adr-lint` | ADR front matter, numbering, supersession and register rows are well-formed | nothing about whether the decisions are good |

**Never report that tests pass without naming the command, the mode, and the baseline.**
"34 failed" is not information until the baseline is known — and here the baseline is zero.

## Decisions and documentation

**When an ADR is required:**

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; adds, drops or majors a dependency family; changes the hashing contract, the Python boundary contract, metadata conventions or the commit contract; any SHOULD deviation; governance changes | ADR **and** a design review (`needs-review`; verdict Accept or Accept-scoped before `status: accepted`) |
| New relation family, pass, kernel contract or backend binding *within* an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | ADR (short); review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, docs wording, patch bumps inside a pinned family, tooling | Neither. An ordinary PR with the evidence field filled |

An ADR enters or changes status only in a PR labeled `adr` and titled `adr: ADR-NNNN
<title>`; the same PR (or a named follow-up `design:` PR) amends the blueprint with a
revision row. `.codex/skills/adr/SKILL.md` explains the §H front-matter fields and when a
record is required.

**Doc conventions.** New files are lowercase kebab-case (exceptions: the three
UPPER_SNAKE principle files, the root governance files, and the design-review skill's
`design_review_{slug}_{date}.md`). YAML front matter on ADRs, plans, capability maps and
the authoritative design. Citations are `blueprint §14.3`, `ADR-0020`, `DM-33`, `G4` —
never line numbers. Generated docs carry `<!-- @generated by pse-schema; do not edit -->`.
Plans go in `docs/plans/`, never in a home directory.

## Off-limits

- `docs/authoritative_design/**` — the blueprint is amended by a `design:` PR that carries
  a revision row, not by an edit in passing.
- `docs/adr/NNNN-*.md` whose front matter `status:` is not `proposed` — accepted records
  are immutable; supersede them (`just adr-supersede`) instead.
- The generated paths in prime directive 2, `external/`, `build/`, `target/`.

A `PreToolUse` hook blocks writes to all of these. When a change to the blueprint or an
accepted ADR is genuinely the work, set `PSE_DESIGN_EDIT=1` for that session and say in
the PR why. The escape exists so the guard can stay strict; using it silently defeats it.

## Agent runtimes

- `AGENTS.md` is the shared authority. `CLAUDE.md` starts with `@AGENTS.md` and
  describes Claude-specific behavior. Codex reads this file directly.
- Skills are canonical in `.codex/skills/`; `.claude/skills` and `.agents/skills`
  expose the same content. `just agent-config-sync` materializes them on Windows
  when symlinks are unavailable.
- Role instructions are canonical in `.claude/agents/`. Native Codex TOML roles
  in `.codex/agents/` are generated by `just agent-config-sync`; never edit them
  directly. Codex roles inherit the user's model; read-only roles remain read-only.
- Both runtimes use `scripts/agent-hooks.py` for startup diagnostics, protected
  edit checks and per-file formatting. `.codex/hooks.json` and `.claude/settings.json`
  contain the runtime wiring. Hooks guard supported file-edit tools; they are not a
  sandbox for arbitrary shell commands or tools. Follow the same protection policy
  for all other actions. Existing session authorization remains authoritative.
- `just lint-agents` checks references, aliases, native role drift and hook wiring.
  `just setup-test` exercises the guard behavior in disposable fixtures.

Before editing a matching scope, read the applicable shared rule file. Claude also
loads these through its native path rules; Codex follows this routing table:

| Scope | Shared rule |
|---|---|
| Rust source and Cargo manifests | `.claude/rules/rust.md` |
| Python source and Python configuration | `.claude/rules/python.md` |
| Documentation | `.claude/rules/docs.md` |
| ADRs and plans | `.claude/rules/decisions.md` |
| Generated paths | `.claude/rules/generated.md` |
| GitHub configuration and workflows | `.claude/rules/ci.md` |

In phase 0, model generators and API-reference doc lint are deferred. A clean
`just codegen-check` proves generated-tree hygiene, not regeneration equivalence.
Parity currently exercises environment preflight, not numerical IDAES equivalence.
