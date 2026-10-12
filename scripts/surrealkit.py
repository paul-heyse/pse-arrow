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
import io
import json
import os
import platform
import secrets
import subprocess
import tarfile
import tomllib
import urllib.request
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import native_operation, surreal_server, test_resources

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping

ROOT = Path(__file__).resolve().parents[1]
GENERATED_SCHEMA = Path("crates/pse-operations/src/generated/surreal.surql")
# Exact server rejection signatures qualified against the generated contracts.
REJECTION_SIGNATURES = {
    "original canonical RPC deadline expired",
    "but field must conform to:",
    "The record 'canonical_analysis_nodes:absent' does not exist",
}


class ToolError(RuntimeError):
    """Tool selection or authenticated fixture operation was refused."""


def selection() -> dict[str, str]:
    value = tomllib.loads((ROOT / ".config/surrealkit.toml").read_text())
    if (
        set(value) != {"version", "repository"}
        or value["repository"] != "surrealdb/surrealkit"
    ):
        raise ToolError("Invalid standalone tooling declaration")
    return value


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def release_sources(declared: Mapping[str, str]) -> dict[str, object]:
    """Record exact-tag source and its locked official build basis, not an attestation."""
    repository, tag = declared["repository"], "v" + declared["version"]
    with urllib.request.urlopen(
        "https://api.github.com/repos/" + repository + "/git/ref/tags/" + tag,
        timeout=30,
    ) as response:
        target = json.load(response)["object"]
    while target["type"] == "tag":
        with urllib.request.urlopen(target["url"], timeout=30) as response:
            target = json.load(response)["object"]
    if target["type"] != "commit" or len(target["sha"]) != 40:
        raise ToolError("Exact release source commit unavailable")
    commit = target["sha"]
    base = "https://raw.githubusercontent.com/" + repository + "/" + commit + "/"
    sources = {}
    for name in (
        "Cargo.lock",
        ".github/workflows/release-binaries.yml",
        "crates/surrealkit/src/scaffold.rs",
    ):
        with urllib.request.urlopen(base + name, timeout=30) as response:
            sources[name] = response.read()
    packages = tomllib.loads(sources["Cargo.lock"].decode())["package"]
    sdk = [package["version"] for package in packages if package["name"] == "surrealdb"]
    scaffold = sources["crates/surrealkit/src/scaffold.rs"].decode()
    setup = scaffold.split('pub const DEFAULT_SETUP: &str = r#"', 1)[1].split('"#;', 1)[
        0
    ]
    return {
        "source_commit": commit,
        "source_sha256": {
            name: hashlib.sha256(data).hexdigest() for name, data in sources.items()
        },
        "resolved_sdk": sdk,
        "release_build_locked": any(
            "cargo build " in line and "--locked" in line and "--release" in line
            for line in sources[".github/workflows/release-binaries.yml"]
            .decode()
            .splitlines()
        ),
        "setup_sha256": hashlib.sha256(setup.encode()).hexdigest(),
    }


def acquire(tool_root: Path) -> Path:
    declared = selection()
    if platform.system() != "Linux" or platform.machine() not in {"x86_64", "aarch64"}:
        raise ToolError("No qualified beta executable for this platform")
    tag = "v" + declared["version"]
    target = platform.machine() + "-unknown-linux-gnu"
    directory = surreal_server.checked_directory(tool_root / tag / target)
    binary, receipt = directory / "surrealkit", directory / "release.json"
    with surreal_server.state_lock(directory):
        if binary.exists() or receipt.exists():
            verify(binary)
            stored = surreal_server.read_json(receipt)
            if (
                "source_commit" not in stored
                or stored.get("release_build_locked") is not True
            ):
                surreal_server.write_json(
                    receipt, {**stored, **release_sources(declared)}
                )
            return binary
        with urllib.request.urlopen(
            "https://api.github.com/repos/"
            + declared["repository"]
            + "/releases/tags/"
            + tag,
            timeout=30,
        ) as response:
            release = json.load(response)
        if not isinstance(release, dict):
            raise ToolError("Invalid official release metadata")
        if release.get("tag_name") != tag or release.get("draft"):
            raise ToolError("Release identity differs from exact tooling selection")
        name = "surrealkit-" + tag + "-" + target + ".tar.gz"
        assets = release.get("assets")
        if not isinstance(assets, list):
            raise ToolError("Missing official executable assets")
        asset = next(
            (a for a in assets if isinstance(a, dict) and a.get("name") == name), None
        )
        if not isinstance(asset, dict):
            raise ToolError("Missing exact executable asset")
        sha = str(asset.get("digest", ""))
        origin = (
            "https://github.com/"
            + declared["repository"]
            + "/releases/download/"
            + tag
            + "/"
        )
        if (
            not sha.startswith("sha256:")
            or len(sha) != 71
            or not str(asset.get("browser_download_url", "")).startswith(origin)
        ):
            raise ToolError("Official checksum or origin unavailable")
        with urllib.request.urlopen(
            str(asset["browser_download_url"]), timeout=60
        ) as response:
            raw = response.read()
        if "sha256:" + hashlib.sha256(raw).hexdigest() != sha:
            raise ToolError("Downloaded beta archive digest differs")
        with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
            members = [
                m
                for m in archive.getmembers()
                if m.isfile() and Path(m.name).name == "surrealkit"
            ]
            if len(members) != 1:
                raise ToolError("Archive has no unique executable")
            source = archive.extractfile(members[0])
            if source is None:
                raise ToolError("Archive executable unavailable")
            payload = source.read()
        with binary.open("xb") as target_file:
            target_file.write(payload)
        binary.chmod(0o700)
        receipt.write_text(
            json.dumps(
                {
                    **declared,
                    **release_sources(declared),
                    "archive_sha256": sha[7:],
                    "binary_sha256": digest(binary),
                    "asset": asset["browser_download_url"],
                },
                indent=2,
            )
            + "\n"
        )
        receipt.chmod(0o600)
        verify(binary)
    return binary


