# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Once-only audit orchestration keeps failed and interrupted science distinct."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

from scripts import case_audit, case_measure


class SourceRecordingProtocolTests(unittest.TestCase):
    def test_manifest_selects_actual_dynamic_backend_families(self) -> None:
        declaration = json.loads(
            (case_measure.ROOT / ".config/process-cases.json").read_text()
        )
        idas = [w for w in declaration["workloads"] if w.get("backend") == "idas"]
        diffsol = [w for w in declaration["workloads"] if w.get("backend") == "diffsol"]
        self.assertTrue(idas)
        self.assertTrue(diffsol)
        for workload in idas:
            with self.subTest(case=workload["id"]):
                families = set(workload["detail_families"])
                self.assertTrue(
                    {"dynamic_segment_summary.v1", "dynamic_output.v1"} <= families
                )
                self.assertFalse(
                    {"dynamic_native_return.v1", "dynamic_native_step.v1"} & families
                )
        for workload in diffsol:
            with self.subTest(case=workload["id"]):
                self.assertIn("dynamic_native_return.v1", workload["detail_families"])

    def test_default_selection_is_materialized_before_launch_and_explicit_request_wins(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            audit = Path(temporary)
            env = {
                "PSE_PROCESS_COST_SPEC": json.dumps(
                    {"detail_families": ["root_evaluation.v1"]}
                )
            }
            selected = case_audit.prepare_source_recording(env, audit)
            request = json.loads(
                Path(selected["PSE_DIAGNOSTIC_RECORD_REQUEST"]).read_text()
            )
            self.assertEqual(request["version"], 2)
            self.assertEqual(request["detail"]["families"], ["root_evaluation.v1"])
            self.assertEqual(request["detail"]["requirement"], "complete")
            self.assertEqual(request["detail"]["duration"], {"secs": 60, "nanos": 0})
            self.assertFalse(Path(request["acknowledgement"]).exists())
            explicit = {
                **env,
                "PSE_DIAGNOSTIC_RECORD_REQUEST": selected[
                    "PSE_DIAGNOSTIC_RECORD_REQUEST"
                ],
            }
            self.assertIs(
                case_audit.prepare_source_recording(explicit, audit), explicit
            )
            self.assertEqual(env.keys(), {"PSE_PROCESS_COST_SPEC"})

    def test_review_selection_uses_v2_envelope_and_v1_detail_with_finite_handoff(
        self,
    ) -> None:
        from pse import codec  # noqa: PLC0415
        from pse.contracts.documents import SourceRecordingRequest  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as temporary:
            audit = Path(temporary)
            selected = case_audit.prepare_source_recording(
                {
                    "PSE_PROCESS_COST_SPEC": json.dumps(
                        {"detail_families": ["root_evaluation.v1"]}
                    )
                },
                audit,
                review_artifacts=True,
            )
            path = Path(selected["PSE_DIAGNOSTIC_RECORD_REQUEST"])
            request = codec.decode_json(path.read_bytes(), SourceRecordingRequest)
            self.assertEqual(request.version, 2)
            self.assertEqual(request.detail.version, 1)
            self.assertEqual(request.detail.families, ("root_evaluation.v1",))
            self.assertEqual(request.detail.budget.memory_bytes, 64 << 20)
            self.assertEqual(request.detail.budget.output_bytes, 256 << 20)
            self.assertIsNotNone(request.review)
            assert request.review is not None
            self.assertTrue(request.review.context.include_sources)
            self.assertTrue(request.review.context.include_native_records)
            self.assertEqual(request.review.context.maximum_bytes, 16 << 20)
            self.assertEqual(request.review.maximum_bytes, 64 << 20)
            self.assertEqual(request.review.maximum_members, 256)
            legacy = json.loads(path.read_text())
            legacy["version"] = 1
            path.write_text(json.dumps(legacy))
            with self.assertRaises(ValueError):
                case_audit.prepare_source_recording(
                    selected, audit, review_artifacts=True
                )

    def test_empty_selection_does_not_create_recording_and_invalid_selection_refuses(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            audit = Path(temporary)
            env = {"PSE_PROCESS_COST_SPEC": json.dumps({"detail_families": []})}
            self.assertIs(case_audit.prepare_source_recording(env, audit), env)
            self.assertEqual(list(audit.iterdir()), [])
            for families in ("root_evaluation.v1", [None], [1]):
                with self.assertRaises(ValueError):
                    case_audit.prepare_source_recording(
                        {
                            "PSE_PROCESS_COST_SPEC": json.dumps(
                                {"detail_families": families}
                            )
                        },
                        audit,
                    )


class DiagnosticDouble:
    """Named mock handles for the observer's native diagnostic boundary."""

    InspectionError = RuntimeError

    def __init__(
        self,
        *,
        list_processes: Mock | None = None,
        read_checkpoint: Mock | None = None,
        inspect_image: Mock | None = None,
        inspect_frozen: Mock | None = None,
        operation_id: Mock | None = None,
        operation_text: Mock | None = None,
    ) -> None:
        self.list_processes = list_processes if list_processes is not None else Mock()
        self.read_checkpoint = (
            read_checkpoint if read_checkpoint is not None else Mock()
        )
        self.inspect_image = inspect_image if inspect_image is not None else Mock()
        self.inspect_frozen = inspect_frozen if inspect_frozen is not None else Mock()
        self.operation_id = operation_id if operation_id is not None else Mock()
        self.operation_text = operation_text if operation_text is not None else Mock()


class CodecDouble:
    def __init__(self) -> None:
        self.decode_json = Mock(side_effect=lambda data, _kind: json.loads(data))
        self.encode_json = Mock(side_effect=lambda value: json.dumps(value).encode())


class ObserverDouble(case_audit.Observer):
    # Production methods still run on this object. Mock controls are accessed
    # separately, without claiming a SimpleNamespace implements the native module.
    diagnostic_double: DiagnosticDouble
    codec_double: CodecDouble
    write_mock: Mock


class AuditTests(unittest.TestCase):
    def install_diagnostics(
        self, observer: ObserverDouble, diagnostics: DiagnosticDouble
    ) -> None:
        observer.diagnostic_double = diagnostics
        self.enterContext(
            patch.object(observer, "diagnostics", diagnostics, create=True)
        )

    def association(self, observer: case_audit.Observer) -> dict:
        association = observer.managed_association
        assert association is not None
        return association

    def managed_reader(self, observer: ObserverDouble, operations: list) -> None:
        observer.codec_double = CodecDouble()
        self.enterContext(
            patch.object(observer, "codec", observer.codec_double, create=True)
        )
        self.enterContext(patch.object(observer, "identity_type", dict, create=True))
        self.enterContext(patch.object(observer, "receiver_type", dict, create=True))
        observer.diagnostic_double.read_checkpoint = Mock(
            return_value=SimpleNamespace(operations=operations)
        )

        def inspect(image: SimpleNamespace, request: dict, **_kwargs: object) -> object:
            if isinstance(request, dict) and "include_related" in request:
                operation = next(
                    operation
                    for operation in image.operations
                    if operation.id == request["operation"]
                )
                return SimpleNamespace(
                    operation=operation, transport=SimpleNamespace(partial=False)
                )
            return SimpleNamespace(operation=None, related=())

        observer.diagnostic_double.inspect_image = Mock(side_effect=inspect)

    def ready_observer(self, directory: Path) -> tuple[ObserverDouble, dict]:
        identity = {
            "process": "exact-instance",
            "locator": {"pid": 424242, "start_ticks": 7, "executable": None},
        }
        marker = {
            "ready": True,
            "canonical_database": "owned",
            "pid": 424242,
            "nonce": "registered-nonce",
            "diagnostics": {
                "identity": identity,
                "checkpoint": str(directory / "worker.json"),
                "enabled": True,
                "writer_failure": None,
            },
        }
        (directory / "managed-target.json").write_text(
            json.dumps({"state": str(directory), "database": "owned"})
        )
        (directory / "primary-receiver.json").write_text(json.dumps(marker))
        observer = object.__new__(ObserverDouble)
        observer.recorded_operations = {}
        observer.frozen_member_image = None
        observer.frozen_member_index = 0
        observer.frozen_request_index = 0
        observer.evidence_type = Mock(side_effect=SimpleNamespace)
        observer.frozen_request_type = Mock(side_effect=SimpleNamespace)
        observer.directory = directory
        observer.managed_checkpoint = None
        observer.managed_association = None
        observer.managed_offset = 0
        observer.managed_cursor = None
        observer.managed_operations = []
        observer.reads = 0
        observer.failures = []
        observer.write_mock = Mock()
        self.enterContext(patch.object(observer, "write", observer.write_mock))
        observer.request = Mock(side_effect=lambda **kwargs: kwargs)
        observer.list_request = Mock()
        self.install_diagnostics(observer, DiagnosticDouble(list_processes=Mock()))
        self.managed_reader(
            observer,
            [SimpleNamespace(id="completed responsibility", outcome="completed")],
        )
        return observer, marker

    def test_ready_locator_associates_short_process_without_catalogue_or_proc(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            observer, marker = self.ready_observer(Path(temporary))
            observer.inspect_managed()
            observer.diagnostic_double.list_processes.assert_not_called()
            observer.diagnostic_double.read_checkpoint.assert_called_once_with(
                marker["diagnostics"]["checkpoint"],
                expected=marker["diagnostics"]["identity"],
            )
            image = observer.diagnostic_double.read_checkpoint.return_value
            self.assertIs(
                observer.diagnostic_double.inspect_image.call_args.args[0], image
            )
            self.assertIs(observer.write_mock.call_args.args[1], image)
            self.assertEqual(observer.failures, [])

    def test_pending_handoff_is_cached_then_reads_after_marker_and_process_disappear(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            image = observer.diagnostic_double.read_checkpoint.return_value
            observer.diagnostic_double.read_checkpoint.side_effect = [
                FileNotFoundError("first checkpoint pending"),
                image,
            ]
            observer.inspect_managed()
            self.assertEqual(
                self.association(observer)["identity"],
                marker["diagnostics"]["identity"],
            )
            self.assertIn("pending", observer.failures[0])
            (directory / "primary-receiver.json").unlink()
            (directory / "managed-target.json").unlink()
            observer.inspect_managed()
            self.assertEqual(observer.diagnostic_double.inspect_image.call_count, 1)
            observer.diagnostic_double.list_processes.assert_not_called()

    def test_cached_association_never_rebinds_to_replacement_pid_or_checkpoint(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            observer.inspect_managed()
            selected = dict(self.association(observer))
            marker["diagnostics"]["identity"]["locator"]["start_ticks"] += 1
            marker["diagnostics"]["identity"]["process"] = "replacement-instance"
            marker["diagnostics"]["checkpoint"] = str(directory / "replacement.json")
            marker["nonce"] = "replacement-nonce"
            (directory / "primary-receiver.json").write_text(json.dumps(marker))
            observer.diagnostic_double.read_checkpoint.side_effect = ValueError(
                "checkpoint process/start identity mismatch"
            )
            observer.inspect_managed()
            self.assertEqual(observer.managed_association, selected)
            self.assertEqual(observer.diagnostic_double.inspect_image.call_count, 1)
            self.assertIn("identity mismatch", observer.failures[-1])
            self.assertEqual(
                observer.diagnostic_double.read_checkpoint.call_args.kwargs["expected"],
                selected["identity"],
            )

    def test_wrong_database_missing_start_and_opt_out_remain_diagnostic_absence(
        self,
    ) -> None:
        for variant in ("database", "start", "disabled"):
            with self.subTest(variant), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                observer, marker = self.ready_observer(directory)
                if variant == "database":
                    marker["canonical_database"] = "wrong"
                elif variant == "start":
                    marker["diagnostics"]["identity"]["locator"]["start_ticks"] = None
                else:
                    marker["diagnostics"]["enabled"] = False
                (directory / "primary-receiver.json").write_text(json.dumps(marker))
                observer.inspect_managed()
                self.assertIsNone(observer.managed_association)
                observer.diagnostic_double.read_checkpoint.assert_not_called()
                observer.diagnostic_double.list_processes.assert_not_called()
                self.assertTrue(
                    json.loads((directory / "primary-receiver.json").read_text())[
                        "ready"
                    ]
                )

    def test_writer_failure_does_not_reject_registered_last_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            marker["diagnostics"]["writer_failure"] = "optional writer failed"
            (directory / "primary-receiver.json").write_text(json.dumps(marker))
            observer.inspect_managed()
            self.assertEqual(
                self.association(observer)["writer_failure"], "optional writer failed"
            )
            self.assertEqual(observer.diagnostic_double.inspect_image.call_count, 1)

    def caller_ready_image(
        self, marker: dict, state: Path
    ) -> tuple[SimpleNamespace, bytes]:
        handoff = {
            **marker["diagnostics"],
            "available": True,
            "receiver_state": str(state),
            "canonical_database": marker["canonical_database"],
            "nonce": marker["nonce"],
        }
        payload = json.dumps(handoff).encode()
        chunks = [payload[index : index + 128] for index in range(0, len(payload), 128)]
        selection_sources = []
        selection_metrics = []
        for name, value in (
            ("receiver.state", str(state)),
            ("receiver.database", marker["canonical_database"]),
        ):
            data = value.encode()
            parts = [data[index : index + 128] for index in range(0, len(data), 128)]
            selection_sources.extend(
                SimpleNamespace(name=f"{name}.{index}", bytes=part)
                for index, part in enumerate(parts)
            )
            selection_metrics.extend(
                [
                    SimpleNamespace(
                        name=f"{name}_chunks",
                        latest={"kind": "Integer", "value": len(parts)},
                    ),
                    SimpleNamespace(
                        name=f"{name}_bytes",
                        latest={"kind": "Integer", "value": len(data)},
                    ),
                ]
            )
        operation = SimpleNamespace(
            id=SimpleNamespace(sequence=7),
            summary=SimpleNamespace(purpose="registered receiver checkpoint"),
            coverage=SimpleNamespace(stale=False),
            metrics=[
                *selection_metrics,
                SimpleNamespace(
                    name="receiver.handoff_chunks",
                    latest={"kind": "Integer", "value": len(chunks)},
                ),
                SimpleNamespace(
                    name="receiver.handoff_bytes",
                    latest={"kind": "Integer", "value": len(payload)},
                ),
            ],
            sources=[
                *selection_sources,
                *(
                    SimpleNamespace(name=f"receiver.handoff.{index}", bytes=chunk)
                    for index, chunk in enumerate(chunks)
                ),
            ],
        )
        return SimpleNamespace(operations=[operation], clock=SimpleNamespace()), payload

    def test_actual_caller_ready_handoff_survives_short_receiver_and_removed_marker(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            target = json.loads((directory / "managed-target.json").read_text())
            target["caller"] = {
                "checkpoint": str(directory / "caller.json"),
                "identity": {"process": "caller-instance"},
            }
            (directory / "managed-target.json").write_text(json.dumps(target))
            (directory / "primary-receiver.json").unlink()
            image, _payload = self.caller_ready_image(marker, directory)
            receiver_image = observer.diagnostic_double.read_checkpoint.return_value

            observer.diagnostic_double.read_checkpoint.side_effect = [
                image,
                receiver_image,
            ]
            observer.inspect_managed()
            self.assertEqual(self.association(observer)["nonce"], marker["nonce"])
            self.assertEqual(
                self.association(observer)["identity"],
                marker["diagnostics"]["identity"],
            )
            self.assertEqual(
                self.association(observer)["source"],
                "actual caller ready responsibility",
            )
            self.assertEqual(observer.diagnostic_double.inspect_image.call_count, 2)
            self.assertIs(
                observer.diagnostic_double.inspect_image.call_args_list[0].args[0],
                image,
            )
            self.assertEqual(
                [
                    call.args[0]
                    for call in observer.diagnostic_double.read_checkpoint.call_args_list
                ],
                [target["caller"]["checkpoint"], marker["diagnostics"]["checkpoint"]],
            )
            observer.diagnostic_double.list_processes.assert_not_called()
            self.assertEqual(observer.failures, [])

    def test_partial_caller_handoff_never_reconstructs_or_falls_back_to_replacement_marker(
        self,
    ) -> None:
        for missing in ("chunk", "inventory", "extent", "selection", "pending"):
            with self.subTest(missing), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                observer, marker = self.ready_observer(directory)
                target = json.loads((directory / "managed-target.json").read_text())
                target["caller"] = {
                    "checkpoint": str(directory / "caller.json"),
                    "identity": {"process": "caller-instance"},
                }
                (directory / "managed-target.json").write_text(json.dumps(target))
                image, _payload = self.caller_ready_image(marker, directory)
                if missing == "chunk":
                    image.operations[0].sources.pop()
                elif missing == "inventory":
                    image.operations[0].metrics.pop()
                elif missing == "extent":
                    next(
                        metric
                        for metric in image.operations[0].metrics
                        if metric.name == "receiver.handoff_bytes"
                    ).latest["value"] += 1
                elif missing == "selection":
                    image.operations[0].sources.pop(0)
                else:
                    image.operations.clear()

                observer.diagnostic_double.read_checkpoint.return_value = image
                observer.inspect_managed()
                self.assertIsNone(observer.managed_association)
                self.assertEqual(
                    observer.diagnostic_double.read_checkpoint.call_count, 1
                )
                self.assertEqual(
                    observer.diagnostic_double.inspect_image.call_count,
                    int(missing not in {"pending", "selection"}),
                )
                observer.diagnostic_double.list_processes.assert_not_called()
                if missing != "pending":
                    self.assertIn("handoff", observer.failures[-1])

    def test_caller_discovery_selects_exact_state_among_same_database_summaries(
        self,
    ) -> None:
        for unrelated_incomplete in (False, True):
            with (
                self.subTest(unrelated_incomplete=unrelated_incomplete),
                tempfile.TemporaryDirectory() as temporary,
            ):
                directory = Path(temporary)
                observer, marker = self.ready_observer(directory)
                selected, _payload = self.caller_ready_image(marker, directory)
                other, _payload = self.caller_ready_image(
                    {**marker, "nonce": "other-receiver"}, directory / "other-state"
                )
                other.operations[0].id.sequence = 8
                if unrelated_incomplete:
                    other.operations[0].metrics = [
                        metric
                        for metric in other.operations[0].metrics
                        if not metric.name.startswith("receiver.handoff_")
                    ]
                    other.operations[0].sources = [
                        source
                        for source in other.operations[0].sources
                        if not source.name.startswith("receiver.handoff.")
                    ]
                selected.operations.extend(other.operations)
                target = {
                    "state": str(directory),
                    "database": "owned",
                    "caller": {"identity": {}, "checkpoint": "caller.json"},
                }

                observer.diagnostic_double.read_checkpoint.return_value = selected
                association = observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control
                assert association is not None
                self.assertEqual(association["nonce"], marker["nonce"])
                observer.diagnostic_double.read_checkpoint.return_value = other
                self.assertIsNone(observer._managed_from_caller(target))  # noqa: SLF001 -- exact caller-summary recovery control

    def test_newer_incomplete_exact_target_never_falls_back_to_older_receiver(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            selected, _payload = self.caller_ready_image(marker, directory)
            newer, _payload = self.caller_ready_image(
                {**marker, "nonce": "new-receiver"}, directory
            )
            newer.operations[0].id.sequence = 8
            newer.operations[0].sources.pop()
            selected.operations.extend(newer.operations)

            observer.diagnostic_double.read_checkpoint.return_value = selected
            target = {
                "state": str(directory),
                "database": "owned",
                "caller": {"identity": {}, "checkpoint": "caller.json"},
            }
            with self.assertRaisesRegex(ValueError, "chunks incomplete"):
                observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control

    def test_long_registered_state_chunks_select_exact_receiver_and_reject_gaps(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            state = (
                directory
                / ("service-state" * 8)
                / ".receivers"
                / ("a" * 64)
                / "canonical-database"
            )
            self.assertGreaterEqual(len(str(state).encode()), 200)
            observer, marker = self.ready_observer(directory)
            marker["canonical_database"] = "database-" + "d" * 128
            selected, _payload = self.caller_ready_image(marker, state)
            other, _payload = self.caller_ready_image(
                {**marker, "nonce": "unrelated"}, state.parent / "other-database"
            )
            other.operations[0].id.sequence = 8
            other.operations[0].metrics = [
                metric
                for metric in other.operations[0].metrics
                if not metric.name.startswith("receiver.handoff_")
            ]
            other.operations[0].sources = [
                source
                for source in other.operations[0].sources
                if not source.name.startswith("receiver.handoff.")
            ]
            selected.operations.extend(other.operations)

            observer.diagnostic_double.read_checkpoint.return_value = selected
            target = {
                "state": str(state),
                "database": marker["canonical_database"],
                "caller": {"identity": {}, "checkpoint": "caller.json"},
            }
            association = observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control
            assert association is not None
            self.assertEqual(association["nonce"], marker["nonce"])
            self.assertEqual(association["state"], str(state))
            self.assertLessEqual(len(selected.operations[0].sources), 16)
            self.assertTrue(
                all(
                    len(source.bytes) <= 128
                    for source in selected.operations[0].sources
                )
            )
            for prefix in ("receiver.state", "receiver.database"):
                with self.subTest(prefix=prefix):
                    image, _payload = self.caller_ready_image(marker, state)
                    image.operations[0].sources = [
                        source
                        for source in image.operations[0].sources
                        if source.name != f"{prefix}.1"
                    ]
                    observer.diagnostic_double.read_checkpoint.return_value = image
                    with self.assertRaisesRegex(ValueError, "chunks incomplete"):
                        observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control

    def test_typed_handoff_must_match_copied_selection_discriminator(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            image, _payload = self.caller_ready_image(marker, directory / "other-state")
            image.operations[0].sources[0].bytes = str(directory).encode()
            next(
                metric
                for metric in image.operations[0].metrics
                if metric.name == "receiver.state_bytes"
            ).latest["value"] = len(str(directory).encode())

            observer.diagnostic_double.read_checkpoint.return_value = image
            target = {
                "state": str(directory),
                "database": "owned",
                "caller": {"identity": {}, "checkpoint": "caller.json"},
            }
            with self.assertRaisesRegex(ValueError, "selection differs"):
                observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control

    def test_expired_caller_summary_cannot_establish_new_association(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            image, _payload = self.caller_ready_image(marker, directory)
            image.operations[0].coverage.stale = True
            target = {
                "state": str(directory),
                "database": "owned",
                "caller": {"identity": {}, "checkpoint": "caller.json"},
            }
            (directory / "managed-target.json").write_text(json.dumps(target))

            observer.diagnostic_double.read_checkpoint.return_value = image
            observer.inspect_managed()
            self.assertIsNone(observer.managed_association)
            self.assertEqual(observer.diagnostic_double.read_checkpoint.call_count, 1)
            self.assertIn("expired", observer.failures[-1])

    def test_cached_caller_association_keeps_reading_prefix_after_summary_expiry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            image, _payload = self.caller_ready_image(marker, directory)
            receiver_image = observer.diagnostic_double.read_checkpoint.return_value
            target = {
                "state": str(directory),
                "database": "owned",
                "caller": {"identity": {}, "checkpoint": "caller.json"},
            }
            (directory / "managed-target.json").write_text(json.dumps(target))

            observer.diagnostic_double.read_checkpoint.side_effect = [
                image,
                receiver_image,
                receiver_image,
            ]
            observer.inspect_managed()
            selected = observer.managed_association
            image.operations[0].coverage.stale = True
            (directory / "managed-target.json").unlink()
            (directory / "primary-receiver.json").unlink()
            observer.inspect_managed()
            self.assertIs(observer.managed_association, selected)
            self.assertEqual(
                [
                    call.args[0]
                    for call in observer.diagnostic_double.read_checkpoint.call_args_list
                ],
                [
                    "caller.json",
                    marker["diagnostics"]["checkpoint"],
                    marker["diagnostics"]["checkpoint"],
                ],
            )
            self.assertEqual(observer.failures, [])

    def test_new_caller_association_requires_clock_but_tolerates_other_transport_loss(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            observer, marker = self.ready_observer(directory)
            image, _payload = self.caller_ready_image(marker, directory)

            observer.diagnostic_double.read_checkpoint.return_value = image
            observer.diagnostic_double.inspect_image.side_effect = None
            observer.diagnostic_double.inspect_image.return_value = SimpleNamespace(
                operation=image.operations[0],
                transport=SimpleNamespace(partial=True),
            )
            target = {
                "state": str(directory),
                "database": "owned",
                "caller": {"identity": {}, "checkpoint": "caller.json"},
            }
            association = observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control
            assert association is not None
            self.assertEqual(association["nonce"], marker["nonce"])
            image.clock = None
            with self.assertRaisesRegex(ValueError, "clock unavailable"):
                observer._managed_from_caller(target)  # noqa: SLF001 -- exact caller-summary recovery control
            self.assertEqual(observer.diagnostic_double.inspect_image.call_count, 1)

    def test_missing_outcome_and_nonzero_exit_cannot_be_success(self) -> None:
        self.assertEqual(case_audit.reconcile_outcome(0, None)["status"], "failed")
        self.assertEqual(
            case_audit.reconcile_outcome(1, {"status": "completed"})["status"], "failed"
        )
        interrupted = case_audit.reconcile_outcome(
            1, {"status": "interrupted", "cleanup": "confirmed"}
        )
        self.assertEqual(interrupted["status"], "interrupted")
        self.assertEqual(interrupted["cleanup"], "confirmed")

    def test_audit_launches_one_process_with_separate_diagnostic_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            program = (
                "import os,json,pathlib; "
                "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR']); "
                "assert os.environ['PSE_DIAGNOSTICS_DIR']==str(p/'checkpoints'); "
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'interrupted','native_joined':True})); "
                "raise SystemExit(2)"
            )
            with patch.object(case_audit, "Observer") as observer:
                observer.return_value.finish.return_value = {"inspection_windows": 1}
                result = case_audit.run_case(
                    [sys.executable, "-c", program],
                    cwd=directory,
                    env=dict(os.environ),
                    directory=directory,
                )
            self.assertEqual(result["status"], "interrupted")
            self.assertTrue(result["native_joined"])
            self.assertEqual(result["exit_code"], 2)
            self.assertEqual(
                json.loads((directory / "diagnostics/observation.json").read_text()),
                result,
            )

    def test_audit_requires_once_only_explicit_nonrepeated_selection(self) -> None:
        for arguments in (
            ["out", "--audit-diagnostics"],
            ["out", "--smoke", "--audit-diagnostics"],
            ["out", "--smoke", "--audit-diagnostics", "--case", "a", "--case", "a"],
            ["out", "--smoke", "--audit-diagnostics", "--case", "document-admission"],
        ):
            with (
                patch.object(sys, "argv", ["case_measure", *arguments]),
                self.assertRaises(SystemExit) as exit_error,
            ):
                case_measure.main()
            self.assertEqual(exit_error.exception.code, 2)

    def test_deadline_requests_cooperative_stop_and_awaits_cleanup(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            program = (
                "import os,pathlib,time,signal,json\n"
                "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR'])\n"
                "signal.signal(signal.SIGTERM,lambda *_: (p/'bad-signal').touch())\n"
                "while not (p/'stop.request').exists(): time.sleep(.01)\n"
                "time.sleep(.3)\n"
                "(p/'cleaned').touch()\n"
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'interrupted',"
                "'native_joins_completed':1,'fixture_cleanups_completed':1}))\n"
                "raise SystemExit(3)\n"
            )
            with patch.object(case_audit, "Observer") as observer:
                observer.return_value.finish.side_effect = lambda: {
                    "cleanup_visible_at_finish": (
                        directory / "diagnostics/cleaned"
                    ).exists()
                }
                result = case_audit.run_case(
                    [sys.executable, "-c", program],
                    cwd=directory,
                    env=dict(os.environ),
                    directory=directory,
                    deadline_seconds=0.05,
                )
            self.assertEqual(result["status"], "interrupted")
            self.assertEqual(result["exit_code"], 3)
            self.assertEqual(result["native_joins_completed"], 1)
            self.assertEqual(result["fixture_cleanups_completed"], 1)
            self.assertTrue(result["cleanup_visible_at_finish"])
            self.assertFalse(result["operator_interrupted"])
            self.assertEqual(result["observation_deadline_seconds"], 0.05)
            self.assertEqual(
                result["observer_stop_reason"], "observation deadline reached"
            )
            self.assertFalse((directory / "diagnostics/bad-signal").exists())

    def test_deadline_does_not_replace_failed_scientific_outcome(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            program = (
                "import os,pathlib,time,json\n"
                "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR'])\n"
                "while not (p/'stop.request').exists(): time.sleep(.01)\n"
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'failed',"
                "'fixture_cleanup_failures':1}))\n"
                "raise SystemExit(1)\n"
            )
            with patch.object(case_audit, "Observer") as observer:
                observer.return_value.finish.return_value = {}
                result = case_audit.run_case(
                    [sys.executable, "-c", program],
                    cwd=directory,
                    env=dict(os.environ),
                    directory=directory,
                    deadline_seconds=0.01,
                )
            self.assertEqual(result["status"], "failed")
            self.assertEqual(result["fixture_cleanup_failures"], 1)
            self.assertEqual(
                result["observer_stop_reason"], "observation deadline reached"
            )

    def test_deadline_requires_audit_and_finite_positive_interval(self) -> None:
        for value in ("nan", "inf", "0", "-1"):
            with (
                self.subTest(value=value),
                patch.object(
                    sys,
                    "argv",
                    [
                        "case_measure",
                        "out",
                        "--smoke",
                        "--audit-diagnostics",
                        "--case",
                        "cold-small-1",
                        "--audit-deadline-seconds",
                        value,
                    ],
                ),
                patch.object(case_measure, "smoke_campaign") as campaign,
                self.assertRaises(SystemExit) as exit_error,
            ):
                case_measure.main()
            self.assertEqual(exit_error.exception.code, 2)
            campaign.assert_not_called()
        with (
            patch.object(
                sys, "argv", ["case_measure", "out", "--audit-deadline-seconds", "60"]
            ),
            self.assertRaises(SystemExit) as exit_error,
        ):
            case_measure.main()
        self.assertEqual(exit_error.exception.code, 2)

    def test_compact_audit_selection_preserves_order_and_forwards_deadline(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = ["warm-small-1", "cold-small-1", "sparse-fit-12-1"]
            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "case_measure",
                        str(directory / "out"),
                        "--smoke",
                        "--audit-diagnostics",
                        "--audit-deadline-seconds",
                        "60",
                        *[argument for case in cases for argument in ("--case", case)],
                    ],
                ),
                patch.object(
                    case_measure.validation,
                    "fresh_output",
                    return_value=directory / "out",
                ),
                patch.object(case_measure, "smoke_campaign") as campaign,
            ):
                self.assertEqual(case_measure.main(), 0)
            self.assertEqual(campaign.call_args.args[4], set(cases))
            self.assertEqual(campaign.call_args.kwargs["audit_order"], cases)
            self.assertEqual(campaign.call_args.kwargs["audit_deadline_seconds"], 60)

    def test_declared_compact_shapes_keep_output_counts_and_stop_capability(
        self,
    ) -> None:
        declaration = json.loads(
            (case_measure.ROOT / ".config/process-cases.json").read_text()
        )
        compact = [
            w for w in declaration["workloads"] if w["id"].startswith("compact-")
        ]
        self.assertTrue(compact)
        self.assertEqual(len({w["id"] for w in compact}), len(compact))
        requested = ["compact-idas-ode-25", "compact-root-coupled-8", "compact-minlp"]
        selected = case_measure.selected_workloads((declaration,), requested)
        self.assertEqual({w["id"] for w in selected}, set(requested))
        by_id = {w["id"]: w for w in selected}
        self.assertEqual(by_id[requested[0]]["samples"], 25)
        self.assertEqual(by_id[requested[1]]["blocks"], 8)
        for workload in compact:
            self.assertTrue(workload["extended"])
            self.assertIs(workload["audit_supported"], True)
            self.assertTrue(workload["case_spec"]["assertions"])
            self.assertTrue(workload["case_spec"]["observations"])

    def test_foreground_interrupt_requests_stop_and_waits_for_child_cleanup(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            child = (
                "import os,pathlib,time,signal,json; "
                "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR']); "
                "signal.signal(signal.SIGINT,lambda *_: (p/'bad-signal').write_text('signal')); "
                "(p/'ready').write_text('ready')\n"
                "while not (p/'stop.request').exists(): time.sleep(.02)\n"
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'interrupted','cleanup':'confirmed'}))\n"
            )
            parent = (
                "from scripts import case_audit; from pathlib import Path; from unittest.mock import patch; import os,sys\n"
                "with patch.object(case_audit,'Observer') as observer:\n"
                " observer.return_value.finish.return_value={}\n"
                " try: case_audit.run_case([sys.executable,'-c',sys.argv[2]],cwd=Path.cwd(),env=dict(os.environ),directory=Path(sys.argv[1]))\n"
                " except KeyboardInterrupt: pass\n"
            )
            with subprocess.Popen(
                [sys.executable, "-c", parent, str(directory), child],
                cwd=case_measure.ROOT,
                start_new_session=True,
            ) as process:
                deadline = time.monotonic() + 10
                while not (directory / "diagnostics/ready").exists():
                    self.assertIsNone(
                        process.poll(), "audit parent exited before child was ready"
                    )
                    self.assertLess(time.monotonic(), deadline)
                    time.sleep(0.02)
                os.killpg(process.pid, signal.SIGINT)
                self.assertEqual(process.wait(timeout=10), 0)
            self.assertFalse((directory / "diagnostics/bad-signal").exists())
            self.assertEqual(
                json.loads((directory / "diagnostics/audit-outcome.json").read_text())[
                    "cleanup"
                ],
                "confirmed",
            )

    def test_managed_discovery_requires_registered_database_and_live_start_identity(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            checkpoint = directory / "worker.json"
            checkpoint.write_text("retained worker prefix")
            (directory / "managed-target.json").write_text(
                json.dumps({"state": str(directory), "database": "owned"})
            )
            (directory / "primary-receiver.json").write_text(
                json.dumps(
                    {"pid": os.getpid(), "ready": True, "canonical_database": "owned"}
                )
            )
            start = int(
                Path(f"/proc/{os.getpid()}/stat")
                .read_text()
                .rsplit(")", 1)[1]
                .split()[19]
            )
            for matches in (False, True):
                observer = object.__new__(ObserverDouble)
                observer.recorded_operations = {}
                observer.frozen_member_image = None
                observer.frozen_member_index = 0
                observer.frozen_request_index = 0
                observer.evidence_type = Mock(side_effect=SimpleNamespace)
                observer.frozen_request_type = Mock(side_effect=SimpleNamespace)
                observer.directory = directory
                observer.managed_checkpoint = None
                observer.managed_operations = []
                observer.managed_offset = 0
                observer.managed_cursor = None
                observer.reads = 1
                observer.failures = []
                observer.list_request = Mock()
                observer.request = Mock()
                observer.write = Mock()
                process = SimpleNamespace(
                    process="worker-instance",
                    locator=SimpleNamespace(
                        pid=os.getpid(), start_ticks=start if matches else start + 1
                    ),
                    checkpoint=str(checkpoint),
                    operations=[SimpleNamespace(id="operation", outcome=None)],
                    partial=False,
                )
                diagnostics = DiagnosticDouble(
                    list_processes=Mock(
                        return_value=SimpleNamespace(processes=[process], next=None)
                    ),
                )
                self.install_diagnostics(observer, diagnostics)
                with self.subTest(matches=matches):
                    self.managed_reader(observer, process.operations)
                    observer.inspect_managed()
                self.assertEqual(
                    observer.managed_checkpoint, checkpoint if matches else None
                )
                self.assertEqual(diagnostics.inspect_image.call_count, int(matches))
                self.assertEqual(observer.failures, [])

    def test_managed_reads_rotate_across_sixteen_active_owners(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "managed-target.json").write_text(
                json.dumps({"state": str(directory), "database": "owned"})
            )
            checkpoint = directory / "worker.json"
            checkpoint.write_text("retained worker prefix")
            observer = object.__new__(ObserverDouble)
            observer.recorded_operations = {}
            observer.frozen_member_image = None
            observer.frozen_member_index = 0
            observer.frozen_request_index = 0
            observer.evidence_type = Mock(side_effect=SimpleNamespace)
            observer.frozen_request_type = Mock(side_effect=SimpleNamespace)
            observer.directory = directory
            observer.managed_checkpoint = checkpoint
            observer.managed_association = {
                "state": str(directory),
                "checkpoint": str(checkpoint),
                "identity": {
                    "process": "selected",
                    "locator": {"pid": 123, "start_ticks": 4},
                },
            }
            observer.managed_operations = list(range(16))
            observer.managed_offset = 0
            observer.reads = 0
            observer.failures = []
            observer.request = Mock(side_effect=lambda **kwargs: kwargs)
            observer.write = Mock()
            self.install_diagnostics(observer, DiagnosticDouble())
            self.managed_reader(observer, [SimpleNamespace(id=i) for i in range(16)])
            observer.inspect_managed()
            self.assertEqual(observer.diagnostic_double.inspect_image.call_count, 8)
            observer.inspect_managed()
            requests = observer.diagnostic_double.inspect_image.call_args_list
            self.assertEqual(
                [call.args[1]["operation"] for call in requests], list(range(16))
            )
            self.assertEqual(observer.failures, [])

    def test_managed_discovery_continues_bounded_pages_across_reads(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            checkpoint = directory / "worker.json"
            checkpoint.write_text("retained worker prefix")
            (directory / "managed-target.json").write_text(
                json.dumps({"state": str(directory), "database": "owned"})
            )
            (directory / "primary-receiver.json").write_text(
                json.dumps(
                    {"pid": os.getpid(), "ready": True, "canonical_database": "owned"}
                )
            )
            start = int(
                Path(f"/proc/{os.getpid()}/stat")
                .read_text()
                .rsplit(")", 1)[1]
                .split()[19]
            )
            process = SimpleNamespace(
                process="worker-instance",
                locator=SimpleNamespace(pid=os.getpid(), start_ticks=start),
                checkpoint=str(checkpoint),
                operations=[SimpleNamespace(id="operation", outcome=None)],
                partial=False,
            )
            observer = object.__new__(ObserverDouble)
            observer.recorded_operations = {}
            observer.frozen_member_image = None
            observer.frozen_member_index = 0
            observer.frozen_request_index = 0
            observer.evidence_type = Mock(side_effect=SimpleNamespace)
            observer.frozen_request_type = Mock(side_effect=SimpleNamespace)
            observer.directory = directory
            observer.managed_checkpoint = None
            observer.managed_operations = []
            observer.managed_offset = 0
            observer.managed_cursor = None
            observer.reads = 0
            observer.failures = []
            observer.request = Mock()
            observer.list_request = Mock(side_effect=lambda **kwargs: kwargs)
            observer.write = Mock()
            self.install_diagnostics(
                observer,
                DiagnosticDouble(
                    list_processes=Mock(
                        side_effect=[
                            *(
                                SimpleNamespace(processes=[], next=f"page{i}")
                                for i in range(1, 5)
                            ),
                            SimpleNamespace(processes=[process], next=None),
                        ]
                    ),
                ),
            )
            self.managed_reader(observer, process.operations)
            observer.inspect_managed()
            self.assertIsNone(observer.managed_checkpoint)
            self.assertEqual(observer.diagnostic_double.list_processes.call_count, 4)
            observer.inspect_managed()
            self.assertEqual(observer.managed_checkpoint, checkpoint)
            self.assertEqual(
                observer.diagnostic_double.list_processes.call_args.args[0]["after"],
                "page4",
            )
            self.assertEqual(observer.failures, [])

    def test_terminal_navigation_includes_completed_related_owner(self) -> None:
        # This typed-snapshot control alone needs the native package boundary.
        from msgspec.structs import replace  # noqa: PLC0415

        from pse import codec  # noqa: PLC0415
        from pse.contracts.documents import (  # noqa: PLC0415
            InspectionDocument,
            OperationId,
        )
        from pse.contracts.enums import OperationOutcome  # noqa: PLC0415

        products = json.loads(
            (
                case_measure.ROOT
                / "python/pse/tests/fixtures/generated-native-boundaries/products.json"
            ).read_text()
        )
        source = codec.decode_json(
            json.dumps(products["diagnostic_inspection"]).encode(), InspectionDocument
        )
        assert source.operation is not None
        completed = replace(
            source.operation,
            id=OperationId(process_instance=(1, 2), sequence=1),
            outcome=OperationOutcome.COMPLETED,
            phases=(),
        )
        child_operation = replace(
            completed, id=OperationId(process_instance=(1, 2), sequence=2)
        )
        identities = {"root": completed.id, "child": child_operation.id}
        names = {value: key for key, value in identities.items()}
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            checkpoint = directory / "worker.json"
            checkpoint.write_text("retained prefix")
            (directory / "case-operation.json").write_text(
                json.dumps({"checkpoint": str(checkpoint), "operation": "root"})
            )
            for terminal in (False, True):
                observer = object.__new__(ObserverDouble)
                observer.recorded_operations = {}
                observer.frozen_member_image = None
                observer.frozen_member_index = 0
                observer.frozen_request_index = 0
                observer.evidence_type = Mock(side_effect=SimpleNamespace)
                observer.frozen_request_type = Mock(side_effect=SimpleNamespace)
                observer.directory = directory
                observer.request = Mock(side_effect=lambda **kwargs: kwargs)
                observer.list_request = Mock()
                observer.write = Mock()
                observer.inspect_managed = Mock()
                observer.reads = 0
                observer.failures = []
                observer.started = time.monotonic()
                root = SimpleNamespace(
                    operation=completed,
                    related=[child_operation],
                )
                child = SimpleNamespace(operation=child_operation, related=[])
                observer.recorded_operations["child"] = child_operation
                self.install_diagnostics(
                    observer,
                    DiagnosticDouble(
                        operation_id=Mock(side_effect=identities.__getitem__),
                        operation_text=Mock(side_effect=names.__getitem__),
                        read_checkpoint=Mock(
                            return_value=SimpleNamespace(operations=(completed,))
                        ),
                        inspect_image=Mock(side_effect=[root, child]),
                        inspect_frozen=Mock(
                            side_effect=[
                                SimpleNamespace(inspection=root),
                                SimpleNamespace(inspection=child),
                            ]
                        ),
                        list_processes=Mock(return_value=SimpleNamespace(processes=[])),
                    ),
                )
                observer.inspect(terminal=terminal)
                if terminal:
                    observer.diagnostic_double.inspect_image.assert_not_called()
                    requests = observer.diagnostic_double.inspect_frozen.call_args_list
                    self.assertTrue(all(call.args[0].version == 2 for call in requests))
                    self.assertEqual(
                        [call.args[1].inspection["operation"] for call in requests],
                        [completed.id, child_operation.id],
                    )
                    self.assertEqual(
                        tuple(op.id for op in requests[0].args[0].operations),
                        (child_operation.id,),
                    )
                    self.assertEqual(observer.frozen_member_index, 1)
                else:
                    requests = observer.diagnostic_double.inspect_image.call_args_list
                    self.assertEqual(
                        [call.args[1]["operation"] for call in requests], [completed.id]
                    )
                self.assertEqual(observer.failures, [])


if __name__ == "__main__":
    unittest.main()
