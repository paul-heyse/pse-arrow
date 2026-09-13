# CI job graph

**Owner:** `paul-heyse`. Every job below has one owner, and every *scheduled*
job has a documented response for when it goes red — a red scheduled job that
nobody has agreed to answer is a notification, not a control.

Workflows: `rust.yml`, `rust-scheduled.yml`, `python.yml`, `wheels.yml`,
`parity.yml`, `solvers-image.yml`, `release.yml`, `docs.yml`, `governance.yml`,
`register-review.yml`, `repo-hygiene.yml`, plus the composite action
`.github/actions/setup-rust`.

Common to every workflow: `permissions: contents: read` at the top, widened per
job; per-ref concurrency with `cancel-in-progress` on pull requests only;
`CARGO_INCREMENTAL=0`, `CARGO_TERM_COLOR=always`, `RUST_BACKTRACE=1`; tool
versions as environment variables; the solver image referenced by digest
(`SOLVER_IMAGE=ghcr.io/paul-heyse/pse-solvers:ci-<hash>@sha256:<digest>`); no
`RUSTFLAGS`; sccache off; every action pinned by commit SHA with a version
comment, kept current by Dependabot and `pinact`.

## Required checks on `main`

| Check context | Job | Runs |
|---|---|---|
| `rust / fmt` | fmt | `cargo fmt --all --check`; `taplo fmt --check` |
| `rust / clippy` | clippy (solver container) | `cargo clippy --workspace --all-targets --locked -- -D warnings`, then again with `--no-default-features` |
| `rust / test` | test (solver container) | `cargo nextest run --workspace --locked --profile ci --features pse-relations/force-validate`; `cargo test --doc --workspace --locked --features pse-relations/force-validate`; `cargo test --benches -p pse-benches -p pse-relations --locked --features pse-relations/force-validate`; JUnit artifact |
| `rust / codegen-diff` | codegen-diff (container: Ipopt headers + libclang) | `cargo xtask codegen --check` |
| `rust / family-check` | family-check | `cargo xtask family-check --evidence docs/capability-maps/evidence/rust/*.lock`; `cargo metadata --locked` |
| `rust / deny` | deny | `cargo deny check`; `cargo audit`; `cargo shear` |
| `python / lint` | lint | `uv sync --locked --group quality --no-install-project`; ruff format/check, pyrefly, import-linter, `reuse lint`, typos, taplo, `uv lock --check`, `scripts/check_generated.py` |
| `python / test` | test (3.11 and 3.14) | maturin-backed `uv sync`; `pytest -m "unit or component" -n auto --cov` |
| `python / parity` | parity (container `ci` stage; 3.13 on PRs, 3.11–3.13 nightly) | installs the Linux x86_64 wheel artifact, `uv run --no-sync pytest --parity` |
| `docs / build` | build | `scripts/adr.py index --check`; `mdbook build docs`; `lychee --offline`; Pages artifact |
| `governance / adr-lint` | adr-lint | `scripts/adr.py lint`; `scripts/check_register.py --lint`; the `needs-adr` label rule; the IDAES tag in `scripts/fetch-external.sh` equals the parity pin |
| `governance / pr-title` | pr-title | Conventional-Commit title check, types and scopes from `cliff.toml` |

The full set above is declared in `.github/setup/ruleset-main-full.json`.
`just gh-setup` applies it; `just gh-setup-check` verifies the live configuration.
Check names are explicit job names, including the aggregate Python test result.
A newly declared check must report before its ruleset is activated.

## Run on pull requests, not required

`rust / docs` (`RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps`), `rust / doc-lint`
(paths-filtered), `rust / coverage` (`cargo llvm-cov nextest … --lcov`, artifact upload),
`rust / coverage-upload` (host-runner Codecov OIDC, informational in phase 0), `rust / semver` (from the first tag; gating
only on PRs labelled `release`), `rust / msrv-floors` (`cargo hack check
--rust-version`, only when `Cargo.lock` changed), `rust / test-macos` and
`rust / test-windows` (phase 1b, `continue-on-error` until a green week),
`wheels / *` (paths-filtered), `governance / reuse`, `governance / agent-config`,
`repo-hygiene / *` (`actionlint`, `zizmor`, `pinact`). `docs / deploy` runs on
`main` through `actions/deploy-pages`.

## Scheduled jobs, and what to do when one goes red

`rust-scheduled.yml` runs weekly (cron `0 6 * * 1`). None of these gates a pull
request; each exists to find a problem before an upgrade does.

