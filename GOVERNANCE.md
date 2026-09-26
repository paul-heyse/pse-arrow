# Governance

This document says who decides what in `pse-arrow`, how a decision is recorded, and how
the things that are deliberately hard to change — the parity pin, the dependency
families, the branch protections — actually move. It is itself governed: changing this
file requires an ADR and a design review (see the table in §2).

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
authority: a review cannot approve an ADR, and an ADR cannot overrule the architecture
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
per decision, `NNNN-kebab-title.md`, following MADR 4 plus the core principles §H front-matter
fields (`id`, `title`, `status`, `date`, `deciders`, `level`, `principles`, `blueprint`,
`review`, `evidence`, `supersedes`/`superseded-by`, `revisit`, `verification`).

**Status lifecycle:** `proposed` → `accepted` | `rejected`; an accepted record later
becomes `deprecated` or `superseded`. An accepted ADR is immutable except for its status
fields and its status history — it is changed by superseding it, never by editing it.
`scripts/adr.py lint` enforces this against `origin/main` and runs as
`governance / adr-lint`.

**Retention (ADR-0096).** `docs/adr/` keeps the decisions whose rationale explains the
current system. A record whose mechanism is gone, or which a later decision fully
replaced, is retired: the file is deleted after its surviving meaning has an owner, and
Git history is the archive. Numbers are never reused, so the index has gaps, and the
highest-numbered record stays until a newer one exists. Retained accepted records remain
immutable; when a cited review or plan is retired, the record's reference moves to the
same path at an immutable commit (`git:<commit>:<path>` or a repository permalink).
Implementation authorization is distinct from formal decision-PR acceptance.

### The decision-PR rule

An ADR enters the repository, or changes status, **only** in a pull request that:

- is **labeled `adr`**, and
- is **titled `adr: ADR-NNNN <title>`**, and
- amends the authoritative collection in the same PR — or in a named
  follow-up PR titled `design: …` — with a revision row in `docs/authoritative_design/blueprint.md` and an inline
  `> Decision: ADR-NNNN` note at the governed section's owner in the collection, and
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
| Alters D1–D14; adds or removes a crate; majors one of the four pinned families (arrow, datafusion, object_store, pyo3); changes the hashing contract, the Python boundary contract, metadata conventions, or the commit contract; any SHOULD deviation; governance changes | **ADR + design review** (label `needs-review`; verdict Accept or Accept-scoped before `status: accepted`) |
| A new relation family, pass, kernel contract, or backend binding within an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | **ADR (short)**; review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, documentation wording, patch bumps inside a pinned family, tooling | **Neither**; an ordinary pull request with the evidence field filled |

Anyone may propose a decision by opening a `design-decision` issue. The maintainer labels
it `needs-adr` when it clears that bar; the `needs-adr` label on an open issue or PR is
checked by `governance / adr-lint`.

### Architecture review and implementation tracking

The [standard declaration](docs/design_review/design_principles/standard.toml) selects the
current core, profile and binding. The six architectural foundations organize the review;
operational and scientific requirements apply where relevant. The
[review template](docs/design_review/design_principles/core/design-review-template.md) owns
the method, including independent architectural-fitness and behavioral-adequacy decisions.
See blueprint §24.4 and ADR-0094 for this governance amendment.

A review records evidence against a specific scope and standard version. The owning plan
holds current adopted-finding dispositions and links scenarios, decisions, packets and their
evidence. Indexes link live status instead of copying it. Accepting a decision and scheduling
work do not certify implementation. Accepted ADR evidence describes support at decision time;
historical review verdicts are not retroactively updated. Optional `standard` and `scenarios`
ADR fields record the reviewed versions and links without changing old records.

### The register

Deliberately deferred decisions are rows in `docs/adr/register.md`:
`R-NN | item | ADR | trigger | check | owner | last-checked | next-check | status`. A row
must have an **observable trigger** and a **check** that can be run. `just register-check`
(or the manually dispatched `register-review.yml`) executes the automatable checks of due
rows. When a row is decided, satisfied or no longer applicable, the decision lands at its
owner and the row is removed; row ids are never reused. The register may be empty.

## 3. Authority, and what overrules what

