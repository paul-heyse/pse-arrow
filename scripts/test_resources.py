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
import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, TypedDict, cast, overload

from scripts import host_admission as host
from scripts import invocation_disposition, resource_store
from scripts import native_operation as operation
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


class ReportLocation(TypedDict):
    state: str
    owner_root: str
    directory_identity: list[int]


class ResourceRecord(TypedDict, total=False):
    pid: int
    start: str
    boot: str
    invocation: str | None
    test: str | None
    state: str
    service_generation: str
    service_owner: str
    unresolved_service: NotRequired[str]
    service_namespace: str
    namespace_creation: str
    credentials_sha256: str
    tool_inputs: dict[str, dict[str, object]]
    tool_rollouts: dict[str, dict[str, str]]
    tool_population: NotRequired[dict[str, object]]
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
    report_custody: ReportLocation
    custody_transition: dict
    custody_history: list[dict]


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
    resource: str,
    *,
    deadline: float | None = None,
    store: resource_store.FileStore | None = None,
    custody_transition: bool = False,
) -> Generator[ResourceRecord, None, None]:
    try:
        with (storage() if store is None else store).edit(
            resource, deadline=deadline
        ) as entry:
            if entry.phase == "disposed":
                raise ResourceError("Unknown resource identity")
            record = entry_facts(entry)
            if record.get("custody_transition") and not custody_transition:
                raise ResourceError("Report custody conversion is reserved; resume its owner")
            identity = resource_identity(record)
            acquired = {
                key: record.get(key)
                for key in ("service_owner", "service_generation", "unresolved_service")
            }
            if identity != entry.identity:
                raise ResourceError("Resource facts contradict logical identity")
            yield record
            if resource_identity(record) != identity:
                raise ResourceError("Resource update cannot replace logical identity")
            if any(record.get(key) != value for key, value in acquired.items()):
                raise ResourceError(
                    "Resource update cannot replace its acquired service binding"
                )
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


def service_registry_binding(
    owner: Path, *, establish: bool = False, deadline: float | None = None
) -> None:
    """Bind covered discovery to the actual service, including empty cohorts."""
    path = owner / ".resource-owner" / "registry.json"
    existing = resource_store.read(path) if path.exists() else None
    if existing is not None and existing.get("registry") != str(registry().resolve()):
        raise ResourceError(
            "Service resource registry coverage differs from selected authority"
        )
    if existing is None and not establish:
        raise ResourceError("Service resource registry coverage is unestablished")
    store = storage()
    store.ensure(resource_store.clock(deadline))
    identity = registry().resolve().stat()
    activation = resource_store.read(registry() / "activation.json")
    expected = {
        "version": 1,
        "registry": str(registry().resolve()),
        "identity": [identity.st_dev, identity.st_ino],
        "activation": activation,
    }
    if existing is not None:
        if existing != expected:
            raise ResourceError(
                "Service resource registry coverage differs from selected authority"
            )
    elif establish:
        resource_store.publish(path, expected)
    else:
        raise ResourceError("Service resource registry coverage is unestablished")


def service_resources(
    owner: Path, *, deadline: float | None = None
) -> dict[str, ResourceRecord]:
    deadline = resource_store.clock(deadline)
    service_registry_binding(owner, deadline=deadline)
    store = storage()
    for resource in store.access_ids(
        "uncovered-service-access/" + resource_store.digest(str(owner)),
        deadline=deadline,
    ):
        entry = store.lookup(resource, deadline=deadline)
        if entry.phase == "active" and entry.facts.get("unresolved_service") == str(
            owner
        ):
            raise ResourceError(
                "Original service association remains unresolved; quiet is unestablished"
            )
    result = {}
    for resource in store.access_ids(
        "service-access/" + resource_store.digest(str(owner)), deadline=deadline
    ):
        entry = store.lookup(resource, deadline=deadline)
        if entry.phase != "active":
            continue  # a protected interrupted departure can leave a stale marker
        record = entry_facts(entry)
        if record.get("service_owner") != str(owner):
            raise ResourceError("Service access projection contradicts its authority")
        result[resource] = record.copy()
    return result


def execution_owner(*, deadline: float | None = None) -> dict | None:
    """Observe native or ordinary command membership through the existing host binding."""
    selected = operation.scope_owner(**operation.deadline_options(deadline))
    if selected is not None:
        return dict(selected)
    allocation = host.inherit(os.environ, **operation.deadline_options(deadline))
    if allocation is None:
        return None
    snapshot = host.readonly_snapshot(
        allocation.directory, deadline=resource_store.clock(deadline)
    )
    record = snapshot["owners"].get(allocation.nonce)
    group = operation.process_group(os.getpid())
    if not isinstance(record, dict):
        return None
    for unit, binding in record["units"].items():
        control = binding.get("group")
        if isinstance(control, str) and (
            group == control or group.startswith(control + "/")
        ):
            observed = operation.unit_observation(
                unit, **operation.deadline_options(deadline)
            )
            if (
                observed.get("InvocationID") == binding.get("invocation")
                and observed.get("ControlGroup") == control
            ):
                return {
                    "unit": unit,
                    "group": control,
                    "invocation": binding["invocation"],
                }
    return None


