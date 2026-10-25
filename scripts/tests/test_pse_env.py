# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The checkout environment boundary: precedence, secrets, placement and status."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import io
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import MagicMock, patch

from scripts import pse_env, surreal_server, test_resources


def identity(_root: Path, env: dict[str, str]) -> dict[str, str]:
    return dict(env)


if TYPE_CHECKING:
    from collections.abc import Callable, Generator


class PseEnvTests(unittest.TestCase):
    def test_description_never_evaluates_local_code_or_acquires_authority(self) -> None:
        touched = self.root / "local-effect"
        (self.root / ".envrc.local").write_text(
            f"touch '{touched}'\nexport PYO3_PYTHON=local\n"
        )
        with (
            patch.object(pse_env, "local_keys", side_effect=AssertionError("shell")),
            patch.object(
                pse_env.host,
                "allocation_metadata",
                side_effect=AssertionError("ledger"),
            ),
            patch.object(
                pse_env.operation, "environment", side_effect=AssertionError("prepare")
            ),
            patch.object(
                pse_env, "manager_available", side_effect=AssertionError("manager")
            ),
            patch.object(
                pse_env.subprocess, "run", side_effect=AssertionError("child")
            ),
        ):
            value = pse_env.operation_description(
                self.root, {"PATH": "/usr/bin", "RUSTC_WRAPPER": ""}, ["solver"]
            )
        self.assertFalse(touched.exists())
        self.assertEqual(value["local_configuration"], "unresolved: not executed")
        self.assertEqual(value["compiler"]["RUSTC_WRAPPER"]["value"], "")
        self.assertFalse(value["compiler"]["RUSTC_WRAPPER"]["local_unresolved"])
        self.assertTrue(value["compiler"]["PYO3_PYTHON"]["local_unresolved"])
        self.assertEqual(value["placement"]["admission"], "not observed or acquired")

    def test_description_supplied_local_and_caller_match_execution_selection(
        self,
    ) -> None:
        caller = {
            "PATH": "/usr/bin",
            "PYO3_PYTHON": "/caller/python",
            "RUSTC_WRAPPER": "",
            "IPOPT_DIR": "/chosen/provider",
        }
        local = {
            "PYO3_PYTHON": "/local/python",
            "LIBCLANG_PATH": "/chosen/llvm/lib",
            "SECRET": "never disclose",
        }
        env = pse_env.compose(self.root, caller, local)
        value = pse_env.operation_description(
            self.root,
            caller,
            ["solver"],
            local=local,
            command=["cargo", "build"],
            resource_class="compile",
        )
        self.assertEqual(value["compiler"]["PYO3_PYTHON"]["value"], env["PYO3_PYTHON"])
        self.assertEqual(value["compiler"]["PYO3_PYTHON"]["source"], "caller")
        self.assertEqual(value["compiler"]["LIBCLANG_PATH"]["source"], "local")
        self.assertEqual(
            value["providers"]["solver"]["inputs"]["IPOPT_DIR"]["value"],
            "/chosen/provider",
        )
        self.assertEqual(value["placement"]["resource_class"], "compile")
        self.assertNotIn("never disclose", json.dumps(value))

    def test_description_json_cli_keeps_command_intent_without_effects(self) -> None:
        output = io.StringIO()
        with (
            patch.object(pse_env, "ROOT", self.root),
            patch.dict(os.environ, {"PATH": "/usr/bin"}, clear=True),
            patch.object(pse_env, "local_keys", side_effect=AssertionError("shell")),
            contextlib.redirect_stdout(output),
        ):
            self.assertEqual(
                pse_env.main(
                    [
                        "--explain",
                        "--json",
                        "--resource-class",
                        "compile",
                        "--",
                        "maturin",
                        "develop",
                        "--profile",
                        "producer",
                        "--token",
                        "private-value",
                    ]
                ),
                0,
            )
        value = json.loads(output.getvalue())
        self.assertEqual(value["version"], 1)
        self.assertEqual(value["command"], ["maturin", "develop"])
        self.assertEqual(value["build_context"]["profile"]["values"], ["producer"])
        self.assertNotIn("private-value", output.getvalue())
        self.assertEqual(value["placement"]["resource_class"], "compile")

    def test_scoped_exits_ignore_unrelated_corrupt_resource_metadata(self) -> None:
        registry = self.root / "unrelated-registry"
        registry.mkdir(mode=0o700)
        metadata = registry / "allocations.json"
        metadata.write_text("corrupt unrelated metadata")
        original = metadata.read_bytes()
        with (
            patch.object(test_resources, "registry", return_value=registry),
            self.assertRaises(ValueError),
        ):
            test_resources.reclaim_reports()
        for native in [False, True]:
            for status in [0, 7, -signal.SIGTERM]:
                owner = MagicMock(spec=pse_env.host.Allocation)
                owner.release.return_value = True
                with (
                    self.subTest(native=native, status=status),
                    patch.object(pse_env, "compose", return_value={}),
                    patch.object(pse_env, "require_enclosing_owner"),
                    patch.object(pse_env, "unexecutable", return_value=None),
                    patch.object(pse_env, "placement", return_value=["owned-scope"]),
                    patch.object(pse_env.host, "inherit", return_value=owner),
                    patch.object(pse_env.operation, "owner_record", return_value=None),
                    patch.object(pse_env.subprocess, "call", return_value=status),
                    patch.object(test_resources, "registry", return_value=registry),
                    patch.object(
                        test_resources,
                        "reclaim_reports",
                        wraps=test_resources.reclaim_reports,
                    ) as reclaim,
                ):
                    options = ["--native=solver"] if native else []
                    self.assertEqual(
                        pse_env.main([*options, "--", "selected-command"]),
                        128 - status if status < 0 else status,
                    )
                owner.release.assert_called_once()
                reclaim.assert_not_called()
        self.assertEqual(metadata.read_bytes(), original)

    def test_scoped_exits_still_report_own_allocation_failure(self) -> None:
        for native in [False, True]:
            owner = MagicMock(spec=pse_env.host.Allocation)
            owner.release.side_effect = pse_env.BoundaryError("own drain failed")
            with (
                self.subTest(native=native),
                patch.object(pse_env, "compose", return_value={}),
                patch.object(pse_env, "require_enclosing_owner"),
                patch.object(pse_env, "unexecutable", return_value=None),
                patch.object(pse_env, "placement", return_value=["owned-scope"]),
                patch.object(pse_env.host, "inherit", return_value=owner),
                patch.object(pse_env.operation, "owner_record", return_value=None),
                patch.object(pse_env.subprocess, "call", return_value=0),
            ):
                options = ["--native=solver"] if native else []
                self.assertEqual(
                    pse_env.main([*options, "--", "selected-command"]),
                    pse_env.FAILURE,
                )
            owner.release.assert_called_once()

    def test_owned_scope_release_refusal_is_125_and_preserves_child_diagnostic(
        self,
    ) -> None:
        for native in (False, True):
            for status in (0, 7, -signal.SIGTERM):
                owner = MagicMock(spec=pse_env.host.Allocation)
                owner.release.return_value = False
                errors = io.StringIO()
                with (
                    self.subTest(native=native, status=status),
                    patch.object(pse_env, "compose", return_value={}),
                    patch.object(pse_env, "require_enclosing_owner"),
                    patch.object(pse_env, "unexecutable", return_value=None),
                    patch.object(pse_env, "placement", return_value=["owned-scope"]),
                    patch.object(pse_env.host, "inherit", return_value=owner),
                    patch.object(pse_env.operation, "owner_record", return_value=None),
                    patch.object(pse_env.subprocess, "call", return_value=status),
                    patch.object(
                        test_resources,
                        "reclaim_reports",
                        side_effect=AssertionError("unrelated sweep"),
                    ),
                    contextlib.redirect_stderr(errors),
                ):
                    options = ["--native=solver"] if native else []
                    self.assertEqual(
                        pse_env.main([*options, "--", "selected-command"]), 125
                    )
                owner.release.assert_called_once()
                self.assertIn(
                    "own allocation retained until actual drain", errors.getvalue()
                )
                self.assertIn(
                    f"child_exit={128 - status if status < 0 else status}",
                    errors.getvalue(),
                )
                self.assertIn(f"raw_child_status={status}", errors.getvalue())

    def test_inherited_paths_never_release_the_enclosing_allocation(self) -> None:
        for native in (False, True):
            owner = MagicMock(spec=pse_env.host.Allocation)
            owner.release.side_effect = AssertionError("release inherited allocation")
            with (
                self.subTest(native=native),
                patch.object(pse_env, "compose", return_value={}),
                patch.object(pse_env, "require_enclosing_owner"),
                patch.object(pse_env, "unexecutable", return_value=None),
                patch.object(pse_env, "placement", return_value=[]),
                patch.object(pse_env.host, "inherit", return_value=owner),
                patch.object(
                    pse_env.operation,
                    "owner_record",
                    return_value=self.root / "inherited-operation" if native else None,
                ),
                patch.object(
                    pse_env.operation,
                    "environment",
                    side_effect=lambda _requested, env: env,
                ),
                patch.object(pse_env, "execute", return_value=7),
                patch.object(
                    pse_env.subprocess, "call", side_effect=AssertionError("new scope")
                ),
            ):
                options = ["--native=solver"] if native else []
                self.assertEqual(pse_env.main([*options, "--", "selected-command"]), 7)
            owner.release.assert_not_called()

    def test_owned_scope_release_exception_keeps_actual_child_result(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.release.side_effect = pse_env.BoundaryError("own drain failed")
        with (
            patch.object(pse_env.host, "inherit", return_value=owner),
            self.assertRaisesRegex(
                pse_env.BoundaryError,
                "child_exit=7; raw_child_status=7; own drain failed",
            ),
        ):
            pse_env.owned_scope(
                [sys.executable, "-c", "raise SystemExit(7)"], dict(os.environ)
            )
        owner.release.assert_called_once()

    def test_native_owned_scope_failed_fence_preserves_observed_clock_through_release(
        self,
    ) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.nonce = "a" * 32
        owner.directory = self.root / "admission"
        owner.release.return_value = True
        unit = "pse-native-" + "b" * 32 + ".scope"
        handoff = self.root / "handoff"
        observer = MagicMock()
        observer.path = self.root / "observer"
        original = time.monotonic() + 1

        def cancelled(
            _command: list[str],
            _environment: dict[str, str],
            *,
            observation: dict[str, object],
            on_reaped: Callable[[], None],
            **_options: object,
        ) -> int:
            observation.update(child_status=-9, cancellation_deadline=original)
            on_reaped()
            return 143

        with (
            patch.object(pse_env.host, "inherit", return_value=owner),
            patch.object(
                pse_env.host,
                "allocation_metadata",
                return_value=contextlib.nullcontext(
                    {"owners": {owner.nonce: {"units": {unit: {}}}}}
                ),
            ),
            patch.object(pse_env.operation, "Operation") as operation,
            patch.object(pse_env.operation, "prepare_handoff", return_value=handoff),
            patch.object(pse_env.operation, "run", side_effect=cancelled),
            patch.object(pse_env.operation, "_record", return_value={}),
            patch.object(
                pse_env.operation, "record_lock", return_value=contextlib.nullcontext()
            ),
            patch.object(pse_env.operation, "write_json"),
            patch.object(
                pse_env.operation, "drained", return_value=MagicMock(complete=True)
            ) as drain,
            patch("scripts.native_cache.cache_root", return_value=self.root),
        ):
            operation.return_value.__enter__.return_value = observer
            self.assertEqual(
                pse_env.owned_scope(
                    ["systemd-run", "--unit=" + unit, "owned-child"], {}
                ),
                143,
            )
        self.assertEqual(drain.call_args.kwargs["deadline"], original)
        owner.release.assert_called_once_with(deadline=original)

    @contextlib.contextmanager
    def native_owned_scope(self) -> Generator[tuple[MagicMock, dict, MagicMock]]:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.nonce = "a" * 32
        owner.directory = self.root / "admission"
        owner.release.return_value = True
        unit = "pse-native-" + "b" * 32 + ".scope"
        scope = {"unit": unit, "group": "/exact-native", "invocation": "c" * 32}
        handoff = self.root / "handoff"
        observer = MagicMock()
        observer.path = self.root / "observer"
        child_record = {"scope": scope}
        observer_record = {}
        child = MagicMock(pid=424242, returncode=0)
        child.wait.return_value = 0
        with (
            patch.object(pse_env.host, "inherit", return_value=owner),
            patch.object(
                pse_env.host,
                "allocation_metadata",
                return_value=contextlib.nullcontext(
                    {"owners": {owner.nonce: {"units": {unit: {}}}}}
                ),
            ),
            patch.object(pse_env.operation, "Operation") as operation,
            patch.object(pse_env.operation, "prepare_handoff", return_value=handoff),
            patch.object(
                pse_env.operation,
                "_record",
                side_effect=lambda path: (
                    child_record if path == handoff else observer_record
                ),
            ),
            patch.object(
                pse_env.operation, "record_lock", return_value=contextlib.nullcontext()
            ),
            patch.object(pse_env.operation, "write_json"),
            patch.object(pse_env.operation.subprocess, "Popen", return_value=child),
            patch.object(
                pse_env.operation,
                "drained",
                return_value=pse_env.operation.DrainObservation(
                    scope,
                    pse_env.operation.DrainPhase.COMPLETE,
                    "exact owned scope terminal",
                ),
            ) as drain,
            patch("scripts.native_cache.cache_root", return_value=self.root),
        ):
            operation.return_value.__enter__.return_value = observer
            yield owner, child_record, drain

    def native_scope_command(self) -> list[str]:
        return [
            "systemd-run",
            "--unit=pse-native-" + "b" * 32 + ".scope",
            "owned-child",
        ]

    def test_native_owned_scope_joins_transient_active_manager_before_release(
        self,
    ) -> None:
        with self.native_owned_scope() as (owner, record, drain):
            drain.side_effect = [
                pse_env.operation.DrainObservation(
                    record["scope"],
                    pse_env.operation.DrainPhase.ACTIVE,
                    "manager still active after reaping",
                ),
                pse_env.operation.DrainObservation(
                    record["scope"],
                    pse_env.operation.DrainPhase.COMPLETE,
                    "same manager lifetime now terminal",
                ),
            ]
            with patch.object(pse_env.time, "sleep") as pause:
                self.assertEqual(
                    pse_env.owned_scope(self.native_scope_command(), {}), 0
                )
            self.assertEqual(drain.call_count, 2)
            self.assertTrue(
                all(call.args[0] is record for call in drain.call_args_list)
            )
            self.assertEqual(
                drain.call_args_list[0].kwargs["deadline"],
                drain.call_args_list[1].kwargs["deadline"],
            )
            pause.assert_called_once()
            owner.release.assert_called_once_with()

    def test_native_owned_scope_unknown_or_mismatched_manager_still_refuses_release(
        self,
    ) -> None:
        for phase in (
            pse_env.operation.DrainPhase.UNAVAILABLE,
            pse_env.operation.DrainPhase.MISMATCH,
        ):
            with (
                self.subTest(phase=phase),
                self.native_owned_scope() as (owner, record, drain),
            ):
                owner.release.return_value = False
                drain.return_value = pse_env.operation.DrainObservation(
                    record["scope"], phase, "manager identity not established"
                )
                with (
                    patch.object(pse_env.time, "sleep") as pause,
                    self.assertRaisesRegex(
                        pse_env.BoundaryError,
                        "own allocation retained until actual drain",
                    ),
                ):
                    pse_env.owned_scope(self.native_scope_command(), {})
                drain.assert_called_once()
                pause.assert_not_called()
                owner.release.assert_called_once_with()

    def test_native_owned_scope_join_does_not_renew_normal_or_cancellation_clock(
        self,
    ) -> None:
        for cancellation in (None, 100.025):
            with (
                self.subTest(cancellation=cancellation),
                self.native_owned_scope() as (owner, record, drain),
            ):
                elapsed = [100.0]
                if cancellation is not None:
                    record["cancel_deadline"] = cancellation
                drain.return_value = pse_env.operation.DrainObservation(
                    record["scope"],
                    pse_env.operation.DrainPhase.ACTIVE,
                    "manager has not settled",
                )

                def advance(delay: float, clock: list[float] = elapsed) -> None:
                    clock[0] += delay

                with (
                    patch.object(
                        pse_env.time,
                        "monotonic",
                        side_effect=lambda clock=elapsed: clock[0],
                    ),
                    patch.object(pse_env.time, "sleep", side_effect=advance),
                    self.assertRaises(pse_env.operation.DeadlineExpiredError),
                ):
                    pse_env.owned_scope(self.native_scope_command(), {})
                expected = cancellation if cancellation is not None else 110.0
                self.assertEqual(elapsed[0], expected)
                self.assertGreater(drain.call_count, 1)
                self.assertTrue(
                    all(
                        call.kwargs["deadline"] == expected
                        for call in drain.call_args_list
                    )
                )
                owner.release.assert_called_once_with(
                    **({"deadline": cancellation} if cancellation is not None else {})
                )

    def test_native_owned_scope_first_signal_during_join_keeps_nonraising_handler_and_original_clock(
        self,
    ) -> None:
        with self.native_owned_scope() as (owner, record, drain):
            original = 100.025
            previous = signal.getsignal(signal.SIGTERM)

            def observe(
                _record: dict, *, deadline: float
            ) -> pse_env.operation.DrainObservation:
                if drain.call_count == 1:
                    self.assertEqual(deadline, 110.0)
                    handler = signal.getsignal(signal.SIGTERM)
                    if not callable(handler):
                        self.fail("join lost the non-raising native signal handler")
                    handler(signal.SIGTERM, None)
                    return pse_env.operation.DrainObservation(
                        record["scope"],
                        pse_env.operation.DrainPhase.ACTIVE,
                        "first signal arrived while joining",
                    )
                self.assertEqual(deadline, original)
                return pse_env.operation.DrainObservation(
                    record["scope"],
                    pse_env.operation.DrainPhase.COMPLETE,
                    "exact receiver terminal",
                )

            drain.side_effect = observe
            with (
                patch.object(pse_env.time, "monotonic", return_value=100.0),
                patch.object(pse_env.time, "sleep") as pause,
                patch.object(
                    pse_env.operation,
                    "cancel_foreground",
                    return_value=(original, signal.SIGTERM),
                ),
            ):
                self.assertEqual(
                    pse_env.owned_scope(self.native_scope_command(), {}),
                    128 + signal.SIGTERM,
                )
            self.assertEqual(signal.getsignal(signal.SIGTERM), previous)
            self.assertEqual(drain.call_count, 2)
            pause.assert_not_called()
            owner.release.assert_called_once_with(deadline=original)

    def test_native_owned_scope_final_run_handoff_preserves_late_original_clock(
        self,
    ) -> None:
        with self.native_owned_scope() as (owner, _record, _drain):
            original = time.monotonic() + 1
            run = pse_env.operation.run

            def finish(
                command: list[str],
                environment: dict[str, str],
                *,
                observation: dict[str, object],
                on_reaped: Callable[[], None],
                cleanup_clock: Callable[[], float | None],
                foreground: Path,
            ) -> int:
                self.assertEqual(
                    run(
                        command,
                        environment,
                        observation=observation,
                        on_reaped=on_reaped,
                        cleanup_clock=cleanup_clock,
                        foreground=foreground,
                    ),
                    0,
                )
                observation["cancellation_deadline"] = original
                return 143

            with patch.object(pse_env.operation, "run", side_effect=finish):
                self.assertEqual(
                    pse_env.owned_scope(self.native_scope_command(), {}), 143
                )
            owner.release.assert_called_once_with(deadline=original)

    def test_new_native_operation_borrows_inherited_allocation_without_release(
        self,
    ) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.release.side_effect = AssertionError("release inherited allocation")
        with (
            patch.dict(os.environ, {}, clear=False),
            patch.object(pse_env.operation, "owner_record", return_value=None),
            patch.object(pse_env, "placement", return_value=[]),
            patch.object(pse_env.host, "inherit", return_value=owner),
            patch.object(pse_env.operation, "Operation") as operation,
            patch.object(
                pse_env.operation,
                "environment",
                side_effect=lambda _requested, env: env,
            ),
            patch.object(pse_env.operation, "run", return_value=7),
            patch("scripts.native_cache.cache_root", return_value=self.root),
        ):
            self.assertEqual(
                pse_env.native(self.root, [], ["selected-command"], {}, scope=True), 7
            )
        owner.release.assert_not_called()
        operation.return_value.__exit__.assert_called_once()

    def test_actual_release_refusal_retains_charge_and_real_child_status(self) -> None:
        for raw, program in (
            (0, "raise SystemExit(0)"),
            (7, "raise SystemExit(7)"),
            (-signal.SIGTERM, "import os,signal; os.kill(os.getpid(), signal.SIGTERM)"),
        ):
            directory = self.root / f"allocation-{raw}"
            allocation = pse_env.host.acquire(
                pse_env.host.select("light"), directory=directory
            )
            allocation.register("owned-refusal.scope")
            environment = {**os.environ, **allocation.environment()}
            status = 128 - raw if raw < 0 else raw
            with (
                self.subTest(raw=raw),
                patch.object(
                    pse_env.host.operation,
                    "unit_observation",
                    return_value={
                        "LoadState": "loaded",
                        "ActiveState": "active",
                        "ControlGroup": "/unbound-but-active",
                    },
                ),
                self.assertRaisesRegex(
                    pse_env.BoundaryError,
                    f"child_exit={status}; raw_child_status={raw}",
                ),
            ):
                pse_env.owned_scope([sys.executable, "-c", program], environment)
            with pse_env.host.allocation_metadata(directory) as ledger:
                retained = ledger["owners"][allocation.nonce]
                self.assertTrue(retained["released"])
                self.assertIn("owned-refusal.scope", retained["units"])

    def test_docs_bash_wrapper_cannot_fall_through_to_product_python(self) -> None:
        product = self.root / ".venv/bin"
        product.mkdir(parents=True)
        (product / "python3").write_text("product interpreter fixture")
        with (
            patch.object(pse_env, "ROOT", self.root),
            patch.object(
                pse_env,
                "compose",
                return_value={
                    "UV_PROJECT_ENVIRONMENT": ".venv",
                    "PATH": str(product) + ":/bin",
                },
            ),
            patch.object(pse_env, "execute") as execute,
            patch.object(pse_env, "placement", return_value=[]) as placement,
            patch.object(pse_env, "unexecutable", return_value=None),
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env, "require_enclosing_owner"),
        ):
            self.assertEqual(
                pse_env.main(
                    ["--docs", "--", "bash", "-c", "python3 -m scripts.docs build"]
                ),
                pse_env.FAILURE,
            )
            execute.assert_not_called()
            placement.assert_not_called()
            execute.return_value = 0
            self.assertEqual(
                pse_env.main(
                    [
                        "--docs",
                        "--",
                        "uv",
                        "sync",
                        "--locked",
                        "--only-group",
                        "docs",
                        "--no-install-project",
                    ]
                ),
                0,
            )
            self.assertEqual(
                execute.call_args.args[1]["UV_PROJECT_ENVIRONMENT"],
                str(self.root / ".venv-docs"),
            )

    def test_docs_selector_overrides_product_without_synchronizing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            selected = pse_env.documentation_environment(
                root, {"UV_PROJECT_ENVIRONMENT": ".venv", "PATH": "/bin"}
            )
            self.assertEqual(
                selected["UV_PROJECT_ENVIRONMENT"], str(root / ".venv-docs")
            )
            self.assertEqual(
                selected["PATH"].split(":")[0], str(root / ".venv-docs/bin")
            )
            self.assertFalse((root / ".venv-docs").exists())

    def test_docs_selector_rejects_product_alias_before_effects(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".venv").mkdir()
            (root / ".venv-docs").symlink_to(root / ".venv", target_is_directory=True)
            with self.assertRaisesRegex(pse_env.BoundaryError, "aliases"):
                pse_env.documentation_environment(
                    root, {"UV_PROJECT_ENVIRONMENT": ".venv"}
                )

    def test_docs_print_moves_inherited_docs_path_ahead_of_product(self) -> None:
        inherited = f"{self.root}/.venv/bin:{self.root}/.venv-docs/bin:/bin"
        with patch.object(pse_env, "local_keys", return_value={}):
            rendered = pse_env.render(
                self.root, {"HOME": str(self.root), "PATH": inherited}, docs=True
            )
        result = subprocess.run(
            [
                "bash",
                "-c",
                rendered
                + '\nprintf "%s\\n" "$PATH"\n'
                + rendered
                + '\nprintf "%s\\n" "$PATH"',
            ],
            env={"PATH": inherited},
            capture_output=True,
            text=True,
            check=True,
        )
        once, twice = result.stdout.splitlines()
        self.assertEqual(once.split(":")[0], str(self.root / ".venv-docs/bin"))
        self.assertEqual(once, twice)

    def test_docs_print_selects_child_environment_even_with_local_product_selector(
        self,
    ) -> None:
        with patch.object(
            pse_env, "local_keys", return_value={"UV_PROJECT_ENVIRONMENT": ".venv"}
        ):
            rendered = pse_env.render(
                self.root, {"HOME": str(self.root), "PATH": "/bin"}, docs=True
            )
        result = subprocess.run(
            [
                "bash",
                "-c",
                rendered + '\nprintf "%s\\n" "$UV_PROJECT_ENVIRONMENT" "$PATH"',
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        lines = result.stdout.splitlines()
        self.assertEqual(lines[0], str(self.root / ".venv-docs"))
        self.assertEqual(lines[1].split(":")[0], str(self.root / ".venv-docs/bin"))

    def setUp(self) -> None:
        self.cpu_ceiling = pse_env.cpu_ceiling
        for owner, attribute, value in (
            (pse_env, "cpu_ceiling", 16),
            (pse_env.os, "sched_getaffinity", set(range(256))),
            (pse_env.operation, "process_group", "/test-allocation"),
        ):
            mocked = patch.object(owner, attribute, return_value=value)
            mocked.start()
            self.addCleanup(mocked.stop)
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        docker = self.root / "docker/solvers/Dockerfile"
        docker.parent.mkdir(parents=True)
        docker.write_text(
            "FROM fixture AS solvers\nENV MKL_CBWR=COMPATIBLE MKL_DYNAMIC=FALSE OMP_CANCELLATION=TRUE\n"
        )
        (self.root / ".python-version").write_text("3.14.7\n")
        (self.root / ".envrc.local").write_text(
            "export PSE_TEST_LICENSE='s3cret value'\nexport PSE_TEST_OTHER=local\n"
        )
        configured = patch.object(pse_env.build_environment, "configure", identity)
        configured.start()
        self.addCleanup(configured.stop)

    def test_recipe_class_precedence_before_placement_and_nested_ownership(
        self,
    ) -> None:
        marker = {
            pse_env.host.MARKER: "/owner/" + "a" * 32,
            "PSE_RESOURCE_CLASS": "exclusive",
        }
        cases = [
            ({}, {}, [], "compile"),
            ({"PSE_RESOURCE_CLASS": "wide"}, {}, [], "wide"),
            ({}, {"PSE_RESOURCE_CLASS": "timing"}, [], "timing"),
            (
                {"PSE_RESOURCE_CLASS": "wide"},
                {},
                ["--resource-class", "light"],
                "light",
            ),
            (marker, {}, [], "exclusive"),
        ]
        for caller, local, options, expected in cases:
            with (
                self.subTest(expected=expected),
                patch.dict(os.environ, caller, clear=True),
                patch.object(pse_env, "ROOT", self.root),
                patch.object(pse_env, "local_keys", return_value=local),
                patch.object(pse_env.host, "inherit", return_value=None),
                patch.object(pse_env, "unexecutable", return_value=None),
                patch.object(pse_env, "placement", return_value=[]) as placement,
                patch.object(pse_env, "execute", return_value=0),
            ):
                self.assertEqual(
                    pse_env.main(
                        [
                            *options,
                            "--recipe-default-class",
                            "compile",
                            "--",
                            "bash",
                            "-c",
                            "cargo check",
                        ]
                    ),
                    0,
                )
                self.assertEqual(
                    placement.call_args.args[0]["PSE_RESOURCE_CLASS"], expected
                )

    def test_admitted_jobs_follow_declaration_and_preserve_explicit_overrides(
        self,
    ) -> None:
        declaration = "16"

        def declared(_root: Path, env: dict[str, str]) -> dict[str, str]:
            result = dict(env)
            result.setdefault("CARGO_BUILD_JOBS", declaration)
            return result

        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = MagicMock(cores=(2, 4, 6, 8))
        with (
            patch.object(pse_env.build_environment, "configure", side_effect=declared),
            patch.object(pse_env.host, "cpu_set", return_value=(2, 4, 6, 8)),
        ):
            env = pse_env.compose(self.root, {}, {})
            self.assertEqual(env["CARGO_BUILD_JOBS"], "16")
            pse_env.admitted_cargo_jobs(env, owner)
            self.assertEqual(env["CARGO_BUILD_JOBS"], "4")
            # Re-entry starts from the current declaration, not the previous lane.
            nested = pse_env.compose(self.root, env, {})
            self.assertEqual(nested["CARGO_BUILD_JOBS"], "16")
            described = pse_env.operation_description(self.root, env, None, local={})
            self.assertEqual(described["compiler"]["CARGO_BUILD_JOBS"]["value"], "16")
            self.assertEqual(
                described["compiler"]["CARGO_BUILD_JOBS"]["source"], "default"
            )
            explicit = pse_env.operation_description(
                self.root, {**env, "CARGO_BUILD_JOBS": "24"}, None, local={}
            )
            self.assertEqual(explicit["compiler"]["CARGO_BUILD_JOBS"]["value"], "24")
            self.assertEqual(
                explicit["compiler"]["CARGO_BUILD_JOBS"]["source"], "caller"
            )
            pse_env.admitted_cargo_jobs(nested, owner)
            self.assertEqual(nested, env)
            declaration = "3"
            changed = pse_env.compose(self.root, env, {})
            pse_env.admitted_cargo_jobs(changed, owner)
            self.assertEqual(changed["CARGO_BUILD_JOBS"], "3")
            for explicit in ("2", "24", "-1"):
                overridden = pse_env.compose(
                    self.root, {**env, "CARGO_BUILD_JOBS": explicit}, {}
                )
                pse_env.admitted_cargo_jobs(overridden, owner)
                self.assertEqual(overridden["CARGO_BUILD_JOBS"], explicit)
                self.assertNotIn(pse_env.CARGO_DEFAULT_JOBS, overridden)
            local = pse_env.compose(self.root, {}, {"CARGO_BUILD_JOBS": "12"})
            pse_env.admitted_cargo_jobs(local, owner)
            self.assertEqual(local["CARGO_BUILD_JOBS"], "12")
            # A declaration already below the lane stays unchanged.
            small = {"CARGO_BUILD_JOBS": "2", pse_env.CARGO_DEFAULT_JOBS: "2"}
            pse_env.admitted_cargo_jobs(small, owner)
            self.assertEqual(small["CARGO_BUILD_JOBS"], "2")

    def test_smt_increases_compiler_capacity_but_keeps_physical_runner_default(
        self,
    ) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = MagicMock(cores=(2, 4))
        with patch.object(pse_env.host, "cpu_set", return_value=(2, 4, 6, 8)):
            env = {"CARGO_BUILD_JOBS": "16", pse_env.CARGO_DEFAULT_JOBS: "16"}
            pse_env.admitted_cargo_jobs(env, owner)
            self.assertEqual(env["CARGO_BUILD_JOBS"], "4")
            self.assertEqual(env["NEXTEST_TEST_THREADS"], "2")
            # Re-entry adapts a repository default; explicit CLI/env overrides survive.
            owner.profile.cores = (2,)
            pse_env.admitted_cargo_jobs(env, owner)
            self.assertEqual(env["NEXTEST_TEST_THREADS"], "1")
            env["NEXTEST_TEST_THREADS"] = "7"
            pse_env.admitted_cargo_jobs(env, owner)
            self.assertEqual(env["NEXTEST_TEST_THREADS"], "7")
            env["CARGO_BUILD_JOBS"] = "9"
            pse_env.admitted_cargo_jobs(env, owner)
            self.assertEqual(env["CARGO_BUILD_JOBS"], "9")

    def test_unknown_native_orchestration_gets_full_unbounded_defaults(self) -> None:
        command = [".venv/bin/python", "/fixture/native_orchestration.py"]
        selected = pse_env.selected_resource_class(
            command, {"PSE_BUILD_MODE": "unbounded"}, native=True
        )
        self.assertEqual(selected, "build")
        for native in (False, True):
            for default in (None, "functional", "wide", "compile", "exclusive"):
                with self.subTest(native=native, default=default):
                    self.assertEqual(
                        pse_env.selected_resource_class(
                            command,
                            {"PSE_BUILD_MODE": "unbounded"},
                            native=native,
                            default=default,
                        ),
                        "build",
                    )
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = pse_env.host.select(selected)
        self.assertEqual(len(owner.profile.cores), 16)
        self.assertTrue(owner.profile.unbounded)
        with patch.object(pse_env, "cpu_ceiling", return_value=32):
            env = {"CARGO_BUILD_JOBS": "32", pse_env.CARGO_DEFAULT_JOBS: "32"}
            pse_env.admitted_cargo_jobs(env, owner)
        self.assertEqual(env["CARGO_BUILD_JOBS"], "32")
        self.assertEqual(env["NEXTEST_TEST_THREADS"], "16")
        for selection in ("timing", "reference"):
            self.assertEqual(
                pse_env.selected_resource_class(
                    command,
                    {"PSE_BUILD_MODE": "unbounded"},
                    native=True,
                    default=selection,
                ),
                selection,
            )
        for explicit in ("functional", "compile", "wide", "exclusive"):
            self.assertEqual(
                pse_env.selected_resource_class(
                    command,
                    {"PSE_BUILD_MODE": "unbounded"},
                    native=True,
                    explicit=explicit,
                ),
                explicit,
            )
        self.assertEqual(
            pse_env.selected_resource_class(
                command,
                {"PSE_BUILD_MODE": "unbounded", "PSE_MEMORY_MAX": "40G"},
                native=True,
            ),
            "functional",
        )

    def test_width_intersects_affinity_and_ancestor_quota(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = MagicMock(cores=(2, 4, 6, 8))
        for affinity, quota, expected in (
            ({2, 4, 6, 8, 10, 12, 14, 16}, 4, 4),
            ({2, 10}, 4, 2),
            ({2, 4, 6, 8}, 2.5, 2),
            ({2, 4}, 0.5, 1),
        ):
            with (
                self.subTest(affinity=affinity, quota=quota),
                patch.object(
                    pse_env.host, "cpu_set", return_value=(2, 4, 6, 8, 10, 12, 14, 16)
                ),
                patch.object(pse_env.os, "sched_getaffinity", return_value=affinity),
                patch.object(pse_env, "cpu_ceiling", return_value=quota),
            ):
                env = {"CARGO_BUILD_JOBS": "16", pse_env.CARGO_DEFAULT_JOBS: "16"}
                pse_env.admitted_cargo_jobs(env, owner)
                self.assertEqual(env["CARGO_BUILD_JOBS"], str(expected))
                self.assertEqual(env["NEXTEST_TEST_THREADS"], str(expected))
                env.update(CARGO_BUILD_JOBS="24", NEXTEST_TEST_THREADS="7")
                pse_env.admitted_cargo_jobs(env, owner)
                self.assertEqual(env["CARGO_BUILD_JOBS"], "24")
                self.assertEqual(env["NEXTEST_TEST_THREADS"], "7")

    def test_width_refuses_unknown_quota_and_empty_usable_affinity(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = MagicMock(cores=(2, 4))
        for quota, affinity in (
            (None, {2, 4}),
            (float("inf"), {2, 4}),
            (0, {2, 4}),
            (2, {9}),
        ):
            with (
                self.subTest(quota=quota, affinity=affinity),
                patch.object(pse_env.host, "cpu_set", return_value=(2, 4)),
                patch.object(pse_env.os, "sched_getaffinity", return_value=affinity),
                patch.object(pse_env, "cpu_ceiling", return_value=quota),
                self.assertRaises(pse_env.BoundaryError),
            ):
                pse_env.runner_width(owner)

    def test_cpu_ceiling_reads_narrowest_finite_ancestor(self) -> None:
        # Bypass the fixture's injected kernel observation for this filesystem control.
        root = self.root / "cgroup"
        child = root / "allocation" / "observer"
        child.mkdir(parents=True)
        (root / "cpu.max").write_text("max 100000")
        (child.parent / "cpu.max").write_text("250000 100000")
        (child / "cpu.max").write_text("400000 100000")
        self.assertEqual(self.cpu_ceiling(child, root=root), 2.5)

    def test_placement_bounds_jobs_after_acquisition(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = pse_env.host.select("compile")
        owner.directory = self.root / "allocation"
        owner.nonce = "a" * 32
        owner.environment.return_value = {
            pse_env.host.MARKER: str(owner.directory / owner.nonce)
        }
        env = {
            "CARGO_BUILD_JOBS": "16",
            pse_env.CARGO_DEFAULT_JOBS: "16",
            "PSE_RESOURCE_CLASS": "compile",
        }
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire", return_value=owner),
            patch.object(pse_env.host, "enforce_parent"),
            patch.object(pse_env.host, "enforce_allocation"),
            patch.object(pse_env.host, "cpu_set", return_value=(1, 2, 3, 4)),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=True),
        ):
            prefix = pse_env.placement(env, native=False)
        self.assertIn("--bind-allocation", prefix)
        self.assertIn("CPUQuota=400%", prefix)
        self.assertEqual(env["CARGO_BUILD_JOBS"], "4")
        self.assertEqual(env[pse_env.CARGO_DEFAULT_JOBS], "4")

    def test_control_environment_does_not_prepare_compiler(self) -> None:
        deadline = time.monotonic() + 1
        with patch.object(
            pse_env.build_environment,
            "configure",
            side_effect=AssertionError("compiler"),
        ):
            env = pse_env.control_environment(
                self.root,
                {"PATH": "/usr/bin", "PSE_TEST_OTHER": "caller"},
                deadline=deadline,
            )
        self.assertEqual(env["PSE_TEST_OTHER"], "caller")
        self.assertEqual(env["PSE_TEST_LICENSE"], "s3cret value")
        self.assertIn(str(self.root / ".venv/bin"), env["PATH"])

    def test_light_control_uses_one_deadline_without_scientific_recovery(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root
        owner.nonce = "a" * 32
        owner.profile = pse_env.host.select("light")
        owner.environment.return_value = {}
        deadline = time.monotonic() + 0.5
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(
                pse_env.host, "acquire_light_control", return_value=owner
            ) as acquire,
            patch.object(pse_env.host, "enforce_parent") as parent,
            patch.object(pse_env.host, "enforce_allocation") as allocation,
            patch.object(pse_env.host, "release_light_control") as release,
            patch.object(
                pse_env.host, "reconcile", side_effect=AssertionError("reconcile")
            ),
            patch.object(
                pse_env, "compose", side_effect=AssertionError("compiler environment")
            ),
            patch.object(
                pse_env.host,
                "control_run",
                side_effect=[
                    subprocess.CompletedProcess([], 0),
                    subprocess.CompletedProcess([], 0, "headers", ""),
                ],
            ) as run,
        ):
            result = pse_env.run_light_control(["fixed-helper"], {}, deadline=deadline)
        self.assertEqual(result.stdout, "headers")
        self.assertEqual(acquire.call_args.kwargs["deadline"], deadline)
        self.assertEqual(parent.call_args.kwargs["deadline"], deadline)
        self.assertEqual(allocation.call_args.kwargs["deadline"], deadline)
        self.assertEqual(release.call_args.kwargs["deadline"], deadline)
        self.assertEqual(run.call_args.kwargs["deadline"], deadline)
        self.assertIn("--scope", run.call_args.args[0])
        self.assertEqual(run.call_args.args[0][-1], "fixed-helper")

    def test_nested_control_uses_verified_owner_without_new_admission(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        deadline = time.monotonic() + 0.5
        with (
            patch.object(pse_env.host, "inherit", return_value=owner) as inherit,
            patch.object(
                pse_env.host,
                "acquire_light_control",
                side_effect=AssertionError("new admission"),
            ),
            patch.object(
                pse_env.host,
                "release_light_control",
                side_effect=AssertionError("release enclosing owner"),
            ),
            patch.object(
                pse_env.host,
                "control_run",
                return_value=subprocess.CompletedProcess([], 0, "headers", ""),
            ) as run,
        ):
            pse_env.run_light_control(["fixed-helper"], {}, deadline=deadline)
        self.assertEqual(inherit.call_args.kwargs["deadline"], deadline)
        self.assertEqual(run.call_args.args[0], ["fixed-helper"])

    def test_control_does_not_return_output_when_cleanup_is_unverified(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root
        owner.nonce = "a" * 32
        owner.profile = pse_env.host.select("light")
        owner.environment.return_value = {}
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire_light_control", return_value=owner),
            patch.object(pse_env.host, "enforce_parent"),
            patch.object(pse_env.host, "enforce_allocation"),
            patch.object(pse_env.host, "release_light_control", return_value=False),
            patch.object(
                pse_env.host,
                "control_run",
                side_effect=[
                    subprocess.CompletedProcess([], 0),
                    subprocess.CompletedProcess([], 0, "secret output", ""),
                ],
            ),
            self.assertRaisesRegex(pse_env.BoundaryError, "retained"),
        ):
            pse_env.run_light_control(
                ["fixed-helper"], {}, deadline=time.monotonic() + 1
            )

    def test_caller_wins_over_local_and_local_fills_the_rest(self) -> None:
        env = pse_env.compose(
            self.root, {"PSE_TEST_OTHER": "caller", "PATH": "/usr/bin"}
        )
        self.assertEqual(env["PSE_TEST_OTHER"], "caller")
        self.assertEqual(env["PSE_TEST_LICENSE"], "s3cret value")
        self.assertEqual(env["PATH"].split(os.pathsep)[0], str(self.root / ".venv/bin"))

    def test_composition_is_idempotent(self) -> None:
        once = pse_env.compose(self.root, {"PATH": "/usr/bin:/bin"})
        self.assertEqual(pse_env.compose(self.root, once), once)

    def test_print_defers_local_values_and_evaluates_to_them(self) -> None:
        text = pse_env.render(
            self.root, {"PATH": "/usr/bin:/bin", "HOME": "/nonexistent-home"}
        )
        self.assertNotIn("s3cret", text)
        result = subprocess.run(
            [
                "bash",
                "-c",
                text + 'printf "%s|%s" "$PSE_TEST_LICENSE" "$PSE_TEST_OTHER"',
            ],
            capture_output=True,
            text=True,
            check=True,
            env={"PATH": "/usr/bin:/bin", "HOME": "/nonexistent-home"},
        )
        self.assertEqual(result.stdout, "s3cret value|local")

    def test_print_keeps_a_caller_value_over_the_deferred_local_one(self) -> None:
        text = pse_env.render(self.root, {"PATH": "/usr/bin:/bin"})
        result = subprocess.run(
            ["bash", "-c", text + 'printf "%s" "$PSE_TEST_OTHER"'],
            capture_output=True,
            text=True,
            check=True,
            env={"PATH": "/usr/bin:/bin", "PSE_TEST_OTHER": "mine"},
        )
        self.assertEqual(result.stdout, "mine")

    def test_slices_nest_by_dash(self) -> None:
        group = pse_env.slice_group("pse-agents.slice")
        self.assertEqual(group.parts[-2:], ("pse.slice", "pse-agents.slice"))

    def test_placement_defaults_and_overrides(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.profile = pse_env.host.select("functional")
        owner.directory = self.root / "admission"
        owner.nonce = "a" * 32
        owner.environment.return_value = {
            pse_env.host.MARKER: str(owner.directory / owner.nonce)
        }
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire", return_value=owner) as acquire,
            patch.object(pse_env.host, "enforce_parent"),
            patch.object(pse_env.host, "enforce_allocation"),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=True),
        ):
            env = {}
            placed = pse_env.placement(env, native=False)
            profile = acquire.call_args.args[0]
            self.assertIn(f"--slice={pse_env.host.allocation_slice(owner)}", placed)
            self.assertIn(f"MemoryMax={profile.memory}", placed)
            self.assertIn("--bind-allocation", placed)
            self.assertTrue(any(arg.startswith("--unit=pse-cmd-") for arg in placed))
            self.assertEqual(
                env[pse_env.host.MARKER], str(owner.directory / owner.nonce)
            )
            with self.assertRaises(pse_env.host.AdmissionError):
                pse_env.placement({"PSE_MEMORY_MAX": "off"}, native=True)

    def test_placement_stays_inside_an_owning_scope_or_without_a_manager(self) -> None:
        owner = {
            "unit": "pse-native-" + "a" * 32 + ".scope",
            "group": "/g",
            "invocation": "b" * 32,
        }
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.operation, "scope_owner", return_value=owner),
            self.assertRaisesRegex(
                pse_env.BoundaryError, "no verified host allocation"
            ),
        ):
            pse_env.placement({}, native=False)
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=False),
            self.assertRaisesRegex(pse_env.BoundaryError, "systemd user manager"),
        ):
            pse_env.placement({}, native=True)

    def test_aggregate_limit_below_the_command_cap_is_reported(self) -> None:
        owner = MagicMock(spec=pse_env.host.Allocation)
        owner.directory = self.root / "admission"
        owner.nonce = "a" * 32
        owner.environment.return_value = {}
        with (
            patch.object(pse_env.host, "inherit", return_value=None),
            patch.object(pse_env.host, "acquire", return_value=owner),
            patch.object(pse_env.operation, "scope_owner", return_value=None),
            patch.object(pse_env, "manager_available", return_value=True),
            patch.object(
                pse_env.host,
                "enforce_parent",
                side_effect=pse_env.host.AdmissionError(
                    "Effective ancestor is below requested aggregate"
                ),
            ),
            self.assertRaisesRegex(
                pse_env.host.AdmissionError, "below requested aggregate"
            ),
        ):
            pse_env.placement({"PSE_MEMORY_MAX": "120G"}, native=False)
        owner.register.assert_not_called()

    def test_boundary_failure_and_missing_command_statuses(self) -> None:
        with patch.object(
            pse_env, "compose", side_effect=pse_env.BoundaryError("broken")
        ):
            self.assertEqual(
                pse_env.main(["--no-scope", "--", "true"]), pse_env.FAILURE
            )
        with patch.object(pse_env, "compose", return_value={"PATH": "/usr/bin:/bin"}):
            self.assertEqual(
                pse_env.main(["--no-scope", "--", "pse-env-missing-command"]), 127
            )


