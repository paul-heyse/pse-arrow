# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The oracle harness protocol (pipeline section 5.2).

The core writes a request file, runs `oracles/<library>.py --request <file> --result <file>` in
the environment the case declares and reads the result file. A harness never reads canonical
records or the database and never imports `thermo_knowledge`; the library answers from its own
bundled data. A harness that cannot run (no script, the library absent, a crash, a result the
protocol does not allow) is `HarnessUnavailable`, and the run is `blocked` with the reason.
"""

from __future__ import annotations

import json
import math
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from thermo_knowledge import config

PROTOCOL = 1
ORACLES_DIR = "oracles"
TIMEOUT_SECONDS = 1800
Status = Literal["ok", "nan", "error"]
_STATUSES = ("ok", "nan", "error")


class HarnessUnavailable(Exception):
    """The harness could not produce a result; the message is the reason a run is blocked."""


@dataclass(frozen=True)
class SubjectRequest:
    key: tuple[str, ...]
    """The library's own key of each subject role, in the order of the slot group's roles."""
    arguments: dict[str, list[float]]
    """Aligned columns of argument values, one entry per point, in `Request.arguments` units."""
    points: int
    """The number of points (the length of every column)."""


@dataclass(frozen=True)
class Request:
    library: str
    call: str
    arguments: dict[str, str]
    """Argument name to the unit its values are stated in."""
    output_unit: str
    subjects: tuple[SubjectRequest, ...]

    def as_json(self) -> dict[str, object]:
        return {
            "protocol": PROTOCOL,
            "library": self.library,
            "call": self.call,
            "arguments": self.arguments,
            "output": {"unit": self.output_unit},
            "subjects": [
                {"key": list(subject.key), "points": subject.points, "arguments": subject.arguments}
                for subject in self.subjects
            ],
        }


@dataclass(frozen=True)
class Point:
    value: float | None
    status: Status
    message: str | None


@dataclass(frozen=True)
class Result:
    library: str
    version: str
    subjects: tuple[tuple[Point, ...], ...]
    """The points of each subject of the request, in its order."""


def harness_script(library: str, oracles: Path | None = None) -> Path:
    return (oracles if oracles is not None else config.TREE_DIR / ORACLES_DIR) / f"{library}.py"


def command(
    script: Path, environment: str, request: Path, result: Path, tree: Path | None = None
) -> list[str]:
    """The command line that runs `script` in `environment`: the interpreter of this process for
    `core`, else `envs/tk-env.sh run <environment> python`."""
    arguments = ["--request", str(request), "--result", str(result)]
    if environment == "core":
        return [sys.executable, str(script), *arguments]
    launcher = (tree if tree is not None else config.TREE_DIR) / "envs" / "tk-env.sh"
    return ["bash", str(launcher), "run", environment, "python", str(script), *arguments]


def _fail(reason: str) -> HarnessUnavailable:
    return HarnessUnavailable(reason)


def parse_result(text: str, request: Request) -> Result:
    """Validate a result file against the request it answers."""
    try:
        data = json.loads(text)
    except json.JSONDecodeError as error:
        raise _fail(f"the result file is not JSON: {error}") from error
    if not isinstance(data, dict) or data.get("protocol") != PROTOCOL:
        raise _fail(f"the result file does not follow protocol {PROTOCOL}")
    library, version = data.get("library"), data.get("version")
    if not (isinstance(library, str) and library and isinstance(version, str) and version):
        raise _fail("the result file names no library and version")
    if data.get("output_unit") != request.output_unit:
        raise _fail(
            f"the harness answered in {data.get('output_unit')!r}, the request states "
            f"{request.output_unit!r}"
        )
    answered = data.get("subjects")
    if not isinstance(answered, list) or len(answered) != len(request.subjects):
        raise _fail(
            f"the result has {len(answered) if isinstance(answered, list) else 'no'} subjects, "
            f"the request {len(request.subjects)}"
        )
    subjects: list[tuple[Point, ...]] = []
    for asked, given in zip(request.subjects, answered, strict=True):
        if not isinstance(given, dict) or given.get("key") != list(asked.key):
            raise _fail(f"the result's subject {given!r} is not the requested {list(asked.key)}")
        points = given.get("points")
        if not isinstance(points, list) or len(points) != asked.points:
            raise _fail(
                f"subject {list(asked.key)}: the result has no answer for each of "
                f"{asked.points} points"
            )
        parsed: list[Point] = []
        for item in points:
            status = item.get("status") if isinstance(item, dict) else None
            if status not in _STATUSES:
                raise _fail(
                    f"subject {list(asked.key)}: point status {status!r} is not ok, nan or error"
                )
            value = item.get("value")
            message = item.get("message")
            if status == "ok":
                if (
                    isinstance(value, bool)
                    or not isinstance(value, int | float)
                    or not math.isfinite(value)
                ):
                    raise _fail(f"subject {list(asked.key)}: an `ok` point has value {value!r}")
                parsed.append(Point(float(value), "ok", None))
            else:
                if value is not None:
                    raise _fail(f"subject {list(asked.key)}: a `{status}` point has a value")
                parsed.append(Point(None, status, None if message is None else str(message)))
        subjects.append(tuple(parsed))
    return Result(library, version, tuple(subjects))


def run(
    request: Request,
    environment: str,
    *,
    oracles: Path | None = None,
    tree: Path | None = None,
    timeout: float = TIMEOUT_SECONDS,
) -> Result:
    """Run the harness of `request.library` in `environment` and return its validated result."""
    script = harness_script(request.library, oracles)
    if not script.is_file():
        raise _fail(f"the harness {script} does not exist")
    with tempfile.TemporaryDirectory(prefix="tk-qualify-") as work:
        request_file, result_file = Path(work) / "request.json", Path(work) / "result.json"
        request_file.write_text(json.dumps(request.as_json(), allow_nan=False), encoding="utf-8")
        line = command(script, environment, request_file, result_file, tree)
        try:
            done = subprocess.run(  # noqa: S603 - a fixed interpreter and our own script
                line,
                capture_output=True,
                text=True,
                timeout=timeout,
                check=False,
                cwd=config.TREE_DIR,
            )
        except subprocess.TimeoutExpired as error:
            raise _fail(f"the harness did not finish within {timeout:g} s") from error
        except OSError as error:
            raise _fail(f"the harness cannot be started ({line[0]}): {error}") from error
        if done.returncode != 0:
            tail = (done.stderr or done.stdout).strip().splitlines()[-3:]
            raise _fail(f"the harness exited {done.returncode}: {' | '.join(tail) or 'no output'}")
        if not result_file.is_file():
            raise _fail("the harness wrote no result file")
        return parse_result(result_file.read_text(encoding="utf-8"), request)


def probe(
    library: str,
    call: str,
    environment: str,
    arguments: dict[str, str],
    output_unit: str,
    *,
    oracles: Path | None = None,
    tree: Path | None = None,
) -> Result:
    """A request with no subjects: what a harness answers with is the library and its version."""
    return run(
        Request(library, call, arguments, output_unit, ()),
        environment,
        oracles=oracles,
        tree=tree,
    )
