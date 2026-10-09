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
import fcntl
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, TypedDict, cast, overload

from scripts import host_admission as host
from scripts import native_operation as operation

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
MARKER = "PSE_TEST_INVOCATION"
IDENTIFIER = re.compile(r"[a-f0-9]{32}")
DATABASE = re.compile(r"canonical_test_(?:[a-z0-9_]{1,48}_)?[a-f0-9]{32}")


class ResourceError(ValueError):
    """Ownership, outcome or retention could not be established."""


class ProcessIdentity(TypedDict):
    pid: int
    start: str
    boot: NotRequired[str]


class ResourceUnit(TypedDict, total=False):
    unit: str
    group: str
    invocation: str
    inode: int


class ResourceRecord(TypedDict, total=False):
    pid: int
    start: str
    boot: str
    invocation: str | None
    test: str | None
    state: str
    service_generation: str
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


class ResourceLedger(TypedDict):
    version: int
    owners: dict[str, ResourceRecord]
    report_cleanup_cursor: NotRequired[str]


class InvocationRecord(TypedDict):
    version: int
    nonce: str
    kind: str
    selected: list[dict[str, str]]
    terminal_owner: str
    collection_owner: NotRequired[ProcessIdentity]


@contextlib.contextmanager
def resource_metadata() -> Generator[ResourceLedger, None, None]:
    """Narrow the private resource ledger, preserving shared metadata IO."""
    with host.metadata(registry()) as ledger:
        owners = ledger.get("owners")
        if not isinstance(owners, dict) or any(
            not isinstance(value, dict) for value in owners.values()
        ):
            raise ResourceError("Invalid resource ledger ownership records")
        for record in owners.values():
            for key in ("references", "borrows", "artifacts"):
                if key in record and not isinstance(record[key], dict):
                    raise ResourceError(
                        "Unknown resource reference or artifact information"
                    )
            if "units" in record and not isinstance(record["units"], list):
                raise ResourceError("Invalid resource drain owners")
        yield cast("ResourceLedger", ledger)


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
    operation.write_json(
        path,
        {
            "version": 1,
            "nonce": nonce,
            "kind": kind,
            "selected": list(selected),
            "terminal_owner": terminal_owner,
        },
    )
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


def publish_python_collection(path: Path, identities: Sequence[str]) -> bool:
    """Freeze one collector's catalog; nested collectors may only consume subsets."""
    descriptor = os.open(
        path.with_suffix(".collection.lock"),
        os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o600,
    )
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        record = read_invocation(path)
        if record["kind"] != "python":
            raise ResourceError("Python collector requires its Python invocation")
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
            operation.write_json(path, record)
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
    finally:
        os.close(descriptor)


def selected_test(payload: Mapping[str, object]) -> tuple[str | None, str | None]:
    path = os.environ.get(MARKER)
    if not path:
        return None, None
    record = read_invocation(Path(path))
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


def register(payload: Mapping[str, object]) -> str:
    nonce, test = selected_test(payload)
    state = Path(str(payload["state"])).resolve(strict=True)
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    config = surreal_server.config_for(state)
    database = str(payload["database"])
    if not DATABASE.fullmatch(database):
        raise ResourceError(
            "Disposable database must be a freshly generated opaque fixture identity"
        )
    kind = str(payload.get("kind", "database"))
    if kind not in {"database", "controls", "evidence"}:
        raise ResourceError("Unknown retention class")
    resource = uuid.uuid4().hex
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
    with surreal_server.context_admission(state):
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
        with resource_metadata() as ledger:
            for existing in ledger["owners"].values():
                if (
                    existing["state"] == str(state)
                    and existing["database"] == database
                    and existing["kind"] == kind
                ):
                    raise ResourceError("Fixture identity already registered")
            ledger["owners"][resource] = {
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
    resource: str, *, units: Sequence[Mapping[str, str | int]] = ()
) -> None:
    with resource_metadata() as ledger:
        record = ledger["owners"].get(resource)
        if not isinstance(record, dict):
            raise ResourceError("Unknown fixture resource")
        record["drained"] = True
        record["units"] = [
            cast("ResourceUnit", dict(unit)) for unit in units
        ] or record["units"]


