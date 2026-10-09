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
    def test_explicit_libclang_file_survives_nested_setup(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            prefix = Path(directory)
            (prefix / "bin").mkdir()
            (prefix / "lib").mkdir()
            (prefix / "bin/clang").touch()
            library = prefix / "lib/libclang.so"
            library.touch()
            selected = {
                "PSE_LLVM_PREFIX": str(prefix),
                "LIBCLANG_PATH": str(library),
            }
            env = build.configure(build.ROOT, selected, cache="off")
            self.assertEqual(env["LIBCLANG_PATH"], str(library))
            self.assertEqual(build.configure(build.ROOT, env), env)
            default = build.configure(
                build.ROOT, {"PSE_LLVM_PREFIX": str(prefix)}, cache="off"
            )
            self.assertEqual(default["LIBCLANG_PATH"], str(prefix / "lib"))
            child = subprocess.check_output(
                [
                    sys.executable,
                    "-m",
                    "scripts.build_environment",
                    "--",
                    sys.executable,
                    "-c",
                    "import os; print(os.environ['LIBCLANG_PATH'])",
                ],
                cwd=build.ROOT,
                env={**os.environ, **env},
                text=True,
            )
            self.assertEqual(child.strip(), str(library))

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
        self.assertEqual(env["SCCACHE_CACHE_SIZE"], "100G")
        self.assertNotIn("CARGO_INCREMENTAL", env)
        self.assertEqual(env["SCCACHE_CLIENT_SIDE"], "1")
        self.assertEqual(env["SCCACHE_DIRECT"], "false")
        self.assertEqual(env["SCCACHE_CONF"], str(build.ROOT / ".config/sccache.toml"))
        self.assertEqual(env["PSE_SCCACHE_BINARY"], "/bin/sccache")
        self.assertEqual(env["RUSTC_WRAPPER"], str(build.ROOT / "scripts/sccache"))
        self.assertEqual(build.configure(build.ROOT, env), env)
        own = {"RUSTC_WRAPPER": "sccache", "SCCACHE_DIR": "/user/cache"}
        configured = build.configure(build.ROOT, own)
        self.assertEqual(configured["SCCACHE_DIR"], own["SCCACHE_DIR"])
        self.assertEqual(configured["PSE_SCCACHE_BINARY"], "sccache")
        self.assertEqual(
            configured["RUSTC_WRAPPER"], str(build.ROOT / "scripts/sccache")
        )

    def test_cache_modes_that_escape_supervision_fall_back_before_setup(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory) / "sccache.toml"
            config.write_text('[dist]\nscheduler_url="http://localhost:12345"\n')
            json_config = Path(directory) / "sccache.json"
            json_config.write_text(
                '{"dist":{"scheduler_url":"http://localhost:12345"}}'
            )
            for extra in (
                {"SCCACHE_ERROR_LOG": ""},
                {"SCCACHE_CONF": str(config)},
                {"SCCACHE_CONF": str(json_config)},
            ):
                original = {
                    "RUSTC_WRAPPER": "/bin/sccache",
                    "PSE_NATIVE_COMPILER_CACHE": "/bin/sccache",
                    **extra,
                }
                env = build.configure(build.ROOT, original)
                self.assertEqual(env["RUSTC_WRAPPER"], "")
                self.assertEqual(env["PSE_NATIVE_COMPILER_CACHE"], "")
                with self.assertRaisesRegex(ValueError, "disables supervised"):
                    build.configure(build.ROOT, original, cache="on")
            json_config.write_text('{"client_side_mode":false}')
            selected = build.configure(
                build.ROOT,
                {"RUSTC_WRAPPER": "/bin/sccache", "SCCACHE_CONF": str(json_config)},
                cache="on",
            )
            self.assertEqual(selected["SCCACHE_CONF"], str(json_config))
            self.assertEqual(selected["SCCACHE_CLIENT_SIDE"], "1")

    def test_target_directory_belongs_to_the_building_checkout(self) -> None:
        # The default <root>/target is Cargo's default and is never exported: sccache
        # keys rustc calls on their CARGO_* environment, so a per-checkout absolute path
        # would stop every other checkout from reusing the shared compiler cache.
        with tempfile.TemporaryDirectory() as other:
            foreign = {"CARGO_TARGET_DIR": str(Path(other) / "target")}
            self.assertNotIn("CARGO_TARGET_DIR", build.configure(build.ROOT, foreign))
        for default in (str(build.ROOT / "target"), "target"):
            self.assertNotIn(
                "CARGO_TARGET_DIR",
                build.configure(build.ROOT, {"CARGO_TARGET_DIR": default}),
            )
        self.assertNotIn("CARGO_TARGET_DIR", build.configure(build.ROOT, {}))
        nested = {"CARGO_TARGET_DIR": str(build.ROOT / "target/measure-production")}
        self.assertEqual(
            build.configure(build.ROOT, nested)["CARGO_TARGET_DIR"],
            nested["CARGO_TARGET_DIR"],
        )
        explicit = {"PSE_CARGO_TARGET_DIR": "/fast/disk/target", **nested}
        self.assertEqual(
            build.configure(build.ROOT, explicit)["CARGO_TARGET_DIR"],
            "/fast/disk/target",
        )

    def test_shell_exports_unset_the_default_target_directory(self) -> None:
        env = {**os.environ, "CARGO_TARGET_DIR": str(build.ROOT / "target")}
        out = subprocess.run(
            [sys.executable, "-m", "scripts.build_environment", "--shell"],
            cwd=build.ROOT,
            env=env,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        self.assertIn("unset CARGO_TARGET_DIR", out.splitlines())

    def test_toolchain_file_is_the_only_toolchain_authority(self) -> None:
        # The environment never selects a toolchain; rustup reads rust-toolchain.toml.
        self.assertNotIn("RUSTUP_TOOLCHAIN", build.configure(build.ROOT, {}))
        settings = build.tomllib.loads((build.ROOT / ".config/build.toml").read_text())
        self.assertEqual(set(settings), {"cache_size", "free_space_gib"})

    def test_frontend_preserves_flags_and_isolates_its_artifacts(self) -> None:
        env = build.configure(
            build.ROOT,
            {"RUSTFLAGS": "-C link-arg=-fuse-ld=mold"},
            cache="off",
            frontend=2,
            jobs=24,
        )
        self.assertEqual(
            env["CARGO_ENCODED_RUSTFLAGS"].split("\x1f"),
            ["-C", "link-arg=-fuse-ld=mold", "-Zthreads=2"],
        )
        self.assertEqual(env["CARGO_BUILD_JOBS"], "24")
        self.assertEqual(env["CARGO_TARGET_DIR"], str(build.ROOT / "target/frontend-2"))
        # A nested recipe keeps the experiment's directories and flags.
        self.assertEqual(build.configure(build.ROOT, env), env)
        with self.assertRaisesRegex(ValueError, "frontend flags already supplied"):
            build.configure(build.ROOT, env, frontend=4)
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
    def setUp(self) -> None:
        # Disposable prefixes must validate independently of an enclosing
        # assessment's already admitted native installations.
        environment = {
            name: value
            for name, value in os.environ.items()
            if name not in {"PSE_NATIVE_OPERATION", "PSE_NATIVE_HANDOFF"}
        }
        isolated = patch.dict(os.environ, environment, clear=True)
        isolated.start()
        self.addCleanup(isolated.stop)

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
            (prefix / required[0]).chmod(0o644)
            (prefix / required[0]).write_bytes(b"corrupt")
            repaired = native_cache.prepare(
                base, "fixture", identity, required, builder
            )
            self.assertNotEqual(repaired, prefix)
            self.assertEqual((prefix / required[0]).read_bytes(), b"corrupt")
            (repaired / required[0]).parent.chmod(0o755)
            (repaired / required[0]).unlink()
            repaired_again = native_cache.prepare(
                base, "fixture", identity, required, builder
            )
            self.assertNotEqual(repaired_again, repaired)
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
            self.assertTrue(native_cache.valid(repaired_again, identity, required))
            self.assertFalse(list((base / "fixture").glob("*/.stage-*")))

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
        with tempfile.TemporaryDirectory() as directory:
            prefix = Path(directory)
            for name in native_cache.SOLVER_FILES:
                path = prefix / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"explicit interface")
            result = subprocess.run(
                [
                    "bash",
                    "-c",
                    'source scripts/native-solver-env.sh; test "$IPOPT_DIR" = "$EXPECTED_PREFIX"',
                ],
                cwd=build.ROOT,
                env={
                    **os.environ,
                    "IPOPT_DIR": str(prefix),
                    "EXPECTED_PREFIX": str(prefix),
                },
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

    def test_native_runner_preserves_selected_canonical_profile_and_loopback(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docker = root / "docker"
            docker.write_text(
                f"#!{sys.executable}\nimport json, sys\nprint(json.dumps(sys.argv[1:]))\n"
            )
            docker.chmod(0o755)
            state = root / "canonical state"
            state.mkdir()
            environment = {
                key: value
                for key, value in os.environ.items()
                if key != "PSE_SURREAL_STATE"
            }
            environment["PATH"] = str(root) + os.pathsep + os.environ["PATH"]
            for selected, expected in [(None, "none"), (state, "host")]:
                current = dict(environment)
                if selected is not None:
                    current["PSE_SURREAL_STATE"] = str(selected)
                args = json.loads(
                    subprocess.check_output(
                        ["bash", "scripts/native-solver-runner.sh", "true"],
                        cwd=build.ROOT,
                        env=current,
                        text=True,
                    )
                )
                self.assertEqual(args[args.index("--network") + 1], expected)
                self.assertNotIn("PSE_DATABASE_URL", args)
                if selected is not None:
                    self.assertIn(f"{state}:{state}:ro", args)
                    self.assertIn(f"PSE_SURREAL_STATE={state}", args)

    def test_solver_image_override_must_be_immutable(self) -> None:
        def runtime(override: str | None) -> subprocess.CompletedProcess[str]:
            env = {k: v for k, v in os.environ.items() if k != "PSE_SOLVER_IMAGE"}
            if override is not None:
                env["PSE_SOLVER_IMAGE"] = override
            return subprocess.run(
                [sys.executable, "scripts/solver-images.py", "runtime", "dev"],
                cwd=build.ROOT,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )

        pinned = json.loads(
            (build.ROOT / ".github/setup/solver-images.json").read_text()
        )["dev"]
        self.assertEqual(runtime(None).stdout.strip(), pinned)
        self.assertEqual(runtime("").stdout.strip(), pinned)
        image_id = "sha256:" + "a" * 64
        digest = "pse-solvers@sha256:" + "b" * 64
        self.assertEqual(runtime(image_id).stdout.strip(), image_id)
        self.assertEqual(runtime(digest).stdout.strip(), digest)
        for mutable in ("pse-solvers:dev-local", "sha256:abc", "latest"):
            result = runtime(mutable)
            self.assertEqual(result.returncode, 2, mutable)
            self.assertIn("not immutable", result.stderr)

    def test_native_runner_uses_the_override_image(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docker = root / "docker"
            docker.write_text(
                f"#!{sys.executable}\nimport json, sys\nprint(json.dumps(sys.argv[1:]))\n"
            )
            docker.chmod(0o755)
            image_id = "sha256:" + "c" * 64
            args = json.loads(
                subprocess.check_output(
                    ["bash", "scripts/native-solver-runner.sh", "true"],
                    cwd=build.ROOT,
                    env={
                        **os.environ,
                        "PSE_SOLVER_IMAGE": image_id,
                        "PATH": str(root) + os.pathsep + os.environ["PATH"],
                    },
                    text=True,
                )
            )
            self.assertEqual(args[-2:], [image_id, "true"])


if __name__ == "__main__":
    unittest.main()
