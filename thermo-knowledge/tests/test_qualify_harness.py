# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The oracle harness protocol against a fake harness run through the core interpreter: the
request and result files, the statuses, and the ways a harness is unavailable."""

from __future__ import annotations

import json
import math
from pathlib import Path

import pytest
from qualify_support import FIXTURES

from thermo_knowledge import config
from thermo_knowledge.qualify import harness
from thermo_knowledge.qualify.harness import HarnessUnavailable, Request, SubjectRequest

ORACLES = FIXTURES / "oracles"
COLUMN = [300.0, 350.0, 380.0]


def request(call: str = "ok", *keys: str, unit: str = "Pa") -> Request:
    subjects = tuple(SubjectRequest((key,), {"T": COLUMN}, len(COLUMN)) for key in keys)
    return Request("fake", call, {"T": "K"}, unit, subjects)


def test_a_point_above_the_reducing_temperature_is_nan_for_the_library() -> None:
    (points,) = harness.run(request("ok", "sp-c"), "core", oracles=ORACLES).subjects
    assert [p.status for p in points] == ["ok", "nan", "nan"]
    assert points[1].value is None


def test_a_result_holds_the_library_its_version_and_each_point_in_the_stated_units() -> None:
    result = harness.run(request("ok", "sp-a", "sp-b"), "core", oracles=ORACLES)
    assert (result.library, result.version) == ("FakeLib", "1.2.3")
    assert [len(points) for points in result.subjects] == [3, 3]
    first = result.subjects[0]
    assert all(point.status == "ok" and point.message is None for point in first)
    expected = 5.0e6 * math.exp(
        400.0
        / 300.0
        * sum(
            n * (1 - 300.0 / 400.0) ** t
            for n, t in zip([-7.5, 1.6, -3.2], [1.0, 1.5, 3.0], strict=True)
        )
    )
    assert first[0].value == pytest.approx(expected, rel=1e-14)


def test_the_request_file_states_the_call_the_units_and_each_subject_with_its_points() -> None:
    document = request("ok", "sp-a").as_json()
    assert document == {
        "protocol": 1,
        "library": "fake",
        "call": "ok",
        "arguments": {"T": "K"},
        "output": {"unit": "Pa"},
        "subjects": [{"key": ["sp-a"], "points": 3, "arguments": {"T": COLUMN}}],
    }
    json.dumps(document, allow_nan=False)


def test_nan_and_error_points_keep_their_status_and_message() -> None:
    (points,) = harness.run(request("mixed", "sp-b"), "core", oracles=ORACLES).subjects
    assert [p.status for p in points] == ["nan", "error", "ok"]
    assert points[0].value is None and points[0].message == "above the reducing temperature"
    assert points[1].value is None and points[1].message == "no such point"
    assert points[2].value is not None


def test_a_request_without_subjects_answers_with_the_library_and_version() -> None:
    result = harness.probe("fake", "ok", "core", {"T": "K"}, "Pa", oracles=ORACLES)
    assert (result.library, result.version, result.subjects) == ("FakeLib", "1.2.3", ())


def test_a_missing_harness_is_unavailable() -> None:
    with pytest.raises(HarnessUnavailable, match="does not exist"):
        harness.run(Request("nothing", "ok", {"T": "K"}, "Pa", ()), "core", oracles=ORACLES)


def test_a_harness_that_fails_is_unavailable_with_what_it_said() -> None:
    with pytest.raises(HarnessUnavailable, match=r"exited 3: fake: the library cannot start"):
        harness.run(request("crash", "sp-a"), "core", oracles=ORACLES)


@pytest.mark.parametrize(
    ("call", "message"),
    [
        ("wrong_unit", "answered in 'bar', the request states 'Pa'"),
        ("short", "no answer for each of 3 points"),
    ],
)
def test_a_result_the_protocol_does_not_allow_is_unavailable(call: str, message: str) -> None:
    with pytest.raises(HarnessUnavailable, match=message):
        harness.run(request(call, "sp-a"), "core", oracles=ORACLES)


def test_a_side_environment_runs_through_the_environment_launcher(tmp_path: Path) -> None:
    script = tmp_path / "x.py"
    line = harness.command(script, "thermotools", tmp_path / "q.json", tmp_path / "r.json")
    launcher = str(config.TREE_DIR / "envs" / "tk-env.sh")
    assert line[:5] == ["bash", launcher, "run", "thermotools", "python"]
    assert line[5:] == [
        str(script),
        "--request",
        str(tmp_path / "q.json"),
        "--result",
        str(tmp_path / "r.json"),
    ]
    core = harness.command(script, "core", tmp_path / "q.json", tmp_path / "r.json")
    assert core[1] == str(script) and core[0] != "bash"


def result_text(**changes: object) -> str:
    body: dict[str, object] = {
        "protocol": 1,
        "library": "L",
        "version": "1",
        "output_unit": "Pa",
        "subjects": [{"key": ["k"], "points": [{"value": 1.0, "status": "ok", "message": None}]}],
    }
    body.update(changes)
    return json.dumps(body)


ONE = Request("l", "c", {"T": "K"}, "Pa", (SubjectRequest(("k",), {"T": [1.0]}, 1),))


@pytest.mark.parametrize(
    ("text", "message"),
    [
        ("not json", "is not JSON"),
        (result_text(protocol=2), "does not follow protocol 1"),
        (result_text(version=""), "names no library and version"),
        (result_text(subjects=[]), "has 0 subjects"),
        (result_text(subjects=[{"key": ["other"], "points": []}]), "is not the requested"),
        (
            result_text(subjects=[{"key": ["k"], "points": [{"value": None, "status": "ok"}]}]),
            "an `ok` point has value None",
        ),
        (
            result_text(subjects=[{"key": ["k"], "points": [{"value": 1.0, "status": "nan"}]}]),
            "a `nan` point has a value",
        ),
        (
            result_text(subjects=[{"key": ["k"], "points": [{"value": 1.0, "status": "fine"}]}]),
            "is not ok, nan or error",
        ),
    ],
)
def test_the_result_is_validated_against_the_request(text: str, message: str) -> None:
    with pytest.raises(HarnessUnavailable, match=message):
        harness.parse_result(text, ONE)
    assert harness.parse_result(result_text(), ONE).subjects[0][0].value == 1.0
