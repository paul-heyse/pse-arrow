# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite parser routing, byte capture, namespace refusal and owned drain."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import json
import os
import signal
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import resource_store
from scripts import tooling_experiments as tooling


class ToolingExperimentTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / "selected.surql"
        self.source.write_text("RETURN 1;\n")
        self.binary = self.root / "surreal"
        self.binary.write_bytes(b"exact-admitted-parser")
        self.receipt = self.root / "release.json"
        self.release = {
            "binary": str(self.binary), "binary_sha256": tooling.surreal_source.digest(self.binary),
            "effective_key": "exact-context", "version": "3.3.4",
        }
        self.receipt.write_text(json.dumps(self.release))
        self.bwrap = self.root / "bwrap"
        self.bwrap.write_bytes(b"exact-bubblewrap")
        self.output = self.root / "output"
        self.calls = []

    def invoke(self, *, library_parser: bool = False) -> tuple[int, dict]:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            code = tooling.main([
                "surrealql-validate", "--input", str(self.source), "--output", str(self.output),
                *( ["--parser-binary", str(self.binary)] if library_parser else ["--server-receipt", str(self.receipt)] ),
            ])
        return code, json.loads(stdout.getvalue())

    def staged(self, command: list[str], output: Path, label: str, work: Path, deadline: float) -> dict:
        self.calls.append((command, label))
        stdout = tooling.BUBBLEWRAP_VERSION if label == "bubblewrap-version" else "statically linked"
        (output / f"{label}.stdout.txt").write_text(stdout)
        (output / f"{label}.stderr.txt").write_text("")
        if label == "parser":
            # The real builder points only at frozen inputs even after caller edits.
            snapshot = work / "input-000.surql"
            self.source.write_text("RETURN changed-after-snapshot;\n")
            self.assertEqual(snapshot.read_text(), "RETURN 1;\n")
            (output / "payload-output/namespace-ready").write_bytes(b"namespace-established\n")
            (output / "payload-output/write-witness").write_bytes(b"owned-output\n")
        return {"status": "returned", "exit_code": 0, "process": {"owned_drain": "complete"}, "errors": [], "truncated": False}

    def configured(self, callback=None):
        stack = contextlib.ExitStack()
        stack.enter_context(patch.object(tooling, "admitted_server", return_value=(self.receipt, self.release)))
        stack.enter_context(patch.object(tooling.shutil, "which", return_value=str(self.bwrap)))
        stack.enter_context(patch.object(tooling, "stage", side_effect=callback or self.staged))
        return stack

    def test_finite_api_freezes_inputs_and_preserves_result(self) -> None:
        with self.configured(), patch.dict(os.environ, {"PSE_TOOL_PROBE_SECRET": "do-not-record"}):
            code, result = self.invoke()
        self.assertEqual(code, 0)
        self.assertEqual(result["outcome"], "valid")
        self.assertEqual(result["drain"], "complete")
        self.assertEqual(result["cleanup"], "complete")
        self.assertFalse(Path(result["private_workspace"]).exists())
        self.assertEqual(resource_store.read(self.output / "result.json"), result)
        self.assertNotIn("do-not-record", json.dumps(result))
        command = self.calls[-1][0]
        self.assertIn("--unshare-net", command)
        self.assertIn("--unshare-user", command)
        self.assertIn("--clearenv", command)
        self.assertNotIn("--ro-bind-try", command)
        self.assertNotIn("--share-net", command)
        self.assertIn('/tool/surreal validate "$@"', command[command.index("-c") + 1])
        self.assertFalse(any(path.name.startswith(".probe-") for path in self.output.iterdir()))
        self.assertTrue((self.output / "server-receipt.json").is_file())
        self.assertEqual(result["parser"]["selection"], "server-cli")
        self.assertEqual(result["parser"]["syntax_profile"], "server-cli-experimental-grammar-disabled")

    def parser_report(self, *, valid: bool = True) -> dict:
        return {
            "schema": "surrealql-validation-v1", "valid": valid,
            "parser": {"library": "surrealdb-syn", "version": "compiled-version", "syntax_profile": "all-features", "scope": "parse-only"},
            "files": [{"input": "/inputs/input-000.surql", "valid": valid,
                       "error": None if valid else {"kind": "syntax", "message": "Parse error: exact library diagnostic"}}],
        }

    def test_explicit_library_parser_preserves_metadata_errors_and_exact_binary(self) -> None:
        self.binary.chmod(0o500)
        expected = self.parser_report(valid=False)

        def library(command, output, label, work, deadline):
            observed = self.staged(command, output, label, work, deadline)
            if label == "parser":
                (output / "parser.stdout.txt").write_text(json.dumps(expected))
                observed["exit_code"] = 1
            return observed

        with self.configured(library), patch.object(tooling, "admitted_server", side_effect=AssertionError("server route selected")):
            code, result = self.invoke(library_parser=True)
        self.assertEqual(code, 1)
        self.assertEqual(result["outcome"], "invalid")
        self.assertEqual(result["validation"], expected)
        self.assertEqual(result["parser"]["selection"], "library-parser")
        self.assertEqual(result["parser"]["reported"]["version"], "compiled-version")
        self.assertEqual(result["parser"]["binary_sha256"], self.release["binary_sha256"])
        self.assertNotIn("server", result)
        self.assertFalse((self.output / "server-receipt.json").exists())
        command = self.calls[-1][0]
        self.assertIn("/tool/parser", command)
        self.assertNotIn("/tool/surreal", command)
        self.assertIn('exec /tool/parser surrealql-validate "$@"', command[command.index("-c") + 1])

    def test_library_report_refuses_missing_or_contradictory_population(self) -> None:
        valid = self.parser_report()
        self.assertEqual(tooling.library_report(json.dumps(valid), 1, 0), valid)
        for change in ("count", "profile", "status", "error", "correspondence"):
            report = self.parser_report()
            count, code = 1, 0
            if change == "count":
                count = 2
            elif change == "profile":
                report["parser"]["syntax_profile"] = "unknown"
            elif change == "status":
                code = 1
            elif change == "error":
                report["files"][0]["valid"] = False
            else:
                report["files"][0]["input"] = "/unselected.surql"
            with self.subTest(change=change), self.assertRaises(ValueError):
                tooling.library_report(json.dumps(report), count, code)

    def test_parser_selection_is_explicit_and_rejects_mixed_sources(self) -> None:
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            tooling.main(["surrealql-validate", "--input", str(self.source), "--output", str(self.output), "--server-receipt", str(self.receipt), "--parser-binary", str(self.binary)])

    def test_namespace_refusal_does_not_run_unconfined_parser(self) -> None:
        def refused(command, output, label, work, deadline):
            if label == "parser":
                self.calls.append((command, label))
                return {"status": "returned", "exit_code": 1, "process": {"owned_drain": "complete"}, "errors": []}
            return self.staged(command, output, label, work, deadline)
        with self.configured(refused):
            code, result = self.invoke()
        self.assertEqual(code, 125)
        self.assertEqual(result["outcome"], "unavailable")
        self.assertEqual(result["cleanup"], "complete")
        self.assertEqual([label for _, label in self.calls].count("parser"), 1)
        self.assertEqual(self.calls[-1][0][0], str(self.bwrap))

    def test_parser_error_is_independent_of_successful_cleanup(self) -> None:
        def invalid(command, output, label, work, deadline):
            result = self.staged(command, output, label, work, deadline)
            if label == "parser":
                result["exit_code"] = 1
            return result
        with self.configured(invalid):
            code, result = self.invoke()
        self.assertEqual((code, result["outcome"], result["cleanup"]), (1, "invalid", "complete"))

    def test_unknown_drain_retains_only_owned_workspace(self) -> None:
        def unresolved(command, output, label, work, deadline):
            result = self.staged(command, output, label, work, deadline)
            if label == "parser":
                result["process"]["owned_drain"] = "unresolved"
            return result
        unrelated = self.root / "unrelated"
        unrelated.write_text("preserve")
        with self.configured(unresolved):
            code, result = self.invoke()
        self.assertEqual(code, 125)
        self.assertEqual(result["outcome"], "valid")
        self.assertEqual(result["cleanup"], "retained-pending-owned-drain")
        self.assertTrue(Path(result["private_workspace"]).is_dir())
        self.assertEqual(unrelated.read_text(), "preserve")

    def test_cancellation_keeps_signal_and_drain(self) -> None:
        def cancelled(command, output, label, work, deadline):
            result = self.staged(command, output, label, work, deadline)
            if label == "parser":
                result["exit_code"] = 128 + signal.SIGTERM
                result["process"]["forwarded_signal"] = signal.SIGTERM
            return result
        with self.configured(cancelled):
            code, result = self.invoke()
        self.assertEqual((code, result["outcome"], result["drain"]), (143, "cancelled", "complete"))
        self.assertEqual(result["cleanup"], "complete")

    def test_preparation_cancellation_launches_nothing(self) -> None:
        def admit(_path):
            os.kill(os.getpid(), signal.SIGTERM)
            return self.receipt, self.release
        with self.configured(), patch.object(tooling, "admitted_server", side_effect=admit):
            code, result = self.invoke()
        self.assertEqual((code, result["outcome"], result["drain"]), (143, "cancelled", "not-started"))
        self.assertEqual(self.calls, [])
        self.assertEqual(result["cleanup"], "complete")

    def test_changed_receipt_or_binary_refuses_before_launch(self) -> None:
        with patch.object(tooling.surreal_source, "admitted_receipt", return_value=False), patch.object(tooling, "stage", side_effect=AssertionError("launched")):
            code, result = self.invoke()
        self.assertEqual(code, 125)
        self.assertEqual(result["drain"], "not-started")
        with patch.object(tooling.surreal_source, "admitted_receipt", return_value=True):
            self.binary.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "binary differs"):
                tooling.admitted_server(self.receipt)

    def test_existing_output_and_symlink_parent_preserve_content(self) -> None:
        self.output.mkdir()
        marker = self.output / "keep"
        marker.write_text("keep")
        code, _ = self.invoke()
        self.assertEqual(code, 125)
        self.assertEqual(marker.read_text(), "keep")
        link = self.root / "symlink"
        link.symlink_to(self.output, target_is_directory=True)
        self.output = link / "uncreated"
        code, _ = self.invoke()
        self.assertEqual(code, 125)
        self.assertFalse(self.output.exists())

    def test_unresolved_libraries_and_unknown_cases_refuse(self) -> None:
        with self.assertRaisesRegex(ValueError, "unresolved"):
            tooling.dependencies("libexample.so => not found", 0)
        with self.assertRaisesRegex(ValueError, "unrecognized"):
            tooling.dependencies("", 0)
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            tooling.main(["arbitrary-command", "--input", str(self.source), "--output", str(self.output)])


