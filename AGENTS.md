# AGENTS.md

Instructions for any coding agent working in this repository. Canonical for both
Claude Code (via `CLAUDE.md`, which imports this file) and Codex (which reads this
file directly).

## What this repository is

`pse-arrow` is a process systems engineering core in Rust. Typed process definitions
in authored relations remain the model authority. Library-owned mathematics,
thermodynamics and native solvers replace custom MathIR/evaluation and production
Pyomo integration. Arrow/DataFusion/Delta retain useful data-boundary, relational and
storage roles. Plan 14 M00–M21 implements typed compilation, FeOS properties, native
solvers, public workflows, dynamics/fitting and executable acceptance tooling.
M22 closes the local Linux scientific/runtime/case-cost scope; see its execution packet
for evidence and the maintainer-approved strict Clippy/release exclusions.

It is a **clean-room re-implementation** of core IDAES-PSE capabilities, parity-tested
against `idaes-pse==2.12.0`. **Not affiliated with IDAES.** Read `external/idaes-pse`
for *behaviour*; never copy its code, docstrings or comments. The parity harness is the
only sanctioned coupling, and the enumerations preserved by name are listed in the
blueprint (§6.14). See `docs/relationship-to-idaes.md`.

Crates are `pse-*` under `crates/`. The Python package is imported as `pse` and lives in
`python/pse`; the PyPI distribution is `pse-arrow`.

## Current implementation direction

[Plan 16 P00–P04](docs/plans/16-p00-p04-execution.md) extends the implemented
foundation with selected composition admission, exact powers, shared execution tags,
scoped identities and explicit material/FeOS/reaction bindings. P05–P18 remain future
work. The packet records the supported boundary and scoped verification; it does not
replace the historical M22 qualification or authorize full Plan 16 acceptance.

[Plan 14](docs/plans/14-library-owned-process-simulator.md) supplies the implemented
baseline. M00–M21 and their approved hard-cut deletions are implemented; see the
[foundation contract](docs/plans/14-math-foundation-contract.md) and
[execution inventory](docs/plans/14-execution-inventory.md). The
[M22 packet](docs/plans/14-m22-execution.md) records local Linux qualification,
measurements, independent reviews and the supported boundary under ADR-0087.
Qualification records executed evidence and authenticated retained origins; no M21
source seal is required. The scoped closure does not claim strict Clippy, remote CI,
release or other-platform acceptance.
It supersedes Plan 13's execution scope: no unfinished package, acceptance ID,
campaign or old source seal is inherited automatically. Retain a graph, Salsa,
publication or resource mechanism only when evidence establishes its role in the new
target. Plan 13's W19/W20 and older incomplete receipts remain historical outcomes.
Build the target directly and remove replaced code/callers/tests without compatibility
APIs or a second production compiler. Follow the execution rhythm below; full
qualification occurs once at M22. Correctness tests retain explicit force-validation.
Full library eligibility remains in force.

## Execution rhythm: pivot first, qualify at the end of the plan

The work is moving the codebase onto the target design and deleting what it replaces.
Spend attention there. A plan is qualified completely, but **once, at its end** — not
after every change. Plans here are large; per-change polish costs more than it catches.

**While implementing a plan:**

- Compile what you touched: `just check-package <pkg>`, or `just check` for cross-crate work.
- Run, or write, the targeted unit tests that show the new behaviour:
  `just unit-package <pkg> <filter>`. A new mechanism gets its tests in the same change.
- Run `just codegen` when a generator or registry declaration changes — regeneration is
  part of the change, not polish. Run `just family-check` only when a pinned-family
  dependency moves.
- **Delete legacy code as soon as it is provably replaced** — the replacement's targeted
  tests pass and every caller has moved. Remove the old mechanism, its callers, its tests
  and its fixtures in the same change. Do not port tests for a deleted mechanism, keep a
  shim, or retain a path "as evidence".

**At the end of the plan, once:** integration, component, solver and Python journeys;
performance campaigns; formatting and lint (`fmt-rust-check`, `clippy-*`, `lint-typos`,
`lint-license`, `quality`); `governance`; `codegen-check`; `adr-lint`/`adr-index`;
`lint-agents`; `docs`; architecture manifest; evidence-labelled
write-ups and the plan Outcome. Fix what they find in one pass, then rerun the affected
gates until the whole set is green against the zero baseline.

**Don't, mid-plan:**

- Run `just doctor` — the SessionStart hook already reports it. Run it only when that
  report shows a failure or a command fails in an environment-shaped way.
- Run format checks. The post-edit hook formats every Rust, Python and TOML file you edit.
- Rerun static checks after documentation-only edits.
- Write per-command receipts, numbered rerun logs or "documentation checkpoint"
  validations into plan documents. A checkpoint records state, decisions and next steps.

