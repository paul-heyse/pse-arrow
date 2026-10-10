# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Declared local validation scope; commands remain owned by the justfile."""

from dataclasses import dataclass, replace

from scripts.native_cache import INPUT_ENV as NATIVE_BUILD_ENVIRONMENT


@dataclass(frozen=True)
class Gate:
    name: str
    args: tuple[str, ...] = ()
    report: str | None = None
    dependencies: tuple[str, ...] = ()
    role: str = "required"
    phase: str = "functional"
    enumerate_native: bool = False
    recipe: str | None = None
    mode: str = ""
    profile: str = ""
    input_scope: str = "unknown"


# Explicit classifications cover direct recipes as well as aggregate expansion.
PERFORMANCE_GATES = frozenset(
    {
        "bench-smoke",
        "bench-cache",
        "bench-production",
        "bench-consolidation",
        "bench-consolidation-native",
        "case-measure",
    }
)


def declared(name: str) -> Gate:
    scope = (
        "generation"
        if name.startswith("codegen-")
        else "documentation"
        if name in {"docs", "docs-rust", "doc-lint"}
        else "rust-product"
        if name
        in {
            "test",
            "native-test",
            "doctest",
            "check",
            "clippy-default",
            "clippy-no-default",
        }
        else "python-product"
        if name in {"native-python", "py-sync-native", "python-stubs"}
        else "tooling"
    )
    return _declared(name, scope)


def _declared(name: str, scope: str) -> Gate:
    """Tool findings and unsupported capabilities retain their declared authority."""
    if name in {
        "audit-dependencies",
        "audit-advisories",
        "audit-shear",
        "audit-machete",
    }:
        return Gate(name, input_scope=scope, role="advisory")
    if name == "doc-lint":
        return Gate(name, input_scope=scope, role="deferred")
    if name in PERFORMANCE_GATES:
        return Gate(name, input_scope=scope, phase="performance")
    return Gate(name, input_scope=scope)


# Aggregates expand recursively, so both ordinary recipes and the full campaign
# have the same continuation behavior. Mutating setup remains dependency ordered.
GROUPS = {
    "fmt-check": ("fmt-rust-check", "lint-toml"),
    "clippy": ("clippy-default", "clippy-no-default"),
    "lint-repo": (
        "lint-toml",
        "lint-typos",
        "lint-license",
        "lint-actions",
        "lint-zizmor",
        "lint-shell",
        "lint-ast",
    ),
    "quality": (
        "python-contracts-check",
        "fmt-py-check",
        "lint-py",
        "typecheck",
        "lint-imports",
        "lint-repo",
        "lint-agents",
        "setup-test",
        "solver-pin-check",
    ),
    "adr-lint": ("adr-frontmatter-check", "adr-index-check", "register-lint"),
    "codegen-check": (
        "codegen-relations-check",
        "codegen-rust-contracts-check",
        "codegen-python-check",
        "python-contracts-check",
        "python-stubs-check",
        "codegen-docs-check",
        "codegen-surreal-check",
        "codegen-bindgen-check",
        "codegen-schemas-check",
        "codegen-hakari-check",
    ),
    "codegen-contracts-check": (
        "codegen-rust-contracts-check",
        "codegen-python-check",
        "codegen-docs-check",
        "codegen-surreal-check",
    ),
    "features-powerset": ("features-combinations", "features-no-default"),
    "governance": ("governance-tests", "codegen-check", "family-check"),
    "ci-fast": ("fmt-check", "check", "clippy", "test", "doctest"),
    "ci-pr": (
        "ci-fast",
        "governance",
        "docs-rust",
        "bench-smoke",
        "quality",
        "adr-lint",
        "docs",
        "py-test",
    ),
    "policy": ("audit-dependencies", "audit-advisories"),
    # The working bundles (AGENTS.md *Execution rhythm*); each step is a recipe.
    "turn-end": ("adr-index", "fmt"),
    "ready": ("skills-sync", "doctor"),
    "hygiene": (
        "lint-agents",
        "adr-frontmatter-check",
        "adr-index-check",
        "register-lint",
        "lint-typos",
        "lint-license",
        "lint-actions",
        "lint-shell",
        "lint-ast",
        "lint-py",
        "typecheck",
        "lint-imports",
        "engine-boundary-check",
        "solver-pin-check",
        "family-check",
        "codegen-check",
        "clippy-default",
        "clippy-no-default",
        "docs-rust",
    ),
    "deps-report": (
        "audit-dependencies",
        "audit-advisories",
        "audit-shear",
        "audit-machete",
    ),
}