def invocation(
    kind: str,
    selected: Sequence[Mapping[str, object]],
    *,
    terminal_owner: str = "runner",
    requirements: Mapping[str, Mapping[str, object]] | None = None,
) -> Path:
    host.protected(registry())
    directory = registry() / "invocations"
    host.protected(directory)
    nonce = uuid.uuid4().hex
    path = directory / f"{nonce}.json"
    declaration: dict[str, object] = {
        "version": 2,
        "nonce": nonce,
        "kind": kind,
        "selected": list(selected),
        "terminal_owner": terminal_owner,
    }
    if kind == "python" and selected:
        declaration["collection_owner"] = {
            "pid": os.getpid(),
            "start": operation.start_identity(os.getpid()),
            "boot": host.boot(),
        }
    storage().create_invocation(declaration)
    producer = {
        "pid": os.getpid(),
        "start": operation.start_identity(os.getpid()),
        "boot": host.boot(),
        "scope": execution_owner(),
        "allocation": os.environ.get(host.MARKER),
        "operation": str(operation.current()) if operation.current() else None,
    }
    if producer["allocation"] and producer["scope"] is not None:
        allocation_path = Path(producer["allocation"])
        owner = host.readonly_snapshot(
            allocation_path.parent, deadline=resource_store.clock()
        )["owners"].get(allocation_path.name)
        if owner is not None:
            binding = owner["units"].get(producer["scope"]["unit"], {})
            if (
                binding.get("group") == producer["scope"]["group"]
                and binding.get("invocation") == producer["scope"]["invocation"]
            ):
                producer["allocation_owner"] = {
                    key: owner[key] for key in ("pid", "start", "boot")
                }
    with storage().invocation_edit(nonce) as state:
        state["producer"] = producer
        # selected fixture owner; retain reciprocal lifecycle boundary
        from scripts import test_requirements  # noqa: PLC0415

        state["requirements"] = (
            dict(requirements)
            if requirements is not None
            else {
                (
                    str(item["binary_id"]) + "::" + str(item["name"])
                    if kind == "rust"
                    else str(item["name"])
                ): (
                    test_requirements.rust(str(item["binary_id"]), str(item["name"]))
                    if kind == "rust"
                    else test_requirements.LOCAL
                ).document()
                for item in selected
            }
        )
        # Supplied selected identities are already collected. Empty selections
        # stay explicitly staged until the runner/collector seals even an empty catalogue.
        state["collected"] = bool(selected)
    resource_store.publish(path, invocation_locator(declaration))
    return path


def invocation_locator(declaration: Mapping[str, object]) -> dict:
    return {
        key: declaration[key] for key in ("version", "nonce", "kind", "terminal_owner")
    }


def read_invocation(path: Path) -> InvocationRecord:
    if (
        path.is_symlink()
        or path.stat().st_uid != os.getuid()
        or path.stat().st_mode & 0o077
    ):
        raise ResourceError("Invocation must be a private owned file")
    record = json.loads(path.read_text())
    if record.get("version") != 2 or not IDENTIFIER.fullmatch(
        str(record.get("nonce", ""))
    ):
        raise ResourceError("Invalid test invocation")
    image = resource_store.FileStore(path.parent.parent).invocation_snapshot(
        record["nonce"]
    )
    declaration = image["declaration"]
    if invocation_locator(declaration) != record:
        raise ResourceError("Invocation locator differs from collected authority")
    record = declaration
    if not isinstance(record.get("selected"), list):
        raise ResourceError("Invalid selected test identities")
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
        if declaration["kind"] == "rust":
            # selected fixture owner; retain reciprocal lifecycle boundary
            from scripts import test_requirements  # noqa: PLC0415

            state["requirements"] = {
                item["binary_id"] + "::" + item["name"]: test_requirements.rust(
                    item["binary_id"], item["name"]
                ).document()
                for item in selected
            }
        state["declaration"] = dict(declaration)
        state["collected"] = True


def publish_python_requirements(
    path: Path, requirements: Mapping[str, Mapping[str, object]]
) -> None:
    """Actual fixture closure publishes demand before selected test execution."""
    record = read_invocation(path)
    store = storage()
    if path.absolute() != store.path("invocations", record["nonce"]).absolute():
        raise ResourceError(
            "Python requirements invocation differs from its selected registry"
        )
    with store.invocation_edit(record["nonce"]) as state:
        declaration = state["declaration"]
        if (
            state["phase"] != "open"
            or invocation_locator(declaration) != invocation_locator(record)
            or declaration["kind"] != "python"
        ):
            raise ResourceError("Requirements require the open collected invocation")
        # A controller can seal the catalogue after this worker authenticated
        # its locator but before it acquired this exclusion. Use the current
        # authoritative selection; late agreeing workers cannot enlarge it.
        if state.get("collected") and not set(requirements).issubset(
            item["name"] for item in declaration["selected"]
        ):
            raise ResourceError(
                "Python requirements are outside the collected invocation"
            )
        current = state.setdefault("requirements", {})
        for identity, requirement in requirements.items():
            if identity in current and current[identity] != requirement:
                raise ResourceError(
                    "Python collectors disagree about selected requirements"
                )
            current[identity] = dict(requirement)


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
        requirements = state.get("requirements", {})
        if any(identity not in requirements for identity in identities):
            raise ResourceError(
                "Python collection lacks actual requirement declarations"
            )
        # selected fixture owner; retain reciprocal lifecycle boundary
        from scripts import test_requirements  # noqa: PLC0415

        try:
            test_requirements.validate_roles(
                {identity: requirements[identity] for identity in identities},
                test_requirements.required_roles(os.environ),
            )
        except ValueError as error:
            raise ResourceError(str(error)) from error
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
            state["declaration"] = dict(record)
            state["requirements"] = {
                identity: requirements[identity] for identity in identities
            }
            state["collected"] = True
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


def require_effect(payload: Mapping[str, object], *, effect: str = "canonical") -> None:
    """Guard selected demand before a fixture or lifecycle owner performs effects."""
    marker = os.environ.get(MARKER)
    if not marker:
        return  # Bare native routes retain their documented narrower association.
    declaration = read_invocation(Path(marker))
    nonce, test = selected_test(payload, declaration=declaration)
    image = storage().invocation_snapshot(str(nonce))
    if image["phase"] != "open" or image["declaration"] != declaration:
        raise ResourceError("Invocation membership is closed; effects refused")
    # selected fixture owner; retain reciprocal lifecycle boundary
    from scripts import test_requirements  # noqa: PLC0415

    try:
        if (
            not image.get("collected")
            or image.get("requirement_policy") != test_requirements.POLICY_VERSION
        ):
            raise ResourceError(
                "Current collected requirement policy is unavailable; effects refused"
            )
        test_requirements.guard(image.get("requirements", {}), str(test), effect)
    except ValueError as error:
        raise ResourceError(str(error)) from error


