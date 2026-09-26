# Deferred-decision register

Every item here was deliberately **deferred with a stated trigger** rather than
decided. Only genuine current deferrals stay: when a row is decided, satisfied by
completed work or no longer applicable, it is removed; Git history retains it (ADR-0096).
Row ids are never reused; a new row takes the next id after the high-water mark below,
which the lint checks. The register may be empty.

Highest issued row id: R-33.

| Column | Meaning |
|---|---|
| `R-NN` | Row id. Cited from ADRs and issues; ids are never reused. |
| `item` | What was deferred, in one line. |
| `ADR` | The decision record that owns the deferral, or `—` when an architecture section owns it. |
| `trigger` | The observable event that ends the deferral. Not a date. |
| `check` | How to tell whether the trigger has fired. A cell starting with `$ ` is a shell command that `scripts/check_register.py --due` runs and reports; anything else is a manual check. |
| `owner` | Who answers for the row. |
| `last-checked` | The date the check was last actually run. |
| `next-check` | When it is due again. |
| `status` | `open` (deferred, waiting) or `watch` (trigger may fire soon). |

`scripts/check_register.py --lint` runs in `just adr-lint`: it validates dates, checks
that every referenced ADR is retained, and fails when a row has a `next-check` in the
past. `just register-check` (and the manually dispatched `register-review` workflow)
runs `--due` and reports the checks of rows that are due.

| R-NN | item | ADR | trigger | check | owner | last-checked | next-check | status |
|---|---|---|---|---|---|---|---|---|
| R-04 | Compiled third-party physical provider packages and their loading boundary (§26, D9) | — | an external provider package is actually proposed | manual: has a provider outside `pse-kernels` been proposed? | paul-heyse | 2026-09-26 | 2027-03-01 | open |
| R-05 | FeOS provider at compatible workspace num-dual pin (§9.8) | ADR-0084 | A new property package requires a different derivative family | manual: review the actual provider and derivative contract before moving pins | paul-heyse | 2026-09-24 | 2026-12-01 | watch |
| R-08 | Ipopt / MUMPS build recipe on platforms other than local Linux (§18.3, §26) | ADR-0028 | a macOS or Windows build of the native solver profile is requested, or the Linux recipe fails to reproduce | $ `ls docker/solvers/checksums.sha256` | paul-heyse | 2026-09-25 | 2026-12-01 | open |
| R-09 | wheel solver linkage: bundle Ipopt/MUMPS versus runtime `libloading` | ADR-0028 | the first wheel that must ship the native backend | manual: does a release need `pse-py` built with the `ipopt` feature? | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-10 | `object_store` multi-writer commit coordination (§20.1, §26) | ADR-0091 | any deployment that is not single-writer local | manual: re-validate the exact publication precondition against the target object store backend | paul-heyse | 2026-09-25 | 2027-03-01 | open |
| R-11 | Proposed: canonical plan-byte contract beyond diagnostic encodings (§14.2) | ADR-0044 | a consumer requires canonical plan bytes or upstream supplies a usable versioned byte-stability contract | manual: review need and codec/engine compatibility at dependency changes; current diagnostic plan bytes stay noncanonical | paul-heyse | 2026-09-13 | 2026-10-13 | open |
| R-12 | Miri re-adoption (dropped: no `unsafe` outside the FFI crates) | — | the first `unsafe` block outside the four allowlisted crates | $ `cargo tree --workspace --depth 0` | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-13 | Renovate as the fallback for Dependabot | ADR-0035 | Dependabot cannot express a grouping the family rule needs | manual: has a family-group PR arrived ungrouped? | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-14 | Python API documentation tool (mkdocstrings or equivalent) | ADR-0095 | a release or external user needs generated Python API reference beyond the typed stubs and guides | manual: is a Python API reference requested for a release? | paul-heyse | 2026-09-25 | 2026-12-01 | open |
| R-15 | crates.io release tooling: `release-plz` versus `cargo-workspaces` | ADR-0002 | the library crates first flip `publish` | manual: decide by ADR before the first `v0.1.0` tag | paul-heyse | 2026-09-25 | 2026-12-01 | open |
| R-16 | a project board and a triage rotation | ADR-0034 | a second contributor | $ `gh api repos/paul-heyse/pse-arrow/collaborators --jq length` | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-17 | Windows symlink policy for `.claude/skills -> .codex/skills` | ADR-0034 | the first Windows contributor, or a Windows CI job that checks out the agent config | $ `git config --get core.symlinks` | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-19 | Python pins for CPython 3.11: numpy 2.5.3, scipy 1.18.1 and pint 0.26.1 declare `requires-python >= 3.12`, so pyproject carries an exact second pin per library under `python_full_version < '3.12'` | ADR-0024 | `requires-python` is raised to 3.12 by ADR, or 3.11 support is dropped | manual | paul-heyse | 2026-09-25 | 2026-12-01 | open |
| R-20 | `cargo xtask doc-lint` is a stub (exit 2) and `rust / doc-lint` is `continue-on-error` until the path indexes under `docs/capability-maps/facts/` exist | ADR-0031 | the first `just evidence-regen` run that writes `docs/capability-maps/facts/*.paths.txt.zst` | $ `ls docs/capability-maps/facts` | paul-heyse | 2026-09-13 | 2026-11-01 | open |
| R-21 | the solver-image pin pull request needs `PIN_PR_TOKEN` (a fine-grained PAT with contents, pull requests and workflows write) because `GITHUB_TOKEN` cannot touch `.github/workflows`; until it is set the workflow prints the manual pin command | ADR-0028 | the secret is created, or the pin moves to a repository variable read through `vars.SOLVER_IMAGE` (decided by ADR) | $ `gh secret list --repo paul-heyse/pse-arrow` | paul-heyse | 2026-09-13 | 2026-11-01 | open |
| R-31 | dependency admission and licence policy are advisory: `deny.toml` bans nothing, its licence list is a review record rather than an allowlist, `rust / deny` is `continue-on-error` and blueprint §3.1 is not an admission list | ADR-0066 | the first crate flips to `publish = true`, or the first PyPI release — whichever comes first; a distributed artifact makes the licence of every linked dependency a question that can no longer be deferred | $ `just policy` | paul-heyse | 2026-09-14 | 2026-12-01 | open |
| R-32 | Tear policies or convergence guarantees beyond native heuristic/exact tear selection and sequential initialization (§17) | ADR-0083 | New tear policy or convergence guarantee is requested | manual: retain independent acyclicity, exhaustive small-graph and actual native initialization witnesses | paul-heyse | 2026-09-24 | 2026-12-01 | watch |
