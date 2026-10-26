# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Practical benchmark observations retain real failures without identity proof gates."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import (
    assessment_store,
    case_measure,
    resource_store,
    scientific_inputs,
    service_borrowers,
    surreal_server,
    test_resources,
    validation,
)
from scripts.tests import assessment_fixture
from scripts.validation_scope import Gate


class CaseMeasurementTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.enterContext(
            patch.object(
                service_borrowers, "registry", return_value=self.root / "registry"
            )
        )

    def reference(self) -> dict:
        return {
            "complete": True,
            "checks": [
                {
                    "gate": "scientific-tests",
                    "status": "passed",
                    "exit_code": 0,
                    "selected": [{"class": "science", "name": "balance"}],
                    "results": [
                        {"class": "science", "name": "balance", "status": "passed"}
                    ],
                }
            ],
        }

    @staticmethod
    def capture(destination: Path, _environment: dict) -> Path:
        destination.write_text(
            json.dumps({"schema": "scientific-inputs-v1", "files": {}, "families": {}})
        )
        return destination

    def consume(self, report: dict) -> dict | None:
        path = self.root / "checks.json"
        assessment_fixture.publish(path.parent, report)
        return case_measure.functional_reference(
            path,
            snapshot={"Cargo.lock": "current"},
            environment={"PSE_MEMORY_MAX": "80G"},
        )

    def test_optional_reference_records_differences_without_rebinding_bytes(
        self,
    ) -> None:
        self.assertIsNone(
            case_measure.functional_reference(None, snapshot={}, environment={})
        )
        report = self.reference()
        report.update(
            version=6,
            source_files={"Cargo.lock": "previous"},
            environment={"PSE_MEMORY_MAX": "4G"},
            provenance_errors=["historical provider identity unavailable"],
        )
        report["checks"][0].update(
            changed_source=["Cargo.lock"],
            changed_environment=["PSE_MEMORY_MAX"],
            mode="a-different-mode",
            native={"files": {"/missing/old/provider": "old"}},
            artifacts={"missing-old-artifact": "old"},
        )
        consumed = self.consume(report)
        assert consumed is not None
        self.assertEqual(consumed["checks"], report["checks"])
        self.assertEqual(consumed["source_differences"], ["Cargo.lock"])
        self.assertEqual(
            consumed["environment_differences"]["PSE_MEMORY_MAX"],
            {"functional": "4G", "measurement": "80G"},
        )
        self.assertEqual(
            consumed["reported_provenance_errors"], report["provenance_errors"]
        )

    def test_historical_functional_reference_is_not_promoted_to_current_evidence(
        self,
    ) -> None:
        report = self.reference()
        self.consume(report)
        path = self.root / "checks.json"
        resource_store.publish(path, {**report, "version": 5})
        before = path.read_bytes()
        with self.assertRaisesRegex(ValueError, "automatic reuse refused"):
            case_measure.functional_reference(path, snapshot={}, environment={})
        self.assertEqual(path.read_bytes(), before)

    def test_actual_test_failures_and_missing_terminal_results_are_refused(
        self,
    ) -> None:
        for mutation in (
            "unfinished",
            "failed",
            "exit",
            "skipped",
            "missing",
            "duplicate",
            "unexpected",
            "report-error",
            "setup-only",
        ):
            report = self.reference()
            check = report["checks"][0]
            if mutation == "unfinished":
                report["complete"] = False
            elif mutation == "failed":
                check["results"][0]["status"] = "failed"
            elif mutation == "exit":
                check["exit_code"] = 7
            elif mutation == "skipped":
                check["results"][0]["status"] = "skipped"
            elif mutation == "missing":
                check["results"] = []
            elif mutation == "duplicate":
                check["results"].append(check["results"][0])
            elif mutation == "unexpected":
                check["results"][0]["name"] = "different"
            elif mutation == "report-error":
                check["report_errors"] = ["malformed terminal report"]
            else:
                check.pop("results")
                check.pop("selected")
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.consume(report)

    def test_actual_assessment_drift_diagnostic_keeps_passing_test_observation(
        self,
    ) -> None:
        output = self.root / "assessment"
        output.mkdir()
        (output / "scientific-tests.log").write_text("completed tests\n")
        check = self.reference()["checks"][0]
        with (
            patch.object(test_resources, "register_report", return_value={}),
            patch.object(test_resources, "finish_report"),
            patch.object(test_resources, "reclaim_reports", return_value=[]),
            patch.object(
                validation,
                "provenance",
                return_value=({"Cargo.lock": "previous"}, self.root / "target", []),
            ),
            patch.object(validation, "sources", return_value={"Cargo.lock": "current"}),
            patch.object(validation, "relevant_environment", return_value={}),
            patch.object(validation, "execute", return_value=check),
        ):
            self.assertEqual(
                validation.run_gates(
                    self.root,
                    output,
                    [Gate("scientific-tests", input_scope="rust-product")],
                ),
                1,
            )
        emitted = assessment_store.export(output)
        self.assertTrue(emitted["complete"])
        self.assertEqual(emitted["checks"][0]["status"], "failed")
        self.assertEqual(
            emitted["checks"][0]["report_errors"],
            ["relevant inputs changed during assessment"],
        )
        consumed = self.consume(emitted)
        assert consumed is not None
        self.assertEqual(consumed["observed_test_status"], "passed")
        self.assertEqual(consumed["checks"], emitted["checks"])

        for mutation in (
            "test-failure",
            "report-error",
            "no-drift",
            "command-failure",
            "interrupted",
            "missing-terminal",
            "unexplained-status",
        ):
            failing = json.loads(json.dumps(emitted))
            result = failing["checks"][0]
            if mutation == "test-failure":
                result["results"][0]["status"] = "failure"
            elif mutation == "report-error":
                result["report_errors"].append("malformed terminal report")
            elif mutation == "no-drift":
                result["changed_source"] = []
            elif mutation == "command-failure":
                result["exit_code"] = 7
            elif mutation == "interrupted":
                result["status"] = "interrupted"
            elif mutation == "missing-terminal":
                result["results"] = []
            else:
                result["report_errors"] = []
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.consume(failing)

    def run_measurement(
        self, *, command_exit: int = 0, rows: str | None = None
    ) -> Path:
        config = self.root / ".config"
        config.mkdir()
        workload = {"id": "cold-small-1", "functional_scopes": ["process"]}
        (config / "process-cases.json").write_text(
            json.dumps({"cargo_profile": "dev", "workloads": [workload]})
        )
        (config / "preparation-cases.json").write_text('{"workloads": []}')
        binary = self.root / "benchmark"
        sample_rows = rows if rows is not None else "100,ns,2\n300,ns,3\n"
        csv_text = "sample_measured_value,unit,iteration_count\n" + sample_rows
        program = (
            f"raise SystemExit({command_exit})\n"
            if command_exit
            else "import os, pathlib\n"
            "output = pathlib.Path(os.environ['PSE_PROCESS_COST_OUTPUT'])\n"
            "capsule = pathlib.Path(os.environ['PSE_SCIENTIFIC_INPUTS'])\n"
            "assert capsule.is_file()\n"
            "(output / 'scientific-input-role').write_text(str(capsule))\n"
            "case = os.environ['PSE_PROCESS_COST_CASE']\n"
            "(output / (case + '-memory.json')).write_text('{\"pool_peak_bytes\": 64}')\n"
            "csv = output / 'criterion/process' / case / 'new/raw.csv'\n"
            "csv.parent.mkdir(parents=True)\n"
            f"csv.write_text({csv_text!r})\n"
        )
        binary.write_text(f"#!{sys.executable}\n" + program)
        binary.chmod(0o755)
        artifact = json.dumps(
            {
                "reason": "compiler-artifact",
                "target": {"name": "native_process"},
                "executable": str(binary),
            }
        )
        output = self.root / "measurement"
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                scientific_inputs, "capture", side_effect=self.capture
            ) as captured,
            patch.object(case_measure, "ROOT", self.root),
            patch.object(
                sys, "argv", ["case_measure", str(output), "--case", workload["id"]]
            ),
            patch.object(
                validation,
                "fresh_output",
                side_effect=lambda *_: (output.mkdir(), output)[1],
            ),
            patch.object(
                validation,
                "sources",
                side_effect=[{"Cargo.lock": "before"}, {"Cargo.lock": "after"}],
            ),
            patch.object(
                case_measure,
                "measurement_environment",
                side_effect=[{"PSE_MEMORY_MAX": "80G"}, {"PSE_MEMORY_MAX": "160G"}],
            ),
            patch.object(
                case_measure.surreal_server,
                "measurement_context",
                return_value={"timing_isolation": "uncontrolled"},
            ),
            patch.object(
                case_measure,
                "build_benchmark",
                return_value=subprocess.CompletedProcess([], 0, stdout=artifact),
            ) as build,
        ):
            self.assertEqual(case_measure.main(), 0)
            self.assertIn(
                "native-process,pse-relations/force-validate", build.call_args.args[0]
            )
            captured.assert_called_once()
        return output

    def test_real_case_runs_without_reference_and_records_changing_conditions(
        self,
    ) -> None:
        output = self.run_measurement()
        report = json.loads((output / "case-measure.json").read_text())
        self.assertTrue(report["measured"])
        self.assertIsNone(report["functional"])
        self.assertEqual(report["selected_cases"], ["cold-small-1"])
        self.assertEqual(
            (output / "process-cost/cold-small-1/scientific-input-role").read_text(),
            str(output / "scientific-inputs.json"),
        )
        self.assertEqual(report["cases"][0]["criterion_statistics"]["sample_count"], 2)
        self.assertEqual(report["source_changes_during_measurement"], ["Cargo.lock"])
        self.assertEqual(
            report["environment_changes_during_measurement"], ["PSE_MEMORY_MAX"]
        )

    def test_real_case_command_failure_never_writes_successful_report(self) -> None:
        with self.assertRaises(subprocess.CalledProcessError) as failed:
            self.run_measurement(command_exit=7)
        self.assertEqual(failed.exception.returncode, 7)
        self.assertFalse((self.root / "measurement/case-measure.json").exists())

    def test_invalid_case_samples_never_write_successful_report(self) -> None:
        with self.assertRaisesRegex(ValueError, "at least two"):
            self.run_measurement(rows="100,ns,2\n")
        self.assertFalse((self.root / "measurement/case-measure.json").exists())

    def test_smoke_prepares_once_before_placement_and_binds_every_process(self) -> None:
        process = {"workloads": [{"id": "first"}, {"id": "second"}]}
        artifact = json.dumps(
            {
                "reason": "compiler-artifact",
                "target": {"name": "native_process"},
                "executable": "/benchmark",
            }
        )
        prepared = self.root / "scientific-inputs.json"

        def placement(binary: str, mode: str, _workload: dict) -> list[str]:
            self.assertTrue(prepared.is_file())
            return [binary, mode]

        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                scientific_inputs, "capture", side_effect=self.capture
            ) as captured,
            patch(
                "scripts.arrow_validation.compose", side_effect=lambda command: command
            ),
            patch.object(
                case_measure,
                "build_benchmark",
                return_value=subprocess.CompletedProcess([], 0, stdout=artifact),
            ),
            patch.object(case_measure, "process_command", side_effect=placement),
            patch.object(
                case_measure.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as run,
        ):
            case_measure.smoke_campaign(
                self.root, "dev", process, {"workloads": []}, {"first", "second"}
            )
            self.assertNotIn(scientific_inputs.ROLE, os.environ)
        captured.assert_called_once()
        self.assertEqual(run.call_count, 2)
        for call in run.call_args_list:
            self.assertEqual(call.kwargs["env"][scientific_inputs.ROLE], str(prepared))

    def test_smoke_refuses_invalid_supplied_capsule_before_build_or_placement(
        self,
    ) -> None:
        supplied = self.root / "supplied.json"
        supplied.write_text('{"schema":"other"}')
        with (
            patch.dict(os.environ, {scientific_inputs.ROLE: str(supplied)}, clear=True),
            patch.object(scientific_inputs, "capture") as captured,
            patch.object(case_measure, "build_benchmark") as build,
            patch.object(case_measure, "process_command") as placed,
            self.assertRaises(ValueError),
        ):
            case_measure.smoke_campaign(
                self.root,
                "dev",
                {"workloads": [{"id": "selected"}]},
                {"workloads": []},
                {"selected"},
            )
        captured.assert_not_called()
        build.assert_not_called()
        placed.assert_not_called()

    def test_invocations_are_separate_from_measured_samples_and_keep_unknown_work(
        self,
    ) -> None:
        records = [
            {"seconds": 3, "work": {"total": None}},
            {"seconds": 5, "work": {"total": 4}},
        ]
        path = self.root / "case-observations.jsonl"
        path.write_text("".join(json.dumps(record) + "\n" for record in records))
        population = case_measure.invocation_population(
            self.root, {"iterations": 2, "observations_file": path.name}
        )
        self.assertEqual(population["record_count"], 2)
        self.assertEqual(population["criterion_membership"]["status"], "unavailable")
        self.assertEqual(
            json.loads(path.read_text().splitlines()[0])["work"]["total"], None
        )
        with self.assertRaisesRegex(ValueError, "count"):
            case_measure.invocation_population(
                self.root, {"iterations": 1, "observations_file": path.name}
            )

    def test_invocation_reader_rejects_escaped_artifacts_and_malformed_records(
        self,
    ) -> None:
        with self.assertRaisesRegex(ValueError, "escapes"):
            case_measure.invocation_population(
                self.root, {"observations_file": "../outside.jsonl"}
            )
        with self.assertRaisesRegex(ValueError, "object records"):
            case_measure.invocation_population(self.root, {"records": [1]})
        self.assertEqual(
            case_measure.invocation_population(self.root, {})["status"], "unavailable"
        )

    def test_managed_command_registers_exact_profile_and_worker(self) -> None:
        receiver = {
            "supervisor_executable": "/python",
            "supervisor_script": "/supervisor.py",
            "worker_executable": "/selected/worker",
        }
        config = {"resources": surreal_server.reference_resources()}
        with (
            patch.dict(case_measure.os.environ, {"PSE_SURREAL_STATE": str(self.root)}),
            patch.object(surreal_server, "config_for", return_value=config),
            patch.object(surreal_server, "checked_primary", return_value=receiver),
        ):
            command = case_measure.process_command(
                "/benchmark", "--test", {"adapter": "managed-durable"}
            )
            self.assertEqual(
                command[command.index("--execution-profile") + 1], "reference"
            )
            self.assertEqual(
                case_measure.managed_fixture_environment(
                    {"adapter": "managed-durable"}
                ),
                {"PSE_WORKER_BINARY": "/selected/worker"},
            )
            self.assertEqual(
                case_measure.managed_fixture_environment({"adapter": "ephemeral"}), {}
            )

    def test_managed_execution_keeps_reference_allocation_requirement(self) -> None:
        with (
            patch.dict(case_measure.os.environ, {"PSE_SURREAL_STATE": str(self.root)}),
            patch.object(surreal_server, "config_for", return_value={"resources": {}}),
            patch.object(surreal_server, "checked_primary") as receiver,
            self.assertRaisesRegex(ValueError, "reference allocation"),
        ):
            case_measure.process_command(
                "/benchmark", "--bench", {"adapter": "managed-durable"}
            )
        receiver.assert_not_called()


if __name__ == "__main__":
    unittest.main()
