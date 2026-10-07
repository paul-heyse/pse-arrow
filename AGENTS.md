# AGENTS.md

Instructions for any coding agent working in this repository. Canonical for both
Claude Code (via `CLAUDE.md`, which imports this file) and Codex (which reads this
file directly). Environment, runtime and tooling mechanics are in
[the agent environment guide](docs/dev/agent-environment.md); read it when a command,
hook, permission, worktree or capability question comes up.

## What this repository is

`pse-arrow` is a process systems engineering core in Rust. Typed process definitions
in authored relations are the model authority. Library-owned mathematics (Symbolica/
Numerica) and class-specific native solvers execute authored scientific knowledge;
Arrow/DataFusion/Delta serve data-boundary, relational and publication roles. Rust and
Python workflows, dynamics and fitting are implemented and qualified locally on Linux;
the architecture sections record the supported scope and its limits.

It is a **clean-room re-implementation** of core IDAES-PSE capabilities. The current parity
reference is `idaes-pse==2.13.0`. **Not affiliated with IDAES.** Read `external/idaes-pse`
for *behaviour*; never copy its code, docstrings or comments. The parity harness is the
only sanctioned coupling; scientific vocabularies have local meaning rather than a
historical spelling obligation (blueprint §6.14, ADR-0160). See `docs/relationship-to-idaes.md`.

Crates are `pse-*` under `crates/`. The Python package is imported as `pse` and lives in
`python/pse`; the PyPI distribution is `pse-arrow`.

## Current implementation direction

The [architecture](docs/authoritative_design/README.md) describes the current system:
start with its [reading guide](docs/authoritative_design/sections/reading-guide.md) and
[overview](docs/authoritative_design/sections/architecture-overview.md). Completed plans are
retired to Git history (ADR-0096); their enduring meaning lives in those sections and the
retained ADRs, and the current qualification basis is summarized in
[§24.2](docs/authoritative_design/sections/operations-and-validation.md). Retired plans,
packets and reviews are not a backlog and authorize nothing.

[Current work](docs/plans/README.md) links the owning plans, packet status and remaining
decision work. New work starts only when the maintainer authorizes it. Build a target
directly and remove replaced code/callers/tests without compatibility APIs or a second
production path. Correctness tests retain explicit force-validation. Full library
eligibility remains in force.

## Execution rhythm: functional scope first; static checks at scope end

The work is moving the codebase onto the target design and deleting what it replaces.
Spend attention there. There are no end-of-turn hooks (ADR-0161); three bundles keep going
after a failure and list what failed:

