# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit selected lifecycle peer; invoked by canonical_boundary_run restart."""

import os
from pathlib import Path

import pytest

from scripts.tests.canonical_boundary_run import restart_journey


@pytest.mark.component
@pytest.mark.test_requirements(canonical=True, roles=("external-restart",))
def test_retained_handle_external_restart(request: pytest.FixtureRequest) -> None:
    restart_journey(
        Path(os.environ["PSE_CANONICAL_RESTART_DIRECTORY"]), request.node.nodeid
    )
