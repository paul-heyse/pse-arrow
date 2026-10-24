# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Endpoint-local preview retention, admitted work and explicit settlement controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import asyncio
import json
import tempfile
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Event, Lock
from unittest.mock import patch

from fastmcp import Client, FastMCP

from scripts import semantic_edits
from scripts.capabilities import specialists


def availability(
    owner: specialists.Specialists, digest: str
) -> specialists.PreviewAvailability:
    selected = owner.preview_status(digest).selected
    if selected is None:
        raise AssertionError("expected selected preview availability")
    return selected


class PreviewLifecycleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "sample.py").write_text("x=1\n")
        self.cut = semantic_edits.capture(self.root, ["sample.py"])

    def preview(self, value: int) -> dict:
        return semantic_edits.replacements(
            self.root, self.cut, {"sample.py": f"x={value}\n"}, producer="test"
        )

    def test_invalid_utf8_current_bytes_are_classified_before_release(self) -> None:
        (self.root / "basis.py").write_text("basis=1\n")
        owner = specialists.Specialists(self.root)
        cut = semantic_edits.capture(self.root, ["sample.py", "basis.py"])
        preview = semantic_edits.replacements(
            self.root, cut, {"sample.py": "x=2\n"}, producer="test"
        )
        handle = owner.retain_preview(preview)["preview_sha256"]
        with (
            patch.object(semantic_edits, "_write", side_effect=OSError("interrupted")),
            self.assertRaises(OSError),
        ):
            owner.apply_preview(handle, "receipt.json")
        (self.root / "sample.py").write_bytes(b"\xffedited")
        (self.root / "basis.py").write_bytes(b"\xfebasis")
        result = owner.reconcile_preview(handle)
        self.assertFalse(result["complete"])
        self.assertEqual(result["files"][0]["state"], "diverged")
        self.assertEqual(result["basis"][0]["state"], "diverged")
        self.assertFalse(availability(owner, handle).reconciliation_required)
        self.assertEqual(owner.release_preview(handle).status, "released")

    def test_small_count_capacity_release_reclaims_without_eviction_or_sibling_loss(
        self,
    ) -> None:
        owner = specialists.Specialists(self.root, preview_capacity=2)
        first, sibling, replacement = (self.preview(value) for value in (2, 3, 4))
        handle = owner.retain_preview(first)["preview_sha256"]
        sibling_handle = owner.retain_preview(sibling)["preview_sha256"]
        before = owner.preview_status(handle)
        self.assertEqual(before.retained_count, 2)
        self.assertEqual(before.count_capacity, 2)
        self.assertEqual(before.byte_capacity, 64 << 20)
        self.assertTrue(availability(owner, handle).available)
        self.assertTrue(availability(owner, handle).releasable)
        with self.assertRaises(specialists.PreviewCapacityError):
            owner.retain_preview(replacement)
        self.assertEqual(owner.retained_preview(handle), first)
        released = owner.release_preview(handle)
        self.assertEqual(released.status, "released")
        self.assertEqual(released.retained_count, 1)
        self.assertEqual(
            released.retained_bytes, len(json.dumps(sibling, sort_keys=True).encode())
        )
        self.assertEqual(owner.retained_preview(sibling_handle), sibling)
        self.assertFalse(availability(owner, handle).available)
        with self.assertRaises(ValueError):
            owner.retained_preview(handle)
        new_handle = owner.retain_preview(replacement)["preview_sha256"]
        self.assertEqual(owner.retained_preview(new_handle), replacement)
        self.assertEqual((self.root / "sample.py").read_text(), "x=1\n")

    def test_small_byte_capacity_reclaimed_exactly_and_caller_mutation_is_ignored(
        self,
    ) -> None:
        first, second = self.preview(2), self.preview(3)
        size = len(json.dumps(first, sort_keys=True).encode())
        owner = specialists.Specialists(self.root, preview_byte_capacity=size)
        handle = owner.retain_preview(first)["preview_sha256"]
        first["files"][0]["after"] = "caller mutation"
        self.assertEqual(owner.retained_preview(handle)["files"][0]["after"], "x=2\n")
        self.assertEqual(owner.preview_status().retained_bytes, size)
        with self.assertRaises(specialists.PreviewCapacityError):
            owner.retain_preview(second)
        self.assertEqual(owner.release_preview(handle).retained_bytes, 0)
        owner.retain_preview(second)
        self.assertEqual(owner.preview_status().retained_bytes, size)

    def test_unknown_foreign_repeated_and_closed_release_share_unavailable(
        self,
    ) -> None:
        owner = specialists.Specialists(self.root)
        foreign = specialists.Specialists(self.root)
        digest = foreign.retain_preview(self.preview(2))["preview_sha256"]
        missing = owner.release_preview(digest)
        unknown = owner.release_preview("0" * 64)
        self.assertEqual(missing.status, unknown.status)
        self.assertEqual(missing.status, "unavailable")
        handle = owner.retain_preview(self.preview(3))["preview_sha256"]
        owner.release_preview(handle)
        self.assertEqual(owner.release_preview(handle).status, "unavailable")
        self.assertEqual(foreign.preview_status().retained_count, 1)
        self.assertEqual(foreign.preview_status().count_capacity, 32)
        for malformed in ("", "f" * 63, "F" * 64, "g" * 64, "a" * 64 + "\n"):
            with self.subTest(digest=malformed):
                with self.assertRaises(ValueError):
                    owner.release_preview(malformed)
                with self.assertRaises(ValueError):
                    owner.preview_status(malformed)
        asyncio.run(foreign.close())
        self.assertEqual(foreign.release_preview(digest).status, "unavailable")
        self.assertFalse(availability(foreign, digest).available)

    def test_release_before_admission_refuses_native_work_without_effects(self) -> None:
        owner = specialists.Specialists(self.root)
        handle = owner.retain_preview(self.preview(2))["preview_sha256"]
        owner.release_preview(handle)
        with (
            patch.object(semantic_edits, "apply") as apply,
            patch.object(semantic_edits, "reconcile") as reconcile,
        ):
            with self.assertRaisesRegex(ValueError, "unknown"):
                owner.apply_preview(handle, "receipt.json")
            with self.assertRaisesRegex(ValueError, "unknown"):
                owner.reconcile_preview(handle)
            apply.assert_not_called()
            reconcile.assert_not_called()
        self.assertEqual(owner.preview_status().retained_count, 0)
        self.assertFalse((self.root / "receipt.json").exists())
        self.assertEqual((self.root / "sample.py").read_text(), "x=1\n")

    def test_successful_apply_settles_without_implicit_release(self) -> None:
        owner = specialists.Specialists(self.root)
        handle = owner.retain_preview(self.preview(2))["preview_sha256"]
        result = owner.apply_preview(handle, "receipt.json")
        self.assertTrue(result["complete"])
        status = owner.preview_status(handle)
        self.assertEqual(status.retained_count, 1)
        self.assertTrue(availability(owner, handle).available)
        self.assertFalse(availability(owner, handle).reconciliation_required)
        self.assertEqual(owner.release_preview(handle).status, "released")
        self.assertEqual((self.root / "sample.py").read_text(), "x=2\n")
        self.assertTrue((self.root / "receipt.json").exists())

    def test_partial_write_keeps_original_handle_until_explicit_reconcile(self) -> None:
        (self.root / "second.py").write_text("x=1\n")
        owner = specialists.Specialists(self.root)
        preview = semantic_edits.replacements(
            self.root,
            semantic_edits.capture(self.root, ["sample.py", "second.py"]),
            {"sample.py": "x=2\n", "second.py": "x=2\n"},
            producer="test",
        )
        handle = owner.retain_preview(preview)["preview_sha256"]
        native_write = vars(semantic_edits)["_write"]

        def interrupted(path: Path, content: bytes) -> None:
            if path.name == "second.py":
                raise OSError("second write failed")
            native_write(path, content)

        with (
            patch.object(semantic_edits, "_write", side_effect=interrupted),
            self.assertRaises(OSError),
        ):
            owner.apply_preview(handle, "receipt.json")
        self.assertTrue(availability(owner, handle).reconciliation_required)
        with self.assertRaisesRegex(ValueError, "reconciliation"):
            owner.release_preview(handle)
        self.assertEqual(owner.retained_preview(handle), preview)
        self.assertEqual(
            json.loads((self.root / "receipt.json").read_text())["preview_sha256"],
            handle,
        )
        reconciled = owner.reconcile_preview(handle)
        self.assertFalse(reconciled["complete"])
        self.assertEqual(
            [item["state"] for item in reconciled["files"]], ["applied", "unchanged"]
        )
        self.assertEqual(owner.release_preview(handle).status, "released")
        self.assertEqual((self.root / "sample.py").read_text(), "x=2\n")
        self.assertEqual((self.root / "second.py").read_text(), "x=1\n")

    def test_exhaustive_reconcile_settles_all_states_without_success_or_repair(
        self,
    ) -> None:
        edited = ["applied.py", "unchanged.py", "diverged.py", "missing.py"]
        basis = ["basis-unchanged.py", "basis-diverged.py", "basis-missing.py"]
        for name in edited + basis:
            (self.root / name).write_text("x=1\n")
        preview = semantic_edits.replacements(
            self.root,
            semantic_edits.capture(self.root, edited + basis),
            dict.fromkeys(edited, "x=2\n"),
            producer="test",
        )
        owner = specialists.Specialists(self.root)
        handle = owner.retain_preview(preview)["preview_sha256"]
        with (
            patch.object(semantic_edits, "apply", side_effect=OSError("failed")),
            self.assertRaises(OSError),
        ):
            owner.apply_preview(handle, "receipt.json")
        (self.root / "applied.py").write_text("x=2\n")
        (self.root / "diverged.py").write_text("x=7\n")
        (self.root / "missing.py").unlink()
        (self.root / "basis-diverged.py").write_text("x=7\n")
        (self.root / "basis-missing.py").unlink()
        result = owner.reconcile_preview(handle)
        self.assertEqual(
            {item["state"] for item in result["files"]},
            {"applied", "unchanged", "diverged", "missing"},
        )
        self.assertEqual(
            {item["state"] for item in result["basis"]},
            {"unchanged", "diverged", "missing"},
        )
        self.assertFalse(result["complete"])
        self.assertEqual(owner.release_preview(handle).status, "released")
        self.assertEqual((self.root / "diverged.py").read_text(), "x=7\n")
        self.assertFalse((self.root / "missing.py").exists())

    def test_incomplete_wrong_digest_and_exceptional_classification_keep_obligation(
        self,
    ) -> None:
        owner = specialists.Specialists(self.root)
        preview = self.preview(2)
        handle = owner.retain_preview(preview)["preview_sha256"]
        with (
            patch.object(semantic_edits, "apply", side_effect=OSError("failed")),
            self.assertRaises(OSError),
        ):
            owner.apply_preview(handle, "receipt.json")
        authentic = semantic_edits.reconcile(preview)
        for partial in (
            {**authentic, "files": []},
            {**authentic, "preview_sha256": "0" * 64},
            {**authentic, "files": [{"path": "sample.py", "state": "unknown"}]},
        ):
            with patch.object(semantic_edits, "reconcile", return_value=partial):
                owner.reconcile_preview(handle)
            with self.assertRaises(ValueError):
                owner.release_preview(handle)
        with (
            patch.object(
                semantic_edits, "reconcile", side_effect=OSError("unreadable")
            ),
            self.assertRaises(OSError),
        ):
            owner.reconcile_preview(handle)
        self.assertTrue(availability(owner, handle).reconciliation_required)
        owner.reconcile_preview(handle)
        self.assertEqual(owner.release_preview(handle).status, "released")

    def test_queued_apply_cannot_be_settled_by_older_reconcile_and_siblings_work(
        self,
    ) -> None:
        owner = specialists.Specialists(self.root)
        preview = self.preview(2)
        handle = owner.retain_preview(preview)["preview_sha256"]
        sibling = owner.retain_preview(self.preview(3))["preview_sha256"]
        reconciling, finish_reconcile, apply_entered, finish_apply, queued = (
            Event() for _ in range(5)
        )
        work = vars(owner)["_preview_work"][handle]
        native_lock = work.lock
        entered = 0
        counter_lock = Lock()

        class NotifyingLock:
            def __enter__(self) -> None:
                nonlocal entered
                with counter_lock:
                    entered += 1
                    if entered == 2:
                        queued.set()
                native_lock.acquire()

            def __exit__(self, *_args: object) -> None:
                native_lock.release()

        native_reconcile = semantic_edits.reconcile

        def reconcile(selected: dict) -> dict:
            reconciling.set()
            if not finish_reconcile.wait(5):
                raise AssertionError("reconcile barrier timed out")
            return native_reconcile(selected)

        def apply(_selected: dict, _receipt: Path) -> dict:
            apply_entered.set()
            if not finish_apply.wait(5):
                raise AssertionError("apply barrier timed out")
            raise OSError("new apply unresolved")

        with (
            ThreadPoolExecutor(max_workers=2) as pool,
            patch.object(work, "lock", NotifyingLock()),
            patch.object(semantic_edits, "reconcile", side_effect=reconcile),
            patch.object(semantic_edits, "apply", side_effect=apply),
        ):
            old = pool.submit(owner.reconcile_preview, handle)
            try:
                self.assertTrue(reconciling.wait(5))
                new = pool.submit(owner.apply_preview, handle, "receipt.json")
                self.assertTrue(queued.wait(5))
                self.assertEqual(availability(owner, handle).admitted, 2)
                with self.assertRaises(ValueError):
                    owner.release_preview(handle)
                self.assertEqual(owner.release_preview(sibling).status, "released")
                finish_reconcile.set()
                old.result(5)
                self.assertTrue(apply_entered.wait(5))
                self.assertTrue(availability(owner, handle).reconciliation_required)
                with self.assertRaises(ValueError):
                    owner.release_preview(handle)
            finally:
                finish_reconcile.set()
                finish_apply.set()
            with self.assertRaises(OSError):
                new.result(5)
        self.assertTrue(availability(owner, handle).reconciliation_required)
        owner.reconcile_preview(handle)
        self.assertEqual(owner.release_preview(handle).status, "released")


