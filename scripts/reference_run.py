# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed reference lifetime: setup, admitted start/init/work, then exact drained stop."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile

import msgspec
from pathlib import Path

from scripts import host_admission, native_operation, surreal_server, test_resources

ROOT = Path(__file__).resolve().parents[1]


def checkpoint(invocation: Path, value: dict) -> None:
    declaration = test_resources.read_invocation(invocation)
    with test_resources.storage().invocation_edit(declaration["nonce"]) as state:
        state["lifecycle"] = value


def phase(command: str, state: Path, *extra: str) -> int:
    arguments = surreal_server.parser().parse_args(
        [command, "--state", str(state), *extra]
    )
    return surreal_server.dispatch(arguments)


def freeze_initializer(source: Path, invocation: Path) -> Path:
    """Capture the selected feature binary before another Cargo producer replaces its alias."""
    target = invocation.with_suffix(".initializer")
    descriptor = os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    with os.fdopen(descriptor, "rb") as original, target.open("xb") as copied:
        before = os.fstat(original.fileno())
        shutil.copyfileobj(original, copied, 1 << 20)
        copied.flush()
        os.fsync(copied.fileno())
        after = os.fstat(original.fileno())
        if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
        ):
            raise ValueError("canonical initializer changed during artifact capture")
    target.chmod(0o500)
    from scripts.generated import deployment  # noqa: PLC0415 -- product initializer protocol

    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        probe = subprocess.run([str(target), "--describe"], stdout=output, stderr=errors,
                               timeout=10, check=False)
        output.seek(0)
        description = output.read(64 * 1024 + 1)
    try:
        if probe.returncode != 0 or len(description) > 64 * 1024 or "version" not in json.loads(description):
            raise ValueError("initializer description unavailable")
        report = msgspec.json.decode(description, type=deployment.CanonicalInitializationReport)
        if (not report.canonical_initialize
                or report.phase is not deployment.CanonicalInitializationPhase.DESCRIPTION):
            raise ValueError("initializer did not describe the selected operation")
    except (ValueError, TypeError) as error:
        raise ValueError("captured initializer does not expose the product initialization protocol; rebuild pse-canonical-init") from error
    return target


