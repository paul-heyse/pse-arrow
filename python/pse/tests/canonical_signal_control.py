# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manual entered-pull/SIGINT control for an exclusively owned canonical fixture.

The default pull control requires a two-file pse-canonical-recovery-fixture receipt.
The --analysis control creates its own scalar result in the selected private database.
Both require a force-validated installed extension. The selected
private fixture's results are collected after cancellation and transport drain;
this control never stops an unselected server instance.
"""

from __future__ import annotations

import argparse
import os
import signal
import sys
import threading
import time
from pathlib import Path
from typing import TYPE_CHECKING

import msgspec
from scripts import surreal_server as server

import pse
from pse._build import compiled_features
from pse.contracts.documents import AnalysisDirection, SensitivityRequest
from pse.contracts.enums import NativeSolveIntent
from pse.tests.physical_input_fixture import inspection_request
from pse.tests.test_canonical_results import (
    _assert_owned_delayed_canonical_sigint,
    _package,
    _queued_server_bytes,
    _settings,
)
from pse.tests.test_modeling_kernel import preparation_policy

if TYPE_CHECKING:
    from types import FrameType


class SignalResultFixture(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    version: int
    event: str
    state: str
    database: str
    run: str
    attempt: str
    relation: str
    terminal_class: str
    first_page_ordinals: tuple[int, int]
    file_instances: tuple[str, str]


def analysis_signal_control(
    settings: pse.EngineSettings, state: Path, database: str, instance: str
) -> dict[str, object]:
    """Interrupt a real analysis RPC, drain, then inspect the original result."""
    assert "force-validate" in compiled_features(), (
        "install the actual correctness extension"
    )
    config = server.config_for(state)
    process = server.read_json(state / "server-process.json")
    assert config["instance_id"] == process["instance_id"] == instance
    assert server._owns_listener(state, config)
    pid = server.integer(process["pid"])
    pidfd = os.pidfd_open(pid)
    runtime = pse.Runtime(settings, substrate=str(state), database=database)
    package, case = _package(runtime)
    preparation = preparation_policy(1_000_000)
    preparation = msgspec.structs.replace(
        preparation,
        compiler=msgspec.structs.replace(
            preparation.compiler,
            assembly=msgspec.structs.replace(
                preparation.compiler.assembly, worker_bytes=settings.math_worker_bytes
            ),
        ),
    )
    solve = msgspec.structs.replace(_settings(), intent=NativeSolveIntent.ROOT)
    parameter = next(
        member.id
        for member in package.inspect(inspection_request(case, solve)).members
        if member.lineage.path == "a" or member.lineage.path.endswith(".a")
    )
    solve = msgspec.structs.replace(
        solve, sensitivity=SensitivityRequest(parameters=(parameter,))
    )
    result = package.prepare_solve(case, solve, preparation=preparation).start().wait()
    assert result.usable, (result.completion, result.diagnostics())
    run, attempt = result.canonical_run_key, result.canonical_attempt_key
    assert run is not None
    assert attempt is not None
    # This control reads durable science. Release the completed solve's model
    # and cached programs before admitting a separate analysis working set.
    del result, package
    runtime.clear_program_cache()
    consumer = "canonical:scientific-selection-response"

    def reserved() -> int:
        return sum(
            item.reserved_bytes
            for item in runtime.resource_usage().top_consumers
            if item.name == consumer
        )

    before = reserved()
    ready = threading.Event()
    observed = threading.Event()
    returned = threading.Event()
    errors: list[BaseException] = []
    original = KeyboardInterrupt("entered result analysis")
    cause = RuntimeError("original analysis signal cause")
    original.__cause__ = cause

    def interrupted(_number: int, _frame: FrameType | None) -> None:
        observed.set()
        raise original

    previous = signal.signal(signal.SIGINT, interrupted)

    def controller() -> None:
        try:
            signal.pidfd_send_signal(pidfd, signal.SIGSTOP)
            deadline = time.monotonic() + 10
            while Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[
                0
            ] not in {"T", "t"}:
                assert time.monotonic() < deadline, "owned server did not stop"
                time.sleep(0.005)
            queued_before = _queued_server_bytes(pid)
            ready.set()
            while reserved() <= before or _queued_server_bytes(pid) <= queued_before:
                assert not returned.is_set(), (
                    "analysis returned before its selected read"
                )
                assert time.monotonic() < deadline, "analysis read did not enter"
                time.sleep(0.005)
            os.kill(os.getpid(), signal.SIGINT)
            assert observed.wait(3), "Python did not observe analysis interruption"
        except BaseException as error:
            errors.append(error)
        finally:
            ready.set()
            signal.pidfd_send_signal(pidfd, signal.SIGCONT)

    peer = threading.Thread(target=controller, name="analysis-signal-owner")
    peer.start()
    try:
        assert ready.wait(12)
        assert not errors, errors
        interruption: KeyboardInterrupt | None = None
        try:
            runtime.result_analysis(
                run,
                attempt,
                pse.AnalysisControls(roots=(), direction=AnalysisDirection.DOWNSTREAM),
            )
        except KeyboardInterrupt as error:
            interruption = error
        finally:
            returned.set()
        peer.join(timeout=15)
        assert not peer.is_alive()
        assert not errors, errors
        assert interruption is not None, "entered result analysis ignored SIGINT"
        assert interruption is original
        assert interruption.__cause__ is cause
        # A racing completion retains its acknowledged analysis identity.
        key = getattr(interruption, "analysis_key", None)
        if isinstance(key, str):
            assert runtime.analysis(key).key == key
        interruption.__traceback__ = None
        deadline = time.monotonic() + 10
        while reserved() > before:
            assert time.monotonic() < deadline, "analysis response did not drain"
            time.sleep(0.01)
        graph = runtime.result_analysis(
            run,
            attempt,
            pse.AnalysisControls(roots=(), direction=AnalysisDirection.DOWNSTREAM),
        )
        assert runtime.analysis(graph.key).key == graph.key
        return {
            "analysis_interrupted": True,
            "original_exception_and_cause_preserved": True,
            "response_drained": True,
            "following_analysis_key": graph.key,
        }
    finally:
        signal.pidfd_send_signal(pidfd, signal.SIGCONT)
        peer.join(timeout=15)
        signal.signal(signal.SIGINT, previous)
        os.close(pidfd)
        runtime.clear_program_cache()
        runtime.close()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", required=True, type=Path)
    parser.add_argument("--fixture-receipt", type=Path)
    parser.add_argument("--analysis", action="store_true")
    parser.add_argument("--database", default="pse")
    parser.add_argument("--exclusive-owned-instance", required=True)
    parser.add_argument("--spill-directory", required=True, type=Path)
    options = parser.parse_args()
    options.spill_directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    settings = pse.EngineSettings(
        memory_limit_bytes=512 << 20,
        # The analysis fixture compiles one scalar equation. Keep its compiler
        # and worker allowances inside this control's existing small pool.
        math_workspace_bytes=8 << 20,
        math_worker_bytes=8 << 20,
        math_foreign_bytes=1 << 20,
        math_inner_session_bytes=0,
        threads=2,
        top_consumers=4096,
        target_partitions=1,
        spill_dir=str(options.spill_directory.resolve()),
        max_spill_bytes=64 << 20,
        batch_size=2048,
        cache=pse.CacheSettings(
            working_bytes=64 << 20,
            concurrent_loads=2,
            inflight_bytes=16 << 20,
            inspection_bytes=4 << 20,
        ),
    )
    if options.analysis:
        outcome = analysis_signal_control(
            settings, options.state, options.database, options.exclusive_owned_instance
        )
        sys.stdout.write(msgspec.json.encode(outcome).decode() + "\n")
        return
    if options.fixture_receipt is None:
        parser.error("--fixture-receipt is required unless --analysis is selected")
    fixture = msgspec.json.decode(
        options.fixture_receipt.read_bytes(), type=SignalResultFixture
    )
    assert fixture.version == 1
    assert fixture.event == "signal-results-seeded"
    assert Path(fixture.state).resolve(strict=True) == options.state.resolve(
        strict=True
    )
    assert fixture.terminal_class == "partial"
    assert fixture.relation == "runtime.solve_metrics"
    assert fixture.first_page_ordinals == (0, 1)
    assert fixture.file_instances[0] != fixture.file_instances[1]
    outcome = _assert_owned_delayed_canonical_sigint(
        settings,
        state=options.state,
        database=fixture.database,
        run=fixture.run,
        attempt=fixture.attempt,
        relation=fixture.relation,
        file_instances=fixture.file_instances,
        owned_instance=options.exclusive_owned_instance,
    )
    sys.stdout.write(msgspec.json.encode(outcome).decode() + "\n")


if __name__ == "__main__":
    main()
