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
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from unittest.mock import patch

from scripts import build_environment, producer_deployment, surreal_server
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
            if name not in {operation.MARKER, "PSE_NATIVE_HANDOFF"}
        }
        isolated = patch.dict(os.environ, environment, clear=True)
        isolated.start()
        self.addCleanup(isolated.stop)

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
                    f"native fixture exited before admission: {process.returncode}"
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
                if name not in {operation.MARKER, "PSE_NATIVE_HANDOFF"}
            },
            "PSE_NATIVE_CACHE": str(base),
            "PSE_NATIVE_CAPABILITIES": "",
            "PSE_MEMORY_MAX": "off",
        }

    def test_child_fixture_environment_starts_independent_owner(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {
                    operation.MARKER: "/outer/.operations/owner.json",
                    "PSE_NATIVE_HANDOFF": "/outer/.operations/handoff.json",
                    "PSE_FIXTURE_CALLER": "preserved",
                },
            ),
        ):
            base = Path(directory)
            environment = self.fixture_environment(base)
            self.assertNotIn(operation.MARKER, environment)
            self.assertNotIn("PSE_NATIVE_HANDOFF", environment)
            self.assertEqual(environment["PSE_NATIVE_CACHE"], str(base))
            self.assertEqual(environment["PSE_FIXTURE_CALLER"], "preserved")
            self.assertEqual(
                os.environ[operation.MARKER], "/outer/.operations/owner.json"
            )

    def kill_scope(self, unit: str) -> None:
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

    def test_actual_zero_capability_entry_owns_child_before_command(self) -> None:
        program = "from scripts import native_operation as n; import json; p=n.owner_record(); print(json.dumps({'active':p is not None,'scope':n._record(p)['scope'],'args':__import__('sys').argv[1:]}))"
        result = subprocess.run(
            [
                "bash",
                str(cache.ROOT / "scripts/native_exec.sh"),
                sys.executable,
                "-c",
                program,
                "space ; literal",
                "$literal",
            ],
            cwd=cache.ROOT,
            env={**os.environ, "PSE_NATIVE_CAPABILITIES": "", "PSE_MEMORY_MAX": "off"},
            check=True,
            capture_output=True,
            text=True,
            timeout=30,
        )
        observed = json.loads(result.stdout)
        self.assertTrue(observed["active"])
        self.assertEqual(observed["args"], ["space ; literal", "$literal"])
        if (
            shutil.which("systemd-run")
            and subprocess.run(
                ["systemctl", "--user", "show-environment"],
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=10,
            ).returncode
            == 0
        ):
            self.assertIsNotNone(
                observed["scope"],
                "available user manager must produce an authentic descendant scope",
            )
        if observed["scope"] is not None:
            self.assertTrue(observed["scope"]["unit"].startswith("pse-native-"))
            self.assertEqual(len(observed["scope"]["invocation"]), 32)

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
                self.assertFalse(operation.active(["solver"], dict(os.environ)))
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
                self.assertFalse(operation.active(["solver"], dict(os.environ)))
            with (
                patch.object(operation, "scope_owner", return_value=None),
                operation.Operation(Path(directory)),
            ):
                self.assertIsNotNone(operation.owner_record())
                self.assertIsNone(operation.current())
                operation.admit("observation:forged-fast-hit", {"value": True})
                self.assertIsNone(operation.remembered("observation:forged-fast-hit"))

    def test_empty_capability_source_reentry_rejects_forged_marker_and_preserves_arguments(
        self,
    ) -> None:
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
            script = base / "recipe.sh"
            script.write_text(
                'set -euo pipefail\nsource "$1/scripts/native-recipe-env.sh" "" "$@"\nshift\n"'
                + str(cache.ROOT / ".venv/bin/python")
                + '" -c \'from scripts import native_operation as n; import sys,json; print(json.dumps({"record":str(n.owner_record()),"args":sys.argv[1:]}))\' "$@"\n'
            )
            result = subprocess.run(
                ["bash", str(script), str(cache.ROOT), "space ; literal", "$literal"],
                cwd=cache.ROOT,
                env={**self.fixture_environment(base), operation.MARKER: str(forged)},
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            observed = json.loads(result.stdout)
            self.assertNotEqual(observed["record"], str(forged))
            self.assertEqual(Path(observed["record"]).parent, base / ".operations")
            self.assertEqual(observed["args"], ["space ; literal", "$literal"])

    def test_snapshot_without_venv_uses_selected_operation_interpreter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root = base / "snapshot"
            scripts = root / "scripts"
            scripts.mkdir(parents=True)
            for name in ("native-recipe-env.sh", "native_exec.sh", "memory-cap.sh"):
                shutil.copy2(cache.ROOT / "scripts" / name, scripts / name)
            (scripts / "native_operation.py").write_text(
                f"import sys\nsys.path.insert(0, {str(cache.ROOT)!r})\nfrom scripts.native_operation import main\nraise SystemExit(main())\n"
            )
            recipe = root / "recipe.sh"
            recipe.write_text(
                'set -euo pipefail\nsource "$1/scripts/native-recipe-env.sh" "" "$@"\n"$PSE_NATIVE_SETUP_PYTHON" -c \'from scripts import native_operation as n; import json; print(json.dumps({"active":n.owner_record() is not None}))\'\n'
            )
            result = subprocess.run(
                ["bash", str(recipe), str(root)],
                cwd=cache.ROOT,
                env={
                    **self.fixture_environment(base),
                    "PSE_NATIVE_SETUP_PYTHON": sys.executable,
                },
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            self.assertFalse((root / ".venv").exists())
            self.assertTrue(json.loads(result.stdout)["active"])

    def test_unscoped_fallback_active_boundary_performs_full_checks(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
            operation.Operation(Path(directory)),
            patch.object(operation, "environment", return_value={}) as validate,
        ):
            self.assertTrue(operation.active(["solver"], {"IPOPT_DIR": "/explicit"}))
            validate.assert_called_once_with(["solver"], {"IPOPT_DIR": "/explicit"})

    def test_real_worker_outlives_launcher_then_generation_reclaims_after_drain(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "disposable-state"
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
            launcher = """import json, sys, subprocess, time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping
from scripts import native_cache as c, native_operation as n, surreal_server as s
base=Path(sys.argv[1]); state=base/'disposable-state'
def builder(stage, work):
 (stage/'lib').mkdir(); (stage/'lib/a').write_text('original archive')
generation=c.prepare(base,'fixture',{'id':1},('lib/a',),builder)
unit=s.worker_unit(state,0); handoff=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':256*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[2],str(base),str(generation)],capabilities=())
worker=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,handoff))
n.write_json(base/'launcher-ready.json',{'scope':n.scope_owner(),'generation':str(generation),'handoff':str(handoff)})
while worker.poll() is None: time.sleep(.02)
"""
            log = (base / "fixture.log").open("w")
            process = subprocess.Popen(
                [
                    "bash",
                    str(cache.ROOT / "scripts/native_exec.sh"),
                    sys.executable,
                    "-c",
                    launcher,
                    str(base),
                    child_program,
                ],
                cwd=cache.ROOT,
                env=self.fixture_environment(base),
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
from scripts import native_cache as c
def builder(stage,work):
 (stage/'lib').mkdir(); (stage/'lib/a').write_text('original archive')
print(json.dumps(str(c.prepare(Path(sys.argv[1]),'fixture',{'id':1},('lib/a',),builder))))
"""
                admitted = subprocess.run(
                    [
                        "bash",
                        str(cache.ROOT / "scripts/native_exec.sh"),
                        sys.executable,
                        "-c",
                        repair,
                        str(base),
                    ],
                    cwd=cache.ROOT,
                    env=self.fixture_environment(base),
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=20,
                )
                replacement = Path(json.loads(admitted.stdout))
                self.assertNotEqual(generation, replacement)
                self.assertTrue(cache.valid(replacement, {"id": 1}, ("lib/a",)))
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
                (base / "stop").touch()
                self.await_drain(handoff)
                self.assertEqual(
                    cache.collect(base, "fixture", {"id": 1}), [generation]
                )
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
            process = subprocess.Popen(
                [
                    "bash",
                    str(cache.ROOT / "scripts/native_exec.sh"),
                    sys.executable,
                    "-c",
                    program,
                    str(base),
                    daemon,
                ],
                cwd=cache.ROOT,
                env=self.fixture_environment(base),
                stdout=log,
                stderr=log,
            )
            observed = None
            try:
                observed = self.await_file(base / "cancel-ready.json", process)
                record = observed["record"]
                self.assertIsNotNone(record["scope"])
                self.assertEqual(observed["group"], record["scope"]["group"])
                os.kill(record["pid"], signal.SIGTERM)
                process.wait(timeout=15)
                self.await_drain(record)
                self.assertFalse(operation.populated(record["scope"]["group"]))
            finally:
                if observed and observed["record"].get("scope"):
                    self.kill_scope(observed["record"]["scope"]["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()


if __name__ == "__main__":
    unittest.main()
