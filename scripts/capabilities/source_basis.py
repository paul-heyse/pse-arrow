# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded local source bytes without constructing a capability server."""
from __future__ import annotations

import ast
import hashlib
import json
import stat
from dataclasses import dataclass
from pathlib import Path
from threading import RLock
from typing import TYPE_CHECKING

from scripts.capabilities.source_modules import module_candidates

if TYPE_CHECKING:
    from collections.abc import Iterable

@dataclass(frozen=True)
class Source:
    digest: str | None
    dependencies: tuple[str, ...]


def local_imports(raw: bytes) -> tuple[str, ...]:
    """Follow declared local imports, including lazy imports inside operations."""
    paths = set()
    for node in ast.walk(ast.parse(raw)):
        names: list[str] = []
        if isinstance(node, ast.Import):
            names = [alias.name for alias in node.names]
        elif isinstance(node, ast.ImportFrom) and node.module:
            names = [node.module, *(node.module + "." + a.name for a in node.names)]
        for name in names:
            if name.startswith("scripts."):
                paths.update(module_candidates(name))
    return tuple(sorted(paths))


class Sources:
    def __init__(self, root: Path, paths: Iterable[str], *, follow_imports: bool = True) -> None:
        self.root = root.resolve(strict=True)
        self.sources: dict[str, Source] = {}
        self.changed: set[str] = set()
        self._lock = RLock()
        pending = list(paths)
        while pending:
            relative = pending.pop()
            if relative in self.sources:
                continue
            path = self.root / relative
            raw = self._read_optional(relative)
            dependencies = (
                local_imports(raw) if follow_imports and raw is not None and path.suffix == ".py" else ()
            )
            self.sources[relative] = Source(
                hashlib.sha256(raw).hexdigest() if raw is not None else None,
                dependencies,
            )
            pending.extend(dependencies)
        self.revision = hashlib.sha256(
            json.dumps(
                {path: source.digest for path, source in sorted(self.sources.items())},
                sort_keys=True,
            ).encode()
        ).hexdigest()

    def _read_optional(self, relative: str) -> bytes | None:
        try:
            return self._read(relative)
        except FileNotFoundError:
            return None

    def _read(self, relative: str) -> bytes:
        path = self.root / relative
        if path.is_symlink() or not path.resolve(strict=True).is_relative_to(self.root):
            raise ValueError("Serving source must remain within its selected checkout")
        before = path.stat()
        if not stat.S_ISREG(before.st_mode) or before.st_size > 4 << 20:
            raise ValueError("Serving source must be a bounded ordinary file")
        raw = path.read_bytes()
        after = path.stat()
        if (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
            before.st_mode,
        ) != (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
            after.st_mode,
        ):
            raise ValueError("Serving source changed during interface capture")
        return raw

    def closure(self, paths: Iterable[str]) -> frozenset[str]:
        selected: set[str] = set()
        pending = list(paths)
        while pending:
            path = pending.pop()
            if path in selected or path not in self.sources:
                continue
            selected.add(path)
            pending.extend(self.sources[path].dependencies)
        return frozenset(selected)

    def check(self, paths: Iterable[str]) -> tuple[str, ...]:
        with self._lock:
            for path in paths:
                if path not in self.sources or path in self.changed:
                    continue
                try:
                    raw = self._read_optional(path)
                    digest = (
                        hashlib.sha256(raw).hexdigest() if raw is not None else None
                    )
                except (OSError, ValueError):
                    self.changed.add(path)
                else:
                    if digest != self.sources[path].digest:
                        self.changed.add(path)
            return tuple(sorted(self.changed.intersection(paths)))

    def verify_composition(self) -> None:
        if self.check(self.sources):
            raise ValueError(
                "Serving source changed while loading; restart the endpoint"
            )

