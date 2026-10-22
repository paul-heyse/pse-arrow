# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Join existing nextest/pytest selection and terminal composition to fixture owners."""

from __future__ import annotations

import json
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import arrow_validation, validation, validation_receipts
from scripts import test_resources as resources

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
RUST_OBSERVER = "PSE_RUST_OBSERVER_CHILD"
EXECUTION_EFFECTS = "PSE_TEST_EXECUTION_EFFECTS"


def cancellation_signal(value: object) -> int | None:
    if value is not None and not isinstance(value, int):
        raise TypeError("Invalid original process cancellation signal")
    return value


def rust_selected(inventory: Mapping[str, object]) -> list[dict[str, str]]:
    suites = inventory["rust-suites"]
    if not isinstance(suites, dict):
        raise TypeError("Invalid nextest suite inventory")
    selected = []
    for suite in suites.values():
        if not isinstance(suite, dict) or not isinstance(suite.get("testcases"), dict):
            raise TypeError("Invalid nextest test inventory")
        for name, test in suite["testcases"].items():
            if not isinstance(test, dict) or not isinstance(
                test.get("filter-match"), dict
            ):
                raise TypeError("Invalid nextest selected test")
            if test["filter-match"]["status"] == "matches":
                identity, executable = suite["binary-id"], suite["binary-path"]
                if not all(
                    isinstance(value, str) for value in (name, identity, executable)
                ):
                    raise TypeError("Invalid nextest executable or test identity")
                selected.append(
                    {"binary_id": identity, "executable": executable, "name": name}
                )
    return selected


def finish(
    path: Path,
    check: dict[str, object],
    terminal_owner: str,
    *,
    deadline: float | None = None,
    deadline_supplier: Callable[[], float | None] | None = None,
    on_frozen: Callable[[], None] | None = None,
    deferred: list[str] | None = None,
) -> bool:
    def current_deadline() -> float | None:
        nonlocal deadline
        if deadline_supplier is not None:
            supplied = deadline_supplier()
            if supplied is not None:
                deadline = supplied if deadline is None else min(deadline, supplied)
        return deadline

    results = check.get("results", [])
    report_errors = check.get("report_errors", [])
    artifacts = check.get("artifacts", {})
    process = check.get("process", {})
    if (
        not isinstance(results, list)
        or not isinstance(report_errors, list)
        or any(not isinstance(item, dict) for item in results)
        or not isinstance(artifacts, dict)
        or not isinstance(process, dict)
    ):
        raise ValueError("Invalid terminal outcome or errors")
    outcomes = {
        str(
            item.get("nodeid") or (str(item["class"]) + "::" + str(item["name"]))
        ): "pass" if item["status"] == "passed" else "failure"
        for item in results
    }
    declaration = resources.read_invocation(path)
    catalog = {
        item["binary_id"] + "::" + item["name"]
        if declaration["kind"] == "rust"
        else item["name"]
        for item in declaration["selected"]
    }
    unexpected = set(outcomes) - catalog
    if unexpected:
        report_errors.append(
            "terminal results outside selected catalog: "
            + ", ".join(sorted(unexpected))
        )
        outcomes = {
            identity: outcome
            for identity, outcome in outcomes.items()
            if identity in catalog
        }
    eligible = resources.finalize(
        path,
        outcomes,
        complete=not check.get("report_errors")
        and check.get("exit_code") in {0, 1, 100},
        terminal_owner=terminal_owner,
        deadline=deadline,
        deadline_supplier=current_deadline,
        on_frozen=on_frozen,
        terminal={
            "workload_exit": check.get("exit_code"),
            "report_errors": list(report_errors),
            "results": list(results),
            "artifacts": dict(artifacts),
            "owned_drain": check.get(
                "owned_drain", process.get("owned_drain", "unobserved")
            ),
            "process": dict(process),
        },
    )
    failed = False
    if deferred is not None:
        deferred.extend(eligible)
        return True
    for resource in eligible:
        try:
            reason = resources.reclaim(resource, deadline=current_deadline())
            if reason != "removed":
                # A referenced/retained resource is valid; an uncertain drain remains pinned.
                print(f"test-resources: {resource}: retained: {reason}")
        except Exception as error:
            failed = True
            report_errors.append(f"fixture cleanup {resource}: {error}")
    if failed:
        check["status"] = "failed"
    return not failed


