# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit nextest historical archives under existing result and artifact owners.

Historical replay never enters source-bound prepared-invocation reuse. The archive
has its own retained producer receipt; each replay gets a new selected invocation.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import TYPE_CHECKING

from pydantic import BaseModel, ConfigDict, Field, model_validator

from scripts import (
    arrow_validation,
    cargo_tools,
    native_operation,
    native_tests,
    scientific_inputs,
    test_resources,
    test_run,
    validation,
    validation_receipts,
)

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "selected-test-archive-v1"
REPLAY_SCHEMA = "selected-test-replay-v1"
NATIVE_UNITS = frozenset(
    {
        "pse-backend-native",
        "pse-ipopt-sys",
        "pse-uno-sys",
        "pse-petsc-sys",
        "pse-kinsol-sys",
        "pse-root-isolation-sys",
        "pse-scip-sys",
    }
)
RUNTIME_ENV = (
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "RUST_MIN_STACK",
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "SYMBOLICA_LICENSE",
)


class ArchiveError(ValueError):
    """A selected historical closure is unavailable or changed."""


class ArchiveRequest(BaseModel):
    """Host selection only; native closure admission remains below."""

    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    output: str = Field(min_length=1)
    package: str = Field(min_length=1)
    filter: str = Field(
        min_length=1,
        description="Native nextest filter expression, for example test(checksum). Bare words are not expressions on this archival route.",
    )
    test: str | None = Field(default=None, min_length=1)
    lib: bool = False
    features: str = ""
    fixture: list[str] = Field(default_factory=list)

    @model_validator(mode="after")
    def selected_target(self) -> ArchiveRequest:
        if self.lib and self.test is not None:
            raise ValueError("select either a library or a named test target")
        for value in self.fixture:
            relative_input(value)
        return self


class ReplayRequest(BaseModel):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    archive: str = Field(min_length=1)
    output: str = Field(min_length=1)


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def runtime_environment(environment: Mapping[str, str]) -> dict[str, str | None]:
    """Retain opaque values, including absence, without publishing license bytes."""
    return {
        name: hashlib.sha256(environment[name].encode()).hexdigest()
        if name in environment
        else None
        for name in RUNTIME_ENV
    }


def relative_input(value: str) -> Path:
    path = Path(value)
    if (
        path.is_absolute()
        or not path.parts
        or any(part in {".", ".."} for part in path.parts)
    ):
        raise ArchiveError(
            "Archive input must be a workspace-relative path without traversal"
        )
    return path


def stage_inputs(root: Path, destination: Path, inputs: list[str]) -> dict[str, str]:
    """Copy declared ordinary fixture/config files; never follow symlinks."""
    manifest: dict[str, str] = {}
    for supplied in inputs:
        selected = root / relative_input(supplied)
        if not selected.exists() or any(
            path.is_symlink() for path in (selected, *selected.parents)
        ):
            raise ArchiveError(f"Missing or symlinked archive input: {supplied}")
        paths = sorted(selected.rglob("*")) if selected.is_dir() else [selected]
        if not paths:
            raise ArchiveError(f"Empty declared archive input: {supplied}")
        for path in paths:
            if path.is_symlink():
                raise ArchiveError(
                    f"Symlinked archive fixture: {path.relative_to(root)}"
                )
            if path.is_dir():
                continue
            if not path.is_file():
                raise ArchiveError("Archive fixtures must be ordinary files")
            relative = str(path.relative_to(root))
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
            manifest[relative] = digest(target)
            if digest(path) != manifest[relative]:
                raise ArchiveError("Archive fixture changed during capture")
    return manifest


def verify_inputs(workspace: Path, manifest: Mapping[str, str]) -> None:
    for relative, expected in manifest.items():
        path = workspace / relative_input(relative)
        if (
            any(part.is_symlink() for part in (path, *path.parents))
            or not path.is_file()
            or digest(path) != expected
        ):
            raise ArchiveError(
                f"Historical fixture/config missing or changed: {relative}"
            )


def native_units(metadata: dict, graph: dict) -> list[str]:
    names = {package["id"]: package["name"] for package in metadata["packages"]}
    return sorted(
        {
            names[unit["pkg_id"]]
            for unit in graph["units"]
            if names[unit["pkg_id"]] in NATIVE_UNITS
        }
    )


