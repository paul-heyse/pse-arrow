# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit G05/F03 inquiry over supplied owner snapshots and a native F01 request.

This deliberately invoked driver never builds, installs, starts a service or
exposes a production multi-question operation. Its two independent native calls
test compact/detail agreement; they do not claim one reused analysis context.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path

from scripts import evidence_comparison_report


def _supplied(path: Path, maximum: int) -> bytes:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > maximum:
        raise ValueError("select one bounded original file without a symlink")
    value = path.read_bytes()
    if len(value) > maximum:
        raise ValueError("selected original exceeds its byte bound")
    return value


def _expected(actual: object, expected: object, location: str = "comparison") -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise TypeError("unexpected native shape at " + location)
        for key, value in expected.items():
            if key not in actual:
                raise AssertionError("missing expected native field at " + location)
            _expected(actual[key], value, location + "." + str(key))
    elif actual != expected:
        raise AssertionError("independent expected value differs at " + location)


def run(
    request_path: Path,
    inputs: list[Path],
    expected_path: Path,
    output: Path,
    *,
    plots: bool = True,
) -> dict[str, object]:
    # Validate finite source/output selections before selecting any native owner.
    if not 1 <= len(inputs) <= 64:
        raise ValueError("select 1..64 explicit original snapshots")
    request_raw = _supplied(request_path, 4 << 20)
    supplied = [_supplied(path, 4 << 20) for path in inputs]
    snapshots = b"[" + b",".join(supplied) + b"]"
    if len(snapshots) > 16 << 20:
        raise ValueError("supplied snapshot population exceeds native boundary")
    expected = json.loads(_supplied(expected_path, 16 << 10))
    if not isinstance(expected, dict) or not expected:
        raise ValueError("supply nonempty independent expected comparison fields")
    if output.exists():
        raise ValueError("select a new inquiry output directory")
    from msgspec.structs import replace  # noqa: PLC0415 -- selected inquiry only.

    import msgspec  # noqa: PLC0415 -- private generated contract consumer.
    from scripts.capabilities import evidence_worker  # noqa: PLC0415
    from scripts.capabilities.native_schema import SchemaSet  # noqa: PLC0415
    from scripts.generated.evidence_comparison_request_input import EvidenceComparisonRequest  # noqa: PLC0415
    from scripts.generated.evidence_comparison_output import EvidenceComparison  # noqa: PLC0415

    request = msgspec.json.decode(request_raw, type=EvidenceComparisonRequest)
    fingerprint = SchemaSet(evidence_worker.ROOT, family="evidence").fingerprint
    def compare(supplied: bytes) -> EvidenceComparison:
        result = evidence_worker.execute({"schema_set_fingerprint":fingerprint,
            "request":json.loads(supplied),"snapshots":json.loads(snapshots),"observations":[]})
        if result.get("outcome") != "observed":
            raise RuntimeError("original private evidence CLI refused the inquiry")
        return msgspec.json.decode(msgspec.json.encode(result["document"]), type=EvidenceComparison)

    detailed = compare(request_raw)
    detailed_raw = msgspec.json.encode(detailed)
    _expected(json.loads(detailed_raw), expected)
    compact_request = replace(request, sections={}, continuation_cut=None)
    compact = compare(msgspec.json.encode(compact_request))
    compact_raw = msgspec.json.encode(compact)
    _expected(json.loads(compact_raw), expected)
    if (
        detailed.cut != compact.cut
        or detailed.summary != compact.summary
        or detailed.baseline_denominator != compact.baseline_denominator
        or detailed.candidate_denominator != compact.candidate_denominator
    ):
        raise AssertionError("compact and detailed original native populations differ")
    output.mkdir(mode=0o700, parents=True)
    (output / "comparison.json").write_bytes(detailed_raw)
    (output / "compact.json").write_bytes(compact_raw)
    report = evidence_comparison_report.write_report(
        output / "comparison.json", output / "report", plots=plots
    )
    result: dict[str, object] = {
        "schema": "plan52-evidence-inquiry-v1",
        "native_calls": 2,
        "original_cut": detailed.cut,
        "independent_expectations": "matched supplied expected fields for detailed and compact native answers",
        "sources": [
            {"path": str(path.absolute()), "sha256": hashlib.sha256(raw).hexdigest()}
            for path, raw in zip(inputs, supplied, strict=True)
        ],
        "request": {
            "path": str(request_path.absolute()),
            "sha256": hashlib.sha256(request_raw).hexdigest(),
        },
        "report": report,
        "matplotlib": importlib.metadata.version("matplotlib")
        if plots
        else "not selected",
        "G05": "Headless typed report and standalone plots evaluated; operator use and interactive host rendering remain unestablished. Ordinary explicit parameters and original result/source links suffice for this bounded inspection. Revisit interaction when concrete filtering or navigation remains unmet.",
        "F03": "Production multi-question delivery remains unselected. These independent calls establish no shared-context reuse or speed benefit. Read the separately executed native cfg(test) control for single-analysis sequential feasibility; a concrete multi-question consumer and operator selection remain required.",
        "not_executed": [
            "indexed store",
            "canonical host UI rendering",
            "operator usability session",
            "production multi-question operation",
            "statistical or causal analysis",
        ],
    }
    with (output / "inquiry.json").open("x") as destination:
        json.dump(result, destination, indent=2)
        destination.write("\n")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--request", type=Path, required=True)
    parser.add_argument("--input", type=Path, action="append", required=True)
    parser.add_argument("--expected", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--no-plots", action="store_true")
    args = parser.parse_args()
    print(
        json.dumps(
            run(
                args.request,
                args.input,
                args.expected,
                args.output,
                plots=not args.no_plots,
            ),
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
