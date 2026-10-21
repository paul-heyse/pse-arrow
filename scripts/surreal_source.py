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
import subprocess
import tarfile
import tempfile
import urllib.request
from pathlib import Path

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
)
DISKANN_SHA256 = "3ead115d8a4f3917c1ae9c3db0aa4dd56251841864ef8637329969f8e5145dd3"
FEATURES = "storage-mem,cli,mcp"
PROFILE = "release"
BUILD_CONFIG = ['profile.release.panic="unwind"']


class BuildError(RuntimeError):
    """A reproducible build or its previously recorded identity was refused."""


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def inputs() -> dict[str, object]:
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
        "patches": patches,
        "build_metadata": metadata,
        "cargo_config_sha256": digest(ROOT / ".cargo/config.toml"),
        "toolchain_sha256": digest(ROOT / "rust-toolchain.toml"),
        "features": FEATURES,
        "profile": PROFILE,
        "build_config": BUILD_CONFIG,
        "diskann_archive_sha256": DISKANN_SHA256,
        "diskann_patch_sha256": digest(INPUTS / "diskann-0.56.0.patch"),
        "platform": f"{platform.system()}-{platform.machine()}",
    }


def installation_name() -> str:
    identity = hashlib.sha256(json.dumps(inputs(), sort_keys=True).encode()).hexdigest()
    return f"v{VERSION}-mx{ENGINE}-{identity[:20]}"


def command(arguments: list[str], *, log_path: Path | None = None) -> str:
    # Cargo invokes the repository compiler-cache wrapper from the acquired
    # source cwd. The interactive Codex shell hook would unload its selected
    # environment there. Children keep the environment already owned by pse-env.
    environment = dict(os.environ)
    environment.pop("BASH_ENV", None)
    environment["SURREAL_BUILD_VERSION"] = VERSION
    environment["SURREAL_BUILD_METADATA"] = str(inputs()["build_metadata"])
    with (
        log_path.open("ab") if log_path is not None else contextlib.nullcontext()
    ) as log:
        result = subprocess.run(
            arguments,
            cwd=ROOT,
            env=environment,
            text=True,
            stdout=log if log is not None else subprocess.PIPE,
            stderr=subprocess.STDOUT if log is not None else subprocess.PIPE,
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
    selected = inputs()
    receipt_path = directory / "release.json"
    source = directory / "source"
    binary = source / "target" / PROFILE / "surreal"
    if receipt_path.exists():
        receipt = json.loads(receipt_path.read_text())
        if receipt.get("inputs") != selected:
            raise BuildError(
                "Installed server inputs changed; use its new input directory"
            )
        verify_source(source)
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
    command(
        [
            str(ROOT / "scripts/pse-env"),
            "--resource-class",
            "compile",
            "--",
            "cargo",
            "build",
            "--manifest-path",
            str(source / "Cargo.toml"),
            "--locked",
            "--no-default-features",
            "--features",
            FEATURES,
            "--profile",
            PROFILE,
            "--bin",
            "surreal",
            *[argument for value in BUILD_CONFIG for argument in ("--config", value)],
        ],
        log_path=directory / "build.log",
    )
    actual = command([str(binary), "version"]).strip()
    if VERSION not in actual or str(selected["build_metadata"]) not in actual:
        raise BuildError("Built executable version differs from its pinned source")
    receipt: dict[str, object] = {
        "version": VERSION,
        "surrealmx_version": ENGINE,
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