**Reporting mid-plan:** what changed, what was deleted, which tests exercised it, and
what is next. Evidence labels and baseline-framed failure counts belong in the plan
Outcome, PR descriptions and ADRs (prime directives 4 and 7).

## Start here

```bash
just --list        # the command surface
just bootstrap     # only if the session's doctor report shows a failure -- idempotent
```

`just --list` is the contract. Prefer a recipe over an ad hoc command: recipes own the
feature flags, profiles, report paths and tool paths, so those can change without you
re-learning them. If no recipe fits, say so rather than improvising a long command line.

`direnv` activates the environment on `cd` (run `direnv allow` once). It never installs
or downloads — `just bootstrap` does that, visibly.

For Python/native development, run `just py-sync` to refresh the editable extension
using the dev profile, then targeted `just py-test`; `just quality` is an end-of-plan
check. Use
`just parity-container` when solver-backed parity is needed. Full wheel/sdist builds
are manual (`just wheels-check <ref>`) or part of a release; ordinary PRs do not wait
for distribution builds. CI uses editable development builds for Python and parity.
These local recipes are available on demand, not prerequisites for committing or
pushing. Git hooks run static checks only: no native compilation, Python tests,
parity or distribution builds. GitHub runs the required suites before merging.

## Prime directives

1. **The baseline is zero.** No quality baselines exist and none will be introduced. A
   lint finding, a failing test, a warning: the target is none, not "no worse than before".
   Zero is the required state when a plan closes and before merge — not after every edit
   (see *Execution rhythm*).
2. **Never edit a generated directory. Fix the generator, then `just codegen`.** The
   generated paths are `docs/generated/`, `crates/*/src/generated/`,
   `crates/pse-ipopt-sys/src/bindings.rs`, and `python/pse/contracts/`. `just codegen-check`
   is what CI runs; a hand edit there is a red diff, not a fix.
3. **One authoritative declaration per meaning.** Version pins live once in `Cargo.toml`
   (`[workspace.dependencies]`) and once in `pyproject.toml`. A schema is declared in the
   registry, never inferred. If you find yourself writing a fact down twice, one of the
   two is wrong and nothing will tell you which.
4. **Label every claim with the evidence vocabulary** (design principles §D): *Proposed*,
   *Interface-checked*, *Implemented*, *Tested*, *Measured*, *Formally established*. The
   labels are mandatory in PR descriptions, ADR `evidence:` fields and a plan's
   Verification and Outcome sections when it closes; interim notes and progress reports
   do not need them. `Tested` and `Measured` must name the test or benchmark and its
   conditions.
5. **An ADR comes before the change**, not after, when the change alters a D1–D14
   decision, adds or removes a crate, or majors one of the four pinned families. See
   "When an ADR is required" below. **Adding a third-party dependency needs none of it** —
   no library and no licence is refused during phases 0–1
   ([dependency policy](docs/dev/dependency-policy.md), ADR-0066).
6. **Use the tool from `.venv` and the pinned toolchain, never `$PATH`.** A stale global
   `ruff` or a nightly `cargo` silently produces a different result than CI.
7. **Report a failure count with its baseline and the command.** Never report that tests
   pass without naming the command, the mode, and the baseline. Mid-plan, one line per
   targeted check you ran is enough; the full accounting belongs to plan close.

## Repository map

| Path | What it is | How to treat it |
|---|---|---|
| `crates/` | Workspace `pse-*` crates declared in `Cargo.toml` | Ours; see `.claude/rules/rust.md` |
| `xtask/` | Everything that needs Rust APIs, JSON or cross-platform behaviour | Logic lives here, the justfile is the surface |
| `tests/` | Workspace test crates: `governance`, `engine`, `conformance`, `lifecycle`, `structural` | `tests/fixtures/` contains source inputs, not a crate |
| `benches/` | Criterion benchmarks (`pse-benches`) | No timing gate in CI; `just bench-smoke` only runs them |
| `python/pse/` | The Python package (import name `pse`) | `python/pse/contracts/` is GENERATED |
| `docs/authoritative_design/` | **The blueprint.** Off-limits to edits | Read constantly, write never |
| `docs/adr/` | Decision records, immutable once accepted | `just adr-new`; index via `just adr-index` |
| `docs/plans/` | Implementation plans, living until done | `just plan <slug>` |
| `docs/capability-maps/` | Pinned third-party API maps + their evidence | `just lib-outline <file>` first; they are large |
| `docs/design_review/` | The layered design standard (`design_principles/standard.toml`: core principles, process-simulator profile, pse-arrow binding) and the reviews written against it | The `design-review` and `design-review-process-simulator` skills' output contract |
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
- **`pyproject.toml`** — every runtime Python library `==`-pinned; tools that only
  execute (ruff, pyrefly, pytest, …) carry floors under `[dependency-groups]` and run
  at whatever `uv.lock` resolved, never installed ad hoc. No release of `uv` itself is
  required.
