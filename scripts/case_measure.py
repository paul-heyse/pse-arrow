# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Criterion CSV collector: untimed build, fresh case process, separate pool and RSS."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import select
import signal
import statistics
import subprocess
import time
from pathlib import Path

from scripts import host_admission, scientific_inputs, validation, validation_receipts
from scripts.validation_scope import RUST_INPUTS

ROOT = Path(__file__).resolve().parents[1]
ENVIRONMENT_CONTROLS = (
    "PSE_SURREAL_STATE",
    "PSE_MEMORY_MAX",
    "PSE_RESOURCE_CLASS",
    "PSE_WORKER_BINARY",
    "CARGO_BUILD_JOBS",
    "RUSTFLAGS",
    "LD_LIBRARY_PATH",
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "OMP_PROC_BIND",
    "OMP_PLACES",
    "HWLOC_COMPONENTS",
)


def measurement_inputs(snapshot: dict) -> dict:
    paths = (
        *RUST_INPUTS,
        "benches",
        ".config/process-cases.json",
        ".config/preparation-cases.json",
        "scripts/case_measure.py",
    )
    return {
        name: value
        for name, value in snapshot.items()
        if any(
            name == prefix
            or name.startswith(prefix + "/")
            or (prefix.startswith("scripts/") and name.startswith(prefix))
            for prefix in paths
        )
    }


def build_benchmark(
    command: list[str], output: Path, phase: str
) -> subprocess.CompletedProcess[str]:
    """Retain Cargo JSON and surface rendered diagnostics before failing the collector."""
    build = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    stdout = build.stdout or ""
    stderr = build.stderr or ""
    json_log = output / f"{phase}-cargo.jsonl"
    stderr_log = output / f"{phase}-cargo.stderr.log"
    json_log.write_text(stdout)
    stderr_log.write_text(stderr)
    if build.returncode:
        diagnostics = []
        for line in stdout.splitlines():
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                diagnostics.append(line)
                continue
            if isinstance(row, dict) and row.get("reason") == "compiler-message":
                rendered = row.get("message", {}).get("rendered")
                if rendered:
                    diagnostics.append(rendered)
        error = subprocess.CalledProcessError(
            build.returncode, command, output=stdout, stderr=stderr
        )
        raise RuntimeError(
            f"Cargo benchmark build failed with exit {build.returncode}; "
            f"full output: {json_log}, {stderr_log}\n"
            + "\n".join(diagnostics)
            + "\n"
            + stderr
        ) from error
    return build


def stop_managed_primary(state: Path, database: str) -> None:
    """Cooperatively drain this fixture's exact receiver before removing its DB."""
    from scripts import (  # noqa: PLC0415 -- load supervisor policy only for managed drain
        surreal_server,
    )

    state = state.resolve()
    with surreal_server.state_lock(state):
        config = surreal_server.config_for(state)
        observed = surreal_server.primary_observation(state)
        group = observed.get("ControlGroup", "")
        if observed["ActiveState"] in {"inactive", "failed"} and (
            not group or not surreal_server.group_populated(group)
        ):
            return
        marker = surreal_server.read_json(state / "primary-receiver.json")
        if marker.get(
            "canonical_database"
        ) != database or not surreal_server.primary_ready(
            state, config, observed, database
        ):
            raise ValueError(
                "refuse to stop a primary not associated with this fixture"
            )
        pid = surreal_server.integer(marker["pid"])
        process = Path(f"/proc/{pid}")
        start = process.joinpath("stat").read_text().rsplit(")", 1)[1].split()[19]
        descriptor = os.pidfd_open(pid)
        try:
            current = surreal_server.read_json(state / "primary-receiver.json")
            if (
                current != marker
                or process.joinpath("stat").read_text().rsplit(")", 1)[1].split()[19]
                != start
                or not surreal_server.primary_ready(
                    state, config, surreal_server.primary_observation(state), database
                )
            ):
                raise ValueError("managed receiver association changed before drain")
            signal.pidfd_send_signal(descriptor, signal.SIGINT)
            poller = select.poll()
            poller.register(descriptor, select.POLLIN)
            deadline = time.monotonic() + 45
            while not poller.poll(100):
                if time.monotonic() >= deadline:
                    raise ValueError("managed receiver cooperative drain timed out")
        finally:
            os.close(descriptor)
        # The installation wrapper and service group must also release before
        # a later iteration can acquire the same finite primary allocation.
        while group and surreal_server.group_populated(group):
            if time.monotonic() >= deadline:
                raise ValueError(
                    "managed primary service did not release its allocation"
                )
            time.sleep(0.02)


