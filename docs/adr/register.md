# Deferred-decision register

Every item here was deliberately **deferred with a stated trigger** rather than
decided. A deferral that nobody revisits is an omission, so each row carries the
observable trigger, a check that can be run, an owner, and the date the check is
next due.

| Column | Meaning |
|---|---|
| `R-NN` | Row id. Cited from ADRs and from issues; ids are never reused. |
| `item` | What was deferred, in one line. |
| `ADR` | The decision record that deferred it, or `—` when the deferral is a blueprint position with no separate record. |
| `trigger` | The observable event that ends the deferral. Not a date. |
| `check` | How to tell whether the trigger has fired. A cell starting with `$ ` is a shell command that `scripts/check_register.py --due` runs and reports; anything else is a manual check. |
| `owner` | Who answers for the row. |
| `last-checked` | The date the check was last actually run. |
| `next-check` | When it is due again. Monthly for automatable checks; phase-gated otherwise. |
| `status` | `open` (deferred, waiting), `watch` (trigger may fire soon), `closed` (decided — the ADR that decided it is in the `ADR` column). |

`scripts/check_register.py --lint` runs in `governance / adr-lint`: it validates
the dates, checks that every referenced ADR exists, and fails when a row that is
not `closed` has a `next-check` in the past. `register-review.yml` runs
`--due` on a monthly cron and opens or updates one `Register review YYYY-MM`
issue from its output, including whatever the shell checks printed. A row is
closed by the ADR that decides it, never by deleting it.

| R-NN | item | ADR | trigger | check | owner | last-checked | next-check | status |
|---|---|---|---|---|---|---|---|---|
| R-01 | `salsa` for sub-pass memoization inside P7 (blueprint §14.3) | ADR-0020 | a measurement shows sub-pass granularity finer than per-instance is needed | read the §24.3 P7 compile-time benchmark for the current slice | paul-heyse | 2026-09-13 | 2027-01-15 | open |
| R-02 | `LogicalPlan::Extension` rule nodes for per-rule attribution (§14.2 rule 8) | ADR-0025 | the §22.4 diff report or agent tooling needs rule attribution inside a plan rendering | manual: is a plan rendering in the diff report on the current slice's scope? | paul-heyse | 2026-09-13 | 2027-01-15 | open |
| R-03 | `datafusion-tracing` and `instrumented-object-store` (§23.1) | ADR-0037 | a release matching the pinned DataFusion 55.1.0 exists | $ `python3 -c "import json,urllib.request as u; print(json.load(u.urlopen('https://crates.io/api/v1/crates/datafusion-tracing'))['crate']['max_version'])"` | paul-heyse | 2026-09-13 | 2026-10-13 | watch |
| R-04 | `datafusion-ffi` for compiled third-party kernel packages (§3.3, §22) | ADR-0037 | the §22 extension model admits compiled third-party kernel packages | manual: has a third-party kernel package been proposed? | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-05 | `feos-core` / `feos` provider, blocked on `num-dual` 0.15 (§9.8) | ADR-0022 | `feos-core` releases a version depending on `num-dual` 0.15 | $ `cargo info feos-core` | paul-heyse | 2026-09-13 | 2026-10-13 | watch |
| R-06 | `egglog` equality-saturation rewrites (§3.3, phase 4) | ADR-0037 | a cross-process, cross-version extraction-determinism test passes | manual: run the determinism spike before any phase-4 adoption | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-07 | declared MSRV floors of the pre-1.0 crates re-checked at every upgrade (§3.1, §26) | ADR-0018 | any `Cargo.lock` change | $ `cargo metadata --locked --format-version 1 --no-deps` | paul-heyse | 2026-09-13 | 2026-10-13 | open |
| R-08 | Ipopt / HSL / MUMPS build recipe per platform (§26, §18.3) | ADR-0028 | phase 1 exit, or a macOS or Windows recipe fails to reproduce | $ `ls docker/solvers/checksums.sha256` | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-09 | wheel solver linkage: bundle Ipopt/MUMPS versus runtime `libloading` | ADR-0028 | the first wheel that must ship the native backend | manual: does a release need `pse-py` built with the `ipopt` feature? | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-10 | `object_store` multi-writer commit coordination (§20.1, §26) | — | any deployment that is not single-writer local | manual: re-validate the ref CAS against the target object store backend | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-11 | `datafusion-proto` byte stability across DataFusion releases (§14.2 rule 5) | ADR-0019 | any DataFusion version bump | $ `git log --oneline -5 -- Cargo.lock` | paul-heyse | 2026-09-13 | 2026-10-13 | open |
| R-12 | Miri re-adoption (dropped: no `unsafe` outside the FFI crates) | — | the first `unsafe` block outside the four allowlisted crates | $ `cargo tree --workspace --depth 0` | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-13 | Renovate as the fallback for Dependabot | ADR-0035 | Dependabot cannot express a grouping the family rule needs | manual: has a family-group PR arrived ungrouped? | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-14 | Python API documentation tool (mkdocstrings or equivalent) | ADR-0036 | `python/pse` grows a public surface beyond `open`/`compile`/`solve`/`build_info` | $ `ls python/pse` | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-15 | crates.io release tooling: `release-plz` versus `cargo-workspaces` | ADR-0002 | phase-0 exit, when the library crates first flip `publish` | manual: decide by ADR before the first `v0.1.0` tag | paul-heyse | 2026-09-13 | 2026-12-01 | open |
| R-16 | a project board and a triage rotation | ADR-0034 | a second contributor | $ `gh api repos/paul-heyse/pse-arrow/collaborators --jq length` | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-17 | Windows symlink policy for `.claude/skills -> .codex/skills` | ADR-0034 | the first Windows contributor, or a Windows CI job that checks out the agent config | $ `git config --get core.symlinks` | paul-heyse | 2026-09-13 | 2027-03-01 | open |
| R-18 | promote `cargo shear` / `cargo machete` from advisory to a PR gate | ADR-0038 | every crate has code — the phase-0 skeleton declares its blueprint dependencies before using them, so unused-dependency tools would fail the whole workspace | $ `cargo shear` | paul-heyse | 2026-09-13 | 2026-12-01 | open |
