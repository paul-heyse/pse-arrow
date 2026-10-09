# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Prepared KLU admission for immutable runtime launches."""

from __future__ import annotations

import contextlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import pytest

from scripts import native_cache as cache
from scripts import native_operation as operation


class PreparedKluTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.base = Path(scratch.name)
        self.prefix = self.base / "selected"
        self.populate(self.prefix)
        self.env = {
            "SUITESPARSE_INCLUDE_DIR": str(self.prefix / "include/suitesparse"),
            "SUITESPARSE_LIBRARY_DIR": str(self.prefix / "lib"),
        }
        self.stack = contextlib.ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(
            patch.object(cache, "cache_root", return_value=self.base)
        )
        self.stack.enter_context(patch.object(cache, "compiler_env", side_effect=dict))
        self.stack.enter_context(
            patch.object(operation, "owner_record", return_value=None)
        )
        self.prepare = self.stack.enter_context(
            patch.object(cache, "klu", return_value=self.prefix)
        )

    def populate(self, prefix: Path) -> None:
        for name in cache.KLU_FILES:
            path = prefix / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"owned consumed interface")

    def test_prepared_prefix_is_byte_admitted_without_cargo_preparation(self) -> None:
        with (
            patch.object(
                cache, "admit_external", wraps=cache.admit_external
            ) as admission,
            patch.object(operation, "admit") as record,
        ):
            actual = operation.environment(["klu"], self.env)
        admission.assert_called_once_with(self.prefix, cache.KLU_FILES)
        self.prepare.assert_not_called()
        assert (
            (actual["SUITESPARSE_LIBRARY_DIR"]) == (self.env["SUITESPARSE_LIBRARY_DIR"])
        )
        prefix_record = record.call_args_list[0].args[1]
        assert not (prefix_record["managed"])
        assert (set(prefix_record["files"])) == (set(cache.KLU_FILES))

    def test_missing_consumed_file_refuses_instead_of_preparing(self) -> None:
        (self.prefix / "lib/libklu.a").unlink()
        with pytest.raises(ValueError, match="lacks required consumed interfaces"):
            operation.environment(["klu"], self.env)
        self.prepare.assert_not_called()

    def test_partial_conflicting_and_relative_paths_refuse(self) -> None:
        for env in (
            {"SUITESPARSE_INCLUDE_DIR": self.env["SUITESPARSE_INCLUDE_DIR"]},
            {**self.env, "SUITESPARSE_LIBRARY_DIR": str(self.base / "other/lib")},
            {**self.env, "SUITESPARSE_INCLUDE_DIR": "relative/include/suitesparse"},
            {**self.env, "SUITESPARSE_LIBRARY_DIR": ""},
        ):
            with (
                self.subTest(env=env),
                pytest.raises(ValueError, match="explicit KLU paths"),
            ):
                operation.environment(["klu"], env)
        self.prepare.assert_not_called()

    def test_absent_paths_keep_development_preparation(self) -> None:
        actual = operation.environment(["klu"], {})
        self.prepare.assert_called_once()
        assert (
            (actual["SUITESPARSE_INCLUDE_DIR"]) == (self.env["SUITESPARSE_INCLUDE_DIR"])
        )

    def test_corrupt_owned_completed_generation_refuses_without_preparation(
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
        env = {
            "SUITESPARSE_INCLUDE_DIR": str(prefix / "include/suitesparse"),
            "SUITESPARSE_LIBRARY_DIR": str(prefix / "lib"),
        }
        with pytest.raises(ValueError, match="generation is corrupt"):
            operation.environment(["klu"], env)
        self.prepare.assert_not_called()


if __name__ == "__main__":
    unittest.main()
