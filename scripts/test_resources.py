# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Associate fixture lifetimes with existing runner outcomes and retained references.

Registration starts pinned. Drop records drain only. The runner supplies reconciled
outcomes; this module neither selects tests nor manufactures a passing result.
Reclaim marks an exact resource before doing work, so a concurrent new reference is
refused rather than allowed to race a removal. Unknown historical evidence is protected.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, TypedDict, cast, overload

from scripts import host_admission as host
from scripts import native_operation as operation
from scripts import resource_store
from scripts.resource_store import StoreError as ResourceError

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping, Sequence
    from io import BufferedReader

ROOT = Path(__file__).resolve().parents[1]
MARKER = "PSE_TEST_INVOCATION"
IDENTIFIER = re.compile(r"[a-f0-9]{32}")
DATABASE = re.compile(r"canonical_test_(?:[a-z0-9_]{1,48}_)?[a-f0-9]{32}")
NAMESPACE = re.compile(r"canonical_test_ns_[a-f0-9]{32}")
CLEANUP_TIMEOUT_SECONDS = 90


class ProcessIdentity(TypedDict):
    pid: int
    start: str
    boot: NotRequired[str]


class ResourceUnit(TypedDict, total=False):
    unit: str
    group: str
    invocation: str
    inode: int


class ReportArtifact(TypedDict):
    role: str
    digest: str
    identity: list[int]


class ReportManifest(TypedDict):
    version: int
    receipt_digest: str
    entries: dict[str, ReportArtifact]
    required_provenance: list[str]


class ResourceRecord(TypedDict, total=False):
    pid: int
    start: str
    boot: str
    invocation: str | None
    test: str | None
    state: str
    service_generation: str
    service_namespace: str
    namespace_creation: str
    credentials_sha256: str
    tool_inputs: dict[str, dict[str, object]]
    tool_rollouts: dict[str, dict[str, str]]
    rollout_probe: dict[str, object]
    maintenance_selection: dict[str, object]
    maintenance_revocation_probe: dict[str, object]
    namespace: str
    database: str
    kind: str
    directory: str | None
    units: list[ResourceUnit]
    drained: bool
    disposition: str
    pin: bool
    manual_pin: bool
    explicit_release: bool
    references: dict[str, str]
    borrows: dict[str, ProcessIdentity]
    cleanup: str
    cleaner: ProcessIdentity
    cleanup_error: str
    artifacts: dict[str, str]
    owner_root: str
    directory_identity: list[int]
    artifact_policy_version: int
    artifact_manifest: ReportManifest
    provenance_verified: bool
    creation_borrower: NotRequired[ProcessIdentity]
    terminal_identity: NotRequired[str]


class InvocationRecord(TypedDict):
    version: int
    nonce: str
    kind: str
    selected: list[dict[str, str]]
    terminal_owner: str
    collection_owner: NotRequired[ProcessIdentity]


def storage() -> resource_store.FileStore:
    return resource_store.FileStore(registry())


def resource_phase(record: ResourceRecord) -> str:
    settled = (
        record.get("drained") is True
        and record.get("pin") is False
        and record.get("references") == {}
        and record.get("borrows") == {}
        and not record.get("creation_borrower")
        and (record.get("disposition") == "pass" or record.get("explicit_release"))
    )
    if settled and record.get("cleanup") == "removed":
        return "disposed"
    if (
        settled
        and record.get("cleanup") == "compacted"
        and record.get("kind") == "evidence"
    ):
        return "cold"
    return "active"


def validate_resource(record: object) -> ResourceRecord:
    if not isinstance(record, dict):
        raise ResourceError("Invalid resource authority")
    if (
        any(
            not isinstance(record.get(key), str)
            for key in ("state", "database", "kind", "cleanup", "disposition")
        )
        or type(record.get("pin")) is not bool
        or type(record.get("drained")) is not bool
        or record["kind"] not in {"database", "namespace", "controls", "evidence"}
        or record["cleanup"] not in {"retained", "removing", "removed", "compacted"}
        or record["disposition"] not in {"incomplete", "pass", "failure"}
        or not isinstance(record.get("references"), dict)
        or not isinstance(record.get("borrows"), dict)
        or not isinstance(record.get("units"), list)
    ):
        raise ResourceError("Unknown resource ownership or disposition information")
    if record["kind"] == "namespace" and (
        not NAMESPACE.fullmatch(str(record.get("namespace", "")))
        or record["database"] != ""
        or not isinstance(record.get("service_namespace"), str)
    ):
        raise ResourceError("Invalid enclosing namespace ownership")
    for key in ("references", "borrows", "artifacts"):
        if key in record and not isinstance(record[key], dict):
            raise ResourceError("Unknown resource reference or artifact information")
    if "units" in record and not isinstance(record["units"], list):
        raise ResourceError("Invalid resource drain owners")
    return cast("ResourceRecord", record)


def entry_facts(entry: resource_store.Entry) -> ResourceRecord:
    record = validate_resource(entry.facts)
    if resource_identity(record) != entry.identity:
        raise ResourceError("Resource facts contradict logical identity")
    return record


def resource_identity(record: Mapping[str, object]) -> list[str]:
    scope = record["namespace"] if record["kind"] == "namespace" else record["database"]
    return [str(record["state"]), str(scope), str(record["kind"])]


@contextlib.contextmanager
def resource_update(
    resource: str, *, deadline: float | None = None
) -> Generator[ResourceRecord, None, None]:
    try:
        with storage().edit(resource, deadline=deadline) as entry:
            if entry.phase == "disposed":
                raise ResourceError("Unknown resource identity")
            record = entry_facts(entry)
            identity = resource_identity(record)
            if identity != entry.identity:
                raise ResourceError("Resource facts contradict logical identity")
            yield record
            if resource_identity(record) != identity:
                raise ResourceError("Resource update cannot replace logical identity")
            entry.phase = resource_phase(record)
    except FileNotFoundError as error:
        raise ResourceError("Unknown resource identity") from error


def cli_json(
    output: str, *, namespace: str | None = None, database: str | None = None
) -> object:
    """Parse one complete JSON value, stripping only the pinned CLI's exact prompt.

    SurrealDB 3.3 server/src/cli/sql.rs constructs `ns/db> `, `ns> ` or `> `.
    Unknown selections, diagnostics and trailing output remain parse failures.
    """
    if database is not None and namespace is None:
        raise ResourceError("CLI database prompt requires its namespace")
    for identifier in (namespace, database):
        if identifier is not None and not re.fullmatch(
            r"[A-Za-z0-9_]{1,128}", identifier
        ):
            raise ResourceError(
                "CLI prompt requires an exact bounded configured identifier"
            )
    prompt = (
        f"{namespace}/{database}> "
        if database is not None
        else f"{namespace}> "
        if namespace is not None
        else "> "
    )
    lines = []
    for line in output.splitlines():
        if line == prompt.rstrip():
            continue
        lines.append(line.removeprefix(prompt))
    return json.loads("\n".join(lines))


