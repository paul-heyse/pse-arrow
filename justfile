# Operational API for pse-arrow.
#
# `just --list` is the first thing to run and the contract every agent and contributor
# works against. Prefer these recipes over ad hoc commands: they own the feature
# selection, profiles, report paths and tool paths, so those can change without
# anything downstream re-learning them.
#
# Recipes are grouped by execution-cost tier:
#   env         create or inspect the working environment
#   discovery   seconds, every task
#   local       seconds to a minute, every edit
#   pr          minutes, before review
#   scheduled   minutes to hours, weekly or risk-triggered
#   decisions   ADRs, plans, the deferred-trigger register
#   mutating    CHANGES SOURCE, ENVIRONMENT OR GITHUB -- never a dependency of a gate
#
# Every gate passes `--locked`: there is no cargo config key for it, and an unlocked
# resolve would silently move a pin.

set shell := ["bash", "-euo", "pipefail", "-c"]

# The development interpreter. `.python-version` is the single source of truth (uv,
# direnv, doctor and CI all read it); earlier interpreters are a CI-matrix concern.
python := env("PSE_PYTHON", trim(read(".python-version")))
venv := env("UV_PROJECT_ENVIRONMENT", ".venv")
# Always the project's own tools, never whatever is on PATH: a stale global ruff
# silently produces a different diff than CI.
bin := venv / if os() == "windows" { "Scripts" } else { "bin" }
py := bin / if os() == "windows" { "python.exe" } else { "python" }
ruff := bin / "ruff"
pyrefly := bin / "pyrefly"
lint_imports := bin / "lint-imports"
reuse := bin / "reuse"
taplo := bin / "taplo"
typos := bin / "typos"
# Arrow's `force_validate` is a feature, not a profile: every test invocation passes it.
validate := "--features pse-relations/force-validate"
evidence_locks := "--evidence docs/capability-maps/evidence/rust/apisurface-Cargo.lock --evidence docs/capability-maps/evidence/rust/support-Cargo.lock"
solver_image := env("PSE_SOLVER_IMAGE", `python3 scripts/solver-images.py ref ci`)

default:
    @just --list --unsorted

# ---------------------------------------------------------------------- env --

[group('env')]
[doc('Full environment from nothing: venv, pinned tools, cargo tools, linters, hooks')]
bootstrap:
    ./scripts/bootstrap.sh

[group('env')]
[doc('Interpreter, Python dependencies and the editable extension only')]
bootstrap-venv:
    ./scripts/bootstrap.sh --venv-only

[group('env')]
[doc('Pinned ruff/pyrefly/import-linter/reuse/taplo/typos from [dependency-groups].quality')]
bootstrap-quality:
    ./scripts/bootstrap.sh --quality-only

[group('env')]
[doc('Pinned cargo development tools via cargo-binstall')]
bootstrap-rust-tools:
    ./scripts/bootstrap.sh --rust-only

[group('env')]
[doc('Repository linters that are plain binaries: actionlint, ast-grep, shellcheck, mdbook')]
bootstrap-linters:
    ./scripts/bootstrap.sh --linters-only

[group('env')]
[doc('Pull the solver container (Ipopt 3.14 + MUMPS + ASL); build it locally with `just solver-image`')]
bootstrap-solvers:
    docker pull {{ solver_image }}

[group('env')]
[doc('Is this working copy ready to do work? Prints the fix for anything missing.')]
doctor *args:
    @python3 scripts/doctor.py {{ args }}

[group('env')]
[doc('Machine-readable environment status, for agents and CI')]
doctor-json:
    @python3 scripts/doctor.py --format=json

[group('env')]
[doc('Fetch the pinned reading copies into external/ (idaes-pse, arrow-rs, datafusion)')]
fetch-external:
    ./scripts/fetch-external.sh

# ---------------------------------------------------------------- discovery --

[group('discovery')]
[doc('Record the exact toolchain and tool inventory to target/tooling-inventory.txt')]
versions:
    @mkdir -p target
    @{ rustc --version; cargo --version; uv --version; \
       [ -x "{{ py }}" ] && "{{ py }}" --version || echo "python: no venv"; \
       cargo install --list | grep -E '^[a-z]' ; } | tee target/tooling-inventory.txt

