# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed private evidence dispatch through the existing independently built CLI."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pydantic import JsonValue

from scripts import evidence_store
from scripts.capabilities.execution_basis import AdmissionRefusedError, verify
from scripts.capabilities.native_contracts import EvidenceCompare
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet

ROOT = Path(__file__).resolve().parents[2]
MAXIMUM_REQUEST_BYTES = 16 << 20


def execute(
    arguments: dict[str, JsonValue], *, root: Path = ROOT
) -> dict[str, JsonValue]:
    """Keep supplied-only analysis out of the scientific Python extension and store."""
    supplied = dict(arguments)
    fingerprint = supplied.pop("schema_set_fingerprint", None)
    try:
        schemas = SchemaSet(root, family="evidence")
    except (OSError, ValueError):
        return {
            "outcome": "stale_interface",
            "code": "pse.evidence.contract_unavailable",
        }
    if fingerprint != schemas.fingerprint:
        return {
            "outcome": "stale_interface",
            "code": "pse.evidence.schema_set_mismatch",
        }
    try:
        # Consume the loaded owner's source checks before entering the native CLI.
        schemas.document("evidence-comparison-request")
        schemas.document("evidence-snapshot")
        schemas.document("evidence-comparison", "output")
    except (OSError, ValueError):
        return {
            "outcome": "stale_interface",
            "code": "pse.evidence.contract_unavailable",
        }
    request = EvidenceCompare.model_validate(supplied, strict=True)
    deadline = time.monotonic() + 30
    with tempfile.TemporaryDirectory(prefix="pse-evidence-comparison-") as temporary:
        directory = Path(temporary)
        request_path = directory / "request.json"
        observations_path = directory / "observations.json"
        request_path.write_text(json.dumps(request.request, allow_nan=False))
        observations_path.write_text(json.dumps(request.observations, allow_nan=False))
        command_arguments = [
            "--request",
            str(request_path),
            "--observations",
            str(observations_path),
        ]
        for index, snapshot in enumerate(request.snapshots):
            path = directory / f"snapshot-{index}.json"
            path.write_text(json.dumps(snapshot, allow_nan=False))
            command_arguments.extend(["--input", str(path)])
        with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output:
            try:
                code = evidence_store.run_offline(
                    command_arguments,
                    deadline=deadline,
                    operation="compare-offline",
                    stdout=output,
                )
            except (
                OSError,
                subprocess.TimeoutExpired,
                evidence_store.ExecutableCurrencyError,
            ):
                return {
                    "outcome": "prerequisite_unavailable",
                    "code": "pse.evidence.owner_unavailable",
                    "remedy": "Explicitly build the private evidence executable with just evidence-build; no build, service or source fallback was attempted.",
                }
            if code:
                return {
                    "outcome": "native_refusal",
                    "code": "pse.evidence.comparison_refused",
                    "exit_status": code,
                }
            output.seek(0)
            encoded = output.read((1 << 20) + 1)
        if len(encoded.encode()) > 1 << 20:
            return {"outcome": "detail_refused", "code": "pse.evidence.response_bound"}
        return {
            "outcome": "observed",
            "document": JSON_OBJECT.validate_json(encoded, strict=True),
        }


def admitted_arguments(raw: bytes) -> dict[str, JsonValue]:
    if len(raw) > MAXIMUM_REQUEST_BYTES:
        raise ValueError("private evidence request exceeds its byte allowance")
    verify(ROOT, "evidence", "evidence_compare", raw)
    envelope = JSON_OBJECT.validate_json(raw, strict=True)
    if (
        set(envelope) != {"operation", "arguments"}
        or envelope["operation"] != "evidence_compare"
    ):
        raise ValueError("unsupported private evidence operation")
    arguments = envelope["arguments"]
    if not isinstance(arguments, dict):
        raise TypeError("private evidence arguments must be an object")
    return arguments


def main() -> int:
    """Consume only the admitted fixed worker envelope; stdout contains its bounded answer."""
    raw = sys.stdin.buffer.read(MAXIMUM_REQUEST_BYTES + 1)
    try:
        arguments = admitted_arguments(raw)
        value = execute(arguments)
    except AdmissionRefusedError as error:
        value = {"outcome": "stale_interface", **error.detail()}
    except (ValueError, TypeError):
        value = {
            "outcome": "malformed_invocation",
            "code": "pse.evidence.input_refused",
        }
    print(json.dumps(value, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
