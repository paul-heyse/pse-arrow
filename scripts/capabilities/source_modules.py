# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Resolve checkout module spellings without executing Python imports."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def module_candidates(name: str) -> tuple[str, ...]:
    """Include package initialization and absence-sensitive module alternatives."""
    if re.fullmatch(r"(?:scripts|pse)(?:\.[A-Za-z_][A-Za-z0-9_]*)*", name) is None:
        return ()
    parts = name.split(".")
    prefix = "python/" if parts[0] == "pse" else ""
    return tuple(
        [
            prefix + "/".join(parts[:end]) + "/__init__.py"
            for end in range(1, len(parts) + 1)
        ]
        + [prefix + "/".join(parts) + ".py"]
    )


def module_sources(root: Path, name: str, *, launch: bool = False) -> tuple[str, ...]:
    """Return existing source owners, including a package's actual -m entry point."""
    candidates = module_candidates(name)
    if not candidates:
        return ()
    module = candidates[-1]
    package = candidates[-2]
    main = package.removesuffix("__init__.py") + "__main__.py"
    selected = package if (root / package).is_file() else module
    if not (root / selected).is_file() and not (launch and (root / main).is_file()):
        return ()
    sources = [path for path in candidates[:-2] if (root / path).is_file()]
    if (root / selected).is_file():
        sources.append(selected)
    if launch and (selected == package or not (root / selected).is_file()):
        if not (root / main).is_file():
            return ()
        sources.append(main)
    return tuple(sources)
