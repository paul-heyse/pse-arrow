# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Reproduce the sole admitted canonical server from exact upstream inputs.

This module never installs an official binary as a fallback. Its caller owns the
installation directory lock and translates BuildError into the supervisor error.
"""

from __future__ import annotations

import contextlib
import hashlib
import json
import os
import platform
import sys
import subprocess
import tarfile
import tempfile
import tomllib
import urllib.request
from pathlib import Path

from scripts import installation_context, pse_env

VERSION = "3.3.4"
REVISION = "1ac2b1dba7cd177621a4cf5d507caebeb2414395"
ENGINE = "1.0.0"
ORIGIN = "https://github.com/surrealdb/surrealdb.git"
ROOT = Path(__file__).resolve().parents[1]
INPUTS = ROOT / "scripts/surrealdb-source"
PATCHES = (
    "mx-1.0-minimal.patch",
    "mx-1.0-controls.patch",
    "grpc-connection-ownership.patch",
    "pinned-compiler.patch",
    "canonical-profile.patch",
    "grpc-session-admission.patch",
    "grpc-status-details.patch",
    "surrealism-locked-build.patch",
    "surrealism-build-features.patch",
    "surrealism-global-controllers.patch",
    "surrealism-memory-count.patch",
    "surrealism-controller-deadline.patch",
)
DISKANN_SHA256 = "3ead115d8a4f3917c1ae9c3db0aa4dd56251841864ef8637329969f8e5145dd3"
FEATURES = "storage-mem,cli,mcp,surrealism"
SURREALISM_SDK = "0.6.4"
PROFILE = "release"
BUILD_CONFIG = ['profile.release.panic="unwind"']


class BuildError(RuntimeError):
    """A reproducible build or its previously recorded identity was refused."""


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def source_inputs() -> dict[str, object]:
    patches = {name: digest(INPUTS / name) for name in PATCHES}
    metadata = (
        "pse.mx.1.0.0."
        + hashlib.sha256(json.dumps(patches, sort_keys=True).encode()).hexdigest()[:16]
    )
    return {
        "version": VERSION,
        "upstream_revision": REVISION,
        "upstream_origin": ORIGIN,
        "surrealmx_version": ENGINE,
        "surrealism_sdk_version": SURREALISM_SDK,
        "patches": patches,
        "build_metadata": metadata,
        "features": FEATURES,
        "profile": PROFILE,
        "build_config": BUILD_CONFIG,
        "diskann_archive_sha256": DISKANN_SHA256,
        "diskann_patch_sha256": digest(INPUTS / "diskann-0.56.0.patch"),
        "platform": f"{platform.system()}-{platform.machine()}",
    }


RECEIPT_SCHEMA = "surreal-source-release-v2"


def build_context() -> tuple[dict, dict, dict[str, str]]:
    try:
        environment = installation_context.controlled_environment(ROOT)
        context, provenance = installation_context.capture(ROOT, environment)
    except (installation_context.ContextError, pse_env.BoundaryError, OSError, subprocess.CalledProcessError) as error:
        raise BuildError(f"Cannot close server build context: {error}") from error
    return context, provenance, environment


def inputs(tool_root: Path | None = None) -> dict[str, object]:
    context, _, _ = build_context()
    return {**source_inputs(), "build_context": context,
            "installation_namespace": str((ROOT if tool_root is None else tool_root).resolve())}


def effective_key(selected: dict[str, object]) -> str:
    return installation_context.identity(selected)


def admitted_receipt(receipt: dict, selected: dict[str, object] | None = None, *, revalidate: bool = False) -> bool:
    """Old incomplete receipts are never silently promoted to safe equivalence."""
    if (receipt.get("schema") != RECEIPT_SCHEMA
        or receipt.get("surrealmx_version") != ENGINE
        or not receipt.get("raw_provenance")
        or not receipt.get("lockfile_sha256")
        or not isinstance(receipt.get("inputs"), dict)
        or not isinstance(receipt["inputs"].get("installation_namespace"), str)):
        return False
    recorded = receipt["inputs"]
    if selected is None and not revalidate:
        # A passive consumer authenticates the already-built generation. It does
        # not ask the compiler/native toolchain to requalify a running server.
        context = recorded.get("build_context")
        if not isinstance(context, dict) or context.get("schema") != installation_context.SCHEMA:
            return False
        if any(not context.get(name) for name in ("configuration", "toolchain_declaration", "tools", "toolchain", "native", "owners")):
            return False
        try:
            configs, _, _ = installation_context.configuration(ROOT, {"HOME": os.environ.get("HOME", str(Path.home())), **({"CARGO_HOME": os.environ["CARGO_HOME"]} if "CARGO_HOME" in os.environ else {})})
            declaration = installation_context.canonical(tomllib.loads((ROOT / "rust-toolchain.toml").read_text()))
        except (OSError, ValueError):
            return False
        return (receipt.get("effective_key") == effective_key(recorded)
                and {key: value for key, value in recorded.items() if key not in {"build_context", "installation_namespace"}} == source_inputs()
                and context["configuration"] == configs
                and context["toolchain_declaration"] == declaration)
    current = inputs(Path(recorded["installation_namespace"])) if selected is None else selected
    return receipt.get("effective_key") == effective_key(current) and recorded == current


def installation_name(tool_root: Path | None = None) -> str:
    return name_for(inputs(tool_root))


def name_for(selected: dict[str, object]) -> str:
    return f"v{VERSION}-mx{ENGINE}-effective-v1-{effective_key(selected)[:20]}"


def command(arguments: list[str], *, log_path: Path | None = None, environment: dict[str, str] | None = None, stream: bool = False) -> str:
    # Cargo invokes the repository compiler-cache wrapper from the acquired
    # source cwd. The interactive Codex shell hook would unload its selected
    # environment there. Children keep the environment already owned by pse-env.
    environment = dict(os.environ) if environment is None else environment.copy()
    environment.pop("BASH_ENV", None)
    environment["SURREAL_BUILD_VERSION"] = VERSION
    environment["SURREAL_BUILD_METADATA"] = str(source_inputs()["build_metadata"])
    with (
        log_path.open("ab") if log_path is not None else contextlib.nullcontext()
    ) as log:
        result = subprocess.run(
            arguments,
            cwd=ROOT,
            env=environment,
            text=True,
            stdout=log if log is not None else None if stream else subprocess.PIPE,
            stderr=subprocess.STDOUT if log is not None else None if stream else subprocess.PIPE,
            check=False,
        )
    if result.returncode:
        if log_path is not None:
            with log_path.open("rb") as output:
                output.seek(max(0, log_path.stat().st_size - 12000))
                detail = output.read(12000).decode(errors="replace")
        else:
            detail = (result.stderr or "")[-12000:]
        raise BuildError(
            f"Source build command failed ({result.returncode}): {arguments[0]}\n{detail}"
        )
    return result.stdout or ""


def verify_source(source: Path) -> None:
    if command(["git", "-C", str(source), "rev-parse", "HEAD"]).strip() != REVISION:
        raise BuildError("Acquired server source differs from its pinned revision")
    # Index owns the applied patches. Any tracked worktree change, or unexpected
    # indexed file, means the acquired source was changed outside this installer.
    if command(["git", "-C", str(source), "diff", "--name-only"]).strip():
        raise BuildError("Acquired server source has unowned modifications")
    expected: dict[str, str] = {}
    for name in PATCHES:
        for line in (INPUTS / name).read_text().splitlines():
            if line.startswith("+++ b/"):
                relative = line.removeprefix("+++ b/")
                expected[relative] = digest(source / relative)
    recorded = json.loads((source.parent / "source-files.json").read_text())
    indexed = command(
        ["git", "-C", str(source), "diff", "--cached", "--name-only"]
    ).splitlines()
    if set(indexed) != set(expected):
        raise BuildError("Acquired server index differs from its patch inputs")
    if expected != recorded:
        raise BuildError("Patched source content differs from its build inputs")
    dependencies = source / ".pse-deps/diskann"
    recorded_dependencies = json.loads(
        (source.parent / "dependency-files.json").read_text()
    )
    if tree_digest(dependencies) != recorded_dependencies:
        raise BuildError(
            "Acquired dependency differs from its checked archive and patch"
        )
    manifest = (source / "Cargo.toml").read_text()
    lock = (source / "Cargo.lock").read_text()
    if 'surrealmx = "=1.0.0"' not in manifest:
        raise BuildError("Source manifest does not select the required MX engine")
    if 'name = "surrealmx"\nversion = "1.0.0"' not in lock:
        raise BuildError("Source lockfile does not resolve the required MX engine")
    if f'name = "surrealism-runtime"\nversion = "{SURREALISM_SDK}"' not in lock:
        raise BuildError(
            "Source lockfile does not resolve the required Surrealism runtime"
        )


def tree_digest(directory: Path) -> dict[str, str]:
    if directory.is_symlink() or any(
        path.is_symlink() for path in directory.rglob("*")
    ):
        raise BuildError("Acquired dependency must not contain symlinks")
    return {
        str(path.relative_to(directory)): digest(path)
        for path in sorted(directory.rglob("*"))
        if path.is_file()
    }


def acquire_diskann(source: Path, scratch: Path) -> None:
    """Apply the one-line generic borrow fix to the checksum-pinned crate."""
    archive = scratch / "diskann-0.56.0.crate"
    url = "https://static.crates.io/crates/diskann/diskann-0.56.0.crate"
    with (
        urllib.request.urlopen(url, timeout=60) as response,
        archive.open("wb") as output,
    ):
        while chunk := response.read(1024 * 1024):
            output.write(chunk)
    if digest(archive) != DISKANN_SHA256:
        raise BuildError("DiskANN source archive failed SHA256 verification")
    dependencies = source / ".pse-deps"
    dependencies.mkdir(mode=0o700)
    with tarfile.open(archive) as bundle:
        for member in bundle.getmembers():
            if not member.name.startswith("diskann-0.56.0/") or not (
                member.isfile() or member.isdir()
            ):
                raise BuildError("DiskANN archive contains an unexpected source entry")
        bundle.extractall(dependencies, filter="data")
    target = dependencies / "diskann"
    (dependencies / "diskann-0.56.0").rename(target)
    command(
        [
            "git",
            "-C",
            str(source),
            "apply",
            "--directory=.pse-deps/diskann",
            str(INPUTS / "diskann-0.56.0.patch"),
        ]
    )


def install(directory: Path) -> dict[str, object]:
    context, provenance, environment = build_context()
    selected = {**source_inputs(), "build_context": context,
                "installation_namespace": str(directory.parent.resolve())}
    if directory.name != name_for(selected):
        raise BuildError("Installer directory differs from its effective namespace; select installation_name(tool_root)")
    receipt_path = directory / "release.json"
    source = directory / "source"
    binary = source / "target" / PROFILE / "surreal"
    if receipt_path.exists():
        receipt = json.loads(receipt_path.read_text())
        if not admitted_receipt(receipt, selected):
            raise BuildError(
                "Installed server inputs changed or receipt is incomplete; use its new input directory"
            )
        verify_source(source)
        if receipt.get("lockfile_sha256") != digest(source / "Cargo.lock"):
            raise BuildError("Installed server lockfile differs from its admitted receipt")
        if not binary.is_file() or receipt.get("binary_sha256") != digest(binary):
            raise BuildError(
                "Installed server binary differs from its admitted receipt"
            )
        return receipt
    if not source.exists():
        with tempfile.TemporaryDirectory(prefix="acquire-", dir=directory) as scratch:
            pending = Path(scratch) / "source"
            command(["git", "init", str(pending)])
            command(
                ["git", "-C", str(pending), "fetch", "--depth", "1", ORIGIN, REVISION]
            )
            command(["git", "-C", str(pending), "checkout", "--detach", "FETCH_HEAD"])
            for name in PATCHES:
                command(
                    ["git", "-C", str(pending), "apply", "--index", str(INPUTS / name)]
                )
            acquire_diskann(pending, Path(scratch))
            paths = command(
                ["git", "-C", str(pending), "diff", "--cached", "--name-only"]
            )
            (directory / "source-files.json").write_text(
                json.dumps(
                    {name: digest(pending / name) for name in paths.splitlines()},
                    sort_keys=True,
                )
                + "\n"
            )
            (directory / "dependency-files.json").write_text(
                json.dumps(tree_digest(pending / ".pse-deps/diskann"), sort_keys=True)
                + "\n"
            )
            pending.rename(source)
    verify_source(source)
    # The supervised child closes its environment *after* pse-env composition,
    # before Cargo/build scripts execute. This prevents local/caller injection.
    request = directory / "build-request.json"
    request.write_text(json.dumps({"inputs": selected, "effective_key": effective_key(selected)}, sort_keys=True) + "\n")
    command(
        [str(ROOT / "scripts/pse-env"), "--recipe-default-class", "compile", "--",
         str(ROOT / ".venv/bin/python"), "-m", "scripts.surreal_source", "--build", str(directory)],
        log_path=directory / "build.log",
        environment=environment,
    )
    after_context, _, _ = build_context()
    if {**source_inputs(), "build_context": after_context, "installation_namespace": str(directory.parent.resolve())} != selected:
        raise BuildError("Server build inputs changed during compilation; receipt refused")
    actual = command([str(binary), "version"]).strip()
    if VERSION not in actual or str(selected["build_metadata"]) not in actual:
        raise BuildError("Built executable version differs from its pinned source")
    command([str(binary), "module", "build", "--help"])
    receipt: dict[str, object] = {
        "schema": RECEIPT_SCHEMA,
        "effective_key": effective_key(selected),
        "raw_provenance": provenance,
        "version": VERSION,
        "surrealmx_version": ENGINE,
        "surrealism_sdk_version": SURREALISM_SDK,
        "upstream_revision": REVISION,
        "binary": str(binary),
        "binary_sha256": digest(binary),
        "lockfile_sha256": digest(source / "Cargo.lock"),
        "inputs": selected,
    }
    temporary = directory / "release.json.pending"
    with temporary.open("w") as output:
        json.dump(receipt, output, sort_keys=True)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    temporary.replace(receipt_path)
    return receipt


def build_child(directory: Path) -> None:
    request = json.loads((directory / "build-request.json").read_text())
    context, _, environment = build_context()
    selected = {**source_inputs(), "build_context": context, "installation_namespace": str(directory.parent.resolve())}
    if request != {"inputs": selected, "effective_key": effective_key(selected)}:
        raise BuildError("Installer child context differs from selected build inputs")
    source = directory / "source"
    verify_source(source)
    command(["cargo", "build", "--manifest-path", str(source / "Cargo.toml"),
             "--locked", "--no-default-features", "--features", FEATURES,
             "--profile", PROFILE, "--bin", "surreal",
             *[argument for value in BUILD_CONFIG for argument in ("--config", value)]],
            environment=environment, stream=True)


if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[1] != "--build":
        raise SystemExit("internal installer usage: --build DIRECTORY")
    build_child(Path(sys.argv[2]).resolve())
