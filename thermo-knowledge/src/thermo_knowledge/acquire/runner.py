# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The three command forms: acquire, check, list.

Only `acquire_sources` reaches the network, and only inside the acquisition kind modules.
`check_sources` and `list_sources` read the lock and the store and nothing else (a `local`
source's check runs local git commands).
"""

from __future__ import annotations

import shutil
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

import msgspec

from thermo_knowledge.acquire import download, git, local, none, pages, store
from thermo_knowledge.acquire.errors import AcquireError, UsageError
from thermo_knowledge.acquire.lock import LockEntry, read_lock, record_entry
from thermo_knowledge.acquire.manifest import (
    ArchiveSpec,
    FileSpec,
    GitSpec,
    LocalSpec,
    Manifest,
    NoneSpec,
    PagesSpec,
)
from thermo_knowledge.acquire.outcome import Outcome
from thermo_knowledge.acquire.runtime import Context, format_time
from thermo_knowledge.acquire.store import PIN_LENGTH, TREE_DIR_NAME


class State(StrEnum):
    ABSENT = "absent"
    PRESENT = "present"
    MISMATCHED = "mismatched"
    NOT_ACQUIRED = "not acquired"


def select(manifests: dict[str, Manifest], ids: list[str]) -> list[Manifest]:
    """The named manifests in the order named, or all of them sorted; unknown ids are refused."""
    if not ids:
        return list(manifests.values())
    unknown = [source_id for source_id in ids if source_id not in manifests]
    if unknown:
        known = ", ".join(manifests) or "(none)"
        raise UsageError(f"unknown source id(s): {', '.join(unknown)}; declared sources: {known}")
    return [manifests[source_id] for source_id in dict.fromkeys(ids)]


def declared_pin(spec: Manifest) -> str | None:
    """The pin the manifest alone determines, or `None` when only acquisition resolves it."""
    acquire = spec.acquire
    match acquire:
        case GitSpec():
            return acquire.commit[:PIN_LENGTH]
        case ArchiveSpec():
            declared = acquire.sha256 or acquire.md5
            return declared[:PIN_LENGTH] if declared else None
        case FileSpec():
            return download.file_pin(acquire.files)
        case LocalSpec():
            return local.declared_pin(acquire)
        case PagesSpec() | NoneSpec():
            return None


def _stores_files(spec: Manifest) -> bool:
    return isinstance(spec.acquire, GitSpec | ArchiveSpec | FileSpec | PagesSpec)


def _lock_entry(acquisition: store.Acquisition) -> LockEntry:
    return LockEntry(
        kind=acquisition.kind,
        pin=acquisition.pin,
        retrieved=acquisition.retrieved,
        file_count=acquisition.file_count,
        total_bytes=acquisition.total_bytes,
        tree_hash=acquisition.tree_hash,
        resolved=acquisition.resolved,
    )


def _same_resolution(a: LockEntry, b: LockEntry) -> bool:
    return msgspec.structs.replace(a, retrieved=b.retrieved) == b


# -- store state --------------------------------------------------------------------------


def _pin_state(
    ctx: Context, manifest: Manifest, entry: LockEntry | None, pin: str
) -> tuple[State, str]:
    directory = store.pin_dir(ctx.raw_dir, manifest.id, pin)
    try:
        acquisition = store.read_acquisition(directory)
    except AcquireError as error:
        if directory.is_dir():
            return State.MISMATCHED, str(error)
        return State.MISMATCHED, f"{directory} is missing although the lock records it"
    if entry is None:
        return State.MISMATCHED, "the lock has no entry for it"
    if entry.tree_hash != acquisition.tree_hash:
        return State.MISMATCHED, "the lock's tree hash differs from the store's"
    return State.PRESENT, ""


def store_state(
    ctx: Context, manifest: Manifest, entries: dict[str, LockEntry]
) -> tuple[State, str]:
    """Where the source stands, from the lock and the store; nothing is hashed.

    absent: neither the lock nor the store has it. present: the store holds the pin the lock
    records and the manifest declares. mismatched: anything in between.
    """
    spec = manifest.acquire
    entry = entries.get(manifest.id)
    kind = manifest.kind
    if isinstance(spec, NoneSpec):
        return State.NOT_ACQUIRED, spec.reason
    if isinstance(spec, LocalSpec):
        if entry is None:
            return State.ABSENT, ""
        if entry.kind != kind or entry.path != spec.path:
            return State.MISMATCHED, "the lock records a different kind or path"
        if spec.commit is not None and entry.resolved != spec.commit:
            return State.MISMATCHED, f"the lock records {entry.pin}"
        return State.PRESENT, ""

    pin = declared_pin(manifest)
    if pin is not None:
        if store.pin_dir(ctx.raw_dir, manifest.id, pin).is_dir():
            if entry is None:
                return State.MISMATCHED, "the lock has no entry for it"
            if entry.kind != kind or entry.pin != pin:
                return State.MISMATCHED, f"the lock records pin {entry.pin}, declared {pin}"
            return _pin_state(ctx, manifest, entry, pin)
        if entry is None:
            return State.ABSENT, ""
        if entry.kind == kind and entry.pin == pin:
            return State.MISMATCHED, "the lock records the pin but the store directory is missing"
        return State.MISMATCHED, f"the lock records pin {entry.pin}, declared {pin}"

    if entry is None:
        return State.ABSENT, ""
    if entry.kind != kind or entry.pin is None:
        return State.MISMATCHED, f"the lock records kind {entry.kind}"
    return _pin_state(ctx, manifest, entry, entry.pin)


# -- list ---------------------------------------------------------------------------------


@dataclass(frozen=True)
class ListRow:
    source_id: str
    tier: str
    kind: str
    pin: str
    state: State
    detail: str

    def format(self) -> str:
        line = f"{self.source_id:<24} {self.tier:<5} {self.kind:<8} {self.pin:<14} {self.state}"
        return f"{line} ({self.detail})" if self.detail else line


def list_sources(
    ctx: Context, manifests: list[Manifest], entries: dict[str, LockEntry]
) -> list[ListRow]:
    rows = []
    for manifest in manifests:
        state, detail = store_state(ctx, manifest, entries)
        rows.append(
            ListRow(
                manifest.id,
                manifest.tier,
                manifest.kind,
                declared_pin(manifest) or "-",
                state,
                detail if state is not State.PRESENT else "",
            )
        )
    return rows


# -- check --------------------------------------------------------------------------------


@dataclass(frozen=True)
class CheckResult:
    source_id: str
    problems: list[str]
    note: str = ""

    @property
    def ok(self) -> bool:
        return not self.problems

    def lines(self) -> list[str]:
        if self.ok:
            return [f"ok    {self.source_id}" + (f" ({self.note})" if self.note else "")]
        return [f"FAIL  {self.source_id}", *[f"      {problem}" for problem in self.problems]]


def _check_local(
    ctx: Context, manifest: Manifest, spec: LocalSpec, entry: LockEntry | None
) -> list[str]:
    try:
        verified = local.verify(ctx.repo_root, manifest.id, spec)
    except AcquireError as error:
        return [str(error)]
    if entry is None:
        return ["not in the lock; run `tk acquire` to record the verified revision"]
    if entry.kind != manifest.kind or entry.resolved != verified.commit:
        return [
            f"the checkout is at {verified.commit} but the lock records "
            f"{entry.resolved or entry.pin}"
        ]
    return []


def _check_stored(ctx: Context, manifest: Manifest, entry: LockEntry | None) -> list[str]:
    if entry is None:
        return ["absent: not in the lock; run `tk acquire`"]
    problems: list[str] = []
    pin = declared_pin(manifest)
    if entry.kind != manifest.kind:
        problems.append(
            f"the lock records kind {entry.kind}, the manifest declares {manifest.kind}"
        )
    if pin is not None and entry.pin != pin:
        problems.append(f"the lock records pin {entry.pin}, the manifest declares {pin}")
    if entry.pin is None or entry.tree_hash is None:
        problems.append("the lock entry has no pin or tree hash")
        return problems
    directory = store.pin_dir(ctx.raw_dir, manifest.id, entry.pin)
    if not directory.is_dir():
        problems.append(f"{directory}: the store directory the lock records is missing")
        return problems
    actual = store.scan_tree(directory / TREE_DIR_NAME)
    try:
        acquisition: store.Acquisition | None = store.read_acquisition(directory)
    except AcquireError as error:
        acquisition = None
        problems.append(str(error))
    if acquisition is not None:
        problems.extend(store.compare_tree(acquisition.files, actual).lines())
        if acquisition.tree_hash != entry.tree_hash:
            problems.append(
                f"{store.ACQUISITION_NAME} records tree hash {acquisition.tree_hash}, "
                f"the lock records {entry.tree_hash}"
            )
    found = store.tree_hash(actual)
    if found != entry.tree_hash:
        problems.append(f"tree hash is {found}, the lock records {entry.tree_hash}")
    return problems


def check_sources(
    ctx: Context, manifests: list[Manifest], entries: dict[str, LockEntry]
) -> list[CheckResult]:
    """Offline: recompute each tree hash from the store and compare it with the lock."""
    results = []
    for manifest in manifests:
        spec = manifest.acquire
        entry = entries.get(manifest.id)
        if isinstance(spec, NoneSpec):
            results.append(CheckResult(manifest.id, [], f"not acquired: {spec.reason}"))
        elif isinstance(spec, LocalSpec):
            results.append(CheckResult(manifest.id, _check_local(ctx, manifest, spec, entry)))
        else:
            results.append(CheckResult(manifest.id, _check_stored(ctx, manifest, entry)))
    return results


# -- acquire ------------------------------------------------------------------------------


@dataclass(frozen=True)
class Result:
    source_id: str
    status: str
    message: str = ""
    ok: bool = True

    def format(self) -> str:
        return f"{self.status:<12} {self.source_id}" + (f"  {self.message}" if self.message else "")


@dataclass
class Acquirer:
    """One `tk acquire` run: acquires sources and keeps the lock in step, source by source.

    The run holds no copy of the lock. Every decision reads the file afresh and every record is
    a locked read-modify-write of one entry (`record_entry`), so runs at the same time keep each
    other's entries.
    """

    ctx: Context
    lock_path: Path

    def __post_init__(self) -> None:
        read_lock(self.lock_path)  # an invalid lock is refused before any source is acquired

    def _current(self, source_id: str) -> LockEntry | None:
        return read_lock(self.lock_path).get(source_id)

    def run(self, manifests: list[Manifest]) -> list[Result]:
        results = []
        for manifest in manifests:
            try:
                results.append(self._one(manifest))
            except AcquireError as error:
                results.append(Result(manifest.id, "refused", str(error), ok=False))
        return results

    def _record(self, source_id: str, entry: LockEntry) -> None:
        record_entry(self.lock_path, source_id, entry)

    def _one(self, manifest: Manifest) -> Result:
        spec = manifest.acquire
        now = format_time(self.ctx.runtime.now())
        if isinstance(spec, NoneSpec):
            entry = none.lock_entry(spec, now)
            return self._keep_or_record(manifest.id, entry, "not acquired", spec.reason)
        if isinstance(spec, LocalSpec):
            verified = local.verify(self.ctx.repo_root, manifest.id, spec)
            entry = LockEntry(
                "local", verified.pin, now, 0, 0, None, resolved=verified.commit, path=verified.path
            )
            return self._keep_or_record(
                manifest.id, entry, "verified", f"{verified.path} at {verified.pin}"
            )
        return self._stored(manifest)

    def _keep_or_record(
        self, source_id: str, entry: LockEntry, status: str, message: str
    ) -> Result:
        current = self._current(source_id)
        if current is not None and _same_resolution(current, entry):
            return Result(source_id, "unchanged", message)
        self._record(source_id, entry)
        return Result(source_id, status, message)

    def _stored(self, manifest: Manifest) -> Result:
        source_id = manifest.id
        current = self._current(source_id)
        pin = declared_pin(manifest)
        if pin is not None:
            if store.pin_dir(self.ctx.raw_dir, source_id, pin).is_dir():
                return self._reverify(manifest, current, pin)
        elif current is not None and current.kind == manifest.kind and current.pin is not None:
            return self._reverify(manifest, current, current.pin)
        entry = self._acquire_new(manifest, pin)
        self._record(source_id, entry)
        return Result(source_id, "acquired", f"pin {entry.pin}, {entry.file_count} files")

    def _reverify(self, manifest: Manifest, current: LockEntry | None, pin: str) -> Result:
        """The pin is in the store: check it against its own record and the lock; never write
        into it."""
        source_id = manifest.id
        directory = store.pin_dir(self.ctx.raw_dir, source_id, pin)
        acquisition = store.read_acquisition(directory)
        actual = store.scan_tree(directory / TREE_DIR_NAME)
        difference = store.compare_tree(acquisition.files, actual)
        if not difference.empty or store.tree_hash(actual) != acquisition.tree_hash:
            details = "\n  ".join(difference.lines()) or "tree hash differs"
            raise AcquireError(
                f"{source_id}: the store at {directory} disagrees with its "
                f"{store.ACQUISITION_NAME}; it is never modified in place, so remove the "
                f"directory and its lock entry to acquire again:\n  {details}"
            )
        entry = _lock_entry(acquisition)
        if current is not None and current.kind == entry.kind and current.pin == entry.pin:
            if current.tree_hash != entry.tree_hash:
                raise AcquireError(
                    f"{source_id}: the store at {directory} has tree hash {entry.tree_hash} but "
                    f"the lock records {current.tree_hash}; remove the directory and the lock "
                    "entry to acquire again"
                )
            return Result(source_id, "unchanged", f"pin {pin}")
        self._record(source_id, entry)
        return Result(source_id, "verified", f"pin {pin} was in the store; lock updated")

    def _acquire_new(self, manifest: Manifest, known_pin: str | None) -> LockEntry:
        runtime = self.ctx.runtime
        source_id = manifest.id
        spec = manifest.acquire
        source_root = store.source_dir(self.ctx.raw_dir, source_id)
        work = store.partial_dir(self.ctx.raw_dir, source_id)
        resumable = isinstance(spec, PagesSpec)
        if not resumable:
            store.remove_tree(work)
        work.mkdir(parents=True, exist_ok=True)
        try:
            match spec:
                case GitSpec():
                    outcome: Outcome = git.acquire(source_id, spec, work)
                case ArchiveSpec():
                    outcome = download.acquire_archive(runtime, source_id, spec, work)
                case FileSpec():
                    outcome = download.acquire_files(runtime, source_id, spec, work)
                case PagesSpec():
                    outcome = pages.acquire(runtime, source_id, spec, work)
                case _:
                    raise AcquireError(f"{source_id}: kind {manifest.kind} has nothing to store")
            if known_pin is not None and outcome.pin != known_pin:
                raise AcquireError(
                    f"{source_id}: acquisition resolved to pin {outcome.pin} but the manifest "
                    f"declares {known_pin}"
                )
            for child in work.iterdir():
                if child.name == TREE_DIR_NAME:
                    continue
                if child.is_dir():
                    shutil.rmtree(child)
                else:
                    child.unlink()
            entries = store.scan_tree(work / TREE_DIR_NAME)
            acquisition = store.build_acquisition(
                source_id=source_id,
                kind=manifest.kind,
                pin=outcome.pin,
                resolved=outcome.resolved,
                retrieved=format_time(runtime.now()),
                tool_versions=outcome.tool_versions,
                urls=outcome.urls,
                details=outcome.details,
                entries=entries,
            )
            store.write_acquisition(work, acquisition)
            store.finalize(work, store.pin_dir(self.ctx.raw_dir, source_id, outcome.pin))
        except BaseException:
            if not resumable:
                store.remove_tree(work)
            store.prune_empty(source_root)
            raise
        return _lock_entry(acquisition)
