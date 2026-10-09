# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The checkout environment boundary: precedence, secrets, placement and status."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

from scripts import pse_env


def identity(_root: Path, env: dict[str, str]) -> dict[str, str]:
    return dict(env)


class PseEnvTests(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        (self.root / ".python-version").write_text("3.14.7\n")
        (self.root / ".envrc.local").write_text(
            "export PSE_TEST_LICENSE='s3cret value'\nexport PSE_TEST_OTHER=local\n"
        )
        configured = patch.object(pse_env.build_environment, "configure", identity)
        configured.start()
        self.addCleanup(configured.stop)

    def test_caller_wins_over_local_and_local_fills_the_rest(self) -> None:
        env = pse_env.compose(
            self.root, {"PSE_TEST_OTHER": "caller", "PATH": "/usr/bin"}
        )
        self.assertEqual(env["PSE_TEST_OTHER"], "caller")
        self.assertEqual(env["PSE_TEST_LICENSE"], "s3cret value")
        self.assertEqual(env["PATH"].split(os.pathsep)[0], str(self.root / ".venv/bin"))

    def test_composition_is_idempotent(self) -> None:
        once = pse_env.compose(self.root, {"PATH": "/usr/bin:/bin"})
        self.assertEqual(pse_env.compose(self.root, once), once)

    def test_print_defers_local_values_and_evaluates_to_them(self) -> None:
        text = pse_env.render(
            self.root, {"PATH": "/usr/bin:/bin", "HOME": "/nonexistent-home"}
        )
        self.assertNotIn("s3cret", text)
        result = subprocess.run(
            [
                "bash",
                "-c",
                text + 'printf "%s|%s" "$PSE_TEST_LICENSE" "$PSE_TEST_OTHER"',
            ],
            capture_output=True,
            text=True,
            check=True,
            env={"PATH": "/usr/bin:/bin", "HOME": "/nonexistent-home"},
        )
        self.assertEqual(result.stdout, "s3cret value|local")

    def test_print_keeps_a_caller_value_over_the_deferred_local_one(self) -> None:
        text = pse_env.render(self.root, {"PATH": "/usr/bin:/bin"})
        result = subprocess.run(
            ["bash", "-c", text + 'printf "%s" "$PSE_TEST_OTHER"'],
            capture_output=True,
            text=True,
            check=True,
            env={"PATH": "/usr/bin:/bin", "PSE_TEST_OTHER": "mine"},
        )
        self.assertEqual(result.stdout, "mine")

    def test_slices_nest_by_dash(self) -> None:
        group = pse_env.slice_group("pse-agents.slice")
        self.assertEqual(group.parts[-2:], ("pse.slice", "pse-agents.slice"))

    def test_placement_defaults_and_overrides(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root / "admission"
        owner.nonce = "a" * 32
        owner.environment.return_value = {
            pse_env.host.MARKER: str(owner.directory / owner.nonce)
        }
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire", return_value=owner) as acquire,
            patch.object(pse_env.host, "enforce_parent"),
            patch.object(pse_env.host, "enforce_allocation"),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=True),
        ):
            env = {}
            placed = pse_env.placement(env, native=False)
            profile = acquire.call_args.args[0]
            self.assertIn(f"--slice={pse_env.host.allocation_slice(owner)}", placed)
            self.assertIn(f"MemoryMax={profile.memory}", placed)
            self.assertIn("--bind-allocation", placed)
            self.assertTrue(any(arg.startswith("--unit=pse-cmd-") for arg in placed))
            self.assertEqual(
                env[pse_env.host.MARKER], str(owner.directory / owner.nonce)
            )
            with self.assertRaises(pse_env.host.AdmissionError):
                pse_env.placement({"PSE_MEMORY_MAX": "off"}, native=True)

    def test_placement_stays_inside_an_owning_scope_or_without_a_manager(self) -> None:
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/g",
            "invocation": "b" * 32,
        }
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.operation, "scope_owner", return_value=owner),
            self.assertRaisesRegex(
                pse_env.BoundaryError, "no verified host allocation"
            ),
        ):
            pse_env.placement({}, native=False)
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=False),
            self.assertRaisesRegex(pse_env.BoundaryError, "systemd user manager"),
        ):
            pse_env.placement({}, native=True)

    def test_aggregate_limit_below_the_command_cap_is_reported(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root / "admission"
        owner.nonce = "a" * 32
        owner.environment.return_value = {}
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire", return_value=owner),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=True),
            patch.object(
                pse_env.host,
                "enforce_parent",
                side_effect=pse_env.host.AdmissionError(
                    "Effective ancestor is below requested aggregate"
                ),
            ),
            self.assertRaisesRegex(
                pse_env.host.AdmissionError, "below requested aggregate"
            ),
        ):
            pse_env.placement({"PSE_MEMORY_MAX": "120G"}, native=False)
        owner.register.assert_not_called()

    def test_boundary_failure_and_missing_command_statuses(self) -> None:
        with patch.object(
            pse_env, "compose", side_effect=pse_env.BoundaryError("broken")
        ):
            self.assertEqual(
                pse_env.main(["--no-scope", "--", "true"]), pse_env.FAILURE
            )
        with patch.object(pse_env, "compose", return_value={"PATH": "/usr/bin:/bin"}):
            self.assertEqual(
                pse_env.main(["--no-scope", "--", "pse-env-missing-command"]), 127
            )


class StoreReadinessTests(unittest.TestCase):
    def test_unset_up_store_fails_with_the_setup_commands(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            self.assertRaises(pse_env.BoundaryError) as raised,
        ):
            pse_env.require_store({"PSE_SURREAL_STATE": directory})
        self.assertIn("just surreal setup", str(raised.exception))

    def test_store_default_is_the_supervisor_default(self) -> None:
        with patch.object(pse_env.build_environment, "configure", identity):
            env = pse_env.compose(
                Path("/checkout"), {"HOME": "/home/x", "PATH": ""}, {}
            )
        self.assertEqual(
            env["PSE_SURREAL_STATE"],
            "/home/x/.local/state/pse-arrow/surreal-functional-v2",
        )
        with patch.object(pse_env.build_environment, "configure", identity):
            chosen = pse_env.compose(
                Path("/checkout"), {"PSE_SURREAL_STATE": "/elsewhere", "PATH": ""}, {}
            )
        self.assertEqual(chosen["PSE_SURREAL_STATE"], "/elsewhere")


if __name__ == "__main__":
    unittest.main()