- `just turn-end` (ADR index, `fmt` with ruff's safe auto-fixes): the root agent runs it at the
  end of a turn that changed files; subagents don't.
- `just ready` (skill sync, `doctor`): after a dependency, toolchain or skill-selection change,
  or when a command fails in an environment-shaped way.
- `just hygiene` (type errors, clippy, lint, codegen drift): once all functional scope in the
  plan is implemented; fix what fails. Integrated testing happens once, at the same point.

None of these is a commit, push or merge prerequisite. Formatting and lint fixes mid-work
rewrite lines you did not author, so every agent would have to re-assess changes that are not
theirs.

**While implementing a plan** — compile checks and targeted tests only:

- Compile what you touched: `just check-package <pkg>` (extra `-p` and cargo flags pass
  through), or `just check` for cross-crate work.
- Run, or write, the targeted unit tests that show the new behaviour:
  `just unit-package <pkg> <word|filterset>`; `just affected` previews the tests a change can
  reach. A new mechanism gets its tests in the same change.
- Run `just codegen` when a generator or registry declaration changes — regeneration is
  part of the change, not polish. `just hygiene` confirms it at scope end (`codegen-check`).
- **Delete legacy code as soon as it is provably replaced** — the replacement's targeted
  tests pass and every caller has moved. Remove the old mechanism, its callers, its tests
  and its fixtures in the same change. Do not port tests for a deleted mechanism, keep a
  shim, or retain a path "as evidence".

**After all functional scope is implemented:** select the relevant integration, component,
solver and Python journeys, performance campaigns, `just hygiene` (fix what fails and re-run
the failing recipe) and the manual checks (`just governance`, `just docs`, the native and
powerset lints). Report the commands, scope and results. The maintainer may also request this
comprehensive qualification at any other time.

**Don't, mid-plan:** run `just doctor`/`just ready` unless the environment changed or a command
failed in an environment-shaped way; run formatters, linters, type checks or other
`just hygiene` checks (`just clippy`, `ruff`, `just quality`, `just ci-fast`) or integration
suites before scope end; rerun static checks after documentation-only edits; write
per-command receipts, numbered rerun logs or "documentation checkpoint" validations into
plans. A checkpoint records state, decisions and next steps.

**Reporting mid-plan:** what changed, what was deleted, which tests exercised it, and
what is next. Evidence labels and baseline-framed failure counts belong in the plan
Outcome, PR descriptions and ADRs (prime directives 4 and 7).

## Start here

```bash
just --list                 # recipes by cost tier; --usage <recipe> for parameters
scripts/pse-env --explain   # what environment and placement a command would get
just bootstrap              # only if `just ready` reports a failure -- idempotent
```

Recipes are shortcuts, not gates: they carry feature flags, profiles, report paths and
tool paths, so use one when it fits. Bare tools are first-class too: `scripts/pse-env --
<command>` supplies the checkout environment (licence, venv, compiler cache, a capped scope)
under any shell, `--native` adds the linked solver environment, and `cargo c` / `cargo t`
carry force-validation. Every recipe runs through `scripts/pse-env`; direnv and Claude
sessions load the same environment, and Codex commands prefix `scripts/pse-env --`.

For Python, `just py-sync` refreshes the editable dev extension (`just py-sync-native` the
linked one); `just py-unit` routes to the right environment for whichever is installed, and
`just py-test` runs unit and component tests against the canonical store at
`$PSE_SURREAL_STATE`. `just parity` runs the IDAES comparisons on this machine. Full wheel
builds are manual (`just wheels-check <ref>`). Git hooks are not installed by bootstrap, and
GitHub check workflows run only when manually dispatched.

## Prime directives

1. **The baseline is zero.** No quality baselines exist and none will be introduced. A
   lint finding, a failing test, a warning: the target is none, not "no worse than before".
   When a check is run, report its actual result against the zero target; it does not
   become a commit, push or merge gate (see *Execution rhythm*).
2. **Never edit a generated directory. Fix the generator, then `just codegen`.** The
   generated paths are `docs/generated/`, `crates/*/src/generated/` (the operational
   store's `crates/pse-operations/src/generated/` included), `crates/pse-operations-queries/`,
   `crates/pse-ipopt-sys/src/bindings.rs`, `python/pse/contracts/` and
   `python/pse/_native.pyi`, plus the cargo-hakari section of
   `crates/pse-workspace-hack/Cargo.toml` and each member's `pse-workspace-hack` line
   (`.config/hakari.toml` is their source). `just codegen-check` covers every generated
   output; a hand edit there is a red diff, not a fix.
3. **One authoritative declaration per meaning.** A dependency is declared once, in
   `Cargo.toml` (`[workspace.dependencies]`) or `pyproject.toml`, and a pin's reason once
   beside it (`[workspace.metadata.pse.pins]`). A schema is declared in the
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
   `ruff` or a `cargo +<other>` silently produces a different result than CI. The pinned
   toolchain is the dated nightly in `rust-toolchain.toml`, its only declaration
   (ADR-0122); rustup selects it for every cargo run in a checkout. Never override it
   with `+toolchain` or `RUSTUP_TOOLCHAIN`: stable Cargo ignores the `[unstable]`
   feature unification, and another nightly compiles a second copy of everything.
7. **Report a failure count with its baseline and the command.** Never report that tests
   pass without naming the command, the mode, and the baseline. Mid-plan, one line per
   targeted check you ran is enough; comprehensive accounting belongs to a manually
   requested qualification report.

## Authority and task routes

Start with the [documentation task routes](docs/README.md), then the relevant architecture
owner and adjacent consumers. Read [current work](docs/plans/README.md) and its active packet
only when the task depends on that baseline; delegated workers use the brief and relevant
owners rather than repeating the root's general orientation. Instructions already available
in context need not be reread; if unavailable, load them before acting. Follow discovered
dependencies beyond initial pointers.

The architecture sections own contracts; ADRs own rationale; reviews supply evidence.
Cite stable section IDs (for example blueprint §14.3), never line numbers. Dependencies and
their pin reasons are declared in Cargo.toml and pyproject.toml; the layered review standard
is selected by `docs/design_review/design_principles/standard.toml`. Do not restate their
authority.

## Invariants

- **One type universe.** Exactly one resolved version of `arrow`, `parquet`,
  `object_store` and `datafusion`. Two majors make `downcast_ref` return `None` with no
  compile error — a silent failure that looks like a logic bug. `just family-check`.
- **`force_validate` is a feature, not a profile.** Every test invocation passes
  `--features pse-relations/force-validate` explicitly; the recipes and `cargo t` do this
  for you. The workspace-hack must never switch it on: hakari simulates all features, so the
  opt-in paths are excluded from its traversal in `.config/hakari.toml`.
- **One feature set per dependency, whatever `-p` selects** (ADR-0122). Cargo's workspace
  feature unification on the pinned nightly and `pse-workspace-hack` keep it; only opt-in
  features (`force-validate`, `native-solvers`) change a dependency's build.
- **`panic = "unwind"`** in every profile. PyO3 turns unwinds into Python exceptions and
  the Ipopt callbacks `catch_unwind`; `abort` would take the interpreter down.
- **Never `target-cpu=native`.** It lets LLVM contract `a*b + c` into an FMA and changes
  floating-point results between your machine and CI.
- **Every `pub enum *Error` derives `thiserror::Error` and implements `miette::Diagnostic`** with a
  blueprint §23.2 code. No `anyhow` in `crates/*`.
- **`typing.Any` is banned in Python.** `from __future__ import annotations` is allowed:
  `pse.governance` and the codec resolve postponed annotations before inspecting them.
- **No library or licence is refused** through phases 0–1. Admission is advisory; the one
  type universe, locked resolution, exact pins and the import boundaries are not
  (§3.3.2, ADR-0066, ADR-0165).
- **Exactly one of `unit`/`component`/`integration`/`performance`** per Python test;
  `conftest.py` hard-fails at collection otherwise.
- **The parity suite fails, it never skips.** A missing solver or a wrong IDAES version
  is a failure, not a skip.

## Dependencies

Every declared dependency is pinned exactly (`==x.y.z` in pyproject, `=x.y.z` in
`[workspace.dependencies]`, a git dependency by full `rev`) and the committed lockfiles hold
the rest (ADR-0165). Add or bump one when the work calls for it, with no ADR or approval: edit
that pin (or its family), run `just upgrade <package> …`, check the lock diff moved only what
you meant, run the tests it affects and name the move in the commit. No wholesale re-resolve
(`uv lock --upgrade`, bare `cargo update`) unless the operator asks; a family moves as a unit
and a family major needs an ADR. Holds go in `[workspace.metadata.pse.pins]` with their reason
([dependency policy](docs/dev/dependency-policy.md#pin-reasons)).

## Gotchas that have already cost time here

Each of these is a real incident, not a hypothetical.

- **Pinning the umbrella crate does not pin the family.** `arrow = "=59.3.0"` binds one
  direct dependency; `arrow-schema`, `arrow-array` and friends can still resolve to a
  different version through a transitive path, and `cargo tree -d` cannot see it.
  `just family-check` exists for exactly this.
- **A rustdoc extraction whose scratch project was deleted made every `[rustdoc:]` marker
  in the capability maps unreproducible.** The evidence lockfiles are committed for that
  reason; `just evidence-regen` rebuilds from them.
- **PyPI `pse` is taken.** The distribution is `pse-arrow`; the import name stays `pse`.
- **Tooling that hard-coded `/home/paul/...` broke on the first move.** Resolve the repo
  root from `git rev-parse --show-toplevel` (shell) or `Path(__file__).parents[N]` (Python).
- **GitHub Actions has no YAML anchors**; **`yaml.safe_load` accepts duplicate keys
  silently** (last wins); **`windows-*` runners default to PowerShell** (set `shell: bash`).
- **`SessionConfig::set_str` and `Field::extension_type()` panic** on invalid input. Both
  are banned in `clippy.toml`; use the typed config path and `try_extension_type`.
- **A heavy native run can exhaust the machine and take the editor down with it** (two
  editor-wide OOM kills on 2026-09-29). Every recipe command now runs in its own
  memory-capped scope in `pse.slice` (`PSE_MEMORY_MAX`, default 120G), so a runaway process
  is killed alone; run an ad hoc heavy command with `scripts/pse-env -- <command>`.
- **uv workspaces enforce a single `requires-python`**, which is why the IDAES parity set is
  a *dependency group* with an environment marker rather than a workspace member.

## Qualification reporting

The [qualification command guide](docs/dev/validation-assessment.md) explains what each
command establishes and its exclusions. Follow *Execution rhythm* for timing; documentation
and instruction changes do not require product qualification. Report command, mode, scope,
baseline and result; a targeted pass does not qualify unexercised product or scientific scope.
The failure baseline remains zero.

## Documentation context and publishing

Start at `docs/authoritative_design/README.md` or `docs/plans/README.md`, then read the
relevant contract and inspect the affected source. Markdown is the agent interface; `just docs`
builds the site (see `docs/dev/documentation.md`). A function-body edit normally needs no
architecture edit; update the owner when an enduring contract, rationale or workflow changes.
When work closes, move enduring meaning to its owner and retire the completed plan and its
resolved reviews; Git history is the archive, not a backlog (ADR-0096).

## Decisions and documentation

Use [design principles](docs/design_review/design_principles/core/design-principles.md) with
[Heuristics for Efficient Architecture](docs/design_review/design_principles/core/efficient-architecture-heuristics.md)
for consequential architectural and implementation choices, qualitatively and without
checklists, cost models or restarting settled reviews. Architectural reviews follow the
design-review skill within the binding's review periods (ADR-0129). Current finding
dispositions and packet status have one owning plan; reviews keep their original scope; ADR
acceptance is not implementation acceptance. See `.claude/rules/decisions.md` and blueprint §24.4.

**When an ADR is required:** the table in `.claude/rules/decisions.md` decides. An ADR and a
design review for D1–D14 changes, crate additions or removals, pinned-family majors, the
hashing/Python-boundary/metadata/commit contracts, SHOULD deviations and governance; a short
ADR for new relation families, passes, kernel contracts or bindings within an accepted
decision; neither for bug fixes, refactors within contracts, tests, docs wording,
dependency changes and tooling. ADRs change status only in a PR labeled `adr` and titled
`adr: ADR-NNNN <title>`, which also amends the architecture with a revision row in
`blueprint.md` (`.codex/skills/adr/SKILL.md`).

**Doc conventions.** New files are lowercase kebab-case (exceptions: the root governance
files and the design-review skill's `design_review_{slug}_{date}.md`). YAML front matter on
ADRs, plans, capability maps and the authoritative design. Citations are `blueprint §14.3`,
`ADR-0082`, `AP-07`, `DP-09`, `PS-10`, `G4` — never line numbers. Plans go in `docs/plans/`,
never in a home directory.

## Off-limits

- `docs/authoritative_design/**` — amended through the decision/design route with a revision
  row in `blueprint.md`, not by an edit in passing.
- `docs/adr/NNNN-*.md` whose front matter `status:` is not `proposed` — accepted records
  are immutable; supersede them (`just adr-supersede`) instead.
- The generated paths in prime directive 2, `external/`, `build/`, `target/`.

A `PreToolUse` hook (`scripts/agent-hooks.py`) blocks edits to all of these, and
`.claude/settings.json` denies them again; neither is a shell sandbox, so follow the same
policy for every other action. When a change to the architecture sections or an accepted ADR
is genuinely the work, authorize it visibly — create an untracked `.design-edit` file stating
why (remove it afterwards), or launch with `PSE_DESIGN_EDIT=1` — and say why in the commit.

## Personal-project checkout workflow

Use the existing checkout on `main` for ordinary development and GitHub updates. Use a
worktree only when concurrent agents genuinely need isolation — `just worktree <name>`
creates one correctly — coordinate file ownership, preserve other agents' uncommitted
changes, and remove fully merged worktrees. Never set `CARGO_TARGET_DIR` or point two
checkouts at one target directory (ADR-0122); the sccache cache is shared. Details:
[agent environment guide](docs/dev/agent-environment.md#worktrees).

## Agent coordination and workflow skills

Use subagents when independent work, context isolation, distinct capabilities or independent
judgment justify coordination and integration. Small or tightly coupled tasks may stay with the root.
Follow the [shared roles and coordination contract](.agents/roles/README.md), including its
explicit model and effort routing. The root agent owns design, integration and acceptance;
executors choose local implementation details within their assigned boundaries.

The [local process skills](.codex/skills/README.md) pair conversational preparation with action:
`plan-design-review` / `design-review`, `plan-creation` / `create-plan`, and
`plan-execution` / `execute-plan`. Planning companions produce adaptable approaches and do not
switch runtime modes. Read `docs/plans/README.md`, then the active plan or packet checkpoint
for the baseline and handoff. Existing plans own status and findings.

## Agent runtimes

`AGENTS.md` is the shared authority; `CLAUDE.md` imports it and adds Claude specifics.
Library skills are selected in `.config/library-skills.toml` (`just ready` links them), shared
roles live in [.agents/roles/](.agents/roles/README.md), and native adapters in
`.claude/agents/` and `.codex/agents/` own model and effort. Hooks, permissions, runtime
capability opt-in (MCP servers, plugins, LSP) and the environment each runtime receives are
described in the [agent environment guide](docs/dev/agent-environment.md). Permission to act is
not an instruction to act — commit, push and publish when the work calls for it.
`just lint-agents` checks references and wiring; `just setup-test` exercises the guards.

Before editing a matching scope, read the applicable shared rule file. Claude also
loads these through its native path rules; Codex follows this routing table:

| Scope | Shared rule |
|---|---|
| Rust source and Cargo manifests | `.claude/rules/rust.md` |
| Python source and Python configuration | `.claude/rules/python.md` |
| Recipes, scripts, build and agent configuration | `.claude/rules/tooling.md` |
| Documentation | `.claude/rules/docs.md` |
| ADRs and plans | `.claude/rules/decisions.md` |
| Generated paths | `.claude/rules/generated.md` |
| GitHub configuration and workflows | `.claude/rules/ci.md` |

Registry model generators and regeneration equivalence follow ADR-0031/0051. Parity covers the
environment and the explicitly selected scientific reference comparisons; it does not
establish numerical IDAES equivalence.