[group('discovery')]
[doc('Heading outline of a document (capability maps and the blueprint are large)')]
lib-outline doc:
    @grep -nE '^#{1,3} ' "{{ doc }}"

[group('discovery')]
[doc('cargo metadata for the workspace (no dependency resolution)')]
metadata:
    cargo metadata --no-deps --format-version 1

# -------------------------------------------------------------------- local --

[group('local')]
[doc('cargo check, workspace, all targets')]
check:
    cargo check --workspace --all-targets --locked

[group('local')]
[doc('clippy with -D warnings, workspace, all targets (default and --no-default-features)')]
clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo clippy --workspace --all-targets --locked --no-default-features -- -D warnings

[group('local')]
[doc('rustfmt and taplo in check mode')]
fmt-check:
    cargo fmt --all -- --check
    "{{ taplo }}" fmt --check --diff

[group('local')]
[doc('Rust tests via nextest with Arrow force_validate on')]
test *args:
    cargo nextest run --workspace --locked {{ validate }} {{ args }}

[group('local')]
[doc('Rust tests for one package')]
test-package pkg *args:
    cargo nextest run -p {{ pkg }} --locked {{ validate }} {{ args }}

[group('local')]
[doc('Doctests (nextest does not run them)')]
doctest:
    cargo test --doc --workspace --locked {{ validate }}

[group('local')]
[doc('rustdoc for the workspace with warnings as errors')]
docs-rust:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

[group('local')]
[doc('Phase-zero generated-tree hygiene; regeneration equivalence remains deferred')]
codegen-check:
    cargo xtask codegen --check

[group('local')]
[doc('One resolved version per family, equal to the pins; shared packages match the evidence lockfiles')]
family-check:
    cargo xtask family-check {{ evidence_locks }} --evidence-families-only

[group('local')]
[doc('Governance tests + codegen-check + family-check')]
governance:
    cargo xtask governance

[group('local')]
[doc('Gate: fmt-check check clippy test doctest')]
ci-fast: fmt-check check clippy test doctest

[group('local')]
[doc('Sync locked dependencies and rebuild the editable native extension with the dev profile')]
py-sync:
    uv sync --locked --extra pyomo

[group('local')]
[doc('ruff format in check mode')]
fmt-py-check:
    "{{ ruff }}" format --check

[group('local')]
[doc('ruff check (no --fix; the baseline is zero)')]
lint-py:
    "{{ ruff }}" check

[group('local')]
[doc('pyrefly type check')]
typecheck:
    "{{ pyrefly }}" check

[group('local')]
[doc('import-linter contracts (numpy/pyomo/idaes boundaries)')]
lint-imports:
    "{{ lint_imports }}"

[group('local')]
[doc('Python tests (unit + component by default; pass -m to override)')]
py-test *args:
    uv run --no-sync pytest -m "unit or component" -n auto {{ args }}

