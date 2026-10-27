# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Constructor-issued executable fixture for scoped consumer controls."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

from scripts import artifact_publication, python_extension

if TYPE_CHECKING:
    from pathlib import Path


def executable(root: Path, tool: Path) -> artifact_publication.Publication:
    return artifact_publication.publish(
        python_extension.stage_receipt("evidence-executable", os.environ, root=root),
        kind="evidence-executable",
        source=tool,
        mode="dev",
        profile="dev",
        validated=True,
        interpreter=str(python_extension.interpreter(os.environ).absolute()),
        features=lambda _candidate: ["force-validate", "evidence-tools"],
    )
