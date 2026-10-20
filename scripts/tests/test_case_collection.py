# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Script-only controls for selected review collection and launch ownership."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType, SimpleNamespace
from unittest.mock import Mock, patch

from scripts import case_audit, case_measure


class LaunchDocuments(ModuleType):
    """Typed test module for the deliberately lazy launch-request import."""

    LaunchCaptureRequest = SimpleNamespace


class CollectionSelectionTests(unittest.TestCase):
    def test_bounded_analysis_retains_full_facts_and_labels_child_selection(
        self,
    ) -> None:
        from pse.contracts.documents import (  # noqa: PLC0415
            AnalysisRequest,
            OperationId,
        )

        root = OperationId(process_instance=(1, 2), sequence=1)
        child = OperationId(process_instance=(1, 2), sequence=2)
        evidence = Mock()
        evidence.invocations = ()
        evidence.checkpoints = ()
        evidence.operations = (Mock(id=child, children=()),)
        diagnostics = Mock()
        diagnostics.InspectionError = RuntimeError
        diagnostics.operation_text.side_effect = lambda operation: str(
            operation.sequence
        )
        actual_analysis = Mock()
        diagnostics.analyze_evidence.side_effect = [
            RuntimeError("analysis operation selection exceeds 64 exact subjects"),
            actual_analysis,
        ]
        observer = Mock(diagnostics=diagnostics)
        result: dict = {}
        request = AnalysisRequest(operation=root)
        analysis = case_audit.analyze_case_evidence(observer, evidence, request, result)
        self.assertIs(analysis, actual_analysis)
        self.assertEqual(result["analysis_scope"]["selected_operation"], "2")
        self.assertIn("full raw facts retained", result["limits"][0])
        calls = diagnostics.analyze_evidence.call_args_list
        self.assertIs(calls[0].args[0], evidence)
        self.assertIs(calls[1].args[0], evidence)
        self.assertEqual(calls[1].args[1].operation, child)
        diagnostics.analyze_evidence.reset_mock()
        diagnostics.analyze_evidence.side_effect = RuntimeError(
            "conflicting original facts"
        )
        with self.assertRaisesRegex(RuntimeError, "conflicting original"):
            case_audit.analyze_case_evidence(observer, evidence, request, {})
        diagnostics.analyze_evidence.assert_called_once()

    def test_managed_reference_cannot_collect_other_owner_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            audit = Path(temporary)
            child = audit / "detail/managed/study/run/attempt/records.jsonl"
            child.parent.mkdir(parents=True)
            child.write_text("original\n")
            self.assertEqual(case_audit.managed_records_path(audit, str(child)), child)
            outside = audit / "other-owner.jsonl"
            outside.write_text("other owner\n")
            with self.assertRaisesRegex(ValueError, "leaves selected"):
                case_audit.managed_records_path(audit, str(outside))
            link = child.parent / "escaped.jsonl"
            link.symlink_to(outside)
            with self.assertRaisesRegex(ValueError, "leaves selected"):
                case_audit.managed_records_path(audit, str(link))

    def test_preparation_selection_excludes_unacknowledged_partial_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            audit = Path(temporary)
            group = audit / "preparation-originals/input-000"
            group.mkdir(parents=True)
            acknowledged = group / "source-000.bin"
            acknowledged.write_bytes(b"PAR1\x00\xff")
            (group / "source-001.bin").write_bytes(b"partial")
            inventory = group / "inputs.json"
            inventory.write_text(
                json.dumps(
                    {
                        "members": [
                            {
                                "path": acknowledged.name,
                                "bytes": acknowledged.stat().st_size,
                            },
                        ]
                    }
                )
            )
            paths, errors = case_audit.preparation_input_members(audit)
            self.assertEqual(paths, [inventory, acknowledged])
            self.assertEqual(errors, [])
            acknowledged.write_bytes(b"truncated")
            paths, errors = case_audit.preparation_input_members(audit)
            self.assertEqual(paths, [inventory])
            self.assertIn("length differs", errors[0])

    def test_preparation_selection_refuses_inventory_escape(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            audit = Path(temporary)
            group = audit / "preparation-originals/input-000"
            group.mkdir(parents=True)
            inventory = group / "inputs.json"
            inventory.write_text(
                json.dumps({"members": [{"path": "../outside", "bytes": 0}]})
            )
            paths, errors = case_audit.preparation_input_members(audit)
            self.assertEqual(paths, [inventory])
            self.assertIn("not contained", errors[0])

    def test_cli_forwards_selected_collection_profile_and_compiler(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "out"
            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "case_measure",
                        str(output),
                        "--smoke",
                        "--audit-diagnostics",
                        "--case",
                        "cold-small-1",
                        "--review-artifacts",
                        "--capture-from-start",
                        "--cargo-profile",
                        "profiling",
                        "--compiled-evaluator",
                    ],
                ),
                patch.object(
                    case_measure.validation, "fresh_output", return_value=output
                ),
                patch.object(case_measure, "smoke_campaign") as campaign,
            ):
                self.assertEqual(case_measure.main(), 0)
            self.assertEqual(campaign.call_args.args[1], "profiling")
            self.assertEqual(campaign.call_args.args[4], {"cold-small-1"})
            self.assertTrue(campaign.call_args.kwargs["review_artifacts"])
            self.assertTrue(campaign.call_args.kwargs["capture_from_start"])
            self.assertTrue(campaign.call_args.kwargs["compiled_evaluator"])

    def test_optional_collection_refuses_missing_audit_before_effects(self) -> None:
        for flag in (
            "--review-artifacts",
            "--capture-from-start",
            "--compiled-evaluator",
        ):
            with (
                self.subTest(flag=flag),
                patch.object(sys, "argv", ["case_measure", "out", flag]),
                patch.object(case_measure.validation, "fresh_output") as fresh,
                self.assertRaises(SystemExit) as error,
            ):
                case_measure.main()
            self.assertEqual(error.exception.code, 2)
            fresh.assert_not_called()

    def test_build_selects_compiled_feature_only_when_requested(self) -> None:
        from scripts import arrow_validation  # noqa: PLC0415

        process = {"workloads": [{"id": "selected"}]}
        for compiled in (False, True):
            with tempfile.TemporaryDirectory() as temporary:
                output = Path(temporary)
                build = SimpleNamespace(
                    stdout=json.dumps(
                        {
                            "reason": "compiler-artifact",
                            "target": {"name": "native_process"},
                            "executable": "/literal/benchmark",
                        }
                    )
                )
                with (
                    self.subTest(compiled=compiled),
                    patch.object(
                        arrow_validation, "compose", side_effect=lambda command: command
                    ),
                    patch.object(
                        case_measure, "build_benchmark", return_value=build
                    ) as builder,
                    patch.object(case_measure.subprocess, "run") as run,
                ):
                    case_measure.smoke_campaign(
                        output,
                        "profiling",
                        process,
                        {"workloads": []},
                        {"selected"},
                        compiled_evaluator=compiled,
                    )
                command = builder.call_args.args[0]
                self.assertEqual(command[command.index("--profile") + 1], "profiling")
                features = command[command.index("--features") + 1].split(",")
                self.assertIn("pse-relations/force-validate", features)
                self.assertEqual("pse-runtime/compiled-evaluator" in features, compiled)
                self.assertEqual(
                    run.call_args.args[0], ["/literal/benchmark", "--test"]
                )

    def test_ordinary_mode_omits_source_request_and_keeps_live_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            program = (
                "import os,pathlib,json; p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR']); "
                "assert 'PSE_DIAGNOSTIC_RECORD_REQUEST' not in os.environ; "
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'completed','cleanup':'confirmed'}))"
            )
            observer = Mock()
            observer.finish.return_value = {"inspection_windows": 1}
            with (
                patch.object(case_audit, "Observer", return_value=observer),
                patch.object(case_audit, "prepare_source_recording") as prepare,
            ):
                result = case_audit.run_case(
                    [sys.executable, "-c", program],
                    cwd=directory,
                    env={
                        **os.environ,
                        "PSE_DIAGNOSTIC_RECORD_REQUEST": "legacy-or-foreign.json",
                    },
                    directory=directory,
                    recording_mode="ordinary",
                )
            prepare.assert_not_called()
            observer.finish.assert_called_once()
            self.assertEqual(result["status"], "completed")
            self.assertEqual(result["recording_mode"], "ordinary")
            self.assertFalse(
                (directory / "diagnostics/recording-request.json").exists()
            )

    def test_cli_forwards_ordinary_mode_without_review(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "out"
            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "case_measure",
                        str(output),
                        "--smoke",
                        "--audit-diagnostics",
                        "--case",
                        "cold-small-1",
                        "--recording-mode",
                        "ordinary",
                    ],
                ),
                patch.object(
                    case_measure.validation, "fresh_output", return_value=output
                ),
                patch.object(case_measure, "smoke_campaign") as campaign,
            ):
                self.assertEqual(case_measure.main(), 0)
            self.assertEqual(campaign.call_args.kwargs["recording_mode"], "ordinary")
            self.assertFalse(campaign.call_args.kwargs["review_artifacts"])


