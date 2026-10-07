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
#   manual      optional checks, from minutes to hours; run when chosen
#   decisions   ADRs, plans, the deferred-trigger register
#   mutating    CHANGES SOURCE, ENVIRONMENT OR GITHUB
#
# Agents run compile checks and functional tests mid-plan. Three bundles keep going after a failure
# and list what failed: `turn-end` at the end of a turn that changed files, `ready` after an
# environment change, and `hygiene` once at scope end.
#
# Every check passes `--locked`: there is no cargo config key for it, and an unlocked
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
[doc('Full environment from nothing: venv, quality tools, cargo tools, linters')]
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
[doc('Repository linters that are plain binaries: actionlint, ast-grep, shellcheck')]
bootstrap-linters:
    ./scripts/bootstrap.sh --linters-only

[group('env')]
[doc('Pull the solver container (Ipopt 3.14 + MUMPS/SPRAL/oneMKL + SCIP 10); build it locally with `just solver-image`')]
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

[group('env')]
[doc('Set up or control the supervised authenticated RocksDB/gRPC canonical substrate')]
surreal command *args:
    "{{ py }}" scripts/surreal_server.py {{ quote(command) }} {{ args }}

[group('env')]
[doc('Initialize a clean canonical database; ordinary runtime opening never installs schema')]
canonical-init state:
    PSE_NATIVE_CAPABILITIES='' bash scripts/native_exec.sh cargo run -p xtask --no-default-features --features canonical-tools --locked -- canonical-init {{ quote(state) }}

[group('local')]
[doc('Targeted canonical server supervisor mechanism controls')]
surreal-test:
    "{{ py }}" -m unittest scripts.tests.test_surreal_server -v

[group('local')]
[doc('Disposable acknowledged-write, abrupt restart and offline backup/restore server control')]
surreal-fixture-test:
    "{{ py }}" -m scripts.tests.surreal_fixture_check

[group('local')]
[doc('Released-server native canonical schema, source/product and gated offline recovery control')]
canonical-recovery-test *args:
    PSE_NATIVE_CAPABILITIES='' bash scripts/native_exec.sh cargo build -p xtask --bin pse-canonical-recovery --no-default-features --features canonical-tools,pse-relations/force-validate --locked
    "{{ py }}" -m scripts.tests.canonical_recovery_check target/debug/pse-canonical-recovery {{ args }}

[group('local')]
[doc('Native gRPC exact codec and guarded revision controls against a supervised server state')]
canonical-test state:
    PSE_SURREAL_STATE={{ quote(state) }} just unit-package pse-operations 'test(canonical_server_unit) | test(canonical_codec_unit)' --features pse-operations/canonical-tests

[group('local')]
[doc('Forced-validation persisted mathematical reconstruction controls on an isolated gRPC database')]
canonical-portable-test state:
    #!/usr/bin/env bash
    set -euo pipefail
    fixture_receipt="$PWD/target/producer-qualified-fixture.json"
    bash scripts/native_exec.sh cargo run -p xtask --no-default-features --locked -- producer-identity-fixture --output "$fixture_receipt"
    PSE_PRODUCER_FIXTURE_RECEIPT="$fixture_receipt" PSE_SURREAL_STATE={{ quote(state) }} NEXTEST_TEST_THREADS=8 just unit-native-package pse-runtime 'canonical-tests,pse-relations/force-validate' 'test(canonical_portable_body)'

[group('local')]
[doc('Actual finite production capture prerequisite for persisted replay controls')]
producer-fixture output:
    bash scripts/native_exec.sh cargo run -p xtask --bin xtask --no-default-features --locked -- producer-identity-fixture --output {{ quote(output) }}

[group('local')]
[doc('Derive a relevant production-unit identity; unknown inputs disable persistent reuse')]
[positional-arguments]
producer-identity *args:
    #!/usr/bin/env bash
    set -euo pipefail
    # Cargo's run launcher injects tool-local loader paths. Build the capture tool
    # separately so the recorded caller is the ordinary native deployment context.
    bash scripts/build-shell.sh -c 'cargo build -p xtask --bin xtask --no-default-features --locked'
    exec bash scripts/native_exec.sh "$PWD/target/debug/xtask" producer-identity "$@"

[group('local')]
[doc('Focused producer identity tests including irrelevant test edits and consumed-input changes')]
producer-identity-test:
    bash scripts/native_exec.sh cargo nextest run -p xtask --bin xtask --no-default-features --locked {{ validate }} -E 'test(producer_)'

[group('local')]
[doc('Capture reviewed runtime, worker and Python deployment targets after producer-profile installation')]
[positional-arguments]
producer-deployment output:
    # Keep the deployment operation and all nested captures under one setup owner.
    bash scripts/native_exec.sh "{{ py }}" -m scripts.producer_deployment "$1"

[group('local')]
[doc('Producer deployment, native runner and assessment admission mechanism controls')]
producer-deployment-unit:
    "{{ py }}" -m unittest scripts.tests.test_producer_deployment scripts.tests.test_native_tests scripts.tests.test_validation

[group('local')]
[doc('Generator identity, unchanged-output and physical fixture controls')]
codegen-unit-test:
    PSE_NATIVE_CAPABILITIES='compiler,solver' bash scripts/native_exec.sh cargo nextest run -p xtask --bin xtask --locked --features package-fixtures {{ validate }} -E 'test(codegen::tests::) | test(codegen::physical::tests::)'

