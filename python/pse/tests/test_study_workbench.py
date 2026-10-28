# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public bounded workbench and passive observer composition on two declared points."""

import os
import secrets
import subprocess
import uuid
from pathlib import Path

import anyio
import msgspec
import pyarrow as pa
import pytest
from scripts import surreal_server

import pse
from pse import codec
from pse.contracts.documents import (
    ControlBrowseRequest,
    StudyCohortPageRequest,
    StudyCohortRequest,
    StudyKpiRequest,
    StudyObservationRequest,
    StudyWorkbenchFilter,
    StudyWorkbenchPageOutput,
    StudyWorkbenchRequest,
)
from pse.contracts.enums import NativeSolveIntent
from pse.contracts.values import SemanticId
from pse.control import ControlClient
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.study_fixtures import point
from pse.tests.study_fixtures import request as study_request
from pse.tests.test_studies import _package


@pytest.mark.component
def test_study_workbench_public_snapshot_and_passive_reconnect(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    # Constant declared science keeps the public journey bounded and independent
    # of nonlinear solver convergence or a production numerical campaign.
    package, case = _package(
        runtime,
        source="""package algebraic { def Root {
        param x:Scalar = 2; annotation report x("constant");
    } }""",
    )
    settings = pse.SolveSettings(intent=NativeSolveIntent.FEASIBLE_POINT)
    local = package.admit_study(
        study_request(
            point(case, settings, 7),
            point(case, settings, 11),
        )
    )
    handle = runtime.start_study(runtime.record_study(local))
    selected = StudyWorkbenchFilter(
        study=handle.study_id.to_hex(),
        project=None,
        branch=None,
        source=None,
        physical_revision=None,
        context=None,
        outcome=None,
        research_reference=None,
    )
    actor = "workbench-" + uuid.uuid4().hex
    password = secrets.token_urlsafe(32)
    actor_file = tmp_path / "control-actor.json"
    descriptor = os.open(actor_file, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(
            msgspec.json.encode(
                {
                    "document_version": 1,
                    "key": actor,
                    "password": password,
                    "browse": True,
                    "cancel": False,
                }
            )
        )
    root = Path(__file__).resolve().parents[3]
    issued = subprocess.run(
        [
            str(root / "target/debug/pse-canonical-init"),
            "--issue-control-actor",
            str(actor_file),
            "--database",
            canonical_substrate.database,
            canonical_substrate.state,
        ],
        capture_output=True,
        check=True,
        timeout=90,
    )
    report = msgspec.json.decode(issued.stdout, type=dict[str, str])
    config = surreal_server.config_for(Path(canonical_substrate.state))

    async def browse() -> StudyWorkbenchPageOutput:
        async with ControlClient(
            f"http://127.0.0.1:{config['port']}",
            str(config["namespace"]),
            canonical_substrate.database,
            None,
            issuing_store=report["issuing_store"],
            timeout=30,
            maximum_request_bytes=65536,
            maximum_response_bytes=2 * 1024 * 1024,
        ) as client:
            await client.sign_in(actor, password)
            return await client.study_workbench_page(
                ControlBrowseRequest(
                    document_version=1,
                    filter=selected,
                    maximum=1,
                    cursor=None,
                )
            )

    browsing = anyio.run(browse)
    assert len(browsing.headers) == 1
    assert browsing.headers[0].point_count == 2
    assert browsing.next is None
    frozen = runtime.freeze_study_cohort(
        StudyCohortRequest(
            filter=selected,
            maximum_studies=1,
            maximum_members=2,
        )
    )
    assert frozen.receipt.complete
    assert frozen.receipt.member_count == 2
    assert [
        row.value.ordinal
        for row in frozen.member_page(
            StudyCohortPageRequest(maximum=2, cursor=None)
        ).members
    ] == [0, 1]
    request_observation = StudyObservationRequest(
        study=handle.study_id.to_hex(),
        maximum_members=2,
        maximum_events=8,
    )
    with runtime.observe_study(request_observation) as observer:
        first = observer.poll()
        cursor = observer.cursor
        assert first.status.header.study == browsing.headers[0].study
    # Closing an observer cannot issue scientific cancellation, even if workers
    # happened to finish these two small constant operations before this read.
    assert not handle.status().cancelled
    with runtime.observe_study(request_observation, cursor=cursor) as reconnected:
        current = reconnected.poll()
        assert not current.status.header.cancelled
        assert len(current.status.members.members) == 2
    foreign = msgspec.structs.replace(
        cursor, issuing_store=cursor.issuing_store + "-foreign"
    )
    with pytest.raises(Exception, match="cursor issuer or exact selection differs"):
        runtime.observe_study(request_observation, cursor=foreign)
    wrong_context = msgspec.structs.replace(
        cursor.context, source=cursor.context.source + "-other"
    )
    wrong = msgspec.structs.replace(cursor, context=wrong_context)
    with pytest.raises(Exception, match="cursor issuer or exact selection differs"):
        runtime.observe_study(request_observation, cursor=wrong)
    with pytest.raises(Exception, match="narrow"):
        runtime.freeze_study_cohort(
            StudyCohortRequest(
                filter=selected,
                maximum_studies=1,
                maximum_members=1,
            )
        )
    # A frozen population remains exactly the captured population after later polls.
    assert frozen.receipt.member_count == 2
    assert not handle.status().cancelled
    # This component owns the explicit local worker; start_study publishes the
    # queue and intentionally does not create an external primary receiver.
    assert runtime.work() >= 3
    retained = handle.wait(controls=pse.StudyWaitControls(timeout_seconds=90))
    point_index, run, attempt = retained.points[0]
    assert point_index == 7
    assert attempt is not None
    original_rows = pa.table(
        runtime.results(run, attempt, "runtime.solve_variables")
    ).to_pylist()
    assert len(original_rows) == 1
    (original_row,) = original_rows
    assert original_row["value"] == 2.0
    assert original_row["quantity_id"] is not None
    assert original_row["unit_id"] is not None

    request = StudyCohortRequest(filter=selected, maximum_studies=1, maximum_members=2)
    completed = runtime.freeze_study_cohort(request)
    kpi = StudyKpiRequest(
        relation="runtime.solve_variables",
        owner=SemanticId(original_row["symbol_id"]),
        field="value",
        partition=f"step:{original_row['step']}",
        start=0,
        end=1,
    )
    # The first capture predates native completion; later usable results must
    # not promote its originally unavailable members.
    original_analysis = frozen.aggregate(kpi)
    assert (original_analysis.included, original_analysis.unavailable) == (0, 2)
    assert original_analysis.progress.state == "complete"
    assert not original_analysis.batches()
    analysis = completed.aggregate(kpi)
    assert (analysis.included, analysis.unavailable) == (2, 0)
    assert analysis.progress.state == "complete"
    aggregate_rows = pa.Table.from_batches(analysis.batches()).to_pylist()
    assert len(aggregate_rows) == 1
    (aggregate_row,) = aggregate_rows
    assert (aggregate_row["count"], aggregate_row["mean"]) == (2, 2.0)
    assert aggregate_row["quantity_id"] == original_row["quantity_id"]
    assert aggregate_row["unit_id"] == original_row["unit_id"]
    classified = analysis.classification_page(
        StudyCohortPageRequest(maximum=2, cursor=None)
    )
    assert [row.value.availability for row in classified.members] == [
        "included",
        "included",
    ]
    with completed.result(0, "runtime.solve_variables", start=0, end=1) as detail:
        assert pa.table(detail).to_pylist() == original_rows

    root = Path(__file__).resolve().parents[3]
    spill = tmp_path / "spill"
    spill.mkdir()
    action_path = tmp_path / "action.json"

    def invoke(action: dict[str, object], *, success: bool = True) -> object:
        action_path.write_bytes(
            msgspec.json.encode({"version": 1, "physical": None, "action": action})
        )
        result = subprocess.run(
            [
                str(root / "target/debug/pse-project"),
                "project",
                "--state",
                canonical_substrate.state,
                "--database",
                canonical_substrate.database,
                "runtime",
                "--request",
                str(action_path),
                "--spill-dir",
                str(spill),
                "--threads",
                "2",
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=90,
        )
        if not success:
            assert result.returncode != 0
            assert "review export output allowance exhausted" in result.stderr, (
                result.stderr
            )
            return None
        assert result.returncode == 0, result.stderr
        envelope = msgspec.json.decode(result.stdout, type=dict[str, object])
        assert envelope["version"] == 1
        return envelope["result"]

    browse_json = msgspec.json.decode(
        codec.encode_json(StudyWorkbenchRequest(filter=selected, maximum=1)),
        type=dict[str, object],
    )
    assert browse_json.pop("document_version") == 1
    cli_page = invoke({"kind": "study_workbench", "request": browse_json})
    assert isinstance(cli_page, dict)
    cli_headers = cli_page["headers"]
    assert isinstance(cli_headers, list)
    (cli_header,) = cli_headers
    assert isinstance(cli_header, dict)
    assert (cli_header["study"], cli_header["point_count"], cli_page["next"]) == (
        browsing.headers[0].study,
        2,
        None,
    )

    request_json = msgspec.json.decode(
        codec.encode_json(request), type=dict[str, object]
    )
    # The CLI action's versioned envelope embeds native bodies; these two
    # generated standalone headers belong to the Python document boundary.
    assert request_json.pop("document_version") == 1
    kpi_json = msgspec.json.decode(codec.encode_json(kpi), type=dict[str, object])
    assert kpi_json.pop("document_version") == 1
    pages = tmp_path / "cohort.ndjson"
    receipt = invoke(
        {"kind": "study_cohort", "request": request_json, "output": str(pages)}
    )
    assert isinstance(receipt, dict)
    assert receipt["bytes"] == pages.stat().st_size
    records = [
        msgspec.json.decode(line, type=dict[str, object])
        for line in pages.read_bytes().splitlines()
    ]
    assert records[-1]["kind"] == "receipt"
    assert records[-1]["body"] == receipt["receipt"]
    exported_members = 0
    for record in records:
        if record["kind"] == "members":
            body = record["body"]
            assert isinstance(body, dict)
            members = body["members"]
            assert isinstance(members, list)
            exported_members += len(members)
    assert exported_members == 2
    refused = tmp_path / "refused.ndjson"
    invoke(
        {
            "kind": "study_cohort",
            "request": request_json,
            "output": str(refused),
            "maximum_bytes": 1,
        },
        success=False,
    )
    assert not refused.exists()
    assert refused.with_suffix(".ndjson.incomplete").is_file()

    aggregate_path = tmp_path / "aggregate.arrow"
    cli_analysis = invoke(
        {
            "kind": "study_analyze",
            "request": request_json,
            "kpi": kpi_json,
            "output": str(aggregate_path),
        }
    )
    assert isinstance(cli_analysis, dict)
    assert (cli_analysis["included"], cli_analysis["unavailable"]) == (2, 0)
    with pa.ipc.open_file(aggregate_path) as reader:
        assert reader.read_all().to_pylist() == aggregate_rows
    detail_path = tmp_path / "detail.arrow"
    cli_detail = invoke(
        {
            "kind": "study_result",
            "request": request_json,
            "member": 0,
            "relation": "runtime.solve_variables",
            "start": 0,
            "end": 1,
            "output": str(detail_path),
            "maximum_bytes": 1 << 20,
        }
    )
    assert isinstance(cli_detail, dict)
    assert cli_detail["bytes"] == detail_path.stat().st_size
    with pa.ipc.open_stream(detail_path) as reader:
        assert reader.read_all().to_pylist() == original_rows

    observation_json = msgspec.json.decode(
        codec.encode_json(request_observation), type=dict[str, object]
    )
    assert observation_json.pop("document_version") == 1
    observed = invoke(
        {"kind": "study_observe", "request": observation_json, "cursor": None}
    )
    assert isinstance(observed, dict)
    status = observed["status"]
    assert isinstance(status, dict)
    header = status["header"]
    assert isinstance(header, dict)
    assert header["study"] == handle.study_id.to_hex()
    assert header["cancelled"] is False
    assert header["terminal"] is True
    sources = observed["sources"]
    assert isinstance(sources, list)
    parent_sources: list[dict[str, object]] = []
    for source in sources:
        assert isinstance(source, dict)
        position = source["position"]
        assert isinstance(position, dict)
        if position["run"] == header["run"]:
            parent_sources.append(source)
            assert position["history_complete"] is True
    assert len(parent_sources) == 1
    assert parent_sources[0]["continuity"] == "history_unavailable"
    assert parent_sources[0]["events"] == []
    assert parent_sources[0]["coverage"] == {"kind": "unknown"}