class CaptureLifetimeTests(unittest.TestCase):
    def run_capture(
        self, *, exit_code: int, deadline: bool
    ) -> tuple[dict, Mock, Mock, Path]:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        directory = Path(temporary.name)
        documents = LaunchDocuments("pse.contracts.documents")
        child_owner = Mock()
        capture = Mock()
        capture.snapshot.return_value = {
            "status": "Failed",
            "errors": ["perf unavailable"],
        }
        capture.stop.return_value = capture.snapshot.return_value
        processes = []
        requests = []
        observer = Mock()
        observer.directory = directory / "diagnostics"
        observer.captures = []

        def launch(request: SimpleNamespace) -> tuple[Mock, Mock]:
            requests.append(request)
            self.assertEqual(os.environ["PSE_CASE_CAPTURE_FROM_START"], "1")
            process = subprocess.Popen(request.argv, cwd=request.cwd)
            processes.append(process)
            child_owner.try_wait.side_effect = process.poll
            child_owner.wait.side_effect = process.wait
            child_owner.close.side_effect = lambda: self.assertIsNotNone(process.poll())
            # Optional sampling has already failed; science remains running.
            self.assertEqual(capture.snapshot()["status"], "Failed")
            self.assertIsNone(process.poll())
            return child_owner, capture

        observer.diagnostics.launch_capture.side_effect = launch

        def finish() -> dict[str, list[str]]:
            self.assertIsNotNone(processes[0].poll())
            for _name, session in observer.captures:
                session.stop()
            return {"diagnostic_failures": ["perf unavailable"]}

        observer.finish.side_effect = finish
        program = (
            "import os,pathlib,time,json,sys; "
            "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR']); "
            "print('science stdout',flush=True); print('science stderr',file=sys.stderr,flush=True)\n"
            + (
                "while not (p/'stop.request').exists(): time.sleep(.01)\n"
                if deadline
                else "time.sleep(.05)\n"
            )
            + "(p/'audit-outcome.json').write_text(json.dumps({'status':'"
            + ("interrupted" if deadline else "completed")
            + "','cleanup':'confirmed'}))\n"
            + f"sys.exit({exit_code})\n"
        )
        before = os.environ.get("PSE_CASE_CAPTURE_FROM_START")
        with (
            patch.dict(sys.modules, {"pse.contracts.documents": documents}),
            patch.object(case_audit, "Observer", return_value=observer),
        ):
            result = case_audit.run_case(
                [sys.executable, "-c", program],
                cwd=directory,
                env=dict(os.environ),
                directory=directory,
                capture_from_start=True,
                deadline_seconds=0.01 if deadline else None,
            )
        self.assertEqual(os.environ.get("PSE_CASE_CAPTURE_FROM_START"), before)
        self.assertEqual(len(requests), 1)
        self.assertTrue(observer.capture_from_start)
        self.assertEqual(observer.captures, [("cpu", capture)])
        capture.stop.assert_called_once()
        child_owner.cancel.assert_not_called()
        child_owner.wait.assert_called_once()
        child_owner.close.assert_called_once()
        log = (directory / "process.log").read_text()
        self.assertIn("science stdout", log)
        self.assertIn("science stderr", log)
        return result, child_owner, capture, directory

    def test_sampler_failure_preserves_success_and_command_output(self) -> None:
        result, _command, _capture, _directory = self.run_capture(
            exit_code=0, deadline=False
        )
        self.assertEqual(result["status"], "completed")
        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(result["diagnostic_failures"], ["perf unavailable"])

    def test_command_failure_is_not_sampler_result(self) -> None:
        result, _command, _capture, _directory = self.run_capture(
            exit_code=7, deadline=False
        )
        self.assertEqual(result["status"], "failed")
        self.assertEqual(result["exit_code"], 7)

    def test_deadline_requests_cooperative_stop_and_drains_before_sampler_join(
        self,
    ) -> None:
        result, _command, _capture, directory = self.run_capture(
            exit_code=0, deadline=True
        )
        self.assertEqual(result["status"], "interrupted")
        self.assertEqual(result["cleanup"], "confirmed")
        self.assertEqual(result["observer_stop_reason"], "observation deadline reached")
        self.assertTrue((directory / "diagnostics/stop.request").exists())


if __name__ == "__main__":
    unittest.main()