def registry() -> Path:
    selected = os.environ.get("PSE_TEST_RESOURCE_STATE")
    if selected:
        path = Path(selected)
        if not path.is_absolute():
            raise ResourceError(
                "PSE_TEST_RESOURCE_STATE must be an absolute private directory"
            )
        return path
    return host.root_path().parent / "test-resources"


def invocation(
    kind: str,
    selected: Sequence[Mapping[str, object]],
    *,
    terminal_owner: str = "runner",
) -> Path:
    host.protected(registry())
    directory = registry() / "invocations"
    host.protected(directory)
    nonce = uuid.uuid4().hex
    path = directory / f"{nonce}.json"
    declaration = {
        "version": 1,
        "nonce": nonce,
        "kind": kind,
        "selected": list(selected),
        "terminal_owner": terminal_owner,
    }
    storage().create_invocation(declaration)
    resource_store.publish(path, declaration)
    return path


def read_invocation(path: Path) -> InvocationRecord:
    if (
        path.is_symlink()
        or path.stat().st_uid != os.getuid()
        or path.stat().st_mode & 0o077
    ):
        raise ResourceError("Invocation must be a private owned file")
    record = json.loads(path.read_text())
    if (
        record.get("version") != 1
        or not IDENTIFIER.fullmatch(str(record.get("nonce", "")))
        or not isinstance(record.get("selected"), list)
    ):
        raise ResourceError("Invalid test invocation")
    selected = record["selected"]
    if any(
        not isinstance(item, dict)
        or any(
            not isinstance(key, str) or not isinstance(value, str)
            for key, value in item.items()
        )
        or "name" not in item
        for item in selected
    ):
        raise ResourceError("Invalid selected test identities")
    return cast("InvocationRecord", record)


def publish_selection(path: Path, selected: Sequence[Mapping[str, str]]) -> None:
    declaration = read_invocation(path)
    with storage().invocation_edit(declaration["nonce"]) as state:
        if state["phase"] != "open":
            raise ResourceError("Frozen invocation cannot change selection")
        current = state["declaration"]
        if current != declaration:
            raise ResourceError("Invocation selection differs from its authority")
        if current["selected"] and current["selected"] != list(selected):
            raise ResourceError("Runner inventory differs from selected invocation")
        declaration["selected"] = [dict(item) for item in selected]
        resource_store.publish(path, declaration)
        state["declaration"] = dict(declaration)


def publish_python_collection(path: Path, identities: Sequence[str]) -> bool:
    """Publish through the same invocation exclusion as registration and freeze."""
    record = read_invocation(path)
    with storage().invocation_edit(record["nonce"]) as state:
        if state["phase"] != "open" or record["kind"] != "python":
            raise ResourceError("Python collector requires an open Python invocation")
        if state["declaration"] != record:
            raise ResourceError("Invocation declaration differs from its authority")
        selected = [{"name": identity} for identity in identities]
        if len(set(identities)) != len(identities):
            raise ResourceError("Python collection contains duplicate identities")
        collector = {
            "pid": os.getpid(),
            "start": operation.start_identity(os.getpid()),
            "boot": host.boot(),
        }
        owner = record.get("collection_owner")
        if owner is None:
            if record["selected"] and record["selected"] != selected:
                raise ResourceError(
                    "Python collector differs from its selected invocation"
                )
            record.update(selected=selected, collection_owner=collector)
            resource_store.publish(path, record)
            state["declaration"] = dict(record)
            return True
        if owner == collector:
            if record["selected"] != selected:
                raise ResourceError("Python collector changed its frozen catalog")
            return True
        if not {item["name"] for item in selected}.issubset(
            {item["name"] for item in record["selected"]}
        ):
            raise ResourceError(
                "Nested Python collection is outside its owning invocation"
            )
        return False


def selected_test(
    payload: Mapping[str, object], *, declaration: InvocationRecord | None = None
) -> tuple[str | None, str | None]:
    path = os.environ.get(MARKER)
    if not path:
        return None, None
    record = declaration if declaration is not None else read_invocation(Path(path))
    test = str(payload.get("test", ""))
    if record["kind"] == "rust":
        parent = os.getppid()
        observed = Path(f"/proc/{parent}/exe").resolve(strict=True)
        args = Path(f"/proc/{parent}/cmdline").read_bytes().split(b"\0")
        if (
            b"--exact" not in args
            or test.encode() not in args
            or observed != Path(str(payload.get("executable", ""))).resolve(strict=True)
        ):
            raise ResourceError(
                "Rust fixture association requires actual nextest executable and exact test argument"
            )
        matches = [
            item
            for item in record["selected"]
            if item.get("name") == test
            and Path(item.get("executable", "")).resolve() == observed
        ]
        if len(matches) != 1:
            raise ResourceError("Rust fixture identity not uniquely selected")
        test = matches[0]["binary_id"] + "::" + test
    else:
        matches = [item for item in record["selected"] if item.get("name") == test]
        if len(matches) != 1:
            raise ResourceError("Python fixture identity not uniquely collected")
    return record["nonce"], test