def observe_setup_failure(payload: Mapping[str, object]) -> None:
    """Retain source-typed setup facts under authenticated selected membership."""
    marker = os.environ.get(MARKER)
    if not marker:
        raise ResourceError("Setup observation requires its managed invocation")
    declaration = read_invocation(Path(marker))
    nonce, test = selected_test(payload, declaration=declaration)
    stage, code = payload.get("stage"), payload.get("code")
    reach = payload.get("assertion_reach")
    if (
        payload.get("kind") != "infrastructure"
        or not isinstance(stage, str)
        or not re.fullmatch(r"[a-z0-9-]{1,128}", stage)
        or code != "runtime::infrastructure"
        or reach not in {"unknown", "not-reached"}
    ):
        raise ResourceError("Invalid source-typed setup failure")
    producer = os.getppid() if payload.get("executable") else os.getpid()
    observation = {
        "kind": "infrastructure",
        "stage": stage,
        "code": code,
        "assertion_reach": reach,
        "producer": {
            "pid": producer,
            "start": operation.start_identity(producer),
            "boot": host.boot(),
        },
    }
    with storage().invocation_edit(str(nonce)) as state:
        if state["phase"] != "open" or state["declaration"] != declaration:
            raise ResourceError("Frozen invocation cannot change setup observations")
        state.setdefault("setup_failures", {})[str(test)] = observation


def observe_peer(payload: Mapping[str, object], child: Path, *, role: str) -> None:
    """Join an actual frozen peer through this registry's immutable identity."""
    from scripts import (  # noqa: PLC0415 -- selected fixture owner; retain reciprocal lifecycle boundary
        run_inspection,
        test_requirements,
    )

    marker = os.environ.get(MARKER)
    if not marker:
        raise ResourceError("Peer observation requires its parent invocation")
    parent = read_invocation(Path(marker))
    nonce, test = selected_test(payload, declaration=parent)
    root, declaration, image = run_inspection.resolve(str(child))
    if (
        root != registry()
        or declaration["nonce"] == nonce
        or image["phase"] != "frozen"
    ):
        raise ResourceError(
            "Peer requires a distinct frozen invocation in the same registry"
        )
    terminal = image["terminal"]
    if (
        terminal.get("complete") is not True
        or terminal.get("evidence", {}).get("assurance") != "pass"
    ):
        raise ResourceError("Peer has no complete reconciled Pass")
    if image.get(
        "requirement_policy"
    ) != test_requirements.POLICY_VERSION or not image.get("collected"):
        raise ResourceError(
            "Historical peer requirement policy cannot supply a current role"
        )
    if declaration["kind"] == "rust":
        observed = {
            value
            for requirement in image.get("requirements", {}).values()
            for value in requirement.get("roles", [])
        }
        if role not in observed:
            raise ResourceError("Peer selected catalog lacks its required role")
    with storage().invocation_edit(str(nonce)) as state:
        if state["phase"] != "open" or state["declaration"] != parent:
            raise ResourceError("Frozen parent cannot acquire a peer observation")
        state.setdefault("peer_invocations", {})[str(test)] = {
            "invocation": declaration["nonce"],
            "terminal_identity": image["terminal_identity"],
            "role": role,
        }


def observe_cleanup(payload: Mapping[str, object], failures: Sequence[str]) -> None:
    """Keep driver cleanup separate from the test's frozen assurance."""
    marker = os.environ.get(MARKER)
    if not marker:
        raise ResourceError("Cleanup observation requires its parent invocation")
    declaration = read_invocation(Path(marker))
    nonce, test = selected_test(payload, declaration=declaration)
    if any(not re.fullmatch(r"[a-z0-9-]{1,128}", value) for value in failures):
        raise ResourceError("Unknown cleanup failure code")
    with storage().invocation_edit(str(nonce)) as state:
        if state["phase"] != "open" or state["declaration"] != declaration:
            raise ResourceError("Frozen invocation cannot change cleanup observation")
        state.setdefault("cleanup_observations", {})[str(test)] = {
            "status": "failed" if failures else "complete",
            "failures": list(failures),
        }


def register(payload: Mapping[str, object], *, deadline: float | None = None) -> str:
    deadline = resource_store.clock(deadline)
    if os.environ.get("PSE_TEST_EXECUTION_EFFECTS") == "native-local":
        raise ResourceError("Canonical fixture requires --execution-effects canonical")
    marker = os.environ.get(MARKER)
    declaration = read_invocation(Path(marker)) if marker else None
    image = None
    if declaration is not None:
        image = storage().invocation_snapshot(declaration["nonce"])
        if image["phase"] != "open" or image.get("declaration") != declaration:
            raise ResourceError(
                "Invocation membership is closed; fixture registration refused before effects"
            )
    nonce, test = selected_test(payload, declaration=declaration)
    if nonce is not None and test is not None:
        # selected fixture owner; retain reciprocal lifecycle boundary
        from scripts import test_requirements  # noqa: PLC0415

        try:
            if (
                image is None
                or not image.get("collected")
                or image.get("requirement_policy") != test_requirements.POLICY_VERSION
            ):
                raise ResourceError(
                    "Current collected requirement policy is unavailable; effects refused"
                )
            test_requirements.guard(image.get("requirements", {}), test, "canonical")
            if payload.get("execution_profile") is not None:
                test_requirements.guard(image.get("requirements", {}), test, "native")
        except ValueError as error:
            raise ResourceError(str(error)) from error
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
    if config.get("profile_version") == 3:
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
        service_owner = surreal_server.service_directory(state, deadline=deadline)
        service_registry_binding(service_owner, establish=True, deadline=deadline)
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
            "service_owner": str(service_owner),
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
    resource: str,
    *,
    deadline: float | None = None,
    store: resource_store.FileStore | None = None,
) -> Generator[str, None, None]:
    from scripts import surreal_server  # noqa: PLC0415

    selected_store = storage() if store is None else store
    snapshot = entry_facts(selected_store.lookup(resource, deadline=deadline))
    admission = (
        contextlib.nullcontext()
        if snapshot["kind"] == "evidence"
        else surreal_server.context_admission(
            Path(snapshot["state"]), deadline=deadline, wait_for_lifecycle=False
        )
    )
    token = uuid.uuid4().hex
    with (
        admission,
        resource_update(resource, deadline=deadline, store=selected_store) as record,
    ):
        if not isinstance(record, dict) or record["cleanup"] in {"removing", "removed"}:
            raise ResourceError(
                "Origin is unknown or unavailable for reference acquisition"
            )
        if any(
            record.get(key) != snapshot.get(key)
            for key in ("state", "service_owner", "service_generation", "kind")
        ):
            raise ResourceError(
                "Resource service association changed during borrower admission"
            )
        record["borrows"][token] = {
            "pid": os.getpid(),
            "start": operation.start_identity(os.getpid()),
            "boot": host.boot(),
        }
    try:
        yield token
    finally:
        # With no caller-wide deadline, releasing a long borrow is a new bounded
        # metadata operation, not reuse of admission's expired metadata clock.
        with resource_update(resource, deadline=deadline, store=store) as record:
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
    selected = str(directory.resolve())
    store = storage()
    original = store.identity_lookup([selected, "", "evidence"], deadline=deadline)
    if original is not None:
        return original
    index = store.path("report-locations", resource_store.digest(selected))
    if not index.exists():
        return None
    value = resource_store.read(index)
    resource = value.get("resource")
    if value.get("state") != selected or not isinstance(resource, str):
        raise ResourceError("Report custody location index changed")
    record = resource_status(resource, deadline=deadline)
    if record.get("report_custody", {}).get("state") != selected:
        raise ResourceError("Report custody location differs from its authority")
    return resource


