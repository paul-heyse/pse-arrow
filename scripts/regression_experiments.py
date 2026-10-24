# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite historical regression searches and explicitly paired measurement inspection."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import statistics
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Never

if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scripts import build_measurements, native_operation, validation

ROOT = Path(__file__).resolve().parents[1]
PREDICATE = "authoring-load-package"
TEST_SOURCE = Path("crates/pse-admission/tests/authoring_load_package.rs")
TEST_NAME = (
    "explicit_and_named_fixtures_decode_generated_rows_and_original_source_spans"
)
BINARY = "pse-admission::authoring_load_package"
SUBJECT = BINARY + "::" + TEST_NAME


def _invalid_protocol(message: str) -> Never:
    """Keep this owner's document/observation refusals in its ValueError contract."""
    raise ValueError(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=root, text=True).strip()


def run_owned(command: list[str], cwd: Path, log: Path) -> tuple[int, dict]:
    """Borrow placement and physical drain from the existing workload owner."""
    observation: dict = {}
    with log.open("w") as stream:
        code = native_operation.run_child_scope(
            command,
            dict(os.environ),
            cwd=cwd,
            observation=observation,
            stdout=stream.fileno(),
            stderr=subprocess.STDOUT,
        )
    return code, observation


def classify_product(result: dict) -> int:
    """Only the declared test's ordinary assertion failure is the bad predicate."""
    terminal = result.get("terminal")
    process = result.get("process", {})
    rows = result.get("report_rows", [])
    if (
        result.get("phase") != "frozen"
        or not isinstance(terminal, dict)
        or terminal.get("complete") is not True
        or result.get("report_errors")
        or result.get("selected_count") != 1
        or result.get("selected_unreported_count") != 0
        or not isinstance(process, dict)
        or process.get("forwarded_signal") is not None
        or len(rows) != 1
        or rows[0].get("class") != BINARY
        or rows[0].get("name") != TEST_NAME
    ):
        return 128
    row = rows[0]
    if row.get("status") == "passed" and result.get("workload_exit") == 0:
        return 0
    if result.get("setup_failures") or row.get("failure_kind") == "infrastructure":
        return 128  # Setup failure is not evidence that this revision is bad.
    if (
        row.get("status") == "failure"
        and result.get("workload_exit") in {1, 100}
        and result.get("predicate_execution", {}).get("failure_type")
        == "test failure with exit code 101"
    ):
        return 1
    return 128


def predicate_execution(result: dict, report: Path) -> dict:
    """Read nextest's structured failure variant from the authenticated JUnit.

    nextest 0.9.146, aggregator/junit.rs non_success_kind_and_type distinguishes
    ExitCode(101) from abort, timeout and leaked handles. Do not search log text.
    The terminal owner's recorded digest binds this small adapter observation.
    """
    expected = result.get("artifacts", {}).get(report.name)
    if expected != digest(report):
        raise ValueError("predicate report differs from the frozen product artifact")
    tree = ET.parse(report)  # noqa: S314 -- authenticated locally generated report
    cases = list(tree.iter("testcase"))
    if (
        len(cases) != 1
        or cases[0].get("classname") != BINARY
        or cases[0].get("name") != TEST_NAME
    ):
        raise ValueError(
            "predicate report does not contain exactly the declared subject"
        )
    case = cases[0]
    if any(
        case.findall(tag)
        for tag in ("rerunFailure", "flakyFailure", "rerunError", "flakyError")
    ):
        raise ValueError("predicate requires one attempt per bounded trial")
    failures = case.findall("failure")
    if len(failures) > 1:
        raise ValueError("ambiguous predicate failure variants")
    return {
        "report": str(report),
        "sha256": expected,
        "failure_type": failures[0].get("type") if failures else None,
    }