def register(payload: Mapping[str, object], *, deadline: float | None = None) -> str:
    deadline = resource_store.clock(deadline)
    if os.environ.get("PSE_TEST_EXECUTION_EFFECTS") == "native-local":
        raise ResourceError("Canonical fixture requires --execution-effects canonical")
    marker = os.environ.get(MARKER)
    declaration = read_invocation(Path(marker)) if marker else None
    nonce, test = selected_test(payload, declaration=declaration)
    state = Path(str(payload["state"])).resolve(strict=True)
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    config = surreal_server.config_for(state)
    kind = str(payload.get("kind", "database"))
    database = str(payload.get("database", ""))
    namespace_selection = str(payload.get("namespace", ""))
    if kind == "namespace":
        if database or not NAMESPACE.fullmatch(namespace_selection):
            raise ResourceError("Namespace fixture requires a fresh enclosing identity")
    elif not DATABASE.fullmatch(database):
        raise ResourceError(
            "Disposable database must be a freshly generated opaque fixture identity"
        )
    if kind not in {"database", "namespace", "controls", "evidence"}:
        raise ResourceError("Unknown retention class")
    control = payload.get("directory")
    if control is not None:
        path = Path(str(control))
        if path.exists() or not path.is_absolute() or state not in path.parents:
            raise ResourceError(
                "Disposable controls must be registered before creating their owned directory"
            )
    units = payload.get("units", [])
    if not isinstance(units, list):
        raise ResourceError("Invalid drain owners")
    if config.get("profile_version") == 2:
        if config.get("parked"):
            allocation = host.inherit(os.environ)
            if allocation is None or not allocation.profile.exclusive:
                raise ResourceError(
                    "Parked storage demand requires its actual admitted owner"
                )
            surreal_server.unpark_service(state)
            config = surreal_server.config_for(state)
        elif config.get("admission") == "open" and not surreal_server.ready(
            state, config
        ):
            surreal_server.start(state, config)
            config = surreal_server.config_for(state)
    # Demand/startup IPC precedes publication. The short service admission gate
    # makes every borrower visible before a lifecycle reservation can begin.
    with surreal_server.context_admission(
        state, deadline=deadline, wait_for_lifecycle=False
    ):
        current = surreal_server.config_for(state)
        if current != config:
            raise ResourceError(
                "Service configuration changed during fixture admission"
            )
        if current.get("admission") != "open" or not current.get("accepting_writes"):
            raise ResourceError("Fixture context admission is closed")
        generation, namespace = current["instance_id"], current["namespace"]
        if not isinstance(generation, str) or not isinstance(namespace, str):
            raise ResourceError("Invalid service generation or namespace")
        facts = {
            "pid": os.getppid() if payload.get("executable") else os.getpid(),
            "start": operation.start_identity(
                os.getppid() if payload.get("executable") else os.getpid()
            ),
            "boot": host.boot(),
            "invocation": nonce,
            "test": test,
            "state": str(state),
            "service_generation": generation,
            "namespace": namespace,
            "database": database,
            "kind": kind,
            "directory": str(control) if control else None,
            "units": units,
            "drained": False,
            "disposition": "incomplete",
            "pin": True,
            "references": {},
            "borrows": {},
            "cleanup": "retained",
        }
        if kind == "namespace":
            facts["service_namespace"] = namespace
            facts["namespace"] = namespace_selection
        facts["creation_borrower"] = {
            "pid": facts["pid"],
            "start": facts["start"],
            "boot": facts["boot"],
        }
        resource = storage().register(
            resource_identity(facts),
            facts,
            invocation=nonce,
            declaration=declaration,
            deadline=deadline,
        )
    profile = payload.get("execution_profile")
    if profile is not None:
        worker = payload.get("worker")
        surreal_server.register_context(
            state, database, str(profile), Path(str(worker)) if worker else None
        )
    return resource


def borrower_alive(record: Mapping[str, object]) -> bool:
    pid = record.get("pid")
    if type(pid) is not int:
        return False
    try:
        return (
            record.get("boot") == host.boot()
            and operation.start_identity(pid) == record["start"]
        )
    except (FileNotFoundError, KeyError):
        return False


def record_drain(
    resource: str,
    *,
    units: Sequence[Mapping[str, str | int]] = (),
    deadline: float | None = None,
) -> None:
    with resource_update(resource, deadline=deadline) as record:
        record["drained"] = True
        record.pop("creation_borrower", None)
        record["units"] = [
            cast("ResourceUnit", dict(unit)) for unit in units
        ] or record["units"]


def finalize(
    path: Path,
    outcomes: Mapping[str, str],
    *,
    complete: bool,
    terminal_owner: str,
    terminal: Mapping[str, object] | None = None,
    deadline: float | None = None,
    deadline_supplier: Callable[[], float | None] | None = None,
    on_frozen: Callable[[], None] | None = None,
) -> list[str]:
    def current_deadline() -> float | None:
        nonlocal deadline
        if deadline_supplier is not None:
            supplied = deadline_supplier()
            if supplied is not None:
                deadline = supplied if deadline is None else min(deadline, supplied)
        return deadline

    declaration = read_invocation(path)
    if declaration["terminal_owner"] != terminal_owner:
        raise ResourceError(
            "Only the selected terminal reconciler may finalize resources"
        )
    selected = {
        item["binary_id"] + "::" + item["name"]
        if declaration["kind"] == "rust"
        else item["name"]
        for item in declaration["selected"]
    }
    if set(outcomes) - selected or any(
        value not in {"pass", "failure", "incomplete"} for value in outcomes.values()
    ):
        raise ResourceError("Terminal outcomes differ from selected catalog")
    frozen = {
        "selection": resource_store.digest(declaration["selected"]),
        "terminal_owner": terminal_owner,
        "complete": complete,
        "outcomes": dict(outcomes),
        "evidence": dict(terminal or {}),
    }
    members = storage().freeze(
        declaration["nonce"],
        frozen,
        declaration=declaration,
        deadline=current_deadline(),
        **({"on_frozen": on_frozen} if on_frozen is not None else {}),
    )
    identity = resource_store.digest(frozen)
    eligible = []
    # Frozen membership is finite. Without an external absolute clock, each
    # exact metadata operation retains its own ordinary bounded wait. A supplied
    # or newly observed cancellation deadline applies to the entire remainder.
    for resource in members:
        with storage().edit(
            resource,
            deadline=current_deadline(),
            settle_if_unchanged=True,
        ) as entry:
            record = entry_facts(entry)
            if record.get("invocation") != declaration["nonce"]:
                raise ResourceError(
                    "Invocation membership contradicts resource association"
                )
            prior = record.get("terminal_identity")
            if prior is not None and prior != identity:
                raise ResourceError(
                    "Resource already projected another terminal identity"
                )
            disposition = (
                outcomes.get(record.get("test") or "", "incomplete")
                if complete
                else "incomplete"
            )
            if prior is not None and record["disposition"] != disposition:
                raise ResourceError("Resource disposition contradicts frozen terminal")
            if prior is None:
                record.update(
                    disposition=disposition,
                    pin=disposition != "pass" or bool(record.get("manual_pin")),
                    terminal_identity=identity,
                )
            entry.phase = resource_phase(record)
            if (
                entry.phase != "disposed"
                and record["disposition"] == "pass"
                and record["drained"]
                and not record.get("creation_borrower")
            ):
                eligible.append(resource)
    with storage().invocation_edit(
        declaration["nonce"], deadline=current_deadline()
    ) as state:
        state["projection"] = "complete"
    return eligible


def resume_projection(
    path: Path,
    *,
    terminal_owner: str,
    deadline: float | None = None,
    deadline_supplier: Callable[[], float | None] | None = None,
) -> list[str]:
    """Project the authentic frozen composition without collecting a newer report."""
    if deadline_supplier is not None:
        supplied = deadline_supplier()
        if supplied is not None:
            deadline = supplied if deadline is None else min(deadline, supplied)
    declaration = read_invocation(path)
    _identity, frozen = storage().terminal(declaration["nonce"], deadline=deadline)
    return finalize(
        path,
        frozen["outcomes"],
        complete=frozen["complete"],
        terminal_owner=terminal_owner,
        terminal=frozen["evidence"],
        deadline=deadline,
        deadline_supplier=deadline_supplier,
    )


@overload
def resource_status(
    resource: str, *, deadline: float | None = None
) -> ResourceRecord: ...


