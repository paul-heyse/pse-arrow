# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit finite copied-investigation and retained native-capture acceptance.

Requires the prepared native extension and an exact target handoff from the native
operational acceptance producer. Never builds, queries/stops ordinary services,
restarts science, or installs tools. Supplied observations are constructed fixture
records with independent source cuts, not claims about actual live backend work.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import shutil
import sys
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import AsyncMock, patch

from scripts import telemetry_selection, telemetry_service
from scripts.capabilities.read_contracts import OwnerRecord
from scripts.telemetry_investigation import (
    BackendClient,
    QueryAnswer,
    QueryRequest,
    QuerySource,
)
from scripts.telemetry_selection import SelectionPage, SelectionReference
from scripts.tests.telemetry_pipeline_acceptance import mapping, records, require

if TYPE_CHECKING:
    from collections.abc import Callable

    from fastmcp import Client
    from fastmcp.client.client import CallToolResult

ROOT = Path(__file__).resolve().parents[2]
MAXIMUM_BYTES = 8 << 20


def observations() -> list[QueryAnswer]:
    first = QueryRequest(
        signal="metrics",
        metric="copied_fixture_work",
        service_instance_id="recorded-process",
        start_time_us=10,
        end_time_us=20,
    )
    source = QuerySource(
        configuration_id="recorded-configuration",
        backend_version="1.1.2",
        backend_process=OwnerRecord.model_validate(
            {"pid": 99, "start": "recorded-incarnation"}
        ),
        stream="copied_fixture_work",
        signal="metrics",
        start_time_us=10,
        end_time_us=20,
        retrieved_at_ns=25_000,
    )
    return [
        QueryAnswer(
            request=first,
            status="partial",
            source=source,
            records=[
                {"_timestamp": 15, "value": 7, "fixture_row": index}
                for index in range(5)
            ],
            returned_rows=5,
            limits=["constructed fixture; source cut is not atomic"],
            reason="related_trace_expired",
        ),
        QueryAnswer(
            request=first.model_copy(update={"start_time_us": 30, "end_time_us": 40}),
            status="empty",
            source=source.model_copy(
                update={
                    "start_time_us": 30,
                    "end_time_us": 40,
                    "retrieved_at_ns": 45_000,
                }
            ),
            limits=["different copied source cut"],
        ),
    ]


def expect_refusal(action: Callable[[], object], code: str) -> str:
    try:
        action()
    except Exception as error:
        return type(error).__name__
    raise RuntimeError(code)


