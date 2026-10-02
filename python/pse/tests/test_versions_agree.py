# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One version number for the wheel, the extension and the workspace (plan §5)."""

import subprocess
import sys
from importlib.metadata import version

import pytest

import pse
from pse import _native
from pse._build import build_info
from pse.contracts.documents import BuildInfo


@pytest.mark.unit
def test_package_extension_and_metadata_agree() -> None:
    distribution = version("pse-arrow")
    assert pse.__version__ == distribution
    assert pse.__version__ == _native.__version__


@pytest.mark.unit
def test_build_info_structures() -> None:
    info = build_info()
    assert isinstance(info, BuildInfo)
    assert info.version == pse.__version__
    assert info.rustc_version
    assert info.profile


@pytest.mark.unit
def test_public_build_info_and_identity_startup_in_fresh_interpreter() -> None:
    program = """
import pse
from pse.contracts.documents import BuildInfo
from pse.contracts.values import ContentHash, SemanticId

assert pse.BuildInfo is BuildInfo
info = pse.build_info()
assert isinstance(info, BuildInfo)
assert info.version == pse.__version__
identity = "01" * 16
assert SemanticId.from_hex(identity).to_hex() == identity
digest = "blake3:" + "02" * 32
assert ContentHash.from_prefixed(digest).to_prefixed() == digest
"""
    subprocess.run(
        [sys.executable, "-c", program],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )


@pytest.mark.unit
def test_extension_belongs_to_this_checkout(
    build_info_matches_checkout: None,
) -> None:
    assert build_info_matches_checkout is None