def expand(names: tuple[str, ...]) -> list[Gate]:
    """Expand independent recipe groups, running each leaf once per invocation."""
    output: dict[str, Gate] = {}
    for name in names:
        for declared_gate in (
            expand(GROUPS[name]) if name in GROUPS else [declared(name)]
        ):
            gate = declared_gate
            if name == "policy":
                gate = replace(gate, role="required")
            existing = output.get(gate.name)
            if existing is None or gate.role == "required":
                output[gate.name] = gate
    return list(output.values())


def comprehensive(python_profile: str = "dev") -> list[Gate]:
    """Canonical linked graph and separate Python runtime-budget processes."""
    static = expand(
        (
            "fmt-check",
            "clippy",
            "quality",
            "governance",
            "adr-lint",
            "check",
            "docs-rust",
            "docs",
        )
    )
    static = [
        replace(
            gate,
            args=("{output}",),
            report="{output}/setup-test.xml",
            recipe="setup-test-report",
            mode="python-setup-unit",
        )
        if gate.name == "setup-test"
        else gate
        for gate in static
        if gate.name != "governance-tests"
    ]
    deployment = python_profile == "producer"
    deployment_gates = (
        [
            Gate(
                "producer-deployment",
                ("{output}/deployment",),
                dependencies=("py-sync-native",),
                mode="source-bound-production-capture",
                profile="producer",
                input_scope="deployment-capture",
            ),
            Gate(
                "python-deployment-association",
                (
                    "{output}/deployment-association",
                    "--producer-deployment",
                    "python/pse/tests/test_canonical_results.py::test_canonical_eligible_deployment_receipt_reopens_original_scalar",
                ),
                report="{output}/deployment-association/native-python.xml",
                dependencies=("producer-deployment",),
                recipe="native-python",
                mode="python-native",
                input_scope="python-product",
            ),
            replace(
                native_gate("native-deployment-association"),
                args=(
                    "--profile",
                    "local",
                    "--run-ignored",
                    "all",
                    "-E",
                    "test(=math::portable::canonical_deployment_tests::canonical_deployment_actual_receipts_enforce_selected_role)",
                ),
                dependencies=("python-deployment-association",),
            ),
        ]
        if deployment
        else []
    )
    return [
        Gate(
            "py-sync-native",
            (python_profile,),
            profile=python_profile,
            input_scope="python-product",
        ),
        *static,
        Gate(
            "python-stubs",
            ("--check",),
            dependencies=("py-sync-native",),
            input_scope="python-product",
        ),
        Gate(
            "producer-fixture",
            ("{output}/producer-fixture.json",),
            input_scope="tooling",
        ),
        *deployment_gates,
        Gate(
            "feature-absence",
            (
                "--profile",
                "local",
                "-p",
                "pse-backend-native",
                "-p",
                "pse-relations",
                "-E",
                "test(feature_absence::) | test(clarabel_tests::clarabel_mkl_pardiso_refused_without_profile)",
            ),
            "{target}/nextest/local/junit.xml",
            enumerate_native=True,
            recipe="feature-absence",
            mode="force-validate-feature-absence",
            input_scope="rust-product",
            profile="local",
            dependencies=("producer-fixture",),
        ),
        Gate("doctest", input_scope="rust-product"),
        replace(
            native_gate(),
            dependencies=("producer-fixture", "python-deployment-association")
            if deployment
            else ("producer-fixture",),
        ),
        replace(
            FUNCTIONAL_SCOPES["managed-primary"],
            dependencies=("native-test",),
        ),
        Gate(
            "native-python",
            ("{output}", "--producer-deployment") if deployment else ("{output}",),
            "{output}/native-python.xml",
            dependencies=("py-sync-native", "python-deployment-association")
            if deployment
            else ("py-sync-native",),
            mode="python-native",
            input_scope="python-product",
        ),
        Gate(
            "managed-python",
            (
                "{output}/managed-python",
                "--managed-primary-route",
                "--producer-deployment",
            )
            if deployment
            else ("{output}/managed-python", "--managed-primary-route"),
            "{output}/managed-python/native-python.xml",
            dependencies=("py-sync-native", "python-deployment-association")
            if deployment
            else ("py-sync-native",),
            recipe="native-python",
            mode="python-native",
            input_scope="python-product",
        ),
    ]


