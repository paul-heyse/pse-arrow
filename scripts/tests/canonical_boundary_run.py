# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit operator evidence and owned same-handle server restart controls.

Run operator through scripts/pse-env --native and restart through --native=solver.
The restart owner selects native inputs only for its Rust preparation/child;
source-server installation retains the caller's ordinary build environment.
Both routes retain the existing selected test and
resource/result owners. Restart creates a new nonresident deployment and never
performs lifecycle operations on the caller's selected service.
"""

from __future__ import annotations

import argparse
import os
import socket
import subprocess
import sys
import time
import uuid
from pathlib import Path

from scripts import native_operation, native_tests, reference_run, test_resources, test_run
from scripts import surreal_server as server
from scripts import validation_receipts

ROOT = Path(__file__).resolve().parents[2]
RESTART_TEST = (
    "canonical_execution::canonical_execution_server_unit::"
    "retained_canonical_store_recovers_a_new_generation_after_external_server_restart"
)
OPERATOR_FILTER = "test(canonical_operator_graph_evidence) | test(canonical_operator_query_evidence)"


def rust_command(selection: str) -> list[str]:
    return [
        "cargo", "nextest", "run", "-p", "pse-operations", "-p", "pse-relations",
        "--lib", "--features",
        "pse-operations/canonical-tests,pse-relations/force-validate",
        "--run-ignored", "only", "-E", selection, "--no-fail-fast",
    ]


def wait_record(
    directory: Path, name: str, process: subprocess.Popen[bytes], deadline: float
) -> dict[str, object]:
    while not (directory / name).is_file():
        if process.poll() is not None:
            raise RuntimeError(f"Restart peer exited before {name}: {process.returncode}")
        if time.monotonic() >= deadline:
            raise TimeoutError(f"Restart peer did not publish {name}")
        time.sleep(0.02)
    return server.read_json(directory / name)


def restart_handshake(
    state: Path, control: Path, process: subprocess.Popen[bytes], deadline: float
) -> None:
    """Only the freshly owned profile may be stopped; peer ACKs bound each phase."""
    ready = wait_record(control, "ready.json", process, deadline)
    if Path(str(ready.get("state"))).resolve() != state.resolve():
        raise RuntimeError("Restart peer selected another service")
    database = ready.get("database")
    if not isinstance(database, str) or not test_resources.DATABASE.fullmatch(database):
        raise RuntimeError("Restart peer did not select its isolated fixture")
    original = native_operation.unit_observation(server.unit_name(state))
    if original.get("ActiveState") != "active" or not original.get("InvocationID"):
        raise RuntimeError("Owned restart service has no active invocation")
    server.stop(state, server.config_for(state), abrupt=True)
    if server.active(state):
        raise RuntimeError("Owned restart service did not stop")
    server.write_json(control / "stopped.json", {"database": database})
    down = wait_record(control, "down-observed.json", process, deadline)
    if down.get("database") != database or not down.get("error"):
        raise RuntimeError("Restart peer did not observe the actual closed service")
    server.start(state, server.config_for(state), deadline=deadline)
    restarted = native_operation.unit_observation(server.unit_name(state))
    if (
        restarted.get("ActiveState") != "active"
        or not restarted.get("InvocationID")
        or restarted["InvocationID"] == original["InvocationID"]
    ):
        raise RuntimeError("Owned restart did not produce a fresh server invocation")
    server.write_json(control / "restarted.json", {"database": database})
    complete = wait_record(control, "complete.json", process, deadline)
    if (
        complete.get("database") != database
        or complete.get("same_store") is not True
        or complete.get("same_attempt") is not True
        or not isinstance(complete.get("next_sequence"), int)
    ):
        raise RuntimeError("Restart peer did not establish the retained-handle capability")


def rust_child(execution: Path) -> int:
    prepared = server.read_json(execution)
    provenance = Path(str(prepared["provenance"]))
    environment = dict(os.environ)
    environment.pop(test_resources.MARKER, None)
    environment["PSE_NATIVE_PROVENANCE"] = str(provenance)
    environment["PSE_WORKER_BINARY"] = str(prepared["worker"])
    environment[test_run.RUST_OBSERVER] = "1"
    environment.update(native_tests.rust_fixture_bindings(provenance, environment))
    inventory = validation_receipts.native_inventory(
        Path(str(prepared["selected"])).read_text()
    )
    arguments = prepared["arguments"]
    if not isinstance(arguments, list) or any(not isinstance(item, str) for item in arguments):
        raise TypeError("Invalid prepared restart selection")
    invocation = test_resources.invocation("rust", test_run.rust_selected(inventory))
    environment[test_resources.MARKER] = str(invocation)
    environment["PSE_RUST_TERMINAL_OWNER"] = "runner"
    code = test_run.run_rust(
        ["cargo", "nextest", "run", *arguments],
        env=environment,
        inventory=inventory,
    )
    terminal = invocation.with_suffix(".terminal.json")
    # Observation only: the existing terminal owner remains the result authority.
    server.write_json(execution.with_name("child-invocation.json"), {
        "parent": prepared["parent_invocation"], "child": str(invocation),
        "terminal": str(terminal), "terminal_sha256": server.file_digest(terminal),
    })
    return code


def restart_journey(directory: Path, test_identity: str) -> None:
    """Prepare first, then create a current-design disposable service and peer."""
    if directory.exists() or directory.is_symlink():
        raise ValueError("Restart requires a fresh owned directory")
    test_resources.require_effect({"test": test_identity}, effect="canonical")
    test_resources.require_effect({"test": test_identity}, effect="native")
    directory.mkdir(mode=0o700, parents=True)
    initializer = reference_run.freeze_initializer(
        Path(os.environ.get("PSE_CANONICAL_TOOLS", str(ROOT / "target/debug/xtask"))),
        directory / "tools.json",
    )
    state = directory / "live"
    control = state / "restart-control"
    environment = native_operation.environment(
        ["solver", "klu", "isolation", "uno", "petsc"], dict(os.environ),
    )
    environment["PSE_CANONICAL_TOOLS"] = str(initializer)
    environment["PSE_CANONICAL_RESTART_CONTROL"] = str(control)
    arguments, worker = native_tests.ordinary_rust_capture(
        rust_command("test(=" + RESTART_TEST + ")"),
        directory / "native.json", directory / "selected.json", environment,
    )
    if worker is None:
        raise ValueError("Restart requires its captured canonical fixture worker")
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    args = server.parser().parse_args([
        "setup", "--state", str(state), "--port", str(port), "--no-resident",
        "--interpretation", server.SUBSTRATE_INTERPRETATION,
        "--memory-mib", "2560", "--server-memory-mib", "2048",
        "--native-workers", "1", "--native-worker-memory-mib", "512",
    ])
    # setup refuses an existing profile; only this fresh path gains stop authority.
    server.setup(args)
    resource: str | None = None
    process: subprocess.Popen[bytes] | None = None
    try:
        server.initialize(state, [str(initializer), "canonical-init", str(state)])
        server.start(state, server.config_for(state))
        resource = test_resources.register({
            "state": str(state), "database": "canonical_test_" + uuid.uuid4().hex,
            "kind": "controls", "directory": str(control), "test": test_identity,
        })
        control.mkdir(mode=0o700)
        execution = directory / "execution.json"
        server.write_json(execution, {
            "arguments": arguments, "worker": str(worker),
            "provenance": str(directory / "native.json"),
            "selected": str(directory / "selected.json"),
            "parent_invocation": os.environ[test_resources.MARKER],
        })
        environment["PSE_SURREAL_STATE"] = str(state)
        environment.pop(test_resources.MARKER, None)
        with (directory / "rust.log").open("xb") as log:
            process = subprocess.Popen(
                [sys.executable, "-m", "scripts.tests.canonical_boundary_run", "rust-child", str(execution)],
                cwd=ROOT, env=environment, stdout=log, stderr=subprocess.STDOUT,
            )
            deadline = time.monotonic() + 360
            restart_handshake(state, control, process, deadline)
            code = process.wait(timeout=max(0.001, deadline - time.monotonic()))
            if code:
                raise RuntimeError(f"Selected restart test failed: {code}; see {directory / 'rust.log'}")
            peer = server.read_json(directory / "child-invocation.json")
            if peer.get("parent") != os.environ[test_resources.MARKER]:
                raise RuntimeError("Restart child lost its parent invocation association")
            child = test_resources.read_invocation(Path(str(peer["child"])))
            if child["kind"] != "rust":
                raise RuntimeError("Restart peer did not execute a distinct Rust invocation")
            terminal = Path(str(peer["terminal"]))
            digest = server.file_digest(terminal)
            if peer.get("terminal_sha256") != digest:
                raise RuntimeError("Restart child terminal observation changed")
            test_resources.observe_peer({"test": test_identity}, Path(str(peer["child"])), role="external-restart")
    finally:
        cleanup_failures: list[str] = []
        original_error = sys.exception()
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
        # Supervisor stop authenticates the exact owned unit and proves drain.
        # Retain profile/evidence for the invocation owner; never remove unknown state.
        if resource is not None:
            try:
                test_resources.record_drain(resource)
            except Exception as error:
                cleanup_failures.append("peer-controls-drain")
                print(f"Restart controls retained: {error}", file=sys.stderr)
        try:
            server.stop(state, server.config_for(state))
        except Exception as error:
            cleanup_failures.append("peer-service-stop")
            print(f"Restart service cleanup failed: {error}", file=sys.stderr)
        try:
            test_resources.observe_cleanup({"test": test_identity}, cleanup_failures)
        except Exception as error:
            if original_error is not None:
                original_error.add_note(f"Cleanup observation unavailable: {error}")
            else:
                raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("operator", "restart", "rust-child"))
    parser.add_argument("directory", type=Path)
    parser.add_argument("--executor", type=Path)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if args.action == "rust-child":
        return rust_child(directory)
    if args.action == "operator":
        environment = dict(os.environ)
        environment["PSE_TEST_REQUIRED_ROLES"] = ",".join(filter(None, [environment.get("PSE_TEST_REQUIRED_ROLES", ""), "operator-graph", "operator-query"]))
        environment["PSE_DIAGNOSTIC_EVIDENCE"] = str(directory)
        if args.executor is not None:
            environment["PSE_CANONICAL_TOOLS"] = str(args.executor.resolve())
        directory.mkdir(mode=0o700, parents=True, exist_ok=False)
        source = environment.get("PSE_CANONICAL_TOOLS")
        if not source:
            raise ValueError("Selected operator evidence requires PSE_CANONICAL_TOOLS")
        environment["PSE_CANONICAL_TOOLS"] = str(reference_run.freeze_initializer(
            Path(source), directory / "tools.json",
        ))
        return test_run.run_rust(rust_command(OPERATOR_FILTER), env=environment)
    environment = dict(os.environ)
    environment["PSE_TEST_REQUIRED_ROLES"] = ",".join(filter(None, [environment.get("PSE_TEST_REQUIRED_ROLES", ""), "external-restart"]))
    environment["PSE_CANONICAL_RESTART_DIRECTORY"] = str(directory)
    if args.executor is not None:
        environment["PSE_CANONICAL_TOOLS"] = str(args.executor.resolve())
    return test_run.run_python([
        sys.executable, "-m", "pytest",
        str(ROOT / "scripts/tests/test_canonical_restart_peer.py"),
        "-m", "component", "-n", "0", "-q",
    ], env=environment)


if __name__ == "__main__":
    raise SystemExit(main())
