# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Build and authenticate the immutable, capability-free study policy module.

The released CLI owns WASI compilation, exports extraction and .surli packaging.
This owner adds locked source provenance and private immutable publication; database
installation remains part of the canonical schema activation transaction.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import tarfile
import tempfile
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import surreal_source

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/pse-study-module")
TARGET = "wasm32-wasip2"
KIND = "pse-study-module-v1"
QUALIFICATION_KIND = "pse-study-module-qualification-v1"
PRODUCTION_EXPORTS = {"study::readiness"}
QUALIFICATION_EXPORTS = PRODUCTION_EXPORTS | {"qualification::busy", "qualification::trap", "qualification::memory_refused"}
ABI = 2
MAX_ARCHIVE_BYTES = 32 * 1024 * 1024
MAX_MANIFEST_BYTES = 64 * 1024
COMPONENT_PREAMBLE = b"\x00asm\x0d\x00\x01\x00"
# The module package owns its capabilities once. Supervisor generations retain
# this same declaration alongside their scripts, rather than copying its values.
DECLARATION = tomllib.loads((ROOT / CRATE / "surrealism.toml").read_text())
CAPABILITIES = DECLARATION["capabilities"]
LIMITS: dict[str, int | bool] = {
    "max_memory_bytes": CAPABILITIES["max_memory_bytes"],
    "max_pool_size": CAPABILITIES["max_pool_size"],
    "max_execution_time_ms": CAPABILITIES["max_execution_time"],
    "max_kv_entries": CAPABILITIES["max_kv_entries"],
    "max_kv_value_bytes": CAPABILITIES["max_kv_value_bytes"],
    "strict_timeout": CAPABILITIES["strict_timeout"],
}
MEMBERS = {
    "surrealism/mod.wasm",
    "surrealism/surrealism.toml",
    "surrealism/exports.toml",
}


class ModuleError(RuntimeError):
    """A module build, artifact or retained activation input was refused."""


