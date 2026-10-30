# Operational API for pse-arrow.
#
# Recipes are shortcuts, not gates: they carry the feature selection, profiles, report
# paths and tool paths, and bare tools read the same configuration (`scripts/pse-env --
# <cmd>` for the environment, `cargo c`/`cargo t` for force-validated checks and tests).
# Workload recipe lines run through scripts/pse-env: the checkout environment under any
# inherited shell, with native foreground ownership. Explicit finite selections retain their
# owned physical admission; native providers require completed preparation. Plan58 records
# current acceptance and remaining scientific/reference/timing obligations.
#
# Finding a recipe: `just --list --group <tier>`, `just --usage <recipe>` (parameters),
# `just --show <recipe>` (body), `just --dump --dump-format json` (everything, structured).
# Selection: `just unit-package <pkg> <word|filterset> [-p other] [nextest args]`,
# `scripts/pse affected` (conservative source reach), PSE_NEXTEST_ACTION=list to preview tests.
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

set shell := ["bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail", "-c"]
set script-interpreter := ["bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail"]

# The development interpreter. `.python-version` is the single source of truth (uv,
# direnv, doctor and CI all read it); earlier interpreters are a CI-matrix concern.
python := env("PSE_PYTHON", trim(read(".python-version")))
venv := env("UV_PROJECT_ENVIRONMENT", ".venv")
# Always the project's own tools, never whatever is on PATH: a stale global ruff
# silently produces a different diff than CI.
bin := venv / if os() == "windows" { "Scripts" } else { "bin" }
py := bin / if os() == "windows" { "python.exe" } else { "python" }
reuse := bin / "reuse"
# Correctness commands derive explicit opt-ins from actual selected Cargo units.
# Listing uses the same selectors, features and build modes as execution.
nextest_action := env("PSE_NEXTEST_ACTION", "run --no-fail-fast")

default:
    @just --list --unsorted

# ---------------------------------------------------------------------- env --

[group('env')]
[doc('Full environment from nothing: venv, quality tools, cargo tools, linters')]
bootstrap:
    ./scripts/bootstrap.sh

[group('env')]
[doc('Independent reference/tool environment only; no product extension is installed')]
bootstrap-venv:
    ./scripts/bootstrap.sh --venv-only

[group('env')]
[doc('ruff/pyrefly/reuse/taplo/typos from [dependency-groups].quality')]
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
[script("bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail")]
bootstrap-solvers:
    solver_image="${PSE_SOLVER_IMAGE-}"
    if [[ ! ${PSE_SOLVER_IMAGE+x} ]]; then
        solver_image=$(scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- solver-images ref ci)
    fi
    docker pull "$solver_image"