EXCLUSIONS = {
    "other platforms and distribution": "This command qualifies the local pinned environment; remote CI, wheels and other hosts have separate commands.",
    "release, coverage, feature powerset, alternate toolchains": "Available separately; not part of the default and native local profiles.",
    "parity": "Run just parity separately when IDAES parity is in scope.",
    "performance": "Run case-measure after functional qualification, with an explicit functional report.",
    "reviews": "Architecture and scientific review are judgments recorded in the owning plan, not command-exit evidence.",
    "scheduled register checks": "register-check is time-dependent; deterministic adr-lint uses register-lint.",
    "dependency inventory": "deps-report is advisory; policy is separately available and strict.",
}

# Versioned, conservative subsystem declarations project the one captured map.
# A missing declaration deliberately retains every contextual input.
INPUT_SCOPE_VERSION = 3
RUST_INPUTS = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".python-version",
    "pyproject.toml",
    "uv.lock",
    ".cargo",
    "crates",
    "xtask",
    "tests",
    "benches",
    "vendor",
    "packages",
    "docker",
    "justfile",
    ".config",
    # Execution, admission, selection and cleanup all affect the observation.
    # Keep their conservative owner closure rather than filename-prefix guesses.
    "scripts",
)
INPUT_SCOPES = {
    "rust-product": RUST_INPUTS,
    "python-product": (
        *RUST_INPUTS,
        "python",
        "pyproject.toml",
        "uv.lock",
        "conftest.py",
        "scripts/python",
    ),
    "deployment-capture": (
        *RUST_INPUTS,
        "python",
        "pyproject.toml",
        "uv.lock",
        "scripts",
    ),
    # Tool policies and data are deliberately included: many governance tools read them.
    "tooling": (
        "scripts",
        "xtask",
        "tests",
        ".config",
        ".github",
        ".claude",
        ".codex",
        ".agents",
        "AGENTS.md",
        "CLAUDE.md",
        "justfile",
        "pyproject.toml",
        "uv.lock",
        "Cargo.toml",
        "Cargo.lock",
        "crates",
        "python",
        "docs",
        "sgrules",
        "sgutils",
        "sgtests",
        "sgconfig.yml",
        "deny.toml",
        "REUSE.toml",
        "LICENSES",
        "clippy.toml",
        ".gitignore",
        ".pre-commit-config.yaml",
    ),
    "documentation": (
        "docs",
        "scripts",
        "xtask",
        "crates",
        "python",
        ".config",
        "Cargo.toml",
        "Cargo.lock",
        "justfile",
        "pyproject.toml",
        "uv.lock",
        "README.md",
    ),
    "generation": (
        "crates",
        "xtask",
        "scripts",
        ".config",
        "python",
        "docs/generated",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".python-version",
        ".cargo",
        "justfile",
        "pyproject.toml",
        "uv.lock",
    ),
}
PRODUCT_ENVIRONMENT = (
    *NATIVE_BUILD_ENVIRONMENT,
    "RUSTUP_TOOLCHAIN",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC_WRAPPER",
    "SUITESPARSE_LIBRARY_DIR",
    "SUITESPARSE_INCLUDE_DIR",
    "UNO_DIR",
    "PETSC_DIR",
    "IPOPT_DIR",
    "SCIPOPTDIR",
    "PSE_SOLVER_IMAGE",
    "PSE_ROOT_ISOLATION_DIR",
    "PYO3_PYTHON",
    "PYO3_CONFIG_FILE",
    "PYO3_ENVIRONMENT_SIGNATURE",
    "LD_LIBRARY_PATH",
    "PSE_LLVM_PREFIX",
    "LIBCLANG_PATH",
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "PSE_SURREAL_STATE",
    "PSE_WORKER_BINARY",
    "PSE_PRODUCER_RECEIPT",
    "PSE_WORKER_PRODUCER_RECEIPT",
    "PSE_PYTHON_PRODUCER_RECEIPT",
    "PSE_PYTHON_DEPLOYMENT_ATTESTATION",
    "PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS",
    "PSE_NATIVE_PROVIDER_RECEIPT",
    "PSE_PRODUCER_FIXTURE_RECEIPT",
    "PSE_MEMORY_MAX",
    "SYMBOLICA_LICENSE",
    "LOCAL_NATIVE_ENVIRONMENT",
    "EFFECTIVE_NATIVE_CONFIGURATION",
)
PRODUCER_REVIEW_INPUTS = (
    "PSE_RUNTIME_PRODUCER_DECLARATIONS",
    "PSE_WORKER_PRODUCER_DECLARATIONS",
    "PSE_PYTHON_PRODUCER_DECLARATIONS",
    "PSE_NATIVE_PRODUCER_INPUTS",
    "PSE_PYTHON_PRODUCER_INPUTS",
)
INPUT_ENVIRONMENT = (
    *PRODUCT_ENVIRONMENT,
    "UV_PROJECT_ENVIRONMENT",
    "PSE_TEST_WORKERS",
    "PYTHONPATH",
    "PYTHONHASHSEED",
    *PRODUCER_REVIEW_INPUTS,
)