def rust_trial(checkout: Path, output: Path) -> dict:
    """Adapt the one supported pure Rust consumer to preparation/result owners.

    This controller is imported from outside the candidate checkout. Historical
    code supplies the selected product, never the predicate or runner adapter.
    Native/schema journeys need their own supported adapter, not this pure route.
    """
    from scripts import (  # noqa: PLC0415 -- selected operation owner; source-only discovery must not prepare this lifecycle
        arrow_validation,
        native_tests,
        run_inspection,
        test_resources,
        test_run,
        validation_receipts,
    )

    native_tests.ROOT = checkout
    test_run.ROOT = checkout
    provenance = output / "native.json"
    selection = output / "selected.json"
    environment = arrow_validation.rust_test_environment(os.environ)
    # No target override: this checkout owns its ordinary Cargo target directory.
    if "CARGO_TARGET_DIR" in environment:
        raise ValueError("historical checkout refuses an inherited target override")
    for key in (
        test_resources.MARKER,
        "PSE_NATIVE_PROVENANCE",
        test_run.RUST_OBSERVER,
        "PSE_WORKER_BINARY",
        "PSE_SURREAL_STATE",
        "PSE_TEST_EXECUTION_PROFILE",
    ):
        environment.pop(key, None)
    command = [
        "cargo",
        "nextest",
        "run",
        "--locked",
        "-p",
        "pse-admission",
        "--test",
        "authoring_load_package",
        "--no-fail-fast",
        "--retries",
        "0",
        "-E",
        f"test(={TEST_NAME})",
    ]
    execution, _worker = native_tests.ordinary_rust_capture(
        command,
        provenance,
        selection,
        environment,
        effects="native-local",
    )
    inventory = validation_receipts.native_inventory(selection.read_text())
    selected = test_run.rust_selected(inventory)
    if [(item["binary_id"], item["name"]) for item in selected] != [
        (BINARY, TEST_NAME)
    ]:
        raise ValueError("historical selection differs from the exact predicate")
    environment["PSE_NATIVE_PROVENANCE"] = str(provenance)
    environment[test_run.EXECUTION_EFFECTS] = "native-local"
    # Verify the same prepared closure before composing the existing invocation.
    native_tests.prepared_rust_inventory(execution, environment, inventory=inventory)
    path = test_resources.invocation("rust", selected)
    command, report = test_run.rust_report_command(
        path, ["cargo", "nextest", "run", *execution]
    )
    test_run.execute_selected(
        path,
        command,
        environment,
        report,
        selected=[{"class": BINARY, "name": TEST_NAME}],
        details={"preparation": str(provenance)},
    )
    result = run_inspection.product_result(str(path))
    result["predicate_execution"] = predicate_execution(result, report)
    return result


def evaluate(checkout: Path, output: Path, expected: str, repetitions: int) -> int:
    """Git's 0/good, 1/bad, 125/untestable, 128+/abort protocol."""
    output.mkdir(parents=True, exist_ok=False)
    outcome: dict = {
        "commit": git(checkout, "rev-parse", "HEAD"),
        "predicate": PREDICATE,
        "predicate_source_sha256": expected,
        "trials": [],
        "code": 128,
    }
    try:
        source = checkout / TEST_SOURCE
        if not source.is_file() or digest(source) != expected:
            outcome.update(
                code=125, reason="historical predicate source is unsupported"
            )
            return 125
        for repetition in range(repetitions):
            trial = output / str(repetition)
            trial.mkdir()
            result = rust_trial(checkout, trial)
            code = classify_product(result)
            outcome["trials"].append(
                {
                    "code": code,
                    "result": {
                        key: result.get(key)
                        for key in (
                            "kind",
                            "registry",
                            "invocation",
                            "run_path",
                            "terminal_identity",
                        )
                    },
                    "subject": SUBJECT,
                    "predicate_execution": result.get("predicate_execution"),
                }
            )
            if code >= 128:
                outcome.update(code=code, reason="incomplete or infrastructure result")
                return code
        codes = {trial["code"] for trial in outcome["trials"]}
        code = codes.pop() if len(codes) == 1 else 125
        outcome.update(
            code=code,
            reason="consistent trials"
            if code != 125
            else "mixed trials are inconclusive",
        )
    except (
        OSError,
        ValueError,
        TypeError,
        KeyError,
        ET.ParseError,
        subprocess.SubprocessError,
    ) as error:
        outcome.update(
            code=128, reason=f"infrastructure: {type(error).__name__}: {error}"
        )
        return 128
    else:
        return code
    finally:
        validation.write_json(output / "predicate.json", outcome)