def offline(output: Path, answers: list[QueryAnswer]) -> dict[str, object]:
    from pse import (  # noqa: PLC0415 -- explicit qualification only; never build
        codec,
        diagnostics,
    )
    from pse.contracts import documents as d  # noqa: PLC0415

    reference = telemetry_selection.export(
        output / "copied",
        answers,
        question="What do the selected native and telemetry observations establish?",
        maximum_output_bytes=MAXIMUM_BYTES,
        missing_context=["related trace unavailable"],
    )
    index = json.loads(codec.encode_json(diagnostics.review_pack(reference.directory)))
    require(
        isinstance(index, dict) and index.get("publication") == "sealed",
        "investigation.native_pack_not_sealed",
    )
    with (
        patch.object(
            BackendClient,
            "query",
            new=AsyncMock(side_effect=AssertionError("offline backend query")),
        ),
        patch.object(
            telemetry_service,
            "inspect",
            side_effect=AssertionError("offline service inspection"),
        ),
        patch.object(
            telemetry_selection.time,
            "time_ns",
            side_effect=AssertionError("offline current clock"),
        ),
    ):
        first = telemetry_selection.page(reference, limit=2)
        second = telemetry_selection.page(reference, limit=2, cursor=first.next_cursor)
        third = telemetry_selection.page(reference, limit=2, cursor=second.next_cursor)
        empty = telemetry_selection.page(reference, selection=1, limit=2)
    require(
        [page.returned_rows for page in (first, second, third)] == [2, 2, 1],
        "investigation.stable_page_sizes_changed",
    )
    require(
        [row["fixture_row"] for page in (first, second, third) for row in page.records]
        == [0, 1, 2, 3, 4],
        "investigation.equal_key_rows_lost_or_repeated",
    )
    require(
        third.copied_selection_complete
        and third.next_cursor is None
        and not third.live_population_complete,
        "investigation.copied_completeness_changed",
    )
    require(
        empty.copied_rows == 0 and empty.source != first.source,
        "investigation.independent_source_cuts_merged",
    )
    bad_digest = reference.model_copy(update={"sha256": "0" * 64})
    digest = expect_refusal(
        lambda: telemetry_selection.page(bad_digest),
        "investigation.changed_digest_accepted",
    )
    wrong_cursor = expect_refusal(
        lambda: telemetry_selection.page(
            reference, selection=1, limit=2, cursor=first.next_cursor
        ),
        "investigation.cross_selection_cursor_accepted",
    )
    wrong_page = expect_refusal(
        lambda: telemetry_selection.page(reference, limit=3, cursor=first.next_cursor),
        "investigation.changed_page_request_accepted",
    )
    bound = expect_refusal(
        lambda: telemetry_selection.export(
            output / "too-small",
            answers,
            question="Which bounded observations are retained?",
            maximum_output_bytes=1,
        ),
        "investigation.output_bound_ignored",
    )
    require(
        not (output / "too-small").exists(),
        "investigation.preflight_bound_published_output",
    )
    (Path(reference.directory) / "unlisted.json").write_text(
        "unlisted bytes must not be a pack read target"
    )
    unlisted = expect_refusal(
        lambda: diagnostics.pack_member(
            reference.directory, "unlisted", maximum_bytes=MAXIMUM_BYTES
        ),
        "investigation.unlisted_member_read",
    )
    altered = output / "changed-member"
    shutil.copytree(reference.directory, altered)
    members = records(mapping(index)["members"])
    original_member = next(
        member
        for member in members
        if mapping(member["selection"])["id"] == "investigation"
    )
    member_path = altered / str(original_member["relative_path"])
    member_bytes = member_path.read_bytes()
    member_path.write_bytes(bytes([member_bytes[0] ^ 1]) + member_bytes[1:])
    changed = expect_refusal(
        lambda: telemetry_selection.page(
            reference.model_copy(update={"directory": str(altered)})
        ),
        "investigation.changed_member_accepted",
    )
    incomplete_path = output / "incomplete"
    missing = output / "absent-source.json"
    incomplete = expect_refusal(
        lambda: diagnostics.export_member_pack(
            incomplete_path,
            d.PackExportRequest(
                selection=d.PackSelection(
                    members=(
                        d.PackMemberSelection(
                            id="investigation",
                            role=d.PackMemberRole.QUERY,
                            required=True,
                            format=d.PackFormat.JSON,
                            document_version=1,
                            source="explicit missing fixture",
                        ),
                    ),
                    maximum_output_bytes=MAXIMUM_BYTES,
                    maximum_document_bytes=MAXIMUM_BYTES,
                ),
                files=(
                    d.PackFileInput(
                        member="investigation",
                        path=str(missing),
                        readable_prefix_bytes=1,
                    ),
                ),
            ),
        ),
        "investigation.missing_required_member_accepted",
    )
    incomplete_index = json.loads(
        codec.encode_json(diagnostics.review_pack(incomplete_path))
    )
    require(
        isinstance(incomplete_index, dict)
        and incomplete_index.get("publication") == "incomplete",
        "investigation.incomplete_publication_hidden",
    )
    unreadable = expect_refusal(
        lambda: diagnostics.pack_member(
            incomplete_path, "investigation", maximum_bytes=MAXIMUM_BYTES
        ),
        "investigation.incomplete_absent_member_invented",
    )
    return {
        "reference": reference.model_dump(mode="json"),
        "index": index,
        "pages": [page.model_dump(mode="json") for page in (first, second, third)],
        "empty_selection": empty.model_dump(mode="json"),
        "refusals": {
            "digest": digest,
            "cross_selection_cursor": wrong_cursor,
            "changed_page_request": wrong_page,
            "output_bound": bound,
            "unlisted": unlisted,
            "changed_member": changed,
            "missing_required_member": incomplete,
            "incomplete_absent_member": unreadable,
        },
        "incomplete_index": incomplete_index,
    }