def report_location(record: ResourceRecord) -> ReportLocation:
    """Physical custody may move; the report's original logical identity does not."""
    location = record.get("report_custody")
    state = location["state"] if location is not None else record.get("state")
    owner_root = location["owner_root"] if location is not None else record.get("owner_root")
    identity = location["directory_identity"] if location is not None else record.get("directory_identity")
    if (
        not isinstance(state, str)
        or not isinstance(owner_root, str)
        or not isinstance(identity, list)
        or len(identity) != 2
        or any(type(value) is not int for value in identity)
    ):
        raise ResourceError("Report custody ownership changed or is unavailable")
    return {"state": state, "owner_root": owner_root, "directory_identity": identity}


def _custody_snapshot(record: ResourceRecord) -> dict:
    """Observe every owned producer byte, including an honest unsealed checkpoint."""
    location = report_location(record)
    directory = Path(location["state"])
    manifest = report_manifest(record)
    if "artifact_manifest" in record and manifest is None:
        raise ResourceError("Malformed producer manifest cannot be converted")
    files: dict[str, ReportArtifact] = {}
    directories: dict[str, list[int]] = {}
    with report_directory(record) as descriptor:
        if manifest is not None:
            verify_manifest_files(descriptor, manifest)
        for path in sorted(directory.rglob("*")):
            information = path.lstat()
            if information.st_uid != os.getuid():
                raise ResourceError("Report custody contains a foreign owner")
            relative = str(path.relative_to(directory))
            if stat.S_ISDIR(information.st_mode):
                directories[relative] = [information.st_dev, information.st_ino, information.st_uid]
            elif stat.S_ISREG(information.st_mode):
                observed = observe_artifact(descriptor, relative)
                if manifest is not None and relative in manifest["entries"]:
                    observed["role"] = manifest["entries"][relative]["role"]
                files[relative] = observed
            else:
                raise ResourceError("Report custody cannot traverse symlinks or special files")
    return {"location": location, "files": files, "directories": directories}


def _custody_ready(record: ResourceRecord, *, deadline: float) -> None:
    if (
        record.get("kind") != "evidence"
        or record.get("artifact_policy_version") != 1
        or record.get("cleanup") not in {"retained", "compacted"}
        or record.get("drained") is not True
        or record.get("borrows") != {}
        or record.get("creation_borrower")
        or borrower_alive(record)
    ):
        raise ResourceError("Report conversion requires an owned drained report without live creators or borrowers")
    for unit in record["units"]:
        if not operation.drained({"scope": unit}, deadline=deadline).complete:
            raise ResourceError("Report conversion requires authenticated descendant drain")


@contextlib.contextmanager
def _custody_parent(
    descriptor: int,
    relative: str,
    directories: Mapping[str, list[int]],
) -> Generator[int, None, None]:
    """Open destination parents by their journaled identities, never by symlink."""
    parts = Path(relative).parts
    with contextlib.ExitStack() as opened:
        parent = descriptor
        for index, component in enumerate(parts[:-1]):
            parent = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)
            opened.callback(os.close, parent)
            information = os.fstat(parent)
            expected = directories.get(str(Path(*parts[:index + 1])))
            if expected != [information.st_dev, information.st_ino, information.st_uid]:
                raise ResourceError("Destination report ancestor identity changed")
        yield parent


