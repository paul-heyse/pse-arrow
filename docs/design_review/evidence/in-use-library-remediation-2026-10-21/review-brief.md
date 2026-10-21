# Common brief: in-use-library design review addressing audit DA-001–DA-032 (contributor)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Common brief given to every contributor: focus, scope rules and maintainer premises P1–P4 (P4 added mid-preparation).
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

You are a read-only contributor to a formal **design review** of pse-arrow, at `/home/paul/pse-arrow`.
The coordinator integrates contributions, and a separate principal reviewer writes the review. You
return text only.

## Focus (the maintainer's selection, 2026-10-21)
- **Library-targeted.** The search space is the libraries the repository *already uses*. Principle
  scope is broad: every audit finding DA-001–032.
- **Purpose: target.** We want the best-in-class design, and we are in the design phase. In the
  maintainer's words: "no compromises, absolutely best approach, look up the library features needed
  to do it."
  - Massive design changes are allowed.
  - **This audit-focused review improves functionality that already exists.** It does not add whole
    new categories of functionality. Added behaviour is in scope only where it completes, corrects or
    substantially improves an existing capability (for example typed refusals for existing persistence,
    or value binding for existing study modes). Judge where that line falls.
  - Within that scope, do not scope down for priority: recommend the best approach.
  - **Minor functionality improvements only where they are clear wins.** Do not propose major
    functionality additions that carry tradeoffs.
  - **Never propose a "compromised" approach that reverts or weakens another part of the system** to
    fix a finding. Examples: giving up lane parallelism, value rebind, typed outcomes or library-owned
    derivatives. A correction that cannot be made without such a regression should be reported as
    "remedy open", with the conflict stated.
- **The prize is library functionality we already have access to but do not use, or under-use,** that
  would improve existing behaviour. Find it, name it precisely, and show which obligations it would own
  completely.

### Search-space rule
In scope:
- direct workspace dependencies at their pins (`/home/paul/pse-arrow/Cargo.toml` `[workspace.dependencies]`, `pyproject.toml`);
- *all* cargo features of those crates, including features currently disabled;
- sibling crates of the same library family that are already resolved, e.g. Numerica/Graphica under
  Symbolica, or the SUNDIALS components.

Out of scope: new third-party libraries. If no in-use library can serve an obligation, record it as
a lead and do not recommend a new library.

## Fixed premises (maintainer, 2026-10-21; sourced from conversation)
- **P1. SurrealDB stays canonical.** It is the canonical state, commit and lifecycle authority. It
  is an *enabler*: durable studies run up to 16 worker case lanes coordinated through SurrealDB's
  guarded functions (claim / started / settle, generation fences, "cancellation wins"). The primary
  is activated on demand from the runtime.
  - Current design (source): `crates/pse-runtime/src/workflow/worker.rs` `serve` / `serve_owned_lanes`;
    `ensure_managed_primary` at `:1621-1660`; `crates/pse-operations/src/generated/surreal.surql`.
  - Any recommendation must keep or improve this lane parallelism.
  - Never propose replacing SurrealDB.
- **P2. Durability target.** Durable work can outlive its caller, and multi-process workspaces are
  supported, **without a daemon**: through SurrealDB coordination, on-demand activation, and recovery
  from the durable record.
- **P4. Vendored-library integration hooks are legitimate integration, not forks (maintainer, 2026-10-21).**
  The project has not edited core solver algorithms. Vendored or patched libraries carry *integration*
  for diagnostics, observation, work and memory accounting, cancellation and admission. Examples: the
  SUNDIALS KLU observers and fixed-pattern application hooks, the IDAS checkpoint/replay budgets, and
  the POUNCE observed-execution extension.
  - Do not recommend removing a hook merely because it lives in a vendored tree.
  - Judge a hunk on numerical grounds only if it actually changes numerical behaviour. For the Uno
    patch, classify each hunk as integration vs numerics before judging it.