def process_command(binary: str, mode: str, workload: dict) -> list[str]:
    """Compilation has finished before a managed observer enters its 4 GiB scope."""
    if workload.get("adapter") != "managed-durable":
        return [binary, mode]
    from scripts import (  # noqa: PLC0415 -- load supervisor policy only for managed adapter
        surreal_server,
    )

    state_value = os.environ.get("PSE_SURREAL_STATE")
    if not state_value:
        raise ValueError("managed measurement requires PSE_SURREAL_STATE")
    state = Path(state_value).resolve()
    config = surreal_server.config_for(state)
    if config["resources"] != surreal_server.reference_resources():
        raise ValueError("managed measurement requires the exact reference allocation")
    receiver = surreal_server.checked_primary(config)
    # The receiving supervisor checks placement, interpreter/worker bytes and
    # actual readiness; package.study then uses its production ensure-primary.
    return [
        receiver["supervisor_executable"],
        receiver["supervisor_script"],
        "observer",
        "--state",
        str(state),
        "--execution-profile",
        "plan28-reference",
        "--observer-command",
        binary,
        mode,
    ]


def managed_fixture_environment(workload: dict) -> dict[str, str]:
    """Register the generated managed fixture DB against its actual receiver."""
    if workload.get("adapter") != "managed-durable":
        return {}
    from scripts import surreal_server  # noqa: PLC0415 -- managed fixture registration

    state = Path(os.environ["PSE_SURREAL_STATE"]).resolve()
    receiver = surreal_server.checked_primary(surreal_server.config_for(state))
    return {"PSE_WORKER_BINARY": receiver["worker_executable"]}


def managed_receiver_conditions(workloads: list[dict]) -> dict | None:
    if not any(workload.get("adapter") == "managed-durable" for workload in workloads):
        return None
    from scripts import (  # noqa: PLC0415 -- load supervisor policy only for managed conditions
        surreal_server,
    )

    state = Path(os.environ["PSE_SURREAL_STATE"]).resolve()
    config = surreal_server.config_for(state)
    if config["resources"] != surreal_server.reference_resources():
        raise ValueError("managed measurement requires the exact reference allocation")
    receiver = surreal_server.checked_primary(config)
    return {
        "state": str(state),
        "resources": config["resources"],
        "deployment": {
            key: config[key]
            for key in ("instance_id", "server", "endpoint", "schema_interpretation")
        },
        "receiver": receiver,
        "metrics_scope": "benchmark counters/RSS belong to the observer; primary counters/RSS are unobserved",
    }


def verify_managed_receiver(receiver: dict | None) -> None:
    if receiver is None:
        return
    from scripts import (  # noqa: PLC0415 -- load supervisor policy only for managed verification
        surreal_server,
    )

    config = surreal_server.config_for(Path(receiver["state"]))
    if (
        config["resources"] != receiver["resources"]
        or {key: config[key] for key in receiver["deployment"]}
        != receiver["deployment"]
        or surreal_server.checked_primary(config) != receiver["receiver"]
    ):
        raise ValueError("managed measurement deployment changed during execution")