def execute_selected(
    path: Path,
    command: Sequence[str],
    environment: Mapping[str, str],
    report: Path,
    *,
    selected: list[dict[str, str]] | None = None,
    selection: Path | None = None,
    terminal_owner: str = "runner",
    details: Mapping[str, object] | None = None,
    launch: Callable[[dict[str, object]], int] | None = None,
    evidence_path: Path | None = None,
) -> int:
    """One selected composer reconciles exactly once, including exceptional return.

    Existing enclosing process owners handle signals and physical drain. An inherited
    child never enters this composer. Persist reconciliation errors separately from
    the workload exit; a missing or partial report cannot manufacture a Pass.
    """
    started = time.time()
    check: dict[str, object] = {
        "exit_code": None,
        "status": "failed",
        "started": started,
        "results": [],
        "report_errors": [],
        "command": list(command),
        "artifacts": {},
        "process": {},
        **dict(details or {}),
    }
    if selected is not None:
        check["selected"] = selected
    process_value = check["process"]
    errors_value = check["report_errors"]
    if not isinstance(process_value, dict) or not isinstance(errors_value, list):
        raise TypeError("Invalid terminal process observation or errors")
    process: dict[str, object] = process_value
    report_errors: list[object] = errors_value
    code = None
    from scripts import native_operation  # noqa: PLC0415 -- shared process owner

    registration = None
    previous_handlers = {}
    committed = False
    cleaning = False
    local_deadline: float | None = None
    local_signal: int | None = None
    deadline: float | None = None
    noted_deadline: float | None = None

    def interrupt(signum: int, frame: object) -> None:
        nonlocal local_deadline, local_signal
        if local_deadline is None:
            local_deadline, local_signal = time.monotonic() + 10, signum
        if committed and not cleaning:
            return
        native_operation.note_composer_cancellation(signum, local_deadline)
        if not committed:
            process["forwarded_signal"] = signum
            check.update(exit_code=128 + signum, status="failed")
        validation.interrupt(signum, frame)

    def frozen() -> None:
        nonlocal committed
        committed = True

    def current_deadline() -> float | None:
        nonlocal deadline, noted_deadline
        # Called only outside metadata locks, including after a post-freeze
        # signal whose handler cannot acquire the parent operation exclusion.
        if local_deadline is not None and (
            noted_deadline is None or local_deadline < noted_deadline
        ):
            if local_signal is None:
                raise ValueError("Cancellation clock lacks its signal")
            native_operation.note_composer_cancellation(local_signal, local_deadline)
            noted_deadline = local_deadline
        process_deadline = process.get("cancellation_deadline")
        if process_deadline is not None and not isinstance(
            process_deadline, (int, float)
        ):
            raise ValueError("Invalid original process cancellation clock")
        clocks = [
            value
            for value in (
                deadline,
                local_deadline,
                process_deadline,
                native_operation.composer_deadline(registration),
            )
            if value is not None
        ]
        deadline = min(clocks) if clocks else None
        return deadline

    # Registration cannot expose a composer with SIGTERM's default disposition.
    for signum in (signal.SIGINT, signal.SIGTERM):
        previous_handlers[signum] = signal.signal(signum, interrupt)
    try:
        launch_mask = signal.pthread_sigmask(
            signal.SIG_BLOCK, {signal.SIGINT, signal.SIGTERM}
        )
        try:
            registration = native_operation.register_composer(path)
        finally:
            signal.pthread_sigmask(signal.SIG_SETMASK, launch_mask)
        if launch is not None:
            code = launch(process)
        else:
            run = (
                native_operation.run_child_scope
                if native_operation.scope_owner() is not None
                else native_operation.run
            )
            code = run(list(command), dict(environment), cwd=ROOT, observation=process)
        check.update(exit_code=code, status="passed" if code == 0 else "failed")
    except BaseException as error:
        if isinstance(error, KeyboardInterrupt):
            code = (
                128 + validation.RECEIVED_SIGNAL
                if isinstance(error, validation.Interrupted)
                else -signal.SIGINT
            )
            check["exit_code"] = code
        report_errors.append(
            f"workload interrupted before terminal return: {type(error).__name__}: {error}"
        )
        raise
    finally:
        reconciled = False
        deferred: list[str] = []
        while not reconciled:
            try:
                deadline = current_deadline()
                received = cancellation_signal(
                    native_operation.composer_signal(registration)
                    or process.get("forwarded_signal")
                )
                if received is not None and not committed:
                    code = 128 + received
                    check.update(exit_code=code, status="failed")
                    process["forwarded_signal"] = received
                if selection is not None:
                    try:
                        validation_receipts.require_fresh(selection, started, "Python")
                        check["selected"] = validation_receipts.python_selection(
                            selection
                        )
                    except (OSError, ValueError) as error:
                        report_errors.append(f"missing current collection: {error}")
                validation.compose_terminal(check, report, report.parent, report.stem)
                finish(
                    path,
                    check,
                    terminal_owner,
                    deadline=deadline,
                    deadline_supplier=current_deadline,
                    on_frozen=frozen,
                    deferred=deferred,
                )
                reconciled = True
            except validation.Interrupted:
                # A signal before freeze retries an incomplete composition. If
                # freeze already committed, resume its immutable truth instead.
                try:
                    _identity, terminal = resources.storage().terminal(
                        path.stem, deadline=deadline
                    )
                except resources.ResourceError:
                    code = 128 + validation.RECEIVED_SIGNAL
                    check.update(exit_code=code, status="failed")
                    report_errors.append(
                        "terminal finalization interrupted before freeze"
                    )
                    continue
                except Exception as error:
                    report_errors.append(
                        f"terminal reconciliation: {type(error).__name__}: {error}"
                    )
                    break
                evidence = terminal["evidence"]
                frozen_process = evidence["process"]
                frozen_errors = evidence["report_errors"]
                if not isinstance(frozen_process, dict) or not isinstance(
                    frozen_errors, list
                ):
                    raise TypeError(
                        "Invalid frozen process observation or errors"
                    ) from None
                process = frozen_process
                report_errors = frozen_errors
                code = evidence["workload_exit"]
                check.update(
                    exit_code=code,
                    status="passed"
                    if code == 0 and not evidence["report_errors"]
                    else "failed",
                    results=evidence["results"],
                    report_errors=evidence["report_errors"],
                    artifacts=evidence["artifacts"],
                    process=evidence["process"],
                )
                committed = True
                try:
                    resources.resume_projection(
                        path,
                        terminal_owner=terminal_owner,
                        deadline=current_deadline(),
                        deadline_supplier=current_deadline,
                    )
                except Exception as error:
                    report_errors.append(
                        f"terminal reconciliation: {type(error).__name__}: {error}"
                    )
                    break
                reconciled = True
            except Exception as error:
                report_errors.append(
                    f"terminal reconciliation: {type(error).__name__}: {error}"
                )
                break
        if report_errors:
            check["status"] = "failed"
        try:
            current_deadline()
            resources.resource_store.publish(path.with_suffix(".terminal.json"), check)
            if evidence_path is not None:
                validation.write_json(evidence_path, check)
            if reconciled:
                resources.host.remaining(current_deadline(), 1)
                native_operation.acknowledge_composer(registration)
        except Exception as error:
            print(
                f"test-resources: terminal evidence remains unresolved: {error}",
                file=sys.stderr,
            )
        for error in report_errors:
            print(f"test-resources: {error}", file=sys.stderr)
        if reconciled and deadline is None and local_deadline is None:
            cleaning = True
            try:
                for resource in deferred:
                    resources.reclaim(resource)
            except (Exception, validation.Interrupted) as error:
                print(
                    f"test-resources: post-terminal cleanup retained: {error}",
                    file=sys.stderr,
                )
            finally:
                cleaning = False
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)
    return code