- **`docs/capability-maps/`** — what the pinned libraries actually expose, with evidence.
  `just lib-outline docs/capability-maps/arrow-rust.md` before reading one.
- **`docs/design_review/design_principles/standard.toml`** — the layered design standard
  used by design reviews: repo-agnostic core principles (`DP-nn`, gates `G1`–`G8`), the
  process-simulator profile (`PS-nn`, `PS-G1`–`PS-G3`) and the pse-arrow binding.
- **`docs/dev/dependency-policy.md`** — what you may depend on and under what licence.
  Short answer: anything. Read it before assuming a library is off-limits.

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
- **Every `pub enum *Error` derives `thiserror::Error` and implements `miette::Diagnostic`** with a
  blueprint §23.2 code. No `anyhow` in `crates/*`.
- **`typing.Any` is banned in Python**, as is `from __future__ import annotations` under
  `python/pse` (PEP 563 obscures types from the explicit and dynamic-hook checks in `pse.governance`).
- **No library or licence is refused** through phases 0–1. Admission is advisory; pinning,
  the one type universe and the import boundaries are not (§3.3.2, ADR-0066).
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

During implementation the inner loop is `just check-package`/`just check`, targeted
`just unit-package` and `just codegen` (see *Execution rhythm*). The table is the
end-of-plan qualification surface.

| Command | Run | Proves | Does not prove |
|---|---|---|---|
| `just ci-fast` | plan close | the workspace formats, compiles, lints clean and its tests and doctests pass | nothing about Python, features, policy or docs |
| `just test` | plan close | Rust tests pass with Arrow `force_validate` on | nothing about doctests, other profiles, or release-only paths |
| `just codegen-check` | plan close | every generated tree equals a fresh regeneration, with no extra or untracked generated files (ADR-0051) | nothing about runtime behavior of the generated interfaces |
| `just family-check` | when a pinned-family dependency moves | one resolved version per dependency family, equal to the pins | nothing about whether that version behaves as documented |
| `just governance` | plan close | the workspace-level invariants hold (pins, crates registered, MSRV, unsafe allowlist, error taxonomy) | nothing about runtime behaviour |
| `just quality` | plan close | Python format/lint/types/import boundaries and repo config are clean | that the code works |
| `just deps-report` | on demand | what is in the dependency graph and under what licences; **advisory, always exits 0** | nothing — it refuses nothing and blocks nothing |
| `just policy` | on demand | the same checks, strictly: no known advisory, no disallowed licence. Opt-in, not in `ci-pr` | nothing about code you wrote, and nothing you are obliged to act on yet (register R-31) |
| `just parity` | plan close, when parity is in scope | the exercised parity checks pass against `idaes-pse==2.12.0` | nothing about cases not exercised, or other IDAES versions |
| `just docs` | plan close | the book builds and its internal links resolve | nothing about whether the prose is true |
| `just adr-lint` | ADR PR / plan close | ADR front matter, numbering, supersession and register rows are well-formed | nothing about whether the decisions are good |

**Never report that tests pass without naming the command, the mode, and the baseline.**
"34 failed" is not information until the baseline is known — and here the baseline is zero.

## Decisions and documentation

**When an ADR is required:**

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; majors one of the four pinned families (arrow, datafusion, object_store, pyo3); changes the hashing contract, the Python boundary contract, metadata conventions or the commit contract; any SHOULD deviation; governance changes | ADR **and** a design review (`needs-review`; verdict Accept or Accept-scoped before `status: accepted`) |
| New relation family, pass, kernel contract or backend binding *within* an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | ADR (short); review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, docs wording, patch bumps inside a pinned family, adding, removing or upgrading a third-party dependency, tooling | Neither. An ordinary PR with the evidence field filled |

An ADR enters or changes status only in a PR labeled `adr` and titled `adr: ADR-NNNN
<title>`; the same PR (or a named follow-up `design:` PR) amends the blueprint with a
revision row. `.codex/skills/adr/SKILL.md` explains the §H front-matter fields and when a
record is required.

**Doc conventions.** New files are lowercase kebab-case (exceptions: the superseded
UPPER_SNAKE and snake_case principle files, the root governance files, and the design-review skill's
`design_review_{slug}_{date}.md`). YAML front matter on ADRs, plans, capability maps and
the authoritative design. Citations are `blueprint §14.3`, `ADR-0020`, `DP-09`, `PS-10`, `G4` —
never line numbers. Generated docs carry `<!-- @generated by pse-schema; do not edit -->`.
Plans go in `docs/plans/`, never in a home directory.

