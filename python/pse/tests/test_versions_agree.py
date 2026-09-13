# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One version number for the wheel, the extension and the workspace (plan §5)."""

from importlib.metadata import version

import pytest

import pse
from pse import _native
from pse._build import BuildInfo, build_info


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
def test_extension_belongs_to_this_checkout(
    build_info_matches_checkout: None,
) -> None:
    assert build_info_matches_checkout is None
