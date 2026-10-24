# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable cache probes keep compilation, statistics and cleanup together."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import io
import json
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch, sentinel

from scripts import build_cache_probe as probe


class BuildCacheProbeTests(unittest.TestCase):
    def test_disposable_server_owns_compile_statistics_and_cleanup(self) -> None:
        selected = {
            "RUSTC_WRAPPER": "/checkout/scripts/sccache",
            "PSE_SCCACHE_BINARY": "/installed/sccache",
            "SCCACHE_DIR": "/shared/cache",
            "SCCACHE_SERVER_UDS": "/shared/cache.sock",
            "SCCACHE_MAX_FRAME_LENGTH": "134217728",
        }
        stats = {"stats": {"cache_hits": {"counts": {"Rust": 1}}}}
        invocations = 0

        def compiler_output(
            command: list[str], *, env: dict[str, str], check: bool
        ) -> None:
            nonlocal invocations
            self.assertTrue(check)
            if "--out-dir" not in command:
                return
            destination = Path(command[command.index("--out-dir") + 1])
            artifact = destination / "libpse_cache_probe.rlib"
            invocations += 1
            if invocations == 2:
                self.assertFalse(artifact.exists(), "hit must recreate removed outputs")
            artifact.write_bytes(b"owned mock output")
            (destination / "pse_cache_probe.d").write_text("owned dep-info")
            entry = Path(env["SCCACHE_DIR"]) / "entry"
            entry.parent.mkdir(exist_ok=True)
            with entry.open("wb") as stream:
                stream.truncate(8 * 1024**2 + 1)

        with (
            patch.object(probe.build_environment, "configure", return_value=selected),
            patch.object(probe.subprocess, "check_output", return_value="/rustc\n"),
            patch.object(
                probe.subprocess, "run", side_effect=compiler_output
            ) as compile_process,
            patch.object(
                probe.build_measurements,
                "start_cache_server",
                return_value=sentinel.server,
            ) as start,
            patch.object(
                probe.build_measurements, "cache_stats", side_effect=[{}, stats]
            ) as observe,
            patch.object(probe.build_measurements, "stop_cache_server") as stop,
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            probe.main()

        environment = start.call_args.args[0]
        self.assertEqual(
            json.loads(output.getvalue())["cache_max_frame_length"], "134217728"
        )
        self.assertEqual(environment["RUSTC_WRAPPER"], "/installed/sccache")
        self.assertNotEqual(environment["SCCACHE_DIR"], "/shared/cache")
        self.assertNotEqual(environment["SCCACHE_SERVER_UDS"], "/shared/cache.sock")
        self.assertEqual(compile_process.call_count, 5)
        for index in (0, 1, 4):
            self.assertEqual(
                compile_process.call_args_list[index].args[0][0], "/installed/sccache"
            )
        self.assertIn("--extern", compile_process.call_args_list[2].args[0])
        self.assertTrue(
            compile_process.call_args_list[3].args[0][0].endswith("/consumer")
        )
        for call in compile_process.call_args_list:
            self.assertEqual(call.kwargs["env"], environment)
        for call in observe.call_args_list:
            self.assertEqual(call.args[0], environment)
        stop.assert_called_once_with(sentinel.server, environment)

    def test_compile_failure_still_reaps_only_disposable_server(self) -> None:
        selected = {
            "RUSTC_WRAPPER": "/checkout/scripts/sccache",
            "PSE_SCCACHE_BINARY": "/installed/sccache",
        }
        with (
            patch.object(probe.build_environment, "configure", return_value=selected),
            patch.object(probe.subprocess, "check_output", return_value="/rustc\n"),
            patch.object(
                probe.subprocess,
                "run",
                side_effect=subprocess.CalledProcessError(1, ["/installed/sccache"]),
            ),
            patch.object(
                probe.build_measurements,
                "start_cache_server",
                return_value=sentinel.server,
            ),
            patch.object(probe.build_measurements, "cache_stats", return_value={}),
            patch.object(probe.build_measurements, "stop_cache_server") as stop,
            self.assertRaises(subprocess.CalledProcessError),
        ):
            probe.main()
        stop.assert_called_once_with(sentinel.server, selected)


if __name__ == "__main__":
    unittest.main()