@overload
def resource_status(
    resource: None = None, *, deadline: float | None = None
) -> dict[str, ResourceRecord]: ...


def resource_status(
    resource: str | None = None, *, deadline: float | None = None
) -> ResourceRecord | dict[str, ResourceRecord]:
    deadline = resource_store.clock(deadline)
    store = storage()
    if resource is not None:
        try:
            entry = store.lookup(resource, deadline=deadline)
        except FileNotFoundError as error:
            raise ResourceError("Unknown resource identity") from error
        if entry.phase == "disposed":
            raise ResourceError("Unknown resource identity")
        return entry_facts(entry).copy()
    result = {}
    for resource in store.active_ids(deadline=deadline):
        entry = store.lookup(resource, deadline=deadline)
        if entry.phase == "active":
            result[resource] = entry_facts(entry).copy()
    return result


def pin(resource: str, retain: bool, *, deadline: float | None = None) -> None:
    with resource_update(resource, deadline=deadline) as record:
        if not isinstance(record, dict) or record["cleanup"] == "removing":
            raise ResourceError("Unknown resource or reclamation already owns it")
        record["pin"] = retain
        record["manual_pin"] = retain
        record["explicit_release"] = not retain


@contextlib.contextmanager
def borrow(
    resource: str, *, deadline: float | None = None
) -> Generator[str, None, None]:
    token = uuid.uuid4().hex
    with resource_update(resource, deadline=deadline) as record:
        if not isinstance(record, dict) or record["cleanup"] in {"removing", "removed"}:
            raise ResourceError(
                "Origin is unknown or unavailable for reference acquisition"
            )
        record["borrows"][token] = {
            "pid": os.getpid(),
            "start": operation.start_identity(os.getpid()),
        }
    try:
        yield token
    finally:
        # With no caller-wide deadline, releasing a long borrow is a new bounded
        # metadata operation, not reuse of admission's expired metadata clock.
        with resource_update(resource, deadline=deadline) as record:
            record["borrows"].pop(token, None)


def retain_reference(
    resource: str, receipt: str, digest: str, *, deadline: float | None = None
) -> None:
    if not re.fullmatch(r"[a-f0-9]{64}", digest):
        raise ResourceError("Reference requires original receipt digest")
    with resource_update(resource, deadline=deadline) as record:
        if not isinstance(record, dict) or record["cleanup"] in {"removing", "removed"}:
            raise ResourceError("Origin reference information is unavailable")
        record["references"][receipt] = digest


def release_reference(
    resource: str, receipt: str, digest: str, *, deadline: float | None = None
) -> None:
    """Explicitly retire one exact evidence dependency; pins/drain still apply."""
    with resource_update(resource, deadline=deadline) as record:
        if not isinstance(record, dict) or record["cleanup"] == "removing":
            raise ResourceError("Unknown origin or reclamation already owns it")
        if record["references"].get(receipt) != digest:
            raise ResourceError("Reference identity or original receipt digest changed")
        del record["references"][receipt]


def register_report(
    directory: Path, *, root: Path = ROOT, deadline: float | None = None
) -> str:
    """Register this new assessment's owned evidence, preserving unknown old reports."""
    deadline = resource_store.clock(deadline)
    selected = directory.absolute()
    if (
        not selected.is_relative_to(root.resolve() / "build")
        or ".." in selected.parts
        or any(path.is_symlink() for path in (selected, *selected.parents))
        or not selected.is_dir()
    ):
        raise ResourceError("Assessment evidence must be a new owned build directory")
    identity = selected.stat()
    facts = {
        "pid": os.getpid(),
        "start": operation.start_identity(os.getpid()),
        "boot": host.boot(),
        "invocation": None,
        "test": None,
        "state": str(selected),
        "kind": "evidence",
        "owner_root": str(root.resolve()),
        "directory_identity": [identity.st_dev, identity.st_ino],
        "database": "",
        "directory": None,
        "units": [],
        "drained": False,
        "disposition": "incomplete",
        "pin": True,
        "references": {},
        "borrows": {},
        "cleanup": "retained",
        "artifacts": {},
        "artifact_policy_version": 1,
        "provenance_verified": False,
    }
    return storage().register(
        [str(selected), "", "evidence"], facts, invocation=None, deadline=deadline
    )


def report_resource(directory: Path, *, deadline: float | None = None) -> str | None:
    return storage().identity_lookup(
        [str(directory.resolve()), "", "evidence"], deadline=deadline
    )


@contextlib.contextmanager
def borrow_report(
    directory: Path, *, deadline: float | None = None
) -> Generator[None, None, None]:
    """Protect a bounded display read without creating a retained dependency."""
    deadline = resource_store.clock(deadline)
    resource = report_resource(directory, deadline=deadline)
    if resource is None:
        yield  # Historical unknown reports cannot be reclaimed by this owner.
    else:
        with borrow(resource, deadline=deadline):
            yield


@contextlib.contextmanager
def reference_report(
    directory: Path,
    consumer: Path | None,
    *,
    deadline: float | None = None,
) -> Generator[None, None, None]:
    """Borrow a registered origin before reading; transfer protection before release."""
    deadline = resource_store.clock(deadline)
    resource = report_resource(directory, deadline=deadline)
    if resource is None:
        raise ResourceError(
            "Origin requires qualified ownership and a sealed receipt before reuse"
        )
    with borrow(resource, deadline=deadline):
        selected = resource_status(resource, deadline=deadline)
        manifest = report_manifest(selected)
        if selected.get("artifact_policy_version") != 1 or manifest is None:
            raise ResourceError("Origin lacks an unchanged sealed receipt")
        original_digest = file_digest(directory / "checks.json")
        if original_digest != manifest["receipt_digest"]:
            raise ResourceError("Origin lacks an unchanged sealed receipt")
        with report_directory(selected) as descriptor:
            verify_manifest_files(descriptor, manifest)
        yield
        if consumer is not None:
            digest = file_digest(directory / "checks.json")
            if digest != original_digest:
                raise ResourceError("Origin receipt changed during reference transfer")
            retain_reference(
                resource, str(consumer.resolve()), digest, deadline=deadline
            )


ARTIFACT_ROLES = frozenset({"receipt", "provenance", "evidence", "scratch", "unknown"})


