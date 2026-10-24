# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable cache probes keep compilation, statistics and cleanup together."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import io
import subprocess
import unittest
from unittest.mock import patch, sentinel

from scripts import build_cache_probe as probe


class BuildCacheProbeTests(unittest.TestCase):
    def test_disposable_server_owns_compile_statistics_and_cleanup(self) -> None:
        selected = {
            "RUSTC_WRAPPER": "/checkout/scripts/sccache",
            "PSE_SCCACHE_BINARY": "/installed/sccache",
            "SCCACHE_DIR": "/shared/cache",
            "SCCACHE_SERVER_UDS": "/shared/cache.sock",
        }
        stats = {"stats": {"cache_hits": {"counts": {"Rust": 1}}}}
        with (
            patch.object(probe.build_environment, "configure", return_value=selected),
            patch.object(probe.subprocess, "check_output", return_value="/rustc\n"),
            patch.object(probe.subprocess, "run") as compile_process,
            patch.object(
                probe.build_measurements,
                "start_cache_server",
                return_value=sentinel.server,
            ) as start,
            patch.object(
                probe.build_measurements, "cache_stats", side_effect=[{}, stats]
            ) as observe,
            patch.object(probe.build_measurements, "stop_cache_server") as stop,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            probe.main()

        environment = start.call_args.args[0]
        self.assertEqual(environment["RUSTC_WRAPPER"], "/installed/sccache")
        self.assertNotEqual(environment["SCCACHE_DIR"], "/shared/cache")
        self.assertNotEqual(environment["SCCACHE_SERVER_UDS"], "/shared/cache.sock")
        self.assertEqual(compile_process.call_count, 3)
        for call in compile_process.call_args_list:
            self.assertEqual(call.args[0][0], "/installed/sccache")
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