def finalize(
    path: Path, outcomes: Mapping[str, str], *, complete: bool, terminal_owner: str
) -> list[str]:
    invocation_record = read_invocation(path)
    if invocation_record["terminal_owner"] != terminal_owner:
        raise ResourceError(
            "Only the selected terminal reconciler may finalize resources"
        )
    eligible = []
    with resource_metadata() as ledger:
        for resource, record in ledger["owners"].items():
            if record["invocation"] != invocation_record["nonce"]:
                continue
            disposition = (
                outcomes.get(record["test"] or "", "incomplete")
                if complete
                else "incomplete"
            )
            record["disposition"] = disposition
            record["pin"] = disposition != "pass" or bool(record.get("manual_pin"))
            if disposition == "pass" and record["drained"]:
                eligible.append(resource)
    return eligible


@overload
def resource_status(resource: str) -> ResourceRecord: ...


@overload
def resource_status(resource: None = None) -> dict[str, ResourceRecord]: ...


def resource_status(
    resource: str | None = None,
) -> ResourceRecord | dict[str, ResourceRecord]:
    with resource_metadata() as ledger:
        owners = ledger["owners"]
        if resource is not None:
            if resource not in owners:
                raise ResourceError("Unknown resource identity")
            return owners[resource].copy()
        return dict(owners)


def pin(resource: str, retain: bool) -> None:
    with resource_metadata() as ledger:
        record = ledger["owners"].get(resource)
        if not isinstance(record, dict) or record["cleanup"] == "removing":
            raise ResourceError("Unknown resource or reclamation already owns it")
        record["pin"] = retain
        record["manual_pin"] = retain
        record["explicit_release"] = not retain


@contextlib.contextmanager
def borrow(resource: str) -> Generator[str, None, None]:
    token = uuid.uuid4().hex
    with resource_metadata() as ledger:
        record = ledger["owners"].get(resource)
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
        with resource_metadata() as ledger:
            ledger["owners"][resource]["borrows"].pop(token, None)


def retain_reference(resource: str, receipt: str, digest: str) -> None:
    if not re.fullmatch(r"[a-f0-9]{64}", digest):
        raise ResourceError("Reference requires original receipt digest")
    with resource_metadata() as ledger:
        record = ledger["owners"].get(resource)
        if not isinstance(record, dict) or record["cleanup"] in {"removing", "removed"}:
            raise ResourceError("Origin reference information is unavailable")
        record["references"][receipt] = digest


def release_reference(resource: str, receipt: str, digest: str) -> None:
    """Explicitly retire one exact evidence dependency; pins/drain still apply."""
    with resource_metadata() as ledger:
        record = ledger["owners"].get(resource)
        if not isinstance(record, dict) or record["cleanup"] == "removing":
            raise ResourceError("Unknown origin or reclamation already owns it")
        if record["references"].get(receipt) != digest:
            raise ResourceError("Reference identity or original receipt digest changed")
        del record["references"][receipt]


def register_report(directory: Path, *, root: Path = ROOT) -> str:
    """Register this new assessment's owned evidence, preserving unknown old reports."""
    selected = directory.resolve(strict=True)
    if not selected.is_relative_to(root.resolve() / "build") or selected.is_symlink():
        raise ResourceError("Assessment evidence must be a new owned build directory")
    resource = uuid.uuid4().hex
    with resource_metadata() as ledger:
        if any(
            record.get("state") == str(selected) for record in ledger["owners"].values()
        ):
            raise ResourceError("Assessment evidence already has an owner")
        identity = selected.stat()
        ledger["owners"][resource] = {
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
        }
    return resource


def report_resource(directory: Path) -> str | None:
    selected = str(directory.resolve())
    with resource_metadata() as ledger:
        matches = [
            resource
            for resource, record in ledger["owners"].items()
            if record.get("kind") == "evidence" and record["state"] == selected
        ]
    if len(matches) > 1:
        raise ResourceError("Ambiguous report ownership")
    return matches[0] if matches else None


@contextlib.contextmanager
def reference_report(
    directory: Path, consumer: Path | None
) -> Generator[None, None, None]:
    """Borrow a registered origin before reading; transfer protection before release."""
    resource = report_resource(directory)
    if resource is None:
        # Historical/unknown reports are never cleanup candidates in this owner.
        yield
        return
    with borrow(resource):
        yield
        if consumer is not None:
            digest = (
                __import__("hashlib")
                .sha256((directory / "checks.json").read_bytes())
                .hexdigest()
            )
            retain_reference(resource, str(consumer.resolve()), digest)


