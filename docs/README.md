# pse-arrow

A process systems engineering core in Rust, with typed process definitions, library-owned
mathematics and native solvers. Arrow, DataFusion and Delta provide data-boundary, relational
and storage capabilities. See the [relationship to IDAES](relationship-to-idaes.md).

| I want to… | Start here |
|---|---|
| Understand the architecture | [Architecture and reading guide](authoritative_design/README.md) |
| Know what is supported | [Scope and open design](authoritative_design/sections/scope-and-open-design.md) |
| Find active work | [Current work](plans/README.md) |
| Review or change a design | [Design-change workflow](authoritative_design/sections/design-change-workflow.md) |
| Research a pinned library | [Capability maps](capability-maps/README.md) |
| Understand a decision | [Decision records](adr/README.md) |
| Work on documentation | [Publishing documentation](dev/documentation.md) |

Search defaults to **Current**: the architecture, development guides, the design standard and
active work. **Reference** includes library maps, generated schemas, proposed decisions and the
blueprint's former-anchor table. **History** holds only a few deliberately retained records;
completed plans and resolved reviews are retired to Git history and are not published.
**Everything** searches all published chapters. Scope describes reading purpose, not
qualification. The chapter footer identifies the collection and any recorded lifecycle status.

Markdown is the canonical source for readers and agents. Follow a task route, read the relevant
contract, and inspect the implementation needed for the change. Qualification results keep
their original conditions; only named tests and measurements support Tested and Measured claims.

## Repository map

| Path | What it is | How to treat it |
|---|---|---|
| `crates/` | Workspace `pse-*` crates declared in `Cargo.toml` | Ours; see `.claude/rules/rust.md`. `pse-workspace-hack` has no code: cargo-hakari generates its dependencies (ADR-0122) |
| `xtask/` | Everything that needs Rust APIs, JSON or cross-platform behaviour | Logic lives here, the justfile is the surface |
| `tests/` | Workspace test crates: `governance`, `engine`, `conformance`, `lifecycle`, `structural` | `tests/fixtures/` contains source inputs, not a crate |
| `benches/` | Criterion benchmarks (`pse-benches`) | No timing gate in CI; `just bench-smoke` only runs them |
| `python/pse/` | The Python package (import name `pse`) | `python/pse/contracts/` is GENERATED |
| `docs/authoritative_design/` | **Authoritative collection:** current contracts in numbered `sections/` pages; `blueprint.md` keeps revisions and former anchors | Start at its README; edits follow the design route below |
| `docs/adr/` | Retained decision records, immutable once accepted; obsolete ones retired to Git (ADR-0096) | `just adr-new`; index via `just adr-index` |
| `docs/plans/` | Active plans, living until done; completed plans retire to Git | `just plan <slug>` |
| `docs/capability-maps/` | Pinned third-party API maps + their evidence | `just lib-outline <file>` first; they are large |
| `docs/design_review/` | The layered design standard (`design_principles/standard.toml`: core principles, process-simulator profile, pse-arrow binding) and reviews whose findings are still open | The `design-review` and `design-review-process-simulator` skills' output contract |
| `docs/generated/` | `pse-schema` output | Never edit |
| `external/` | Pinned read-only checkouts (`just fetch-external`) | **Not source.** Gitignored, never edited, never copied from |
| `build/`, `target/` | Build output | **Not source.** Regenerable |
| `docker/solvers/` | The Ipopt 3.14 + MUMPS + ASL recipe | Changing it changes CI's solver image |

## Where authority lives

Do not restate these; cite them.

- **`docs/authoritative_design/README.md`** — the entry to the authoritative architecture:
  numbered pages under `sections/`. Section identifiers have one owner and remain stable
  when moved: cite `blueprint §14.3`, never a line number. `blueprint.md` retains the
  collection revision history and maps former single-file anchors to their owners.
- **`docs/adr/README.md`** (generated index) and the ADRs themselves — *why* a decision
  was made. The sections say what is true; ADRs say why; reviews are evidence, not authority.
  Only decisions whose rationale explains the current system are retained (ADR-0096).
- **`docs/adr/register.md`** — every deferred decision with its trigger, check and next
  review date. `just register-check` runs the ones that are due.
- **`Cargo.toml` header comment** — dependencies float as carets under `Cargo.lock`; why
  the arrow/datafusion/object_store/pyo3 family pins are what they are, why `=` pins alone
  are not sufficient, and `[workspace.metadata.pse.pins]`, the reason for every other exact
  pin (ADR-0159).
- **`pyproject.toml`** — libraries and tools carry `>=` floors and run at whatever
  `uv.lock` resolved, never installed ad hoc; the parity group and test oracles are `==`
  pins, each with its reason beside it. No release of `uv` itself is
  required.
- **`docs/capability-maps/`** — what the pinned libraries actually expose, with evidence.
  `just lib-outline docs/capability-maps/arrow-rust.md` before reading one.
- **`docs/design_review/design_principles/standard.toml`** — the layered design standard
  used by design reviews: six architectural foundations (`AP-nn`), operational refinements
  (`DP-nn`, gates `G1`–`G9`), the
  process-simulator profile (`PS-nn`, `PS-G1`–`PS-G3`) and the pse-arrow binding.
- **`docs/dev/dependency-policy.md`** — what you may depend on and under what licence.
  Short answer: anything. Read it before assuming a library is off-limits.