def run_python(
    command: Sequence[str],
    *,
    env: Mapping[str, str] | None = None,
    artifact: Mapping[str, object] | None = None,
) -> int:
    environment = dict(os.environ if env is None else env)
    if resources.MARKER in environment:
        return subprocess.call(command, cwd=ROOT, env=environment)
    path = resources.invocation("python", [])
    environment[resources.MARKER] = str(path)
    report = path.with_suffix(".xml")
    selection = path.with_suffix(".selected.txt")
    environment["PSE_TEST_ENUMERATION"] = str(selection)
    return execute_selected(
        path,
        [*command, "--junitxml=" + str(report)],
        environment,
        report,
        selection=selection,
        details={"artifacts": {"python": dict(artifact)}}
        if artifact is not None
        else None,
    )


def python_observer(
    command: list[str],
    extra: list[str],
    launch: Callable[[dict[str, object]], int],
    *,
    terminal_owner: str = "standalone",
    default_report: bool = False,
    artifact: Mapping[str, object] | None = None,
) -> int:
    """The existing observer forwarding parent owns standalone reconciliation."""
    if resources.MARKER in os.environ or "--terminal-owner=assessment" in extra:
        return launch({})
    destinations = [
        Path(item.split("=", 1)[1]) for item in extra if item.startswith("--junitxml=")
    ]
    if len(destinations) > 1 or (not destinations and not default_report):
        raise ValueError("native Python execution requires one JUnit destination")
    path = resources.invocation("python", [], terminal_owner=terminal_owner)
    report = destinations[0] if destinations else path.with_suffix(".xml")
    if not destinations:
        command.append("--junitxml=" + str(report))
    selection = report.with_name(report.stem + "-selected.txt")
    forwarded = {resources.MARKER: str(path), "PSE_TEST_ENUMERATION": str(selection)}
    if artifact is not None:
        from scripts import python_extension  # noqa: PLC0415 -- selected observation

        forwarded.update(python_extension.execution_environment({}, artifact))
    previous = {key: os.environ.get(key) for key in forwarded}
    os.environ.update(forwarded)
    try:
        return execute_selected(
            path,
            command,
            dict(os.environ),
            report,
            selection=selection,
            terminal_owner=terminal_owner,
            launch=launch,
            evidence_path=report.with_suffix(".terminal.json")
            if destinations
            else None,
            details={"artifacts": {"python": dict(artifact)}}
            if artifact is not None
            else None,
        )
    finally:
        for key, value in previous.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def run_rust(
    command: Sequence[str],
    *,
    env: Mapping[str, str] | None = None,
    inventory: Mapping[str, object] | None = None,
    effects: str = "canonical",
    package_scope: bool = False,
) -> int:
    environment = arrow_validation.rust_test_environment(
        os.environ if env is None else env
    )
    if effects not in {"canonical", "native-local"}:
        raise ValueError("Unknown Rust execution effects")
    environment[EXECUTION_EFFECTS] = effects
    if effects == "native-local":
        # Local execution cannot accidentally inherit canonical fixture authority.
        for name in (
            "PSE_SURREAL_STATE",
            "PSE_WORKER_BINARY",
            "PSE_TEST_EXECUTION_PROFILE",
        ):
            environment.pop(name, None)
    if (
        environment.get("PSE_NATIVE_OPERATION")
        and environment.get(RUST_OBSERVER) != "1"
        and "--binaries-metadata" not in command
    ):
        return observer_rust(
            list(command), environment, effects=effects, package_scope=package_scope
        )
    if package_scope:
        raise ValueError("package-scoped invocation requires its preparing observer")
    inherited = environment.get(resources.MARKER)
    run = list(command)
    position = run.index("run")
    from scripts.native_tests import managed_list_arguments  # noqa: PLC0415

    listing = [
        *run[:position],
        "list",
        *managed_list_arguments(run[position + 1 :]),
        "--message-format",
        "json",
    ]
    if inventory is None:
        if "--binaries-metadata" in command or "--cargo-metadata" in command:
            raise ValueError("Retained Rust metadata requires its prepared inventory")
        response = subprocess.run(
            listing,
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        if response.returncode:
            print(response.stdout)
            print(response.stderr)
            return response.returncode
        inventory = validation_receipts.native_inventory(response.stdout)
    else:
        from scripts.native_tests import prepared_rust_inventory  # noqa: PLC0415

        inventory = prepared_rust_inventory(
            list(command[position + 1 :]), environment, inventory=inventory
        )
    selected = rust_selected(inventory)
    if inherited:
        path = Path(inherited)
        declaration = resources.read_invocation(path)
        if declaration["kind"] != "rust" or (
            declaration["selected"] and declaration["selected"] != selected
        ):
            raise resources.ResourceError(
                "Runner inventory differs from selected invocation"
            )

        resources.publish_selection(path, selected)
        if environment.get("PSE_RUST_TERMINAL_OWNER") != "runner":
            return subprocess.call(command, cwd=ROOT, env=environment)
    else:
        path = resources.invocation("rust", selected)
    environment[resources.MARKER] = str(path)
    run, report = rust_report_command(path, run)
    return execute_selected(
        path,
        run,
        environment,
        report,
        selected=[
            {"class": item["binary_id"], "name": item["name"]} for item in selected
        ],
    )


def rust_report_command(path: Path, run: list[str]) -> tuple[list[str], Path]:
    profile = "default"
    for index, item in enumerate(run):
        if item == "--profile":
            profile = run[index + 1]
        elif item.startswith("--profile="):
            profile = item.split("=", 1)[1]
    configuration = ROOT / ".config/nextest.toml"
    forwarded = []
    index = 0
    while index < len(run):
        if run[index] == "--config-file":
            configuration = Path(run[index + 1])
            index += 2
        elif run[index].startswith("--config-file="):
            configuration = Path(run[index].split("=", 1)[1])
            index += 1
        else:
            forwarded.append(run[index])
            index += 1
    run = forwarded
    config = configuration.read_text()

    expression = rf"(?m)(^\[profile\.{re.escape(profile)}\.junit\]\n)path\s*=\s*[^\n]+"
    report = path.with_suffix(".xml")
    config, count = re.subn(
        expression, lambda match: match[1] + "path = " + json.dumps(str(report)), config
    )
    if not count:
        config += f"\n[profile.{profile}.junit]\npath = {json.dumps(str(report))}\nstore-failure-output = true\n"
    config_path = path.with_suffix(".toml")
    config_path.write_text(config)
    position = run.index("--") if "--" in run else len(run)
    run[position:position] = ["--config-file", str(config_path)]
    return run, report


def observer_rust(
    command: list[str],
    environment: dict[str, str],
    *,
    effects: str = "canonical",
    package_scope: bool = False,
) -> int:
    """Prepare exact artifacts before the scientific observer's process cap."""
    from scripts import (  # noqa: PLC0415 -- runner cycle
        native_tests,
        surreal_server,
    )

    path = environment.get("PSE_NATIVE_PROVENANCE")
    if path is None:
        output = validation.fresh_output(
            ROOT, ROOT / "build/native-tests" / str(time.time_ns())
        )
        path = str(output / "native.json")
    provenance = Path(path).resolve()
    selection = Path(
        environment.get(
            "PSE_NATIVE_SELECTION", str(provenance.with_name("native-selected.json"))
        )
    ).resolve()
    execution, worker = native_tests.ordinary_rust_capture(
        command,
        provenance,
        selection,
        environment,
        effects=effects,
        package_scope=package_scope,
    )
    inventory = validation_receipts.native_inventory(selection.read_text())
    fixtures = native_tests.rust_fixture_bindings(provenance, environment)
    environment.update(fixtures)
    if effects == "native-local":
        validation_receipts.verify_native(json.loads(provenance.read_text()))
        return run_rust(
            ["cargo", "nextest", "run", *execution],
            env={
                **environment,
                "PSE_NATIVE_PROVENANCE": str(provenance),
                "PSE_NATIVE_SELECTION": str(selection),
            },
            inventory=inventory,
            effects=effects,
        )
    if worker is None:
        raise resources.ResourceError("Canonical execution requires its worker")
    selected = rust_selected(inventory)
    inherited = environment.get(resources.MARKER)
    if inherited:
        invocation = Path(inherited)
        declaration = resources.read_invocation(invocation)
        if declaration["kind"] != "rust" or (
            declaration["selected"] and declaration["selected"] != selected
        ):
            raise resources.ResourceError(
                "Runner inventory differs from selected invocation"
            )
        resources.publish_selection(invocation, selected)
    else:
        invocation = resources.invocation("rust", selected)
    forwarded = {
        **fixtures,
        resources.MARKER: str(invocation),
        "PSE_NATIVE_PROVENANCE": str(provenance),
        "PSE_NATIVE_SELECTION": str(selection),
        "PSE_WORKER_BINARY": str(worker),
        "PSE_TEST_EXECUTION_PROFILE": "exclusive-observer",
        RUST_OBSERVER: "1",
        "PSE_RUST_TERMINAL_OWNER": "inherited",
        "RUST_MIN_STACK": environment["RUST_MIN_STACK"],
    }
    # observer uses the existing process environment; restore every carried
    # value after launch rather than leaving invocation authority ambient.
    previous = {key: os.environ.get(key) for key in forwarded}
    report: Path | None = None
    if not inherited:
        configured, report = rust_report_command(
            invocation, ["cargo", "nextest", "run", *execution]
        )
        execution = configured[3:]
    os.environ.update(forwarded)
    try:
        child = [
            sys.executable,
            "-m",
            "scripts.test_run",
            "--observer-child",
            *execution,
        ]

        def launch(observation: dict[str, object]) -> int:
            return surreal_server.observer(
                Path(environment["PSE_SURREAL_STATE"]),
                child,
                profile="exclusive-observer",
                observation=observation,
            )

        if inherited:
            return launch({})
        if report is None:
            raise ValueError("Selected Rust observer lacks its report destination")
        return execute_selected(
            invocation,
            child,
            {**environment, **forwarded},
            report,
            selected=[
                {"class": item["binary_id"], "name": item["name"]} for item in selected
            ],
            launch=launch,
        )
    finally:
        for key, value in previous.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def observer_child(extra: list[str]) -> int:
    """Verify retained artifacts, then run; this role never builds."""
    from scripts import native_tests  # noqa: PLC0415 -- owner cycle

    if (
        os.environ.get(RUST_OBSERVER) != "1"
        or os.environ.get("PSE_TEST_EXECUTION_PROFILE") != "exclusive-observer"
    ):
        raise resources.ResourceError(
            "Rust observer child requires its declared profile"
        )
    provenance = Path(os.environ["PSE_NATIVE_PROVENANCE"])
    # Reuse the managed verifier, not its feature or test-selection lowering.
    # run_rust sees retained metadata and the authenticated observer sentinel.
    return native_tests.managed_rust_run(
        [f"--managed-primary-worker={os.environ['PSE_WORKER_BINARY']}", *extra],
        provenance,
    )


if __name__ == "__main__":
    if sys.argv[1:2] != ["--observer-child"]:
        raise SystemExit("test_run expects an observer child invocation")
    raise SystemExit(observer_child(sys.argv[2:]))
