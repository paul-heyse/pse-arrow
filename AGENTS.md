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
decision work. The active Plan58 ordinary command, activation and generic Cargo
correctness routes now use the independent native host. Prepare it explicitly with
`scripts/bootstrap-host`; help/read/launch never builds it. This wave supports configured
unbounded ordinary work, existing docs environments and explicit compiler/KLU preparation.
`--native=compiler,klu` selects that prepared profile; bare `--native`, `--store`, explicit finite
or scientific placement, and reference/timing admission currently refuse while their Plan58
owners are unfinished. Do not bypass that refusal through the retiring Python controllers.
The environment guide records supported placement and unfinished consumer cutovers. New work
starts only when the maintainer authorizes it. Build a target directly and remove replaced code/callers/tests without compatibility APIs or a second
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
tool paths, so use one when it fits. A composed recipe supports a mode across every stage
or rejects it before effects; use `just codegen-check` for comparison and zero-argument
`just codegen` for publication. Bare tools are first-class too: `scripts/pse-env --
<command>` supplies the checkout environment (licence, venv, compiler cache, a supervised scope)
under any shell, `--native` adds the linked solver environment, and `cargo c` / `cargo t`
carry force-validation. Workload recipes run through `scripts/pse-env`; the fixed read-only
`just activity` observer runs directly so unavailable workload admission or compiler setup
remains diagnosable. These exceptions accept observer options or the fixed capability server,
never an arbitrary command.
Direnv and Claude
sessions load the same environment, and Codex commands prefix `scripts/pse-env --`.