class StoreReadinessTests(unittest.TestCase):
    def test_store_deadline_survives_host_lookup_and_refuses_late_effects(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            config: dict[str, object] = {"admission": "open", "parked": True}
            surreal_server.write_json(state / "config.json", config)
            environment = {"PSE_SURREAL_STATE": str(state)}
            with patch.object(
                surreal_server.time, "monotonic", return_value=100
            ) as clock:

                def inherited(_env: object, *, deadline: float) -> MagicMock:
                    self.assertEqual(deadline, 140)
                    clock.return_value = 141
                    return MagicMock(spec=pse_env.host.Allocation)

                with (
                    patch.object(surreal_server, "config_for", return_value=config),
                    patch.object(
                        pse_env.host, "inherit", side_effect=inherited
                    ) as inherit,
                    patch.object(surreal_server, "unpark_service") as resume,
                    patch.object(surreal_server, "start") as start,
                    patch.object(surreal_server, "service_readiness") as observe,
                    self.assertRaises(surreal_server.DeadlineExpiredError),
                ):
                    pse_env.require_store(environment, deadline=140)
                inherit.assert_called_once_with(environment, deadline=140)
                resume.assert_not_called()
                start.assert_not_called()
                observe.assert_not_called()

    def test_store_readiness_deadline_reaches_storage_ledger(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            config: dict[str, object] = {
                "admission": "open",
                "instance_id": "selected",
                "resources": {"server_memory_bytes": 1024},
            }
            surreal_server.write_json(state / "config.json", config)
            surreal_server.write_json(
                state / "service-launch.json",
                {
                    "deadline": 90,
                    "generation": "selected",
                    "service_class": "functional",
                    "server_memory_bytes": 1024,
                    "allocation": str(state / "ledger" / ("a" * 32)),
                },
            )
            with patch.object(
                surreal_server.time, "monotonic", return_value=100
            ) as clock:

                def contended(selected: Path, *, deadline: float) -> dict[str, object]:
                    self.assertEqual(selected, state / "ledger")
                    self.assertEqual(deadline, 140)
                    clock.return_value = 141
                    raise pse_env.host.AdmissionError("original ledger clock expired")

                with (
                    patch.object(surreal_server, "config_for", return_value=config),
                    patch.object(
                        pse_env.host,
                        "inherit",
                        return_value=MagicMock(spec=pse_env.host.Allocation),
                    ),
                    patch.object(
                        surreal_server,
                        "systemctl",
                        return_value=subprocess.CompletedProcess(
                            [], 0, "ActiveState=activating\n"
                        ),
                    ),
                    patch.object(
                        pse_env.host, "readonly_snapshot", side_effect=contended
                    ) as ledger,
                    patch.object(surreal_server, "start") as start,
                    patch.object(surreal_server.urllib.request, "urlopen") as health,
                    self.assertRaises(surreal_server.DeadlineExpiredError),
                ):
                    pse_env.require_store(
                        {"PSE_SURREAL_STATE": str(state)}, deadline=140
                    )
                ledger.assert_called_once_with(state / "ledger", deadline=140)
                start.assert_not_called()
                health.assert_not_called()

    def test_store_readiness_health_uses_original_remaining_budget(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            config: dict[str, object] = {"admission": "open", "port": 18244}
            surreal_server.write_json(state / "config.json", config)
            response = MagicMock()
            response.__enter__.return_value.status = 200
            with (
                patch.object(surreal_server.time, "monotonic", return_value=100),
                patch.object(surreal_server, "config_for", return_value=config),
                patch.object(
                    pse_env.host,
                    "inherit",
                    return_value=MagicMock(spec=pse_env.host.Allocation),
                ) as inherit,
                patch.object(
                    surreal_server,
                    "systemctl",
                    return_value=subprocess.CompletedProcess(
                        [], 0, "ActiveState=active\n"
                    ),
                ),
                patch.object(
                    surreal_server, "listening_inodes", return_value={"selected-socket"}
                ),
                patch.object(
                    surreal_server, "_owns_listener", return_value=True
                ) as listener,
                patch.object(
                    surreal_server, "protocol_ready", return_value=True
                ) as protocol,
                patch.object(
                    surreal_server.urllib.request, "urlopen", return_value=response
                ) as health,
                patch.object(surreal_server, "start") as start,
            ):
                pse_env.require_store(
                    {"PSE_SURREAL_STATE": str(state)}, deadline=100.25
                )
            self.assertEqual(inherit.call_args.kwargs["deadline"], 100.25)
            self.assertEqual(listener.call_count, 2)
            self.assertEqual(protocol.call_count, 2)
            self.assertEqual(health.call_count, 2)
            for observed in (listener, protocol):
                for call in observed.call_args_list:
                    self.assertEqual(call.kwargs["deadline"], 100.25)
            for call in health.call_args_list:
                self.assertEqual(call.kwargs["timeout"], 0.25)
            start.assert_not_called()

    def test_store_deadline_uses_earlier_enclosing_lifecycle_clock(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            config: dict[str, object] = {"admission": "open", "parked": True}
            surreal_server.write_json(state / "config.json", config)
            with patch.object(
                surreal_server.time, "monotonic", return_value=100
            ) as clock:

                def inherited(_env: object, *, deadline: float) -> MagicMock:
                    self.assertEqual(deadline, 120)
                    clock.return_value = 121
                    return MagicMock(spec=pse_env.host.Allocation)

                with (
                    self.assertRaises(surreal_server.DeadlineExpiredError),
                    surreal_server.lifecycle_deadline(120),
                    patch.object(
                        surreal_server, "config_for", return_value=config
                    ) as configuration,
                    patch.object(pse_env.host, "inherit", side_effect=inherited),
                    patch.object(surreal_server, "unpark_service") as resume,
                    patch.object(surreal_server, "start") as start,
                ):
                    pse_env.require_store(
                        {"PSE_SURREAL_STATE": str(state)}, deadline=140
                    )
                configuration.assert_called_once_with(state, deadline=120)
                resume.assert_not_called()
                start.assert_not_called()

    def test_functional_explicit_store_demand_resumes_only_selected_independent_store(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary) / "selected"
            state.mkdir(mode=0o700)
            directory = Path(temporary) / "admission"
            config: dict[str, object] = {
                "parked": True,
                "admission": "quiesced",
                "accepting_writes": False,
                "resources": surreal_server.execution_resources("functional"),
                "instance_id": "selected-functional",
                "service_class": "functional",
                "resident": True,
            }
            surreal_server.write_json(state / "config.json", config)
            caller = MagicMock(spec=pse_env.host.Allocation)
            caller.profile = pse_env.host.select("functional")
            deadline = time.monotonic() + 1

            def resume(selected: Path) -> None:
                self.assertEqual(selected, state)
                # Startup retains its ordinary independent store allocation,
                # rather than borrowing this non-exclusive scientific caller.
                storage = surreal_server.service_allocation(selected, config, deadline)
                self.assertEqual(
                    storage.profile, pse_env.host.select("store-functional")
                )
                config.update(parked=False, admission="open", accepting_writes=True)
                surreal_server.write_json(state / "config.json", config)

            with (
                patch.object(pse_env.host, "root_path", return_value=directory),
                patch.object(pse_env.host, "inherit", return_value=caller),
                patch.object(
                    pse_env.host,
                    "memory_info",
                    return_value={"MemAvailable": 1 << 40, "MemTotal": 1 << 40},
                ),
                patch.object(pse_env.host, "cpu_set", side_effect=tuple),
                patch.object(
                    surreal_server, "unpark_service", side_effect=resume
                ) as resumed,
                patch.object(
                    surreal_server, "config_for", side_effect=lambda _: dict(config)
                ),
                patch.object(surreal_server, "ready", return_value=True),
                patch.object(surreal_server, "start") as start,
            ):
                pse_env.require_store({"PSE_SURREAL_STATE": str(state)})
            resumed.assert_called_once_with(state)
            start.assert_not_called()
            launch = surreal_server.read_json(state / "service-launch.json")
            self.assertEqual(launch["deadline"], deadline)
            with pse_env.host.allocation_metadata(directory) as ledger:
                (storage,) = ledger["owners"].values()
                self.assertEqual(storage["class"], "store-functional")
                self.assertEqual(storage["memory"], 8 * pse_env.host.GIB)
                self.assertEqual(storage["service"], str(state))

    def test_functional_store_demand_cannot_resume_reference_partition(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            config: dict[str, object] = {
                "parked": True,
                "admission": "quiesced",
                "accepting_writes": False,
                "resources": surreal_server.reference_resources(),
            }
            surreal_server.write_json(state / "config.json", config)
            original = (state / "config.json").read_bytes()
            caller = MagicMock(spec=pse_env.host.Allocation)
            caller.profile = pse_env.host.select("functional")
            with (
                patch.object(pse_env.host, "inherit", return_value=caller),
                patch.object(
                    surreal_server,
                    "unpark_service",
                    side_effect=lambda selected: surreal_server.service_allocation(
                        selected, config, time.monotonic() + 1
                    ),
                ),
                patch.object(pse_env.host, "acquire") as acquire,
                patch.object(surreal_server, "ready") as ready,
                self.assertRaisesRegex(
                    pse_env.host.AdmissionError, "Resident store request exceeds"
                ),
            ):
                pse_env.require_store({"PSE_SURREAL_STATE": str(state)})
            acquire.assert_not_called()
            ready.assert_not_called()
            self.assertEqual((state / "config.json").read_bytes(), original)

    def test_selected_parked_store_unknown_readiness_refusal_escapes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            surreal_server.write_json(
                state / "config.json", {"parked": True, "admission": "quiesced"}
            )
            original = (state / "config.json").read_bytes()
            caller = MagicMock(spec=pse_env.host.Allocation)
            caller.profile = pse_env.host.select("functional")
            with (
                patch.object(pse_env.host, "inherit", return_value=caller),
                patch.object(
                    surreal_server,
                    "unpark_service",
                    side_effect=surreal_server.SupervisorError(
                        "Service readiness is unavailable; startup refused"
                    ),
                ),
                patch.object(surreal_server, "start") as start,
                patch.object(surreal_server, "ready") as ready,
                self.assertRaisesRegex(surreal_server.SupervisorError, "unavailable"),
            ):
                pse_env.require_store({"PSE_SURREAL_STATE": str(state)})
            start.assert_not_called()
            ready.assert_not_called()
            self.assertEqual((state / "config.json").read_bytes(), original)

    def test_store_precheck_without_admitted_owner_is_readonly(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            for admission, parked, allowed in (
                ("open", False, True),
                ("quiesced", True, True),
                ("quiesced", False, False),
                ("validation_required", True, False),
            ):
                with self.subTest(admission=admission, parked=parked):
                    surreal_server.write_json(
                        state / "config.json",
                        {"admission": admission, "parked": parked},
                    )
                    original = (state / "config.json").read_bytes()
                    with (
                        patch.object(pse_env.host, "inherit", return_value=None),
                        patch.object(surreal_server, "unpark_service") as resume,
                        patch.object(surreal_server, "start") as start,
                        patch.object(surreal_server, "ready") as ready,
                    ):
                        if allowed:
                            pse_env.require_store({"PSE_SURREAL_STATE": str(state)})
                        else:
                            with self.assertRaisesRegex(
                                pse_env.BoundaryError, "not admitted"
                            ):
                                pse_env.require_store({"PSE_SURREAL_STATE": str(state)})
                    resume.assert_not_called()
                    start.assert_not_called()
                    ready.assert_not_called()
                    self.assertEqual((state / "config.json").read_bytes(), original)

    def test_unset_up_store_fails_with_the_setup_commands(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            self.assertRaises(pse_env.BoundaryError) as raised,
        ):
            pse_env.require_store({"PSE_SURREAL_STATE": directory})
        self.assertIn("just surreal setup", str(raised.exception))

    def test_store_default_is_the_supervisor_default(self) -> None:
        with patch.object(pse_env.build_environment, "configure", identity):
            env = pse_env.compose(
                Path("/checkout"), {"HOME": "/home/x", "PATH": ""}, {}
            )
        self.assertEqual(
            env["PSE_SURREAL_STATE"],
            "/home/x/.local/state/pse-arrow/surreal-functional-v2",
        )
        with patch.object(pse_env.build_environment, "configure", identity):
            chosen = pse_env.compose(
                Path("/checkout"), {"PSE_SURREAL_STATE": "/elsewhere", "PATH": ""}, {}
            )
        self.assertEqual(chosen["PSE_SURREAL_STATE"], "/elsewhere")


if __name__ == "__main__":
    unittest.main()
