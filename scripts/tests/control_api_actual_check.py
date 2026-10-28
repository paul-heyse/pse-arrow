# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Attach-only native-independent control acceptance on an explicitly owned test database.

The caller owns the selected server/database lifecycle and has provisioned two actors
through pse-canonical-init: one browse+cancel, one browse-only. This driver never reads
bootstrap credentials, starts a service, initializes schema or releases fixture owners.
"""

from __future__ import annotations

import argparse
import importlib.abc
import json
import os
import secrets
import stat
import subprocess
import sys
import time
import uuid
from pathlib import Path

import anyio

from scripts import surreal_server as server


class NativeBlocker(importlib.abc.MetaPathFinder):
    """Record forbidden scientific import attempts independently from success."""

    def __init__(self) -> None:
        self.attempts: list[str] = []

    def find_spec(
        self, fullname: str, _path: object = None, _target: object = None
    ) -> None:
        if fullname in {"pse._native", "pse._build", "pse._scientific"}:
            self.attempts.append(fullname)
            raise ImportError("Scientific imports are refused in control acceptance")


def actor_credentials(path: Path) -> tuple[str, str]:
    """Read the explicitly supplied private provisioning input, without echoing it."""
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    with os.fdopen(os.open(path, flags), "rb") as stream:
        metadata = os.fstat(stream.fileno())
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_mode & 0o077
            or metadata.st_uid != os.getuid()
            or metadata.st_size > 16 * 1024
        ):
            raise ValueError("Control acceptance requires a bounded private actor file")
        raw = stream.read(16 * 1024 + 1)
    if len(raw) > 16 * 1024:
        raise ValueError("Control acceptance actor file grew beyond its bound")
    value: object = json.loads(raw)
    if not isinstance(value, dict):
        raise TypeError("Control acceptance actor input is invalid")
    key, password = value.get("key"), value.get("password")
    if not isinstance(key, str) or not isinstance(password, str):
        raise TypeError("Control acceptance actor credentials are invalid")
    return key, password


def fixture_control_ingress(
    state: Path, directory: Path, initializer: Path, foreign_database: str
) -> dict[str, object]:
    """Provision scoped actors and run the public child inside an existing fixture."""
    config = server.config_for(state)
    if not (directory / "fixture-ownership.json").is_file() or not server.owns_listener(
        state, config
    ):
        raise server.SupervisorError(
            "Control ingress requires its live disposable fixture owner"
        )

    def issue(name: str, browse: bool, cancel: bool) -> tuple[Path, dict[str, object]]:
        path = directory / (name + ".json")
        body = {
            "document_version": 1,
            "key": name + "-" + uuid.uuid4().hex,
            "password": secrets.token_urlsafe(36),
            "browse": browse,
            "cancel": cancel,
        }
        with open(
            path, "x", opener=lambda file, flags: os.open(file, flags, 0o600)
        ) as stream:
            json.dump(body, stream)
        issued = subprocess.run(
            [str(initializer), "--issue-control-actor", str(path), str(state)],
            capture_output=True,
            text=True,
            check=False,
        )
        if issued.returncode:
            raise server.SupervisorError(
                "Control actor provisioning refused: " + issued.stderr.strip()
            )
        safe: dict[str, object] = json.loads(issued.stdout)
        if set(safe) != {"actor_id", "issuing_store"}:
            raise server.SupervisorError("Unexpected safe actor provisioning response")
        return path, safe

    owner, identity = issue("controller", True, True)
    viewer, viewer_identity = issue("browser", True, False)
    if identity["issuing_store"] != viewer_identity["issuing_store"]:
        raise server.SupervisorError("Provisioned actor issuer premises differ")
    completed = subprocess.run(
        [
            sys.executable,
            "-m",
            "scripts.tests.control_api_actual_check",
            "--endpoint",
            f"http://127.0.0.1:{config['port']}",
            "--namespace",
            str(config["namespace"]),
            "--database",
            str(config["database"]),
            "--foreign-database",
            foreign_database,
            "--issuing-store",
            str(identity["issuing_store"]),
            "--actor",
            str(owner),
            "--browse-only-actor",
            str(viewer),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    (directory / "control-api.stdout.log").write_text(completed.stdout)
    (directory / "control-api.stderr.log").write_text(completed.stderr)
    print(completed.stdout, end="", flush=True)
    if completed.returncode:
        print(completed.stderr, file=sys.stderr, end="", flush=True)
        raise server.SupervisorError(
            "Public actual-server control failed; original output retained"
        )
    _, post = issue("post-witness", False, False)
    if post["issuing_store"] != identity["issuing_store"]:
        raise server.SupervisorError("Independent post-control issuer witness changed")
    result: dict[str, object] = json.loads(completed.stdout)
    result["independent_issuer_before_after"] = True
    return result


async def check(args: argparse.Namespace, blocker: NativeBlocker) -> dict[str, object]:
    # Public import occurs only after installation of the independent import blocker.
    from pse.contracts.documents import (  # noqa: PLC0415
        ControlBrowseRequest,
        ControlCancelRequest,
        ControlReconcileRequest,
        StudyWorkbenchCursorBody,
        StudyWorkbenchFilter,
    )
    from pse.control import (  # noqa: PLC0415
        ControlClient,
        ControlError,
        ControlRefusedError,
    )

    checks: list[str] = []
    key, password = actor_credentials(args.actor)
    viewer_key, viewer_password = actor_credentials(args.browse_only_actor)
    maximum_response_bytes = 2 * 1024 * 1024

    def client(
        *, issuer: str | None = None, database: str | None = None
    ) -> ControlClient:
        return ControlClient(
            args.endpoint,
            args.namespace,
            database or args.database,
            None,
            issuing_store=issuer or args.issuing_store,
            timeout=30,
            maximum_request_bytes=64 * 1024,
            maximum_response_bytes=maximum_response_bytes,
        )

    absent = "control-absent-" + uuid.uuid4().hex
    operation = uuid.uuid4().hex
    browse = ControlBrowseRequest(
        document_version=1,
        filter=StudyWorkbenchFilter(study=absent),
        maximum=1,
        cursor=None,
    )
    cancel = ControlCancelRequest(
        document_version=1,
        issuing_store=args.issuing_store,
        operation_id=operation,
        study=absent,
    )
    reconcile = ControlReconcileRequest(
        document_version=1,
        issuing_store=args.issuing_store,
        operation_id=operation,
    )
    async with client() as owner:
        await owner.sign_in(key, password)
        page = await owner.study_workbench_page(browse)
        assert page.headers == ()
        assert page.next is None
        checks.append("actual-empty-compact-selection")
        valid_cursor = ControlBrowseRequest(
            document_version=1,
            filter=browse.filter,
            maximum=1,
            cursor=StudyWorkbenchCursorBody(
                issuing_store=args.issuing_store,
                filter=browse.filter,
                after=absent,
            ),
        )
        continued = await owner.study_workbench_page(valid_cursor)
        assert continued.headers == ()
        assert continued.next is None
        checks.append("actual-valid-matching-cursor-selection")
        receipt = await owner.cancel_study(cancel)
        assert receipt.operation_id == operation
        assert receipt.study == absent
        assert receipt.status == "refused"
        assert not receipt.cancelled
        checks.append("actual-absent-study-refusal-receipt")
        assert await owner.cancel_study(cancel) == receipt
        assert (await owner.reconcile(reconcile)).receipt == receipt
        checks.append("same-command-replay-and-original-reconciliation")
        changed = ControlCancelRequest(
            document_version=1,
            issuing_store=args.issuing_store,
            operation_id=operation,
            study=absent + "-changed",
        )
        refusal_reason = None
        try:
            await owner.cancel_study(changed)
        except ControlRefusedError as error:
            refusal_reason = str(error)
        assert refusal_reason == "control_command_identity_reused"
        assert (await owner.reconcile(reconcile)).receipt == receipt
        checks.append("changed-intent-refusal-preserves-original")
        # Test-only raw requests challenge authorization outside the constrained API.
        # The actual issued marker exists (independently witnessed by provisioning),
        # so empty read/write results cannot be explained by a missing record.
        import httpx2  # noqa: PLC0415 -- deliberate negative protocol peer

        token = owner._token  # noqa: SLF001 -- fixture token used solely for direct-API denial controls

        import msgspec  # noqa: PLC0415 -- encode the actual generated positive request before corruption

        invalid_cursor = json.loads(msgspec.json.encode(valid_cursor))
        invalid_cursor["cursor"]["unexpected"] = True
        envelope = {
            "expires_at": time.time_ns() // 1000 + 30_000_000,
            "issuing_store": args.issuing_store,
            "request": invalid_cursor,
        }
        with anyio.fail_after(30):
            async with httpx2.AsyncClient(
                trust_env=False, follow_redirects=False
            ) as raw:
                async with raw.stream(
                    "POST",
                    f"{args.endpoint}/api/{args.namespace}/{args.database}/pse/control/v1/browse",
                    content=json.dumps(envelope).encode(),
                    headers={
                        "Authorization": "Bearer " + str(token),
                        "Content-Type": "application/json",
                        "Accept-Encoding": "identity",
                    },
                ) as response:
                    body = bytearray()
                    async for chunk in response.aiter_raw():
                        assert len(body) + len(chunk) <= maximum_response_bytes
                        body.extend(chunk)
                    assert response.status_code == 500
                    assert b"PSE_REFUSAL:control_request_invalid" in body
        checks.append("actual-cursor-unknown-field-contract-refusal")

        # An independent positive routing control established this database/API.
        # Reuse the valid original token across the URL-selected database boundary.
        with anyio.fail_after(30):
            async with httpx2.AsyncClient(
                trust_env=False, follow_redirects=False
            ) as raw:
                async with raw.stream(
                    "POST",
                    f"{args.endpoint}/api/{args.namespace}/{args.foreign_database}/pse/control/v1/browse",
                    content=b"{}",
                    headers={
                        "Authorization": "Bearer " + str(token),
                        "Content-Type": "application/json",
                        "Accept-Encoding": "identity",
                    },
                ) as response:
                    assert response.status_code in (401, 403), (
                        "Valid actor token crossed the database URL boundary"
                    )
        checks.append("actual-valid-token-cross-database-url-authentication-denial")

        async def sql(text: str) -> tuple[int, object]:
            with anyio.fail_after(30):
                async with httpx2.AsyncClient(
                    trust_env=False, follow_redirects=False
                ) as raw:
                    async with raw.stream(
                        "POST",
                        args.endpoint + "/sql",
                        content=text.encode("utf-8"),
                        headers={
                            "Authorization": "Bearer " + str(token),
                            "surreal-ns": args.namespace,
                            "surreal-db": args.database,
                            "Accept": "application/json",
                            "Accept-Encoding": "identity",
                        },
                    ) as response:
                        assert (
                            response.headers.get("content-encoding", "identity").lower()
                            == "identity"
                        )
                        body = bytearray()
                        async for chunk in response.aiter_raw():
                            assert len(body) + len(chunk) <= maximum_response_bytes
                            body.extend(chunk)
                        return response.status_code, json.loads(body)

        forged_command = (
            "RETURN fn::pse_control_v1::cancel("
            + str(time.time_ns() // 1000 + 30_000_000)
            + ",'canonical_control_actors:forged',{document_version:1,issuing_store:'"
            + str(uuid.UUID(args.issuing_store))
            + "',operation_id:'"
            + uuid.uuid4().hex
            + "',study:'"
            + absent
            + "'});"
        )
        for query, label in (
            ("RETURN fn::pse_control_v1::actor('cancel');", "direct-helper-function"),
            (forged_command, "direct-command-forged-actor"),
        ):
            status, denied = await sql(query)
            denial_text = json.dumps(denied).lower()
            assert status in (401, 403) or (
                status == 200
                and isinstance(denied, list)
                and len(denied) == 1
                and isinstance(denied[0], dict)
                and denied[0].get("status") == "ERR"
                and any(
                    word in denial_text
                    for word in ("permission", "not allowed", "access")
                )
            ), "Direct function did not fail at authorization"
            checks.append("actual-" + label + "-authorization-denial")

        for query, label in (
            (
                "SELECT * FROM canonical_interpretations WHERE id=canonical_interpretations:current;",
                "private-existing-record-read-denial",
            ),
            (
                "UPDATE canonical_interpretations:current SET issuing_store=issuing_store RETURN AFTER;",
                "private-existing-record-write-denial",
            ),
        ):
            status, denied = await sql(query)
            assert status == 200
            assert isinstance(denied, list)
            assert len(denied) == 1
            result = denied[0]
            assert isinstance(result, dict)
            if result.get("status") == "ERR":
                text = str(result.get("result", "")).lower()
                assert any(
                    word in text for word in ("permission", "not allowed", "access")
                ), "Private-record challenge failed for an unrelated reason"
            else:
                assert result.get("status") == "OK"
                assert result.get("result") in (
                    None,
                    [],
                )
            checks.append("actual-" + label)

        unobserved = ControlReconcileRequest(
            document_version=1,
            issuing_store=args.issuing_store,
            operation_id=uuid.uuid4().hex,
        )
        assert (await owner.reconcile(unobserved)).receipt is None
        checks.append("missing-receipt-remains-unobserved")

    async with client() as viewer:
        await viewer.sign_in(viewer_key, viewer_password)
        assert not (await viewer.study_workbench_page(browse)).headers
        try:
            await viewer.cancel_study(cancel)
        except ControlError:
            pass
        else:
            raise AssertionError("Browse-only actor acquired cancellation authority")
        checks.append("actual-browse-only-actor-denial")

    # Challenge endpoint authentication itself, rather than only the client's local token check.
    import httpx2  # noqa: PLC0415 -- test-only deliberately unauthenticated protocol request

    with anyio.fail_after(30):
        async with httpx2.AsyncClient(trust_env=False, follow_redirects=False) as raw:
            async with raw.stream(
                "POST",
                f"{args.endpoint}/api/{args.namespace}/{args.database}/pse/control/v1/browse",
                content=b"{}",
                headers={
                    "Content-Type": "application/json",
                    "Accept-Encoding": "identity",
                },
            ) as response:
                assert response.status_code in (401, 403), (
                    "Unauthenticated endpoint request was not refused"
                )
    checks.append("actual-unauthenticated-endpoint-denial")

    # This uses the same live server and valid actor but an old/foreign immutable issuer.
    stale_issuer = str(uuid.uuid4())
    async with client(issuer=stale_issuer) as stale:
        await stale.sign_in(key, password)
        stale_request = ControlCancelRequest(
            document_version=1,
            issuing_store=stale_issuer,
            operation_id=operation,
            study=absent,
        )
        try:
            await stale.cancel_study(stale_request)
        except ControlError:
            pass
        else:
            raise AssertionError(
                "Foreign issuer command reached current database state"
            )
        checks.append("actual-stale-issuer-denial")

    assert not blocker.attempts, "Control attempted to load a scientific module"
    assert not {"pse._native", "pse._build", "pse._scientific"}.intersection(
        sys.modules
    )
    checks.append("native-loading-unavailable-through-public-journey")
    return {
        "scope": "real control ingress, actors and refused-command receipts; no native worker or scientific qualification",
        "checks": checks,
        "failed": 0,
        "baseline": 0,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--namespace", required=True)
    parser.add_argument("--database", required=True)
    parser.add_argument(
        "--foreign-database",
        required=True,
        help="Existing isolated fixture DB with positively witnessed browse route",
    )
    parser.add_argument("--issuing-store", required=True)
    parser.add_argument("--actor", type=Path, required=True)
    parser.add_argument("--browse-only-actor", type=Path, required=True)
    args = parser.parse_args()
    blocker = NativeBlocker()
    sys.meta_path.insert(0, blocker)
    print(json.dumps(anyio.run(check, args, blocker), sort_keys=True))


if __name__ == "__main__":
    main()
