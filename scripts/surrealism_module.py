# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Historical receipt readers for retained immutable study policy artifacts.

Construction belongs to pse-host study-module prepare. Legacy supervisor callers
fail before module construction effects until their native consumer handoff lands.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat
import tarfile
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING, NoReturn

from scripts import surreal_source

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/pse-study-module")
TARGET = "wasm32-wasip2"
KIND = "pse-study-module-v1"
QUALIFICATION_KIND = "pse-study-module-qualification-v1"
LONG_HOLD_KIND = "pse-study-module-qualification-long-hold-v1"
LONG_HOLD_MANIFEST = CRATE / "surrealism-qualification-long-hold.toml"
PRODUCTION_EXPORTS = {"study::readiness"}
QUALIFICATION_EXPORTS = PRODUCTION_EXPORTS | {
    "qualification::busy",
    "qualification::trap",
    "qualification::memory_refused",
    "qualification::hold",
}
ABI = 2
MAX_ARCHIVE_BYTES = 32 * 1024 * 1024
MAX_MANIFEST_BYTES = 64 * 1024
COMPONENT_PREAMBLE = b"\x00asm\x0d\x00\x01\x00"
# The module package owns its capabilities once. Supervisor generations retain
# this same declaration alongside their scripts, rather than copying its values.
DECLARATION = tomllib.loads((ROOT / CRATE / "surrealism.toml").read_text())
CAPABILITIES = DECLARATION["capabilities"]
LONG_HOLD_DECLARATION = tomllib.loads((ROOT / LONG_HOLD_MANIFEST).read_text())
LONG_HOLD_CAPABILITIES = LONG_HOLD_DECLARATION["capabilities"]
if LONG_HOLD_DECLARATION != {
    **DECLARATION,
    "capabilities": {**CAPABILITIES, "max_execution_time": 6000},
}:
    raise RuntimeError(
        "Long-hold qualification may change only its declared execution time"
    )
LIMITS: dict[str, int | bool] = {
    "max_memory_bytes": CAPABILITIES["max_memory_bytes"],
    "max_pool_size": CAPABILITIES["max_pool_size"],
    "max_execution_time_ms": CAPABILITIES["max_execution_time"],
    "max_kv_entries": CAPABILITIES["max_kv_entries"],
    "max_kv_value_bytes": CAPABILITIES["max_kv_value_bytes"],
    "strict_timeout": CAPABILITIES["strict_timeout"],
}
LONG_HOLD_LIMITS = {
    **LIMITS,
    "max_execution_time_ms": LONG_HOLD_CAPABILITIES["max_execution_time"],
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
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(item not in "0123456789abcdef" for item in value)
    ):
        raise ModuleError("Module provenance requires an exact SHA256 identity")
    return value