def lifetime(
    state: Path,
    invocation: Path,
    command: list[str],
    initializer: Path,
    *,
    setup_result: dict | None = None,
) -> dict:
    owner = host_admission.inherit(os.environ)
    if owner is None or owner.profile.name != "reference":
        raise ValueError(
            "reference lifetime requires its authentic reference allocation"
        )
    value = {
        **dict(setup_result or {}),
        "invocation": str(invocation),
        "state": str(state),
        "allocation": owner.nonce,
        "setup": "complete",
        "start": "not_run",
        "initialize": "not_run",
        "workload": "not_run",
        "signal": None,
        "drain": "not_run",
        "cleanup": "not_run",
        "normal_resume": "pending owner release",
        "command": command,
    }
    config = surreal_server.config_for(state)
    selected: dict[str, str] | None = None
    instance = config["instance_id"]

    def run_selected() -> None:
        nonlocal selected, config
        if (
            setup_result is not None
            and isinstance(setup_result.get("initializer"), dict)
            and test_resources.file_digest(initializer)
            != setup_result["initializer"].get("sha256")
        ):
            raise ValueError(
                "captured initializer bytes changed; initialization refused"
            )
        # Serialize only start/selection. Workloads remain free to use their
        # existing context owners; finalization reacquires and rechecks identity.
        with surreal_server.lifecycle_reservation(state, purpose="startup"):
            observed = surreal_server.service_readiness(state, config)
            if observed.phase is not surreal_server.Readiness.ACTIONABLE_ABSENT:
                raise ValueError(
                    "reference-run requires an observed stopped owned service"
                )
            if config.get("admission") == "validation_required" or config.get(
                "derived_rebuild_pending"
            ):
                value["initialize"] = phase(
                    "initialize",
                    state,
                    "--initializer-command",
                    str(initializer),
                    str(state),
                )
                if value["initialize"] != 0:
                    return
                config = surreal_server.config_for(state)
            else:
                value["initialize"] = "already admitted"
            surreal_server.start(state, config)
            selected = surreal_server.storage_unit_observation(state)
            value["start"] = "complete"
        checkpoint(invocation, value)
        observation: dict[str, object] = {}
        environment = dict(os.environ)
        receiver = surreal_server.config_for(state).get("primary_receiver")
        if isinstance(receiver, dict) and isinstance(
            receiver.get("worker_executable"), str
        ):
            environment["PSE_WORKER_BINARY"] = receiver["worker_executable"]
            value["worker_executable"] = receiver["worker_executable"]
        value["workload"] = native_operation.run_child_scope(
            command, environment, cwd=ROOT, observation=observation
        )
        value["process"] = observation
        value["signal"] = observation.get("forwarded_signal")
        value["drain"] = observation.get("owned_drain", "unobserved")

    try:
        run_selected()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        value["error"] = f"{type(error).__name__}: {error}"
    finally:
        if selected is not None:

            def cleanup_selected() -> None:
                with surreal_server.lifecycle_reservation(state, purpose="stop"):
                    current = surreal_server.config_for(state)
                    actual = surreal_server.storage_unit_observation(state)
                    if current["instance_id"] != instance or actual.get(
                        "InvocationID"
                    ) != selected.get("InvocationID"):
                        raise ValueError(
                            "selected service generation changed; exact cleanup refused"
                        )
                    value["quiesce"] = phase("quiesce", state)
                    if value["quiesce"] != 0:
                        raise ValueError("selected service did not quiesce")
                    value["cleanup"] = phase("stop", state, "--drained")
                    if value["cleanup"] != 0:
                        raise ValueError("selected drained stop failed")
                    value["service_drain"] = "complete"

            try:
                cleanup_selected()
            except (
                OSError,
                ValueError,
                RuntimeError,
                subprocess.SubprocessError,
            ) as error:
                value["cleanup"] = "unresolved"
                value["cleanup_error"] = f"{type(error).__name__}: {error}"
        checkpoint(invocation, value)
    return value