| Job | Owner | Red means | Documented response |
|---|---|---|---|
| `features-powerset` (`cargo hack --feature-powerset --depth 2`) | paul-heyse | a feature combination does not compile — most often an optional dependency used outside its `#[cfg]` | Reproduce with the printed feature set, fix the `cfg` or the optional dependency, and add the combination to the PR-time `--no-default-features` clippy run if it is load-bearing. Open a `kind/bug` issue the same week; do not disable the job. |
| `test-release` (`--cargo-profile release`) | paul-heyse | a test passes at `dev` optimization and fails at `release` — a float-sensitivity or an undefined-order assumption | Treat as a correctness bug, not a flake. Reproduce locally with `just test-release`, fix the test or the code, and record the cause; a release-only failure in numerics is exactly the class ADR-0030 and the `force_validate` feature exist to surface. |
| `udeps` (pinned nightly) | paul-heyse | a declared dependency is unused | Remove it, or record it in `tests/governance/tooling_deps.toml` with a reason. During phase 0 the skeleton declares blueprint dependencies before using them, which is why this is scheduled rather than gating (register row R-18). |
| `mutants` on `pse-ids` and `pse-mathir` (`continue-on-error`) | paul-heyse | surviving mutants in the identity or IR crates — a test suite that does not discriminate | Read the artifact, add tests for the survivors that matter, and note the ones deliberately left (a mutant in a `Debug` impl is not worth a test). Never treat the count as a gate. |
| `unsafe-surface` (`cargo geiger`) | paul-heyse | `unsafe` outside the four allowlisted crates, or a growth inside them | Check it against `[workspace.metadata.pse] unsafe-allowlist`; if it is genuinely new non-FFI `unsafe`, the Miri deferral's trigger has fired (register row R-12) and the response is an ADR, not a suppression. |
| `doc-lint` (full) | paul-heyse | a backticked `a::b::c` or `[rustdoc:…]` identifier in `docs/**` no longer resolves | Usually an upstream rename. Fix the document, or add the identifier to the allowlist with a reason. If a capability map is affected, regenerate its evidence rather than editing the claim. |
| `floors-latest` (`cargo update --dry-run` + floors against a temp lock) | paul-heyse | a newer version of a pre-1.0 crate raises its declared MSRV above ours, or a family would go mixed | This is the early warning ADR-0018 is built around. Record it on register row R-07 with the date, and decide the upgrade deliberately — bump the toolchain by ADR, or hold the pin. |
| `solvers-image-rebuild-check` (no cache) | paul-heyse | the solver recipe no longer reproduces the same library checksums | Diff the build log against the last good one, find the moving input (a base-image digest, an upstream tarball, a compiler version), pin it in `docker/solvers/`, and update `checksums.sha256`. Until it reproduces, treat parity iteration counts as unverified (register row R-08). |

`solvers-image.yml` builds the solver container on every pull request that touches
`docker/solvers/**` and, on `main`, pushes it to GHCR and opens the pull request that
moves the `SOLVER_IMAGE` pin (a digest reference inside the workflow files, so a
checkout is reproducible on its own). Opening that pull request needs the
`PIN_PR_TOKEN` secret: a fine-grained personal access token scoped to this
repository with contents, pull requests and workflows write, because the default
`GITHUB_TOKEN` cannot create or update files under `.github/workflows`. Without the
secret the job succeeds, uploads the image manifest, and prints the exact
`just solver-pin-update --ci … --dev …` command (register row R-21).
The canonical pins are in `.github/setup/solver-images.json`, outside the solver
recipe tree hash. `just solver-pin-check` checks every literal consumer.
`just solver-rebuild-check` repeats the no-cache library comparison locally.

`register-review.yml` runs monthly with `issues: write`, executes
`scripts/check_register.py --due` — including the shell checks in the `check`
column — and opens or updates one `Register review YYYY-MM` issue. **Red means
the workflow itself failed**, not that a check reported something: `--due`
reports a failing command and never fails on it. The response is to run it
locally, then to act on each due row in the issue before closing it.

`codecov.yml` is informational, with `after_n_builds` equal to the number of
uploads; coverage never blocks a merge in phase 0.

## Local equivalents

| CI tier | Local recipe | What it proves |
|---|---|---|
| fast feedback | `just ci-fast` (fmt-check, check, clippy, test, doctest) | The workspace compiles and the unit and component tests pass at `dev`. |
| pull-request gate | `just ci-pr` (adds governance, policy, docs, bench-smoke, quality, adr-lint, py-test) | Local Rust/Python and repository checks; container parity is separate. |
| parity preflight | `just parity-container` | Existing IDAES/solver preflight cases in the pinned dev image, using a separate Python 3.13 environment. |
| scheduled | `just features-powerset`, `just test-release`, `just udeps`, `just mutants-file`, `just unsafe-surface`, `just floors-latest` | Individually reproduces one weekly job. |

`just --list` is the contract: if a check exists in CI and has no local recipe,
that is a bug in the justfile.

## Phase-zero limits

Code generation and API-reference doc lint remain deferred (register R-20).
PR-time API doc lint emits a deferral notice when the path indexes are absent;
its CLI remains an exit-2 stub and scheduled checks retain the register trigger.
`codegen-check` detects staged, unstaged and untracked generated-path changes; it
does not prove a real schema or binding generator exists. The parity suite currently
qualifies its environment and package boundaries, not numerical model parity.

The wheel workflow supports manual, non-publishing qualification. It builds all
five native platform targets, checks `cp311-abi3` tags, installs each wheel under
Python 3.11 and 3.14 outside the source tree, and builds/installs the sdist.