def content(result: CallToolResult) -> dict[str, object]:
    value = mapping(result.structured_content)
    while set(value) == {"result"}:
        value = mapping(value["result"])
    return value


async def invoke(
    client: Client, name: str, arguments: dict[str, object], *, action: bool = False
) -> dict[str, object]:
    await client.call_tool(
        "capabilities_describe", {"capability": name, "include_schema": True}
    )
    await client.call_tool("capabilities_select", {"capabilities": [name]})
    return content(
        await client.call_tool(
            "capabilities_invoke_action" if action else "capabilities_invoke_read",
            {"capability": name, "arguments": arguments},
        )
    )


def selected_page(value: dict[str, object]) -> SelectionPage:
    return SelectionPage.model_validate(value["payload"])


async def stdio(
    output: Path,
    answers: list[QueryAnswer],
    target: dict[str, object],
    evidence: dict[str, object],
) -> dict[str, object]:
    from fastmcp import Client  # noqa: PLC0415
    from fastmcp.client.transports import StdioTransport  # noqa: PLC0415
    from mcp_types import TextContent, TextResourceContents  # noqa: PLC0415

    environment = {**os.environ, "OTEL_SDK_DISABLED": "true"}
    transport = StdioTransport(
        command=sys.executable,
        args=["-m", "scripts.capabilities"],
        cwd=str(ROOT),
        env=environment,
        keep_alive=False,
    )
    async with Client(transport, timeout=30) as client:
        exported = await invoke(
            client,
            "telemetry_investigation_export",
            {
                "destination": str(output / "stdio-copied"),
                "question": "Which original observations can a later connection read?",
                "selections": [answer.model_dump(mode="json") for answer in answers],
                "maximum_output_bytes": MAXIMUM_BYTES,
            },
            action=True,
        )
        reference = SelectionReference.model_validate(exported)
        arguments: dict[str, object] = {
            "reference": reference.model_dump(mode="json"),
            "selection": 0,
            "limit": 2,
        }
        fixed = selected_page(
            await invoke(client, "telemetry_selection_read", arguments)
        )
        direct = selected_page(
            content(await client.call_tool("telemetry_selection_read", arguments))
        )
        require(
            fixed == direct and [row["fixture_row"] for row in fixed.records] == [0, 1],
            "investigation.fixed_and_direct_reads_differ",
        )
        uri = f"pse://telemetry/selection/{telemetry_selection.resource_reference(reference)}/0/2"
        resource = (await client.read_resource(uri))[0]
        if not isinstance(resource, TextResourceContents):
            raise TypeError("investigation.resource_not_json_text")
        direct_resource = selected_page(mapping(json.loads(resource.text)))
        projection = await client.call_tool("read_resource", {"uri": uri})
        projected = projection.content[0]
        if not isinstance(projected, TextContent):
            raise TypeError("investigation.projection_not_json_text")
        projected_resource = selected_page(mapping(json.loads(projected.text)))
        require(
            direct_resource == fixed and projected_resource == fixed,
            "investigation.resource_projections_changed_source",
        )
        captures: dict[str, object] = {}
        evidence.update(
            {
                "reference": reference.model_dump(mode="json"),
                "direct": direct.model_dump(mode="json"),
                "fixed_invoker": fixed.model_dump(mode="json"),
                "resource": direct_resource.model_dump(mode="json"),
                "projected_resource": projected_resource.model_dump(mode="json"),
                "capture": captures,
            }
        )
        await capture_cases(client, output, target, answers[0], captures)
        return evidence


