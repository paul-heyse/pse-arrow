# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run pinned beta tooling inside registered, namespace-owned fixtures.

Generated definitions remain authoritative. This adapter neither activates the
canonical interpretation nor infers database settlement from a child exit.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import platform
import secrets
import subprocess
import tempfile
import tomllib
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, cast
from urllib.parse import urlsplit

from scripts import (
    native_operation,
    surreal_server,
    surrealkit_population,
    surrealkit_source,
    test_resources,
)

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping

ROOT = Path(__file__).resolve().parents[1]
GENERATED_SCHEMA = Path("crates/pse-operations/src/generated/surreal.surql")
# Exact server rejection signatures qualified against the generated contracts.
REJECTION_SIGNATURES = {
    "PSE_REFUSAL:original_canonical_rpc_deadline_expired",
    "but field must conform to:",
    "The record 'canonical_analysis_nodes:absent' does not exist",
}


class ToolError(RuntimeError):
    """Tool selection or authenticated fixture operation was refused."""


def grpc_endpoint(endpoint: str) -> str:
    """Use gRPC on the exact loopback listener owned by the supervisor."""
    value = urlsplit(endpoint)
    if (
        value.scheme not in {"http", "grpc"}
        or value.hostname != "127.0.0.1"
        or value.port is None
        or value.username is not None
        or value.password is not None
        or value.path not in {"", "/"}
        or value.query
        or value.fragment
    ):
        raise ToolError("Toolkit requires the exact owned loopback gRPC listener")
    return "grpc://127.0.0.1:" + str(value.port)


def selection() -> dict[str, str]:
    value = tomllib.loads((ROOT / ".config/surrealkit.toml").read_text())
    if (
        set(value) != {"version", "repository", "source_revision"}
        or value["repository"] != "surrealdb/surrealkit"
        or len(value["source_revision"]) != 40
        or any(c not in "0123456789abcdef" for c in value["source_revision"])
    ):
        raise ToolError("Invalid standalone tooling declaration")
    return value


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def acquire(tool_root: Path) -> Path:
    """The selected toolkit uses the same patched gRPC SDK as canonical clients."""
    declared = selection()
    if platform.system() != "Linux" or platform.machine() not in {"x86_64", "aarch64"}:
        raise ToolError("No qualified source toolkit for this platform")
    directory = surreal_server.checked_directory(
        tool_root / ("v" + declared["version"]) / ("grpc-" + platform.machine())
    )
    with surreal_server.state_lock(directory):
        identity = surrealkit_source.inputs(declared)
        binary = directory / "artifacts" / identity / "surrealkit"
        if binary.parent.exists():
            verify(binary)
            # Complete a prior publication whose post-rename parent sync may
            # have failed before its caller received an acquisition ACK.
            surrealkit_source.sync_directory(binary.parent.parent)
            return binary
        try:
            built = surrealkit_source.build(
                directory, declared, expected_inputs=identity
            )
        except surrealkit_source.BuildError as error:
            raise ToolError(str(error)) from error
        if built != binary:
            raise ToolError("Toolkit build published a different input identity")
        verify(binary)
    return binary


def _artifact_receipt(binary: Path) -> dict[str, object]:
    receipt_path = binary.parent / "release.json"
    if (
        not binary.is_absolute()
        or binary != surreal_server.lexical_absolute(binary)
        or any(path.is_symlink() for path in (binary, *binary.parents))
        or not binary.is_file()
        or binary.stat().st_uid != os.getuid()
        or receipt_path.is_symlink()
        or not receipt_path.is_file()
        or receipt_path.stat().st_uid != os.getuid()
    ):
        raise ToolError("Executable is not an owned regular immutable artifact")
    receipt = surreal_server.read_json(receipt_path)
    identity = receipt.get("sdk_inputs_sha256")
    if (
        not isinstance(identity, str)
        or len(identity) != 64
        or any(c not in "0123456789abcdef" for c in identity)
        or binary.name != "surrealkit"
        or binary.parent.name != identity
        or binary.parent.parent.name != "artifacts"
        or receipt.get("artifact") != str(binary)
        or receipt.get("publication") != "surrealkit-source-v1"
        or receipt.get("binary_sha256") != digest(binary)
        or receipt.get("source_build") is not True
        or receipt.get("release_build_locked") is not True
    ):
        raise ToolError("Executable provenance differs from immutable publication")
    return receipt