def samples(path: Path) -> dict:
    """Read Criterion's public CSV format; retain the original Criterion report."""
    values = []
    total_time = total_iterations = 0.0
    with path.open(newline="") as source:
        for row in csv.DictReader(source):
            elapsed, iterations = (
                float(row["sample_measured_value"]),
                float(row["iteration_count"]),
            )
            if row["unit"] != "ns" or not all(
                math.isfinite(v) and v > 0 for v in (elapsed, iterations)
            ):
                raise ValueError(
                    "Criterion samples require finite positive nanoseconds and iteration counts"
                )
            value = elapsed / iterations
            if not math.isfinite(value) or value <= 0:
                raise ValueError(
                    "Criterion per-iteration samples must be finite and positive"
                )
            values.append(value)
            total_time += elapsed
            total_iterations += iterations
    if len(values) < 2:
        raise ValueError("at least two Criterion samples are required")
    if not all(math.isfinite(value) for value in (total_time, total_iterations)):
        raise ValueError("Criterion sample totals must be finite")
    return {
        "mean_nanoseconds": total_time / total_iterations,
        "sample_count": len(values),
        "sample_mean_nanoseconds": statistics.mean(values),
        "sample_standard_error_nanoseconds": statistics.stdev(values)
        / math.sqrt(len(values)),
        "sample_min_nanoseconds": min(values),
        "sample_max_nanoseconds": max(values),
        "uncertainty": "sample standard error, not a bootstrap confidence interval",
    }


def invocation_population(directory: Path, memory: dict) -> dict:
    """Read actual invocation artifacts without assigning Criterion sample membership."""
    filename = memory.get("observations_file")
    if filename is not None:
        if not isinstance(filename, str):
            raise ValueError("invocation observations_file must be a relative path")
        path = (directory / filename).resolve()
        if not path.is_relative_to(directory.resolve()):
            raise ValueError("invocation observations_file escapes its case directory")
        with path.open() as source:
            records = [json.loads(line) for line in source if line.strip()]
        location = filename
    elif "records" in memory:
        records = memory["records"]
        location = "memory.json:records"
    else:
        return {
            "status": "unavailable",
            "reason": "case supplied no per-invocation artifact",
        }
    if not isinstance(records, list) or not all(
        isinstance(record, dict) for record in records
    ):
        raise ValueError("invocation artifact must contain object records")
    expected = memory.get("iterations")
    if expected is not None and (
        isinstance(expected, bool)
        or not isinstance(expected, int)
        or expected != len(records)
    ):
        raise ValueError(
            "invocation record count does not match actual executed iterations"
        )
    return {
        "status": "available",
        "source": location,
        "record_count": len(records),
        "population": "all executed harness invocations, including warmups when executed",
        "criterion_membership": {
            "status": "unavailable",
            "reason": "Criterion raw CSV supplies measured sample statistics, not invocation identities",
        },
        "join": "none; read this artifact separately from Criterion statistics",
    }


def measurement_environment() -> dict[str, str]:
    """Record execution controls without opening or authenticating producer receipts."""
    return {key: os.environ[key] for key in ENVIRONMENT_CONTROLS if key in os.environ}


def functional_reference(
    path: Path | None, *, snapshot: dict, environment: dict[str, str]
) -> dict | None:
    """Reference actual test outcomes without claiming equivalent execution conditions."""
    if path is None:
        return None
    report = path / "checks.json" if path.is_dir() else path
    evidence = json.loads(report.read_text())
    checks = evidence.get("checks", [])
    if not checks or evidence.get("complete") is not True:
        raise ValueError(
            "functional reference must contain completed test observations"
        )
    tests = []
    for check in checks:
        drift_diagnostic = "relevant inputs changed during assessment"
        attribution_only = bool(
            check.get("changed_source") or check.get("changed_environment")
        )
        report_errors = check.get("report_errors", [])
        terminal_errors = [
            error
            for error in report_errors
            if not (attribution_only and error == drift_diagnostic)
        ]
        if (
            check.get("exit_code") != 0
            or check.get("status") not in {"passed", "failed"}
            or (
                check.get("status") == "failed"
                and drift_diagnostic not in report_errors
            )
        ):
            raise ValueError(
                f"failed or unfinished functional reference: {check.get('gate')}"
            )
        if terminal_errors:
            raise ValueError(f"invalid functional test report: {check.get('gate')}")
        if not check.get("selected") and not check.get("results"):
            continue  # Setup commands are recorded, but are not test observations.
        terminal: dict = {
            **check,
            "status": "passed",
            "results": list(check.get("results", [])),
            "report_errors": [],
        }
        validation.compose_selection(terminal)
        if (
            not terminal["results"]
            or terminal["report_errors"]
            or any(result["status"] != "passed" for result in terminal["results"])
        ):
            raise ValueError(f"incomplete functional test results: {check.get('gate')}")
        tests.append(check)
    if not tests:
        raise ValueError("functional reference contains no executed tests")
    prior_sources = evidence.get("source_files")
    prior_environment = evidence.get("environment", {})
    return {
        "path": str(report.resolve()),
        "observed_test_status": "passed",
        "checks": tests,
        "scope": evidence.get("scope", []),
        "source_differences": (
            validation_receipts.changed(measurement_inputs(prior_sources), snapshot)
            if isinstance(prior_sources, dict)
            else None
        ),
        "environment_differences": {
            key: {
                "functional": prior_environment.get(key),
                "measurement": environment.get(key),
            }
            for key in ENVIRONMENT_CONTROLS
            if prior_environment.get(key) != environment.get(key)
        },
        "conditions_equivalent": "not established; reference only",
        "reported_provenance_errors": evidence.get("provenance_errors", []),
    }


