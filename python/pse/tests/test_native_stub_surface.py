# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The hand-written stub matches the extension's surface (plan §5, O7)."""

import ast
from pathlib import Path

import pytest

from pse import _native

STUB = Path(_native.__file__).with_name("_native.pyi")

#: Names every module has; they are not part of the extension's surface.
MODULE_DUNDERS = frozenset(
    {
        "__all__",
        "__builtins__",
        "__dict__",
        "__doc__",
        "__file__",
        "__loader__",
        "__name__",
        "__package__",
        "__path__",
        "__spec__",
    }
)


def _declared_in_stub() -> set[str]:
    tree = ast.parse(STUB.read_text(encoding="utf-8"))
    declared: set[str] = set()
    for node in tree.body:
        if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef | ast.ClassDef):
            declared.add(node.name)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            declared.add(node.target.id)
        elif isinstance(node, ast.Assign):
            declared.update(
                target.id for target in node.targets if isinstance(target, ast.Name)
            )
    return declared


def _exported_by_extension() -> set[str]:
    return {
        name
        for name in dir(_native)
        if name not in MODULE_DUNDERS
        and (not name.startswith("_") or name.startswith("__"))
    }


@pytest.mark.unit
def test_stub_exists_beside_the_extension() -> None:
    assert STUB.is_file(), f"{STUB} is missing; the stub ships with the wheel"


@pytest.mark.unit
def test_stub_declares_exactly_the_extension_surface() -> None:
    declared = _declared_in_stub()
    exported = _exported_by_extension()
    assert declared == exported, (
        f"pse/_native.pyi declares {sorted(declared)} but pse._native exports "
        f"{sorted(exported)}; update the stub in the same commit as the crate"
    )


@pytest.mark.unit
def test_build_info_returns_a_string_mapping() -> None:
    info = _native.build_info()
    assert isinstance(info, dict)
    assert all(isinstance(key, str) for key in info)
    # The two embedded lockfiles cross as bytes; everything else is a string.
    byte_keys = {"cargo_lock_bytes", "uv_lock_bytes"}
    assert byte_keys <= info.keys()
    assert all(isinstance(info[key], bytes) for key in byte_keys)
    assert all(
        isinstance(value, str) for key, value in info.items() if key not in byte_keys
    )