def _verify_version(binary: Path, receipt: Mapping[str, object]) -> None:
    version = receipt.get("version")
    if not isinstance(version, str):
        raise ToolError("Executable release version is absent")
    result = subprocess.run(
        [str(binary), "--version"],
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )
    if result.returncode or result.stdout.strip() != "surrealkit " + version:
        raise ToolError("Executable version differs from declared release")


def verify(binary: Path) -> dict[str, object]:
    """Ordinary execution always requires this checkout's current inputs."""
    receipt = _artifact_receipt(binary)
    declared = selection()
    if any(receipt.get(k) != v for k, v in declared.items()) or receipt.get(
        "sdk_inputs_sha256"
    ) != surrealkit_source.inputs(declared):
        raise ToolError("Executable provenance differs from declared release")
    _verify_version(binary, receipt)
    return receipt


def verify_saved(binary: Path, expected: Mapping[str, object]) -> dict[str, object]:
    """Verify only the exact tool tuple loaded from authenticated pending state."""
    receipt = _artifact_receipt(binary)
    if receipt != dict(expected):
        raise ToolError("Recovery tool provenance differs from submitted rollout")
    _verify_version(binary, receipt)
    return receipt


def provision(state: Path, *, test: str = "") -> str:
    namespace = "canonical_test_ns_" + uuid.uuid4().hex
    directory = state.resolve(strict=True) / ".tooling" / namespace
    resource = test_resources.register(
        {
            "state": str(state),
            "kind": "namespace",
            "namespace": namespace,
            "directory": str(directory),
            "test": test,
        }
    )
    directory.mkdir(parents=True, mode=0o700)
    credentials = {
        "username": "pse_tool",
        "password": secrets.token_urlsafe(32),
        "inspection_username": "pse_inspection",
        "inspection_password": secrets.token_urlsafe(32),
    }
    secret = directory / "credentials.json"
    secret.write_text(json.dumps(credentials))
    secret.chmod(0o600)
    config = surreal_server.config_for(state)
    with test_resources.resource_update(resource) as record:
        record["namespace_creation"] = "submitted"
        record["credentials_sha256"] = digest(secret)
    quoted = surreal_server.sql_identifier(namespace)
    sql = (
        f"DEFINE USER pse_tool ON NAMESPACE PASSWORD {json.dumps(credentials['password'])} ROLES OWNER;"
        f"DEFINE USER pse_inspection ON NAMESPACE PASSWORD {json.dumps(credentials['inspection_password'])} ROLES VIEWER;"
    )
    if test_resources.namespace_sql(config, f"DEFINE NAMESPACE {quoted};") != [
        None
    ] or test_resources.namespace_sql(config, sql, namespace=namespace) != [None, None]:
        raise ToolError(
            "Namespace provisioning unproved; registered scope remains pinned"
        )
    with test_resources.resource_update(resource) as record:
        record["namespace_creation"] = "acknowledged"
    test_resources.record_drain(resource)
    return resource


