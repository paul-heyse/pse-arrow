# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit writer fencing, original custody and interrupted format recovery."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import os
import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import resource_conversion as conversion
from scripts import resource_store as files
from scripts import test_requirements, test_resources


class ResourceConversionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "registry"
        files.protected(self.root)
        self.enterContext(
            patch.object(
                conversion.host, "readonly_snapshot", return_value={"owners": {}}
            )
        )
        self.enterContext(patch.object(conversion.host, "inherit", return_value=None))
        self.processes = self.enterContext(
            patch.object(conversion, "processes", return_value=[])
        )
        self.nonce = "a" * 32
        self.header = {
            "version": 1,
            "nonce": self.nonce,
            "kind": "python",
            "selected": [{"name": "test::one"}],
            "terminal_owner": "runner",
        }
        self.terminal = {
            "selection": files.digest(self.header["selected"]),
            "complete": True,
            "outcomes": {"test::one": "pass"},
            "evidence": {"original": True},
        }
        self.state = {
            "version": 2,
            "declaration": self.header,
            "requirements": {"test::one": test_requirements.LOCAL.document()},
            "phase": "frozen",
            "members": {},
            "terminal": self.terminal,
            "terminal_identity": files.digest(self.terminal),
        }
        files.publish(
            self.root / "activation.json",
            {"version": 2, "format": "exact-resource-files", "migration": "f" * 64},
        )
        files.publish(
            self.root / "allocations.json",
            {"version": 2, "format": "legacy-refusal", "migration": "f" * 64},
        )
        files.publish(self.root / "invocations" / (self.nonce + ".json"), self.header)
        files.publish(
            self.root / "invocation-state" / (self.nonce + ".json"), self.state
        )

    def convert(self) -> dict:
        return conversion.convert(self.root, deadline=time.monotonic() + 10)

    def pending(self) -> dict:
        transition = {
            "version": 1,
            "phase": "pending",
            "original_activation": files.read(self.root / "activation.json"),
            "services": [],
            "sources": {},
        }
        for family in ("invocations", "invocation-state"):
            name = family + "/" + self.nonce + ".json"
            data = (self.root / name).read_bytes()
            conversion.archive(self.root / "original-v2" / name, data)
            transition["sources"][name] = hashlib.sha256(data).hexdigest()
        files.publish(self.root / "format-transition.json", transition)
        files.publish(
            self.root / "activation.json",
            {"version": 0, "format": "conversion-pending"},
        )
        return transition

    def test_recovery_source_authority_rejects_traversal_unknown_family_and_invalid_digest(
        self,
    ) -> None:
        transition = self.pending()
        for name, digest in (
            ("../activation.json", "a" * 64),
            ("other/" + self.nonce + ".json", "a" * 64),
            ("invocations/" + self.nonce.upper() + ".json", "a" * 64),
            ("invocations/" + self.nonce + ".json", "bad"),
        ):
            with self.subTest(name=name, digest=digest):
                files.publish(
                    self.root / "format-transition.json",
                    {**transition, "sources": {name: digest}},
                )
                with self.assertRaisesRegex(files.StoreError, "source authority"):
                    self.convert()
                self.assertEqual(
                    files.read(self.root / "activation.json")["version"], 0
                )
        for sources in (None, [], "invalid"):
            with self.subTest(sources=sources):
                files.publish(
                    self.root / "format-transition.json",
                    {**transition, "sources": sources},
                )
                with self.assertRaisesRegex(files.StoreError, "source authority"):
                    self.convert()

    def test_recovery_authenticates_digest_and_header_before_any_current_rewrite(
        self,
    ) -> None:
        transition = self.pending()
        path = self.root / "original-v2/invocations" / (self.nonce + ".json")
        files.publish(path, {**self.header, "selected": []})
        with self.assertRaisesRegex(files.StoreError, "digest mismatch"):
            self.convert()
        name = "invocations/" + self.nonce + ".json"
        transition["sources"][name] = hashlib.sha256(path.read_bytes()).hexdigest()
        files.publish(self.root / "format-transition.json", transition)
        with self.assertRaisesRegex(files.StoreError, "conflict"):
            self.convert()
        self.assertEqual(
            files.read(self.root / "invocation-state" / (self.nonce + ".json")),
            self.state,
        )

    def test_archived_original_leaf_parent_and_private_permissions_are_enforced(
        self,
    ) -> None:
        self.pending()
        path = self.root / "original-v2/invocations" / (self.nonce + ".json")
        original = path.read_bytes()
        outside = self.root.parent / "outside.json"
        outside.write_bytes(original)
        outside.chmod(0o600)
        path.unlink()
        path.symlink_to(outside)
        with self.assertRaises(OSError):
            self.convert()
        with self.assertRaises(OSError):
            conversion.original(self.root, self.nonce)
        with self.assertRaises(OSError):
            conversion.archive(path, original)
        path.unlink()
        path.write_bytes(original)
        path.chmod(0o644)
        with self.assertRaisesRegex(conversion.host.AdmissionError, "Unsafe"):
            conversion.original(self.root, self.nonce)
        path.chmod(0o600)
        parent = path.parent
        retained = parent.with_name("retained")
        parent.rename(retained)
        parent.symlink_to(retained, target_is_directory=True)
        with self.assertRaises(OSError):
            self.convert()
        with self.assertRaises(OSError):
            conversion.original(self.root, self.nonce)
        self.assertEqual(outside.read_bytes(), original)

    def test_original_input_family_symlink_refuses_before_pending_fence(self) -> None:
        parent = self.root / "invocations"
        retained = parent.with_name("retained")
        parent.rename(retained)
        parent.symlink_to(retained, target_is_directory=True)
        with self.assertRaises(OSError):
            self.convert()
        self.assertFalse((self.root / "format-transition.json").exists())

    def test_registry_parent_symlink_refuses_without_creating_lock_or_fence(
        self,
    ) -> None:
        alias = self.root.parent / "alias"
        alias.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(OSError):
            conversion.convert(alias, deadline=time.monotonic() + 5)
        self.assertFalse((self.root / ".lock").exists())
        self.assertFalse((self.root / "format-transition.json").exists())

    def test_service_symlink_refuses_before_pending_fence(self) -> None:
        service = self.root.parent / "service"
        files.protected(service)
        alias = self.root.parent / "alias"
        alias.symlink_to(service, target_is_directory=True)
        with self.assertRaisesRegex(files.StoreError, "service authority"):
            conversion.convert(
                self.root, deadline=time.monotonic() + 5, services=[alias]
            )
        self.assertFalse((self.root / "format-transition.json").exists())

    def test_original_migration_identity_and_legacy_fence_must_agree_before_pending(
        self,
    ) -> None:
        activation = files.read(self.root / "activation.json")
        files.publish(self.root / "activation.json", {**activation, "migration": "bad"})
        with self.assertRaisesRegex(files.StoreError, "recovery authority"):
            self.convert()
        self.assertFalse((self.root / "format-transition.json").exists())
        files.publish(self.root / "activation.json", activation)
        files.publish(
            self.root / "allocations.json",
            {"version": 2, "format": "legacy-refusal", "migration": "e" * 64},
        )
        with self.assertRaisesRegex(files.StoreError, "durable legacy-writer fence"):
            self.convert()
        self.assertFalse((self.root / "format-transition.json").exists())

    def test_recovery_and_initial_capture_are_bounded_before_reading_payloads(
        self,
    ) -> None:
        with (
            patch.object(conversion, "MAX_BYTES", 1),
            self.assertRaisesRegex(files.StoreError, "byte bound"),
        ):
            self.convert()
        self.assertFalse((self.root / "format-transition.json").exists())
        self.pending()
        with (
            patch.object(conversion, "MAX_BYTES", 1),
            self.assertRaisesRegex(files.StoreError, "byte bound"),
        ):
            self.convert()
        with (
            patch.object(conversion, "MAX_IMAGES", 1),
            self.assertRaisesRegex(files.StoreError, "source authority"),
        ):
            self.convert()

    def test_idle_capability_server_does_not_block_registry_conversion(self) -> None:
        # Endpoint identity is unrelated to acquired registry ownership. Ordinary
        # conversion must not census arbitrary live servers or require restarts.
        self.processes.side_effect = AssertionError("unrelated process census")
        self.assertEqual(self.convert()["version"], 3)
        self.processes.assert_not_called()

    def test_own_allocation_exemption_refuses_other_live_scope_children(self) -> None:
        self.processes.return_value = [
            {"pid": os.getpid(), "start": "self", "argv": ["python", "convert.py"]},
            {"pid": 123, "start": "456", "argv": ["python", "child.py"]},
        ]
        owner = {"units": {"self.scope": {"group": "/self"}}}
        with (
            patch.object(
                conversion.host,
                "readonly_snapshot",
                return_value={"owners": {"nonce": owner}},
            ),
            patch.object(
                conversion.host, "inherit", return_value=SimpleNamespace(nonce="nonce")
            ),
            patch.object(
                conversion.operation,
                "process_group",
                side_effect=lambda pid: "/self/child" if pid == 123 else "/self",
            ),
            patch.object(conversion.operation, "start_identity", return_value="456"),
        ):
            with self.assertRaisesRegex(
                files.StoreError, "Other live conversion scope member pid=123"
            ):
                self.convert()
            self.assertFalse((self.root / "format-transition.json").exists())
            self.processes.return_value.pop()
            self.assertEqual(self.convert()["version"], 3)

    def test_convert_preserves_original_bytes_terminal_digest_and_private_locator(
        self,
    ) -> None:
        source = (self.root / "invocation-state" / (self.nonce + ".json")).read_bytes()
        self.convert()
        store = files.FileStore(self.root)
        self.assertEqual(
            store.terminal(self.nonce), (files.digest(self.terminal), self.terminal)
        )
        self.assertEqual(
            (
                self.root / "original-v2/invocation-state" / (self.nonce + ".json")
            ).read_bytes(),
            source,
        )
        view = store.invocation_snapshot(self.nonce)
        self.assertEqual(view["requirements"], self.state["requirements"])
        compact = files.read(store.path("invocation-state", self.nonce))
        self.assertNotIn("declaration", compact)
        self.assertNotIn("requirements", compact)
        self.assertNotIn("terminal", compact)
        self.assertEqual(
            test_resources.read_invocation(store.path("invocations", self.nonce))[
                "selected"
            ],
            self.header["selected"],
        )
        original = conversion.original(self.root, self.nonce)
        self.assertFalse(original["automatic_reuse"])
        self.assertEqual(original["invocation-state"], self.state)
        self.assertEqual(self.convert()["version"], 3)

    def test_live_writer_and_contradictory_header_refuse_before_fence(self) -> None:
        self.state["producer"] = {
            "pid": os.getpid(),
            "boot": conversion.host.boot(),
            "start": conversion.operation.start_identity(os.getpid()),
        }
        files.publish(
            self.root / "invocation-state" / (self.nonce + ".json"), self.state
        )
        with self.assertRaisesRegex(files.StoreError, "Live exact"):
            self.convert()
        self.assertFalse((self.root / "format-transition.json").exists())
        self.state.pop("producer")
        files.publish(
            self.root / "invocation-state" / (self.nonce + ".json"), self.state
        )
        files.publish(
            self.root / "invocations" / (self.nonce + ".json"),
            {**self.header, "selected": []},
        )
        with self.assertRaisesRegex(files.StoreError, "conflict"):
            self.convert()
        self.assertFalse((self.root / "format-transition.json").exists())

    def test_interrupted_activation_fences_both_layouts_and_recovers_from_originals(
        self,
    ) -> None:
        publish = files.publish

        def interrupt(path: Path, value: dict) -> None:
            if path == self.root / "activation.json" and value.get("version") == 3:
                raise OSError("injected activation interruption")
            publish(path, value)

        with (
            patch.object(files, "publish", side_effect=interrupt),
            self.assertRaisesRegex(OSError, "injected"),
        ):
            self.convert()
        with self.assertRaisesRegex(files.StoreError, "fenced"):
            files.FileStore(self.root).ensure(time.monotonic() + 1)
        self.assertNotEqual(files.read(self.root / "activation.json")["version"], 2)
        self.convert()
        self.assertEqual(
            files.FileStore(self.root).terminal(self.nonce)[0],
            files.digest(self.terminal),
        )

    def test_missing_original_requirements_are_retained_without_inventing_today_policy(
        self,
    ) -> None:
        self.state.pop("requirements")
        files.publish(
            self.root / "invocation-state" / (self.nonce + ".json"), self.state
        )
        self.convert()
        view = files.FileStore(self.root).invocation_snapshot(self.nonce)
        self.assertEqual(view["requirements"], {})
        self.assertFalse(view["collected"])
        self.assertIn("unqualified", view["original_basis"])

    def test_nonterminal_original_cannot_reopen_for_late_effects(self) -> None:
        self.state.update(phase="open")
        self.state.pop("terminal")
        self.state.pop("terminal_identity")
        files.publish(
            self.root / "invocation-state" / (self.nonce + ".json"), self.state
        )
        self.convert()
        self.assertEqual(
            files.FileStore(self.root).invocation_snapshot(self.nonce)["phase"],
            "unresolved",
        )
        self.assertEqual(
            conversion.original(self.root, self.nonce)["invocation-state"]["phase"],
            "open",
        )

    def test_retired_receiver_is_uncovered_and_old_known_key_writer_cannot_reenter(
        self,
    ) -> None:
        service = self.root.parent / "service"
        files.protected(service)
        resource = "b" * 32
        state = service / ".receivers" / "retired"
        facts = {
            "state": str(state),
            "kind": "database",
            "database": "canonical_test_" + "b" * 32,
            "service_generation": "old",
            "cleanup": "retained",
            "disposition": "incomplete",
            "pin": True,
            "drained": False,
            "references": {},
            "borrows": {},
            "units": [],
        }
        files.publish(
            self.root / "resources" / (resource + ".json"),
            {
                "version": 2,
                "resource": resource,
                "phase": "active",
                "identity": test_resources.resource_identity(facts),
                "facts": facts,
            },
        )
        conversion.convert(
            self.root, deadline=time.monotonic() + 10, services=[service]
        )
        entry = files.FileStore(self.root).lookup(resource)
        self.assertNotIn("service_owner", entry.facts)
        self.assertEqual(entry.facts["unresolved_service"], str(service))
        # An old reader that passed ensure before the fence still acquires the
        # same exact key exclusion; its v2 entry decoder refuses after conversion.
        with files.FileStore(self.root).lock(
            "resources", resource, time.monotonic() + 1
        ):
            self.assertNotEqual(
                files.read(self.root / "resources" / (resource + ".json"))["version"], 2
            )
        with (
            patch.object(test_resources, "registry", return_value=self.root),
            self.assertRaisesRegex(files.StoreError, "unresolved"),
        ):
            test_resources.service_resources(service)


class ProcessCohortTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.proc = Path(temporary.name)

    def process(
        self, pid: int, command: list[str], *, state: str = "S", start: str = "123"
    ) -> Path:
        path = self.proc / str(pid)
        path.mkdir()
        (path / "stat").write_text(
            f"{pid} (python worker) " + " ".join([state, *(["0"] * 18), start])
        )
        (path / "cmdline").write_bytes(
            b"\0".join(os.fsencode(item) for item in command)
            + (b"\0" if command else b"")
        )
        return path

    def test_proc_discovery_retains_live_exact_identity_and_ignores_dead_processes(
        self,
    ) -> None:
        self.process(123, ["/venv/python", "-m", "scripts.capabilities"])
        self.process(124, [], state="Z")
        self.assertEqual(
            conversion.processes(time.monotonic() + 5, self.proc),
            [
                {
                    "pid": 123,
                    "start": "123",
                    "argv": ["/venv/python", "-m", "scripts.capabilities"],
                }
            ],
        )

    def test_proc_discovery_refuses_empty_live_command_and_unreadable_peer(
        self,
    ) -> None:
        path = self.process(123, [])
        with self.assertRaisesRegex(
            files.StoreError, "Unresolved live process command"
        ):
            conversion.processes(time.monotonic() + 5, self.proc)
        (path / "cmdline").write_bytes(b"python\0")
        read_text = Path.read_text

        def unreadable(path: Path, *args, **kwargs):
            if path.name == "stat":
                raise PermissionError("injected unreadable live peer")
            return read_text(path, *args, **kwargs)

        with (
            patch.object(Path, "read_text", unreadable),
            self.assertRaisesRegex(files.StoreError, "Unresolved live process"),
        ):
            conversion.processes(time.monotonic() + 5, self.proc)

    def test_missing_command_of_still_live_peer_cannot_be_treated_as_departure(
        self,
    ) -> None:
        path = self.process(123, ["python", "-m", "scripts.capabilities"])
        (path / "cmdline").unlink()
        with self.assertRaisesRegex(files.StoreError, "Unresolved live process"):
            conversion.processes(time.monotonic() + 5, self.proc)

    def test_proc_discovery_refuses_pid_reuse_between_command_observations(
        self,
    ) -> None:
        self.process(123, ["python", "-m", "scripts.capabilities"])
        before = "123 (python) " + " ".join(["S", *(["0"] * 18), "1"])
        after = "123 (python) " + " ".join(["S", *(["0"] * 18), "2"])
        with (
            patch.object(Path, "read_text", side_effect=[before, after]),
            self.assertRaisesRegex(files.StoreError, "identity changed"),
        ):
            conversion.processes(time.monotonic() + 5, self.proc)
