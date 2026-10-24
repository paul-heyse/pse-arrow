# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Durable measurements keep the observer separate from native execution."""
# ruff: noqa: PT009 -- stdlib controls exercise command dispatch

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import scientific_inputs, surreal_server, thermodynamic_campaign


class CampaignPlacementTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.output = Path(temporary.name) / "results"

    def test_campaign_enters_observer_before_constructing_runtime(self) -> None:
        prepared = self.output / "scientific-inputs.json"

        def capture(destination: Path, environment: dict) -> Path:
            self.assertEqual(environment["PSE_SURREAL_STATE"], "/owned/state")
            destination.write_text(
                json.dumps(
                    {"schema": "scientific-inputs-v1", "files": {}, "families": {}}
                )
            )
            return destination

        def observer(*_args: object) -> int:
            self.assertEqual(os.environ[scientific_inputs.ROLE], str(prepared))
            self.assertTrue(prepared.is_file())
            return 7

        with (
            patch.object(sys, "argv", ["campaign", "flash", str(self.output)]),
            patch.dict(os.environ, {"PSE_SURREAL_STATE": "/owned/state"}, clear=True),
            patch.object(scientific_inputs, "capture", side_effect=capture) as captured,
            patch.object(surreal_server, "observer", side_effect=observer) as placed,
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
                str(self.output),
                "--managed-primary-child",
            ],
        )
        captured.assert_called_once()
        measure.assert_not_called()

    def test_observer_child_retains_original_phase_and_output(self) -> None:
        with (
            patch.object(
                sys,
                "argv",
                ["campaign", "study", "/owned/results", "--managed-primary-child"],
            ),
            patch.object(surreal_server, "observer") as placed,
            patch.object(scientific_inputs, "capture") as captured,
            patch.object(thermodynamic_campaign, "measure") as measure,
        ):
            self.assertEqual(thermodynamic_campaign.main(), 0)
        measure.assert_called_once_with(Path("/owned/results"), "study")
        placed.assert_not_called()
        captured.assert_not_called()

    def test_invalid_explicit_capsule_blocks_observer_without_recapture(self) -> None:
        supplied = self.output.parent / "supplied.json"
        supplied.write_text('{"schema":"other"}')
        with (
            patch.object(sys, "argv", ["campaign", "flash", str(self.output)]),
            patch.dict(
                os.environ,
                {
                    "PSE_SURREAL_STATE": "/owned/state",
                    scientific_inputs.ROLE: str(supplied),
                },
                clear=True,
            ),
            patch.object(scientific_inputs, "capture") as captured,
            patch.object(surreal_server, "observer") as placed,
            self.assertRaises(ValueError),
        ):
            thermodynamic_campaign.main()
        captured.assert_not_called()
        placed.assert_not_called()

    def test_authored_documents_delegate_to_captured_conformance_loader(self) -> None:
        path = Path("packages/reference/physical")
        frozen = {"materials/physical.yaml": b"captured"}
        loader = SimpleNamespace(
            _documents=lambda selected: frozen if selected == path else {}
        )
        with patch.dict(sys.modules, {"pse.conformance": loader}):
            self.assertIs(thermodynamic_campaign.authored_documents(path), frozen)