def _object(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise ModuleError("Module metadata requires a string-keyed object")
    return value


def _identity(value: object) -> str:
    if not isinstance(value, str) or len(value) != 64 or any(
        item not in "0123456789abcdef" for item in value
    ):
        raise ModuleError("Module provenance requires an exact SHA256 identity")
    return value


def _input_digest(inputs: Mapping[str, object]) -> str:
    return hashlib.sha256(
        json.dumps(inputs, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def _command(arguments: list[str]) -> str:
    environment = dict(os.environ)
    environment.pop("BASH_ENV", None)
    result = subprocess.run(
        arguments, cwd=ROOT, env=environment, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if result.returncode:
        raise ModuleError(
            f"Module build command failed ({result.returncode}): {arguments[0]}\n"
            + result.stderr[-12000:]
        )
    return result.stdout


def source_inputs(server: Mapping[str, object], *, qualification: bool = False) -> dict[str, object]:
    """Record the declared local dependency closure and exact resolved lockfile."""
    if server.get("surrealism_sdk_version") != surreal_source.SURREALISM_SDK:
        raise ModuleError("Module builder does not carry the selected Surrealism SDK")
    binary = Path(str(server.get("binary", "")))
    binary_digest = _identity(server.get("binary_sha256"))
    if not binary.is_file() or surreal_source.digest(binary) != binary_digest:
        raise ModuleError("Module builder binary differs from its admitted receipt")
    feature_arguments = ["--features", "pse-study-module/module-qualification"] if qualification else []
    metadata = _object(json.loads(_command([
        "cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", TARGET,
        "--manifest-path", str(ROOT / CRATE / "Cargo.toml"), *feature_arguments,
    ])))
    packages = metadata.get("packages")
    resolve = _object(metadata.get("resolve"))
    nodes = resolve.get("nodes")
    if not isinstance(packages, list) or not isinstance(nodes, list):
        raise ModuleError("Module dependency inventory is unavailable")
    packages_by_id: dict[str, dict[str, object]] = {}
    nodes_by_id: dict[str, dict[str, object]] = {}
    selected: str | None = None
    for value in packages:
        package = _object(value)
        identity = package.get("id")
        if not isinstance(identity, str):
            raise ModuleError("Module package identity is unavailable")
        packages_by_id[identity] = package
        if package.get("manifest_path") == str(ROOT / CRATE / "Cargo.toml"):
            selected = identity
    for value in nodes:
        node = _object(value)
        identity = node.get("id")
        if not isinstance(identity, str):
            raise ModuleError("Module resolved node identity is unavailable")
        nodes_by_id[identity] = node
    if selected is None:
        raise ModuleError("Selected study module is absent from its workspace")
    pending = [selected]
    visited: set[str] = set()
    sources: dict[str, str] = {}
    while pending:
        identity = pending.pop()
        if identity in visited:
            continue
        visited.add(identity)
        package = packages_by_id[identity]
        dependencies = nodes_by_id[identity].get("deps")
        if not isinstance(dependencies, list):
            raise ModuleError("Module resolved dependency edges are unavailable")
        for value in dependencies:
            dependency = _object(value)
            kinds = dependency.get("dep_kinds")
            target = dependency.get("pkg")
            if not isinstance(kinds, list) or not isinstance(target, str):
                raise ModuleError("Module dependency kind or identity is unavailable")
            if any(_object(kind).get("kind") != "dev" for kind in kinds):
                pending.append(target)
        if package.get("source") is not None:
            continue
        manifest = package.get("manifest_path")
        if not isinstance(manifest, str):
            raise ModuleError("Module local package manifest is unavailable")
        directory = Path(manifest).parent
        if not directory.is_relative_to(ROOT):
            raise ModuleError("Module local dependency escaped its declared workspace")
        for path in sorted(directory.rglob("*")):
            if any(part in {"target", ".git", "__pycache__"} for part in path.relative_to(directory).parts):
                continue
            if path.is_symlink():
                raise ModuleError("Module source inputs must not traverse symlinks")
            if path.is_file():
                sources[str(path.relative_to(ROOT))] = surreal_source.digest(path)
    for relative in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml"):
        sources[relative] = surreal_source.digest(ROOT / relative)
    return {
        "kind": QUALIFICATION_KIND if qualification else KIND,
        "features": ["module-qualification"] if qualification else [],
        "sdk_version": surreal_source.SURREALISM_SDK,
        "target": TARGET,
        "profile": "release",
        "builder_sha256": binary_digest,
        "sources": sources,
    }


def _configuration(value: object) -> None:
    config = _object(value)
    if config.get("abi") != ABI or config.get("target") != "rust":
        raise ModuleError("Module component ABI or language differs")
    if config.get("package") != DECLARATION["package"]:
        raise ModuleError("Module package identity differs")
    capabilities = _object(config.get("capabilities"))
    if (
        capabilities != CAPABILITIES
        or any(capabilities.get(name) is not False for name in ("allow_scripting", "allow_arbitrary_queries"))
        or capabilities.get("allow_functions") != [] or capabilities.get("allow_net") != []
        or capabilities.get("max_kv_entries") != 0 or capabilities.get("max_kv_value_bytes") != 0
        or capabilities.get("strict_timeout") is not True
        or config.get("attach", {}) != {}
    ):
        raise ModuleError("Module host capabilities, attachment or resource limits differ")
    if set(config) - {"abi", "target", "package", "capabilities", "attach"}:
        raise ModuleError("Module config carries unadmitted fields")


def inspect_archive(path: Path, *, qualification: bool = False) -> None:
    """Validate bounded package contents without extracting or granting host access."""
    if not 0 < path.stat().st_size <= MAX_ARCHIVE_BYTES:
        raise ModuleError("Module archive exceeds its admitted extent")
    seen: set[str] = set()
    try:
        with tarfile.open(path, mode="r|zst") as archive:
            for member in archive:
                if member.name not in MEMBERS or member.name in seen or not member.isfile():
                    raise ModuleError("Module archive has an extra, duplicate or linked member")
                seen.add(member.name)
                maximum = MAX_ARCHIVE_BYTES if member.name.endswith(".wasm") else MAX_MANIFEST_BYTES
                if not 0 < member.size <= maximum:
                    raise ModuleError("Module archive member exceeds its admitted extent")
                stream = archive.extractfile(member)
                if stream is None:
                    raise ModuleError("Module archive member is unavailable")
                if member.name.endswith(".wasm"):
                    if stream.read(8) != COMPONENT_PREAMBLE:
                        raise ModuleError("Module is not a WASI Preview 2 component")
                else:
                    value = tomllib.loads(stream.read(maximum + 1).decode())
                    if member.name.endswith("surrealism.toml"):
                        _configuration(value)
                    else:
                        functions = value.get("functions")
                        expected = QUALIFICATION_EXPORTS if qualification else PRODUCTION_EXPORTS
                        if not isinstance(functions, list) or len(functions) != len(expected):
                            raise ModuleError("Module exports differ from its selected production or qualification contract")
                        names: set[str] = set()
                        for item in functions:
                            export = _object(item)
                            name = export.get("name")
                            if not isinstance(name, str) or name in names or export.get("writeable") is not False:
                                raise ModuleError("Module policy export identity or write permission differs")
                            names.add(name)
                        if names != expected:
                            raise ModuleError("Module policy exports differ from the selected feature contract")
    except (tarfile.TarError, UnicodeError, ValueError) as error:
        raise ModuleError("Module archive is unreadable") from error
    if seen != MEMBERS:
        raise ModuleError("Module archive is incomplete")


def verify(value: object, *, qualification: bool = False) -> dict[str, object]:
    """Authenticate the retained immutable artifact, not current mutable sources."""
    receipt = _object(value)
    inputs = _object(receipt.get("inputs"))
    kind = QUALIFICATION_KIND if qualification else KIND
    features = ["module-qualification"] if qualification else []
    artifact = receipt.get("artifact")
    if (
        receipt.get("kind") != kind or receipt.get("sdk_version") != surreal_source.SURREALISM_SDK
        or receipt.get("abi") != ABI or receipt.get("limits") != LIMITS
        or inputs.get("kind") != kind or inputs.get("features") != features or inputs.get("sdk_version") != surreal_source.SURREALISM_SDK
        or inputs.get("target") != TARGET or inputs.get("profile") != "release"
        or not isinstance(artifact, str)
    ):
        raise ModuleError("Module receipt or resource contract differs")
    _identity(inputs.get("builder_sha256"))
    source_files = _object(inputs.get("sources"))
    if not source_files:
        raise ModuleError("Module source provenance is incomplete")
    for digest in source_files.values():
        _identity(digest)
    path = Path(artifact)
    if not path.is_absolute() or os.pathsep in str(path.parent) or path.name != "study.surli" or path.parent.name != _input_digest(inputs):
        raise ModuleError("Module artifact does not belong to its immutable input directory")
    if any(parent.is_symlink() for parent in (path, *path.parents)):
        raise ModuleError("Module artifact must not traverse symlinks")
    status = path.stat()
    parent = path.parent.stat()
    if (
        not stat.S_ISREG(status.st_mode) or status.st_uid != os.getuid() or status.st_nlink != 1
        or stat.S_IMODE(status.st_mode) != 0o400 or parent.st_uid != os.getuid()
        or stat.S_IMODE(parent.st_mode) != 0o700 or status.st_size != receipt.get("bytes")
    ):
        raise ModuleError("Module artifact ownership, immutability or extent differs")
    if surreal_source.digest(path) != _identity(receipt.get("sha256")):
        raise ModuleError("Module artifact differs from its admitted SHA256")
    inspect_archive(path, qualification=qualification)
    return receipt


def install(server: Mapping[str, object], tool_root: Path, *, qualification: bool = False) -> dict[str, object]:
    """Build once with the pinned CLI, then publish a private immutable artifact."""
    inputs = source_inputs(server, qualification=qualification)
    modules = tool_root / "modules"
    modules.mkdir(mode=0o700, parents=True, exist_ok=True)
    directory = modules / _input_digest(inputs)
    if directory.exists():
        receipt = _object(json.loads((directory / "module.json").read_text()))
        if receipt.get("inputs") != inputs:
            raise ModuleError("Module cached inputs differ")
        return verify(receipt, qualification=qualification)
    with tempfile.TemporaryDirectory(prefix="pending-", dir=modules) as scratch:
        pending = Path(scratch)
        artifact = pending / "study.surli"
        lock_digest = surreal_source.digest(ROOT / "Cargo.lock")
        feature_arguments = ["--features", "module-qualification"] if qualification else []
        _command([
            str(server["binary"]), "db", "module", "build", *feature_arguments,
            "--out", str(artifact), str(ROOT / CRATE),
        ])
        if surreal_source.digest(ROOT / "Cargo.lock") != lock_digest or source_inputs(server, qualification=qualification) != inputs:
            raise ModuleError("Module sources or locked dependencies changed during its build")
        inspect_archive(artifact, qualification=qualification)
        artifact.chmod(0o400)
        receipt: dict[str, object] = {
            "kind": QUALIFICATION_KIND if qualification else KIND, "artifact": str(directory / "study.surli"),
            "sha256": surreal_source.digest(artifact), "bytes": artifact.stat().st_size,
            "sdk_version": surreal_source.SURREALISM_SDK, "abi": ABI,
            "inputs": inputs, "limits": dict(LIMITS),
        }
        record = pending / "module.json"
        with record.open("w") as output:
            json.dump(receipt, output, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        record.chmod(0o600)
        with artifact.open("rb") as output:
            os.fsync(output.fileno())
        # Rename a fully constructed generation; an interrupted build is never
        # an admitted artifact and cannot overwrite another completed generation.
        try:
            pending.rename(directory)
        except FileExistsError:
            existing = verify(_object(json.loads((directory / "module.json").read_text())), qualification=qualification)
            if existing != receipt:
                raise ModuleError("Concurrent module generation differs from the exact completed artifact")
            return existing
        descriptor = os.open(modules, os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    return verify(receipt, qualification=qualification)


def main() -> int:
    """Explicit artifact preparation; this command never starts or mutates a database."""
    import argparse  # noqa: PLC0415 -- CLI dependencies stay outside the retained verifier path.

    from scripts import surreal_server  # noqa: PLC0415 -- Avoid reciprocal supervisor import during startup.

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool-root", type=Path)
    parser.add_argument("--qualification", action="store_true")
    arguments = parser.parse_args()
    root = arguments.tool_root or surreal_server.default_tool_root()
    receipt = install(surreal_server.install(None, root), root, qualification=arguments.qualification)
    print(json.dumps({
        "kind": receipt["kind"], "artifact": receipt["artifact"],
        "sha256": receipt["sha256"], "bytes": receipt["bytes"],
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