For Python, `just py-sync` refreshes the interactive editable extension (`just py-sync-native` the
linked one), without force-validation. `just py-unit` and `just py-test` first build the explicitly
validated correctness extension; `just py-sync-correctness` selects that build directly. `just py-unit`
routes to the right environment for whichever is installed, and
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
   generated paths are `docs/generated/`, `scripts/generated/`, `xtask/src/generated/`, `crates/*/src/generated/` (the operational
   store's `crates/pse-operations/src/generated/` included), `crates/pse-operations-queries/`,
   `crates/pse-ipopt-sys/src/bindings.rs`, `python/pse/contracts/`, `python/pse/generated/` and
   `python/pse/_native.pyi`. `just codegen-check` covers every generated
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
- **`force_validate` is a feature, not a profile.** Every actual Arrow-consuming
  correctness check/test closure explicitly activates validation, including normal/build
  and selected-root dev dependencies in target and host contexts (ADR-0170). Scope-aware
  routes verify Arrow leaf activation and refuse unknown coverage; pure closures do not
  select unrelated relation targets. Bare invocations carry the same obligation.
- **One feature set per dependency, whatever `-p` selects** (ADR-0122). Cargo's workspace
  feature unification on the supported pinned nightly keeps it; only opt-in
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
  editor-wide OOM kills on 2026-09-29). Workload recipe commands run in their own
  supervised scope, coordinated by `.config/agent-capacity.toml`. The maintainer-selected
  local `PSE_BUILD_MODE=unbounded` gives all ordinary work, including native/test orchestration,
  shared use of all 16 physical cores and 32 logical CPUs without an OS memory cap. Cargo
  defaults to 32 jobs; test runners default to 16 physical lanes. Do not introduce smaller
  CPU partitions, compiler-job limits, RAM budgets or serial execution for independent work
  on this workstation. Explicit operator selections and scientific pool, exclusion and
  timing/reference contracts retain their meanings; small test fixtures are not host caps.
  The environment guide owns this selection and its precedence.
  `PSE_MEMORY_MAX` explicitly selects a finite allocation and widens its ancestors coherently;
  run an ad hoc heavy command with `scripts/pse-env -- <command>`.
  **Do not pass `--resource-class compile` merely to identify a compiler workload:** that
  explicitly selects the finite 24 GiB profile, overriding the local uncapped default.
  When a composed command needs a compile hint, use `--recipe-default-class compile`.
  Recipes with their own `scripts/pse-env` entry can run directly; avoid an extra finite
  outer wrapper around `just clippy`, `just hygiene` or another compile aggregate.
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
resolved reviews; Git history is the archive, not a backlog (ADR-0096). Multiple plans may be
active concurrently. Use the [document lifecycle and scope routes](docs/dev/documentation.md#metadata-and-aggregate-scope)
for metadata, cross-plan content ownership and retirement preflight; derived views do not
replace native status owners or authorize work.

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

**Commit regularly in discrete, coherent changes during authorized implementation.**
Commit an integrated item when it has a clear purpose and reviewable boundary; do not wait
for the entire plan, scope-end qualification or the end of the conversation. Include its
necessary declarations, generated outputs, consumers and deletions together, and describe
the verification actually performed and any material remaining obligations. A commit does
not establish plan completion or authorize a push. The coordinator owns the shared index
and commits unless it explicitly delegates that responsibility. Stage only owned paths or
hunks, preserving unrelated and concurrent work.

## Agent coordination and workflow skills

Use subagents when another context, capability or perspective supplies useful progress, learning
or independent judgment worth briefing and integration. Shape assignments around coherent outcomes,
available inputs and shared decisions; components and role names are possible organizing aids.
Follow the [shared roles and coordination contract](.agents/roles/README.md), including its
explicit model and effort routing. The root remains accountable for design coherence, integration
and acceptance; workers exercise delegated judgment within accepted outcomes and contracts.

The [local process skills](.codex/skills/README.md) pair conversational preparation with action:
`plan-design-review` / `design-review`, `plan-functional-extension-review` /
`functional-extension-review`, `plan-creation` / `create-plan`, and
`plan-execution` / `execute-plan`. Planning companions produce adaptable approaches and do not
switch runtime modes. Read `docs/plans/README.md`, then the active plan or packet checkpoint
for the baseline and handoff. Existing plans own status and findings.

### Design audits and solution focus

Use design-principle-audit before a new diagnostic design-review engagement about production
or broad system scope. The [living diagnostic account](docs/design_review/audit/README.md) records the source baseline,
standard and coverage; it is not default context for ordinary production edits. The operator
selects holistic/unconstrained, library-targeted, principle-focused or combined solution review;
clarify an unspecified choice. Agents choose investigation and depth inside that focus.
Preparation and review can share the applicable refreshed audit.

For an explicitly requested test-scope audit or a diagnostic review focused on test architecture
and evidence, use design-principle-audit-tests and the
[separate test account](docs/design_review/test-audit/README.md). Load the
[test-assessment companion](docs/design_review/design_principles/core/test-assessment.md)
through `[core].test_assessment` alongside the existing standard and applicable profiles.
Test-only engagements do not initiate a broad production audit; mixed scope identifies the
applicable accounts under the binding. Each account retains its baseline and each finding one
primary home. Ordinary test maintenance does not trigger an audit or a full test campaign.

Use design-review-audit at requested or explicitly planned boundaries. Publish the
[net-change companion](docs/design_review/reviews/audits/README.md) and revise the principal
review in place, preserving the exact input in Git and attributing the revised judgment.
Use plan-completion-audit for claimed scope completion and correct that same owning plan,
including affected summaries, packages, findings and next work. Separate implementation,
required qualification and benefit evidence. These assessments do not recur per slice and do
not alone authorize production repairs. Existing plans keep scheduled disposition and handoff.

### Functional capability exploration

Use `plan-functional-extension-review` / `functional-extension-review` to explore useful product,
developer, operator or research capabilities within the operator's chosen outcome, workflow,
subsystem or ecosystem. Publish [opportunity reviews](docs/functional_reviews/README.md); no
diagnostic audit, existing caller or principle violation is required. Design principles shape
proposed realizations. Selected opportunities enter existing plan and decision routes without
automatic scope approval. Requested recommendation audits preserve this value lens and publish
their [functional companions](docs/functional_reviews/audits/README.md) plus an integrated review.

### Plan improvement

Use `plan-improvement-review` at requested or explicitly planned boundaries. Review the remaining
change footprint, including substantially affected supporting components, for design-principle
alignment, general enhancements and execution improvements. Judge recommendations for coherent
integration and total burden; their value can extend beyond the plan's functional target. Publish a
standalone [plan-improvement review](docs/plan_improvement_reviews/README.md); plan revision from
it is separately instructed through existing planning and decision routes. No broad audit
prerequisite, separate update skill or automatic adoption follows; formal review cadence remains
unchanged.

### Library research

Optional [library overviews](docs/libraries/README.md) give repository-agnostic capability
awareness. Deliberate leverage inquiries use the [library-leverage framework](docs/library_leverage/README.md):
`leverage-codebase-audit` identifies concrete opportunities, `leverage-library-assessment`
researches focused capability applicability and accumulates reusable types, and
`leverage-opportunity-review` compares complete options and develops bounded recommendations.
Use relevant base patterns and patterns collected in assessment outputs on the same footing;
all three stages can discover or refine useful additional framings. Retain useful types and
output patterns after their current matches retire; promotion into base guidance is manual.
Ordinary implementation and API questions do not initiate this workflow, and its reviews do not
replace the existing formal review cadence or authorize implementation.
Context7 is the first documentation lookup, followed by primary sources, upstream
tests or installed sources for consequential details. Native solver
work continues to load native-solver-libraries, the only selected active library skill.
Other central skills are [inactive references](.codex/skills/README.md#inactive-central-library-skill-references);
they are no longer discovered or required here. Leave older plan references for deliberate
revisits. Missing coverage or an empty search does not establish capability absence.

## Agent runtimes

`AGENTS.md` is the shared authority; `CLAUDE.md` imports it and adds Claude specifics.
Only native-solver-libraries is selected in `.config/library-skills.toml` (`just ready` links it); shared
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
