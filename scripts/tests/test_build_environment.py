# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated environment and persistent installation controls."""
# ruff: noqa: PT009, PT027, S108 -- stdlib setup controls

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import build_environment as build
from scripts import native_cache


class BuildEnvironmentTests(unittest.TestCase):
    def test_wrappers_disabled_missing_and_explicit_overrides(self) -> None:
        with patch.object(build.shutil, "which", return_value=None):
            self.assertNotIn("RUSTC_WRAPPER", build.configure(build.ROOT, {}))
            with self.assertRaisesRegex(ValueError, "not installed"):
                build.configure(build.ROOT, {}, cache="on")
        for wrapper in ("", "/custom/wrapper"):
            self.assertEqual(
                build.configure(build.ROOT, {"RUSTC_WRAPPER": wrapper})[
                    "RUSTC_WRAPPER"
                ],
                wrapper,
            )
        env = build.configure(
            build.ROOT,
            {"RUSTC_WRAPPER": "custom", "RUSTC_WORKSPACE_WRAPPER": "custom"},
            cache="off",
        )
        self.assertEqual(env["RUSTC_WRAPPER"], "")
        self.assertEqual(env["RUSTC_WORKSPACE_WRAPPER"], "")

    def test_cache_has_dedicated_owner_and_keeps_incremental(self) -> None:
        with patch.object(build.shutil, "which", return_value="/bin/sccache"):
            env = build.configure(build.ROOT, {"XDG_CACHE_HOME": "/tmp/example"})
        self.assertEqual(env["SCCACHE_DIR"], "/tmp/example/pse-arrow/sccache")
        self.assertEqual(env["SCCACHE_CACHE_SIZE"], "32G")
        self.assertNotIn("CARGO_INCREMENTAL", env)
        self.assertEqual(build.configure(build.ROOT, env), env)
        own = {"RUSTC_WRAPPER": "sccache", "SCCACHE_DIR": "/user/cache"}
        self.assertEqual(
            {key: build.configure(build.ROOT, own)[key] for key in own}, own
        )

    def test_nightly_preserves_flags_and_stable_rejects_unstable(self) -> None:
        env = build.configure(
            build.ROOT,
            {"RUSTFLAGS": "-C link-arg=-fuse-ld=mold"},
            mode="nightly",
            cache="off",
            frontend=2,
            jobs=24,
        )
        self.assertEqual(
            env["CARGO_ENCODED_RUSTFLAGS"].split("\x1f"),
            ["-C", "link-arg=-fuse-ld=mold", "-Zthreads=2"],
        )
        self.assertEqual(env["CARGO_BUILD_JOBS"], "24")
        self.assertIn("nightly-", env["CARGO_TARGET_DIR"])
        with self.assertRaisesRegex(ValueError, "unstable"):
            build.configure(build.ROOT, env, mode="stable")
        flags = build.effective_flags(
            build.ROOT,
            {"CARGO_ENCODED_RUSTFLAGS": "-C\x1fdebuginfo=0", "RUSTFLAGS": "ignored"},
        )
        self.assertEqual(flags, ["-C", "debuginfo=0"])

    def test_nested_subprocess_inherits_disabled_wrapper_and_quoted_arguments(
        self,
    ) -> None:
        code = "import json, os, sys; print(json.dumps([os.environ['RUSTC_WRAPPER'], sys.argv[1]]))"
        argument = "spaces ' quotes $HOME `pwd`"
        result = subprocess.check_output(
            [
                sys.executable,
                "-m",
                "scripts.build_environment",
                "--cache",
                "off",
                "--",
                sys.executable,
                "-c",
                code,
                argument,
            ],
            cwd=build.ROOT,
            text=True,
        )
        self.assertEqual(json.loads(result), ["", argument])


class NativeCacheTests(unittest.TestCase):
    def test_identity_loss_corruption_and_interrupted_install(self) -> None:
        calls = []

        def builder(stage: Path, _work: Path) -> None:
            calls.append(stage)
            (stage / "lib").mkdir()
            (stage / "lib/library.a").write_bytes(b"compiled bytes")

        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            required = ("lib/library.a",)
            identity = {"compiler": "one"}
            prefix = native_cache.prepare(base, "fixture", identity, required, builder)
            self.assertEqual(
                native_cache.prepare(base, "fixture", identity, required, builder),
                prefix,
            )
            self.assertEqual(len(calls), 1)
            (prefix / required[0]).write_bytes(b"corrupt")
            native_cache.prepare(base, "fixture", identity, required, builder)
            (prefix / required[0]).unlink()
            native_cache.prepare(base, "fixture", identity, required, builder)
            self.assertEqual(len(calls), 3)
            self.assertNotEqual(
                native_cache.prepare(
                    base, "fixture", {"compiler": "two"}, required, builder
                ),
                prefix,
            )
            with self.assertRaisesRegex(ValueError, "incomplete"):
                native_cache.prepare(
                    base,
                    "fixture",
                    {"compiler": "interrupted"},
                    required,
                    lambda *_: None,
                )
            self.assertTrue(native_cache.valid(prefix, identity, required))
            self.assertFalse(list((base / "fixture").glob(".*-*")))

    def test_concurrent_preparations_publish_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            code = """
import sys, time
from pathlib import Path
from scripts.native_cache import prepare
base = Path(sys.argv[1])
def builder(stage, work):
    with (base / 'calls').open('a') as log: log.write('build\\n')
    time.sleep(0.1)
    (stage / 'lib').mkdir()
    (stage / 'lib/a').write_text('bytes')
print(prepare(base, 'fixture', {'id': 1}, ('lib/a',), builder))
"""
            processes = [
                subprocess.Popen(
                    [sys.executable, "-c", code, directory],
                    stdout=subprocess.PIPE,
                    text=True,
                    cwd=build.ROOT,
                )
                for _ in range(2)
            ]
            outputs = [p.communicate()[0] for p in processes]
            self.assertEqual([p.returncode for p in processes], [0, 0])
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual((Path(directory) / "calls").read_text(), "build\n")

    def test_explicit_solver_prefix_needs_no_docker(self) -> None:
        result = subprocess.run(
            [
                "bash",
                "-c",
                'source scripts/native-solver-env.sh; test "$IPOPT_DIR" = /explicit',
            ],
            cwd=build.ROOT,
            env={**os.environ, "IPOPT_DIR": "/explicit"},
            check=False,
        )
        self.assertEqual(result.returncode, 0)

    def test_native_runner_mounts_external_target_read_only(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docker = root / "docker"
            docker.write_text(
                f"#!{sys.executable}\nimport json, sys\nprint(json.dumps(sys.argv[1:]))\n"
            )
            docker.chmod(0o755)
            target = root / "target with spaces"
            target.mkdir()
            args = json.loads(
                subprocess.check_output(
                    ["bash", "scripts/native-solver-runner.sh", str(target / "test")],
                    cwd=build.ROOT,
                    env={
                        **os.environ,
                        "CARGO_TARGET_DIR": str(target),
                        "PATH": str(root) + os.pathsep + os.environ["PATH"],
                    },
                    text=True,
                )
            )
            self.assertIn(f"{target}:{target}:ro", args)
            self.assertIn(f"{build.ROOT}:{build.ROOT}:ro", args)
            self.assertEqual(args[-1], str(target / "test"))


if __name__ == "__main__":
    unittest.main()