def freeze_controller(output: Path) -> Path:
    directory = output / "controller"
    directory.mkdir()
    return build_measurements.snapshot(ROOT, directory)


def remove_owned(path: Path, identity: tuple[int, int]) -> None:
    stat = path.lstat()
    if (
        path.is_symlink()
        or (stat.st_dev, stat.st_ino) != identity
        or stat.st_uid != os.getuid()
    ):
        raise ValueError("owned checkout identity changed; cleanup refused")
    shutil.rmtree(path)


def bisect(args: argparse.Namespace) -> tuple[int, dict]:
    source = args.repo.resolve()
    good = git(source, "rev-parse", "--verify", args.good + "^{commit}")
    bad = git(source, "rev-parse", "--verify", args.bad + "^{commit}")
    if good != args.good or bad != args.bad:
        raise ValueError("good and bad must be full exact commit IDs")
    if good == bad:
        raise ValueError("good and bad must differ")
    subprocess.run(
        ["git", "merge-base", "--is-ancestor", good, bad], cwd=source, check=True
    )
    output = args.output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    result: dict = {
        "predicate": PREDICATE,
        "good": good,
        "bad": bad,
        "repetitions": args.repetitions,
        "status": "aborted",
        "code": 128,
        "premise": "monotone behavior of the exact selected predicate; a narrowed commit is a causal lead",
        "processes": [],
        "cleanup": "retained",
        "predicate_results": [],
    }
    checkout = output / "checkout"
    checkout_identity = None
    drained = True

    def execute(command: list[str], cwd: Path, log: Path) -> int:
        nonlocal drained
        previous = drained
        drained = False  # Exceptions cannot authorize cleanup with an older witness.
        code, observation = run_owned(command, cwd, log)
        result["processes"].append(observation)
        drained = previous and observation.get("owned_drain") == "complete"
        return code

    try:
        controller = freeze_controller(output)
        expected = digest(controller / TEST_SOURCE)
        result["controller"] = str(controller)
        result["predicate_source_sha256"] = expected
        result["controller_source"] = str(output / "controller/original-source.json")
        code = execute(
            [
                "git",
                "clone",
                "--quiet",
                "--no-hardlinks",
                "--no-checkout",
                str(source),
                str(checkout),
            ],
            output,
            output / "clone.log",
        )
        if code or not drained:
            result["code"] = max(code, 128)
            return result["code"], result
        stat = checkout.stat()
        checkout_identity = stat.st_dev, stat.st_ino
        trials = output / "predicates"
        trials.mkdir()
        launcher = [
            sys.executable,
            str(controller / "scripts/regression_experiments.py"),
            "_evaluate",
            "--checkout",
            str(checkout),
            "--output",
            str(trials),
            "--expected",
            expected,
            "--repetitions",
            str(args.repetitions),
        ]
        # Verify endpoint declarations before trusting them as Git facts.
        for name, revision, expected_code in (("good", good, 0), ("bad", bad, 1)):
            git(checkout, "checkout", "--quiet", "--detach", revision)
            code = execute(launcher, checkout, output / f"endpoint-{name}.log")
            if not drained:
                result.update(code=128, reason="endpoint physical drain unconfirmed")
                return 128, result
            if code != expected_code:
                result.update(
                    code=code if code >= 128 else 125,
                    status="aborted" if code >= 128 else "inconclusive",
                    reason=f"{name} endpoint did not establish its declared predicate",
                )
                return result["code"], result
        git(checkout, "bisect", "start", bad, good)
        code = execute(
            ["git", "bisect", "run", *launcher],
            checkout,
            output / "bisect.log",
        )
        results = [
            json.loads(path.read_text())
            for path in sorted(trials.glob("*/predicate.json"))
        ]
        result["predicate_results"] = [
            str(path) for path in sorted(trials.glob("*/predicate.json"))
        ]
        result["git_log"] = git(checkout, "bisect", "log")
        aborted = (
            any(item["code"] >= 128 for item in results) or code >= 128 or not drained
        )
        if not aborted and code == 0:
            result.update(
                status="narrowed",
                code=0,
                causal_lead=git(checkout, "rev-parse", "refs/bisect/bad"),
            )
        elif not aborted and any(item["code"] == 125 for item in results):
            goods = git(
                checkout, "for-each-ref", "--format=%(objectname)", "refs/bisect/good-*"
            ).splitlines()
            candidates = git(
                checkout, "rev-list", "refs/bisect/bad", "--not", *goods
            ).splitlines()
            result.update(status="inconclusive", code=125, possible_commits=candidates)
        else:
            result.update(status="aborted", code=max(code, 128))
    except (
        OSError,
        ValueError,
        TypeError,
        KeyError,
        subprocess.SubprocessError,
    ) as error:
        result.update(
            status="aborted",
            code=128,
            reason=f"infrastructure: {type(error).__name__}: {error}",
        )
        return 128, result
    else:
        return result["code"], result
    finally:
        if checkout_identity is not None and args.cleanup and drained:
            try:
                remove_owned(checkout, checkout_identity)
                result["cleanup"] = "owned checkout removed after complete drain"
            except (OSError, ValueError) as error:
                result["cleanup"] = f"retained: {error}"
        elif args.cleanup and not drained:
            result["cleanup"] = "retained: physical drain unconfirmed"
        if (output / "predicates").is_dir():
            result["predicate_results"] = [
                str(path)
                for path in sorted((output / "predicates").glob("*/predicate.json"))
            ]
        validation.write_json(output / "bisect.json", result)


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
                key = reference["sha256"], reference["sample"]
                if key in membership:
                    _invalid_protocol("sample membership is reused across pairs")
                membership.add(key)
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
    search = commands.add_parser("bisect")
    search.add_argument("--repo", type=Path, default=ROOT)
    search.add_argument("--good", required=True)
    search.add_argument("--bad", required=True)
    search.add_argument("--predicate", choices=(PREDICATE,), required=True)
    search.add_argument("--output", type=Path, required=True)
    search.add_argument("--repetitions", type=int, default=1)
    search.add_argument("--cleanup", action="store_true")
    paired = commands.add_parser("compare")
    paired.add_argument("--pairs", type=Path, required=True)
    paired.add_argument("--output", type=Path, required=True)
    paired.add_argument("--max-relative-spread", type=float, default=0.1)
    internal = commands.add_parser("_evaluate", help=argparse.SUPPRESS)
    internal.add_argument("--checkout", type=Path, required=True)
    internal.add_argument("--output", type=Path, required=True)
    internal.add_argument("--expected", required=True)
    internal.add_argument("--repetitions", type=int, required=True)
    args = parser.parse_args(argv)
    if args.command in {"bisect", "_evaluate"} and not 1 <= args.repetitions <= 20:
        parser.error("repetitions must be between 1 and 20")
    try:
        if args.command == "_evaluate":
            return evaluate(
                args.checkout,
                args.output / str(time.time_ns()),
                args.expected,
                args.repetitions,
            )
        if args.command == "bisect":
            code, result = bisect(args)
        else:
            if (
                not math.isfinite(args.max_relative_spread)
                or args.max_relative_spread < 0
            ):
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
        subprocess.SubprocessError,
    ) as error:
        print(f"regression-experiments: {error}", file=sys.stderr)
        return 128
    else:
        return code


if __name__ == "__main__":
    raise SystemExit(main())
