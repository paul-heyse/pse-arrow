# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixtures shared by the parity comparisons."""

import pytest

import pse

from . import support


@pytest.fixture(scope="session")
def runtime(tmp_path_factory: pytest.TempPathFactory) -> pse.Runtime:
    """The one runtime of the comparisons' process."""
    return support.runtime(tmp_path_factory.mktemp("parity-spill"))