def _custody_copy(
    journal_path: Path,
    journal: dict,
    item: dict,
    *,
    deadline: float,
) -> None:
    source = validate_resource(item["original"])
    expected = item["snapshot"]
    if _custody_snapshot(source) != expected:
        raise ResourceError("Source report changed during custody conversion")
    target = Path(item["destination"])
    if "destination_identity" not in item:
        base: ResourceRecord = {
            "state": journal["destination"],
            "owner_root": journal["root"],
            "directory_identity": journal["destination_identity"][:2],
        }
        with report_directory(base) as descriptor:
            os.mkdir(target.name, mode=0o700, dir_fd=descriptor)
            os.fsync(descriptor)
        information = target.stat()
        item["destination_identity"] = [information.st_dev, information.st_ino]
        resource_store.publish(journal_path, journal)
    information = target.lstat()
    if (
        not stat.S_ISDIR(information.st_mode)
        or information.st_uid != os.getuid()
        or [information.st_dev, information.st_ino] != item["destination_identity"]
    ):
        raise ResourceError("Destination report custody identity changed")
    target_record: ResourceRecord = {
        "state": str(target),
        "owner_root": journal["root"],
        "directory_identity": item["destination_identity"],
    }
    directories = item.setdefault("directories", {})
    copied = item.setdefault("files", {})
    with report_directory(source) as descriptor, report_directory(target_record) as target_descriptor:
        for relative in expected["directories"]:
            with _custody_parent(target_descriptor, relative, directories) as parent:
                if relative not in directories:
                    os.mkdir(Path(relative).name, mode=0o700, dir_fd=parent)
                    information = os.stat(Path(relative).name, dir_fd=parent, follow_symlinks=False)
                    directories[relative] = [information.st_dev, information.st_ino, information.st_uid]
                    os.fsync(parent)
                    resource_store.publish(journal_path, journal)
                information = os.stat(Path(relative).name, dir_fd=parent, follow_symlinks=False)
                if (
                    not stat.S_ISDIR(information.st_mode)
                    or [information.st_dev, information.st_ino, information.st_uid] != directories[relative]
                ):
                    raise ResourceError("Destination report subdirectory changed")
        for relative, artifact in expected["files"].items():
            host.remaining(deadline, 1)
            with _custody_parent(target_descriptor, relative, directories) as parent:
                if relative not in copied:
                    created = os.open(Path(relative).name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC, 0o600, dir_fd=parent)
                    information = os.fstat(created)
                    os.fsync(created)
                    os.close(created)
                    os.fsync(parent)
                    copied[relative] = {
                        "identity": [information.st_dev, information.st_ino, information.st_uid],
                        "complete": False,
                    }
                    resource_store.publish(journal_path, journal)
                selected = copied[relative]
                information = os.stat(Path(relative).name, dir_fd=parent, follow_symlinks=False)
                if (
                    not stat.S_ISREG(information.st_mode)
                    or [information.st_dev, information.st_ino, information.st_uid] != selected["identity"]
                ):
                    raise ResourceError("Destination report artifact identity changed")
                if selected["complete"]:
                    if observe_artifact(target_descriptor, relative)["digest"] != artifact["digest"]:
                        raise ResourceError("Copied report artifact changed")
                    continue
                with artifact_file(descriptor, relative) as (_, _, stream):
                    if observe_artifact(descriptor, relative)["digest"] != artifact["digest"]:
                        raise ResourceError("Source report artifact changed")
                    output = os.open(Path(relative).name, os.O_WRONLY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)
                    with os.fdopen(output, "wb") as destination:
                        information = os.fstat(output)
                        if [information.st_dev, information.st_ino, information.st_uid] != selected["identity"]:
                            raise ResourceError("Destination report artifact changed before copy")
                        destination.truncate(0)
                        shutil.copyfileobj(stream, destination)
                        destination.flush()
                        os.fchmod(destination.fileno(), stat.S_IMODE(os.fstat(stream.fileno()).st_mode))
                        os.fsync(destination.fileno())
                if observe_artifact(target_descriptor, relative)["digest"] != artifact["digest"]:
                    raise ResourceError("Report byte copy differs from original")
                selected["complete"] = True
                resource_store.publish(journal_path, journal)
    if _custody_snapshot(source) != expected:
        raise ResourceError("Source report changed before custody publication")
    actual = {str(path.relative_to(target)) for path in target.rglob("*")}
    if actual != set(expected["directories"]) | set(expected["files"]):
        raise ResourceError("Unexpected file in destination report custody")
    resource_store.sync_directory(target)


