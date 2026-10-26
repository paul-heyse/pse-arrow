# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Single-use observation of the Cargo child selected by strict Maturin installation.

The Rust capture owner validates the supported actual arguments, executes Cargo and
seals its raw realization before this adapter permits installation to continue.
Ordinary development never selects this adapter.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

CAPTURE = "PSE_PRODUCER_CARGO_CAPTURE"
PYTHON = "PSE_PRODUCER_CARGO_PYTHON"
PREPARATION_GUARD_FD = "PSE_PRODUCER_PREPARATION_GUARD_FD"
CURRENCY_CAPTURE = "PSE_ORDINARY_CARGO_CAPTURE"
OBSERVER_SOURCE_INPUTS = (
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "xtask/Cargo.toml",
    "xtask/src/main.rs", "xtask/src/producer_identity.rs", "xtask/src/producing_inputs.rs",
    "scripts/workspace.rs",
)


def configure_currency(
    root: Path, directory: Path, environment: Mapping[str, str], *, observer: Mapping[str, object]
) -> dict[str, str]:
    """Observe the actual installer Cargo child without replacing its compiler."""
    path = directory / "ordinary-cargo.json"
    write_new(path, {"version": 1, "workspace_root": str(root.resolve()),
                     "cargo": cargo_executable(environment), "capture_options": [],
                     "observer_preparation": dict(observer)})
    return {**environment, CURRENCY_CAPTURE: str(path), PYTHON: sys.executable,
            "CARGO": str(root / "scripts/producer-cargo")}