def file_digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def report_manifest(record: ResourceRecord) -> ReportManifest | None:
    """Unknown or malformed manifests never supply disposal authority."""
    manifest = record.get("artifact_manifest")
    if (
        not isinstance(manifest, dict)
        or type(manifest.get("version")) is not int
        or manifest["version"] != 1
    ):
        return None
    if not isinstance(manifest.get("receipt_digest"), str) or not re.fullmatch(
        r"[a-f0-9]{64}", manifest["receipt_digest"]
    ):
        return None
    entries = manifest.get("entries")
    if not isinstance(entries, dict):
        return None
    required = manifest.get("required_provenance")
    if (
        not isinstance(required, list)
        or not required
        or any(not isinstance(name, str) for name in required)
    ):
        return None
    for relative, artifact in entries.items():
        if not isinstance(relative, str) or not isinstance(artifact, dict):
            return None
        if (
            not isinstance(artifact.get("role"), str)
            or artifact["role"] not in ARTIFACT_ROLES
            or not isinstance(artifact.get("digest"), str)
        ):
            return None
        if not re.fullmatch(r"[a-f0-9]{64}", artifact["digest"]):
            return None
        identity = artifact.get("identity")
        if (
            not isinstance(identity, list)
            or len(identity) != 3
            or any(type(item) is not int for item in identity)
        ):
            return None
        path = Path(relative)
        if (
            path.is_absolute()
            or not path.parts
            or any(part in {"..", "."} for part in path.parts)
        ):
            return None
    receipt = entries.get("checks.json")
    if (
        receipt is None
        or receipt["role"] != "receipt"
        or receipt["digest"] != manifest["receipt_digest"]
    ):
        return None
    return manifest


@contextlib.contextmanager
def report_directory(record: ResourceRecord) -> Generator[int, None, None]:
    """Keep exact directory descriptors without following any ancestor symlink."""
    directory = Path(record["state"])
    if (
        not directory.is_absolute()
        or ".." in directory.parts
        or not directory.is_relative_to(Path(record["owner_root"]) / "build")
    ):
        raise ResourceError("Evidence directory ownership changed")
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    try:
        with contextlib.ExitStack() as descriptors:
            current = os.open(directory.anchor, flags)
            descriptors.callback(os.close, current)
            for component in directory.parts[1:]:
                current = os.open(component, flags, dir_fd=current)
                descriptors.callback(os.close, current)
            identity = os.fstat(current)
            if (
                record.get("directory_identity") != [identity.st_dev, identity.st_ino]
                or identity.st_uid != os.getuid()
            ):
                raise ResourceError("Evidence directory ownership changed")
            yield current
    except OSError as error:
        raise ResourceError(
            "Evidence path ownership changed; cleanup refused"
        ) from error


@contextlib.contextmanager
def artifact_file(
    directory: int, relative: str
) -> Generator[tuple[int, str, BufferedReader], None, None]:
    path = Path(relative)
    if (
        path.is_absolute()
        or not path.parts
        or any(part in {"..", "."} for part in path.parts)
    ):
        raise ResourceError("Evidence artifact ownership changed")
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    with contextlib.ExitStack() as parents:
        parent = directory
        for component in path.parts[:-1]:
            parent = os.open(component, flags, dir_fd=parent)
            parents.callback(os.close, parent)
        descriptor = os.open(
            path.name,
            os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK,
            dir_fd=parent,
        )
        with os.fdopen(descriptor, "rb") as stream:
            yield parent, path.name, stream


def observe_artifact(directory: int, relative: str) -> ReportArtifact:
    with artifact_file(directory, relative) as (parent, name, stream):
        observed = os.fstat(stream.fileno())
        if not stat.S_ISREG(observed.st_mode) or observed.st_uid != os.getuid():
            raise ResourceError("Evidence artifact ownership changed")
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
        latest = os.stat(name, dir_fd=parent, follow_symlinks=False)
        if (
            observed.st_dev,
            observed.st_ino,
            observed.st_size,
            observed.st_mtime_ns,
        ) != (latest.st_dev, latest.st_ino, latest.st_size, latest.st_mtime_ns):
            raise ResourceError("Evidence artifact changed while sealing")
        return {
            "role": "unknown",
            "digest": digest,
            "identity": [observed.st_dev, observed.st_ino, observed.st_uid],
        }


def verify_manifest_files(directory: int, manifest: ReportManifest) -> None:
    """Verify provenance and retained bytes before reuse or the first cleanup effect."""
    for relative, expected in manifest["entries"].items():
        try:
            observed = observe_artifact(directory, relative)
        except FileNotFoundError:
            if expected["role"] == "scratch":
                continue  # A prior interrupted cleanup may have removed it.
            raise ResourceError("Retained report artifact is missing") from None
        if (
            observed["identity"] != expected["identity"]
            or observed["digest"] != expected["digest"]
        ):
            raise ResourceError(
                "Registered evidence artifact ownership changed; cleanup refused"
            )


def report_outcome(receipt: Mapping[str, object]) -> tuple[bool, bool]:
    """Keep execution outcome separate from proof sufficient for automatic disposal."""
    complete = receipt.get("complete") is True
    checks = receipt.get("checks")
    scope = receipt.get("scope")
    qualified = False
    if isinstance(checks, list) and isinstance(scope, list) and scope:
        from scripts.validation_receipts import (  # noqa: PLC0415 -- reciprocal receipt/resource owner
            qualified as check_qualified,
        )

        qualified = (
            all(
                isinstance(check, dict)
                and isinstance(check.get("status"), str)
                and isinstance(check.get("gate"), str)
                and check_qualified(check)
                for check in checks
            )
            and all(
                isinstance(declaration, dict)
                and isinstance(declaration.get("name"), str)
                for declaration in scope
            )
            and {check.get("gate") for check in checks}
            == {declaration["name"] for declaration in scope}
            and len(checks) == len(scope)
            and len({check["gate"] for check in checks}) == len(checks)
        )
    passed = (
        receipt.get("version") == 5
        and complete
        and receipt.get("required_checks_covered") is True
        and qualified
    )
    provenance = (
        receipt.get("input_coverage") is True
        and receipt.get("source_unchanged") is True
        and receipt.get("provenance_errors") == []
    )
    return passed, provenance


