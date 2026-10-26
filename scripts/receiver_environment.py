# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Select prepared inputs at publication; restore them without running preparation."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import TYPE_CHECKING

import msgspec

from scripts import native_cache as cache
from scripts import native_operation as operation

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from scripts.generated import deployment

# Existing capability path declarations remain the owner of those variables.
# These are additional nonsecret dynamic compiler/loader selections, never budgets.
_DYNAMIC_VALUES = ("CC", "CXX", "FC", "CFLAGS", "CXXFLAGS", "FFLAGS", "CPPFLAGS", "LDFLAGS",
                   "CPATH", "CPLUS_INCLUDE_PATH", "LIBRARY_PATH", "LD_LIBRARY_PATH",
                   "CLANG_PATH", "LIBCLANG_PATH", "LLVM_CONFIG_PATH", "PSE_LLVM_PREFIX",
                   "BINDGEN_EXTRA_CLANG_ARGS", "PSE_NATIVE_CACHE")


class ReceiverEnvironmentError(ValueError):
    """Prepared inputs are unavailable or differ from the admitted generation."""


def variable_names(capabilities: Sequence[str]) -> tuple[str, ...]:
    if any(capability not in operation.CAPABILITIES for capability in capabilities):
        raise ReceiverEnvironmentError("Unknown prepared receiver capability")
    return tuple(dict.fromkeys((*_DYNAMIC_VALUES, *(name for capability in capabilities
                                for name in operation.CAPABILITY_PATHS[capability]))))


def _prefixes(capabilities: Sequence[str], values: Mapping[str, str]) -> dict[Path, tuple[str, ...]]:
    result: dict[Path, tuple[str, ...]] = {}
    def include(prefix: Path, required: tuple[str, ...]) -> None:
        result[prefix] = tuple(dict.fromkeys((*result.get(prefix, ()), *required)))

    def selected(name: str) -> Path:
        value = values.get(name)
        if not value or value == "off" or not Path(value).is_absolute():
            raise ReceiverEnvironmentError(f"Prepare {name} before receiver publication/readmission")
        return Path(value).resolve(strict=True)
    if any(kind in capabilities for kind in ("solver", "uno", "petsc")):
        prefix = selected("IPOPT_DIR")
        if selected("SCIPOPTDIR") != prefix:
            raise ReceiverEnvironmentError("Prepared solver paths select different prefixes")
        include(prefix, cache.SOLVER_FILES)
    if "klu" in capabilities:
        include_directory, library_directory = selected("SUITESPARSE_INCLUDE_DIR"), selected("SUITESPARSE_LIBRARY_DIR")
        prefix = include_directory.parent.parent
        if include_directory != prefix / "include/suitesparse" or library_directory != prefix / "lib":
            raise ReceiverEnvironmentError("Prepared KLU paths select different prefixes")
        include(prefix, cache.KLU_FILES)
    if "isolation" in capabilities:
        include(selected("PSE_ROOT_ISOLATION_DIR"), cache.ISOLATION_FILES)
    for kind, name in (("uno", "UNO_DIR"), ("petsc", "PETSC_DIR")):
        if kind in capabilities:
            include(selected(name), cache.pipeline_files(kind))
    return result


def _receipts(capabilities: Sequence[str], values: Mapping[str, str]) -> dict[str, str]:
    receipts: dict[str, str] = {}
    for prefix, required in _prefixes(capabilities, values).items():
        receipt = prefix / ".complete.json"
        if receipt.is_symlink() or not receipt.is_file():
            raise ReceiverEnvironmentError("Immutable receiver requires an existing managed native generation")
        before = cache.digest(receipt)
        # This existing owner verifies the whole original managed generation and
        # pins it in the actual operation. Receipt SHA alone grants no admission.
        if not cache.admit_external(prefix, required):
            raise ReceiverEnvironmentError("Immutable receiver requires managed native preparation")
        if cache.digest(receipt) != before:
            raise ReceiverEnvironmentError("Prepared native receipt changed during verification")
        receipts[str(prefix)] = before
    return receipts


def capture(capabilities: Sequence[str], environment: Mapping[str, str]) -> bytes:
    from scripts.generated import deployment  # noqa: PLC0415 -- compiled private document owner

    values = {name: environment[name] for name in variable_names(capabilities) if name in environment}
    values["PSE_NATIVE_CACHE"] = str(cache.cache_root(dict(environment)).resolve())
    # Compiler discovery belongs before publication. Retain its selected path and
    # arguments; restore never queries clang or invokes any provider builder.
    if any(kind in capabilities for kind in ("compiler", "klu", "isolation", "uno", "petsc")) and not values.get("CLANG_PATH"):
        raise ReceiverEnvironmentError("Prepare the selected compiler before receiver publication/readmission")
    result = deployment.PreparedReceiverEnvironment(capabilities=tuple(capabilities), values=values,
                                                     provider_receipts=_receipts(capabilities, values))
    return msgspec.json.encode(result)


def read(path: Path) -> deployment.PreparedReceiverEnvironment:
    from scripts.generated import deployment  # noqa: PLC0415 -- compiled private document owner

    if path.is_symlink() or any(parent.is_symlink() for parent in path.parents):
        raise ReceiverEnvironmentError("Prepared receiver inputs must not traverse symlinks")
    with path.open("rb") as source:
        raw = source.read(1024 * 1024 + 1)
    if len(raw) > 1024 * 1024:
        raise ReceiverEnvironmentError("Prepared receiver inputs exceed their bounded envelope")
    if not isinstance(value := json.loads(raw), dict) or "version" not in value:
        raise ReceiverEnvironmentError("Prepared receiver inputs omitted their persisted version")
    return msgspec.json.decode(raw, type=deployment.PreparedReceiverEnvironment)


def selection(selected: deployment.PreparedReceiverEnvironment, capabilities: Sequence[str],
              environment: Mapping[str, str]) -> dict[str, str]:
    """Apply the exact published selection, including names absent at publication."""
    names = variable_names(capabilities)
    if tuple(capabilities) != selected.capabilities or set(selected.values) - set(names):
        raise ReceiverEnvironmentError("Receiver inputs differ from their selected capability owner")
    result = dict(environment)
    for name in names:
        result.pop(name, None)
    # Snapshot values exclude licenses, credentials and allocation/thread budgets.
    result.update(selected.values)
    return result


def restore(path: Path, capabilities: Sequence[str], environment: Mapping[str, str]) -> dict[str, str]:
    selected = read(path)
    result = selection(selected, capabilities, operation.runtime_selection(list(capabilities), environment))
    if _receipts(capabilities, result) != selected.provider_receipts:
        raise ReceiverEnvironmentError("Prepared native generation differs; explicitly readmit the receiver")
    return result


def compiler_files(values: Mapping[str, str]) -> dict[str, str]:
    """Bind selected dynamic compiler executables/libraries to the v2 runtime inputs."""
    paths = [Path(value) for name in ("CLANG_PATH", "LLVM_CONFIG_PATH") if (value := values.get(name))]
    if selected := values.get("LIBCLANG_PATH"):
        path = Path(selected)
        paths.append(path / "libclang.so" if path.is_dir() else path)
    result: dict[str, str] = {}
    for path in paths:
        if not path.is_absolute() or not path.is_file():
            raise ReceiverEnvironmentError("Prepared dynamic compiler input is unavailable")
        with path.open("rb") as source:
            result[str(path)] = hashlib.file_digest(source, "sha256").hexdigest()
    return result
