# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Catalog routing and publication preserve original outcomes and observation values."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import csv
import json
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, NoReturn, TextIO
from unittest.mock import patch

from scripts import pipeline_review

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator, Mapping, Sequence

_DICT_WRITER = csv.DictWriter
_WRITE_TEXT = Path.write_text


class _ObservedWriter:
    """Observe actual streamed rows while the standard writer owns CSV encoding."""

    def __init__(
        self,
        destination: TextIO,
        fieldnames: Sequence[str],
        observe: Callable[[Mapping[str, object]], None],
    ) -> None:
        self._writer = _DICT_WRITER(destination, fieldnames)
        self._observe = observe

    def writeheader(self) -> int:
        return self._writer.writeheader()

    def writerow(self, row: Mapping[str, object]) -> int:
        self._observe(row)
        return self._writer.writerow(row)


class PipelineReviewTests(unittest.TestCase):
    def test_catalog_preserves_explicit_existing_selection_and_target(self) -> None:
        selected = pipeline_review.catalog(["k4-scalar-preparation", "cold-small-1"])
        self.assertEqual(
            [target for target, _ in selected],
            ["modeling_preparation", "native_process"],
        )
        self.assertEqual(
            [workload["id"] for _, workload in selected],
            ["k4-scalar-preparation", "cold-small-1"],
        )
        for invalid in ([], ["unknown-case"], ["cold-small-1", "cold-small-1"]):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                pipeline_review.catalog(invalid)

    def test_prebuilt_map_and_literal_path_overrides(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            binary = directory / "literal binary"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(0o700)
            source = directory / "binaries.json"
            source.write_text(json.dumps({"binaries": {"native_process": str(binary)}}))
            binaries = pipeline_review.binaries_from(
                source, [f"modeling_preparation={binary}"]
            )
            self.assertEqual(
                binaries, {"native_process": binary, "modeling_preparation": binary}
            )
            with self.assertRaises(ValueError):
                pipeline_review.binaries_from(None, ["native_process=missing-binary"])

    def test_execute_routes_real_spec_and_keeps_scientific_failure(self) -> None:
        workload = pipeline_review.catalog(["cold-small-1"])[0][1]
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "case"
            with (
                patch.object(
                    pipeline_review.case_measure,
                    "managed_fixture_environment",
                    return_value={"ACTUAL_MANAGED": "selected"},
                ),
                patch.object(
                    pipeline_review.case_measure,
                    "process_command",
                    return_value=["literal-binary", "--test"],
                ) as command,
                patch.object(
                    pipeline_review.case_audit,
                    "run_case",
                    return_value={
                        "status": "failed",
                        "exit_code": 7,
                        "fixture_cleanup_failures": 1,
                    },
                ) as audit,
            ):
                record = pipeline_review.execute_case(
                    "/bin/true",
                    workload,
                    directory,
                    recording_mode="ordinary",
                    env={"CALLER": "value"},
                    label="paired ordinary",
                )
            self.assertEqual(command.call_args.args[2], workload)
            self.assertEqual(audit.call_args.args[0], ["literal-binary", "--test"])
            environment = audit.call_args.kwargs["env"]
            self.assertEqual(json.loads(environment["PSE_PROCESS_COST_SPEC"]), workload)
            self.assertEqual(environment["PSE_PROCESS_COST_OUTPUT"], str(directory))
            self.assertEqual(environment["PSE_PREPARATION_OUTPUT"], str(directory))
            self.assertEqual(environment["ACTUAL_MANAGED"], "selected")
            self.assertEqual(audit.call_args.kwargs["recording_mode"], "ordinary")
            self.assertEqual(record["result"]["exit_code"], 7)
            self.assertEqual(
                json.loads((directory / "case-run.json").read_text()), record
            )

    def test_publish_aggregates_exact_original_values_without_summing_or_overwriting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            case = root / "runs/case-one"
            case.mkdir(parents=True)
            (case / "case-run.json").write_text(
                json.dumps(
                    {
                        "case": "cold-small-1",
                        "result": {
                            "status": "completed",
                            "fixture_cleanup_failures": 1,
                        },
                    }
                )
            )
            (case / "cold-small-1-memory.json").write_text(
                json.dumps({"pool_peak_bytes": 12, "native_work": None})
            )
            observation = case / "cold-small-1-observations.jsonl"
            original = b'{"seconds":0.001,"work":{"iterations":7}}\n{"seconds":0.002,"work":null}\n'
            observation.write_bytes(original)
            raw = case / "criterion/samples.csv"
            raw.parent.mkdir()
            raw.write_bytes(b"original criterion sample bytes\n")
            supplement = root / "supplement.json"
            supplement.write_text(
                json.dumps({"actual": {"factorizations": 3}, "unavailable": None})
            )
            output = root / "publication"
            published, index = pipeline_review.publish(
                output,
                runs=[root / "runs"],
                supplements=[supplement],
                label="retained <review>",
            )
            self.assertEqual(index["cases"][0]["status"], "failed")
            self.assertEqual(index["cases"][0]["result"]["status"], "completed")
            self.assertEqual(index["errors"], [])
            self.assertEqual(observation.read_bytes(), original)
            self.assertEqual(raw.read_bytes(), b"original criterion sample bytes\n")
            self.assertEqual(published.parent, output)
            self.assertTrue(published.name.startswith("review-"))
            with (published / "observations.csv").open() as source:
                rows = list(csv.DictReader(source))
            timings = [row for row in rows if row["pointer"] == "/seconds"]
            self.assertEqual([row["value_json"] for row in timings], ["0.001", "0.002"])
            self.assertEqual([row["record_line"] for row in timings], ["1", "2"])
            self.assertTrue(
                any(
                    row["pointer"] == "/native_work" and row["value_json"] == "null"
                    for row in rows
                )
            )
            self.assertEqual(index["scalar_rows"], 8)
            self.assertEqual(
                [
                    (
                        row["source"],
                        row["record_line"],
                        row["pointer"],
                        row["value_json"],
                    )
                    for row in rows
                ],
                [
                    (
                        "../../runs/case-one/cold-small-1-memory.json",
                        "",
                        "/pool_peak_bytes",
                        "12",
                    ),
                    (
                        "../../runs/case-one/cold-small-1-memory.json",
                        "",
                        "/native_work",
                        "null",
                    ),
                    (
                        "../../runs/case-one/cold-small-1-observations.jsonl",
                        "1",
                        "/seconds",
                        "0.001",
                    ),
                    (
                        "../../runs/case-one/cold-small-1-observations.jsonl",
                        "1",
                        "/work/iterations",
                        "7",
                    ),
                    (
                        "../../runs/case-one/cold-small-1-observations.jsonl",
                        "2",
                        "/seconds",
                        "0.002",
                    ),
                    (
                        "../../runs/case-one/cold-small-1-observations.jsonl",
                        "2",
                        "/work",
                        "null",
                    ),
                    ("../../supplement.json", "", "/actual/factorizations", "3"),
                    ("../../supplement.json", "", "/unavailable", "null"),
                ],
            )
            for row in rows:
                self.assertTrue((published / row["source"]).is_file())
            self.assertIn("retained projection", index["scope"])
            self.assertIn("criterion/samples.csv", (published / "index.md").read_text())
            self.assertIn(
                "retained &lt;review&gt;", (published / "index.html").read_text()
            )
            self.assertEqual(json.loads((published / "index.json").read_text()), index)

    def test_publish_streams_before_decoding_next_record_with_bounded_row_storage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "samples.jsonl"
            source.write_text("original samples retained\n")
            written = 0

            def observe(_row: Mapping[str, object]) -> None:
                nonlocal written
                written += 1

            def make_writer(
                destination: TextIO, fieldnames: Sequence[str]
            ) -> _ObservedWriter:
                return _ObservedWriter(destination, fieldnames, observe)

            def records(_path: Path) -> Iterator[tuple[int, object]]:
                for line in range(1, 20_001):
                    self.assertEqual(written, line - 1)
                    yield line, {"value": line}

            with (
                patch.object(
                    pipeline_review.csv, "DictWriter", side_effect=make_writer
                ),
                patch.object(pipeline_review, "_read_json_lines", side_effect=records),
            ):
                published, index = pipeline_review.publish(
                    root / "output", runs=[], supplements=[source]
                )
            self.assertEqual(written, 20_000)
            self.assertEqual(index["scalar_rows"], 20_000)
            with (published / "observations.csv").open() as output:
                self.assertEqual(sum(1 for _ in csv.DictReader(output)), 20_000)
            self.assertEqual(source.read_text(), "original samples retained\n")

    def test_publish_preserves_distinct_samples_pointers_nulls_and_truthful_input_errors(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "samples.jsonl"
            original = (
                b'{"a/b~c":[null,true,3]}\n\n{"a/b~c":[null,true,3]}\n{invalid}\n'
            )
            source.write_bytes(original)
            missing = root / "missing.json"
            published, index = pipeline_review.publish(
                root / "output", runs=[], supplements=[source, missing]
            )
            with (published / "observations.csv").open() as output:
                rows = list(csv.DictReader(output))
            self.assertEqual(
                [
                    (row["record_line"], row["pointer"], row["value_json"])
                    for row in rows
                ],
                [
                    (line, pointer, value)
                    for line in ("1", "3")
                    for pointer, value in (
                        ("/a~1b~0c/0", "null"),
                        ("/a~1b~0c/1", "true"),
                        ("/a~1b~0c/2", "3"),
                    )
                ],
            )
            self.assertEqual(index["scalar_rows"], 6)
            self.assertEqual(len(index["errors"]), 2)
            self.assertIn("samples.jsonl", index["errors"][0])
            self.assertIn("line 4:", index["errors"][0])
            self.assertIn("supplement unavailable", index["errors"][1])
            self.assertEqual(source.read_bytes(), original)
            self.assertIn("not independent observations", index["scope"])
            self.assertIn("samples.jsonl", (published / "index.md").read_text())

    def test_publication_write_or_interrupted_finalization_preserves_previous_generation(
        self,
    ) -> None:
        for failure in (
            "csv",
            "index.json",
            "index.md",
            "index.html",
            "rename",
            "interrupt",
        ):
            with self.subTest(failure=failure):
                self._assert_publication_failure_preserves_generation(failure)

    def _assert_publication_failure_preserves_generation(self, failure: str) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "output"
            output.mkdir()
            source = output / "original.json"
            source.write_text('{"exact":7}')
            previous, _ = pipeline_review.publish(output, runs=[], supplements=[source])
            original_files = {
                path.name: path.read_bytes() for path in previous.iterdir()
            }
            destinations: list[TextIO] = []

            def fail_write(
                path: Path,
                data: str,
                encoding: str | None = None,
                errors: str | None = None,
                newline: str | None = None,
            ) -> int:
                if path.name == failure:
                    raise OSError("publication write failed")
                return _WRITE_TEXT(path, data, encoding, errors, newline)

            def observe(_value: Mapping[str, object]) -> None:
                if failure == "csv":
                    raise OSError("CSV write failed")

            def make_writer(
                destination: TextIO, fieldnames: Sequence[str]
            ) -> _ObservedWriter:
                destinations.append(destination)
                return _ObservedWriter(destination, fieldnames, observe)

            def finalize(path: Path, _destination: Path) -> NoReturn:
                self.assertIn(
                    failure, ("rename", "interrupt"), "write failure was swallowed"
                )
                self.assertTrue(all(destination.closed for destination in destinations))
                self.assertEqual(
                    set(original_files), {member.name for member in path.iterdir()}
                )
                self.assertEqual(
                    json.loads((path / "index.json").read_text())["scalar_rows"], 1
                )
                if failure == "interrupt":
                    raise KeyboardInterrupt
                raise OSError("publication rename failed")

            with (
                patch.object(Path, "write_text", new=fail_write),
                patch.object(
                    pipeline_review.csv, "DictWriter", side_effect=make_writer
                ),
                patch.object(Path, "rename", new=finalize),
                self.assertRaises(
                    KeyboardInterrupt if failure == "interrupt" else OSError
                ),
            ):
                pipeline_review.publish(output, runs=[], supplements=[source])
            self.assertEqual(
                {path.name: path.read_bytes() for path in previous.iterdir()},
                original_files,
            )
            self.assertEqual(set(output.iterdir()), {previous, source})
            self.assertEqual(source.read_text(), '{"exact":7}')

    def test_publish_only_reports_fresh_directory_and_repeat_preserves_prior_output(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "raw.json"
            source.write_text('{"sample":1}')
            output = root / "campaign"
            with patch("builtins.print") as announced:
                for _ in range(2):
                    status = pipeline_review.main(
                        [str(output), "--publish-only", "--supplement", str(source)]
                    )
                    self.assertEqual(status, 0)
            paths = [call.args[0] for call in announced.call_args_list]
            self.assertEqual(len(set(paths)), 2)
            for report in paths:
                self.assertTrue(report.is_file())
                self.assertEqual(report.parent.parent, output)
                self.assertIn("raw.json", report.read_text())
            self.assertEqual(len(list(output.iterdir())), 2)

    def test_campaign_continues_after_one_case_execution_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "campaign"
            with (
                patch.object(
                    pipeline_review.case_audit,
                    "run_case",
                    side_effect=[
                        RuntimeError("original failure"),
                        {"status": "completed"},
                    ],
                ) as audit,
                patch.object(
                    pipeline_review.case_measure,
                    "managed_fixture_environment",
                    return_value={},
                ),
                patch.object(
                    pipeline_review.case_measure,
                    "process_command",
                    return_value=["/bin/true", "--test"],
                ),
            ):
                status = pipeline_review.main(
                    [
                        str(output),
                        "--binary",
                        "native_process=/bin/true",
                        "--case",
                        "cold-small-1",
                        "--case",
                        "warm-small-1",
                        "--recording-mode",
                        "ordinary",
                    ]
                )
            self.assertEqual(status, 1)
            self.assertEqual(audit.call_count, 2)
            publications = list(output.glob("review-*"))
            self.assertEqual(len(publications), 1)
            index = json.loads((publications[0] / "index.json").read_text())
            self.assertEqual(len(index["cases"]), 2)
            first = next(
                case for case in index["cases"] if case["case"] == "cold-small-1"
            )
            self.assertIn("original failure", first["execution_error"])
            self.assertEqual(first["status"], "failed")


if __name__ == "__main__":
    unittest.main()
