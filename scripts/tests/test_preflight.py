# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Preflight names the first missing prerequisite with its fix and exits 125."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import preflight


class PreflightTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def manifest(self, dependencies: str) -> Path:
        for name in ("pkg", "physical"):
            (self.root / name).mkdir()
            (self.root / name / "package.toml").write_text("")
        path = self.root / "conformance.toml"
        path.write_text(
            '[settings]\nmemory_limit_bytes = 1\n\n[[runs]]\nname = "one"\n'
            f'package = "pkg"\nphysical = "physical"\ndependencies = {dependencies}\n'
        )
        return path

    def test_conformance_manifest_requires_every_package_root(self) -> None:
        preflight.check(["conformance"], {}, self.manifest('["physical"]'))
        (self.root / "conformance.toml").unlink()
        for name in ("pkg", "physical"):
            (self.root / name / "package.toml").unlink()
            (self.root / name).rmdir()
        with self.assertRaisesRegex(
            preflight.BoundaryError, "'absent', which has no package.toml"
        ):
            preflight.check(["conformance"], {}, self.manifest('["absent"]'))

    def test_incomplete_explicit_solver_prefix_is_refused_without_preparation(
        self,
    ) -> None:
        with (
            patch.object(preflight.native_cache, "solver") as solver,
            self.assertRaisesRegex(
                preflight.BoundaryError, "prerequisite native: IPOPT_DIR="
            ),
        ):
            preflight.check(["native"], {"IPOPT_DIR": str(self.root)})
        solver.assert_not_called()

    def test_missing_store_exits_125_with_its_fix(self) -> None:
        stderr = io.StringIO()
        with (
            patch.dict(os.environ, {"PSE_SURREAL_STATE": str(self.root / "state")}),
            contextlib.redirect_stderr(stderr),
        ):
            code = preflight.main(["store"])
        self.assertEqual(code, 125)
        self.assertRegex(
            stderr.getvalue(), r"^pse-env: prerequisite store: .*just surreal setup"
        )

    def test_order_reports_the_first_missing_prerequisite(self) -> None:
        with (
            patch.object(preflight.doctor, "extension_kind", return_value="dev"),
            self.assertRaisesRegex(
                preflight.BoundaryError, "native-extension: .*py-sync-native"
            ),
        ):
            preflight.check(["extension", "native-extension", "store"], {})


if __name__ == "__main__":
    unittest.main()