def require_pure_closure(metadata: dict, graph: dict) -> None:
    selected = native_units(metadata, graph)
    if selected:
        raise ArchiveError(
            "Native archival is not adopted: deployment/ABI/license/config closure is not "
            "staged by native-profile-v1; regenerate the exact historical checkout through "
            "its native owner instead. Selected native units: " + ", ".join(selected)
        )


def library_closure(binaries: list[str], environment: dict[str, str]) -> dict[str, str]:
    files: dict[str, str] = {}
    for binary in binaries:
        observed = subprocess.run(
            ["ldd", binary],
            env=environment,
            text=True,
            capture_output=True,
            check=False,
        )
        if (
            observed.returncode
            and "not a dynamic executable" not in observed.stderr
            and "statically linked" not in observed.stdout
        ):
            raise ArchiveError(
                f"Cannot inspect runtime libraries for {binary}: {observed.stderr}"
            )
        if "not found" in observed.stdout:
            raise ArchiveError("Historical executable has a missing runtime dependency")
        for line in observed.stdout.splitlines():
            absolute = [word for word in line.split() if word.startswith("/")]
            if absolute:
                path = Path(absolute[0]).resolve(strict=True)
                files[str(path)] = digest(path)
    return files


def stage_scientific_input(native: dict, workspace: Path) -> tuple[dict, dict]:
    """Retain a logical input role, independently of the producer's absolute path."""
    supplied = (
        native.get("rust_invocation", {})
        .get("fixtures", {})
        .get(scientific_inputs.ROLE)
    )
    if supplied is None:
        return {}, {}
    source = Path(supplied)
    expected = native["files"].get(str(source))
    if source.is_symlink() or not source.is_file() or expected != digest(source):
        raise ArchiveError("Scientific producer input changed before staging")
    relative = ".pse-inputs/scientific.json"
    destination = workspace / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("xb") as stream:
        stream.write(source.read_bytes())
    if digest(destination) != expected or digest(source) != expected:
        raise ArchiveError("Scientific producer input changed during staging")
    return {relative: expected}, {
        scientific_inputs.ROLE: {"path": relative, "sha256": expected}
    }


def bind_input_roles(
    manifest: dict, workspace: Path, environment: dict[str, str]
) -> None:
    """Never inherit an original producer binding or use the live checkout on replay."""
    environment.pop(scientific_inputs.ROLE, None)
    for role, binding in manifest.get("input_roles", {}).items():
        if role != scientific_inputs.ROLE:
            raise ArchiveError("Unknown historical input role")
        relative = str(relative_input(binding["path"]))
        if manifest["fixtures"].get(relative) != binding["sha256"]:
            raise ArchiveError("Historical input role is not a staged fixture")
        path = workspace / relative
        try:
            verify_inputs(workspace, {relative: binding["sha256"]})
        except ArchiveError as error:
            raise ArchiveError("Historical input role bytes changed") from error
        environment[role] = str(path.resolve())


def preflight(manifest: dict, directory: Path, environment: dict[str, str]) -> None:
    if (
        manifest.get("schema") != SCHEMA
        or manifest.get("native_replay") != "unsupported"
    ):
        raise ArchiveError("Unsupported archive closure contract")
    if manifest.get("platform") != [platform.system(), platform.machine()]:
        raise ArchiveError("Archive runtime platform differs")
    if manifest.get("nextest") != cargo_tools.selected_identity("cargo-nextest"):
        raise ArchiveError("Archive requires its exact nextest tool identity")
    if manifest.get("runtime_environment") != runtime_environment(environment):
        raise ArchiveError("Historical runtime environment differs")
    verify_inputs(directory / "workspace", manifest["fixtures"])
    bind_input_roles(manifest, directory / "workspace", {})
    archive = directory / "tests.tar.zst"
    if (
        not archive.is_file()
        or archive.is_symlink()
        or digest(archive) != manifest["archive_sha256"]
    ):
        raise ArchiveError("Retained test archive missing or changed")
    for supplied, expected in manifest["libraries"].items():
        path = Path(supplied)
        if not path.is_file() or digest(path) != expected:
            raise ArchiveError(
                f"Historical runtime dependency missing or changed: {supplied}"
            )