[group('env')]
[doc('Is this working copy ready to do work? Prints the fix for anything missing.')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
doctor *args:
    scripts/pse-host doctor "$@"

[group('env')]
[doc('Machine-readable environment status, for agents and CI')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
doctor-json:
    scripts/pse-host doctor --json

[group('env')]
[doc('Fetch the pinned reading copies into external/ (idaes-pse, arrow-rs, datafusion)')]
fetch-external:
    ./scripts/fetch-external.sh

[group('env')]
[doc('Build the immutable pinned study module; --qualification selects a separately identified probe artifact')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
surreal-module *args:
    scripts/pse-host study-module prepare "$@"

[group('diagnostics')]
[doc('Explicit native telemetry service actions and passive configuration/health reads')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
telemetry *args:
    exec scripts/pse-host telemetry-service "$@"

[group('env')]
[doc('Explicitly build the product allocation materializer used before managed deployment effects')]
deployment-build:
    scripts/pse-env --native= -- scripts/cargo-bin/cargo-pse-correctness build -p pse-runtime --bin pse-deployment --no-default-features --locked

[group('local')]
[doc('Initialize an explicitly selected existing native deployment; no service starts implicitly')]
canonical-init state:
    scripts/cargo-bin/cargo-pse-correctness build -p pse-operations --bin pse-canonical-init --no-default-features --locked
    target/debug/pse-canonical-init {{ quote(state) }}

[group('local')]
[doc('Explicitly build the dedicated tooling evidence operator; inspection never builds it implicitly')]
evidence-build:
    scripts/cargo-bin/cargo-pse-correctness build -p xtask --bin pse-evidence --no-default-features --features evidence-tools --locked

[group('local')]
[doc('Inspect supplied evidence offline or explicitly manage its independently disposable index')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
evidence *args:
    test -x target/debug/pse-evidence || { echo "pse-evidence: run just evidence-build explicitly" >&2; exit 125; }
    exec target/debug/pse-evidence evidence "$@"

[group('local')]
[doc('Explicitly build the dedicated project metadata operator')]
project-build:
    scripts/pse-env --native= -- scripts/cargo-bin/cargo-pse-correctness build -p pse-runtime --bin pse-project --no-default-features --features product-operators --locked

[group('local')]
[doc('Use owned project metadata commands; --state selects the existing canonical deployment')]
[script("bash", "scripts/pse-env", "--native=", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
project *args:
    target/debug/pse-project project "$@"

[group('local')]
[doc('Native server and immutable module preparation controls')]
surreal-test:
    scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast -p pse-host --lib --locked -E 'test(server::) | test(study_module::)'

[group('local')]
[doc('Source-built codec and revision controls in a fresh native-owned canonical fixture')]
[positional-arguments]
[script]
canonical-test module_receipt *args:
    module_receipt="$1"
    shift
    scripts/cargo-bin/cargo-pse-correctness build -p pse-operations --features native-service --bin pse-canonical-service --bin pse-canonical-test --locked
    exec target/debug/pse-canonical-test --root "$PWD" --correctness "$PWD/target/host-tools/current/cargo-pse-correctness" --nextest "$(command -v cargo-nextest)" --helper "$PWD/target/debug/pse-canonical-service" --module-receipt "$module_receipt" --state-parent "${TMPDIR:-/tmp}" --build-arg=-p --build-arg=pse-operations --build-arg=--lib --build-arg=--features --build-arg=canonical-tests,native-service --build-arg=--locked -- -E 'test(canonical_server_unit) | test(canonical_codec_unit)' "$@"

[group('local')]
[doc('Persisted reconstruction controls in a fresh explicitly selected native fixture')]
[script("bash", "scripts/pse-env", "--native=compiler,solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
canonical-portable-test module_receipt *args:
    module_receipt="$1"
    shift
    fixture_receipt="$PWD/target/producer-qualified-fixture.json"
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- producer-identity-fixture --output "$fixture_receipt"
    export PSE_PRODUCER_FIXTURE_RECEIPT="$fixture_receipt"
    exec just native-test "$module_receipt" --build-arg=-p --build-arg=pse-runtime --build-arg=--lib --build-arg=--features --build-arg=canonical-tests,native-solvers --build-arg=--locked -- -E 'test(canonical_portable_body)' "$@"

[group('local')]
[doc('Actual finite production capture prerequisite for persisted replay controls')]
producer-fixture output:
    scripts/pse-env --native -- cargo run -p xtask --bin xtask --no-default-features --locked -- producer-identity-fixture --output {{ quote(output) }}

[group('local')]
[doc('Derive a relevant production-unit identity; unknown inputs disable persistent reuse')]
[positional-arguments]
[script]
producer-identity *args:
    # Cargo's run launcher injects tool-local loader paths. Build the capture tool
    # separately so the recorded caller is the ordinary native deployment context.
    cargo build -p xtask --bin xtask --no-default-features --locked
    exec scripts/pse-env --native -- "$PWD/target/debug/xtask" producer-identity "$@"

[group('local')]
[doc('Focused producer identity tests including irrelevant test edits and consumed-input changes')]
[positional-arguments]
producer-identity-test *args:
    scripts/pse-env --native -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p xtask --bin xtask --no-default-features --locked -E 'test(producer_)' "$@"

[group('local')]
[doc('Generator identity, unchanged-output and physical fixture controls')]
[positional-arguments]
codegen-unit-test *args:
    scripts/pse-env --native=compiler,solver -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p xtask --bin xtask --locked --features full-codegen -E 'test(codegen::tests::) | test(codegen::physical::tests::)' "$@"

[group('local')]
[doc('Worker CLI admission budgets against the configured managed process allocation')]
[positional-arguments]
worker-cli-unit-test *args:
    scripts/pse-env --native -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-runtime --bin pse-worker --locked --features product-operators,native-solvers "$@"

[group('local')]
[doc('Miri controls for the pure qualified scientific reconstruction authority')]
scientific-replay-miri:
    scripts/cargo-bin/cargo-pse-correctness miri test -p pse-ids --lib --locked scientific_replay::tests

[group('local')]
[doc('Run native scientific worker journeys using explicitly built worker/control helpers and a prepared module receipt')]
[script("bash", "scripts/pse-env", "--native=compiler,solver,klu", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
worker-test *args:
    : "${PSE_NATIVE_FIXTURE_SERVICE_HELPER:?select the already-built native canonical-service helper}"
    : "${PSE_WORKER_BINARY:?select the already-built scientific pse-worker}"
    export PSE_NATIVE_STUDY_WORKER="$PSE_WORKER_BINARY"
    exec scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-runtime --test worker --locked --features native-study-tests,product-operators,solver-ipopt,solver-scip "$@"

# ---------------------------------------------------------------- discovery --

[group('discovery')]
[doc('Native tool and host observations as structured output; never repairs prerequisites')]
versions:
    scripts/pse-host doctor --json

[group('discovery')]
[doc('Heading outline of a document (capability maps and the blueprint are large)')]
[positional-arguments]
[script("bash", "-euo", "pipefail")]
lib-outline doc:
    exec scripts/pse --format human outline "$1"

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
fmt-rust-check:
    cargo fmt --all -- --check

[group('local')]
lint-toml:
    uv run taplo fmt --check --diff

[group('local')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
clippy-default *args:
    scripts/cargo-bin/cargo-pse-correctness clippy --keep-going --workspace --all-targets --locked "$@" -- -D warnings

[group('local')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
clippy-no-default *args:
    scripts/cargo-bin/cargo-pse-correctness clippy --keep-going --workspace --all-targets --locked --no-default-features "$@" -- -D warnings

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
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
adr-frontmatter-check:
    cargo run --locked -p xtask --no-default-features -- adr lint

[group('local')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
adr-index-check:
    cargo run --locked -p xtask --no-default-features -- adr index --check

[group('local')]
register-lint:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- register lint

[group('local')]
codegen-relations-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --features physical-admission,force-validate --locked -- codegen --only relations --check

[group('mutating')]
[doc('Generate Rust contracts only, without physical fixtures or compiler workflows')]
codegen-rust-contracts:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only rust-contracts

[group('local')]
codegen-rust-contracts-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only rust-contracts --check

[group('mutating')]
[doc('Generate physical fixtures through lower admission, without Runtime or solver construction')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "functional", "--", "bash", "-euo", "pipefail")]
codegen-physical:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --features physical-admission,force-validate --locked -- codegen --only physical

[group('local')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "functional", "--", "bash", "-euo", "pipefail")]
codegen-physical-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --features physical-admission,force-validate --locked -- codegen --only physical --check

[group('local')]
codegen-docs-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only docs --check

[group('local')]
[doc('The canonical native schema and codecs match a fresh regeneration')]
codegen-surreal-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only surreal --check

[group('local')]
codegen-bindgen-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --features native-abi --locked -- codegen --only bindgen --check

[group('local')]
[doc('The published document and authoring JSON Schemas and the Python document types match a fresh derivation from the owning crates (ADR-0116)')]
codegen-schemas-check:
    scripts/pse-env --native= -- scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --features product-schemas,force-validate --locked -- codegen --only schemas --check

[group('local')]
[positional-arguments]
governance-tests *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-tests-governance --locked "$@"

[group('local')]
audit-dependencies:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- audit audit-dependencies

[group('local')]
audit-advisories:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- audit audit-advisories

[group('local')]
audit-shear:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- audit audit-shear

[group('local')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
features-combinations:
    CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo-bin/cargo-pse-correctness" cargo hack --keep-going check --workspace --feature-powerset --depth 2 --locked

[group('local')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
features-no-default:
    CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo-bin/cargo-pse-correctness" cargo hack --keep-going check --workspace --no-default-features --locked

[group('local')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
doctest-release:
    scripts/cargo-bin/cargo-pse-correctness test --no-fail-fast --doc --workspace --locked --release

[group('local')]
[doc('Measure current native consolidation, including diagnostic campaigns with an open acceptance barrier')]
bench-consolidation-native:
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}"
    cargo tree -p pse-benches --locked --features pse-benches/force-validate -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/force-validation-features.txt"
    scripts/cargo-bin/cargo-pse-correctness bench --no-fail-fast -p pse-benches --bench native_consolidation --locked

[group('local')]
[doc('cargo check, workspace, all targets')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check *args:
    scripts/cargo-bin/cargo-pse-correctness check --keep-going --workspace --all-targets --locked "$@"

[group('local')]
[doc('Compile one library during a bounded architectural replacement; no tests or dev dependencies')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-library pkg *args:
    scripts/cargo-bin/cargo-pse-correctness check --keep-going -p "$1" --lib --locked "${@:2}"

[group('local')]
[doc('Compile one package and its test sources without executing tests')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-package pkg *args:
    scripts/cargo-bin/cargo-pse-correctness check --keep-going -p "$1" --all-targets --locked "${@:2}"

[group('local')]
[doc('Compile a named Rust test target without executing tests during the architectural pivot')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-test pkg target *args:
    scripts/cargo-bin/cargo-pse-correctness check --keep-going -p "$1" --test "$2" --locked "${@:3}"

[group('local')]
[doc('Compile one linked native test target without executing its journeys')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-native-test pkg target features:
    scripts/pse-env --native -- scripts/cargo-bin/cargo-pse-correctness check -p "$1" --test "$2" --locked --features "$3"

[group('local')]
[doc('Compile selected native package targets without replacing running test executables')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-native-package pkg features capabilities="solver,klu,isolation,uno,petsc":
    scripts/pse-env --native="$3" -- scripts/cargo-bin/cargo-pse-correctness check -p "$1" --all-targets --locked --features "$2"

[group('local')]
[doc('Run selected functional controls in one Rust test target with explicit Arrow validation')]
[positional-arguments]
functional-test-target pkg target filter *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p "$1" --test "$2" --locked -E "$3" "${@:4}"

# Just stops after the first failed recipe; this thin aggregate retains every status.
[private]
[positional-arguments]
[script("bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail")]
run-all +recipes:
    failed=()
    for recipe in "$@"; do
        if just "$recipe"; then :; else failed+=("$recipe"); fi
    done
    if (( ${#failed[@]} )); then
        printf 'Failed recipes: %s\n' "${failed[*]}" >&2
        exit 1
    fi

[group('local')]
[doc('clippy with -D warnings, workspace, all targets (default and --no-default-features)')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
clippy:
    just run-all clippy-default clippy-no-default

[group('local')]
[doc('rustfmt and taplo in check mode')]
fmt-check:
    just run-all fmt-rust-check lint-toml

[group('local')]
[doc('Rust tests via nextest with Arrow force_validate on')]
[positional-arguments]
test *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} --workspace --locked "$@"

[group('local')]
[doc('Rust tests for one package')]
[positional-arguments]
test-package pkg *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p "$1" --locked "${@:2}"

[group('local')]
[doc('Library units of one package: a bare word means test(word); extra -p widens; native Nextest owns selection')]
[script("bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
unit-package pkg filter *args:
    package="$1"
    filter="$2"
    shift 2
    if [[ "$filter" =~ ^[A-Za-z0-9_]+(::[A-Za-z0-9_]+)*(::)?$ ]]; then filter="test($filter)"; fi
    command=(scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p "$package" --lib --locked -E "$filter" "$@")
    printf '%q ' "${command[@]}"
    printf '\n'
    exec "${command[@]}"

[group('local')]
[doc('Explicit library units in one workspace feature graph; native Nextest owns selection')]
[script("bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
unit-libraries filter *args:
    filter="$1"
    shift
    if [[ "$filter" =~ ^[A-Za-z0-9_]+(::[A-Za-z0-9_]+)*(::)?$ ]]; then filter="test($filter)"; fi
    command=(scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} --workspace --lib --locked -E "$filter" "$@")
    printf '%q ' "${command[@]}"
    printf '\n'
    exec "${command[@]}"




[group('local')]
[doc('Resolve newly declared library profiles without updating unrelated dependency selections')]
resolve-math-profiles:
    scripts/cargo-bin/cargo-pse-correctness check -p pse-math

[group('local')]
[doc('Resolve existing pinned dependencies offline after workspace edge changes')]
lock-workspace:
    cargo metadata --offline --format-version 1 > /dev/null

[group('local')]
[doc('Resolve changed Python dependency declarations without upgrading unrelated packages')]
lock-python:
    UV_LOCKED=0 uv lock


[group('local')]
[doc('Isolated engine and assurance units; no storage/compiler/solver journeys')]
[positional-arguments]
dev-native-engine *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-testkit -p pse-engine --lib --locked -E 'package(pse-testkit) and test(native_unit_) or package(pse-engine) and (test(session::config::tests::) or test(cache_service::policy::tests::) or test(session::execution::tests::))' "$@"

[group('local')]
[doc('Static manifest/error governance units; no product execution')]
[positional-arguments]
dev-native-boundaries *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-tests-governance --test every_crate_registered --test dependency_pins --test dependency_floors --test error_taxonomy --locked "$@"

[group('local')]
[doc('Check resolved N06 ownership and deletions without executing product code')]
engine-boundary-check:
    scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast -p pse-tests-governance --test native_engine_boundaries --locked

[group('local')]
[doc('Native preparation, environment and correctness-owner controls')]
native-setup-unit:
    scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast -p pse-host --lib --locked -E 'test(native::) | test(boundary::) | test(correctness::)'

[group('local')]
[doc('Unit control for pinned Ipopt C linking and ABI widths; no solver execution')]
[script]
unit-ipopt-abi:
    exec scripts/pse-env --native=solver -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-ipopt-sys --lib --locked --features pse-ipopt-sys/link -E 'test(abi_tests::)'

[group('local')]
[doc('Prepare source-pinned validated root isolation with FILIB and bundled SoPlex')]
[script("bash", "scripts/pse-env", "--native=isolation", "--", "bash", "-euo", "pipefail")]
native-isolation-prepare:
    printf '%s\n' "$PSE_ROOT_ISOLATION_DIR"

[group('local')]
[doc('Prepare pinned Uno or PETSc scoped foreign inputs against the existing native provider')]
[script]
native-pipeline-prepare kind:
    case {{ quote(kind) }} in uno) variable=UNO_DIR ;; petsc) variable=PETSC_DIR ;; *) exit 2 ;; esac
    exec scripts/pse-env --native={{ quote(kind) }} -- printenv "$variable"

[group('local')]
[doc('Bounded unit controls for a pinned scoped Uno or PETSc project ABI')]
[positional-arguments]
[script]
unit-pipeline-binding kind *args:
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    shift
    exec scripts/pse-env --native={{ quote(kind) }} -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p "pse-{{ kind }}-sys" --lib --locked --features "pse-{{ kind }}-sys/link" -E 'package(pse-{{ kind }}-sys)' "$@"

[group('local')]
[doc('Compile a scoped native pipeline adapter with its qualified foreign prefix')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-pipeline-native kind:
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    exec scripts/pse-env --native={{ quote(kind) }} -- scripts/cargo-bin/cargo-pse-correctness check --keep-going -p pse-backend-native -p pse-runtime --all-targets --locked --features 'pse-backend-native/{{ kind }}'

[group('local')]
[doc('Targeted safe native pipeline adapter controls with a qualified foreign prefix')]
[positional-arguments]
[script]
unit-pipeline-native kind filter *args:
    case {{ quote(kind) }} in uno|petsc) ;; *) exit 2 ;; esac
    shift 2
    exec scripts/pse-env --native={{ quote(kind) }} -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-backend-native --lib --locked --features 'pse-backend-native/{{ kind }}' -E {{ quote(filter) }} "$@"

[group('local')]
[doc('Compile native solver adapters and unit contracts; no solver journeys')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
check-solver-contracts:
    scripts/cargo-bin/cargo-pse-correctness check --keep-going -p pse-backend-native -p pse-runtime -p pse-compiler --all-targets --locked --features pse-runtime/native-solvers

[group('local')]
[doc('Static lint of the linked native workspace, conformance and benchmark consumers')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
lint-solver-contracts:
    scripts/cargo-bin/cargo-pse-correctness clippy --keep-going --no-deps --workspace --all-targets --locked --features pse-runtime/native-solvers,pse-tests-conformance/native-acceptance,pse-benches/native-process -- -D warnings

[group('local')]
[doc('Native callback, upload, status, ABI and lifetime units; no native convergence journeys')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "exclusive", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
unit-native-contracts:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p pse-backend-native -p pse-ipopt-sys -p pse-compiler -p pse-structural -p pse-runtime -p pse-math --lib --locked --features pse-runtime/native-solvers -E 'package(pse-backend-native) | package(pse-compiler) | package(pse-math) | test(abi_tests::) | test(flowsheet::tests::) | test(initialization::tests::) | test(math::tests::) | test(workflow::tests::)'



[group('local')]
[doc('Doctests (nextest does not run them)')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
doctest:
    scripts/cargo-bin/cargo-pse-correctness test --no-fail-fast --doc --workspace --locked

[group('local')]
[doc('rustdoc for the workspace with warnings as errors')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
docs-rust *args:
    RUSTDOCFLAGS="-D warnings" cargo doc --keep-going --workspace --no-deps --locked "$@"

[group('local')]
[doc('Regenerate schema targets and pinned Ipopt bindings in scratch space and compare both ways')]
codegen-check:
    just run-all codegen-all-check

[group('local')]
[doc('Compare every Rust-owned generated target in one full-codegen invocation; report independent failures')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--native=compiler,solver", "--", "bash", "-euo", "pipefail")]
codegen-all-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --features full-codegen,force-validate --locked -- codegen --check

[group('local')]
[doc('One resolved version per family, equal to the pins; shared packages match the evidence lockfiles')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
family-check:
    cargo run -p xtask --no-default-features --locked -- family-check

[group('local')]
[doc('Governance tests + codegen-check + family-check')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
governance:
    just run-all governance-tests codegen-check family-check

[group('local')]
[doc('Optional aggregate: fmt-check check clippy test doctest')]
ci-fast:
    just run-all fmt-check check clippy test doctest

[group('local')]
[doc('ruff format in check mode')]
fmt-py-check:
    uv run ruff format --config pyproject.toml --check

[group('local')]
[doc('ruff check (no --fix; the baseline is zero)')]
lint-py:
    uv run ruff check --config pyproject.toml

[group('local')]
[doc('pyrefly type check, including warnings (the baseline is zero)')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
typecheck:
    uv run pyrefly check --min-severity warn
    uv run pyrefly check --config pyproject.toml --min-severity warn --python-version 3.12 --python-interpreter-path build/plan14-reference/bin/python scripts/plan14_reference.py scripts/feos_entropy_reference.py scripts/pr_stability_reference.py

# Native consumers require explicitly prepared providers and retain their own
# typed prerequisite, scientific and fixture failures.
reference_manifest := "packages/reference/conformance.toml"

[group('local')]
[doc('Run admitted manifest-driven conformance through the native Rust owner')]
[script("bash", "scripts/pse-env", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
modeling-conformance *args:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask --locked -p xtask --features native-solvers,force-validate -- conformance "$@"

[group('local')]
[doc('Run every reference fixture once from packages/reference/conformance.toml; Arrow reports in build/seed-conformance')]
[script("bash", "scripts/pse-env", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
seed-conformance:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask --locked -p xtask --features native-solvers,force-validate -- conformance --manifest {{ reference_manifest }} --report-dir build/seed-conformance

[group('local')]
[doc('Repository-config lint: taplo, typos, reuse, actionlint, zizmor, shellcheck, ast-grep')]
lint-repo:
    just run-all lint-toml lint-typos lint-license lint-actions lint-zizmor lint-shell lint-ast

# Native just aggregates stream ordinary tool output, continue after independent failures,
# and report the failed recipes. Their command lists have one owner here.

[group('mutating')]
[doc('End of a turn that changed files (root agent): regenerate the ADR index and format')]
[positional-arguments]
turn-end:
    just run-all adr-index fmt

[group('env')]
[doc('After a dependency, toolchain or skill-selection change, or an environment-shaped failure: sync library skills and run doctor')]
[positional-arguments]
ready:
    just run-all skills-sync doctor

[group('manual')]
[doc('Scope end: every non-functional check; keeps going and lists failures; fix them and re-run one with just <id>')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
hygiene:
    just run-all lint-agents adr-frontmatter-check adr-index-check register-lint lint-typos lint-license lint-actions lint-shell lint-ast lint-py typecheck engine-boundary-check solver-pin-check family-check audit-shear codegen-check clippy-default clippy-no-default docs-rust

[group('local')]
[doc('Agent configuration: symlinks resolve, every referenced doc path exists')]
lint-agents:
    cargo run --locked -p xtask --no-default-features -- agent-config-check

[group('local')]
[doc('Python and repository quality gate')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
quality:
    just run-all fmt-py-check lint-py typecheck lint-repo lint-agents solver-pin-check

# ------------------------------------------------------------------- manual --

[group('local')]
[doc('Dependency and licence REPORT: findings are advisory and never fail it; a tool that cannot run does')]
deps-report:
    just run-all audit-dependencies audit-advisories audit-shear

[group('manual')]
[doc('Opt-in strict audit: cargo deny (advisories, bans, licenses, sources) + cargo audit. Not in ci-pr')]
policy:
    just run-all audit-dependencies audit-advisories

[group('manual')]
[doc('Every benchmark runs once (no timing gate)')]
bench-smoke:
    scripts/cargo-bin/cargo-pse-correctness test --no-fail-fast --benches -p pse-benches --locked

[group('local')]
[doc('Native cache, round and reuse benchmark measurements')]
bench-cache:
    scripts/cargo-bin/cargo-pse-correctness bench --no-fail-fast -p pse-benches --bench native_cache --locked

[group('manual')]
[doc('Identifiers named in docs resolve in the extracted API facts')]
doc-lint:
    cargo xtask doc-lint

[group('manual')]
[doc('ADR lint, index check, and register lint')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
adr-lint:
    just run-all adr-frontmatter-check adr-index-check register-lint

[group('manual')]
[doc('Build documentation HTML and scoped search (no product environment)')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
docs:
    cargo run --locked -p xtask --no-default-features -- docs build

[group('local')]
[doc('Metadata, lifecycle, publisher and citation behavior in the docs environment')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
docs-test:
    scripts/cargo-bin/cargo-pse-correctness nextest run --bin xtask --locked -p xtask --lib --no-default-features -E test\(documentation::\)

[group('local')]
[doc('Install the documentation tool versions from docs/site.toml')]
bootstrap-docs:
    ./scripts/bootstrap.sh --docs-only

[group('manual')]
[doc('Serve the documentation book locally')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
docs-serve:
    cargo run --locked -p xtask --no-default-features -- docs serve

[group('manual')]
[doc('Four native candidate exchanges with the explicitly isolated IDAES/Pyomo reference environment')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
parity *args:
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --only-group parity --no-install-project --python 3.13
    scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast -p pse-runtime --test reference_degeneracy_exchange --test reference_covariance_exchange --test reference_sensitivity_exchange --test reference_pid_exchange --features solver-highs,solver-ipopt,solver-diffsol --locked "$@"

[group('manual')]
[doc('Optional aggregate of native Rust, repository quality and documentation checks; references are separate')]
ci-pr:
    just run-all ci-fast governance docs-rust bench-smoke quality adr-lint docs

# ------------------------------------------------------------ deeper manual --

[group('manual')]
[doc('cargo hack feature powerset (depth 2) and --no-default-features')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
features-powerset:
    just run-all features-combinations features-no-default

[group('manual')]
[doc('Tests under the release profile (catches optimisation-dependent paths)')]
[positional-arguments]
test-release *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} --workspace --locked --cargo-profile release "$@"

[group('manual')]
[doc('Would a `cargo update` raise a pre-1.0 crate floor? (dry run)')]
floors-latest:
    cargo update --dry-run --workspace 2>&1 | tail -n 40

# ---------------------------------------------------------------- decisions --

[group('decisions')]
[doc('New ADR from the template: just adr-new my-slug --title "..."')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
adr-new slug *args:
    cargo run --locked -p xtask --no-default-features -- adr new "$@"

[group('decisions')]
[doc('Regenerate the source-readable docs/adr/README.md index')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
adr-index:
    cargo run --locked -p xtask --no-default-features -- adr index

[group('decisions')]
[doc('Mark one ADR superseded by another (symmetric links, status history)')]
[script("bash", "scripts/pse-env", "--docs", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
adr-supersede old new:
    cargo run --locked -p xtask --no-default-features -- adr supersede "$@"

[group('decisions')]
[doc('New implementation plan under docs/plans/NN-<slug>.md')]
[script]
plan slug:
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
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- register due

# ----------------------------------------------------------------- mutating --

[group('mutating')]
[doc('Format Rust, TOML and Python in place and apply ruff auto-fixes; lint-py reports the rest')]
fmt:
    cargo fmt --all
    uv run taplo fmt
    uv run ruff check --config pyproject.toml --fix --quiet --exit-zero
    uv run ruff format --config pyproject.toml

[group('mutating')]
[doc('Regenerate native relations, JSON schemas, documentation, canonical codecs and Ipopt bindings')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "functional", "--", "bash", "-euo", "pipefail")]
codegen:
    scripts/pse-env --native=compiler,solver -- /bin/true
    # Contracts and the canonical schema first: the full generator's package loader
    # compiles against them, so a registry change never needs a separate bootstrap.
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only rust-contracts
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only surreal
    scripts/pse-env --native=compiler,solver -- scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --features full-codegen,force-validate --locked -- codegen


[group('mutating')]
[doc('Generate schema contracts and reference docs without executing physical package fixtures')]
[script]
codegen-contracts:
    unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only rust-contracts
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only docs
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- codegen --only surreal

[group('mutating')]
[doc('Accept pending insta snapshots')]
snapshots-accept:
    cargo insta accept

[group('mutating')]
[doc('Build the solver container locally (Ipopt 3.14 with MUMPS/SPRAL/oneMKL Pardiso, SCIP 10; ~4 minutes on 32 threads)')]
[confirm('Build the solver image locally?')]
solver-image target="ci":
    docker build --target {{ target }} -t pse-solvers:{{ target }}-local --build-arg RUST_TOOLCHAIN="$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)" -f docker/solvers/Dockerfile docker/solvers

# Move named packages after a deliberate version change (ADR-0165). Declared dependencies
# are exact, so bump one by editing its pin (`cargo add name@=x.y.z`, `UV_LOCKED=0 uv add 'name==x.y.z'`)
# and pass its name here; an undeclared (transitive) package moves by name alone
# (`name@version` selects one of several resolved majors, as `cargo update -p` suggests).
# There is no whole-lock form: a wholesale re-resolve is an operator request. Upgrade-specific
# checks join this recipe: `just family-check` must pass, so a split family fails
# here rather than as a misleading `downcast_ref` miss in the next test. A family moves as a
# unit. Then run the tests the move affects.
[group('mutating')]
[doc('Move named packages after a pin edit, check families: just upgrade package ...')]
[positional-arguments]
[script]
upgrade +packages:
    before="$(mktemp -d)"
    trap 'rm -rf "$before"' EXIT
    cp Cargo.lock uv.lock "$before/"
    for p in "$@"; do
        name="${p%%@*}"
        moved=0
        if grep -qiE "^name = \"${name//[-_]/[-_]}\"$" uv.lock; then
            UV_LOCKED=0 uv lock --upgrade-package "$name"; moved=1
        fi
        if grep -qE "^name = \"$name\"$" Cargo.lock; then
            cargo update -p "$p"; moved=1
        fi
        if [ "$moved" = 0 ]; then
            echo "upgrade: $name is in neither uv.lock nor Cargo.lock" >&2
            exit 1
        fi
    done
    uv run python - "$before" <<'PY'
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
    cargo run --locked -p xtask --no-default-features -- agent-config

[group('local')]
[doc('Apply .config/library-skills.toml for Codex and Claude Code')]
skills-sync:
    cargo run --locked -p xtask --no-default-features -- skills

[group('local')]
[doc('Check selected shared library skill links without changing files')]
skills-check:
    cargo run --locked -p xtask --no-default-features -- skills --check

[group('local')]
[doc('Native host setup controls with nextest-owned reporting')]
[positional-arguments]
setup-test *args:
    scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast -p pse-host --lib --locked "$@"

[group('mutating')]
[doc('Update immutable solver pins and synchronize workflow/devcontainer literals')]
[confirm('Update the solver image pins?')]
[positional-arguments]
solver-pin-update *args:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- solver-images update "$@"

[group('local')]
[doc('Verify every literal solver image consumer matches the manifest')]
solver-pin-check:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- solver-images check

[group('manual')]
[doc('Rebuild the solver libraries without Docker layer cache and compare published checksums')]
solver-rebuild-check:
    scripts/cargo-bin/cargo-pse-correctness build -p xtask --bin xtask --no-default-features --locked
    PSE_XTASK_BINARY="$PWD/target/debug/xtask" bash scripts/solver-rebuild-check.sh

[group('local')]
[doc('Spelling with the project-pinned tool')]
lint-typos:
    uv run typos

[group('local')]
[doc('License headers with the project-pinned tool')]
lint-license:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --locked -- reuse-lint

[group('local')]
[doc('Measure native consolidation after current functional qualification')]
bench-consolidation:
    just bench-consolidation-native

[group('local')]
[doc('Compare generated contracts without physical package fixtures or integration')]
codegen-contracts-check:
    just run-all codegen-rust-contracts-check codegen-docs-check codegen-surreal-check

[group('local')]
[doc('Compile N07-N08 consumers and tests without executing integration journeys')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-native-contracts *args:
    scripts/cargo-bin/cargo-pse-correctness check --keep-going --workspace --all-targets --locked "$@"

[group('discovery')]
[doc('Compile and enumerate exact workspace test identities without executing them')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
list-native-contracts *args:
    scripts/cargo-bin/cargo-pse-correctness nextest list --workspace --locked --message-format json "$@"

[group('local')]
[doc('Lint the N07-N08 implementation and consumer test sources without execution')]
lint-native-contracts:
    scripts/cargo-bin/cargo-pse-correctness clippy --keep-going -p pse-engine -p pse-schema -p pse-compiler -p pse-rules -p pse-backend-native -p pse-runtime -p pse-benches --all-targets --locked -- -D warnings

[group('local')]
[doc('Lint the native data pivot and every consumer, without executing test journeys')]
lint-native-data:
    scripts/cargo-bin/cargo-pse-correctness clippy --keep-going --workspace --all-targets --locked -- -D warnings




[group('local')]
[doc('Production-equivalent native measurements without force validation, in an isolated target directory')]
bench-production:
    mkdir -p "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production"
    CARGO_TARGET_DIR=target/measure-production cargo tree -p pse-benches --locked -e features --format '{p} features=[{f}]' > "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}/production/features.txt"
    CARGO_TARGET_DIR=target/measure-production PSE_ACCEPTANCE_OUTPUT="$(realpath -m "${PSE_ACCEPTANCE_OUTPUT:-build/measurements}")/production" cargo bench --no-fail-fast -p pse-benches --bench native_cache --bench native_consolidation --locked


[group('local')]
[doc('Reproduce the three independent process/fit references without importing the product')]
plan53a-reference:
    UV_PROJECT_ENVIRONMENT=build/plan53a-reference uv sync --locked --python 3.12 --no-default-groups --group test --no-install-project
    build/plan53a-reference/bin/python -m scripts.pfr_reference
    build/plan53a-reference/bin/python -m scripts.cstr_controller_reference
    build/plan53a-reference/bin/python -m scripts.decay_fit_reference

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
    scripts/pse-env --native= -- cargo run --package xtask --locked --features thermodynamic-oracles -- feos-reference packages/reference/data/oracles/feos-0.10.1/data

[group('mutating')]
[doc('Freeze the declared 200 paired Latin-hypercube thermodynamic campaign feeds')]
thermodynamic-feeds:
    .venv/bin/python -m scripts.thermodynamic_feeds

[group('manual')]
[doc('Freeze the declared IDAES 2.13.0 SRK oracle observations through the isolated parity environment')]
[script("bash", "scripts/pse-env", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
srk-reference:
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --group parity --no-install-project --python 3.13
    export PATH="$IPOPT_DIR/bin:$PATH"
    .venv-parity/bin/python scripts/idaes_srk_reference.py

[group('manual')]
[doc('Freeze the declared binary NRTL defaults through IDAES 2.13.0 equality evaluation')]
[script("bash", "scripts/pse-env", "--native=solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
nrtl-reference:
    UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --group parity --no-install-project --python 3.13
    .venv-parity/bin/python scripts/idaes_nrtl_reference.py




[group('local')]
[doc('Run an existing recipe with compiler wrappers explicitly disabled')]
[positional-arguments]
build-uncached +args:
    exec scripts/pse-env --cache off -- just "$@"

[group('discovery')]
[doc('Read actual current or named systemd scope/service ownership without workload admission or compiler setup')]
[positional-arguments]
[script("bash", "-euo", "pipefail")]
activity *args:
    exec scripts/pse-host activity "$@"

[group('env')]
[doc('Git-owned worktree creation; pass literal git worktree add arguments, then prepare its native host explicitly')]
[positional-arguments]
[script("bash", "-euo", "pipefail")]
worktree +args:
    exec git worktree add "$@"

[group('local')]
[doc('Selected native library tests; database cases use native-test with an explicit module receipt')]
[script("bash", "scripts/pse-env", "--native=compiler,solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
unit-native-selected filter *args:
    filter="$1"
    shift
    if [[ "$filter" =~ ^[A-Za-z0-9_]+(::[A-Za-z0-9_]+)*(::)?$ ]]; then filter="test($filter)"; fi
    exec scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} --workspace --lib --locked --features pse-runtime/native-solvers -E "$filter" "$@"

[group('local')]
[doc('Compile the native process measurement target with pinned native environment and force validation')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "compile", "--", "bash", "-euo", "pipefail")]
check-native-process-bench:
    scripts/pse-env --native -- scripts/cargo-bin/cargo-pse-correctness check -p pse-benches --bench native_process --locked --features pse-benches/native-process

[group('local')]
[doc('Native package library tests without implicit store setup; nextest owns selection')]
[script("bash", "scripts/pse-env", "--native=compiler,solver,klu,isolation,uno,petsc", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
unit-native-package pkg features filter *args:
    package="$1"
    features="$2"
    filter="$3"
    shift 3
    if [[ "$filter" =~ ^[A-Za-z0-9_]+(::[A-Za-z0-9_]+)*(::)?$ ]]; then filter="test($filter)"; fi
    exec scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p "$package" --lib --locked --features "$features" -E "$filter" "$@"

[group('local')]
[doc('Native-local library units using explicitly prepared compiler/provider capabilities')]
[script("bash", "scripts/pse-env", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
unit-native-capability-package pkg features capabilities filter *args:
    package="$1"
    features="$2"
    capabilities="$3"
    filter="$4"
    shift 4
    if [[ "$filter" =~ ^[A-Za-z0-9_]+(::[A-Za-z0-9_]+)*(::)?$ ]]; then filter="test($filter)"; fi
    command=(scripts/pse-env "--native=$capabilities" -- scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} -p "$package" --lib --locked --features "$features" -E "$filter" "$@")
    printf '%q ' "${command[@]}"
    printf '\n'
    exec "${command[@]}"

[group('local')]
[doc('Focused default-feature refusal controls; keeps the real serial QDLDL absence branch')]
[positional-arguments]
feature-absence *args:
    scripts/cargo-bin/cargo-pse-correctness nextest {{ nextest_action }} --locked "$@"

[group('local')]
[doc('Run source-built tests in a fresh native canonical fixture; repeat --build-arg for Cargo selectors and place run filters after --')]
[positional-arguments]
[script("bash", "scripts/pse-env", "--native", "--", "bash", "-euo", "pipefail")]
native-test module_receipt *args:
    module_receipt="$1"
    shift
    scripts/cargo-bin/cargo-pse-correctness build -p pse-operations --features native-service --bin pse-canonical-service --bin pse-canonical-test --locked
    exec target/debug/pse-canonical-test --root "$PWD" --correctness "$PWD/target/host-tools/current/cargo-pse-correctness" --nextest "$(command -v cargo-nextest)" --helper "$PWD/target/debug/pse-canonical-service" --module-receipt "$module_receipt" --state-parent "${TMPDIR:-/tmp}" "$@"

[group('manual')]
[doc('Install one pinned tool: trace-processor (default), mcpls, rr or sccache; no capture')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
[positional-arguments]
diagnostic-tools tool="trace-processor":
    scripts/pse-host diagnostic-tools install "$1"

[group('manual')]
[doc('Install pinned optional Python allocation tools into .venv-diagnostics without building/replacing the project')]
[script("bash", "scripts/pse-env", "--recipe-default-class", "light", "--", "bash", "-euo", "pipefail")]
diagnostic-python-tools:
    UV_PROJECT_ENVIRONMENT=.venv-diagnostics uv sync --locked --group diagnostics --no-install-project

[group('discovery')]
[doc('Build a bounded source-only knowledge corpus and coverage report from explicit JSON selection')]
[positional-arguments]
workspace-sources *args:
    scripts/cargo-bin/cargo-pse-correctness run --bin xtask -p xtask --no-default-features --features evidence-tools --locked -- workspace-sources "$@"

[group('manual')]
[doc('Independent reference helper assertions in the locked numerical-reference environment')]
reference-unit:
    UV_PROJECT_ENVIRONMENT=.venv-parity uv run --locked --only-group parity --python 3.13 -- python -m unittest scripts.tests.test_scientific_references