- **P3. Other formats are tactical only.** Parquet, Arrow IPC or other file formats may serve as
  tactical payloads referenced from SurrealDB. They are never a second state, commit or lifecycle
  authority.

## Baseline
- Source commit: `a791c533eca0f65b195895f071e60cd1e38e121c`.
- Audit publication: `0530dcaa1`. HEAD = baseline + audit pages, clean tree.
- Date: 2026-10-21. Cite sources as `path:line`.

## Grounding
Judge against the standard and the functional target.
- **Standard:** `docs/design_review/design_principles/standard.toml`
  - core 3.4: `core/design-principles.md`, `core/efficient-architecture-heuristics.md`,
    `core/design-review-template.md`
  - process-simulator profile 1.5: `profiles/process-simulator/principles.md`, `review.md`
  - binding: `binding/pse-arrow.md`
  - skills: `.codex/skills/design-review/SKILL.md` + `REFERENCE.md`,
    `.codex/skills/design-review-process-simulator/SKILL.md`
- **Functional target:** the profile's *Functional target*, plus blueprint §0/§0.2 capability
  families (`docs/authoritative_design/sections/architecture-overview.md`).
- **Current design (which may change):** ADRs, D1–D14, architecture sections, binding policies,
  AGENTS.md, rules. Read them to understand the subject, not as constraints. Label anything you
  report from them as "current design".

## The audit (diagnostic input, open to challenge)
- `docs/design_review/audit/README.md`: baseline, coverage, finding index, and **strengths to
  preserve**.
- Topic pages:
  - `semantic-model-and-authority.md` (DA-001–006, 031, 032)
  - `structure-and-invariants.md` (DA-007–011)
  - `boundaries-and-composition.md` (DA-012–020)
  - `local-reasoning-and-assurance.md` (DA-021–024)
  - `execution-fit.md` (DA-025–030)

## EXCLUSIONS (mandatory)
Do NOT read, cite or follow:
- `docs/design_review/reviews/**`, `docs/design_review/evidence/**`, `docs/plans/**`;
- any review linked from ADRs or sections;
- `/tmp/claude-1000/-home-paul-pse-arrow/15c6fe71-a184-4329-a417-2fc2e56d8c52/scratchpad/review/`
  (a discarded earlier attempt).

Finding IDs from earlier reviews in code comments are not evidence. Do not use
`docs/capability-maps/`, `library-utilization.jsonl` or the library-catalog MCP.

## Evidence route
1. Context7: `resolve-library-id`, then `query-docs` per concept.
2. Then installed sources: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`,
   `~/.cargo/git/checkouts/`, `/home/paul/pse-arrow/vendor/`, the native-solver-libraries skill
   corpus, and the `.venv` site-packages for Python.
3. Then upstream tests.

Name concrete API items (types, functions, traits, options, SQL statements, feature flags) with a
`path:line` or doc reference. State conditions and limits. If you cannot settle something, say
"unresolved"; never assert absence from an empty search.

## Effects
- No file edits.
- No builds, tests or services (the machine is shared).
- Read-only commands are fine: `rg`, `git`, `ls`, `cargo metadata --no-deps`,
  `cargo tree --offline --locked -e features`.
- If a probe would change a recommendation, describe it (command, expected observation, what it
  settles) and return it.

## Judgment
- Prefer a library capability that owns a *complete* responsibility over one that replaces an
  isolated function and leaves duplicated machinery around it.
- For each candidate, also give the reverse check: which audit strength, enabler or guarantee would
  it remove or weaken? Pay particular attention to:
  - SurrealDB-coordinated parallelism
  - daemonless durability
  - value rebind
  - guard survival
  - library-owned derivatives
  - typed solver status
  - original-space qualification
  - DP-19 truthfulness
- Use the evidence vocabulary honestly (*Interface-checked*, *Implemented*, *Proposed*,
  *Tested*/*Measured* only for things actually run). Speed claims without measurement are
  *Proposed*.