def capture_json(command: list[str], environment: dict[str, str]) -> dict:
    observation: dict[str, object] = {}
    with (
        tempfile.TemporaryFile(mode="w+") as stdout,
        tempfile.TemporaryFile(mode="w+") as stderr,
    ):
        code = native_operation.run_child_scope(
            command,
            environment,
            cwd=ROOT,
            observation=observation,
            stdout=stdout.fileno(),
            stderr=stderr.fileno(),
        )
        if code or observation.get("owned_drain") != "complete":
            stderr.seek(0)
            raise ArchiveError(
                f"nextest inventory failed or did not drain ({code}): {stderr.read()}"
            )
        stdout.seek(0)
        return validation_receipts.native_inventory(stdout.read())


def new_output(path: Path) -> str:
    path = path.absolute()
    if (
        not path.is_relative_to(ROOT / "build")
        or ".." in path.parts
        or any(parent.is_symlink() for parent in (path, *path.parents))
    ):
        raise ArchiveError(
            "Experiment output must be a fresh owned checkout build directory"
        )
    path.mkdir(parents=True, mode=0o700, exist_ok=False)
    return test_resources.register_report(path)


def seal(
    directory: Path,
    resource: str,
    receipt: dict,
    provenance: list[str],
    scratch: list[str],
) -> None:
    validation.write_json(directory / "checks.json", receipt)
    roles = {
        str(path.relative_to(directory)): "evidence"
        for path in directory.rglob("*")
        if path.is_file()
    }
    roles.update(dict.fromkeys(provenance, "provenance"))
    roles.update(dict.fromkeys(scratch, "scratch"))
    roles["checks.json"] = "receipt"
    # A selected investigation remains pinned until explicitly released, including
    # after successful execution. The ordinary report owner provides disposition.
    test_resources.pin(resource, True)
    test_resources.finish_report(
        resource, receipt, roles=roles, required_provenance=provenance
    )


