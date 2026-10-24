# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded, source-owned guidance tied to one loaded interface revision.

Skill main files use ordinary lazy resources: FastMCP's SkillProvider eagerly
loads SKILL.md, which would bypass this shared snapshot and refusal contract.
Supporting Markdown links remain relative to the returned source path; reading
a guide neither follows those links nor activates a workflow.
"""

from __future__ import annotations

import os
import stat
from contextlib import ExitStack, contextmanager
from dataclasses import dataclass
from functools import partial
from hashlib import sha256
from pathlib import Path
from threading import RLock
from typing import TYPE_CHECKING

from fastmcp.resources import FunctionResource

if TYPE_CHECKING:
    from collections.abc import Generator, Sequence

    from fastmcp import FastMCP

    from scripts.capabilities.model import Guide

MAX_GUIDE_BYTES = 128 * 1024


class GuidanceError(ValueError):
    """A structured refusal containing no source body or arbitrary file access."""

    def __init__(self, code: str, guide_id: str, message: str) -> None:
        self.details = {"code": code, "guide_id": guide_id, "message": message}
        super().__init__(self.details)


@dataclass(frozen=True)
class _Identity:
    path: str
    device: int
    inode: int
    size: int
    modified: int
    changed: int


@dataclass(frozen=True)
class _Snapshot:
    content: str
    digest: str


class Guidance:
    """Read only registered Markdown; retain complete captured instructions.

    Construction stats declared paths without reading their bodies. First reads
    must still match those identities. Once a relevant source changes, restart
    stays required: captured bodies remain readable, new bodies are withheld.
    A bundle is ordered and deduplicated and never truncates an instruction.
    """

    def __init__(self, root: Path, guides: Sequence[Guide]) -> None:
        self.root = root.resolve(strict=True)
        self.guides = tuple(guides)
        self._guides: dict[str, Guide] = {}
        self._snapshots: dict[str, _Snapshot] = {}
        self._changed: set[str] = set()
        self._lock = RLock()
        for guide in self.guides:
            if guide.id in self._guides:
                raise GuidanceError(
                    "duplicate_guide", guide.id, "Guide ID is repeated."
                )
            path = Path(guide.path)
            if path.is_absolute() or ".." in path.parts or path.suffix.lower() != ".md":
                raise GuidanceError(
                    "invalid_path",
                    guide.id,
                    "Guide must name checkout-relative Markdown.",
                )
            self._guides[guide.id] = guide
        self._baseline = {guide.id: self._observe(guide) for guide in self.guides}

    @contextmanager
    def _open(self, guide: Guide) -> Generator[tuple[int, str], None, None]:
        """Resolve internal links, then open beneath root without following swaps."""
        try:
            resolved = (self.root / guide.path).resolve(strict=True)
            relative = resolved.relative_to(self.root)
        except ValueError as error:
            raise GuidanceError(
                "path_escape", guide.id, "Guide resolves outside the checkout."
            ) from error
        except (OSError, RuntimeError) as error:
            raise GuidanceError(
                "missing_source", guide.id, "Guide source is missing or inaccessible."
            ) from error
        try:
            with ExitStack() as stack:
                directory = os.open(
                    self.root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
                )
                stack.callback(os.close, directory)
                for part in relative.parts[:-1]:
                    directory = os.open(
                        part,
                        os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                        dir_fd=directory,
                    )
                    stack.callback(os.close, directory)
                descriptor = os.open(
                    relative.name,
                    os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                    dir_fd=directory,
                )
                stack.callback(os.close, descriptor)
                if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                    raise GuidanceError(
                        "invalid_source",
                        guide.id,
                        "Guide source must be a regular file.",
                    )
                yield descriptor, relative.as_posix()
        except OSError as error:
            raise GuidanceError(
                "missing_source", guide.id, "Guide source is missing or inaccessible."
            ) from error

    @staticmethod
    def _identity(descriptor: int, path: str) -> _Identity:
        info = os.fstat(descriptor)
        return _Identity(
            path,
            info.st_dev,
            info.st_ino,
            info.st_size,
            info.st_mtime_ns,
            info.st_ctime_ns,
        )

    def _observe(self, guide: Guide) -> _Identity | GuidanceError:
        try:
            with self._open(guide) as (descriptor, path):
                return self._identity(descriptor, path)
        except GuidanceError as error:
            return error

    def _capture(self, guide: Guide, baseline: _Identity) -> _Snapshot | None:
        with self._open(guide) as (descriptor, path):
            before = self._identity(descriptor, path)
            if before != baseline:
                self._changed.add(guide.id)
                return None
            if before.size > MAX_GUIDE_BYTES:
                raise GuidanceError(
                    "oversized_source",
                    guide.id,
                    "Guide exceeds the 128 KiB instruction limit.",
                )
            # A bounded read also protects against a writer growing the source.
            with os.fdopen(os.dup(descriptor), "rb") as source:
                data = source.read(MAX_GUIDE_BYTES + 1)
            if (
                self._identity(descriptor, path) != baseline
                or self._observe(guide) != baseline
            ):
                self._changed.add(guide.id)
                return None
            if len(data) > MAX_GUIDE_BYTES:
                raise GuidanceError(
                    "oversized_source",
                    guide.id,
                    "Guide exceeds the 128 KiB instruction limit.",
                )
        try:
            content = data.decode("utf-8")
        except UnicodeDecodeError as error:
            raise GuidanceError(
                "invalid_encoding", guide.id, "Guide source must be UTF-8 Markdown."
            ) from error
        return _Snapshot(content, sha256(data).hexdigest())

    def read(self, guide_id: str) -> dict[str, object]:
        """Read one exact guide identity, never a caller-supplied path."""
        return self.instruction_bundle([guide_id])[0]

    def instruction_bundle(self, ids: Sequence[str]) -> list[dict[str, object]]:
        """Return complete selected snapshots or an explicit restart refusal."""
        selected = list(dict.fromkeys(ids))
        with self._lock:
            for guide_id in selected:
                if guide_id not in self._guides:
                    raise GuidanceError(
                        "unknown_guide", guide_id, "Guide ID is not registered."
                    )
            relevant = selected
            for guide_id in relevant:
                current = self._observe(self._guides[guide_id])
                baseline = self._baseline[guide_id]
                if isinstance(baseline, GuidanceError):
                    if (
                        isinstance(current, GuidanceError)
                        and current.details == baseline.details
                    ):
                        raise current
                    self._changed.add(guide_id)
                elif current != baseline:
                    self._changed.add(guide_id)
            pending: dict[str, _Snapshot] = {}
            if not self._changed.intersection(selected):
                for guide_id in selected:
                    if guide_id not in self._snapshots:
                        baseline = self._baseline[guide_id]
                        if isinstance(baseline, GuidanceError):
                            raise baseline
                        try:
                            snapshot = self._capture(self._guides[guide_id], baseline)
                        except GuidanceError as error:
                            if error.details["code"] == "missing_source":
                                self._changed.add(guide_id)
                                break
                            raise
                        if snapshot is None:
                            break
                        pending[guide_id] = snapshot
                # Recheck the whole selection before making a bundle visible.
                for guide_id in relevant:
                    if (
                        self._observe(self._guides[guide_id])
                        != self._baseline[guide_id]
                    ):
                        self._changed.add(guide_id)
                if not self._changed.intersection(selected):
                    self._snapshots.update(pending)
            restart = bool(self._changed.intersection(selected))
            return [self._result(self._guides[guide_id], restart) for guide_id in selected]

    def _result(self, guide: Guide, restart: bool) -> dict[str, object]:
        snapshot = self._snapshots.get(guide.id)
        return {
            "id": guide.id,
            "uri": guide.uri,
            "path": guide.path,
            "content": snapshot.content if snapshot else "",
            "digest": snapshot.digest if snapshot else None,
            "restart_required": restart,
            "complete": snapshot is not None,
            "omissions": []
            if snapshot
            else ["Restart the interface before reading this guide."],
        }


def register_guides(server: FastMCP, guidance: Guidance) -> dict[str, str]:
    """Register lazy resources; the composition root owns visibility selection."""
    keys: dict[str, str] = {}
    for guide in guidance.guides:
        resource = FunctionResource.from_function(
            partial(guidance.read, guide.id),
            uri=guide.uri,
            name=guide.id,
            title=guide.title,
            description="Owned Markdown instructions; retrieval performs no workflow effects.",
            mime_type="application/json",
            tags={"guide:" + guide.id},
        )
        server.add_resource(resource)
        keys[guide.id] = resource.key
    return keys
