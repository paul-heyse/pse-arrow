# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed main-thread qualification adapter over the native assessment owner."""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
from pathlib import Path

from pydantic import ValidationError

from scripts import preflight, pse_env, validation, validation_receipts
from scripts.capabilities.diagnostics import safe_path
from scripts.capabilities.qualification_contracts import (
    AssessmentRequest,
    CompilationRequest,
    OutputRequest,
)
from scripts.capabilities.worker_progress import OwnerReference, ProgressReporter
from scripts.validation_scope import Gate, expand

ROOT = Path(__file__).resolve().parents[2]


def selected(
    operation: str, document: str
) -> tuple[list[Gate], Path, bool, AssessmentRequest | None]:
    """Map only fixed typed operations; command and lifecycle meaning stay native."""
    if operation == "assessment":
        assessment = AssessmentRequest.model_validate_json(document)
        return (
            assessment.gates(),
            ROOT / assessment.output,
            assessment.group is None or assessment.group == "features-powerset",
            assessment,
        )
    if operation == "compilation":
        compilation = CompilationRequest.model_validate_json(document)
        return compilation.gates(), ROOT / compilation.output, True, None
    if operation not in {"contracts-check", "contracts-generate", "documentation"}:
        raise ValueError("unsupported fixed qualification operation")
    request = OutputRequest.model_validate_json(document)
    if operation == "contracts-check":
        return expand(("codegen-contracts-check",)), ROOT / request.output, False, None
    if operation == "contracts-generate":
        return (
            [Gate("codegen-contracts", input_scope="generation")],
            ROOT / request.output,
            False,
            None,
        )
    if operation == "documentation":
        return (
            [Gate("docs", input_scope="documentation")],
            ROOT / request.output,
            True,
            None,
        )
    raise ValueError("unsupported fixed qualification operation")


