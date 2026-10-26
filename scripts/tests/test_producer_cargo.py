# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual installer selection, single-use capture and pre-return sealing controls."""
# ruff: noqa: PT009, PT027 -- stdlib process controls

from __future__ import annotations

import contextlib
import fcntl
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import build_observation, producing_currency
from scripts import producer_cargo as observer

if TYPE_CHECKING:
    from collections.abc import Callable


class ProducerCargoTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / "capture"
        self.output.mkdir(mode=0o700)
        self.options = [
            "--package",
            "fixture",
            "--profile",
            "producer",
            "--features",
            "force-validate,native-solvers",
            "--cdylib-manifest",
            "Cargo.toml",
            "--output",
            str(self.output / "python.json"),
            "--build-evidence",
            str(self.output / "python-build-evidence.json"),
        ]
        self.environment = observer.configure(
            self.root, self.output, self.options, os.environ
        )
        self.configuration = Path(self.environment[observer.CAPTURE])
        self.arguments = [
            "rustc",
            "--manifest-path",
            str(self.root / "Cargo.toml"),
            "--profile",
            "producer",
            "--features",
            "force-validate,native-solvers",
            "--lib",
            "--crate-type",
            "cdylib",
            "--locked",
            "--message-format",
            "json-render-diagnostics",
        ]

    def seal(self, *, eligible: bool = True) -> None:
        (self.output / "python.json").write_text(
            json.dumps(
                {
                    "receipt_version": 2,
                    "persistent_reuse_eligible": eligible,
                    "reasons": [] if eligible else ["unknown macro input"],
                }
            )
        )
        closure = self.output / "python-build-evidence.json.closure"
        closure.mkdir()
        raw = b'{"actual":"fixture protected index"}'
        (closure / "index.json").write_bytes(raw)
        (closure / "COMPLETE").write_text(hashlib.sha256(raw).hexdigest())

    def ordinary_environment(self, output: Path | None = None) -> dict[str, str]:
        for name in (*observer.OBSERVER_SOURCE_INPUTS, "target/debug/xtask"):
            source = self.root / name
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text("selected observer fixture")
        witness = producing_currency.Witness()
        sources = {
            str(self.root / name): witness.file(self.root / name)
            for name in observer.OBSERVER_SOURCE_INPUTS
        }
        prepared = {
            "source_basis": producing_currency.basis(sources),
            "artifact_stamp": list(
                producing_currency.stamp(self.root / "target/debug/xtask")
            ),
            "command": ["fixture: explicitly prepared observer"],
        }
        return observer.configure_currency(
            self.root,
            self.output if output is None else output,
            os.environ,
            observer=prepared,
        )

    def test_ordinary_observer_forwards_actual_installer_command_and_compiler_selection(
        self,
    ) -> None:
        environment = self.ordinary_environment()
        environment["RUSTC_WRAPPER"] = "selected-sccache-wrapper"
        environment["RUSTC_WORKSPACE_WRAPPER"] = "selected-workspace-wrapper"
        selected = Path(environment[observer.CURRENCY_CAPTURE])
        calls = []

        def run(
            command: list[str],
            *,
            env: dict[str, str],
            record: Callable[[bytes, str], None],
        ) -> int:
            calls.append((command, env))
            record(
                b'{"reason":"compiler-artifact","filenames":["selected.so"]}', "stdout"
            )
            record(b'{"reason":"build-finished","success":true}', "stdout")
            record(b"installer text is not a Cargo record", "stderr")
            return 0

        def extract(command: list[str], **_kwargs: object) -> int:
            if command[1] == "producing-graph":
                Path(command[-1]).write_text(
                    '{"roots":[0],"units":[{"pkg_id":"fixture","dependencies":[]}]}'
                )
            elif command[1] == "producing-inputs":
                Path(command[-1]).write_text('{"root_artifacts":[]}')
            return 0

        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call", side_effect=extract) as extractor,
            patch.object(
                observer.subprocess, "check_output", return_value=b'{"packages":[]}'
            ),
            patch.object(build_observation, "run_command", side_effect=run),
            patch.object(hashlib, "file_digest", wraps=hashlib.file_digest) as digest,
        ):
            self.assertEqual(observer.observe(self.arguments, environment), 0)
        self.assertEqual(calls[0][0][1:], self.arguments)
        child = calls[0][1]
        self.assertEqual(child["RUSTC_WRAPPER"], "selected-sccache-wrapper")
        self.assertEqual(child["RUSTC_WORKSPACE_WRAPPER"], "selected-workspace-wrapper")
        self.assertNotIn("CARGO", child)
        self.assertEqual(extractor.call_args_list[0].args[0][1], "producing-graph")
        self.assertEqual(extractor.call_args_list[1].args[0][1], "producing-inputs")
        records = selected.with_suffix(".messages.jsonl").read_text()
        self.assertIn('"reason":"compiler-artifact"', records)
        self.assertNotIn("installer text", records)
        self.assertTrue(
            json.loads(selected.with_suffix(".status.json").read_text())["supported"]
        )
        invocation = json.loads(selected.with_suffix(".selection.json").read_text())
        actual_observer = invocation["observer"]["artifact"]
        self.assertEqual(
            actual_observer["sha256"],
            hashlib.sha256(b"selected observer fixture").hexdigest(),
        )
        inputs = json.loads(selected.with_suffix(".inputs.json").read_text())
        self.assertNotIn(str(self.root / "target/debug/xtask"), inputs["files"])
        self.assertTrue(
            all(
                str(self.root / name) in inputs["files"]
                for name in observer.OBSERVER_SOURCE_INPUTS
            )
        )
        self.assertEqual(
            sum(
                call.args[0].name == str(self.root / "target/debug/xtask")
                for call in digest.call_args_list
            ),
            1,
        )

    def test_ordinary_observer_keeps_conservative_path_when_extraction_is_unavailable(
        self,
    ) -> None:
        environment = self.ordinary_environment()

        def extract(command: list[str], **_kwargs: object) -> int:
            if command[1] == "producing-graph":
                Path(command[-1]).write_text(
                    '{"roots":[0],"units":[{"pkg_id":"fixture","dependencies":[]}]}'
                )
                return 0
            return 7

        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call", side_effect=extract),
            patch.object(
                observer.subprocess, "check_output", return_value=b'{"packages":[]}'
            ),
            patch.object(build_observation, "run_command", return_value=0),
        ):
            self.assertEqual(observer.observe(self.arguments, environment), 0)
        selected = Path(environment[observer.CURRENCY_CAPTURE])
        self.assertFalse(
            json.loads(selected.with_suffix(".status.json").read_text())["supported"]
        )
        self.assertFalse(selected.with_suffix(".selection.json").exists())

    def test_ordinary_observer_refuses_binary_or_source_replacement_during_extraction(
        self,
    ) -> None:
        for index, (changed, identical) in enumerate(
            (
                ("target/debug/xtask", False),
                ("xtask/src/producing_inputs.rs", False),
                ("target/debug/xtask", True),
            )
        ):
            with self.subTest(changed=changed, identical=identical):
                output = self.output / str(index)
                output.mkdir(mode=0o700)
                environment = self.ordinary_environment(output)

                def extract(
                    command: list[str],
                    *,
                    changed: str = changed,
                    identical: bool = identical,
                    **_kwargs: object,
                ) -> int:
                    if command[1] == "producing-graph":
                        Path(command[-1]).write_text(
                            '{"roots":[0],"units":[{"pkg_id":"fixture","dependencies":[]}]}'
                        )
                    else:
                        Path(command[-1]).write_text('{"root_artifacts":[]}')
                        changed_path = self.root / changed
                        if identical:
                            replacement = self.root / "observer-replacement"
                            replacement.write_bytes(changed_path.read_bytes())
                            replacement.chmod(changed_path.stat().st_mode & 0o777)
                            replacement.replace(changed_path)
                        else:
                            changed_path.write_text("raced observer fixture")
                    return 0

                with (
                    contextlib.chdir(self.root),
                    patch.object(observer.subprocess, "call", side_effect=extract),
                    patch.object(
                        observer.subprocess,
                        "check_output",
                        return_value=b'{"packages":[]}',
                    ),
                    patch.object(build_observation, "run_command", return_value=0),
                ):
                    self.assertEqual(observer.observe(self.arguments, environment), 0)
                selected = Path(environment[observer.CURRENCY_CAPTURE])
                status = json.loads(selected.with_suffix(".status.json").read_text())
                self.assertEqual(
                    status, {"supported": False, "cause": "observer-changed"}
                )
                self.assertFalse(selected.with_suffix(".selection.json").exists())

    def test_ordinary_observer_refuses_source_change_after_helper_preparation(
        self,
    ) -> None:
        environment = self.ordinary_environment()
        (self.root / "xtask/src/producing_inputs.rs").write_text(
            "new source after explicit helper build"
        )
        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call", return_value=0),
            patch.object(build_observation, "run_command", return_value=0),
        ):
            self.assertEqual(observer.observe(self.arguments, environment), 0)
        selected = Path(environment[observer.CURRENCY_CAPTURE])
        self.assertEqual(
            json.loads(selected.with_suffix(".status.json").read_text()),
            {"supported": False, "cause": "observer-preparation-changed"},
        )
        self.assertFalse(selected.with_suffix(".selection.json").exists())

    def test_target_guard_remains_charged_until_actual_writer_descendant_exits(
        self,
    ) -> None:
        path = self.root / "target-guard"
        with path.open("w+b") as owner:
            fcntl.flock(owner, fcntl.LOCK_EX | fcntl.LOCK_NB)
            environment = {
                **os.environ,
                observer.PREPARATION_GUARD_FD: str(owner.fileno()),
            }
            child = subprocess.Popen(
                [
                    sys.executable,
                    "-c",
                    "import os,sys;os.fstat(int(os.environ['PSE_PRODUCER_PREPARATION_GUARD_FD']));print('writer-ready',flush=True);sys.stdin.buffer.read(1)",
                ],
                env=environment,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                pass_fds=observer.preparation_child_fds(environment),
            )
            try:
                self.assertIsNotNone(child.stdout)
                self.assertEqual(child.stdout.readline(), b"writer-ready\n")
                owner.close()
                with path.open("r+b") as competing:
                    with self.assertRaises(BlockingIOError):
                        fcntl.flock(competing, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    _, errors = child.communicate(b"x", timeout=5)
                    self.assertEqual(child.returncode, 0, errors)
                    fcntl.flock(competing, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                if child.poll() is None:
                    child.kill()
                    child.communicate(timeout=5)

    def test_actual_child_context_is_captured_without_synthetic_sdk_or_secret_dump(
        self,
    ) -> None:
        child = {
            **self.environment,
            "PYO3_PYTHON": "/actual/python",
            "PYO3_ENVIRONMENT_SIGNATURE": "cpython-3.14-64bit",
            "PYTHON_SYS_EXECUTABLE": "/actual/python",
            "PYO3_BUILD_EXTENSION_MODULE": "1",
            "RUSTC_WRAPPER": "",
            "IPOPT_DIR": "/actual/solver",
            "SYMBOLICA_LICENSE": "never-observed",
            "CARGO_BUILD_JOBS": "2",
            "CARGO_TERM_COLOR": "always",
            "CARGO_REGISTRIES_PRIVATE_TOKEN": "never-observed-registry-token",
        }

        def capture(
            command: list[str],
            *,
            cwd: Path,
            env: dict[str, str],
            pass_fds: tuple[int, ...],
        ) -> int:
            self.assertEqual(cwd, self.root)
            self.assertEqual(pass_fds, ())
            self.assertEqual(
                command[:2],
                [str(self.root / "target/debug/xtask"), "producer-identity"],
            )
            self.assertNotIn("CARGO", env)
            self.assertNotIn(observer.CAPTURE, env)
            self.assertNotIn(observer.PYTHON, env)
            self.assertIn("PYO3_PYTHON=/actual/python", command)
            self.assertIn("RUSTC_WRAPPER=", command)
            self.assertIn("IPOPT_DIR=/actual/solver", command)
            self.assertIn("CARGO_BUILD_JOBS=2", command)
            self.assertIn("CARGO_TERM_COLOR=always", command)
            self.assertNotIn("never-observed", " ".join(command))
            self.seal()
            return 0

        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call", side_effect=capture),
        ):
            self.assertEqual(observer.observe(self.arguments, child), 0)
        actual = json.loads(
            self.configuration.with_suffix(".observation.json").read_text()
        )
        self.assertEqual(actual["arguments"], self.arguments)
        self.assertEqual(
            actual["effective_environment"]["PYTHON_SYS_EXECUTABLE"], "/actual/python"
        )
        self.assertEqual(actual["effective_environment"]["CARGO_BUILD_JOBS"], "2")
        self.assertEqual(actual["effective_environment"]["CARGO_TERM_COLOR"], "always")
        self.assertNotIn("never-observed", json.dumps(actual))
        self.assertEqual(
            json.loads(self.configuration.with_suffix(".status.json").read_text()),
            {"returncode": 0, "sealed": True},
        )
        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call") as run,
            self.assertRaises(FileExistsError),
        ):
            observer.observe(self.arguments, child)
        run.assert_not_called()

    def test_metadata_and_version_passthrough_do_not_consume_the_build_claim(
        self,
    ) -> None:
        for arguments in (["metadata", "--format-version", "1"], ["--version"]):
            with (
                contextlib.chdir(self.root),
                patch.object(observer.subprocess, "call", return_value=0) as run,
            ):
                self.assertEqual(observer.observe(arguments, self.environment), 0)
            self.assertEqual(run.call_args.args[0][1:], arguments)
            self.assertNotIn("CARGO", run.call_args.kwargs["env"])
            self.assertFalse(self.configuration.with_suffix(".claim.json").exists())

    def test_unsupported_command_and_wrong_checkout_refuse_before_capture(self) -> None:
        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call") as run,
        ):
            with self.assertRaisesRegex(ValueError, "unsupported actual installer"):
                observer.observe(["build"], self.environment)
            self.assertFalse(self.configuration.with_suffix(".claim.json").exists())
        with (
            patch.object(observer.subprocess, "call") as run,
            self.assertRaisesRegex(ValueError, "different working directory"),
        ):
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
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call") as run,
            self.assertRaisesRegex(ValueError, "different Cargo executable"),
        ):
            observer.observe(
                self.arguments, {**self.environment, "PATH": str(binary.parent)}
            )
        run.assert_not_called()
        self.assertFalse(self.configuration.with_suffix(".claim.json").exists())

    def test_hardlinked_rustup_style_proxy_names_are_distinct_invocations(self) -> None:
        directory = self.root / "proxies"
        directory.mkdir()
        cargo = directory / "cargo"
        rustc = directory / "rustc"
        cargo.write_text(
            '#!/bin/sh\ncase "$0" in */cargo) echo cargo ;; *) echo rustc ;; esac\n'
        )
        cargo.chmod(0o700)
        os.link(cargo, rustc)
        self.assertTrue(cargo.samefile(rustc))
        self.assertEqual(
            subprocess.check_output([str(cargo)], text=True).strip(), "cargo"
        )
        self.assertEqual(
            subprocess.check_output([str(rustc)], text=True).strip(), "rustc"
        )
        environment = {"PATH": str(directory), "CARGO": str(rustc)}
        with self.assertRaisesRegex(ValueError, "differing"):
            observer.cargo_executable(environment)
        self.assertEqual(
            observer.cargo_executable({"PATH": str(directory), "CARGO": str(cargo)}),
            str(cargo),
        )

    def test_symlink_proxy_preserves_selected_invocation_and_normalizes_parents(
        self,
    ) -> None:
        directory = self.root / "proxy-links"
        directory.mkdir()
        proxy = directory / "rustup"
        proxy.write_text("#!/bin/sh\nexit 0\n")
        proxy.chmod(0o700)
        cargo = directory / "cargo"
        cargo.symlink_to(proxy)
        self.assertEqual(
            observer.cargo_executable(
                {
                    "PATH": str(directory),
                    "CARGO": str(directory / "../proxy-links/cargo"),
                }
            ),
            str(cargo),
        )
        self.assertNotEqual(str(cargo), str(cargo.resolve()))
        with self.assertRaisesRegex(ValueError, "differing"):
            observer.cargo_executable({"PATH": str(directory), "CARGO": str(proxy)})

    def test_nonzero_capture_retains_failure_and_cannot_claim_sealing(self) -> None:
        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call", return_value=17),
        ):
            self.assertEqual(observer.observe(self.arguments, self.environment), 17)
        self.assertEqual(
            json.loads(self.configuration.with_suffix(".status.json").read_text()),
            {"returncode": 17, "sealed": False},
        )
        self.assertEqual(
            json.loads(self.configuration.with_suffix(".arguments.json").read_text()),
            self.arguments,
        )

    def test_success_without_eligible_sealed_realization_refuses_installation(
        self,
    ) -> None:
        for failure in ("ineligible", "missing-seal", "tampered-seal"):
            with self.subTest(failure=failure):
                directory = self.root / failure
                directory.mkdir(mode=0o700)
                self.output = directory
                self.options[self.options.index("--output") + 1] = str(
                    directory / "python.json"
                )
                self.options[self.options.index("--build-evidence") + 1] = str(
                    directory / "python-build-evidence.json"
                )
                environment = observer.configure(
                    self.root, directory, self.options, os.environ
                )

                def capture(
                    _command: list[str],
                    *,
                    failure: str = failure,
                    directory: Path = directory,
                    **_kwargs: object,
                ) -> int:
                    self.seal(eligible=failure != "ineligible")
                    complete = directory / "python-build-evidence.json.closure/COMPLETE"
                    if failure == "missing-seal":
                        complete.unlink()
                    elif failure == "tampered-seal":
                        complete.write_text("changed")
                    return 0

                with (
                    contextlib.chdir(self.root),
                    patch.object(observer.subprocess, "call", side_effect=capture),
                    self.assertRaises((OSError, ValueError)),
                ):
                    observer.observe(self.arguments, environment)
                status = json.loads(
                    (directory / "python-installer-cargo.status.json").read_text()
                )
                self.assertEqual(status["returncode"], 1)
                self.assertFalse(status["sealed"])

    def test_pyo3_configuration_bytes_are_protected_and_change_refuses_return(
        self,
    ) -> None:
        source = self.root / "pyo3.cfg"
        source.write_bytes(b"actual-sdk-config")
        environment = {**self.environment, "PYO3_CONFIG_FILE": str(source)}

        def capture(_command: list[str], **_kwargs: object) -> int:
            self.seal()
            source.write_bytes(b"changed-sdk-config")
            return 0

        with (
            contextlib.chdir(self.root),
            patch.object(observer.subprocess, "call", side_effect=capture),
            self.assertRaisesRegex(ValueError, "PyO3 configuration changed"),
        ):
            observer.observe(self.arguments, environment)
        self.assertEqual(
            self.configuration.with_suffix(".pyo3-config").read_bytes(),
            b"actual-sdk-config",
        )

    def test_real_adapter_streams_cargo_stdout_before_selected_child_exits(
        self,
    ) -> None:
        # An actual process handshake proves the adapter does not buffer the
        # machine stream until capture completion. Its fake core owns this fixture.
        tool = self.root / "target/debug/xtask"
        tool.parent.mkdir(parents=True)
        release = self.root / "release"
        tool.write_text(
            f"#!{sys.executable}\n"
            """import json,sys,time
from pathlib import Path
from typing import TYPE_CHECKING
args=sys.argv
print('{"reason":"fixture-before-exit"}',flush=True)
release=Path("""
            + repr(str(release))
            + """)
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
"""
        )
        tool.chmod(0o700)
        process = subprocess.Popen(
            [sys.executable, str(Path(observer.__file__)), *self.arguments],
            cwd=self.root,
            env=self.environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
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
