# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
import contextlib

# ruff: noqa: PT009, PT027 -- stdlib unittest controls
import hashlib
import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import affected
from scripts import coverage_correspondence as coverage


def junit(identity: str, name: str = "selected") -> bytes:
    return f'<testsuites uuid="{identity}"><testsuite><testcase classname="actual-binary" name="{name}"/></testsuite></testsuites>'.encode()


class CorrespondenceTests(unittest.TestCase):
    def test_rejection_branch_is_missing_then_observed_without_test_attribution(
        self,
    ) -> None:
        path = "crates/example/src/lib.rs"
        file = {
            "filename": str(coverage.ROOT / path),
            "segments": [[3, 1, 0, True, True, False], [4, 1, 0, False, False, False]],
            "branches": [[3, 1, 3, 8, 0, 1, 0, 0, 4]],
        }
        report = {
            "type": "llvm.coverage.json.export",
            "version": "3.1.0",
            "data": [{"files": [file]}],
        }
        missing = coverage.correspondence(report, {path: [{"start": 3, "lines": 1}]})[0]
        self.assertEqual(missing["missing_lines"], [3])
        self.assertEqual(missing["branches"][0]["true_count"], 0)
        file["segments"][0][2] = 1
        file["branches"][0][4] = 1
        reached = coverage.correspondence(report, {path: [{"start": 3, "lines": 1}]})[0]
        self.assertEqual(reached["covered_lines"], [3])
        self.assertEqual(reached["missing_lines"], [])
        self.assertEqual(
            coverage.correspondence(
                report, {"crates/example/tests/a.rs": [{"start": 1, "lines": 1}]}
            ),
            [],
        )

    def test_actual_nextest_store_location_and_stale_junit_are_distinct(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".config").mkdir()
            (root / ".config/nextest.toml").write_text(
                '[profile.ci.junit]\npath="junit.xml"\n'
            )
            report = root / "target/nextest/ci/junit.xml"
            report.parent.mkdir(parents=True)
            report.write_bytes(junit("old"))
            for changed in (False, True):
                output = root / ("fresh" if changed else "stale")
                output.mkdir()

                def run(
                    *_args: object, changed: bool = changed, **_kwargs: object
                ) -> subprocess.CompletedProcess[bytes]:
                    if changed:
                        report.write_bytes(junit("new", "only-executed"))
                    return subprocess.CompletedProcess([], 0 if changed else 1)

                with (
                    patch.object(coverage, "ROOT", root),
                    patch.object(
                        coverage.validation, "fresh_output", return_value=output
                    ),
                    patch.object(
                        coverage, "context", return_value={"exact": "context"}
                    ),
                    patch.object(
                        coverage.arrow_validation,
                        "compose",
                        side_effect=lambda command: command,
                    ),
                    patch.object(coverage.subprocess, "run", side_effect=run),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    coverage.capture(output, ["-p", "selected"], True)
                capture = json.loads((output / "capture.json").read_text())
                self.assertEqual(
                    capture["population_status"],
                    "observed" if changed else "unavailable",
                )
                if changed:
                    self.assertEqual(
                        capture["population"],
                        [
                            {
                                "binary": "actual-binary",
                                "test": "only-executed",
                                "outcome": "passed",
                            }
                        ],
                    )
                    self.assertEqual(
                        capture["population_sha256"],
                        hashlib.sha256(
                            (output / "population.xml").read_bytes()
                        ).hexdigest(),
                    )
                else:
                    self.assertIsNone(capture["population"])
                    self.assertIn("not replaced", capture["population_error"])

    def test_junit_refuses_bounded_dtd_and_entity_declarations(self) -> None:
        self.assertEqual(
            coverage.population(junit("control")),
            [
                {
                    "binary": "actual-binary",
                    "test": "selected",
                    "outcome": "passed",
                }
            ],
        )
        for declaration in (
            b'<!DOCTYPE testsuites><testsuites uuid="bounded"><testcase classname="owned" name="selected"/></testsuites>',
            b'<!DOCTYPE testsuites [<!ENTITY case_name "not-expanded">]><testsuites uuid="bounded"><testcase classname="owned" name="&case_name;"/></testsuites>',
        ):
            with (
                self.subTest(document=declaration),
                self.assertRaisesRegex(ValueError, "malformed nextest JUnit"),
            ):
                coverage.population(declaration)

    def test_unsupported_llvm_protocol_and_config_are_not_guessed(self) -> None:
        with self.assertRaisesRegex(ValueError, "protocol"):
            coverage.correspondence({"version": "future", "data": []}, {})
        with self.assertRaisesRegex(ValueError, "merged"):
            coverage.correspondence(
                {
                    "type": "llvm.coverage.json.export",
                    "version": "3.1.0",
                    "data": [{"files": []}, {"files": []}],
                },
                {},
            )
        with self.assertRaisesRegex(ValueError, "configuration"):
            coverage.junit_path(["--tool-config-file", "third-party:/config"])

    def test_affected_keeps_dependency_fallback_for_unavailable_coverage(self) -> None:
        outputs = []
        for capture in (None, Path("missing/capture")):
            output = io.StringIO()
            with (
                patch.object(
                    affected,
                    "changed_files",
                    return_value=[("M", "crates/selected/src/lib.rs")],
                ),
                patch.object(
                    affected,
                    "workspace_packages",
                    return_value=(
                        {
                            "selected": Path("crates/selected"),
                            "consumer": Path("crates/consumer"),
                        },
                        {"selected": {"consumer"}},
                    ),
                ),
                patch.object(affected, "compose", side_effect=lambda command: command),
                contextlib.redirect_stdout(output),
            ):
                args = [
                    "--json",
                    *(["--coverage", str(capture)] if capture is not None else []),
                ]
                self.assertEqual(affected.main(args), 0)
            outputs.append(json.loads(output.getvalue()))
        self.assertEqual(outputs[0]["command"], outputs[1]["command"])
        self.assertEqual(outputs[1]["selected_packages"], ["consumer", "selected"])
        self.assertEqual(outputs[1]["coverage"]["status"], "unavailable")

    def test_cli_default_uses_fresh_output_owner(self) -> None:
        with patch.object(coverage, "capture", return_value=0) as selected:
            self.assertEqual(coverage.main(["capture", "--", "-p", "actual"]), 0)
        self.assertEqual(selected.call_args.args, (None, ["-p", "actual"], False))
