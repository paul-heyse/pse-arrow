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
from pathlib import Path

from scripts import surreal_server as server
from scripts.tests.surreal_fixture_check import query


def fixture_api_transactions(state: Path, directory: Path) -> dict[str, object]:
    """Prove writes before errors roll back, while normal 200/409 writes commit."""
    config = server.config_for(state)
    if not (directory / "fixture-ownership.json").is_file() or not server.owns_listener(
        state, config
    ):
        raise server.SupervisorError("API transaction check requires its live fixture owner")
    database = "control_api_" + uuid.uuid4().hex
    namespace = str(config["namespace"])
    prefix = f"USE NS {namespace}; USE DB {database}; "
    server.write_json(directory / "api-transaction-owner.json", {
        "state": str(state), "instance_id": config["instance_id"],
        "namespace": namespace, "database": database,
    })
    # Existing fixed fixture query owner handles bootstrap authority without
    # exposing it to the public client or producing credential-bearing output.
    query(state, f"DEFINE DATABASE {database}; " + prefix + """
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
    """)

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
            content = response.read(65537)
            if len(content) > 65536:
                raise server.SupervisorError("Toy API response exceeds its bound")
            server.write_json(directory / ("api-response-" + path.replace("/", "-") + ".json"), {
                "status": response.status, "body": content.decode("utf-8", errors="replace"),
            })
            return response.status, content

    # Positive route and typed write controls prevent setup/routing/schema errors
    # from being mistaken for a successful rollback observation.
    status, body = invoke("pse/control/v1/browse", "POST")
    assert status == 200 and json.loads(body) == {"marker": "known-foreign-database-route"}
    status, body = invoke("success")
    assert status == 200 and json.loads(body) == {"marker": "welltyped-200"}
    status, body = invoke("explicit_refusal")
    assert status == 409 and json.loads(body) == {"marker": "welltyped-409"}
    status, body = invoke("write_failure")
    assert status == 500 and "owned API write failure witness" in body.decode(), (
        "API did not return the intended after-write THROW response"
    )
    rows = query(state, prefix + "SELECT VALUE marker FROM probe ORDER BY marker;")
    assert rows[-1]["result"] == ["welltyped-200", "welltyped-409"], (
        "Actual API error committed a partial effect or positive response lost a write"
    )
    proof: dict[str, object] = {
        "database": database,
        "checks": ["known-foreign-database-route", "http-200-welltyped-write-committed",
                   "http-409-explicit-response-write-committed", "http-after-write-throw-500-preserved",
                   "independent-root-readback-error-write-absent"],
        "failed": 0, "baseline": 0,
    }
    server.write_json(directory / "api-transaction-result.json", proof)
    return proof
