# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only inspection of explicitly paired historical measurement reports."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import statistics
import sys
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def positive(value: object) -> float:
    if (
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(value)
        or value <= 0
    ):
        raise ValueError("measurement sample must be finite and positive")
    return float(value)


def source_identity(files: dict) -> str:
    return hashlib.sha256(
        json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def retain_sample(membership: set[tuple[str, str]], sha256: str, sample: str) -> None:
    key = sha256, sample
    if key in membership:
        raise ValueError("sample membership is reused across pairs")
    membership.add(key)


def measurement(reference: dict, base: Path) -> tuple[float, dict, dict]:
    """Project exact producer facts; declarations cannot fill missing observations.

    Comparison conditions must equal this projection. Source is the one supported
    treatment: each side separately declares its exact source inventory identity.
    """
    path = (base / reference["path"]).resolve()
    report = json.loads(path.read_text())
    sample = reference["sample"]
    pressure = {
        key: report.get(key) for key in ("host_conditions_start", "host_conditions_end")
    }
    association = {
        "path": str(path),
        "sha256": digest(path),
        "sample": sample,
        "pressure": pressure,
    }
    if "builds" in report:
        item = report["builds"][sample]
        if (
            item.get("exit_code") != 0
            or report.get("original_source_unchanged") is not True
        ):
            raise ValueError("build sample is incomplete or source changed")
        directory = path.parent / sample
        command_path, source_path, result_path = (
            directory / name
            for name in ("command.json", "source-files.json", "result.json")
        )
        command = json.loads(command_path.read_text())
        files = json.loads(source_path.read_text())
        if json.loads(result_path.read_text()) != item:
            raise ValueError(
                "named build invocation differs from its summary membership"
            )
        environment = command["environment"]
        if (
            not isinstance(environment, dict)
            or not isinstance(command["command"], list)
            or not isinstance(files, dict)
            or not files
        ):
            raise ValueError("build conditions or source inventory unavailable")
        native_names = {
            "IPOPT_DIR",
            "SUITESPARSE_LIBRARY_DIR",
            "LIBCLANG_PATH",
            "BINDGEN_EXTRA_CLANG_ARGS",
        }
        cache_names = {
            "RUSTC_WRAPPER",
            "SCCACHE_CACHE_SIZE",
            "SCCACHE_MAX_FRAME_LENGTH",
            "PSE_NATIVE_COMPILER_CACHE",
        }
        paths = {"CARGO_TARGET_DIR", "SCCACHE_DIR", "SCCACHE_SERVER_UDS"}
        environment = {
            key: value for key, value in environment.items() if value is not None
        }
        phase = (
            sample.rsplit("-", 1)[0] if sample.rsplit("-", 1)[-1].isdigit() else sample
        )
        actual = {
            "input": {
                key: value
                for key, value in files.items()
                if key.startswith("tests/fixtures/")
            },
            "start": {
                "phase": phase,
                "correctness_execution": report.get("correctness_execution"),
            },
            "cache": {
                "state": report.get("cache_state"),
                "environment": {
                    key: value
                    for key, value in environment.items()
                    if key in cache_names
                },
            },
            "build": {
                "command": command["command"],
                "environment": {
                    key: value
                    for key, value in environment.items()
                    if key not in native_names | cache_names | paths
                },
                "toolchain": files.get("rust-toolchain.toml"),
            },
            "native": {
                key: value for key, value in environment.items() if key in native_names
            },
        }
        if (
            actual["start"]["correctness_execution"] is None
            or actual["cache"]["state"] is None
            or actual["build"]["toolchain"] is None
        ):
            raise ValueError("build start/cache/toolchain conditions unavailable")
        association.update(
            source_identity=source_identity(files),
            population="one named build invocation",
            artifacts={
                str(item): digest(item)
                for item in (command_path, source_path, result_path)
            },
        )
        return positive(item.get("wall_seconds")), actual, association
    if report.get("schema") == "process-cost-v5":
        if (
            report.get("measured") is not True
            or report.get("source_changes_during_measurement") != []
            or report.get("environment_changes_during_measurement") != []
        ):
            raise ValueError(
                "case measurement is incomplete or changed during execution"
            )
        cases = [case for case in report["cases"] if case.get("id") == sample]
        if len(cases) != 1:
            raise ValueError("case reference must select exactly one actual population")
        item = cases[0]
        stats = item["criterion_statistics"]
        if stats.get("sample_count", 0) < 2:
            raise ValueError("case has no complete measured sample population")
        files = report["source_observations_start"]
        environment = report["environment_start"]
        if (
            not isinstance(files, dict)
            or not files
            or not isinstance(environment, dict)
            or environment != report.get("environment_end")
        ):
            raise ValueError("case source or environment conditions unavailable")
        native_environment = {
            key: value
            for key, value in environment.items()
            if key.endswith("_NUM_THREADS")
            or key
            in {
                "LD_LIBRARY_PATH",
                "IPOPT_DIR",
                "OMP_PROC_BIND",
                "OMP_PLACES",
                "HWLOC_COMPONENTS",
            }
        }
        service = (
            pressure["host_conditions_start"].get("service")
            if isinstance(pressure["host_conditions_start"], dict)
            else None
        )
        actual = {
            "input": item.get("workload"),
            "start": {
                "sampling": item.get("sampling"),
                "setup": item.get("setup"),
                "timed_scope": item.get("timed_scope"),
                "untimed_scope": item.get("untimed_scope"),
            },
            "cache": service.get("filesystem_cache")
            if isinstance(service, dict)
            else None,
            "build": {
                "profile": report.get("profile"),
                "compilation_timed": report.get("compilation_timed"),
                "environment": {
                    key: value
                    for key, value in environment.items()
                    if key not in native_environment
                },
            },
            "native": {
                "environment": native_environment,
                "threads": item.get("threads"),
                "native_threads": item.get("native_threads"),
                "receiver": report.get("managed_receiver"),
            },
        }
        if (
            actual["input"] is None
            or any(value is None for value in actual["start"].values())
            or actual["build"]["profile"] is None
            or actual["build"]["compilation_timed"] is None
        ):
            raise ValueError("case input/start/build conditions unavailable")
        value = positive(stats.get("mean_nanoseconds"))
        association.update(
            source_identity=source_identity(files),
            population="Criterion measured samples only",
            statistics=stats,
            criterion_invocation_join="unavailable; harness invocations are not measured sample identities",
        )
        return value, actual, association
    raise ValueError("unsupported measurement owner")


def compare(path: Path, max_relative_spread: float) -> dict:
    declaration = json.loads(path.read_text())
    pairs = declaration["pairs"]
    conditions = declaration.get("conditions")
    treatment = declaration.get("treatment")
    required = {"input", "start", "cache", "build", "native"}
    result: dict = {
        "status": "inconclusive",
        "pairs": [],
        "reasons": [],
        "conditions": conditions,
        "treatment": treatment,
    }
    if not isinstance(conditions, dict) or set(conditions) != required:
        result["reasons"].append(
            "declare exact observed input/start/cache/build/native conditions"
        )
    if (
        not isinstance(treatment, dict)
        or treatment.get("dimension") != "source"
        or not all(
            isinstance(treatment.get(side), str) for side in ("baseline", "candidate")
        )
    ):
        result["reasons"].append(
            "declare baseline/candidate source inventory identities as the treatment"
        )
    if len(pairs) < 3:
        result["reasons"].append(
            "at least three independent declared pairs are required"
        )
    membership = set()
    ratios = []
    all_conditions = []
    for pair in pairs:
        if pair.get("order") not in (
            ["baseline", "candidate"],
            ["candidate", "baseline"],
        ):
            result["reasons"].append("each pair must declare execution order")
        record = {"order": pair.get("order")}
        values = {}
        try:
            for side in ("baseline", "candidate"):
                value, actual, reference = measurement(pair[side], path.parent)
                retain_sample(membership, reference["sha256"], reference["sample"])
                values[side] = value
                all_conditions.append(actual)
                record[side] = {
                    **reference,
                    "value": value,
                    "actual_conditions": actual,
                }
                if actual != conditions:
                    result["reasons"].append(
                        "declared conditions differ from actual producer observations"
                    )
                if (
                    not isinstance(treatment, dict)
                    or treatment.get(side) != reference["source_identity"]
                ):
                    result["reasons"].append(
                        "source treatment differs from actual producer inventory"
                    )
                if actual["cache"] in (None, "uncontrolled", "unknown"):
                    result["reasons"].append(
                        "cache conditions are unobserved or uncontrolled"
                    )
                stats = reference.get("statistics")
                if isinstance(stats, dict):
                    standard_error = stats.get("sample_standard_error_nanoseconds")
                    mean = stats.get("sample_mean_nanoseconds")
                    if standard_error is None or mean is None:
                        result["reasons"].append(
                            "within-population sample uncertainty unavailable"
                        )
                    elif (
                        not isinstance(standard_error, (int, float))
                        or not math.isfinite(standard_error)
                        or standard_error < 0
                        or standard_error / positive(mean) > max_relative_spread
                    ):
                        result["reasons"].append("within-population samples are noisy")
                pressure = reference["pressure"]
                if not all(
                    isinstance(pressure.get(key), dict)
                    and "competing_cooperating_owners" in pressure[key]
                    and "external_processes" in pressure[key]
                    for key in ("host_conditions_start", "host_conditions_end")
                ):
                    result["reasons"].append("D04 pressure observations unavailable")
                for observation in pressure.values():
                    if isinstance(observation, dict) and observation.get(
                        "competing_cooperating_owners"
                    ):
                        result["reasons"].append(
                            "cooperating workload pressure was observed"
                        )
            ratios.append(values["candidate"] / values["baseline"])
        except (OSError, ValueError, TypeError, KeyError) as error:
            result["reasons"].append(str(error))
        result["pairs"].append(record)
    if all_conditions and any(item != all_conditions[0] for item in all_conditions):
        result["reasons"].append("actual measurement conditions are incompatible")
    if ratios:
        median = statistics.median(ratios)
        spread = (max(ratios) - min(ratios)) / median
        result.update(
            candidate_over_baseline_median=median,
            relative_ratio_spread=spread,
            max_relative_spread=max_relative_spread,
        )
        if spread > max_relative_spread:
            result["reasons"].append("paired ratios are noisy")
    if not result["reasons"] and len(ratios) == len(pairs):
        result["status"] = "descriptive"
    result["reasons"] = list(dict.fromkeys(result["reasons"]))
    result["interpretation"] = (
        "descriptive paired observations; no automatic winner or statistical significance claim"
    )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    paired = commands.add_parser("compare")
    paired.add_argument("--pairs", type=Path, required=True)
    paired.add_argument("--output", type=Path, required=True)
    paired.add_argument("--max-relative-spread", type=float, default=0.1)
    args = parser.parse_args(argv)
    try:
        if not math.isfinite(args.max_relative_spread) or args.max_relative_spread < 0:
            parser.error("max relative spread must be finite and nonnegative")
        result = compare(args.pairs, args.max_relative_spread)
        with args.output.open("x") as destination:
            json.dump(result, destination, indent=2)
            destination.write("\n")
        code = 0 if result["status"] == "descriptive" else 125
        print(json.dumps(result, indent=2))
    except (
        OSError,
        ValueError,
        TypeError,
        KeyError,
    ) as error:
        print(f"regression-experiments: {error}", file=sys.stderr)
        return 128
    else:
        return code


if __name__ == "__main__":
    raise SystemExit(main())