def _input_digest(inputs: Mapping[str, object]) -> str:
    return hashlib.sha256(
        json.dumps(inputs, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def _variant(qualification: bool, long_hold: bool) -> tuple[str, list[str]]:
    if long_hold and not qualification:
        raise ModuleError("Long-hold is available only for explicit qualification")
    if long_hold:
        return LONG_HOLD_KIND, ["module-qualification-long-hold"]
    return (
        (QUALIFICATION_KIND, ["module-qualification"]) if qualification else (KIND, [])
    )


def _configuration(value: object, *, long_hold: bool = False) -> None:
    config = _object(value)
    if config.get("abi") != ABI or config.get("target") != "rust":
        raise ModuleError("Module component ABI or language differs")
    if config.get("package") != DECLARATION["package"]:
        raise ModuleError("Module package identity differs")
    capabilities = _object(config.get("capabilities"))
    expected = LONG_HOLD_CAPABILITIES if long_hold else CAPABILITIES
    if (
        capabilities != expected
        or any(
            type(capabilities.get(name)) is not type(value)
            for name, value in expected.items()
        )
        or any(
            capabilities.get(name) is not False
            for name in ("allow_scripting", "allow_arbitrary_queries")
        )
        or capabilities.get("allow_functions") != []
        or capabilities.get("allow_net") != []
        or capabilities.get("max_kv_entries") != 0
        or capabilities.get("max_kv_value_bytes") != 0
        or capabilities.get("strict_timeout") is not True
        or config.get("attach", {}) != {}
    ):
        raise ModuleError(
            "Module host capabilities, attachment or resource limits differ"
        )
    if set(config) - {"abi", "target", "package", "capabilities", "attach"}:
        raise ModuleError("Module config carries unadmitted fields")


def inspect_archive(
    path: Path, *, qualification: bool = False, long_hold: bool = False
) -> None:
    """Validate bounded package contents without extracting or granting host access."""
    _variant(qualification, long_hold)
    if not 0 < path.stat().st_size <= MAX_ARCHIVE_BYTES:
        raise ModuleError("Module archive exceeds its admitted extent")
    seen: set[str] = set()
    try:
        with tarfile.open(path, mode="r|zst") as archive:
            for member in archive:
                if (
                    member.name not in MEMBERS
                    or member.name in seen
                    or not member.isfile()
                ):
                    raise ModuleError(
                        "Module archive has an extra, duplicate or linked member"
                    )
                seen.add(member.name)
                maximum = (
                    MAX_ARCHIVE_BYTES
                    if member.name.endswith(".wasm")
                    else MAX_MANIFEST_BYTES
                )
                if not 0 < member.size <= maximum:
                    raise ModuleError(
                        "Module archive member exceeds its admitted extent"
                    )
                stream = archive.extractfile(member)
                if stream is None:
                    raise ModuleError("Module archive member is unavailable")
                if member.name.endswith(".wasm"):
                    if stream.read(8) != COMPONENT_PREAMBLE:
                        raise ModuleError("Module is not a WASI Preview 2 component")
                else:
                    value = tomllib.loads(stream.read(maximum + 1).decode())
                    if member.name.endswith("surrealism.toml"):
                        _configuration(value, long_hold=long_hold)
                    else:
                        functions = value.get("functions")
                        expected = (
                            QUALIFICATION_EXPORTS
                            if qualification
                            else PRODUCTION_EXPORTS
                        )
                        if not isinstance(functions, list) or len(functions) != len(
                            expected
                        ):
                            raise ModuleError(
                                "Module exports differ from its selected production or qualification contract"
                            )
                        names: set[str] = set()
                        for item in functions:
                            export = _object(item)
                            name = export.get("name")
                            if (
                                not isinstance(name, str)
                                or name in names
                                or export.get("writeable") is not False
                            ):
                                raise ModuleError(
                                    "Module policy export identity or write permission differs"
                                )
                            names.add(name)
                        if names != expected:
                            raise ModuleError(
                                "Module policy exports differ from the selected feature contract"
                            )
    except (tarfile.TarError, UnicodeError, ValueError) as error:
        raise ModuleError("Module archive is unreadable") from error
    if seen != MEMBERS:
        raise ModuleError("Module archive is incomplete")


def verify(
    value: object, *, qualification: bool = False, long_hold: bool = False
) -> dict[str, object]:
    """Authenticate the retained immutable artifact, not current mutable sources."""
    receipt = _object(value)
    inputs = _object(receipt.get("inputs"))
    limits = _object(receipt.get("limits"))
    kind, features = _variant(qualification, long_hold)
    expected_limits = LONG_HOLD_LIMITS if long_hold else LIMITS
    artifact = receipt.get("artifact")
    if (
        receipt.get("kind") != kind
        or receipt.get("sdk_version") != surreal_source.SURREALISM_SDK
        or receipt.get("abi") != ABI
        or limits != expected_limits
        or any(
            type(limits.get(name)) is not type(value)
            for name, value in expected_limits.items()
        )
        or inputs.get("kind") != kind
        or inputs.get("features") != features
        or inputs.get("sdk_version") != surreal_source.SURREALISM_SDK
        or inputs.get("target") != TARGET
        or inputs.get("profile") != "release"
        or not isinstance(artifact, str)
    ):
        raise ModuleError("Module receipt or resource contract differs")
    _identity(inputs.get("builder_sha256"))
    source_files = _object(inputs.get("sources"))
    if not source_files:
        raise ModuleError("Module source provenance is incomplete")
    for digest in source_files.values():
        _identity(digest)
    if long_hold:
        packaging = _object(inputs.get("packaging"))
        if (
            set(packaging) != {"variant", "packer_sha256", "manifest_sha256"}
            or packaging.get("variant") != "qualification-long-hold-v1"
            or packaging.get("manifest_sha256")
            != source_files.get(str(LONG_HOLD_MANIFEST))
        ):
            raise ModuleError("Long-hold module packaging provenance differs")
        _identity(packaging.get("packer_sha256"))
        _identity(packaging.get("manifest_sha256"))
    path = Path(artifact)
    if (
        not path.is_absolute()
        or os.pathsep in str(path.parent)
        or path.name != "study.surli"
        or path.parent.name != _input_digest(inputs)
    ):
        raise ModuleError(
            "Module artifact does not belong to its immutable input directory"
        )
    if any(parent.is_symlink() for parent in (path, *path.parents)):
        raise ModuleError("Module artifact must not traverse symlinks")
    status = path.stat()
    parent = path.parent.stat()
    if (
        not stat.S_ISREG(status.st_mode)
        or status.st_uid != os.getuid()
        or status.st_nlink != 1
        or stat.S_IMODE(status.st_mode) != 0o400
        or parent.st_uid != os.getuid()
        or stat.S_IMODE(parent.st_mode) != 0o700
        or status.st_size != receipt.get("bytes")
    ):
        raise ModuleError("Module artifact ownership, immutability or extent differs")
    if surreal_source.digest(path) != _identity(receipt.get("sha256")):
        raise ModuleError("Module artifact differs from its admitted SHA256")
    inspect_archive(path, qualification=qualification, long_hold=long_hold)
    return receipt


def refuse_legacy_construction() -> NoReturn:
    """Refuse the retired constructor before commands, hashing or file changes.

    Existing surreal_server callers must hand off an explicitly prepared native
    receipt; they do not authorize a second constructor or an implicit native run.
    """
    raise ModuleError(
        "Python study-module construction is retired; explicitly run "
        "scripts/pse-env -- scripts/pse-host study-module prepare and hand its "
        "module.json receipt to the native fixture/operations owner. Legacy "
        "surreal_server construction callers still require consumer cutover."
    )


def main() -> NoReturn:
    """Refuse the retired CLI without importing or invoking a supervisor."""
    refuse_legacy_construction()


if __name__ == "__main__":
    raise SystemExit(main())