def archive(request: ArchiveRequest) -> dict:
    """Archive one explicit pure closure, using the existing resource and child owners."""
    output = Path(request.output).absolute()
    resource = new_output(output)
    environment = arrow_validation.rust_test_environment(os.environ)
    environment[test_run.EXECUTION_EFFECTS] = "native-local"
    for name in (
        test_resources.MARKER,
        "PSE_NATIVE_PROVENANCE",
        test_run.RUST_OBSERVER,
        scientific_inputs.ROLE,
    ):
        environment.pop(name, None)
    build = ["cargo", "nextest", "list", "--locked", "-p", request.package]
    build.extend(["--test", request.test] if request.test else ["--lib"])
    if request.features:
        build.extend(["--features", request.features])
    resolution = native_tests.correctness_resolution(build, environment)
    metadata, graph = json.loads(resolution.metadata), json.loads(resolution.unit_graph)
    try:
        require_pure_closure(metadata, graph)
    except ArchiveError as error:
        validation.write_json(
            output / "native-nonadoption.json",
            {
                "schema": "native-archive-nonadoption-v1",
                "reason": str(error),
                "source_commit": validation.git(ROOT, "rev-parse", "HEAD")
                .decode()
                .strip(),
                "selected_native_units": native_units(metadata, graph),
                "resource": resource,
                "revisit": "A concrete native investigation can stage and verify actual deployment, ABI, license/config/data and admission closure.",
            },
        )
        raise
    provenance = output / "producer-provenance.json"
    selection = output / "producer-selection.json"
    original = [
        "cargo",
        "nextest",
        "run",
        *resolution.command[3:],
        "-E",
        request.filter,
    ]
    execution, _worker = native_tests.ordinary_rust_capture(
        original, provenance, selection, environment, effects="native-local"
    )
    native = json.loads(provenance.read_text())
    environment.update(native_tests.rust_fixture_bindings(provenance, environment))
    inventory = json.loads(selection.read_text())
    selected = test_run.rust_selected(inventory)
    if not selected:
        raise ArchiveError("Historical selection is empty")
    package = next(
        package
        for package in metadata["packages"]
        if package["name"] == request.package
    )
    package_manifest = str(Path(package["manifest_path"]).relative_to(ROOT))
    fixtures = stage_inputs(
        ROOT,
        output / "workspace",
        ["Cargo.toml", ".config/nextest.toml", package_manifest, *request.fixture],
    )
    source_fixtures = fixtures.copy()
    scientific_fixtures, input_roles = stage_scientific_input(
        native, output / "workspace"
    )
    fixtures.update(scientific_fixtures)
    environment["PSE_NATIVE_PROVENANCE"] = str(provenance)
    native_tests.prepared_rust_inventory(execution, environment, inventory=inventory)
    invocation = test_resources.invocation("rust", selected)
    environment[test_resources.MARKER] = str(invocation)
    producer_command, report = test_run.rust_report_command(
        invocation, ["cargo", "nextest", "run", *execution]
    )
    test_run.execute_selected(
        invocation,
        producer_command,
        environment,
        report,
        selected=[
            {"class": item["binary_id"], "name": item["name"]} for item in selected
        ],
        launch=lambda process: native_operation.run_child_scope(
            producer_command, environment, cwd=ROOT, observation=process
        ),
    )
    producer = json.loads(invocation.with_suffix(".terminal.json").read_text())
    validation.write_json(output / "producer-terminal.json", producer)
    if producer.get("process", {}).get("forwarded_signal") is not None:
        result = {
            "status": "interrupted",
            "exit_code": producer["exit_code"],
            "output": str(output),
            "resource": resource,
            "producer_invocation": str(invocation),
            "producer_status": producer["status"],
            "process": producer["process"],
            "archive_created": False,
        }
        validation.write_json(output / "archive-interrupted.json", result)
        test_resources.pin(resource, True)
        return result
    archive_command = [
        "cargo",
        "nextest",
        "archive",
        *resolution.command[3:],
        "--archive-file",
        str(output / "tests.tar.zst"),
    ]
    archive_observation: dict[str, object] = {}
    with (output / "archive.log").open("w") as log:
        code = native_operation.run_child_scope(
            archive_command,
            environment,
            cwd=ROOT,
            observation=archive_observation,
            stdout=log.fileno(),
            stderr=log.fileno(),
        )
    if code or archive_observation.get("owned_drain") != "complete":
        raise ArchiveError(f"Archive creation failed or did not drain: {code}")
    native_tests.prepared_rust_inventory(execution, environment, inventory=inventory)
    verify_inputs(output / "workspace", fixtures)
    verify_inputs(ROOT, source_fixtures)
    manifest = {
        "schema": SCHEMA,
        "nextest": cargo_tools.selected_identity("cargo-nextest"),
        "platform": [platform.system(), platform.machine()],
        "source_commit": validation.git(ROOT, "rev-parse", "HEAD").decode().strip(),
        "source_cut": native["rust_invocation"]["source"],
        "toolchain": native["toolchain"],
        "validated_unit_graph": graph,
        "build_command": resolution.command,
        "filter": request.filter,
        "selected": selected,
        "fixtures": fixtures,
        "input_roles": input_roles,
        "libraries": library_closure(
            [item["executable"] for item in selected], environment
        ),
        "runtime_environment": runtime_environment(environment),
        "archive_sha256": digest(output / "tests.tar.zst"),
        "native_replay": "unsupported",
        "producer_invocation": str(invocation),
        "resource": resource,
        "claims": "Historical producer inputs; replay does not qualify current source.",
    }
    validation.write_json(output / "archive.json", manifest)
    receipt = {
        "version": 5,
        "complete": True,
        "baseline_failures": 0,
        "required_checks_covered": True,
        "input_coverage": True,
        "source_unchanged": True,
        "provenance_errors": [],
        "scope": [{"name": "historical-producer"}, {"name": "archive"}],
        "checks": [
            {
                **producer,
                "gate": "historical-producer",
                "log": "producer-terminal.json",
            },
            {
                "gate": "archive",
                "status": "passed",
                "exit_code": code,
                "log": "archive.log",
                "process": archive_observation,
            },
        ],
        "historical_archive": "archive.json",
    }
    seal(
        output,
        resource,
        receipt,
        [
            "archive.json",
            "producer-provenance.json",
            *["workspace/" + name for name in fixtures],
        ],
        ["tests.tar.zst"],
    )
    return {
        "status": "archived",
        "output": str(output),
        "archive_manifest": str(output / "archive.json"),
        "resource": resource,
        "producer_invocation": str(invocation),
        "producer_status": producer["status"],
    }