def finish_report(
    resource: str,
    receipt: Mapping[str, object],
    *,
    roles: Mapping[str, str],
    required_provenance: Sequence[str],
    deadline: float | None = None,
) -> None:
    """Seal producer-declared meaning once; no suffix or filename disposal policy."""
    deadline = resource_store.clock(deadline)
    selected = resource_status(resource, deadline=deadline)
    directory = Path(selected["state"])
    if (
        selected.get("artifact_policy_version") != 1
        or "artifact_manifest" in selected
        or selected["cleanup"] != "retained"
    ):
        raise ResourceError("Report manifest is unavailable or already sealed")
    if roles.get("checks.json") != "receipt" or any(
        not isinstance(name, str)
        or not isinstance(role, str)
        or role not in ARTIFACT_ROLES
        for name, role in roles.items()
    ):
        raise ResourceError("Producer must declare valid roles and the checks receipt")
    if json.loads((directory / "checks.json").read_text()) != json.loads(
        json.dumps(receipt)
    ):
        raise ResourceError("Finalized receipt differs from the outcome being sealed")
    checks = receipt.get("checks", [])
    if not isinstance(checks, list) or any(
        not isinstance(check, dict) for check in checks
    ):
        raise ResourceError("Invalid report check records; evidence remains pinned")
    for check in checks:
        if check.get("origin") and Path(check["origin"]).resolve() != directory:
            continue
        for name in (check.get("log"), *check.get("artifacts", {})):
            if name is not None and roles.get(name) == "scratch":
                raise ResourceError(
                    "Receipt-consumed evidence cannot be disposable scratch"
                )
    entries: dict[str, ReportArtifact] = {}
    with report_directory(selected) as descriptor:
        for path in directory.rglob("*"):
            if path.is_symlink():
                raise ResourceError(
                    "New report contains a symlink; evidence remains pinned"
                )
            if path.is_file():
                relative = str(path.relative_to(directory))
                entry = observe_artifact(descriptor, relative)
                entry["role"] = roles.get(relative, "unknown")
                entries[relative] = entry
    if set(roles) - entries.keys():
        raise ResourceError("Declared producer artifact is missing")
    manifest: ReportManifest = {
        "version": 1,
        "receipt_digest": entries["checks.json"]["digest"],
        "entries": entries,
        "required_provenance": list(required_provenance),
    }
    artifacts = {
        name: entry["digest"]
        for name, entry in entries.items()
        if entry["role"] == "scratch"
    }
    complete = receipt.get("complete") is True
    passed, provenance = report_outcome(receipt)
    provenance = (
        provenance
        and bool(required_provenance)
        and all(
            name in entries and entries[name]["role"] == "provenance"
            for name in required_provenance
        )
    )
    units: list[ResourceUnit] = []
    allocation = host.inherit(os.environ)
    if allocation is not None:
        with host.allocation_metadata(
            allocation.directory, deadline=deadline
        ) as owners:
            units = [
                {"unit": name, **bound}
                for name, bound in owners["owners"][allocation.nonce]["units"].items()
            ]
    with resource_update(resource, deadline=deadline) as record:
        if "artifact_manifest" in record or record["cleanup"] != "retained":
            raise ResourceError("Report manifest was concurrently sealed or reserved")
        record.update(
            drained=complete,
            units=units,
            disposition="pass" if passed else "failure" if complete else "incomplete",
            pin=not passed or not provenance or bool(record.get("manual_pin")),
            artifacts=artifacts,
            artifact_manifest=manifest,
            provenance_verified=provenance,
        )


def reclaim_reports() -> list[str]:
    """Explicit bounded retry of eligible reports and deferred database cleanup."""
    deadline = resource_store.clock()
    store = storage()
    store.ensure(deadline)
    cursor_path = registry() / "cleanup-cursor.json"
    with store.lock("sweeps", "cleanup", deadline):
        cursor = (
            resource_store.read(cursor_path).get("after", "")
            if cursor_path.exists()
            else ""
        )
        candidates = []
        visited = store.active_ids(limit=64, after=str(cursor), deadline=deadline)
        for resource in visited:
            record = resource_status(resource, deadline=deadline)
            if (
                record.get("disposition") == "pass"
                and not record.get("pin", True)
                and not record.get("references")
                and not record.get("borrows")
            ):
                candidates.append(resource)
                if len(candidates) == 4:
                    break
        if candidates:
            resource_store.publish(cursor_path, {"after": candidates[-1]})
        elif visited:
            resource_store.publish(cursor_path, {"after": visited[-1]})
    errors = []
    for resource in candidates:
        try:
            record = resource_status(resource, deadline=deadline)
            if (
                record.get("disposition") != "pass"
                or record.get("pin", True)
                or record.get("references")
                or record.get("borrows")
            ):
                continue
            reclaim(resource, deadline=deadline)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            errors.append(f"{resource}: {error}")
    return errors


def remove_report(record: ResourceRecord, *, deadline: float | None = None) -> None:
    """Remove only sealed scratch, after checking all retained provenance and evidence."""
    manifest = report_manifest(record)
    if manifest is None:
        raise ResourceError("Missing producer artifact manifest")
    with report_directory(record) as descriptor:
        # Validate the whole sealed snapshot before the first effect. Missing scratch
        # is allowed only for resuming an earlier interrupted cleanup.
        verify_manifest_files(descriptor, manifest)
        for relative, expected in manifest["entries"].items():
            if deadline is not None:
                host.remaining(deadline, 1)
            if expected["role"] != "scratch":
                continue
            try:
                with artifact_file(descriptor, relative) as (parent, name, stream):
                    observed = os.fstat(stream.fileno())
                    if [observed.st_dev, observed.st_ino, observed.st_uid] != expected[
                        "identity"
                    ] or not stat.S_ISREG(observed.st_mode):
                        raise ResourceError("Evidence artifact ownership changed")
                    if (
                        hashlib.file_digest(stream, "sha256").hexdigest()
                        != expected["digest"]
                    ):
                        raise ResourceError(
                            "Registered evidence artifact changed; cleanup refused"
                        )
                    latest = os.stat(name, dir_fd=parent, follow_symlinks=False)
                    if (observed.st_dev, observed.st_ino) != (
                        latest.st_dev,
                        latest.st_ino,
                    ):
                        raise ResourceError("Evidence artifact ownership changed")
                    os.unlink(name, dir_fd=parent)
            except FileNotFoundError:
                continue


def eligible(
    record: ResourceRecord, *, check_units: bool = True, deadline: float | None = None
) -> str | None:
    if record.get("creation_borrower"):
        return "fixture borrowers not drained"
    if record["pin"]:
        return "retention pin"
    if record["disposition"] != "pass" and not record.get("explicit_release"):
        return "missing reconciled pass or explicit release"
    if not record["drained"]:
        return "fixture borrowers not drained"
    if record["kind"] not in {"database", "namespace", "controls", "evidence"}:
        return "unknown retention owner"
    if not isinstance(record.get("references"), dict) or not isinstance(
        record.get("borrows"), dict
    ):
        return "unknown reference eligibility"
    if record["references"] or record["borrows"]:
        return "protected reference or active borrow"
    if check_units:
        for owner in record["units"]:
            observed = operation.drained(
                {"scope": owner},
                **({"deadline": deadline} if deadline is not None else {}),
            )
            if not observed.complete:
                return "supervised descendant not drained: " + observed.diagnostic()
    if record["kind"] == "evidence" and "owner_root" in record:
        manifest = report_manifest(record)
        if manifest is None:
            return "missing producer artifact manifest"
        if record.get("provenance_verified") is not True:
            return "unverified report provenance"
        scratch = {
            name: item["digest"]
            for name, item in manifest["entries"].items()
            if item["role"] == "scratch"
        }
        if scratch != record.get("artifacts"):
            return "changed disposable artifact projection"
        if not scratch:
            return "no disposable report artifacts"
    return None


