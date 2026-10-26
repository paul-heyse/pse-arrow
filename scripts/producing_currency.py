# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Endpoint-held witnesses for finite, actually observed producing input sets.

There is no publication store here: preparation owners put the input selection and
snapshot in their existing receipt. File bytes are hashed only after metadata or
link selection changes; directory membership is part of the snapshot.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping


def stamp(path: Path) -> tuple[int, int, int, int, int, int]:
    value = path.stat()
    return (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
            value.st_ctime_ns, value.st_mode)


def string_list(value: object) -> list[str]:
    if not isinstance(value, list):
        raise ValueError("malformed producing input selection")
    result = [item for item in value if isinstance(item, str)]
    if len(result) != len(value):
        raise ValueError("malformed producing input selection")
    return result


class Witness:
    """One endpoint's observations; a receipt alone cannot authorize a loaded image."""

    def __init__(self) -> None:
        self.files: dict[str, tuple[tuple[object, ...], dict[str, object]]] = {}
        self.directories: dict[str, tuple[tuple[int, ...], tuple[str, ...]]] = {}

    def file(self, path: Path) -> dict[str, object]:
        path = path.absolute()
        name = str(path)
        link = str(path.readlink()) if path.is_symlink() else None
        try:
            before = stamp(path)
        except FileNotFoundError:
            return {"path": name, "resolved": str(path.resolve()), "link": link,
                    "sha256": None, "mode": None}
        if not stat.S_ISREG(before[-1]):
            raise ValueError("producing file input is not an ordinary file")
        key = (*before, str(path.resolve()), link)
        cached = self.files.get(name)
        if cached is not None and cached[0] == key:
            return cached[1]
        with path.open("rb") as stream:
            opened = os.fstat(stream.fileno())
            opened_stamp = (opened.st_dev, opened.st_ino, opened.st_size,
                            opened.st_mtime_ns, opened.st_ctime_ns, opened.st_mode)
            if opened_stamp != before:
                raise ValueError("producing input changed before its observation")
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
            after = os.fstat(stream.fileno())
        if before != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns,
                      after.st_ctime_ns, after.st_mode) or before != stamp(path):
            raise ValueError("producing input changed during its observation")
        value = {"path": name, "resolved": str(path.resolve()), "link": link,
                 "sha256": digest, "mode": before[-1] & 0o777}
        self.files[name] = (key, value)
        return value

    def tree(self, root: Path) -> dict[str, object]:
        """Track additions/deletions and nested changes without rehashing bytes."""
        root = root.absolute()
        if root.is_symlink():
            raise ValueError("directory symlinks require conservative preparation")
        if not root.exists():
            return {str(root): {"absent": True}}
        if not root.is_dir():
            return {str(root): self.file(root)}
        result: dict[str, object] = {}
        pending = [root]
        while pending:
            directory = pending.pop()
            before = stamp(directory)
            cached = self.directories.get(str(directory))
            if cached is not None and cached[0] == before:
                names = cached[1]
            else:
                names = tuple(sorted(path.name for path in directory.iterdir()
                                     if path.name not in {".git", "target"}))
                if before != stamp(directory):
                    raise ValueError("producing directory membership changed")
                self.directories[str(directory)] = (before, names)
            result[str(directory)] = {"members": list(names), "mode": before[-1] & 0o777}
            for name in names:
                path = directory / name
                if path.is_dir():
                    if path.is_symlink():
                        raise ValueError("directory symlinks require conservative preparation")
                    pending.append(path)
                else:
                    result[str(path)] = self.file(path)
        return result

    def inputs(
        self, selection: Mapping[str, object], environment: Mapping[str, str]
    ) -> dict[str, object]:
        if selection.get("version") != 1 or selection.get("scope") != "ordinary-cargo-inputs":
            raise ValueError("unsupported producing input selection")
        files, directories, names = (string_list(selection.get(key)) for key in ("files", "directories", "environment"))
        observed = {name: self.file(Path(name)) for name in files}
        for directory in directories:
            observed.update(self.tree(Path(directory)))
        return {"files": observed, "environment": {
            name: hashlib.sha256(json.dumps(environment.get(name), ensure_ascii=True).encode()).hexdigest()
            for name in names}}


def basis(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def retained_selection(selection: Mapping[str, object]) -> dict[str, object]:
    """Retain actual record meaning without persisting raw compile environment values."""
    result = dict(selection)
    environment = result.get("rustc_environment")
    if isinstance(environment, dict):
        result["rustc_environment"] = {name: basis(value) for name, value in environment.items()}
    scripts = result.get("build_scripts")
    if isinstance(scripts, list):
        retained = []
        for script in scripts:
            if not isinstance(script, dict):
                raise ValueError("malformed actual build-script record")
            value = dict(script)
            raw = value.get("env")
            if isinstance(raw, list):
                if any(not isinstance(pair, list) or len(pair) != 2 or not isinstance(pair[0], str) for pair in raw):
                    raise ValueError("malformed actual build-script environment")
                value["env"] = [[pair[0], basis(pair[1])] for pair in raw]
            retained.append(value)
        result["build_scripts"] = retained
    invocation = result.get("invocation")
    if isinstance(invocation, dict):
        invocation = dict(invocation)
        environment = invocation.get("effective_environment")
        if isinstance(environment, dict):
            invocation["effective_environment"] = {name: basis(value) for name, value in environment.items()}
        result["invocation"] = invocation
    return result