def fixture(
    resource: str,
    *,
    _population_recovery: bool = False,
) -> tuple[test_resources.ResourceRecord, dict[str, object], dict[str, object]]:
    record = test_resources.resource_status(resource)
    if (
        not isinstance(record, dict)
        or record.get("kind") != "namespace"
        or record.get("namespace_creation") != "acknowledged"
        or record.get("cleanup") != "retained"
    ):
        raise ToolError("Tooling requires an acknowledged registered namespace owner")
    if not _population_recovery and (
        record.get("drained") is not True
        or record.get("creation_borrower")
        or record.get("borrows")
    ):
        raise ToolError(
            "Prior tooling activity requires exact settlement and drain before reuse"
        )
    if not _population_recovery and record.get("tool_population", {}).get(
        "phase"
    ) not in {None, "settled"}:
        raise ToolError("Named population requires independent settlement before reuse")
    if _population_recovery:
        if record.get("tool_population", {}).get("phase") != "population_submitted":
            raise ToolError("Only the original pending population admits recovery")
        borrower = record.get("creation_borrower")
        if record.get("borrows") or (
            borrower
            and test_resources.borrower_alive(borrower)
            and borrower.get("pid") != os.getpid()
        ):
            raise ToolError("Population setup is still owned by a live borrower")
    config = surreal_server.config_for(Path(record["state"]))
    if (
        config["instance_id"] != record["service_generation"]
        or config["namespace"] != record["service_namespace"]
        or config.get("admission") != "open"
        or not config.get("accepting_writes")
    ):
        raise ToolError("Selected fixture service is unavailable or changed")
    secret = Path(str(record["directory"])) / "credentials.json"
    if secret.is_symlink() or digest(secret) != record.get("credentials_sha256"):
        raise ToolError("Namespace credentials changed")
    return record, config, surreal_server.read_json(secret)


def stage(resource: str, schema: Path) -> Path:
    record, _, _ = fixture(resource)
    if schema.resolve() != (ROOT / GENERATED_SCHEMA).resolve():
        raise ToolError("Schema input must be the production generated declaration")
    data = schema.read_bytes()
    if not data.strip():
        raise ToolError("Empty generated schema refused")
    folder = Path(str(record["directory"])) / ("input-" + uuid.uuid4().hex)
    (folder / "schema").mkdir(parents=True, mode=0o700)
    (folder / "schema/canonical.surql").write_bytes(data)
    (folder / "tests/suites").mkdir(parents=True)
    (folder / "tests/suites/canonical.toml").write_bytes(
        (ROOT / "scripts/tests/fixtures/surrealkit/canonical.toml").read_bytes()
    )
    (folder / "snapshots").mkdir()
    (folder / "snapshots/schema_snapshot.json").write_text(
        json.dumps({"version": 1, "files": []}) + "\n"
    )
    (folder / "snapshots/catalog_snapshot.json").write_text(
        json.dumps({"version": 2, "entities": []}) + "\n"
    )
    provenance: dict[str, object] = {
        "schema_sha256": hashlib.sha256(data).hexdigest(),
        "producer": str(GENERATED_SCHEMA),
        "files": {"canonical.surql": hashlib.sha256(data).hexdigest()},
    }
    (folder / "input.json").write_text(json.dumps(provenance) + "\n")
    # The resource authority, rather than an editable tool input, owns staging.
    with test_resources.resource_update(resource) as current:
        current.setdefault("tool_inputs", {})[folder.name] = provenance
    return folder


def environment(
    config: Mapping[str, object],
    record: Mapping[str, object],
    credentials: Mapping[str, object],
    folder: Path,
) -> dict[str, str]:
    result = {
        k: v
        for k, v in os.environ.items()
        if not k.startswith(("SURREAL", "DATABASE_"))
    }
    result.update(
        SURREALDB_HOST=grpc_endpoint(str(config["endpoint"])),
        SURREALDB_NAMESPACE=str(record["namespace"]),
        SURREALDB_NAME="tooling",
        SURREALDB_USER=str(credentials["username"]),
        SURREALDB_PASSWORD=str(credentials["password"]),
        SURREALDB_AUTH_LEVEL="namespace",
        SURREALDB_FOLDER=str(folder),
        SURREALDB_CONNECT_TIMEOUT_SECS="15",
        SURREALDB_QUERY_TIMEOUT_SECS="30",
    )
    if "inspection_username" in credentials and "inspection_password" in credentials:
        result.update(
            PSE_SURREALKIT_INSPECTION_USER=str(credentials["inspection_username"]),
            PSE_SURREALKIT_INSPECTION_PASSWORD=str(credentials["inspection_password"]),
        )
    return result


INSPECTION_ACTOR = {
    "inspection": {
        "kind": "namespace",
        "username_env": "PSE_SURREALKIT_INSPECTION_USER",
        "password_env": "PSE_SURREALKIT_INSPECTION_PASSWORD",
    }
}


