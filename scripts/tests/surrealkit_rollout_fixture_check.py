# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable frozen forward-chain controls, separate from persistent maintenance.

The entire current generated canonical DDL remains its producer's input. Only
this probe owns the two synchronous additive declarations below. No canonical
interpretation is activated, and no failed/unknown SQL execution is disposed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import threading
import time
import tomllib
import urllib.error
import urllib.request
import uuid
from pathlib import Path

from scripts import native_operation
from scripts import surreal_server as server
from scripts import surrealkit as tool
from scripts import surrealkit_maintenance as maintenance
from scripts import test_resources as resources

PRODUCER = "scripts/tests/surrealkit_rollout_fixture_check.py"
REVOCATION_SQL = "SLEEP 20s; RETURN 'owner-revocation-barrier';"
DECLARATIONS = {
    "p37_probe_1.surql": b"DEFINE TABLE plan37_rollout_probe SCHEMAFULL;\nDEFINE FIELD value ON TABLE plan37_rollout_probe TYPE string;\n",
    "p37_probe_2.surql": b"DEFINE FIELD extra ON TABLE plan37_rollout_probe TYPE option<string>;\n",
}


def authorize(resource: str, folder: Path, additions: int) -> None:
    """Only this test producer can add declarations to its own staged probe."""
    record, _, _ = tool.fixture(resource)
    if folder.parent != Path(str(record["directory"])):
        raise tool.ToolError("Probe input escaped its registered namespace")
    authority = {
        "producer": PRODUCER,
        "producer_sha256": tool.digest(Path(__file__)),
        "folder": folder.name,
        "canonical_sha256": tool.digest(tool.ROOT / tool.GENERATED_SCHEMA),
        "additives": {
            name: hashlib.sha256(data).hexdigest()
            for name, data in list(DECLARATIONS.items())[:additions]
        },
    }
    for name, data in list(DECLARATIONS.items())[:additions]:
        path = folder / "schema" / name
        if path.exists() and path.read_bytes() != data:
            raise tool.ToolError("Probe-owned additive declaration changed")
        path.write_bytes(data)
    with resources.resource_update(resource) as current:
        current["rollout_probe"] = authority
    server.write_json(folder / "probe-input.json", authority)


def validate(resource: str, folder: Path) -> None:
    record, _, _ = tool.fixture(resource)
    owner = record.get("rollout_probe")
    if (
        not isinstance(owner, dict)
        or owner != server.read_json(folder / "probe-input.json")
        or owner.get("producer") != PRODUCER
        or owner.get("producer_sha256") != tool.digest(Path(__file__))
    ):
        raise tool.ToolError("Probe declaration authority differs")
    if (
        folder.parent != Path(str(record["directory"]))
        or folder.name != owner["folder"]
    ):
        raise tool.ToolError("Probe folder ownership differs")
    if any(
        path.is_symlink() or path.name in {".env", ".env.local", "surrealkit.toml"}
        for path in folder.rglob("*")
    ):
        raise tool.ToolError("Probe target or symlink injection refused")
    generated = tool.ROOT / tool.GENERATED_SCHEMA
    if (
        owner["canonical_sha256"] != tool.digest(generated)
        or (folder / "schema/canonical.surql").read_bytes() != generated.read_bytes()
    ):
        raise tool.ToolError("Production canonical declaration changed")
    actual = {
        str(path.relative_to(folder / "schema")): tool.digest(path)
        for path in (folder / "schema").rglob("*")
        if path.is_file()
    }
    if actual != {"canonical.surql": owner["canonical_sha256"], **owner["additives"]}:
        raise tool.ToolError("Unowned probe schema file set refused")
    for name, digest in owner["additives"].items():
        if hashlib.sha256(DECLARATIONS[name]).hexdigest() != digest:
            raise tool.ToolError("Probe additive bytes are not producer-owned")


