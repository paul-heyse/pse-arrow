# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The checkout environment boundary: precedence, secrets, placement and status."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

from scripts import pse_env


def identity(_root: Path, env: dict[str, str]) -> dict[str, str]:
    return dict(env)


class PseEnvTests(unittest.TestCase):
    def test_docs_bash_wrapper_cannot_fall_through_to_product_python(self) -> None:
        product = self.root / ".venv/bin"
        product.mkdir(parents=True)
        (product / "python3").write_text("product interpreter fixture")
        with (
            patch.object(pse_env, "ROOT", self.root),
            patch.object(
                pse_env,
                "compose",
                return_value={
                    "UV_PROJECT_ENVIRONMENT": ".venv",
                    "PATH": str(product) + ":/bin",
                },
            ),
            patch.object(pse_env, "execute") as execute,
            patch.object(pse_env, "placement", return_value=[]) as placement,
            patch.object(pse_env, "unexecutable", return_value=None),
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env, "require_enclosing_owner"),
        ):
            self.assertEqual(
                pse_env.main(
                    ["--docs", "--", "bash", "-c", "python3 -m scripts.docs build"]
                ),
                pse_env.FAILURE,
            )
            execute.assert_not_called()
            placement.assert_not_called()
            execute.return_value = 0
            self.assertEqual(
                pse_env.main(
                    [
                        "--docs",
                        "--",
                        "uv",
                        "sync",
                        "--locked",
                        "--only-group",
                        "docs",
                        "--no-install-project",
                    ]
                ),
                0,
            )
            self.assertEqual(
                execute.call_args.args[1]["UV_PROJECT_ENVIRONMENT"],
                str(self.root / ".venv-docs"),
            )

    def test_docs_selector_overrides_product_without_synchronizing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            selected = pse_env.documentation_environment(
                root, {"UV_PROJECT_ENVIRONMENT": ".venv", "PATH": "/bin"}
            )
            self.assertEqual(
                selected["UV_PROJECT_ENVIRONMENT"], str(root / ".venv-docs")
            )
            self.assertEqual(
                selected["PATH"].split(":")[0], str(root / ".venv-docs/bin")
            )
            self.assertFalse((root / ".venv-docs").exists())

    def test_docs_selector_rejects_product_alias_before_effects(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".venv").mkdir()
            (root / ".venv-docs").symlink_to(root / ".venv", target_is_directory=True)
            with self.assertRaisesRegex(pse_env.BoundaryError, "aliases"):
                pse_env.documentation_environment(
                    root, {"UV_PROJECT_ENVIRONMENT": ".venv"}
                )

    def test_docs_print_moves_inherited_docs_path_ahead_of_product(self) -> None:
        inherited = f"{self.root}/.venv/bin:{self.root}/.venv-docs/bin:/bin"
        with patch.object(pse_env, "local_keys", return_value={}):
            rendered = pse_env.render(
                self.root, {"HOME": str(self.root), "PATH": inherited}, docs=True
            )
        result = subprocess.run(
            [
                "bash",
                "-c",
                rendered
                + '\nprintf "%s\\n" "$PATH"\n'
                + rendered
                + '\nprintf "%s\\n" "$PATH"',
            ],
            env={"PATH": inherited},
            capture_output=True,
            text=True,
            check=True,
        )
        once, twice = result.stdout.splitlines()
        self.assertEqual(once.split(":")[0], str(self.root / ".venv-docs/bin"))
        self.assertEqual(once, twice)

    def test_docs_print_selects_child_environment_even_with_local_product_selector(
        self,
    ) -> None:
        with patch.object(
            pse_env, "local_keys", return_value={"UV_PROJECT_ENVIRONMENT": ".venv"}
        ):
            rendered = pse_env.render(
                self.root, {"HOME": str(self.root), "PATH": "/bin"}, docs=True
            )
        result = subprocess.run(
            [
                "bash",
                "-c",
                rendered + '\nprintf "%s\\n" "$UV_PROJECT_ENVIRONMENT" "$PATH"',
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        lines = result.stdout.splitlines()
        self.assertEqual(lines[0], str(self.root / ".venv-docs"))
        self.assertEqual(lines[1].split(":")[0], str(self.root / ".venv-docs/bin"))

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

    def test_recipe_class_precedence_before_placement_and_nested_ownership(
        self,
    ) -> None:
        marker = {
            pse_env.host.MARKER: "/owner/" + "a" * 32,
            "PSE_RESOURCE_CLASS": "exclusive",
        }
        cases = [
            ({}, {}, [], "compile"),
            ({"PSE_RESOURCE_CLASS": "wide"}, {}, [], "wide"),
            ({}, {"PSE_RESOURCE_CLASS": "timing"}, [], "timing"),
            (
                {"PSE_RESOURCE_CLASS": "wide"},
                {},
                ["--resource-class", "light"],
                "light",
            ),
            (marker, {}, [], "exclusive"),
        ]
        for caller, local, options, expected in cases:
            with (
                self.subTest(expected=expected),
                patch.dict(os.environ, caller, clear=True),
                patch.object(pse_env, "ROOT", self.root),
                patch.object(pse_env, "local_keys", return_value=local),
                patch.object(pse_env.host, "inherit", return_value=None),
                patch.object(pse_env, "unexecutable", return_value=None),
                patch.object(pse_env, "placement", return_value=[]) as placement,
                patch.object(pse_env, "execute", return_value=0),
            ):
                self.assertEqual(
                    pse_env.main(
                        [
                            *options,
                            "--recipe-default-class",
                            "compile",
                            "--",
                            "bash",
                            "-c",
                            "cargo check",
                        ]
                    ),
                    0,
                )
                self.assertEqual(
                    placement.call_args.args[0]["PSE_RESOURCE_CLASS"], expected
                )

    def test_admitted_jobs_follow_declaration_and_preserve_explicit_overrides(
        self,
    ) -> None:
        declaration = "16"

        def declared(_root: Path, env: dict[str, str]) -> dict[str, str]:
            result = dict(env)
            result.setdefault("CARGO_BUILD_JOBS", declaration)
            return result

        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = MagicMock(cores=(2, 4))
        with (
            patch.object(pse_env.build_environment, "configure", side_effect=declared),
            patch.object(pse_env.host, "cpu_set", return_value=(2, 4, 6, 8)),
        ):
            env = pse_env.compose(self.root, {}, {})
            self.assertEqual(env["CARGO_BUILD_JOBS"], "16")
            pse_env.admitted_cargo_jobs(env, owner)
            self.assertEqual(env["CARGO_BUILD_JOBS"], "4")
            # Re-entry starts from the current declaration, not the previous lane.
            nested = pse_env.compose(self.root, env, {})
            self.assertEqual(nested["CARGO_BUILD_JOBS"], "16")
            pse_env.admitted_cargo_jobs(nested, owner)
            self.assertEqual(nested, env)
            declaration = "3"
            changed = pse_env.compose(self.root, env, {})
            pse_env.admitted_cargo_jobs(changed, owner)
            self.assertEqual(changed["CARGO_BUILD_JOBS"], "3")
            for explicit in ("2", "24", "-1"):
                overridden = pse_env.compose(
                    self.root, {**env, "CARGO_BUILD_JOBS": explicit}, {}
                )
                pse_env.admitted_cargo_jobs(overridden, owner)
                self.assertEqual(overridden["CARGO_BUILD_JOBS"], explicit)
                self.assertNotIn(pse_env.CARGO_DEFAULT_JOBS, overridden)
            local = pse_env.compose(self.root, {}, {"CARGO_BUILD_JOBS": "12"})
            pse_env.admitted_cargo_jobs(local, owner)
            self.assertEqual(local["CARGO_BUILD_JOBS"], "12")
            # A declaration already below the lane stays unchanged.
            small = {"CARGO_BUILD_JOBS": "2", pse_env.CARGO_DEFAULT_JOBS: "2"}
            pse_env.admitted_cargo_jobs(small, owner)
            self.assertEqual(small["CARGO_BUILD_JOBS"], "2")

    def test_placement_bounds_jobs_after_acquisition(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = pse_env.host.select("compile")
        owner.directory = self.root / "allocation"
        owner.nonce = "a" * 32
        owner.environment.return_value = {
            pse_env.host.MARKER: str(owner.directory / owner.nonce)
        }
        env = {
            "CARGO_BUILD_JOBS": "16",
            pse_env.CARGO_DEFAULT_JOBS: "16",
            "PSE_RESOURCE_CLASS": "compile",
        }
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire", return_value=owner),
            patch.object(pse_env.host, "enforce_parent"),
            patch.object(pse_env.host, "enforce_allocation"),
            patch.object(pse_env.host, "cpu_set", return_value=(1, 2, 3, 4)),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=True),
        ):
            prefix = pse_env.placement(env, native=False)
        self.assertIn("--bind-allocation", prefix)
        self.assertEqual(env["CARGO_BUILD_JOBS"], "4")
        self.assertEqual(env[pse_env.CARGO_DEFAULT_JOBS], "4")

    def test_control_environment_does_not_prepare_compiler(self) -> None:
        deadline = time.monotonic() + 1
        with patch.object(
            pse_env.build_environment,
            "configure",
            side_effect=AssertionError("compiler"),
        ):
            env = pse_env.control_environment(
                self.root,
                {"PATH": "/usr/bin", "PSE_TEST_OTHER": "caller"},
                deadline=deadline,
            )
        self.assertEqual(env["PSE_TEST_OTHER"], "caller")
        self.assertEqual(env["PSE_TEST_LICENSE"], "s3cret value")
        self.assertIn(str(self.root / ".venv/bin"), env["PATH"])

    def test_light_control_uses_one_deadline_without_scientific_recovery(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root
        owner.nonce = "a" * 32
        owner.profile = pse_env.host.select("light")
        owner.environment.return_value = {}
        deadline = time.monotonic() + 0.5
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(
                pse_env.host, "acquire_light_control", return_value=owner
            ) as acquire,
            patch.object(pse_env.host, "enforce_parent") as parent,
            patch.object(pse_env.host, "enforce_allocation") as allocation,
            patch.object(pse_env.host, "release_light_control") as release,
            patch.object(
                pse_env.host, "reconcile", side_effect=AssertionError("reconcile")
            ),
            patch.object(
                pse_env, "compose", side_effect=AssertionError("compiler environment")
            ),
            patch.object(
                pse_env.host,
                "control_run",
                side_effect=[
                    subprocess.CompletedProcess([], 0),
                    subprocess.CompletedProcess([], 0, "headers", ""),
                ],
            ) as run,
        ):
            result = pse_env.run_light_control(["fixed-helper"], {}, deadline=deadline)
        self.assertEqual(result.stdout, "headers")
        self.assertEqual(acquire.call_args.kwargs["deadline"], deadline)
        self.assertEqual(parent.call_args.kwargs["deadline"], deadline)
        self.assertEqual(allocation.call_args.kwargs["deadline"], deadline)
        self.assertEqual(release.call_args.kwargs["deadline"], deadline)
        self.assertEqual(run.call_args.kwargs["deadline"], deadline)
        self.assertIn("--scope", run.call_args.args[0])
        self.assertEqual(run.call_args.args[0][-1], "fixed-helper")

    def test_nested_control_uses_verified_owner_without_new_admission(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        deadline = time.monotonic() + 0.5
        with (
            patch.object(pse_env.host, "inherit", return_value=owner) as inherit,
            patch.object(
                pse_env.host,
                "acquire_light_control",
                side_effect=AssertionError("new admission"),
            ),
            patch.object(
                pse_env.host,
                "release_light_control",
                side_effect=AssertionError("release enclosing owner"),
            ),
            patch.object(
                pse_env.host,
                "control_run",
                return_value=subprocess.CompletedProcess([], 0, "headers", ""),
            ) as run,
        ):
            pse_env.run_light_control(["fixed-helper"], {}, deadline=deadline)
        self.assertEqual(inherit.call_args.kwargs["deadline"], deadline)
        self.assertEqual(run.call_args.args[0], ["fixed-helper"])

    def test_control_does_not_return_output_when_cleanup_is_unverified(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root
        owner.nonce = "a" * 32
        owner.profile = pse_env.host.select("light")
        owner.environment.return_value = {}
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire_light_control", return_value=owner),
            patch.object(pse_env.host, "enforce_parent"),
            patch.object(pse_env.host, "enforce_allocation"),
            patch.object(pse_env.host, "release_light_control", return_value=False),
            patch.object(
                pse_env.host,
                "control_run",
                side_effect=[
                    subprocess.CompletedProcess([], 0),
                    subprocess.CompletedProcess([], 0, "secret output", ""),
                ],
            ),
            self.assertRaisesRegex(pse_env.BoundaryError, "retained"),
        ):
            pse_env.run_light_control(
                ["fixed-helper"], {}, deadline=time.monotonic() + 1
            )

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