def validate_input(
    folder: Path,
    record: Mapping[str, object],
    mode: str = "",
    release: Mapping[str, object] | None = None,
) -> None:
    del mode  # The shared command interface validates all modes against one inventory.
    root = Path(str(record["directory"])).resolve(strict=True)
    if folder.is_symlink() or root not in folder.resolve(strict=True).parents:
        raise ToolError("Tool input must belong to this registered namespace")
    if any(
        p.name in {".env", ".env.local", "surrealkit.toml"} or p.is_symlink()
        for p in folder.rglob("*")
    ):
        raise ToolError("Ambient configuration, targets or symlinked inputs refused")
    manifest = json.loads((folder / "input.json").read_text())
    authority = record.get("tool_inputs", {})
    expected = authority.get(folder.name) if isinstance(authority, dict) else None
    files = {
        str(path.relative_to(folder / "schema")): digest(path)
        for path in (folder / "schema").rglob("*")
        if path.is_file()
    }
    if (
        not isinstance(expected, dict)
        or manifest != expected
        or expected.get("producer") != str(GENERATED_SCHEMA)
        or expected.get("files") != files
        or expected.get("schema_sha256") != digest(ROOT / GENERATED_SCHEMA)
    ):
        raise ToolError(
            "Generated input file set or authoritative staging provenance changed"
        )
    suites = {
        str(path.relative_to(folder / "tests")): path.read_bytes()
        for path in (folder / "tests").rglob("*")
        if path.is_file()
    }
    if suites != {
        "suites/canonical.toml": (
            ROOT / "scripts/tests/fixtures/surrealkit/canonical.toml"
        ).read_bytes()
    }:
        raise ToolError(
            "Only authoritative qualified declarative SQL fixtures are admitted"
        )
    setup = folder / "setup.surql"
    if setup.exists() and (
        release is None or digest(setup) != release.get("setup_sha256")
    ):
        raise ToolError("Only exact release built-in setup SQL is admitted")
    if (folder / "seed").exists() or (folder / "seeds").exists():
        raise ToolError("Seed SQL requires separate settlement qualification")
    expected_rollouts = cast(
        "Mapping[str, object]", record.get("tool_rollouts", {})
    ).get(folder.name, {})
    rollout_files = {
        str(path.relative_to(folder / "rollouts")): digest(path)
        for path in (folder / "rollouts").rglob("*")
        if path.is_file()
    }
    if rollout_files != expected_rollouts:
        raise ToolError("Frozen rollout inputs differ from their generated owner")
    for path in folder.rglob("*.toml"):
        data = tomllib.loads(path.read_text())
        # Only the exact fixture-bound namespace VIEWER actor is qualified.
        if data.get("actors", INSPECTION_ACTOR) != INSPECTION_ACTOR or any(
            k in data
            for k in (
                "host",
                "namespace",
                "database",
                "user",
                "password",
                "targets",
                "target",
                "auth_level",
            )
        ):
            raise ToolError("Unqualified explicit actor or target override")

        def has_http_target(value: object) -> bool:
            if isinstance(value, dict):
                return bool({"base_url", "file", "schema_from"} & value.keys()) or any(
                    has_http_target(item) for item in value.values()
                )
            return isinstance(value, list) and any(
                has_http_target(item) for item in value
            )

        cases = data.get("cases", [])
        if (
            not isinstance(cases, list)
            or has_http_target(data)
            or any(
                not isinstance(case, dict)
                or case.get("kind") not in {"sql_expect", "permissions_matrix"}
                for case in cases
            )
        ):
            raise ToolError(
                "Initial tooling subset admits SQL expectations only; HTTP targets refused"
            )
        if "schema_from" in data or (
            isinstance(data.get("rollouts"), dict)
            and data["rollouts"].get("parity") is False
        ):
            raise ToolError(
                "Suite schema selection and parity suppression are not admitted"
            )
        for case in cases:
            if case.get("actor") not in {None, "root", "inspection"}:
                raise ToolError(
                    "Only the fixture bootstrap or inspection actor is admitted"
                )
            if case.get("kind") == "permissions_matrix" and (
                case.get("actor") != "inspection"
                or case.get("table") != "canonical_guards"
                or case.get("record_id") != "exact"
                or case.get("rules")
                != [
                    {"action": "select", "allow": True},
                    {"action": "create", "allow": False},
                ]
            ):
                raise ToolError(
                    "Only the exact qualified inspection permission matrix is admitted"
                )
            if (
                case.get("allow") is False
                and case.get("error_contains") not in REJECTION_SIGNATURES
            ):
                raise ToolError(
                    "Negative expectations require a qualified server rejection signature"
                )


