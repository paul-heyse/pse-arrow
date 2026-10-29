# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure comparison faults; these tests never activate an SDK or native solver."""
# Exception cases use the stdlib unittest API consistently with this script suite.
# ruff: noqa: PT027

from __future__ import annotations

import unittest
from typing import TYPE_CHECKING

from scripts.report_contracts import OwnerRecord
from scripts.telemetry_investigation import QueryAnswer, QueryRequest, QuerySource
from scripts.tests.telemetry_native_operational_acceptance import compare_points

if TYPE_CHECKING:
    from scripts.report_contracts import JsonValue


class NativePopulationComparisonTests(unittest.TestCase):
    def answer(self, rows: list[dict[str, JsonValue]]) -> QueryAnswer:
        metric = "pse_native_supervisor_events"
        request = QueryRequest(
            signal="metrics",
            metric=metric,
            start_time_us=1,
            end_time_us=100,
            service_instance_id="fixture-instance",
        )
        source = QuerySource(
            configuration_id="fixture",
            backend_version="1.1.2",
            backend_process=OwnerRecord.model_validate({}),
            stream=metric,
            signal="metrics",
            start_time_us=1,
            end_time_us=100,
            retrieved_at_ns=100_000,
        )
        return QueryAnswer(
            request=request,
            status="available",
            source=source,
            records=rows,
            returned_rows=len(rows),
        )

    def point(
        self, timestamp: int, value: int, *, instance: str = "fixture-instance"
    ) -> dict[str, JsonValue]:
        return {
            "_timestamp": timestamp,
            "event": "attempt",
            "value": value,
            "service_instance_id": instance,
            "aggregation_temporality": "AGGREGATION_TEMPORALITY_CUMULATIVE",
            "start_time": "1000",
        }

    def test_repeated_cumulative_points_are_not_added(self) -> None:
        answer = self.answer([self.point(10, 2), self.point(20, 3), self.point(30, 3)])
        compare_points(answer, [({"event": "attempt"}, 3)], "fixture-instance", 15)

    def test_latest_population_mismatch_is_not_hidden_by_prior_match(self) -> None:
        answer = self.answer([self.point(10, 3), self.point(20, 4)])
        with self.assertRaisesRegex(RuntimeError, "native_backend_population_differs"):
            compare_points(answer, [({"event": "attempt"}, 3)], "fixture-instance", 1)

    def test_incarnations_are_never_combined(self) -> None:
        answer = self.answer(
            [self.point(10, 3), self.point(20, 3, instance="restarted-instance")]
        )
        with self.assertRaisesRegex(RuntimeError, "metric_process_changed"):
            compare_points(answer, [({"event": "attempt"}, 3)], "fixture-instance", 1)

    def test_missing_first_source_interval_is_not_zero_work(self) -> None:
        answer = self.answer([self.point(10, 0)])
        with self.assertRaisesRegex(RuntimeError, "metric_series_missing"):
            compare_points(answer, [({"event": "attempt"}, 0)], "fixture-instance", 20)


if __name__ == "__main__":
    unittest.main()
