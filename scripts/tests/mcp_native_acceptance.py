# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit current MCP/native assessment; no production repair or ordinary services.

Run through scripts/pse-env --native. Each mode writes a fresh evidence directory.
Canonical hold publishes exact original reader arguments for a separate host journey.
"""

from __future__ import annotations

import argparse
import asyncio
import base64
import copy
import gc
import hashlib
import json
import os
import shutil
import socket
import stat
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from fastmcp import Client
from fastmcp.client.transports import StdioTransport

from scripts import surreal_server as server
from scripts.tests.surreal_fixture_check import (
    INTERPRETATION,
    owned_fixture,
    record_ownership,
)
from scripts.tests.surreal_fixture_check import (
    query as fixture_query,
)
from scripts.tests.telemetry_operational_acceptance import environment
from scripts.tests.telemetry_pipeline_acceptance import Fixture, mapping, records
from scripts.tests.test_capability_native import (
    list_value,
    object_value,
    preparation_request,
    source_request,
)

if TYPE_CHECKING:
    from collections.abc import Awaitable, Callable, Mapping

    from pydantic import JsonValue

ROOT = Path(__file__).resolve().parents[2]


def write(path: Path, value: object) -> None:
    def bytes_value(item: object) -> object:
        if isinstance(item, bytes):
            return {"bytes_hex": item.hex()}
        raise TypeError("unsupported evidence JSON value: " + type(item).__name__)

    path.write_text(
        json.dumps(value, indent=2, allow_nan=False, default=bytes_value) + "\n"
    )


def require(value: object, message: str) -> None:
    if not value:
        raise AssertionError(message)


def normalized(value: object) -> dict:
    result = json.loads(json.dumps(value, allow_nan=False))
    if not isinstance(result, dict):
        raise TypeError("Expected one normalized native document object")
    return result


def package_document(value: JsonValue) -> dict[str, JsonValue]:
    return object_value(list_value(object_value(value)["packages"])[0])


def document_bytes(value: JsonValue) -> bytes:
    items = list_value(value)
    if not all(
        isinstance(item, int) and not isinstance(item, bool) and 0 <= item <= 255
        for item in items
    ):
        raise TypeError("Expected an exact array of document octets")
    octets: list[int] = []
    for item in items:
        if not isinstance(item, int):
            raise TypeError("Expected document octet")
        octets.append(item)
    return bytes(octets)


def connection(environment: dict[str, str] | None = None) -> Client:
    return Client(
        StdioTransport(
            command=str(ROOT / ".venv/bin/python"),
            args=["-m", "scripts.capabilities"],
            cwd=str(ROOT),
            env=environment or {**os.environ, "OTEL_SDK_DISABLED": "true"},
            keep_alive=False,
        ),
        timeout=180,
    )


class Recorder:
    def __init__(self, output: Path) -> None:
        self.output = output
        self.calls: list[dict[str, object]] = []
        self.controls: list[dict[str, object]] = []

    async def invoke(
        self,
        client: Client,
        name: str,
        arguments: dict,
        *,
        expect_refusal: bool = False,
    ) -> dict:
        described = await client.call_tool(
            "capabilities_describe", {"capability": name, "include_schema": True}
        )
        description = described.structured_content
        assert isinstance(description, dict), "missing live operation description"
        selected = await client.call_tool(
            "capabilities_select", {"capabilities": [name]}
        )
        tools = await client.list_tools(cache_mode="refresh")
        tool = next(tool for tool in tools if tool.name == name)
        read = tool.annotations is not None and tool.annotations.read_only_hint is True
        result = await client.call_tool(
            "capabilities_invoke_read" if read else "capabilities_invoke_action",
            {"capability": name, "arguments": arguments},
            raise_on_error=False,
        )
        value = result.structured_content
        while isinstance(value, dict) and set(value) == {"result"}:
            value = value["result"]
        item = {
            "capability": name,
            "arguments": arguments,
            "expected_refusal": expect_refusal,
            "is_error": result.is_error,
            "interface": description.get("interface"),
            "selection": selected.structured_content,
            "structured_content": value,
            "content": [block.model_dump(mode="json") for block in result.content],
            "meta": result.meta,
        }
        self.calls.append(item)
        write(self.output / "calls.json", self.calls)
        if expect_refusal:
            return item
        require(not result.is_error, f"{name}: transport/tool refusal")
        assert isinstance(value, dict), f"{name}: object response absent"
        payload = value.get("payload", value)
        require(isinstance(payload, dict), f"{name}: payload absent")
        require(
            payload.get("outcome")
            not in {
                "native_refusal",
                "stale_interface",
                "malformed_invocation",
                "prerequisite_unavailable",
                "detail_refused",
                "incomplete_observation",
            },
            f"{name}: native refusal {payload}",
        )
        return payload

    async def case(self, name: str, operation: Callable[[], Awaitable[None]]) -> None:
        first = len(self.calls)
        try:
            await operation()
        except Exception as error:
            self.controls.append(
                {
                    "case": name,
                    "status": "failed",
                    "error_type": type(error).__name__,
                    "error": str(error),
                }
            )
        else:
            self.controls.append({"case": name, "status": "passed"})
        for call in self.calls[first:]:
            call["control"] = name
            call["oracles_completed"] = self.controls[-1]["status"] == "passed"
        write(self.output / "calls.json", self.calls)
        write(self.output / "controls.json", self.controls)


async def models(recorder: Recorder) -> None:
    supplied = source_request()
    common = {key: supplied[key] for key in ("physical", "documents")}
    manifest = document_bytes(
        package_document(common["documents"])["package.toml"]
    ).decode()
    body = "package p { test bounded fixture { dof 0; route steady; procedure solve; } { var x:Scalar; annotation start x(1); eq solution:x==2; } }"
    common["documents"] = {
        "version": 1,
        "packages": [
            {
                "package.toml": list(manifest.encode()),
                "models/selected.pse": list(body.encode()),
            }
        ],
    }
    async with connection() as client:

        async def admit() -> None:
            result = await recorder.invoke(client, "native_source_admit", supplied)
            require(
                result["outcome"] == "admitted" and not result["durable"],
                "ephemeral admission changed",
            )
            root = "bb" * 16
            preparation = copy.deepcopy(supplied)
            package_document(preparation["documents"])["models/fixed.pse"] = list(
                f'package local {{@id("{root}") def Root {{var x:Scalar;eq e:x*x==1;}}}}'.encode()
            )
            preparation["request"] = preparation_request(root)
            observed = await recorder.invoke(
                client, "native_preparation_discover", preparation
            )
            require(
                observed["outcome"] == "admitted",
                "preparation observation not admitted",
            )

        await recorder.case("ephemeral-source-and-preparation", admit)

        async def inspect() -> None:
            page = await recorder.invoke(
                client,
                "native_model_declarations",
                {
                    **common,
                    "request": {"document_version": 1, "offset": 0, "limit": 16},
                },
            )
            case = next(
                row["declaration_id"]
                for row in page["document"]["declarations"]
                if row["name"] == "bounded"
            )
            request = copy.deepcopy(
                json.loads(
                    (
                        ROOT
                        / "docs/generated/schema/modeling-inspection-request.input.schema.json"
                    ).read_text()
                )["examples"][0]
            )
            request.update(case=case, limit=1)
            request["settings"]["intent"] = "root"
            explained = await recorder.invoke(
                client,
                "capability_explain",
                {
                    "capability": "native_model_inspect",
                    "arguments": {**common, "request": request},
                },
            )
            require(
                explained["input_schema_valid"]
                and explained["admission"] == "not_attempted",
                "explain admitted work",
            )
            result = await recorder.invoke(
                client, "native_model_inspect", {**common, "request": request}
            )
            require(
                result["document"]["source"] == page["document"]["source"],
                "source lineage changed",
            )
            require(
                result["document"]["population"]["members"] >= 1
                and len(result["document"]["members"]) == 1,
                "known model member absent",
            )
            other = copy.deepcopy(common["documents"])
            package_document(other)["models/selected.pse"] = list(
                body.replace("x==2", "x==3").encode()
            )
            compared = await recorder.invoke(
                client,
                "native_model_compare",
                {
                    **common,
                    "other_documents": other,
                    "other_physical": common["physical"],
                    "request": {
                        "document_version": 1,
                        "source_only": False,
                        "offset": 0,
                        "limit": 16,
                        "logical": [],
                    },
                },
            )
            require(compared["document"]["total"] > 0, "known authored change absent")
            require(
                compared["document"]["comparison"]["from"]
                != compared["document"]["comparison"]["to"],
                "authored identities collapsed",
            )

        await recorder.case("declaration-inspection-comparison", inspect)


async def canonical_reads(recorder: Recorder, state: Path, hold_seconds: int) -> None:
    import pyarrow as pa  # noqa: PLC0415 -- explicit native acceptance

    import pse  # noqa: PLC0415 -- explicit native acceptance
    from pse import codec  # noqa: PLC0415
    from pse.contracts import documents  # noqa: PLC0415
    from pse.tests import study_fixtures  # noqa: PLC0415
    from pse.tests.test_studies import _package  # noqa: PLC0415

    database = str(server.config_for(state)["database"])
    spill = recorder.output / "spill"
    spill.mkdir()
    settings = pse.EngineSettings(
        memory_limit_bytes=8 << 30,
        threads=16,
        spill_dir=str(spill),
        max_spill_bytes=1 << 30,
        batch_size=8192,
    )
    runtime = pse.Runtime(settings, substrate=str(state), database=database)
    reader_arguments: dict[str, object] = {}
    try:
        source = 'package algebraic { def Root { var x:Scalar; annotation bounds x(0,2); annotation start x(1); let cost:Scalar=x; annotation objective cost(minimize); eq floor:x>=0.5; annotation report x("minimum"); } }'
        package, case = _package(runtime, source=source)
        supplied = source_request()
        physical = runtime.physical_from_documents(
            {
                name: document_bytes(raw)
                for name, raw in package_document(supplied["physical"]).items()
            }
        )
        project_package = runtime.modeling_from_documents(
            [
                {
                    name: document_bytes(raw)
                    for name, raw in package_document(supplied["documents"]).items()
                }
            ],
            physical,
        )
        runtime.project_create(
            "mcp-assessment-create", "mcp-assessment-project", "MCP assessment fixture"
        )
        revision = runtime.project_fork(
            "mcp-assessment-project",
            "mcp-assessment-main",
            "mcp-assessment-problem",
            "mcp-assessment-fork",
            project_package,
        )
        deployment = {"state": str(state), "database": database}
        environment = {
            **os.environ,
            "PSE_SURREAL_STATE": str(state),
            "OTEL_SDK_DISABLED": "true",
        }
        async with connection(environment) as client:

            async def project_reads() -> None:
                selected = await recorder.invoke(
                    client,
                    "native_project_select",
                    {
                        **deployment,
                        "physical": supplied["physical"],
                        "request": {
                            "version": 1,
                            "reference": {
                                "kind": "branch",
                                "key": "mcp-assessment-main",
                            },
                        },
                    },
                )
                require(
                    selected["document"]["revision"] == revision,
                    "selected revision differs from original fork",
                )
                metadata = await recorder.invoke(
                    client,
                    "native_project_metadata",
                    {
                        **deployment,
                        "request": {
                            "document_version": 1,
                            "kind": "projects",
                            "owner": "",
                            "cursor": None,
                        },
                    },
                )
                require(
                    any(
                        item["key"] == "mcp-assessment-project"
                        for item in metadata["document"]["projects"]
                    ),
                    "known project missing",
                )
                original = await recorder.invoke(
                    client,
                    "native_revision_source",
                    {
                        **deployment,
                        "physical": supplied["physical"],
                        "revision": revision,
                        "logical": ["models/fixed.pse"],
                    },
                )
                require(
                    original["canonical_revision"] == revision,
                    "revision source changed",
                )
                refused = await recorder.invoke(
                    client,
                    "native_project_select",
                    {
                        **deployment,
                        "physical": supplied["physical"],
                        "request": {
                            "version": 1,
                            "reference": {
                                "kind": "branch",
                                "key": "deliberately-absent",
                            },
                        },
                    },
                    expect_refusal=True,
                )
                require(
                    refused["is_error"]
                    or refused["structured_content"]["payload"]["outcome"]
                    == "native_refusal",
                    "absent branch did not refuse",
                )

            await recorder.case("project-selection-source-and-refusal", project_reads)

            async def knowledge_reads() -> None:
                from scripts import (  # noqa: PLC0415
                    evidence_store,
                    knowledge_store,
                    workspace_domain_contracts,
                    workspace_domains,
                )

                evidence_store.prepare(state, deadline=time.monotonic() + 30)
                tool = ROOT / "target/debug/pse-evidence"
                require(
                    evidence_store.run_native(
                        state, tool, ["init"], deadline=time.monotonic() + 30
                    )
                    == 0,
                    "native evidence initialization failed",
                )
                source = workspace_domains.query(
                    workspace_domain_contracts.parse_query(
                        {
                            "concept": "generated-output",
                            "identity": "python/pse/_native.pyi",
                            "limit": 32,
                        }
                    ),
                    root=ROOT,
                )
                require(source["complete"], "native stub source projection incomplete")
                expected = {
                    (
                        f"declaration:{kind}:_native",
                        "generator:xtask/src/codegen/python_stubs.rs#run",
                        "generated_artifact:python/pse/_native.pyi",
                    )
                    for kind in (
                        "pyo3_module",
                        "pyo3_metadata",
                        "pyo3_exception_supplement",
                    )
                }
                require(
                    {
                        (row["declaration_id"], row["generator_id"], row["artifact_id"])
                        for row in source["productions"]
                    }
                    == expected,
                    "known three producer triples changed",
                )
                ranges = {
                    row["source_id"]: (
                        row["start_line"],
                        min(row["end_line"], row["start_line"] + 239),
                    )
                    for row in source["section_ranges"]
                    if row["subject_id"] == "contract:21.5"
                }
                corpus = knowledge_store.corpus(
                    (),
                    root=ROOT,
                    concept="generated-output",
                    identity="python/pse/_native.pyi",
                    ranges=ranges,
                )
                write(recorder.output / "knowledge-corpus.json", corpus)
                imported = knowledge_store.native(
                    state,
                    tool,
                    [
                        "knowledge-import",
                        "--input",
                        str(recorder.output / "knowledge-corpus.json"),
                    ],
                    deadline=time.monotonic() + 30,
                )
                write(recorder.output / "knowledge-import.json", imported)
                query = {
                    "version": 1,
                    "mode": "productions",
                    "artifact_id": "generated_artifact:python/pse/_native.pyi",
                    "limit": 1,
                }
                direct = knowledge_store.indexed(
                    query, state=state, tool=tool, root=ROOT
                )
                page = await recorder.invoke(
                    client, "knowledge_query", {"request": query, "state": str(state)}
                )
                require(
                    page["document"] == direct["document"],
                    "indexed MCP page differs from original native query",
                )
                require(
                    len(page["document"]["productions"]) == 1
                    and page["document"]["next"] is not None,
                    "indexed one-row continuation missing",
                )
                explanation = await recorder.invoke(
                    client, "knowledge_explain", {"request": query, "state": str(state)}
                )
                require(
                    explanation["document"]["generation"]
                    == page["document"]["generation"],
                    "knowledge explain generation changed",
                )
                wrong = {
                    **query,
                    "artifact_id": "deliberately-absent",
                    "cursor": page["document"]["next"],
                }
                refused = await recorder.invoke(
                    client,
                    "knowledge_query",
                    {"request": wrong, "state": str(state)},
                    expect_refusal=True,
                )
                require(
                    refused["structured_content"]["payload"]["code"]
                    == "pse.knowledge.cursor_context",
                    "changed query did not fence cursor",
                )

            await recorder.case(
                "authentic-indexed-knowledge-and-explanation", knowledge_reads
            )

            async def result_reads() -> None:
                handle = package.prepare_solve(case, pse.SolveSettings()).start()
                solved = handle.wait()
                require(solved.usable, "original tiny LP result unusable")
                run, attempt = solved.canonical_run_key, solved.canonical_attempt_key
                assert run is not None, "original solve lacks canonical run identity"
                assert attempt is not None, (
                    "original solve lacks canonical attempt identity"
                )
                relation = "runtime.solve_variables"
                with runtime.results(run, attempt, relation, start=0, end=1) as stream:
                    expected = pa.table(stream)
                write(recorder.output / "expected-solve-row.json", expected.to_pylist())
                require(
                    expected.num_rows == 1,
                    "known one-variable LP row population changed",
                )
                require(
                    expected.column("value")[0].as_py() == 0.5,
                    "independent minimum x=0.5 changed",
                )
                request = {
                    **deployment,
                    "run": run,
                    "attempt": attempt,
                    "relation": relation,
                    "start": 0,
                    "end": 1,
                    "maximum_bytes": 65536,
                }
                observed = await recorder.invoke(client, "native_result_read", request)
                require(
                    observed["outcome"] == "observed", "original result not observed"
                )
                raw = base64.b64decode("".join(observed["chunks"]), validate=True)
                actual = pa.ipc.open_stream(raw).read_all()
                require(
                    actual.equals(expected, check_metadata=True),
                    "MCP IPC differs from original protected reader",
                )
                write(
                    recorder.output / "ipc.json",
                    {
                        "sha256": hashlib.sha256(raw).hexdigest(),
                        "bytes": len(raw),
                        "rows": actual.num_rows,
                        "original_rows": expected.to_pylist(),
                    },
                )
                study = runtime.record_study(
                    package.admit_study(
                        study_fixtures.request(
                            study_fixtures.point(case, pse.SolveSettings(), 0)
                        )
                    )
                )
                study_handle = runtime.start_study(study)
                runtime.work(maximum_actions=8)
                retained = study_handle.wait(
                    controls=pse.StudyWaitControls(timeout_seconds=15)
                )
                selected = {
                    "document_version": 2,
                    "run": retained.run,
                    "attempt": retained.attempt,
                    "relation": "runtime.study_outcomes",
                    "start": 0,
                    "end": 1,
                    "view": {
                        "kind": "values",
                        "fields": [
                            {
                                "field": "start",
                                "steps": [{"kind": "struct_field", "name": "kind"}],
                            }
                        ],
                        "maximum_rows": 1,
                    },
                }
                original = normalized(
                    codec.document_builtins(
                        runtime.inspect_result(
                            codec.decode_json(
                                json.dumps(selected), documents.ResultInspectionRequest
                            )
                        )
                    )
                )
                require(
                    original["kind"] == "values", "nested original values not observed"
                )
                require(
                    original["rows"][0]["source_row"] == 0
                    and original["rows"][0]["values"][0]
                    == {"kind": "text", "value": "fresh"},
                    "independent nested value differs",
                )
                arguments = {**deployment, "request": selected}
                nested = await recorder.invoke(
                    client, "native_result_inspect", arguments
                )
                require(
                    nested["document"] == original,
                    "MCP nested JSON differs from original native JSON",
                )
                cli = subprocess.run(
                    [
                        sys.executable,
                        "-m",
                        "scripts.capability_cli",
                        "invoke",
                        "native_result_inspect",
                        "--input-json",
                        json.dumps(arguments),
                    ],
                    cwd=ROOT,
                    env=environment,
                    capture_output=True,
                    text=True,
                    timeout=180,
                    check=False,
                )
                (recorder.output / "cli.stderr").write_text(cli.stderr)
                require(cli.returncode == 0, "nested CLI failed")
                cli_result = json.loads(cli.stdout)
                write(recorder.output / "cli-result.json", cli_result)
                require(
                    not cli_result["is_error"]
                    and cli_result["structured_content"]["payload"]["document"]
                    == original,
                    "CLI nested value differs",
                )
                reader_arguments.update(
                    capability="native_result_inspect",
                    arguments=arguments,
                    expected_cell={"kind": "text", "value": "fresh"},
                    result_read_arguments=request,
                )
                write(recorder.output / "original-nested.json", original)
                wrong = copy.deepcopy(arguments)
                wrong["request"]["view"]["fields"][0]["field"] = (
                    "deliberately_absent_field"
                )
                refused = await recorder.invoke(
                    client, "native_result_inspect", wrong, expect_refusal=True
                )
                require(
                    refused["structured_content"]["payload"]["document"]["kind"]
                    == "refused"
                    and refused["structured_content"]["payload"]["document"]["reason"]
                    == "unknown_field",
                    "missing nested path lacks original typed refusal",
                )

            await recorder.case("protected-ipc-and-nested-result", result_reads)
        if hold_seconds and reader_arguments:
            write(
                recorder.output / "host-ready.json",
                {
                    "state": str(state),
                    "database": database,
                    "controller_pid": os.getpid(),
                    "reader": reader_arguments,
                    "release": str(recorder.output / "release"),
                    "deadline_monotonic": time.monotonic() + hold_seconds,
                },
            )
            until = time.monotonic() + hold_seconds
            while not (recorder.output / "release").exists():
                if time.monotonic() >= until:
                    raise TimeoutError("owned canonical host handoff expired")
                time.sleep(0.1)
    finally:
        runtime.close()
        gc.collect()
        server.drain_context(state, database, deadline=time.monotonic() + 60)


def canonical(
    recorder: Recorder, hold_seconds: int, selected_initializer: Path
) -> None:
    fixtures = recorder.output / "fixtures"
    fixtures.mkdir()
    initializer = recorder.output / "canonical-initializer"
    shutil.copyfile(selected_initializer, initializer)
    initializer.chmod(0o700)
    check = subprocess.run(
        [str(initializer), "--help"], capture_output=True, timeout=15, check=False
    )
    require(
        check.returncode == 0 and b"canonical-init" in check.stdout,
        "selected current initializer lacks canonical-tools",
    )
    with initializer.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    write(
        recorder.output / "initializer.json",
        {
            "sha256": digest,
            "source": str(selected_initializer),
            "selected": str(initializer),
            "help": check.stdout.decode(),
        },
    )
    try:
        with owned_fixture(fixtures, recorder.output / "lifecycle") as directory:
            state = directory / "live"
            with socket.socket() as probe:
                probe.bind(("127.0.0.1", 0))
                port = probe.getsockname()[1]
            server.setup(
                server.parser().parse_args(
                    [
                        "setup",
                        "--state",
                        str(state),
                        "--port",
                        str(port),
                        "--no-resident",
                        "--interpretation",
                        INTERPRETATION,
                    ]
                )
            )
            try:
                server.initialize(
                    state, [str(initializer), "canonical-init", str(state)]
                )
                server.start(state, server.config_for(state))
                record_ownership(directory, server.unit_name(state))
                asyncio.run(canonical_reads(recorder, state, hold_seconds))
            finally:
                server.stop(state, server.config_for(state))
    finally:
        info = initializer.lstat()
        require(
            stat.S_ISREG(info.st_mode)
            and info.st_uid == os.getuid()
            and info.st_nlink == 1,
            "owned initializer identity changed",
        )
        with initializer.open("rb") as stream:
            require(
                hashlib.file_digest(stream, "sha256").hexdigest() == digest,
                "owned initializer bytes changed",
            )
        initializer.unlink()


def telemetry(
    recorder: Recorder,
    selected_initializer: Path,
    hold_seconds: int,
    *,
    finite_only: bool = False,
) -> None:
    from scripts.tests import plan52_f02_acceptance as subject  # noqa: PLC0415

    fixture = Fixture(recorder.output / "telemetry-state", 90)
    initializer = recorder.output / "canonical-initializer"
    shutil.copyfile(selected_initializer, initializer)
    initializer.chmod(0o700)
    check = subprocess.run(
        [str(initializer), "--help"], capture_output=True, timeout=15, check=False
    )
    require(
        check.returncode == 0 and b"canonical-init" in check.stdout,
        "selected initializer lacks canonical-tools",
    )
    write(
        recorder.output / "initializer.json",
        {
            "source": str(selected_initializer),
            "sha256": hashlib.sha256(initializer.read_bytes()).hexdigest(),
            "help": check.stdout.decode(),
        },
    )

    def preflight() -> None:
        help_result = subprocess.run(
            [str(initializer), "--help"], capture_output=True, timeout=15, check=False
        )
        require(
            help_result.returncode == 0 and b"canonical-init" in help_result.stdout,
            "fixed initializer lost canonical capability",
        )

    def initialize(state: Path) -> None:
        server.initialize(state, [str(initializer), "canonical-init", str(state)])

    passed = False
    original_trace_rows = subject.trace_rows

    async def observed_trace_rows(
        telemetry_state: Path, trace_id: str, started: int
    ) -> list[dict[str, object]]:
        rows = await original_trace_rows(telemetry_state, trace_id, started)
        write(recorder.output / "last-trace-observation.json", rows)
        return rows

    original_journey = subject.journey

    def finite_journey(
        telemetry_state: Path, state: Path, database: str, output: Path
    ) -> None:
        from uuid import uuid4  # noqa: PLC0415 -- original finite owner selection

        from scripts import surreal_telemetry  # noqa: PLC0415

        finite = output / "finite"
        finite.mkdir(mode=0o700)
        trace_id = uuid4().hex
        started = time.time_ns() // 1000
        selected = environment(telemetry_state)
        selected["PSE_SURREAL_STATE"] = str(state)
        receipt: dict[str, object] = {
            "status": "running",
            "scope": "finite_only; continuous prior failure retained separately",
            "baseline_failures": 0,
        }
        try:
            with patch.dict(os.environ, selected, clear=True):
                capture = surreal_telemetry.capture_operation(
                    state,
                    output / "capture",
                    [
                        sys.executable,
                        "-m",
                        "scripts.tests.plan52_f02_acceptance",
                        "--canonical",
                        "--telemetry-state",
                        str(telemetry_state),
                        "--server-state",
                        str(state),
                        "--database",
                        database,
                        "--output",
                        str(finite),
                        "--expected-trace",
                        trace_id,
                    ],
                )
            receipt["capture"] = capture
            require(capture.get("workload") == 0, "f02.finite_workload_failed")
            require(
                mapping(capture.get("restoration", {})).get("phase") == "restored",
                "f02.finite_restoration_missing",
            )
            require(
                not capture.get("capture_errors") and capture.get("omitted_spans") == 0,
                "f02.finite_capture_incomplete",
            )
            captured = [
                subject.compact(row)
                for row in records(capture["spans"])
                if row.get("trace_id") == trace_id
            ]
            instance = mapping(mapping(capture["activation"])["after"])[
                "telemetry_service_instance_id"
            ]
            require(isinstance(instance, str), "f02.finite_server_incarnation_missing")
            for row in captured:
                row["service_instance_id"] = instance
            produced = server.read_json(finite / "producer.json")
            require(
                produced.get("status") == "passed", "f02.finite_native_producer_failed"
            )
            receipt["finite"] = subject.observe(
                telemetry_state,
                trace_id,
                started,
                subject.records(produced["siblings"]),
                captured,
            )
            receipt["status"] = "passed"
        except BaseException as error:
            receipt.update(
                status="failed", failure=type(error).__name__, reason=str(error)
            )
            raise
        finally:
            write(output / "acceptance.json", receipt)

    try:
        fixture.configure()
        fixture.start()
        started_us = time.time_ns() // 1000
        with (
            patch.object(subject, "canonical_tools_preflight", preflight),
            patch.object(subject, "initialize_canonical", initialize),
            patch.object(subject, "trace_rows", observed_trace_rows),
            patch.object(
                subject, "journey", finite_journey if finite_only else original_journey
            ),
            patch.dict(os.environ, environment(fixture.state), clear=True),
        ):
            subject.run(fixture.state, recorder.output / "f02")
        passed = True
        recorder.controls.append(
            {
                "case": "real-finite-native-parentage"
                if finite_only
                else "real-continuous-finite-native-parentage",
                "status": "passed",
            }
        )
        if hold_seconds:
            original = json.loads((recorder.output / "f02/acceptance.json").read_text())
            trace = original["continuous"]["trace_id"]
            span = original["continuous"]["parent_chains"][0][0]["span_id"]
            write(
                recorder.output / "host-ready.json",
                {
                    "state": str(fixture.state),
                    "controller_pid": os.getpid(),
                    "capability": "telemetry_query",
                    "arguments": {
                        "state": str(fixture.state),
                        "request": {
                            "signal": "traces",
                            "trace_id": trace,
                            "span_id": span,
                            "start_time_us": started_us,
                            "end_time_us": time.time_ns() // 1000 + 1,
                            "limit": 1000,
                        },
                    },
                    "expected_trace_id": trace,
                    "expected_span_id": span,
                    "release": str(recorder.output / "release"),
                    "deadline_monotonic": time.monotonic() + hold_seconds,
                },
            )
            until = time.monotonic() + hold_seconds
            while not (recorder.output / "release").exists():
                if time.monotonic() >= until:
                    raise TimeoutError("owned telemetry host handoff expired")
                time.sleep(0.1)
    finally:
        cleanup = fixture.cleanup(remove_state=passed)
        write(recorder.output / "telemetry-cleanup.json", cleanup)
        initializer.unlink()


def telemetry_mcp(recorder: Recorder, *, prepare: bool = False) -> None:
    from unittest.mock import patch  # noqa: PLC0415 -- retained test-only observation

    from scripts.tests import telemetry_mcp_acceptance as subject  # noqa: PLC0415

    calls: list[dict[str, object]] = []
    original = subject.selected
    original_start = Fixture.start

    def start(fixture: Fixture) -> None:
        original_start(fixture)
        if prepare:
            environment = {
                key: value
                for key, value in os.environ.items()
                if not key.startswith(("OTEL_", "PSE_TELEMETRY_"))
            }
            environment.update(
                {
                    "PSE_TELEMETRY_STATE": str(fixture.state),
                    "PSE_DIAGNOSTICS_DIR": str(
                        recorder.output / "joined/native-checkpoints"
                    ),
                    "OTEL_EXPORTER_OTLP_TIMEOUT": "1",
                    "OTEL_BSP_SCHEDULE_DELAY": "100",
                    "OTEL_METRIC_EXPORT_INTERVAL": "250",
                    "OTEL_BSP_EXPORT_TIMEOUT": "2000",
                }
            )

            async def selected_preparation() -> None:
                async with connection(environment) as client:
                    result = await recorder.invoke(
                        client,
                        "python_extension_native_stage",
                        {
                            "request": {
                                "stage": "prepare",
                                "profile": "dev",
                                "validated": True,
                            }
                        },
                    )
                    require(
                        result["exit_code"] == 0,
                        "explicit selected telemetry environment preparation failed",
                    )
                    recorder.calls[-1]["control"] = (
                        "explicit-selected-telemetry-preparation"
                    )
                    recorder.calls[-1]["oracles_completed"] = True
                    recorder.controls.append(
                        {
                            "case": "explicit-selected-telemetry-preparation",
                            "status": "passed",
                        }
                    )

            asyncio.run(selected_preparation())

    async def observed(
        client: Client,
        name: str,
        arguments: Mapping[str, object],
        *,
        action: bool = False,
    ) -> dict[str, object]:
        result = await original(client, name, arguments, action=action)
        calls.append({"capability": name, "arguments": arguments, "payload": result})
        write(recorder.output / "observed-calls.json", calls)
        return result

    with (
        patch.object(subject, "selected", observed),
        patch.object(Fixture, "start", start),
    ):
        status = subject.run(recorder.output / "joined")
    require(
        status == 0,
        "original joined telemetry/MCP acceptance failed; inspect retained payloads",
    )
    recorder.controls.append(
        {"case": "joined-telemetry-mcp-original-assertions", "status": "passed"}
    )


def telemetry_queries(recorder: Recorder, hold_seconds: int) -> None:
    from scripts.tests import (  # noqa: PLC0415
        telemetry_operational_acceptance,
        telemetry_semantics_acceptance,
    )

    fixture = Fixture(recorder.output / "telemetry-state", 90)
    passed = False
    try:
        fixture.configure()
        fixture.start()
        operational = asyncio.run(
            telemetry_operational_acceptance.run(
                fixture.state, recorder.output / "operational"
            )
        )
        recorder.controls.append(
            {
                "case": "actual-sdk-metrics-sampling-resets-and-typed-mcp",
                "status": "passed" if operational == 0 else "failed",
            }
        )
        semantic = asyncio.run(
            telemetry_semantics_acceptance.run(
                fixture.state, recorder.output / "semantics"
            )
        )
        recorder.controls.append(
            {
                "case": "actual-estimators-and-exemplars",
                "status": "passed" if semantic == 0 else "failed",
            }
        )

        async def query_controls() -> None:
            environment = {**os.environ, "PSE_TELEMETRY_STATE": str(fixture.state)}
            async with connection(environment) as client:
                await recorder.invoke(client, "telemetry_schema", {})
                await recorder.invoke(
                    client, "telemetry_configuration", {"state": str(fixture.state)}
                )
                await recorder.invoke(
                    client, "telemetry_health", {"state": str(fixture.state)}
                )
                await recorder.invoke(
                    client,
                    "telemetry_watch_configure",
                    {"state": str(fixture.state), "enabled": True},
                )
                now = time.time_ns() // 1000
                await recorder.invoke(
                    client,
                    "telemetry_watch_read",
                    {
                        "state": str(fixture.state),
                        "start_time_us": now - 600_000_000,
                        "end_time_us": now,
                    },
                )
                await recorder.invoke(
                    client,
                    "telemetry_watch_configure",
                    {"state": str(fixture.state), "enabled": False},
                )

        asyncio.run(
            recorder.case("schema-health-and-explicit-watch-controls", query_controls)
        )
        passed = all(item["status"] == "passed" for item in recorder.controls)
        if hold_seconds and operational == 0:
            answer = json.loads(
                (recorder.output / "operational/acceptance.json").read_text()
            )["always_on"]["canonical_mcp"]["pse_build_records"]
            write(
                recorder.output / "host-ready.json",
                {
                    "state": str(fixture.state),
                    "controller_pid": os.getpid(),
                    "capability": "telemetry_query",
                    "arguments": {
                        "state": str(fixture.state),
                        "request": answer["request"],
                    },
                    "expected_record": {"record": "compiler-message", "value": 3},
                    "original_metric_answer": str(
                        recorder.output / "operational/acceptance.json"
                    ),
                    "release": str(recorder.output / "release"),
                    "deadline_monotonic": time.monotonic() + hold_seconds,
                },
            )
            until = time.monotonic() + hold_seconds
            while (
                not (recorder.output / "release").exists() and time.monotonic() < until
            ):
                time.sleep(0.1)
    finally:
        write(
            recorder.output / "telemetry-cleanup.json",
            fixture.cleanup(remove_state=passed),
        )


def invocation_fixture(output: Path) -> None:
    import pse  # noqa: PLC0415 -- explicit fixture producer after preparation
    from pse import codec  # noqa: PLC0415
    from pse.contracts.enums import (  # noqa: PLC0415
        NativeBackend,
        NativeSolveIntent,
        PresolvePolicyKind,
    )
    from pse.tests.test_local_runtime import physical_documents  # noqa: PLC0415
    from pse.tests.test_native_workflow import package_documents  # noqa: PLC0415

    spill = output / "invocation-spill"
    spill.mkdir()
    runtime = pse.Runtime(
        pse.EngineSettings(
            memory_limit_bytes=64 << 30,
            threads=2,
            spill_dir=str(spill),
            max_spill_bytes=1 << 30,
            batch_size=32,
        ),
        ephemeral=True,
    )
    try:
        physical = runtime.physical_from_documents(physical_documents())
        package = runtime.modeling_from_documents(
            [
                package_documents(
                    "package invocation { def Root { var x:Scalar; eq balance:x==3; annotation start x(0); } }"
                )
            ],
            physical,
        )
        root = next(
            row.declaration_id for row in package.declarations() if row.name == "Root"
        )
        result = (
            package.prepare_solve(
                root,
                pse.SolveSettings(
                    intent=NativeSolveIntent.ROOT,
                    backend=NativeBackend.KINSOL,
                    presolve=PresolvePolicyKind.OFF,
                ),
            )
            .start()
            .wait()
        )
        require(result.usable, "original invocation fixture solve unusable")
        (output / "invocation.json").write_bytes(
            codec.encode_json(result.invocation_evidence)
        )
        write(
            output / "invocation-expected.json",
            {
                "run_id": str(result.invocation_evidence.run_id),
                "operation": normalized(
                    codec.document_builtins(result.invocation_evidence.operation)
                ),
                "usable": result.usable,
            },
        )
    finally:
        runtime.close()


def investigation(
    recorder: Recorder,
    hold_seconds: int,
    resume_target: Path | None,
    selected_state: Path | None,
    readers_from: Path | None,
) -> None:
    fixture = (
        Fixture(recorder.output / "telemetry-state", 90)
        if resume_target is None
        else None
    )
    passed = False
    try:
        if fixture is not None:
            fixture.configure()
            fixture.start()
            state = fixture.state
            native_output = recorder.output / "native-operational"
        else:
            if selected_state is None or resume_target is None:
                raise ValueError(
                    "Offline investigation resume requires exact original state and target"
                )
            state = selected_state
            native_output = resume_target.parent
        selected = environment(state, "always_on")
        selected.update(
            PSE_DIAGNOSTICS_DIR=str(native_output / "native-checkpoints"),
            PSE_DIAGNOSTICS="on",
            OTEL_LOGS_EXPORTER="none",
        )

        async def prepare() -> None:
            async with connection(selected) as client:

                async def action() -> None:
                    result = await recorder.invoke(
                        client,
                        "python_extension_native_stage",
                        {
                            "request": {
                                "stage": "prepare",
                                "profile": "dev",
                                "validated": True,
                            }
                        },
                    )
                    require(
                        result["exit_code"] == 0,
                        "exact investigation environment preparation failed",
                    )

                await recorder.case(
                    "explicit-investigation-environment-preparation", action
                )

        if fixture is not None:
            asyncio.run(prepare())
            require(
                recorder.controls[-1]["status"] == "passed",
                "investigation prerequisites failed",
            )
            command = [
                sys.executable,
                "-m",
                "scripts.tests.telemetry_native_operational_acceptance",
                "--state",
                str(state),
                "--output",
                str(native_output),
                "--sampler",
                "always_on",
            ]
            with (recorder.output / "native-operational.log").open("wb") as log:
                result = subprocess.run(
                    command,
                    cwd=ROOT,
                    env=selected,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    timeout=240,
                    check=False,
                )
            require(
                result.returncode == 0,
                "original native metric operational acceptance failed",
            )
            recorder.controls.append(
                {
                    "case": "two-native-solves-and-original-source-versus-backend",
                    "status": "passed",
                }
            )
        else:
            recorder.controls.append(
                {
                    "case": "retained-target-offline-read-no-native-operational-replay",
                    "status": "passed",
                    "source": str(resume_target),
                }
            )
        if hold_seconds and fixture is not None:
            observed = json.loads((native_output / "acceptance.json").read_text())[
                "observed"
            ]
            answer = observed["backend"]["pse_native_supervisors"]
            write(
                recorder.output / "host-ready.json",
                {
                    "state": str(state),
                    "controller_pid": os.getpid(),
                    "capability": "telemetry_query",
                    "arguments": {"state": str(state), "request": answer["request"]},
                    "expected_record": {"value": 0},
                    "original_metric_answer": str(native_output / "acceptance.json"),
                    "release": str(recorder.output / "release"),
                    "deadline_monotonic": time.monotonic() + hold_seconds,
                },
            )
        target = native_output / "producer-ready.json"
        if readers_from is None:
            command = [
                sys.executable,
                "-m",
                "scripts.tests.telemetry_investigation_acceptance",
                "--output",
                str(recorder.output / "investigation"),
                "--native-target",
                str(target),
            ]
            with (recorder.output / "investigation.log").open("wb") as log:
                result = subprocess.run(
                    command,
                    cwd=ROOT,
                    env=selected,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    timeout=180,
                    check=False,
                )
            require(
                result.returncode == 0,
                "original copied investigation/Timeline acceptance failed",
            )
            recorder.controls.append(
                {
                    "case": "native-copied-custody-stdio-capture-and-refusals",
                    "status": "passed",
                }
            )
            invocation_output = recorder.output / "invocation-fixture"
            with (recorder.output / "invocation-producer.log").open("wb") as log:
                result = subprocess.run(
                    [
                        sys.executable,
                        "-m",
                        "scripts.tests.mcp_native_acceptance",
                        "invocation-producer",
                        "--output",
                        str(invocation_output),
                    ],
                    cwd=ROOT,
                    env=selected,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    timeout=90,
                    check=False,
                )
            require(
                result.returncode == 0, "original invocation fixture producer failed"
            )
        else:
            require(
                json.loads(
                    (readers_from / "investigation/acceptance.json").read_text()
                )["status"]
                == "passed",
                "original copied capture did not pass",
            )
            invocation_output = readers_from / "invocation-fixture"
            require(
                (invocation_output / "invocation.json").is_file(),
                "exact original invocation file missing",
            )
            recorder.controls.append(
                {
                    "case": "read-only-original-invocation-and-capture-no-replay",
                    "status": "passed",
                    "source": str(readers_from),
                }
            )

        async def readers() -> None:
            from pse import codec, diagnostics  # noqa: PLC0415
            from pse.contracts import documents  # noqa: PLC0415

            produced = json.loads(target.read_text())
            checkpoint = Path(produced["checkpoint"])
            original = diagnostics.read_checkpoint(checkpoint)
            require(
                normalized(codec.document_builtins(original.locator))
                == produced["expected"]["locator"],
                "retained original native target identity changed",
            )
            operation = produced["operation"]
            inspect_request = {
                "version": 2,
                "operation": operation,
                "window": {"secs": 60, "nanos": 0},
                "include_related": False,
                "related_limit": 0,
                "numerical_history_policy": None,
            }
            import pytest  # noqa: PLC0415 -- reuse original bounded fixture producer

            from pse.tests.test_capability_adapters import (  # noqa: PLC0415 -- native import only after preparation
                test_capability_offline_diagnostic_reader_preserves_original_native_outcome as compact_fixture,
            )

            compact_directory = recorder.output / "bounded-checkpoint-fixture"
            compact_directory.mkdir()
            with pytest.MonkeyPatch.context() as monkeypatch:
                await asyncio.to_thread(compact_fixture, compact_directory, monkeypatch)
            compact_path = compact_directory / "retained.json"
            compact = diagnostics.read_checkpoint(compact_path)
            compact_request = {
                **inspect_request,
                "operation": normalized(
                    codec.document_builtins(compact.operations[0].id)
                ),
            }
            frozen = documents.FrozenEvidenceSetOutput(
                version=3, checkpoints=(compact,)
            )
            frozen_path = recorder.output / "bounded-frozen-evidence.json"
            frozen_path.write_bytes(codec.encode_json(frozen))
            async with connection(selected) as client:

                async def diagnostic() -> None:
                    observed = await recorder.invoke(
                        client,
                        "native_diagnostic_checkpoint",
                        {"checkpoint": str(compact_path), "expected": None},
                    )
                    require(
                        observed["document"]["process"] == compact.process,
                        "bounded standalone checkpoint identity changed",
                    )
                    page = await recorder.invoke(
                        client,
                        "native_diagnostic_processes",
                        {
                            "directory": str(compact_path.parent),
                            "request": {
                                "document_version": 1,
                                "after": None,
                                "limit": 16,
                            },
                        },
                    )
                    require(
                        any(
                            row["checkpoint"] == str(compact_path)
                            for row in page["document"]["processes"]
                        ),
                        "bounded fixture checkpoint not discovered",
                    )
                    inspected = await recorder.invoke(
                        client,
                        "native_diagnostic_inspect",
                        {"checkpoint": str(compact_path), "request": compact_request},
                    )
                    require(
                        inspected["document"]["operation"]["id"]
                        == compact_request["operation"],
                        "exact diagnostic operation changed",
                    )
                    frozen_read = await recorder.invoke(
                        client,
                        "native_evidence_inspect",
                        {
                            "path": str(frozen_path),
                            "request": {
                                "version": 1,
                                "inspection": compact_request,
                                "window": {"kind": "recorded"},
                            },
                        },
                    )
                    require(
                        frozen_read["document"]["inspection"]["operation"]["id"]
                        == compact_request["operation"],
                        "frozen original operation changed",
                    )
                    invocation = await recorder.invoke(
                        client,
                        "native_invocation_read",
                        {"path": str(invocation_output / "invocation.json")},
                    )
                    expected = json.loads(
                        (invocation_output / "invocation.json").read_text()
                    )
                    require(
                        invocation["document"] == expected,
                        "original invocation JSON changed",
                    )

                await recorder.case(
                    "bounded-original-diagnostic-frozen-and-invocation-readers",
                    diagnostic,
                )

                async def compare() -> None:
                    original_inputs = (
                        ROOT
                        / "build/capabilities-52o/conditional-f01-authentic-inputs-20261024/comparison-inputs.json"
                    )
                    fixture_inputs = json.loads(original_inputs.read_text())[
                        "comparison"
                    ]
                    observed = await recorder.invoke(
                        client, "native_evidence_compare", fixture_inputs
                    )
                    doc = observed["document"]
                    require(
                        doc["baseline_denominator"]["selected_tests"] == 3
                        and doc["baseline_denominator"]["failures"] == 2
                        and doc["baseline_denominator"]["unexecuted"] == 1,
                        "supplied historical baseline denominator changed",
                    )
                    require(
                        doc["candidate_denominator"]["selected_tests"] == 4
                        and doc["candidate_denominator"]["successes"] == 2
                        and doc["candidate_denominator"]["failures"] == 1,
                        "supplied historical candidate denominator changed",
                    )
                    require(
                        doc["summary"]["persistent_failures"] == 1
                        and doc["summary"]["resolved_failures"] == 1
                        and doc["summary"]["unmatched_candidate"] == 1,
                        "supplied original comparison classification changed",
                    )

                await recorder.case(
                    "current-native-comparison-over-supplied-historical-fixture",
                    compare,
                )

        with patch.dict(os.environ, selected, clear=True):
            asyncio.run(readers())
        passed = all(item["status"] == "passed" for item in recorder.controls)
    finally:
        handoff = recorder.output / "host-ready.json"
        if hold_seconds and handoff.exists():
            until = json.loads(handoff.read_text())["deadline_monotonic"]
            while (
                not (recorder.output / "release").exists() and time.monotonic() < until
            ):
                time.sleep(0.1)
        if fixture is not None:
            write(
                recorder.output / "telemetry-cleanup.json",
                fixture.cleanup(remove_state=passed),
            )
        else:
            write(
                recorder.output / "telemetry-cleanup.json",
                {
                    "services": "not_started_by_offline_resume",
                    "state": "original_previously_drained_failure_evidence_retained",
                },
            )


async def held_reads(
    recorder: Recorder, state: Path, *, database_only: bool = False
) -> None:
    import pse  # noqa: PLC0415
    from pse.contracts import documents  # noqa: PLC0415
    from pse.tests.test_project_portability import export_request  # noqa: PLC0415

    database = str(server.config_for(state)["database"])
    deployment = {"state": str(state), "database": database}
    supplied = source_request()
    spill = recorder.output / "spill"
    spill.mkdir()
    runtime = pse.Runtime(
        pse.EngineSettings(
            memory_limit_bytes=8 << 30,
            threads=16,
            spill_dir=str(spill),
            max_spill_bytes=1 << 30,
            batch_size=8192,
        ),
        substrate=str(state),
        database=database,
    )
    try:
        physical = runtime.physical_from_documents(
            {
                name: document_bytes(raw)
                for name, raw in package_document(supplied["physical"]).items()
            }
        )
        selected = runtime.project_select(
            documents.ProjectSelectRequest(
                reference=documents.ProjectReferenceBranch(key="mcp-assessment-main")
            ),
            physical,
        )
        pack = recorder.output / "project-pack"
        runtime.project_export(pack, export_request(selected), physical)
        environment = {
            **os.environ,
            "PSE_SURREAL_STATE": str(state),
            "OTEL_SDK_DISABLED": "true",
        }
        async with connection(environment) as client:

            async def source_detail() -> None:
                inspected = await recorder.invoke(
                    client, "native_project_bundle_inspect", {"path": str(pack)}
                )
                require(
                    inspected["document"]["origin"]["revision"] == selected.revision,
                    "offline exported revision differs",
                )
                require(
                    inspected["digest"] == pse.ProjectBundle(pack).digest,
                    "offline original digest differs",
                )
                research = await recorder.invoke(
                    client,
                    "native_research_read",
                    {
                        **deployment,
                        "request": {
                            "document_version": 1,
                            "kind": "datasets",
                            "project": "mcp-assessment-project",
                            "after": "",
                            "limit": 16,
                        },
                    },
                )
                require(
                    research["outcome"] == "observed", "research metadata not observed"
                )
                root = "bb" * 16
                source = copy.deepcopy(supplied["documents"])
                package_document(source)["models/fixed.pse"] = list(
                    f'package local {{@id("{root}") def Root {{var x:Scalar;eq e:x*x==1;}}}}'.encode()
                )
                package = runtime.modeling_from_documents(
                    [
                        {
                            name: document_bytes(raw)
                            for name, raw in package_document(source).items()
                        }
                    ],
                    physical,
                )
                revision = runtime.project_fork(
                    "mcp-assessment-project",
                    "mcp-preparation-main",
                    "mcp-preparation-problem",
                    "mcp-preparation-fork",
                    package,
                )
                observed = await recorder.invoke(
                    client,
                    "native_revision_preparation_discover",
                    {
                        **deployment,
                        "physical": supplied["physical"],
                        "revision": revision,
                        "request": preparation_request(root),
                    },
                )
                require(
                    observed["outcome"] == "observed",
                    "retained preparation metadata absent",
                )

            if not database_only:
                await recorder.case(
                    "portable-pack-research-and-retained-preparation", source_detail
                )

            async def database_reads() -> None:
                selected_database = await recorder.invoke(client, "database_select", {})
                fence = selected_database["selection"]
                for name, arguments in (
                    (
                        "database_info",
                        {
                            "selection": fence,
                            "target": "db",
                            "namespace": "pse",
                            "database": database,
                        },
                    ),
                    (
                        "database_list",
                        {
                            "selection": fence,
                            "kind": "tables",
                            "namespace": "pse",
                            "database": database,
                        },
                    ),
                    (
                        "database_query",
                        {
                            "selection": fence,
                            "query": "RETURN $expected;",
                            "parameters": {"expected": 7},
                            "namespace": "pse",
                            "database": database,
                        },
                    ),
                ):
                    answer = await recorder.invoke(client, name, arguments)
                    require(
                        not answer["native"]["isError"],
                        "selected native toolkit refused valid request",
                    )
                wrong = await recorder.invoke(
                    client,
                    "database_info",
                    {"selection": "0" * 64, "target": "db"},
                    expect_refusal=True,
                )
                require(wrong["is_error"], "wrong selected generation did not refuse")
                await recorder.invoke(
                    client,
                    "database_query",
                    {
                        "selection": fence,
                        "query": "CREATE mcp_assessment_forbidden SET value=1;",
                        "namespace": "pse",
                        "database": database,
                    },
                    expect_refusal=True,
                )
                require(
                    fixture_query(state, "SELECT * FROM mcp_assessment_forbidden;")[0][
                        "result"
                    ]
                    == [],
                    "VIEWER request changed owned rows",
                )

            await recorder.case("viewer-database-toolkit-and-authority", database_reads)

            async def result_controls() -> None:
                original = json.loads(
                    (
                        recorder.output.parent / "canonical-explicit/host-ready.json"
                    ).read_text()
                )
                arguments = original["reader"]["arguments"]
                bad = copy.deepcopy(arguments)
                bad["request"]["view"]["fields"][0]["field"] = (
                    "deliberately_absent_field"
                )
                refused = await recorder.invoke(
                    client, "native_result_inspect", bad, expect_refusal=True
                )
                require(
                    refused["structured_content"]["payload"]["document"]["kind"]
                    == "refused"
                    and refused["structured_content"]["payload"]["document"]["reason"]
                    == "unknown_field",
                    "original typed unknown_field refusal missing",
                )
                observed = await recorder.invoke(
                    client, "native_result_inspect", arguments
                )
                require(
                    observed["document"]["rows"][0]["values"][0]
                    == {"kind": "text", "value": "fresh"},
                    "fresh repeated read changed original cell",
                )

            await recorder.case(
                "nested-original-refusal-and-repeat-read", result_controls
            )
    finally:
        runtime.close()
        gc.collect()


async def toolkit(recorder: Recorder, state: Path) -> None:
    database = str(server.config_for(state)["database"])
    environment = {
        **os.environ,
        "PSE_SURREAL_STATE": str(state),
        "OTEL_SDK_DISABLED": "true",
    }
    async with connection(environment) as client:
        selected = await recorder.invoke(client, "database_select", {})
        fence = selected["selection"]
        info = await recorder.invoke(
            client,
            "database_info",
            {
                "selection": fence,
                "target": "canonical_projects",
                "namespace": "pse",
                "database": database,
            },
        )
        require(
            not info["native"]["isError"]
            and not info["native"]["result"].get("truncated"),
            "narrow original table info unavailable",
        )
        listed = await recorder.invoke(
            client,
            "database_list",
            {
                "selection": fence,
                "kind": "fields",
                "table": "canonical_projects",
                "namespace": "pse",
                "database": database,
            },
        )
        require(not listed["native"]["isError"], "original table fields unavailable")
        answer = await recorder.invoke(
            client,
            "database_query",
            {
                "selection": fence,
                "query": "RETURN $expected;",
                "parameters": {"expected": 7},
                "namespace": "pse",
                "database": database,
            },
        )
        require(
            answer["native"]["result"]["value"] == 7,
            "parameterized query value changed",
        )
        wrong = await recorder.invoke(
            client,
            "database_info",
            {"selection": "0" * 64, "target": "db"},
            expect_refusal=True,
        )
        require(wrong["is_error"], "wrong selection fence did not refuse")
        before = fixture_query(state, "SELECT * FROM mcp_assessment_forbidden;")[0][
            "result"
        ]
        denied = await recorder.invoke(
            client,
            "database_query",
            {
                "selection": fence,
                "query": "CREATE mcp_assessment_forbidden SET value=1;",
                "namespace": "pse",
                "database": database,
            },
            expect_refusal=True,
        )
        after = fixture_query(state, "SELECT * FROM mcp_assessment_forbidden;")[0][
            "result"
        ]
        require(before == after == [], "VIEWER query changed fixture rows")
        write(
            recorder.output / "viewer-no-effect.json",
            {
                "before_rows": before,
                "after_rows": after,
                "native_response": denied["structured_content"],
                "interpretation": "native VIEWER no-op is empty original value; lack of mutation is independently checked through the original privileged fixture reader",
            },
        )
        recorder.controls.append(
            {
                "case": "narrow-database-read-fence-and-viewer-no-effect",
                "status": "passed",
            }
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=(
            "model",
            "canonical",
            "telemetry",
            "telemetry-mcp",
            "held",
            "database",
            "toolkit",
            "telemetry-query",
            "investigation",
            "invocation-producer",
        ),
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--hold-seconds", type=int, default=0)
    parser.add_argument("--initializer", type=Path)
    parser.add_argument("--state", type=Path)
    parser.add_argument("--resume-target", type=Path)
    parser.add_argument("--resume-evidence", type=Path)
    parser.add_argument("--prepare", action="store_true")
    parser.add_argument("--finite-only", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    recorder = Recorder(output)
    receipt: dict[str, object] = {
        "mode": args.mode,
        "baseline_failures": 0,
        "status": "running",
    }
    try:
        if args.mode == "model":
            asyncio.run(models(recorder))
        elif args.mode == "canonical":
            require(
                args.initializer is not None,
                "canonical mode requires an explicit current feature artifact",
            )
            canonical(recorder, args.hold_seconds, args.initializer.resolve())
        elif args.mode == "telemetry":
            require(
                args.initializer is not None,
                "telemetry mode requires an explicit current canonical initializer",
            )
            telemetry(
                recorder,
                args.initializer.resolve(),
                args.hold_seconds,
                finite_only=args.finite_only,
            )
        elif args.mode in {"held", "database"}:
            require(
                args.state is not None, "held mode requires an explicitly owned state"
            )
            asyncio.run(
                held_reads(
                    recorder,
                    args.state.resolve(),
                    database_only=args.mode == "database",
                )
            )
        elif args.mode == "toolkit":
            require(
                args.state is not None,
                "toolkit mode requires an explicitly owned state",
            )
            asyncio.run(toolkit(recorder, args.state.resolve()))
        elif args.mode == "telemetry-query":
            telemetry_queries(recorder, args.hold_seconds)
        elif args.mode == "investigation":
            investigation(
                recorder,
                args.hold_seconds,
                args.resume_target.resolve() if args.resume_target else None,
                args.state.resolve() if args.state else None,
                args.resume_evidence.resolve() if args.resume_evidence else None,
            )
        elif args.mode == "invocation-producer":
            invocation_fixture(output)
        else:
            telemetry_mcp(recorder, prepare=args.prepare)
        receipt["status"] = (
            "failed"
            if any(item["status"] == "failed" for item in recorder.controls)
            else "passed"
        )
    except Exception as error:
        receipt.update(
            status="failed", error_type=type(error).__name__, error=str(error)
        )
    receipt["controls"] = recorder.controls
    successful = set()
    for call in recorder.calls:
        value = call.get("structured_content")
        payload = value.get("payload", value) if isinstance(value, dict) else {}
        native = payload.get("native", {})
        document = payload.get("document", {})
        if (
            not call["is_error"]
            and not call["expected_refusal"]
            and payload.get("outcome") in {None, "observed", "admitted", "selected"}
            and not native.get("isError")
            and (not isinstance(document, dict) or document.get("kind") != "refused")
            and (
                call.get("oracles_completed") is True
                or ("control" not in call and receipt["status"] == "passed")
            )
        ):
            successful.add(str(call["capability"]))
    receipt["successful_capabilities"] = sorted(successful)
    receipt["refusal_capabilities"] = sorted(
        {str(call["capability"]) for call in recorder.calls if call["expected_refusal"]}
    )
    write(output / "receipt.json", receipt)
    print(
        json.dumps(
            {"status": receipt["status"], "receipt": str(output / "receipt.json")}
        )
    )
    return 0 if receipt["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