def remove_owned(record: ResourceRecord, *, deadline: float | None = None) -> None:
    if record["kind"] == "evidence":
        remove_report(record, deadline=deadline)
        return
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    state = Path(record["state"])
    config = surreal_server.config_for(
        state, **({"deadline": deadline} if deadline is not None else {})
    )
    if record["kind"] == "namespace":
        if (
            config["instance_id"] != record["service_generation"]
            or config["namespace"] != record.get("service_namespace")
            or not NAMESPACE.fullmatch(record["namespace"])
        ):
            raise ResourceError("Service or enclosing namespace ownership changed")
        remove_namespace(record, config, deadline=deadline)
        directory = record.get("directory")
        if directory:
            path = Path(directory)
            if path.is_symlink() or state not in path.parents:
                raise ResourceError("Unsafe namespace control-directory ownership")
            if path.exists():
                shutil.rmtree(path)
        return
    if (
        config["instance_id"] != record["service_generation"]
        or config["namespace"] != record["namespace"]
        or not DATABASE.fullmatch(record["database"])
    ):
        raise ResourceError(
            "Service generation or disposable database ownership changed"
        )
    if record["kind"] == "database":
        remove_database(
            record, config, **({"deadline": deadline} if deadline is not None else {})
        )
    directory = record.get("directory")
    if directory:
        path = Path(directory)
        if path.is_symlink() or state not in path.parents:
            raise ResourceError("Unsafe control-directory ownership")
        if path.exists():
            shutil.rmtree(path)
    if record["kind"] != "database":
        return
    context = state / ".contexts" / f"{record['database']}.json"
    if context.is_symlink():
        raise ResourceError("Unsafe context descriptor")
    context.unlink(missing_ok=True)


def namespace_sql(
    config: Mapping[str, object], sql: str, *, namespace: str | None = None, database: str | None = None, deadline: float | None = None
) -> list[object]:
    """Checked root administration for registered enclosing fixture namespaces."""
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    credentials = surreal_server.read_json(Path(str(config["credentials_file"])))
    environment = {
        key: value for key, value in os.environ.items()
        if not key.startswith(("SURREAL", "DATABASE_"))
    }
    environment.update(SURREAL_USER=str(credentials["username"]), SURREAL_PASS=str(credentials["password"]))
    server = config["server"]
    if not isinstance(server, dict):
        raise ResourceError("Invalid namespace administration binary")
    # The pinned CLI unconditionally saves SQL to history.txt. Keep credential
    # provisioning inputs private and ephemeral, away from the checkout/history.
    with tempfile.TemporaryDirectory(dir=Path(str(config["credentials_file"])).parent) as working:
        result = subprocess.run(
            [str(server["binary"]), "sql", "--endpoint", str(config["endpoint"]), "--json", "--hide-welcome", *(["--namespace", namespace] if namespace else []), *(["--database", database] if database else [])],
            input=sql, env=environment, cwd=working, capture_output=True, text=True, check=False,
            timeout=CLEANUP_TIMEOUT_SECONDS if deadline is None else host.remaining(deadline, CLEANUP_TIMEOUT_SECONDS),
        )
    try:
        values = cli_json(result.stdout, namespace=namespace, database=database)
    except ValueError as error:
        raise ResourceError("Namespace operation acknowledgment unavailable") from error
    if result.returncode or result.stderr.strip() or not isinstance(values, list):
        raise ResourceError("Namespace operation failed; ownership remains protective")
    return values


def remove_namespace(
    record: ResourceRecord, config: Mapping[str, object], *, deadline: float | None = None
) -> None:
    """Reconcile exact children, then remove only the registered enclosing scope."""
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    namespace = surreal_server.sql_identifier(record["namespace"])
    root_catalog = namespace_sql(config, "INFO FOR ROOT;", deadline=deadline)
    if len(root_catalog) != 1 or not isinstance(root_catalog[0], dict) or not isinstance(root_catalog[0].get("namespaces"), dict):
        raise ResourceError("Owned namespace existence inventory unavailable")
    if record["namespace"] not in root_catalog[0]["namespaces"]:
        return  # Exact root inventory proves an interrupted removal already completed.
    catalog = namespace_sql(config, "INFO FOR NS;", namespace=record["namespace"], deadline=deadline)
    if len(catalog) != 1 or not isinstance(catalog[0], dict):
        raise ResourceError("Owned namespace child inventory unavailable")
    children = catalog[0].get("databases")
    if not isinstance(children, dict) or not all(isinstance(name, str) for name in children):
        raise ResourceError("Owned namespace child inventory malformed")
    statements = [f"REMOVE DATABASE IF EXISTS {surreal_server.sql_identifier(name)};" for name in sorted(children)]
    if statements and namespace_sql(config, " ".join(statements), namespace=record["namespace"], deadline=deadline) != [None] * len(statements):
        raise ResourceError("Owned namespace child cleanup unproved; resource remains charged")
    # The pinned SQL shell reselects its session namespace after every query.
    # Remove the enclosing namespace from an unselected root session, otherwise
    # that reselect silently recreates the just-removed namespace.
    if namespace_sql(config, f"REMOVE NAMESPACE IF EXISTS {namespace};", deadline=deadline) != [None]:
        raise ResourceError("Owned namespace cleanup unproved; resource remains charged")
    after = namespace_sql(config, "INFO FOR ROOT;", deadline=deadline)
    if len(after) != 1 or not isinstance(after[0], dict) or not isinstance(after[0].get("namespaces"), dict) or record["namespace"] in after[0]["namespaces"]:
        raise ResourceError("Owned namespace absence unproved; resource remains charged")


def remove_database(
    record: Mapping[str, object],
    config: Mapping[str, object],
    *,
    deadline: float | None = None,
) -> None:
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    credentials = surreal_server.read_json(
        Path(str(config["credentials_file"])),
        **({"deadline": deadline} if deadline is not None else {}),
    )
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("SURREAL_")
    }
    environment.update(
        SURREAL_USER=str(credentials["username"]),
        SURREAL_PASS=str(credentials["password"]),
    )
    server = config["server"]
    if not isinstance(server, dict):
        raise ResourceError("Invalid server binary configuration")
    command = [
        str(server["binary"]),
        "sql",
        "--endpoint",
        str(config["endpoint"]),
        "--namespace",
        str(config["namespace"]),
        "--json",
        "--hide-welcome",
    ]
    result = subprocess.run(
        command,
        input=f"REMOVE DATABASE IF EXISTS {record['database']};\n",
        cwd=str(record["state"]),
        env=environment,
        capture_output=True,
        text=True,
        check=False,
        timeout=(
            CLEANUP_TIMEOUT_SECONDS
            if deadline is None
            else host.remaining(deadline, CLEANUP_TIMEOUT_SECONDS)
        ),
    )
    try:
        completed = cli_json(result.stdout, namespace=str(config["namespace"])) == [
            None
        ]
    except ValueError:
        completed = False
    if result.returncode or result.stderr.strip() or not completed:
        raise ResourceError("Owned database cleanup failed; resource remains pinned")