[group('local')]
[doc('Repository-config lint: taplo, typos, reuse, actionlint, zizmor, shellcheck, ast-grep')]
lint-repo:
    "{{ taplo }}" fmt --check --diff
    "{{ typos }}"
    "{{ reuse }}" lint
    actionlint
    uvx zizmor .github/workflows
    shellcheck scripts/*.sh .claude/hooks/*.sh
    ast-grep scan --config sgconfig.yml

[group('local')]
[doc('Agent configuration: symlinks resolve, every referenced doc path exists')]
lint-agents:
    python3 scripts/check_agent_config.py

[group('local')]
[doc('Python and repository quality gate')]
quality: fmt-py-check lint-py typecheck lint-imports lint-repo lint-agents setup-test solver-pin-check

# ----------------------------------------------------------------------- pr --

[group('pr')]
[doc('cargo deny (advisories, bans, licenses, sources) + cargo audit')]
policy:
    cargo deny --all-features --locked check
    cargo audit --deny warnings

[group('pr')]
[doc('Coverage via cargo-llvm-cov + nextest -> lcov.info')]
coverage:
    cargo llvm-cov nextest --workspace --locked {{ validate }} --lcov --output-path lcov.info \
      --ignore-filename-regex '(generated/|xtask/|benches/|tests/)'

[group('pr')]
[doc('Every benchmark runs once (no timing gate)')]
bench-smoke:
    cargo test --benches -p pse-benches -p pse-relations --locked {{ validate }}

[group('pr')]
[doc('Identifiers named in docs resolve in the extracted API facts')]
doc-lint:
    cargo xtask doc-lint

[group('pr')]
[doc('ADR lint, index check, and register lint')]
adr-lint:
    python3 scripts/adr.py lint
    python3 scripts/adr.py index --check
    python3 scripts/check_register.py --lint

[group('pr')]
[doc('Build the documentation book (mdBook)')]
docs:
    mdbook build docs

[group('pr')]
[doc('Serve the documentation book locally')]
docs-serve:
    mdbook serve docs --open

[group('pr')]
[doc('Parity suite against IDAES 2.12.0 in the parity environment (fails, never skips, without a solver)')]
parity *args:
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --extra pyomo --group parity --python 3.13
    UV_PROJECT_ENVIRONMENT=.venv-parity uv run --no-sync pytest --parity -m "unit or component or integration" {{ args }}

[group('pr')]
[doc('Local Rust, Python, quality and documentation gates; container parity is separate')]
ci-pr: ci-fast governance policy docs-rust bench-smoke quality adr-lint docs py-test

# ---------------------------------------------------------------- scheduled --

[group('scheduled')]
[doc('cargo hack feature powerset (depth 2) and --no-default-features')]
features-powerset:
    cargo hack check --workspace --feature-powerset --depth 2 --locked
    cargo hack check --workspace --no-default-features --locked

[group('scheduled')]
[doc('Tests under the release profile (catches optimisation-dependent paths)')]
test-release:
    cargo nextest run --workspace --locked --cargo-profile release {{ validate }}

[group('scheduled')]
[doc('Unused dependencies (informational in phase 0: the skeleton declares its blueprint dependencies before using them)')]
deps-unused:
    cargo shear || true
    cargo machete || true

[group('scheduled')]
[doc('cargo udeps on the pinned nightly')]
udeps nightly="nightly-2026-09-08":
    cargo +{{ nightly }} udeps --workspace --all-targets

[group('scheduled')]
[doc('Mutation testing for one file')]
mutants-file path:
    cargo mutants -f {{ path }} --no-shuffle -j 2

[group('scheduled')]
[doc('Unsafe surface report (cargo geiger)')]
unsafe-surface:
    cargo geiger --all-features || true

[group('scheduled')]
[doc('Would a `cargo update` raise a pre-1.0 crate floor? (dry run)')]
floors-latest:
    cargo update --dry-run --workspace 2>&1 | tail -n 40

# ---------------------------------------------------------------- decisions --

[group('decisions')]
[doc('New ADR from the template: just adr-new my-slug --title "..."')]
adr-new slug *args:
    python3 scripts/adr.py new {{ slug }} {{ args }}

[group('decisions')]
[doc('Regenerate docs/adr/README.md and the SUMMARY.md ADR block')]
adr-index:
    python3 scripts/adr.py index

[group('decisions')]
[doc('Mark one ADR superseded by another (symmetric links, status history)')]
adr-supersede old new:
    python3 scripts/adr.py supersede {{ old }} {{ new }}

[group('decisions')]
[doc('New implementation plan under docs/plans/NN-<slug>.md')]
plan slug:
    #!/usr/bin/env bash
    set -euo pipefail
    n=$(ls docs/plans | grep -E '^[0-9]{2}-' | sort | tail -n1 | cut -c1-2)
    next=$(printf '%02d' $((10#${n:-0} + 1)))
    f="docs/plans/${next}-{{ slug }}.md"
    [ -e "$f" ] && { echo "exists: $f" >&2; exit 1; }
    printf -- '---\ntitle: {{ slug }}\nstatus: draft\ndate: %s\nadrs: []\nphase: 0\n---\n\n# {{ slug }}\n\n## Context\n\n## Decisions\n\n## Plan\n\n## Verification\n\n## Open items\n\n## Outcome (recorded after implementation)\n\n### What was built\n\n### A mistake made and corrected\n\n### Deviations from the plan, deliberate\n' "$(date +%F)" > "$f"
    echo "$f"

[group('decisions')]
[doc('Rows of the deferred-trigger register that are due, with their checks run')]
register-check:
    python3 scripts/check_register.py --due

# ----------------------------------------------------------------- mutating --

[group('mutating')]
[doc('Format Rust, TOML and Python in place')]
fmt:
    cargo fmt --all
    "{{ taplo }}" fmt
    "{{ ruff }}" format

[group('mutating')]
[doc('Regenerate relations, Python contracts, docs/generated and the Ipopt bindings')]
codegen *args:
    cargo xtask codegen {{ args }}

[group('mutating')]
[doc('Accept pending insta snapshots')]
snapshots-accept:
    cargo insta accept

[group('mutating')]
[doc('Bump the workspace version, regenerate CHANGELOG.md, commit and tag')]
[confirm('Cut a release?')]
release version:
    cargo xtask release {{ version }}

[group('mutating')]
[doc('Build the solver container locally (Ipopt 3.14 + MUMPS + ASL; 10-30 minutes)')]
[confirm('Build the solver image locally?')]
solver-image target="ci":
    docker build --target {{ target }} -t pse-solvers:{{ target }}-local -f docker/solvers/Dockerfile docker/solvers

[group('mutating')]
[doc('Regenerate the capability-map evidence (rustdoc extraction, probes, exported requirements)')]
evidence-regen:
    ./scripts/evidence-regen.sh

[group('mutating')]
[doc('Sync GitHub labels from .github/labels.yml')]
[confirm('Create/update labels on GitHub?')]
labels-sync:
    ./scripts/labels-sync.sh

[group('mutating')]
[doc('Apply the declared GitHub configuration (.github/setup/*.json) idempotently')]
[confirm('Apply repository settings, rulesets and environments on GitHub?')]
gh-setup *args:
    ./scripts/gh-setup.sh {{ args }}

[group('mutating')]
[doc('Move one Python pin deliberately: just lock-upgrade <package>')]
lock-upgrade pkg:
    uv lock --upgrade-package {{ pkg }}

[group('mutating')]
[doc('Regenerate native Codex roles and materialize shared skill aliases')]
agent-config-sync:
    python3 scripts/agent-config.py

[group('local')]
[doc('Behavioral tests for repository setup and agent guards (stdlib only)')]
setup-test:
    python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v

[group('mutating')]
[doc('Update immutable solver pins and synchronize workflow/devcontainer literals')]
[confirm('Update the solver image pins?')]
solver-pin-update *args:
    python3 scripts/solver-images.py update {{ args }}

[group('local')]
[doc('Verify every literal solver image consumer matches the manifest')]
solver-pin-check:
    python3 scripts/solver-images.py check

[group('scheduled')]
[doc('Rebuild the solver libraries without Docker layer cache and compare published checksums')]
solver-rebuild-check:
    bash scripts/solver-rebuild-check.sh

[group('pr')]
[doc('Parity preflight in the pinned dev image with isolated Linux caches')]
parity-container *args:
    ./scripts/parity-container.sh {{ args }}

[group('discovery')]
[doc('Read-only comparison of live GitHub configuration with full declarations')]
gh-setup-check:
    python3 scripts/github-config.py

[group('local')]
[doc('Spelling with the project-pinned tool')]
lint-typos:
    "{{ typos }}"

[group('local')]
[doc('License headers with the project-pinned tool')]
lint-license:
    "{{ reuse }}" lint

[group('scheduled')]
[doc('Manually qualify all wheel platforms and the sdist on GitHub, without publishing')]
wheels-check ref="main":
    gh workflow run wheels.yml --ref "{{ ref }}" -f targets=all