def selected_workloads(
    declarations: tuple[dict, ...], selection: list[str]
) -> list[dict]:
    """Select declared identities exactly; no selector means the complete campaign."""
    workloads = [w for declaration in declarations for w in declaration["workloads"]]
    identities = [w["id"] for w in workloads]
    if len(set(identities)) != len(identities):
        raise ValueError("measurement workload identities must be unique")
    unknown = sorted(set(selection) - set(identities))
    if unknown:
        raise ValueError(f"unknown measurement cases: {', '.join(unknown)}")
    return [w for w in workloads if not selection or w["id"] in selection]


def smoke_campaign(
    output: Path,
    profile: str,
    process: dict,
    preparation: dict,
    selected: set[str],
    *,
    audit_order: list[str] | None = None,
    audit_deadline_seconds: float | None = None,
    review_artifacts: bool = False,
    capture_from_start: bool = False,
    compiled_evaluator: bool = False,
    recording_mode: str = "selected",
) -> None:
    """Execute benchmark controls once; this produces no performance qualification."""
    targets = {
        target
        for declaration, target in (
            (process, "native_process"),
            (preparation, "modeling_preparation"),
            ({"workloads": [{"id": "document-admission"}]}, "document_admission"),
        )
        if any(workload["id"] in selected for workload in declaration["workloads"])
    }
    environment = dict(os.environ)
    if "native_process" in targets:
        scientific_inputs.prepare_controller(
            output / "scientific-inputs.json", environment
        )
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--locked",
        "--profile",
        profile,
        "--features",
        "native-process,pse-relations/force-validate",
        "--no-run",
        "--message-format=json",
    ]
    if compiled_evaluator:
        command[command.index("--features") + 1] += ",pse-runtime/compiled-evaluator"
    for target in sorted(targets):
        command.extend(("--bench", target))
    # Every smoke executes correctness assertions. Statistical collection remains
    # separate, while all smoke routes validate their actual Arrow closure.
    from scripts import arrow_validation  # noqa: PLC0415

    command = arrow_validation.compose(command)
    build = build_benchmark(command, output, "smoke")
    binaries = {
        row["target"]["name"]: row["executable"]
        for line in build.stdout.splitlines()
        if line.startswith("{")
        for row in [json.loads(line)]
        if row.get("reason") == "compiler-artifact"
        and row.get("executable")
        and row.get("target", {}).get("name") in targets
    }
    if set(binaries) != targets:
        raise ValueError("all selected benchmark control executables must be built")
    receiver = managed_receiver_conditions(
        [workload for workload in process["workloads"] if workload["id"] in selected]
    )
    cases = [
        (workload, target)
        for declaration, target in (
            (process, "native_process"),
            (preparation, "modeling_preparation"),
            ({"workloads": [{"id": "document-admission"}]}, "document_admission"),
        )
        for workload in declaration["workloads"]
        if workload["id"] in selected
    ]
    if audit_order is not None:
        cases.sort(key=lambda case: audit_order.index(case[0]["id"]))
    results = []
    for workload, target in cases:
        name = workload["id"]
        if name not in selected:
            continue
        directory = output / name
        directory.mkdir()
        env = {
            **environment,
            "PSE_PROCESS_COST_CASE": name,
            "PSE_PROCESS_COST_SPEC": json.dumps(workload),
            "PSE_PROCESS_COST_OUTPUT": str(directory),
            "PSE_PREPARATION_CASE": name,
            "PSE_PREPARATION_OUTPUT": str(directory),
            "PSE_ADMISSION_OUTPUT": str(directory),
        }
        env.update(managed_fixture_environment(workload))
        if audit_order is not None:
            from scripts import case_audit  # noqa: PLC0415 -- opt-in native reader

            try:
                result = case_audit.run_case(
                    process_command(binaries[target], "--test", workload),
                    cwd=ROOT,
                    env=env,
                    directory=directory,
                    deadline_seconds=audit_deadline_seconds,
                    review_artifacts=review_artifacts,
                    capture_from_start=capture_from_start,
                    recording_mode=recording_mode,
                )
            except KeyboardInterrupt:
                observation = directory / "diagnostics/observation.json"
                if observation.exists():
                    results.append(
                        {"case": name, **json.loads(observation.read_text())}
                    )
                validation.write_json(
                    output / "case-audit.json",
                    {"cases": results, "measured": False, "campaign_interrupted": True},
                )
                raise
            results.append({"case": name, **result})
            validation.write_json(
                output / "case-audit.json", {"cases": results, "measured": False}
            )
            continue
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                process_command(binaries[target], "--test", workload),
                cwd=ROOT,
                env=env,
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
    verify_managed_receiver(receiver)
    validation.write_json(
        output / "case-smoke.json",
        {
            "schema": "case-smoke-v1",
            "measured": False,
            "selected_cases": sorted(selected),
            "managed_receiver": receiver,
            "profile": profile,
            "compiled_evaluator": compiled_evaluator,
            "review_artifacts": review_artifacts,
            "recording_mode": recording_mode,
            "capture_from_start": capture_from_start,
            "binaries": binaries,
        },
    )
    if any(result["status"] != "completed" for result in results):
        raise RuntimeError(
            f"audit contains failed or interrupted cases; see {output / 'case-audit.json'}"
        )


