# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded, source-owned guidance tied to one loaded interface revision.

Skill main files use ordinary lazy resources: FastMCP's SkillProvider eagerly
loads SKILL.md, which would bypass this shared snapshot and refusal contract.
Owned process-skill packages share captured membership and source identities with
their main guide. Deliberate supporting-file reads neither activate nor execute a
skill. The stock provider's eager construction and live reads do not preserve
this lazy captured-revision contract.
"""

from __future__ import annotations

import mimetypes
import os
import stat
from contextlib import ExitStack, contextmanager
from dataclasses import dataclass
from functools import partial
from hashlib import sha256
from pathlib import Path
from threading import RLock
from typing import TYPE_CHECKING, TypedDict

from fastmcp.resources import FunctionResource, ResourceContent, ResourceResult

if TYPE_CHECKING:
    from collections.abc import Generator, Sequence

    from fastmcp import FastMCP

    from scripts.capabilities.model import Guide

MAX_GUIDE_BYTES = 128 * 1024
MAX_SKILL_FILES = 64
MAX_SKILL_BYTES = 1024 * 1024


class SupportingFile(TypedDict):
    id: str
    path: str
    uri: str
    digest: str
    bytes: int


class PackageLimits(TypedDict):
    file_bytes: int
    files: int
    package_bytes: int


class SkillManifest(TypedDict):
    guide_id: str
    guide_digest: str
    package_digest: str
    complete: bool
    limits: PackageLimits
    files: list[SupportingFile]


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
    content: str | bytes
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
        # Stat-only membership capture keeps startup lazy. Only local process
        # skill packages are served; shared symlinked library skills retain their
        # separate loader/access owner.
        self._packages = {
            guide.id: self._observe_package(guide)
            for guide in self.guides
            if guide.kind == "skill" and Path(guide.path).name == "SKILL.md"
        }
        self._package_snapshots: dict[str, dict[str, _Snapshot]] = {}

    def _observe_package(self, guide: Guide) -> dict[str, _Identity] | GuidanceError:
        """Walk stat-only membership beneath held no-follow directory handles."""
        try:
            members: dict[str, _Identity] = {}
            total = 0

            def walk(descriptor: int, parent: Path) -> None:
                nonlocal total
                for name in sorted(os.listdir(descriptor)):
                    info = os.stat(name, dir_fd=descriptor, follow_symlinks=False)
                    if stat.S_ISLNK(info.st_mode):
                        raise GuidanceError("package_symlink", guide.id, "Skill packages cannot contain symlinked paths.")
                    relative = parent / name
                    if stat.S_ISDIR(info.st_mode):
                        child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                                        dir_fd=descriptor)
                        try:
                            walk(child, relative)
                        finally:
                            os.close(child)
                    elif stat.S_ISREG(info.st_mode):
                        source = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                                         dir_fd=descriptor)
                        try:
                            if not stat.S_ISREG(os.fstat(source).st_mode):
                                raise GuidanceError("invalid_source", guide.id, "Skill sources must be regular files.")
                            observed = self._identity(source, relative.as_posix())
                        finally:
                            os.close(source)
                        members[relative.as_posix()] = observed
                        total += observed.size
                        if observed.size > MAX_GUIDE_BYTES:
                            raise GuidanceError("oversized_source", guide.id, "A skill file exceeds the 128 KiB limit.")
                        if len(members) > MAX_SKILL_FILES or total > MAX_SKILL_BYTES:
                            raise GuidanceError("oversized_package", guide.id, "A skill package exceeds 64 files or 1 MiB.")
                    else:
                        raise GuidanceError("invalid_source", guide.id, "Skill sources must be regular files.")

            parent = Path(guide.path).parent
            with ExitStack() as stack:
                descriptor = os.open(self.root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
                stack.callback(os.close, descriptor)
                relative = Path()
                for part in parent.parts:
                    if stat.S_ISLNK(os.stat(part, dir_fd=descriptor, follow_symlinks=False).st_mode):
                        raise GuidanceError("package_symlink", guide.id, "Skill packages cannot contain symlinked paths.")
                    descriptor = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                                         dir_fd=descriptor)
                    stack.callback(os.close, descriptor)
                    relative /= part
                walk(descriptor, relative)
            if guide.path not in members:
                raise GuidanceError("missing_source", guide.id, "Skill main source is missing or inaccessible.")
            return dict(sorted(members.items()))
        except (OSError, RuntimeError):
            return GuidanceError("missing_source", guide.id, "Skill package is missing or inaccessible.")
        except GuidanceError as error:
            return error

    @staticmethod
    def _file_id(path: str) -> str:
        return sha256(path.encode()).hexdigest()

    @staticmethod
    def _package_digest(captured: dict[str, _Snapshot]) -> str:
        digest = sha256()
        for path, snapshot in captured.items():
            encoded = path.encode()
            digest.update(len(encoded).to_bytes(8, "big"))
            digest.update(encoded)
            digest.update(bytes.fromhex(snapshot.digest))
        return digest.hexdigest()

    def resource_owners(self) -> dict[str, str]:
        """Exact source-resource ownership for visibility and loaded fences."""
        owners = {guide.uri: guide.id for guide in self.guides}
        for identity, package in self._packages.items():
            uri = self._guides[identity].uri
            owners[uri + "/manifest"] = identity
            if isinstance(package, dict):
                owners.update({uri + "/files/" + self._file_id(path): identity
                               for path in package if path != self._guides[identity].path})
        return owners

    def _prepare_packages(self, selected: list[str]) -> dict[str, dict[str, _Snapshot]]:
        pending: dict[str, dict[str, _Snapshot]] = {}
        for identity in selected:
            if identity not in self._packages:
                continue
            baseline = self._packages[identity]
            if isinstance(baseline, GuidanceError):
                raise baseline
            if self._observe_package(self._guides[identity]) != baseline:
                self._changed.add(identity)
                continue
            if identity in self._package_snapshots:
                continue
            captured: dict[str, _Snapshot] = {}
            for path, source_identity in baseline.items():
                source = self._guides[identity].model_copy(update={"path": path})
                snapshot = self._capture(source, source_identity)
                if snapshot is None:
                    break
                captured[path] = snapshot
            if len(captured) == len(baseline):
                pending[identity] = captured
            else:
                self._changed.add(identity)
        # The bundle owner rechecks all main/package identities together before
        # making any of these pending snapshots visible.
        return pending

    def manifest(self, guide_id: str) -> SkillManifest:
        """Describe the complete captured skill package without embedding bodies."""
        with self._lock:
            main = self.read(guide_id)
            if guide_id not in self._packages:
                raise GuidanceError("not_skill_package", guide_id, "Guide has no owned supporting-file package.")
            if main["restart_required"] or not main["complete"]:
                raise GuidanceError("restart_required", guide_id, "Skill package changed; restart before reading supporting files.")
            guide = self._guides[guide_id]
            captured = self._package_snapshots[guide_id]
            return {
                "guide_id": guide_id, "guide_digest": self._snapshots[guide_id].digest,
                "package_digest": self._package_digest(captured),
                "complete": True,
                "limits": {"file_bytes": MAX_GUIDE_BYTES, "files": MAX_SKILL_FILES, "package_bytes": MAX_SKILL_BYTES},
                "files": [
                    {"id": self._file_id(path), "path": Path(path).relative_to(Path(guide.path).parent).as_posix(),
                     "uri": guide.uri if path == guide.path else guide.uri + "/files/" + self._file_id(path),
                     "digest": snapshot.digest, "bytes": len(snapshot.content.encode() if isinstance(snapshot.content, str) else snapshot.content)}
                    for path, snapshot in captured.items()
                ],
            }

    def read_support(self, guide_id: str, file_id: str) -> ResourceResult:
        with self._lock:
            manifest = self.manifest(guide_id)
            guide = self._guides[guide_id]
            captured = self._package_snapshots[guide_id]
            path = next((path for path in captured if self._file_id(path) == file_id and path != guide.path), None)
            if path is None:
                raise GuidanceError("unknown_support", guide_id, "Supporting file is not in this captured skill package.")
            snapshot = captured[path]
            metadata = {"guide_id": guide_id, "guide_digest": manifest["guide_digest"],
                        "package_digest": manifest["package_digest"], "digest": snapshot.digest,
                        "path": path, "complete": True, "restart_required": False}
            return ResourceResult(contents=[ResourceContent(content=snapshot.content,
                mime_type=mimetypes.guess_type(path)[0] or "text/plain", meta=metadata)], meta=metadata)

    def references(self, ids: Sequence[str]) -> list[dict[str, object]]:
        """Compact complete-resource receipts; no instruction body is duplicated."""
        return [{key: value for key, value in item.items() if key != "content"}
                for item in self.instruction_bundle(ids)]

    def resource(self, guide_id: str) -> ResourceResult:
        value = self.read(guide_id)
        metadata = {key: item for key, item in value.items() if key != "content"}
        return ResourceResult(contents=[ResourceContent(content=value,
            mime_type="application/json", meta=metadata)], meta=metadata)

    def manifest_resource(self, guide_id: str) -> ResourceResult:
        manifest = self.manifest(guide_id)
        return ResourceResult(contents=[ResourceContent(content=manifest, mime_type="application/json")],
                              meta={"guide_id": guide_id, "package_digest": manifest["package_digest"]})

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
        content: str | bytes
        try:
            content = data.decode("utf-8")
        except UnicodeDecodeError as error:
            if guide.path == self._guides[guide.id].path:
                raise GuidanceError(
                    "invalid_encoding", guide.id, "Guide source must be UTF-8 Markdown."
                ) from error
            content = data
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
            packages = self._prepare_packages(selected)
            pending: dict[str, _Snapshot] = {
                identity: captured[self._guides[identity].path]
                for identity, captured in packages.items()
            }
            if not self._changed.intersection(selected):
                for guide_id in selected:
                    if guide_id not in self._snapshots and guide_id not in pending:
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
                for identity in selected:
                    if identity in self._packages and self._observe_package(self._guides[identity]) != self._packages[identity]:
                        self._changed.add(identity)
                if not self._changed.intersection(selected):
                    self._snapshots.update(pending)
                    self._package_snapshots.update(packages)
            restart = bool(self._changed.intersection(selected))
            return [
                self._result(self._guides[guide_id], restart) for guide_id in selected
            ]

    def _result(self, guide: Guide, restart: bool) -> dict[str, object]:
        snapshot = self._snapshots.get(guide.id)
        package = self._package_snapshots.get(guide.id)
        return {
            "id": guide.id,
            "uri": guide.uri,
            "path": guide.path,
            "content": snapshot.content if snapshot else "",
            "digest": snapshot.digest if snapshot else None,
            "restart_required": restart,
            "complete": snapshot is not None,
            "supporting_manifest": guide.uri + "/manifest" if guide.id in self._packages else None,
            "package_digest": self._package_digest(package) if package else None,
            "omissions": []
            if snapshot
            else ["Restart the interface before reading this guide."],
        }


def register_guides(server: FastMCP, guidance: Guidance) -> dict[str, str]:
    """Register lazy resources; the composition root owns visibility selection."""
    keys: dict[str, str] = {}
    for guide in guidance.guides:
        resource = FunctionResource.from_function(
            partial(guidance.resource, guide.id),
            uri=guide.uri,
            name=guide.id,
            title=guide.title,
            description="Owned Markdown instructions; retrieval performs no workflow effects.",
            mime_type="application/json",
            tags={"guide:" + guide.id},
        )
        server.add_resource(resource)
        keys[guide.id] = resource.key
        package = guidance._packages.get(guide.id)
        if package is not None:
            manifest = FunctionResource.from_function(
                partial(guidance.manifest_resource, guide.id), uri=guide.uri + "/manifest",
                name=guide.id + "-manifest", description="Complete captured owned skill supporting-file manifest; no activation or execution.",
                mime_type="application/json", tags={"guide:" + guide.id})
            server.add_resource(manifest)
            keys[guide.id + "/manifest"] = manifest.key
        if isinstance(package, dict):
            for path in package:
                if path == guide.path:
                    continue
                file_id = guidance._file_id(path)
                support = FunctionResource.from_function(
                    partial(guidance.read_support, guide.id, file_id),
                    uri=guide.uri + "/files/" + file_id, name=guide.id + "-" + file_id,
                    description="Read one complete captured owned supporting file; no trust, activation or execution.",
                    mime_type=mimetypes.guess_type(path)[0] or "text/plain", tags={"guide:" + guide.id})
                server.add_resource(support)
                keys[guide.id + "/" + file_id] = support.key
    return keys