[group('local')]
[doc('Focused explicit Python target selection controls without solver discovery')]
python-runner-unit-test:
    cargo nextest run -p xtask --bin xtask --no-default-features --locked {{ validate }} -E 'test(python_selection_)'

[group('local')]
[doc('Worker CLI admission budgets against the configured managed process allocation')]
worker-cli-unit-test:
    bash scripts/native_exec.sh cargo nextest run -p xtask --bin pse-worker --locked --features native-solvers {{ validate }}

[group('local')]
[doc('Miri controls for the pure qualified scientific reconstruction authority')]
scientific-replay-miri:
    cargo miri test -p pse-ids -p pse-relations --lib --locked {{ validate }} scientific_replay::tests

[group('local')]
[doc('Run canonical execution actions in a supervised native worker; e.g. --until-idle --maximum-actions 100')]
pse-worker *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo build --locked -p xtask --bin pse-worker --features native-solvers
    "{{ py }}" scripts/surreal_server.py worker --worker-command "$PWD/target/debug/pse-worker" {{ args }}

[group('local')]
[doc('The pse-worker journey: the worker binary runs an authored case in a child process against an isolated store')]
worker-test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    bash scripts/native_exec.sh cargo build -p xtask --bin pse-worker --locked --features native-solvers
    PSE_WORKER_BINARY="$PWD/target/debug/pse-worker" bash scripts/native_exec.sh cargo nextest {{ nextest_action }} -p pse-runtime --test worker --locked --features pse-runtime/native-solvers,pse-relations/force-validate {{ args }}

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
    exec bash scripts/native_exec.sh "{{ py }}" -m scripts.validation "${assessment_output[@]}" "$@"

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
    shellcheck scripts/*.sh

[group('local')]
lint-ast:
    ast-grep test --config sgconfig.yml --skip-snapshot-tests
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
[doc('The canonical native schema and codecs match a fresh regeneration')]
codegen-surreal-check:
    cargo run -p xtask --no-default-features --locked -- codegen --only surreal --check

[group('local')]
codegen-bindgen-check:
    cargo run -p xtask --no-default-features --locked -- codegen --only bindgen --check

[group('mutating')]
[doc('Regenerate the workspace feature owner after dependency retirement or changes')]
codegen-hakari:
    cargo hakari generate
    cargo hakari manage-deps --yes

[group('local')]
[doc('The cargo-hakari workspace-hack matches a fresh generation and every managed member depends on it (ADR-0122)')]
codegen-hakari-check:
    cargo hakari generate --diff
    cargo hakari manage-deps --dry-run

[group('local')]
[doc('The published document and authoring JSON Schemas and the Python document types match a fresh derivation from the owning crates (ADR-0116)')]
codegen-schemas-check:
    PSE_NATIVE_CAPABILITIES='' bash scripts/native_exec.sh cargo run -p xtask --locked -- codegen --only schemas --check

[group('local')]
governance-tests *args:
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
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    export CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo_feature_check.py"
    export PATH="$PWD/.venv/bin:$PATH"
    cargo hack --keep-going check --workspace --feature-powerset --depth 2 --locked

[group('local')]
features-no-default:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    export CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo_feature_check.py"
    export PATH="$PWD/.venv/bin:$PATH"
    cargo hack --keep-going check --workspace --no-default-features --locked

[group('local')]
doctest-release:
    cargo test --no-fail-fast --doc --workspace --exclude pse-py --locked --release {{ validate }}

[group('local')]
[doc('Measure current native consolidation, including diagnostic campaigns with an open acceptance barrier')]
bench-consolidation-native:
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
[doc('Compile one linked native test target without executing its journeys')]
check-native-test pkg target features:
    bash scripts/native_exec.sh cargo check -p {{ pkg }} --test {{ target }} --locked --features {{ features }},pse-relations/force-validate

[group('local')]
[doc('Compile selected native package targets without replacing running test executables')]
check-native-package pkg features capabilities="solver,klu,isolation,uno,petsc":
    PSE_NATIVE_CAPABILITIES={{ quote(capabilities) }} bash scripts/native_exec.sh cargo check -p {{ pkg }} -p pse-relations --all-targets --locked --features {{ features }},pse-relations/force-validate

[group('local')]
[doc('Run selected functional controls in one Rust test target with explicit Arrow validation')]
functional-test-target pkg target filter *args:
    bash scripts/memory-cap.sh cargo nextest {{ nextest_action }} -p {{ pkg }} -p pse-relations --test {{ target }} --locked {{ validate }} -E {{ quote(filter) }} {{ args }}

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
    cargo nextest {{ nextest_action }} --workspace --locked {{ validate }} "$@"

[group('local')]
[doc('Rust tests for one package')]
[positional-arguments]
test-package pkg *args:
    cargo nextest {{ nextest_action }} -p "$1" --locked {{ validate }} "${@:2}"

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
[doc('Isolated contract-foundation units with one Cargo feature graph; no integration journeys')]
unit-contract-foundations *args:
    cargo nextest {{ nextest_action }} -p pse-ids -p pse-diagnostics -p pse-schema -p pse-relations -p pse-compiler --lib --locked {{ validate }} -E 'test(consolidation_unit::)' {{ args }}

[group('local')]
[doc('Isolated Rust foundation units; no compiler/storage/solver journeys')]
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
[doc('Isolated source/governance ownership and error contract checks')]
unit-rust-foundations-governance *args:
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --test no_shadow_structs --test every_crate_registered --test error_taxonomy --locked {{ validate }} {{ args }}

[group('local')]
[doc('Isolated engine and assurance units; no storage/compiler/solver journeys')]
dev-native-engine *args:
    cargo nextest {{ nextest_action }} -p pse-testkit -p pse-engine -p pse-relations --lib --locked {{ validate }} -E 'package(pse-testkit) and test(native_unit_) or package(pse-engine) and (test(session::config::tests::) or test(cache_service::policy::tests::) or test(session::execution::tests::))' {{ args }}

[group('local')]
[doc('Static manifest/error governance units; no product execution')]
dev-native-boundaries *args:
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --test every_crate_registered --test dependency_pins --test dependency_floors --test error_taxonomy --locked {{ validate }} {{ args }}

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
    "{{ py }}" -m scripts.build_measurements "$@"

[group('local')]
[doc('Bounded native generation, operation admission and surviving-child lifecycle units')]
native-setup-unit:
    "{{ py }}" -m unittest scripts.tests.test_native_operation scripts.tests.test_build_environment scripts.tests.test_highs_provider_candidate -v

[group('local')]
[doc('Unit control for pinned Ipopt C linking and ABI widths; no solver execution')]
unit-ipopt-abi:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    exec bash scripts/memory-cap.sh "{{ py }}" -m scripts.native_operation --capabilities solver -- cargo nextest run -p pse-ipopt-sys -p pse-relations --lib --locked --features pse-ipopt-sys/link,pse-relations/force-validate -E 'test(abi_tests::)'

[group('local')]
[doc('Prepare source-pinned validated root isolation with FILIB and bundled SoPlex')]
native-isolation-prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "isolation" "$@"
    printf '%s\n' "$PSE_ROOT_ISOLATION_DIR"

[group('local')]
[doc('Prepare pinned Uno or PETSc scoped foreign inputs against the existing native provider')]
native-pipeline-prepare kind:
    #!/usr/bin/env bash
    set -euo pipefail
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    source scripts/native-recipe-env.sh {{ quote(kind) }} "$@"
    case {{ quote(kind) }} in
      uno) printf '%s\n' "$UNO_DIR" ;;
      petsc) printf '%s\n' "$PETSC_DIR" ;;
    esac

[group('local')]
[doc('Bounded unit controls for a pinned scoped Uno or PETSc project ABI')]
[positional-arguments]
unit-pipeline-binding kind *args:
    #!/usr/bin/env bash
    set -euo pipefail
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    source scripts/native-recipe-env.sh {{ quote(kind) }} "$@"
    shift
    bash scripts/memory-cap.sh cargo nextest run --no-fail-fast -p "pse-{{ kind }}-sys" -p pse-relations --lib --locked --features "pse-{{ kind }}-sys/link,pse-relations/force-validate" -E 'package(pse-{{ kind }}-sys)' "$@"

[group('local')]
[doc('Compile a scoped native pipeline adapter with its qualified foreign prefix')]
check-pipeline-native kind:
    #!/usr/bin/env bash
    set -euo pipefail
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    source scripts/native-recipe-env.sh {{ quote(kind) }} "$@"
    cargo check --keep-going -p pse-backend-native -p pse-runtime -p pse-relations --all-targets --locked --features 'pse-backend-native/{{ kind }},pse-relations/force-validate'

[group('local')]
[doc('Targeted safe native pipeline adapter controls with a qualified foreign prefix')]
[positional-arguments]
unit-pipeline-native kind filter *args:
    #!/usr/bin/env bash
    set -euo pipefail
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    source scripts/native-recipe-env.sh {{ quote(kind) }} "$@"
    shift 2
    bash scripts/memory-cap.sh cargo nextest run --no-fail-fast -p pse-backend-native -p pse-relations --lib --locked --features 'pse-backend-native/{{ kind }},pse-relations/force-validate' -E {{ quote(filter) }} "$@"

[group('local')]
[doc('Compile native solver adapters and unit contracts; no solver journeys')]
check-solver-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo check --keep-going -p pse-backend-native -p pse-runtime -p pse-compiler -p pse-relations --all-targets --locked --features pse-runtime/native-solvers,pse-relations/force-validate

[group('local')]
[doc('Static lint of the linked native workspace, conformance and benchmark consumers')]
lint-solver-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo clippy --keep-going --no-deps --workspace --all-targets --locked --features pse-py/native-solvers,pse-tests-conformance/native-acceptance,pse-benches/native-process,pse-relations/force-validate -- -D warnings

[group('local')]
[doc('Native callback, upload, status, ABI and lifetime units; no native convergence journeys')]
unit-native-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo nextest run -p pse-backend-native -p pse-ipopt-sys -p pse-relations -p pse-compiler -p pse-structural -p pse-runtime -p pse-math --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'package(pse-backend-native) | package(pse-compiler) | package(pse-math) | test(abi_tests::) | test(flowsheet::tests::) | test(initialization::tests::) | test(math::tests::) | test(workflow::tests::)'


[group('local')]
[doc('Public native workflow declaration, lifecycle and result units; no solver journeys')]
unit-public-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo nextest run -p pse-runtime -p pse-relations --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'test(workflow::tests::)'

[group('local')]
[doc('Doctests (nextest does not run them)')]
doctest:
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
[doc('Optional aggregate: fmt-check check clippy test doctest')]
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
py-sync-native profile="dev":
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    uv sync --locked --no-install-project
    VIRTUAL_ENV="{{ absolute_path(venv) }}" "{{ bin / 'maturin' }}" develop --uv --profile {{ quote(profile) }} --locked --features force-validate,native-solvers
    cargo run -p xtask --no-default-features --locked -- python-stubs

[group('local')]
[doc('Compile the linked public native Python boundary with Arrow validation')]
check-native-python:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo check -p pse-py --locked --features force-validate,native-solvers

[group('local')]
[doc('Targeted Python native workflow units under the explicit linked solver runtime')]
py-native-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver" "$@"
    export LD_LIBRARY_PATH="$IPOPT_DIR/lib:${LD_LIBRARY_PATH:-}"
    "{{ py }}" -m pytest python/pse/tests/test_native_workflow.py python/pse/tests/test_native_boundary_contracts.py -m unit -q

[group('codegen')]
[doc('Generate or --check the actual compiled native Python API stub; run py-sync after Rust edits')]
python-stubs *args:
    bash scripts/native_exec.sh cargo run -p xtask --no-default-features --locked -- python-stubs {{ args }}

[group('local')]
[doc('ruff format in check mode')]
fmt-py-check:
    "{{ ruff }}" format --config pyproject.toml --check

[group('local')]
[doc('ruff check (no --fix; the baseline is zero)')]
lint-py:
    "{{ ruff }}" check --config pyproject.toml

[group('local')]
[doc('pyrefly type check, including warnings (the baseline is zero)')]
typecheck:
    "{{ pyrefly }}" check --min-severity warn

[group('local')]
[doc('import-linter contracts (numpy/pyomo/idaes boundaries)')]
lint-imports:
    "{{ lint_imports }}"



[group('local')]
[doc('Python tests against a fresh native store (unit + component; pass -m to override)')]
[positional-arguments]
py-test *args:
    cargo run --quiet --package xtask --locked {{ validate }} -- python-tests "$@"

[group('local')]
[doc('Python unit tests without compiling or publishing an inspection fixture')]
[positional-arguments]
py-unit *args:
    uv run --no-sync pytest --maxfail=0 --continue-on-collection-errors -m unit "$@"

[group('local')]
[doc('Targeted Python functional units with the linked native solver environment')]
[positional-arguments]
py-unit-native *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    bash scripts/memory-cap.sh "{{ py }}" -m pytest --maxfail=0 --continue-on-collection-errors -m unit "$@"

[group('local')]
[doc('Run data-authored modeling fixtures and shared conformance checks; accepts Python module CLI arguments')]
modeling-conformance *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    bash scripts/memory-cap.sh "{{ py }}" -m pse.conformance {{ args }}

[group('local')]
[doc('Run every reference fixture once from packages/reference/conformance.toml; Arrow reports in build/seed-conformance')]
seed-conformance:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    bash scripts/memory-cap.sh "{{ py }}" -m pse.conformance --manifest packages/reference/conformance.toml --report-dir build/seed-conformance

[group('local')]
[doc('Repository-config lint: taplo, typos, reuse, actionlint, zizmor, shellcheck, ast-grep')]
lint-repo:
    python3 -m scripts.validation --group lint-repo

turn_end_steps := "adr-index fmt"
ready_steps := "skills-sync doctor"
hygiene_checks := "lint-agents adr-frontmatter-check adr-index-check register-lint lint-typos lint-license lint-actions lint-shell lint-ast lint-py typecheck lint-imports engine-boundary-check solver-pin-check family-check codegen-hakari-check codegen-relations-check codegen-rust-contracts-check codegen-docs-check codegen-surreal-check clippy-default clippy-no-default docs-rust"

[group('mutating')]
[doc('End of a turn that changed files (root agent): regenerate the ADR index and format')]
turn-end: (_bundle "turn-end" turn_end_steps)

[group('env')]
[doc('After a dependency, toolchain or skill-selection change, or an environment-shaped failure: sync library skills and run doctor')]
ready: (_bundle "ready" ready_steps)

[group('manual')]
[doc('Scope end: every non-functional check; keeps going and lists failures; fix them and re-run one with just <id>')]
hygiene: (_bundle "hygiene" hygiene_checks)

# Run each recipe, keep going after a failure, and list the failures.
[private]
_bundle name steps:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    for step in {{ steps }}; do
        echo "== just $step"
        just "$step" || failed+=("$step")
    done
    if [ ${#failed[@]} -gt 0 ]; then
        echo "{{ name }}: ${#failed[@]} failed: ${failed[*]} (re-run one with just <id>)"
        exit 1
    fi
    echo "{{ name }}: all passed"

[group('local')]
[doc('Agent configuration: symlinks resolve, every referenced doc path exists')]
lint-agents:
    python3 scripts/check_agent_config.py

[group('local')]
[doc('Python and repository quality gate')]
quality:
    python3 -m scripts.validation --group quality

# ------------------------------------------------------------------- manual --

[group('local')]
[doc('Dependency and licence REPORT: advisory, always exits 0, nothing here blocks a merge')]
deps-report:
    python3 -m scripts.validation --group deps-report --advisory

[group('manual')]
[doc('Opt-in strict audit: cargo deny (advisories, bans, licenses, sources) + cargo audit. Not in ci-pr')]
policy:
    python3 -m scripts.validation --group policy

[group('manual')]
[doc('Coverage via cargo-llvm-cov + nextest -> lcov.info')]
coverage output="build/coverage" *args:
    mkdir -p {{ quote(output) }}
    CARGO_LLVM_COV_TARGET_DIR={{ quote(output) }} cargo llvm-cov nextest --workspace --locked {{ validate }} --profile ci --no-fail-fast --lcov --output-path {{ quote(output / "lcov.info") }} \
      --ignore-filename-regex '(generated/|xtask/|benches/|tests/)' {{ args }}

[group('manual')]
[doc('Every benchmark runs once (no timing gate)')]
bench-smoke:
    cargo test --no-fail-fast --benches -p pse-benches -p pse-relations --locked {{ validate }}

[group('local')]
[doc('Native cache, round and reuse benchmark measurements')]
bench-cache:
    cargo bench --no-fail-fast -p pse-benches -p pse-relations --bench native_cache --locked {{ validate }}

[group('manual')]
[doc('Identifiers named in docs resolve in the extracted API facts')]
doc-lint:
    cargo xtask doc-lint

[group('manual')]
[doc('ADR lint, index check, and register lint')]
adr-lint:
    python3 -m scripts.validation --group adr-lint

[group('manual')]
[doc('Build documentation HTML and scoped search (no product environment)')]
docs:
    python3 -m scripts.docs build

[group('local')]
[doc('Publisher and citation fixtures (stdlib plus declared documentation binaries)')]
docs-test:
    python3 -m unittest scripts.tests.test_docs scripts.tests.docs_integration -v

[group('local')]
[doc('Install the documentation tool versions from docs/site.toml')]
bootstrap-docs:
    python3 -m scripts.docs install

[group('manual')]
[doc('Serve the documentation book locally')]
docs-serve:
    python3 -m scripts.docs serve

[group('local')]
[doc('Parity suite against IDAES 2.13.0 on this machine: pse built on the linked native solvers, the extracted solver prefix first on PATH (fails, never skips)')]
[positional-arguments]
parity *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    maturin="$(realpath "{{ bin / 'maturin' }}")"
    export UV_PROJECT_ENVIRONMENT=.venv-parity
    uv sync --locked --group parity --no-install-project --python 3.13
    VIRTUAL_ENV="$PWD/.venv-parity" "$maturin" develop --uv --profile dev --locked --features force-validate,native-solvers
    export PATH="$IPOPT_DIR/bin:$PATH"
    bash scripts/memory-cap.sh .venv-parity/bin/python -m pytest --maxfail=0 --parity -m "unit or component or integration" python/pse/parity "$@"

[group('manual')]
[doc('Optional aggregate of Rust, Python, quality and documentation checks; parity is separate')]
ci-pr:
    python3 -m scripts.validation --group ci-pr

# ------------------------------------------------------------ deeper manual --

[group('manual')]
[doc('cargo hack feature powerset (depth 2) and --no-default-features')]
features-powerset:
    python3 -m scripts.validation --group features-powerset

[group('manual')]
[doc('Tests under the release profile (catches optimisation-dependent paths)')]
test-release *args:
    cargo nextest {{ nextest_action }} --workspace --locked --cargo-profile release {{ validate }} {{ args }}

[group('manual')]
[doc('cargo udeps on the pinned nightly toolchain')]
udeps:
    cargo udeps --workspace --all-targets --locked

[group('manual')]
[doc('Mutation testing for one file')]
mutants-file path:
    cargo mutants -f {{ path }} --no-shuffle -j 2

[group('manual')]
[doc('Unsafe surface report (cargo geiger)')]
unsafe-surface:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver" "$@"
    exec "{{ py }}" -m scripts.audit_tools unsafe-surface

[group('manual')]
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
[doc('Regenerate the source-readable docs/adr/README.md index')]
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
    # Numbers are never reused: count retained plans and every plan ever added (ADR-0096).
    n=$( { ls docs/plans; git log --no-renames --diff-filter=A --name-only --format= -- docs/plans 2>/dev/null | sed 's#.*/##'; } | grep -E '^[0-9]{2}-' | sort | tail -n1 | cut -c1-2)
    next=$(printf '%02d' $((10#${n:-0} + 1)))
    f="docs/plans/${next}-{{ slug }}.md"
    [ -e "$f" ] && { echo "exists: $f" >&2; exit 1; }
    printf -- '---\ntitle: {{ slug }}\nstatus: draft\ndate: %s\nadrs: []\nreview_sources: []\nscenario_sources: []\n---\n\n# {{ slug }}\n\n## Context\n\n## Decisions\n\n## Architectural drivers and scenarios\n\nLink relevant scenario definitions; state responsibilities, consumed contracts and expected change boundaries.\n\n## Plan\n\n| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced code / deletion | Status or status-owner link |\n|---|---|---|---|---|\n\n## Finding dispositions\n\nThis table owns adopted finding status; link packet evidence instead of copying execution reports.\n\n| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |\n|---|---|---|---|---|\n\n## Verification\n\nTargeted checks accompany implementation; one final stage qualifies the applicable scope.\n\n## Open items\n\n## Outcome (recorded after implementation)\n\n### What was built\n\n### A mistake made and corrected\n\n### Deviations from the plan, deliberate\n' "$(date +%F)" > "$f"
    echo "$f"

[group('decisions')]
[doc('Rows of the deferred-trigger register that are due, with their checks run')]
register-check:
    python3 scripts/check_register.py --due

# ----------------------------------------------------------------- mutating --

[group('mutating')]
[doc('Format Rust, TOML and Python in place and apply ruff auto-fixes; lint-py reports the rest')]
fmt:
    cargo fmt --all
    "{{ taplo }}" fmt
    "{{ ruff }}" check --config pyproject.toml --fix --quiet --exit-zero
    "{{ ruff }}" format --config pyproject.toml

[group('mutating')]
[doc('Regenerate the library-utilization catalog (docs/library-utilization.jsonl) and the usage index behind the library-catalog MCP server')]
library-catalog:
    "{{ py }}" scripts/library_utilization.py --write

[group('mutating')]
[doc('Regenerate relations, Python contracts, docs/generated, the canonical native schema and codecs, the Ipopt bindings and the cargo-hakari workspace-hack')]
codegen *args:
    PSE_NATIVE_CAPABILITIES='compiler,solver' bash scripts/native_exec.sh cargo run --quiet -p xtask --features package-fixtures --locked -- codegen {{ args }}
    cargo hakari generate
    cargo hakari manage-deps --yes

[group('mutating')]
[doc('Generate Rust contracts, rebuild their package loader, then regenerate complete outputs')]
codegen-bootstrap *args:
    cargo run -p xtask --no-default-features --locked -- codegen --only rust-contracts
    cargo run -p xtask --no-default-features --locked -- codegen --only surreal
    just codegen {{ args }}

[group('mutating')]
[doc('Generate schema contracts and reference docs without executing physical package fixtures')]
codegen-contracts:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/build-env.sh
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    cargo run -p xtask --no-default-features --locked -- codegen --only rust-contracts
    cargo run -p xtask --no-default-features --locked -- codegen --only python
    cargo run -p xtask --no-default-features --locked -- codegen --only docs
    cargo run -p xtask --no-default-features --locked -- codegen --only surreal

[group('local')]
[doc('Exercise pure candidate annotation checking and refusal of native codec execution')]
codegen-annotations-test:
    "{{ py }}" -m unittest scripts.tests.test_python_contract_annotations -v

[group('mutating')]
[doc('Accept pending insta snapshots')]
snapshots-accept:
    cargo insta accept

[group('mutating')]
[doc('Build the solver container locally (Ipopt 3.14 with MUMPS/SPRAL/oneMKL Pardiso, SCIP 10; ~4 minutes on 32 threads)')]
[confirm('Build the solver image locally?')]
solver-image target="ci":
    docker build --target {{ target }} -t pse-solvers:{{ target }}-local --build-arg RUST_TOOLCHAIN="$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)" -f docker/solvers/Dockerfile docker/solvers

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

# Move named packages after a deliberate version change (ADR-0165). Declared dependencies
# are exact, so bump one by editing its pin (`cargo add name@=x.y.z`, `uv add 'name==x.y.z'`)
# and pass its name here; an undeclared (transitive) package moves by name alone
# (`name@version` selects one of several resolved majors, as `cargo update -p` suggests).
# There is no whole-lock form: a wholesale re-resolve is an operator request. Upgrade-specific
# checks join this recipe: the cargo-hakari workspace-hack is regenerated from the moved
# graph (as `just codegen` does) and `just family-check` must pass, so a split family fails
# here rather than as a misleading `downcast_ref` miss in the next test. A family moves as a
# unit. Then run the tests the move affects.
[group('mutating')]
[doc('Move named packages after a pin edit, regenerate hakari, check families: just upgrade package ...')]
upgrade +packages:
    #!/usr/bin/env bash
    set -euo pipefail
    before="$(mktemp -d)"
    trap 'rm -rf "$before"' EXIT
    cp Cargo.lock uv.lock "$before/"
    for p in {{ packages }}; do
        name="${p%%@*}"
        moved=0
        if grep -qiE "^name = \"${name//[-_]/[-_]}\"$" uv.lock; then
            uv lock --upgrade-package "$name"; moved=1
        fi
        if grep -qE "^name = \"$name\"$" Cargo.lock; then
            cargo update -p "$p"; moved=1
        fi
        if [ "$moved" = 0 ]; then
            echo "upgrade: $name is in neither uv.lock nor Cargo.lock" >&2
            exit 1
        fi
    done
    cargo hakari generate
    cargo hakari manage-deps --yes
    python3 - "$before" <<'PY'
    import sys, tomllib
    from pathlib import Path
    def versions(path):
        out = {}
        for pkg in tomllib.loads(Path(path).read_text()).get("package", []):
            out.setdefault(pkg["name"], set()).add(pkg.get("version", "(dynamic)"))
        return out
    for lock in ("Cargo.lock", "uv.lock"):
        old, new = versions(Path(sys.argv[1]) / lock), versions(lock)
        moved = [
            f"  {name}: {', '.join(sorted(old.get(name, set()) - new.get(name, set()))) or '(new)'}"
            f" -> {', '.join(sorted(new.get(name, set()) - old.get(name, set()))) or '(removed)'}"
            for name in sorted(old.keys() | new.keys())
            if old.get(name) != new.get(name)
        ]
        print(f"upgrade: {lock}: {len(moved)} package(s) moved")
        if moved:
            print("\n".join(moved))
    PY
    just family-check

[group('mutating')]
[doc('Install the pse.slice user unit: CPU weight below the editor, memory-pressure monitoring (machine-local)')]
slice-install:
    install -Dm644 .config/systemd/pse.slice "$HOME/.config/systemd/user/pse.slice"
    systemctl --user daemon-reload

[group('mutating')]
[doc('Materialize shared skill aliases; native agent adapters are maintained separately')]
agent-config-sync:
    python3 scripts/agent-config.py

[group('local')]
[doc('Apply .config/library-skills.toml for Codex and Claude Code')]
skills-sync:
    python3 scripts/library_skills.py

[group('local')]
[doc('Check selected shared library skill links without changing files')]
skills-check:
    python3 scripts/library_skills.py --check

[group('local')]
[doc('Behavioral tests for repository setup and agent guards (JUnit via the venv)')]
setup-test:
    {{ py }} -m scripts.setup_report build/setup-tests

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

[group('manual')]
[doc('Rebuild the solver libraries without Docker layer cache and compare published checksums')]
solver-rebuild-check:
    bash scripts/solver-rebuild-check.sh

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
    "{{ py }}" -m scripts.reuse_lint

[group('manual')]
[doc('Manually qualify all wheel platforms and the sdist on GitHub, without publishing')]
wheels-check ref="main":
    gh workflow run wheels.yml --ref "{{ ref }}" -f targets=all



[group('local')]
[doc('Measure native consolidation after current functional qualification')]
bench-consolidation:
    just bench-consolidation-native

[group('local')]
[doc('Compare generated contracts without physical package fixtures or integration')]
codegen-contracts-check:
    python3 -m scripts.validation --group codegen-contracts-check

[group('local')]
[doc('Targeted runner selection, report, reuse, policy and measurement-CSV units')]
unit-consolidation-tools:
    "{{ py }}" -m unittest scripts.tests.test_validation scripts.tests.test_execution_contracts scripts.tests.test_build_measurements

[group('local')]
[doc('Static source taxonomy checks only; no runtime integration journey')]
unit-consolidation-governance *args:
    cargo nextest {{ nextest_action }} -p pse-tests-governance -p pse-relations --test error_taxonomy --locked {{ validate }} {{ args }}

[group('local')]
[doc('Isolated native operation and function units; no compiler/storage/solver journeys')]
dev-native-contracts *args:
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
    cargo clippy --keep-going -p pse-engine -p pse-schema -p pse-relations -p pse-compiler -p pse-rules -p pse-backend-native -p pse-runtime -p pse-benches --all-targets --locked {{ validate }} -- -D warnings

[group('local')]
[doc('Lint the native data pivot and every consumer, without executing test journeys')]
lint-native-data:
    cargo clippy --keep-going --workspace --all-targets --locked {{ validate }} -- -D warnings



[group('local')]
[doc('Isolated native Python stub declaration and export consistency units')]
dev-native-boundary-tools *args:
    cargo nextest {{ nextest_action }} -p xtask --no-default-features --locked {{ validate }} -E 'test(codegen::python_stubs::)' {{ args }}

# Exhaustive generated annotation checks are explicit, not package import work.
[group('local')]
python-contracts-check:
    bash scripts/native_exec.sh {{ py }} scripts/check_python_contracts.py

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
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production"
    CARGO_TARGET_DIR=target/measure-production cargo tree -p pse-benches --locked -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production/features.txt"
    CARGO_TARGET_DIR=target/measure-production PSE_ACCEPTANCE_OUTPUT="$(realpath -m "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}")/production" bash scripts/memory-cap.sh cargo bench --no-fail-fast -p pse-benches --bench native_cache --bench native_consolidation --locked

[group('local')]
[doc('M17-M18 native operator, derivative composition and workflow contract units')]
unit-dynamics-fitting:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    cargo nextest run -p pse-backend-native -p pse-runtime -p pse-math -p pse-compiler -p pse-relations --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'test(dynamics::) | test(fitting::) | test(workflow::tests::) | test(function_coordinates)'

[group('mutating')]
[doc('Regenerate independent PC-SAFT and Peng-Robinson references in an isolated locked Python 3.12 environment')]
plan14-reference:
    UV_PROJECT_ENVIRONMENT=build/plan14-reference uv sync --locked --python 3.12 --no-default-groups --group thermo-reference --no-install-project
    build/plan14-reference/bin/python scripts/plan14_reference.py
    build/plan14-reference/bin/python -m scripts.feos_entropy_reference
    build/plan14-reference/bin/python scripts/pr_stability_reference.py

[group('mutating')]
[doc('Freeze seven independent FeOS 0.10.1 PC-SAFT states into the typed oracle bank')]
feos-reference:
    PSE_NATIVE_CAPABILITIES='' bash scripts/native_exec.sh cargo run --package xtask --locked --features thermodynamic-oracles -- feos-reference packages/reference/data/oracles/feos-0.10.1/data

[group('mutating')]
[doc('Freeze the declared 200 paired Latin-hypercube thermodynamic campaign feeds')]
thermodynamic-feeds:
    .venv/bin/python -m scripts.thermodynamic_feeds

[group('manual')]
[doc('Measure every paired flash or 1000-point value-study outcome in a fresh directory')]
thermodynamic-campaign phase output:
    bash scripts/native_exec.sh .venv/bin/python -m scripts.thermodynamic_campaign {{phase}} {{output}}

[group('manual')]
[doc('Freeze the declared IDAES 2.13.0 SRK oracle observations through the isolated parity environment')]
srk-reference:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --group parity --no-install-project --python 3.13
    export PATH="$IPOPT_DIR/bin:$PATH"
    bash scripts/memory-cap.sh .venv-parity/bin/python scripts/idaes_srk_reference.py

[group('manual')]
[doc('Freeze the declared binary NRTL defaults through IDAES 2.13.0 equality evaluation')]
nrtl-reference:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --group parity --no-install-project --python 3.13
    bash scripts/memory-cap.sh .venv-parity/bin/python scripts/idaes_nrtl_reference.py

[group('local')]
[doc('Bit-exact source-to-generated physical fixture equivalence; no solver workflow')]
unit-physical-fixture:
    cargo nextest {{ nextest_action }} -p xtask --locked {{ validate }} -E 'test(codegen::physical::tests::standard_fixture_matches_yaml)'

[group('local')]
[doc('M21 isolated new physical, structural and library-boundary unit controls')]
unit-m21:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    if [[ -f .envrc.local ]]; then source .envrc.local; fi
    cargo nextest run -p pse-runtime -p pse-backend-native -p pse-kernels -p pse-math --lib --locked --features pse-runtime/native-solvers,pse-relations/force-validate -E 'test(contribution_authority_derives_equations_and_independent_closure) | test(implicit_response_checks_scaled_backward_error) | test(roots_and_nlp_admit_original_matching_by_class) | test(integrated_balances_carry_segments_and_refuse_undeclared_jumps) | test(declared_envelopes_bind_identity_and_reject_unrequested_pressure) | test(current_function_vocabulary_roundtrips_without_legacy_fallback)'

[group('local')]
[doc('M21 pure source grammar and ID framing controls after hard deletion')]
unit-m21-authoring *args:
    cargo nextest run -p pse-authoring -p pse-relations --test dsl_examples --test dsl_roundtrip --locked {{ validate }} {{ args }}
    cargo nextest run -p pse-ids -p pse-relations --lib --test golden_vectors --locked {{ validate }} -E 'package(pse-ids)' {{ args }}

[group('local')]
[doc('Run an existing recipe with the parallel rustc frontend (-Zthreads=1|2|4|8), compiler caching and an isolated target directory')]
[positional-arguments]
build-frontend threads +args:
    python3 -m scripts.build_environment --frontend "$1" --cache on -- just "${@:2}"

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

[group('local')]
[doc('Explicit targeted library checks against the shared linked native feature graph')]
unit-native-selected filter *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh "solver,klu,isolation,uno,petsc" "$@"
    bash scripts/memory-cap.sh cargo nextest {{ nextest_action }} --workspace --lib --locked --features pse-py/native-solvers,pse-relations/force-validate -E {{ quote(filter) }} {{ args }}

[group('local')]
[doc('Compile the native process measurement target with pinned native environment and force validation')]
check-native-process-bench:
    bash scripts/native_exec.sh cargo check -p pse-benches --bench native_process --locked --features pse-benches/native-process,pse-relations/force-validate

[group('local')]
[doc('Targeted native package units with explicitly selected adapter features')]
unit-native-package pkg features filter *args:
    just unit-native-capability-package {{ quote(pkg) }} {{ quote(features) }} 'solver,klu,isolation,uno,petsc' {{ quote(filter) }} {{ args }}

[group('local')]
[doc('Targeted native package units with an explicit setup capability request')]
unit-native-capability-package pkg features capabilities filter *args:
    #!/usr/bin/env bash
    set -euo pipefail
    source scripts/native-recipe-env.sh {{ quote(capabilities) }} "$@"
    bash scripts/memory-cap.sh cargo nextest {{ nextest_action }} -p {{ pkg }} -p pse-relations --lib --locked --features {{ features }},pse-relations/force-validate -E {{ quote(filter) }} {{ args }}

[group('local')]
[doc('Focused default-feature refusal controls; keeps the real serial QDLDL absence branch')]
[positional-arguments]
feature-absence *args:
    bash scripts/memory-cap.sh cargo nextest {{ nextest_action }} --locked {{ validate }} "$@"

[group('local')]
[doc('Full workspace native feature graph with Arrow force validation; nextest owns selection')]
[positional-arguments]
native-test *args:
    bash scripts/native_exec.sh "{{ py }}" -m scripts.native_tests rust "$@"

[group('local')]
[doc('Full linked Python unit/component/integration scope; refresh with py-sync-native after Rust edits')]
[positional-arguments]
native-python output *args:
    #!/usr/bin/env bash
    set -euo pipefail
    native_output="$1"
    shift
    exec bash scripts/native_exec.sh "{{ py }}" -m scripts.native_tests python --junitxml="$native_output/native-python.xml" "$@"

[group('local')]
[doc('Run selected native benchmark controls once; no performance receipt or timing samples')]
[positional-arguments]
bench-case-smoke output *args:
    bash scripts/native_exec.sh "{{ py }}" -m scripts.case_measure --smoke "$@"

[group('local')]
[doc('Fresh-process Criterion cases; requires --functional-from with completed local qualification')]
[positional-arguments]
case-measure output *args:
    bash scripts/native_exec.sh "{{ py }}" -m scripts.case_measure "$@"