@contextlib.contextmanager
def activity(
    resource: str,
    *,
    _population_recovery: bool = False,
) -> Generator[
    tuple[test_resources.ResourceRecord, dict[str, object], dict[str, object]],
    None,
    None,
]:
    """Publish actual SQL activity under the existing service admission owner."""
    record, _, _ = fixture(resource, _population_recovery=_population_recovery)
    with contextlib.ExitStack() as owned:
        with surreal_server.context_admission(
            Path(record["state"]), wait_for_lifecycle=False
        ):
            record, config, credentials = fixture(
                resource, _population_recovery=_population_recovery
            )
            owned.enter_context(test_resources.borrow(resource))
            with test_resources.resource_update(resource) as current:
                current["drained"] = False
                current["creation_borrower"] = {
                    "pid": os.getpid(),
                    "start": native_operation.start_identity(os.getpid()),
                    "boot": test_resources.host.boot(),
                }
        yield record, config, credentials


def _population_sql(
    config: Mapping[str, object], record: Mapping[str, object], sql: str,
) -> list[object]:
    values = test_resources.namespace_sql(
        config, sql, namespace=str(record["namespace"]),
        database=surrealkit_population.DATABASE,
    )
    if not values or any(isinstance(value, str) for value in values):
        encoded = json.dumps(values, sort_keys=True, separators=(",", ":")).encode()
        errors = [{"statement": index, "error": value}
                  for index, value in enumerate(values) if isinstance(value, str)]
        response = Path(str(record["directory"])) / ("population-response-" + uuid.uuid4().hex + ".json")
        surreal_server.write_json(response, {
            "sql_sha256": hashlib.sha256(sql.encode()).hexdigest(),
            "response_sha256": hashlib.sha256(encoded).hexdigest(),
            "response": values, "errors": errors,
        })
        specific = next((str(item["error"]) for item in errors
                         if "not executed" not in str(item["error"]).lower()),
                        str(errors[0]["error"]) if errors else "acknowledgment unavailable (empty response)")
        raise ToolError("Population SQL refused; scope remains pinned: " + specific[:240] + "; private response " + str(response))
    return values


def inspection_sql(
    config: Mapping[str, object], record: Mapping[str, object],
    credentials: Mapping[str, object], sql: str,
) -> list[object]:
    """A fresh namespace VIEWER session; root readback determines write denial."""
    env = {key: value for key, value in os.environ.items() if not key.startswith(("SURREAL", "DATABASE_"))}
    env.update(SURREAL_USER=str(credentials["inspection_username"]),
               SURREAL_PASS=str(credentials["inspection_password"]))
    server = cast("Mapping[str, object]", config["server"])
    namespace = str(record["namespace"])
    database = surrealkit_population.DATABASE
    with tempfile.TemporaryDirectory(dir=str(record["directory"])) as working:
        result = subprocess.run(
            [str(server["binary"]), "sql", "--endpoint", str(config["endpoint"]),
             "--auth-level", "namespace", "--namespace", namespace,
             "--database", database, "--json", "--hide-welcome"],
            input=sql, env=env, cwd=working, capture_output=True, text=True,
            timeout=30, check=False,
        )
    try:
        values = test_resources.cli_json(result.stdout, namespace=namespace, database=database)
    except ValueError as error:
        raise ToolError("Inspection session acknowledgment unavailable") from error
    if result.returncode or result.stderr.strip() or not isinstance(values, list):
        raise ToolError("Inspection session failed; population remains pinned")
    return values