def child(binary: Path, folder: Path, output: Path, command: list[str]) -> int:
    """Internal child in the existing admitted scope; retain actual beta output."""
    if native_operation.scope_owner() is None:
        raise tool.ToolError("Probe child lacks admitted native ownership")
    with (
        (output / "stdout").open("xb") as stdout,
        (output / "stderr").open("xb") as stderr,
    ):
        result = subprocess.run(
            [str(binary), *command],
            cwd=folder,
            env=dict(os.environ),
            stdout=stdout,
            stderr=stderr,
            timeout=120,
            check=False,
        )
    return result.returncode


def beta(
    binary: Path,
    resource: str,
    folder: Path,
    command: list[str],
    *,
    database: str = "forward",
    offline: bool = False,
    expected_failure: bool = False,
    control_folder: Path | None = None,
) -> dict[str, object]:
    validate(resource, folder)
    tool.verify(binary)
    selected_folder = control_folder or folder
    if control_folder is not None and (
        not offline or control_folder.parent != folder.parent
    ):
        raise tool.ToolError("Offline mutation is outside exact probe ownership")
    output = Path(str(resources.resource_status(resource)["directory"])) / (
        "command-" + uuid.uuid4().hex
    )
    output.mkdir(mode=0o700)
    observation: dict[str, object] = {}
    with tool.activity(resource) as (record, config, credentials):
        env = tool.environment(config, record, credentials, selected_folder)
        env["SURREALDB_NAME"] = database
        if offline:
            if command != ["rollout", "lint"]:
                raise tool.ToolError("Offline controls only admit beta lint")
            env["SURREALDB_HOST"] = "http://127.0.0.1:1"
        status = native_operation.run_child_scope(
            [
                sys.executable,
                "-m",
                "scripts.tests.surrealkit_rollout_fixture_check",
                "_child",
                "--binary",
                str(binary),
                "--folder",
                str(selected_folder),
                "--output",
                str(output),
                "--",
                *command,
            ],
            env,
            cwd=tool.ROOT,
            observation=observation,
        )
        result = {
            "command": command,
            "database": database,
            "offline": offline,
            "exit_code": status,
            "child": observation,
            "output": str(output),
        }
        server.write_json(output / "result.json", result)
        if observation.get("owned_drain") != "complete":
            raise tool.ToolError("Probe child drain unproved; namespace retained")
        if bool(status) != expected_failure:
            raise tool.ToolError(
                "Unexpected beta outcome; submitted SQL remains owned at " + str(output)
            )
        # Expected lint rejection never connected; successful effect commands
        # are still checked against the independent synchronous data oracle.
        resources.record_drain(resource)
    return result