def observe_currency(arguments: list[str], environment: Mapping[str, str]) -> int:
    """Forward actual JSON bytes to Maturin and retain only its producing inputs.

    This has no strict-producer reviews, refresh/clean operation, source rewriting
    or persistent admission service. Transient evidence becomes one existing
    preparation receipt only after installation and image association succeed.
    """
    from scripts import build_observation, producing_currency  # noqa: PLC0415

    path = Path(environment[CURRENCY_CAPTURE])
    configuration = read_configuration(path)
    root = Path(str(configuration["workspace_root"]))
    if Path.cwd().resolve() != root.resolve():
        raise ValueError("ordinary Cargo observer belongs to another checkout")
    child_environment = {name: value for name, value in environment.items()
                         if name not in {"CARGO", CURRENCY_CAPTURE, PYTHON}}
    cargo = str(configuration["cargo"])
    if cargo_executable(child_environment) != cargo:
        raise ValueError("ordinary installer selected another Cargo executable")
    if arguments and arguments[0] in {"--version", "-V", "version", "metadata"}:
        return subprocess.call([cargo, *arguments], env=child_environment)
    if not arguments or arguments[0] != "rustc":
        raise ValueError("unsupported ordinary installer Cargo command")
    write_new(path.with_suffix(".claim.json"), {"arguments": arguments})
    arguments_path = path.with_suffix(".arguments.json")
    write_new(arguments_path, arguments)
    graph = path.with_suffix(".graph.json")
    metadata = path.with_suffix(".metadata.json")
    extractor = root / "target/debug/xtask"
    observer_witness = producing_currency.Witness()
    observer_artifact = observer_witness.file(extractor)
    observer_stamp = producing_currency.stamp(extractor) if extractor.is_file() else None
    observer_sources = {str(root / name): observer_witness.file(root / name)
                        for name in OBSERVER_SOURCE_INPUTS}
    prepared = configuration.get("observer_preparation")
    prepared_current = (observer_stamp is not None and isinstance(prepared, dict)
                        and prepared.get("source_basis") == producing_currency.basis(observer_sources)
                        and prepared.get("artifact_stamp") == list(observer_stamp))
    code = subprocess.call([str(extractor), "producing-graph", "--arguments",
                            str(arguments_path), "--output", str(graph)], env=child_environment)
    observed_graph = code == 0
    observed_graph = observed_graph and prepared_current and observer_artifact["sha256"] is not None and all(
        value["sha256"] is not None for value in observer_sources.values())
    if observed_graph:
        metadata.write_bytes(subprocess.check_output(
            [cargo, "metadata", "--format-version=1", "--locked", "--offline", "--all-features"],
            env=child_environment))
    package_witness = None
    package_selection = None
    before_packages = None
    if observed_graph:
        actual_graph, actual_metadata = json.loads(graph.read_bytes()), json.loads(metadata.read_bytes())
        pending, selected = list(actual_graph["roots"]), set()
        while pending:
            index = pending.pop()
            if index not in selected:
                selected.add(index)
                pending.extend(edge["index"] for edge in actual_graph["units"][index]["dependencies"])
        package_ids = {actual_graph["units"][index]["pkg_id"] for index in selected}
        package_selection = {"version": 1, "scope": "ordinary-cargo-inputs", "files": [], "environment": [],
            "directories": sorted({str(Path(package["manifest_path"]).parent)
                                   for package in actual_metadata["packages"] if package["id"] in package_ids})}
        package_witness = producing_currency.Witness()
        try:
            before_packages = package_witness.inputs(package_selection, child_environment)
        except (OSError, ValueError):
            observed_graph = False
    records = path.with_suffix(".messages.jsonl")
    with records.open("wb") as output:
        def record(line: bytes, stream: str) -> None:
            if stream == "stdout":
                output.write(line + b"\n")
        code = build_observation.run_command([cargo, *arguments], env=child_environment, record=record)
    if code:
        return code
    if not observed_graph:
        write_new(path.with_suffix(".status.json"), {"supported": False,
            "cause": "observer-preparation-changed" if not prepared_current else "graph-unavailable"})
        return 0
    if package_witness is None or package_selection is None or before_packages != package_witness.inputs(package_selection, child_environment):
        write_new(path.with_suffix(".status.json"), {"supported": False, "cause": "package-inputs-raced"})
        return 0
    code = subprocess.call([str(extractor), "producing-inputs", "--graph", str(graph),
                            "--messages", str(records), "--metadata", str(metadata),
                            "--output", str(path.with_suffix(".inputs.json"))], env=child_environment)
    if (observer_witness.file(extractor) != observer_artifact
            or producing_currency.stamp(extractor) != observer_stamp
            or observer_sources != {name: observer_witness.file(Path(name)) for name in observer_sources}):
        write_new(path.with_suffix(".status.json"), {"supported": False, "cause": "observer-changed"})
        return 0
    if code == 0:
        actual_inputs = json.loads(path.with_suffix(".inputs.json").read_bytes())
        # The binary is historical observer metadata, not a mutable ongoing
        # input alias. Its source and selected toolchain remain current inputs.
        actual_inputs["files"] = sorted(set(producing_currency.string_list(actual_inputs.get("files", []))) | set(observer_sources))
        path.with_suffix(".inputs.json").write_text(json.dumps(actual_inputs))
        artifact_witness = producing_currency.Witness()
        artifacts = {name: artifact_witness.file(Path(name))
                     for message in actual_inputs["root_artifacts"] for name in message["filenames"]
                     if Path(name).is_file()}
        write_new(path.with_suffix(".selection.json"), {
            "arguments": arguments,
            "effective_environment": effective_environment(child_environment),
            "artifacts": artifacts,
            "observer": {"artifact": observer_artifact,
                         "artifact_stamp": observer_stamp,
                         "preparation_command": prepared.get("command") if isinstance(prepared, dict) else None,
                         "source_basis": producing_currency.basis(observer_sources)},
        })
    write_new(path.with_suffix(".status.json"), {"supported": code == 0,
                                               "cause": "extracted" if code == 0 else "extraction-unavailable"})
    # Missing supported extraction retains the preparation owner's conservative
    # checks. It never turns successful Cargo output into an installation error.
    return 0


def preparation_child_fds(environment: Mapping[str, str]) -> tuple[int, ...]:
    """Retain the target-local owner's lock through actual writer descendants."""
    value = environment.get(PREPARATION_GUARD_FD)
    if value is None:
        return ()
    descriptor = int(value)
    if descriptor < 3 or not stat.S_ISREG(os.fstat(descriptor).st_mode):
        raise ValueError("invalid preparation lifetime descriptor")
    return (descriptor,)


# Effective compiler/SDK selections, never arbitrary environment or credentials.
EFFECTIVE_ENVIRONMENT = (
    "CFLAGS_x86_64_unknown_linux_gnu",
    "CXXFLAGS_x86_64_unknown_linux_gnu",
    "CC_ENABLE_DEBUG_OUTPUT",
    "SOURCE_DATE_EPOCH",
    "TZ",
    "RUSTFMT",
    "LIBCLANG_PATH",
    "CLANG_NO_DEFAULT_CONFIG",
    "PYO3_CONFIG_FILE",
    "PYO3_PYTHON",
    "PYO3_ENVIRONMENT_SIGNATURE",
    "PYTHON_SYS_EXECUTABLE",
    "RUSTC_WRAPPER",
    "PSE_SCCACHE_BINARY",
    "RUSTC_WORKSPACE_WRAPPER",
    "PSE_NATIVE_COMPILER_CACHE",
    "CARGO_BUILD_JOBS",
    "CARGO_TERM_COLOR",
    "PYO3_BUILD_EXTENSION_MODULE",
    "PSE_LLVM_PREFIX",
    "CLANG_PATH",
    "LLVM_CONFIG_PATH",
    "BINDGEN_EXTRA_CLANG_ARGS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CFLAGS",
    "CXXFLAGS",
    "CPPFLAGS",
    "LDFLAGS",
    "CC",
    "CXX",
    "CMAKE_TOOLCHAIN_FILE",
    "COMPILER_PATH",
    "GCC_EXEC_PREFIX",
    "IPOPT_DIR",
    "SCIPOPTDIR",
    "UNO_DIR",
    "PETSC_DIR",
    "SUITESPARSE_INCLUDE_DIR",
    "SUITESPARSE_LIBRARY_DIR",
    "PSE_KLU_PROVIDER_RECEIPT",
    "PSE_ROOT_ISOLATION_DIR",
)


