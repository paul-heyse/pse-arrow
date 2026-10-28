# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual API transaction controls inside an already owned disposable service.

Only a unique toy database is modified. The caller retains the service and fixture
owners until this function returns, and preserves failed fixtures for diagnosis.
"""

from __future__ import annotations

import json
import urllib.error
import urllib.request
import uuid
from typing import TYPE_CHECKING

from scripts import surreal_server as server
from scripts.tests.surreal_fixture_check import query

if TYPE_CHECKING:
    from pathlib import Path


def fixture_api_transactions(state: Path, directory: Path) -> dict[str, object]:
    """Prove writes before errors roll back, while normal 200/409 writes commit."""
    config = server.config_for(state)
    if not (directory / "fixture-ownership.json").is_file() or not server.owns_listener(
        state, config
    ):
        raise server.SupervisorError(
            "API transaction check requires its live fixture owner"
        )
    database = "control_api_" + uuid.uuid4().hex
    namespace = str(config["namespace"])
    prefix = f"USE NS {namespace}; USE DB {database}; "
    server.write_json(
        directory / "api-transaction-owner.json",
        {
            "state": str(state),
            "instance_id": config["instance_id"],
            "namespace": namespace,
            "database": database,
        },
    )
    # Existing fixed fixture query owner handles bootstrap authority without
    # exposing it to the public client or producing credential-bearing output.
    query(
        state,
        f"DEFINE DATABASE {database}; "
        + prefix
        + """
        DEFINE TABLE probe SCHEMAFULL;
        DEFINE FIELD marker ON probe TYPE string;
        DEFINE API '/write_failure' FOR get PERMISSIONS FULL MIDDLEWARE api::res::body('json') THEN {
            CREATE probe:failure SET marker='welltyped-before-throw';
            THROW 'owned API write failure witness';
        };
        DEFINE API '/success' FOR get PERMISSIONS FULL MIDDLEWARE api::res::body('json') THEN {
            CREATE probe:success SET marker='welltyped-200';
            RETURN {status:200,body:{marker:'welltyped-200'}};
        };
        DEFINE API '/explicit_refusal' FOR get PERMISSIONS FULL MIDDLEWARE api::res::body('json') THEN {
            CREATE probe:refusal SET marker='welltyped-409';
            RETURN {status:409,body:{marker:'welltyped-409'}};
        };
        DEFINE API '/pse/control/v1/browse' FOR post PERMISSIONS FULL MIDDLEWARE api::res::body('json') THEN {
            RETURN {status:200,body:{marker:'known-foreign-database-route'}};
        };
    """,
    )

    def invoke(path: str, method: str = "GET") -> tuple[int, bytes]:
        request = urllib.request.Request(
            f"http://127.0.0.1:{config['port']}/api/{namespace}/{database}/{path}",
            method=method,
            headers={"Accept": "application/json", "Accept-Encoding": "identity"},
        )
        try:
            response = urllib.request.urlopen(request, timeout=30)  # noqa: S310 -- verified owned loopback service
        except urllib.error.HTTPError as error:
            response = error
        with response:
            status: object = response.status
            content: object = response.read(65537)
            if type(status) is not int:
                raise server.SupervisorError(
                    "Toy API response has no integer HTTP status"
                )
            if not isinstance(content, bytes):
                raise server.SupervisorError("Toy API response is not bytes")
            if len(content) > 65536:
                raise server.SupervisorError("Toy API response exceeds its bound")
            server.write_json(
                directory / ("api-response-" + path.replace("/", "-") + ".json"),
                {
                    "status": status,
                    "body": content.decode("utf-8", errors="replace"),
                },
            )
            return status, content

    # Positive route and typed write controls prevent setup/routing/schema errors
    # from being mistaken for a successful rollback observation.
    status, body = invoke("pse/control/v1/browse", "POST")
    assert status == 200
    assert json.loads(body) == {"marker": "known-foreign-database-route"}
    status, body = invoke("success")
    assert status == 200
    assert json.loads(body) == {"marker": "welltyped-200"}
    status, body = invoke("explicit_refusal")
    assert status == 409
    assert json.loads(body) == {"marker": "welltyped-409"}
    status, body = invoke("write_failure")
    assert status == 500
    assert "owned API write failure witness" in body.decode(), (
        "API did not return the intended after-write THROW response"
    )
    rows = query(
        state, "SELECT VALUE marker FROM probe ORDER BY marker;", database=database
    )
    assert rows[-1]["result"] == ["welltyped-200", "welltyped-409"], (
        "Actual API error committed a partial effect or positive response lost a write"
    )
    proof: dict[str, object] = {
        "database": database,
        "checks": [
            "known-foreign-database-route",
            "http-200-welltyped-write-committed",
            "http-409-explicit-response-write-committed",
            "http-after-write-throw-500-preserved",
            "independent-root-readback-error-write-absent",
        ],
        "failed": 0,
        "baseline": 0,
    }
    server.write_json(directory / "api-transaction-result.json", proof)
    return proof


def fixture_strict_json(
    state: Path, directory: Path, database: str
) -> dict[str, object]:
    """Exercise strict library decoding against the legacy parser on an owned toy DB."""
    config = server.config_for(state)
    if not (directory / "fixture-ownership.json").is_file() or not server.owns_listener(
        state, config
    ):
        raise server.SupervisorError(
            "Strict JSON probe requires its live fixture owner"
        )
    declaration = server.read_json(directory / "api-transaction-owner.json")
    if (
        declaration.get("state") != str(state)
        or declaration.get("instance_id") != config["instance_id"]
        or declaration.get("database") != database
    ):
        raise server.SupervisorError("Strict JSON probe toy database identity differs")
    query(
        state,
        """
        DEFINE TABLE json_probe SCHEMAFULL;
        DEFINE FIELD hits ON json_probe TYPE int;
        CREATE json_probe:counter SET hits=0;
        DEFINE FUNCTION fn::control_probe::json($request:object) -> object {
            IF $request.headers['content-type']!='application/json' OR !type::is_bytes($request.body) {
                THROW 'PSE_REFUSAL:control_request_invalid';
            };
            RETURN encoding::json::decode(<string>$request.body);
        } PERMISSIONS NONE;
        DEFINE API '/strict_json' FOR post PERMISSIONS FULL
            MIDDLEWARE api::req::max_body(65536),api::res::body('json')
            THEN {
                LET $body=fn::control_probe::json($request);
                UPDATE json_probe:counter SET hits+=1;
                IF $body.fail=true { THROW 'owned strict JSON after-write witness'; };
                RETURN {status:200,body:{value:$body,string_flags:$body.probes.map(|$value| type::is_string($value))}};
            };
        DEFINE API '/legacy_json' FOR post PERMISSIONS FULL
            MIDDLEWARE api::req::max_body(65536),api::req::body('json'),api::res::body('json')
            THEN {
                RETURN {status:200,body:{uuid_is_string:type::is_string($request.body.uuid),uuid_is_uuid:type::is_uuid($request.body.uuid)}};
            };
    """,
        database=database,
    )
    observations: list[dict[str, object]] = []

    def invoke(
        path: str, body: bytes, content_type: str = "application/json"
    ) -> tuple[int, bytes]:
        request = urllib.request.Request(
            f"http://127.0.0.1:{config['port']}/api/{config['namespace']}/{database}/{path}",
            data=body,
            headers={"Content-Type": content_type, "Accept-Encoding": "identity"},
        )
        try:
            response = urllib.request.urlopen(request, timeout=30)  # noqa: S310 -- authenticated owned loopback fixture selection
        except urllib.error.HTTPError as error:
            response = error
        with response:
            status: object = response.status
            content: object = response.read(65537)
            if type(status) is not int or not isinstance(content, bytes):
                raise server.SupervisorError(
                    "Strict JSON response status/body types differ"
                )
            if len(content) > 65536:
                raise server.SupervisorError(
                    "Strict JSON response exceeds its allowance"
                )
            observations.append(
                {
                    "path": path,
                    "status": status,
                    "body": content.decode(errors="replace"),
                }
            )
            server.write_json(
                directory / "strict-json-observations.json", {"responses": observations}
            )
            return status, content

    uuid_text = "ABCDEFAB-CDEF-4ABC-8DEF-ABCDEFABCDEF"
    date_text = "2026-10-28T12:34:56.123456+05:30"
    record_text = "CaseSensitiveTable:CaseSensitiveId"
    payload = {
        "probes": [
            uuid_text,
            date_text,
            record_text,
            "literal/null",
            'escaped/"slash\\',
            "\U0001f9ea",
        ],
        "nested": {"uuid": uuid_text, "values": [date_text, {"record": record_text}]},
    }
    status, body = invoke("strict_json", json.dumps(payload).encode())
    assert status == 200
    assert json.loads(body) == {
        "value": payload,
        "string_flags": [True] * len(payload["probes"]),
    }
    status, body = invoke("legacy_json", json.dumps({"uuid": uuid_text}).encode())
    assert status == 200
    assert json.loads(body) == {"uuid_is_string": False, "uuid_is_uuid": True}

    def hits() -> object:
        return query(
            state, "SELECT VALUE hits FROM ONLY json_probe:counter;", database=database
        )[-1]["result"]

    assert hits() == 1
    for malformed in (b'{"probes":["\xff"]}', b'{"probes":[}'):
        status, body = invoke("strict_json", malformed)
        assert status == 500
        assert body
        assert hits() == 1
    status, body = invoke("strict_json", json.dumps(payload).encode(), "text/plain")
    assert status == 500
    assert b"PSE_REFUSAL:control_request_invalid" in body
    assert hits() == 1
    status, _ = invoke("strict_json", b" " * 65537)
    assert status == 413
    assert hits() == 1
    status, body = invoke("strict_json", json.dumps({**payload, "fail": True}).encode())
    assert status == 500
    assert b"owned strict JSON after-write witness" in body
    assert hits() == 1
    result: dict[str, object] = {
        "database": database,
        "checks": [
            "exact-uuid-date-record-nested-unicode-strings",
            "explicit-plain-string-types",
            "legacy-middleware-uuid-promotion-witness",
            "invalid-utf8-refused-before-handler",
            "malformed-json-refused-before-handler",
            "wrong-content-type-refused-before-handler",
            "oversize-refused-before-handler",
            "strict-json-after-write-throw-rollback",
        ],
        "failed": 0,
        "baseline": 0,
    }
    server.write_json(directory / "strict-json-result.json", result)
    return result