def finish_report(resource: str, receipt: Mapping[str, object]) -> None:
    selected = resource_status(resource)
    directory = Path(selected["state"])
    compact = {"checks.json", "scope.json", "summary.txt"}
    artifacts = {}
    for path in directory.rglob("*"):
        if path.is_symlink():
            raise ResourceError(
                "New report contains a symlink; evidence remains pinned"
            )
        if (
            path.is_file()
            and path.name not in compact
            and path.suffix != ".json"
            and "selected" not in path.name
        ):
            artifacts[str(path.relative_to(directory))] = (
                __import__("hashlib").sha256(path.read_bytes()).hexdigest()
            )
    complete = bool(receipt.get("complete"))
    passed = complete and bool(receipt.get("required_checks_covered"))
    units: list[ResourceUnit] = []
    allocation = host.inherit(os.environ)
    if allocation is not None:
        with host.allocation_metadata(allocation.directory) as owners:
            units = [
                {"unit": name, **bound}
                for name, bound in owners["owners"][allocation.nonce]["units"].items()
            ]
    with resource_metadata() as ledger:
        ledger["owners"][resource].update(
            drained=complete,
            units=units,
            disposition="pass" if passed else "failure" if complete else "incomplete",
            pin=not passed or bool(ledger["owners"][resource].get("manual_pin")),
            artifacts=artifacts,
        )


def reclaim_reports() -> list[str]:
    """Bounded opportunities to compact new successful, actually drained reports."""
    deadline = __import__("time").monotonic() + 5
    with resource_metadata() as ledger:
        available = sorted(
            resource
            for resource, record in ledger["owners"].items()
            if record.get("kind") == "evidence"
            and record.get("disposition") == "pass"
            and not record.get("pin", True)
            and record.get("cleanup") == "retained"
            and record.get("references") == {}
            and record.get("borrows") == {}
        )
        cursor = ledger.get("report_cleanup_cursor", "")
        candidates = (
            [resource for resource in available if resource > cursor]
            + [resource for resource in available if resource <= cursor]
        )[:4]
        if candidates:
            ledger["report_cleanup_cursor"] = candidates[-1]
    errors = []
    for resource in candidates:
        if __import__("time").monotonic() >= deadline:
            break
        try:
            reclaim(resource)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            errors.append(f"{resource}: {error}")
    return errors


def remove_report(record: ResourceRecord) -> None:
    """Compact only the exact registered files after outcome, drain and references."""
    directory = Path(record["state"])
    if (
        not directory.is_absolute()
        or ".." in directory.parts
        or not directory.is_relative_to(Path(record["owner_root"]) / "build")
    ):
        raise ResourceError("Evidence directory ownership changed")

    # Walk and retain directory descriptors: an ancestor symlink must never
    # redirect a checked relative artifact to unrelated material.
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    with contextlib.ExitStack() as descriptors:
        current = os.open(directory.anchor, flags)
        descriptors.callback(os.close, current)
        try:
            for component in directory.parts[1:]:
                current = os.open(component, flags, dir_fd=current)
                descriptors.callback(os.close, current)
            identity = os.fstat(current)
            if (
                record.get("directory_identity") != [identity.st_dev, identity.st_ino]
                or identity.st_uid != os.getuid()
            ):
                raise ResourceError("Evidence directory ownership changed")
            for relative, expected in record["artifacts"].items():
                path = Path(relative)
                if (
                    path.is_absolute()
                    or not path.parts
                    or any(part in {"..", "."} for part in path.parts)
                ):
                    raise ResourceError("Evidence artifact ownership changed")
                with contextlib.ExitStack() as parents:
                    parent = current
                    try:
                        for component in path.parts[:-1]:
                            parent = os.open(component, flags, dir_fd=parent)
                            parents.callback(os.close, parent)
                        descriptor = os.open(
                            path.name,
                            os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK,
                            dir_fd=parent,
                        )
                    except FileNotFoundError:
                        continue  # An earlier interrupted cleanup may have removed it.
                    with os.fdopen(descriptor, "rb") as stream:
                        observed = os.fstat(stream.fileno())
                        if (
                            not stat.S_ISREG(observed.st_mode)
                            or observed.st_uid != os.getuid()
                        ):
                            raise ResourceError("Evidence artifact ownership changed")
                        if (
                            hashlib.file_digest(stream, "sha256").hexdigest()
                            != expected
                        ):
                            raise ResourceError(
                                "Registered evidence artifact changed; cleanup refused"
                            )
                        latest = os.stat(
                            path.name, dir_fd=parent, follow_symlinks=False
                        )
                        if (observed.st_dev, observed.st_ino) != (
                            latest.st_dev,
                            latest.st_ino,
                        ):
                            raise ResourceError("Evidence artifact ownership changed")
                        os.unlink(path.name, dir_fd=parent)
        except OSError as error:
            raise ResourceError(
                "Evidence path ownership changed; cleanup refused"
            ) from error