def input_identity(scope: str, snapshot: dict, environment: dict) -> dict:
    """Project captured identities; preserve absent paths, modes and symlink hashes."""
    paths = INPUT_SCOPES.get(scope)
    return {
        "scope": scope,
        "version": INPUT_SCOPE_VERSION,
        "definition": list(paths) if paths is not None else None,
        "files": {
            name: value
            for name, value in snapshot.items()
            if paths is None
            or any(
                name == prefix
                or name.startswith(prefix + "/")
                or (prefix.startswith("scripts/") and name.startswith(prefix))
                for prefix in paths
            )
        },
        "environment": {
            key: value
            for key, value in environment.items()
            if paths is None or key in INPUT_ENVIRONMENT
        },
    }


def native_gate(name: str = "native-test", selection: str | None = None) -> Gate:
    """Declared linked invocation; covering identity never infers filter equivalence."""
    return Gate(
        name,
        ("--profile", "local", *(("-E", selection) if selection else ())),
        "{target}/nextest/local/junit.xml",
        recipe="native-test" if name != "native-test" else None,
        mode="native-force-validate",
        profile="local",
        input_scope="rust-product",
    )


# Small subsystem selections, rather than an inventory of exact individual tests.
# Conservative owner packages cover controls whose narrower namespace is unproven.
FUNCTIONAL_SCOPES = {
    "native": native_gate(),
    "managed-primary": replace(
        native_gate("managed-native"),
        args=("--profile", "local", "--managed-primary-route"),
    ),
    "process": native_gate(
        "functional-process",
        "package(pse-runtime) | (package(pse-tests-conformance) & test(acceptance::))",
    ),
    "preparation": native_gate(
        "functional-preparation", "package(pse-runtime) | package(pse-compiler)"
    ),
    "admission": native_gate(
        "functional-admission", "package(pse-authoring) | package(pse-runtime)"
    ),
    "lifecycle": native_gate(
        "functional-lifecycle",
        "package(pse-runtime) | package(pse-operations) | package(pse-tests-lifecycle) | (package(pse-tests-conformance) & test(acceptance::connected_results_resource::))",
    ),
}