1. `docs/authoritative_design/README.md` — entry to the **authoritative collection**:
   numbered pages under `sections/` that describe the current system. Each section has one
   owner; identifiers survive moves; `blueprint.md` keeps the collection revision history and
   maps former anchors.
2. `docs/adr/` — **why**. ADRs explain and amend; an amendment is only real once the
   blueprint carries the revision row.
3. `docs/design_review/reviews/` — **evidence**, never authority; removed once adopted
   findings and enduring rationale have their owners.
4. `docs/plans/` — **how work is sequenced**. Living until done, then closed with an
   `## Outcome` recording a mistake made and corrected, and deliberate deviations. A
   completed plan retires once its enduring meaning has moved to the contract owner.
5. The code. *When the code and a plan disagree, the code is what runs* — and one of the
   two is then a bug.

## 4. Branch protection, and bypassing it

`main` retains a repository ruleset (declared in
`.github/setup/ruleset-main-full.json`, applied by `just gh-setup`): no deletion,
no non-fast-forward updates, linear history and signed commits. Direct commits and
pushes are allowed. CI has no required status checks and pull requests are optional.
Check workflows run only when manually dispatched; see `docs/dev/ci.md`. Tags matching
`v*` retain their separate release ruleset.

**Bypass policy.** The maintainer's admin role is on the ruleset bypass list with mode
`always`, for recovery from a broken `main`. Bypass is a break-glass action:

- Every bypass push **must** be followed by an issue labeled `governance` stating what
  was pushed, why the normal path could not be used, and what prevents a recurrence.
- A manually run failing check is reviewed when the maintainer chooses; it is not a
  ruleset gate.
- Bypass pushes must be **signed** — the ruleset requires signatures and does not exempt
  bypass actors. Squash merges are signed by GitHub; a direct push needs local SSH or
  GPG signing configured.
- The bypass list is part of the declared configuration. Changing it is a governance
  change and needs an ADR (§2 table).

The `main` ruleset has no required status checks or required pull request. A future
change to that policy is a separate maintainer decision.

## 5. How the pins move

**The IDAES parity pin (`idaes-pse==2.12.0`).** This is the only durable coupling to
IDAES and it is what "parity" means. Moving it requires a short ADR that records: the new
version, what changed in the numerical behaviour under test, which parity tests moved and
why, and whether any preserved enumeration changed. The PR must update the pin in
`[dependency-groups] parity`, the tag in `scripts/fetch-external.sh` (the two are
compared by `governance / adr-lint`), `docs/relationship-to-idaes.md`, and any tolerance
that had to change — each tolerance change called out individually in the PR body.
A parity run can be requested to review the pinned interpreters. The pin is never moved
in the same PR as a behavioural change on our side.

**Adding a dependency.** Nothing. No library is refused and no licence is grounds to
refuse one through phases 0–1: add it, pin it exactly, commit the lockfile. No ADR, no
design review, no blueprint row. See [dependency policy](docs/dev/dependency-policy.md)
and ADR-0066 for the reasoning, what is still enforced, and register row R-31 — the dated
obligation to answer the licensing question before anything is published.

**Dependency families (arrow, datafusion, pyo3, object_store).** A family is pinned with
`=` once in `[workspace.dependencies]` and moves as a unit. A **major** move follows all
six steps below; a patch bump inside the pinned family needs none of them:

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
6. `cargo semver-checks` must be green; a `datafusion-proto` bump also re-checks byte
   stability (register row). `cargo deny check` reports but does not gate (ADR-0066) —
   read it, act on what matters, and do not treat it as a blocker.

Dependabot proposes these as grouped PRs (`arrow-family`, `datafusion-family`,
`pyo3-family`, `codegen`, `numerics`) so the unit is preserved; a PR that moves part of a
group is closed, not merged. Python pins move with `uv lock --upgrade-package <name>`;
`idaes-pse` is explicitly ignored by Dependabot because it is the parity pin.

**The Rust toolchain and MSRV.** `rust-toolchain.toml` and
`[workspace.package] rust-version` are kept equal by a governance test. They move
together, in their own PR, with an ADR when the MSRV rises (ADR-0018).

**The solver image.** `docker/solvers/` is content-addressed: manually dispatching
`solvers-image.yml` with `publish=true` publishes an image and prepares a pin update PR.
Changing the recipe
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