def relocate_reports(
    resources: Sequence[str],
    destination: Path,
    *,
    root: Path = ROOT,
    deadline: float | None = None,
) -> dict:
    """Convert exact retained reports to current physical custody without qualifying them.

    A durable batch reservation blocks readers and disposal during conversion. Original
    identities, bytes, outcomes and provenance remain unchanged. The separate custody
    snapshot grants no scientific reuse or disposal authority to an unsealed report.
    The caller removes the former checkout only after this operation completes.
    """
    deadline = time.monotonic() + 300 if deadline is None else deadline
    selected = sorted(resources)
    if not selected or len(set(selected)) != len(selected) or any(not IDENTIFIER.fullmatch(value) for value in selected):
        raise ResourceError("Report conversion requires distinct exact resource identities")
    root = root.resolve()
    destination = destination.absolute()
    if (
        not destination.is_relative_to(root / "build")
        or ".." in destination.parts
        or any(path.is_symlink() for path in (destination, *destination.parents))
    ):
        raise ResourceError("Converted reports require an owned current-checkout build directory")
    store = storage()
    store.ensure(deadline)
    batch = resource_store.digest({"resources": selected, "destination": str(destination), "root": str(root)})
    journal_path = store.path("report-custody", batch)
    owner = {"pid": os.getpid(), "start": operation.start_identity(os.getpid()), "boot": host.boot()}
    with store.lock("report-custody", batch, deadline):
        if journal_path.exists():
            journal = resource_store.read(journal_path)
            if journal.get("version") != 1 or journal.get("resources") != selected or journal.get("destination") != str(destination) or journal.get("root") != str(root):
                raise ResourceError("Report custody journal identity changed")
            if journal.get("phase") != "complete" and borrower_alive(journal.get("owner", {})):
                raise ResourceError("Report custody conversion still has a live effect owner")
        else:
            if destination.exists():
                raise ResourceError("Report custody destination must be fresh")
            journal = {"version": 1, "batch": batch, "resources": selected, "destination": str(destination), "root": str(root), "phase": "preparing", "items": {}}
        complete = journal.get("phase") == "complete"
        if not complete:
            journal["owner"] = owner
            resource_store.publish(journal_path, journal)
    try:
        if not complete:
            for resource in selected:
                if resource not in journal["items"]:
                    original = resource_status(resource, deadline=deadline)
                    _custody_ready(original, deadline=deadline)
                    if original.get("custody_transition"):
                        raise ResourceError("Report already belongs to another custody conversion")
                    journal["items"][resource] = {"original": original, "snapshot": _custody_snapshot(original), "destination": str(destination / resource)}
                    resource_store.publish(journal_path, journal)
                item = journal["items"][resource]
                with resource_update(resource, deadline=deadline, custody_transition=True) as record:
                    transition = record.get("custody_transition")
                    if transition is None:
                        if record != item["original"]:
                            raise ResourceError("Report authority changed before custody reservation")
                        _custody_ready(record, deadline=deadline)
                        record["custody_transition"] = {"batch": batch, "journal": str(journal_path)}
                    elif transition.get("batch") != batch:
                        raise ResourceError("Report belongs to another custody reservation")
            if "destination_identity" not in journal:
                resource_store.protected(destination)
                information = destination.stat()
                journal["destination_identity"] = [information.st_dev, information.st_ino, information.st_uid]
                resource_store.publish(journal_path, journal)
            information = destination.lstat()
            if not stat.S_ISDIR(information.st_mode) or [information.st_dev, information.st_ino, information.st_uid] != journal["destination_identity"]:
                raise ResourceError("Conversion destination ownership changed")
            journal["phase"] = "copying"
            resource_store.publish(journal_path, journal)
            for resource in selected:
                _custody_copy(journal_path, journal, journal["items"][resource], deadline=deadline)
            paths = {item["snapshot"]["location"]["state"]: item["destination"] for item in journal["items"].values()}
            for resource in selected:
                item = journal["items"][resource]
                location: ReportLocation = {"state": item["destination"], "owner_root": str(root), "directory_identity": item["destination_identity"]}
                with resource_update(resource, deadline=deadline, custody_transition=True) as record:
                    if record.get("report_custody") == location:
                        resource_store.publish(store.path("report-locations", resource_store.digest(item["destination"])), {"state": item["destination"], "resource": resource, "batch": batch})
                        continue
                    expected_record = {**item["original"], "custody_transition": {"batch": batch, "journal": str(journal_path)}}
                    if record != expected_record:
                        raise ResourceError("Report authority changed during conversion")
                    record["report_custody"] = location
                    record.setdefault("custody_history", []).append({"batch": batch, "journal": str(journal_path), "source": item["snapshot"]["location"]})
                    manifest = record.get("artifact_manifest")
                    if manifest is not None:
                        for relative, artifact in manifest["entries"].items():
                            if relative in item["files"]:
                                artifact["identity"] = item["files"][relative]["identity"]
                    references = {}
                    for reference, digest in record["references"].items():
                        replacement = reference
                        for original_path, target_path in paths.items():
                            if Path(reference).is_relative_to(original_path):
                                replacement = str(Path(target_path) / Path(reference).relative_to(original_path))
                                break
                        if replacement in references and references[replacement] != digest:
                            raise ResourceError("Converted report references conflict")
                        references[replacement] = digest
                    record["references"] = references
                resource_store.publish(store.path("report-locations", resource_store.digest(item["destination"])), {"state": item["destination"], "resource": resource, "batch": batch})
            journal["phase"] = "complete"
            journal.pop("owner", None)
            resource_store.publish(journal_path, journal)
        for resource in selected:
            item = journal["items"][resource]
            record = resource_status(resource, deadline=deadline)
            with report_directory(record) as descriptor:
                for relative, artifact in item["snapshot"]["files"].items():
                    observed = observe_artifact(descriptor, relative)
                    if observed["digest"] != artifact["digest"] or observed["identity"] != item["files"][relative]["identity"]:
                        raise ResourceError("Converted producer evidence changed")
            with resource_update(resource, deadline=deadline, custody_transition=True) as current:
                if current.get("custody_transition", {}).get("batch") == batch:
                    current.pop("custody_transition", None)
        return {"version": 1, "status": "complete", "batch": batch, "journal": str(journal_path), "reports": {resource: item["destination"] for resource, item in journal["items"].items()}}
    except BaseException:
        if journal.get("phase") != "complete":
            journal["phase"] = "interrupted"
            journal.pop("owner", None)
            resource_store.publish(journal_path, journal)
        raise


@contextlib.contextmanager
def borrow_report(
    directory: Path, *, deadline: float | None = None
) -> Generator[None, None, None]:
    """Protect a display read; preserve an optional caller-wide deadline."""
    resource = report_resource(directory, deadline=deadline)
    if resource is None:
        yield  # Historical unknown reports cannot be reclaimed by this owner.
    else:
        with borrow(resource, deadline=deadline):
            yield


def read_report_file(
    directory: Path,
    relative: str,
    *,
    require_sealed: bool = False,
    deadline: float | None = None,
) -> tuple[bytes, dict[str, object]]:
    """Authenticate the exact selected bytes observed by one protected reader.

    The caller holds its report borrow. Unknown reports remain explicit inspection
    inputs; a known sealed owner must match its recorded root inode and digest.
    This does not hydrate or hash any unselected source or gate detail.
    """
    resource = report_resource(directory, deadline=deadline)
    record = (
        resource_status(resource, deadline=deadline) if resource is not None else None
    )
    if record is not None and record.get("custody_transition"):
        raise ResourceError("Report custody conversion is reserved; resume its owner")
    manifest = report_manifest(record) if record is not None else None
    if require_sealed and (
        record is None or record.get("artifact_policy_version") != 1 or manifest is None
    ):
        raise ResourceError(
            "Origin requires qualified ownership and a sealed receipt before reuse"
        )
    if record is not None and "artifact_manifest" in record and manifest is None:
        raise ResourceError("Known report has an invalid sealed artifact manifest")

    directory_owned = (
        record is not None
        and isinstance(record.get("owner_root"), str)
        and isinstance(record.get("directory_identity"), list)
    )
    if manifest is not None and not directory_owned:
        raise ResourceError("Sealed report directory ownership is unavailable")

    def observed(stream: BufferedReader) -> tuple[bytes, list[int]]:
        information = os.fstat(stream.fileno())
        if relative == "checks.json" or relative.startswith("details/"):
            host.validate_private_file(information, "assessment metadata")
        elif not stat.S_ISREG(information.st_mode) or information.st_uid != os.getuid():
            raise ResourceError("Evidence artifact ownership changed")
        return stream.read(), [
            information.st_dev,
            information.st_ino,
            information.st_uid,
        ]

    if record is not None and directory_owned:
        with (
            report_directory(record) as descriptor,
            artifact_file(descriptor, relative) as (_, _, stream),
        ):
            data, identity = observed(stream)
    else:
        descriptor = os.open(
            directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
        )
        try:
            with artifact_file(descriptor, relative) as (_, _, stream):
                data, identity = observed(stream)
        finally:
            os.close(descriptor)
    observed_digest = hashlib.sha256(data).hexdigest()
    if manifest is not None:
        expected = manifest["entries"].get(relative)
        if (
            expected is None
            or expected["identity"] != identity
            or expected["digest"] != observed_digest
            or (
                relative == "checks.json"
                and manifest["receipt_digest"] != observed_digest
            )
        ):
            raise ResourceError(
                "Sealed assessment artifact ownership or digest changed: " + relative
            )
        custody: dict[str, object] = {
            "status": "sealed",
            "resource": resource,
            "receipt_digest": manifest["receipt_digest"],
            "automatic_reuse": record is not None
            and record.get("artifact_policy_version") == 1,
        }

    elif record is not None:
        custody = {
            "status": "unsealed-producer-checkpoint"
            if directory_owned
            else "registered-unqualified-inspection",
            "resource": resource,
            "automatic_reuse": False,
        }
    else:
        custody = {"status": "unregistered-inspection", "automatic_reuse": False}
    return data, custody


