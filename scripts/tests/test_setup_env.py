# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise preparation selection without building the native host or installing packages."""
# ruff: noqa: PT009 -- standard-library shell dispatch controls

from __future__ import annotations

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class SetupEnvironmentTests(unittest.TestCase):
    def test_selected_preparation_cannot_overwrite_another_environment(self) -> None:
        for script, mode, prefix, arguments in (
            (
                "setup-env",
                "tools",
                ".venv",
                ["sync", "--locked", "--only-group", "quality", "--no-install-project"],
            ),
            (
                "setup-env",
                "reference",
                ".venv-parity",
                [
                    "sync",
                    "--locked",
                    "--only-group",
                    "parity",
                    "--no-install-project",
                    "--python",
                    "3.13",
                ],
            ),
            (
                "bootstrap.sh",
                "--venv-only",
                ".venv",
                ["sync", "--locked", "--no-install-project"],
            ),
            (
                "bootstrap.sh",
                "--quality-only",
                ".venv",
                ["sync", "--locked", "--no-install-project", "--group", "quality"],
            ),
        ):
            with (
                self.subTest(script=script, mode=mode),
                tempfile.TemporaryDirectory(prefix="pse setup ") as temporary,
            ):
                root = Path(temporary)
                scripts = root / "scripts"
                scripts.mkdir()
                shutil.copyfile(ROOT / "scripts" / script, scripts / script)
                tools = root / "tools"
                tools.mkdir()
                log = root / "uv-arguments"
                for name in ("bootstrap-host", "pse-host"):
                    target = scripts / name
                    target.write_text("#!/bin/sh\nexit 0\n")
                    target.chmod(0o700)
                adapter = scripts / "pse-env"
                adapter.write_text('#!/bin/sh\n[ "$1" = -- ] && shift\nexec "$@"\n')
                adapter.chmod(0o700)
                cargo = tools / "cargo"
                cargo.write_text("#!/bin/sh\nexit 0\n")
                cargo.chmod(0o700)
                uv = tools / "uv"
                uv.write_text(
                    "#!/bin/sh\n"
                    'printf \'%s\\n\' "$UV_PROJECT_ENVIRONMENT" "${VIRTUAL_ENV-unset}" '
                    '"${UV_PYTHON-unset}" "${UV_PROJECT-unset}" '
                    '"${UV_WORKING_DIR-unset}" "${UV_NO_SYNC-unset}" '
                    '"${UV_FROZEN-unset}" "$@" > "$CALL_LOG"\n'
                    'mkdir -p "$UV_PROJECT_ENVIRONMENT"\n'
                    'printf selected > "$UV_PROJECT_ENVIRONMENT/prepared"\n'
                    'exit "${SYNC_EXIT:-0}"\n'
                )
                uv.chmod(0o700)
                foreign = root / "foreign"
                foreign.mkdir()
                sentinel = foreign / "preserve"
                sentinel.write_text("foreign")
                dev = root / ".venv"
                dev.mkdir()
                interpreter = dev / "interpreter"
                interpreter.write_text("development")
                environment = {
                    "PATH": f"{tools}:/usr/bin:/bin",
                    "HOME": str(root),
                    "CALL_LOG": str(log),
                    "UV_PROJECT_ENVIRONMENT": str(foreign),
                    "VIRTUAL_ENV": str(foreign),
                    "UV_PYTHON": "3.11",
                    "UV_PROJECT": str(foreign),
                    "UV_WORKING_DIR": str(foreign),
                    "UV_NO_SYNC": "1",
                    "UV_FROZEN": "1",
                    "UV_LOCKED": "1",
                }
                result = subprocess.run(
                    ["/bin/bash", str(scripts / script), mode],
                    cwd="/",
                    env=environment,
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=15,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(
                    log.read_text().splitlines(),
                    [str(root / prefix), *(["unset"] * 6), *arguments],
                )
                self.assertEqual(sentinel.read_text(), "foreign")
                self.assertFalse((foreign / "prepared").exists())
                self.assertEqual(interpreter.read_text(), "development")
                self.assertTrue((root / prefix / "prepared").is_file())
                if prefix != ".venv":
                    self.assertFalse((dev / "prepared").exists())
                failed = subprocess.run(
                    ["/bin/bash", str(scripts / script), mode],
                    cwd="/",
                    env={**environment, "SYNC_EXIT": "23"},
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=15,
                )
                self.assertEqual(failed.returncode, 23, failed.stderr)

    def test_invalid_setup_mode_does_not_prepare_python(self) -> None:
        # The usage error must precede uv resolution even in a hostile caller environment.
        result = subprocess.run(
            ["/bin/bash", str(ROOT / "scripts/setup-env"), "invalid"],
            env={"PATH": "/usr/bin:/bin", "UV_PROJECT_ENVIRONMENT": "/unrelated"},
            capture_output=True,
            text=True,
            check=False,
            timeout=15,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr)


if __name__ == "__main__":
    unittest.main()
