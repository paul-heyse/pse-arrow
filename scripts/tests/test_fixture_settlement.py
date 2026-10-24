# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Controls for the complete Python fixture ownership join."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import importlib.util
import sys
import tempfile
import threading
import time
import types
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import Mock, patch

from scripts import surreal_server, test_resources

if TYPE_CHECKING:
    import pse
    from pse.tests.canonical_fixture import CanonicalFixture as Fixture


class FixtureNativeDrain:
    @staticmethod
    def _drain_fixture_connections(_handles: list[object], _expiry: int) -> None:
        pass


class Borrower:
    _handle: object = object()

    def clear_program_cache(self) -> None:
        pass

    def close(self) -> None:
        pass


def fixture_type() -> type[Fixture]:
    # Exercise the Python ownership adapter without importing the linked native
    # extension or opening any database. Its borrower surface is exactly these
    # two synchronous logical operations; physical drain is separately observed.
    path = Path(__file__).resolve().parents[2] / "python/pse/tests/canonical_fixture.py"
    spec = importlib.util.spec_from_file_location("fixture_settlement_control", path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    fake_pse = types.ModuleType("pse")
    fake_pse.__dict__["Runtime"] = Borrower
    fake_build = types.ModuleType("pse._build")
    fake_build.__dict__["_NativeRuntime"] = FixtureNativeDrain
    with patch.dict(
        sys.modules, {"pse": fake_pse, "pse._build": fake_build, spec.name: module}
    ):
        spec.loader.exec_module(module)
    return cast("type[Fixture]", module.CanonicalFixture)


CanonicalFixture = fixture_type()


class FixtureSettlement(unittest.TestCase):
    def test_failed_cache_and_close_keep_siblings_and_receiver_independent(
        self,
    ) -> None:
        events: list[str] = []
        first = Mock(spec=Borrower)
        second = Mock(spec=Borrower)

        def fail_cache() -> None:
            events.append("first-cache")
            raise ValueError("cache refusal")

        def fail_close() -> None:
            events.append("first-close")
            raise RuntimeError("escaped reader")

        first.clear_program_cache.side_effect = fail_cache
        first.close.side_effect = fail_close
        second.clear_program_cache.side_effect = lambda: events.append("second-cache")
        second.close.side_effect = lambda: events.append("second-close")
        fixture = CanonicalFixture(
            "state",
            "database",
            "resource",
            [cast("pse.Runtime", first), cast("pse.Runtime", second)],
        )
        deadline = time.monotonic() + 1
        with (
            patch.object(
                surreal_server,
                "drain_context",
                side_effect=lambda *_args, **_kwargs: events.append("receiver"),
            ) as physical,
            patch.object(test_resources, "record_drain") as recorded,
            self.assertRaises(BaseExceptionGroup) as raised,
        ):
            fixture.settle(registered_context=True, deadline=deadline)
        self.assertEqual(
            events,
            ["first-cache", "second-cache", "first-close", "second-close", "receiver"],
        )
        physical.assert_called_once_with(Path("state"), "database", deadline=deadline)
        recorded.assert_not_called()
        self.assertEqual(fixture.borrowers, [first])
        nested = raised.exception.exceptions[0]
        assert isinstance(nested, BaseExceptionGroup)
        self.assertIn("cache refusal", str(nested.exceptions[0]))
        self.assertIn("escaped reader", str(nested.exceptions[1]))

    def test_physical_failure_preserves_original_clock_on_repeated_cleanup(
        self,
    ) -> None:
        fixture = CanonicalFixture("state", "database", "resource")
        deadline = time.monotonic() + 1
        with (
            patch.object(
                surreal_server,
                "drain_context",
                side_effect=KeyboardInterrupt("cancelled"),
            ) as physical,
            patch.object(test_resources, "record_drain") as recorded,
        ):
            for offered in (deadline, deadline + 90):
                with self.assertRaises(BaseExceptionGroup):
                    fixture.settle(registered_context=True, deadline=offered)
        self.assertEqual(physical.call_count, 2)
        for call in physical.call_args_list:
            self.assertEqual(call.kwargs["deadline"], deadline)
        recorded.assert_not_called()

    def test_two_failed_owners_retain_each_error_and_never_publish_drain(self) -> None:
        fixture = CanonicalFixture("state", "database", "resource")
        with (
            patch.object(
                type(fixture), "drain", side_effect=ValueError("local retained reader")
            ),
            patch.object(
                surreal_server,
                "drain_context",
                side_effect=RuntimeError("receiver unknown"),
            ),
            patch.object(test_resources, "record_drain") as recorded,
            self.assertRaises(BaseExceptionGroup) as raised,
        ):
            fixture.settle(registered_context=True)
        self.assertEqual(
            [str(error) for error in raised.exception.exceptions],
            ["local retained reader", "receiver unknown"],
        )
        recorded.assert_not_called()

    def test_complete_join_publishes_only_physical_ack_under_same_clock(self) -> None:
        fixture = CanonicalFixture("state", "database", "resource")
        deadline = time.monotonic() + 1
        units = [{"unit": "receiver", "group": "group", "invocation": "nonce"}]
        with (
            patch.object(surreal_server, "drain_context", return_value=units),
            patch.object(test_resources, "record_drain") as recorded,
        ):
            fixture.settle(registered_context=True, deadline=deadline)
        recorded.assert_called_once_with("resource", units=units, deadline=deadline)

    def test_sdk_stall_does_not_block_receiver_and_same_expiry_reaches_both(
        self,
    ) -> None:
        fixture = CanonicalFixture(
            "state", "database", "resource", [cast("pse.Runtime", Mock(spec=Borrower))]
        )
        receiver_finished = threading.Event()
        deadline = time.monotonic() + 1

        def receiver(*_args: object, **_kwargs: object) -> list[dict[str, str]]:
            receiver_finished.set()
            return []

        def sdk(_handles: list[object], expiry: int) -> None:
            self.assertLessEqual(expiry, int(deadline * 1_000_000_000))
            self.assertTrue(receiver_finished.wait(timeout=0.5))
            raise RuntimeError("retained SDK reader")

        with (
            patch.object(
                surreal_server, "drain_context", side_effect=receiver
            ) as native,
            patch.object(
                FixtureNativeDrain, "_drain_fixture_connections", side_effect=sdk
            ),
            patch.object(test_resources, "record_drain") as recorded,
            self.assertRaises(BaseExceptionGroup),
        ):
            fixture.settle(registered_context=True, deadline=deadline)
        native.assert_called_once_with(Path("state"), "database", deadline=deadline)
        self.assertEqual(len(fixture.borrowers), 1)
        recorded.assert_not_called()

    def test_unresponsive_receiver_cannot_renew_outer_join(self) -> None:
        fixture = CanonicalFixture("state", "database", "resource")
        release = threading.Event()
        finished = threading.Event()

        def receiver(*_args: object, **_kwargs: object) -> list[dict[str, str]]:
            try:
                release.wait(timeout=1)
                return []
            finally:
                finished.set()

        started = time.monotonic()
        with (
            patch.object(surreal_server, "drain_context", side_effect=receiver),
            patch.object(test_resources, "record_drain") as recorded,
            self.assertRaises(BaseExceptionGroup) as raised,
        ):
            fixture.settle(registered_context=True, deadline=started + 0.02)
        self.assertLess(time.monotonic() - started, 0.5)
        self.assertIsInstance(raised.exception.exceptions[0], TimeoutError)
        recorded.assert_not_called()
        release.set()
        self.assertTrue(finished.wait(timeout=1))

    def test_settlement_closes_new_runtime_admission_before_snapshot(self) -> None:
        fixture = CanonicalFixture("state", "database", "resource")
        fixture.settlement_deadline()
        with self.assertRaisesRegex(RuntimeError, "closed runtime admission"):
            fixture.runtime(cast("pse.EngineSettings", object()))

    def test_expired_outer_clock_never_signals_receiver(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            (state / ".contexts").mkdir()
            (state / ".contexts/database.json").write_text("{}")
            previous = getattr(surreal_server._STARTUP, "deadline", None)  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
            surreal_server._STARTUP.deadline = time.monotonic() - 1  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
            try:
                with (
                    patch.object(
                        surreal_server, "receiver_context", return_value=state
                    ),
                    patch.object(
                        surreal_server,
                        "config_for",
                        return_value={"database": "database"},
                    ),
                    patch.object(surreal_server.signal, "pidfd_send_signal") as signal,
                    self.assertRaises(surreal_server.DeadlineExpiredError),
                ):
                    surreal_server.drain_context(
                        state, "database", deadline=time.monotonic() + 90
                    )
                signal.assert_not_called()
            finally:
                surreal_server._STARTUP.deadline = previous  # noqa: SLF001 -- exercise the original owner failure/consistency boundary


if __name__ == "__main__":
    unittest.main()
