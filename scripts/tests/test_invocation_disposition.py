# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""New private invocation controls cover abandonment without a forged terminal."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import invocation_disposition as disposition
from scripts import native_operation, resource_store, test_resources
from scripts.tests.test_test_resources import drain_fixture


class InvocationDispositionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for selected in (
            patch.object(test_resources, "registry", return_value=self.root / "registry"),
            patch.object(native_operation, "scope_owner", return_value=None),
            patch.object(native_operation, "drained", return_value=drain_fixture(True)),
        ):
            selected.start()
            self.addCleanup(selected.stop)
        self.path = test_resources.invocation("python", [{"name": "fixture::selected"}])
        self.store = test_resources.storage()
        self.dead = {"pid": 99999999, "start": "never", "boot": test_resources.host.boot(),
                     "scope": {"unit": "pse-native-" + "a" * 32 + ".scope", "group": "/private-control", "invocation": "b" * 32}}
        with self.store.invocation_edit(self.path.stem) as state:
            state["producer"] = self.dead
        self.facts: test_resources.ResourceRecord = {"pid": self.dead["pid"], "start": "never", "boot": self.dead["boot"],
                      "state": str(self.root), "database": "canonical_test_" + "a" * 32,
                      "namespace": "pse", "service_generation": "owned", "kind": "database",
                      "directory": None, "invocation": self.path.stem, "test": "fixture::selected",
                      "cleanup": "retained", "disposition": "incomplete", "pin": True, "drained": True,
                      "units": [self.dead["scope"]], "references": {}, "borrows": {}}
        self.resource = self.store.register(test_resources.resource_identity(self.facts), dict(self.facts), invocation=self.path.stem)

    def apply(self) -> dict:
        return disposition.apply(str(self.path), deadline=time.monotonic() + 5)

    def test_missing_terminal_fence_rejects_late_registration_and_freeze(self) -> None:
        with patch.object(test_resources, "reclaim", return_value="removed") as reclaim:
            result = self.apply()
        self.assertEqual(result["action"], "membership fenced")
        reclaim.assert_called_once()
        image = self.store.invocation_snapshot(self.path.stem)
        self.assertEqual(image["phase"], "abandoned")
        self.assertNotIn("terminal", image)
        self.assertNotIn("terminal_identity", image)
        self.assertNotIn("acknowledged", image)
        with self.assertRaisesRegex(ValueError, "frozen"):
            self.store.register(["another"], {}, invocation=self.path.stem)
        with self.assertRaisesRegex(ValueError, "abandoned"):
            self.store.freeze(self.path.stem, {})

    def test_unknown_creation_and_active_reference_are_fenced_but_never_released(self) -> None:
        updates: list[test_resources.ResourceRecord] = [
            {"creation_borrower": {"pid": 99999999, "start": "never"}}, {"references": {"report": "digest"}},
            {"borrows": {"active": {"pid": 99999999, "start": "never"}}}, {"manual_pin": True},
        ]
        for update in updates:
            with test_resources.resource_update(self.resource) as record:
                record.update(update)
            with patch.object(test_resources, "reclaim", side_effect=AssertionError("protected removal")):
                result = self.apply()
            self.assertTrue(result["resources"][0]["reason"])
            self.assertTrue(test_resources.resource_status(self.resource)["pin"])

    def test_live_composer_and_missing_physical_scope_block_before_fence(self) -> None:
        with self.store.invocation_edit(self.path.stem) as state:
            state["composer"] = {**self.dead, "pid": os.getpid(), "start": native_operation.start_identity(os.getpid())}
        result = self.apply()
        self.assertEqual(result["action"], "blocked")
        self.assertEqual(self.store.invocation_snapshot(self.path.stem)["phase"], "open")
        with self.store.invocation_edit(self.path.stem) as state:
            state.pop("composer")
            state["producer"]["scope"] = None
        self.assertEqual(self.apply()["action"], "blocked")

    def test_interrupted_projection_resumes_same_disposition(self) -> None:
        with patch.object(test_resources, "reclaim", side_effect=KeyboardInterrupt), self.assertRaises(KeyboardInterrupt):
            self.apply()
        identity = self.store.invocation_snapshot(self.path.stem)["abandonment"]["identity"]
        with patch.object(test_resources, "reclaim", return_value="removed"):
            self.apply()
        image = self.store.invocation_snapshot(self.path.stem)
        self.assertEqual(image["abandonment"]["identity"], identity)
        self.assertEqual(image["abandonment"]["resources"][self.resource], "removed")
        self.assertNotIn("terminal", image)

    def test_registration_change_requires_new_inspection_and_frozen_outcome_is_immutable(self) -> None:
        before = self.store.invocation_snapshot(self.path.stem)
        self.store.register(["new"], {}, invocation=self.path.stem)
        with self.assertRaisesRegex(ValueError, "changed since"):
            self.store.abandon(self.path.stem, before)
        # Use a fresh exact invocation to retain a genuine immutable terminal.
        path = test_resources.invocation("python", [])
        test_resources.finalize(path, {}, complete=False, terminal_owner="runner", terminal={"workload_exit": 130})
        with self.store.invocation_edit(path.stem) as state:
            state["producer"] = self.dead
        terminal = self.store.terminal(path.stem)
        disposition.apply(str(path), deadline=time.monotonic() + 5)
        self.assertEqual(self.store.terminal(path.stem), terminal)

    def test_pending_never_published_is_resolved_but_uncertain_publication_is_retained(self) -> None:
        with self.store.invocation_edit(self.path.stem) as state:
            state["members"]["f" * 32] = {"phase": "pending", "publication": "not-started", "identity": ["never"]}
        with patch.object(test_resources, "reclaim", return_value="removed"):
            self.apply()
        self.assertNotIn("f" * 32, self.store.invocation_snapshot(self.path.stem)["members"])

    def test_composer_cannot_register_after_missing_terminal_membership_closes(self) -> None:
        with patch.object(test_resources, "reclaim", return_value="removed"):
            self.apply()
        with self.assertRaisesRegex(ValueError, "membership is closed"):
            native_operation.register_composer(self.path)

    def test_host_guard_accepts_separate_abandonment_only_after_exact_composer_drain(self) -> None:
        scope = self.dead["scope"]
        with self.store.invocation_edit(self.path.stem) as state:
            state["composer"] = self.dead
        area = self.root / ".operations"
        area.mkdir(mode=0o700)
        operation = area / ("c" * 32 + ".json")
        token = "d" * 32
        record = {"version": native_operation.VERSION, "scope": scope,
                  "selected_composers": {self.path.stem: {**self.dead, "token": token,
                    "registry": str(self.root / "registry"), "invocation": str(self.path), "phase": "registered"}}}
        resource_store.publish(operation, record)
        guard = {"scope": scope, "token": token, "operation": str(operation), "phase": "registered"}
        self.assertFalse(native_operation.terminal_guard_complete(self.path.stem, guard))
        with patch.object(test_resources, "reclaim", return_value="removed"):
            self.apply()
        self.assertTrue(native_operation.terminal_guard_complete(self.path.stem, guard))
        self.assertEqual(resource_store.read(operation), record)
        self.assertEqual(guard["phase"], "registered")
        with patch.object(native_operation, "drained", return_value=drain_fixture(False)):
            self.assertFalse(native_operation.terminal_guard_complete(self.path.stem, guard))
        with self.store.invocation_edit(self.path.stem) as state:
            state["composer"]["start"] = "changed"
        self.assertFalse(native_operation.terminal_guard_complete(self.path.stem, guard))

    def test_generation_change_and_uncertain_pending_are_retained(self) -> None:
        with self.store.invocation_edit(self.path.stem) as state:
            state["members"]["e" * 32] = {"phase": "pending", "publication": "uncertain", "identity": ["uncertain"]}
        with self.assertRaisesRegex(ValueError, "Uncertain resource publication"):
            self.apply()
        self.assertEqual(self.store.invocation_snapshot(self.path.stem)["phase"], "open")
        with self.store.invocation_edit(self.path.stem) as state:
            state["members"].pop("e" * 32)
        config = {"instance_id": "changed", "namespace": "pse", "admission": "open", "accepting_writes": True}
        with (
            patch("scripts.surreal_server.config_for", return_value=config),
            patch("scripts.surreal_server.context_admission", side_effect=lambda *_args, **_kwargs: contextlib.nullcontext()),
            patch.object(test_resources, "remove_owned", side_effect=AssertionError("changed-generation removal")),
        ):
            result = self.apply()
        self.assertIn("ownership changed", result["resources"][0]["disposition_result"])
        self.assertEqual(test_resources.resource_status(self.resource)["cleanup"], "retained")

    def test_bounded_exact_report_bytes_do_not_double_count_links_or_follow_symlinks(self) -> None:
        directory = self.root / "build" / "evidence"
        directory.mkdir(parents=True)
        payload = directory / "payload"
        payload.write_bytes(b"x" * 8192)
        os.link(payload, directory / "same-inode")
        (directory / "outside").symlink_to(self.root / "not-owned")
        (self.root / "not-owned").write_bytes(b"y" * 65536)
        identity = test_resources.register_report(directory, root=self.root)
        record = test_resources.resource_status(identity)
        observed = disposition.storage_observation(record, deadline=time.monotonic() + 5, seen=set())
        self.assertEqual(observed["status"], "partial")
        self.assertEqual(observed["logical_bytes"], 8192)
        self.assertEqual(observed["exclusive_allocated_bytes"], 0)
        self.assertGreater(observed["shared_allocated_bytes"], 0)
        bounded = disposition.storage_observation(record, deadline=time.monotonic() + 5, seen=set(), limit=1)
        self.assertEqual(bounded["status"], "partial")
        self.assertEqual(bounded["entries"], 1)
        unsupported = disposition.storage_observation(self.facts, deadline=time.monotonic() + 5, seen=set())
        self.assertEqual(unsupported["status"], "unavailable")
        self.assertIn("shared database", unsupported["reason"])