def populate(resource: str, name: str = surrealkit_population.NAME) -> dict[str, object]:
    """Install only a named versioned specimen into a fresh owned database.

    Input-folder seed refusal and toolkit --no-seed remain in force. The resource
    records submission before any SQL; failure/interruption never clears its pin.
    """
    try:
        content = surrealkit_population.identity(name)
        sql = surrealkit_population.sql(name)
    except ValueError as error:
        raise ToolError(str(error)) from error
    record, _, _ = fixture(resource)
    if record.get("tool_population"):
        raise ToolError("This namespace already owns a population; select a fresh fixture")
    with activity(resource) as (record, config, _):
        if record.get("tool_population"):
            raise ToolError("This namespace already owns a population; select a fresh fixture")
        pending: dict[str, object] = {
            "name": name, "content_sha256": content,
            "schema_sha256": digest(ROOT / GENERATED_SCHEMA),
            "namespace": record["namespace"], "database": surrealkit_population.DATABASE,
            "service_generation": record["service_generation"],
            "credentials_sha256": record["credentials_sha256"], "phase": "schema_submitted",
        }
        with test_resources.resource_update(resource) as current:
            current["tool_population"] = pending
        catalog = test_resources.namespace_sql(config, "INFO FOR NS;", namespace=record["namespace"])
        if len(catalog) != 1 or not isinstance(catalog[0], dict) or not isinstance(catalog[0].get("databases"), dict):
            raise ToolError("Population database inventory unavailable")
        if surrealkit_population.DATABASE in catalog[0]["databases"]:
            raise ToolError("Named population requires an absent target database")
        if test_resources.namespace_sql(config,
            "DEFINE DATABASE " + surreal_server.sql_identifier(surrealkit_population.DATABASE) + ";",
            namespace=record["namespace"]) != [None]:
            raise ToolError("Named population database creation unproved")
        declaration = (ROOT / GENERATED_SCHEMA).read_text()
        if not declaration.startswith("-- @generated by pse-schema; do not edit\n"):
            raise ToolError("Unknown generated declaration header")
        _population_sql(config, record, declaration)
        with test_resources.resource_update(resource) as current:
            current["tool_population"]["phase"] = "population_submitted"
        _population_sql(config, record, sql)
        # Deliberately no drain/settled inference from mutation acknowledgments.
    return settle_population(resource)


def settle_population(resource: str) -> dict[str, object]:
    """Reconcile the original submission, including interrupted post-submit setup.

    The atomic population must be fully visible. An absent or partial population
    stays unknown/protected; this never resubmits mutations to finish a specimen.
    """
    before = test_resources.resource_status(resource)
    recovery = before.get("tool_population", {}).get("phase") == "population_submitted"
    with activity(resource, _population_recovery=recovery) as (record, config, credentials):
        pending = record.get("tool_population", {})
        name = pending.get("name")
        if name != surrealkit_population.NAME or pending.get("phase") not in {"population_submitted", "settled"}:
            raise ToolError("No exact submitted named population to settle")
        expected = {
            "name": name, "content_sha256": surrealkit_population.identity(str(name)),
            "schema_sha256": digest(ROOT / GENERATED_SCHEMA),
            "namespace": record["namespace"], "database": surrealkit_population.DATABASE,
            "service_generation": record["service_generation"],
            "credentials_sha256": record["credentials_sha256"],
        }
        if any(pending.get(key) != value for key, value in expected.items()):
            raise ToolError("Population identity differs from its submitted owner")
        for query, answer in surrealkit_population.checks(str(name)):
            if _population_sql(config, record, query) != [answer]:
                raise ToolError("Population content/row/relation settlement differs from its declaration")
        # The read actor must actually see the rows; an empty SELECT is no proof.
        if inspection_sql(config, record, credentials, "SELECT VALUE key FROM canonical_projects ORDER BY key;") != [["fixture_alpha", "fixture_beta", "fixture_gamma"]]:
            raise ToolError("Population inspection read permission unproved")
        inspection_sql(config, record, credentials,
            "CREATE canonical_projects:fixture_viewer_forbidden SET key='fixture_viewer_forbidden',title='forbidden',deleted=false,generation=0dec;")
        if _population_sql(config, record, "SELECT * FROM canonical_projects:fixture_viewer_forbidden;") != [[]]:
            raise ToolError("Population inspection write denial unproved")
        for hint in ("", "WITH INDEX study_ready_candidates", "WITH NOINDEX"):
            query = surrealkit_population.READINESS.format(hint=hint) + ";"
            if _population_sql(config, record, query) != [surrealkit_population.READINESS_ANSWER]:
                raise ToolError("Population readiness ordered answer differs")
        proof = {**expected, "phase": "settled", "rows": {
            table: len(rows) for table, rows in surrealkit_population.rows(str(name)).items()
        }, "permissions": "viewer_reads_owner_proves_no_write",
            "readiness_answer": surrealkit_population.READINESS_ANSWER}
        with test_resources.resource_update(resource) as current:
            current["tool_population"] = proof
        test_resources.record_drain(resource)
        return proof


