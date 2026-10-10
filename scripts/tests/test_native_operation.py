# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Operation ownership, immutable publication and archive association controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import fcntl
import io
import json
import os
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from unittest.mock import patch

from scripts import (
    build_environment,
    build_measurements,
    producer_deployment,
    pse_env,
    surreal_server,
    validation,
)
from scripts import native_cache as cache
from scripts import native_operation as operation
from scripts import native_pipeline_cache as pipeline


class NativeOperationTests(unittest.TestCase):
    def setUp(self) -> None:
        # Each fixture supplies its own admission owner, including when this
        # suite runs inside a real assessment operation.
        environment = {
            name: value
            for name, value in os.environ.items()
            if name
            not in {
                operation.MARKER,
                "PSE_NATIVE_HANDOFF",
                pse_env.host.MARKER,
                "PSE_ADMISSION_DEADLINE",
            }
        }
        isolated = patch.dict(os.environ, environment, clear=True)
        isolated.start()
        self.addCleanup(isolated.stop)
        self.fixture_units: set[str] = set()
        self.fixture_parent_scope = operation.scope_owner()
        self.fixture_parent_group = operation.process_group(os.getpid())

    def test_selected_library_paths_do_not_add_working_directory_search(self) -> None:
        for previous in (None, ""):
            environment = {} if previous is None else {"LD_LIBRARY_PATH": previous}
            selected = cache.prepend_library_path("/selected/lib", environment)
            self.assertEqual(selected.split(os.pathsep), ["/selected/lib"])
            environment["LD_LIBRARY_PATH"] = selected
            self.assertEqual(
                cache.prepend_library_path("/pipeline/lib", environment).split(
                    os.pathsep
                ),
                ["/pipeline/lib", "/selected/lib"],
            )
        self.assertEqual(
            cache.prepend_library_path(
                "/selected/lib", {"LD_LIBRARY_PATH": "/explicit/lib:/other/lib"}
            ),
            "/selected/lib:/explicit/lib:/other/lib",
        )
        environment = {"LD_LIBRARY_PATH": "/caller/lib::/caller/lib:/solver/lib:"}
        libraries = ("/solver/lib", "/uno/lib", "/petsc/lib")
        for library in libraries:
            environment["LD_LIBRARY_PATH"] = cache.prepend_library_path(
                library, environment
            )
        expected = "/petsc/lib:/uno/lib:/solver/lib:/caller/lib::/caller/lib:"
        self.assertEqual(environment["LD_LIBRARY_PATH"], expected)
        for library in libraries:
            environment["LD_LIBRARY_PATH"] = cache.prepend_library_path(
                library, environment
            )
        self.assertEqual(environment["LD_LIBRARY_PATH"], expected)

    def await_file(self, path: Path, process: subprocess.Popen | None = None) -> dict:
        deadline = time.monotonic() + 15
        while not path.exists():
            if process is not None and process.poll() is not None:
                self.fail(
                    f"native fixture exited before admission: {process.returncode}; "
                    + (
                        (path.parent / "fixture.log").read_text()[-3000:]
                        if (path.parent / "fixture.log").is_file()
                        else ""
                    )
                )
            if time.monotonic() >= deadline:
                self.fail("native fixture did not admit within its bounded deadline")
            time.sleep(0.02)
        return json.loads(path.read_text())

    def await_drain(self, record: Mapping[str, object]) -> None:
        deadline = time.monotonic() + 15
        while not operation.drained(record):
            if time.monotonic() >= deadline:
                self.fail("authentic native fixture scope did not drain")
            time.sleep(0.02)

    def fixture_environment(self, base: Path) -> dict[str, str]:
        return {
            **{
                name: value
                for name, value in os.environ.items()
                if name
                not in {
                    operation.MARKER,
                    "PSE_NATIVE_HANDOFF",
                    pse_env.host.MARKER,
                    "PSE_ADMISSION_DEADLINE",
                }
            },
            "PSE_NATIVE_CACHE": str(base),
            "PSE_NATIVE_CAPABILITIES": "",
            "PSE_MEMORY_MAX": "512M",
            "PSE_RESOURCE_CLASS": "light",
            **pse_env.manager_environment(),
        }

    def fixture_command(
        self, command: list[str], environment: dict[str, str]
    ) -> list[str]:
        self.assert_parent_scope_unchanged()
        # These zero-capability fixtures own disposable generations and may kill
        # their entire scope. Compose a new fixture root through the placement
        # owner; do not change the production policy that nested users stay put.
        # Only command construction sees this test-isolation premise. The real
        # child observes and validates its actual unit and invocation normally.
        with patch.object(operation, "scope_owner", return_value=None):
            prefix = pse_env.placement(environment, native=True)
        allocation = pse_env.host.inherit(environment)
        self.assertIsNotNone(allocation, "fixture must own its host allocation")
        assert allocation is not None
        self.addCleanup(allocation.release)
        self.assertTrue(prefix, "independent fixture requires the local user manager")
        self.assertIn(f"MemoryMax={512 * (1 << 20)}", prefix)
        unit = next(
            argument.removeprefix("--unit=")
            for argument in prefix
            if argument.startswith("--unit=")
        )
        if self.fixture_parent_scope is not None:
            self.assertNotEqual(unit, self.fixture_parent_scope["unit"])
        self.fixture_units.add(unit)
        return [*prefix, *command]

    def assert_parent_scope_unchanged(self) -> None:
        self.assertEqual(
            operation.process_group(os.getpid()), self.fixture_parent_group
        )
        self.assertEqual(operation.scope_owner(), self.fixture_parent_scope)

    def assert_fixture_scope(
        self, owner: Mapping[str, str], *, live: bool = True
    ) -> None:
        self.assert_parent_scope_unchanged()
        self.assertIn(owner["unit"], self.fixture_units)
        self.assertNotEqual(owner["group"], self.fixture_parent_group)
        self.assertFalse(owner["group"].startswith(self.fixture_parent_group + "/"))
        self.assertFalse(self.fixture_parent_group.startswith(owner["group"] + "/"))
        if self.fixture_parent_scope is not None:
            self.assertNotEqual(owner["unit"], self.fixture_parent_scope["unit"])
            self.assertNotEqual(owner["group"], self.fixture_parent_scope["group"])
            self.assertNotEqual(
                owner["invocation"], self.fixture_parent_scope["invocation"]
            )
        if live:
            observed = operation.unit_observation(owner["unit"])
            self.assertEqual(observed["LoadState"], "loaded")
            self.assertEqual(observed["ControlGroup"], owner["group"])
            self.assertEqual(observed["InvocationID"], owner["invocation"])

    def test_child_fixture_environment_starts_independent_owner(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {
                    operation.MARKER: "/outer/.operations/owner.json",
                    "PSE_NATIVE_HANDOFF": "/outer/.operations/handoff.json",
                    pse_env.host.MARKER: "/outer/host/" + "c" * 32,
                    "PSE_ADMISSION_DEADLINE": "1",
                    "PSE_FIXTURE_CALLER": "preserved",
                },
            ),
        ):
            base = Path(directory)
            environment = self.fixture_environment(base)
            self.assertNotIn(operation.MARKER, environment)
            self.assertNotIn("PSE_NATIVE_HANDOFF", environment)
            self.assertNotIn(pse_env.host.MARKER, environment)
            self.assertNotIn("PSE_ADMISSION_DEADLINE", environment)
            self.assertEqual(environment["PSE_NATIVE_CACHE"], str(base))
            self.assertEqual(environment["PSE_FIXTURE_CALLER"], "preserved")
            self.assertEqual(
                os.environ[operation.MARKER], "/outer/.operations/owner.json"
            )

    def test_independent_fixture_command_preserves_nested_scope_policy(self) -> None:
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/parent",
            "invocation": "b" * 32,
        }
        self.fixture_parent_scope = owner
        self.fixture_parent_group = owner["group"]
        with (
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(pse_env, "manager_available", return_value=True),
            patch.object(pse_env, "manager_environment", return_value={}),
            patch.object(pse_env, "limits", return_value=[]),
        ):
            environment = self.fixture_environment(Path("/disposable-fixture"))
            command = self.fixture_command(["fixture", "literal argument"], environment)
            self.assertEqual(command[-2:], ["fixture", "literal argument"])
            self.assertEqual(len(self.fixture_units), 1)
            self.assertNotIn(owner["unit"], self.fixture_units)
            self.assertEqual(operation.scope_owner(), owner)
            self.assertEqual(pse_env.placement(environment, native=True), [])

    def test_fixture_cleanup_refuses_enclosing_scope(self) -> None:
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/parent",
            "invocation": "b" * 32,
        }
        self.fixture_parent_scope = owner
        self.fixture_parent_group = owner["group"]
        with (
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(operation.subprocess, "run") as manager,
        ):
            with self.assertRaisesRegex(AssertionError, "only a minted fixture"):
                self.kill_scope(owner["unit"])
            manager.assert_not_called()

    def kill_scope(self, unit: str) -> None:
        self.assert_parent_scope_unchanged()
        self.assertIn(
            unit, self.fixture_units, "only a minted fixture scope may be killed"
        )
        if self.fixture_parent_scope is not None:
            self.assertNotEqual(unit, self.fixture_parent_scope["unit"])
        observed = operation.unit_observation(unit)
        if observed["LoadState"] == "not-found":
            return  # A collected fixture has no remaining children to signal.
        self.assertEqual(observed["LoadState"], "loaded")
        if observed["ControlGroup"] == "":
            return  # An inactive fixture no longer owns a cgroup to signal.
        self.assert_fixture_scope(
            {
                "unit": unit,
                "group": observed["ControlGroup"],
                "invocation": observed["InvocationID"],
            },
            live=False,
        )
        subprocess.run(
            [
                "systemctl",
                "--user",
                "kill",
                "--kill-whom=all",
                "--signal=SIGKILL",
                unit,
            ],
            env=surreal_server.systemd_environment(),
            check=False,
            capture_output=True,
            timeout=10,
        )

    def builder(self, stage: Path, _work: Path) -> None:
        (stage / "lib").mkdir()
        (stage / "lib/a").write_bytes(b"archive")
        (stage / "include").mkdir()
        (stage / "include/a.h").write_bytes(b"interface")

    def test_new_operation_verifies_bytes_nested_use_reuses_admission(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(
                operation,
                "scope_owner",
                return_value={
                    "unit": "pse-native-" + "a" * 32 + ".scope",
                    "group": "/scope",
                    "invocation": "b" * 32,
                },
            ),
            patch.object(operation, "populated", return_value=True),
        ):
            base = Path(directory)
            with (
                operation.Operation(base),
                patch.object(
                    cache, "installation_files", wraps=cache.installation_files
                ) as inventory,
            ):
                first = cache.prepare(
                    base, "fixture", {"id": 1}, ("lib/a", "include/a.h"), self.builder
                )
                count = inventory.call_count
                self.assertEqual(
                    cache.prepare(
                        base,
                        "fixture",
                        {"id": 1},
                        ("lib/a", "include/a.h"),
                        self.builder,
                    ),
                    first,
                )
                self.assertEqual(inventory.call_count, count)
            (first / "include/a.h").chmod(0o644)
            (first / "include/a.h").write_bytes(b"corrupt interface")
            with operation.Operation(base):
                replacement = cache.prepare(
                    base, "fixture", {"id": 1}, ("lib/a", "include/a.h"), self.builder
                )
            self.assertNotEqual(first, replacement)
            self.assertEqual((first / "include/a.h").read_bytes(), b"corrupt interface")
            self.assertTrue(
                cache.valid(replacement, {"id": 1}, ("lib/a", "include/a.h"))
            )

    def test_existing_reader_does_not_take_construction_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            expected = cache.prepare(
                base, "fixture", {"id": 1}, ("lib/a",), self.builder
            )
            owner = cache.identity_location(base, "fixture", {"id": 1})
            with (owner / ".build.lock").open("a") as construction:
                fcntl.flock(construction, fcntl.LOCK_EX)
                self.assertEqual(
                    cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder),
                    expected,
                )

    def test_existing_writable_generation_is_sealed_without_rebuild(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            prefix = cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder)
            original = cache.installation_files(prefix)
            for path in (*prefix.rglob("*"), prefix):
                path.chmod(path.stat().st_mode | 0o222)
            self.assertTrue(cache.valid(prefix, {"id": 1}, ("lib/a",)))
            with patch.object(
                self,
                "builder",
                side_effect=AssertionError("historical generation rebuilt"),
            ):
                self.assertEqual(
                    cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder),
                    prefix,
                )
            self.assertEqual(cache.installation_files(prefix), original)
            self.assertTrue(
                all(
                    not path.stat().st_mode & 0o222
                    for path in (*prefix.rglob("*"), prefix)
                )
            )
            (prefix / "include/a.h").chmod(0o644)
            with patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(base)}):
                cache.admit_external(prefix, ("lib/a",))
            self.assertFalse((prefix / "include/a.h").stat().st_mode & 0o222)
            self.assertEqual(cache.installation_files(prefix), original)

    def test_manual_shell_exports_verify_each_use_without_fast_authority(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)

            def solver(selected: Path) -> Path:
                return cache.prepare(
                    selected, "fixture", {"id": 1}, ("lib/a",), self.builder
                )

            with (
                patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(base)}, clear=True),
                patch.object(operation, "scope_owner", return_value=None),
                patch.object(cache, "SOLVER_FILES", ("lib/a",)),
                patch.object(cache, "solver", side_effect=solver),
                patch.object(cache, "runtime_env", return_value={}),
                patch.object(
                    build_environment, "configure", side_effect=lambda _root, env: env
                ),
                patch.object(
                    cache, "installation_files", wraps=cache.installation_files
                ) as inventory,
                patch.object(
                    sys,
                    "argv",
                    ["native_operation", "--capabilities", "solver", "--shell"],
                ),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(operation.main(), 0)
                count = inventory.call_count
                self.assertEqual(operation.main(), 0)
                self.assertGreater(inventory.call_count, count)
                self.assertIsNone(operation.owner_record())
                self.assertIsNone(operation.remembered("capability:solver"))
            guards = [
                json.loads(path.read_text())
                for path in (base / ".operations").glob("*.json")
            ]
            self.assertTrue(guards)
            self.assertTrue(
                all(
                    record["scope"] is None and record["generations"]
                    for record in guards
                )
            )

    def test_manual_math_source_requests_only_klu_and_isolation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            interpreter = Path(directory) / "setup-python"
            arguments = Path(directory) / "arguments"
            interpreter.write_text(
                '#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$NATIVE_FIXTURE_ARGUMENTS"\n'
            )
            interpreter.chmod(0o755)
            subprocess.run(
                ["bash", "-c", "source scripts/native-math-env.sh"],
                cwd=cache.ROOT,
                env={
                    **os.environ,
                    "PSE_NATIVE_SETUP_PYTHON": str(interpreter),
                    "NATIVE_FIXTURE_ARGUMENTS": str(arguments),
                },
                check=True,
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(
                arguments.read_text().splitlines()[1:],
                ["--capabilities", "klu,isolation", "--shell"],
            )

    def test_managed_primary_service_requires_actual_group_and_invocation(self) -> None:
        unit = "pse-surreal-worker-" + "a" * 16 + "-0.service"
        group = "/user.slice/pse-reference-fixture.slice/" + unit
        observation = {
            "LoadState": "loaded",
            "ControlGroup": group,
            "InvocationID": "b" * 32,
        }
        with (
            patch.object(
                operation, "process_group", return_value=group + "/native-child"
            ),
            patch.object(operation, "unit_observation", return_value=observation),
        ):
            self.assertEqual(
                operation.scope_owner(),
                {"unit": unit, "group": group, "invocation": "b" * 32},
            )
        for invalid in (
            {**observation, "ControlGroup": "/another"},
            {**observation, "InvocationID": "unknown"},
        ):
            with (
                patch.object(operation, "process_group", return_value=group),
                patch.object(operation, "unit_observation", return_value=invalid),
            ):
                self.assertIsNone(operation.scope_owner())

    def test_marker_requires_actual_scope_membership_and_generation(self) -> None:
        unit = "pse-native-" + "a" * 32 + ".scope"
        owner = {"unit": unit, "group": "/user.slice/" + unit, "invocation": "b" * 32}
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=owner),
            operation.Operation(Path(directory)) as active,
        ):
            self.assertEqual(operation.current(), active.path)
            operation.admit("receipt", {"value": 1})
            with patch.object(
                operation,
                "scope_owner",
                return_value={**owner, "invocation": "c" * 32},
            ):
                self.assertIsNone(operation.current())
                self.assertIsNone(operation.remembered("receipt"))
            with patch.object(operation, "scope_owner", return_value=None):
                self.assertIsNone(operation.current())
        with (
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(
                operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": owner["group"],
                    "InvocationID": owner["invocation"],
                },
            ),
        ):
            self.assertEqual(operation.scope_owner(), owner)
        with (
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(
                operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": "/other",
                    "InvocationID": owner["invocation"],
                },
            ),
        ):
            self.assertIsNone(operation.scope_owner())

    def test_parent_death_populated_scope_stays_pinned_unknown_owner_retained(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            first = cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder)
            (first / "lib/a").chmod(0o644)
            (first / "lib/a").write_bytes(b"changed")
            cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder)
            records = base / ".operations"
            for record in records.glob("*.json"):
                record.unlink()
            owner = {
                "unit": "pse-native-" + "a" * 32 + ".scope",
                "group": "/scope",
                "invocation": "b" * 32,
            }
            operation.write_json(
                records / "dead-parent.json",
                {
                    "version": operation.VERSION,
                    "pid": 999999999,
                    "start": "dead",
                    "scope": owner,
                    "admissions": {},
                    "generations": [str(first)],
                },
            )
            with patch.object(operation, "populated", return_value=True):
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
            with patch.object(
                operation, "populated", side_effect=ValueError("unavailable")
            ):
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
            with (
                patch.object(operation, "populated", return_value=False),
                patch.object(
                    operation,
                    "unit_observation",
                    return_value={"LoadState": "not-found"},
                ),
            ):
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [first])

    def test_external_lookalike_receipt_never_grants_owned_reuse(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            prefix = base / "lookalike/generations/fake"
            prefix.mkdir(parents=True)
            self.builder(prefix, base)
            (prefix / ".complete.json").write_text(
                json.dumps(
                    {
                        "version": cache.RECEIPT_VERSION,
                        "identity": {"id": 1},
                        "files": cache.installation_files(prefix),
                    }
                )
            )
            with (
                operation.Operation(base),
                patch.object(
                    cache, "installation_files", wraps=cache.installation_files
                ) as inventory,
            ):
                cache.admit_external(prefix, ("lib/a",))
                cache.admit_external(prefix, ("lib/a",))
                self.assertEqual(inventory.call_count, 2)
                (prefix / "include/a.h").write_bytes(b"changed")
                with self.assertRaisesRegex(ValueError, "changed inside"):
                    cache.admit_external(prefix, ("lib/a",))

    def test_highs_candidate_uses_complete_cargo_association_before_build(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            high = base / "actual-cargo-out"
            (high / "lib").mkdir(parents=True)
            (high / "include/highs").mkdir(parents=True)
            (high / "lib/libhighs.a").write_bytes(b"actual archive")
            (high / "include/highs/Highs.h").write_bytes(b"actual header")
            env = {"PSE_NATIVE_CACHE": str(base)}
            inputs = {"unit": "actual locked feature profile"}
            expected = pipeline.highs_files(high)
            env["PSE_NATIVE_PROVIDER_RECEIPT"] = str(base / "reviewed-provider.json")
            with (
                patch.object(
                    pipeline,
                    "_build_highs_archive",
                    return_value=(high, "registry#highs-sys@1.15.0"),
                ) as build,
                patch.object(
                    producer_deployment,
                    "verify_native_provider",
                    side_effect=lambda *_: pipeline.highs_files(high) == expected,
                    create=True,
                ) as verify,
            ):
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(build.call_count, 1)
                self.assertEqual(
                    verify.call_args.args[:3],
                    (
                        Path(env["PSE_NATIVE_PROVIDER_RECEIPT"]),
                        "registry#highs-sys@1.15.0",
                        high,
                    ),
                )
                (high / "include/highs/Highs.h").write_bytes(b"changed header")
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(build.call_count, 2)
                del env["PSE_NATIVE_PROVIDER_RECEIPT"]
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(
                    build.call_count,
                    4,
                    "unqualified dev existence cannot grant persistent reuse",
                )

    def test_child_scope_handoff_protects_launch_window_and_new_invocation(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            unit = "pse-surreal-worker-" + "a" * 16 + "-0.scope"
            with operation.Operation(base):
                generation = cache.prepare(
                    base, "fixture", {"id": 1}, ("lib/a",), self.builder
                )
                handoff = operation.prepare_handoff(unit)
            self.assertIsNotNone(handoff)
            self.assertEqual(
                operation._record(handoff)["generations"],  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
                [str(generation)],
            )
            self.assertFalse(operation.drained(operation._record(handoff)))  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
            child_owner = {"unit": unit, "group": "/worker", "invocation": "b" * 32}
            with patch.object(operation, "scope_owner", return_value=child_owner):
                operation.bind_handoff(handoff)
            self.assertEqual(operation._record(handoff)["scope"], child_owner)  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
            with patch.object(operation, "populated", return_value=True):
                self.assertFalse(operation.drained(operation._record(handoff)))  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
            with (
                patch.object(
                    operation,
                    "scope_owner",
                    return_value={**child_owner, "invocation": "c" * 32},
                ),
                self.assertRaisesRegex(ValueError, "differs"),
            ):
                operation.bind_handoff(handoff)

    def test_cancellation_drains_authentic_scope_and_fallback_signals_group(
        self,
    ) -> None:
        child = type("Child", (), {"pid": 12345})()
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/scope",
            "invocation": "b" * 32,
        }
        with (
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(operation.subprocess, "run") as manager,
            patch.object(operation.os, "killpg") as group,
        ):
            operation.cancel_children(child, 15)
            self.assertEqual(manager.call_args.args[0][-1], owner["unit"])
            self.assertIn("--kill-whom=all", manager.call_args.args[0])
            group.assert_called_once_with(child.pid, 15)
        with (
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(operation.subprocess, "run") as manager,
            patch.object(operation.os, "killpg") as group,
        ):
            operation.cancel_children(child, 2)
            manager.assert_not_called()
            group.assert_called_once_with(child.pid, 2)

    def test_foreground_pending_handoff_consumes_parent_cancellation_fence(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            unit = "pse-native-" + "a" * 32 + ".scope"
            with operation.Operation(base) as parent:
                handoff = operation.prepare_handoff(unit, foreground=True)
                assert handoff is not None
                record = json.loads(parent.path.read_text())
                record["cancelled"] = True
                operation.write_json(parent.path, record)
                self.assertNotIn("cancelled", json.loads(handoff.read_text()))
                child = {"unit": unit, "group": "/foreground", "invocation": "b" * 32}
                with (
                    patch.object(operation, "scope_owner", return_value=child),
                    patch.object(pse_env.host, "group_identity", return_value=17),
                    self.assertRaisesRegex(ValueError, "cancelled"),
                ):
                    operation.bind_handoff(handoff)
                bound = json.loads(handoff.read_text())
                self.assertEqual(bound["scope"], child)
                self.assertEqual(bound["scope_inode"], 17)
                self.assertTrue(bound["cancelled"])
                with self.assertRaisesRegex(ValueError, "parent is cancelled"):
                    operation.prepare_handoff(unit, foreground=True)
                # Persistent primary/worker handoffs remain independently owned.
                primary = operation.prepare_handoff(
                    "pse-surreal-worker-" + "d" * 16 + "-0.service"
                )
                assert primary is not None
                self.assertNotIn("foreground", json.loads(primary.read_text()))

    def test_cancellation_manager_failure_still_signals_direct_payload(self) -> None:
        child = type("Child", (), {"pid": 12345})()
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/scope",
            "invocation": "b" * 32,
        }
        with (
            patch.object(
                operation,
                "cancel_foreground_handoffs",
                side_effect=ValueError("unknown child"),
            ),
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(
                operation.subprocess,
                "run",
                side_effect=subprocess.TimeoutExpired("systemctl", 10),
            ),
            patch.object(operation.os, "killpg") as group,
            contextlib.redirect_stderr(io.StringIO()) as errors,
        ):
            with self.assertRaisesRegex(ValueError, "cancellation incomplete"):
                operation.cancel_children(child, signal.SIGTERM)
            group.assert_called_once_with(child.pid, signal.SIGTERM)
            self.assertIn("unknown child", errors.getvalue())
            self.assertIn("scope cancellation failed", errors.getvalue())

    def test_creator_completion_restores_markers_when_foreground_drain_fails(
        self,
    ) -> None:
        previous = {
            key: os.environ.get(key)
            for key in (operation.MARKER, "PSE_NATIVE_CACHE", "PSE_NATIVE_SETUP_PYTHON")
        }
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(
                operation,
                "cancel_foreground_handoffs",
                side_effect=ValueError("unknown foreground lifetime"),
            ) as settle,
        ):
            with (
                self.assertRaisesRegex(ValueError, "unknown foreground lifetime"),
                operation.Operation(Path(directory)) as parent,
            ):
                self.assertEqual(os.environ[operation.MARKER], str(parent.path))
            settle.assert_called_once()
        self.assertEqual({key: os.environ.get(key) for key in previous}, previous)

    def test_nested_run_completion_preserves_the_live_creator(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            with operation.Operation(Path(directory)) as parent:
                handoff = operation.prepare_handoff(
                    "pse-native-" + "a" * 32 + ".scope", foreground=True
                )
                assert handoff is not None
                with patch.object(operation, "cancel_foreground_handoffs") as settle:
                    self.assertEqual(
                        operation.run([sys.executable, "-c", "pass"], dict(os.environ)),
                        0,
                    )
                    settle.assert_not_called()
                self.assertNotIn("cancelled", json.loads(parent.path.read_text()))
                self.assertNotIn("cancelled", json.loads(handoff.read_text()))
            self.assertTrue(json.loads(handoff.read_text())["cancelled"])

    def test_foreground_cancellation_malformed_associations_refuse_conservatively(
        self,
    ) -> None:
        for children in (17, [None]):
            with (
                self.subTest(children=children),
                tempfile.TemporaryDirectory() as directory,
                patch.object(operation, "scope_owner", return_value=None),
                operation.Operation(Path(directory)) as parent,
            ):
                record = dict(json.loads(parent.path.read_text()))
                record["foreground_children"] = children
                operation.write_json(parent.path, record)
                with patch.object(operation.subprocess, "run") as manager:
                    with self.assertRaises(ValueError):
                        operation.cancel_foreground_handoffs()
                    manager.assert_not_called()
                self.assertTrue(json.loads(parent.path.read_text())["cancelled"])
                # Unknown associations also refuse creator finalization.
                record["foreground_children"] = []
                operation.write_json(parent.path, record)

    def test_foreground_cancellation_fences_every_child_and_refuses_replacements(
        self,
    ) -> None:
        for replacement in (
            "invocation",
            "group",
            "inode",
            "missing",
            "missing-scope",
            "invalid-scope",
        ):
            with (
                self.subTest(replacement=replacement),
                tempfile.TemporaryDirectory() as directory,
                patch.object(operation, "scope_owner", return_value=None),
            ):
                base = Path(directory)
                stopped = []
                with (
                    self.assertRaisesRegex(ValueError, "cancellation incomplete")
                    if replacement in {"missing", "missing-scope", "invalid-scope"}
                    else contextlib.nullcontext(),
                    operation.Operation(base),
                ):
                    paths = []
                    owners = []
                    for letter in ("a", "b"):
                        unit = "pse-native-" + letter * 32 + ".scope"
                        path = operation.prepare_handoff(unit, foreground=True)
                        assert path is not None
                        child = {
                            "unit": unit,
                            "group": "/" + letter,
                            "invocation": letter * 32,
                        }
                        with (
                            patch.object(operation, "scope_owner", return_value=child),
                            patch.object(
                                pse_env.host, "group_identity", return_value=17
                            ),
                        ):
                            operation.bind_handoff(path)
                        paths.append(path)
                        owners.append(child)
                    durable = operation.prepare_handoff(
                        "pse-surreal-worker-" + "d" * 16 + "-0.service"
                    )
                    assert durable is not None
                    if replacement == "missing":
                        paths[0].unlink()
                    elif replacement in {"missing-scope", "invalid-scope"}:
                        record = dict(json.loads(paths[0].read_text()))
                        if replacement == "missing-scope":
                            del record["scope"]
                        else:
                            record["scope"] = 17
                        operation.write_json(paths[0], record)

                    def observe(
                        unit: str,
                        owners: list[dict[str, str]] = owners,
                        replacement: str = replacement,
                    ) -> dict[str, str]:
                        selected = next(
                            owner for owner in owners if owner["unit"] == unit
                        )
                        result = {
                            "LoadState": "loaded",
                            "ActiveState": "active",
                            "ControlGroup": selected["group"],
                            "InvocationID": selected["invocation"],
                        }
                        if unit == owners[0]["unit"] and replacement in {
                            "invocation",
                            "group",
                        }:
                            result[
                                "InvocationID"
                                if replacement == "invocation"
                                else "ControlGroup"
                            ] = "replaced"
                        return result

                    def stop(
                        command: list[str],
                        paths: list[Path] = paths,
                        stopped: list[str] = stopped,
                        **_kwargs: object,
                    ) -> None:
                        self.assertTrue(
                            all(
                                json.loads(path.read_text())["cancelled"]
                                for path in paths
                                if path.exists()
                            )
                        )
                        stopped.append(command[-1])

                    with (
                        patch.object(
                            operation,
                            "drained",
                            side_effect=lambda record, stopped=stopped: (
                                record["scope"]["unit"] in stopped
                            ),
                        ),
                        patch.object(
                            operation, "unit_observation", side_effect=observe
                        ),
                        patch.object(
                            pse_env.host,
                            "group_identity",
                            side_effect=lambda group, replacement=replacement: (
                                18 if replacement == "inode" and group == "/a" else 17
                            ),
                        ),
                        patch.object(operation.subprocess, "run", side_effect=stop),
                        self.assertRaisesRegex(ValueError, "cancellation incomplete"),
                    ):
                        operation.cancel_foreground_handoffs()
                    self.assertEqual(stopped, [owners[1]["unit"]])
                    self.assertNotIn("cancelled", json.loads(durable.read_text()))

    def test_foreground_handoff_collector_retains_binding_until_parent_retires(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            with operation.Operation(base) as parent:
                path = operation.prepare_handoff(
                    "pse-native-" + "a" * 32 + ".scope", foreground=True
                )
                assert path is not None
                record = json.loads(path.read_text())
                record["scope"] = {
                    "unit": record["handoff"],
                    "group": "/child",
                    "invocation": "b" * 32,
                }
                record["generations"] = ["settled-child-only"]
                operation.write_json(path, record)
                with patch.object(
                    operation,
                    "drained",
                    side_effect=lambda record: record["scope"] is not None,
                ):
                    self.assertEqual(operation.pinned_generations(base), set())
                    self.assertTrue(path.exists())
                    parent.path.unlink()
                    self.assertEqual(operation.pinned_generations(base), set())
                    self.assertFalse(path.exists())

    def test_record_exclusion_defers_cancellation_until_flock_is_released(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "operation.json"
            received = []

            def cancel(_signum: int, _frame: object) -> None:
                with operation.record_lock(path):
                    received.append("cancelled")

            previous = signal.signal(signal.SIGTERM, cancel)
            try:
                with operation.record_lock(path):
                    os.kill(os.getpid(), signal.SIGTERM)
                    self.assertEqual(received, [])
                self.assertEqual(received, ["cancelled"])
            finally:
                signal.signal(signal.SIGTERM, previous)

    def test_thread_budget_defaults_to_one_and_honours_the_caller(self) -> None:
        defaulted = operation.environment([], {})
        self.assertEqual(
            {name: defaulted[name] for name in operation.THREAD_VARIABLES},
            dict.fromkeys(operation.THREAD_VARIABLES, "1"),
        )
        chosen = operation.environment(
            [], {"OMP_NUM_THREADS": "4", "MKL_NUM_THREADS": "off"}
        )
        self.assertEqual(chosen["OMP_NUM_THREADS"], "4")
        self.assertEqual(chosen["OPENBLAS_NUM_THREADS"], "1")
        self.assertNotIn("MKL_NUM_THREADS", chosen)

    def test_off_survives_nested_native_setup_until_a_value_is_chosen(self) -> None:
        outer = operation.environment([], {"OMP_NUM_THREADS": "off"})
        self.assertNotIn("OMP_NUM_THREADS", outer)
        nested = operation.environment([], outer)
        self.assertNotIn("OMP_NUM_THREADS", nested)
        self.assertEqual(nested["OPENBLAS_NUM_THREADS"], "1")
        chosen = operation.environment([], {**nested, "OMP_NUM_THREADS": "6"})
        self.assertEqual(chosen["OMP_NUM_THREADS"], "6")
        self.assertNotIn(operation.OFF_MARKER, chosen)

    def test_solver_runtime_enforces_owned_keys_and_defaults_the_rest(self) -> None:
        runtime = {
            "MKL_CBWR": "COMPATIBLE",
            "OMP_PLACES": "sockets",
            "OMP_NUM_THREADS": "1",
        }
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(cache, "solver", return_value=Path(directory)),
            patch.object(cache, "runtime_env", return_value=runtime),
            patch("sys.stderr", new_callable=io.StringIO) as notices,
        ):
            result = operation.environment(
                ["solver"],
                {"MKL_CBWR": "AUTO", "OMP_PLACES": "cores", "OMP_NUM_THREADS": "8"},
            )
        self.assertEqual(result["MKL_CBWR"], "COMPATIBLE")
        self.assertEqual(result["OMP_PLACES"], "cores")
        self.assertEqual(result["OMP_NUM_THREADS"], "8")
        self.assertEqual(result["SCIPOPTDIR"], directory)
        self.assertIn("pse-env: refused MKL_CBWR=AUTO", notices.getvalue())
        self.assertNotIn("OMP_PLACES", notices.getvalue())

    def test_signal_death_reports_shell_status(self) -> None:
        self.assertEqual(
            operation.run(["sh", "-c", "kill -TERM $$"], dict(os.environ)),
            128 + signal.SIGTERM,
        )

    def test_build_timing_observes_authentic_setup_child_and_scope_drain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            environment = self.fixture_environment(output / "cache")
            environment[operation.BUILD_TIMING] = str(output / "native-operation.json")
            program = "from scripts import native_operation as n; import os; assert n.current() is not None; assert n.BUILD_TIMING not in os.environ"
            command = self.fixture_command(
                [
                    sys.executable,
                    "-m",
                    "scripts.native_operation",
                    "--capabilities",
                    "",
                    "--",
                    sys.executable,
                    "-c",
                    program,
                ],
                environment,
            )
            started = time.monotonic()
            result = subprocess.run(
                command, cwd=cache.ROOT, env=environment, check=False
            )
            native = json.loads((output / "native-operation.json").read_text())
            self.assertEqual(result.returncode, 0)
            self.assertEqual(native["requested_capabilities"], [])
            self.assert_fixture_scope(native["owner"]["scope"], live=False)
            # Mirror pse-env's outer launcher: release the exact fixture's empty
            # allocation before asking the native owner to prove terminal drain.
            self.assertFalse(operation.populated(native["owner"]["scope"]["group"]))
            allocation = pse_env.host.inherit(environment)
            self.assertIsNotNone(allocation)
            assert allocation is not None
            self.assertTrue(allocation.release())
            self.await_drain(native["owner"])
            self.assertEqual(
                build_measurements.complete_operation(output, started, 0), 0
            )
            receipt = json.loads((output / "complete-operation.json").read_text())
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["phases"]["final_drain"]["status"], "passed")
            self.assertGreaterEqual(receipt["wall_seconds"], 0)
            self.assertEqual(native["setup_status"], "passed")
            self.assertEqual(native["child_exit_code"], 0)

    def test_actual_zero_capability_entry_owns_child_before_command(self) -> None:
        program = "from scripts import native_operation as n; import json; p=n.owner_record(); print(json.dumps({'active':p is not None,'scope':n._record(p)['scope'],'args':__import__('sys').argv[1:]}))"
        with tempfile.TemporaryDirectory() as directory:
            environment = self.fixture_environment(Path(directory))
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native=",
                    "--",
                    sys.executable,
                    "-c",
                    program,
                    "space ; literal",
                    "$literal",
                ],
                environment,
            )
            result = subprocess.run(
                command,
                cwd=cache.ROOT,
                env=environment,
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
        observed = json.loads(result.stdout)
        self.assertTrue(observed["active"])
        self.assertEqual(observed["args"], ["space ; literal", "$literal"])
        self.assertIsNotNone(observed["scope"])
        self.assert_fixture_scope(observed["scope"], live=False)
        self.assertTrue(observed["scope"]["unit"].startswith("pse-native-"))
        self.assertEqual(len(observed["scope"]["invocation"]), 32)
        self.await_drain({"scope": observed["scope"]})

    def test_handcrafted_marker_and_incomplete_capability_cannot_grant_admission(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            forged = Path(directory) / "arbitrary.json"
            operation.write_json(
                forged,
                {
                    "version": operation.VERSION,
                    "pid": os.getpid(),
                    "start": operation.start_identity(os.getpid()),
                    "scope": None,
                    "admissions": {"capability:solver": {"inputs": {}}},
                    "generations": [],
                },
            )
            with patch.dict(os.environ, {operation.MARKER: str(forged)}):
                self.assertIsNone(operation.current())
                self.assertIsNone(operation.owner_record())
            owner = {
                "unit": "pse-native-" + "a" * 32 + ".scope",
                "group": "/scope",
                "invocation": "b" * 32,
            }
            with (
                patch.object(operation, "scope_owner", return_value=owner),
                operation.Operation(Path(directory)),
            ):
                operation.admit("capability:solver", {"inputs": {}})
                self.assertEqual(
                    operation.remembered("capability:solver"), {"inputs": {}}
                )
            with (
                patch.object(operation, "scope_owner", return_value=None),
                operation.Operation(Path(directory)),
            ):
                self.assertIsNotNone(operation.owner_record())
                self.assertIsNone(operation.current())
                operation.admit("observation:forged-fast-hit", {"value": True})
                self.assertIsNone(operation.remembered("observation:forged-fast-hit"))

    def test_native_entry_rejects_forged_marker_and_preserves_arguments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            forged = base / "arbitrary.json"
            operation.write_json(
                forged,
                {
                    "version": operation.VERSION,
                    "pid": os.getpid(),
                    "start": operation.start_identity(os.getpid()),
                    "scope": None,
                    "admissions": {},
                    "generations": [],
                },
            )
            environment = {
                **self.fixture_environment(base),
                operation.MARKER: str(forged),
            }
            result = subprocess.run(
                self.fixture_command(
                    [
                        str(cache.ROOT / "scripts/pse-env"),
                        "--native=",
                        "--",
                        str(cache.ROOT / ".venv/bin/python"),
                        "-c",
                        'from scripts import native_operation as n; import sys,json; print(json.dumps({"record":str(n.owner_record()),"scope":n.scope_owner(),"args":sys.argv[1:]}))',
                        "space ; literal",
                        "$literal",
                    ],
                    environment,
                ),
                cwd=cache.ROOT,
                env=environment,
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            observed = json.loads(result.stdout)
            self.assertNotEqual(observed["record"], str(forged))
            self.assertEqual(Path(observed["record"]).parent, base / ".operations")
            self.assertEqual(observed["args"], ["space ; literal", "$literal"])
            self.assertIsNotNone(observed["scope"])
            self.assert_fixture_scope(observed["scope"], live=False)

    def test_snapshot_without_venv_uses_selected_operation_interpreter(self) -> None:
        from scripts import pse_env  # noqa: PLC0415 -- the boundary that selects it

        with tempfile.TemporaryDirectory() as directory:
            snapshot = Path(directory)
            self.assertEqual(
                pse_env.setup_python(
                    snapshot, {"PSE_NATIVE_SETUP_PYTHON": "/chosen/python"}
                ),
                "/chosen/python",
            )
            self.assertEqual(pse_env.setup_python(snapshot, {}), sys.executable)

    def test_real_worker_outlives_launcher_then_generation_reclaims_after_drain(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "disposable-state"
            self.fixture_units.add(surreal_server.worker_unit(state, 0))
            child_program = """import json, sys, time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from scripts import native_operation as n
base=Path(sys.argv[1]); p=n.current(); record=n._record(p)
n.write_json(base/'worker-ready.json', {'scope':record['scope'],'operation':str(p),'original':(Path(sys.argv[2])/'lib/a').read_text()})
deadline=time.monotonic()+30
while not (base/'stop').exists() and time.monotonic()<deadline: time.sleep(.02)
"""
            launcher = """import json, os, sys, subprocess, time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from scripts import native_cache as c, native_operation as n, surreal_server as s, host_admission as h
base=Path(sys.argv[1]); state=base/'disposable-state'
def builder(stage, work):
 (stage/'lib').mkdir(); (stage/'lib/a').write_text('original archive')
generation=c.prepare(base,'fixture',{'id':1},('lib/a',),builder)
unit=s.worker_unit(state,0); owner=h.inherit(os.environ); assert owner is not None; owner.register(unit); handoff=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':256*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[2],str(base),str(generation)],capabilities=())
worker=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,handoff))
n.write_json(base/'launcher-ready.json',{'scope':n.scope_owner(),'generation':str(generation),'handoff':str(handoff)})
while worker.poll() is None: time.sleep(.02)
"""
            log = (base / "fixture.log").open("w")
            self.addCleanup(log.close)
            environment = self.fixture_environment(base)
            process = subprocess.Popen(
                self.fixture_command(
                    [
                        str(cache.ROOT / "scripts/pse-env"),
                        "--native",
                        "--",
                        sys.executable,
                        "-c",
                        launcher,
                        str(base),
                        child_program,
                    ],
                    environment,
                ),
                cwd=cache.ROOT,
                env=environment,
                stdout=log,
                stderr=log,
            )
            parent = None
            worker = None
            try:
                parent = self.await_file(base / "launcher-ready.json", process)
                worker = self.await_file(base / "worker-ready.json", process)
                self.assertIsNotNone(
                    parent["scope"],
                    "genuine lifecycle control requires the local user manager",
                )
                self.assertEqual(
                    worker["scope"]["unit"], surreal_server.worker_unit(state, 0)
                )
                self.assert_fixture_scope(parent["scope"])
                self.assert_fixture_scope(worker["scope"])
                self.assertNotEqual(parent["scope"]["unit"], worker["scope"]["unit"])
                self.assertEqual(worker["original"], "original archive")
                handoff = operation._record(Path(parent["handoff"]))  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
                self.assertEqual(handoff["scope"], worker["scope"])
                self.kill_scope(parent["scope"]["unit"])
                process.wait(timeout=10)
                self.assertTrue(operation.populated(worker["scope"]["group"]))
                generation = Path(parent["generation"])
                self.assertEqual(generation.stat().st_mode & 0o222, 0)
                # Explicit privileged corruption exercises new admission. The
                # surviving child already owns the original escaped fixture bytes.
                (generation / "lib/a").chmod(0o644)
                (generation / "lib/a").write_text("corrupt archive")
                repair = """import json,sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from scripts import native_cache as c, native_operation as n
def builder(stage,work):
 (stage/'lib').mkdir(); (stage/'lib/a').write_text('original archive')
generation=c.prepare(Path(sys.argv[1]),'fixture',{'id':1},('lib/a',),builder)
print(json.dumps({'generation':str(generation),'scope':n.scope_owner()}))
"""
                environment = self.fixture_environment(base)
                admitted = subprocess.run(
                    self.fixture_command(
                        [
                            str(cache.ROOT / "scripts/pse-env"),
                            "--native",
                            "--",
                            sys.executable,
                            "-c",
                            repair,
                            str(base),
                        ],
                        environment,
                    ),
                    cwd=cache.ROOT,
                    env=environment,
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=20,
                )
                repaired = json.loads(admitted.stdout)
                self.assertIsNotNone(repaired["scope"])
                self.assert_fixture_scope(repaired["scope"], live=False)
                self.assertNotEqual(repaired["scope"]["unit"], parent["scope"]["unit"])
                self.assertNotEqual(repaired["scope"]["unit"], worker["scope"]["unit"])
                replacement = Path(repaired["generation"])
                self.assertNotEqual(generation, replacement)
                self.assertTrue(cache.valid(replacement, {"id": 1}, ("lib/a",)))
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
                (base / "stop").touch()
                self.await_drain(handoff)
                self.assertEqual(
                    cache.collect(base, "fixture", {"id": 1}), [generation]
                )
                self.assert_parent_scope_unchanged()
            finally:
                (base / "stop").touch()
                for owner in (worker, parent):
                    if owner and owner.get("scope"):
                        self.kill_scope(owner["scope"]["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()

    def test_real_cancellation_drains_descendant_that_changed_session(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            daemon = "import time; time.sleep(30)"
            program = """import sys,subprocess,time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from scripts import native_operation as n
base=Path(sys.argv[1]); descendant=subprocess.Popen([sys.executable,'-c',sys.argv[2]],start_new_session=True)
record=n._record(n.current()); n.write_json(base/'cancel-ready.json',{'record':record,'descendant':descendant.pid,'group':n.process_group(descendant.pid)})
time.sleep(30)
"""
            log = (base / "cancel.log").open("w")
            self.addCleanup(log.close)
            environment = self.fixture_environment(base)
            process = subprocess.Popen(
                self.fixture_command(
                    [
                        str(cache.ROOT / "scripts/pse-env"),
                        "--native",
                        "--",
                        sys.executable,
                        "-c",
                        program,
                        str(base),
                        daemon,
                    ],
                    environment,
                ),
                cwd=cache.ROOT,
                env=environment,
                stdout=log,
                stderr=log,
            )
            observed = None
            try:
                observed = self.await_file(base / "cancel-ready.json", process)
                record = observed["record"]
                self.assertIsNotNone(record["scope"])
                self.assert_fixture_scope(record["scope"])
                self.assertEqual(observed["group"], record["scope"]["group"])
                self.assertEqual(
                    operation.process_group(record["pid"]), record["scope"]["group"]
                )
                os.kill(record["pid"], signal.SIGTERM)
                process.wait(timeout=15)
                self.await_drain(record)
                self.assertFalse(operation.populated(record["scope"]["group"]))
                self.assert_parent_scope_unchanged()
            finally:
                if observed and observed["record"].get("scope"):
                    self.kill_scope(observed["record"]["scope"]["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()

    def test_real_assessment_cancellation_drains_nested_observers_preserves_storage_unit(
        self,
    ) -> None:
        # Real manager lifetimes exercise cancellation; the independent storage
        # role is a disposable process, not a scientific database durability claim.
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "disposable-state"
            state.mkdir()
            self.fixture_units.add(surreal_server.worker_unit(state, 0))
            runner = """import os,signal,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); signal.signal(signal.SIGTERM,signal.SIG_IGN)
n.write_json(base/'runner-ready.json',{'record':n._record(n.current()),'pid':os.getpid(),'session':os.getsid(0)})
time.sleep(30)
"""
            storage = """import os,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); n.write_json(base/'storage-ready.json',{'record':n._record(n.current()),'pid':os.getpid()})
deadline=time.monotonic()+30
while not (base/'stop').exists() and time.monotonic()<deadline: time.sleep(.02)
"""
            # The fixture owns zero native capabilities. Command construction,
            # native handoff/binding, manager membership and stop are production.
            foreground = """
def foreground(program, arguments):
 unit='pse-native-'+uuid.uuid4().hex+'.scope'; owner=h.inherit(os.environ); owner.register(unit)
 handoff=n.prepare_handoff(unit,foreground=True)
 allocation={'execution':{'cpu_threads':1,'observer_memory_bytes':128*1024**2}}
 command=s.observer_scope_command(state,allocation,unit,[sys.executable,'-c',program,*arguments])
 command[command.index('--capabilities')+1]=''
 env=s.systemd_environment(); env.pop(n.MARKER,None); env['PSE_NATIVE_HANDOFF']=str(handoff)
 return subprocess.Popen(command,env=env),handoff
"""
            middle = (
                """import os,sys,subprocess,uuid
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h
base=Path(sys.argv[1]); state=base/'disposable-state'
"""
                + foreground
                + """
child,handoff=foreground(sys.argv[2],[str(base)])
n.write_json(base/'middle-ready.json',{'record':n._record(n.current()),'handoff':str(handoff),'pid':os.getpid()})
child.wait()
"""
            )
            launcher = (
                """import os,sys,subprocess,uuid
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h
base=Path(sys.argv[1]); state=base/'disposable-state'
"""
                + foreground
                + """
owner=h.inherit(os.environ); unit=s.worker_unit(state,0); owner.register(unit); durable=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':64*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[4],str(base)],capabilities=())
persistent=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,durable))
child,handoff=foreground(sys.argv[2],[str(base),sys.argv[3]])
n.write_json(base/'launcher-ready.json',{'record':n._record(n.current()),'handoff':str(handoff),'durable':str(durable)})
child.wait()
"""
            )
            environment = self.fixture_environment(base)
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native",
                    "--",
                    sys.executable,
                    "-c",
                    launcher,
                    str(base),
                    middle,
                    runner,
                    storage,
                ],
                environment,
            )
            errors = []

            def interrupt_when_running() -> None:
                try:
                    self.await_file(base / "runner-ready.json")
                    self.await_file(base / "storage-ready.json")
                except AssertionError as error:
                    errors.append(str(error))
                finally:
                    os.kill(os.getpid(), signal.SIGINT)

            interrupter = threading.Thread(target=interrupt_when_running)
            interrupter.start()
            observed = []
            try:
                result = validation.execute(
                    cache.ROOT, base, "interrupted-observer", command, environment
                )
                interrupter.join()
                self.assertEqual(
                    errors, [], (base / "interrupted-observer.log").read_text()[-4000:]
                )
                self.assertEqual(result["status"], "interrupted")
                parent = json.loads((base / "launcher-ready.json").read_text())
                middle_owner = json.loads((base / "middle-ready.json").read_text())
                actual_runner = json.loads((base / "runner-ready.json").read_text())
                persistent = json.loads((base / "storage-ready.json").read_text())
                observed = [parent, middle_owner, actual_runner, persistent]
                for item in observed:
                    owner = item["record"]["scope"]
                    self.fixture_units.add(owner["unit"])
                    self.assert_fixture_scope(owner, live=False)
                self.assertEqual(actual_runner["session"], actual_runner["pid"])
                self.assertNotEqual(
                    actual_runner["pid"], actual_runner["record"]["pid"]
                )
                for item in observed[:3]:
                    self.await_drain(item["record"])
                for handoff in (parent["handoff"], middle_owner["handoff"]):
                    self.await_drain(json.loads(Path(handoff).read_text()))
                process = Path(f"/proc/{actual_runner['pid']}/stat")
                self.assertTrue(
                    not process.exists()
                    or process.read_text().rsplit(")", 1)[1].split()[0] == "Z"
                )
                self.assertTrue(
                    operation.populated(persistent["record"]["scope"]["group"])
                )
                self.assert_fixture_scope(persistent["record"]["scope"])
                self.assertFalse(
                    operation.drained(json.loads(Path(parent["durable"]).read_text()))
                )
                (base / "stop").touch()
                self.await_drain(persistent["record"])
                self.assert_parent_scope_unchanged()
            finally:
                interrupter.join()
                (base / "stop").touch()
                for name in (
                    "runner-ready.json",
                    "middle-ready.json",
                    "storage-ready.json",
                    "launcher-ready.json",
                ):
                    path = base / name
                    if path.exists():
                        item = json.loads(path.read_text())
                        owner = item["record"]["scope"]
                        self.fixture_units.add(owner["unit"])
                        self.kill_scope(owner["unit"])

    def test_real_validation_term_finishes_creator_and_production_observer(
        self,
    ) -> None:
        # The production observer owns its handoff/environment/launch. Scientific
        # placement and exclusive-observer cap borrowing are separate controls.
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "state"
            state.mkdir(mode=0o700)
            surreal_server.write_json(
                state / "config.json",
                {
                    "owner": surreal_server.OWNER,
                    "profile_version": 2,
                    "websocket_max_message_bytes": surreal_server.MESSAGE_BYTES,
                    "max_message_bytes": surreal_server.MESSAGE_BYTES,
                    "resources": surreal_server.reference_resources(),
                    "port": 18241,
                    "endpoint": "ws://127.0.0.1:18241",
                    "interpretation": "fixture",
                    "schema_interpretation": "fixture",
                    "accepting_writes": True,
                    "admission": "open",
                },
            )
            self.fixture_units.add(surreal_server.worker_unit(state, 0))
            runner = """import os,signal,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); signal.signal(signal.SIGTERM,signal.SIG_IGN)
n.write_json(base/'production-runner.json',{'record':n._record(n.current()),'pid':os.getpid(),'session':os.getsid(0)})
time.sleep(30)
"""
            storage = """import os,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); n.write_json(base/'production-storage.json',{'record':n._record(n.current()),'pid':os.getpid()})
deadline=time.monotonic()+30
while not (base/'stop').exists() and time.monotonic()<deadline: time.sleep(.02)
"""
            gate = """import sys
from pathlib import Path
from unittest.mock import patch
from scripts import surreal_server as s
base=Path(sys.argv[1]); state=base/'state'; original=s.observer_scope_command
def fixture_command(state,allocation,unit,command):
 selected={'execution':{'cpu_threads':1,'observer_memory_bytes':128*1024**2}}
 result=original(state,selected,unit,command)
 result[result.index('--capabilities')+1]=''
 return result
with patch.object(s,'ensure_execution_placement'),patch.object(s,'observer_scope_command',side_effect=fixture_command):
 sys.exit(s.observer(state,[sys.executable,'-c',sys.argv[2],str(base)]))
"""
            validator = """import os,sys,signal,subprocess
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h,validation as v
base=Path(sys.argv[1]); state=base/'state'; owner=h.inherit(os.environ)
unit=s.worker_unit(state,0); owner.register(unit); durable=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':64*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[4],str(base)],capabilities=())
persistent=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,durable))
signal.signal(signal.SIGTERM,v.interrupt)
n.write_json(base/'production-validator.json',{'record':n._record(n.current()),'pid':os.getpid(),'durable':str(durable)})
result=v.execute(Path.cwd(),base,'production-gate',[sys.executable,'-c',sys.argv[2],str(base),sys.argv[3]],dict(os.environ))
n.write_json(base/'production-validation-complete.json',result)
sys.exit(128+v.RECEIVED_SIGNAL if result['status']=='interrupted' else result['exit_code'])
"""
            environment = self.fixture_environment(base)
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native",
                    "--",
                    sys.executable,
                    "-c",
                    validator,
                    str(base),
                    gate,
                    runner,
                    storage,
                ],
                environment,
            )
            log = (base / "fixture.log").open("w")
            process = subprocess.Popen(
                command,
                cwd=cache.ROOT,
                env=environment,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            observed = []
            try:
                actual_runner = self.await_file(
                    base / "production-runner.json", process
                )
                persistent = self.await_file(base / "production-storage.json", process)
                validation_owner = self.await_file(
                    base / "production-validator.json", process
                )
                observed = [actual_runner, persistent, validation_owner]
                for item in observed:
                    owner = item["record"]["scope"]
                    self.fixture_units.add(owner["unit"])
                    self.assert_fixture_scope(owner)
                self.assertNotEqual(
                    validation_owner["pid"], validation_owner["record"]["pid"]
                )
                self.assertEqual(actual_runner["session"], actual_runner["pid"])
                # TERM reaches the actual validation process, not its native creator.
                os.kill(validation_owner["pid"], signal.SIGTERM)
                process.wait(timeout=20)
                self.assertEqual(
                    process.returncode, 143, (base / "fixture.log").read_text()[-4000:]
                )
                completed = json.loads(
                    (base / "production-validation-complete.json").read_text()
                )
                self.assertEqual(completed["status"], "interrupted")
                for item in (actual_runner, validation_owner):
                    self.await_drain(item["record"])
                # The ready snapshot precedes observer registration; read the
                # actual creator record by its authenticated PID association.
                creators = [
                    json.loads(path.read_text())
                    for path in (base / ".operations").glob("*.json")
                    if json.loads(path.read_text()).get("pid")
                    == validation_owner["record"]["pid"]
                ]
                self.assertEqual(len(creators), 1)
                children = creators[0]["foreground_children"]
                self.assertEqual(len(children), 1)
                child = json.loads(Path(children[0]).read_text())
                self.assertTrue(child["cancelled"])
                self.await_drain(child)
                self.assert_fixture_scope(persistent["record"]["scope"])
                self.assertFalse(
                    operation.drained(
                        json.loads(Path(validation_owner["durable"]).read_text())
                    )
                )
                (base / "stop").touch()
                self.await_drain(persistent["record"])
                self.assert_parent_scope_unchanged()
            finally:
                (base / "stop").touch()
                for name in (
                    "production-runner.json",
                    "production-storage.json",
                    "production-validator.json",
                ):
                    path = base / name
                    if path.exists():
                        owner = json.loads(path.read_text())["record"]["scope"]
                        self.fixture_units.add(owner["unit"])
                        self.kill_scope(owner["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()

    def test_real_cancelled_pending_observer_binds_drain_without_running_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            program = """import os,sys,subprocess,uuid
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h
base=Path(sys.argv[1]); state=base/'state'; state.mkdir()
unit='pse-native-'+uuid.uuid4().hex+'.scope'; owner=h.inherit(os.environ); owner.register(unit)
handoff=n.prepare_handoff(unit,foreground=True); n.cancel_foreground_handoffs()
assert not n.drained(n._record(handoff))
allocation={'execution':{'cpu_threads':1,'observer_memory_bytes':128*1024**2}}
payload="from pathlib import Path; import sys; Path(sys.argv[1]).touch()"
command=s.observer_scope_command(state,allocation,unit,[sys.executable,'-c',payload,str(base/'payload-ran')])
command[command.index('--capabilities')+1]=''
env=s.systemd_environment(); env.pop(n.MARKER,None); env['PSE_NATIVE_HANDOFF']=str(handoff)
status=subprocess.call(command,env=env)
n.write_json(base/'pending-complete.json',{'status':status,'handoff':n._record(handoff),'parent':n._record(n.current())})
"""
            environment = self.fixture_environment(base)
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native",
                    "--",
                    sys.executable,
                    "-c",
                    program,
                    str(base),
                ],
                environment,
            )
            result = subprocess.run(
                command,
                cwd=cache.ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            completed = json.loads((base / "pending-complete.json").read_text())
            child = completed["handoff"]
            self.fixture_units.add(child["scope"]["unit"])
            self.assertNotEqual(completed["status"], 0)
            self.assertFalse((base / "payload-ran").exists())
            self.assertTrue(child["cancelled"])
            self.assertIsInstance(child["scope_inode"], int)
            self.assert_fixture_scope(child["scope"], live=False)
            self.await_drain(child)
            self.await_drain(completed["parent"])


if __name__ == "__main__":
    unittest.main()