def inspect(
    directory: Path,
    *,
    offset: int = 0,
    limit: int | None = None,
    manifest_sha256: str | None = None,
) -> dict:
    if offset < 0 or (limit is not None and not 1 <= limit <= 100):
        raise ArchiveError("select nonnegative offset and limit in 1..100")
    directory = directory.resolve(strict=True)
    with test_resources.reference_report(directory, None):
        identity = digest(directory / "archive.json")
        if manifest_sha256 is not None and identity != manifest_sha256:
            raise ArchiveError("Historical archive manifest identity changed")
        manifest = json.loads((directory / "archive.json").read_text())
        if manifest.get("schema") != SCHEMA:
            raise ArchiveError("Unsupported archive document")
        archive_path = directory / "tests.tar.zst"
        if (
            not archive_path.is_file()
            or archive_path.is_symlink()
            or digest(archive_path) != manifest["archive_sha256"]
        ):
            raise ArchiveError("Retained test archive missing or changed")
        verify_inputs(directory / "workspace", manifest["fixtures"])
        if digest(directory / "archive.json") != identity:
            raise ArchiveError("Historical archive manifest changed during inspection")
        selected = manifest["selected"]
        end = len(selected) if limit is None else offset + limit
        return {
            "status": "inspected",
            "schema": SCHEMA,
            "source_commit": manifest["source_commit"],
            "selected": selected[offset:end],
            "archive": str(directory),
            "manifest_sha256": identity,
            "selection_total": len(selected),
            "next_offset": end if end < len(selected) else None,
            "fixtures": manifest["fixtures"],
            "native_replay": manifest["native_replay"],
            "producer_invocation": manifest["producer_invocation"],
            "claims": manifest["claims"],
            "libraries": manifest["libraries"],
        }