class ActualInvocationDispositionTests(unittest.TestCase):
    def test_real_dead_native_composer_abandons_without_terminal_or_ack_and_releases_guard(self) -> None:
        root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary)
            environment = {key: value for key, value in os.environ.items() if key not in {
                native_operation.MARKER, "PSE_NATIVE_HANDOFF", test_resources.host.MARKER,
                "PSE_ADMISSION_DEADLINE", "PSE_MEMORY_MAX", "PSE_TEST_INVOCATION"}}
            environment.update(PSE_TEST_RESOURCE_STATE=str(fixture / "registry"),
                               XDG_STATE_HOME=str(fixture / "state"), PSE_NATIVE_CACHE=str(fixture / "native"),
                               PSE_NATIVE_CAPABILITIES="", PSE_RESOURCE_CLASS="light", PSE_MEMORY_MAX="512M")
            script = "from scripts import test_resources as r, native_operation as n; p=r.invocation('python', [{'name':'fixture::not_run'}]); n.register_composer(p); print(p, flush=True)"
            child = subprocess.run([str(root / "scripts/pse-env"), "--native=", "--resource-class", "light", "--", sys.executable, "-c", script],
                                   env=environment, cwd=root, capture_output=True, text=True, timeout=30, check=False)
            self.assertIn(child.returncode, (0, 125), child.stderr)
            paths = list((fixture / "registry/invocations").glob("*.json"))
            self.assertEqual(len(paths), 1, child.stderr)
            invocation = paths[0]
            with patch.dict(os.environ, environment, clear=True):
                before = test_resources.storage().invocation_snapshot(invocation.stem)
                self.assertNotIn("terminal", before)
                result = disposition.apply(str(invocation), deadline=time.monotonic() + 10)
                self.assertEqual(result["action"], "membership fenced", result)
                self.assertIn(result["capacity_release"], ("released", "already absent"), result)
                after = test_resources.storage().invocation_snapshot(invocation.stem)
                self.assertNotIn("terminal", after)
                self.assertNotIn("terminal_identity", after)
                operation = resource_store.read(Path(before["producer"]["operation"]))
                self.assertEqual(operation["selected_composers"][invocation.stem]["phase"], "registered")


if __name__ == "__main__":
    unittest.main()
