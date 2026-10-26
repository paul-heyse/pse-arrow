# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact subjects, independently stopped helpers and genuine small tool controls."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import json
import os
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import operation_lifetime
from scripts import active_diagnostics as active


class ActiveDiagnosticsTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def invoke(
        self, action: str, *options: str, command: list[str] | None = None
    ) -> tuple[int, dict]:
        output = self.root / f"action-{len(list(self.root.glob('action-*')))}"
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = active.main(
                [
                    action,
                    "--output",
                    str(output),
                    *options,
                    *(["--", *command] if command else []),
                ]
            )
        return code, json.loads(captured.getvalue())

    def test_changed_identity_and_unsafe_modes_refuse_before_tool(self) -> None:
        with patch.object(
            active, "collect", side_effect=AssertionError("tool launched")
        ):
            code, result = self.invoke(
                "python-snapshot", "--pid", str(os.getpid()), "--start", "0"
            )
            self.assertEqual(code, 1)
            self.assertIn("identity changed", result["error"])
            code, result = self.invoke(
                "python-sample",
                "--pid",
                str(os.getpid()),
                "--start",
                active.operation_lifetime.start_identity(os.getpid()),
                "--native",
                "--nonblocking",
            )
            self.assertEqual(code, 1)
            self.assertIn("cannot be nonblocking", result["error"])

    def test_fractional_python_sampling_refuses_before_tool(self) -> None:
        with patch.object(
            active, "collect", side_effect=AssertionError("tool launched")
        ):
            code, result = self.invoke(
                "python-sample",
                "--seconds",
                "0.5",
                "--pid",
                str(os.getpid()),
                "--start",
                active.operation_lifetime.start_identity(os.getpid()),
            )
        self.assertEqual(code, 1)
        self.assertIn("whole number of seconds", result["error"])

    def test_genuine_invocation_is_optional_and_read_never_launches(self) -> None:
        code, result = self.invoke("feasibility")
        self.assertEqual(code, 0)
        self.assertIsNone(result["invocation"])
        self.assertEqual(result["schema"], active.SCHEMA)
        self.assertEqual(
            result["memray"]["decision"],
            "optional isolated pinned Python allocation route",
        )
        with patch.object(
            active, "collect", side_effect=AssertionError("read executed")
        ):
            read = active.read_action(self.root / "action-0")
        self.assertEqual(read["action"], "feasibility")

    def test_read_replaced_text_never_emits_unverified_tail(self) -> None:
        directory = self.root / "readable"
        directory.mkdir(mode=0o700)
        text = directory / "capture.stdout.txt"
        text.write_text("verified original\n")
        active.publish(directory, {"schema": active.SCHEMA, "action": "control"})
        text.write_text("replacement must not escape\n")
        observed = active.read_action(directory, tail=20)["artifact_observations"][0]
        self.assertEqual(observed["status"], "changed")
        self.assertEqual(observed["tail"], [])

    def test_required_decoder_failure_is_partial(self) -> None:
        def capture(
            _command: list[str], output: Path, label: str, **_options: object
        ) -> dict:
            if label == "perf-record":
                (output / "perf.data").write_bytes(b"raw data")
            return {
                "status": "returned",
                "exit_code": 1 if label == "perf-stacks" else 0,
            }

        tool = {"status": "available", "path": "/selected/perf"}
        with (
            patch.object(active, "collect", side_effect=capture),
            patch.object(active, "selected_tool", return_value=tool),
        ):
            code, result = self.invoke(
                "python-perf", command=[sys.executable, "-c", "pass"]
            )
        self.assertEqual(code, 1)
        self.assertEqual(result["capture_status"], "collected")
        self.assertEqual(result["status"], "partial")
        self.assertEqual(result["decoders"][0]["exit_code"], 1)

    def test_rr_trace_fingerprint_refuses_changed_runtime_and_symlinks(self) -> None:
        trace = self.root / "trace"
        trace.mkdir()
        (trace / "data").write_bytes(b"recorded data")
        first = active.trace_reference(trace)
        active.trace_reference(trace, first["sha256"])
        (trace / "data").write_bytes(b"replacement")
        with self.assertRaisesRegex(ValueError, "differs"):
            active.trace_reference(trace, first["sha256"])
        (trace / "alias").symlink_to(trace / "data")
        with self.assertRaisesRegex(ValueError, "symlinks"):
            active.trace_reference(trace)

    def test_memray_requires_selected_pinned_runtime_before_workload(self) -> None:
        with patch.object(
            active, "collect", side_effect=AssertionError("workload launched")
        ):
            code, result = self.invoke("memray", command=[sys.executable, "-c", "pass"])
        self.assertEqual(code, 1)
        self.assertIn("isolated pinned tool environment", result["error"])

    def test_memray_version_follows_project_declaration(self) -> None:
        version = {
            "version": "9.8.7",
            "prefix": "/selected/env",
            "executable": sys.executable,
            "python": sys.version,
        }
        completed = subprocess.CompletedProcess([], 0, json.dumps(version), "")
        with (
            patch.object(
                active.doctor,
                "pyproject",
                return_value={
                    "dependency-groups": {
                        "diagnostics": ["memray==9.8.7; sys_platform == 'linux'"]
                    }
                },
            ),
            patch.object(active.subprocess, "run", return_value=completed),
        ):
            selected = active.memray_runtime(Path(sys.executable))
        self.assertEqual(selected["runtime"]["version"], "9.8.7")

    def test_unavailable_core_and_wrong_executable_are_not_attachment(self) -> None:
        binary = Path(sys.executable).resolve()
        with patch.object(
            active, "collect", side_effect=AssertionError("debugger launched")
        ):
            code, result = self.invoke(
                "gdb-core",
                "--core",
                str(self.root / "missing.core"),
                "--executable",
                str(binary),
                "--executable-sha256",
                "0" * 64,
            )
            self.assertEqual(code, 1)
            self.assertIn("executable identity differs", result["error"])
            code, result = self.invoke(
                "gdb-core",
                "--core",
                str(self.root / "missing.core"),
                "--executable",
                str(binary),
                "--executable-sha256",
                active.regular(binary)["sha256"],
            )
            self.assertEqual(code, 1)
            self.assertIn("No such file", result["error"])

    def test_identified_owned_launch_preserves_literal_argv(self) -> None:
        output = self.root / "identified"
        output.mkdir(mode=0o700)
        marker = output / "subject.json"
        payload = "literal space ; $(touch forbidden)"
        command = [
            sys.executable,
            "-c",
            "import json,sys; print(json.dumps(sys.argv[1:]))",
            payload,
        ]
        captured = active.collect(
            active.identified_launch(command, marker),
            output,
            "target",
            seconds=5,
            output_bytes=65536,
        )
        self.assertEqual(captured["exit_code"], 0, captured)
        self.assertEqual(
            json.loads((output / "target.stdout.txt").read_text()), [payload]
        )
        subject = active.resource_store.read(marker)
        self.assertGreater(subject["pid"], 0)
        self.assertTrue(subject["start"].isdigit())
        self.assertEqual(marker.stat().st_mode & 0o777, 0o600)

    def test_identified_launch_restores_only_selected_loader_path_before_reexec(
        self,
    ) -> None:
        output = self.root / "selected-loader"
        output.mkdir(mode=0o700)
        marker = output / "subject.json"
        selected = "/selected/lib path;literal"
        command = [
            sys.executable,
            "-c",
            "import json,os;print(json.dumps({'path':os.environ.get('LD_LIBRARY_PATH')}))",
        ]
        with patch.dict(os.environ, {"LD_LIBRARY_PATH": selected}):
            launch = active.identified_launch(command, marker)
            self.assertEqual(json.loads(launch[4]), {"LD_LIBRARY_PATH": selected})
            captured = active.collect(
                launch,
                output,
                "target",
                seconds=5,
                output_bytes=65536,
                environment_remove=("LD_LIBRARY_PATH",),
            )
        self.assertEqual(captured["exit_code"], 0, captured)
        self.assertEqual(
            json.loads((output / "target.stdout.txt").read_text()), {"path": selected}
        )
        self.assertEqual(
            active.resource_store.read(marker)["restored_environment"],
            ["LD_LIBRARY_PATH"],
        )

    def test_decoder_environment_choice_does_not_change_parent(self) -> None:
        output = self.root / "local-symbols"
        output.mkdir(mode=0o700)
        with patch.dict(os.environ, {"DEBUGINFOD_URLS": "https://example.invalid"}):
            value = active.collect(
                [
                    sys.executable,
                    "-c",
                    "import os; print('DEBUGINFOD_URLS' in os.environ)",
                ],
                output,
                "decoder",
                seconds=5,
                output_bytes=4096,
                environment_remove=("DEBUGINFOD_URLS",),
            )
            self.assertEqual(os.environ["DEBUGINFOD_URLS"], "https://example.invalid")
        self.assertEqual(value["exit_code"], 0, value)
        self.assertEqual((output / "decoder.stdout.txt").read_text().strip(), "False")
        self.assertEqual(value["environment_removed"], ["DEBUGINFOD_URLS"])

    def test_real_bounded_helper_timeout_does_not_stop_unrelated_target(self) -> None:
        target = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"])

        def cleanup() -> None:
            if target.poll() is None:
                target.kill()
            target.wait(timeout=5)

        self.addCleanup(cleanup)
        output = self.root / "collector"
        output.mkdir()
        captured = active.collect(
            [
                sys.executable,
                "-c",
                "import os,time; os.write(1,b'x'*200000); time.sleep(30)",
            ],
            output,
            "helper",
            seconds=2,
            output_bytes=1024,
        )
        self.assertEqual(captured["status"], "timed_out")
        self.assertTrue(captured["truncated"])
        self.assertEqual((output / "helper.stdout.txt").stat().st_size, 1024)
        self.assertIsNone(target.poll())
        target.terminate()
        target.wait(timeout=5)

    def test_actual_python_native_snapshot_and_previous_stop_preservation(self) -> None:
        target = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"])
        try:
            start = active.operation_lifetime.start_identity(target.pid)
            code, result = self.invoke(
                "python-snapshot",
                "--pid",
                str(target.pid),
                "--start",
                start,
                "--native",
            )
            self.assertEqual(code, 0, result)
            self.assertIsNone(target.poll())
            text = (self.root / "action-0/py-spy.stdout.txt").read_text()
            self.assertIn("sleep", text)
            os.kill(target.pid, signal.SIGSTOP)
            os.waitpid(target.pid, os.WUNTRACED)
            code, result = self.invoke(
                "python-snapshot", "--pid", str(target.pid), "--start", start
            )
            self.assertEqual(code, 1)
            self.assertIn("already stopped", result["error"])
            self.assertTrue(
                active.process_subject(target.pid, start)["state"].startswith("T")
            )
        finally:
            os.kill(target.pid, signal.SIGCONT)
            target.terminate()
            target.wait(timeout=5)

    def test_actual_native_crash_allocation_and_permission_controls(self) -> None:
        source = self.root / "native-control.c"
        binary = self.root / "native-control"
        source.write_text("""#include <stdlib.h>
#include <string.h>
#include <unistd.h>
volatile void *retained;
__attribute__((noinline)) void allocate_temporary(void) {
 for (int i=0;i<500;i++) { char *p=malloc(4096); memset(p,i,4096); asm volatile("" : : "r"(p) : "memory"); free(p); }
}
__attribute__((noinline)) void allocate_retained(void) { retained=malloc(4*1024*1024); }
__attribute__((noinline)) void crash_native(void) { *(volatile int *)0=1; }
int main(int argc,char **argv) { if(argc>1 && !strcmp(argv[1],"crash")) crash_native(); allocate_retained(); allocate_temporary(); usleep(10000); return 0; }
""")
        subprocess.run(
            [
                "cc",
                "-O0",
                "-g",
                "-fno-omit-frame-pointer",
                str(source),
                "-o",
                str(binary),
            ],
            check=True,
            capture_output=True,
        )
        code, result = self.invoke("gdb-launch", command=[str(binary), "crash"])
        self.assertEqual(code, 0, result)
        self.assertIn({"signal": "SIGSEGV"}, result["target_outcome"])
        self.assertIn(
            "crash_native", (self.root / "action-0/gdb.stdout.txt").read_text()
        )
        code, result = self.invoke("heaptrack", command=[str(binary)])
        self.assertEqual(code, 0, result)
        summary = (self.root / "action-1/heaptrack-summary.stdout.txt").read_text()
        self.assertIn("allocate_temporary", summary)
        self.assertIn("allocate_retained", summary)
        self.assertIn("PEAK MEMORY", summary)
        code, result = self.invoke("sched", command=[str(binary)])
        if code:
            self.assertEqual(result["status"], "unavailable")
            refusal = (self.root / "action-2/sched-record.stderr.txt").read_text()
            self.assertRegex(
                refusal, "No permissions|Permission|trace|Operation not permitted"
            )
        else:
            self.assertTrue((self.root / "action-2/sched-timehist.stdout.txt").exists())
        code, result = self.invoke(
            "stat", "--events", "cycles:u,instructions:u", command=[str(binary)]
        )
        if code:
            self.assertIn(
                "not", (self.root / "action-3/perf-stat.stderr.txt").read_text().lower()
            )
        else:
            self.assertTrue((self.root / "action-3/stat.json").stat().st_size)

    def test_gdb_launch_preserves_literal_spaces_and_shell_metacharacters(self) -> None:
        payload = f"$(touch {self.root / 'must-not-exist'})"
        code, result = self.invoke(
            "gdb-launch",
            command=[
                sys.executable,
                "-c",
                "import json,sys; print('LITERAL_ARGV '+json.dumps(sys.argv[1:]))",
                "space ; literal",
                payload,
            ],
        )
        self.assertEqual(code, 0, result)
        output = (self.root / "action-0/gdb.stdout.txt").read_text()
        row = next(
            line for line in output.splitlines() if line.startswith("LITERAL_ARGV ")
        )
        self.assertEqual(
            json.loads(row.removeprefix("LITERAL_ARGV ")), ["space ; literal", payload]
        )
        self.assertFalse((self.root / "must-not-exist").exists())


if __name__ == "__main__":
    unittest.main()