def eligible(record: ResourceRecord, *, check_units: bool = True) -> str | None:
    if record["pin"]:
        return "retention pin"
    if record["disposition"] != "pass" and not record.get("explicit_release"):
        return "missing reconciled pass or explicit release"
    if not record["drained"]:
        return "fixture borrowers not drained"
    if record["kind"] not in {"database", "controls", "evidence"}:
        return "unknown retention owner"
    if not isinstance(record.get("references"), dict) or not isinstance(
        record.get("borrows"), dict
    ):
        return "unknown reference eligibility"
    if record["references"] or record["borrows"]:
        return "protected reference or active borrow"
    if check_units:
        for owner in record["units"]:
            if not operation.drained({"scope": owner}):
                return "supervised descendant not drained"
    return None


def remove_owned(record: ResourceRecord) -> None:
    if record["kind"] == "evidence":
        remove_report(record)
        return
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    state = Path(record["state"])
    config = surreal_server.config_for(state)
    if (
        config["instance_id"] != record["service_generation"]
        or config["namespace"] != record["namespace"]
        or not DATABASE.fullmatch(record["database"])
    ):
        raise ResourceError(
            "Service generation or disposable database ownership changed"
        )
    if record["kind"] == "database":
        remove_database(record, config)
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


def remove_database(record: Mapping[str, object], config: Mapping[str, object]) -> None:
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    credentials = surreal_server.read_json(Path(str(config["credentials_file"])))
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
        timeout=90,
    )
    try:
        completed = cli_json(result.stdout, namespace=str(config["namespace"])) == [
            None
        ]
    except ValueError:
        completed = False
    if result.returncode or result.stderr.strip() or not completed:
        raise ResourceError("Owned database cleanup failed; resource remains pinned")


def cleanup_reason(record: ResourceRecord, *, check_units: bool = True) -> str | None:
    if record["cleanup"] in {"removed", "compacted"}:
        return str(record["cleanup"])
    if record["cleanup"] == "removing":
        cleaner = record.get("cleaner")
        if not isinstance(cleaner, dict):
            return "unknown cleanup owner"
        if borrower_alive(cleaner):
            return "active cleanup owner"
    return eligible(record, check_units=check_units)


def reclaim(resource: str) -> str:
    if not IDENTIFIER.fullmatch(resource):
        raise ResourceError("Release/reclaim selects an exact resource identity")
    snapshot = resource_status(resource)
    reason = cleanup_reason(snapshot)
    if reason:
        return reason
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    state = Path(snapshot["state"])
    evidence = snapshot["kind"] == "evidence"
    config = None if evidence else surreal_server.config_for(state)
    admission = (
        contextlib.nullcontext()
        if evidence
        else surreal_server.context_admission(state)
    )
    with admission:
        if not evidence:
            current = surreal_server.config_for(state)
            if (
                current != config
                or current["instance_id"] != snapshot["service_generation"]
                or current["namespace"] != snapshot["namespace"]
            ):
                raise ResourceError(
                    "Service configuration or cleanup ownership changed during admission"
                )
            if current.get("admission") != "open" or not current.get(
                "accepting_writes"
            ):
                raise ResourceError("Cleanup context admission is closed")
        with resource_metadata() as ledger:
            record = ledger["owners"].get(resource)
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
                "units",
            ):
                if record.get(key) != snapshot.get(key):
                    raise ResourceError("Cleanup ownership changed during admission")
            record["cleanup"] = "removing"
            record["cleaner"] = {
                "pid": os.getpid(),
                "start": operation.start_identity(os.getpid()),
                "boot": host.boot(),
            }
            selected = record.copy()
    try:
        remove_owned(selected)
    except Exception as error:
        with resource_metadata() as ledger:
            ledger["owners"][resource].update(
                cleanup="retained", pin=True, cleanup_error=str(error)
            )
        raise
    with resource_metadata() as ledger:
        ledger["owners"][resource]["cleanup"] = (
            "compacted" if selected["kind"] == "evidence" else "removed"
        )
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
        ),
    )
    parser.add_argument("resource", nargs="?")
    parser.add_argument("--receipt")
    parser.add_argument("--digest")
    args = parser.parse_args()
    if args.action in {"register", "drain"}:
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