def write_new(path: Path, value: object) -> None:
    descriptor = os.open(
        path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600
    )
    with os.fdopen(descriptor, "w") as output:
        json.dump(value, output, indent=2)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())


def absolute_invocation_path(path: str) -> str:
    """Normalize lexical selection without dereferencing name-dispatched proxies."""
    # Path.resolve() would turn Cargo's rustup proxy into a rustup invocation;
    # Path.absolute() does not normalize parent components like abspath does.
    return os.path.abspath(path)  # noqa: PTH100 -- invocation spelling is authority


def cargo_executable(environment: Mapping[str, str]) -> str:
    selected = shutil.which("cargo", path=environment.get("PATH", os.defpath))
    if selected is None:
        raise ValueError("strict installer requires the selected Cargo executable")
    explicit = environment.get("CARGO")
    if explicit is not None:
        if not explicit:
            raise ValueError(
                "strict installer does not support an empty explicit CARGO"
            )
        chosen = shutil.which(explicit, path=environment.get("PATH", os.defpath))
        if chosen is None or absolute_invocation_path(
            chosen
        ) != absolute_invocation_path(selected):
            raise ValueError(
                "strict installer does not support CARGO differing from the selected PATH Cargo"
            )
    return absolute_invocation_path(selected)


def configure(
    root: Path, directory: Path, options: list[str], environment: Mapping[str, str]
) -> dict[str, str]:
    """Bind one installer to a private immutable command selection before launch."""
    cargo = cargo_executable(environment)
    path = directory / "python-installer-cargo.json"
    write_new(
        path,
        {
            "version": 1,
            "workspace_root": str(root.resolve()),
            "cargo": cargo,
            "capture_options": options,
        },
    )
    return {
        **environment,
        CAPTURE: str(path),
        PYTHON: sys.executable,
        "CARGO": str(root / "scripts/producer-cargo"),
    }


def read_configuration(path: Path) -> dict[str, object]:
    observed = path.lstat()
    parent = path.parent.stat()
    if (
        not stat.S_ISREG(observed.st_mode)
        or observed.st_uid != os.getuid()
        or stat.S_IMODE(observed.st_mode) != 0o600
        or parent.st_uid != os.getuid()
        or stat.S_IMODE(parent.st_mode) != 0o700
    ):
        raise ValueError(
            "strict Cargo observer requires its owned private configuration"
        )
    with path.open() as stream:
        value = json.load(stream)
    if (
        not isinstance(value, dict)
        or type(value.get("version")) is not int
        or value.get("version") != 1
    ):
        raise ValueError("unsupported strict Cargo observer configuration")
    if not isinstance(value.get("workspace_root"), str) or not isinstance(
        value.get("cargo"), str
    ):
        raise TypeError("strict Cargo observer lacks selected executables/context")
    options = value.get("capture_options")
    if not isinstance(options, list) or any(
        not isinstance(item, str) for item in options
    ):
        raise ValueError("strict Cargo observer lacks capture options")
    return value


def effective_environment(environment: Mapping[str, str]) -> dict[str, str]:
    return {
        name: environment[name] for name in EFFECTIVE_ENVIRONMENT if name in environment
    }


def require_sealed_realization(options: list[str]) -> None:
    receipt_path = Path(options[options.index("--output") + 1])
    evidence_path = Path(options[options.index("--build-evidence") + 1])
    with receipt_path.open() as stream:
        receipt = json.load(stream)
    if (
        not isinstance(receipt, dict)
        or receipt.get("persistent_reuse_eligible") is not True
        or receipt.get("reasons") != []
        or receipt.get("receipt_version") != 2
    ):
        raise ValueError(
            "actual installer realization lacks qualified producer capture"
        )
    closure = Path(str(evidence_path) + ".closure")
    with (closure / "index.json").open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if (closure / "COMPLETE").read_text() != digest:
        raise ValueError(
            "actual installer realization lacks completed protected evidence"
        )