async def capture_cases(
    client: Client,
    output: Path,
    target: dict[str, object],
    suspect: QueryAnswer,
    evidence: dict[str, object],
) -> dict[str, object]:
    request = {
        "version": 1,
        "checkpoint": target["checkpoint"],
        "operation": target["operation"],
        "kind": "Timeline",
        "output": str(output / "timeline"),
        "duration": {"secs": 0, "nanos": 50_000_000},
        "budget": {"memory_bytes": 4 << 20, "output_bytes": MAXIMUM_BYTES},
    }
    supplied = {
        "request": request,
        "expected": target["expected"],
        "suspect": suspect.model_dump(mode="json"),
    }
    timeline = mapping(
        (await invoke(client, "telemetry_profile_capture", supplied, action=True))[
            "payload"
        ]
    )
    evidence["timeline"] = timeline
    require(
        timeline.get("outcome") == "observed", "investigation.timeline_capture_refused"
    )
    document = mapping(timeline["document"])
    require(
        document.get("kind") == "Timeline"
        and document.get("operation") == target["operation"],
        "investigation.timeline_target_changed",
    )
    require(
        mapping(timeline["telemetry_context"])["atomic_identity_admission"] is False,
        "investigation.capture_identity_overclaimed",
    )
    require(records(document["outputs"]), "investigation.timeline_output_absent")
    cpu_output = output / "finished-cpu-refused"
    cpu = mapping(
        (
            await invoke(
                client,
                "telemetry_profile_capture",
                {
                    **supplied,
                    "request": {**request, "kind": "Cpu", "output": str(cpu_output)},
                },
                action=True,
            )
        )["payload"]
    )
    evidence["finished_cpu_refusal"] = cpu
    require(
        cpu.get("outcome") != "observed" and not cpu_output.exists(),
        "investigation.completed_operation_cpu_accepted",
    )
    wrong = {
        **mapping(target["expected"]),
        "process": "deliberately-wrong-process-incarnation",
    }
    wrong_output = output / "wrong-identity-refused"
    identity = mapping(
        (
            await invoke(
                client,
                "telemetry_profile_capture",
                {
                    **supplied,
                    "expected": wrong,
                    "request": {**request, "output": str(wrong_output)},
                },
                action=True,
            )
        )["payload"]
    )
    evidence["wrong_identity_refusal"] = identity
    require(
        identity.get("outcome") != "observed" and not wrong_output.exists(),
        "investigation.wrong_process_identity_accepted",
    )
    return {
        "timeline": timeline,
        "finished_cpu_refusal": cpu,
        "wrong_identity_refusal": identity,
    }


async def run(output: Path, target_path: Path) -> int:
    output.mkdir(parents=True, exist_ok=False, mode=0o700)
    receipt: dict[str, object] = {
        "status": "running",
        "baseline_failures": 0,
        "scope": "finite supplied telemetry observations in real native custody and canonical STDIO capture",
        "maximum_output_bytes": MAXIMUM_BYTES,
    }
    try:
        answers = observations()
        target = mapping(json.loads(target_path.read_bytes()))
        receipt["native_target"] = {
            name: target[name] for name in ("checkpoint", "operation", "expected")
        }
        receipt["offline"] = offline(output, answers)
        canonical: dict[str, object] = {}
        receipt["canonical_stdio"] = canonical
        await stdio(output, answers, target, canonical)
        receipt["status"] = "passed"
    except Exception as error:
        receipt.update(
            status="failed",
            error=str(error)
            if isinstance(error, RuntimeError)
            else type(error).__name__,
        )
    path = output / "acceptance.json"
    path.write_text(json.dumps(receipt, indent=2, allow_nan=False) + "\n")
    path.chmod(0o600)
    print(json.dumps({"status": receipt["status"], "receipt": str(path)}))
    return 0 if receipt["status"] == "passed" else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--native-target", type=Path, required=True)
    args = parser.parse_args()
    return asyncio.run(run(args.output.absolute(), args.native_target.absolute()))


if __name__ == "__main__":
    raise SystemExit(main())
