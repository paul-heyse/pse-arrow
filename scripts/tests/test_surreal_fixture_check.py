# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure controls for the disposable canonical lifecycle fixture composition."""

from __future__ import annotations

import contextlib
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import MagicMock, patch

import pytest

if TYPE_CHECKING:
    from collections.abc import Generator

from scripts.tests import surreal_fixture_check as fixture


class SurrealFixtureRetirementTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.selected = {
            "instance_id": "fixture-generation",
            "server": {"version": "fixture"},
            "service_supervisor": {"identity": "fixture"},
        }
        for boundary in (
            patch.object(fixture.server, "all_contexts_drained"),
            patch.object(fixture.server, "active", return_value=False),
            patch.object(fixture.server, "config_for", return_value=self.selected),
            patch.object(
                fixture.host_admission,
                "root_path",
                return_value=self.root / "admission",
            ),
        ):
            boundary.start()
            self.addCleanup(boundary.stop)
        snapshot = patch.object(
            fixture.host_admission,
            "readonly_snapshot",
            return_value={"version": 1, "owners": {}},
        )
        self.snapshot = snapshot.start()
        self.addCleanup(snapshot.stop)
        drained = patch.object(fixture.host_admission, "drained", return_value=True)
        self.drained = drained.start()
        self.addCleanup(drained.stop)

    def fixture_files(self, directory: Path, *, attached: bool = False) -> None:
        if not attached:
            for state in (directory / "live", directory / "restored"):
                state.mkdir()
                fixture.server.write_json(state / "config.json", self.selected)
        fixture.server.write_json(
            directory / "fixture-ownership.json",
            {
                "scopes": []
                if attached
                else [{"unit": "fixture.service", "boot": "fixture-boot"}]
            },
        )

    def referenced_fixture_files(
        self, directory: Path, state_name: str, reference: str
    ) -> None:
        self.fixture_files(directory)
        state = str(directory / state_name)
        self.snapshot.return_value = {
            "version": 1,
            "owners": {
                "retained-owner": {
                    "released": True,
                    reference: state if reference == "service" else [state],
                }
            },
        }

    def test_retains_fixture_and_withholds_evidence_for_any_host_reference(
        self,
    ) -> None:
        for state_name in ("live", "restored"):
            for reference in ("service", "borrowed_services"):
                with self.subTest(state=state_name, reference=reference):
                    evidence = self.root / f"evidence-{state_name}-{reference}"
                    with (
                        pytest.raises(
                            fixture.server.SupervisorError, match="host allocation"
                        ),
                        fixture.owned_fixture(self.root, evidence) as directory,
                    ):
                        self.referenced_fixture_files(directory, state_name, reference)
                    assert directory.is_dir()
                    assert not evidence.exists()
                    outcome = fixture.server.read_json(
                        directory / "fixture-outcome.json"
                    )
                    assert outcome["outcome"] == "failed"
                    assert outcome["retained"] is True
                    state = str(directory / state_name)
                    assert self.snapshot.return_value["owners"] == {
                        "retained-owner": {
                            "released": True,
                            reference: state if reference == "service" else [state],
                        }
                    }

    def test_unrelated_host_references_allow_positive_retirement(self) -> None:
        self.snapshot.return_value = {
            "version": 1,
            "owners": {
                "other": {
                    "service": "/other/service",
                    "borrowed_services": ["/another/service"],
                }
            },
        }
        evidence = self.root / "evidence"
        with fixture.owned_fixture(self.root, evidence) as directory:
            self.fixture_files(directory)
        assert not directory.exists()
        proof = fixture.server.read_json(evidence / "passed.json")
        assert proof["fixture_retired"] is True
        assert proof["all_contexts_and_kernel_scopes_drained"] is True
        self.snapshot.assert_called_once()

    def test_one_original_finish_deadline_bounds_drain_and_host_snapshot(self) -> None:
        directory = self.root / "fixture"
        directory.mkdir()
        self.fixture_files(directory)
        with patch.object(fixture.time, "monotonic", return_value=100.0):
            assert fixture.finish_fixture(directory, None) is None
        assert self.drained.call_args.kwargs["deadline"] == 190.0
        self.snapshot.assert_called_once_with(self.root / "admission", deadline=190.0)

    def test_unavailable_host_snapshot_retains_fixture_without_evidence(self) -> None:
        evidence = self.root / "evidence"
        self.snapshot.side_effect = fixture.host_admission.AdmissionError("unavailable")
        with (
            pytest.raises(fixture.host_admission.AdmissionError, match="unavailable"),
            fixture.owned_fixture(self.root, evidence) as directory,
        ):
            self.fixture_files(directory)
        assert directory.is_dir()
        assert not evidence.exists()

    def test_attached_mcp_retirement_does_not_require_external_service_release(
        self,
    ) -> None:
        external = str(self.root / "existing-service")
        self.snapshot.return_value = {
            "version": 1,
            "owners": {"external-owner": {"borrowed_services": [external]}},
        }
        evidence = self.root / "evidence"
        with fixture.owned_fixture(self.root, evidence) as directory:
            self.fixture_files(directory, attached=True)
            fixture.server.write_json(
                directory / "mcp-control.json",
                {
                    "attached_state": external,
                    "namespace_absent": True,
                    "clients_closed": True,
                    "service_generation": "fixture-generation",
                },
            )
        assert not directory.exists()
        assert (
            fixture.server.read_json(evidence / "passed.json")["fixture_retired"]
            is True
        )
        self.snapshot.assert_not_called()


class SurrealFixtureCheckTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.state = self.root / "live"
        self.state.mkdir()
        selected_root = patch.object(fixture, "ROOT", self.root)
        selected_root.start()
        self.addCleanup(selected_root.stop)

    def initializer(self) -> Path:
        binary = self.root / "target/debug/xtask"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"fixture-owned placeholder; never executed")
        binary.chmod(0o700)
        return binary

    def test_query_uses_selected_context_after_restore(self) -> None:
        config = {"port": 18249, "namespace": "restored_ns", "database": "current_db"}
        response = [{"status": "OK", "result": [fixture.FIXTURE]}]
        with (
            patch.object(fixture.server, "config_for", return_value=config),
            patch.object(
                fixture.server,
                "read_json",
                return_value={"username": "fixture", "password": "private"},
            ),
            patch.object(
                fixture.urllib.request,
                "urlopen",
                return_value=io.StringIO(json.dumps(response)),
            ) as opened,
        ):
            assert (
                fixture.query(self.state, "SELECT * FROM supervisor_probe:ack;")
                == response
            )
        request = opened.call_args.args[0]
        assert request.headers["Surreal-ns"] == "restored_ns"
        assert request.headers["Surreal-db"] == "current_db"

    def test_initializer_requires_built_executable_without_building(self) -> None:
        with patch.object(fixture.subprocess, "run") as run:
            with pytest.raises(fixture.server.SupervisorError, match="canonical-tools"):
                fixture.initialize_canonical(self.state)
            run.assert_not_called()
            binary = self.initializer()
            binary.chmod(0o600)
            with pytest.raises(fixture.server.SupervisorError, match="canonical-tools"):
                fixture.initialize_canonical(self.state)
            run.assert_not_called()

    def test_initializer_selects_owned_state_and_retains_private_output(self) -> None:
        binary = self.initializer()
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_SURREAL_STATE": "/other/state",
                    "PSE_CANONICAL_INITIALIZER": "stale",
                },
            ),
            patch.object(
                fixture.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, "initialized", ""),
            ) as run,
        ):
            fixture.initialize_canonical(self.state)
        assert run.call_args.args[0] == [str(binary), "canonical-init", str(self.state)]
        assert run.call_args.kwargs["cwd"] == self.root
        environment = run.call_args.kwargs["env"]
        assert environment["PSE_SURREAL_STATE"] == str(self.state)
        assert "PSE_CANONICAL_INITIALIZER" not in environment
        diagnostic = self.state / "canonical-initializer.log"
        assert diagnostic.read_text() == "initialized\n"
        assert diagnostic.stat().st_mode & 0o777 == 0o600

    def test_failed_initializer_retains_diagnostic_without_exposing_output(
        self,
    ) -> None:
        self.initializer()
        with (
            patch.object(
                fixture.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 2, "partial", "private detail"
                ),
            ),
            pytest.raises(
                fixture.server.SupervisorError, match="initializer failed"
            ) as error,
        ):
            fixture.initialize_canonical(self.state)
        assert "private detail" not in str(error.value)
        assert (
            self.state / "canonical-initializer.log"
        ).read_text() == "partial\nprivate detail"

    @contextlib.contextmanager
    def journey_controls(self) -> Generator[MagicMock, None, None]:
        """Substitute only orchestration boundaries; never launch a real service."""
        probe = MagicMock()
        probe.getsockname.return_value = ("127.0.0.1", 18249)
        with (
            patch.dict(os.environ, {"XDG_STATE_HOME": str(self.root)}),
            patch.object(
                fixture, "owned_fixture", return_value=contextlib.nullcontext(self.root)
            ),
            patch.object(fixture.socket, "socket") as socket,
            patch.object(fixture.server, "setup"),
            patch.object(fixture.server, "config_for", return_value={}),
            patch.object(fixture.server, "start"),
            patch.object(fixture, "record_ownership"),
            patch.object(fixture.server, "stop") as stop,
        ):
            socket.return_value.__enter__.return_value = probe
            yield stop

    def test_each_non_mcp_mode_initializes_before_toy_write_and_readback(self) -> None:
        modes = (
            "full",
            "receiver",
            "restart",
            "bounded",
        )
        for mode in modes:
            with self.subTest(mode=mode):
                events: list[str] = []

                def initialize(state: Path, *, events: list[str] = events) -> None:
                    assert state == self.state
                    events.append("initialize")

                def query(
                    state: Path, statement: str, *, events: list[str] = events
                ) -> list[dict[str, object]]:
                    assert state == self.state
                    assert "supervisor_probe:ack" in statement
                    assert "canonical_interpretations" not in statement
                    events.append("write")
                    return []

                def readback(state: Path, *, events: list[str] = events) -> None:
                    assert state == self.state
                    events.append("readback")
                    raise RuntimeError("stop after first readback")

                with (
                    self.journey_controls() as stop,
                    patch.object(
                        fixture, "initialize_canonical", side_effect=initialize
                    ),
                    patch.object(fixture, "query", side_effect=query),
                    patch.object(fixture, "check", side_effect=readback),
                    pytest.raises(RuntimeError, match="stop after first readback"),
                ):
                    fixture.journey(
                        receiver_closures_only=mode == "receiver",
                        restart_exhaustion_only=mode == "restart",
                        bounded_b_controls=mode == "bounded",
                    )
                assert events == ["initialize", "write", "readback"]
                stop.assert_called_once_with(self.state, {})

    def test_initializer_failure_stops_before_toy_write(self) -> None:
        with (
            self.journey_controls() as stop,
            patch.object(
                fixture,
                "initialize_canonical",
                side_effect=fixture.server.SupervisorError("initializer failed"),
            ),
            patch.object(fixture, "query") as query,
            patch.object(fixture, "check") as check,
            pytest.raises(fixture.server.SupervisorError, match="initializer failed"),
        ):
            fixture.journey()
        query.assert_not_called()
        check.assert_not_called()
        stop.assert_called_once_with(self.state, {})

    def test_mcp_modes_do_not_require_canonical_initializer(self) -> None:
        with (
            self.journey_controls(),
            patch.object(fixture, "initialize_canonical") as initialize,
            patch.object(fixture, "query") as query,
            patch.object(fixture, "check") as check,
            patch.object(fixture.server, "write_json"),
            patch(
                "scripts.tests.test_surreal_mcp.fixture_mcp_controls", return_value={}
            ),
        ):
            fixture.journey(mcp_only=True)
            fixture.journey(mcp_only=True, mcp_state=self.root / "existing-service")
        initialize.assert_not_called()
        query.assert_not_called()
        check.assert_not_called()
