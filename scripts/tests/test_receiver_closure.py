# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Frozen receiver imports include real dependencies and reject uncertain closure."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import receiver_closure as closure


class ReceiverClosureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(self.directory) / "source"
        self.root.mkdir()
        self.write("scripts/__init__.py", "")

    def write(self, name: str, source: str) -> None:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source)

    def test_regular_parents_generated_and_relative_imports_freeze_together(
        self,
    ) -> None:
        self.write("scripts/generated/__init__.py", "")
        self.write("scripts/generated/deployment.py", "class Allocation: pass\n")
        self.write("scripts/owner/__init__.py", "from . import child\n")
        self.write(
            "scripts/owner/child.py", "from scripts.generated import deployment\n"
        )
        self.write("scripts/entry.py", "from scripts.owner import child\n")
        files = closure.source_files(self.root, ["scripts/entry.py"])
        self.assertEqual(
            set(files),
            {
                "scripts/__init__.py",
                "scripts/entry.py",
                "scripts/owner/__init__.py",
                "scripts/owner/child.py",
                "scripts/generated/__init__.py",
                "scripts/generated/deployment.py",
            },
        )
        frozen = Path(self.directory) / "frozen"
        for name in files:
            target = frozen / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(self.root / name, target)
        closure.probe(frozen, files, Path(sys.executable))
        (frozen / "scripts/generated/deployment.py").unlink()
        with self.assertRaises(closure.ClosureError):
            closure.probe(frozen, files, Path(sys.executable))

    def test_computed_import_requires_its_exact_owner_declaration(self) -> None:
        self.write(
            "scripts/entry.py",
            "import importlib\ndef load(name): return importlib.import_module(name)\n",
        )
        self.write("scripts/provider.py", "value = 1\n")
        with self.assertRaisesRegex(closure.ClosureError, "explicit owner declaration"):
            closure.source_files(self.root, ["scripts/entry.py"])
        files = closure.source_files(
            self.root,
            ["scripts/entry.py"],
            dynamic={"scripts/entry.py": ("scripts.provider",)},
        )
        self.assertIn("scripts/provider.py", files)

    def test_unrelated_script_change_does_not_change_receiver_inputs(self) -> None:
        self.write("scripts/entry.py", "from scripts import provider\n")
        self.write("scripts/provider.py", "value = 1\n")
        self.write("scripts/unrelated.py", "value = 2\n")

        def identity() -> str:
            files = closure.source_files(self.root, ["scripts/entry.py"])
            return hashlib.sha256(
                b"".join(
                    name.encode() + (self.root / name).read_bytes() for name in files
                )
            ).hexdigest()

        original = identity()
        self.write("scripts/unrelated.py", "value = 3\n")
        self.assertEqual(identity(), original)
        self.write("scripts/provider.py", "value = 4\n")
        self.assertNotEqual(identity(), original)

    def test_missing_owned_module_refuses_before_publication(self) -> None:
        self.write("scripts/entry.py", "from scripts import missing\n")
        with self.assertRaisesRegex(closure.ClosureError, "missing"):
            closure.source_files(self.root, ["scripts/entry.py"])

    def test_actual_receiver_imports_only_its_frozen_sources_and_declared_configuration(
        self,
    ) -> None:
        from scripts import surreal_server as server

        root = Path(__file__).resolve().parents[2]
        files = closure.source_files(root, ["scripts/surreal_server.py"])
        self.assertIn("scripts/generated/deployment.py", files)
        self.assertIn("scripts/generated/telemetry_settings.py", files)
        self.assertFalse(any(name.startswith("scripts/test_") for name in files))
        for replaced in (
            "native_tests",
            "producer_deployment",
            "python_extension",
            "validation",
        ):
            self.assertNotIn(f"scripts/{replaced}.py", files)
        frozen = Path(self.directory) / "actual-frozen"
        for name in (*files, *server.GENERATION_CONFIGURATION):
            target = frozen / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(root / name, target)
        runtime = closure.probe(frozen, files, Path(sys.executable))
        self.assertIn(str(Path(sys.executable).absolute()), runtime)
        self.assertTrue(any("msgspec" in name for name in runtime))
        injected = Path(self.directory) / "unadmitted-python"
        (injected / "scripts").mkdir(parents=True)
        for name in ("scripts/__init__.py", "msgspec.py", "sitecustomize.py"):
            (injected / name).write_text(
                "raise RuntimeError('unadmitted Python provider selected')\n"
            )
        environment = {
            **os.environ,
            "PYTHONPATH": str(injected),
            "PYTHONHOME": str(injected),
            "PYTHONUSERBASE": str(injected),
            "PYTHONDONTWRITEBYTECODE": "0",
        }
        for entry in ("surreal_server.py", "native_operation.py"):
            with self.subTest(entry=entry):
                child = subprocess.run(
                    [
                        sys.executable,
                        "-I",
                        "-B",
                        str(frozen / "scripts" / entry),
                        "--help",
                    ],
                    cwd=injected,
                    env=environment,
                    capture_output=True,
                    timeout=20,
                    check=False,
                )
                self.assertEqual(
                    child.returncode, 0, child.stderr.decode(errors="replace")
                )
                self.assertIn(b"usage:", child.stdout)
        self.assertFalse(list(frozen.rglob("__pycache__")))

    def test_runtime_inputs_invalidate_generation_and_v1_remains_observation_only(
        self,
    ) -> None:
        from scripts import surreal_server as server

        state = Path(self.directory) / "state"
        state.mkdir()
        provider = state / "provider.py"
        provider.write_text("value = 1\n")
        provider.chmod(0o600)
        content = b"retained supervisor source\n"
        files = {"scripts/surreal_server.py": hashlib.sha256(content).hexdigest()}
        for version in (1, 2):
            declaration = (
                files
                if version == 1
                else {
                    "files": files,
                    "runtime": {str(provider): closure.digest(provider)},
                    "interpreter": str(Path(sys.executable).absolute()),
                }
            )
            identity = hashlib.sha256(
                json.dumps(declaration, sort_keys=True, separators=(",", ":")).encode()
            ).hexdigest()
            directory = state / identity
            (directory / "scripts").mkdir(parents=True)
            script = directory / "scripts/surreal_server.py"
            script.write_bytes(content)
            script.chmod(0o600)
            server.write_json(
                directory / "generation.json",
                {
                    "version": version,
                    "identity": identity,
                    **({"files": files} if version == 1 else declaration),
                },
            )
            server.verify_generation(directory)
            if version == 1:
                with self.assertRaisesRegex(server.SupervisorError, "readmission"):
                    server.activatable_generation(directory)
            else:
                server.activatable_generation(directory)
                provider.write_text("value = 2\n")
                with self.assertRaisesRegex(
                    server.SupervisorError, "provider inputs changed"
                ):
                    server.verify_generation(directory)

    def test_actual_publication_binds_frozen_operator_and_runtime_inputs(self) -> None:
        from scripts import surreal_server as server

        state = Path(self.directory) / "publication"
        state.mkdir(mode=0o700)
        helper = Path(self.directory) / "selected-operator"
        helper.write_text("#!/bin/sh\nexit 0\n")
        helper.chmod(0o700)
        with patch.dict(os.environ, {"PSE_DEPLOYMENT_OPERATOR": str(helper)}):
            selected = server.publish_generation(state)
        directory = Path(selected["supervisor_script"]).parents[1]
        manifest = server.read_json(directory / "generation.json")
        self.assertEqual(manifest["version"], 2)
        self.assertEqual(
            manifest["files"]["bin/pse-deployment"], closure.digest(helper)
        )
        self.assertIn(str(Path(sys.executable).absolute()), manifest["runtime"])
        self.assertEqual(
            selected["supervisor_executable"], str(Path(sys.executable).absolute())
        )
        server.activatable_generation(directory)
        (directory / "bin/pse-deployment").write_text("changed operator")
        with self.assertRaisesRegex(server.SupervisorError, "closure changed"):
            server.verify_generation(directory)


if __name__ == "__main__":
    unittest.main()