def execute(
    state: Path,
    worker: Path,
    command: list[str],
    *,
    profile: str = "reference",
    tooling_fixture: bool = False,
    initializer: Path | None = None,
) -> dict:
    if not command or profile != "reference":
        raise ValueError("select reference and a literal workload argv")
    if host_admission.MARKER in os.environ:
        raise ValueError(
            "run reference-run from outside an allocation; it owns its setup and reference phases"
        )
    state, worker = state.absolute(), worker.resolve(strict=True)
    invocation = test_resources.invocation("reference", [])
    value: dict = {
        "invocation": str(invocation),
        "setup": "not_run",
        "workload": "not_run",
        "drain": "not_run",
        "cleanup": "not_run",
        "normal_resume": "not_run",
    }
    checkpoint(invocation, value)
    environment = {**os.environ, "PSE_SURREAL_STATE": str(state)}
    boundary = [str(ROOT / "scripts/pse-env")]
    value["deployment_build"] = (
        "explicit prebuilt deployment operator" if environment.get("PSE_DEPLOYMENT_OPERATOR")
        else subprocess.call([
            *boundary, "--native=", "--", sys.executable, "-m", "scripts.arrow_validation", "build",
            "-p", "pse-runtime", "--bin", "pse-deployment", "--no-default-features", "--locked",
        ], env=environment, cwd=ROOT)
    )
    if isinstance(value["deployment_build"], int) and value["deployment_build"] != 0:
        value["setup"] = "deployment build failed"
        checkpoint(invocation, value)
        return value
    # Build/setup effects precede the finite scientific allocation. The explicit
    # worker is already built; canonical schema tooling is prepared here.
    build = [
        *boundary,
        "--native=",
        "--",
        sys.executable,
        "-m",
        "scripts.arrow_validation",
        "build",
        "-p",
        "pse-operations",
        "--bin",
        "pse-canonical-init",
        "--no-default-features",
        "--locked",
    ]
    value["build"] = (
        subprocess.call(build, env=environment, cwd=ROOT)
        if initializer is None
        else "explicit prebuilt initializer"
    )
    if isinstance(value["build"], int) and value["build"] != 0:
        value["setup"] = "build failed"
        checkpoint(invocation, value)
        return value
    try:
        initializer = freeze_initializer(
            initializer or ROOT / "target/debug/pse-canonical-init", invocation
        )
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        value["setup"] = f"initializer unavailable: {error}"
        checkpoint(invocation, value)
        return value
    value["initializer"] = {
        "path": str(initializer),
        "sha256": test_resources.file_digest(initializer),
    }
    setup = [
        *boundary,
        "--native=" + ",".join(surreal_server.WORKER_CAPABILITIES),
        "--recipe-default-class",
        "light",
        "--",
        sys.executable,
        "-m",
        "scripts.surreal_server",
        "setup",
        "--state",
        str(state),
        "--execution-profile",
        profile,
        "--worker-executable",
        str(worker),
    ]
    if tooling_fixture:
        setup.append("--tooling-fixture")
    value["setup"] = subprocess.call(setup, env=environment, cwd=ROOT)
    checkpoint(invocation, value)
    if value["setup"]:
        return value
    run = [
        *boundary,
        "--native=",
        "--resource-class",
        "reference",
        "--",
        sys.executable,
        "-m",
        "scripts.reference_run",
        "_lifetime",
        "--state",
        str(state),
        "--invocation",
        str(invocation),
        "--initializer",
        str(initializer),
        "--",
        *command,
    ]
    value["execution_boundary"] = subprocess.call(run, env=environment, cwd=ROOT)
    state_image = test_resources.storage().invocation_snapshot(invocation.stem)
    value = {
        **state_image.get("lifecycle", value),
        "execution_boundary": value["execution_boundary"],
    }
    # The boundary owns exact allocation drain/release. Reconciliation uses the
    # existing parked-service intent and may honestly refuse or defer resumption.
    try:
        value["normal_resume"] = host_admission.status(services=surreal_server.ServiceAdmission())
    except (OSError, ValueError, RuntimeError) as error:
        value["normal_resume"] = {"status": "unavailable", "error": str(error)}
    checkpoint(invocation, value)
    return value


def main(argv: list[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    internal = arguments[:1] == ["_lifetime"]
    if internal:
        arguments.pop(0)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--worker", type=Path, required=not internal)
    parser.add_argument(
        "--profile", choices=("reference",), default="reference"
    )
    parser.add_argument("--tooling-fixture", action="store_true")
    parser.add_argument("--invocation", type=Path, required=internal)
    parser.add_argument(
        "--initializer",
        type=Path,
        required=internal,
        help="selected prebuilt product canonical initializer; otherwise build it before reference admission",
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(arguments)
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("a literal workload command after -- is required")
    try:
        value = (
            lifetime(
                args.state,
                args.invocation,
                command,
                args.initializer,
                setup_result=test_resources.storage()
                .invocation_snapshot(args.invocation.stem)
                .get("lifecycle"),
            )
            if internal
            else execute(
                args.state,
                args.worker,
                command,
                profile=args.profile,
                tooling_fixture=args.tooling_fixture,
                initializer=args.initializer,
            )
        )
    except (OSError, ValueError, RuntimeError) as error:
        print(f"reference-run: {error}", file=sys.stderr)
        return 1
    print(json.dumps(value, indent=2))
    print(f"Run handle: just result {value['invocation']} --kind product-test")
    return 0 if value.get("workload") == 0 and value.get("cleanup") == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
