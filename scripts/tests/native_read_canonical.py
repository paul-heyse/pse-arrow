# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read the root-selected, live public canonical fixture handoff; never start a service."""

from __future__ import annotations

import json
import math
import os
import stat
import time
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import process_observation
from scripts.capabilities.native_schema import JSON_OBJECT

if TYPE_CHECKING:
    from pydantic import JsonValue


@dataclass(frozen=True)
class CanonicalReadSelection:
    state: str
    database: str
    arguments: dict[str, JsonValue]

    @classmethod
    def selected(cls) -> CanonicalReadSelection:
        supplied = os.environ.get("PSE_NATIVE_READ_CANONICAL_HANDOFF")
        if supplied is None:
            raise ValueError(
                "root must select the live owned canonical host-ready.json"
            )
        source = Path(supplied)
        info = source.lstat()
        if (
            source.is_symlink()
            or not stat.S_ISREG(info.st_mode)
            or info.st_uid != os.getuid()
            or info.st_size > 256 << 10
        ):
            raise ValueError(
                "canonical host handoff is not an owned bounded ordinary file"
            )
        value = json.loads(source.read_bytes())
        if not isinstance(value, dict):
            raise TypeError("canonical host handoff must be an object")
        deadline, controller = (
            value.get("deadline_monotonic"),
            value.get("controller_pid"),
        )
        if (
            not isinstance(deadline, (float, int))
            or isinstance(deadline, bool)
            or not math.isfinite(deadline)
            or deadline <= time.monotonic()
            or type(controller) is not int
        ):
            raise ValueError(
                "canonical fixture hold expired or lacks its live controller"
            )
        observed = process_observation.observe_process(controller)
        if observed.availability is not process_observation.Availability.PRESENT:
            raise ValueError("canonical fixture controller is not currently observable")
        release = value.get("release")
        if not isinstance(release, str) or Path(release).exists():
            raise ValueError("canonical fixture has already been released")
        reader = value.get("reader")
        if (
            not isinstance(reader, dict)
            or reader.get("capability") != "native_result_inspect"
        ):
            raise ValueError(
                "canonical fixture lacks its original public result reader"
            )
        arguments = JSON_OBJECT.validate_python(reader.get("arguments"))
        state, database = value.get("state"), value.get("database")
        if (
            not isinstance(state, str)
            or not isinstance(database, str)
            or arguments.get("state") != state
            or arguments.get("database") != database
        ):
            raise ValueError("canonical reader belongs to another selected deployment")
        request = arguments.get("request")
        if (
            not isinstance(request, dict)
            or request.get("relation") != "runtime.study_outcomes"
            or request.get("start") != 0
            or request.get("end") != 1
        ):
            raise ValueError(
                "control requires the original retained study outcome fixture window"
            )
        return cls(state, database, arguments)
