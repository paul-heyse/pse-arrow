# Governance

This document says who decides what in `pse-arrow`, how a decision is recorded, and how
the things that are deliberately hard to change — the parity pin, the dependency
families, the branch protections — actually move. It is itself governed: changing this
file requires an ADR and a design review (see the table in §3).

## 1. Roles

**Maintainer.** Currently one: [@paul-heyse](https://github.com/paul-heyse)
(paul@heyse.io), who is also the owner of the repository. The maintainer merges pull
requests, cuts releases, holds the ruleset bypass, administers the GitHub configuration,
and is the point of contact for security reports and Code of Conduct enforcement.
`.github/CODEOWNERS` assigns review of every path to the maintainer.

**Contributor.** Anyone who opens an issue or a pull request. Contributors do not need
commit access — everything lands through a pull request, including the maintainer's own
work.

**Reviewer (design).** For changes that require a design review, the reviewer is whoever
runs the review and records it under `docs/design_review/reviews/`. Today that is the
maintainer or an agent acting under the design-review skill. A review is *evidence*, not
authority: a review cannot approve an ADR, and an ADR cannot overrule the blueprint
without amending it.

### Adding a maintainer

A second maintainer is added by an ADR (`level: decision`, labeled `governance`) that
names the person, the scope of their commit rights, and the date. On merge the maintainer
of record:

1. adds them to the repository with the `Maintain` role (`Admin` only by a further ADR);
2. adds them to `.github/CODEOWNERS` and to the ruleset bypass list in
   `.github/setup/ruleset-main.json`, then re-runs `just gh-setup`;
3. adds them to the `release` environment reviewers in `.github/setup/env-release.json`;
4. records them in `CITATION.cff` if they wish to be credited.

Maintainer status lapses by resignation, or by an ADR after twelve months of inactivity.
The same three steps are reversed. With two or more maintainers, required approvals on
the `main` ruleset move from `0` to `1` — this is currently `0` only because a sole
maintainer cannot approve their own pull request.

There is no project board and no formal committee at this size. A board is a register row
(`project board at second contributor`) and appears when there is a second contributor.

## 2. Decision process

Decisions are recorded as **Architecture Decision Records** under `docs/adr/`, one file
per decision, `NNNN-kebab-title.md`, following MADR 4 plus the charter §H front-matter
fields (`id`, `title`, `status`, `date`, `deciders`, `level`, `principles`, `blueprint`,
`review`, `evidence`, `supersedes`/`superseded-by`, `revisit`, `verification`).

**Status lifecycle:** `proposed` → `accepted` | `rejected`; an accepted record later
becomes `deprecated` or `superseded`. An accepted ADR is immutable except for its status
fields and its status history — it is changed by superseding it, never by editing it.
`scripts/adr.py lint` enforces this against `origin/main` and runs as
`governance / adr-lint`.

### The decision-PR rule

An ADR enters the repository, or changes status, **only** in a pull request that:

- is **labeled `adr`**, and
- is **titled `adr: ADR-NNNN <title>`**, and
- amends `docs/authoritative_design/blueprint.md` in the same PR — or in a named
  follow-up PR titled `design: …` — with a revision row and an inline
  `> Decision: ADR-NNNN` note at the governed section, and
- is merged by the maintainer. **The maintainer's merge is the approval**; there is no
  separate sign-off step.

A pull request may merge with an ADR still at `status: proposed` **only** if it also
carries the `needs-review` label, which records that the design review is outstanding.
Merging a proposed ADR is a way to publish a direction for comment, not a way to skip the
review: the follow-up PR that moves it to `accepted` is the one that must cite the
review's verdict (Accept or Accept-scoped) in the `review:` field.

An ADR whose `level` is `must-gap` **narrows scope**; it never claims compliance. A
`should-deviation` records a deliberate departure with its compensating control.

### When an ADR or a design review is required

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; adds, drops, or majors a dependency family; changes the hashing contract, the Python boundary contract, metadata conventions, or the commit contract; any SHOULD deviation; governance changes | **ADR + design review** (label `needs-review`; verdict Accept or Accept-scoped before `status: accepted`) |
| A new relation family, pass, kernel contract, or backend binding within an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | **ADR (short)**; review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, documentation wording, patch bumps inside a pinned family, tooling | **Neither**; an ordinary pull request with the evidence field filled |

Anyone may propose a decision by opening a `design-decision` issue. The maintainer labels
it `needs-adr` when it clears that bar; the `needs-adr` label on an open issue or PR is
checked by `governance / adr-lint`.

### The register

Deliberately deferred decisions are rows in `docs/adr/register.md`:
`R-NN | item | ADR | trigger | check | owner | last-checked | next-check | status`. A row
must have an **observable trigger** and a **check** that can be run. `register-review.yml`
runs monthly, executes the automatable checks, and opens or updates a single
`Register review YYYY-MM` issue labeled `register`. Closing a row means writing the ADR
its trigger called for — not deleting the row.

## 3. Authority, and what overrules what

1. `docs/authoritative_design/blueprint.md` — **authoritative**. One file, revised in
   git; section numbers are stable citation targets.
2. `docs/adr/` — **why**. ADRs explain and amend; an amendment is only real once the
   blueprint carries the revision row.
3. `docs/design_review/reviews/` — **evidence**, never authority.
4. `docs/plans/` — **how work is sequenced**. Living until done, then closed with an
   `## Outcome` recording a mistake made and corrected, and deliberate deviations.
5. The code. *When the code and a plan disagree, the code is what runs* — and one of the
   two is then a bug.

## 4. Branch protection, and bypassing it

`main` is protected by a repository ruleset (declared in
`.github/setup/ruleset-main.json`, applied by `just gh-setup`): no deletion, no
non-fast-forward, linear history required, signed commits required, pull request
required (0 approvals while there is one maintainer; stale reviews dismissed; threads
must be resolved; squash merge only), and the required status checks listed in
`docs/dev/ci.md`. Tags matching `v*` are protected by a second ruleset: creation, update
and deletion restricted, signatures required.

**Bypass policy.** The maintainer's admin role is on the ruleset bypass list with mode
`always`, because a sole maintainer must be able to recover the repository — for example
to repair a broken `main` or to land a required-check definition that does not yet
report. Bypass is a break-glass action, not a shortcut:

- Every bypass push **must** be followed by an issue labeled `governance` stating what
  was pushed, why the normal path could not be used, and what prevents a recurrence.
- A bypass is never used to skip a failing check on ordinary work. A check that is wrong
  gets fixed or removed by a PR; a check that is right gets obeyed.
- Bypass pushes must be **signed** — the ruleset requires signatures and does not exempt
  bypass actors. Squash merges are signed by GitHub; a direct push needs local SSH or
  GPG signing configured.
- The bypass list is part of the declared configuration. Changing it is a governance
  change and needs an ADR (§2 table).

Required checks are staged deliberately: a check that never reports blocks every pull
request, so `python / *` joins the required list only after the Python package first
reports. `.github/setup/ruleset-main-full.json` holds the full list for that re-run.

## 5. How the pins move

**The IDAES parity pin (`idaes-pse==2.12.0`).** This is the only durable coupling to
IDAES and it is what "parity" means. Moving it requires a short ADR that records: the new
version, what changed in the numerical behaviour under test, which parity tests moved and
why, and whether any preserved enumeration changed. The PR must update the pin in
`[dependency-groups] parity`, the tag in `scripts/fetch-external.sh` (the two are
compared by `governance / adr-lint`), `docs/relationship-to-idaes.md`, and any tolerance
that had to change — each tolerance change called out individually in the PR body. A
parity run must be green on the pinned interpreters before merge. The pin is never moved
in the same PR as a behavioural change on our side.

**Dependency families (arrow, datafusion, pyo3, object_store).** A family is pinned with
`=` once in `[workspace.dependencies]` and moves as a unit:

1. An ADR records the move, the reason, and what was re-checked.
2. `cargo update -p <crate> --precise <version>` for each member — never a bare
   `cargo update`; `Cargo.lock` is committed.
3. `cargo xtask family-check` must pass: exactly one resolved version per family, and
   packages shared with the evidence lockfiles must match. `cargo tree -d` cannot detect
   a mixed family; this check can.
4. The corresponding capability map under `docs/capability-maps/` is regenerated
   (`just evidence-regen`) and its `pins:` front matter updated.
5. Pre-1.0 crate floors are re-checked (`dependency_floors` in `tests/governance`), and
   any register row whose trigger the move fires is answered.
6. `cargo deny check` and `cargo semver-checks` must be green; a `datafusion-proto`
   bump also re-checks byte stability (register row).

Dependabot proposes these as grouped PRs (`arrow-family`, `datafusion-family`,
`pyo3-family`, `codegen`, `numerics`) so the unit is preserved; a PR that moves part of a
group is closed, not merged. Python pins move with `uv lock --upgrade-package <name>`;
`idaes-pse` is explicitly ignored by Dependabot because it is the parity pin.

**The Rust toolchain and MSRV.** `rust-toolchain.toml` and
`[workspace.package] rust-version` are kept equal by a governance test. They move
together, in their own PR, with an ADR when the MSRV rises (ADR-0018).

**The solver image.** `docker/solvers/` is content-addressed: pushing a new image updates
`SOLVER_IMAGE` in the workflows through an automatically opened PR. Changing the recipe
(Ipopt, MUMPS, or ASL version, or a build flag that can move iteration counts) needs an
ADR, because parity trajectories depend on it.

## 6. Releases

Only the maintainer releases. Versioning is a **single workspace version**; there is one
`v*` tag per release, always on `main`, always signed. `just release <version>` bumps the
version, regenerates `CHANGELOG.md` with `git-cliff` from the Conventional-Commit squash
titles, commits, and tags. Pushing the tag runs `release.yml`, which gates on the version
agreeing across `Cargo.toml`, the tag, every wheel filename, and a `CHANGELOG.md`
heading, then publishes through the `test-release` environment (TestPyPI, branch policy
`main`) and the `release` environment (PyPI, tag policy `v*`, maintainer review). Phase 0
is `0.0.x`; `v0.1.0` marks phase-1 exit.

`cargo-semver-checks` is informational on ordinary pull requests and **gating** on pull
requests labeled `release`.

## 7. Code of Conduct enforcement

The [Code of Conduct](CODE_OF_CONDUCT.md) (Contributor Covenant 2.1) applies in every
project space: issues, pull requests, discussions, commit messages, and any space where
someone is representing the project.

Reports go to **paul@heyse.io**. The maintainer acknowledges within **7 days**,
investigates privately, and applies the Contributor Covenant enforcement ladder
(correction → warning → temporary ban → permanent ban). The reporter's identity and the
details of the report are kept confidential; the outcome is communicated to the reporter
and, where a public action was taken, noted publicly without identifying the reporter.

If a report concerns the maintainer, or the maintainer has a conflict of interest, the
reporter may escalate to GitHub Support, which can act on the repository independently.
Until a second maintainer exists there is no internal appeal; that gap is recorded here
deliberately rather than pretended away, and closes when §1's "adding a maintainer" path
is used.

## 8. Changing this document

Governance changes are in the top row of the §2 table: ADR **and** design review. The
pull request is labeled `governance` and `adr`, and the ADR's `verification:` field names
what shows the new rule holds — usually a lint in `scripts/adr.py`, a field in
`.github/setup/*.json`, or a line in `docs/dev/ci.md`.
