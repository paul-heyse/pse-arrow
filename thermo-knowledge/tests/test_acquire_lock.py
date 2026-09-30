# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The lock under concurrent `tk acquire` runs: one source's record is a locked
read-modify-write, and the file is replaced atomically."""

from __future__ import annotations

import json
import os
import threading
from pathlib import Path

import pytest
from test_acquire_support import make_context, write_manifest

from thermo_knowledge.acquire import lock as lock_module
from thermo_knowledge.acquire.lock import (
    LockEntry,
    encode_lock,
    exclusive,
    lock_guard_path,
    read_lock,
    record_entry,
    write_lock,
)
from thermo_knowledge.acquire.manifest import load_sources
from thermo_knowledge.acquire.runner import Acquirer, select

NONE_BODY = 'kind = "none"\nreason = "terms of use"'


def none_entry(retrieved: str = "2026-09-30T12:00:00Z") -> LockEntry:
    return LockEntry("none", None, retrieved, 0, 0, None, reason="terms of use")


def sources_for(tmp_path: Path, *ids: str) -> Path:
    sources = tmp_path / "sources"
    for source_id in ids:
        write_manifest(sources, source_id, NONE_BODY)
    return sources


def acquirer(tmp_path: Path, lock: Path) -> Acquirer:
    return Acquirer(make_context(tmp_path), lock)


@pytest.mark.parametrize("order", [("janaf", "thermoml"), ("thermoml", "janaf")])
def test_acquirers_from_the_same_lock_state_keep_each_others_entries(
    tmp_path: Path, order: tuple[str, str]
) -> None:
    sources = sources_for(tmp_path, "janaf", "thermoml")
    lock = tmp_path / "sources.lock"
    manifests = load_sources(sources)
    # Both runs start from the same (empty) lock state, as two processes started together do.
    runs = {name: acquirer(tmp_path, lock) for name in order}
    for name in order:
        [result] = runs[name].run(select(manifests, [name]))
        assert result.ok, result.message
        assert result.status == "not acquired"
    assert set(read_lock(lock)) == {"janaf", "thermoml"}
    assert lock.read_bytes() == encode_lock(read_lock(lock))


def test_the_keep_or_record_decision_uses_the_current_file(tmp_path: Path) -> None:
    sources = sources_for(tmp_path, "janaf")
    lock = tmp_path / "sources.lock"
    manifests = select(load_sources(sources), [])
    first, second = acquirer(tmp_path, lock), acquirer(tmp_path, lock)
    [recorded] = first.run(manifests)
    assert recorded.status == "not acquired"
    before = lock.read_bytes()
    [again] = second.run(manifests)  # built before the first run recorded; must see its entry
    assert again.status == "unchanged"
    assert lock.read_bytes() == before


def test_record_entry_reads_the_file_under_the_lock_and_sets_one_entry(tmp_path: Path) -> None:
    lock = tmp_path / "sources.lock"
    write_lock(lock, {"a": none_entry("2026-01-01T00:00:00Z")})
    record_entry(lock, "b", none_entry())
    record_entry(lock, "a", none_entry("2026-02-02T00:00:00Z"))
    entries = read_lock(lock)
    assert set(entries) == {"a", "b"}
    assert entries["a"].retrieved == "2026-02-02T00:00:00Z"
    assert lock.read_bytes() == encode_lock(entries)
    assert json.loads(lock.read_text())["version"] == 1


def test_a_recorder_waits_for_the_holder_of_the_exclusive_lock(tmp_path: Path) -> None:
    lock = tmp_path / "sources.lock"
    write_lock(lock, {})
    started = threading.Event()

    def record() -> None:
        started.set()
        record_entry(lock, "a", none_entry())

    worker = threading.Thread(target=record)
    with exclusive(lock):
        worker.start()
        started.wait()
        worker.join(timeout=0.3)
        assert worker.is_alive()  # blocked on the sidecar lock
        assert read_lock(lock) == {}
    worker.join(timeout=10)
    assert not worker.is_alive()
    assert set(read_lock(lock)) == {"a"}


def test_the_lock_is_replaced_by_rename_and_never_partially_visible(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    lock = tmp_path / "sources.lock"
    write_lock(lock, {"a": none_entry()})
    old = lock.read_bytes()
    replaced: list[tuple[Path, Path]] = []
    real_replace = os.replace

    def watching_replace(source: Path, target: Path) -> None:
        replaced.append((Path(source), Path(target)))
        assert Path(target).read_bytes() == old  # the lock is still the old, complete file
        assert Path(source).parent == Path(target).parent  # a rename within one directory
        assert Path(source).read_bytes() == encode_lock(
            {"a": none_entry(), "b": none_entry()}
        )  # and the new content is complete before it is visible
        real_replace(source, target)

    monkeypatch.setattr(lock_module.os, "replace", watching_replace)
    record_entry(lock, "b", none_entry())
    assert [target for _, target in replaced] == [lock]
    assert set(read_lock(lock)) == {"a", "b"}
    assert sorted(path.name for path in tmp_path.iterdir()) == [
        "sources.lock",
        lock_guard_path(lock).name,
    ]


def test_a_failed_write_leaves_the_old_lock_and_no_temporary_file(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    lock = tmp_path / "sources.lock"
    write_lock(lock, {"a": none_entry()})
    old = lock.read_bytes()

    def failing_replace(source: Path, target: Path) -> None:
        raise OSError("disk gone")

    monkeypatch.setattr(lock_module.os, "replace", failing_replace)
    with pytest.raises(OSError, match="disk gone"):
        record_entry(lock, "b", none_entry())
    assert lock.read_bytes() == old
    assert sorted(path.name for path in tmp_path.iterdir()) == [
        "sources.lock",
        lock_guard_path(lock).name,
    ]


def test_the_sidecar_lock_file_is_gitignored_by_the_tree() -> None:
    rules = (Path(__file__).parents[1] / ".gitignore").read_text().splitlines()
    assert "sources.lock.flock" in rules
