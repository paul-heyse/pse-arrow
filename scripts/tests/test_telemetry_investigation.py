# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent selected query and reset-provenance contracts."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import asyncio
import copy
import json
import unittest
from typing import TYPE_CHECKING
from unittest.mock import AsyncMock, patch
from urllib.parse import parse_qs, urlsplit

import httpx2
from pydantic import ValidationError

from scripts import telemetry_service
from scripts.telemetry_investigation import (
    BackendClient,
    QueryAnswer,
    QueryRequest,
    MetricQueryRefusal,
    backend_request,
)

if TYPE_CHECKING:
    from collections.abc import AsyncIterator


class QueryTests(unittest.IsolatedAsyncioTestCase):
    def test_metric_dimensions_are_instrument_specific_and_preserve_grouping(
        self,
    ) -> None:
        selected = self.request(
            metric="pse_assessment_duration",
            kind="p95",
            metric_filters={"mode": "native-force-validate", "stage": "execution"},
            group_by=["status"],
        )
        path, body = backend_request("default", "pse_assessment_duration", selected)
        expression = parse_qs(urlsplit(path).query)["query"][0]
        self.assertEqual(
            expression,
            'histogram_quantile(0.95,sum by(le,status)(rate(pse_assessment_duration_bucket{mode="native-force-validate",stage="execution"}[120s])))',
        )
        self.assertIsNone(body)
        for override in [
            {"metric_filters": {"status": "arbitrary-process-id"}},
            {"metric_filters": {"record": "compiler-artifact"}},
            {"group_by": ["status", "status"]},
            {"group_by": ["private"]},
            {"kind": "points", "group_by": ["status"]},
        ]:
            with self.assertRaises(ValidationError):
                self.request(metric="pse_assessment_duration", **override)

    def test_metric_points_filter_before_backend_expansion(self) -> None:
        selected = self.request(
            metric="pse_build_duration", metric_filters={"status": "failed"}
        )
        _, body = backend_request("default", "pse_build_duration", selected)
        assert body is not None
        query = body["query"]
        assert isinstance(query, dict)
        self.assertEqual(
            query["sql"],
            "SELECT * FROM \"pse_build_duration\" WHERE status = 'failed' ORDER BY _timestamp",
        )

    def test_native_attribute_uses_original_domain_and_owned_backend_name(self) -> None:
        with patch(
            "scripts.telemetry_investigation.metric_declarations",
            return_value=[
                {
                    "backend_name": "pse_runtime_executor_workers",
                    "attribute_values": {
                        "executor.role": ["worker", "python", "canonical_tools"]
                    },
                }
            ],
        ):
            selected = self.request(
                metric="pse_runtime_executor_workers",
                metric_filters={"executor.role": "worker"},
            )
            _, body = backend_request(
                "default", "pse_runtime_executor_workers", selected
            )
            assert body is not None
            query = body["query"]
            assert isinstance(query, dict)
            self.assertIn("executor_role = 'worker'", query["sql"])
            with self.assertRaises(ValidationError):
                self.request(
                    metric="pse_runtime_executor_workers",
                    metric_filters={"executor.role": "arbitrary-job-id"},
                )

    def test_grouped_fraction_preserves_numerator_denominator_population(self) -> None:
        selected = self.request(
            metric="pse_assessment_duration",
            kind="fraction_below",
            threshold=0.01,
            metric_filters={"stage": "execution"},
            group_by=["mode"],
        )
        path, _ = backend_request("default", "pse_assessment_duration", selected)
        expression = parse_qs(urlsplit(path).query)["query"][0]
        self.assertEqual(
            expression,
            'sum by(mode)(rate(pse_assessment_duration_bucket{le="0.01",stage="execution"}[120s]))/sum by(mode)(rate(pse_assessment_duration_count{stage="execution"}[120s]))',
        )

    async def asyncSetUp(self) -> None:
        self.client = BackendClient()
        self.addAsyncCleanup(self.client.close)
        self.inspected = {
            "configuration": {
                "configuration_id": "one",
                "organization": "default",
                "streams": {"traces": "pse_traces", "logs": "pse_logs"},
            },
            "services": {
                "openobserve": {
                    "ownership": "verified",
                    "process": {"pid": 1, "incarnation": "one"},
                }
            },
            "versions": {"openobserve": "1.1.2"},
        }

    def request(self, **overrides: object) -> QueryRequest:
        return QueryRequest.model_validate(
            {
                "signal": "metrics",
                "metric": "native_work",
                "start_time_us": 1_000_000,
                "end_time_us": 121_000_000,
                **overrides,
            }
        )

    async def answer(
        self,
        response: object,
        *,
        selected: QueryRequest | None = None,
        after: dict | None = None,
    ) -> QueryAnswer:
        with (
            patch.object(
                telemetry_service,
                "inspect",
                side_effect=[self.inspected, after or self.inspected],
            ),
            patch.object(
                telemetry_service,
                "backend_access",
                return_value=("http://127.0.0.1:5080", {}),
            ),
            patch.object(
                self.client,
                "request",
                new=AsyncMock(return_value=(200, json.dumps(response).encode())),
            ),
        ):
            return await self.client.query(None, selected or self.request())

    async def test_overlapping_selections_share_pool_without_closing_sibling(
        self,
    ) -> None:
        ready = asyncio.Event()
        arrivals = 0
        clients: list[httpx2.AsyncClient] = []
        constructor = httpx2.AsyncClient

        class Payload(httpx2.AsyncByteStream):
            async def __aiter__(self) -> AsyncIterator[bytes]:
                yield b'{"hits": []}'

        async def handle(_request: httpx2.Request) -> httpx2.Response:
            nonlocal arrivals
            arrivals += 1
            if arrivals == 2:
                ready.set()
            await asyncio.wait_for(ready.wait(), timeout=2)
            self.assertTrue(all(not client.is_closed for client in clients))
            return httpx2.Response(200, stream=Payload())

        def factory(**_options: object) -> httpx2.AsyncClient:
            client = constructor(transport=httpx2.MockTransport(handle))
            clients.append(client)
            return client

        def inspect(state: str, **_options: object) -> dict:
            result = copy.deepcopy(self.inspected)
            result["configuration"]["configuration_id"] = state
            return result

        with (
            patch.object(telemetry_service, "inspect", side_effect=inspect),
            patch.object(
                telemetry_service,
                "backend_access",
                side_effect=lambda state, **_options: (f"http://{state}.test", {}),
            ),
            patch.object(httpx2, "AsyncClient", side_effect=factory),
        ):
            first, second = await asyncio.gather(
                self.client.query("one", self.request()),
                self.client.query("two", self.request()),
            )
        self.assertEqual((first.status, second.status), ("empty", "empty"))
        self.assertEqual(len(clients), 1)
        await self.client.close()
        self.assertTrue(clients[0].is_closed)

    async def test_original_points_keep_reset_and_unknown_source_fields(self) -> None:
        points = [
            {
                "_timestamp": 1_000_000,
                "start_time": 900_000,
                "value": 1,
                "owner_epoch": "a",
            },
            {
                "_timestamp": 2_000_000,
                "start_time": 900_000,
                "value": 4,
                "owner_epoch": "a",
            },
            {
                "_timestamp": 3_000_000,
                "start_time": 2_500_000,
                "value": 2,
                "owner_epoch": "b",
            },
        ]
        answer = await self.answer({"hits": points, "total": 3})
        self.assertEqual(answer.records, points)
        self.assertEqual(answer.meaning, "observed_points")
        self.assertFalse(answer.complete)
        self.assertIn("source epochs", " ".join(answer.limits))

    async def test_parent_and_link_meaning_not_inferred(self) -> None:
        selected = self.request(signal="traces", metric=None, trace_id="a" * 32)
        answer = await self.answer(
            {
                "hits": [
                    {
                        "trace_id": "a" * 32,
                        "span_id": "1" * 16,
                        "reference_parent_span_id": "2" * 16,
                        "links": [{"span_id": "3" * 16}],
                        "service_service_instance_id": "p1",
                    }
                ]
            },
            selected=selected,
        )
        self.assertEqual(answer.records[0]["parent_span_id"], "2" * 16)
        self.assertEqual(answer.records[0]["links"], [{"span_id": "3" * 16}])
        self.assertEqual(answer.records[0]["service_instance_id"], "p1")

    async def test_source_change_discards_body(self) -> None:
        changed = {**self.inspected, "configuration": {"configuration_id": "two"}}
        answer = await self.answer({"hits": [{"value": 999}]}, after=changed)
        self.assertEqual(answer.status, "stale")
        self.assertEqual(answer.records, [])

    async def test_estimates_and_examples_never_claim_exactness(self) -> None:
        for kind, meaning in [
            ("rate", "estimated_rate"),
            ("increase", "estimated_increase"),
            ("p95", "estimated_quantile"),
            ("exemplars", "related_examples"),
        ]:
            selected = self.request(kind=kind, metric="pse_build_duration" if kind == "p95" else "pse_build_records")
            response = {
                "status": "success",
                "data": [{"exemplars": [{"value": 4}]}]
                if kind == "exemplars"
                else {"result": [{"values": [[1, "2.1"]]}]},
            }
            answer = await self.answer(response, selected=selected)
            self.assertEqual(answer.meaning, meaning)
            self.assertFalse(answer.complete)

    async def test_fraction_requires_an_emitted_boundary_and_retains_units(
        self,
    ) -> None:
        unsupported = await self.answer(
            {"status": "success", "data": {"result": []}},
            selected=self.request(
                metric="pse_build_duration", kind="fraction_below", threshold=0.0101
            ),
        )
        self.assertEqual(unsupported.status, "unsupported")
        selected = self.request(
            metric="pse_build_duration", kind="fraction_below", threshold=0.01
        )
        observed = await self.answer(
            {"status": "success", "data": {"result": [{"values": [[1, "0.6667"]]}]}},
            selected=selected,
        )
        self.assertEqual(observed.meaning, "estimated_fraction")
        self.assertEqual(observed.unit, "1")
        duration = await self.answer(
            {"hits": [{"value": 0.01}]},
            selected=self.request(metric="pse_build_duration_sum"),
        )
        self.assertIsNone(
            duration.unit,
            "Unknown backend derivative names must not acquire invented units",
        )

    async def test_nonfinite_backend_points_refuse_the_answer(self) -> None:
        answer = await self.answer({"hits": [{"value": float("nan")}]})
        self.assertEqual(answer.status, "unavailable")
        self.assertEqual(answer.records, [])

    async def test_missing_or_partial_data_is_not_a_zero_population(self) -> None:
        empty = await self.answer({"hits": []})
        partial = await self.answer({"hits": [{"value": 1}], "is_partial": True})
        invalid = await self.answer({"hits": "private invalid backend body"})
        self.assertEqual(
            (empty.status, partial.status, invalid.status),
            ("empty", "partial", "unavailable"),
        )
        self.assertNotIn("private", invalid.model_dump_json())

    def test_closed_inputs_reject_query_injection_and_wrong_units(self) -> None:
        for override in [
            {"metric": "x' OR true"},
            {"start_time_us": True},
            {"end_time_us": 1},
            {"limit": 0},
            {"kind": "sql"},
            {"signal": "traces", "metric": None},
            {"kind": "rate", "step_seconds": 1, "limit": 2},
            {"sql": "select *"},
        ]:
            with self.assertRaises(ValidationError):
                self.request(**override)
        path, body = backend_request(
            "default", "pse_build_records", self.request(metric="pse_build_records", kind="rate")
        )
        self.assertIn("start=1.0", path)
        self.assertIn("end=121.0", path)
        self.assertIsNone(body)

    async def test_instrument_refusals_precede_backend_or_service_access(self) -> None:
        cases = [
            ("pse_build_active", "rate", "metric_requires_monotonic_counter"),
            ("pse_build_active", "increase", "metric_requires_monotonic_counter"),
            ("pse_host_pressure", "rate", "metric_temporality_not_cumulative"),
            ("pse_build_records", "p95", "metric_requires_monotonic_histogram"),
            ("unknown_stream", "increase", "metric_instrument_not_declared"),
            ("pse_build_duration_sum", "p95", "metric_instrument_not_declared"),
            ("pse_owner_interval", "exemplars", "metric_exemplars_not_supported"),
        ]
        with (
            patch.object(telemetry_service, "inspect") as inspect,
            patch.object(telemetry_service, "backend_access") as access,
            patch.object(self.client, "request", new_callable=AsyncMock) as request,
        ):
            for metric, kind, reason in cases:
                with self.subTest(metric=metric, kind=kind):
                    answer = await self.client.query(None, self.request(metric=metric, kind=kind))
                    self.assertEqual((answer.status, answer.reason), ("unsupported", reason))
                    self.assertEqual(answer.records, [])
                    self.assertIsNone(answer.source)
                    self.assertFalse(answer.complete)
            inspect.assert_not_called()
            access.assert_not_called()
            request.assert_not_called()

    def test_estimators_require_complete_actual_instrument_semantics(self) -> None:
        base = {
            "backend_name": "fixture",
            "kind": "counter",
            "monotonic": True,
            "temporality": "cumulative",
            "unit": "{event}",
        }
        for changed, reason in [
            ({"monotonic": False}, "metric_requires_monotonic_counter"),
            ({"monotonic": None}, "metric_requires_monotonic_counter"),
            ({"temporality": "delta"}, "metric_temporality_not_cumulative"),
            ({"unit": None}, "metric_unit_not_declared"),
        ]:
            with self.subTest(changed=changed), patch(
                "scripts.telemetry_investigation.metric_declarations", return_value=[{**base, **changed}]
            ), self.assertRaises(MetricQueryRefusal) as caught:
                backend_request("default", "fixture", self.request(metric="fixture", kind="rate"))
            self.assertEqual(caught.exception.reason, reason)
        for buckets in [[], [1.0, 1.0], [2.0, 1.0], [float("inf")], [True]]:
            with self.subTest(buckets=buckets), patch(
                "scripts.telemetry_investigation.metric_declarations",
                return_value=[{**base, "kind": "histogram", "buckets": buckets}],
            ), self.assertRaises(MetricQueryRefusal) as caught:
                backend_request("default", "fixture", self.request(metric="fixture", kind="p95"))
            self.assertEqual(caught.exception.reason, "metric_histogram_buckets_invalid")

    def test_exemplar_support_comes_from_owner_capability_without_name_inference(self) -> None:
        for stream, support in [
            ("pse_runtime_fixture", True),
            ("host_fixture", False),
            ("unknown_sdk_fixture", None),
        ]:
            declaration = {"backend_name": stream, "unit": "{event}", "exemplars": support}
            with self.subTest(stream=stream), patch(
                "scripts.telemetry_investigation.metric_declarations", return_value=[declaration]
            ):
                selected = self.request(metric=stream, kind="exemplars")
                if support is True:
                    path, body = backend_request("default", stream, selected)
                    self.assertIn("/query_exemplars?", path)
                    self.assertIsNone(body)
                else:
                    with self.assertRaises(MetricQueryRefusal) as caught:
                        backend_request("default", stream, selected)
                    self.assertEqual(caught.exception.reason, "metric_exemplars_not_supported")

    async def test_unknown_absent_and_observed_zero_remain_distinct(self) -> None:
        zero = await self.answer({"hits": [{"value": 0}]})
        absent = await self.answer({"hits": []})
        self.assertEqual((zero.status, zero.records, zero.unit), ("available", [{"value": 0}], None))
        self.assertEqual((absent.status, absent.records, absent.unit), ("empty", [], None))

    def test_trace_points_do_not_require_unemitted_optional_attribute_columns(
        self,
    ) -> None:
        selected = self.request(
            signal="traces", metric=None, service_instance_id="ordinary-python-sdk"
        )
        path, body = backend_request("default", "traces", selected)
        self.assertIn("type=traces", path)
        assert body is not None
        selected_query = body["query"]
        assert isinstance(selected_query, dict)
        query = selected_query["sql"]
        self.assertEqual(
            query,
            "SELECT * FROM \"traces\" WHERE service_service_instance_id = 'ordinary-python-sdk' ORDER BY _timestamp,span_id",
        )
        self.assertNotIn("pse_operation_id", query)


if __name__ == "__main__":
    unittest.main()