def execute(
    operation: str, document: str, *, progress: ProgressReporter | None = None
) -> dict[str, object]:
    if operation == "rust-test":
        from scripts.capabilities.rust_test_contracts import (  # noqa: PLC0415 -- selected request only.
            RustTestRequest,
        )
        from scripts.capabilities.rust_tests import (  # noqa: PLC0415 -- lazy operation owner.
            execute as run_tests,
        )

        return run_tests(
            ROOT, RustTestRequest.model_validate_json(document), progress=progress
        )
    gates, requested_output, capture, request = selected(operation, document)
    kinds = validation.preflight_kinds(gates)
    skipped = os.environ.get("PSE_PREFLIGHT") == "off"
    if kinds and not skipped:
        preflight.check(kinds, pse_env.compose(ROOT, os.environ))
    output = validation.fresh_output(ROOT, requested_output)
    previous = {
        name: signal.getsignal(name) for name in (signal.SIGTERM, signal.SIGHUP)
    }
    for name in previous:
        signal.signal(name, validation.interrupt)
    interrupted = False
    try:
        try:
            code = validation.run_gates(
                ROOT,
                output,
                gates,
                capture=capture,
                reuse_from=(ROOT / request.reuse_from)
                if request is not None and request.reuse_from is not None
                else None,
                reuse=tuple(request.reuse) if request is not None else (),
                transfer=tuple(request.transfer) if request is not None else (),
                change_reason=request.change_reason if request is not None else None,
                on_progress=(
                    lambda _stage, path: progress.publish(
                        "assessment",
                        "started",
                        reference=OwnerReference(kind="assessment", locator=str(path)),
                    )
                )
                if progress is not None
                else None,
            )
        except KeyboardInterrupt:
            # The assessment owns checkpoints, process groups and terminal
            # composition. A signal outside its gate wait retains that partial
            # receipt, exactly as the native CLI does.
            interrupted = True
            code = 128 + validation.RECEIVED_SIGNAL
    finally:
        for name, handler in previous.items():
            signal.signal(name, handler)
    path = output / "checks.json"
    from scripts import assessment_store, test_resources  # noqa: PLC0415

    receipt = {}
    summaries = {}
    failed = set()
    if path.is_file():
        with test_resources.borrow_report(output):
            image = assessment_store.root(output)
            summaries = {check["gate"]: check for check in image["checks"]}
            failed = {
                check["gate"]
                for check in image["checks"]
                if check["status"] in {"failed", "interrupted", "blocked"}
            }
            receipt = assessment_store.selected(output, image, gates=failed)
    checks = receipt.get("checks", [])
    if any(check.get("status") == "interrupted" for check in checks):
        code = 128 + validation.RECEIVED_SIGNAL
    result: dict[str, object] = {
        "operation": operation,
        "exit_code": code,
        "output": str(output),
        "receipt": str(path) if path.is_file() else None,
        "receipt_sha256": validation_receipts.digest(path) if path.is_file() else None,
        "reader": {"capability": "assessment_read", "directory": str(output)},
        "complete": receipt.get("complete", False),
        "required_checks_covered": receipt.get("required_checks_covered", False),
        "input_coverage": receipt.get("input_coverage", False),
        "source_unchanged": receipt.get("source_unchanged"),
        "provenance_errors": receipt.get("provenance_errors", []),
        "contextual_changes": {
            "status": "not_hydrated" if receipt else "unavailable",
            "source_cut_ref": receipt.get("source_cut_ref"),
            "reader": {
                "capability": "assessment_read",
                "directory": str(output),
                "source_detail": True,
            },
        },
        "scope": receipt.get("scope", []),
        "checks": [
            {
                **{
                    key: check.get(key)
                    for key in (
                        "gate",
                        "status",
                        "exit_code",
                        "evidence_kind",
                        "resource_invocation",
                        "process",
                    )
                },
                **{
                    key: check[key]
                    for key in (
                        "report_errors",
                        "spawn_error",
                        "dependencies",
                        "changed_source",
                        "changed_environment",
                    )
                    if key in check
                },
                "detail": summaries[check["gate"]]["detail"],
                "population": summaries[check["gate"]]["population"],
                "detail_status": "hydrated"
                if check["gate"] in failed
                else "not_hydrated",
            }
            for check in checks
        ],
        "interrupted_between_gates": interrupted,
        "preflight": {"required": kinds, "skipped_by_environment": skipped},
    }
    if operation == "documentation" and code == 0 and (ROOT / "docs/book").is_dir():
        result["site"] = str(ROOT / "docs/book")
    return result


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    if len(arguments) != 3 or arguments[0] not in {
        "assessment",
        "compilation",
        "contracts-check",
        "contracts-generate",
        "documentation",
        "rust-test",
    }:
        raise ValueError(
            "select one fixed qualification operation, request and response file"
        )
    operation, source, destination = arguments
    progress = ProgressReporter(Path(source).parent / "progress.json", operation)
    response: dict[str, object]
    try:
        progress.publish("request", "started")
        response = {
            "result": execute(operation, Path(source).read_text(), progress=progress)
        }
        progress.publish("request", "completed")
    except (
        OSError,
        ValueError,
        RuntimeError,
        subprocess.SubprocessError,
        pse_env.BoundaryError,
    ) as error:
        detail: dict[str, object] = {
            "code": "pse.qualification." + operation + ".refused",
            "cause": type(error).__name__,
            "stage": "qualification",
            "message": "The selected qualification request could not be served; supplied values are redacted.",
            "remedy": "Inspect the original assessment owner and any issued receipt before deciding whether another action is safe. No automatic retry or effect settlement is inferred.",
        }
        if isinstance(error, ValidationError):
            detail["code"] = "pse.qualification.input_refused"
            detail["message"] = "Invalid host selection; supplied values omitted."
            detail["stage"] = "input"
            detail["field_count"] = error.error_count()
            detail["fields"] = [
                {"loc": safe_path(list(item["loc"]), frozenset()), "type": item["type"]}
                for item in error.errors(include_input=False, include_context=False)[
                    :16
                ]
            ]
        elif isinstance(error, pse_env.BoundaryError):
            detail["code"] = "pse.qualification.prerequisite_unavailable"
            detail["exit_code"] = pse_env.FAILURE
            detail["stage"] = "preparation"
            detail["remedy"] = (
                "Explicitly inspect and prepare the required environment, then inspect any original owner receipt before repeating work. No automatic retry is inferred."
            )
        response = {"error": detail}
    with Path(destination).open("x") as output:
        json.dump(response, output)
    return 0


if __name__ == "__main__":
    from scripts.telemetry_process import run_helper

    raise SystemExit(run_helper("pse-qualification-worker", main))
