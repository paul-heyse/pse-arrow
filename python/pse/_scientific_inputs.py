# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read explicit immutable scientific inputs for repository reference journeys.

No preparation cache or mutable source fallback: the invocation supplies the capsule.
Ordinary user-authored package paths continue through their normal document loader.
"""

from __future__ import annotations

import os
from functools import lru_cache
from pathlib import Path

import msgspec

ROLE = "PSE_SCIENTIFIC_INPUTS"
_REFERENCE_ROOT = Path(__file__).resolve().parents[2] / "packages/reference"


class _Family(msgspec.Struct, frozen=True):
    packages: list[str]


class _Capture(msgspec.Struct, frozen=True):
    schema: str
    files: dict[str, list[int]]
    families: dict[str, _Family]


@lru_cache(maxsize=1)
def _capture(path: str) -> _Capture:
    value = msgspec.json.decode(Path(path).read_bytes(), type=_Capture)
    if value.schema != "scientific-inputs-v1":
        message = "unknown scientific input capsule"
        raise ValueError(message)
    return value


def _input() -> _Capture:
    path = os.environ.get(ROLE)
    if not path:
        message = (
            "reference inputs require explicit preparation: scripts/pse-env -- python "
            "scripts/scientific_inputs.py capture <output>; bind PSE_SCIENTIFIC_INPUTS"
        )
        raise ValueError(message)
    return _capture(path)


def _reference_path(path: Path) -> str | None:
    # Only this checkout's declared reference tree selects captured inputs. A user
    # directory with the same suffix remains ordinary authored input. Normalize
    # lexically so retained captures still work after source files are removed.
    selected = Path(os.path.abspath(path))  # noqa: PTH100 -- lexical normalization must retain symlink identity and work after the captured source is removed.
    try:
        relative = selected.relative_to(_REFERENCE_ROOT)
    except ValueError:
        return None
    return "packages/reference/" + relative.as_posix()


def reference_bytes(path: Path) -> bytes:
    """Manifest/policy edges use the same immutable bytes as package transport."""
    relative = _reference_path(path)
    if relative is None:
        return path.read_bytes()
    return bytes(_input().files[relative])


def package_documents(path: Path) -> dict[str, bytes] | None:
    """Resolve a reference root by logical path in the captured package closure."""
    relative = _reference_path(path)
    if relative is None:
        return None
    return _package(_input(), relative)


def _package(value: _Capture, relative: str) -> dict[str, bytes]:
    packages = {p for family in value.families.values() for p in family.packages}
    if relative not in packages:
        message = f"reference package absent from capture: {relative}"
        raise ValueError(message)
    prefix = relative + "/"
    documents = {
        name[len(prefix) :]: bytes(data)
        for name, data in value.files.items()
        if name.startswith(prefix)
    }
    if "package.toml" not in documents:
        message = f"captured package header absent: {relative}"
        raise ValueError(message)
    return documents


def family_packages(name: str) -> tuple[dict[str, bytes], ...]:
    value = _input()
    return tuple(_package(value, root) for root in value.families[name].packages)
