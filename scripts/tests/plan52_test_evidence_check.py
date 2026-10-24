# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit finite C01/C03 controls over the existing scientific replay boundary.

Run through scripts/pse-env --resource-class build -- .venv/bin/python -m
scripts.tests.plan52_test_evidence_check --output build/assessment/NEW-DIRECTORY.
The rejection line receives a comment only; exact original bytes are restored.
No production behavior, test assertion or repository policy is changed.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import json
import os
import subprocess
from pathlib import Path
from unittest.mock import patch

from scripts import affected, assertion_sensitivity, coverage_correspondence, validation

ROOT = Path(__file__).resolve().parents[2]
TARGET = "crates/pse-ids/src/scientific_replay.rs"


def affected_fallback(output: Path, capture: Path) -> list[dict]:
    """A second real source edit makes stale history optional to the same reach."""
    source = ROOT / TARGET
    original = source.read_bytes()
    probe = original.replace(
        b"if payload_bytes > MAX_ADMITTED_RECIPE_BYTES {",
        b"if payload_bytes > MAX_ADMITTED_RECIPE_BYTES { // stale history fallback probe",
        1,
    )
    if probe == original:
        raise ValueError("expected exact scientific replay rejection predicate")
    try:
        source.write_bytes(probe)
        fallback = []
        for selected in (None, capture, output / "missing"):
            stream = io.StringIO()
            with contextlib.redirect_stdout(stream):
                code = affected.main(
                    [
                        "--json",
                        "--base",
                        "HEAD",
                        *(["--coverage", str(selected)] if selected else []),
                    ]
                )
            assert code == 0
            fallback.append(json.loads(stream.getvalue()))
        assert "pse-ids" in fallback[0]["changed_packages"]
        assert fallback[0]["selected_packages"]
        assert all(row["command"] == fallback[0]["command"] for row in fallback)
        assert fallback[1]["coverage"]["status"] == "stale-or-unsuccessful"
        assert fallback[2]["coverage"]["status"] == "unavailable"
        return fallback
    finally:
        if source.read_bytes() != probe:
            raise RuntimeError(
                "scientific replay source changed outside fallback probe"
            )
        source.write_bytes(original)


def run(output: Path) -> int:
    output = validation.fresh_output(ROOT, output, "test-evidence")
    source = ROOT / TARGET
    original = source.read_bytes()
    old = b"if payload_bytes > MAX_ADMITTED_RECIPE_BYTES {"
    changed = original.replace(old, old + b" // selected rejection coverage probe", 1)
    if changed == original:
        raise ValueError("expected exact scientific replay rejection predicate")
    try:
        source.write_bytes(changed)
        diff = subprocess.check_output(["git", "diff", "--", TARGET], cwd=ROOT)
        (output / "selected.diff").write_bytes(diff)
        before = output / "unreached"
        after = output / "reached"
        assert (
            coverage_correspondence.capture(
                before,
                [
                    "-p",
                    "pse-ids",
                    "--lib",
                    "-E",
                    "test(=float::tests::signed_zero_is_preserved_and_distinguished)",
                ],
                True,
            )
            == 0
        )
        missing = coverage_correspondence.inspect(before, "HEAD")
        validation.write_json(output / "unreached-fresh.json", missing)
        assert missing["fresh"] and missing["population_status"] == "observed"
        row = next(row for row in missing["regions"] if row["path"] == TARGET)
        assert row["branches"] and all(
            branch["true_count"] == 0 for branch in row["branches"]
        )
        assert (
            coverage_correspondence.capture(
                after,
                [
                    "-p",
                    "pse-ids",
                    "--lib",
                    "-E",
                    "test(=scientific_replay::tests::scientific_replay_authority_binds_exact_records)",
                ],
                True,
            )
            == 0
        )
        reached = coverage_correspondence.inspect(after, "HEAD")
        validation.write_json(output / "reached-fresh.json", reached)
        assert reached["fresh"] and reached["population_status"] == "observed"
        row = next(row for row in reached["regions"] if row["path"] == TARGET)
        assert any(branch["true_count"] > 0 for branch in row["branches"])
        assert len(reached["population"]) == 1 and reached["population"][0][
            "test"
        ].endswith("scientific_replay_authority_binds_exact_records")
        with patch.dict(
            os.environ,
            {"RUSTFLAGS": os.environ.get("RUSTFLAGS", "") + " -C opt-level=1"},
        ):
            drift = coverage_correspondence.inspect(after, "HEAD")
        validation.write_json(output / "build-drift.json", drift)
        assert not drift["fresh"] and drift["regions"] == []
    finally:
        # Refuse to overwrite any unexpected concurrent edit of this narrow probe.
        if source.read_bytes() != changed:
            raise RuntimeError(
                "scientific replay source changed outside the owned probe; original preserved in memory"
            )
        source.write_bytes(original)
    stale = coverage_correspondence.inspect(output / "reached", "HEAD")
    validation.write_json(output / "source-drift.json", stale)
    assert not stale["fresh"] and stale["regions"] == []
    fallback = affected_fallback(output, output / "reached")
    validation.write_json(output / "affected-fallback.json", fallback)
    pattern = r"replace > with >= in QualifiedScientificRecipe::scratch_bytes"
    selections = (
        (
            "insufficient",
            "scientific_replay_authority_binds_exact_records",
            "missed",
            2,
        ),
        (
            "adequate",
            "scientific_replay_large_logical_description_preserves_record_binding",
            "caught",
            0,
        ),
    )
    mutations = []
    for label, name, expected, expected_exit in selections:
        directory = output / label
        code = assertion_sensitivity.run(
            TARGET,
            directory,
            [
                "-p",
                "pse-ids",
                "--lib",
                "-E",
                f"test(=scientific_replay::tests::{name})",
            ],
            pattern,
            None,
            False,
        )
        record = json.loads((directory / "sensitivity.json").read_text())
        assert code == expected_exit, (label, code)
        assert (
            record["baseline_status"] == "passed"
            and record["protocol_status"] == "observed"
        )
        assert record["unchanged_during_run"]
        outcomes = [
            row
            for row in record["outcomes"]
            if isinstance(row["original"]["scenario"], dict)
        ]
        assert len(outcomes) == 1 and outcomes[0]["classification"] == expected
        baseline = next(
            row["original"]
            for row in record["outcomes"]
            if row["original"]["scenario"] == "Baseline"
        )
        build = next(
            phase for phase in baseline["phase_results"] if phase["phase"] == "Build"
        )
        assert "--locked" in build["argv"] and "--lib" in build["argv"]
        mutations.append(
            {
                "selection": label,
                "tool_exit": code,
                "baseline": "passed",
                "mutant": expected,
                "path": str(directory / "sensitivity.json"),
            }
        )
    assert source.read_bytes() == original
    validation.write_json(
        output / "control.json",
        {
            "source": TARGET,
            "restored_sha256": hashlib.sha256(original).hexdigest(),
            "coverage": "same changed rejection branch: unreached then reached; one actual nextest test per capture",
            "drift": "source restoration and compiler flag change both stale; affected selection unchanged",
            "mutation": mutations,
            "assertion": "MAX_ADMITTED_RECIPE_BYTES is legal independently of the implementation; > to >= survives ordinary/oversized checks but is rejected by the exact maximum-byte counterexample",
            "limits": "pure pse-ids scientific authority boundary only; native solvers and full campaigns not qualified",
        },
    )
    print(output / "control.json")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    return run(parser.parse_args().output)


if __name__ == "__main__":
    raise SystemExit(main())
