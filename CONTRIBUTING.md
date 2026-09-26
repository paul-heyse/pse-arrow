# Contributing to pse-arrow

This project is in early design and implementation. Architecture evolves through the
blueprint and decision process; current implementation scope lives in the active plans.
Changes should be justified, verified and reproducible. This document tells you how
to set up, what workflow a change follows, and what each command actually proves.

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md). Governance
(roles, decision process, how pins move) is in [GOVERNANCE.md](GOVERNANCE.md). Security
issues go through [SECURITY.md](SECURITY.md), never a public issue.

## 1. Environment

Prerequisites: `git`, a C toolchain, [`just`](https://just.systems),
[`uv`](https://docs.astral.sh/uv/) (any current release), and either Docker/Podman (for the solver
container) or a local Ipopt 3.14.x. Rust is installed *by the repository*: the pinned
toolchain comes from `rust-toolchain.toml` — never `rustup default`.

```bash
git clone https://github.com/paul-heyse/pse-arrow
cd pse-arrow
just doctor        # says what is missing; no network, no writes
just bootstrap     # pinned toolchain, cargo tools, .venv
just doctor        # should now be clean
just --list        # the command surface
```

Notes that save time:

- Tools are used from `.venv/bin` and from the pinned toolchain, **never from `$PATH`**.
  A globally installed `ruff` or `cargo-nextest` is not the one CI runs.
- `.python-version` is the single source of truth for the interpreter.
- `direnv allow` loads `.envrc`, which is deliberately fast and touches no network.
- `scripts/fetch-external.sh` clones `idaes-pse`, `arrow-rs`, and `datafusion` at their
  pinned tags into `external/` (gitignored). Those are **reading copies**. Never copy
  code out of them — see the clean-room rule below.
- The solver container (`ghcr.io/paul-heyse/pse-solvers`) is how CI gets Ipopt 3.14.x +
  MUMPS + ASL. `.devcontainer/` uses its `dev` stage. A system Ipopt older than 3.14 will
  not do.

## 2. Repository map

See the layout table in [README.md](README.md#layout). The three directories that behave
differently from the rest:

- `docs/authoritative_design/README.md` — architecture entrypoint. The blueprint and
  focused numbered sections share stable citation identities. Design amendments need an
  ADR and a blueprint revision row; moved sections retain old anchor links.
- `docs/generated/`, `crates/*/src/generated/`, `python/pse/contracts/` — **generated**.
  Never edit by hand; fix the generator and run `just codegen`.
- `external/`, `target/`, `build/` — not yours to edit; regenerable or fetched.

## 3. The change workflow

1. **State the change and its driver.** An issue can organize discussion (bug, design
   decision, parity gap, library upgrade). For architecture, identify representative changes
   and the responsibility boundaries they should affect.
2. **Use the checkout workflow in [AGENTS.md](AGENTS.md#personal-project-checkout-workflow).**
   Ordinary maintainer development uses the existing checkout; preserve concurrent work.
3. **Decide whether an ADR is required** — the table in §4. If one is, the ADR lands in
   its own PR labeled `adr`, titled `adr: ADR-NNNN <title>`, before or with the code.
4. **Work small.** Run focused checks when useful. CI is entirely manual and does not
   run or gate commits, pushes, pull requests, merges or plan closure. Use `just ci-fast`,
   `just ci-pr`, `just parity-container` or the GitHub Actions **Run workflow** button
   whenever you choose to review the project. Full wheels/sdists remain manual
   (`just wheels-check <ref>`) or release-time checks.
5. **Commit** with Conventional Commit subjects so the changelog remains readable. Types:
   `feat fix perf refactor docs test build ci chore deps adr design`. Scopes are the
   crate or area short names and are free-form.
6. **Use a PR when it helps review.** Fill its template with the evidence actually
   gathered and the clean-room attestation. Direct signed commits to `main` are also
   allowed. CI is never a PR or merge prerequisite.

### Evidence vocabulary (§D)

Use the [core evidence vocabulary](docs/design_review/design_principles/core/design-principles.md#d-evidence-vocabulary)
for PR descriptions, ADR evidence, register rows and plan Verification/Outcome at closure.
Tested and Measured name the executed check and its conditions. Interim checkpoints describe
state and next steps. Document acceptance, implementation and product qualification are distinct.

## 4. When an ADR or a design review is required

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; majors one of the four pinned families (arrow, datafusion, object_store, pyo3); changes the hashing contract, the Python boundary contract, metadata conventions, or the commit contract; any SHOULD deviation; governance changes | **ADR + design review** — label `needs-review`; verdict Accept or Accept-scoped before the ADR becomes `accepted` |
| A new relation family, pass, kernel contract, or backend binding *within* an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | **ADR (short)**; review at maintainer discretion |
| Bug fixes; refactors inside existing contracts; tests; documentation wording; a patch bump inside an already-pinned family; adding, removing or upgrading a third-party dependency; tooling | **Neither** — an ordinary PR with the evidence field filled |

`just adr-new` scaffolds a record from `docs/adr/template.md`; `just adr-lint` checks it;
`just adr-index` regenerates `docs/adr/README.md` and the book's decisions section. An
accepted ADR is immutable except for its status fields — change it by superseding
(`just adr-supersede`). Deferred items live as rows in `docs/adr/register.md` with an
observable trigger and a check; `just register-check` reports which are due.

Architectural reviews use the versions in
[`standard.toml`](docs/design_review/design_principles/standard.toml). The
[review template](docs/design_review/design_principles/core/design-review-template.md) starts
with responsibilities and change scenarios and assesses architectural fitness separately from
behavioral/scientific adequacy. Library fit includes integration ownership and cost.

Link adopted findings to their plan-owned disposition, scenarios, decisions and packet evidence.
Keep current status in one place; indexes link it. An accepted ADR or scheduled packet does not
establish implementation. Historical reviews keep their original version, scope and evidence.

## 5. What each command proves — and does not prove

| Command | Proves | Does **not** prove |
|---|---|---|
| `just check` | the workspace type-checks with default features | that it links against Ipopt, or that any test passes |
| `just clippy` | no lint escapes, at default and `--no-default-features` | anything about runtime behaviour |
| `just test` | unit/integration tests pass **with `force-validate` on**, so Arrow invariants are enforced | release-profile behaviour, or anything needing a solver beyond the container's |
| `just doctest` | documentation examples compile and run | that the surrounding prose is accurate |
| `just codegen-check` | committed generated sources match the generator, byte for byte | that the generator is *correct* |
| `just family-check` | exactly one resolved version per pinned family, matching the evidence lockfiles | that the pinned versions are the best ones |
| `just governance` | the repository's own invariants (pins, registration, MSRV, floors, unsafe allowlist, error taxonomy) | anything about the domain |
| `just deps-report` | what is in the dependency graph and under what licences — **advisory, always exits 0** | nothing: it refuses nothing, and no library or licence blocks a merge (§3.3.2, ADR-0066) |
| `just policy` | the same `cargo deny` + `cargo audit` checks run strictly. Opt-in; not part of `just ci-pr` | that a dependency is *appropriate* |
| `just coverage` | line/branch coverage numbers | that the covered lines assert anything meaningful |
| `just bench-smoke` | benchmarks still build and run | any performance claim — that needs `Measured` with conditions |
| `just quality` | ruff, pyrefly, import-linter, REUSE, lockfile freshness | that the Python surface matches the native extension |
| `just parity` | agreement with `idaes-pse==2.12.0` on the trajectories under test, on Linux with Ipopt 3.14.x | agreement anywhere else, or that IDAES is right where we differ |
| `just doc-lint` | every backticked `a::b::c` in `docs/**` resolves in the pinned API surface | that the prose describes what the API does |
| `just ci-pr` | the local composite Rust, Python, quality and documentation checks pass | GitHub's interpreter matrix, container parity, or deeper manual jobs |

GitHub checks and `just ci-pr` are optional and run only when requested.
Say what you verified using the §D vocabulary, and say what you did not.

## 6. Generated code, pins, and lockfiles

- **Generated sources are committed and diffed.** `rust / codegen-diff` runs
  `cargo xtask codegen --check` and fails on any drift, including untracked files. If you
  change a schema, run `just codegen` and commit the result *in the same PR*.
- **Every dependency is pinned.** Rust families use `=`-pins declared once in
  `[workspace.dependencies]`; Python libraries are `==`-pinned. `Cargo.lock` and
  `uv.lock` are committed and every gate passes `--locked`.
- **Moving a pin is a deliberate act.** Rust: `cargo update -p <name> --precise <ver>`;
  Python: `uv lock --upgrade-package <name>`. Never a bare `cargo update` or `uv lock`
  in a feature PR. A family **major** (arrow, datafusion, pyo3, object_store) needs an
  ADR, a regenerated capability map, and a green `family-check`.
- **Adding a dependency needs none of that.** No library is refused and no licence is
  grounds to refuse one through phases 0–1: add it, pin it, commit the lockfile, and
  carry on. See [dependency policy](docs/dev/dependency-policy.md) for what *is* still
  enforced and why.
- `cargo tree -d` cannot detect a *mixed* family — `cargo xtask family-check` is the
  check that can. Pinning the umbrella crate does not pin the family.
- Dependabot opens grouped PRs monthly (cargo, uv) and weekly (actions). Groups exist so
  a family moves as a unit; splitting a group in a PR is a red flag.

## 7. Documentation conventions

- New files are lowercase kebab-case. Exceptions: the three UPPER_SNAKE design-principle
  files, the root governance files, and the design-review skill's output names.
- YAML front matter is required on ADRs, plans, and capability maps.
- Cite as `blueprint §14.3`, `ADR-0020`, `AP-06`, `DP-09`, `PS-10`, `G4`. Section numbers are stable: insert
  `§14.3.1`, never renumber.
- Generated documents carry `<!-- @generated by pse-schema; do not edit -->`.
- Internal links must resolve — `docs / build` runs `lychee --offline` over the book.

## 8. Licensing and the clean-room rule

Contributions are dual licensed **MIT OR Apache-2.0**. Every authored source file carries
a two-line SPDX header:

```text
SPDX-License-Identifier: MIT OR Apache-2.0
Copyright (c) 2026 Paul Heyse
```

`REUSE.toml` covers file types that cannot carry a header. Run `just lint-license`
locally or dispatch `governance.yml` when you want to check it.

**Clean room.** You may read `external/idaes-pse` (and any other reference
implementation) to understand *behaviour*. You may not copy code, comments, or docstrings
from it, and you may not paste from a source whose license is not MIT- or
Apache-compatible. The PR template asks you to attest to this; the attestation is the
point, not the checkbox.

## 9. Releases

Releases are cut by the maintainer: `just release <version>` bumps the single workspace
version, regenerates `CHANGELOG.md` with `git-cliff`, and tags `v<version>` on `main`.
Pushing the tag runs `release.yml` (version gate → wheels → attestations → TestPyPI →
smoke test → PyPI → GitHub Release). Contributors do not tag. See
[GOVERNANCE.md](GOVERNANCE.md#6-releases) for who may, and `CHANGELOG.md` for what shipped.

## Documentation development

Follow [publishing documentation](docs/dev/documentation.md) for collection discovery,
scoped search and independent docs tooling. Add pages inside a declared collection without
editing a second navigation list. Update enduring explanations/contracts when they change;
ordinary implementation edits need no proof manifest or duplicated symbol inventory.
