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

set shell := ["bash", "scripts/build-shell.sh", "-euo", "pipefail", "-c"]

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
# Listing uses the same selectors, features and build modes as execution.
nextest_action := env("PSE_NEXTEST_ACTION", "run --no-fail-fast")
solver_image := env("PSE_SOLVER_IMAGE", `python3 scripts/solver-images.py ref ci`)

default:
    @just --list --unsorted

# ---------------------------------------------------------------------- env --

[group('env')]
[doc('Full environment from nothing: venv, quality tools, cargo tools, linters, hooks')]
bootstrap:
    ./scripts/bootstrap.sh

[group('env')]
[doc('Interpreter, Python dependencies and the editable extension only')]
bootstrap-venv:
    ./scripts/bootstrap.sh --venv-only

[group('env')]
[doc('ruff/pyrefly/import-linter/reuse/taplo/typos from [dependency-groups].quality')]
bootstrap-quality:
    ./scripts/bootstrap.sh --quality-only

[group('env')]
[doc('Cargo development tools via cargo-binstall (current releases)')]
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

[group('discovery')]
[doc('Resolve the current native dependency declarations using already acquired sources')]
metadata-resolve:
    cargo metadata --offline --format-version 1

[group('discovery')]
[doc('Verify or regenerate the pinned Delta source override; pass --apply explicitly to regenerate')]
delta-source source *args:
    {{ py }} scripts/vendor-delta.py {{ source }} {{ args }}

# -------------------------------------------------------------------- local --

[group('local')]
[doc('All current-environment checks, continuing after failures; durable logs and reports in a new directory')]
[positional-arguments]
assessment output="" *args:
    #!/usr/bin/env bash
    set -euo pipefail
    assessment_output=()
    if [[ -n "$1" ]]; then
        assessment_output=(--output "$1")
    fi
    shift
    exec "{{ py }}" -m scripts.validation "${assessment_output[@]}" "$@"

[group('discovery')]
[doc('Machine-readable assessment scope and explicit environment exclusions; executes no checks')]
assessment-list *args:
    "{{ py }}" -m scripts.validation --list {{ args }}

[group('local')]
fmt-rust-check:
    cargo fmt --all -- --check

[group('local')]
lint-toml:
    "{{ taplo }}" fmt --check --diff

[group('local')]
clippy-default:
    cargo clippy --keep-going --workspace --all-targets --locked -- -D warnings

[group('local')]
clippy-no-default:
    cargo clippy --keep-going --workspace --all-targets --locked --no-default-features -- -D warnings

[group('local')]
lint-actions:
    actionlint

[group('local')]
lint-zizmor:
    uvx zizmor --offline .github/workflows

