# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Retired KLU construction refuses; legacy artifact reads retain their contract."""

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import pytest

from scripts import build_environment
from scripts import native_cache as cache
from scripts import native_operation as operation


class PreparedKluTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.base = Path(scratch.name)
        self.prefix = self.base / "selected"
        self.populate(self.prefix)

    def populate(self, prefix: Path) -> None:
        for name in cache.KLU_FILES:
            path = prefix / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"owned consumed interface")

    def test_retired_klu_refuses_before_preparation_or_timing(self) -> None:
        for requested in (["klu"], ["compiler", "klu"], ["solver", "klu"]):
            with (
                self.subTest(requested=requested),
                patch.object(cache, "cache_root", side_effect=AssertionError("cache")),
                patch.object(cache, "compiler_env", side_effect=AssertionError("compiler")),
                patch.object(cache, "solver", side_effect=AssertionError("image")),
                patch.object(operation, "current", side_effect=AssertionError("timing")),
                pytest.raises(ValueError, match="Python KLU preparation is retired"),
            ):
                operation.environment(requested, {operation.BUILD_TIMING: "must-not-write"})

    def test_retired_cli_refuses_before_environment_or_receiver_effects(self) -> None:
        with (
            patch.object(
                operation.sys,
                "argv",
                [
                    "native_operation",
                    "--capabilities",
                    "solver,klu",
                    "--prepared-environment",
                    "must-not-read",
                    "--",
                    "must-not-run",
                ],
            ),
            patch.object(
                build_environment, "configure", side_effect=AssertionError("compiler")
            ),
            patch.object(cache, "cache_root", side_effect=AssertionError("cache")),
            patch.object(
                operation.receiver_environment,
                "read",
                side_effect=AssertionError("receiver"),
            ),
            pytest.raises(
                ValueError, match="native prepare --capabilities compiler,klu"
            ),
        ):
            operation.main()

    def test_legacy_cache_cli_has_no_klu_constructor(self) -> None:
        with (
            patch.object(cache.sys, "argv", ["native_cache", "klu"]),
            patch.object(cache, "cache_root", side_effect=AssertionError("cache")),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            with pytest.raises(SystemExit) as stopped:
                cache.main()
            assert stopped.value.code == 2

    def test_prior_managed_admission_returns_truth_without_reobserving_bytes(
        self,
    ) -> None:
        prior = {"managed": True, "required": list(cache.KLU_FILES)}
        with (
            patch.object(operation, "remembered", return_value=prior),
            patch.object(
                cache,
                "installation_files",
                side_effect=AssertionError("qualified immutable bytes reobserved"),
            ),
        ):
            assert cache.admit_external(self.prefix, cache.KLU_FILES)

    def test_legacy_corrupt_owned_generation_still_refuses_reader_admission(
        self,
    ) -> None:
        identity = {"fixture": "klu"}
        owner = cache.identity_location(self.base, "klu", identity)
        prefix = owner / "generations" / ("a" * 32)
        self.populate(prefix)
        receipt = {
            "version": cache.RECEIPT_VERSION,
            "identity": identity,
            "files": cache.installation_files(prefix),
        }
        (prefix / ".complete.json").write_text(json.dumps(receipt))
        (prefix / "lib/libklu.a").write_bytes(b"changed after completion")
        with (
            patch.object(cache, "cache_root", return_value=self.base),
            patch.object(operation, "remembered", return_value=None),
            patch.object(operation, "pin"),
            pytest.raises(ValueError, match="generation is corrupt"),
        ):
            cache.admit_external(prefix, cache.KLU_FILES)


if __name__ == "__main__":
    unittest.main()
