# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual installer selection, single-use capture and pre-return sealing controls."""
# ruff: noqa: PT009, PT027 -- stdlib process controls

from __future__ import annotations

import contextlib
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import producer_cargo as observer


class ProducerCargoTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / "capture"
        self.output.mkdir(mode=0o700)
        self.options = [
            "--package", "fixture", "--profile", "producer", "--features", "force-validate,native-solvers",
            "--cdylib-manifest", "Cargo.toml", "--output", str(self.output / "python.json"),
            "--build-evidence", str(self.output / "python-build-evidence.json"),
        ]
        self.environment = observer.configure(self.root, self.output, self.options, os.environ)
        self.configuration = Path(self.environment[observer.CAPTURE])
        self.arguments = [
            "rustc", "--manifest-path", str(self.root / "Cargo.toml"), "--profile", "producer",
            "--features", "force-validate,native-solvers", "--lib", "--crate-type", "cdylib",
            "--locked", "--message-format", "json-render-diagnostics",
        ]

    def seal(self, *, eligible: bool = True) -> None:
        (self.output / "python.json").write_text(json.dumps({
            "receipt_version": 2, "persistent_reuse_eligible": eligible,
            "reasons": [] if eligible else ["unknown macro input"],
        }))
        closure = self.output / "python-build-evidence.json.closure"
        closure.mkdir()
        raw = b'{"actual":"fixture protected index"}'
        (closure / "index.json").write_bytes(raw)
        (closure / "COMPLETE").write_text(hashlib.sha256(raw).hexdigest())

    def test_actual_child_context_is_captured_without_synthetic_sdk_or_secret_dump(self) -> None:
        child = {
            **self.environment, "PYO3_PYTHON": "/actual/python", "PYO3_ENVIRONMENT_SIGNATURE": "cpython-3.14-64bit",
            "PYTHON_SYS_EXECUTABLE": "/actual/python", "PYO3_BUILD_EXTENSION_MODULE": "1",
            "RUSTC_WRAPPER": "", "IPOPT_DIR": "/actual/solver", "SYMBOLICA_LICENSE": "never-observed",
        }

        def capture(command: list[str], *, cwd: Path, env: dict[str, str]) -> int:
            self.assertEqual(cwd, self.root)
            self.assertEqual(command[:2], [str(self.root / "target/debug/xtask"), "producer-identity"])
            self.assertNotIn("CARGO", env)
            self.assertNotIn(observer.CAPTURE, env)
            self.assertNotIn(observer.PYTHON, env)
            self.assertIn("PYO3_PYTHON=/actual/python", command)
            self.assertIn("RUSTC_WRAPPER=", command)
            self.assertIn("IPOPT_DIR=/actual/solver", command)
            self.assertNotIn("never-observed", " ".join(command))
            self.seal()
            return 0

        with contextlib.chdir(self.root), patch.object(observer.subprocess, "call", side_effect=capture):
            self.assertEqual(observer.observe(self.arguments, child), 0)
        actual = json.loads(self.configuration.with_suffix(".observation.json").read_text())
        self.assertEqual(actual["arguments"], self.arguments)
        self.assertEqual(actual["effective_environment"]["PYTHON_SYS_EXECUTABLE"], "/actual/python")
        self.assertNotIn("never-observed", json.dumps(actual))
        self.assertEqual(json.loads(self.configuration.with_suffix(".status.json").read_text()), {"returncode": 0, "sealed": True})
        with (
            contextlib.chdir(self.root), patch.object(observer.subprocess, "call") as run,
            self.assertRaises(FileExistsError),
        ):
            observer.observe(self.arguments, child)
        run.assert_not_called()

    def test_metadata_and_version_passthrough_do_not_consume_the_build_claim(self) -> None:
        for arguments in (["metadata", "--format-version", "1"], ["--version"]):
            with contextlib.chdir(self.root), patch.object(observer.subprocess, "call", return_value=0) as run:
                self.assertEqual(observer.observe(arguments, self.environment), 0)
            self.assertEqual(run.call_args.args[0][1:], arguments)
            self.assertNotIn("CARGO", run.call_args.kwargs["env"])
            self.assertFalse(self.configuration.with_suffix(".claim.json").exists())

    def test_unsupported_command_and_wrong_checkout_refuse_before_capture(self) -> None:
        with contextlib.chdir(self.root), patch.object(observer.subprocess, "call") as run:
            with self.assertRaisesRegex(ValueError, "unsupported actual installer"):
                observer.observe(["build"], self.environment)
            self.assertFalse(self.configuration.with_suffix(".claim.json").exists())
        with patch.object(observer.subprocess, "call") as run:
            with self.assertRaisesRegex(ValueError, "different working directory"):
                observer.observe(self.arguments, self.environment)
        run.assert_not_called()

    def test_configuration_and_cargo_selection_refuse_unsupported_changes(self) -> None:
        with self.assertRaisesRegex(ValueError, "empty explicit CARGO"):
            observer.cargo_executable({**os.environ, "CARGO": ""})
        with self.assertRaisesRegex(ValueError, "differing"):
            observer.cargo_executable({**os.environ, "CARGO": "/bin/false"})
        self.configuration.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "owned private"):
            observer.read_configuration(self.configuration)
        self.configuration.chmod(0o600)
        configuration = json.loads(self.configuration.read_text())
        configuration["version"] = True
        self.configuration.write_text(json.dumps(configuration))
        with self.assertRaisesRegex(ValueError, "unsupported"):
            observer.read_configuration(self.configuration)

    def test_changed_actual_cargo_path_refuses_before_any_build_claim(self) -> None:
        binary = self.root / "other-bin/cargo"
        binary.parent.mkdir()
        binary.write_text("#!/bin/sh\nexit 0\n")
        binary.chmod(0o700)
        with (
            contextlib.chdir(self.root), patch.object(observer.subprocess, "call") as run,
            self.assertRaisesRegex(ValueError, "different Cargo executable"),
        ):
            observer.observe(self.arguments, {**self.environment, "PATH": str(binary.parent)})
        run.assert_not_called()
        self.assertFalse(self.configuration.with_suffix(".claim.json").exists())

    def test_nonzero_capture_retains_failure_and_cannot_claim_sealing(self) -> None:
        with contextlib.chdir(self.root), patch.object(observer.subprocess, "call", return_value=17):
            self.assertEqual(observer.observe(self.arguments, self.environment), 17)
        self.assertEqual(json.loads(self.configuration.with_suffix(".status.json").read_text()), {"returncode": 17, "sealed": False})
        self.assertEqual(json.loads(self.configuration.with_suffix(".arguments.json").read_text()), self.arguments)

    def test_success_without_eligible_sealed_realization_refuses_installation(self) -> None:
        for failure in ("ineligible", "missing-seal", "tampered-seal"):
            with self.subTest(failure=failure):
                directory = self.root / failure
                directory.mkdir(mode=0o700)
                self.output = directory
                self.options[self.options.index("--output") + 1] = str(directory / "python.json")
                self.options[self.options.index("--build-evidence") + 1] = str(directory / "python-build-evidence.json")
                environment = observer.configure(self.root, directory, self.options, os.environ)

                def capture(_command: list[str], **_kwargs: object) -> int:
                    self.seal(eligible=failure != "ineligible")
                    complete = directory / "python-build-evidence.json.closure/COMPLETE"
                    if failure == "missing-seal":
                        complete.unlink()
                    elif failure == "tampered-seal":
                        complete.write_text("changed")
                    return 0

                with contextlib.chdir(self.root), patch.object(observer.subprocess, "call", side_effect=capture), self.assertRaises((OSError, ValueError)):
                    observer.observe(self.arguments, environment)
                status = json.loads((directory / "python-installer-cargo.status.json").read_text())
                self.assertEqual(status["returncode"], 1)
                self.assertFalse(status["sealed"])

    def test_pyo3_configuration_bytes_are_protected_and_change_refuses_return(self) -> None:
        source = self.root / "pyo3.cfg"
        source.write_bytes(b"actual-sdk-config")
        environment = {**self.environment, "PYO3_CONFIG_FILE": str(source)}

        def capture(_command: list[str], **_kwargs: object) -> int:
            self.seal()
            source.write_bytes(b"changed-sdk-config")
            return 0

        with contextlib.chdir(self.root), patch.object(observer.subprocess, "call", side_effect=capture), self.assertRaisesRegex(ValueError, "PyO3 configuration changed"):
            observer.observe(self.arguments, environment)
        self.assertEqual(self.configuration.with_suffix(".pyo3-config").read_bytes(), b"actual-sdk-config")

    def test_real_adapter_streams_cargo_stdout_before_selected_child_exits(self) -> None:
        # An actual process handshake proves the adapter does not buffer the
        # machine stream until capture completion. Its fake core owns this fixture.
        tool = self.root / "target/debug/xtask"
        tool.parent.mkdir(parents=True)
        release = self.root / "release"
        tool.write_text(f"#!{sys.executable}\n" + """import json,sys,time
from pathlib import Path
args=sys.argv
print('{"reason":"fixture-before-exit"}',flush=True)
release=Path(""" + repr(str(release)) + """)
until=time.monotonic()+5
while not release.exists():
 if time.monotonic()>=until:sys.exit(17)
 time.sleep(.01)
output=Path(args[args.index('--output')+1])
output.write_text(json.dumps({'receipt_version':2,'persistent_reuse_eligible':True,'reasons':[]}))
closure=Path(args[args.index('--build-evidence')+1]+'.closure');closure.mkdir()
raw=b'fixture sealed index';(closure/'index.json').write_bytes(raw)
import hashlib
(closure/'COMPLETE').write_text(hashlib.sha256(raw).hexdigest())
""")
        tool.chmod(0o700)
        process = subprocess.Popen(
            [sys.executable, str(Path(observer.__file__)), *self.arguments],
            cwd=self.root, env=self.environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        try:
            self.assertIsNotNone(process.stdout)
            line = process.stdout.readline()
            self.assertEqual(line, b'{"reason":"fixture-before-exit"}\n')
            self.assertIsNone(process.poll())
            release.touch()
            remaining, errors = process.communicate(timeout=10)
            self.assertEqual(process.returncode, 0, errors.decode())
            self.assertEqual(remaining, b"")
        finally:
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=10)


if __name__ == "__main__":
    unittest.main()