def require_unchanged_pyo3_configuration(observation: Mapping[str, object]) -> None:
    config = observation.get("pyo3_configuration")
    if isinstance(config, dict):
        source = Path(str(config["original"]))
        if (
            str(source.resolve()) != config["canonical"]
            or source.stat().st_mode != config["mode"]
            or hashlib.sha256(source.read_bytes()).hexdigest() != config["sha256"]
        ):
            raise ValueError(
                "actual PyO3 configuration changed during installer capture"
            )


def observe(arguments: list[str], environment: Mapping[str, str]) -> int:
    if CURRENCY_CAPTURE in environment:
        return observe_currency(arguments, environment)
    selected = environment.get(CAPTURE)
    if not selected:
        raise ValueError("strict Cargo observer has no selected capture")
    path = Path(selected)
    configuration = read_configuration(path)
    root = Path(str(configuration["workspace_root"]))
    if Path.cwd().resolve() != root.resolve():
        raise ValueError(
            "actual installer Cargo child belongs to a different working directory"
        )
    cargo = str(configuration["cargo"])
    child_environment = {
        name: value
        for name, value in environment.items()
        if name not in {"CARGO", CAPTURE, PYTHON}
    }
    actual_cargo = shutil.which("cargo", path=child_environment.get("PATH", os.defpath))
    if actual_cargo is None or absolute_invocation_path(
        actual_cargo
    ) != absolute_invocation_path(cargo):
        raise ValueError("actual installer PATH selects a different Cargo executable")
    if arguments and arguments[0] in {"--version", "-V", "version", "metadata"}:
        return subprocess.call(
            [cargo, *arguments],
            cwd=root,
            env=child_environment,
            pass_fds=preparation_child_fds(environment),
        )
    if not arguments or arguments[0] != "rustc":
        raise ValueError("unsupported actual installer Cargo command")
    # A second build cannot replace the selected realization or its evidence.
    write_new(
        path.with_suffix(".claim.json"), {"pid": os.getpid(), "arguments": arguments}
    )
    arguments_path = path.with_suffix(".arguments.json")
    write_new(arguments_path, arguments)
    effective = effective_environment(environment)
    observation: dict[str, object] = {
        "version": 1,
        "cargo": cargo,
        "cwd": str(Path.cwd()),
        "arguments": arguments,
        "effective_environment": effective,
    }
    if effective.get("PYO3_CONFIG_FILE"):
        source = Path(effective["PYO3_CONFIG_FILE"])
        if not source.is_absolute():
            source = root / source
        raw = source.read_bytes()
        protected = path.with_suffix(".pyo3-config")
        descriptor = os.open(
            protected, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600
        )
        with os.fdopen(descriptor, "wb") as output:
            output.write(raw)
        observation["pyo3_configuration"] = {
            "original": str(source),
            "canonical": str(source.resolve()),
            "protected": str(protected),
            "sha256": hashlib.sha256(raw).hexdigest(),
            "mode": source.stat().st_mode,
        }
    write_new(path.with_suffix(".observation.json"), observation)
    options = configuration["capture_options"]
    if not isinstance(options, list) or any(
        not isinstance(item, str) for item in options
    ):
        raise TypeError("invalid capture option list")
    command = [
        str(root / "target/debug/xtask"),
        "producer-identity",
        *options,
        "--observed-cargo-arguments",
        str(arguments_path),
    ]
    for name, value in effective.items():
        command.extend(("--environment", f"{name}={value}"))
    result = subprocess.call(
        command,
        cwd=root,
        env=child_environment,
        pass_fds=preparation_child_fds(environment),
    )
    if result == 0:
        try:
            require_sealed_realization(options)
            require_unchanged_pyo3_configuration(observation)
        except (OSError, ValueError, TypeError, KeyError) as error:
            write_new(
                path.with_suffix(".status.json"),
                {
                    "returncode": 1,
                    "capture_returncode": result,
                    "sealed": False,
                    "refusal": str(error),
                },
            )
            raise
    write_new(
        path.with_suffix(".status.json"), {"returncode": result, "sealed": result == 0}
    )
    return result


def main() -> int:
    try:
        return observe(sys.argv[1:], os.environ)
    except (OSError, ValueError, TypeError, KeyError) as error:
        print(
            f"strict Cargo observer refused: {type(error).__name__}: {error}",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