class NativeNamespaceTests(unittest.TestCase):
    """Revealing checks use actual Bubblewrap and the existing child-scope owner."""

    def test_actual_readonly_fixture_private_output_environment_and_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "logs"
            output.mkdir(mode=0o700)
            payload = root / "payload"
            payload.mkdir(mode=0o700)
            witness = root / "readonly-witness"
            witness.write_bytes(tooling.WITNESS)
            shell = Path("/bin/sh").resolve()
            deadline = time.monotonic() + 30
            observed = tooling.stage(["/usr/bin/ldd", str(shell)], output, "shell-runtime", root, deadline)
            self.assertEqual(observed["process"]["owned_drain"], "complete")
            dependencies = tooling.dependencies((output / "shell-runtime.stdout.txt").read_text(), observed["exit_code"])
            command = tooling.namespace_command(
                Path("/usr/local/bin/bwrap"), {"sh": shell, "surreal": shell},
                {path: path.resolve() for path in dependencies}, [], witness, payload,
            )
            split = command.index("--")
            script = "[ -z \"${PSE_E04_SECRET+x}\" ] || exit 81; [ ! -e /etc/passwd ] || exit 82; " + tooling.PARSER_LAUNCH.replace('exec /tool/surreal validate "$@"', "exit 0")
            with patch.dict(os.environ, {"PSE_E04_SECRET": "private"}):
                result = tooling.stage([*command[:split + 1], "/tool/sh", "-c", script], output, "witness", root, deadline)
            self.assertTrue(tooling.successful(result), result)
            self.assertEqual(witness.read_bytes(), tooling.WITNESS)
            self.assertEqual((payload / "write-witness").read_bytes(), b"owned-output\n")
            timed = tooling.stage([*command[:split + 1], "/tool/sh", "-c", "while :; do :; done"], output, "cancel", root, time.monotonic() + 0.4)
            self.assertEqual(timed["status"], "timed_out")
            self.assertEqual(timed["process"]["owned_drain"], "complete")
            self.assertEqual(timed["process"]["forwarded_signal"], signal.SIGTERM)


if __name__ == "__main__":
    unittest.main()