def cleanup_reason(
    record: ResourceRecord, *, check_units: bool = True, deadline: float | None = None
) -> str | None:
    if record["cleanup"] in {"removed", "compacted"}:
        return str(record["cleanup"])
    if record["cleanup"] == "removing":
        cleaner = record.get("cleaner")
        if not isinstance(cleaner, dict):
            return "unknown cleanup owner"
        if borrower_alive(cleaner):
            return "active cleanup owner"
    return eligible(record, check_units=check_units, deadline=deadline)


def reclaim(resource: str, *, deadline: float | None = None) -> str:
    bounded = deadline is not None
    # The existing removal allowance covers its claim, effect and settlement.
    # Preserve a supplied original clock rather than renewing it after removal.
    deadline = (
        time.monotonic() + CLEANUP_TIMEOUT_SECONDS if deadline is None else deadline
    )
    if not IDENTIFIER.fullmatch(resource):
        raise ResourceError("Release/reclaim selects an exact resource identity")
    try:
        entry = storage().lookup(resource, deadline=deadline)
    except FileNotFoundError as error:
        raise ResourceError(
            "Missing resource authority; disposal remains unproved"
        ) from error
    if entry.phase == "disposed":
        # The stable resource authority, rather than absence, proves disposal.
        return "removed"
    snapshot = entry_facts(entry).copy()
    reason = cleanup_reason(snapshot, deadline=deadline)
    if reason == "no disposable report artifacts":
        with resource_update(resource, deadline=deadline) as record:
            if record is None or record.get("units") != snapshot.get("units"):
                raise ResourceError("Report drain ownership changed")
            current_reason = cleanup_reason(record, check_units=False)
            if current_reason != reason:
                return current_reason or "retained"
            # Retained producer evidence still has an exact reference owner, but
            # no cleanup work remains after authentic drain.
            record["cleanup"] = "compacted"
        return reason
    if reason:
        return reason
    if bounded and snapshot["kind"] != "evidence" and snapshot.get("directory"):
        return "directory cleanup deferred outside the bounded terminal path"
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    state = Path(snapshot["state"])
    evidence = snapshot["kind"] == "evidence"
    config = None if evidence else surreal_server.config_for(state, deadline=deadline)
    if config is not None and (
        config.get("admission") != "open" or not config.get("accepting_writes")
    ):
        # A test-owned service may stop before terminal outcome reconciliation.
        # Keep removal deferred until the same service admits cleanup again.
        return "service context admission is closed"
    admission = (
        contextlib.nullcontext()
        if evidence
        else surreal_server.context_admission(
            state, deadline=deadline, wait_for_lifecycle=False
        )
    )
    with admission:
        if not evidence:
            current = surreal_server.config_for(state, deadline=deadline)
            if (
                current != config
                or current["instance_id"] != snapshot["service_generation"]
                or current["namespace"] != snapshot.get("service_namespace", snapshot["namespace"])
            ):
                raise ResourceError(
                    "Service configuration or cleanup ownership changed during admission"
                )
            if current.get("admission") != "open" or not current.get(
                "accepting_writes"
            ):
                raise ResourceError("Cleanup context admission is closed")
        with resource_update(resource, deadline=deadline) as record:
            if not isinstance(record, dict):
                raise ResourceError("Unknown resource")
            reason = cleanup_reason(record, check_units=False)
            if reason:
                return reason
            # Physical unit drain was observed before entering service exclusion.
            # Refuse any changed ownership rather than doing drain IPC inside it.
            for key in (
                "state",
                "kind",
                "database",
                "service_generation",
                "namespace",
                "service_namespace",
                "units",
                "directory_identity",
                "owner_root",
                "artifact_manifest",
                "artifacts",
                "provenance_verified",
            ):
                if record.get(key) != snapshot.get(key):
                    raise ResourceError("Cleanup ownership changed during admission")
            record["cleanup"] = "removing"
            if bounded:
                record["pin"] = True
            record["cleaner"] = {
                "pid": os.getpid(),
                "start": operation.start_identity(os.getpid()),
                "boot": host.boot(),
            }
            selected = record.copy()
    try:
        remove_owned(selected, deadline=deadline)
    except Exception as error:
        with resource_update(resource, deadline=deadline) as record:
            record.update(cleanup="retained", pin=True, cleanup_error=str(error))
        raise
    with resource_update(resource, deadline=deadline) as record:
        record["cleanup"] = "compacted" if selected["kind"] == "evidence" else "removed"
        record["pin"] = False
    return "removed"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action",
        choices=(
            "register",
            "drain",
            "list",
            "status",
            "pin",
            "release",
            "release-reference",
            "reclaim",
            "migrate",
            "resume",
        ),
    )
    parser.add_argument("resource", nargs="?")
    parser.add_argument("--receipt")
    parser.add_argument("--digest")
    parser.add_argument(
        "--terminal-owner", choices=("runner", "standalone", "assessment")
    )
    args = parser.parse_args()
    if args.action == "migrate":
        if not os.environ.get("PSE_TEST_RESOURCE_STATE"):
            parser.error(
                "migration requires an explicitly selected PSE_TEST_RESOURCE_STATE; inspect and authorize its existing content first"
            )
        storage().migrate()
    elif args.action == "resume":
        if not args.resource or not args.terminal_owner:
            parser.error(
                "resume requires the exact invocation path and --terminal-owner"
            )
        print(
            json.dumps(
                resume_projection(
                    Path(args.resource), terminal_owner=args.terminal_owner
                )
            )
        )
    elif args.action in {"register", "drain"}:
        payload = json.load(sys.stdin)
        if args.action == "register":
            print(json.dumps({"resource": register(payload)}))
        else:
            resource = str(payload["resource"])
            record = resource_status(resource)
            state = Path(record["state"])
            units = payload.get("units", [])
            if (state / ".contexts" / f"{record['database']}.json").exists():
                from scripts import surreal_server  # noqa: PLC0415 -- owner cycle

                units = [
                    *units,
                    *surreal_server.drain_context(state, record["database"]),
                ]
            record_drain(resource, units=units)
            print(json.dumps({"drained": True}))
    elif args.action in {"list", "status"}:
        print(json.dumps(resource_status(args.resource), sort_keys=True))
    else:
        if not args.resource:
            parser.error("select an exact resource identity")
        if args.action == "pin":
            pin(args.resource, True)
        elif args.action == "release":
            pin(args.resource, False)
        elif args.action == "release-reference":
            if not args.receipt or not args.digest:
                parser.error(
                    "reference release requires its exact --receipt and --digest"
                )
            release_reference(args.resource, args.receipt, args.digest)
        else:
            print(reclaim(args.resource))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