def read_report_root(
    directory: Path,
    *,
    require_sealed: bool = False,
    deadline: float | None = None,
) -> tuple[bytes, dict[str, object]]:
    """Read and authenticate one small root from the same file descriptor."""
    return read_report_file(
        directory, "checks.json", require_sealed=require_sealed, deadline=deadline
    )


@contextlib.contextmanager
def reference_report(
    directory: Path,
    consumer: Path | None,
    *,
    deadline: float | None = None,
    full_integrity: bool = False,
) -> Generator[None, None, None]:
    """Borrow before reading; transfer protection under the optional caller deadline."""
    resource = report_resource(directory, deadline=deadline)
    if resource is None:
        raise ResourceError(
            "Origin requires qualified ownership and a sealed receipt before reuse"
        )
    with borrow(resource, deadline=deadline):
        _, custody = read_report_root(directory, require_sealed=True, deadline=deadline)
        original_digest = str(custody["receipt_digest"])
        if full_integrity:
            selected = resource_status(resource, deadline=deadline)
            manifest = report_manifest(selected)
            if manifest is None:
                raise ResourceError("Origin lacks an unchanged sealed receipt")
            with report_directory(selected) as descriptor:
                verify_manifest_files(descriptor, manifest)
        if consumer is not None:
            retain_reference(
                resource, str(consumer.resolve()), original_digest, deadline=deadline
            )
        # Durable dependent custody precedes any caller publication. A failed
        # caller may retain a reference; it must never expose an unprotected origin.
        yield
        _, after = read_report_root(directory, require_sealed=True, deadline=deadline)
        if after["receipt_digest"] != original_digest:
            raise ResourceError("Origin receipt changed during reference transfer")


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
    selected = report_location(record)
    directory = Path(selected["state"])
    if (
        not directory.is_absolute()
        or ".." in directory.parts
        or not directory.is_relative_to(Path(selected["owner_root"]) / "build")
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
                selected["directory_identity"] != [identity.st_dev, identity.st_ino]
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
        receipt.get("version") == 6
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
    directory = Path(report_location(selected)["state"])
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
    from scripts import assessment_store  # noqa: PLC0415

    if assessment_store.export(directory) != json.loads(json.dumps(receipt)):
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
    roles = {**roles, **assessment_store.artifact_roles(directory)}
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
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    deadline = resource_store.clock()
    store = storage()
    store.ensure(deadline)
    cursor_path = registry() / "cleanup-cursor.json"
    with store.lock("sweeps", "cleanup", deadline):
        cursors = resource_store.read(cursor_path) if cursor_path.exists() else {}
        candidates = []
        for family in ("scratch-candidates", "fixture-candidates"):
            visited = store.access_ids(
                family, limit=4, after=str(cursors.get(family, "")), deadline=deadline
            )
            candidates.extend(visited)
            if visited:
                cursors[family] = visited[-1]
        resource_store.publish(cursor_path, cursors)
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
        except (
            OSError,
            ValueError,
            subprocess.SubprocessError,
            surreal_server.SupervisorError,
        ) as error:
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
    if record.get("custody_transition"):
        return "report custody conversion is reserved"
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
    config: Mapping[str, object],
    sql: str,
    *,
    namespace: str | None = None,
    database: str | None = None,
    deadline: float | None = None,
) -> list[object]:
    """Bounded root SQL with exact unselected, namespace or database context."""
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    if database is not None and namespace is None:
        raise ResourceError("Database administration requires its namespace")
    if any(
        value is not None and not re.fullmatch(r"[A-Za-z0-9_]{1,128}", value)
        for value in (namespace, database)
    ):
        raise ResourceError(
            "Namespace administration requires exact bounded identifiers"
        )
    namespace_only = namespace is not None and database is None
    query = (
        "USE NS " + surreal_server.sql_identifier(namespace) + ";\n" + sql
        if namespace_only
        else sql
    )
    try:
        results = surreal_server.administrative_results(
            Path(str(config["credentials_file"])).parent,
            query,
            min(deadline, time.monotonic() + CLEANUP_TIMEOUT_SECONDS)
            if deadline is not None
            else time.monotonic() + CLEANUP_TIMEOUT_SECONDS,
            namespace=namespace if database is not None else None,
            database=database,
            select_context=database is not None,
        )
    except (surreal_server.SupervisorError, OSError, ValueError) as error:
        raise ResourceError(
            "Namespace operation acknowledgment unavailable; ownership remains protective"
        ) from error
    if namespace_only:
        if results[0]["status"] != "OK" or results[0]["result"] != {
            "namespace": namespace,
            "database": None,
        }:
            raise ResourceError("Namespace-only selection acknowledgment differs")
        results = results[1:]
    if not results:
        raise ResourceError("Namespace operation acknowledgment unavailable")
    return [row["result"] for row in results]


