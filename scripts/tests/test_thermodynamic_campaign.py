# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Durable measurements keep the observer separate from native execution."""
# ruff: noqa: PT009 -- stdlib controls exercise command dispatch

from __future__ import annotations

import os
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_server, thermodynamic_campaign


class CampaignPlacementTests(unittest.TestCase):
    def test_campaign_enters_observer_before_constructing_runtime(self) -> None:
        with (
            patch.object(sys, "argv", ["campaign", "flash", "/owned/results"]),
            patch.dict(os.environ, {"PSE_SURREAL_STATE": "/owned/state"}),
            patch.object(surreal_server, "observer", return_value=7) as placed,
            patch.object(thermodynamic_campaign, "measure") as measure,
        ):
            self.assertEqual(thermodynamic_campaign.main(), 7)
        placed.assert_called_once_with(
            Path("/owned/state"),
            [
                sys.executable,
                "-m",
                "scripts.thermodynamic_campaign",
                "flash",
                "/owned/results",
                "--managed-primary-child",
            ],
        )
        measure.assert_not_called()

    def test_observer_child_retains_original_phase_and_output(self) -> None:
        with (
            patch.object(
                sys,
                "argv",
                ["campaign", "study", "/owned/results", "--managed-primary-child"],
            ),
            patch.object(surreal_server, "observer") as placed,
            patch.object(thermodynamic_campaign, "measure") as measure,
        ):
            self.assertEqual(thermodynamic_campaign.main(), 0)
        measure.assert_called_once_with(Path("/owned/results"), "study")
        placed.assert_not_called()
