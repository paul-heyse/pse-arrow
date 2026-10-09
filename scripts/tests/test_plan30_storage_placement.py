# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Dedicated timing storage selection and independent resident accounting."""

from __future__ import annotations

import contextlib
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from scripts import host_admission as host
from scripts import surreal_server as server


class StoragePlacementTests(unittest.TestCase):
    def test_timing_setup_records_storage_lane_and_refuses_cross_lane_context(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            state = Path(scratch) / "state"
            args = server.parser().parse_args(
                [
                    "setup",
                    "--state",
                    str(state),
                    "--execution-profile",
                    "timing",
                    "--worker-executable",
                    sys.executable,
                ]
            )
            receiver = {"worker_executable": sys.executable}
            with (
                patch.object(
                    server, "install", return_value={"binary": sys.executable}
                ),
                patch.object(server, "primary_receiver", return_value=receiver),
                patch.object(
                    server,
                    "publish_generation",
                    return_value={"supervisor_script": sys.executable},
                ),
                patch.object(
                    server, "public_status", side_effect=lambda _state, config: config
                ),
            ):
                config = server.setup(args)
                assert config["service_class"] == "timing"
                assert config["resources"] == server.execution_resources("timing")
                assert server.setup(args)["service_class"] == "timing"
                args.execution_profile = "functional"
                with pytest.raises(server.SupervisorError, match="differs"):
                    server.setup(args)
            with pytest.raises(server.SupervisorError, match="dedicated timing"):
                server.register_context(state, "functional_case", "functional")
            config["service_class"] = "functional"
            server.write_json(state / "config.json", config)
            with pytest.raises(server.SupervisorError, match="dedicated timing"):
                server.register_context(state, "timing_case", "timing")

    def test_resident_storage_has_its_own_allocation_outside_exclusive_mode(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            state = Path(scratch)
            caller = MagicMock(spec=host.Allocation)
            caller.profile = host.select("timing")
            store = MagicMock(spec=host.Allocation)
            store.profile = host.select("store-timing")
            store.directory = state / "admission"
            store.nonce = "a" * 32
            ledger = {"owners": {store.nonce: {"units": {}}}}
            config: dict[str, object] = {
                "instance_id": "owned",
                "service_class": "timing",
                "resources": server.execution_resources("timing"),
                "resident": True,
            }
            with (
                patch.object(host, "inherit", return_value=caller),
                patch.object(host, "acquire", return_value=store) as acquire,
                patch.object(
                    host,
                    "allocation_metadata",
                    return_value=contextlib.nullcontext(ledger),
                ),
            ):
                selected = server.service_allocation(state, config, 123.0)
            assert selected is store
            assert acquire.call_args.args[0].name == "store-timing"
            assert acquire.call_args.kwargs["deadline"] == 123.0
            assert caller.profile.memory + store.profile.memory == 64 * host.GIB
            store.register.assert_called_once_with(server.unit_name(state))
            caller.register.assert_not_called()


if __name__ == "__main__":
    unittest.main()
