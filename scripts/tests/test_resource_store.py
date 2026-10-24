# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual-file crash, exclusion and cutover controls for exact metadata."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import os
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import host_admission as host
from scripts import resource_store as files
from scripts import test_resources as resources

if TYPE_CHECKING:
    from collections.abc import Mapping


class ExactStoreTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "registry"
        self.store = files.FileStore(self.root)
        self.nonce = "a" * 32
        self.declaration = {"nonce": self.nonce, "selected": [], "kind": "python"}
        self.store.create_invocation(self.declaration)
        self.identity = ["/owned/state", "fresh_database", "embedded"]
        self.facts = {"invocation": self.nonce, "pin": True}

    def register(self) -> str:
        return self.store.register(self.identity, self.facts, invocation=self.nonce)

    def test_duplicate_and_full_identity_collision_refuse(self) -> None:
        resource = self.register()
        self.assertEqual(self.store.identity_lookup(self.identity), resource)
        with self.assertRaisesRegex(files.StoreError, "already registered"):
            self.register()
        original = files.digest(self.identity)
        digest = files.digest
        with (
            patch.object(
                files,
                "digest",
                side_effect=lambda value: (
                    original if isinstance(value, list) else digest(value)
                ),
            ),
            self.assertRaisesRegex(files.StoreError, "hash collision"),
        ):
            self.store.register(["/other", "other", "embedded"], {}, invocation=None)
        self.assertEqual(self.store.lookup(resource).facts, self.facts)

    def test_registration_publishes_three_exact_coordination_states(self) -> None:
        coordination = self.store.path("invocation-state", self.nonce)
        publish = files.publish
        transitions: list[tuple[str, str]] = []

        def observe(path: Path, value: Mapping[str, object]) -> None:
            publish(path, value)
            if path == coordination:
                state = files.read(path)
                member = next(iter(state["members"].values()))
                transitions.append((member["phase"], member["publication"]))

        with patch.object(files, "publish", side_effect=observe):
            resource = self.register()
        self.assertEqual(
            transitions,
            [
                ("pending", "not-started"),
                ("pending", "uncertain"),
                ("registered", "durable"),
            ],
        )
        self.assertEqual(
            files.read(coordination),
            {
                "version": 3,
                "phase": "open",
                "members": {
                    resource: {
                        "phase": "registered",
                        "identity": self.identity,
                        "publication": "durable",
                    }
                },
            },
        )
        self.assertEqual(self.store.identity_lookup(self.identity), resource)
        self.assertEqual(self.store.lookup(resource).facts, self.facts)
        self.assertEqual(self.store.active_ids(), [resource])
        self.assertFalse(
            self.store.path("identity-pending", files.digest(self.identity)).exists()
        )

    def test_final_registration_publication_failure_recovers_at_freeze(self) -> None:
        coordination = self.store.path("invocation-state", self.nonce)
        publish = files.publish
        coordination_writes = 0

        def fail_completion(path: Path, value: Mapping[str, object]) -> None:
            nonlocal coordination_writes
            if path == coordination:
                coordination_writes += 1
                if coordination_writes == 3:
                    raise OSError("injected final coordination publication failure")
            publish(path, value)

        with (
            patch.object(files, "publish", side_effect=fail_completion),
            self.assertRaisesRegex(OSError, "final coordination publication"),
        ):
            self.register()
        self.assertEqual(coordination_writes, 3)
        state = files.read(coordination)
        resource = next(iter(state["members"]))
        self.assertEqual(
            state["members"],
            {
                resource: {
                    "phase": "pending",
                    "identity": self.identity,
                    "publication": "uncertain",
                }
            },
        )
        # Reopen from durable files, without the failed caller's in-memory state.
        recovered = files.FileStore(self.root)
        self.assertEqual(recovered.identity_lookup(self.identity), resource)
        self.assertEqual(recovered.lookup(resource).facts, self.facts)
        self.assertFalse(
            recovered.path("identity-pending", files.digest(self.identity)).exists()
        )
        terminal = {"exit": None}
        self.assertEqual(recovered.freeze(self.nonce, terminal), [resource])
        self.assertEqual(
            files.read(coordination)["members"],
            {
                resource: {
                    "phase": "registered",
                    "identity": self.identity,
                    "publication": "durable",
                }
            },
        )
        self.assertEqual(
            recovered.terminal(self.nonce), (files.digest(terminal), terminal)
        )
        self.assertEqual(recovered.lookup(resource).facts, self.facts)
        self.assertEqual(recovered.active_ids(), [resource])

    def test_freeze_first_prevents_any_registration_publication(self) -> None:
        self.assertEqual(self.store.freeze(self.nonce, {"exit": 0}), [])
        with self.assertRaisesRegex(files.StoreError, "frozen"):
            self.register()
        self.assertEqual(self.store.active_ids(), [])

    def test_frozen_terminal_corruption_refuses_readback(self) -> None:
        self.store.freeze(self.nonce, {"exit": 0})
        path = self.store.path("invocation-state", self.nonce)
        state = files.read(path)
        path = self.store.path("invocation-terminals", state["terminal_ref"])
        detail = files.read(path)
        detail["terminal"]["exit"] = 7
        files.publish(path, detail)
        with self.assertRaisesRegex(files.StoreError, "frozen invocation terminal"):
            self.store.terminal(self.nonce)

    def test_unchanged_projection_settles_prior_rename_without_replacing_authority(
        self,
    ) -> None:
        resource = self.register()
        authority = self.store.path("resources", resource)
        marker = self.store.path("active", resource)
        sync = files.sync_directory

        def fail_after_rename(directory: Path) -> None:
            if directory == authority.parent:
                raise OSError("resource rename not settled")
            sync(directory)

        with (
            patch.object(files, "sync_directory", side_effect=fail_after_rename),
            self.assertRaisesRegex(OSError, "resource rename not settled"),
            self.store.edit(resource, settle_if_unchanged=True) as entry,
        ):
            entry.facts["terminal_identity"] = "b" * 64
        self.assertEqual(files.read(authority)["facts"]["terminal_identity"], "b" * 64)
        inode = authority.stat().st_ino
        synced = []
        fsync = files.os.fsync

        def observe(descriptor: int) -> None:
            synced.append(Path(f"/proc/self/fd/{descriptor}").readlink())
            fsync(descriptor)

        with (
            patch.object(files, "publish", wraps=files.publish) as publish,
            patch.object(files.os, "fsync", side_effect=observe),
            self.store.edit(resource, settle_if_unchanged=True),
        ):
            pass
        publish.assert_not_called()
        self.assertEqual(authority.stat().st_ino, inode)
        self.assertTrue(
            {authority, authority.parent, marker, marker.parent, self.root}
            <= set(synced)
        )
        with (
            patch.object(files.os, "fsync", side_effect=OSError("settlement failed")),
            self.assertRaisesRegex(OSError, "settlement failed"),
            self.store.edit(resource, settle_if_unchanged=True),
        ):
            pass

    def test_unchanged_projection_repairs_missing_or_stale_access_marker(self) -> None:
        resource = self.register()
        marker = self.store.path("active", resource)
        for missing in (True, False):
            with self.subTest(missing=missing):
                if missing:
                    marker.unlink()
                else:
                    files.publish(marker, {"resource": "b" * 32})
                with (
                    patch.object(files, "publish", wraps=files.publish) as publish,
                    self.store.edit(resource, settle_if_unchanged=True),
                ):
                    pass
                self.assertIn(marker, [call.args[0] for call in publish.call_args_list])
                self.assertEqual(files.read(marker), {"resource": resource})
        with self.store.edit(resource) as entry:
            entry.phase = "disposed"
        with (
            patch.object(files, "publish", wraps=files.publish) as publish,
            patch.object(files, "sync_directory", wraps=files.sync_directory) as sync,
            self.store.edit(resource, settle_if_unchanged=True),
        ):
            pass
        publish.assert_not_called()
        self.assertFalse(marker.exists())
        self.assertIn(marker.parent, [call.args[0] for call in sync.call_args_list])

    def test_two_actual_fresh_first_openers_share_initialization(self) -> None:
        fresh = self.root.parent / "concurrent-fresh"
        ready, release = self.root.parent / "initializing", self.root.parent / "release"
        script = (
            "import sys,time; from pathlib import Path\n"
            "from scripts import resource_store as files\n"
            "root,ready,release=map(Path,sys.argv[1:4]); publish=files.publish\n"
            "def delayed(path,value):\n"
            "    if path.name=='activation.json':\n"
            "        ready.write_text('initializing')\n"
            "        deadline=time.monotonic()+3\n"
            "        while not release.exists() and time.monotonic()<deadline: time.sleep(.01)\n"
            "    publish(path,value)\n"
            "files.publish=delayed\n"
            "files.FileStore(root).create_invocation({'nonce':'a'*32})"
        )
        first = subprocess.Popen(
            [sys.executable, "-c", script, str(fresh), str(ready), str(release)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        second = None
        try:
            deadline = time.monotonic() + 2
            while not ready.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(ready.exists())
            script = "from pathlib import Path; from scripts.resource_store import FileStore; import sys; FileStore(Path(sys.argv[1])).create_invocation({'nonce':'b'*32})"
            second = subprocess.Popen(
                [sys.executable, "-c", script, str(fresh)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            time.sleep(0.1)
            self.assertIsNone(second.poll())
            release.write_text("release")
            self.assertEqual(first.communicate(timeout=3)[1], "")
            self.assertEqual(second.communicate(timeout=3)[1], "")
            self.assertEqual((first.returncode, second.returncode), (0, 0))
        finally:
            release.write_text("release")
            for child in (first, second):
                if child is not None and child.poll() is None:
                    child.kill()
                    child.communicate(timeout=3)

    def test_registration_first_blocks_freeze_until_registered(self) -> None:
        entered, release, frozen = (
            threading.Event(),
            threading.Event(),
            threading.Event(),
        )
        published = files.publish
        result, errors = [], []

        def pause(path: Path, value: dict) -> None:
            published(path, value)
            if path.parent.name == "resources":
                entered.set()
                if not release.wait(2):
                    raise TimeoutError("test release")

        def register() -> None:
            try:
                result.append(self.register())
            except Exception as error:
                errors.append(error)

        def freeze() -> None:
            try:
                result.append(self.store.freeze(self.nonce, {"exit": 143}))
                frozen.set()
            except Exception as error:
                errors.append(error)

        with patch.object(files, "publish", side_effect=pause):
            first = threading.Thread(target=register)
            second = threading.Thread(target=freeze)
            first.start()
            self.assertTrue(entered.wait(2))
            second.start()
            self.assertFalse(frozen.wait(0.05))
            release.set()
            first.join(2)
            second.join(2)
        self.assertEqual(errors, [])
        self.assertEqual(result[1], [result[0]])
        self.assertTrue(self.store.lookup(result[0]).facts["pin"])

    def test_interrupted_registration_pending_recovers_exact_authority(self) -> None:
        publish = files.publish

        def fail_index(path: Path, value: dict) -> None:
            if path.parent.name == "identities":
                raise OSError("injected index publication failure")
            publish(path, value)

        with (
            patch.object(files, "publish", side_effect=fail_index),
            self.assertRaises(OSError),
        ):
            self.register()
        state = files.read(self.store.path("invocation-state", self.nonce))
        resource = next(iter(state["members"]))
        self.assertEqual(state["members"][resource]["phase"], "pending")
        self.assertEqual(self.store.freeze(self.nonce, {"exit": None}), [resource])
        self.assertEqual(self.store.identity_lookup(self.identity), resource)
        self.assertFalse(
            self.store.path("identity-pending", files.digest(self.identity)).exists()
        )

    def test_uncertain_absence_refuses_but_proven_never_publication_rolls_back(
        self,
    ) -> None:
        resource = "b" * 32
        with self.store.invocation_edit(self.nonce) as state:
            state["members"][resource] = {
                "phase": "pending",
                "identity": self.identity,
                "publication": "uncertain",
            }
        with self.assertRaisesRegex(files.StoreError, "Uncertain"):
            self.store.freeze(self.nonce, {})
        with self.store.invocation_edit(self.nonce) as state:
            self.assertEqual(state["phase"], "open")
            state["members"][resource]["publication"] = "not-started"
        self.assertEqual(self.store.freeze(self.nonce, {}), [])

    def test_bare_interrupted_publication_cannot_recreate_owner(self) -> None:
        publish = files.publish

        def fail_authority(path: Path, value: dict) -> None:
            if path.parent.name == "resources":
                raise OSError("write interrupted")
            publish(path, value)

        with (
            patch.object(files, "publish", side_effect=fail_authority),
            self.assertRaises(OSError),
        ):
            self.store.register(self.identity, {}, invocation=None)
        with self.assertRaisesRegex(files.StoreError, "Uncertain"):
            self.store.identity_lookup(self.identity)
        with self.assertRaisesRegex(files.StoreError, "already registered"):
            self.store.register(self.identity, {}, invocation=None)
        self.assertEqual(len(self.store.active_ids()), 1)

    def test_stable_resource_lock_and_exact_cold_lookup_ignore_history(self) -> None:
        resource = self.register()
        lock = self.root / "locks/resources" / resource
        inode = lock.stat().st_ino
        with self.store.edit(resource) as entry:
            entry.phase = "cold"
        with patch.object(
            Path, "iterdir", side_effect=AssertionError("history enumerated")
        ):
            self.assertEqual(self.store.lookup(resource).phase, "cold")
            self.assertEqual(self.store.identity_lookup(self.identity), resource)
        self.assertEqual(self.store.active_ids(), [])
        with self.store.edit(resource) as entry:
            entry.phase = "disposed"
        self.assertEqual(lock.stat().st_ino, inode)
        self.assertEqual(self.store.lookup(resource).phase, "disposed")

    def test_original_deadline_is_not_renewed_after_invocation_wait(self) -> None:
        release, entered = threading.Event(), threading.Event()

        def hold() -> None:
            with self.store.lock("invocations", self.nonce, time.monotonic() + 2):
                entered.set()
                release.wait(2)

        thread = threading.Thread(target=hold)
        thread.start()
        self.assertTrue(entered.wait(2))
        started = time.monotonic()
        try:
            with self.assertRaises(host.AdmissionError):
                self.store.register(
                    self.identity, {}, invocation=self.nonce, deadline=started + 0.08
                )
            self.assertLess(time.monotonic() - started, 0.5)
        finally:
            release.set()
            thread.join(2)

    def test_publication_replace_failure_preserves_previous_authority(self) -> None:
        resource = self.register()
        with (
            patch.object(files.os, "replace", side_effect=OSError("replace failure")),
            self.assertRaises(OSError),
            self.store.edit(resource) as entry,
        ):
            entry.facts["pin"] = False
        self.assertTrue(self.store.lookup(resource).facts["pin"])
        self.assertFalse(list((self.root / "resources").glob("*.tmp")))

    def test_publication_fsync_failure_preserves_previous_authority(self) -> None:
        resource = self.register()
        with (
            patch.object(files.os, "fsync", side_effect=OSError("fsync failure")),
            self.assertRaises(OSError),
            self.store.edit(resource) as entry,
        ):
            entry.facts["pin"] = False
        self.assertTrue(self.store.lookup(resource).facts["pin"])

    def test_existing_marker_retry_settles_directory_before_exposing_authority(
        self,
    ) -> None:
        entry = files.Entry("d" * 32, ["/state", "db", "embedded"], {})
        synchronize = files.sync_directory
        failed = False

        def interrupt(path: Path) -> None:
            nonlocal failed
            if (
                path == self.root / "active"
                and self.store.path("active", entry.resource).exists()
                and not failed
            ):
                failed = True
                raise OSError("injected marker directory sync")
            synchronize(path)

        with (
            patch.object(files, "sync_directory", side_effect=interrupt),
            self.assertRaisesRegex(OSError, "injected"),
        ):
            self.store._save(entry)  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
        self.assertFalse(self.store.path("resources", entry.resource).exists())
        events = []
        settle, publish = files.settle, files.publish

        def settle_existing(path: Path) -> None:
            events.append(("settle", path.parent.name))
            settle(path)

        def expose(path: Path, value: dict) -> None:
            events.append(("publish", path.parent.name))
            publish(path, value)

        with (
            patch.object(files, "settle", side_effect=settle_existing),
            patch.object(files, "publish", side_effect=expose),
        ):
            self.store._save(entry)  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
        self.assertLess(
            events.index(("settle", "active")), events.index(("publish", "resources"))
        )

    def test_interrupted_service_membership_never_exposes_unprotected_authority(
        self,
    ) -> None:
        resource = "d" * 32
        entry = files.Entry(
            resource,
            ["/owned/state", "fixture", "database"],
            {"kind": "database", "service_owner": "/owned/service"},
        )
        family = "service-access/" + files.digest("/owned/service")
        marker = self.store.path(family, resource)
        publish = files.publish

        def interrupt_arrival(path: Path, value: Mapping[str, object]) -> None:
            publish(path, value)
            if path == marker:
                raise OSError("service marker durable, authority unpublished")

        with (
            patch.object(files, "publish", side_effect=interrupt_arrival),
            self.assertRaisesRegex(OSError, "authority unpublished"),
        ):
            self.store._save(entry)  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
        self.assertEqual(self.store.access_ids(family), [resource])
        with self.assertRaises(FileNotFoundError):
            self.store.lookup(resource)

        # Retrying settles the existing protective path before authority appears.
        self.store._save(entry)  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
        self.assertEqual(self.store.lookup(resource), entry)
        unlink = Path.unlink

        def interrupt_departure(path: Path, missing_ok: bool = False) -> None:
            if path == marker:
                self.assertEqual(
                    self.store.resource_snapshot(resource).phase, "disposed"
                )
                raise OSError("authority departed, service marker retained")
            unlink(path, missing_ok=missing_ok)

        with (
            patch.object(Path, "unlink", interrupt_departure),
            self.assertRaisesRegex(OSError, "marker retained"),
            self.store.edit(resource) as selected,
        ):
            selected.phase = "disposed"
        self.assertEqual(self.store.lookup(resource).phase, "disposed")
        self.assertEqual(self.store.access_ids(family), [resource])

    def test_activation_without_fence_and_unknown_active_marker_refuse(self) -> None:
        files.publish(self.root / "allocations.json", {"version": 1, "owners": {}})
        with self.assertRaisesRegex(files.StoreError, "fence"):
            self.store.lookup("b" * 32)
        self.store = files.FileStore(self.root.parent / "fresh")
        self.store.ensure(time.monotonic() + 2)
        files.publish(self.store.root / "active/unknown.json", {})
        with self.assertRaisesRegex(files.StoreError, "Unknown active"):
            self.store.active_ids()

    def legacy(self) -> tuple[files.FileStore, str, dict]:
        store = files.FileStore(self.root / "legacy")
        resource = "c" * 32
        facts = {
            "state": "/state",
            "database": "fresh",
            "kind": "embedded",
            "pin": True,
            "drained": False,
            "cleanup": "retained",
            "references": {"receipt": "original"},
            "borrows": {"live": {"pid": os.getpid()}},
            "opaque": {"preserved": True},
        }
        files.publish(
            store.root / "allocations.json", {"version": 1, "owners": {resource: facts}}
        )
        return store, resource, facts

    def test_migration_failure_before_fence_preserves_writable_legacy(self) -> None:
        store, resource, facts = self.legacy()
        publish = files.publish

        def fail_fence(path: Path, value: dict) -> None:
            if path.name == "allocations.json":
                raise OSError("fence failed")
            publish(path, value)

        with (
            patch.object(files, "publish", side_effect=fail_fence),
            self.assertRaises(OSError),
        ):
            store.migrate()
        with host.metadata(store.root) as legacy:
            self.assertEqual(legacy["owners"][resource], facts)
        self.assertFalse((store.root / "activation.json").exists())
        store.migrate()
        self.assertEqual(store.lookup(resource).facts, facts)

    def test_fenced_unactivated_recovery_refuses_waiting_legacy_writer(self) -> None:
        store, resource, facts = self.legacy()
        publish = files.publish
        waiting = store.root.parent / "waiting"
        child = None

        def stop_before_activation(path: Path, value: dict) -> None:
            nonlocal child
            if path.name == "allocations.json":
                # Start the actual old owner while the migration still holds .lock.
                code = "from pathlib import Path; from scripts import host_admission as h; import sys; Path(sys.argv[2]).write_text('waiting');\nwith h.metadata(Path(sys.argv[1])): pass\n"
                child = subprocess.Popen(
                    [sys.executable, "-c", code, str(store.root), str(waiting)],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                )
                deadline = time.monotonic() + 2
                while not waiting.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(waiting.exists())
                self.assertIsNone(child.poll())
            if path.name == "activation.json":
                raise OSError("activation interrupted")
            publish(path, value)

        with (
            patch.object(files, "publish", side_effect=stop_before_activation),
            self.assertRaises(OSError),
        ):
            store.migrate()
        self.assertIsNotNone(child)
        _stdout, stderr = child.communicate(timeout=3)
        self.assertNotEqual(child.returncode, 0, stderr)
        self.assertEqual(files.read(store.root / "allocations.json")["version"], 2)
        with self.assertRaisesRegex(files.StoreError, "explicit fenced"):
            store.lookup(resource)
        store.migrate()
        self.assertEqual(store.lookup(resource).facts, facts)
        with self.assertRaises(host.AdmissionError), host.metadata(store.root):
            self.fail("old writer accepted a fenced registry")

    def test_migration_contradictory_import_and_changed_fence_refuse(self) -> None:
        store, resource, _facts = self.legacy()
        store.migrate()
        (store.root / "activation.json").unlink()
        with store.lock("resources", resource, time.monotonic() + 2):
            entry = store._entry(resource)  # noqa: SLF001 -- inject corruption under the owner lock
            entry.facts["pin"] = False
            store._save(entry)  # noqa: SLF001 -- persist deliberately contradictory imported authority
        with self.assertRaisesRegex(files.StoreError, "differs"):
            store.migrate()
        self.assertEqual(files.read(store.root / "allocations.json")["version"], 2)

    def test_migration_replace_and_fsync_failures_preserve_fence_order(self) -> None:
        for operation in ("replace", "fsync"):
            for destination in ("allocations.json", "activation.json"):
                with self.subTest(operation=operation, destination=destination):
                    self.assert_migration_failure_case(operation, destination)

    def assert_migration_failure_case(self, operation: str, destination: str) -> None:
        self.root = self.root.parent / (operation + destination.replace(".", "-"))
        store, resource, facts = self.legacy()
        publish = files.publish

        def fail(path: Path, value: dict) -> None:
            if path.name == destination:
                with patch.object(
                    files.os,
                    operation,
                    side_effect=OSError("durability interrupted"),
                ):
                    publish(path, value)
            else:
                publish(path, value)

        with (
            patch.object(files, "publish", side_effect=fail),
            self.assertRaisesRegex(OSError, "durability interrupted"),
        ):
            store.migrate()
        self.assertFalse((store.root / "activation.json").exists())
        self.assertEqual(
            files.read(store.root / "allocations.json")["version"],
            1 if destination == "allocations.json" else 2,
        )
        store.migrate()
        self.assertEqual(store.lookup(resource).facts, facts)

    def test_normal_operations_do_not_read_imported_history_manifest(self) -> None:
        resource = self.register()
        read = files.read

        def exact(path: Path) -> dict:
            if path.name == "migration.json":
                raise AssertionError("historical manifest read on exact operation")
            return read(path)

        with patch.object(files, "read", side_effect=exact):
            self.assertEqual(self.store.lookup(resource).resource, resource)
            self.assertEqual(self.store.identity_lookup(self.identity), resource)
            self.assertEqual(self.store.active_ids(), [resource])

    def test_dangling_legacy_report_index_prevents_activation(self) -> None:
        store, _resource, _facts = self.legacy()
        directory = "/unknown/evidence"
        key = hashlib.sha256(directory.encode()).hexdigest()
        files.publish(
            store.root / "reports" / (key + ".json"),
            {"resource": "d" * 32, "state": directory},
        )
        with self.assertRaisesRegex(files.StoreError, "Dangling"):
            store.migrate()
        self.assertEqual(files.read(store.root / "allocations.json")["version"], 1)
        self.assertFalse((store.root / "activation.json").exists())

    def test_private_selector_rejects_relative_and_symlink_directory(self) -> None:
        with (
            patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": "relative"}),
            self.assertRaisesRegex(resources.ResourceError, "absolute"),
        ):
            resources.registry()
        link = self.root.parent / "link"
        link.symlink_to(self.root, target_is_directory=True)
        with (
            patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(link)}),
            self.assertRaises(host.AdmissionError),
        ):
            resources.storage().ensure(time.monotonic() + 2)


if __name__ == "__main__":
    unittest.main()
