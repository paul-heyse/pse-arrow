# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Operation ownership, immutable publication and archive association controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import fcntl
import io
import json
import os
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, TypedDict

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
from unittest.mock import MagicMock, patch

from scripts import (
    build_environment,
    build_measurements,
    host_admission,
    producer_deployment,
    pse_env,
    surreal_server,
    validation,
)
from scripts import native_cache as cache
from scripts import native_operation as operation
from scripts import native_pipeline_cache as pipeline


class FixtureScope(TypedDict):
    scope: operation.ScopeOwner
    scope_inode: int


class FixtureEntry(FixtureScope):
    pid: int
    start: str
    allocation: str


def drain_fixture(complete: bool) -> operation.DrainObservation:
    return operation.DrainObservation(
        None,
        operation.DrainPhase.COMPLETE if complete else operation.DrainPhase.POPULATED,
        "controlled observation",
    )


class NativeOperationTests(unittest.TestCase):
    def test_prelaunch_expiry_retires_only_exact_unlaunched_handoff(self) -> None:
        for mode in (
            "expired",
            "sigint",
            "parent-clock",
            "parent-sigint",
            "child-sigint",
            "mismatch",
            "missing-owner",
        ):
            with self.subTest(mode=mode):
                self.assert_prelaunch_handoff_case(mode)

    def assert_prelaunch_handoff_case(self, mode: str) -> None:
        now = [100.0]
        prepared: list[Path] = []
        observation: dict[str, object] = {}
        allocation = MagicMock(spec=host_admission.Allocation)
        allocation.nonce = "c" * 32
        allocation.profile = host_admission.select("reference")
        scope: operation.ScopeOwner = {
            "unit": "parent.scope",
            "group": "/parent",
            "invocation": "b" * 32,
        }
        original_prepare = operation.prepare_handoff
        original_mask = signal.pthread_sigmask
        handlers = {
            signum: signal.getsignal(signum)
            for signum in (signal.SIGINT, signal.SIGTERM)
        }
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            parent = base / ("a" * 32 + ".json")
            sibling = base / ("handoff-" + "d" * 32 + ".json")
            operation.write_json(
                parent,
                {
                    "version": operation.VERSION,
                    "scope": scope,
                    "admissions": {},
                    "generations": ["retained-generation"],
                    "foreground_children": [str(sibling)],
                },
            )
            sibling.write_text("retained sibling evidence")

            def parent_owner(*, deadline: float | None = None) -> Path | None:
                self.assertIsNotNone(deadline)
                if mode == "missing-owner" and prepared:
                    return None
                return parent

            def prepare(
                unit: str,
                *,
                foreground: bool,
                deadline: float | None = None,
            ) -> Path:
                handoff = original_prepare(
                    unit,
                    foreground=foreground,
                    deadline=deadline,
                )
                assert handoff is not None
                prepared.append(handoff)
                if mode == "mismatch":
                    changed = json.loads(handoff.read_text())
                    changed["handoff"] = "different.scope"
                    operation.write_json(handoff, changed)
                if mode == "parent-clock":
                    changed_parent = json.loads(parent.read_text())
                    changed_parent["cancel_deadline"] = 105
                    operation.write_json(parent, changed_parent)
                if mode in {"parent-sigint", "child-sigint"}:
                    cancelled_path = parent if mode == "parent-sigint" else handoff
                    cancelled_record = json.loads(cancelled_path.read_text())
                    cancelled_record.update(
                        cancelled=True,
                        cancel_deadline=105,
                        cancel_signal=signal.SIGINT,
                    )
                    operation.write_json(cancelled_path, cancelled_record)
                if mode != "sigint":
                    now[0] = 100.3
                return handoff

            delivered = False

            def restore_mask(
                how: int,
                mask: set[int | signal.Signals],
            ) -> set[int | signal.Signals]:
                nonlocal delivered
                previous = original_mask(how, mask)
                if (
                    mode == "sigint"
                    and prepared
                    and not delivered
                    and how == signal.SIG_SETMASK
                ):
                    delivered = True
                    now[0] = 100.1
                    signal.raise_signal(signal.SIGINT)
                    now[0] = 100.3
                return previous

            with (
                patch.object(host_admission, "inherit", return_value=allocation),
                patch.object(operation, "scope_owner", return_value=scope),
                patch.object(operation, "owner_record", side_effect=parent_owner),
                patch.object(pse_env, "current_memory_ceiling", return_value=4 << 30),
                patch.object(operation.time, "monotonic", side_effect=lambda: now[0]),
                patch.object(operation, "prepare_handoff", side_effect=prepare),
                patch.object(
                    operation.signal, "pthread_sigmask", side_effect=restore_mask
                ),
                patch.object(operation.subprocess, "Popen") as spawn,
                patch.object(operation, "drained") as drain,
                patch.object(
                    operation,
                    "fence_foreground_handoff",
                    wraps=operation.fence_foreground_handoff,
                ) as fence,
            ):
                if mode in {"sigint", "parent-sigint", "child-sigint"}:
                    self.assertEqual(
                        operation.run_child_scope(
                            ["never-launched"],
                            {},
                            deadline=100.2,
                            observation=observation,
                        ),
                        128 + signal.SIGINT,
                    )
                else:
                    expected = (
                        subprocess.TimeoutExpired
                        if mode in {"expired", "parent-clock"}
                        else ValueError
                    )
                    with self.assertRaises(expected):
                        operation.run_child_scope(
                            ["never-launched"],
                            {},
                            deadline=100.2,
                            observation=observation,
                        )
                spawn.assert_not_called()
                drain.assert_not_called()
                allocation.register.assert_called_once()
                allocation.release.assert_not_called()
                self.assertEqual(len(prepared), 1)
                selected = prepared[0]
                if mode in {
                    "expired",
                    "sigint",
                    "parent-clock",
                    "parent-sigint",
                    "child-sigint",
                }:
                    self.assertFalse(selected.exists())
                    self.assertEqual(
                        json.loads(parent.read_text())["foreground_children"],
                        [str(sibling)],
                    )
                    fence.assert_called_once()
                    expected_clock = 110.1 if mode == "sigint" else 110.3
                    if mode in {"parent-sigint", "child-sigint"}:
                        expected_clock = 105
                    self.assertEqual(
                        fence.call_args.kwargs["deadline"],
                        105 if mode == "parent-clock" else expected_clock,
                    )
                    self.assertEqual(
                        observation["cancellation_deadline"], expected_clock
                    )
                    self.assertEqual(observation["owned_drain"], "not-started")
                else:
                    self.assertTrue(selected.exists())
                    self.assertIn(
                        str(selected),
                        json.loads(parent.read_text())["foreground_children"],
                    )
                    fence.assert_not_called()
                    self.assertNotEqual(observation["owned_drain"], "not-started")
                self.assertEqual(sibling.read_text(), "retained sibling evidence")
                self.assertIsNone(observation["child_pid"])
                self.assertIsNone(observation["child_status"])
                self.assertEqual(
                    observation["forwarded_signal"],
                    signal.SIGINT
                    if mode in {"sigint", "parent-sigint", "child-sigint"}
                    else signal.SIGTERM,
                )
        self.assertEqual(
            {signum: signal.getsignal(signum) for signum in handlers}, handlers
        )

    def test_workload_deadline_cancels_then_drains_and_preserves_earlier_sigint(
        self,
    ) -> None:
        for mode in (
            "expired",
            "sigint",
            "inherited-sigint",
            "unbounded",
            "drain-refused",
        ):
            with self.subTest(mode=mode):
                self.assert_workload_deadline_case(mode)

    def assert_workload_deadline_case(self, mode: str) -> None:
        now = [100.0]
        events: list[str] = []
        observation: dict[str, object] = {}

        class Child:
            pid = 2147483647
            returncode: int | None = None
            waits = 0

            def wait(self, timeout: float) -> int:
                self.waits += 1
                if mode == "unbounded":
                    self.returncode = 0
                elif mode == "sigint" and self.waits == 1:
                    signal.raise_signal(signal.SIGINT)
                    now[0] = 100.3
                    raise subprocess.TimeoutExpired("controlled", timeout)
                if self.returncode is not None:
                    return self.returncode
                now[0] += timeout
                raise subprocess.TimeoutExpired("controlled", timeout)

        child = Child()

        def cancel(_child: object, signum: int, *, deadline: float) -> None:
            events.append("cancel")
            self.assertEqual(deadline, now[0] + 10)
            child.returncode = -signum

        def drained() -> None:
            self.assertIsNotNone(child.returncode)
            events.append("drained")
            if mode == "drain-refused":
                raise ValueError("owned drain unavailable")

        def cancel_foreground(
            process: object, signum: int, _path: Path, *, deadline: float
        ) -> tuple[float, int]:
            cancel(process, signum, deadline=deadline)
            child.returncode = -signal.SIGINT
            return 105, signal.SIGINT

        with (
            patch.object(operation.time, "monotonic", side_effect=lambda: now[0]),
            patch.object(operation.subprocess, "Popen", return_value=child) as spawn,
            patch.object(operation, "cancel_children", side_effect=cancel) as cancelled,
            patch.object(
                operation, "cancel_foreground", side_effect=cancel_foreground
            ) as foreground_cancelled,
        ):
            if mode in {"expired", "drain-refused"}:
                expected = (
                    subprocess.TimeoutExpired if mode == "expired" else ValueError
                )
                with self.assertRaises(expected):
                    operation.run(
                        ["controlled"],
                        {},
                        deadline=100.2,
                        observation=observation,
                        on_reaped=drained,
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                    )
            else:
                status = operation.run(
                    ["controlled"],
                    {},
                    deadline=None if mode == "unbounded" else 100.2,
                    foreground=Path("/owned/handoff")
                    if mode == "inherited-sigint"
                    else None,
                    observation=observation,
                    on_reaped=drained,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
                self.assertEqual(status, 0 if mode == "unbounded" else 130)
        self.assertEqual(events[-1], "drained")
        self.assertEqual(spawn.call_args.kwargs["stdout"], subprocess.DEVNULL)
        self.assertEqual(spawn.call_args.kwargs["stderr"], subprocess.DEVNULL)
        if mode == "unbounded":
            cancelled.assert_not_called()
        elif mode == "inherited-sigint":
            cancelled.assert_not_called()
            foreground_cancelled.assert_called_once()
            self.assertEqual(observation["cancellation_deadline"], 105)
            self.assertEqual(observation["forwarded_signal"], signal.SIGINT)
        else:
            cancelled.assert_called_once()
            self.assertEqual(events, ["cancel", "drained"])
            if mode == "sigint":
                self.assertEqual(observation["cancellation_deadline"], 110)
                self.assertEqual(observation["forwarded_signal"], signal.SIGINT)

    def test_original_cancellation_clock_preserves_exact_numeric_authority(
        self,
    ) -> None:
        integer = (1 << 53) + 3
        clock = operation.original_cancellation_clock(integer)
        self.assertIs(clock, integer)
        self.assertEqual(clock, integer)
        fractional = 105.25
        self.assertIs(operation.original_cancellation_clock(fractional), fractional)
        for invalid in (
            True,
            False,
            float("nan"),
            float("inf"),
            -float("inf"),
            None,
            "105",
        ):
            with (
                self.subTest(value=invalid),
                self.assertRaisesRegex(
                    ValueError, "invalid original cancellation clock"
                ),
            ):
                operation.original_cancellation_clock(invalid)

    def setUp(self) -> None:
        # Each fixture supplies its own admission owner, including when this
        # suite runs inside a real assessment operation.
        environment = {
            name: value
            for name, value in os.environ.items()
            if name
            not in {
                operation.MARKER,
                "PSE_NATIVE_HANDOFF",
                pse_env.host.MARKER,
                "PSE_ADMISSION_DEADLINE",
            }
        }
        isolated = patch.dict(os.environ, environment, clear=True)
        isolated.start()
        self.addCleanup(isolated.stop)
        self.fixture_units: set[str] = set()
        self.settled_entry_fixtures: set[Path] = set()
        self.fixture_parent_scope = operation.scope_owner()
        self.fixture_parent_group = operation.process_group(os.getpid())

    def test_selected_library_paths_do_not_add_working_directory_search(self) -> None:
        for previous in (None, ""):
            environment = {} if previous is None else {"LD_LIBRARY_PATH": previous}
            selected = cache.prepend_library_path("/selected/lib", environment)
            self.assertEqual(selected.split(os.pathsep), ["/selected/lib"])
            environment["LD_LIBRARY_PATH"] = selected
            self.assertEqual(
                cache.prepend_library_path("/pipeline/lib", environment).split(
                    os.pathsep
                ),
                ["/pipeline/lib", "/selected/lib"],
            )
        self.assertEqual(
            cache.prepend_library_path(
                "/selected/lib", {"LD_LIBRARY_PATH": "/explicit/lib:/other/lib"}
            ),
            "/selected/lib:/explicit/lib:/other/lib",
        )
        environment = {"LD_LIBRARY_PATH": "/caller/lib::/caller/lib:/solver/lib:"}
        libraries = ("/solver/lib", "/uno/lib", "/petsc/lib")
        for library in libraries:
            environment["LD_LIBRARY_PATH"] = cache.prepend_library_path(
                library, environment
            )
        expected = "/petsc/lib:/uno/lib:/solver/lib:/caller/lib::/caller/lib:"
        self.assertEqual(environment["LD_LIBRARY_PATH"], expected)
        for library in libraries:
            environment["LD_LIBRARY_PATH"] = cache.prepend_library_path(
                library, environment
            )
        self.assertEqual(environment["LD_LIBRARY_PATH"], expected)

    def await_file(self, path: Path, process: subprocess.Popen | None = None) -> dict:
        deadline = time.monotonic() + 15
        while not path.exists():
            if process is not None and process.poll() is not None:
                self.fail(
                    f"native fixture exited before admission: {process.returncode}; "
                    + (
                        (path.parent / "fixture.log").read_text()[-3000:]
                        if (path.parent / "fixture.log").is_file()
                        else ""
                    )
                )
            if time.monotonic() >= deadline:
                self.fail("native fixture did not admit within its bounded deadline")
            time.sleep(0.02)
        return json.loads(path.read_text())

    def await_drain(self, record: Mapping[str, object]) -> None:
        deadline = time.monotonic() + 15
        while True:
            observed = operation.drained(record)
            if observed.complete:
                return
            if time.monotonic() >= deadline:
                self.fail(
                    observed.diagnostic()
                    + "; authentic native fixture scope did not drain"
                )
            time.sleep(0.02)

    def fixture_environment(self, base: Path) -> dict[str, str]:
        return {
            **{
                name: value
                for name, value in os.environ.items()
                if name
                not in {
                    operation.MARKER,
                    "PSE_NATIVE_HANDOFF",
                    pse_env.host.MARKER,
                    "PSE_ADMISSION_DEADLINE",
                }
            },
            "PSE_NATIVE_CACHE": str(base),
            "PSE_NATIVE_CAPABILITIES": "",
            "PSE_MEMORY_MAX": "512M",
            "PSE_RESOURCE_CLASS": "light",
            **pse_env.manager_environment(),
        }

    def fixture_command(
        self,
        command: list[str],
        environment: dict[str, str],
        *,
        native: bool = True,
        cleanup: bool = True,
    ) -> list[str]:
        self.assert_parent_scope_unchanged()
        # These zero-capability fixtures own disposable generations and may kill
        # their entire scope. Compose a new fixture root through the placement
        # owner; do not change the production policy that nested users stay put.
        # Only command construction sees this test-isolation premise. The real
        # child observes and validates its actual unit and invocation normally.
        with patch.object(operation, "scope_owner", return_value=None):
            prefix = pse_env.placement(environment, native=native)
        allocation = pse_env.host.inherit(environment)
        self.assertIsNotNone(allocation, "fixture must own its host allocation")
        assert allocation is not None
        if cleanup:
            self.addCleanup(allocation.release)
        self.assertTrue(prefix, "independent fixture requires the local user manager")
        self.assertIn(f"MemoryMax={512 * (1 << 20)}", prefix)
        unit = next(
            argument.removeprefix("--unit=")
            for argument in prefix
            if argument.startswith("--unit=")
        )
        if self.fixture_parent_scope is not None:
            self.assertNotEqual(unit, self.fixture_parent_scope["unit"])
        self.fixture_units.add(unit)
        return [*prefix, *command]

    def entry_fixture_command(
        self,
        base: Path,
        command: list[str],
        environment: dict[str, str],
        *,
        inherited_manager: bool = True,
    ) -> list[str]:
        # The actual entry must acquire its own native allocation. First leave
        # any enclosing assessment's native scope through a real ordinary scope;
        # only its authenticated child drops that fixture's inherited authority.
        # Both light allocations remain charged in the real host ledger.
        prefix = self.fixture_command([], environment, native=False, cleanup=False)
        allocation = pse_env.host.inherit(environment)
        assert allocation is not None
        unit = next(
            argument.removeprefix("--unit=")
            for argument in prefix
            if argument.startswith("--unit=")
        )
        self.assertTrue(unit.startswith("pse-cmd-"))
        self.addCleanup(self.settle_entry_fixture, base, unit, allocation)
        launcher = (
            "import os,sys; from pathlib import Path\n"
            "from scripts import native_operation as n,host_admission as h\n"
            "base=Path(sys.argv[1]); unit=sys.argv[2]\n"
            "allocation=h.inherit(os.environ); assert allocation is not None\n"
            "observed=n.unit_observation(unit); group=n.process_group(os.getpid())\n"
            "assert observed['LoadState']=='loaded' and observed['ControlGroup']==group\n"
            "assert n.scope_owner() is None\n"
            "inode=h.group_identity(group); assert type(inode) is int\n"
            "n.write_json(base/'entry-launcher.json',{'pid':os.getpid(),'start':n.start_identity(os.getpid()),'allocation':os.environ[h.MARKER],'scope_inode':inode,'scope':{'unit':unit,'group':group,'invocation':observed['InvocationID']}})\n"
            "environment=dict(os.environ)\n"
            "for name in (h.MARKER,n.MARKER,'PSE_NATIVE_HANDOFF','PSE_ADMISSION_DEADLINE'): environment.pop(name,None)\n"
            "if sys.argv[3]=='0':\n for name in ('XDG_RUNTIME_DIR','DBUS_SESSION_BUS_ADDRESS'): environment.pop(name,None)\n"
            "os.execvpe(sys.argv[4],sys.argv[4:],environment)"
        )
        return [
            *prefix,
            sys.executable,
            "-c",
            launcher,
            str(base),
            unit,
            "1" if inherited_manager else "0",
            *command,
        ]

    def settle_fixture_scope(self, record: FixtureScope, *, deadline: float) -> None:
        if operation.drained(record, deadline=deadline).complete:
            return
        owner = record["scope"]
        self.fixture_units.add(owner["unit"])
        self.assert_fixture_scope(owner, live=False, deadline=deadline)
        self.assertIs(type(record.get("scope_inode")), int)
        self.assertEqual(
            host_admission.group_identity(owner["group"]), record["scope_inode"]
        )
        observed = operation.unit_observation(owner["unit"], deadline=deadline)
        self.assertEqual(observed["LoadState"], "loaded")
        self.assertEqual(observed["ControlGroup"], owner["group"])
        self.assertEqual(observed["InvocationID"], owner["invocation"])
        result = subprocess.run(
            [
                "systemctl",
                "--user",
                "kill",
                "--kill-whom=all",
                "--signal=SIGKILL",
                owner["unit"],
            ],
            env=surreal_server.systemd_environment(),
            check=False,
            capture_output=True,
            text=True,
            timeout=host_admission.remaining(deadline, 5),
        )
        if result.returncode:
            self.assertTrue(
                operation.drained(record, deadline=deadline).complete,
                f"Fixture kill failed ({result.returncode}): {result.stderr}",
            )
        while not operation.drained(record, deadline=deadline).complete:
            time.sleep(min(0.02, host_admission.remaining(deadline, 0.02)))

    def signal_fixture_entry(self, entry: FixtureEntry, *, deadline: float) -> None:
        owner = entry["scope"]
        self.assert_fixture_scope(owner, live=False, deadline=deadline)
        descriptor = None
        try:
            descriptor = os.pidfd_open(entry["pid"])
            if (
                operation.start_identity(entry["pid"]) == entry["start"]
                and operation.process_group(entry["pid"]) == owner["group"]
                and host_admission.group_identity(owner["group"])
                == entry["scope_inode"]
            ):
                observed = operation.unit_observation(owner["unit"], deadline=deadline)
                if (
                    observed.get("ControlGroup") == owner["group"]
                    and observed.get("InvocationID") == owner["invocation"]
                ):
                    signal.pidfd_send_signal(descriptor, signal.SIGTERM)
        except (FileNotFoundError, ProcessLookupError):
            pass  # A vanished process grants no PID signal authority.
        finally:
            if descriptor is not None:
                os.close(descriptor)

    def settle_entry_fixture(
        self, base: Path, unit: str, allocation: host_admission.Allocation
    ) -> None:
        if base in self.settled_entry_fixtures:
            return
        deadline = time.monotonic() + 45
        errors: list[str] = []
        entry: FixtureEntry | None = None
        ordinary: host_admission.AllocationRecord | None = None
        launcher_path = base / "entry-launcher.json"
        if launcher_path.exists():
            try:
                candidate: FixtureEntry = json.loads(launcher_path.read_text())
                self.assertEqual(candidate["scope"]["unit"], unit)
                self.assertEqual(
                    candidate["allocation"],
                    str(allocation.directory / allocation.nonce),
                )
                entry = candidate
                self.signal_fixture_entry(entry, deadline=deadline)
                grace = min(deadline, time.monotonic() + 12)
                while (
                    time.monotonic() < grace
                    and not operation.drained(entry, deadline=grace).complete
                ):
                    time.sleep(0.02)
            except Exception as error:  # Continue independent authenticated cleanup.
                errors.append(f"entry signal: {type(error).__name__}: {error}")
        try:
            snapshot = host_admission.readonly_snapshot(
                allocation.directory, deadline=deadline
            )
            ordinary = snapshot["owners"].get(allocation.nonce)
            if ordinary is not None:
                binding = ordinary["units"][unit]
                if binding:
                    self.settle_fixture_scope(
                        {
                            "scope": {
                                "unit": unit,
                                "group": binding["group"],
                                "invocation": binding["invocation"],
                            },
                            "scope_inode": binding["inode"],
                        },
                        deadline=deadline,
                    )
        except Exception as error:
            errors.append(f"ordinary drain: {type(error).__name__}: {error}")
        # The real ledger grants authority only for allocations created by this
        # launcher's exact lifetime. An unproved drain remains a cleanup failure.
        owned: dict[str, host_admission.AllocationRecord] = {}
        try:
            snapshot = host_admission.readonly_snapshot(
                allocation.directory, deadline=deadline
            )
            if entry is not None:
                owned = {
                    nonce: owner
                    for nonce, owner in snapshot["owners"].items()
                    if nonce != allocation.nonce
                    and owner["boot"] == host_admission.boot()
                    and owner["pid"] == entry["pid"]
                    and owner["start"] == entry["start"]
                }
            records = {
                str(path): operation._record(path)  # noqa: SLF001 -- retain exact private fixture operation evidence
                for path in (base / ".operations").glob("*.json")
            }
            operation.write_json(
                base / "entry-cleanup.json",
                {
                    "entry": entry,
                    "ordinary_allocation": ordinary,
                    "allocations": owned,
                    "operations": records,
                    "previous": json.loads((base / "entry-cleanup.json").read_text())
                    if (base / "entry-cleanup.json").exists()
                    else None,
                },
            )
        except Exception as error:
            errors.append(f"ownership snapshot: {type(error).__name__}: {error}")
        for nonce, owner in owned.items():
            for selected_unit, binding in owner["units"].items():
                if not binding:
                    continue  # Release itself verifies any unbound launch is absent.
                try:
                    self.settle_fixture_scope(
                        {
                            "scope": {
                                "unit": selected_unit,
                                "group": binding["group"],
                                "invocation": binding["invocation"],
                            },
                            "scope_inode": binding["inode"],
                        },
                        deadline=deadline,
                    )
                except Exception as error:
                    errors.append(
                        f"{selected_unit} drain: {type(error).__name__}: {error}"
                    )
            native_allocation = host_admission.Allocation(
                allocation.directory,
                nonce,
                host_admission.Profile(
                    owner["class"],
                    owner["memory"],
                    owner["lane"],
                    owner["slots"],
                    tuple(owner["cores"]),
                    owner["exclusive"],
                ),
                owner["deadline"],
            )
            try:
                self.assertTrue(
                    native_allocation.release(deadline=deadline),
                    f"Fixture allocation {nonce} remains charged; evidence at {base}",
                )
            except Exception as error:
                errors.append(f"{nonce} release: {type(error).__name__}: {error}")
        try:
            self.assertTrue(
                allocation.release(deadline=deadline),
                f"Ordinary fixture remains charged; evidence at {base}",
            )
        except Exception as error:
            errors.append(f"ordinary release: {type(error).__name__}: {error}")
        self.assertEqual(errors, [], f"Retained fixture cleanup evidence: {base}")
        self.settled_entry_fixtures.add(base)

    def assert_parent_scope_unchanged(self, *, deadline: float | None = None) -> None:
        self.assertEqual(
            operation.process_group(os.getpid()), self.fixture_parent_group
        )
        observed = (
            operation.scope_owner()
            if deadline is None
            else operation.scope_owner(deadline=deadline)
        )
        self.assertEqual(observed, self.fixture_parent_scope)

    def assert_fixture_scope(
        self,
        owner: operation.ScopeOwner | Mapping[str, str],
        *,
        live: bool = True,
        deadline: float | None = None,
    ) -> None:
        self.assert_parent_scope_unchanged(deadline=deadline)
        self.assertIn(owner["unit"], self.fixture_units)
        self.assertNotEqual(owner["group"], self.fixture_parent_group)
        self.assertFalse(owner["group"].startswith(self.fixture_parent_group + "/"))
        self.assertFalse(self.fixture_parent_group.startswith(owner["group"] + "/"))
        if self.fixture_parent_scope is not None:
            self.assertNotEqual(owner["unit"], self.fixture_parent_scope["unit"])
            self.assertNotEqual(owner["group"], self.fixture_parent_scope["group"])
            self.assertNotEqual(
                owner["invocation"], self.fixture_parent_scope["invocation"]
            )
        if live:
            observed = (
                operation.unit_observation(owner["unit"])
                if deadline is None
                else operation.unit_observation(owner["unit"], deadline=deadline)
            )
            self.assertEqual(observed["LoadState"], "loaded")
            self.assertEqual(observed["ControlGroup"], owner["group"])
            self.assertEqual(observed["InvocationID"], owner["invocation"])

    def test_child_fixture_environment_starts_independent_owner(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {
                    operation.MARKER: "/outer/.operations/owner.json",
                    "PSE_NATIVE_HANDOFF": "/outer/.operations/handoff.json",
                    pse_env.host.MARKER: "/outer/host/" + "c" * 32,
                    "PSE_ADMISSION_DEADLINE": "1",
                    "PSE_FIXTURE_CALLER": "preserved",
                },
            ),
        ):
            base = Path(directory)
            environment = self.fixture_environment(base)
            self.assertNotIn(operation.MARKER, environment)
            self.assertNotIn("PSE_NATIVE_HANDOFF", environment)
            self.assertNotIn(pse_env.host.MARKER, environment)
            self.assertNotIn("PSE_ADMISSION_DEADLINE", environment)
            self.assertEqual(environment["PSE_NATIVE_CACHE"], str(base))
            self.assertEqual(environment["PSE_FIXTURE_CALLER"], "preserved")
            self.assertEqual(
                os.environ[operation.MARKER], "/outer/.operations/owner.json"
            )

    def test_independent_fixture_command_preserves_nested_scope_policy(self) -> None:
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/parent",
            "invocation": "b" * 32,
        }
        self.fixture_parent_scope = owner
        self.fixture_parent_group = owner["group"]
        with (
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(pse_env, "manager_available", return_value=True),
            patch.object(pse_env, "manager_environment", return_value={}),
            patch.object(pse_env, "limits", return_value=[]),
            patch.object(pse_env, "cpu_ceiling", return_value=1.0),
            patch.object(
                os,
                "sched_getaffinity",
                return_value=set(
                    host_admission.cpu_set(host_admission.select("light").cores)
                ),
            ),
        ):
            environment = self.fixture_environment(Path("/disposable-fixture"))
            command = self.fixture_command(["fixture", "literal argument"], environment)
            self.assertEqual(command[-2:], ["fixture", "literal argument"])
            self.assertEqual(len(self.fixture_units), 1)
            self.assertNotIn(owner["unit"], self.fixture_units)
            self.assertEqual(operation.scope_owner(), owner)
            self.assertEqual(pse_env.placement(environment, native=True), [])
            self.assertEqual(environment["NEXTEST_TEST_THREADS"], "1")

    def test_fixture_cleanup_refuses_enclosing_scope(self) -> None:
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/parent",
            "invocation": "b" * 32,
        }
        self.fixture_parent_scope = owner
        self.fixture_parent_group = owner["group"]
        with (
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(operation.subprocess, "run") as manager,
        ):
            with self.assertRaisesRegex(AssertionError, "only a minted fixture"):
                self.kill_scope(owner["unit"])
            manager.assert_not_called()

    def test_entry_cleanup_lost_pid_authority_never_signals_replacement(self) -> None:
        unit = "pse-cmd-" + "a" * 32 + ".scope"
        self.fixture_units.add(unit)
        entry: FixtureEntry = {
            "pid": 12345,
            "start": "original-start",
            "allocation": "/fixture/allocation",
            "scope": {"unit": unit, "group": "/fixture", "invocation": "b" * 32},
            "scope_inode": 123,
        }
        for vanished_before_open in (False, True):
            with (
                self.subTest(vanished_before_open=vanished_before_open),
                patch.object(self, "assert_parent_scope_unchanged"),
                patch.object(
                    os,
                    "pidfd_open",
                    side_effect=ProcessLookupError if vanished_before_open else None,
                    return_value=7,
                ),
                patch.object(
                    operation, "start_identity", side_effect=FileNotFoundError
                ),
                patch.object(signal, "pidfd_send_signal") as forwarded,
                patch.object(os, "close") as close,
            ):
                self.signal_fixture_entry(entry, deadline=time.monotonic() + 1)
                forwarded.assert_not_called()
                if vanished_before_open:
                    close.assert_not_called()
                else:
                    close.assert_called_once_with(7)

    def test_entry_cleanup_refused_release_retains_sibling_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            ordinary_unit = "pse-cmd-" + "a" * 32 + ".scope"
            native_unit = "pse-native-" + "b" * 32 + ".scope"
            profile = host_admission.Profile("light", 512 << 20, "light", 1, (8,))
            allocation = host_admission.Allocation(
                base / "ledger", "c" * 32, profile, time.monotonic() + 30
            )
            entry: FixtureEntry = {
                "pid": 12345,
                "start": "fixture-start",
                "allocation": str(allocation.directory / allocation.nonce),
                "scope": {
                    "unit": ordinary_unit,
                    "group": "/ordinary",
                    "invocation": "d" * 32,
                },
                "scope_inode": 123,
            }
            operation.write_json(base / "entry-launcher.json", entry)
            (base / ".operations").mkdir()
            operation_record = {
                "version": operation.VERSION,
                "scope": None,
                "admissions": {},
                "generations": [],
            }
            operation.write_json(base / ".operations/owner.json", operation_record)
            native: host_admission.AllocationRecord = {
                "boot": "fixture-boot",
                "pid": entry["pid"],
                "start": entry["start"],
                "class": profile.name,
                "memory": profile.memory,
                "lane": profile.lane,
                "slots": profile.slots,
                "cores": list(profile.cores),
                "exclusive": False,
                "deadline": allocation.deadline,
                "units": {
                    native_unit: {
                        "group": "/native",
                        "invocation": "e" * 32,
                        "inode": 456,
                    }
                },
            }
            ordinary: host_admission.AllocationRecord = {
                **native,
                "pid": os.getpid(),
                "start": "parent-start",
                "units": {
                    ordinary_unit: {
                        "group": "/ordinary",
                        "invocation": "d" * 32,
                        "inode": 123,
                    }
                },
            }
            with (
                patch.object(self, "signal_fixture_entry"),
                patch.object(self, "settle_fixture_scope") as settle,
                patch.object(operation, "drained", return_value=drain_fixture(True)),
                patch.object(host_admission, "boot", return_value="fixture-boot"),
                patch.object(
                    host_admission,
                    "readonly_snapshot",
                    return_value={
                        "version": 1,
                        "owners": {
                            allocation.nonce: ordinary,
                            "f" * 32: native,
                            "0" * 32: {**native, "pid": entry["pid"] + 1},
                        },
                    },
                ),
                patch.object(
                    host_admission.Allocation, "release", return_value=False
                ) as release,
                self.assertRaisesRegex(AssertionError, "Retained fixture cleanup"),
            ):
                self.settle_entry_fixture(base, ordinary_unit, allocation)
            self.assertEqual(settle.call_count, 2)
            self.assertEqual(release.call_count, 2)
            self.assertNotIn(base, self.settled_entry_fixtures)
            evidence = json.loads((base / "entry-cleanup.json").read_text())
            self.assertEqual(evidence["allocations"], {"f" * 32: native})
            self.assertEqual(evidence["entry"], entry)
            self.assertEqual(
                evidence["operations"][str(base / ".operations/owner.json")],
                operation_record,
            )

    def kill_scope(self, unit: str) -> None:
        self.assert_parent_scope_unchanged()
        self.assertIn(
            unit, self.fixture_units, "only a minted fixture scope may be killed"
        )
        if self.fixture_parent_scope is not None:
            self.assertNotEqual(unit, self.fixture_parent_scope["unit"])
        observed = operation.unit_observation(unit)
        if observed["LoadState"] == "not-found":
            return  # A collected fixture has no remaining children to signal.
        self.assertEqual(observed["LoadState"], "loaded")
        if observed["ControlGroup"] == "":
            return  # An inactive fixture no longer owns a cgroup to signal.
        self.assert_fixture_scope(
            {
                "unit": unit,
                "group": observed["ControlGroup"],
                "invocation": observed["InvocationID"],
            },
            live=False,
        )
        subprocess.run(
            [
                "systemctl",
                "--user",
                "kill",
                "--kill-whom=all",
                "--signal=SIGKILL",
                unit,
            ],
            env=surreal_server.systemd_environment(),
            check=False,
            capture_output=True,
            timeout=10,
        )

    def builder(self, stage: Path, _work: Path) -> None:
        (stage / "lib").mkdir()
        (stage / "lib/a").write_bytes(b"archive")
        (stage / "include").mkdir()
        (stage / "include/a.h").write_bytes(b"interface")

    def test_new_operation_verifies_bytes_nested_use_reuses_admission(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(
                operation,
                "scope_owner",
                return_value={
                    "unit": "pse-native-" + "a" * 32 + ".scope",
                    "group": "/scope",
                    "invocation": "b" * 32,
                },
            ),
            patch.object(operation, "populated", return_value=True),
        ):
            base = Path(directory)
            with (
                operation.Operation(base),
                patch.object(
                    cache, "installation_files", wraps=cache.installation_files
                ) as inventory,
            ):
                first = cache.prepare(
                    base, "fixture", {"id": 1}, ("lib/a", "include/a.h"), self.builder
                )
                count = inventory.call_count
                self.assertEqual(
                    cache.prepare(
                        base,
                        "fixture",
                        {"id": 1},
                        ("lib/a", "include/a.h"),
                        self.builder,
                    ),
                    first,
                )
                self.assertEqual(inventory.call_count, count)
            (first / "include/a.h").chmod(0o644)
            (first / "include/a.h").write_bytes(b"corrupt interface")
            with operation.Operation(base):
                replacement = cache.prepare(
                    base, "fixture", {"id": 1}, ("lib/a", "include/a.h"), self.builder
                )
            self.assertNotEqual(first, replacement)
            self.assertEqual((first / "include/a.h").read_bytes(), b"corrupt interface")
            self.assertTrue(
                cache.valid(replacement, {"id": 1}, ("lib/a", "include/a.h"))
            )

    def test_existing_reader_does_not_take_construction_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            expected = cache.prepare(
                base, "fixture", {"id": 1}, ("lib/a",), self.builder
            )
            owner = cache.identity_location(base, "fixture", {"id": 1})
            with (owner / ".build.lock").open("a") as construction:
                fcntl.flock(construction, fcntl.LOCK_EX)
                self.assertEqual(
                    cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder),
                    expected,
                )

    def test_existing_writable_generation_is_sealed_without_rebuild(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            prefix = cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder)
            original = cache.installation_files(prefix)
            for path in (*prefix.rglob("*"), prefix):
                path.chmod(path.stat().st_mode | 0o222)
            self.assertTrue(cache.valid(prefix, {"id": 1}, ("lib/a",)))
            with patch.object(
                self,
                "builder",
                side_effect=AssertionError("historical generation rebuilt"),
            ):
                self.assertEqual(
                    cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder),
                    prefix,
                )
            self.assertEqual(cache.installation_files(prefix), original)
            self.assertTrue(
                all(
                    not path.stat().st_mode & 0o222
                    for path in (*prefix.rglob("*"), prefix)
                )
            )
            (prefix / "include/a.h").chmod(0o644)
            with patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(base)}):
                self.assertTrue(cache.admit_external(prefix, ("lib/a",)))
            self.assertFalse((prefix / "include/a.h").stat().st_mode & 0o222)
            self.assertEqual(cache.installation_files(prefix), original)

    def test_manual_shell_exports_verify_each_use_without_fast_authority(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)

            def solver(selected: Path) -> Path:
                return cache.prepare(
                    selected, "fixture", {"id": 1}, ("lib/a",), self.builder
                )

            with (
                patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(base)}, clear=True),
                patch.object(operation, "scope_owner", return_value=None),
                patch.object(cache, "SOLVER_FILES", ("lib/a",)),
                patch.object(cache, "solver", side_effect=solver),
                patch.object(cache, "runtime_env", return_value={}),
                patch.object(
                    build_environment, "configure", side_effect=lambda _root, env: env
                ),
                patch.object(
                    cache, "installation_files", wraps=cache.installation_files
                ) as inventory,
                patch.object(
                    sys,
                    "argv",
                    ["native_operation", "--capabilities", "solver", "--shell"],
                ),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(operation.main(), 0)
                count = inventory.call_count
                self.assertEqual(operation.main(), 0)
                self.assertGreater(inventory.call_count, count)
                self.assertIsNone(operation.owner_record())
                self.assertIsNone(operation.remembered("capability:solver"))
            guards = [
                json.loads(path.read_text())
                for path in (base / ".operations").glob("*.json")
            ]
            self.assertTrue(guards)
            self.assertTrue(
                all(
                    record["scope"] is None and record["generations"]
                    for record in guards
                )
            )

    def test_manual_math_source_requests_only_klu_and_isolation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            interpreter = Path(directory) / "setup-python"
            arguments = Path(directory) / "arguments"
            interpreter.write_text(
                '#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$NATIVE_FIXTURE_ARGUMENTS"\n'
            )
            interpreter.chmod(0o755)
            subprocess.run(
                ["bash", "-c", "source scripts/native-math-env.sh"],
                cwd=cache.ROOT,
                env={
                    **os.environ,
                    "PSE_NATIVE_SETUP_PYTHON": str(interpreter),
                    "NATIVE_FIXTURE_ARGUMENTS": str(arguments),
                },
                check=True,
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(
                arguments.read_text().splitlines()[1:],
                ["--capabilities", "klu,isolation", "--shell"],
            )

    def test_managed_primary_service_requires_actual_group_and_invocation(self) -> None:
        unit = "pse-surreal-worker-" + "a" * 16 + "-0.service"
        group = "/user.slice/pse-reference-fixture.slice/" + unit
        observation = {
            "LoadState": "loaded",
            "ControlGroup": group,
            "InvocationID": "b" * 32,
        }
        with (
            patch.object(
                operation, "process_group", return_value=group + "/native-child"
            ),
            patch.object(operation, "unit_observation", return_value=observation),
        ):
            self.assertEqual(
                operation.scope_owner(),
                {"unit": unit, "group": group, "invocation": "b" * 32},
            )
        for invalid in (
            {**observation, "ControlGroup": "/another"},
            {**observation, "InvocationID": "unknown"},
        ):
            with (
                patch.object(operation, "process_group", return_value=group),
                patch.object(operation, "unit_observation", return_value=invalid),
            ):
                self.assertIsNone(operation.scope_owner())

    def test_marker_requires_actual_scope_membership_and_generation(self) -> None:
        unit = "pse-native-" + "a" * 32 + ".scope"
        owner = {"unit": unit, "group": "/user.slice/" + unit, "invocation": "b" * 32}
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=owner),
            operation.Operation(Path(directory)) as active,
        ):
            self.assertEqual(operation.current(), active.path)
            operation.admit("receipt", {"value": 1})
            with patch.object(
                operation,
                "scope_owner",
                return_value={**owner, "invocation": "c" * 32},
            ):
                self.assertIsNone(operation.current())
                self.assertIsNone(operation.remembered("receipt"))
            with patch.object(operation, "scope_owner", return_value=None):
                self.assertIsNone(operation.current())
        with (
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(
                operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": owner["group"],
                    "InvocationID": owner["invocation"],
                },
            ),
        ):
            self.assertEqual(operation.scope_owner(), owner)
        with (
            patch.object(operation, "process_group", return_value=owner["group"]),
            patch.object(
                operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": "/other",
                    "InvocationID": owner["invocation"],
                },
            ),
        ):
            self.assertIsNone(operation.scope_owner())

    def test_parent_death_populated_scope_stays_pinned_unknown_owner_retained(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            first = cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder)
            (first / "lib/a").chmod(0o644)
            (first / "lib/a").write_bytes(b"changed")
            cache.prepare(base, "fixture", {"id": 1}, ("lib/a",), self.builder)
            records = base / ".operations"
            for record in records.glob("*.json"):
                record.unlink()
            owner = {
                "unit": "pse-native-" + "a" * 32 + ".scope",
                "group": "/scope",
                "invocation": "b" * 32,
            }
            operation.write_json(
                records / "dead-parent.json",
                {
                    "version": operation.VERSION,
                    "pid": 999999999,
                    "start": "dead",
                    "scope": owner,
                    "admissions": {},
                    "generations": [str(first)],
                },
            )
            with patch.object(operation, "populated", return_value=True):
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
            with patch.object(
                operation, "populated", side_effect=ValueError("unavailable")
            ):
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
            with (
                patch.object(operation, "populated", return_value=False),
                patch.object(
                    operation,
                    "unit_observation",
                    return_value={"LoadState": "not-found"},
                ),
            ):
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [first])

    def test_external_lookalike_receipt_never_grants_owned_reuse(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            prefix = base / "lookalike/generations/fake"
            prefix.mkdir(parents=True)
            self.builder(prefix, base)
            (prefix / ".complete.json").write_text(
                json.dumps(
                    {
                        "version": cache.RECEIPT_VERSION,
                        "identity": {"id": 1},
                        "files": cache.installation_files(prefix),
                    }
                )
            )
            with (
                operation.Operation(base),
                patch.object(
                    cache, "installation_files", wraps=cache.installation_files
                ) as inventory,
            ):
                self.assertFalse(cache.admit_external(prefix, ("lib/a",)))
                self.assertFalse(cache.admit_external(prefix, ("lib/a",)))
                self.assertEqual(inventory.call_count, 2)
                (prefix / "include/a.h").write_bytes(b"changed")
                with self.assertRaisesRegex(ValueError, "changed inside"):
                    cache.admit_external(prefix, ("lib/a",))

    def test_highs_candidate_uses_complete_cargo_association_before_build(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            high = base / "actual-cargo-out"
            (high / "lib").mkdir(parents=True)
            (high / "include/highs").mkdir(parents=True)
            (high / "lib/libhighs.a").write_bytes(b"actual archive")
            (high / "include/highs/Highs.h").write_bytes(b"actual header")
            env = {"PSE_NATIVE_CACHE": str(base)}
            inputs = {"unit": "actual locked feature profile"}
            expected = pipeline.highs_files(high)
            env["PSE_NATIVE_PROVIDER_RECEIPT"] = str(base / "reviewed-provider.json")
            with (
                patch.object(
                    pipeline,
                    "_build_highs_archive",
                    return_value=(high, "registry#highs-sys@1.15.0"),
                ) as build,
                patch.object(
                    producer_deployment,
                    "verify_native_provider",
                    side_effect=lambda *_: pipeline.highs_files(high) == expected,
                    create=True,
                ) as verify,
            ):
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(build.call_count, 1)
                self.assertEqual(
                    verify.call_args.args[:3],
                    (
                        Path(env["PSE_NATIVE_PROVIDER_RECEIPT"]),
                        "registry#highs-sys@1.15.0",
                        high,
                    ),
                )
                (high / "include/highs/Highs.h").write_bytes(b"changed header")
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(build.call_count, 2)
                del env["PSE_NATIVE_PROVIDER_RECEIPT"]
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(pipeline._highs_candidate(env, inputs), high)  # noqa: SLF001 -- control the provider candidate-before-build boundary directly
                self.assertEqual(
                    build.call_count,
                    4,
                    "unqualified dev existence cannot grant persistent reuse",
                )

    def test_child_scope_handoff_protects_launch_window_and_new_invocation(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            unit = "pse-surreal-worker-" + "a" * 16 + "-0.scope"
            with operation.Operation(base):
                generation = cache.prepare(
                    base, "fixture", {"id": 1}, ("lib/a",), self.builder
                )
                handoff = operation.prepare_handoff(unit)
            self.assertIsNotNone(handoff)
            self.assertEqual(
                operation._record(handoff)["generations"],  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
                [str(generation)],
            )
            self.assertFalse(operation.drained(operation._record(handoff)).complete)  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
            child_owner = {"unit": unit, "group": "/worker", "invocation": "b" * 32}
            with patch.object(operation, "scope_owner", return_value=child_owner):
                operation.bind_handoff(handoff)
            self.assertEqual(operation._record(handoff)["scope"], child_owner)  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
            with patch.object(operation, "populated", return_value=True):
                self.assertFalse(operation.drained(operation._record(handoff)).complete)  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
            with (
                patch.object(
                    operation,
                    "scope_owner",
                    return_value={**child_owner, "invocation": "c" * 32},
                ),
                self.assertRaisesRegex(ValueError, "differs"),
            ):
                operation.bind_handoff(handoff)

    def test_registration_handoff_requires_exact_owner_and_child_inode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            pending = base / "worker-admission-0.json"
            operation.write_json(pending, {"nonce": "selected"})
            unit = "pse-surreal-worker-" + "a" * 16 + "-0.scope"
            with (
                patch.object(operation, "owner_record", return_value=None),
                patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(base)}),
            ):
                with self.assertRaisesRegex(ValueError, "changed"):
                    operation.prepare_handoff(
                        unit, reservation=(pending, "predecessor")
                    )
                handoff = operation.prepare_handoff(
                    unit, reservation=(pending, "selected")
                )
            assert handoff is not None
            scope = {"unit": unit, "group": "/owned", "invocation": "b" * 32}
            with (
                patch.object(operation, "scope_owner", return_value=scope),
                patch.object(host_admission, "group_identity", return_value=None),
                self.assertRaisesRegex(ValueError, "generation"),
            ):
                operation.bind_handoff(handoff)
            self.assertIsNone(operation._record(handoff)["scope"])  # noqa: SLF001 -- inspect child-owned proof before publication
            with (
                patch.object(operation, "scope_owner", return_value=scope),
                patch.object(host_admission, "group_identity", return_value=17),
            ):
                operation.bind_handoff(handoff)
            record = operation._record(handoff)  # noqa: SLF001 -- inspect published child ownership proof
            self.assertEqual(record["scope_inode"], 17)
            self.assertTrue(operation.registration_pending(record))
            operation.write_json(pending, {"nonce": "replacement"})
            self.assertFalse(operation.registration_pending(record))

    def test_handoff_clock_checks_after_lock_acquisition_and_before_publication(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            parent = base / "parent.json"
            operation.write_json(
                parent,
                {
                    "version": operation.VERSION,
                    "scope": None,
                    "admissions": {},
                    "generations": [],
                },
            )
            original_mask = signal.pthread_sigmask(signal.SIG_BLOCK, set())
            with (
                patch.object(operation.time, "monotonic", side_effect=[10, 10, 12]),
                self.assertRaises(operation.DeadlineExpiredError),
                operation.record_lock(parent, deadline=11),
            ):
                self.fail("expired lock acquisition must not enter the protected body")
            self.assertEqual(
                signal.pthread_sigmask(signal.SIG_BLOCK, set()), original_mask
            )
            clock = [10.0]

            def expire() -> object:
                clock[0] = 12
                return type("Nonce", (), {"hex": "late"})()

            with (
                patch.object(operation, "owner_record", return_value=parent),
                patch.object(operation.time, "monotonic", side_effect=lambda: clock[0]),
                patch.object(operation.uuid, "uuid4", side_effect=expire),
                self.assertRaises(operation.DeadlineExpiredError),
            ):
                operation.prepare_handoff("owned.scope", deadline=11)
            self.assertFalse(list(base.glob("handoff-*.json")))
            self.assertEqual(
                signal.pthread_sigmask(signal.SIG_BLOCK, set()), original_mask
            )

    def test_handoff_parent_discovery_consumes_original_manager_clock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            operations = base / ".operations"
            operations.mkdir(mode=0o700)
            parent = operations / "parent.json"
            unit = "pse-native-" + "a" * 32 + ".scope"
            scope = {"unit": unit, "group": "/owned/" + unit, "invocation": "b" * 32}
            operation.write_json(
                parent,
                {
                    "version": operation.VERSION,
                    "scope": scope,
                    "admissions": {},
                    "generations": [],
                },
            )
            clock = [10.0]

            def timeout(
                _command: list[str], **kwargs: object
            ) -> subprocess.CompletedProcess[str]:
                self.assertEqual(kwargs["timeout"], 0.25)
                clock[0] = 10.25
                raise subprocess.TimeoutExpired("manager", 0.25)

            with (
                patch.dict(
                    os.environ,
                    {"PSE_NATIVE_CACHE": str(base), operation.MARKER: str(parent)},
                ),
                patch.object(operation, "process_group", return_value=scope["group"]),
                patch.object(operation.time, "monotonic", side_effect=lambda: clock[0]),
                patch.object(
                    operation.subprocess, "run", side_effect=timeout
                ) as manager,
                self.assertRaises(operation.DeadlineExpiredError),
            ):
                operation.prepare_handoff("never.scope", deadline=10.25)
            manager.assert_called_once()
            self.assertFalse(list(operations.glob("handoff-*.json")))

    def test_cancellation_drains_authentic_scope_and_fallback_signals_group(
        self,
    ) -> None:
        child = type("Child", (), {"pid": 12345})()
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/scope",
            "invocation": "b" * 32,
        }
        with (
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(operation.subprocess, "run") as manager,
            patch.object(operation.os, "killpg") as group,
        ):
            operation.cancel_children(child, 15)
            self.assertEqual(manager.call_args.args[0][-1], owner["unit"])
            self.assertIn("--kill-whom=all", manager.call_args.args[0])
            group.assert_called_once_with(child.pid, 15)
        with (
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(operation.subprocess, "run") as manager,
            patch.object(operation.os, "killpg") as group,
        ):
            operation.cancel_children(child, 2)
            manager.assert_not_called()
            group.assert_called_once_with(child.pid, 2)

    def test_foreground_pending_handoff_consumes_parent_cancellation_fence(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            base = Path(directory)
            unit = "pse-native-" + "a" * 32 + ".scope"
            with operation.Operation(base) as parent:
                handoff = operation.prepare_handoff(unit, foreground=True)
                assert handoff is not None
                record = json.loads(parent.path.read_text())
                record["cancelled"] = True
                operation.write_json(parent.path, record)
                self.assertNotIn("cancelled", json.loads(handoff.read_text()))
                child = {"unit": unit, "group": "/foreground", "invocation": "b" * 32}
                with (
                    patch.object(operation, "scope_owner", return_value=child),
                    patch.object(pse_env.host, "group_identity", return_value=17),
                    self.assertRaisesRegex(ValueError, "cancelled"),
                ):
                    operation.bind_handoff(handoff)
                bound = json.loads(handoff.read_text())
                self.assertEqual(bound["scope"], child)
                self.assertEqual(bound["scope_inode"], 17)
                self.assertTrue(bound["cancelled"])
                with self.assertRaisesRegex(ValueError, "parent is cancelled"):
                    operation.prepare_handoff(unit, foreground=True)
                # Persistent primary/worker handoffs remain independently owned.
                primary = operation.prepare_handoff(
                    "pse-surreal-worker-" + "d" * 16 + "-0.service"
                )
                assert primary is not None
                self.assertNotIn("foreground", json.loads(primary.read_text()))

    def test_cancellation_manager_failure_still_signals_direct_payload(self) -> None:
        child = type("Child", (), {"pid": 12345})()
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/scope",
            "invocation": "b" * 32,
        }
        with (
            patch.object(
                operation,
                "cancel_foreground_handoffs",
                side_effect=ValueError("unknown child"),
            ),
            patch.object(operation, "scope_owner", return_value=owner),
            patch.object(
                operation.subprocess,
                "run",
                side_effect=subprocess.TimeoutExpired("systemctl", 10),
            ),
            patch.object(operation.os, "killpg") as group,
            contextlib.redirect_stderr(io.StringIO()) as errors,
        ):
            with self.assertRaisesRegex(ValueError, "cancellation incomplete"):
                operation.cancel_children(child, signal.SIGTERM)
            group.assert_called_once_with(child.pid, signal.SIGTERM)
            self.assertIn("unknown child", errors.getvalue())
            self.assertIn("scope cancellation failed", errors.getvalue())

    def test_creator_completion_restores_markers_when_foreground_drain_fails(
        self,
    ) -> None:
        previous = {
            key: os.environ.get(key)
            for key in (operation.MARKER, "PSE_NATIVE_CACHE", "PSE_NATIVE_SETUP_PYTHON")
        }
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(
                operation,
                "cancel_foreground_handoffs",
                side_effect=ValueError("unknown foreground lifetime"),
            ) as settle,
        ):
            with (
                self.assertRaisesRegex(ValueError, "unknown foreground lifetime"),
                operation.Operation(Path(directory)) as parent,
            ):
                self.assertEqual(os.environ[operation.MARKER], str(parent.path))
            settle.assert_called_once()
        self.assertEqual({key: os.environ.get(key) for key in previous}, previous)

    def test_nested_run_completion_preserves_the_live_creator(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
        ):
            with operation.Operation(Path(directory)) as parent:
                handoff = operation.prepare_handoff(
                    "pse-native-" + "a" * 32 + ".scope", foreground=True
                )
                assert handoff is not None
                with patch.object(operation, "cancel_foreground_handoffs") as settle:
                    self.assertEqual(
                        operation.run([sys.executable, "-c", "pass"], dict(os.environ)),
                        0,
                    )
                    settle.assert_not_called()
                self.assertNotIn("cancelled", json.loads(parent.path.read_text()))
                self.assertNotIn("cancelled", json.loads(handoff.read_text()))
            self.assertTrue(json.loads(handoff.read_text())["cancelled"])

    def test_foreground_cancellation_malformed_associations_refuse_conservatively(
        self,
    ) -> None:
        for children in (17, [None]):
            with (
                self.subTest(children=children),
                tempfile.TemporaryDirectory() as directory,
                patch.object(operation, "scope_owner", return_value=None),
                operation.Operation(Path(directory)) as parent,
            ):
                record = dict(json.loads(parent.path.read_text()))
                record["foreground_children"] = children
                operation.write_json(parent.path, record)
                with patch.object(operation.subprocess, "run") as manager:
                    with self.assertRaises(ValueError):
                        operation.cancel_foreground_handoffs()
                    manager.assert_not_called()
                self.assertTrue(json.loads(parent.path.read_text())["cancelled"])
                # Unknown associations also refuse creator finalization.
                record["foreground_children"] = []
                operation.write_json(parent.path, record)

    def test_foreground_cancellation_fences_every_child_and_refuses_replacements(
        self,
    ) -> None:
        for replacement in (
            "invocation",
            "group",
            "inode",
            "missing",
            "missing-scope",
            "invalid-scope",
        ):
            with (
                self.subTest(replacement=replacement),
                tempfile.TemporaryDirectory() as directory,
                patch.object(operation, "scope_owner", return_value=None),
            ):
                base = Path(directory)
                stopped = []
                with (
                    self.assertRaisesRegex(ValueError, "cancellation incomplete")
                    if replacement in {"missing", "missing-scope", "invalid-scope"}
                    else contextlib.nullcontext(),
                    operation.Operation(base),
                ):
                    paths = []
                    owners = []
                    for letter in ("a", "b"):
                        unit = "pse-native-" + letter * 32 + ".scope"
                        path = operation.prepare_handoff(unit, foreground=True)
                        assert path is not None
                        child = {
                            "unit": unit,
                            "group": "/" + letter,
                            "invocation": letter * 32,
                        }
                        with (
                            patch.object(operation, "scope_owner", return_value=child),
                            patch.object(
                                pse_env.host, "group_identity", return_value=17
                            ),
                        ):
                            operation.bind_handoff(path)
                        paths.append(path)
                        owners.append(child)
                    durable = operation.prepare_handoff(
                        "pse-surreal-worker-" + "d" * 16 + "-0.service"
                    )
                    assert durable is not None
                    if replacement == "missing":
                        paths[0].unlink()
                    elif replacement in {"missing-scope", "invalid-scope"}:
                        record = dict(json.loads(paths[0].read_text()))
                        if replacement == "missing-scope":
                            del record["scope"]
                        else:
                            record["scope"] = 17
                        operation.write_json(paths[0], record)

                    def observe(
                        unit: str,
                        owners: list[dict[str, str]] = owners,
                        replacement: str = replacement,
                    ) -> dict[str, str]:
                        selected = next(
                            owner for owner in owners if owner["unit"] == unit
                        )
                        result = {
                            "LoadState": "loaded",
                            "ActiveState": "active",
                            "ControlGroup": selected["group"],
                            "InvocationID": selected["invocation"],
                        }
                        if unit == owners[0]["unit"] and replacement in {
                            "invocation",
                            "group",
                        }:
                            result[
                                "InvocationID"
                                if replacement == "invocation"
                                else "ControlGroup"
                            ] = "replaced"
                        return result

                    def stop(
                        command: list[str],
                        paths: list[Path] = paths,
                        stopped: list[str] = stopped,
                        **_kwargs: object,
                    ) -> None:
                        self.assertTrue(
                            all(
                                json.loads(path.read_text())["cancelled"]
                                for path in paths
                                if path.exists()
                            )
                        )
                        stopped.append(command[-1])

                    with (
                        patch.object(
                            operation,
                            "drained",
                            side_effect=lambda record, stopped=stopped: drain_fixture(
                                record["scope"]["unit"] in stopped
                            ),
                        ),
                        patch.object(
                            operation, "unit_observation", side_effect=observe
                        ),
                        patch.object(
                            pse_env.host,
                            "group_identity",
                            side_effect=lambda group, replacement=replacement: (
                                18 if replacement == "inode" and group == "/a" else 17
                            ),
                        ),
                        patch.object(operation.subprocess, "run", side_effect=stop),
                        self.assertRaisesRegex(ValueError, "cancellation incomplete"),
                    ):
                        operation.cancel_foreground_handoffs()
                    self.assertEqual(stopped, [owners[1]["unit"]])
                    self.assertNotIn("cancelled", json.loads(durable.read_text()))

    def test_foreground_handoff_collector_retains_binding_until_parent_retires(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(
                operation,
                "drained",
                side_effect=lambda record: drain_fixture(record["scope"] is not None),
            ),
        ):
            base = Path(directory)
            with operation.Operation(base) as parent:
                path = operation.prepare_handoff(
                    "pse-native-" + "a" * 32 + ".scope", foreground=True
                )
                assert path is not None
                record = json.loads(path.read_text())
                record["scope"] = {
                    "unit": record["handoff"],
                    "group": "/child",
                    "invocation": "b" * 32,
                }
                record["generations"] = ["settled-child-only"]
                operation.write_json(path, record)
                self.assertEqual(operation.pinned_generations(base), set())
                self.assertTrue(path.exists())
                self.assertTrue(parent.path.exists())
            # The authentic creator settles its cancellation fence before its
            # retirement is observed. Deleting a live parent's record is loss
            # of authority, not a successful collector transition.
            self.assertTrue(json.loads(path.read_text())["cancelled"])
            parent.path.unlink()
            self.assertEqual(operation.pinned_generations(base), set())
            self.assertFalse(path.exists())

    def test_missing_live_creator_record_refuses_exit_and_restores_markers(
        self,
    ) -> None:
        previous = {
            key: os.environ.get(key)
            for key in (operation.MARKER, "PSE_NATIVE_CACHE", "PSE_NATIVE_SETUP_PYTHON")
        }
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(operation, "cancel_foreground_handoffs") as settle,
            self.assertRaises(FileNotFoundError),
            operation.Operation(Path(directory)) as parent,
        ):
            parent.path.unlink()
        settle.assert_not_called()
        self.assertEqual({key: os.environ.get(key) for key in previous}, previous)

    def test_record_exclusion_defers_cancellation_until_flock_is_released(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "operation.json"
            received = []

            def cancel(_signum: int, _frame: object) -> None:
                with operation.record_lock(path):
                    received.append("cancelled")

            previous = signal.signal(signal.SIGTERM, cancel)
            try:
                with operation.record_lock(path):
                    os.kill(os.getpid(), signal.SIGTERM)
                    self.assertEqual(received, [])
                self.assertEqual(received, ["cancelled"])
            finally:
                signal.signal(signal.SIGTERM, previous)

    def test_thread_budget_defaults_to_one_and_honours_the_caller(self) -> None:
        defaulted = operation.environment([], {})
        self.assertEqual(
            {name: defaulted[name] for name in operation.THREAD_VARIABLES},
            dict.fromkeys(operation.THREAD_VARIABLES, "1"),
        )
        chosen = operation.environment(
            [], {"OMP_NUM_THREADS": "4", "MKL_NUM_THREADS": "off"}
        )
        self.assertEqual(chosen["OMP_NUM_THREADS"], "4")
        self.assertEqual(chosen["OPENBLAS_NUM_THREADS"], "1")
        self.assertNotIn("MKL_NUM_THREADS", chosen)

    def test_off_survives_nested_native_setup_until_a_value_is_chosen(self) -> None:
        outer = operation.environment([], {"OMP_NUM_THREADS": "off"})
        self.assertNotIn("OMP_NUM_THREADS", outer)
        nested = operation.environment([], outer)
        self.assertNotIn("OMP_NUM_THREADS", nested)
        self.assertEqual(nested["OPENBLAS_NUM_THREADS"], "1")
        chosen = operation.environment([], {**nested, "OMP_NUM_THREADS": "6"})
        self.assertEqual(chosen["OMP_NUM_THREADS"], "6")
        self.assertNotIn(operation.OFF_MARKER, chosen)

    def test_solver_runtime_enforces_owned_keys_and_defaults_the_rest(self) -> None:
        runtime = {
            "MKL_CBWR": "COMPATIBLE",
            "OMP_PLACES": "sockets",
            "OMP_NUM_THREADS": "1",
        }
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(cache, "solver", return_value=Path(directory)),
            patch.object(cache, "runtime_env", return_value=runtime),
            patch("sys.stderr", new_callable=io.StringIO) as notices,
        ):
            result = operation.environment(
                ["solver"],
                {"MKL_CBWR": "AUTO", "OMP_PLACES": "cores", "OMP_NUM_THREADS": "8"},
            )
        self.assertEqual(result["MKL_CBWR"], "COMPATIBLE")
        self.assertEqual(result["OMP_PLACES"], "cores")
        self.assertEqual(result["OMP_NUM_THREADS"], "8")
        self.assertEqual(result["SCIPOPTDIR"], directory)
        self.assertIn("pse-env: refused MKL_CBWR=AUTO", notices.getvalue())
        self.assertNotIn("OMP_PLACES", notices.getvalue())

    def test_signal_death_reports_shell_status(self) -> None:
        self.assertEqual(
            operation.run(["sh", "-c", "kill -TERM $$"], dict(os.environ)),
            128 + signal.SIGTERM,
        )

    def test_build_timing_observes_authentic_setup_child_and_scope_drain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            environment = self.fixture_environment(output / "cache")
            environment[operation.BUILD_TIMING] = str(output / "native-operation.json")
            program = "from scripts import native_operation as n; import os; assert n.current() is not None; assert n.BUILD_TIMING not in os.environ"
            command = self.fixture_command(
                [
                    sys.executable,
                    "-m",
                    "scripts.native_operation",
                    "--capabilities",
                    "",
                    "--",
                    sys.executable,
                    "-c",
                    program,
                ],
                environment,
            )
            started = time.monotonic()
            result = subprocess.run(
                command, cwd=cache.ROOT, env=environment, check=False
            )
            native = json.loads((output / "native-operation.json").read_text())
            self.assertEqual(result.returncode, 0)
            self.assertEqual(native["requested_capabilities"], [])
            self.assert_fixture_scope(native["owner"]["scope"], live=False)
            # Mirror pse-env's outer launcher: the exact allocation release
            # verifies its units before native ownership proves terminal drain.
            allocation = pse_env.host.inherit(environment)
            self.assertIsNotNone(allocation)
            assert allocation is not None
            self.assertTrue(allocation.release())
            self.await_drain(native["owner"])
            self.assertEqual(
                build_measurements.complete_operation(output, started, 0), 0
            )
            receipt = json.loads((output / "complete-operation.json").read_text())
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["phases"]["final_drain"]["status"], "passed")
            self.assertGreaterEqual(receipt["wall_seconds"], 0)
            self.assertEqual(native["setup_status"], "passed")
            self.assertEqual(native["child_exit_code"], 0)

    def test_actual_zero_capability_entry_owns_child_before_command(self) -> None:
        program = "from scripts import native_operation as n; import json; p=n.owner_record(); print(json.dumps({'active':p is not None,'scope':n._record(p)['scope'],'args':__import__('sys').argv[1:]}))"
        with tempfile.TemporaryDirectory() as directory:
            environment = self.fixture_environment(Path(directory))
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native=",
                    "--",
                    sys.executable,
                    "-c",
                    program,
                    "space ; literal",
                    "$literal",
                ],
                environment,
            )
            result = subprocess.run(
                command,
                cwd=cache.ROOT,
                env=environment,
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
        observed = json.loads(result.stdout)
        self.assertTrue(observed["active"])
        self.assertEqual(observed["args"], ["space ; literal", "$literal"])
        self.assertIsNotNone(observed["scope"])
        self.assert_fixture_scope(observed["scope"], live=False)
        self.assertTrue(observed["scope"]["unit"].startswith("pse-native-"))
        self.assertEqual(len(observed["scope"]["invocation"]), 32)
        self.await_drain({"scope": observed["scope"]})

    def test_real_selected_terminal_survives_native_scope_signal_and_opaque_child_drain(
        self,
    ) -> None:
        self.check_real_selected_signal("composer")

    def test_real_selected_terminal_survives_outer_native_owner_signal(self) -> None:
        self.check_real_selected_signal("owner")

    def test_real_selected_terminal_survives_outer_pse_env_entry_signal(self) -> None:
        self.check_real_selected_signal("entry")

    def test_real_entry_cancellation_without_inherited_manager_environment(
        self,
    ) -> None:
        self.check_real_selected_signal("entry", inherited_manager=False)

    def test_real_uninterrupted_selected_composer_owns_one_durable_terminal(
        self,
    ) -> None:
        with contextlib.nullcontext(
            tempfile.mkdtemp(prefix="pse36-e2-actual-")
        ) as directory:
            base = Path(directory)
            environment = self.fixture_environment(base)
            environment["PSE_TEST_RESOURCE_STATE"] = str(base / "registry")
            junit = '<testsuite><testcase classname="control.py" name="test_case"><properties><property name="nodeid" value="control.py::test_case"/></properties></testcase></testsuite>'
            payload = f"import sys; from pathlib import Path; Path(sys.argv[1]).write_text({junit!r})"
            composer = "\n".join(
                [
                    "import os,sys; from pathlib import Path",
                    "from scripts import native_operation as n,test_run,test_resources as r,host_admission as h",
                    "base=Path(sys.argv[1]); path=r.invocation('python',[{'name':'control.py::test_case'}])",
                    f"code=test_run.execute_selected(path,[sys.executable,'-c',{payload!r},str(path.with_suffix('.xml'))],os.environ,path.with_suffix('.xml'),selected=[{{'nodeid':'control.py::test_case'}}])",
                    "n.write_json(base/'completed.json',{'code':code,'scope':n.scope_owner(),'operation':n._record(n.owner_record()),'invocation':str(path),'allocation':os.environ[h.MARKER]})",
                    "raise SystemExit(code)",
                ]
            )
            command = self.entry_fixture_command(
                base,
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native=",
                    "--",
                    sys.executable,
                    "-c",
                    composer,
                    str(base),
                ],
                environment,
            )
            result = subprocess.run(
                command,
                cwd=cache.ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            completed = self.await_file(base / "completed.json")
            entry = self.await_file(base / "entry-launcher.json")
            self.assert_fixture_scope(entry["scope"], live=False)
            self.assertNotEqual(entry["allocation"], completed["allocation"])
            path = Path(completed["invocation"])
            evidence = json.loads(path.with_suffix(".terminal.json").read_text())
            self.assertEqual(evidence["exit_code"], 0)
            self.assertEqual(evidence["report_errors"], [])
            self.assertEqual(evidence["process"]["owned_drain"], "complete")
            state = json.loads(
                (base / "registry/invocation-state" / path.name).read_text()
            )
            self.assertTrue(state["terminal"]["complete"])
            registrations = completed["operation"]["selected_composers"]
            self.assertEqual(list(registrations), [path.stem])
            self.assertEqual(registrations[path.stem]["phase"], "complete")
            owners = [completed["scope"], evidence["process"]["scope"]]
            for owner in owners:
                self.fixture_units.add(owner["unit"])
                self.assert_fixture_scope(owner, live=False)
                self.await_drain({"scope": owner})
            allocation = Path(completed["allocation"])
            with pse_env.host.allocation_metadata(allocation.parent) as ledger:
                self.assertNotIn(allocation.name, ledger["owners"])
            self.await_drain({"scope": entry["scope"]})
            outer_allocation = pse_env.host.inherit(environment)
            assert outer_allocation is not None
            self.assertTrue(outer_allocation.release())
            self.assert_parent_scope_unchanged()

    def test_real_unacknowledged_composer_is_killed_without_fabricated_terminal(
        self,
    ) -> None:
        with contextlib.nullcontext(
            tempfile.mkdtemp(prefix="pse36-e2-actual-")
        ) as directory:
            base = Path(directory)
            environment = self.fixture_environment(base)
            environment["PSE_TEST_RESOURCE_STATE"] = str(base / "registry")
            composer = "\n".join(  # noqa: FLY002 -- retain separate executable fault-injection statements
                [
                    "import signal,sys,time; from pathlib import Path",
                    "from scripts import native_operation as n,test_resources as r",
                    "signal.signal(signal.SIGINT,signal.SIG_IGN); signal.signal(signal.SIGTERM,signal.SIG_IGN)",
                    # This control deliberately leaves no terminal acknowledgment.
                    # Real shared-host guard retention is tested in a private ledger
                    # so this disposable scope cannot strand host admission.
                    "n.register_terminal_guard=lambda *_args,**_kwargs:None",
                    "base=Path(sys.argv[1]); path=r.invocation('python',[]); n.register_composer(path)",
                    "n.write_json(base/'ready.json',{'scope':n.scope_owner(),'operation':str(n.owner_record()),'owner':n._record(n.owner_record()),'invocation':str(path)})",
                    "time.sleep(30)",
                ]
            )
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native=",
                    "--",
                    sys.executable,
                    "-c",
                    composer,
                    str(base),
                ],
                environment,
            )
            with (base / "fixture.log").open("w") as log:
                process = subprocess.Popen(
                    command,
                    cwd=cache.ROOT,
                    env=environment,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                )
                owner = None
                try:
                    selected = self.await_file(base / "ready.json", process)
                    owner = selected["scope"]
                    self.assert_fixture_scope(owner)
                    started = time.monotonic()
                    os.kill(selected["owner"]["pid"], signal.SIGTERM)
                    process.wait(timeout=15)
                    self.assertLess(time.monotonic() - started, 12)
                    normalized = (
                        128 - process.returncode
                        if process.returncode < 0
                        else process.returncode
                    )
                    self.assertEqual(
                        normalized,
                        128 + signal.SIGKILL,
                        (base / "fixture.log").read_text(),
                    )
                    self.assertIn(
                        "acknowledgment unresolved", (base / "fixture.log").read_text()
                    )
                    self.await_drain({"scope": owner})
                    path = Path(selected["invocation"])
                    self.assertFalse(path.with_suffix(".terminal.json").exists())
                    state = json.loads(
                        (base / "registry/invocation-state" / path.name).read_text()
                    )
                    self.assertEqual(state["phase"], "open")
                    record = operation._record(Path(selected["operation"]))  # noqa: SLF001 -- assert the exact durable private operation protocol
                    self.assertEqual(
                        record["selected_composers"][path.stem]["phase"], "registered"
                    )
                    self.assert_parent_scope_unchanged()
                finally:
                    if owner is not None:
                        self.kill_scope(owner["unit"])
                    if process.poll() is None:
                        process.kill()
                    process.wait(timeout=10)

    def check_real_selected_signal(
        self, target: str, *, inherited_manager: bool = True
    ) -> None:
        for signum in (signal.SIGINT, signal.SIGTERM):
            with (
                self.subTest(signal=signum),
                contextlib.nullcontext(
                    tempfile.mkdtemp(prefix="pse36-e2-actual-")
                ) as directory,
            ):
                base = Path(directory)
                environment = self.fixture_environment(base)
                environment["PSE_TEST_RESOURCE_STATE"] = str(base / "registry")
                opaque = "import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); signal.signal(signal.SIGINT,signal.SIG_IGN); time.sleep(30)"
                workload = "\n".join(
                    [
                        "import os,signal,subprocess,sys,time; from pathlib import Path",
                        "from scripts import native_operation as n",
                        "base=Path(sys.argv[1])",
                        "signal.signal(signal.SIGTERM,signal.SIG_IGN); signal.signal(signal.SIGINT,signal.SIG_IGN)",
                        f"opaque=subprocess.Popen([sys.executable,'-c',{opaque!r}],start_new_session=True)",
                        "n.write_json(base/'workload-ready.json',{'record':n._record(n.owner_record()),'opaque':opaque.pid,'session':os.getsid(opaque.pid)})",
                        "time.sleep(30)",
                    ]
                )
                composer = "\n".join(
                    [
                        "import os,sys; from pathlib import Path",
                        "from scripts import native_operation as n,test_run,test_resources as r,host_admission as h",
                        "base=Path(sys.argv[1]); path=r.invocation('python',[{'name':'control.py::test_case'}])",
                        "n.write_json(base/'composer-ready.json',{'scope':n.scope_owner(),'pid':os.getpid(),'owner':n._record(n.owner_record()),'invocation':str(path),'allocation':os.environ[h.MARKER]})",
                        f"code=test_run.execute_selected(path,[sys.executable,'-c',{workload!r},str(base)],os.environ,path.with_suffix('.xml'),selected=[{{'nodeid':'control.py::test_case'}}])",
                        "n.write_json(base/'composer-finished.json',{'code':code})",
                        "raise SystemExit(code)",
                    ]
                )
                command = [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native=",
                    "--",
                    sys.executable,
                    "-c",
                    composer,
                    str(base),
                ]
                if target == "entry":
                    command = self.entry_fixture_command(
                        base,
                        command,
                        environment,
                        inherited_manager=inherited_manager,
                    )
                else:
                    command = self.fixture_command(command, environment)
                fixture_allocation = pse_env.host.inherit(environment)
                assert fixture_allocation is not None
                fixture_unit = next(
                    argument.removeprefix("--unit=")
                    for argument in command
                    if argument.startswith("--unit=")
                )
                log = (base / "fixture.log").open("w")
                process = subprocess.Popen(
                    command,
                    cwd=cache.ROOT,
                    env=environment,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                )
                owners = []
                entry = None
                try:
                    parent = self.await_file(base / "composer-ready.json", process)
                    owners.append(parent["scope"])
                    self.fixture_units.add(parent["scope"]["unit"])
                    child = self.await_file(base / "workload-ready.json", process)
                    owners.append(child["record"]["scope"])
                    if target == "entry":
                        entry = self.await_file(base / "entry-launcher.json", process)
                        self.assert_fixture_scope(entry["scope"])
                        self.assertNotEqual(entry["allocation"], parent["allocation"])
                        self.assertEqual(
                            operation.start_identity(entry["pid"]), entry["start"]
                        )
                        self.assertIn(
                            str(cache.ROOT / "scripts/pse_env.py").encode(),
                            Path(f"/proc/{entry['pid']}/cmdline")
                            .read_bytes()
                            .split(b"\0"),
                        )
                    self.fixture_units.add(owners[1]["unit"])
                    for owner in owners:
                        self.assert_fixture_scope(owner)
                    self.assertNotEqual(owners[0], owners[1])
                    self.assertEqual(child["opaque"], child["session"])
                    self.assertEqual(
                        operation.process_group(child["opaque"]), owners[1]["group"]
                    )
                    if target == "composer":
                        recipient = parent["pid"]
                    elif target == "entry":
                        assert entry is not None
                        recipient = entry["pid"]
                    else:
                        recipient = parent["owner"]["pid"]
                    os.kill(recipient, signum)
                    process.wait(timeout=20)
                    normalized = (
                        128 - process.returncode
                        if process.returncode < 0
                        else process.returncode
                    )
                    self.assertEqual(
                        normalized,
                        128 + signal.SIGKILL if target == "owner" else 128 + signum,
                        (base / "fixture.log").read_text()[-4000:],
                    )
                    if target == "composer":
                        self.assertEqual(
                            self.await_file(base / "composer-finished.json")["code"],
                            128 + signum,
                        )
                    path = Path(parent["invocation"])
                    terminal = json.loads(
                        path.with_suffix(".terminal.json").read_text()
                    )
                    self.assertEqual(terminal["exit_code"], 128 + signum)
                    self.assertEqual(terminal["process"]["forwarded_signal"], signum)
                    self.assertEqual(terminal["process"]["owned_drain"], "complete")
                    self.assertEqual(terminal["process"]["scope"], owners[1])
                    state = json.loads(
                        (base / "registry/invocation-state" / path.name).read_text()
                    )
                    self.assertEqual(state["phase"], "frozen")
                    self.assertFalse(state["terminal"]["complete"])
                    for owner in owners:
                        self.await_drain({"scope": owner})
                    self.assert_parent_scope_unchanged()
                    if target == "entry":
                        assert entry is not None
                        self.await_drain({"scope": entry["scope"]})
                        native_allocation = Path(parent["allocation"])
                        with pse_env.host.allocation_metadata(
                            native_allocation.parent
                        ) as ledger:
                            self.assertNotIn(native_allocation.name, ledger["owners"])
                    allocation = pse_env.host.inherit(environment)
                    assert allocation is not None
                    self.assertTrue(
                        allocation.release(),
                        "retain fixture evidence until exact allocation release",
                    )
                finally:
                    try:
                        if target == "entry":
                            self.settle_entry_fixture(
                                base, fixture_unit, fixture_allocation
                            )
                        else:
                            with contextlib.ExitStack() as cleanup:
                                for owner in owners:
                                    cleanup.callback(self.kill_scope, owner["unit"])
                    finally:
                        try:
                            if process.poll() is None:
                                process.kill()
                            process.wait(timeout=10)
                        finally:
                            log.close()

    def test_command_parent_requires_exact_registered_kernel_binding(self) -> None:
        unit = "pse-cmd-" + "a" * 32 + ".scope"
        group = "/allocation/" + unit
        allocation = MagicMock(spec=host_admission.Allocation)
        allocation.directory = Path("/owned/admission")
        allocation.nonce = "c" * 32
        binding = {"group": group, "invocation": "b" * 32, "inode": 17}
        for mode in (
            "bound", "descendant", "unbound-own-pid", "missing", "stale-boot",
            "wrong-group", "wrong-invocation", "wrong-inode", "missing-inode",
            "inactive", "unloaded", "unregistered-unit",
        ):
            with self.subTest(mode=mode):
                selected = dict(binding)
                owner = {
                    "boot": host_admission.boot(),
                    "pid": os.getpid(),
                    "start": operation.start_identity(os.getpid()),
                    "units": {unit: selected},
                }
                observed = {
                    "LoadState": "loaded", "ActiveState": "active",
                    "ControlGroup": group, "InvocationID": "b" * 32,
                }
                if mode == "unbound-own-pid":
                    selected.clear()
                if mode == "missing-inode":
                    selected.pop("inode")
                if mode == "stale-boot":
                    owner["boot"] = "old-boot"
                if mode == "wrong-group":
                    observed["ControlGroup"] = "/other/" + unit
                if mode == "wrong-invocation":
                    observed["InvocationID"] = "d" * 32
                if mode == "inactive":
                    observed["ActiveState"] = "inactive"
                if mode == "unloaded":
                    observed["LoadState"] = "not-found"
                ledger = {"owners": {} if mode == "missing" else {allocation.nonce: owner}}
                actual = (
                    group + "/nested" if mode == "descendant" else
                    "/other/pse-cmd-" + "e" * 32 + ".scope" if mode == "unregistered-unit" else group
                )
                with (
                    patch.object(host_admission, "allocation_metadata", return_value=contextlib.nullcontext(ledger)),
                    patch.object(host_admission, "readonly_snapshot", return_value=ledger),
                    patch.object(operation, "process_group", return_value=actual),
                    patch.object(host_admission, "control_unit_observation", return_value=observed),
                    patch.object(host_admission, "group_identity", return_value=18 if mode == "wrong-inode" else 17),
                ):
                    for deadline in (None, time.monotonic() + 5):
                        if mode in {"bound", "descendant"}:
                            operation.require_command_parent(allocation, deadline=deadline)
                        else:
                            with self.assertRaises(ValueError):
                                operation.require_command_parent(allocation, deadline=deadline)

    def test_command_parent_foreground_record_does_not_grant_native_reuse(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            allocation = MagicMock(spec=host_admission.Allocation)
            records = []
            deadline = time.monotonic() + 5

            def selected(*_args: object, **options: object) -> int:
                record = operation.owner_record()
                self.assertIsNotNone(record)
                assert record is not None
                records.append(record)
                self.assertIsNone(operation.current())
                self.assertIsNone(operation._record(record)["scope"])
                self.assertEqual(options["deadline"], deadline)
                return 37

            with (
                patch.object(host_admission, "inherit", return_value=allocation),
                patch.object(operation, "scope_owner", return_value=None),
                patch.object(operation, "require_command_parent") as proof,
                patch.object(operation, "_run_child_scope", side_effect=selected) as payload,
            ):
                self.assertEqual(operation.run_child_scope(
                    ["payload"], {"PSE_NATIVE_CACHE": str(base)}, deadline=deadline,
                ), 37)
                proof.assert_called_once_with(allocation, deadline=deadline)
                self.assertIsNone(operation.owner_record())
                self.assertTrue(operation._record(records[0])["cancelled"])
                with patch.dict(os.environ, {operation.MARKER: str(base / "forged.json")}):
                    with self.assertRaisesRegex(ValueError, "contradictory operation parent"):
                        operation.run_child_scope(["payload"], {"PSE_NATIVE_CACHE": str(base)})
                self.assertEqual(payload.call_count, 1)

    def test_real_command_parent_file_effect_handoff_drains_descendants(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            environment = self.fixture_environment(base)
            (base / "payload.py").write_text("""import subprocess, sys
from pathlib import Path
from scripts import native_operation as n
child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(.4)'])
n.write_json(Path(sys.argv[1])/'child.json',{'scope':n.scope_owner(),'pid':child.pid})
""")
            program = """import os, subprocess, sys, time
from pathlib import Path
from scripts import native_operation as n, surreal_server as s
base=Path(sys.argv[1]); group=n.process_group(os.getpid())
assert n.scope_owner() is None and n.owner_record() is None
status=s.run_file_effect([sys.executable,'-c',(base/'payload.py').read_text(),str(base)],dict(os.environ),deadline=time.monotonic()+20)
assert n.process_group(os.getpid())==group and n.scope_owner() is None and n.owner_record() is None
expired=False
try:
 s.run_file_effect([sys.executable,'-c','import time; time.sleep(30)'],dict(os.environ),deadline=time.monotonic()+.4)
except subprocess.TimeoutExpired:
 expired=True
assert expired and n.scope_owner() is None and n.owner_record() is None
records=[n._record(p) for p in (base/'.operations').glob('handoff-*.json')]
assert len(records)==2 and all(n.drained(record).complete for record in records)
n.write_json(base/'result.json',{'status':status,'parent_group':group,'handoffs':records})
"""
            command = self.fixture_command(
                [sys.executable, "-c", program, str(base)], environment,
                native=False, cleanup=False,
            )
            allocation = host_admission.inherit(environment)
            assert allocation is not None
            try:
                result = subprocess.run(
                    command, cwd=cache.ROOT, env=environment, capture_output=True,
                    text=True, timeout=35, check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                observed = json.loads((base / "result.json").read_text())
                self.assertEqual(observed["status"], 0)
                self.assertIn("pse-cmd-", observed["parent_group"])
                for handoff in observed["handoffs"]:
                    self.assertNotEqual(handoff["scope"]["group"], observed["parent_group"])
                    self.assertTrue(operation.drained(handoff).complete)
                    self.assertIsNone(operation._record(Path(handoff["parent"]))["scope"])
                self.assertEqual(sum("cancel_deadline" in handoff for handoff in observed["handoffs"]), 1)
                child = json.loads((base / "child.json").read_text())
                self.assertFalse(Path(f"/proc/{child['pid']}").exists())
            finally:
                deadline = time.monotonic() + 20
                ledger = host_admission.readonly_snapshot(allocation.directory, deadline=deadline)
                owner = ledger["owners"].get(allocation.nonce)
                if owner is not None:
                    for unit, binding in owner["units"].items():
                        if binding:
                            self.settle_fixture_scope({
                                "scope": {"unit": unit, "group": binding["group"], "invocation": binding["invocation"]},
                                "scope_inode": binding["inode"],
                            }, deadline=deadline)
                self.assertTrue(allocation.release(deadline=deadline))
            self.assert_parent_scope_unchanged()

    def test_selected_workload_preserves_observer_cap_without_environment_authority(
        self,
    ) -> None:
        allocation = MagicMock(spec=host_admission.Allocation)
        allocation.nonce = "c" * 32
        allocation.profile = host_admission.select("reference")
        selected = {"unit": "child.scope", "group": "/child", "invocation": "b" * 32}

        def completed(
            _command: list[str],
            _environment: dict[str, str],
            *,
            on_reaped: Callable[[], None],
            **_options: object,
        ) -> int:
            on_reaped()
            return 0

        with (
            patch.object(host_admission, "inherit", return_value=allocation),
            patch.object(operation, "scope_owner", return_value={"unit": "parent"}),
            patch.object(pse_env, "current_memory_ceiling", return_value=4 << 30),
            patch.object(operation.time, "monotonic", return_value=100),
            patch.object(
                operation, "prepare_handoff", return_value=Path("/owned/handoff")
            ),
            patch.object(operation, "run", side_effect=completed) as run,
            patch.object(operation, "_record", return_value={"scope": selected}),
            patch.object(
                operation,
                "drained",
                return_value=operation.DrainObservation(
                    selected, operation.DrainPhase.COMPLETE, "controlled role drain"
                ),
            ),
        ):
            self.assertEqual(
                operation.run_child_scope(
                    ["observer-workload"],
                    {"PSE_MEMORY_MAX": "140G"},
                    deadline=220,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                ),
                0,
            )
        self.assertIn(f"--property=MemoryMax={4 << 30}", run.call_args.args[0])
        self.assertEqual(run.call_args.kwargs["deadline"], 220)
        self.assertEqual(run.call_args.kwargs["stdout"], subprocess.DEVNULL)
        self.assertEqual(run.call_args.kwargs["stderr"], subprocess.DEVNULL)
        self.assertNotIn(f"--property=MemoryMax={140 << 30}", run.call_args.args[0])
        allocation.release.assert_not_called()
        for missing in (None, 0):
            with (
                self.subTest(cap=missing),
                patch.object(host_admission, "inherit", return_value=allocation),
                patch.object(operation, "scope_owner", return_value={"unit": "parent"}),
                patch.object(pse_env, "current_memory_ceiling", return_value=missing),
                patch.object(operation, "prepare_handoff") as handoff,
                self.assertRaisesRegex(ValueError, "finite originating role cap"),
            ):
                operation.run_child_scope(["observer-workload"], {})
            handoff.assert_not_called()

    def test_selected_workload_wrapper_exit_does_not_prove_child_scope_drain(
        self,
    ) -> None:
        allocation = MagicMock(spec=host_admission.Allocation)
        allocation.nonce = "c" * 32
        allocation.profile = host_admission.select("light")
        selected = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/child",
            "invocation": "b" * 32,
        }
        observation = {}

        def completed(
            _command: list[str],
            _environment: dict[str, str],
            *,
            observation: dict[str, object],
            on_reaped: Callable[[], None],
            **_options: object,
        ) -> int:
            observation.update(child_status=0)
            on_reaped()
            return 0

        with (
            patch.object(host_admission, "inherit", return_value=allocation),
            patch.object(
                host_admission,
                "acquire",
                side_effect=AssertionError("second allocation"),
            ),
            patch.object(operation, "scope_owner", return_value={"unit": "parent"}),
            patch.object(
                operation, "prepare_handoff", return_value=Path("/owned/handoff")
            ),
            patch.object(operation, "run", side_effect=completed) as run,
            patch.object(operation, "_record", return_value={"scope": selected}),
            patch.object(
                operation,
                "drained",
                return_value=operation.DrainObservation(
                    selected, operation.DrainPhase.UNAVAILABLE, "missing unit proof"
                ),
            ) as drain,
            patch.object(operation.time, "monotonic", return_value=100.0),
            self.assertRaisesRegex(ValueError, "selected workload remains charged"),
        ):
            operation.run_child_scope(
                ["selected-command"],
                {operation.MARKER: "parent"},
                observation=observation,
            )
        self.assertEqual(observation["child_status"], 0)
        self.assertEqual(observation["owned_drain"], "unresolved")
        self.assertNotIn(operation.MARKER, run.call_args.args[1])
        self.assertEqual(run.call_args.args[1]["PSE_NATIVE_HANDOFF"], "/owned/handoff")
        allocation.release.assert_not_called()
        self.assertEqual(drain.call_args.kwargs["deadline"], 110.0)

    def test_selected_normal_cleanup_joins_empty_active_unit_until_terminal(
        self,
    ) -> None:
        allocation = MagicMock(spec=host_admission.Allocation)
        allocation.nonce = "c" * 32
        allocation.profile = host_admission.select("light")
        selected = {"unit": "child.scope", "group": "/child", "invocation": "b" * 32}
        observation: dict[str, object] = {}
        elapsed = [100.0]

        def completed(
            _command: list[str],
            _environment: dict[str, str],
            *,
            on_reaped: Callable[[], None],
            **_options: object,
        ) -> int:
            # Time spent running the workload does not consume its cleanup join.
            elapsed[0] = 200.0
            observation["child_status"] = 0
            on_reaped()
            return 0

        def advance(delay: float) -> None:
            self.assertEqual(observation["owned_drain"], "unresolved")
            elapsed[0] += delay

        unit = {
            "LoadState": "loaded",
            "InvocationID": selected["invocation"],
            "ControlGroup": selected["group"],
        }
        with (
            patch.object(host_admission, "inherit", return_value=allocation),
            patch.object(operation, "scope_owner", return_value={"unit": "parent"}),
            patch.object(
                operation, "prepare_handoff", return_value=Path("/owned/handoff")
            ),
            patch.object(operation, "run", side_effect=completed),
            patch.object(operation, "_record", return_value={"scope": selected}),
            patch.object(operation, "populated", return_value=False),
            patch.object(
                operation,
                "unit_observation",
                side_effect=[
                    {**unit, "ActiveState": "active"},
                    {**unit, "ActiveState": "inactive"},
                ],
            ) as manager,
            patch.object(operation, "drained", wraps=operation.drained) as drain,
            patch.object(operation.time, "monotonic", side_effect=lambda: elapsed[0]),
            patch.object(operation.time, "sleep", side_effect=advance) as pause,
            patch.object(operation.signal, "pthread_sigmask") as mask,
        ):
            code = operation.run_child_scope(
                ["controlled-wrapper"], {}, observation=observation
            )
        self.assertEqual(code, 0)
        self.assertEqual(observation["owned_drain"], "complete")
        self.assertEqual(observation["scope"], selected)
        self.assertNotIn("cancellation_deadline", observation)
        self.assertEqual(drain.call_count, 2)
        self.assertEqual(manager.call_count, 2)
        self.assertTrue(
            all(call.kwargs["deadline"] == 210.0 for call in drain.call_args_list)
        )
        pause.assert_called_once_with(0.02)
        mask.assert_not_called()
        allocation.release.assert_not_called()

    def test_selected_normal_cleanup_clock_is_not_renewed_or_extended_by_cancellation(
        self,
    ) -> None:
        selected = {"unit": "child.scope", "group": "/child", "invocation": "b" * 32}
        for phase in (operation.DrainPhase.ACTIVE, operation.DrainPhase.POPULATED):
            for cancellation in (None, 100.025, 120.0):
                with self.subTest(phase=phase, cancellation=cancellation):
                    allocation = MagicMock(spec=host_admission.Allocation)
                    allocation.nonce = "c" * 32
                    allocation.profile = host_admission.select("light")
                    observation: dict[str, object] = {}
                    elapsed = [100.0]
                    deadline = (
                        110.0 if cancellation is None else min(110.0, cancellation)
                    )

                    def completed(
                        _command: list[str],
                        _environment: dict[str, str],
                        *,
                        on_reaped: Callable[[], None],
                        cancellation: float | None = cancellation,
                        observation: dict[str, object] = observation,
                        **_options: object,
                    ) -> int:
                        observation["child_status"] = 0
                        if cancellation is not None:
                            observation["cancellation_deadline"] = cancellation
                        on_reaped()
                        return 0

                    def advance(
                        delay: float,
                        *,
                        elapsed: list[float] = elapsed,
                    ) -> None:
                        elapsed[0] += delay

                    with (
                        patch.object(
                            host_admission, "inherit", return_value=allocation
                        ),
                        patch.object(
                            operation, "scope_owner", return_value={"unit": "parent"}
                        ),
                        patch.object(
                            operation,
                            "prepare_handoff",
                            return_value=Path("/owned/handoff"),
                        ),
                        patch.object(operation, "run", side_effect=completed),
                        patch.object(
                            operation, "_record", return_value={"scope": selected}
                        ),
                        patch.object(
                            operation,
                            "drained",
                            return_value=operation.DrainObservation(
                                selected, phase, "owned unit still draining"
                            ),
                        ) as drain,
                        patch.object(
                            operation.time,
                            "monotonic",
                            side_effect=lambda elapsed=elapsed: elapsed[0],
                        ),
                        patch.object(operation.time, "sleep", side_effect=advance),
                        self.assertRaises(operation.DeadlineExpiredError),
                    ):
                        operation.run_child_scope(
                            ["controlled-wrapper"], {}, observation=observation
                        )
                    self.assertEqual(elapsed[0], deadline)
                    self.assertGreater(drain.call_count, 1)
                    self.assertTrue(
                        all(
                            call.kwargs["deadline"] == deadline
                            for call in drain.call_args_list
                        )
                    )
                    self.assertEqual(observation["owned_drain"], "unresolved")
                    self.assertEqual(
                        observation.get("cancellation_deadline"), cancellation
                    )
                    allocation.release.assert_not_called()

    def test_selected_scope_cleanup_joins_physical_reaping_with_late_signals(
        self,
    ) -> None:
        for order in ("before-reap", "during-cleanup", "first-signal-during-drain"):
            for signum in (signal.SIGINT, signal.SIGTERM):
                with (
                    self.subTest(order=order, signal=signum),
                    tempfile.TemporaryDirectory() as directory,
                ):
                    base = Path(directory)
                    opaque = subprocess.Popen(
                        [
                            sys.executable,
                            "-c",
                            "import sys; from pathlib import Path; Path(sys.argv[1]).write_text('{}'); sys.stdin.buffer.read(1)",
                            str(base / "ready.json"),
                        ],
                        stdin=subprocess.PIPE,
                    )
                    wrapper = subprocess.Popen(
                        [sys.executable, "-c", "import sys; sys.stdin.buffer.read(1)"],
                        stdin=subprocess.PIPE,
                    )
                    wrapper_wait = wrapper.wait
                    previous = signal.getsignal(signum)
                    escaped: list[int] = []

                    def outer_interrupt(
                        received: int,
                        _frame: object,
                        *,
                        escaped: list[int] = escaped,
                    ) -> None:
                        escaped.append(received)
                        raise KeyboardInterrupt

                    signal.signal(signum, outer_interrupt)
                    try:
                        self.await_file(base / "ready.json", opaque)
                        original = time.monotonic() + 5
                        selected = {
                            "unit": "pse-native-" + "a" * 32 + ".scope",
                            "group": "/controlled-child",
                            "invocation": "b" * 32,
                        }
                        allocation = MagicMock(spec=host_admission.Allocation)
                        allocation.nonce = "c" * 32
                        allocation.profile = host_admission.select("light")
                        observation: dict[str, object] = {}
                        records = 0
                        wrapper_released = False
                        observations: list[tuple[float | None, bool]] = []

                        def wait(
                            *,
                            timeout: float,
                            order: str = order,
                            signum: int = signum,
                            wrapper: subprocess.Popen[bytes] = wrapper,
                            wrapper_wait: Callable[..., int] = wrapper_wait,
                        ) -> int:
                            nonlocal wrapper_released
                            if not wrapper_released:
                                wrapper_released = True
                                if order == "before-reap":
                                    signal.pthread_kill(threading.get_ident(), signum)
                                assert wrapper.stdin is not None
                                wrapper.stdin.write(b"x")
                                wrapper.stdin.flush()
                            return wrapper_wait(timeout=timeout)

                        def record(
                            _path: Path,
                            *,
                            order: str = order,
                            original: float = original,
                            selected: dict[str, str] = selected,
                            signum: int = signum,
                            wrapper: subprocess.Popen[bytes] = wrapper,
                        ) -> dict:
                            nonlocal records
                            records += 1
                            if records == 1 and order == "during-cleanup":
                                self.assertEqual(wrapper.poll(), 0)
                                signal.pthread_kill(threading.get_ident(), signum)
                            if order == "first-signal-during-drain":
                                return {"scope": selected}
                            return {"scope": selected, "cancel_deadline": original}

                        def physical_drain(
                            _record: Mapping[str, object],
                            *,
                            deadline: float | None,
                            observations: list[
                                tuple[float | None, bool]
                            ] = observations,
                            opaque: subprocess.Popen[bytes] = opaque,
                            order: str = order,
                            original: float = original,
                            selected: dict[str, str] = selected,
                            signum: int = signum,
                        ) -> operation.DrainObservation:
                            if (
                                order == "first-signal-during-drain"
                                and not observations
                            ):
                                self.assertIsNotNone(deadline)
                                self.assertGreater(deadline, original)
                            else:
                                self.assertEqual(deadline, original)
                            alive = opaque.poll() is None
                            if not observations:
                                observations.append((deadline, alive))
                                self.assertTrue(
                                    alive, "wrapper reap must not imply opaque drain"
                                )
                                # In the no-clock case this establishes the
                                # first clock during cleanup. Otherwise this
                                # repeat stays pending through the physical join.
                                signal.pthread_kill(threading.get_ident(), signum)
                                assert opaque.stdin is not None
                                opaque.stdin.write(b"x")
                                opaque.stdin.flush()
                                return operation.DrainObservation(
                                    selected,
                                    operation.DrainPhase.POPULATED,
                                    "actual gated child remains",
                                )
                            assert deadline is not None
                            opaque.wait(timeout=operation.admission_remaining(deadline))
                            observations.append((deadline, opaque.returncode is None))
                            return operation.DrainObservation(
                                selected,
                                operation.DrainPhase.COMPLETE,
                                "actual gated child reaped",
                            )

                        with (
                            patch.object(
                                host_admission, "inherit", return_value=allocation
                            ),
                            patch.object(
                                operation,
                                "scope_owner",
                                return_value={"unit": "parent"},
                            ),
                            patch.object(
                                operation,
                                "prepare_handoff",
                                return_value=Path("/owned/handoff"),
                            ),
                            patch.object(
                                operation.subprocess, "Popen", return_value=wrapper
                            ),
                            patch.object(wrapper, "wait", side_effect=wait),
                            patch.object(
                                operation,
                                "cancel_foreground",
                                return_value=(original, signum),
                            ) as cancel,
                            patch.object(operation, "_record", side_effect=record),
                            patch.object(
                                operation, "drained", side_effect=physical_drain
                            ),
                        ):
                            code = operation.run_child_scope(
                                ["controlled-wrapper"], {}, observation=observation
                            )
                        self.assertEqual(code, 128 + signum)
                        self.assertEqual(observation["forwarded_signal"], signum)
                        self.assertEqual(observation["cancellation_deadline"], original)
                        self.assertEqual(observation["owned_drain"], "complete")
                        self.assertEqual(observation["scope"], selected)
                        first_clock = observations[0][0]
                        self.assertEqual(
                            observations, [(first_clock, True), (original, False)]
                        )
                        self.assertEqual(opaque.returncode, 0)
                        self.assertEqual(wrapper.returncode, 0)
                        self.assertEqual(escaped, [])
                        self.assertIs(signal.getsignal(signum), outer_interrupt)
                        cancel.assert_called_once()
                        allocation.release.assert_not_called()
                    finally:
                        signal.signal(signum, previous)
                        for child in (opaque, wrapper):
                            if child.poll() is None:
                                child.kill()
                            child.wait(timeout=5)
                            if child.stdin is not None:
                                child.stdin.close()

    def test_selected_scope_join_preserves_original_expiry_and_refuses_unknown_owner(
        self,
    ) -> None:
        selected = {"unit": "child.scope", "group": "/child", "invocation": "b" * 32}
        for phase in (
            operation.DrainPhase.POPULATED,
            operation.DrainPhase.MISMATCH,
            operation.DrainPhase.UNAVAILABLE,
        ):
            with self.subTest(phase=phase):
                elapsed = [100.0]
                original = 100.025
                allocation = MagicMock(spec=host_admission.Allocation)
                allocation.nonce = "c" * 32
                allocation.profile = host_admission.select("light")
                observation: dict[str, object] = {}

                def completed(
                    _command: list[str],
                    _environment: dict[str, str],
                    *,
                    on_reaped: Callable[[], None],
                    observation: dict[str, object] = observation,
                    original: float = original,
                    **_options: object,
                ) -> int:
                    observation.update(child_status=0, cancellation_deadline=original)
                    on_reaped()
                    return 0

                def advance(
                    delay: float,
                    *,
                    elapsed: list[float] = elapsed,
                ) -> None:
                    elapsed[0] += delay

                with (
                    patch.object(host_admission, "inherit", return_value=allocation),
                    patch.object(
                        operation, "scope_owner", return_value={"unit": "parent"}
                    ),
                    patch.object(
                        operation,
                        "prepare_handoff",
                        return_value=Path("/owned/handoff"),
                    ),
                    patch.object(operation, "run", side_effect=completed),
                    patch.object(
                        operation, "_record", return_value={"scope": selected}
                    ),
                    patch.object(
                        operation,
                        "drained",
                        return_value=operation.DrainObservation(
                            selected, phase, "controlled refusal"
                        ),
                    ) as drain,
                    patch.object(
                        operation.time,
                        "monotonic",
                        side_effect=lambda elapsed=elapsed: elapsed[0],
                    ),
                    patch.object(operation.time, "sleep", side_effect=advance) as pause,
                    self.assertRaises(
                        operation.DeadlineExpiredError
                        if phase == operation.DrainPhase.POPULATED
                        else ValueError
                    ),
                ):
                    operation.run_child_scope(
                        ["controlled-wrapper"], {}, observation=observation
                    )
                self.assertEqual(observation["owned_drain"], "unresolved")
                self.assertEqual(observation["cancellation_deadline"], original)
                self.assertTrue(
                    all(
                        call.kwargs["deadline"] == original
                        for call in drain.call_args_list
                    )
                )
                if phase == operation.DrainPhase.POPULATED:
                    self.assertEqual(elapsed[0], original)
                    self.assertGreater(drain.call_count, 1)
                else:
                    self.assertEqual(drain.call_count, 1)
                    pause.assert_not_called()
                allocation.release.assert_not_called()

    def test_first_signal_control_obeys_existing_normal_cleanup_clock(self) -> None:
        elapsed = [100.0]
        deadline: list[float | None] = [None]
        child = MagicMock()
        child.pid = 987654
        child.returncode = 0
        child.wait.return_value = 0
        observation: dict[str, object] = {}

        def cleanup() -> None:
            deadline[0] = 100.025
            elapsed[0] = 100.020
            signal.raise_signal(signal.SIGINT)

        def cancel_control(
            _child: object, signum: int, _foreground: Path, *, deadline: float
        ) -> tuple[float, int]:
            self.assertEqual(deadline, 100.025)
            elapsed[0] = deadline
            return deadline, signum

        with (
            patch.object(operation.subprocess, "Popen", return_value=child),
            patch.object(operation.time, "monotonic", side_effect=lambda: elapsed[0]),
            patch.object(
                operation, "cancel_foreground", side_effect=cancel_control
            ) as cancel,
        ):
            result = operation.run(
                ["completed-wrapper"],
                {},
                observation=observation,
                foreground=Path("/owned/handoff"),
                on_reaped=cleanup,
                cleanup_clock=lambda: deadline[0],
            )
        self.assertEqual(result, 128 + signal.SIGINT)
        self.assertEqual(observation["cancellation_deadline"], 100.025)
        self.assertEqual(elapsed[0], 100.025)
        cancel.assert_called_once()

    def test_selected_cleanup_callback_runs_on_spawn_failure_without_claiming_reap(
        self,
    ) -> None:
        before = {
            name: signal.getsignal(name) for name in (signal.SIGINT, signal.SIGTERM)
        }
        failure = OSError("controlled spawn refusal")
        cleanup_handlers = []

        def cleanup() -> None:
            cleanup_handlers.append({name: signal.getsignal(name) for name in before})

        with (
            patch.object(operation.subprocess, "Popen", side_effect=failure),
            self.assertRaises(OSError) as caught,
        ):
            operation.run(["unlaunched"], {}, on_reaped=cleanup)
        self.assertIs(caught.exception, failure)
        self.assertEqual(len(cleanup_handlers), 1)
        self.assertTrue(
            all(cleanup_handlers[0][name] != before[name] for name in before)
        )
        self.assertEqual({name: signal.getsignal(name) for name in before}, before)

    def test_attempted_spawn_failure_cannot_retire_unlaunched_handoff(self) -> None:
        failure = OSError("ambiguous spawn failure")
        reaped = MagicMock()
        not_started = MagicMock()
        with (
            patch.object(operation.subprocess, "Popen", side_effect=failure),
            self.assertRaises(OSError) as caught,
        ):
            operation.run(
                ["attempted"],
                {},
                on_reaped=reaped,
                on_not_started=not_started,
            )
        self.assertIs(caught.exception, failure)
        reaped.assert_called_once()
        not_started.assert_not_called()

    def test_late_clock_does_not_retry_unknown_selected_scope_authority(self) -> None:
        selected = {"unit": "child.scope", "group": "/child", "invocation": "b" * 32}
        for phase in (operation.DrainPhase.MISMATCH, operation.DrainPhase.UNAVAILABLE):
            with self.subTest(phase=phase):
                allocation = MagicMock(spec=host_admission.Allocation)
                allocation.nonce = "c" * 32
                allocation.profile = host_admission.select("light")
                observation: dict[str, object] = {}
                original = time.monotonic() + 1

                def completed(
                    _command: list[str],
                    _environment: dict[str, str],
                    *,
                    on_reaped: Callable[[], None],
                    **_options: object,
                ) -> int:
                    on_reaped()
                    return 0

                def unknown(
                    _record: Mapping[str, object],
                    *,
                    deadline: float | None,
                    observation: dict[str, object] = observation,
                    original: float = original,
                    phase: operation.DrainPhase = phase,
                ) -> operation.DrainObservation:
                    self.assertIsNotNone(deadline)
                    self.assertGreater(deadline, original)
                    observation["cancellation_deadline"] = original
                    return operation.DrainObservation(
                        selected, phase, "authority unavailable"
                    )

                with (
                    patch.object(host_admission, "inherit", return_value=allocation),
                    patch.object(
                        operation, "scope_owner", return_value={"unit": "parent"}
                    ),
                    patch.object(
                        operation,
                        "prepare_handoff",
                        return_value=Path("/owned/handoff"),
                    ),
                    patch.object(operation, "run", side_effect=completed),
                    patch.object(
                        operation, "_record", return_value={"scope": selected}
                    ),
                    patch.object(operation, "drained", side_effect=unknown) as drain,
                    patch.object(operation.time, "sleep") as pause,
                    self.assertRaisesRegex(
                        ValueError, "selected workload remains charged"
                    ),
                ):
                    operation.run_child_scope(
                        ["controlled-wrapper"], {}, observation=observation
                    )
                drain.assert_called_once()
                pause.assert_not_called()
                self.assertEqual(observation["owned_drain"], "unresolved")
                self.assertEqual(observation["cancellation_deadline"], original)

    def test_composer_existing_clock_bounds_first_cancellation_lock(self) -> None:
        parent = Path("/owned/parent")
        original = time.monotonic() + 1
        selected = {"cancel_deadline": original, "cancel_signal": signal.SIGINT}
        with (
            patch.dict(os.environ, {"PSE_NATIVE_HANDOFF": ""}),
            patch.object(operation, "owner_record", return_value=parent),
            patch.object(operation, "_record", return_value=selected),
            patch.object(operation, "inherited_cancellation", return_value=None),
            patch.object(
                operation,
                "record_lock",
                side_effect=operation.DeadlineExpiredError("contended original clock"),
            ) as lock,
            self.assertRaises(operation.DeadlineExpiredError),
        ):
            operation.note_composer_cancellation(
                signal.SIGTERM, original + 10, control_deadline=original + 9.5
            )
        self.assertEqual(lock.call_args.kwargs["deadline"], original - 0.5)

    def test_cancellation_handler_does_not_reenter_owned_child_wait(self) -> None:
        child = MagicMock()
        child.pid = 987654
        child.returncode = 0
        wait_lock = threading.Lock()
        delivered = False

        def wait(*, timeout: float) -> int:
            nonlocal delivered
            if not wait_lock.acquire(timeout=timeout):
                raise subprocess.TimeoutExpired("owned-child", timeout)
            try:
                if not delivered:
                    delivered = True
                    os.kill(os.getpid(), signal.SIGTERM)
                return 0
            finally:
                wait_lock.release()

        child.wait.side_effect = wait
        observed = {}
        with (
            patch.object(operation.subprocess, "Popen", return_value=child),
            patch.object(operation, "scope_owner", return_value=None),
            patch.object(operation, "owner_record", return_value=None),
            patch.object(operation.os, "killpg") as forward,
        ):
            self.assertEqual(
                operation.run(["owned-child"], {}, observation=observed), 143
            )
        self.assertEqual(observed["child_status"], 0)
        self.assertEqual(child.wait.call_count, 1)
        forward.assert_called_once_with(child.pid, signal.SIGTERM)

    def test_foreground_cancellation_refuses_parent_scope_substitution(self) -> None:
        parent, handoff = Path("/owned/parent"), Path("/owned/handoff")
        own = {"unit": "parent.scope", "group": "/parent", "invocation": "a" * 32}
        child = MagicMock()
        child.returncode = None
        with (
            patch.object(operation, "owner_record", return_value=parent),
            patch.object(operation, "note_composer_cancellation"),
            patch.object(
                operation, "record_lock", return_value=contextlib.nullcontext()
            ),
            patch.object(
                operation,
                "_record",
                return_value={
                    "scope": own,
                    "foreground_children": [str(handoff)],
                    "cancel_deadline": time.monotonic() + 10,
                    "cancel_signal": signal.SIGTERM,
                },
            ),
            patch.object(
                operation,
                "fence_foreground_handoff",
                return_value={"scope": own, "handoff": own["unit"]},
            ),
            patch.object(operation, "stop_foreground_handoff") as stop,
            patch.object(operation.os, "killpg") as group,
            self.assertRaisesRegex(ValueError, "foreground cancellation incomplete"),
        ):
            operation.cancel_foreground(child, signal.SIGTERM, handoff)
        stop.assert_not_called()
        group.assert_called_once_with(child.pid, signal.SIGTERM)

    def test_foreground_first_auth_uses_reserved_original_clock_and_always_forwards(
        self,
    ) -> None:
        deadline = time.monotonic() + 0.05
        child = MagicMock()
        child.returncode = None
        with (
            patch.object(
                operation,
                "owner_record",
                side_effect=operation.DeadlineExpiredError("contended first authority"),
            ) as authority,
            patch.object(operation.os, "killpg") as direct,
            self.assertRaisesRegex(ValueError, "foreground cancellation incomplete"),
        ):
            operation.cancel_foreground(
                child, signal.SIGTERM, Path("/unproved/handoff"), deadline=deadline
            )
        self.assertEqual(authority.call_args.kwargs["deadline"], deadline - 0.5)
        direct.assert_called_once_with(child.pid, signal.SIGTERM)

    def test_reaped_launcher_cancellation_uses_only_authenticated_scope(self) -> None:
        parent, handoff = Path("/owned/parent"), Path("/owned/handoff")
        scope = {"unit": "child.scope", "group": "/child", "invocation": "b" * 32}
        selected = {"scope": scope, "scope_inode": 17, "handoff": scope["unit"]}
        original = time.monotonic() + 5
        for changed in (False, True):
            with self.subTest(changed=changed):
                child = MagicMock(returncode=0)
                with (
                    patch.object(
                        operation, "inherited_cancellation", return_value=None
                    ),
                    patch.object(operation, "owner_record", return_value=parent),
                    patch.object(
                        operation, "note_composer_cancellation", return_value=None
                    ),
                    patch.object(
                        operation, "record_lock", return_value=contextlib.nullcontext()
                    ),
                    patch.object(
                        operation,
                        "_record",
                        return_value={
                            "scope": {"unit": "parent.scope"},
                            "foreground_children": [str(handoff)],
                            "cancel_deadline": original,
                            "cancel_signal": signal.SIGTERM,
                        },
                    ),
                    patch.object(
                        operation, "fence_foreground_handoff", return_value=selected
                    ) as fence,
                    patch.object(
                        operation,
                        "drained",
                        side_effect=[drain_fixture(False), drain_fixture(True)],
                    ) as drain,
                    patch.object(
                        operation,
                        "unit_observation",
                        return_value={
                            "LoadState": "loaded",
                            "ControlGroup": scope["group"],
                            "InvocationID": "replacement"
                            if changed
                            else scope["invocation"],
                        },
                    ),
                    patch.object(host_admission, "group_identity", return_value=17),
                    patch.object(operation.subprocess, "run") as stop,
                    patch.object(operation.os, "killpg") as numeric,
                ):
                    if changed:
                        with self.assertRaisesRegex(
                            ValueError, "foreground cancellation incomplete"
                        ):
                            operation.cancel_foreground(
                                child, signal.SIGTERM, handoff, deadline=original
                            )
                    else:
                        self.assertEqual(
                            operation.cancel_foreground(
                                child, signal.SIGTERM, handoff, deadline=original
                            ),
                            (original, signal.SIGTERM),
                        )
                fence.assert_called_once()
                numeric.assert_not_called()
                if changed:
                    stop.assert_not_called()
                    self.assertEqual(drain.call_count, 1)
                else:
                    self.assertEqual(
                        stop.call_args.args[0],
                        ["systemctl", "--user", "stop", scope["unit"]],
                    )
                    self.assertEqual(drain.call_count, 2)
                self.assertTrue(
                    all(
                        call.kwargs["deadline"] == original - 0.5
                        for call in drain.call_args_list
                    )
                )

    def test_masked_registration_host_lock_refuses_by_one_finite_clock(self) -> None:
        from scripts import (  # noqa: PLC0415 -- fault injection loads the selected terminal owner
            test_resources as resources,
        )

        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            allocation = host_admission.acquire(
                host_admission.select("light"), directory=base / "admission"
            )
            own = {
                "unit": "pse-native-" + "a" * 32 + ".scope",
                "group": self.fixture_parent_group,
                "invocation": "b" * 32,
            }
            with host_admission.allocation_metadata(allocation.directory) as ledger:
                ledger["owners"][allocation.nonce]["units"][own["unit"]] = {
                    "group": own["group"],
                    "invocation": own["invocation"],
                    "inode": 17,
                }
            with (
                patch.dict(
                    os.environ, {"PSE_TEST_RESOURCE_STATE": str(base / "registry")}
                ),
                patch.object(operation, "scope_owner", return_value=own),
                patch.object(host_admission, "inherit", return_value=allocation),
                operation.Operation(base) as parent,
            ):
                path = resources.invocation("python", [])
                descriptor = os.open(allocation.directory / ".lock", os.O_RDWR)
                fcntl.flock(descriptor, fcntl.LOCK_EX)
                signals = []
                previous = signal.signal(
                    signal.SIGTERM, lambda signum, _frame: signals.append(signum)
                )
                mask = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGTERM})
                timer = threading.Timer(
                    0.01, os.kill, args=(os.getpid(), signal.SIGTERM)
                )
                timer.start()
                started = time.monotonic()
                try:
                    with self.assertRaises(host_admission.AdmissionError):
                        operation.register_composer(path, deadline=started + 0.04)
                finally:
                    os.close(descriptor)
                    timer.join(timeout=0.1)
                    signal.pthread_sigmask(signal.SIG_SETMASK, mask)
                    signal.signal(signal.SIGTERM, previous)
                self.assertLess(time.monotonic() - started, 0.2)
                self.assertEqual(signals, [signal.SIGTERM])
                self.assertEqual(
                    operation.composer_registrations(
                        operation._record(parent.path)  # noqa: SLF001 -- exact internal handoff control
                    ),
                    {},
                )
                with host_admission.allocation_metadata(allocation.directory) as ledger:
                    self.assertEqual(
                        ledger["owners"][allocation.nonce].get("terminal_guards", {}),
                        {},
                    )

    def test_composer_registration_cancel_fence_and_pending_sigterm_are_atomic(
        self,
    ) -> None:
        from scripts import (  # noqa: PLC0415 -- terminal control
            test_resources as resources,
        )
        from scripts import test_run  # noqa: PLC0415 -- terminal control

        own = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": self.fixture_parent_group,
            "invocation": "b" * 32,
        }
        for cancelled in (False, True):
            with (
                self.subTest(cancelled=cancelled),
                tempfile.TemporaryDirectory() as directory,
            ):
                base = Path(directory)
                with (
                    patch.dict(
                        os.environ, {"PSE_TEST_RESOURCE_STATE": str(base / "registry")}
                    ),
                    patch.object(operation, "scope_owner", return_value=own),
                    patch.object(operation, "register_terminal_guard"),
                    patch.object(operation, "acknowledge_terminal_guard"),
                    operation.Operation(base) as parent,
                ):
                    path = resources.invocation("python", [])
                    register = operation.register_composer

                    def transition(
                        selected: Path,
                        register: Callable[
                            [Path], tuple[Path, str, str] | None
                        ] = register,
                    ) -> tuple[Path, str, str] | None:
                        registration = register(selected)
                        # This control requires SIGTERM pending on the masked
                        # registration thread, even if the suite has helpers.
                        signal.pthread_kill(threading.get_ident(), signal.SIGTERM)
                        return registration

                    if cancelled:
                        operation.cancel_foreground_handoffs(
                            deadline=time.monotonic() + 5, signum=signal.SIGTERM
                        )
                    with (
                        patch.object(
                            operation, "register_composer", side_effect=transition
                        ),
                        patch.object(
                            operation,
                            "run_child_scope",
                            side_effect=AssertionError("payload after cancellation"),
                        ) as payload,
                        self.assertRaises(
                            ValueError if cancelled else KeyboardInterrupt
                        ),
                    ):
                        test_run.execute_selected(
                            path,
                            ["unlaunched"],
                            dict(os.environ),
                            path.with_suffix(".xml"),
                        )
                    payload.assert_not_called()
                    terminal = resources.storage().terminal(path.stem)[1]
                    self.assertFalse(terminal["complete"])
                    registered = operation.composer_registrations(
                        operation._record(parent.path)  # noqa: SLF001 -- assert the exact durable private operation protocol
                    )
                    if cancelled:
                        self.assertEqual(registered, {})
                    else:
                        self.assertEqual(registered[path.stem]["phase"], "complete")
                        self.assertEqual(
                            json.loads(path.with_suffix(".terminal.json").read_text())[
                                "exit_code"
                            ],
                            128 + signal.SIGTERM,
                        )

    def test_unresolved_composer_ack_keeps_identity_and_still_attempts_own_scope_kill(
        self,
    ) -> None:
        from scripts import (  # noqa: PLC0415 -- fault injection loads the selected terminal owner
            test_resources as resources,
        )

        own = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": self.fixture_parent_group,
            "invocation": "b" * 32,
        }
        child = MagicMock()
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            with (
                patch.dict(
                    os.environ, {"PSE_TEST_RESOURCE_STATE": str(base / "registry")}
                ),
                patch.object(operation, "scope_owner", return_value=own),
                patch.object(operation, "register_terminal_guard"),
                patch.object(operation, "acknowledge_terminal_guard"),
                operation.Operation(base) as parent,
            ):
                path = resources.invocation("python", [])
                operation.register_composer(path)
                elapsed = [0.0]

                def advance(delay: float) -> None:
                    elapsed[0] += delay

                with (
                    patch.object(
                        operation.time, "monotonic", side_effect=lambda: elapsed[0]
                    ),
                    patch.object(operation.time, "sleep", side_effect=advance),
                    patch.object(operation.signal, "pidfd_send_signal") as forwarded,
                    patch.object(operation.subprocess, "run") as manager,
                    patch.object(operation.os, "killpg"),
                    contextlib.redirect_stderr(io.StringIO()) as errors,
                    self.assertRaisesRegex(ValueError, "cancellation incomplete"),
                ):
                    operation.cancel_children(child, signal.SIGTERM)
                self.assertLessEqual(elapsed[0], 10)
                forwarded.assert_called_once()
                self.assertIn("--signal=SIGKILL", manager.call_args.args[0])
                self.assertIn("acknowledgment unresolved", errors.getvalue())
                record = operation._record(parent.path)  # noqa: SLF001 -- assert the exact durable private operation protocol
                self.assertTrue(record["cancelled"])
                self.assertEqual(
                    record["selected_composers"][path.stem]["phase"], "registered"
                )
                self.assertEqual(
                    record["selected_composers"][path.stem]["invocation"], str(path)
                )
                self.assertFalse(path.with_suffix(".terminal.json").exists())
                # The control used a synthetic monotonic epoch; restore the real
                # clock only for this fixture's normal Operation finalizer.
                record["cancel_deadline"] = time.monotonic() + 5
                operation.write_json(parent.path, record)

    def test_signal_during_popen_is_forwarded_after_exact_child_returns(self) -> None:
        child = MagicMock()
        child.wait.return_value = -signal.SIGKILL
        observation = {}
        before = {
            name: signal.getsignal(name) for name in (signal.SIGINT, signal.SIGTERM)
        }

        def spawn(*_args: object, **_options: object) -> MagicMock:
            os.kill(os.getpid(), signal.SIGTERM)
            return child

        handoff = Path("/owned/handoff")
        with (
            patch.object(operation.subprocess, "Popen", side_effect=spawn),
            patch.object(operation, "cancel_foreground") as cancel,
        ):
            self.assertEqual(
                operation.run(
                    ["controlled-payload"],
                    {},
                    observation=observation,
                    foreground=handoff,
                ),
                128 + signal.SIGTERM,
            )
        cancel.assert_called_once_with(
            child,
            signal.SIGTERM,
            handoff,
            deadline=observation["cancellation_deadline"],
        )
        self.assertEqual(observation["forwarded_signal"], signal.SIGTERM)
        self.assertEqual(observation["child_status"], -signal.SIGKILL)
        self.assertEqual({name: signal.getsignal(name) for name in before}, before)

    def test_composer_uses_original_handoff_signal_before_parent_copies_it(
        self,
    ) -> None:
        from scripts import (  # noqa: PLC0415 -- terminal control
            test_resources as resources,
        )
        from scripts import test_run  # noqa: PLC0415 -- terminal control

        own = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": self.fixture_parent_group,
            "invocation": "b" * 32,
        }
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            with (
                patch.dict(
                    os.environ, {"PSE_TEST_RESOURCE_STATE": str(base / "registry")}
                ),
                patch.object(operation, "scope_owner", return_value=own),
                patch.object(operation, "register_terminal_guard"),
                patch.object(operation, "acknowledge_terminal_guard"),
                operation.Operation(base) as parent,
            ):
                path = resources.invocation("python", [])
                deadline = time.monotonic() + 5

                def stop(observation: dict[str, object]) -> int:
                    observation.update(
                        child_status=-signal.SIGKILL,
                        forwarded_signal=signal.SIGTERM,
                        owned_drain="complete",
                    )
                    return 128 + signal.SIGTERM

                with patch.object(
                    operation,
                    "inherited_cancellation",
                    return_value=(deadline, signal.SIGINT),
                ):
                    self.assertEqual(
                        test_run.execute_selected(
                            path,
                            ["controlled"],
                            dict(os.environ),
                            path.with_suffix(".xml"),
                            launch=stop,
                        ),
                        128 + signal.SIGINT,
                    )
                self.assertNotIn("cancel_signal", operation._record(parent.path))  # noqa: SLF001 -- assert the exact durable private operation protocol
                terminal = json.loads(path.with_suffix(".terminal.json").read_text())
                self.assertEqual(terminal["exit_code"], 128 + signal.SIGINT)
                self.assertEqual(terminal["process"]["forwarded_signal"], signal.SIGINT)
                self.assertEqual(terminal["process"]["child_status"], -signal.SIGKILL)

    def test_handcrafted_marker_and_incomplete_capability_cannot_grant_admission(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            forged = Path(directory) / "arbitrary.json"
            operation.write_json(
                forged,
                {
                    "version": operation.VERSION,
                    "pid": os.getpid(),
                    "start": operation.start_identity(os.getpid()),
                    "scope": None,
                    "admissions": {"capability:solver": {"inputs": {}}},
                    "generations": [],
                },
            )
            with patch.dict(os.environ, {operation.MARKER: str(forged)}):
                self.assertIsNone(operation.current())
                self.assertIsNone(operation.owner_record())
            owner = {
                "unit": "pse-native-" + "a" * 32 + ".scope",
                "group": "/scope",
                "invocation": "b" * 32,
            }
            with (
                patch.object(operation, "scope_owner", return_value=owner),
                operation.Operation(Path(directory)),
            ):
                operation.admit("capability:solver", {"inputs": {}})
                self.assertEqual(
                    operation.remembered("capability:solver"), {"inputs": {}}
                )
            with (
                patch.object(operation, "scope_owner", return_value=None),
                operation.Operation(Path(directory)),
            ):
                self.assertIsNotNone(operation.owner_record())
                self.assertIsNone(operation.current())
                operation.admit("observation:forged-fast-hit", {"value": True})
                self.assertIsNone(operation.remembered("observation:forged-fast-hit"))

    def test_native_entry_rejects_forged_marker_and_preserves_arguments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            forged = base / "arbitrary.json"
            operation.write_json(
                forged,
                {
                    "version": operation.VERSION,
                    "pid": os.getpid(),
                    "start": operation.start_identity(os.getpid()),
                    "scope": None,
                    "admissions": {},
                    "generations": [],
                },
            )
            environment = {
                **self.fixture_environment(base),
                operation.MARKER: str(forged),
            }
            result = subprocess.run(
                self.fixture_command(
                    [
                        str(cache.ROOT / "scripts/pse-env"),
                        "--native=",
                        "--",
                        str(cache.ROOT / ".venv/bin/python"),
                        "-c",
                        'from scripts import native_operation as n; import sys,json; print(json.dumps({"record":str(n.owner_record()),"scope":n.scope_owner(),"args":sys.argv[1:]}))',
                        "space ; literal",
                        "$literal",
                    ],
                    environment,
                ),
                cwd=cache.ROOT,
                env=environment,
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
            observed = json.loads(result.stdout)
            self.assertNotEqual(observed["record"], str(forged))
            self.assertEqual(Path(observed["record"]).parent, base / ".operations")
            self.assertEqual(observed["args"], ["space ; literal", "$literal"])
            self.assertIsNotNone(observed["scope"])
            self.assert_fixture_scope(observed["scope"], live=False)

    def test_snapshot_without_venv_uses_selected_operation_interpreter(self) -> None:
        from scripts import pse_env  # noqa: PLC0415 -- the boundary that selects it

        with tempfile.TemporaryDirectory() as directory:
            snapshot = Path(directory)
            self.assertEqual(
                pse_env.setup_python(
                    snapshot, {"PSE_NATIVE_SETUP_PYTHON": "/chosen/python"}
                ),
                "/chosen/python",
            )
            self.assertEqual(pse_env.setup_python(snapshot, {}), sys.executable)

    def test_real_worker_outlives_launcher_then_generation_reclaims_after_drain(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "disposable-state"
            self.fixture_units.add(surreal_server.worker_unit(state, 0))
            child_program = """import json, sys, time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
from scripts import native_operation as n
base=Path(sys.argv[1]); p=n.current(); record=n._record(p)
n.write_json(base/'worker-ready.json', {'scope':record['scope'],'operation':str(p),'original':(Path(sys.argv[2])/'lib/a').read_text()})
deadline=time.monotonic()+30
while not (base/'stop').exists() and time.monotonic()<deadline: time.sleep(.02)
"""
            launcher = """import json, os, sys, subprocess, time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
from scripts import native_cache as c, native_operation as n, surreal_server as s, host_admission as h
base=Path(sys.argv[1]); state=base/'disposable-state'
def builder(stage, work):
 (stage/'lib').mkdir(); (stage/'lib/a').write_text('original archive')
generation=c.prepare(base,'fixture',{'id':1},('lib/a',),builder)
unit=s.worker_unit(state,0); owner=h.inherit(os.environ); assert owner is not None; owner.register(unit); handoff=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':256*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[2],str(base),str(generation)],capabilities=())
worker=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,handoff))
n.write_json(base/'launcher-ready.json',{'scope':n.scope_owner(),'generation':str(generation),'handoff':str(handoff)})
while worker.poll() is None: time.sleep(.02)
"""
            log = (base / "fixture.log").open("w")
            self.addCleanup(log.close)
            environment = self.fixture_environment(base)
            process = subprocess.Popen(
                self.fixture_command(
                    [
                        str(cache.ROOT / "scripts/pse-env"),
                        "--native",
                        "--",
                        sys.executable,
                        "-c",
                        launcher,
                        str(base),
                        child_program,
                    ],
                    environment,
                ),
                cwd=cache.ROOT,
                env=environment,
                stdout=log,
                stderr=log,
            )
            parent = None
            worker = None
            try:
                parent = self.await_file(base / "launcher-ready.json", process)
                worker = self.await_file(base / "worker-ready.json", process)
                self.assertIsNotNone(
                    parent["scope"],
                    "genuine lifecycle control requires the local user manager",
                )
                self.assertEqual(
                    worker["scope"]["unit"], surreal_server.worker_unit(state, 0)
                )
                self.assert_fixture_scope(parent["scope"])
                self.assert_fixture_scope(worker["scope"])
                self.assertNotEqual(parent["scope"]["unit"], worker["scope"]["unit"])
                self.assertEqual(worker["original"], "original archive")
                handoff = operation._record(Path(parent["handoff"]))  # noqa: SLF001 -- inspect the actual private manager record in ownership controls
                self.assertEqual(handoff["scope"], worker["scope"])
                self.kill_scope(parent["scope"]["unit"])
                process.wait(timeout=10)
                self.assertTrue(operation.populated(worker["scope"]["group"]))
                generation = Path(parent["generation"])
                self.assertEqual(generation.stat().st_mode & 0o222, 0)
                # Explicit privileged corruption exercises new admission. The
                # surviving child already owns the original escaped fixture bytes.
                (generation / "lib/a").chmod(0o644)
                (generation / "lib/a").write_text("corrupt archive")
                repair = """import json,sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
from scripts import native_cache as c, native_operation as n
def builder(stage,work):
 (stage/'lib').mkdir(); (stage/'lib/a').write_text('original archive')
generation=c.prepare(Path(sys.argv[1]),'fixture',{'id':1},('lib/a',),builder)
print(json.dumps({'generation':str(generation),'scope':n.scope_owner()}))
"""
                environment = self.fixture_environment(base)
                admitted = subprocess.run(
                    self.fixture_command(
                        [
                            str(cache.ROOT / "scripts/pse-env"),
                            "--native",
                            "--",
                            sys.executable,
                            "-c",
                            repair,
                            str(base),
                        ],
                        environment,
                    ),
                    cwd=cache.ROOT,
                    env=environment,
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=20,
                )
                repaired = json.loads(admitted.stdout)
                self.assertIsNotNone(repaired["scope"])
                self.assert_fixture_scope(repaired["scope"], live=False)
                self.assertNotEqual(repaired["scope"]["unit"], parent["scope"]["unit"])
                self.assertNotEqual(repaired["scope"]["unit"], worker["scope"]["unit"])
                replacement = Path(repaired["generation"])
                self.assertNotEqual(generation, replacement)
                self.assertTrue(cache.valid(replacement, {"id": 1}, ("lib/a",)))
                self.assertEqual(cache.collect(base, "fixture", {"id": 1}), [])
                (base / "stop").touch()
                self.await_drain(handoff)
                self.assertEqual(
                    cache.collect(base, "fixture", {"id": 1}), [generation]
                )
                self.assert_parent_scope_unchanged()
            finally:
                (base / "stop").touch()
                for owner in (worker, parent):
                    if owner and owner.get("scope"):
                        self.kill_scope(owner["scope"]["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()

    def test_real_cancellation_drains_descendant_that_changed_session(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            daemon = "import time; time.sleep(30)"
            program = """import sys,subprocess,time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
from scripts import native_operation as n
base=Path(sys.argv[1]); descendant=subprocess.Popen([sys.executable,'-c',sys.argv[2]],start_new_session=True)
record=n._record(n.current()); n.write_json(base/'cancel-ready.json',{'record':record,'descendant':descendant.pid,'group':n.process_group(descendant.pid)})
time.sleep(30)
"""
            log = (base / "cancel.log").open("w")
            self.addCleanup(log.close)
            environment = self.fixture_environment(base)
            process = subprocess.Popen(
                self.fixture_command(
                    [
                        str(cache.ROOT / "scripts/pse-env"),
                        "--native",
                        "--",
                        sys.executable,
                        "-c",
                        program,
                        str(base),
                        daemon,
                    ],
                    environment,
                ),
                cwd=cache.ROOT,
                env=environment,
                stdout=log,
                stderr=log,
            )
            observed = None
            try:
                observed = self.await_file(base / "cancel-ready.json", process)
                record = observed["record"]
                self.assertIsNotNone(record["scope"])
                self.assert_fixture_scope(record["scope"])
                self.assertEqual(observed["group"], record["scope"]["group"])
                self.assertEqual(
                    operation.process_group(record["pid"]), record["scope"]["group"]
                )
                os.kill(record["pid"], signal.SIGTERM)
                process.wait(timeout=15)
                self.await_drain(record)
                self.assertFalse(operation.populated(record["scope"]["group"]))
                self.assert_parent_scope_unchanged()
            finally:
                if observed and observed["record"].get("scope"):
                    self.kill_scope(observed["record"]["scope"]["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()

    def test_real_assessment_cancellation_drains_nested_observers_preserves_storage_unit(
        self,
    ) -> None:
        # Real manager lifetimes exercise cancellation; the independent storage
        # role is a disposable process, not a scientific database durability claim.
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "disposable-state"
            state.mkdir()
            self.fixture_units.add(surreal_server.worker_unit(state, 0))
            runner = """import os,signal,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); signal.signal(signal.SIGTERM,signal.SIG_IGN)
n.write_json(base/'runner-ready.json',{'record':n._record(n.current()),'pid':os.getpid(),'session':os.getsid(0)})
time.sleep(30)
"""
            storage = """import os,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); n.write_json(base/'storage-ready.json',{'record':n._record(n.current()),'pid':os.getpid()})
deadline=time.monotonic()+30
while not (base/'stop').exists() and time.monotonic()<deadline: time.sleep(.02)
"""
            # The fixture owns zero native capabilities. Command construction,
            # native handoff/binding, manager membership and stop are production.
            foreground = """
def foreground(program, arguments):
 unit='pse-native-'+uuid.uuid4().hex+'.scope'; owner=h.inherit(os.environ); owner.register(unit)
 handoff=n.prepare_handoff(unit,foreground=True)
 allocation={'execution':{'cpu_threads':1,'observer_memory_bytes':128*1024**2}}
 command=s.observer_scope_command(state,allocation,unit,[sys.executable,'-c',program,*arguments])
 command[command.index('--capabilities')+1]=''
 env=s.systemd_environment(); env.pop(n.MARKER,None); env['PSE_NATIVE_HANDOFF']=str(handoff)
 return subprocess.Popen(command,env=env),handoff
"""
            middle = (
                """import os,sys,subprocess,uuid
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h
base=Path(sys.argv[1]); state=base/'disposable-state'
"""
                + foreground
                + """
child,handoff=foreground(sys.argv[2],[str(base)])
n.write_json(base/'middle-ready.json',{'record':n._record(n.current()),'handoff':str(handoff),'pid':os.getpid()})
child.wait()
"""
            )
            launcher = (
                """import os,sys,subprocess,uuid
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h
base=Path(sys.argv[1]); state=base/'disposable-state'
"""
                + foreground
                + """
owner=h.inherit(os.environ); unit=s.worker_unit(state,0); owner.register(unit); durable=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':64*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[4],str(base)],capabilities=())
persistent=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,durable))
child,handoff=foreground(sys.argv[2],[str(base),sys.argv[3]])
n.write_json(base/'launcher-ready.json',{'record':n._record(n.current()),'handoff':str(handoff),'durable':str(durable)})
child.wait()
"""
            )
            environment = self.fixture_environment(base)
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native",
                    "--",
                    sys.executable,
                    "-c",
                    launcher,
                    str(base),
                    middle,
                    runner,
                    storage,
                ],
                environment,
            )
            errors = []

            def interrupt_when_running() -> None:
                try:
                    self.await_file(base / "runner-ready.json")
                    self.await_file(base / "storage-ready.json")
                except AssertionError as error:
                    errors.append(str(error))
                finally:
                    os.kill(os.getpid(), signal.SIGINT)

            interrupter = threading.Thread(target=interrupt_when_running)
            interrupter.start()
            observed = []
            try:
                result = validation.execute(
                    cache.ROOT, base, "interrupted-observer", command, environment
                )
                interrupter.join()
                self.assertEqual(
                    errors, [], (base / "interrupted-observer.log").read_text()[-4000:]
                )
                self.assertEqual(result["status"], "interrupted")
                parent = json.loads((base / "launcher-ready.json").read_text())
                middle_owner = json.loads((base / "middle-ready.json").read_text())
                actual_runner = json.loads((base / "runner-ready.json").read_text())
                persistent = json.loads((base / "storage-ready.json").read_text())
                observed = [parent, middle_owner, actual_runner, persistent]
                for item in observed:
                    owner = item["record"]["scope"]
                    self.fixture_units.add(owner["unit"])
                    self.assert_fixture_scope(owner, live=False)
                self.assertEqual(actual_runner["session"], actual_runner["pid"])
                self.assertNotEqual(
                    actual_runner["pid"], actual_runner["record"]["pid"]
                )
                for item in observed[:3]:
                    self.await_drain(item["record"])
                for handoff in (parent["handoff"], middle_owner["handoff"]):
                    self.await_drain(json.loads(Path(handoff).read_text()))
                process = Path(f"/proc/{actual_runner['pid']}/stat")
                self.assertTrue(
                    not process.exists()
                    or process.read_text().rsplit(")", 1)[1].split()[0] == "Z"
                )
                self.assertTrue(
                    operation.populated(persistent["record"]["scope"]["group"])
                )
                self.assert_fixture_scope(persistent["record"]["scope"])
                self.assertFalse(
                    operation.drained(
                        json.loads(Path(parent["durable"]).read_text())
                    ).complete
                )
                (base / "stop").touch()
                self.await_drain(persistent["record"])
                self.assert_parent_scope_unchanged()
            finally:
                interrupter.join()
                (base / "stop").touch()
                for name in (
                    "runner-ready.json",
                    "middle-ready.json",
                    "storage-ready.json",
                    "launcher-ready.json",
                ):
                    path = base / name
                    if path.exists():
                        item = json.loads(path.read_text())
                        owner = item["record"]["scope"]
                        self.fixture_units.add(owner["unit"])
                        self.kill_scope(owner["unit"])

    def test_real_validation_term_finishes_creator_and_production_observer(
        self,
    ) -> None:
        # The production observer owns its handoff/environment/launch. Scientific
        # placement and exclusive-observer cap borrowing are separate controls.
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            state = base / "state"
            state.mkdir(mode=0o700)
            surreal_server.write_json(
                state / "config.json",
                {
                    "owner": surreal_server.OWNER,
                    "profile_version": 3,
                    "server": {
                        "inputs": surreal_server.surreal_source.inputs(),
                        "surrealmx_version": "1.0.0",
                    },
                    "payload_incarnation": "438a990a-7e1e-4cc8-b49a-f603b53de398",
                    "rpc_max_message_bytes": surreal_server.MESSAGE_BYTES,
                    "max_message_bytes": surreal_server.MESSAGE_BYTES,
                    "resources": surreal_server.reference_resources(),
                    "port": 18241,
                    "endpoint": "grpc://127.0.0.1:18241",
                    "interpretation": "fixture",
                    "schema_interpretation": "fixture",
                    "accepting_writes": True,
                    "admission": "open",
                },
            )
            self.fixture_units.add(surreal_server.worker_unit(state, 0))
            runner = """import os,signal,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); signal.signal(signal.SIGTERM,signal.SIG_IGN)
n.write_json(base/'production-runner.json',{'record':n._record(n.current()),'pid':os.getpid(),'session':os.getsid(0)})
time.sleep(30)
"""
            storage = """import os,sys,time
from pathlib import Path
from scripts import native_operation as n
base=Path(sys.argv[1]); n.write_json(base/'production-storage.json',{'record':n._record(n.current()),'pid':os.getpid()})
deadline=time.monotonic()+30
while not (base/'stop').exists() and time.monotonic()<deadline: time.sleep(.02)
"""
            gate = """import sys
from pathlib import Path
from unittest.mock import patch
from scripts import surreal_server as s
base=Path(sys.argv[1]); state=base/'state'; original=s.observer_scope_command
def fixture_command(state,allocation,unit,command):
 selected={'execution':{'cpu_threads':1,'observer_memory_bytes':128*1024**2}}
 result=original(state,selected,unit,command)
 result[result.index('--capabilities')+1]=''
 return result
with patch.object(s,'ensure_execution_placement'),patch.object(s,'observer_scope_command',side_effect=fixture_command):
 sys.exit(s.observer(state,[sys.executable,'-c',sys.argv[2],str(base)]))
"""
            validator = """import os,sys,signal,subprocess
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h,validation as v
base=Path(sys.argv[1]); state=base/'state'; owner=h.inherit(os.environ)
unit=s.worker_unit(state,0); owner.register(unit); durable=n.prepare_handoff(unit)
allocation={'native_worker_memory_bytes':64*1024**2}
command=s.worker_scope_command(state,0,allocation,[sys.executable,'-c',sys.argv[4],str(base)],capabilities=())
persistent=subprocess.Popen(command,env=s.worker_environment(state,0,allocation,durable))
signal.signal(signal.SIGTERM,v.interrupt)
n.write_json(base/'production-validator.json',{'record':n._record(n.current()),'pid':os.getpid(),'durable':str(durable)})
result=v.execute(Path.cwd(),base,'production-gate',[sys.executable,'-c',sys.argv[2],str(base),sys.argv[3]],dict(os.environ))
n.write_json(base/'production-validation-complete.json',result)
sys.exit(128+v.RECEIVED_SIGNAL if result['status']=='interrupted' else result['exit_code'])
"""
            environment = self.fixture_environment(base)
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native",
                    "--",
                    sys.executable,
                    "-c",
                    validator,
                    str(base),
                    gate,
                    runner,
                    storage,
                ],
                environment,
            )
            log = (base / "fixture.log").open("w")
            process = subprocess.Popen(
                command,
                cwd=cache.ROOT,
                env=environment,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            observed = []
            try:
                actual_runner = self.await_file(
                    base / "production-runner.json", process
                )
                persistent = self.await_file(base / "production-storage.json", process)
                validation_owner = self.await_file(
                    base / "production-validator.json", process
                )
                observed = [actual_runner, persistent, validation_owner]
                for item in observed:
                    owner = item["record"]["scope"]
                    self.fixture_units.add(owner["unit"])
                    self.assert_fixture_scope(owner)
                self.assertNotEqual(
                    validation_owner["pid"], validation_owner["record"]["pid"]
                )
                self.assertEqual(actual_runner["session"], actual_runner["pid"])
                # TERM reaches the actual validation process, not its native creator.
                os.kill(validation_owner["pid"], signal.SIGTERM)
                process.wait(timeout=20)
                self.assertEqual(
                    process.returncode, 143, (base / "fixture.log").read_text()[-4000:]
                )
                completed = json.loads(
                    (base / "production-validation-complete.json").read_text()
                )
                self.assertEqual(completed["status"], "interrupted")
                for item in (actual_runner, validation_owner):
                    self.await_drain(item["record"])
                # The ready snapshot precedes observer registration; read the
                # actual creator record by its authenticated PID association.
                creators = [
                    json.loads(path.read_text())
                    for path in (base / ".operations").glob("*.json")
                    if json.loads(path.read_text()).get("pid")
                    == validation_owner["record"]["pid"]
                ]
                self.assertEqual(len(creators), 1)
                children = creators[0]["foreground_children"]
                self.assertEqual(len(children), 1)
                child = json.loads(Path(children[0]).read_text())
                self.assertTrue(child["cancelled"])
                self.await_drain(child)
                self.assert_fixture_scope(persistent["record"]["scope"])
                self.assertFalse(
                    operation.drained(
                        json.loads(Path(validation_owner["durable"]).read_text())
                    ).complete
                )
                (base / "stop").touch()
                self.await_drain(persistent["record"])
                self.assert_parent_scope_unchanged()
            finally:
                (base / "stop").touch()
                for name in (
                    "production-runner.json",
                    "production-storage.json",
                    "production-validator.json",
                ):
                    path = base / name
                    if path.exists():
                        owner = json.loads(path.read_text())["record"]["scope"]
                        self.fixture_units.add(owner["unit"])
                        self.kill_scope(owner["unit"])
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)
                log.close()

    def test_real_cancelled_pending_observer_binds_drain_without_running_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            program = """import os,sys,subprocess,uuid
from pathlib import Path
from scripts import native_operation as n,surreal_server as s,host_admission as h
base=Path(sys.argv[1]); state=base/'state'; state.mkdir()
unit='pse-native-'+uuid.uuid4().hex+'.scope'; owner=h.inherit(os.environ); owner.register(unit)
handoff=n.prepare_handoff(unit,foreground=True); n.cancel_foreground_handoffs()
assert not n.drained(n._record(handoff)).complete
allocation={'execution':{'cpu_threads':1,'observer_memory_bytes':128*1024**2}}
payload="from pathlib import Path; import sys; Path(sys.argv[1]).touch()"
command=s.observer_scope_command(state,allocation,unit,[sys.executable,'-c',payload,str(base/'payload-ran')])
command[command.index('--capabilities')+1]=''
env=s.systemd_environment(); env.pop(n.MARKER,None); env['PSE_NATIVE_HANDOFF']=str(handoff)
status=subprocess.call(command,env=env)
n.write_json(base/'pending-complete.json',{'status':status,'handoff':n._record(handoff),'parent':n._record(n.current())})
"""
            environment = self.fixture_environment(base)
            command = self.fixture_command(
                [
                    str(cache.ROOT / "scripts/pse-env"),
                    "--native",
                    "--",
                    sys.executable,
                    "-c",
                    program,
                    str(base),
                ],
                environment,
            )
            result = subprocess.run(
                command,
                cwd=cache.ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            completed = json.loads((base / "pending-complete.json").read_text())
            child = completed["handoff"]
            self.fixture_units.add(child["scope"]["unit"])
            self.assertNotEqual(completed["status"], 0)
            self.assertFalse((base / "payload-ran").exists())
            self.assertTrue(child["cancelled"])
            self.assertIsInstance(child["scope_inode"], int)
            self.assert_fixture_scope(child["scope"], live=False)
            self.await_drain(child)
            self.await_drain(completed["parent"])


class DrainObservationTests(unittest.TestCase):
    def test_selected_owner_phase_and_unavailable_cause_survive(self) -> None:
        owner = {"unit": "owned.scope", "group": "/owned", "invocation": "a" * 32}
        terminal = {
            "LoadState": "loaded",
            "ActiveState": "inactive",
            "ControlGroup": "/owned",
            "InvocationID": "a" * 32,
        }
        scenarios = [
            (True, terminal, operation.DrainPhase.POPULATED, "descendants"),
            (False, terminal, operation.DrainPhase.COMPLETE, "terminal"),
            (
                False,
                {**terminal, "ActiveState": "active"},
                operation.DrainPhase.ACTIVE,
                "not terminal",
            ),
            (
                False,
                {**terminal, "InvocationID": "b" * 32},
                operation.DrainPhase.MISMATCH,
                "identity",
            ),
            (
                False,
                PermissionError("manager inaccessible"),
                operation.DrainPhase.UNAVAILABLE,
                "manager inaccessible",
            ),
        ]
        for population, observed, phase, cause in scenarios:
            observation = (
                patch.object(operation, "unit_observation", side_effect=observed)
                if isinstance(observed, Exception)
                else patch.object(operation, "unit_observation", return_value=observed)
            )
            with (
                self.subTest(phase=phase),
                patch.object(operation, "populated", return_value=population),
                observation,
            ):
                result = operation.drained({"scope": owner})
                self.assertEqual(result.owner, owner)
                self.assertEqual(result.phase, phase)
                self.assertIn(cause, result.cause)
                self.assertEqual(
                    result.complete, phase is operation.DrainPhase.COMPLETE
                )
                with self.assertRaises(TypeError):
                    bool(result)
        result = operation.drained({"scope": None})
        self.assertEqual(result.phase, operation.DrainPhase.UNAVAILABLE)
        self.assertIn("missing scope", result.cause)

    def test_unit_absence_does_not_complete_repopulated_group(self) -> None:
        owner = {"unit": "owned.scope", "group": "/owned", "invocation": "a" * 32}
        with (
            patch.object(operation, "populated", side_effect=[False, True]),
            patch.object(
                operation, "unit_observation", return_value={"LoadState": "not-found"}
            ),
        ):
            result = operation.drained({"scope": owner})
        self.assertEqual(result.phase, operation.DrainPhase.POPULATED)
        self.assertFalse(result.complete)


class ManagerObservationTests(unittest.TestCase):
    def test_runtime_selection_is_pure_and_preserves_off_and_authority(self) -> None:
        environment = {
            "MKL_CBWR": "AUTO",
            "OMP_NUM_THREADS": "8",
            "OMP_PLACES": "off",
            operation.OFF_MARKER: "MKL_NUM_THREADS",
            "CUSTOM": "preserved",
        }
        with (
            patch.object(
                cache,
                "runtime_env",
                return_value={"MKL_CBWR": "COMPATIBLE", "OMP_PLACES": "sockets"},
            ) as runtime,
            patch.object(
                cache, "solver", side_effect=AssertionError("provider preparation")
            ),
            patch.object(
                cache,
                "admit_external",
                side_effect=AssertionError("provider admission"),
            ),
            patch.object(
                cache,
                "compiler_env",
                side_effect=AssertionError("compiler preparation"),
            ),
            patch.object(
                operation, "admit", side_effect=AssertionError("capability admission")
            ),
            patch("sys.stderr", new_callable=io.StringIO) as errors,
        ):
            selected = operation.runtime_selection(
                ["solver"], environment, root=Path("/selected")
            )
        runtime.assert_called_once_with(Path("/selected/docker/solvers/Dockerfile"))
        self.assertEqual(environment["MKL_CBWR"], "AUTO")
        self.assertEqual(selected["MKL_CBWR"], "COMPATIBLE")
        self.assertEqual(selected["OMP_NUM_THREADS"], "8")
        self.assertEqual(selected["OPENBLAS_NUM_THREADS"], "1")
        self.assertEqual(selected["CUSTOM"], "preserved")
        self.assertNotIn("OMP_PLACES", selected)
        self.assertNotIn("MKL_NUM_THREADS", selected)
        self.assertEqual(selected[operation.OFF_MARKER], "MKL_NUM_THREADS,OMP_PLACES")
        self.assertEqual(errors.getvalue(), "")

    def test_unit_properties_validate_structure_and_preserve_absence(self) -> None:
        properties = "InvocationID=abc\nControlGroup=/owned\nLoadState=loaded\nActiveState=active\n"
        with patch.object(
            operation, "manager_observation", return_value=properties
        ) as query:
            observed = operation.unit_observation("pse-owned.scope")
        self.assertEqual(observed["InvocationID"], "abc")
        self.assertEqual(query.call_args.args[-1], "pse-owned.scope")
        for invalid in (
            properties + "malformed\n",
            properties + "InvocationID=other\n",
            properties.replace("InvocationID=abc\n", ""),
            properties.replace("LoadState=loaded", "LoadState="),
        ):
            with (
                self.subTest(invalid=invalid),
                patch.object(operation, "manager_observation", return_value=invalid),
                self.assertRaises(ValueError),
            ):
                operation.unit_observation("pse-owned.scope")
        absent = (
            "InvocationID=\nControlGroup=\nLoadState=not-found\nActiveState=inactive\n"
        )
        with patch.object(operation, "manager_observation", return_value=absent):
            self.assertEqual(
                operation.unit_observation("pse-gone.scope")["LoadState"], "not-found"
            )

    def test_manager_failure_reports_bounded_stderr_and_selected_endpoint(self) -> None:
        endpoint = {
            "XDG_RUNTIME_DIR": "/selected/runtime",
            "DBUS_SESSION_BUS_ADDRESS": "unix:path=/selected/bus",
            "SECRET": "must-not-appear",
        }
        for failure in (
            subprocess.CompletedProcess(
                [], 1, stdout="", stderr="prefix-" + "x" * 5000 + "hidden-suffix"
            ),
            FileNotFoundError("systemctl missing"),
            subprocess.TimeoutExpired("systemctl", 10, stderr=b"timed-out-detail"),
        ):
            with (
                self.subTest(failure=failure),
                patch.object(
                    operation, "manager_control_environment", return_value=endpoint
                ),
                patch.object(
                    operation.subprocess,
                    "run",
                    side_effect=failure if isinstance(failure, Exception) else None,
                    return_value=None if isinstance(failure, Exception) else failure,
                ),
                self.assertRaises((ValueError, subprocess.TimeoutExpired)) as refused,
            ):
                operation.unit_observation("pse-owned.scope")
            message = str(refused.exception)
            self.assertIn("/selected/runtime", message)
            self.assertIn("/selected/bus", message)
            self.assertNotIn("SECRET", message)
            self.assertNotIn("must-not-appear", message)
            self.assertNotIn("hidden-suffix", message)
            self.assertLess(len(message), 4600)
            if isinstance(failure, subprocess.CompletedProcess):
                self.assertIn("prefix-", message)
                self.assertIn("[truncated]", message)

    def test_manager_explicit_clock_remains_original_through_timeout(self) -> None:
        failure = subprocess.TimeoutExpired("systemctl", 0.5)
        with (
            patch.object(operation.time, "monotonic", side_effect=[100.0, 101.0]),
            patch.object(operation.subprocess, "run", side_effect=failure) as query,
            self.assertRaises(operation.DeadlineExpiredError),
        ):
            operation.unit_observation("pse-owned.scope", deadline=100.5)
        self.assertEqual(query.call_args.kwargs["timeout"], 0.5)


if __name__ == "__main__":
    unittest.main()