def remove_namespace(
    record: ResourceRecord,
    config: Mapping[str, object],
    *,
    deadline: float | None = None,
) -> None:
    """Reconcile exact children, then remove only the registered enclosing scope."""
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    namespace = surreal_server.sql_identifier(record["namespace"])
    root_catalog = namespace_sql(config, "INFO FOR ROOT;", deadline=deadline)
    if (
        len(root_catalog) != 1
        or not isinstance(root_catalog[0], dict)
        or not isinstance(root_catalog[0].get("namespaces"), dict)
    ):
        raise ResourceError("Owned namespace existence inventory unavailable")
    if record["namespace"] not in root_catalog[0]["namespaces"]:
        return  # Exact root inventory proves an interrupted removal already completed.
    catalog = namespace_sql(
        config, "INFO FOR NS;", namespace=record["namespace"], deadline=deadline
    )
    if len(catalog) != 1 or not isinstance(catalog[0], dict):
        raise ResourceError("Owned namespace child inventory unavailable")
    children = catalog[0].get("databases")
    if not isinstance(children, dict) or not all(
        isinstance(name, str) for name in children
    ):
        raise ResourceError("Owned namespace child inventory malformed")
    statements = [
        f"REMOVE DATABASE IF EXISTS {surreal_server.sql_identifier(name)};"
        for name in sorted(children)
    ]
    if statements and namespace_sql(
        config, " ".join(statements), namespace=record["namespace"], deadline=deadline
    ) != [None] * len(statements):
        raise ResourceError(
            "Owned namespace child cleanup unproved; resource remains charged"
        )
    # Enclosing namespace removal is deliberately unselected; later observation
    # must not acquire a new context for the namespace being proved absent.
    if namespace_sql(
        config, f"REMOVE NAMESPACE IF EXISTS {namespace};", deadline=deadline
    ) != [None]:
        raise ResourceError(
            "Owned namespace cleanup unproved; resource remains charged"
        )
    after = namespace_sql(config, "INFO FOR ROOT;", deadline=deadline)
    if (
        len(after) != 1
        or not isinstance(after[0], dict)
        or not isinstance(after[0].get("namespaces"), dict)
        or record["namespace"] in after[0]["namespaces"]
    ):
        raise ResourceError(
            "Owned namespace absence unproved; resource remains charged"
        )


def remove_database(
    record: Mapping[str, object],
    config: Mapping[str, object],
    *,
    deadline: float | None = None,
) -> None:
    from scripts import surreal_server  # noqa: PLC0415 -- service cycle

    values = namespace_sql(
        config,
        "REMOVE DATABASE IF EXISTS "
        + surreal_server.sql_identifier(record["database"])
        + ";",
        namespace=str(config["namespace"]),
        deadline=deadline,
    )
    if values != [None]:
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
                or current["namespace"]
                != snapshot.get("service_namespace", snapshot["namespace"])
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
            "setup-failure",
            "drain",
            "record-drain",
            "list",
            "status",
            "pin",
            "release",
            "release-reference",
            "reclaim",
            "migrate",
            "convert",
            "original",
            "resume",
            "abandon",
        ),
    )
    parser.add_argument("resource", nargs="?")
    parser.add_argument("--receipt")
    parser.add_argument("--digest")
    parser.add_argument(
        "--apply",
        action="store_true",
        help="apply exact invocation abandonment instead of inspection",
    )
    parser.add_argument("--seconds", type=float, default=30)
    parser.add_argument("--service", type=Path, action="append", default=[])
    parser.add_argument("--storage", action="store_true")
    parser.add_argument(
        "--terminal-owner", choices=("runner", "standalone", "assessment")
    )
    args = parser.parse_args()
    if args.apply and args.action != "abandon":
        parser.error("--apply belongs to invocation abandonment")
    if args.action == "abandon":
        if not args.resource:
            parser.error("abandon requires an exact invocation declaration or identity")
        return invocation_disposition.main(
            [
                args.resource,
                "--seconds",
                str(args.seconds),
                *(["--apply"] if args.apply else []),
                *(["--storage"] if args.storage else []),
            ]
        )
    if args.action in {"convert", "original"}:
        from scripts import resource_conversion  # noqa: PLC0415

        if not os.environ.get("PSE_TEST_RESOURCE_STATE"):
            parser.error(
                "conversion/original inspection requires an exact PSE_TEST_RESOURCE_STATE"
            )
        if args.action == "convert":
            services = [
                *args.service,
                *(
                    [Path(os.environ["PSE_SURREAL_STATE"])]
                    if os.environ.get("PSE_SURREAL_STATE")
                    else []
                ),
            ]
            print(
                json.dumps(
                    resource_conversion.convert(
                        registry(),
                        deadline=time.monotonic() + args.seconds,
                        services=services,
                    )
                )
            )
        elif not args.resource:
            parser.error("original inspection requires an exact invocation identity")
        else:
            print(json.dumps(resource_conversion.original(registry(), args.resource)))
    elif args.action == "migrate":
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
    elif args.action in {"register", "drain", "record-drain", "setup-failure"}:
        payload = json.load(sys.stdin)
        if args.action == "register":
            print(json.dumps({"resource": register(payload)}))
        elif args.action == "setup-failure":
            observe_setup_failure(payload)
            print(json.dumps({"observed": True}))
        else:
            resource = str(payload["resource"])
            expires = payload.get("deadline_monotonic_ns")
            if type(expires) is not int or expires <= 0:
                raise ResourceError("Drain requires its original monotonic deadline")
            deadline = expires / 1_000_000_000
            host.remaining(deadline, 1)
            record = resource_status(resource, deadline=deadline)
            state = Path(record["state"])
            units = payload.get("units", [])
            if (
                args.action == "drain"
                and (state / ".contexts" / f"{record['database']}.json").exists()
            ):
                from scripts import surreal_server  # noqa: PLC0415 -- owner cycle

                units = [
                    *units,
                    *surreal_server.drain_context(
                        state, record["database"], deadline=deadline
                    ),
                ]
            if args.action == "record-drain" or not payload.get("physical_only", False):
                record_drain(resource, units=units, deadline=deadline)
            print(
                json.dumps(
                    {
                        "drained": args.action == "record-drain"
                        or not payload.get("physical_only", False),
                        "units": units,
                    }
                )
            )
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