def preparation_campaign(
    output: Path, profile: str, workloads: list[dict], admission_selected: bool
) -> dict:
    """Build untimed, then isolate the selected preparation/admission workloads."""
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--bench",
        "modeling_preparation",
        "--bench",
        "document_admission",
        "--locked",
        "--profile",
        profile,
        "--features",
        "native-process,pse-relations/force-validate",
        "--no-run",
        "--message-format=json",
    ]
    build = build_benchmark(command, output, "preparation")
    binaries = {
        row["target"]["name"]: row["executable"]
        for line in build.stdout.splitlines()
        if line.startswith("{")
        for row in [json.loads(line)]
        if row.get("reason") == "compiler-artifact"
        and row.get("executable")
        and row.get("target", {}).get("name")
        in {"modeling_preparation", "document_admission"}
    }
    if set(binaries) != {"modeling_preparation", "document_admission"}:
        raise ValueError(
            "both preparation and admission benchmark artifacts are required"
        )
    cases = []
    for workload in workloads:
        name = workload["id"]
        directory = output / "preparation" / name
        directory.mkdir(parents=True, exist_ok=False)
        env = {
            **os.environ,
            "PSE_PREPARATION_CASE": name,
            "PSE_PREPARATION_OUTPUT": str(directory),
        }
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                [binaries["modeling_preparation"], "--bench"],
                cwd=ROOT,
                env=env,
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        cases.append(
            {
                **json.loads((directory / f"{name}.json").read_text()),
                **samples(
                    directory / "criterion/modeling_preparation" / name / "new/raw.csv"
                ),
            }
        )
    admission = None
    if admission_selected:
        directory = output / "admission"
        directory.mkdir(parents=True, exist_ok=False)
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                [binaries["document_admission"], "--bench"],
                cwd=ROOT,
                env={**os.environ, "PSE_ADMISSION_OUTPUT": str(directory)},
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        admission = json.loads((directory / "admission.json").read_text())
        admission["samples"] = {
            phase: samples(
                directory
                / "criterion/document_admission"
                / phase
                / "100000/new/raw.csv"
            )
            for phase in ("load_and_decode", "admit")
        }
    return {
        "profile": profile,
        "binaries": binaries,
        "cases": cases,
        "admission": admission,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, nargs="?")
    parser.add_argument(
        "--stop-managed-primary", metavar="DATABASE", help=argparse.SUPPRESS
    )
    parser.add_argument("--state", type=Path, help=argparse.SUPPRESS)
    parser.add_argument(
        "--functional-from",
        type=Path,
        help="Optional completed checks.json test reference; execution conditions may differ.",
    )
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="Run each selected control once without timing samples or a measurement receipt.",
    )
    parser.add_argument(
        "--audit-diagnostics",
        action="store_true",
        help="Observe explicitly selected once-only smoke cases, retaining live diagnostics and cooperative stop support.",
    )
    parser.add_argument(
        "--recording-mode",
        choices=("ordinary", "selected"),
        default="selected",
        help="Audit source recording selection; ordinary retains live audit without requesting source detail.",
    )
    parser.add_argument(
        "--review-artifacts",
        action="store_true",
        help="Retain original joined review context and bounded offline packs for audited cases.",
    )
    parser.add_argument(
        "--capture-from-start",
        action="store_true",
        help="Arm optional CPU sampling before command exec and timeline before case preparation.",
    )
    parser.add_argument(
        "--cargo-profile",
        help="Override the declaration's Cargo profile (for example profiling).",
    )
    parser.add_argument(
        "--compiled-evaluator",
        action="store_true",
        help="Build the compiled evaluator capability for explicitly selected smoke cases.",
    )
    parser.add_argument(
        "--audit-deadline-seconds",
        type=float,
        help="Request cooperative stop after this finite positive observation interval, then await native join and fixture cleanup; not a hard execution deadline.",
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        help="Declared process, preparation or document-admission case; repeat to select several. Defaults to all.",
    )
    args = parser.parse_args()
    if args.recording_mode == "ordinary" and (
        not args.audit_diagnostics or args.review_artifacts
    ):
        parser.error(
            "ordinary recording mode requires --audit-diagnostics and excludes --review-artifacts"
        )
    if (
        args.review_artifacts or args.capture_from_start
    ) and not args.audit_diagnostics:
        parser.error(
            "--review-artifacts and --capture-from-start require --audit-diagnostics"
        )
    if args.compiled_evaluator and (not args.smoke or not args.case):
        parser.error(
            "--compiled-evaluator requires --smoke and explicit --case selection"
        )
    if args.audit_deadline_seconds is not None and (
        not args.audit_diagnostics
        or not math.isfinite(args.audit_deadline_seconds)
        or args.audit_deadline_seconds <= 0
    ):
        parser.error(
            "--audit-deadline-seconds requires --audit-diagnostics and a finite positive interval"
        )
    if args.audit_diagnostics and (not args.smoke or not args.case):
        parser.error(
            "--audit-diagnostics requires --smoke and explicit --case selection"
        )
    if args.audit_diagnostics and len(set(args.case)) != len(args.case):
        parser.error("audit case selections must not repeat")
    if args.stop_managed_primary:
        if args.state is None or args.output is not None:
            parser.error("fixture drain requires --state and no output")
        stop_managed_primary(args.state, args.stop_managed_primary)
        return 0
    if args.output is None:
        parser.error("measurement output is required")
    declaration = json.loads((ROOT / ".config/process-cases.json").read_text())
    preparation = json.loads((ROOT / ".config/preparation-cases.json").read_text())
    admission = {
        "workloads": [{"id": "document-admission", "functional_scopes": ["admission"]}]
    }
    selection = selected_workloads((declaration, preparation, admission), args.case)
    selected = {w["id"] for w in selection}
    unsupported = {w["id"] for w in selection if w.get("audit_supported") is not True}
    if args.audit_diagnostics and unsupported:
        parser.error(
            "diagnostic audit stop/cleanup wiring is not supported for: "
            + ", ".join(sorted(unsupported))
        )
    if args.smoke:
        if args.functional_from:
            parser.error("smoke controls do not consume qualification receipts")
        output = validation.fresh_output(ROOT, args.output)
        smoke_campaign(
            output,
            args.cargo_profile or declaration["cargo_profile"],
            declaration,
            preparation,
            selected,
            audit_order=args.case if args.audit_diagnostics else None,
            audit_deadline_seconds=args.audit_deadline_seconds,
            review_artifacts=args.review_artifacts,
            capture_from_start=args.capture_from_start,
            compiled_evaluator=args.compiled_evaluator,
            recording_mode=args.recording_mode,
        )
        return 0
    snapshot = measurement_inputs(validation.sources(ROOT))
    environment_started = measurement_environment()
    functional = functional_reference(
        args.functional_from, snapshot=snapshot, environment=environment_started
    )
    output = validation.fresh_output(ROOT, args.output)
    environment = dict(os.environ)
    scientific_inputs.prepare_controller(output / "scientific-inputs.json", environment)
    started = time.time()
    host_started = host_admission.measurement_context()
    profile = {
        "cargo_profile": args.cargo_profile or declaration["cargo_profile"],
        "features": ["native-process", "pse-relations/force-validate"],
    }
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--bench",
        "native_process",
        "--locked",
        "--profile",
        profile["cargo_profile"],
        "--features",
        "native-process,pse-relations/force-validate",
        "--no-run",
        "--message-format=json",
    ]
    build = build_benchmark(command, output, "process")
    artifacts = [
        json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")
    ]
    binaries = {
        r["executable"]
        for r in artifacts
        if r.get("reason") == "compiler-artifact"
        and r.get("target", {}).get("name") == "native_process"
        and r.get("executable")
    }
    if len(binaries) != 1:
        raise ValueError("expected one compiled benchmark executable")
    binary = Path(binaries.pop())
    receiver = managed_receiver_conditions(selection)
    cases = []
    for workload in declaration["workloads"]:
        name = workload["id"]
        if name not in selected:
            continue
        directory = output / "process-cost" / name
        directory.mkdir(parents=True, exist_ok=False)
        env = {
            **environment,
            "PSE_PROCESS_COST_CASE": name,
            "PSE_PROCESS_COST_OUTPUT": str(directory),
            "PSE_PROCESS_COST_SPEC": json.dumps(workload),
        }
        env.update(managed_fixture_environment(workload))
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                process_command(str(binary), "--bench", workload),
                cwd=ROOT,
                env=env,
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        memory = json.loads((directory / f"{name}-memory.json").read_text())
        sample_summary = samples(directory / "criterion/process" / name / "new/raw.csv")
        files = {
            str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in directory.rglob("*")
            if p.is_file()
        }
        cases.append(
            {
                **memory,
                "criterion_statistics": sample_summary,
                "invocation_observations": invocation_population(directory, memory),
                "artifacts": files,
            }
        )
    report = {
        "schema": "process-cost-v5",
        "measured": True,
        "profile": profile,
        "functional": functional,
        "started": started,
        "host_conditions_start": host_started,
        "source_observations_start": snapshot,
        "environment_start": environment_started,
        "compilation_timed": False,
        "selected_cases": [w["id"] for w in selection],
        "binary_path": str(binary.resolve()),
        "managed_receiver": receiver,
        "thread_environment": {
            k: os.environ.get(k)
            for k in ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"]
        },
        "cases": cases,
    }
    preparation_report: dict = (
        preparation_campaign(
            output,
            profile["cargo_profile"],
            [w for w in preparation["workloads"] if w["id"] in selected],
            "document-admission" in selected,
        )
        if any(w["id"] in selected for w in preparation["workloads"])
        or "document-admission" in selected
        else {"cases": [], "admission": None, "not_selected": True}
    )
    report["thermodynamic_preparation"] = preparation_report
    report["host_conditions_end"] = host_admission.measurement_context()
    snapshot_end = measurement_inputs(validation.sources(ROOT))
    environment_end = measurement_environment()
    report["source_changes_during_measurement"] = validation_receipts.changed(
        snapshot, snapshot_end
    )
    report["environment_end"] = environment_end
    report["environment_changes_during_measurement"] = validation_receipts.changed(
        environment_started, environment_end
    )
    verify_managed_receiver(receiver)
    validation.write_json(output / "case-measure.json", report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