def verify(binary: Path) -> dict[str, object]:
    if binary.is_symlink() or binary.parent.is_symlink() or not binary.is_file():
        raise ToolError("Executable is not an owned regular file")
    receipt = surreal_server.read_json(binary.parent / "release.json")
    declared = selection()
    if any(receipt.get(k) != v for k, v in declared.items()) or receipt.get(
        "binary_sha256"
    ) != digest(binary):
        raise ToolError("Executable provenance differs from declared release")
    result = subprocess.run(
        [str(binary), "--version"],
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )
    if (
        result.returncode
        or result.stdout.strip() != "surrealkit " + declared["version"]
    ):
        raise ToolError("Executable version differs from declared release")
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
    credentials = {"username": "pse_tool", "password": secrets.token_urlsafe(32)}
    secret = directory / "credentials.json"
    secret.write_text(json.dumps(credentials))
    secret.chmod(0o600)
    config = surreal_server.config_for(state)
    with test_resources.resource_update(resource) as record:
        record["namespace_creation"] = "submitted"
        record["credentials_sha256"] = digest(secret)
    quoted = surreal_server.sql_identifier(namespace)
    sql = f"DEFINE USER pse_tool ON NAMESPACE PASSWORD {json.dumps(credentials['password'])} ROLES OWNER;"
    if test_resources.namespace_sql(config, f"DEFINE NAMESPACE {quoted};") != [
        None
    ] or test_resources.namespace_sql(config, sql, namespace=namespace) != [None]:
        raise ToolError(
            "Namespace provisioning unproved; registered scope remains pinned"
        )
    with test_resources.resource_update(resource) as record:
        record["namespace_creation"] = "acknowledged"
    test_resources.record_drain(resource)
    return resource


def fixture(
    resource: str,
) -> tuple[test_resources.ResourceRecord, dict[str, object], dict[str, object]]:
    record = test_resources.resource_status(resource)
    if (
        not isinstance(record, dict)
        or record.get("kind") != "namespace"
        or record.get("namespace_creation") != "acknowledged"
        or record.get("cleanup") != "retained"
    ):
        raise ToolError("Tooling requires an acknowledged registered namespace owner")
    if (
        record.get("drained") is not True
        or record.get("creation_borrower")
        or record.get("borrows")
    ):
        raise ToolError(
            "Prior tooling activity requires exact settlement and drain before reuse"
        )
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
    provenance = {
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
        SURREALDB_HOST=str(config["endpoint"]),
        SURREALDB_NAMESPACE=str(record["namespace"]),
        SURREALDB_NAME="tooling",
        SURREALDB_USER=str(credentials["username"]),
        SURREALDB_PASSWORD=str(credentials["password"]),
        SURREALDB_AUTH_LEVEL="namespace",
        SURREALDB_FOLDER=str(folder),
        SURREALDB_CONNECT_TIMEOUT_SECS="15",
        SURREALDB_QUERY_TIMEOUT_SECS="30",
    )
    return result


def validate_input(
    folder: Path,
    record: Mapping[str, object],
    mode: str = "",
    release: Mapping[str, object] | None = None,
) -> None:
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
    expected_rollouts = record.get("tool_rollouts", {}).get(folder.name, {})
    rollout_files = {
        str(path.relative_to(folder / "rollouts")): digest(path)
        for path in (folder / "rollouts").rglob("*")
        if path.is_file()
    }
    if rollout_files != expected_rollouts:
        raise ToolError("Frozen rollout inputs differ from their generated owner")
    for path in folder.rglob("*.toml"):
        data = tomllib.loads(path.read_text())
        # Initial supported subset is the default namespace bootstrap actor.
        # Explicit actors/config overrides need separate qualification.
        if "actors" in data or any(
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
                not isinstance(case, dict) or case.get("kind") != "sql_expect"
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
) -> Generator[
    tuple[test_resources.ResourceRecord, dict[str, object], dict[str, object]],
    None,
    None,
]:
    """Publish actual SQL activity under the existing service admission owner."""
    record, _, _ = fixture(resource)
    with contextlib.ExitStack() as owned:
        with surreal_server.context_admission(
            Path(record["state"]), wait_for_lifecycle=False
        ):
            record, config, credentials = fixture(resource)
            owned.enter_context(test_resources.borrow(resource))
            with test_resources.resource_update(resource) as current:
                current["drained"] = False
                current["creation_borrower"] = {
                    "pid": os.getpid(),
                    "start": native_operation.start_identity(os.getpid()),
                    "boot": test_resources.host.boot(),
                }
        yield record, config, credentials


def execute(binary: Path, resource: str, folder: Path, mode: str) -> int:
    release = verify(binary)
    record, config, credentials = fixture(resource)
    validate_input(folder, record, mode, release)
    commands = {
        "sync": ["sync", "--no-prune"],
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
            "plan",
            "replay",
            "test-sync",
            "test-both",
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
    args = parser.parse_args()
    try:
        if args.command == "install":
            print(acquire(args.tool_root))
            return 0
        if args.command == "provision" and args.state is not None:
            print(provision(args.state))
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
