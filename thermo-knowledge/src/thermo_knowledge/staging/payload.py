# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Which files of an acquired tree belong to a source's payload.

The payload is what the source manifest's `[payload] include` globs match, less what `exclude`
matches. Globs use POSIX paths relative to the tree: `*` and `?` stay within one path segment,
`**/` spans any number of directories (including none), and a trailing `**` matches everything
below. Hidden files match like any other.
"""

from __future__ import annotations

import os
import re
from collections.abc import Iterable
from pathlib import Path


def glob_regex(pattern: str) -> re.Pattern[str]:
    """The regular expression a payload glob denotes."""
    parts: list[str] = []
    index = 0
    while index < len(pattern):
        char = pattern[index]
        if pattern.startswith("**/", index):
            parts.append("(?:.*/)?")
            index += 3
        elif pattern.startswith("**", index):
            parts.append(".*")
            index += 2
        elif char == "*":
            parts.append("[^/]*")
            index += 1
        elif char == "?":
            parts.append("[^/]")
            index += 1
        else:
            parts.append(re.escape(char))
            index += 1
    return re.compile("".join(parts) + r"\Z", re.DOTALL)


def matches(pattern: str, path: str) -> bool:
    """Whether the payload glob `pattern` matches the relative POSIX `path`."""
    return glob_regex(pattern).match(path) is not None


def tree_files(tree: Path) -> list[str]:
    """Every file (and symbolic link) under `tree` as a sorted relative POSIX path."""
    found: list[str] = []
    for root, directories, files in os.walk(tree, followlinks=False):
        relative = Path(root).relative_to(tree)
        for name in files:
            found.append((relative / name).as_posix())
        for name in directories:
            if (Path(root) / name).is_symlink():
                found.append((relative / name).as_posix())
    return sorted(found, key=lambda path: path.encode("utf-8", "surrogateescape"))


def payload_files(tree: Path, include: Iterable[str], exclude: Iterable[str]) -> list[str]:
    """The tree's files matched by `include` and not by `exclude`, sorted."""
    included = [glob_regex(pattern) for pattern in include]
    excluded = [glob_regex(pattern) for pattern in exclude]
    return [
        path
        for path in tree_files(tree)
        if any(expression.match(path) for expression in included)
        and not any(expression.match(path) for expression in excluded)
    ]
