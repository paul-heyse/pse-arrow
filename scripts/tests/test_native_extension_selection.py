# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Installed correctness consumes the recorded image selection without preparing it."""

# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import json
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from scripts import python_extension
from scripts.capabilities import (
    execution_basis,
    native_contracts,
    native_read_sessions,
    process,
    workers,
)
from scripts.capabilities.native_extension import (
    ExtensionSelectionError,
    extension_currency,
    selected_extension,
)


class InstalledExtensionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.path = self.root / "extension.json"
        self.receipt = patch.object(
            python_extension, "stage_receipt", return_value=self.path
        )
        self.receipt.start()
        self.addCleanup(self.receipt.stop)

    def write(self, mode: str = "dev", profile: str = "correctness") -> None:
        document = {
            "document_version": 1,
            "kind": "extension",
            "generation": "owned-generation",
            "mode": mode,
            "profile": profile,
            "validated": True,
            "interpreter": str(python_extension.interpreter({}).absolute()),
            "artifact": {
                "binary": str(self.root / "_native.so"),
                "sha256": "recorded",
                "features": ["force-validate"]
                + (["native-solvers"] if mode == "native" else []),
            },
        }
        self.path.write_text(json.dumps(document))

    def test_receipt_mode_and_profile_select_original_currency_owner(self) -> None:
        for mode in ("dev", "native"):
            self.write(mode, "custom-profile")
            with (
                patch.object(
                    python_extension, "currency_check", return_value=object()
                ) as currency,
                patch.object(python_extension, "capture") as capture,
                patch.object(python_extension, "run_stages") as stages,
            ):
                extension_currency({}, "installed_correctness", validated=True)
            currency.assert_called_once_with(
                {}, mode=mode, profile="custom-profile", validated=True
            )
            capture.assert_not_called()
            stages.assert_not_called()

    def test_missing_malformed_and_inconsistent_receipt_refuse_without_guess_or_probe(
        self,
    ) -> None:
        malformed = (
            "not-json",
            json.dumps({}),
            json.dumps(
                {"inputs": {"mode": "unsupported", "profile": "dev", "validated": True}}
            ),
        )
        with (
            patch.object(python_extension, "currency_check") as currency,
            patch.object(python_extension, "capture") as capture,
            patch.object(python_extension, "run_stages") as stages,
        ):
            with self.assertRaises(ExtensionSelectionError):
                extension_currency({}, "installed_correctness", validated=True)
            for contents in malformed:
                self.path.write_text(contents)
                with self.assertRaises(ExtensionSelectionError):
                    extension_currency({}, "installed_correctness", validated=True)
            self.write()
            document = json.loads(self.path.read_text())
            document["profile"] = "not/a/profile"
            self.path.write_text(json.dumps(document))
            with self.assertRaises(ExtensionSelectionError):
                extension_currency({}, "installed_correctness", validated=True)
        currency.assert_not_called()
        capture.assert_not_called()
        stages.assert_not_called()

    def test_worker_policy_prepares_native_only_for_recorded_native_image(self) -> None:
        requirement = types.SimpleNamespace(
            mode="installed_correctness", validated=True
        )
        binding = types.SimpleNamespace(extension_requirement=requirement)
        with (
            patch.object(native_contracts, "binding_for", return_value=binding),
            patch.object(
                python_extension,
                "bind_interpreter",
                side_effect=lambda environment: environment,
            ),
        ):
            for mode in ("dev", "native"):
                self.write(mode)
                expected = (
                    workers.WORKERS["native-read-prepared"].environment
                    if mode == "native"
                    else ()
                )
                self.assertEqual(
                    workers.selected_worker_environment(
                        "native-read-prepared", "owner-read", {}
                    ),
                    expected,
                )
                self.assertEqual(
                    workers.selected_worker_environment("native", "owner-read", {}),
                    expected,
                )
                self.assertEqual(
                    workers.selected_worker_environment(
                        "native-read", "owner-read", {}
                    ),
                    (),
                )
            self.write("dev")
            requirement.mode = "native_correctness"
            with self.assertRaises(ExtensionSelectionError):
                workers.selected_worker_environment(
                    "native-read-prepared", "owner-read", {}
                )
            requirement.mode = "installed_correctness"
            self.path.unlink()
            with self.assertRaises(ExtensionSelectionError):
                workers.selected_worker_environment(
                    "native-read-prepared", "owner-read", {}
                )
            # Passive route cannot obtain preparation merely from reusing its context.
            self.assertEqual(
                workers.selected_worker_environment("native-read", "owner-read", {}), ()
            )

    def test_explicit_selection_is_preserved_and_validation_cannot_weaken(self) -> None:
        self.write("native", "recorded")
        selected = selected_extension(
            {},
            "installed_correctness",
            validated=True,
            explicit_mode="dev",
            explicit_profile="explicit",
        )
        self.assertEqual((selected.mode, selected.profile), ("dev", "explicit"))
        with self.assertRaises(ExtensionSelectionError):
            selected_extension(
                {}, "native_correctness", validated=True, explicit_mode="dev"
            )
        with (
            patch.object(python_extension, "currency_check") as currency,
            self.assertRaises(ExtensionSelectionError),
        ):
            extension_currency({}, "installed_correctness", validated=False)
        currency.assert_not_called()


class ExtensionStartupTests(unittest.IsolatedAsyncioTestCase):
    async def test_unknown_installed_selection_acquires_no_process_or_session_resources(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            call = workers.WorkerCall("selected", "owner-read")
            route = workers.Worker(
                "fixture.worker",
                frozenset({"owner-read"}),
                environment_selection="installed-extension",
            )
            binding = types.SimpleNamespace(
                extension_requirement=types.SimpleNamespace(
                    mode="installed_correctness", validated=True
                )
            )
            handoff = execution_basis.Handoff(
                definition=execution_basis.Definition(
                    checkout=str(root),
                    operation="owner-read",
                    wire_revision="a" * 64,
                    sources={},
                ),
                worker="selected",
                operation="owner-read",
                route_revision="b" * 64,
                request_sha256="c" * 64,
            )
            with (
                patch.dict(workers.WORKERS, {"selected": route}),
                patch.object(native_contracts, "binding_for", return_value=binding),
                patch.object(
                    python_extension,
                    "stage_receipt",
                    return_value=root / "missing.json",
                ),
                patch.object(process.tempfile, "TemporaryDirectory") as temporary,
                patch.object(
                    process.asyncio, "create_subprocess_exec", new_callable=AsyncMock
                ) as spawn,
            ):
                value = await process.run_worker(root, call, {})
                self.assertEqual(
                    value["code"],
                    "pse.native.installed_extension_selection_unavailable",
                )
                session = native_read_sessions.ReadSession(root, "context")
                with self.assertRaises(ExtensionSelectionError):
                    await session._start(handoff)  # noqa: SLF001 -- assert selection precedes every owned startup effect
                await session.close()
            temporary.assert_not_called()
            spawn.assert_not_called()


if __name__ == "__main__":
    unittest.main()
