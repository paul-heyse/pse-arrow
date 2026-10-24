# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit per-test mutable database and local runtime borrower ownership."""

from __future__ import annotations

import os
import queue
import threading
import time
from pathlib import Path

import attrs
from scripts import surreal_server, test_resources

import pse
from pse._build import _NativeRuntime


@attrs.frozen
class CanonicalFixture:
    state: str
    database: str
    resource: str
    borrowers: list[pse.Runtime] = attrs.field(factory=list, repr=False)
    _settlement_clock: list[float] = attrs.field(factory=list, repr=False, init=False)

    def runtime(
        self,
        settings: pse.EngineSettings,
        *,
        producer: str | None = None,
        ephemeral: bool = False,
    ) -> pse.Runtime:
        if self._settlement_clock:
            message = "Canonical fixture settlement has closed runtime admission"
            raise RuntimeError(message)
        if not ephemeral:
            test_resources.require_effect(
                {"test": os.environ.get("PYTEST_CURRENT_TEST", "").rsplit(" (", 1)[0]},
                effect="native",
            )
        runtime = pse.Runtime(
            settings,
            substrate=self.state,
            database=self.database,
            producer=producer,
            ephemeral=ephemeral,
        )
        self.borrowers.append(runtime)
        return runtime

    def settlement_deadline(self, deadline: float | None = None) -> float:
        offered = time.monotonic() + 45 if deadline is None else deadline
        if self._settlement_clock:
            offered = min(offered, self._settlement_clock[0])
        self._settlement_clock[:] = [offered]
        return offered

    def drain(self) -> None:
        # Snapshot before cleanup. Optional caches can retain a sibling's source
        # protection, so attempt every eviction before releasing any caller.
        borrowers = tuple(self.borrowers)
        errors: list[BaseException] = []
        for index, runtime in enumerate(borrowers):
            try:
                runtime.clear_program_cache()
            except BaseException as error:
                error.add_note(f"fixture borrower {index}: program cache eviction")
                errors.append(error)
        retained = []
        for index, runtime in enumerate(borrowers):
            try:
                runtime.close()
            except BaseException as error:
                error.add_note(f"fixture borrower {index}: logical caller close")
                errors.append(error)
                retained.append(runtime)
        self.borrowers[:] = retained
        if errors:
            message = "Canonical fixture caller cleanup failed"
            raise BaseExceptionGroup(message, errors)

    def settle(
        self, *, registered_context: bool, deadline: float | None = None
    ) -> None:
        # Logical caller close and the native receiver are separate owners.
        # Continue physical settlement after a caller failure, but publish drain
        # only when every required owner acknowledged settlement.
        deadline = self.settlement_deadline(deadline)
        borrowers = tuple(self.borrowers)
        errors: list[BaseException] = []
        try:
            self.drain()
        except BaseException as error:
            errors.append(error)
        # Start native settlement before joining SDK clients. The native owner
        # receives the identical expiry; an SDK reader cannot consume its chance
        # to progress. A late/unknown native result never publishes registry drain.
        completion: queue.Queue[tuple[list[dict[str, str]], BaseException | None]] = (
            queue.Queue(maxsize=1)
        )

        def drain_receiver() -> None:
            try:
                units = surreal_server.drain_context(
                    Path(self.state), self.database, deadline=deadline
                )
            except BaseException as error:
                completion.put(([], error))
            else:
                completion.put((units, None))

        if registered_context:
            threading.Thread(
                target=drain_receiver, name="fixture-native-drain", daemon=True
            ).start()
        try:
            _NativeRuntime._drain_fixture_connections(
                [runtime._handle for runtime in borrowers],
                max(0, int(deadline * 1_000_000_000) - 1),
            )
        except BaseException as error:
            self.borrowers[:] = borrowers
            errors.append(error)
        units: list[dict[str, str]] = []
        if registered_context:
            while True:
                try:
                    units, failure = completion.get(
                        timeout=max(0, deadline - time.monotonic())
                    )
                except queue.Empty:
                    errors.append(
                        TimeoutError(
                            "Native fixture drain remains unknown at original expiry"
                        )
                    )
                    break
                except BaseException as error:
                    # Repeated cancellation retains the same offer. The native
                    # owner remains protected and bounded by its original clock.
                    errors.append(error)
                    continue
                if failure is not None:
                    errors.append(failure)
                break
        if errors:
            message = "Canonical fixture physical settlement incomplete"
            raise BaseExceptionGroup(message, errors)
        test_resources.record_drain(self.resource, units=units, deadline=deadline)
