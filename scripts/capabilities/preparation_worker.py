# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed preparation/import child; pse-env has already resolved its one environment."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import python_extension
from scripts.capabilities.execution_basis import AdmissionRefused
from scripts.capabilities.preparation import (
    CurrencyCheckRequest,
    ImportCheckRequest,
    ImportCheckResult,
    StageInvocation,
)

if TYPE_CHECKING:
    from collections.abc import Callable

    from pydantic import JsonValue


def invoke(
    operation: str,
    document: str,
    *,
    on_stage: Callable[[str, python_extension.StageResult | None], None] | None = None,
) -> dict[str, JsonValue]:
    if operation == "stage":
        request = StageInvocation.model_validate_json(document)
        result = python_extension.run_stages(
            request.stage,
            request.mode,
            request.profile,
            python_extension.bind_interpreter(os.environ),
            validated=request.validated,
            on_stage=on_stage,
        )
        return result.model_dump(mode="json")
    if operation == "import-check":
        request = ImportCheckRequest.model_validate_json(document)
        if on_stage is not None:
            on_stage("import", None)
        try:
            artifact = python_extension.ExtensionArtifact.model_validate(
                # Native read dispatch uses this same fixed checkout interpreter,
                # rather than a caller's alternate installer environment.
                python_extension.capture(
                    python_extension.bind_interpreter(
                        {**os.environ, "UV_PROJECT_ENVIRONMENT": sys.prefix}
                    ),
                    validated=request.validated,
                    native=request.mode == "native",
                )
            )
        except (OSError, ValueError, TypeError, subprocess.SubprocessError) as error:
            result = ImportCheckResult(
                mode=request.mode,
                validated=request.validated,
                compatible=False,
                cause=type(error).__name__,
            )
        else:
            result = ImportCheckResult(
                mode=request.mode,
                validated=request.validated,
                compatible=True,
                artifact=artifact,
            )
        if on_stage is not None:
            on_stage(
                "import",
                python_extension.StageResult(
                    stage="admit",
                    status="admitted" if result.compatible else "failed",
                    exit_code=0 if result.compatible else 1,
                    artifact=result.artifact,
                ),
            )
        return result.model_dump(mode="json")
    if operation == "currency-check":
        request = CurrencyCheckRequest.model_validate_json(document)
        if on_stage is not None:
            on_stage("currency", None)
        currency = python_extension.currency_check(
            python_extension.bind_interpreter(
                {**os.environ, "UV_PROJECT_ENVIRONMENT": sys.prefix}
            ),
            mode=request.mode,
            profile=request.profile,
            validated=request.validated,
        )
        if on_stage is not None:
            on_stage(
                "currency",
                python_extension.StageResult(
                    stage="admit",
                    status="checked" if currency.state == "current" else "failed",
                    exit_code=0 if currency.state == "current" else 1,
                    artifact=currency.artifact,
                ),
            )
        return currency.model_dump(mode="json")
    raise ValueError("unsupported fixed preparation operation")


def main(argv: list[str] | None = None) -> int:
    from scripts.capabilities.worker_progress import ProgressReporter  # noqa: PLC0415

    arguments = sys.argv[1:] if argv is None else argv
    if len(arguments) != 3 or arguments[0] not in {"stage", "import-check", "currency-check"}:
        raise ValueError(
            "select one fixed preparation operation, request file and response file"
        )
    operation, source, destination = arguments
    reporter = ProgressReporter(Path(source).parent / "progress.json", operation)
    active_stage: str | None = None

    def observe(stage: str, result: python_extension.StageResult | None) -> None:
        nonlocal active_stage
        active_stage = stage
        reporter.publish(
            stage,
            "started"
            if result is None
            else "failed"
            if result.exit_code
            else "completed",
        )

    try:
        from scripts.capabilities.execution_basis import verify, ENVIRONMENT, Handoff
        supplied = os.environ.get(ENVIRONMENT)
        if supplied:
            identity = Handoff.model_validate_json(Path(supplied).read_bytes())
            verify(Path(__file__).resolve().parents[2], identity.worker, operation, Path(source).read_bytes())
        response = {
            "result": invoke(operation, Path(source).read_text(), on_stage=observe)
        }
    except (
        OSError,
        ValueError,
        TypeError,
        RuntimeError,
        subprocess.SubprocessError,
    ) as error:
        if active_stage is not None:
            reporter.publish(active_stage, "failed")
        response = {
            "error": {
                "code": "pse.preparation.refused",
                "stage": "preparation",
                "cause": type(error).__name__,
                "message": "The selected preparation could not return a completed owner result; supplied values are redacted.",
                "remedy": "Inspect the original stage owner and existing artifacts before explicitly selecting further preparation. No retry, rollback or successful installation is inferred.",
            }
        }
        if isinstance(error, AdmissionRefused):
            response = {"error": error.detail()}
    with Path(destination).open("x") as stream:
        json.dump(response, stream)
    return 0


if __name__ == "__main__":
    from scripts.telemetry_process import run_helper

    raise SystemExit(run_helper("pse-preparation-worker", main))
