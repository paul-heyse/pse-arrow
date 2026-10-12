# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixtures shared by the parity comparisons."""

import idaes
import pytest
from idaes import config

from . import support

# Importing IDAES puts its own solver directory first on PATH, ahead of the pinned
# solver prefix. The pinned builds stay first; IDAES's (PETSc) is the fallback.
idaes.cfg.use_idaes_solvers = False
config.reconfig(idaes.cfg)


@pytest.fixture(scope="session")
def parity_context(tmp_path_factory: pytest.TempPathFactory) -> support.ParityContext:
    """One Runtime and its lazily admitted immutable physical inputs."""
    return support.ParityContext(
        support.runtime(tmp_path_factory.mktemp("parity-spill"))
    )