[group('local')]
lint-shell:
    shellcheck scripts/*.sh .claude/hooks/*.sh

[group('local')]
lint-ast:
    ast-grep scan --config sgconfig.yml

[group('local')]
adr-frontmatter-check:
    python3 scripts/adr.py lint

[group('local')]
adr-index-check:
    python3 scripts/adr.py index --check

[group('local')]
register-lint:
    python3 scripts/check_register.py --lint

[group('local')]
codegen-relations-check:
    "{{ py }}" -m scripts.implementation_phase guard codegen-relations-check
    cargo xtask codegen --only relations --check

[group('mutating')]
[doc('Generate Rust contracts only, without physical fixtures or compiler workflows')]
codegen-rust-contracts:
    cargo run -p xtask --no-default-features --locked -- codegen --only rust-contracts

[group('local')]
codegen-rust-contracts-check:
    cargo run -p xtask --no-default-features --locked -- codegen --only rust-contracts --check

[group('local')]
codegen-python-check:
    cargo run -p xtask --no-default-features --locked -- codegen --only python --check

[group('local')]
codegen-docs-check:
    cargo run -p xtask --no-default-features --locked -- codegen --only docs --check

[group('local')]
codegen-bindgen-check:
    cargo run -p xtask --no-default-features --locked -- codegen --only bindgen --check

[group('local')]
governance-tests *args:
    "{{ py }}" -m scripts.implementation_phase guard governance-tests
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --locked {{ validate }} {{ args }}

[group('local')]
audit-dependencies:
    "{{ py }}" -m scripts.audit_tools audit-dependencies

[group('local')]
audit-advisories:
    "{{ py }}" -m scripts.audit_tools audit-advisories

[group('local')]
audit-shear:
    "{{ py }}" -m scripts.audit_tools audit-shear

[group('local')]
audit-machete:
    "{{ py }}" -m scripts.audit_tools audit-machete

[group('local')]
features-combinations:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    "{{ py }}" -m scripts.implementation_phase guard features-combinations
    source scripts/native-solver-env.sh
    cargo hack --keep-going check --workspace --feature-powerset --depth 2 --locked

[group('local')]
features-no-default:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    "{{ py }}" -m scripts.implementation_phase guard features-no-default
    source scripts/native-solver-env.sh
    cargo hack --keep-going check --workspace --no-default-features --locked

[group('local')]
doctest-release:
    "{{ py }}" -m scripts.implementation_phase guard doctest-release
    cargo test --no-fail-fast --doc --workspace --exclude pse-py --locked --release {{ validate }}

[group('local')]
[doc('Independent Python collection/execution; fixture failures surface as component errors while unit tests continue')]
assessment-python output:
    "{{ py }}" -m scripts.implementation_phase guard assessment-python
    PSE_INSPECTION_PUBLICATION={{ quote(output / "inspection") }} uv run --no-sync pytest python/pse/tests -m "unit or component" -n auto --maxfail=0 --continue-on-collection-errors --junitxml={{ quote(output / "python.xml") }}

[group('local')]
assessment-python-unit output:
    "{{ py }}" -m scripts.implementation_phase guard assessment-python
    uv run --no-sync pytest python/pse/tests -m unit -n auto --maxfail=0 --continue-on-collection-errors --junitxml={{ quote(output / "python-unit.xml") }}

[group('local')]
assessment-python-component output:
    "{{ py }}" -m scripts.implementation_phase guard assessment-python
    PSE_INSPECTION_PUBLICATION="${PSE_INSPECTION_PUBLICATION:-{{ output }}/inspection}" uv run --no-sync pytest python/pse/tests -m component -n auto --maxfail=0 --continue-on-collection-errors --junitxml={{ quote(output / "python-component.xml") }}

[group('local')]
assessment-python-integration output:
    "{{ py }}" -m scripts.implementation_phase guard assessment-python
    PSE_INSPECTION_PUBLICATION="${PSE_INSPECTION_PUBLICATION:-{{ output }}/inspection}" uv run --no-sync pytest python/pse/tests -m integration -n auto --maxfail=0 --continue-on-collection-errors --junitxml={{ quote(output / "python-integration.xml") }}

[group('local')]
[doc('Measure current native consolidation, including diagnostic campaigns with an open acceptance barrier')]
bench-consolidation-native:
    "{{ py }}" -m scripts.implementation_phase guard bench-consolidation-native
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}"
    cargo tree -p pse-benches --locked {{ validate }} -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/force-validation-features.txt"
    cargo bench --no-fail-fast -p pse-benches --bench native_consolidation --locked {{ validate }}

[group('local')]
[doc('cargo check, workspace, all targets')]
check:
    cargo check --keep-going --workspace --all-targets --locked

[group('local')]
[doc('Compile one library during a bounded architectural replacement; no tests or dev dependencies')]
check-library pkg:
    cargo check --keep-going -p {{ pkg }} --lib --locked

[group('local')]
[doc('Compile one package and its test sources without executing tests')]
check-package pkg:
    cargo check --keep-going -p {{ pkg }} -p pse-relations --all-targets --locked {{ validate }}

[group('local')]
[doc('Compile a named Rust test target without executing tests during the architectural pivot')]
check-test pkg target:
    cargo check --keep-going -p {{ pkg }} --test {{ target }} --locked {{ validate }}

[group('local')]
[doc('clippy with -D warnings, workspace, all targets (default and --no-default-features)')]
clippy:
    python3 -m scripts.validation --group clippy

[group('local')]
[doc('rustfmt and taplo in check mode')]
fmt-check:
    python3 -m scripts.validation --group fmt-check

[group('local')]
[doc('Rust tests via nextest with Arrow force_validate on')]
[positional-arguments]
test *args:
    "{{ py }}" -m scripts.implementation_phase guard test
    cargo nextest {{ nextest_action }} --workspace --locked {{ validate }} "$@"

[group('local')]
[doc('Rust tests for one package')]
test-package pkg *args:
    "{{ py }}" -m scripts.implementation_phase guard test-package
    cargo nextest {{ nextest_action }} -p {{ pkg }} --locked {{ validate }} {{ args }}

[group('local')]
[doc('Explicitly selected Rust unit tests only; review the filter to exclude storage/compiler/solver journeys even under --lib')]
unit-package pkg filter *args:
    cargo nextest {{ nextest_action }} -p {{ pkg }} -p pse-relations --lib --locked {{ validate }} -E {{ quote(filter) }} {{ args }}

[group('local')]
[doc('Explicit isolated library-unit selection in one workspace feature graph; review the filter to exclude product journeys')]
unit-libraries filter *args:
    cargo nextest {{ nextest_action }} --workspace --lib --locked {{ validate }} -E {{ quote(filter) }} {{ args }}

[group('local')]
[doc('Isolated generated row/column codecs and allocation ownership; no compiler or storage journeys')]
unit-typed-boundaries *args:
    cargo nextest {{ nextest_action }} -p pse-engine --test typed_collection --locked {{ validate }} {{ args }}

[group('local')]
[doc('Plan 10 N00–N05 isolated units with one Cargo feature graph; no integration journeys')]
unit-contract-foundations *args:
    cargo nextest {{ nextest_action }} -p pse-ids -p pse-diagnostics -p pse-schema -p pse-relations -p pse-compiler -p pse-catalog --lib --locked {{ validate }} -E 'test(consolidation_unit::)' {{ args }}

[group('local')]
[doc('Plan 13 W00-W06 isolated foundation units; no compiler/storage/solver journeys')]
unit-rust-foundations *args:
    cargo nextest {{ nextest_action }} -p pse-buildinfo -p pse-codegen -p pse-compiler -p pse-structural -p pse-runtime -p pse-relations -p pse-columnar -p pse-engine --lib --locked {{ validate }} -E 'package(pse-columnar) and (test(diagnostic_unit::) or test(consolidation_unit::)) or test(foundation_unit) or package(pse-codegen) and test(consolidation_unit::) or package(pse-relations) and test(consolidation_unit::) or test(session::assembly::derived::integrated_performance_unit::)' {{ args }}

[group('rust')]
[doc('Resolve newly declared library profiles without updating unrelated dependency selections')]
resolve-math-profiles:
    cargo check -p pse-math

[group('rust')]
[doc('Resolve existing pinned dependencies offline after workspace edge changes')]
lock-workspace:
    cargo metadata --offline --format-version 1 > /dev/null

[group('local')]
[doc('Resolve changed Python dependency declarations without upgrading unrelated packages')]
lock-python:
    uv lock

[group('local')]
[doc('Plan 13 isolated source/governance checks, including pure regeneration')]
unit-rust-foundations-governance *args:
    "{{ py }}" -m unittest scripts.tests.test_implementation_phase
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --test no_shadow_structs --test every_crate_registered --test codegen_regeneration --test error_taxonomy --locked {{ validate }} {{ args }}

[group('local')]
[doc('Plan 10 N06 isolated engine and assurance units; no storage/compiler/solver journeys')]
dev-native-engine *args:
    cargo nextest {{ nextest_action }} -p pse-testkit -p pse-engine -p pse-relations --lib --locked {{ validate }} -E 'package(pse-testkit) and test(native_unit_) or package(pse-engine) and (test(session::config::tests::) or test(cache_service::policy::tests::) or test(session::execution::tests::))' {{ args }}

[group('local')]
[doc('Plan 10 N06 static manifest/error units; no product execution')]
dev-native-boundaries *args:
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --test every_crate_registered --test pins_match_blueprint --test dependency_floors --test error_taxonomy --locked {{ validate }} {{ args }}

[group('local')]
[doc('Check resolved N06 ownership and deletions without executing product code')]
engine-boundary-check:
    cargo metadata --format-version 1 --locked --offline | .venv/bin/python scripts/native-engine-boundaries.py

[group('local')]
[doc('Measure current isolated cold/warm/body/API builds, cache reuse and optional recovery; executes no tests')]
[positional-arguments]
bench-builds output *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    "{{ py }}" -m scripts.build_measurements "$@"

[group('local')]
[doc('Unit control for pinned Ipopt C linking and ABI widths; no solver execution')]
unit-ipopt-abi:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    cargo nextest run -p pse-ipopt-sys -p pse-relations --lib --locked --features pse-ipopt-sys/link,pse-relations/force-validate -E 'test(abi_tests::)'

[group('local')]
[doc('Compile Plan 14 native solver adapters and unit contracts; no solver journeys')]
check-solver-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    cargo check -p pse-backend-native -p pse-runtime -p pse-compiler -p pse-relations --all-targets --locked --features pse-runtime/native-solvers,pse-relations/force-validate

[group('local')]
[doc('Static lint of the linked native math, runtime and Python boundary contracts')]
lint-solver-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    cargo clippy --no-deps -p pse-backend-native -p pse-runtime -p pse-py -p pse-compiler -p pse-math --all-targets --locked --features pse-py/native-solvers,pse-relations/force-validate -- -D warnings

[group('local')]
[doc('Plan 14 callback, upload, status, ABI and lifetime units; no native convergence journeys')]
unit-native-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    cargo nextest run -p pse-backend-native -p pse-ipopt-sys -p pse-relations -p pse-compiler -p pse-structural -p pse-runtime -p pse-math --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'package(pse-backend-native) | package(pse-compiler) | package(pse-math) | test(abi_tests::) | test(flowsheet::tests::) | test(initialization::tests::) | test(math::tests::) | test(workflow::tests::)'


[group('local')]
[doc('Public native workflow declaration, lifecycle and result units; no solver journeys')]
unit-public-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    cargo nextest run -p pse-runtime -p pse-relations --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'test(workflow::tests::)'

[group('local')]
[doc('Measure existing Delta recovery and conflict controls in explicit validation modes')]
[positional-arguments]
bench-recovery mode *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    mode="$1"
    shift
    features="pse-relations/force-validate"
    case "$mode" in
      force-validation) ;;
      production)
        features=""
        export CARGO_TARGET_DIR=target/measure-production
        ;;
      *) echo "expected force-validation or production" >&2; exit 2 ;;
    esac
    export PSE_RECOVERY_MEASURE=1
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}"
    cargo tree -p pse-catalog --locked --features "$features" -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/recovery-$mode-features.txt"
    action=(run --no-fail-fast --test-threads 1)
    if [[ "${PSE_NEXTEST_ACTION:-}" == "list --message-format json" ]]; then
      action=(list --message-format json)
    fi
    exec cargo nextest "${action[@]}" --locked -p pse-catalog --test unified_delta_contracts --cargo-profile release --features "$features" -E 'test(=publication_reconciles_actual_lost_commit_acknowledgments) or test(=concurrent_publication_creation_and_parent_updates_have_one_winner) or test(=conditional_control_update_conflicts_with_a_stale_writer)' "$@"


[group('local')]
[doc('Doctests (nextest does not run them)')]
doctest:
    "{{ py }}" -m scripts.implementation_phase guard doctest
    # Cargo cannot run doctests for the pse-py cdylib target.
    cargo test --no-fail-fast --doc --workspace --exclude pse-py --locked {{ validate }}

[group('local')]
[doc('rustdoc for the workspace with warnings as errors')]
docs-rust:
    RUSTDOCFLAGS="-D warnings" cargo doc --keep-going --workspace --no-deps --locked

[group('local')]
[doc('Regenerate schema targets and pinned Ipopt bindings in scratch space and compare both ways')]
codegen-check:
    python3 -m scripts.validation --group codegen-check

[group('local')]
[doc('One resolved version per family, equal to the pins; shared packages match the evidence lockfiles')]
family-check:
    cargo run -p xtask --no-default-features --locked -- family-check

[group('local')]
[doc('Governance tests + codegen-check + family-check')]
governance:
    python3 -m scripts.validation --group governance

[group('local')]
[doc('Gate: fmt-check check clippy test doctest')]
ci-fast:
    python3 -m scripts.validation --group ci-fast

[group('local')]
[doc('Sync locked dependencies and rebuild the editable native extension with the dev profile')]
py-sync:
    uv sync --locked
    VIRTUAL_ENV="{{ absolute_path(venv) }}" "{{ bin / 'maturin' }}" develop --skip-install --profile dev --locked --features force-validate
    cargo run -p xtask --no-default-features --locked -- python-stubs

[group('local')]
[doc('Build the editable native solver workflow with its explicit linked library environment')]
py-sync-native:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    uv sync --locked --no-install-project
    VIRTUAL_ENV="{{ absolute_path(venv) }}" "{{ bin / 'maturin' }}" develop --uv --profile dev --locked --features force-validate,native-solvers
    cargo run -p xtask --no-default-features --locked -- python-stubs

[group('local')]
[doc('Compile the linked public native Python boundary with Arrow validation')]
check-native-python:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    cargo check -p pse-py --locked --features force-validate,native-solvers

[group('local')]
[doc('Targeted Python native workflow units under the explicit linked solver runtime')]
py-native-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    "{{ py }}" -m pytest python/pse/tests/test_native_workflow.py python/pse/tests/test_native_boundary_contracts.py -m unit -q

[group('codegen')]
[doc('Generate or --check the actual compiled native Python API stub; run py-sync after Rust edits')]
python-stubs *args:
    cargo run -p xtask --no-default-features --locked -- python-stubs {{ args }}

[group('local')]
[doc('ruff format in check mode')]
fmt-py-check:
    "{{ ruff }}" format --check

[group('local')]
[doc('ruff check (no --fix; the baseline is zero)')]
lint-py:
    "{{ ruff }}" check

[group('local')]
[doc('pyrefly type check, including warnings (the baseline is zero)')]
typecheck:
    "{{ pyrefly }}" check --min-severity warn

[group('local')]
[doc('import-linter contracts (numpy/pyomo/idaes boundaries)')]
lint-imports:
    "{{ lint_imports }}"

[group('local')]
[doc('Publish and reopen a fresh native store for inspection; destination must be new')]
inspection-fixture output:
    "{{ py }}" -m scripts.implementation_phase guard inspection-fixture
    cargo run --quiet --package xtask --locked {{ validate }} -- inspection-fixture {{ quote(output) }}

[group('local')]
[doc('Plan-qualified current-function architecture campaign; run after all implementation and deletions')]
[positional-arguments]
architecture-acceptance output *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
    cargo run --quiet --package xtask --locked {{ validate }} -- architecture-acceptance "$@"

[group('local')]
[doc('Python tests against a fresh native store (unit + component; pass -m to override)')]
py-test *args:
    "{{ py }}" -m scripts.implementation_phase guard py-test
    cargo run --quiet --package xtask --locked {{ validate }} -- python-tests {{ args }}

[group('local')]
[doc('Python unit tests without compiling or publishing an inspection fixture')]
py-unit *args:
    uv run --no-sync pytest --maxfail=0 --continue-on-collection-errors -m unit {{ args }}

[group('local')]
[doc('Repository-config lint: taplo, typos, reuse, actionlint, zizmor, shellcheck, ast-grep')]
lint-repo:
    python3 -m scripts.validation --group lint-repo

[group('local')]
[doc('Agent configuration: symlinks resolve, every referenced doc path exists')]
lint-agents:
    python3 scripts/check_agent_config.py

[group('local')]
[doc('Python and repository quality gate')]
quality:
    python3 -m scripts.validation --group quality

# ----------------------------------------------------------------------- pr --

[group('local')]
[doc('Dependency and licence REPORT: advisory, always exits 0, nothing here blocks a merge')]
deps-report:
    python3 -m scripts.validation --group deps-report --advisory

[group('pr')]
[doc('Opt-in strict audit: cargo deny (advisories, bans, licenses, sources) + cargo audit. Not in ci-pr')]
policy:
    python3 -m scripts.validation --group policy

[group('pr')]
[doc('Coverage via cargo-llvm-cov + nextest -> lcov.info')]
coverage output="build/coverage" *args:
    "{{ py }}" -m scripts.implementation_phase guard coverage
    mkdir -p {{ quote(output) }}
    CARGO_LLVM_COV_TARGET_DIR={{ quote(output) }} cargo llvm-cov nextest --workspace --locked {{ validate }} --profile ci --no-fail-fast --lcov --output-path {{ quote(output / "lcov.info") }} \
      --ignore-filename-regex '(generated/|xtask/|benches/|tests/)' {{ args }}

[group('pr')]
[doc('Every benchmark runs once (no timing gate)')]
bench-smoke:
    "{{ py }}" -m scripts.implementation_phase guard bench-smoke
    cargo test --no-fail-fast --benches -p pse-benches -p pse-relations --locked {{ validate }}

[group('local')]
[doc('Plan 09 final-phase cache/round/reuse measurements; never run before the implementation/deletion barrier')]
bench-cache:
    "{{ py }}" -m scripts.implementation_phase guard bench-cache
    cargo bench --no-fail-fast -p pse-benches -p pse-relations --bench native_cache --locked {{ validate }}

[group('pr')]
[doc('Identifiers named in docs resolve in the extracted API facts')]
doc-lint:
    cargo xtask doc-lint

[group('pr')]
[doc('ADR lint, index check, and register lint')]
adr-lint:
    python3 -m scripts.validation --group adr-lint

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
    "{{ py }}" -m scripts.implementation_phase guard parity
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --group parity --python 3.13
    UV_PROJECT_ENVIRONMENT=.venv-parity uv run --no-sync pytest --maxfail=0 --continue-on-collection-errors --parity -m "unit or component or integration" {{ args }}

[group('pr')]
[doc('Local Rust, Python, quality and documentation gates; container parity is separate')]
ci-pr:
    python3 -m scripts.validation --group ci-pr

# ---------------------------------------------------------------- scheduled --

[group('scheduled')]
[doc('cargo hack feature powerset (depth 2) and --no-default-features')]
features-powerset:
    python3 -m scripts.validation --group features-powerset

[group('scheduled')]
[doc('Tests under the release profile (catches optimisation-dependent paths)')]
test-release *args:
    "{{ py }}" -m scripts.implementation_phase guard test-release
    cargo nextest {{ nextest_action }} --workspace --locked --cargo-profile release {{ validate }} {{ args }}

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
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    exec "{{ py }}" -m scripts.audit_tools unsafe-surface

[group('scheduled')]
[doc('Would a `cargo update` raise a pre-1.0 crate floor? (dry run)')]
floors-latest:
    cargo update --dry-run --workspace 2>&1 | tail -n 40

# ---------------------------------------------------------------- decisions --

[group('decisions')]
[doc('New ADR from the template: just adr-new my-slug --title "..."')]
[positional-arguments]
adr-new slug *args:
    python3 scripts/adr.py new "$@"

[group('decisions')]
[doc('Regenerate docs/adr/README.md and the SUMMARY.md ADR block')]
adr-index:
    python3 scripts/adr.py index

[group('decisions')]
[doc('Mark one ADR superseded by another (symmetric links, status history)')]
[positional-arguments]
adr-supersede old new:
    python3 scripts/adr.py supersede "$@"

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
    "{{ py }}" -m scripts.implementation_phase guard codegen
    cargo xtask codegen {{ args }}

[group('mutating')]
[doc('Generate Rust contracts, rebuild their package loader, then regenerate complete outputs')]
codegen-bootstrap *args:
    "{{ py }}" -m scripts.implementation_phase guard codegen-bootstrap
    cargo run -p xtask --no-default-features -- codegen --only rust-contracts
    cargo xtask codegen {{ args }}

[group('mutating')]
[doc('Generate schema contracts and reference docs without executing physical package fixtures')]
codegen-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    cargo run -p xtask --no-default-features --locked -- codegen --only rust-contracts
    cargo run -p xtask --no-default-features --locked -- codegen --only python
    cargo run -p xtask --no-default-features --locked -- codegen --only docs

[group('mutating')]
[doc('Regenerate concrete invariant fixtures from the declared typed contracts')]
conformance-fixtures:
    cargo run --package pse-tests-conformance --example invariant_fixtures --locked {{ validate }}

[group('local')]
[doc('Compare pure invariant literal and test generation without executing any invariant query')]
conformance-fixtures-check:
    cargo run --package pse-tests-conformance --example invariant_fixtures --locked {{ validate }} -- --check

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
    python3 -m unittest discover -s scripts/tests -t . -p 'test_*.py' -v

[group('local')]
[doc('Same isolated setup controls with library-owned JUnit and selected-case evidence')]
setup-test-report output:
    "{{ py }}" -m scripts.setup_report {{ quote(output) }}

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
    "{{ py }}" -m scripts.implementation_phase guard parity-container
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


[group('local')]
[doc('Validate current acceptance source mappings without running product gates')]
architecture-manifest *args:
    "{{ py }}" -m scripts.implementation_phase check-manifest {{ args }}


[group('local')]
[doc('Measure native consolidation after current functional qualification')]
bench-consolidation:
    "{{ py }}" -m scripts.implementation_phase guard bench-consolidation
    just bench-consolidation-native

[group('local')]
[doc('Compare generated contracts without physical package fixtures or integration')]
codegen-contracts-check:
    python3 -m scripts.validation --group codegen-contracts-check

[group('local')]
[doc('Isolated source-receipt/tooling units, plan successors and functional/performance barriers')]
unit-consolidation-tools *args:
    "{{ py }}" -m unittest scripts.tests.test_validation scripts.tests.test_implementation_phase
    cargo nextest {{ nextest_action }} -p xtask --no-default-features --locked {{ validate }} -E 'test(architecture_acceptance::consolidation_unit::)' {{ args }}

[group('local')]
[doc('Static source taxonomy checks only; no runtime integration journey')]
unit-consolidation-governance *args:
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --test error_taxonomy --locked {{ validate }} {{ args }}

[group('local')]
[doc('Plan 10 N07-N08 isolated native operation and function units; no compiler/storage/solver journeys')]
dev-native-contracts *args:
    "{{ py }}" -m scripts.implementation_phase guard dev-native-contracts
    cargo nextest {{ nextest_action }} -p pse-engine -p pse-schema -p pse-relations --lib --locked {{ validate }} -E 'package(pse-engine) and (test(native_operation_unit::) or test(native_function_unit::) or test(session::contract::tests::) or test(session::round::tests::) or test(session::cache::tests::) or test(session::commands::deferred::tests::) or test(session::scalar::list_concat::tests::)) or package(pse-schema) and test(literal::consolidation_unit::)' {{ args }}

[group('local')]
[doc('Compile N07-N08 consumers and tests without executing integration journeys')]
check-native-contracts:
    cargo check --keep-going --workspace --all-targets --locked {{ validate }}

[group('discovery')]
[doc('Compile and enumerate exact workspace test identities without executing them')]
list-native-contracts:
    cargo nextest list --workspace --locked {{ validate }} --message-format json

[group('local')]
[doc('Lint the N07-N08 implementation and consumer test sources without execution')]
lint-native-contracts:
    cargo clippy --keep-going -p pse-engine -p pse-schema -p pse-relations -p pse-compiler -p pse-rules -p pse-catalog -p pse-backend-native -p pse-runtime -p pse-benches --all-targets --locked {{ validate }} -- -D warnings

[group('local')]
[doc('Lint the native data pivot and every consumer, without executing test journeys')]
lint-native-data:
    cargo clippy --keep-going --workspace --all-targets --locked {{ validate }} -- -D warnings

[group('local')]
[doc('Plan 10 N14/N15 isolated Delta contracts, bounded IO, retention and Arrow stream units; no Delta commits or publication journeys')]
dev-delta-boundaries *args:
    "{{ py }}" -m scripts.implementation_phase guard dev-delta-boundaries
    cargo nextest {{ nextest_action }} -p pse-catalog -p pse-runtime -p pse-relations --lib --locked {{ validate }} -E 'test(delta_boundary_unit::) or package(pse-catalog) and (test(delta::contract::tests::) or test(delta::layout::tests::))' {{ args }}

[group('local')]
[doc('Isolated native Python stub declaration and export consistency units')]
dev-native-boundary-tools *args:
    cargo nextest {{ nextest_action }} -p xtask --no-default-features --locked {{ validate }} -E 'test(codegen::python_stubs::)' {{ args }}

# Exhaustive generated annotation checks are explicit, not package import work.
[group('local')]
python-contracts-check:
    {{ py }} scripts/check_python_contracts.py

[group('local')]
[doc('Build the release Python extension with explicit Arrow force validation for measurements')]
py-measure-force:
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}"
    cargo tree -p pse-py --locked --features force-validate -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/python-force-validation-features.txt"
    VIRTUAL_ENV="{{ absolute_path(venv) }}" "{{ bin / 'maturin' }}" develop --skip-install --release --locked --features force-validate

[group('local')]
[doc('Build the production release Python extension without force validation in an isolated target')]
py-measure-production:
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}"
    CARGO_TARGET_DIR=target/measure-production cargo tree -p pse-py --locked -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/python-production-features.txt"
    VIRTUAL_ENV="{{ absolute_path(venv) }}" CARGO_TARGET_DIR=target/measure-production "{{ bin / 'maturin' }}" develop --skip-install --release --locked

[group('local')]
[doc('Production-equivalent native measurements without force validation, in an isolated target directory')]
bench-production:
    "{{ py }}" -m scripts.implementation_phase guard bench-consolidation-native
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production"
    CARGO_TARGET_DIR=target/measure-production cargo tree -p pse-benches --locked -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production/features.txt"
    CARGO_TARGET_DIR=target/measure-production PSE_ACCEPTANCE_OUTPUT="${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production" cargo bench --no-fail-fast -p pse-benches --bench native_cache --bench native_consolidation --locked

[group('local')]
[doc('M17-M18 native operator, derivative composition and workflow contract units')]
unit-dynamics-fitting:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    cargo nextest run -p pse-backend-native -p pse-runtime -p pse-math -p pse-compiler -p pse-relations --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'test(dynamics::) | test(fitting::) | test(workflow::tests::) | test(function_coordinates)'

[group('mutating')]
[doc('Regenerate independent PC-SAFT references in an isolated locked Python 3.12 environment')]
plan14-reference:
    UV_PROJECT_ENVIRONMENT=build/plan14-reference uv sync --locked --python 3.12 --only-group thermo-reference
    build/plan14-reference/bin/python scripts/plan14_reference.py
    build/plan14-reference/bin/python -m scripts.feos_entropy_reference

[group('local')]
[doc('Bit-exact source-to-generated physical fixture equivalence; no solver workflow')]
unit-physical-fixture:
    cargo nextest {{ nextest_action }} -p xtask --locked {{ validate }} -E 'test(codegen::physical::tests::standard_fixture_matches_yaml)'

[group('local')]
[doc('Author shared physical process declarations from explicit SI contracts and frozen independent references')]
plan14-fixtures:
    "{{ py }}" scripts/plan14_fixtures.py

[group('local')]
[doc('Discover exact native acceptance identities without running their bodies')]
plan14-discover:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
    "{{ py }}" -m scripts.plan14_acceptance discover rust-native --output build/plan14/discovery-native.json
    "{{ py }}" -m scripts.plan14_acceptance discover python-native --output build/plan14/discovery-python.json

[group('local')]
[doc('M22 native acceptance in the exact manifest feature graph; list action does not execute cases')]
plan14-native *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
    "{{ py }}" -m scripts.plan14_acceptance run rust-native {{ args }}

[group('local')]
[doc('M22 public Python process acceptance using the linked native extension')]
plan14-python output:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
    "{{ py }}" -m scripts.plan14_acceptance run python-native --junitxml="{{ output }}/plan14-python.xml"

[group('local')]
[doc('Isolated exact-witness, deletion and phase-guard controls')]
plan14-tools output:
    "{{ py }}" -m scripts.plan14_acceptance tools {{ output }}

[group('local')]
[doc('M22 complete cold/warm Criterion cost with independent process RSS; requires current functional qualification')]
plan14-measure output:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
    "{{ py }}" -m scripts.plan14_measure {{ output }}

[group('local')]
[doc('M22 current-source independent G1-G8 and PS-G1-PS-G3 review collection; missing decisions fail')]
plan14-reviews output:
    "{{ py }}" -m scripts.plan14_review {{ output }}

[group('local')]
[doc('Pure shared acceptance source-contract decoding; no runtime or native solve')]
unit-plan14-sources:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    "{{ py }}" -m pytest python/pse/tests/test_plan14_acceptance.py::test_shared_source_contracts

[group('local')]
[doc('M21 exact manifest units through the existing source-bound runner and actual reports')]
plan14-development output:
    "{{ py }}" -m scripts.plan14_acceptance development {{ quote(output) }}

[group('local')]
[doc('One unit-only development profile; invoked by the development evidence collector')]
plan14-development-run profile output *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
    if [[ "{{ profile }}" == python-tools ]]; then
      "{{ py }}" -m scripts.plan14_acceptance tools {{ quote(output) }} --unit-only
    else
      "{{ py }}" -m scripts.plan14_acceptance run {{ quote(profile) }} --unit-only {{ args }}
    fi

[group('local')]
[doc('M21 isolated new physical, structural and library-boundary unit controls')]
unit-m21:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    source scripts/native-solver-env.sh
    source scripts/native-math-env.sh
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    cargo nextest run -p pse-runtime -p pse-backend-native -p pse-kernels -p pse-math --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'test(contribution_authority_derives_equations_and_independent_closure) | test(implicit_response_checks_scaled_backward_error) | test(roots_and_nlp_admit_original_matching_by_class) | test(integrated_balances_carry_segments_and_refuse_undeclared_jumps) | test(declared_envelopes_bind_identity_and_reject_unrequested_pressure) | test(current_function_vocabulary_roundtrips_without_legacy_fallback)'

[group('local')]
[doc('M21 pure source grammar and ID framing controls after hard deletion')]
unit-m21-authoring *args:
    cargo nextest run -p pse-authoring -p pse-relations --test dsl_examples --test dsl_roundtrip --locked {{ validate }} {{ args }}
    cargo nextest run -p pse-ids -p pse-relations --lib --test golden_vectors --locked {{ validate }} -E 'package(pse-ids)' {{ args }}

[group('local')]
[doc('Run an existing recipe with the dated experimental nightly and compiler caching')]
[positional-arguments]
build-dev +args:
    python3 -m scripts.build_environment --mode nightly --cache on -- just "$@"

[group('local')]
[doc('Run an existing recipe with the canonical stable toolchain')]
[positional-arguments]
build-stable +args:
    python3 -m scripts.build_environment --mode stable -- just "$@"

[group('local')]
[doc('Run an existing recipe with compiler wrappers explicitly disabled')]
[positional-arguments]
build-uncached +args:
    python3 -m scripts.build_environment --cache off -- just "$@"

[group('local')]
[doc('Qualify sccache parser and incremental pass-through in a disposable cache')]
build-cache-probe:
    "{{ py }}" -m scripts.build_cache_probe

[group('discovery')]
[doc('Inventory build and persistent cache storage; never deletes artifacts')]
build-storage:
    "{{ py }}" -m scripts.build_storage

[group('discovery')]
[doc('Plan or verify a system LLVM migration; direct sudo --apply is explicit')]
[positional-arguments]
llvm-system *args:
    python3 scripts/llvm_system.py "$@"