def replay(directory: Path, output: Path) -> dict:
    directory = directory.resolve(strict=True)
    output = output.absolute()
    environment = arrow_validation.rust_test_environment(os.environ)
    environment[test_run.EXECUTION_EFFECTS] = "native-local"
    for name in (
        test_resources.MARKER,
        "PSE_NATIVE_PROVENANCE",
        test_run.RUST_OBSERVER,
        scientific_inputs.ROLE,
    ):
        environment.pop(name, None)
    resource = new_output(output)
    with test_resources.reference_report(directory, output / "checks.json"):
        manifest = json.loads((directory / "archive.json").read_text())
        preflight(manifest, directory, environment)
        fixture_manifest = stage_inputs(
            directory / "workspace", output / "workspace", list(manifest["fixtures"])
        )
        bind_input_roles(manifest, output / "workspace", environment)
        extraction = output / "extracted"
        extraction.mkdir(mode=0o700)
        common = [
            "--archive-file",
            str(directory / "tests.tar.zst"),
            "--extract-to",
            str(extraction),
            "--workspace-remap",
            str(output / "workspace"),
        ]
        inventory = capture_json(
            [
                "cargo",
                "nextest",
                "list",
                *common,
                "-E",
                manifest["filter"],
                "--message-format",
                "json",
                "--config-file",
                str(output / "workspace/.config/nextest.toml"),
            ],
            environment,
        )
        selected = test_run.rust_selected(inventory)
        if sorted((row["binary_id"], row["name"]) for row in selected) != sorted(
            (row["binary_id"], row["name"]) for row in manifest["selected"]
        ):
            raise ArchiveError("Historical selection changed during extraction")
        if (
            library_closure([item["executable"] for item in selected], environment)
            != manifest["libraries"]
        ):
            raise ArchiveError(
                "Extracted executable resolves different runtime dependencies"
            )
        invocation = test_resources.invocation("rust", selected)
        environment[test_resources.MARKER] = str(invocation)
        run = [
            "cargo",
            "nextest",
            "run",
            "--binaries-metadata",
            str(extraction / "target/nextest/binaries-metadata.json"),
            "--cargo-metadata",
            str(extraction / "target/nextest/cargo-metadata.json"),
            "--target-dir-remap",
            str(extraction / "target"),
            "--workspace-remap",
            str(output / "workspace"),
            "-E",
            manifest["filter"],
            "--config-file",
            str(output / "workspace/.config/nextest.toml"),
        ]
        run, report = test_run.rust_report_command(invocation, run)
        code = test_run.execute_selected(
            invocation,
            run,
            environment,
            report,
            selected=[
                {"class": item["binary_id"], "name": item["name"]} for item in selected
            ],
            details={
                "historical_producer": manifest["producer_invocation"],
                "historical_source": manifest["source_commit"],
                "qualification": "historical executable only",
            },
            launch=lambda process: native_operation.run_child_scope(
                run, environment, cwd=ROOT, observation=process
            ),
        )
        terminal = json.loads(invocation.with_suffix(".terminal.json").read_text())
        verify_inputs(output / "workspace", fixture_manifest)
        preflight(manifest, directory, environment)
        result = {
            "schema": REPLAY_SCHEMA,
            "producer_invocation": manifest["producer_invocation"],
            "invocation": str(invocation),
            "source_commit": manifest["source_commit"],
            "archive": str(directory),
            "archive_receipt_sha256": digest(directory / "checks.json"),
            "output": str(output),
            "resource": resource,
            "terminal": str(output / "replay-terminal.json"),
            "process": terminal.get("process", {}),
            "workspace_remap": str(output / "workspace"),
            "fixture_manifest": fixture_manifest,
            "status": terminal["status"],
            "exit_code": code,
            "current_source_qualified": False,
        }
        validation.write_json(output / "replay.json", result)
        validation.write_json(output / "replay-terminal.json", terminal)
        interrupted = (
            terminal["status"] == "interrupted"
            or terminal.get("process", {}).get("forwarded_signal") is not None
        )
        receipt = {
            "version": 5,
            "complete": not interrupted,
            "baseline_failures": 0,
            "required_checks_covered": not interrupted,
            "input_coverage": True,
            "source_unchanged": True,
            "provenance_errors": [],
            "scope": [{"name": "historical-replay"}],
            "checks": [
                {**terminal, "gate": "historical-replay", "log": "replay-terminal.json"}
            ],
            "input_scope": "retained historical executable and declared fixtures; not current source",
        }
        scratch = [
            str(path.relative_to(output))
            for path in extraction.rglob("*")
            if path.is_file() and not path.is_symlink()
        ]
        # nextest can include symlinks in an extraction. They are not safe report
        # artifacts; keep such an unsupported extraction pinned rather than sealing.
        seal(
            output,
            resource,
            receipt,
            ["replay.json", *["workspace/" + name for name in fixture_manifest]],
            scratch,
        )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="operation", required=True)
    creation = subparsers.add_parser("archive")
    creation.add_argument("--output", type=Path, required=True)
    creation.add_argument("--package", required=True)
    target = creation.add_mutually_exclusive_group()
    target.add_argument("--test")
    target.add_argument("--lib", action="store_true")
    creation.add_argument("--filter", required=True)
    creation.add_argument("--fixture", action="append", default=[])
    creation.add_argument("--features", default="")
    for verb in ("inspect", "replay"):
        operation = subparsers.add_parser(verb)
        operation.add_argument("archive", type=Path)
        if verb == "replay":
            operation.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        result = (
            archive(
                ArchiveRequest(
                    output=str(args.output),
                    package=args.package,
                    filter=args.filter,
                    test=args.test,
                    lib=args.lib,
                    features=args.features,
                    fixture=args.fixture,
                )
            )
            if args.operation == "archive"
            else inspect(args.archive)
            if args.operation == "inspect"
            else replay(args.archive, args.output)
        )
        print(json.dumps(result, indent=2, sort_keys=True))
        return int(result.get("exit_code", 0))
    except (
        ArchiveError,
        test_resources.ResourceError,
        OSError,
        ValueError,
        subprocess.SubprocessError,
    ) as error:
        print(f"experiment: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
