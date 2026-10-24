# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Checkout creation preserves selections without leaking its managed router."""

from __future__ import annotations

import os
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import worktree


class WorktreeEnvironmentTests(unittest.TestCase):
    def test_child_recomposes_only_source_checkout_managed_wrappers(self) -> None:
        caller = {
            "RUSTC_WRAPPER": str(worktree.ROOT / "scripts/sccache"),
            "PSE_NATIVE_COMPILER_CACHE": str(worktree.ROOT / "scripts/sccache"),
            "PSE_SCCACHE_BINARY": "/selected/real-sccache",
            "OTHER": "retained",
        }
        with patch.dict(os.environ, caller, clear=True):
            assert worktree.command_environment(worktree.ROOT) == caller
            child = worktree.command_environment(Path("/new/checkout"))
            assert "RUSTC_WRAPPER" not in child
            assert "PSE_NATIVE_COMPILER_CACHE" not in child
            assert child["PSE_SCCACHE_BINARY"] == caller["PSE_SCCACHE_BINARY"]
            assert dict(os.environ) == caller
        for wrapper in ("", "/custom/wrapper", "/explicit/sccache"):
            with (
                self.subTest(wrapper=wrapper),
                patch.dict(os.environ, {"RUSTC_WRAPPER": wrapper}, clear=True),
            ):
                assert (
                    worktree.command_environment(Path("/new/checkout"))["RUSTC_WRAPPER"]
                    == wrapper
                )

    def test_child_commands_receive_recomposed_environment(self) -> None:
        with (
            patch.dict(
                os.environ,
                {"RUSTC_WRAPPER": str(worktree.ROOT / "scripts/sccache")},
                clear=True,
            ),
            patch.object(worktree.subprocess, "run") as launch,
        ):
            worktree.run(["just", "skills-sync"], Path("/new/checkout"))
        assert launch.call_args.kwargs["env"] == {}
        assert launch.call_args.kwargs["check"]