class PreviewToolTests(unittest.IsolatedAsyncioTestCase):
    async def test_actual_tools_expose_typed_outputs_and_validate_exact_digest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "sample.py").write_text("x=1\n")
            owner = specialists.Specialists(root)
            preview = semantic_edits.replacements(
                root,
                semantic_edits.capture(root, ["sample.py"]),
                {"sample.py": "x=2\n"},
                producer="test",
            )
            handle = owner.retain_preview(preview)["preview_sha256"]
            specs, tools = specialists.bindings(root, owner=owner)
            server = FastMCP("preview-lifecycle", strict_input_validation=True)
            for name in ("edit_preview_status", "edit_preview_release"):
                server.add_tool(tools[name])
            async with Client(server) as client:
                exposed = {tool.name: tool for tool in await client.list_tools()}
                for tool in exposed.values():
                    schema = tool.output_schema
                    if schema is None:
                        raise AssertionError(
                            "typed preview tool must expose output schema"
                        )
                    self.assertEqual(schema["additionalProperties"], False)
                    self.assertIn("retained_count", schema["properties"])
                    self.assertIn("byte_capacity", schema["properties"])
                status = await client.call_tool(
                    "edit_preview_status", {"preview_sha256": handle}
                )
                typed = specialists.PreviewStatus.model_validate(
                    status.structured_content
                )
                if typed.selected is None:
                    raise AssertionError("expected selected preview availability")
                self.assertTrue(typed.selected.available)
                self.assertEqual(typed.retained_count, 1)
                released = await client.call_tool(
                    "edit_preview_release", {"preview_sha256": handle}
                )
                self.assertEqual(
                    specialists.PreviewRelease.model_validate(
                        released.structured_content
                    ).status,
                    "released",
                )
                again = await client.call_tool(
                    "edit_preview_release", {"preview_sha256": handle}
                )
                self.assertEqual(
                    specialists.PreviewRelease.model_validate(
                        again.structured_content
                    ).status,
                    "unavailable",
                )
                for name in exposed:
                    invalid = await client.call_tool(
                        name, {"preview_sha256": "bad"}, raise_on_error=False
                    )
                    self.assertTrue(invalid.is_error)
                status_annotations = exposed["edit_preview_status"].annotations
                release_annotations = exposed["edit_preview_release"].annotations
                if status_annotations is None or release_annotations is None:
                    raise AssertionError("preview tools must describe their effects")
                self.assertTrue(status_annotations.read_only_hint)
                self.assertFalse(release_annotations.read_only_hint)
                self.assertTrue(release_annotations.destructive_hint)
                self.assertTrue(release_annotations.idempotent_hint)
            self.assertEqual(
                {spec.family for spec in specs if spec.id in exposed},
                {"controlled-edits"},
            )
            await owner.close()

    async def test_cancelled_awaiter_keeps_admission_until_sync_owner_returns(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "sample.py").write_text("x=1\n")
            owner = specialists.Specialists(root)
            preview = semantic_edits.replacements(
                root,
                semantic_edits.capture(root, ["sample.py"]),
                {"sample.py": "x=2\n"},
                producer="test",
            )
            handle = owner.retain_preview(preview)["preview_sha256"]
            _, tools = specialists.bindings(root, owner=owner)
            started, finish, returned = Event(), Event(), Event()
            native = owner.apply_preview

            def apply(digest: str, receipt: str) -> dict:
                try:
                    return native(digest, receipt)
                finally:
                    returned.set()

            def blocked(_preview: dict, _receipt: Path) -> dict:
                started.set()
                if not finish.wait(5):
                    raise AssertionError("worker barrier timed out")
                raise OSError("interrupted")

            with (
                patch.object(semantic_edits, "apply", side_effect=blocked),
                patch.object(owner, "apply_preview", side_effect=apply),
            ):
                request = asyncio.create_task(
                    tools["edit_preview_apply"].run(
                        {"preview_sha256": handle, "receipt": "receipt.json"}
                    )
                )
                try:
                    self.assertTrue(await asyncio.to_thread(started.wait, 5))
                    request.cancel()
                    self.assertEqual(availability(owner, handle).admitted, 1)
                    with self.assertRaisesRegex(Exception, "pse.specialist.refused"):
                        await tools["edit_preview_release"].run(
                            {"preview_sha256": handle}
                        )
                    self.assertEqual(owner.retained_preview(handle), preview)
                finally:
                    finish.set()
                    self.assertTrue(await asyncio.to_thread(returned.wait, 5))
                    with self.assertRaises(asyncio.CancelledError):
                        await request
            self.assertEqual(availability(owner, handle).admitted, 0)
            self.assertTrue(availability(owner, handle).reconciliation_required)
            owner.reconcile_preview(handle)
            self.assertEqual(owner.release_preview(handle).status, "released")
            await owner.close()


if __name__ == "__main__":
    unittest.main()
