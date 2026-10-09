# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Nested requests must fit both allocation and actual role ceilings."""

from __future__ import annotations

import unittest
from types import SimpleNamespace
from unittest.mock import patch

import pytest

from scripts import host_admission as host
from scripts import pse_env


class NestedCapacityTests(unittest.TestCase):
    def test_wider_nested_request_cannot_borrow_a_smaller_owner(self) -> None:
        owner = SimpleNamespace(profile=host.select("functional"))
        with (
            patch.object(host, "inherit", return_value=owner),
            pytest.raises(pse_env.BoundaryError, match="widened allocation"),
        ):
            pse_env.placement({"PSE_MEMORY_MAX": "80G"}, native=False)

    def test_actual_observer_cap_cannot_be_hidden_by_wider_allocation(self) -> None:
        owner = SimpleNamespace(profile=host.select("exclusive"))
        with (
            patch.object(host, "inherit", return_value=owner),
            patch.object(pse_env, "current_memory_ceiling", return_value=64 * host.GIB),
            pytest.raises(pse_env.BoundaryError, match="widened role"),
        ):
            pse_env.placement({"PSE_MEMORY_MAX": "80G"}, native=True)

    def test_matching_nested_request_reuses_owner_without_new_admission(self) -> None:
        owner = SimpleNamespace(profile=host.select("functional"))
        with (
            patch.object(host, "inherit", return_value=owner),
            patch.object(pse_env, "current_memory_ceiling", return_value=40 * host.GIB),
            patch.object(host, "acquire") as acquire,
        ):
            assert (
                pse_env.placement(
                    {"PSE_MEMORY_MAX": "40G", "PSE_RESOURCE_CLASS": "functional"},
                    native=True,
                )
                == []
            )
            acquire.assert_not_called()

    def test_native_scope_without_host_owner_cannot_bypass_admission(self) -> None:
        with (
            patch.object(host, "inherit", return_value=None),
            patch.object(
                pse_env.operation, "scope_owner", return_value={"group": "/unknown"}
            ),
            pytest.raises(pse_env.BoundaryError, match="no verified host"),
        ):
            pse_env.placement({}, native=True)

    def test_light_and_compile_nested_jobs_keep_the_timing_lane(self) -> None:
        owner = SimpleNamespace(profile=host.select("timing"))
        with (
            patch.object(host, "inherit", return_value=owner),
            patch.object(
                pse_env, "current_memory_ceiling", return_value=owner.profile.memory
            ),
            patch.object(host, "acquire") as acquire,
        ):
            for resource_class in ("light", "compile"):
                assert (
                    pse_env.placement(
                        {"PSE_RESOURCE_CLASS": resource_class, "PSE_MEMORY_MAX": "56G"},
                        native=False,
                    )
                    == []
                )
            acquire.assert_not_called()