## Off-limits

- `docs/authoritative_design/**` — the blueprint is amended by a `design:` PR that carries
  a revision row, not by an edit in passing.
- `docs/adr/NNNN-*.md` whose front matter `status:` is not `proposed` — accepted records
  are immutable; supersede them (`just adr-supersede`) instead.
- The generated paths in prime directive 2, `external/`, `build/`, `target/`.

A `PreToolUse` hook blocks writes to all of these, and `.claude/settings.json` denies the
generated paths, `build/`, `target/`, `external/` and `.git/` again with `Edit(/…)` rules. The
rules are anchored at the repository root with a leading `/`: `./path` resolves against the
session's current directory, and as a single-segment deny it matches the same name at any depth,
which refused every skill's own `build/`. Claude Code consults only `Edit` and `Read` path rules
(`Edit` covers writes), so `Write(...)` path rules do nothing; `just lint-agents` rejects both
mistakes. The hook's scope is the working copy plus the agent runtime's own directories: it
allows `~/.claude` (or `CLAUDE_CONFIG_DIR`), `~/.codex`, the session temp directory and
anything named in `PSE_AGENT_WRITABLE`, and refuses every other path outside the working copy
so a stray edit cannot land in another checkout. That
allowance is for runtime state — memory, scratch files, the runtime's own configuration.
Project state still belongs here: plans go in `docs/plans/`, never in a private home
directory. When a change to the blueprint or an accepted ADR is genuinely the work, set
`PSE_DESIGN_EDIT=1` for that session and say in the PR why. The escape exists so the guard
can stay strict; using it silently defeats it.

## Personal-project checkout workflow

This is a personal project. Use the existing checkout on `main` for ordinary
development and GitHub updates. Do not routinely create or switch branches,
worktrees or separate checkouts for a task.

Use separate branches or worktrees only when genuinely concurrent agent editing
requires isolation. Coordinate file ownership first and preserve other agents'
uncommitted changes. Keep checkout and Cargo target paths stable to retain build
artifacts and compiler-cache reuse; the Plan 15 second-worktree experiment produced
no Rust cache hits after those paths changed.

## Agent runtimes

- `AGENTS.md` is the shared authority. `CLAUDE.md` starts with `@AGENTS.md` and
  describes Claude-specific behavior. Codex reads this file directly.
- Skills are canonical in `.codex/skills/`; `.claude/skills` and `.agents/skills`
  expose the same content. `just agent-config-sync` materializes them on Windows
  when symlinks are unavailable. Only the repository-process skills (`adr`,
  `design-review`) are tracked; library capability skills are local-only and gitignored
  until they move to a repository of their own. Skills committed before that rule stay
  tracked.
- Role instructions are canonical in `.claude/agents/`. Native Codex TOML roles
  in `.codex/agents/` are generated by `just agent-config-sync`; never edit them
  directly. Codex roles inherit the user's model; read-only roles remain read-only.
- Both runtimes use `scripts/agent-hooks.py` for startup diagnostics, protected
  edit checks and per-file formatting. `.codex/hooks.json` and `.claude/settings.json`
  contain the runtime wiring. Hooks guard supported file-edit tools; they are not a
  sandbox for arbitrary shell commands or tools. Follow the same protection policy
  for all other actions. Existing session authorization remains authoritative.
- Development runs without approval prompts. `.claude/settings.json` allows the tools
  outright and keeps one specific `deny` list for the protected paths; a session that
  wants no prompt at all sets `permissions.defaultMode` in the gitignored
  `.claude/settings.local.json`. In that mode `deny` rules and the `PreToolUse` hook are
  still enforced but `ask` rules are not, so a gate that must hold belongs in `deny` or
  in `scripts/agent-hooks.py` -- never in `ask`. Recipes that reach outside the working
  copy (`release`, `solver-image`, `labels-sync`, `gh-setup`, `solver-pin-update`) keep
  their `just` confirmation, which is not a prompt: with no terminal it fails rather than
  asking, so run one deliberately with `just --yes <recipe>`. Permission to act is not an
  instruction to act -- commit, push and publish when the work calls for it.
- `just lint-agents` checks references, aliases, native role drift, hook wiring and that
  every file-path deny rule is an anchored `Edit`/`Read` rule.
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

Wave 1 supplies registry model generators and regeneration equivalence under ADR-0051.
API-reference doc lint remains deferred. Parity covers the environment and the explicitly
exercised compatibility names; it does not establish numerical IDAES equivalence.