def sql(resource: str, database: str, query: str) -> list[object]:
    """Independent authenticated synchronous data oracle using private NS access."""
    with tool.activity(resource) as (record, config, credentials):
        endpoint = (
            str(config["endpoint"])
            .replace("ws://", "http://", 1)
            .replace("wss://", "https://", 1)
        )
        request = urllib.request.Request(
            endpoint + "/signin",
            data=json.dumps(
                {
                    "ns": record["namespace"],
                    "user": credentials["username"],
                    "pass": credentials["password"],
                }
            ).encode(),
            headers={"Accept": "application/json", "Content-Type": "application/json"},
        )
        with urllib.request.urlopen(request, timeout=15) as response:
            authentication = json.load(response)
        if not isinstance(authentication, dict) or not isinstance(
            authentication.get("token"), str
        ):
            raise tool.ToolError("Private namespace sign-in was not acknowledged")
        request = urllib.request.Request(
            endpoint + "/sql",
            data=query.encode(),
            headers={
                "Accept": "application/json",
                "Authorization": "Bearer " + authentication["token"],
                "surreal-ns": str(record["namespace"]),
                "surreal-db": database,
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                values = json.load(response)
        except urllib.error.HTTPError as error:
            body = error.read().decode()
            server.write_json(
                Path(str(record["directory"]))
                / ("sql-error-" + uuid.uuid4().hex + ".json"),
                {"query": query, "status": error.code, "body": body},
            )
            raise tool.ToolError("Probe SQL HTTP failure: " + body) from error
        if not isinstance(values, list) or any(
            not isinstance(row, dict) or row.get("status") != "OK" for row in values
        ):
            raise tool.ToolError(
                "Synchronous probe SQL did not acknowledge every statement"
            )
        resources.record_drain(resource)
        return [row["result"] for row in values]


def offline_controls(
    binary: Path, resource: str, folder: Path
) -> list[dict[str, object]]:
    manifests = sorted((folder / "rollouts").glob("*.toml"))
    if len(manifests) != 3:
        raise tool.ToolError("Probe requires baseline plus two generated forward steps")
    results: list[dict[str, object]] = []
    for kind in ("missing", "falsified", "fork", "gap"):
        copy = folder.parent / ("offline-" + kind + "-" + uuid.uuid4().hex)
        shutil.copytree(folder, copy)
        if kind in {"missing", "falsified"}:
            manifest = tomllib.loads(manifests[-1].read_text())
            reference = next(
                file for step in manifest["steps"] for file in step.get("files", [])
            )
            frozen = copy / "rollouts" / manifest["id"] / reference["path"]
            if kind == "missing":
                frozen.unlink()
            else:
                frozen.write_text("DEFINE TABLE falsified_probe;")
        elif kind == "gap":
            (copy / "rollouts" / manifests[1].name).unlink()
        else:
            current = copy / "rollouts" / manifests[-1].name
            data = tomllib.loads(current.read_text())
            baseline = tomllib.loads(manifests[0].read_text())
            current.write_text(
                current.read_text().replace(
                    str(data["source_schema_hash"]), str(baseline["target_schema_hash"])
                )
            )
        results.append(
            beta(
                binary,
                resource,
                folder,
                ["rollout", "lint"],
                offline=True,
                expected_failure=True,
                control_folder=copy,
            )
        )
        results[-1]["control"] = kind
    return results


def run(state: Path, tool_root: Path, evidence: Path) -> None:
    if evidence.exists():
        raise tool.ToolError("Evidence destination already exists")
    evidence.mkdir(parents=True, mode=0o700)
    invocation = resources.invocation(
        "surrealkit-rollout",
        [{"name": "frozen_forward_chain"}],
        terminal_owner="standalone",
    )
    previous = os.environ.get(resources.MARKER)
    os.environ[resources.MARKER] = str(invocation)
    finalized = False
    try:
        # Registration precedes every beta child, including release verification.
        resource = tool.provision(state, test="frozen_forward_chain")
        binary = tool.acquire(tool_root)
        folder = tool.stage(resource, tool.ROOT / tool.GENERATED_SCHEMA)
        authorize(resource, folder, 0)
        proof: dict[str, object] = {
            "invocation": str(invocation),
            "resource": resource,
            "release": tool.verify(binary),
            "commands": [],
        }
        for index, name in enumerate(
            ("000_target", "001_probe_table", "002_probe_field")
        ):
            authorize(resource, folder, index)
            proof["commands"].append(
                beta(binary, resource, folder, ["rollout", "plan", "--name", name])
            )
            proof["commands"].append(
                beta(binary, resource, folder, ["rollout", "up", "--complete"])
            )
            if index == 1:
                assert (
                    sql(
                        resource,
                        "forward",
                        "CREATE plan37_rollout_probe:kept SET value='preserved'; RETURN (SELECT VALUE value FROM ONLY plan37_rollout_probe:kept);",
                    )[-1]
                    == "preserved"
                )
            if index == 2:
                proof["forward_data"] = sql(
                    resource,
                    "forward",
                    "RETURN (SELECT VALUE value FROM ONLY plan37_rollout_probe:kept); UPDATE plan37_rollout_probe:kept SET extra='expanded'; RETURN (SELECT `value`,extra FROM ONLY plan37_rollout_probe:kept);",
                )
                assert proof["forward_data"][0] == "preserved" and proof[
                    "forward_data"
                ][-1] == {"value": "preserved", "extra": "expanded"}
        proof["offline_controls"] = offline_controls(binary, resource, folder)
        proof["commands"].append(
            beta(
                binary,
                resource,
                folder,
                ["rollout", "up", "--complete"],
                database="catchup",
            )
        )
        catalogs = sql(
            resource, "catchup", "INFO FOR DB; INFO FOR TABLE plan37_rollout_probe;"
        )
        assert (
            "canonical_runs" in catalogs[0]["tables"]
            and "canonical_interpretations" in catalogs[0]["tables"]
        )
        assert {"value", "extra"} <= catalogs[1]["fields"].keys()
        proof["catchup_data"] = sql(
            resource,
            "catchup",
            "CREATE plan37_rollout_probe:kept SET value='preserved',extra='expanded'; RETURN (SELECT `value`,extra FROM ONLY plan37_rollout_probe:kept);",
        )
        assert proof["catchup_data"][-1] == {"value": "preserved", "extra": "expanded"}
        proof.update(
            catchup_catalogs=catalogs,
            producer=resources.resource_status(resource)["rollout_probe"],
            frozen_files={
                str(path.relative_to(folder / "rollouts")): tool.digest(path)
                for path in (folder / "rollouts").rglob("*")
                if path.is_file()
            },
        )
        for index, result in enumerate(
            [*proof["commands"], *proof["offline_controls"]]
        ):
            retained = evidence / ("command-" + str(index))
            shutil.copytree(Path(result["output"]), retained)
            result["output"] = str(retained)
        shutil.copytree(folder / "rollouts", evidence / "frozen-rollouts")
        server.write_json(evidence / "qualification.json", proof)
        eligible = resources.finalize(
            invocation,
            {"frozen_forward_chain": "pass"},
            complete=True,
            terminal_owner="standalone",
            terminal={"evidence": str(evidence / "qualification.json")},
        )
        finalized = True
        assert resource in eligible
        assert resources.reclaim(resource) == "removed"
        server.write_json(
            evidence / "disposal.json", {"resource": resource, "cleanup": "removed"}
        )
    except BaseException:
        if not finalized:
            resources.finalize(
                invocation,
                {"frozen_forward_chain": "failure"},
                complete=True,
                terminal_owner="standalone",
                terminal={"evidence": str(evidence), "failure": True},
            )
        raise
    finally:
        if previous is None:
            os.environ.pop(resources.MARKER, None)
        else:
            os.environ[resources.MARKER] = previous


def checked_catalog(values: list[object], *, bootstrap: bool = False) -> list[object]:
    """Retry only exact beta metadata absence while its original clock runs."""
    errors = [value for value in values if isinstance(value, str)]
    missing = {
        "The table '__rollout' does not exist",
        "The table '__entity' does not exist",
    }
    if errors:
        if bootstrap and all(value in missing for value in errors):
            return []
        raise tool.ToolError("Exact maintenance catalog query refused: " + repr(values))
    return values


def owner_revocation(
    state: Path, resource: str, folder: Path, binary: Path, evidence: Path
) -> None:
    """Revoke this test's reservation, keeping its native composer alive.

    Root supplies an exclusive closed disposable service selection, registered
    control and beta-generated single frozen baseline. This never switches a
    selected namespace/database or establishes production rollout acceptance.
    """
    if evidence.exists():
        raise tool.ToolError("Evidence destination already exists")
    config = maintenance.selected(state)
    record = resources.resource_status(resource)
    selection = {
        "state": str(state),
        "instance_id": config["instance_id"],
        "namespace": config["namespace"],
        "database": config["database"],
    }
    if (
        record.get("kind") not in {"controls", "evidence"}
        or record.get("cleanup") != "retained"
        or record.get("state") != str(state)
        or record.get("namespace") != config["namespace"]
        or record.get("maintenance_selection") != selection
        or record.get("service_generation") != config["instance_id"]
        or record.get("drained") is not True
        or record.get("creation_borrower")
        or record.get("borrows")
        or folder.parent != Path(str(record["directory"]))
    ):
        raise tool.ToolError(
            "Revocation requires its exact registered disposable maintenance control"
        )
    if config.get(maintenance.PENDING) is not None:
        raise tool.ToolError("Preexisting pending operation is not this probe")
    original_files, _ = maintenance.inputs(folder)
    manifests = sorted((folder / "rollouts").glob("*.toml"))
    if (
        len(manifests) != 1
        or (folder / "schema/canonical.surql").read_bytes()
        != (tool.ROOT / tool.GENERATED_SCHEMA).read_bytes()
    ):
        raise tool.ToolError(
            "Revocation requires the complete current generated baseline"
        )
    manifest = manifests[0]
    spec = tomllib.loads(manifest.read_text())
    if not any(
        file.get("hash") == tool.digest(tool.ROOT / tool.GENERATED_SCHEMA)
        for step in spec["steps"]
        for file in step.get("files", [])
    ):
        raise tool.ToolError(
            "Frozen baseline does not bind the actual production declaration"
        )
    barrier = (
        '[[steps]]\nid = "owner_revocation_barrier"\nphase = "start"\n'
        'kind = "run_sql"\nsql = ' + json.dumps(REVOCATION_SQL) + "\n\n"
    )
    manifest.write_text(
        manifest.read_text().replace("[[steps]]", barrier + "[[steps]]", 1)
    )
    _, selected_digest = maintenance.inputs(folder)
    authority = {
        "producer": PRODUCER,
        "producer_sha256": tool.digest(Path(__file__)),
        "input_sha256": selected_digest,
        "manifest_sha256": tool.digest(manifest),
        "original_manifest_sha256": hashlib.sha256(
            original_files["rollouts/" + manifest.name]
        ).hexdigest(),
        "barrier": REVOCATION_SQL,
        "control": "reservation_identity_revocation",
    }
    with resources.resource_update(resource) as current:
        current["maintenance_revocation_probe"] = authority
    evidence.mkdir(parents=True, mode=0o700)
    proof: dict[str, object] = {
        "resource": resource,
        "authority": authority,
        "tool": tool.verify(binary),
        "process_death_exercised": False,
    }
    observer_errors: list[str] = []
    revoked: dict[str, object] = {}
    with resources.borrow(resource), server.lifecycle_reservation(state):
        owner = server.read_json(state / "lifecycle-owner.json")
        proof["original_owner"] = owner

        # The control's borrow protects evidence files. Actual maintenance SQL
        # belongs to the reservation/pending marker, not an application fixture
        # borrower that would prevent its own required quiescence.
        def oracle(query: str, *, bootstrap: bool = False) -> list[object]:
            actual = server.config_for(state)
            maintenance.exact_owner(state, owner)
            server.require_maintenance_owner(state, actual)
            values = resources.namespace_sql(
                actual,
                query,
                namespace=str(actual["namespace"]),
                database=str(actual["database"]),
            )
            return checked_catalog(values, bootstrap=bootstrap)

        def observe_and_revoke() -> None:
            deadline = time.monotonic() + 25
            try:
                while time.monotonic() < deadline:
                    value = server.config_for(state).get(maintenance.PENDING)
                    if (
                        isinstance(value, dict)
                        and value.get("phase") == "submitted"
                        and value.get("attempts")
                    ):
                        attempt = Path(value["attempts"][-1]["directory"])
                        child_path = attempt / "child.json"
                        if child_path.exists():
                            child_record = server.read_json(child_path)
                            if isinstance(child_record.get("native_record"), dict):
                                # Exact beta.6 Module::default().partition(Lock)
                                # is 'lock'; retain its actual expiry/owner row.
                                values = oracle(
                                    "SELECT status,steps FROM __rollout; SELECT ns,key,val,val.expires_at > time::now() AS ttl_live FROM __entity WHERE ns='lock' AND key='global'; INFO FOR DB;",
                                    bootstrap=True,
                                )
                                if (
                                    values
                                    and values[0]
                                    and len(values[1]) == 1
                                    and values[1][0]["ttl_live"] is True
                                    and values[0][0]["status"] == "running_start"
                                    and any(
                                        step["step_id"] == "owner_revocation_barrier"
                                        and step["status"] == "running"
                                        for step in values[0][0]["steps"]
                                    )
                                ):
                                    proof.update(
                                        before_revocation=values,
                                        child_before_revocation=child_record,
                                        original_pending_nonce=value["nonce"],
                                    )
                                    revoked.update(
                                        owner, nonce="revoked-" + uuid.uuid4().hex
                                    )
                                    with server.state_lock(state, cleanup=True):
                                        maintenance.exact_owner(state, owner)
                                        server.write_json(
                                            state / "lifecycle-owner.json", revoked
                                        )
                                    proof["revoked_owner"] = dict(revoked)
                                    return
                    time.sleep(0.05)
                raise tool.ToolError(
                    "Actual running barrier and live beta TTL were not observed"
                )
            except BaseException as error:
                observer_errors.append(f"{type(error).__name__}: {error}")

        observer = threading.Thread(target=observe_and_revoke)
        observer.start()
        try:
            maintenance.execute(
                binary, state, folder, "start", rollout_id=spec["id"], timeout=90
            )
        except (
            maintenance.MaintenanceError,
            server.SupervisorError,
            OSError,
            ValueError,
        ) as error:
            proof["execution_error"] = f"{type(error).__name__}: {error}"
        finally:
            observer.join(timeout=30)
        if observer.is_alive() or observer_errors or not revoked:
            server.write_json(
                evidence / "unresolved.json",
                {**proof, "observer_errors": observer_errors},
            )
            raise tool.ToolError(
                "Owner revocation was not established; retain exact pending/listener"
            )
        value = server.config_for(state)[maintenance.PENDING]
        maintenance.require_child_drains(state, value)
        attempt = value["attempts"][-1]
        result = server.read_json(Path(attempt["directory"]) / "child.json")
        if (
            attempt.get("child", {}).get("owned_drain") != "complete"
            or result.get("reason") != "owner_lost"
            or value["sql_outcome"] != "unknown"
            or value["input_sha256"] != selected_digest
        ):
            raise tool.ToolError(
                "Original beta child stop/drain or unknown SQL identity unproved"
            )
        proof.update(after_child_drain=value, child_result=result)
        # Only this probe's original live process may restore its own revoked
        # reservation after exact child drain, to exercise actual listener drain.
        with server.state_lock(state, cleanup=True):
            if server.read_json(
                state / "lifecycle-owner.json"
            ) != revoked or not server.reservation_live(owner):
                raise tool.ToolError(
                    "Revocation owner changed; no authority restoration"
                )
            server.write_json(state / "lifecycle-owner.json", owner)
        maintenance.exact_owner(state, owner)
        server.drain_maintenance_listener(state)
        proof["listener_drain"] = "complete"
        refusals: dict[str, str] = {}
        for name, action in [
            ("open", lambda: server.open_context_admission(state)),
            ("start", lambda: server.start(state, server.config_for(state))),
        ]:
            try:
                action()
            except server.SupervisorError as error:
                refusals[name] = str(error)
            else:
                raise tool.ToolError(
                    "Ordinary " + name + " admitted an unknown rollout"
                )
        proof["ordinary_refusals"] = refusals
        recovered = maintenance.execute(
            binary, state, folder, "status", pending_nonce=value["nonce"]
        )
        if recovered["sql_outcome"] != "unknown" or recovered["phase"] != "unknown":
            raise tool.ToolError("Status erased the original uncertain phase")
        try:
            maintenance.execute(binary, state, folder, "repair", rollout_id=spec["id"], pending_nonce=value["nonce"], timeout=15)
        except maintenance.MaintenanceError as error:
            proof["repair_refusal"] = str(error)
        else:
            raise tool.ToolError("Metadata repair admitted unsettled SQL")
        proof["supported_repair_semantics"] = "metadata repair requires independent SQL settlement first"

        # A killed beta process normally retains its still-valid global lock.
        # One exact repair may therefore refuse; never manipulate its TTL.
        def unsettled_operation(
            _state: Path, _endpoint: str, selected: dict[str, object]
        ) -> dict[str, object]:
            rows = oracle("SELECT status,steps FROM __rollout; INFO FOR DB;")
            proof["recovery_catalog"] = rows
            if (
                selected["input_sha256"] != selected_digest
                or rows[0][0]["status"] not in {"running_start", "failed"}
                or any(
                    step["step_id"] == "owner_revocation_barrier"
                    and step["status"] == "completed"
                    for step in rows[0][0]["steps"]
                )
                or rows[1]["tables"] != proof["before_revocation"][2]["tables"]
            ):
                raise tool.ToolError(
                    "Interrupted nonmutating barrier has unexpected effects"
                )
            raise maintenance.MaintenanceError(
                "Beta metadata repair does not acknowledge interrupted SQL; exact step outcome remains unresolved"
            )

        try:
            maintenance.settle(state, value["nonce"], unsettled_operation)
        except maintenance.MaintenanceError as error:
            proof["settlement_refusal"] = str(error)
        else:
            raise tool.ToolError("Unsettled beta phase was cleared")
        final = server.config_for(state)
        if (
            final.get(maintenance.PENDING, {}).get("nonce") != value["nonce"]
            or final.get("accepting_writes") is not False
            or (state / ".maintenance-listener.json").exists()
        ):
            raise tool.ToolError("Unknown rollout did not retain closed/drained state")
        proof.update(
            final_pending=final[maintenance.PENDING],
            activation=False,
            admission_opened=False,
        )
        shutil.copytree(folder, evidence / "selected-input")
        shutil.copytree(state / ".surrealkit" / value["nonce"], evidence / "attempts")
        server.write_json(evidence / "qualification.json", proof)
        resources.record_drain(resource)
    # The failed/interrupted rollout remains an exact durable owner obligation.
    # Never finalize it as an eligible successful namespace removal.


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        nargs="?",
        choices=["run", "_child", "owner-revocation"],
        default="run",
    )
    parser.add_argument("--state", type=Path)
    parser.add_argument("--resource")
    parser.add_argument("--tool-root", type=Path)
    parser.add_argument("--evidence", type=Path)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--folder", type=Path)
    parser.add_argument("--output", type=Path)
    args, remaining = parser.parse_known_args()
    if args.command == "_child":
        if (
            args.binary is None
            or args.folder is None
            or args.output is None
            or remaining[:1] != ["--"]
        ):
            parser.error("Exact internal child selection required")
        raise SystemExit(child(args.binary, args.folder, args.output, remaining[1:]))
    if args.command == "owner-revocation":
        if (
            args.state is None
            or args.resource is None
            or args.folder is None
            or args.binary is None
            or args.evidence is None
            or remaining
        ):
            parser.error(
                "Select exact closed disposable state, resource, frozen baseline, binary and evidence"
            )
        owner_revocation(
            args.state, args.resource, args.folder, args.binary, args.evidence
        )
        return
    if (
        args.state is None
        or args.tool_root is None
        or args.evidence is None
        or remaining
    ):
        parser.error(
            "Select explicit disposable service, pinned tool root and evidence"
        )
    run(args.state, args.tool_root, args.evidence)


if __name__ == "__main__":
    main()