def execute(binary: Path, resource: str, folder: Path, mode: str) -> int:
    release = verify(binary)
    record, _, _ = fixture(resource)
    validate_input(folder, record, mode, release)
    commands = {
        "sync": ["sync", "--no-prune"],
        "lint": ["rollout", "lint"],
        "plan": ["rollout", "plan", "--name", "initial_target"],
        "replay": ["rollout", "up", "--complete"],
        "test-sync": [
            "test",
            "--schema-from",
            "sync",
            "--no-seed",
            "--keep-db",
            "--json-out",
            str(folder / "test-sync.json"),
        ],
        "test-both": [
            "test",
            "--schema-from",
            "both",
            "--no-seed",
            "--keep-db",
            "--json-out",
            str(folder / "test-both.json"),
        ],
    }
    if mode not in commands:
        raise ToolError("Unsupported standalone tooling command")
    observation: dict[str, object] = {}
    with activity(resource) as (record, config, credentials):
        status = native_operation.run_child_scope(
            [str(binary), *commands[mode]],
            environment(config, record, credentials, folder),
            cwd=folder,
            observation=observation,
        )
        surreal_server.write_json(
            folder / ("command-" + uuid.uuid4().hex + ".json"),
            {"mode": mode, "exit_code": status, "child": observation},
        )
        if status == 0 and observation.get("owned_drain") == "complete":
            if mode == "plan":
                frozen = {
                    str(path.relative_to(folder / "rollouts")): digest(path)
                    for path in (folder / "rollouts").rglob("*")
                    if path.is_file()
                }
                with test_resources.resource_update(resource) as current:
                    current.setdefault("tool_rollouts", {})[folder.name] = frozen
            test_resources.record_drain(resource)
    if observation.get("owned_drain") != "complete":
        raise ToolError("Tool child drain unproved; namespace remains protected")
    return status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=[
            "install",
            "provision",
            "stage",
            "sync",
            "lint",
            "plan",
            "replay",
            "test-sync",
            "test-both",
            "populate",
            "population-settle",
        ],
    )
    parser.add_argument(
        "--tool-root",
        type=Path,
        default=Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share")))
        / "pse-arrow/tools/surrealkit",
    )
    parser.add_argument("--state", type=Path)
    parser.add_argument("--resource")
    parser.add_argument("--folder", type=Path)
    parser.add_argument("--population", default=surrealkit_population.NAME)
    args = parser.parse_args()
    try:
        if args.command == "install":
            print(acquire(args.tool_root))
            return 0
        if args.command == "provision" and args.state is not None:
            print(provision(args.state))
            return 0
        if args.command in {"populate", "population-settle"} and args.resource:
            print(json.dumps(populate(args.resource, args.population) if args.command == "populate" else settle_population(args.resource)))
            return 0
        if args.command == "stage" and args.resource:
            print(
                stage(
                    args.resource,
                    ROOT / "crates/pse-operations/src/generated/surreal.surql",
                )
            )
            return 0
        if args.resource and args.folder is not None:
            return execute(
                acquire(args.tool_root), args.resource, args.folder, args.command
            )
        parser.error("Select the explicit state or registered resource/input folder")
    except (
        ToolError,
        test_resources.ResourceError,
        surreal_server.SupervisorError,
    ) as error:
        parser.exit(2, str(error) + "\n")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
