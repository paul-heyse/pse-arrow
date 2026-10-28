# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Own the released local SurrealDB server, its private state and resource budget.

Application admission and native-worker drain belong to runtime composition. The
supervisor records that admission state; privileged direct database clients must
not be used to bypass it. See docs/dev/surreal-substrate.md.
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import errno
import fcntl
import hashlib
import http.client
import json
import math
import os
import platform
import re
import secrets
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, cast

# Recorded receivers launch this file directly, independently of the caller's
# working directory. Its sibling capability owners belong to this installation.
if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

# CLI and immutable direct-script launches share one lifecycle/clock authority
# shared by service admission composition; a second module would see this
# process's live reservation as a competing owner.
if __name__ == "__main__":
    sys.modules["scripts.surreal_server"] = sys.modules[__name__]

from scripts import (
    backup_image,
    execution_profiles,
    host_admission,
    native_operation,
    operation_lifetime,
    process_observation,
    pse_env,
    receiver_closure,
    receiver_environment,
    recovery_operation,
    service_borrowers,
    surreal_source,
    surrealism_module,
    telemetry_service,
)
from scripts.generated import deployment
from scripts.generated.deployment import DeploymentReadiness as Readiness

if TYPE_CHECKING:
    import io
    from collections.abc import Callable, Generator, Mapping


MIB = 1024 * 1024
GIB = 1024 * MIB
_POLICY_PATH = (
    Path(__file__).resolve().parents[1]
    / "crates/pse-operations/src/generated/surreal-policy.json"
)
_POLICY = json.loads(_POLICY_PATH.read_text())
MESSAGE_BYTES = _POLICY["max_message_bytes"]
SUBSTRATE_INTERPRETATION = _POLICY["interpretation"]
OWNER = "pse-arrow-surreal-v1"
SCRIPT = Path(__file__).resolve()
_STARTUP = threading.local()
_LIFECYCLE = threading.local()
WORKER_CAPABILITIES = ("solver", "klu", "isolation", "uno", "petsc")


GENERATION_CONFIGURATION = (
    ".config/agent-capacity.toml",
    ".config/build.toml",
    ".config/sccache.toml",
    ".cargo/config.toml",
    "packages/reference/conformance.toml",
    "docker/solvers/Dockerfile",
    "Cargo.toml",
    "rust-toolchain.toml",
    ".python-version",
    "crates/pse-operations/src/generated/surreal-policy.json",
    "crates/pse-study-module/surrealism.toml",
    "crates/pse-study-module/surrealism-qualification-long-hold.toml",
)


class SupervisorError(RuntimeError):
    """An actionable lifecycle error with no credential-bearing output."""


class DeadlineExpiredError(SupervisorError):
    """The original admission/control deadline expired before the next effect."""


@dataclass(frozen=True)
class ServiceObservation:
    phase: Readiness
    state: Path
    instance_id: object
    unit: str
    cause: str
    reservation: Mapping[str, object] | None = None
    manager: Mapping[str, str] = field(default_factory=dict)
    listener_owned: bool = False
    protocol_ready: bool = False

    def diagnostic(self) -> str:
        return f"Service readiness is {self.phase.value}: {self.cause}; selected={self.state}, instance_id={self.instance_id}, unit={self.unit}; reservation={self.reservation}; observed={dict(self.manager)}"

    def document(self) -> dict[str, object]:
        import msgspec  # noqa: PLC0415 -- private typed observation transport

        return object_mapping(
            msgspec.to_builtins(
                deployment.DeploymentObservation(
                    phase=self.phase,
                    state=str(self.state),
                    unit=self.unit,
                    cause=self.cause,
                    instance_id=self.instance_id
                    if isinstance(self.instance_id, str)
                    else None,
                )
            )
        )

    def __bool__(self) -> bool:
        raise TypeError("Consume ServiceObservation.phase explicitly")


class LifecycleBusyError(SupervisorError):
    def __init__(self, nonce: object) -> None:
        super().__init__(
            "Another live owner is changing this service; retry after its operation finishes"
        )
        self.nonce = nonce


def unadmitted_service_launch_drained(
    state: Path,
    unit: str,
    launch: dict[str, object],
    *,
    deadline: float | None = None,
) -> bool:
    """Authenticate a materialized initial launch that never admitted a process."""
    if "binding" in launch:
        return False
    config = config_for(state, deadline=deadline)
    drain_config = storage_launch_drain_config(state, config, launch, deadline=deadline)
    observed = operation_lifetime.unit_observation(
        unit, **operation_lifetime.deadline_options(deadline)
    )
    if (
        observed.get("LoadState") != "loaded"
        or observed.get("ActiveState") not in {"inactive", "failed"}
        or observed.get("ControlGroup") != ""
        or observed.get("InvocationID") != ""
    ):
        return False
    readiness = service_readiness(state, drain_config, deadline=deadline)
    if (
        readiness.phase is not Readiness.ACTIONABLE_ABSENT
        or readiness.state != state
        or readiness.unit != unit
        or readiness.instance_id != launch.get("generation")
        or readiness.listener_owned
        or any(
            readiness.manager.get(key) != observed.get(key)
            for key in ("ActiveState", "ControlGroup", "InvocationID")
        )
        or config_for(state, deadline=deadline) != config
        or storage_launch_drain_config(state, config, launch, deadline=deadline)
        != drain_config
        or read_json(state / "service-launch.json", deadline=deadline) != launch
        or operation_lifetime.unit_observation(
            unit, **operation_lifetime.deadline_options(deadline)
        )
        != observed
    ):
        return False
    # A concurrent admission or unexpected receipt must keep the allocation held.
    # Recheck the exact missing path after all selection/manager observations.
    try:
        read_json(state / "server-process.json", deadline=deadline)
    except FileNotFoundError:
        return True
    return False


def service_processes_drained(
    record: Mapping[str, object],
    allocation: Path | None,
    *,
    deadline: float | None = None,
) -> bool:
    """Do not reclaim an affiliated storage process merely because its cgroup emptied."""
    raw_services = record.get("borrowed_services", [])
    units = record.get("units")
    if (
        not isinstance(raw_services, list)
        or any(not isinstance(value, str) for value in raw_services)
        or not isinstance(units, dict)
    ):
        return False
    services = list(cast("list[str]", raw_services))
    if record.get("service"):
        if not isinstance(record["service"], str):
            return False
        services.append(record["service"])
    for selected in services:
        if deadline is not None:
            host_admission.require_admission_time(
                deadline, "Original drain clock expired"
            )
        state = Path(selected)
        unit = unit_name(state)
        if unit not in units:
            continue  # A caller observing another role did not select this storage unit.
        if allocation is None:
            return False
        launch = None
        try:
            launch = read_json(state / "service-launch.json", deadline=deadline)
            affiliation = launch["allocation"]
            if not isinstance(affiliation, str) or not re.fullmatch(
                r"[a-f0-9]{32}", Path(affiliation).name
            ):
                return False
            if Path(affiliation).absolute() != allocation.absolute():
                continue  # A stale service descriptor belongs to a different owner.
            process = read_json(state / "server-process.json", deadline=deadline)
            generation = launch["generation"]
            if (
                not isinstance(generation, str)
                or not generation
                or process["instance_id"] != generation
                or not isinstance(process["start"], str)
                or not process["start"]
            ):
                return False
            pid = host_admission.integer(process["pid"])
            if pid <= 0:
                return False
            if process["allocation"] != affiliation:
                # A failed pre-bind restart publishes a new launch while the
                # previous immutable process receipt remains. Authenticate that
                # precise pending launch; a mismatched bound lifetime stays held.
                previous = process["allocation"]
                if (
                    units[unit]
                    or "binding" in launch
                    or not isinstance(previous, str)
                    or not re.fullmatch(r"[a-f0-9]{32}", Path(previous).name)
                    or not Path(previous).is_absolute()
                    or Path(previous).parent != allocation.parent
                    or Path(previous) == allocation
                    or str(Path(previous)) != previous
                ):
                    return False
                try:
                    operation_lifetime.start_identity(pid)
                except FileNotFoundError:
                    pass
                else:
                    return False  # Live or reused PID is uncertain, not an ACK.
                config = config_for(state, deadline=deadline)
                drain_config = storage_launch_drain_config(
                    state, config, launch, deadline=deadline
                )
                try:
                    observed = operation_lifetime.unit_observation(
                        unit, **operation_lifetime.deadline_options(deadline)
                    )
                except subprocess.SubprocessError:
                    return False
                if (
                    observed.get("LoadState") != "loaded"
                    or observed.get("ActiveState") not in {"inactive", "failed"}
                    or observed.get("ControlGroup")
                ):
                    return False
                try:
                    readiness = service_readiness(
                        state, drain_config, deadline=deadline
                    )
                except subprocess.SubprocessError:
                    return False
                if (
                    readiness.phase is not Readiness.ACTIONABLE_ABSENT
                    or readiness.state != state
                    or readiness.unit != unit
                    or readiness.instance_id != generation
                    or readiness.listener_owned
                    or any(
                        readiness.manager.get(key) != observed.get(key)
                        for key in ("ActiveState", "ControlGroup", "InvocationID")
                    )
                    or config_for(state, deadline=deadline) != config
                    or storage_launch_drain_config(
                        state, config, launch, deadline=deadline
                    )
                    != drain_config
                    or read_json(state / "service-launch.json", deadline=deadline)
                    != launch
                    or read_json(state / "server-process.json", deadline=deadline)
                    != process
                ):
                    return False
                continue
            try:
                if operation_lifetime.start_identity(pid) == process["start"]:
                    return False
            except FileNotFoundError:
                pass
        except FileNotFoundError:
            # Registration precedes the launch. No bound lifetime can use this
            # missing-process proof; a materialized initial unit needs exact
            # selection and repeated terminal/absence observations as well.
            if (
                units[unit]
                or not isinstance(units[unit], dict)
                or (launch is not None and "binding" in launch)
            ):
                return False
            try:
                (state / "server-process.json").read_text()
            except FileNotFoundError:
                try:
                    observed = (
                        operation_lifetime.unit_observation(unit)
                        if deadline is None
                        else operation_lifetime.unit_observation(
                            unit, deadline=deadline
                        )
                    )
                    if observed.get("LoadState") == "not-found":
                        continue
                    if launch is not None and unadmitted_service_launch_drained(
                        state, unit, launch, deadline=deadline
                    ):
                        continue
                except (
                    OSError,
                    ValueError,
                    TypeError,
                    KeyError,
                    subprocess.SubprocessError,
                    SupervisorError,
                ):
                    pass
            except OSError:
                pass
            return False
        except (
            OSError,
            ValueError,
            TypeError,
            KeyError,
            SupervisorError,
        ):
            return False
    return True


def drain_borrowed_services(
    directory: Path,
    record: Mapping[str, object],
    *,
    allocation: Path,
    explicit: bool = False,
    deadline: float | None = None,
) -> None:
    """An exclusive caller returns borrowed storage after all other roles drain."""
    raw_services = record.get("borrowed_services", [])
    if not isinstance(raw_services, list) or any(
        not isinstance(value, str) for value in raw_services
    ):
        raise host_admission.AdmissionError("Invalid borrowed service ownership")
    services = cast("list[str]", raw_services)
    raw_units = record.get("units")
    if not isinstance(raw_units, dict):
        raise host_admission.AdmissionError("Invalid borrowed service unit ownership")
    bound_units = cast("dict[str, host_admission.BoundUnit]", raw_units)
    if not services or (not explicit and host_admission.caller_alive(record)):
        return
    units = {unit_name(Path(value)) for value in services}
    other = {
        **record,
        "units": {
            unit: owner for unit, owner in bound_units.items() if unit not in units
        },
        "released": True,
    }
    options = operation_lifetime.deadline_options(deadline)
    if not host_admission.drained(
        other, deadline=deadline, services=ServiceAdmission()
    ):
        return
    for value in services:
        state = Path(value)
        with (
            lifecycle_deadline(deadline),
            lifecycle_reservation(state),
        ):
            launch = json.loads((state / "service-launch.json").read_text())
            affiliation = launch["allocation"]
            if not isinstance(affiliation, str) or not re.fullmatch(
                r"[a-f0-9]{32}", Path(affiliation).name
            ):
                raise host_admission.AdmissionError(
                    "Borrowed storage allocation is unobservable"
                )
            if Path(affiliation).absolute() != allocation.absolute():
                continue  # A newer allocation owns this service; the snapshot cannot act on it.
            unit = unit_name(state)
            expected = bound_units.get(unit)
            observed = operation_lifetime.unit_observation(unit, **options)
            if (
                not expected
                and "binding" not in launch
                and observed.get("LoadState") == "not-found"
            ):
                continue  # No storage lifetime was ever bound for this launch.
            if (
                not isinstance(expected, dict)
                or type(expected.get("inode")) is not int
                or not isinstance(expected.get("group"), str)
                or not expected.get("group")
                or not re.fullmatch(r"[a-f0-9]{32}", expected.get("invocation", ""))
                or launch.get("binding") != expected
            ):
                raise host_admission.AdmissionError("Borrowed storage binding changed")
            config = config_for(state, **options)
            if config.get("instance_id") != launch.get("generation"):
                raise host_admission.AdmissionError(
                    "Borrowed storage generation changed"
                )
            if observed.get("LoadState") != "not-found":
                stopped = observed.get("ActiveState") in {"inactive", "failed"}
                identity = host_admission.group_identity(expected["group"])
                if (
                    observed.get("ControlGroup")
                    not in ({"", expected["group"]} if stopped else {expected["group"]})
                    or observed.get("InvocationID")
                    not in (
                        {"", expected["invocation"]}
                        if stopped
                        else {expected["invocation"]}
                    )
                    or (
                        not stopped
                        and observed.get("ActiveState")
                        not in {"active", "activating", "deactivating"}
                    )
                    or identity
                    not in (
                        {None, expected["inode"]} if stopped else {expected["inode"]}
                    )
                ):
                    raise host_admission.AdmissionError(
                        "Borrowed storage lifetime changed"
                    )
            # Residency permits borrowing a running service; it does not reverse
            # an intentional stop or a failed service on allocation release.
            if not config.get("resident") or observed.get("ActiveState") != "active":
                workers_drained(state, config)
                stop(state, config)
            else:
                try:
                    park_service(state, expected_binding=expected, **options)
                finally:
                    queue_parked_service(directory, state, **options)


def queue_parked_service(
    directory: Path, state: Path, *, deadline: float | None = None
) -> None:
    """Publish actual parking intent while the caller owns the service lifecycle."""
    if config_for(state, **operation_lifetime.deadline_options(deadline)).get("parked"):
        selected = str(state)
        with host_admission.allocation_metadata(directory, deadline=deadline) as ledger:
            if selected not in ledger.setdefault("parked_services", []):
                ledger["parked_services"].append(selected)


class ServiceAdmission:
    """Compose host allocation with this supervisor's exact storage lifetime."""

    def processes_drained(
        self,
        record: Mapping[str, object],
        allocation: Path | None,
        *,
        deadline: float | None = None,
    ) -> bool:
        return service_processes_drained(record, allocation, deadline=deadline)

    def return_borrowed(
        self,
        directory: Path,
        record: Mapping[str, object],
        *,
        allocation: Path,
        explicit: bool = False,
        deadline: float | None = None,
    ) -> None:
        drain_borrowed_services(
            directory,
            record,
            allocation=allocation,
            explicit=explicit,
            deadline=deadline,
        )

    def park(
        self,
        directory: Path,
        owners: Mapping[str, host_admission.AllocationRecord],
        *,
        deadline: float,
    ) -> None:
        for owner in owners.values():
            host_admission.require_admission_time(
                deadline,
                "Original admission clock expired while parking owned services",
            )
            service = owner["service"]
            path = Path(service)
            binding = owner["units"].get(unit_name(path))
            if not binding:
                raise host_admission.AdmissionError(
                    "Parkable service lacks its bound lifetime"
                )
            with lifecycle_reservation(path):
                try:
                    park_service(
                        path,
                        deadline=deadline,
                        expected_binding=binding,
                    )
                finally:
                    # Publish resume intent under the same lifecycle
                    # as parking. Preserve partial-stop recovery,
                    # but a stale stopped premise queues nothing.
                    queue_parked_service(directory, path)

    def resume(self, state: Path, *, deadline: float | None = None) -> None:
        unpark_service(state, deadline=deadline)


def measurement_context(env: Mapping[str, str] | None = None) -> dict[str, object]:
    source = os.environ if env is None else env
    service = None
    selected = source.get("PSE_SURREAL_STATE")
    if selected:
        state_path = Path(selected)
        config = config_for(state_path)
        process = state_path / "server-process.json"
        service = {
            "state": selected,
            "generation": config["instance_id"],
            "resident": bool(config.get("resident")),
            "server_process": read_json(process) if process.is_file() else None,
            "supervisor": config.get("service_supervisor"),
            "receiver": config.get("primary_receiver"),
            "execution_profile": source.get("PSE_TEST_EXECUTION_PROFILE"),
            "filesystem_cache": "uncontrolled",
            "surrealmx_memory": "not independently measured",
            "schema_provisioning_timed": "declared by selected journey",
        }
    return host_admission.measurement_context(source, service=service)


def read_json(path: Path, *, deadline: float | None = None) -> dict[str, object]:
    def unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise SupervisorError("Owned metadata has a duplicate field")
            result[key] = value
        return result

    if deadline is not None:
        return object_mapping(
            json.loads(
                private_record_bytes(path, deadline), object_pairs_hook=unique_object
            )
        )
    value = json.loads(path.read_text(), object_pairs_hook=unique_object)
    if not isinstance(value, dict):
        raise SupervisorError(f"Expected an object in {path.name}")
    return value


def object_mapping(value: object) -> dict[str, object]:
    """Narrow an owned metadata object before consuming its named fields."""
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise SupervisorError("Expected a string-keyed owned metadata object")
    return value


def object_inventory(value: object) -> list[dict[str, object]]:
    """Validate a complete metadata collection before callers act on any member."""
    if not isinstance(value, list):
        raise SupervisorError("Expected an owned metadata inventory")
    return [object_mapping(item) for item in value]


def string_mapping(value: object) -> dict[str, str]:
    """Require string values where the owned metadata schema declares them."""
    selected = object_mapping(value)
    if any(not isinstance(item, str) for item in selected.values()):
        raise SupervisorError("Expected string values in owned metadata")
    return cast("dict[str, str]", selected)


def integer(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise SupervisorError("Expected an integer in owned configuration")
    return value


def write_json(path: Path, value: Mapping[str, object]) -> None:
    """Replace private metadata atomically and persist the directory entry."""
    fd, name = tempfile.mkstemp(prefix=f".{path.name}-", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as stream:
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        Path(name).replace(path)
        directory = os.open(path.parent, os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        Path(name).unlink(missing_ok=True)


def lexical_absolute(path: Path) -> Path:
    """Normalize lexical aliases without following symlinks or hiding their identity."""
    # Path.resolve follows links; Path.absolute alone retains '..' aliases. This
    # lexical normalization preserves the precheck and stable systemd slot names.
    return Path(os.path.abspath(path))  # noqa: PTH100 -- Must normalize '..' without following symlinks.


def checked_directory(path: Path, *, empty: bool = False) -> Path:
    path = path.absolute()
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise SupervisorError("State and backup paths must not traverse symlinks")
    path = lexical_absolute(path)
    if path.exists():
        if not path.is_dir() or path.stat().st_uid != os.getuid():
            raise SupervisorError("Directory must be owned by the current user")
        if empty and any(path.iterdir()):
            raise SupervisorError(
                "Refusing to initialize or replace a nonempty directory"
            )
    else:
        path.mkdir(parents=True, mode=0o700)
    path.chmod(0o700)
    return path


@contextlib.contextmanager
def state_lock(
    state: Path, *, deadline: float | None = None, cleanup: bool = False
) -> Generator[None, None, None]:
    if not cleanup:
        inherited = getattr(_STARTUP, "deadline", None)
        deadline = (
            inherited
            if deadline is None
            else deadline
            if inherited is None
            else min(deadline, inherited)
        )
    else:
        deadline = None

    def lock_remaining() -> float:
        if deadline is None:
            raise SupervisorError("Metadata exclusion has no admission deadline")
        try:
            return remaining(deadline)
        except DeadlineExpiredError as error:
            raise DeadlineExpiredError(
                f"Metadata exclusion deadline expired; selected={state}; admission refused"
            ) from error

    if deadline is not None:
        lock_remaining()
    fd = os.open(
        state / ".supervisor.lock", os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600
    )
    try:
        while True:
            if deadline is not None:
                lock_remaining()
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                time.sleep(0.02 if deadline is None else min(0.02, lock_remaining()))
        if deadline is not None:
            lock_remaining()
        yield
    finally:
        os.close(fd)


def publish_generation(
    state: Path, worker: Path | None = None, producer: Path | None = None
) -> dict[str, str]:
    """Freeze the complete owned Python/configuration closure before launch."""
    root = SCRIPT.parents[1]
    try:
        owned = receiver_closure.source_files(root, ["scripts/surreal_server.py"])
    except receiver_closure.ClosureError as error:
        raise SupervisorError(str(error)) from error
    sources = [
        *(root / name for name in owned),
        root / "scripts/sccache",
        *sorted((root / "scripts/surrealdb-source").glob("*.patch")),
    ]
    for relative in GENERATION_CONFIGURATION:
        sources.append(root / relative)
    prepared_inputs = (
        receiver_environment.capture(WORKER_CAPABILITIES, systemd_environment())
        if worker is not None
        else None
    )
    entries = {str(path.relative_to(root)): file_digest(path) for path in sources}
    if prepared_inputs is not None:
        entries["native-environment.json"] = hashlib.sha256(prepared_inputs).hexdigest()
    operator = execution_profiles.operator()
    if not operator.is_file() or not os.access(operator, os.X_OK):
        raise SupervisorError("Build pse-deployment before publishing a receiver")
    entries["bin/pse-deployment"] = file_digest(operator)
    if worker is not None:
        entries["bin/pse-worker"] = file_digest(worker)
    if producer is not None:
        entries["producer/worker.json"] = file_digest(producer)
    generations = state / ".generations"
    generations.mkdir(mode=0o700, exist_ok=True)
    # Probe exactly the candidate bytes; importing the development checkout does
    # not establish that the retained generation can import independently.
    pending = generations / ("pending-" + uuid.uuid4().hex)
    pending.mkdir(mode=0o700)
    for source in sources:
        target = pending / source.relative_to(root)
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        target.chmod(0o700 if source == root / "scripts/sccache" else 0o600)
    if prepared_inputs is not None:
        with (pending / "native-environment.json").open("xb") as output:
            output.write(prepared_inputs)
            output.flush()
            os.fsync(output.fileno())
        (pending / "native-environment.json").chmod(0o600)
    for name, source in (
        ("bin/pse-deployment", operator),
        ("bin/pse-worker", worker),
        ("producer/worker.json", producer),
    ):
        if source is not None:
            target = pending / name
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            target.chmod(0o700 if name.startswith("bin/") else 0o600)
    if any(
        file_digest(pending / name) != expected for name, expected in entries.items()
    ):
        raise SupervisorError(
            "Source changed while publishing immutable generation; pending bytes preserved"
        )
    interpreter = Path(sys.executable).absolute()
    try:
        providers = receiver_closure.probe(pending, owned, interpreter)
    except receiver_closure.ClosureError as error:
        raise SupervisorError(str(error)) from error
    if prepared_inputs is not None:
        import msgspec  # noqa: PLC0415 -- compiled runtime-input schema

        providers.update(
            receiver_environment.compiler_files(
                msgspec.json.decode(
                    prepared_inputs, type=deployment.PreparedReceiverEnvironment
                ).values
            )
        )
    declaration = {
        "files": entries,
        "runtime": providers,
        "interpreter": str(interpreter),
    }
    identity = hashlib.sha256(
        json.dumps(declaration, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    destination = generations / identity
    write_json(
        pending / "generation.json", {"version": 2, "identity": identity, **declaration}
    )
    if not destination.exists():
        # Concurrent complete generations have identical content; preserve pending.
        try:
            pending.rename(destination)
        except OSError as error:
            if error.errno not in {errno.EEXIST, errno.ENOTEMPTY}:
                raise
    else:
        shutil.rmtree(pending)
    verify_generation(destination)
    return {
        "supervisor_executable": str(interpreter),
        "supervisor_script": str(destination / "scripts/surreal_server.py"),
        "supervisor_sha256": entries["scripts/surreal_server.py"],
        **(
            {
                "worker_executable": str(destination / "bin/pse-worker"),
                "worker_sha256": entries["bin/pse-worker"],
            }
            if worker is not None
            else {}
        ),
    }


@contextlib.contextmanager
def lifecycle_reservation(
    state: Path, *, purpose: str = "maintenance", deadline: float | None = None
) -> Generator[None, None, None]:
    """Reserve an operation with short metadata exclusion, never an IPC-held lock."""
    state = service_directory(state)
    deadline = getattr(_STARTUP, "deadline", None) if deadline is None else deadline
    if deadline is not None:
        remaining(deadline)
    if purpose not in {"startup", "stop", "maintenance"}:
        raise SupervisorError("Unknown lifecycle reservation purpose")
    if getattr(_LIFECYCLE, "state", None) == state:
        yield
        return

    path = state / "lifecycle-owner.json"
    selected: dict[str, object] = {
        "nonce": uuid.uuid4().hex,
        "pid": os.getpid(),
        "start": operation_lifetime.start_identity(os.getpid()),
        "boot": process_observation.boot(),
        "purpose": purpose,
    }
    with state_lock(state, deadline=deadline):
        previous = read_json(path) if path.exists() else None
        if (
            previous is not None
            and previous["boot"] == selected["boot"]
            and reservation_live(previous)
        ):
            raise LifecycleBusyError(previous["nonce"])
        if deadline is not None:
            remaining(deadline)
        write_json(path, selected)
    previous_state = getattr(_LIFECYCLE, "state", None)
    _LIFECYCLE.state = state
    try:
        yield
    finally:
        try:
            # This reservation already owns the listener. Its existing stop
            # lifetime must survive expiry/cancellation of admission.
            admission_deadline = getattr(_STARTUP, "deadline", None)
            _STARTUP.deadline = None
            try:
                drain_maintenance_listener(state)
            finally:
                _STARTUP.deadline = admission_deadline
            with state_lock(state, cleanup=True):
                if path.exists() and read_json(path).get("nonce") == selected["nonce"]:
                    path.unlink()
        finally:
            _LIFECYCLE.state = previous_state


def wait_reservation(state: Path, path: Path, nonce: object, deadline: float) -> None:
    """Wait for this exact owner without extending the caller's admission clock."""
    while True:
        remaining(deadline)
        with state_lock(state, deadline=deadline):
            if not path.exists():
                remaining(deadline)
                return
            current = read_json(path)
            if current.get("nonce") != nonce or not reservation_live(current):
                remaining(deadline)
                return
        time.sleep(min(0.02, remaining(deadline)))


def reservation_live(reservation: dict[str, object]) -> bool:
    if (
        reservation.get("boot") is not None
        and reservation["boot"] != process_observation.boot()
    ):
        return False
    if reservation.get("unresolved") is True:
        # Launcher exit cannot settle a dispatched manager job or its descendants.
        return True
    try:
        process = process_observation.process_stat(integer(reservation["pid"]))
        return process[0] not in {"Z", "X", "x"} and process[19] == reservation["start"]
    except FileNotFoundError:
        return False
    except (OSError, ValueError, IndexError, KeyError):
        return True


@contextlib.contextmanager
def context_admission(
    state: Path, *, deadline: float | None = None, wait_for_lifecycle: bool = True
) -> Generator[None, None, None]:
    """Serialize only admission metadata against all service lifecycle changes."""
    inherited = getattr(_STARTUP, "deadline", None)
    deadline = (
        inherited
        if deadline is None
        else deadline
        if inherited is None
        else min(deadline, inherited)
    )
    service = service_directory(state)
    while True:
        with state_lock(service, deadline=deadline):
            lifecycle = service / "lifecycle-owner.json"
            owner = read_json(lifecycle) if lifecycle.exists() else None
            live = owner is not None and reservation_live(owner)
            if not live:
                config = config_for(service)
                if config["admission"] != "open" or not config["accepting_writes"]:
                    raise SupervisorError("Service context admission is closed")
                if service == state:
                    if deadline is not None:
                        remaining(deadline)
                    yield
                else:
                    with state_lock(state, deadline=deadline):
                        if deadline is not None:
                            remaining(deadline)
                        yield
                return
            if deadline is None or not wait_for_lifecycle:
                raise SupervisorError(
                    "Service lifecycle operation has closed context admission"
                )
        wait_reservation(service, lifecycle, owner["nonce"], deadline)


def managed_contexts(state: Path) -> list[Path]:
    """Inventory registered generations while lifecycle admission is reserved."""
    contexts = [state]
    for descriptor in sorted((state / ".contexts").glob("*.json")):
        if descriptor.is_symlink():
            raise SupervisorError("Context descriptor must not traverse symlinks")
        context = read_json(descriptor)
        selected = receiver_context(state, str(context["database"]))
        if selected == state:
            raise SupervisorError("Registered receiver context is missing")
        contexts.append(selected)
    return contexts


def close_context_admission(
    state: Path, *, admission: str = "quiesced", cleanup: bool = False
) -> None:
    # No admission can pass the service reservation while these small writes run.
    for context in managed_contexts(state):
        with state_lock(context, cleanup=cleanup):
            config = config_for(context)
            config["accepting_writes"] = False
            if config["admission"] != "validation_required":
                config["admission"] = admission
            write_json(context / "config.json", config)


def open_context_admission(state: Path) -> None:
    for context in managed_contexts(state):
        with state_lock(context):
            config = config_for(context)
            if (
                config.get("backup_pending") is not None
                or config.get("derived_rebuild_pending")
                or config.get("surrealkit_rollout_pending")
            ):
                raise SupervisorError("Incomplete owned maintenance remains closed")
            deadline = getattr(_STARTUP, "deadline", None)
            if deadline is not None:
                remaining(deadline)
            if config.get("payload_frozen"):
                if (
                    payload_query(
                        context,
                        config,
                        f"RETURN fn::pse_files_v1::thaw({payload_expiry()},{json.dumps(config['payload_incarnation'])});",
                    )
                    is not True
                ):
                    raise SupervisorError("File admission reopen was not acknowledged")
                config.pop("payload_frozen")
            config.update(accepting_writes=True, admission="open")
            write_json(context / "config.json", config)


def require_borrowers_drained(state: Path) -> None:
    """Native-free registered contexts also own an admitted service lifetime."""
    owner = service_directory(state)
    require_tooling_drained(owner)
    for borrower in service_borrowers.obligations(owner):
        selected = borrower.state
        if selected != owner and owner / ".receivers" not in selected.parents:
            continue
        if borrower.protected is not None:
            raise SupervisorError(borrower.protected)
        if (
            borrower.requires_association
            and selected != owner
            and service_directory(selected) != owner
        ):
            raise SupervisorError("Live context has an unresolved service association")
        if borrower.active_owner:
            raise SupervisorError(
                "A live test context still borrows this storage service"
            )


def all_contexts_drained(state: Path) -> None:
    require_borrowers_drained(state)
    # Offline recovery shares the enclosing control's original clock. Standalone
    # offline callers use the existing ninety-second stop-control lifetime.
    recovery_deadline = getattr(_STARTUP, "deadline", None)
    if recovery_deadline is None:
        recovery_deadline = time.monotonic() + 90
    for context in managed_contexts(state):
        context_processes_drained(context, deadline=recovery_deadline)


def context_processes_drained(context: Path, *, deadline: float) -> None:
    """Observe exact launch, observer and worker obligations without fixture disposition."""
    for pending in [
        context / "primary-admission.json",
        context / "observer-launch.json",
        context / "primary-observer.json",
        *context.glob("worker-admission-*.json"),
    ]:
        if pending.exists() and read_json(pending).get("unresolved") is True:
            reconcile_registration(context, pending, deadline=deadline)
        if pending.exists() and reservation_live(read_json(pending)):
            raise SupervisorError(
                "Context launch reservation remains active; wait for admission to settle"
            )
    if observer_launch_busy(context):
        raise SupervisorError(
            "Context observer launch remains active; drain before offline action"
        )
    observer = context / "primary-observer.json"
    if observer.exists():
        registration = read_json(observer)
        # verify_observer records actual membership for an existing process,
        # which need not have a supervisor-created unit name. A dead leader
        # does not free that recorded group while descendants survive.
        group = registration.get("group", "")
        if registration.get("unit"):
            group = systemctl(
                "show",
                "--property=ControlGroup",
                "--value",
                str(registration["unit"]),
                check=False,
            ).stdout.strip()
        if not isinstance(group, str):
            raise SupervisorError("Context observer lacks recorded kernel membership")
        if group.startswith("/") and group_populated(group):
            raise SupervisorError(
                "Context observer process group remains populated; drain before offline action"
            )
    workers_drained(context, config_for(context))


def owned_generations(state: Path) -> set[Path]:
    generations: set[Path] = set()
    for context in managed_contexts(state):
        config = config_for(context)
        for role in ("service_supervisor", "primary_receiver"):
            receiver = config.get(role)
            if isinstance(receiver, dict) and receiver.get("supervisor_script"):
                generation = Path(str(receiver["supervisor_script"])).parents[1]
                if generation.parent != state / ".generations":
                    raise SupervisorError(
                        "Admitted source generation is outside this service state"
                    )
                if generation not in generations:
                    verify_generation(generation)
                    generations.add(generation)
    return generations


@contextlib.contextmanager
def lifecycle_deadline(deadline: float | None) -> Generator[float | None, None, None]:
    """Share one original control clock across public lifecycle composition."""
    if deadline is not None and not math.isfinite(deadline):
        raise SupervisorError("Lifecycle control requires a finite absolute deadline")
    previous = getattr(_STARTUP, "deadline", None)
    selected = (
        previous
        if deadline is None
        else deadline
        if previous is None
        else min(previous, deadline)
    )
    _STARTUP.deadline = selected
    try:
        if selected is not None:
            remaining(selected)
        yield selected
        if selected is not None:
            remaining(selected)
    finally:
        _STARTUP.deadline = previous


def readmit_supervisor(
    state: Path,
    *,
    deadline: float | None = None,
    execution_profile: str | None = None,
    worker_executable: Path | None = None,
    verifier: Path | None = None,
) -> dict[str, object]:
    """Explicitly readmit current artifacts while preserving the closed target."""
    with lifecycle_deadline(deadline), lifecycle_reservation(state):
        return _readmit_supervisor(
            state,
            execution_profile=execution_profile,
            worker_executable=worker_executable,
            verifier=verifier,
        )


def _readmit_supervisor(
    state: Path,
    *,
    execution_profile: str | None = None,
    worker_executable: Path | None = None,
    verifier: Path | None = None,
) -> dict[str, object]:
    config = config_for(state)
    if config.get("backup_pending") is not None:
        raise SupervisorError(
            "Backup pending owner requires explicit resume before readmission"
        )
    if (
        active(state)
        or config.get("accepting_writes")
        or config.get("admission") not in {"quiesced", "validation_required"}
    ):
        raise SupervisorError(
            "Readmission requires this selected service stopped with closed admission"
        )
    # Validate every offered artifact and the unchanged allocation before any
    # publication, history or config effects. Readmission grants no new capacity.
    receiver = None
    if execution_profile is not None:
        if config.get("resources") != execution_resources(execution_profile):
            raise SupervisorError(
                "Readmission execution profile differs from the selected allocation"
            )
        expected_class = "timing" if execution_profile == "timing" else "functional"
        if config.get("service_class", "functional") != expected_class:
            raise SupervisorError(
                "Readmission execution profile differs from the selected storage lane"
            )
        receiver = primary_receiver(worker_executable)
    elif worker_executable is not None:
        raise SupervisorError(
            "Receiver readmission requires an explicit execution profile"
        )
    selected_verifier = payload_verifier(verifier) if verifier is not None else None
    target_fields = (
        "instance_id",
        "namespace",
        "database",
        "payload_incarnation",
        "admission",
        "accepting_writes",
        "resources",
        "service_class",
        "telemetry_state",
        "derived_rebuild_pending",
        "surrealkit_rollout_pending",
        "fresh_root_bootstrap",
    )
    original_target = {name: config.get(name) for name in target_fields}
    close_context_admission(state)
    all_contexts_drained(state)
    deadline = getattr(_STARTUP, "deadline", None)
    if deadline is not None:
        remaining(deadline)
    history = state / ".profile-history"
    history.mkdir(mode=0o700, exist_ok=True)
    write_json(history / (uuid.uuid4().hex + ".json"), config)
    if "telemetry_state" not in config:
        settle_legacy_telemetry_launch(state, config, history, deadline=deadline)
    telemetry_state = (
        recorded_telemetry_state(config)
        if "telemetry_state" in config
        else str(telemetry_service.state_directory())
    )
    credentials = read_json(state / "credentials.json")
    if (
        "selection_username" not in credentials
        or "selection_password" not in credentials
    ):
        write_json(history / (uuid.uuid4().hex + "-credentials.json"), credentials)
        credentials.update(
            selection_username="pse-selection",
            selection_password=secrets.token_urlsafe(36),
        )
        write_json(state / "credentials.json", credentials)
    if deadline is not None:
        remaining(deadline)
    supervisor = (
        publish_generation(state, Path(receiver["worker_executable"]))
        if receiver is not None
        else publish_generation(state)
    )
    if (
        receiver is not None
        and supervisor.get("worker_sha256") != receiver["worker_sha256"]
    ):
        raise SupervisorError(
            "Receiver artifact changed during readmission; admission remains closed"
        )
    server = dict(object_mapping(config["server"]))
    server["binary_sha256"] = file_digest(Path(str(server["binary"])))
    with state_lock(state):
        config = config_for(state)
        if {name: config.get(name) for name in target_fields} != original_target:
            raise SupervisorError(
                "Readmission target or maintenance intent changed; admission remains closed"
            )
        if receiver is not None:
            config["primary_receiver"] = supervisor
        if selected_verifier is not None:
            config["payload_verifier"] = selected_verifier
        config.update(
            service_supervisor=supervisor,
            server=server,
            restart_qualified=False,
            telemetry_state=telemetry_state,
        )
        write_json(state / "config.json", config)
    return public_status(state, config)


def settle_legacy_telemetry_launch(
    state: Path,
    config: dict[str, object],
    history: Path,
    *,
    deadline: float | None,
) -> None:
    """Retire the original launch before changing its authenticated selection."""
    deadline = time.monotonic() + 90 if deadline is None else deadline
    path = state / "service-launch.json"
    if not path.exists():
        return
    launch = storage_launch(state, config, deadline=deadline)
    allocation = Path(str(launch.get("allocation", "")))
    if not allocation.is_absolute() or not re.fullmatch(
        r"[a-f0-9]{32}", allocation.name
    ):
        raise SupervisorError("Legacy launch lacks its original allocation")
    if "binding" not in launch and not unadmitted_service_launch_drained(
        state, unit_name(state), launch, deadline=deadline
    ):
        raise SupervisorError(
            "Legacy unbound launch must drain before telemetry readmission"
        )
    record = host_admission.readonly_snapshot(allocation.parent, deadline=deadline)[
        "owners"
    ].get(allocation.name)
    owner = None
    if record is not None:
        if (
            (
                record.get("service") != str(state)
                and str(state) not in record.get("borrowed_services", [])
            )
            or record.get("service") not in {None, str(state)}
            or any(
                selected != str(state)
                for selected in record.get("borrowed_services", [])
            )
            or set(record["units"]) != {unit_name(state)}
            or record["units"].get(unit_name(state)) != launch.get("binding", {})
        ):
            raise SupervisorError(
                "Legacy allocation must be dedicated to this exact launch; settle its enclosing owner first"
            )
        profile = host_admission.Profile(
            record["class"],
            record["memory"],
            record["lane"],
            record["slots"],
            tuple(record["cores"]),
            record["exclusive"],
        )
        owner = host_admission.Allocation(
            allocation.parent,
            allocation.name,
            profile,
            record["deadline"],
            services=ServiceAdmission(),
        )
    stop(state, config)
    if owner is not None and not owner.release(deadline=deadline):
        raise SupervisorError(
            "Legacy allocation must settle before telemetry readmission"
        )
    if (
        allocation.name
        in host_admission.readonly_snapshot(allocation.parent, deadline=deadline)[
            "owners"
        ]
    ):
        raise SupervisorError(
            "Legacy allocation must settle before telemetry readmission"
        )
    if read_json(path, deadline=deadline) != launch:
        raise SupervisorError("Legacy launch changed during telemetry readmission")
    path.rename(history / (uuid.uuid4().hex + "-service-launch.json"))


@dataclass(frozen=True)
class GenerationVerification:
    files: dict[str, str]
    observation: tuple[tuple[object, ...], ...]
    runtime: dict[str, str] = field(default_factory=dict)
    version: int = 1


@dataclass
class PrimaryVerification:
    # Only the primary-admission owner installs this object. Offline managed
    # mutation refuses its live reservation in all_contexts_drained(). Observed
    # metadata invalidates evidence; it never independently qualifies bytes.
    generations: dict[Path, GenerationVerification] = field(default_factory=dict)
    process: str | None = None
    process_observed: bool = False


def generation_observation(
    directory: Path, names: Mapping[str, str], runtime: Mapping[str, str] | None = None
) -> tuple[tuple[object, ...], ...]:
    observed: list[tuple[object, ...]] = []
    for path in (directory, *directory.parents):
        information = path.lstat()
        if not stat.S_ISDIR(information.st_mode):
            raise SupervisorError("Immutable generation must not traverse symlinks")
        observed.append(
            (
                str(path),
                information.st_dev,
                information.st_ino,
                information.st_mode,
                information.st_uid,
                information.st_gid,
            )
        )
    for name in ("generation.json", *names):
        path = directory / name
        if not path.is_relative_to(directory) or ".." in Path(name).parts:
            raise SupervisorError("Immutable supervisor/receiver closure changed")
        for parent in path.parents:
            if parent == directory:
                break
            information = parent.lstat()
            if not stat.S_ISDIR(information.st_mode):
                raise SupervisorError("Immutable generation must not traverse symlinks")
            observed.append(
                (
                    str(parent),
                    information.st_dev,
                    information.st_ino,
                    information.st_mode,
                    information.st_uid,
                    information.st_gid,
                )
            )
        information = path.lstat()
        if not stat.S_ISREG(information.st_mode):
            raise SupervisorError("Immutable generation must not traverse symlinks")
        if name == "generation.json" and (
            information.st_uid != os.getuid()
            or stat.S_IMODE(information.st_mode) != 0o600
        ):
            raise SupervisorError(
                "Immutable generation manifest lost its private ownership"
            )
        observed.append(
            (
                str(path),
                information.st_dev,
                information.st_ino,
                information.st_mode,
                information.st_uid,
                information.st_gid,
                information.st_size,
                information.st_mtime_ns,
                information.st_ctime_ns,
            )
        )
        deadline = getattr(_STARTUP, "deadline", None)
        if deadline is not None:
            remaining(deadline)
    for name in runtime or {}:
        information = Path(name).stat()
        observed.append(
            (
                name,
                information.st_dev,
                information.st_ino,
                information.st_mode,
                information.st_uid,
                information.st_size,
                information.st_mtime_ns,
                information.st_ctime_ns,
            )
        )
    return tuple(observed)


def verify_generation(directory: Path) -> GenerationVerification:
    owner = getattr(_STARTUP, "verification", None)
    retained = owner.generations.get(directory) if owner is not None else None
    if (
        retained is not None
        and generation_observation(directory, retained.files, retained.runtime)
        == retained.observation
    ):
        return retained
    if owner is not None:
        owner.generations.pop(directory, None)
    if any(path.is_symlink() for path in (directory, *directory.parents)):
        raise SupervisorError("Immutable generation must not traverse symlinks")
    manifest = directory / "generation.json"
    if manifest.is_symlink():
        raise SupervisorError("Immutable generation must not traverse symlinks")
    receipt = read_json(manifest)
    if (
        receipt.get("version") not in {1, 2}
        or receipt.get("identity") != directory.name
        or not isinstance(receipt.get("files"), dict)
    ):
        raise SupervisorError("Missing immutable generation identity")
    files = receipt["files"]
    runtime = receipt.get("runtime", {}) if receipt["version"] == 2 else {}
    if not isinstance(runtime, dict) or not all(
        isinstance(name, str) and Path(name).is_absolute() and isinstance(expected, str)
        for name, expected in runtime.items()
    ):
        raise SupervisorError("Immutable interpreter/provider declaration changed")
    declaration = (
        {"files": files, "runtime": runtime, "interpreter": receipt.get("interpreter")}
        if receipt["version"] == 2
        else files
    )
    identity = hashlib.sha256(
        json.dumps(declaration, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    if identity != directory.name:
        raise SupervisorError("Immutable generation declaration changed")
    if not all(
        isinstance(name, str) and isinstance(expected, str)
        for name, expected in files.items()
    ):
        raise SupervisorError("Immutable generation declaration changed")
    before = generation_observation(directory, files, runtime)
    for name, expected in files.items():
        path = directory / name
        if not path.is_relative_to(directory) or ".." in Path(name).parts:
            raise SupervisorError("Immutable supervisor/receiver closure changed")
        if any(
            part.is_symlink()
            for part in (path, *path.parents)
            if part.is_relative_to(directory)
        ):
            raise SupervisorError("Immutable generation must not traverse symlinks")
        if file_digest(path) != expected:
            raise SupervisorError("Immutable supervisor/receiver closure changed")
        executable = name in {"scripts/sccache", "bin/pse-worker", "bin/pse-deployment"}
        information = path.stat()
        if information.st_uid != os.getuid() or stat.S_IMODE(information.st_mode) != (
            0o700 if executable else 0o600
        ):
            raise SupervisorError(
                "Immutable executable closure lost its admitted execution mode or private ownership"
            )
    for name, expected in runtime.items():
        if file_digest(Path(name)) != expected:
            raise SupervisorError("Immutable interpreter/provider inputs changed")
    after = generation_observation(directory, files, runtime)
    if before != after:
        raise SupervisorError("Immutable generation changed during verification")
    verified = GenerationVerification(
        dict(files), after, dict(runtime), receipt["version"]
    )
    if owner is not None:
        owner.generations[directory] = verified
    return verified


def activatable_generation(directory: Path) -> GenerationVerification:
    verified = verify_generation(directory)
    if verified.version != 2:
        raise SupervisorError(
            "Retained generation requires explicit offline readmission before startup"
        )
    return verified


def execution_resources(name: str) -> dict[str, object]:
    selected = name
    try:
        return execution_profiles.profile(selected)
    except execution_profiles.DeploymentError as error:
        raise SupervisorError(str(error)) from error


def context_document(value: Mapping[str, object]) -> dict[str, object]:
    import msgspec  # noqa: PLC0415 -- private typed boundary

    if "version" not in value:
        raise SupervisorError("Registered context omitted its persisted version")
    try:
        return object_mapping(
            msgspec.to_builtins(
                msgspec.convert(
                    value, type=deployment.RegisteredNativeContext, strict=True
                )
            )
        )
    except msgspec.ValidationError as error:
        raise SupervisorError(
            "Registered context differs from its owned deployment schema"
        ) from error


def register_context(
    state: Path, database: str, profile: str, worker: Path | None = None
) -> dict[str, object]:
    """Create an explicit receiver/context descriptor without mutating service selection."""
    config = config_for(state)
    timing_service = config.get("service_class") == "timing"
    if (profile == "timing") != timing_service:
        raise SupervisorError("Timing contexts require their dedicated timing service")
    if (
        not database
        or len(database) > 128
        or any(
            not (character.isascii() and (character.isalnum() or character == "_"))
            for character in database
        )
    ):
        raise SupervisorError("Context database requires a bounded ASCII identity")
    with context_admission(state):
        if (state / ".contexts" / f"{database}.json").exists():
            raise SupervisorError(
                "Context is already registered; use its explicit owner"
            )
    allocation = execution_resources(profile)
    producer, producer_digest = selected_worker_receipt(systemd_environment())
    receiver = (
        publish_generation(state, worker, Path(producer) if producer else None)
        if worker is not None
        else None
    )
    directory = state / ".contexts"
    target = directory / f"{database}.json"
    if target.exists():
        raise SupervisorError("Context is already registered; use its explicit owner")
    identity = hashlib.sha256(
        json.dumps(
            {
                "receiver": receiver,
                "resources": allocation,
                "producer": producer,
                "producer_digest": producer_digest,
            },
            sort_keys=True,
        ).encode()
    ).hexdigest()
    receiver_state = state / ".receivers" / identity / database
    selected = dict(config)
    selected.update(
        database=database,
        resources=allocation,
        primary_receiver=receiver,
        service_state=str(state),
        receiver_generation=identity,
        resident=False,
    )
    if producer is not None and receiver is not None:
        selected["worker_producer_receipt"] = {
            "path": str(
                Path(receiver["supervisor_script"]).parents[1] / "producer/worker.json"
            ),
            "sha256": producer_digest,
            "origin": producer,
        }
    with context_admission(state):
        if target.exists():
            raise SupervisorError(
                "Context was registered concurrently; use its explicit owner"
            )
        # mkdir(parents=True) uses the process umask for intermediate parents.
        # Native selection authenticates every receiver parent independently.
        for parent in (state, directory, state / ".receivers", receiver_state.parent):
            with contextlib.suppress(FileExistsError):
                parent.mkdir(mode=0o700)
            information = parent.lstat()
            if (
                not stat.S_ISDIR(information.st_mode)
                or information.st_uid != os.getuid()
                or information.st_mode & 0o077
            ):
                raise SupervisorError(
                    "Receiver state parents must be private real owned directories"
                )
        try:
            receiver_state.mkdir(mode=0o700)
        except FileExistsError as error:
            raise SupervisorError(
                "Receiver state already exists; retain its explicit owner"
            ) from error
        selected.update(config_for(state))
        selected.update(
            database=database,
            resources=allocation,
            primary_receiver=receiver,
            service_state=str(state),
            receiver_generation=identity,
            resident=False,
        )
        if producer is not None and receiver is not None:
            selected["worker_producer_receipt"] = {
                "path": str(
                    Path(receiver["supervisor_script"]).parents[1]
                    / "producer/worker.json"
                ),
                "sha256": producer_digest,
                "origin": producer,
            }
        write_json(receiver_state / "config.json", selected)
        write_json(
            target,
            context_document(
                {
                    "version": 1,
                    "database": database,
                    "resources": allocation,
                    "primary_receiver": receiver,
                    "receiver_state": str(receiver_state),
                    "receiver_generation": identity,
                }
            ),
        )
    return read_json(target)


def receiver_context(state: Path, database: str | None) -> Path:
    if database is None:
        return state
    path = state / ".contexts" / f"{database}.json"
    if not path.exists():
        return state
    context = context_document(read_json(path))
    selected = Path(str(context["receiver_state"]))
    if (
        selected.is_symlink()
        or state / ".receivers" not in selected.parents
        or context["database"] != database
    ):
        raise SupervisorError("Invalid registered receiver/context association")
    config = config_for(selected)
    if (
        config.get("service_state") != str(state)
        or config["database"] != database
        or config.get("receiver_generation") != context["receiver_generation"]
    ):
        raise SupervisorError(
            "Receiver generation does not match selected service/context"
        )
    return selected


def service_directory(state: Path, *, deadline: float | None = None) -> Path:
    path = state / "config.json"
    if not path.is_file():
        return state
    config = read_json(path, deadline=deadline)
    selected = config.get("service_state")
    if selected is None:
        return state
    result = Path(str(selected))
    if result / ".receivers" not in state.parents:
        raise SupervisorError("Receiver state is not owned by the recorded service")
    return result


def service_config_for(
    state: Path, *, deadline: float | None = None
) -> dict[str, object]:
    """Attest service identity without opening scientific interpretation or workers."""
    if (
        not state.is_absolute()
        or ".." in state.parts
        or any(path.is_symlink() for path in (state, *state.parents))
    ):
        raise SupervisorError("Service state must be an exact absolute owned directory")
    information = state.stat()
    if (
        not stat.S_ISDIR(information.st_mode)
        or information.st_uid != os.getuid()
        or information.st_mode & 0o077
    ):
        raise SupervisorError("Service state must be a private owned directory")
    config = object_mapping(
        json.loads(
            private_record_bytes(
                state / "config.json", deadline, maximum_bytes=1024 * 1024
            )
        )
    )
    if (
        config.get("owner") != OWNER
        or config.get("profile_version") != 3
        or config.get("service_state") is not None
    ):
        raise SupervisorError("Tooling requires the selected service root")
    try:
        uuid.UUID(str(config.get("instance_id")))
    except ValueError as error:
        raise SupervisorError("Invalid selected service identity") from error
    port = integer(config.get("port"))
    if not 1 <= port <= 65535 or config.get("endpoint") != f"grpc://127.0.0.1:{port}":
        raise SupervisorError(
            "Tooling requires the recorded authenticated loopback service"
        )
    receipt = object_mapping(config.get("server"))
    if not surreal_source.admitted_receipt(receipt):
        raise SupervisorError("Tooling service is not the admitted MX generation")
    if (state / "fresh-engine-pending.json").exists():
        raise SupervisorError("Fresh-engine initialization is incomplete")
    return config


@contextlib.contextmanager
def tooling_activity(
    state: Path, *, deadline: float
) -> Generator[tuple[dict[str, object], int], None, None]:
    """Register an exact service-only lifetime, inherited by the foreground tool.

    Only this activity's kernel lock survives RPC work. The service metadata lock
    is released before yielding. An inherited descriptor retains the lifetime if
    the wrapper dies; inactive markers are collected by lifecycle drain.
    """
    descriptor = None
    path = None
    try:
        with state_lock(state, deadline=deadline):
            config = service_config_for(state, deadline=deadline)
            lifecycle = state / "lifecycle-owner.json"
            if lifecycle.exists() and reservation_live(
                read_json(lifecycle, deadline=deadline)
            ):
                raise SupervisorError(
                    "Service lifecycle operation has closed tooling admission"
                )
            if (
                config.get("admission") != "open"
                or config.get("accepting_writes") is not True
            ):
                raise SupervisorError("Service tooling admission is closed")
            directory = state / ".tooling" / ".activities"
            host_admission.protected(state / ".tooling")
            host_admission.protected(directory)
            path = directory / (uuid.uuid4().hex + ".json")
            descriptor = os.open(
                path,
                os.O_CREAT | os.O_EXCL | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
                0o600,
            )
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            os.write(
                descriptor,
                json.dumps(
                    {"version": 1, "service_instance_id": config["instance_id"]}
                ).encode(),
            )
            os.fsync(descriptor)
        yield config, descriptor
    finally:
        if descriptor is not None:
            # Do not LOCK_UN: the child may still hold this open description.
            os.close(descriptor)
        if path is not None:
            # Remove normally finished activities promptly. If an inherited
            # child is still running or the original clock expired, preserve
            # the marker for the existing lifecycle drain to inspect.
            with (
                contextlib.suppress(OSError, DeadlineExpiredError),
                state_lock(state, deadline=deadline),
            ):
                finished = os.open(path, os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC)
                try:
                    fcntl.flock(finished, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    path.unlink()
                finally:
                    os.close(finished)


def require_tooling_drained(state: Path) -> None:
    """Called under the existing lifecycle reservation before any service effect."""
    directory = state / ".tooling" / ".activities"
    if not directory.exists():
        return
    if any(path.is_symlink() for path in (directory, directory.parent)):
        raise SupervisorError("Tooling activity directory changed")
    for selected in (directory.parent, directory):
        information = selected.lstat()
        if (
            not stat.S_ISDIR(information.st_mode)
            or information.st_uid != os.getuid()
            or information.st_mode & 0o077
        ):
            raise SupervisorError(
                "Tooling activity directory must be private and owned"
            )
    for path in directory.iterdir():
        if not re.fullmatch(r"[a-f0-9]{32}\.json", path.name):
            raise SupervisorError("Unknown service tooling activity")
        descriptor = os.open(
            path, os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
        )
        try:
            host_admission.validate_private_file(
                os.fstat(descriptor), "tooling activity"
            )
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise SupervisorError(
                    "A live tooling command still borrows this service"
                ) from error
            path.unlink()
        finally:
            os.close(descriptor)


def admitted_runtime_server(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> bool:
    """Select immutable receipt validation only for this exact bound supervisor."""
    receipt = object_mapping(config["server"])
    supervisor = config.get("service_supervisor")
    if not isinstance(supervisor, dict) or supervisor.get("supervisor_script") != str(
        SCRIPT
    ):
        if SCRIPT.parent.parent.parent.name == ".generations":
            raise SupervisorError(
                "Running immutable supervisor is not the selected service binding"
            )
        return surreal_source.admitted_receipt(receipt)
    directory = SCRIPT.parent.parent
    service = service_directory(state, deadline=deadline)
    if (
        SCRIPT.name != "surreal_server.py"
        or SCRIPT.parent.name != "scripts"
        or directory.parent != service / ".generations"
        or supervisor.get("supervisor_executable")
        not in {
            str(Path(sys.executable).absolute()),
            str(Path(sys.executable).resolve()),
        }
    ):
        raise SupervisorError(
            "Running supervisor is not its bound immutable service generation"
        )
    verified = verify_generation(directory)
    if verified.files.get("scripts/surreal_server.py") != supervisor.get(
        "supervisor_sha256"
    ):
        raise SupervisorError(
            "Running supervisor differs from its immutable service generation"
        )
    return surreal_source.admitted_runtime_receipt(receipt, directory)


def config_for(state: Path, *, deadline: float | None = None) -> dict[str, object]:
    if (
        any(path.is_symlink() for path in (state, *state.parents))
        or (state / "config.json").is_symlink()
    ):
        raise SupervisorError("Owned state must not traverse symlinks")
    config = read_json(state / "config.json", deadline=deadline)
    if "telemetry_state" in config:
        recorded_telemetry_state(config)
    retiring = config.get("profile_version") == 2 and getattr(
        _LIFECYCLE, "retire_engine", False
    )
    if config.get("owner") != OWNER or (
        config.get("profile_version") != 3 and not retiring
    ):
        raise SupervisorError(
            "Unsupported server profile; retire it with fresh-engine before ordinary use"
        )
    if (state / "fresh-engine-pending.json").exists() and not getattr(
        _LIFECYCLE, "retire_engine", False
    ):
        raise SupervisorError(
            "Fresh-engine initialization is incomplete; resume that exact closed operation"
        )
    if (
        config.get(
            "websocket_max_message_bytes" if retiring else "rpc_max_message_bytes"
        )
        != MESSAGE_BYTES
    ):
        raise SupervisorError("Unsupported native RPC message profile")
    # A retired profile can predate payload incarnations. This owner only
    # verifies drain and discards its generated files; it never opens them.
    if not retiring:
        incarnation = config.get("payload_incarnation")
        if not isinstance(incarnation, str):
            raise SupervisorError(
                "Missing payload incarnation; explicitly regenerate this profile"
            )
        try:
            parsed_incarnation = uuid.UUID(incarnation)
        except ValueError as error:
            raise SupervisorError("Invalid payload incarnation") from error
        if str(parsed_incarnation) != incarnation:
            raise SupervisorError("Invalid payload incarnation")
    budget = config.get("resources")
    if not isinstance(budget, dict):
        raise SupervisorError("Missing resource allocation")
    if retiring:
        expected = resources(
            integer(budget["total_memory_bytes"]),
            integer(budget["server_memory_bytes"]),
            integer(budget["native_workers"]),
            integer(budget["native_worker_memory_bytes"]),
            budget.get("execution"),
        )
        expected = {
            key: value
            for key, value in expected.items()
            if not key.startswith("surrealmx_") and key != "grpc_max_attached_sessions"
        }
        expected.update(
            rocksdb_block_cache_bytes=integer(budget["server_memory_bytes"]) // 4,
            rocksdb_write_buffer_bytes=min(
                32 * MIB, integer(budget["server_memory_bytes"]) // 64
            ),
            rocksdb_write_buffers=2,
        )
        if budget != expected:
            raise SupervisorError(
                "Retiring resource overrides differ from their original allocation"
            )
    else:
        try:
            execution_profiles.validate_configuration(config, deadline=deadline)
        except execution_profiles.DeploymentError as error:
            raise SupervisorError(str(error)) from error
    port = integer(config["port"])
    scheme = "ws" if retiring else "grpc"
    if not 1 <= port <= 65535 or config["endpoint"] != f"{scheme}://127.0.0.1:{port}":
        raise SupervisorError(
            "Only the recorded authenticated loopback endpoint is supported"
        )
    if not retiring and not getattr(_LIFECYCLE, "retire_engine", False):
        if not admitted_runtime_server(state, config, deadline=deadline):
            raise SupervisorError(
                "Server is not the admitted source-built MX generation"
            )
        study_module(config)
        qualification_module(config)
        module_runtime_allocation(budget)
    if (
        config["max_message_bytes"] != MESSAGE_BYTES
        or config["schema_interpretation"] != config["interpretation"]
    ):
        raise SupervisorError("Inconsistent substrate interpretation or message limit")
    if (
        not isinstance(config["accepting_writes"], bool)
        or not isinstance(config["admission"], str)
        or config["admission"]
        not in {"open", "quiescing", "quiesced", "validation_required"}
    ):
        raise SupervisorError("Invalid application admission state")
    return config


def resources(
    total: int, server: int, workers: int, worker: int, execution: object = None
) -> dict[str, object]:
    try:
        return execution_profiles.materialize(total, server, workers, worker, execution)
    except execution_profiles.DeploymentError as error:
        raise SupervisorError(str(error)) from error


def reference_resources() -> dict[str, object]:
    return execution_resources("reference")


def primary_receiver(executable: Path | None) -> dict[str, str]:
    if executable is None:
        raise SupervisorError(
            "Selecting reference execution requires --worker-executable"
        )
    worker = executable.resolve(strict=True)
    interpreter = Path(sys.executable).absolute()
    if not worker.is_file() or not os.access(worker, os.X_OK):
        raise SupervisorError("Primary receiver must name a built executable")
    return {
        "supervisor_executable": str(interpreter),
        "supervisor_script": str(SCRIPT),
        "worker_executable": str(worker),
        "supervisor_sha256": file_digest(SCRIPT),
        "worker_sha256": file_digest(worker),
    }


def default_tool_root() -> Path:
    return (
        Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share")))
        / "pse-arrow/tools/surreal"
    )


def install(
    version: str | None, tool_root: Path, *, build: bool = False
) -> dict[str, object]:
    """Reuse the pinned external package; only build-server authorizes compilation."""
    if version not in {None, "v" + surreal_source.VERSION}:
        raise SupervisorError(
            "Canonical storage requires pinned SurrealDB 3.3.4 with MX 1.0.0"
        )
    if platform.system() != "Linux" or platform.machine() not in {"x86_64", "aarch64"}:
        raise SupervisorError(
            "The managed source server supports Linux amd64/arm64 only"
        )
    try:
        directory = checked_directory(tool_root / surreal_source.installation_name())
        with state_lock(directory):
            return (
                surreal_source.install(directory)
                if build
                else surreal_source.require_installed(directory)
            )
    except (surreal_source.BuildError, OSError, ValueError) as error:
        raise SupervisorError(str(error)) from error


def file_digest(path: Path) -> str:
    with path.open("rb") as stream:
        digest = hashlib.sha256()
        while True:
            deadline = getattr(_STARTUP, "deadline", None)
            if deadline is not None:
                remaining(deadline)
            chunk = stream.read(MIB)
            if not chunk:
                break
            digest.update(chunk)
        if deadline is not None:
            remaining(deadline)
        return digest.hexdigest()


def setup(args: argparse.Namespace) -> dict[str, object]:
    args.interpretation = args.interpretation or SUBSTRATE_INTERPRETATION
    if (args.state / "config.json").exists():
        service_borrowers.service_registry_binding(args.state.absolute())
        config = config_for(args.state.absolute())
        if config["interpretation"] != args.interpretation:
            raise SupervisorError("Existing state has a different interpretation")
        allocation = config["resources"]
        if args.tooling_fixture and config.get("tooling_fixture") is not True:
            raise SupervisorError(
                "Existing state is not a beta tooling fixture; select a fresh explicit tooling state"
            )
        if args.execution_profile:
            if not isinstance(allocation, dict):
                raise SupervisorError("Missing recorded execution allocation")
            if "execution" not in allocation:
                raise SupervisorError(
                    "Existing state requires offline reconfigure to select reference execution"
                )
            if allocation != execution_resources(args.execution_profile):
                raise SupervisorError(
                    "Existing execution profile differs; reconfigure offline"
                )
            expected_class = (
                "timing" if args.execution_profile == "timing" else "functional"
            )
            if config.get("service_class", "functional") != expected_class:
                raise SupervisorError(
                    "Existing storage lane differs; reconfigure offline"
                )
        return public_status(args.state.absolute(), config)
    # Download first so network failure leaves no half-initialized application state.
    default = execution_profiles.default_allocation()
    allocation = resources(
        integer(default["total_memory_bytes"])
        if args.memory_mib is None
        else args.memory_mib * MIB,
        integer(default["server_memory_bytes"])
        if args.server_memory_mib is None
        else args.server_memory_mib * MIB,
        integer(default["native_workers"])
        if args.native_workers is None
        else args.native_workers,
        integer(default["native_worker_memory_bytes"])
        if args.native_worker_memory_mib is None
        else args.native_worker_memory_mib * MIB,
    )
    receiver = None
    if args.execution_profile:
        if any(
            value is not None
            for value in (
                args.memory_mib,
                args.server_memory_mib,
                args.native_workers,
                args.native_worker_memory_mib,
            )
        ):
            raise SupervisorError(
                "Reference execution supplies exact capacities; memory overrides are refused"
            )
        allocation = execution_resources(args.execution_profile)
        receiver = primary_receiver(args.worker_executable)
    port = 18240 if args.port is None else args.port
    if (
        not 1 <= port <= 65535
        or not args.interpretation
        or not args.interpretation.strip()
    ):
        raise SupervisorError(
            "Require a valid loopback port and an explicit interpretation identity"
        )
    tool_root = args.tool_root
    if tool_root is None:
        tool_root = default_tool_root()
    module_runtime_allocation(allocation)
    server = install(args.version, tool_root)
    try:
        module = surrealism_module.install(server, tool_root)
    except (surrealism_module.ModuleError, OSError, ValueError) as error:
        raise SupervisorError(str(error)) from error
    state = checked_directory(args.state, empty=True)
    with state_lock(state):
        if any(path.name != ".supervisor.lock" for path in state.iterdir()):
            raise SupervisorError(
                "State was initialized concurrently; refusing to replace it"
            )
        config: dict[str, object] = {
            "owner": OWNER,
            "profile_version": 3,
            "instance_id": str(uuid.uuid4()),
            "payload_incarnation": str(uuid.uuid4()),
            "server": server,
            "study_module": module,
            "port": port,
            "namespace": "pse",
            "database": "canonical",
            "endpoint": f"grpc://127.0.0.1:{port}",
            "credentials_file": str(state / "credentials.json"),
            "schema_interpretation": args.interpretation,
            "accepting_writes": False,
            "interpretation": args.interpretation,
            "admission": "validation_required",
            "max_message_bytes": MESSAGE_BYTES,
            "rpc_max_message_bytes": MESSAGE_BYTES,
            "tooling_fixture": args.tooling_fixture,
            "resources": allocation,
            "log_max_bytes": 8 * MIB,
            "log_backups": 2,
            "service_class": "timing"
            if args.execution_profile == "timing"
            else "functional",
            "telemetry_state": str(telemetry_service.state_directory()),
        }
        write_json(
            state / "credentials.json",
            {
                "username": "pse-local",
                "password": secrets.token_urlsafe(36),
                "selection_username": "pse-selection",
                "selection_password": secrets.token_urlsafe(36),
            },
        )
        config["fresh_root_bootstrap"] = {
            "instance_id": config["instance_id"],
            "credentials_sha256": file_digest(state / "credentials.json"),
            "nonce": uuid.uuid4().hex,
        }
        if args.payload_verifier is not None:
            config["payload_verifier"] = payload_verifier(args.payload_verifier)
        if receiver is not None:
            config["primary_receiver"] = publish_generation(
                state, Path(receiver["worker_executable"])
            )
        config["service_supervisor"] = publish_generation(state)
        config["resident"] = (
            args.resident
            if args.resident is not None
            else not bool(args.execution_profile)
            or args.execution_profile in {"functional", "wide"}
        )
        service_borrowers.service_registry_binding(state, establish=True)
        write_json(state / "config.json", config)
        (state / "tmp").mkdir(mode=0o700)
        payload_root(state, config).mkdir(mode=0o700, parents=True)
        # Published supervisor generations retain the selected already-built product artifact.
        write_json(state / "config.json", config)
    return public_status(state, config)


def reconfigure(
    state: Path, config: dict[str, object], args: argparse.Namespace
) -> None:
    """Replace only the allocation after proving the owned lifecycle is offline."""
    if config.get("backup_pending") is not None:
        raise SupervisorError(
            "Backup pending owner requires explicit resume before reconfiguration"
        )
    if any(
        value is not None
        for value in (
            args.native_workers,
            args.port,
            args.version,
            args.interpretation,
            args.tool_root,
        )
    ):
        raise SupervisorError(
            "reconfigure accepts memory options only; worker count and state identity are fixed"
        )
    if config["admission"] != "quiesced" or config["accepting_writes"]:
        raise SupervisorError(
            "Quiesce admission and stop the server before reconfigure"
        )
    prior = config["resources"]
    if not isinstance(prior, dict):
        raise SupervisorError("Invalid resource configuration")
    allocation = resources(
        integer(prior["total_memory_bytes"])
        if args.memory_mib is None
        else args.memory_mib * MIB,
        integer(prior["server_memory_bytes"])
        if args.server_memory_mib is None
        else args.server_memory_mib * MIB,
        integer(prior["native_workers"]),
        integer(prior["native_worker_memory_bytes"])
        if args.native_worker_memory_mib is None
        else args.native_worker_memory_mib * MIB,
        prior.get("execution"),
    )
    receiver = config.get("primary_receiver")
    if args.execution_profile:
        if any(
            value is not None
            for value in (
                args.memory_mib,
                args.server_memory_mib,
                args.native_worker_memory_mib,
            )
        ):
            raise SupervisorError(
                "Reference execution supplies exact capacities; memory overrides are refused"
            )
        allocation = execution_resources(args.execution_profile)
        receiver = primary_receiver(args.worker_executable)
    elif args.worker_executable is not None:
        raise SupervisorError(
            "Receiver selection requires an explicit execution profile"
        )
    # A budget can fit the process/worker envelope yet fail the selected native
    # module pool. Refuse it before any generation publication or config write.
    module_runtime_allocation(allocation)
    observed = systemctl(
        "show",
        "--property=ActiveState",
        "--property=ControlGroup",
        unit_name(state),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in observed.stdout.splitlines() if "=" in line
    )
    if (
        observed.returncode
        or fields.get("ActiveState") not in {"inactive", "failed"}
        or "ControlGroup" not in fields
        or group_populated(fields["ControlGroup"])
    ):
        raise SupervisorError("Server must be verified stopped before reconfigure")
    workers_drained(state, config)
    if args.execution_profile:
        receiver = publish_generation(
            state, Path(str(object_mapping(receiver)["worker_executable"]))
        )
    updated = dict(config)
    updated["resources"] = allocation
    if args.execution_profile:
        updated["service_class"] = (
            "timing" if args.execution_profile == "timing" else "functional"
        )
    if receiver is not None:
        updated["primary_receiver"] = receiver
    write_json(state / "config.json", updated)
    config.update(updated)


def systemd_environment() -> dict[str, str]:
    environment = os.environ.copy()
    runtime = Path(f"/run/user/{os.getuid()}")
    if (runtime / "bus").is_socket():
        environment.setdefault("XDG_RUNTIME_DIR", str(runtime))
        environment.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    return environment


def systemctl(
    *arguments: str,
    check: bool = True,
    cleanup_timeout: float | None = None,
    deadline: float | None = None,
) -> subprocess.CompletedProcess[str]:
    inherited = getattr(_STARTUP, "deadline", None)
    deadline = (
        inherited
        if deadline is None
        else deadline
        if inherited is None
        else min(deadline, inherited)
    )
    timeout = (
        cleanup_timeout
        if cleanup_timeout is not None
        else (60 if deadline is None else remaining(deadline))
    )
    result = subprocess.run(
        ["systemctl", "--user", *arguments],
        env=systemd_environment(),
        capture_output=True,
        text=True,
        timeout=timeout,
        check=False,
    )
    if deadline is not None and cleanup_timeout is None:
        remaining(deadline)
    if check and result.returncode:
        raise SupervisorError(
            "User systemd operation failed; a user manager is required for capped supervision"
        )
    return result


def unit_name(state: Path) -> str:
    state = service_directory(state)
    return f"pse-surreal-{hashlib.sha256(str(lexical_absolute(state)).encode()).hexdigest()[:16]}.service"


def active(state: Path) -> bool:
    return (
        systemctl("is-active", "--quiet", unit_name(state), check=False).returncode == 0
    )


def reset_failure_window(state: Path) -> None:
    """A collected inactive unit has no live failure window to reset."""
    unit = unit_name(state)
    result = systemctl("reset-failed", unit, check=False)
    if result.returncode:
        result = systemctl(
            "show",
            "--property=ActiveState",
            "--property=ControlGroup",
            unit,
            check=False,
        )
        observed = dict(
            line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
        )
        if (
            result.returncode
            or observed.get("ActiveState") not in {"inactive", "failed"}
            or group_populated(observed.get("ControlGroup", ""))
        ):
            raise SupervisorError(
                "Cannot reset an unresolved live service failure window"
            )


def worker_unit(state: Path, slot: int) -> str:
    """A slot has one persistent systemd name, including across launcher death."""
    return f"pse-surreal-worker-{hashlib.sha256(str(lexical_absolute(state)).encode()).hexdigest()[:16]}-{slot}.scope"


def group_populated(group: str) -> bool:
    if not group:
        return False
    try:
        return process_observation.populated(group)
    except ValueError as error:
        raise SupervisorError(
            "Cannot verify managed worker cgroup population"
        ) from error


def worker_observation(
    state: Path, slot: int, *, deadline: float | None = None
) -> dict[str, str]:
    result = systemctl(
        "show",
        "--property=LoadState",
        "--property=ActiveState",
        "--property=ControlGroup",
        "--property=MemoryMax",
        worker_unit(state, slot),
        check=False,
        deadline=deadline,
    )
    fields = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if result.returncode or any(
        key not in fields for key in ("LoadState", "ActiveState", "ControlGroup")
    ):
        raise SupervisorError(
            "Cannot verify managed worker slot; a user systemd manager is required"
        )
    return fields


def worker_busy(observation: dict[str, str]) -> bool:
    return observation["ActiveState"] not in {"inactive", "failed"} or group_populated(
        observation["ControlGroup"]
    )


def settled_worker(state: Path, slot: int) -> dict[str, str]:
    observation = worker_observation(state, slot)
    # A scope whose launcher died may remain loaded after its processes exit.
    # Only a proven empty kernel group can be collected before slot reuse.
    if (
        observation["ActiveState"] not in {"inactive", "failed"}
        and observation["ControlGroup"]
        and not group_populated(observation["ControlGroup"])
    ):
        systemctl("stop", worker_unit(state, slot))
        observation = worker_observation(state, slot)
    return observation


def workers_drained(state: Path, config: dict[str, object]) -> None:
    """Caller acknowledgment alone cannot establish a stopped process inventory."""
    systemctl("show-environment")
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    if "execution" in allocation and worker_busy(primary_observation(state)):
        raise SupervisorError(
            "Primary receiver remains active; quiesce and drain before offline copy"
        )
    for slot in range(integer(allocation["native_workers"])):
        if worker_busy(settled_worker(state, slot)):
            raise SupervisorError(
                "Managed native worker slots remain active; quiesce and drain before offline copy"
            )


def worker_environment(
    state: Path, slot: int, allocation: dict[str, object], handoff: Path | None = None
) -> dict[str, str]:
    environment = systemd_environment()
    from scripts import (  # noqa: PLC0415 -- optional shared passive launch selection.
        telemetry_launch,
    )

    telemetry_launch.configure(SCRIPT.parents[1], environment)
    # Never inherit the ordinary ad hoc workload's optional/off/default cap policy.
    environment.pop("PSE_MEMORY_MAX", None)
    environment.pop("PSE_NATIVE_OPERATION", None)
    environment.pop("PSE_NATIVE_HANDOFF", None)
    if handoff is not None:
        environment["PSE_NATIVE_HANDOFF"] = str(handoff)
    environment.update(
        {
            "PSE_SURREAL_STATE": str(state),
            "PSE_NATIVE_WORKER_SLOT": str(slot),
            "PSE_NATIVE_WORKER_MEMORY_BYTES": str(
                integer(allocation["native_worker_memory_bytes"])
            ),
        }
    )
    return environment


def primary_environment(
    state: Path,
    allocation: dict[str, object],
    receiver: dict[str, str],
    handoff: Path | None = None,
) -> tuple[dict[str, str], tuple[str, ...]]:
    """Carry role-specific inputs to the actual service; the worker validates them."""
    environment = worker_environment(state, 0, allocation, handoff)
    inputs = receiver_environment.read(
        Path(receiver["supervisor_script"]).parents[1] / "native-environment.json"
    )
    environment = receiver_environment.selection(
        inputs, WORKER_CAPABILITIES, environment
    )
    selected = environment.get("PSE_WORKER_BINARY")
    if selected is not None:
        try:
            actual = Path(selected).resolve(strict=True)
        except OSError as error:
            raise SupervisorError(
                "Selected worker requires offline receiver readmission"
            ) from error
        if file_digest(actual) != receiver["worker_sha256"]:
            raise SupervisorError(
                "Selected worker differs from the configured receiver; drain and readmit it offline"
            )
    # A Python caller's generic receipt is its own role. A service must neither
    # inherit it nor fall back to a stale user-manager generic receipt.
    environment.pop("PSE_PRODUCER_RECEIPT", None)
    admitted = config_for(state).get("worker_producer_receipt")
    if isinstance(admitted, dict):
        frozen = Path(str(admitted["path"]))
        if file_digest(frozen) != admitted["sha256"]:
            raise SupervisorError("Immutable worker producer receipt changed")
        environment["PSE_WORKER_PRODUCER_RECEIPT"] = str(frozen)
    receipt, _ = selected_worker_receipt(environment)
    if receipt is not None:
        environment["PSE_PRODUCER_RECEIPT"] = receipt
    capabilities = WORKER_CAPABILITIES
    names = tuple(
        dict.fromkeys(
            (
                "PSE_PRODUCER_RECEIPT",
                "PSE_NATIVE_OPERATION",
                "PSE_NATIVE_HANDOFF",
                "PSE_NATIVE_CACHE",
                "PSE_HOST_ALLOCATION",
                "PSE_ADMISSION_DEADLINE",
                "PSE_RESOURCE_CLASS",
                "PSE_TEST_EXECUTION_PROFILE",
                "PSE_NATIVE_SETUP_PYTHON",
                "PSE_SOLVER_IMAGE",
                "PSE_NATIVE_PROVIDER_RECEIPT",
                "SYMBOLICA_LICENSE",
                "PSE_NATIVE_TELEMETRY_SETTINGS",
                "LD_LIBRARY_PATH",
                "PSE_LLVM_PREFIX",
                "LIBCLANG_PATH",
                native_operation.OFF_MARKER,
                *native_operation.OVERRIDABLE,
                *receiver_environment.variable_names(capabilities),
                *(
                    name
                    for capability in capabilities
                    for name in native_operation.CAPABILITY_PATHS[capability]
                ),
            )
        )
    )
    return environment, names


def selected_worker_receipt(
    environment: dict[str, str],
) -> tuple[str | None, str | None]:
    selected = environment.get("PSE_WORKER_PRODUCER_RECEIPT")
    if selected is None:
        return None, None
    if not selected:
        raise SupervisorError("An explicit worker producer receipt must name a file")
    try:
        path = Path(selected).resolve(strict=True)
        return str(path), file_digest(path)
    except OSError as error:
        raise SupervisorError(
            "Cannot associate the selected worker producer receipt"
        ) from error


def primary_receipt_ready(
    process: Path, marker: dict[str, object], environment: dict[str, str]
) -> bool:
    """Associate actual receiving inputs; header/artifact admission stays in the worker."""
    selected, digest = selected_worker_receipt(environment)
    assignments = (process / "environ").read_bytes().split(b"\0")
    generic = [
        item.removeprefix(b"PSE_PRODUCER_RECEIPT=")
        for item in assignments
        if item.startswith(b"PSE_PRODUCER_RECEIPT=")
    ]
    if selected is None:
        return (
            not generic
            and marker.get("producer_receipt_path") is None
            and marker.get("producer_receipt_sha256") is None
        )
    return (
        generic == [os.fsencode(selected)]
        and marker.get("producer_receipt_path") == selected
        and marker.get("producer_receipt_sha256") == digest
    )


def primary_service_environment(
    environment: dict[str, str], names: tuple[str, ...]
) -> list[str]:
    """Use caller values without putting sensitive values in the command line."""
    arguments = [f"--setenv={name}" for name in names if name in environment]
    absent = [name for name in names if name not in environment]
    if absent:
        arguments.append("--property=UnsetEnvironment=" + " ".join(absent))
    return arguments


def placement_slice() -> list[str]:
    """Workers and servers launch their own units, so they name the agent slice too."""
    root = str(Path(__file__).resolve().parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)
    from scripts import pse_env  # noqa: PLC0415 -- after root selection

    selected = pse_env.slice_name(os.environ)
    return [] if selected is None else [f"--slice={selected}"]


def execution_slice(state: Path) -> str:

    placement = placement_observation(state)
    if placement is not None and placement.get("allocation_slice"):
        return str(placement["allocation_slice"])
    allocation = inherit_host_owner()
    if allocation is None:
        raise SupervisorError("Scientific placement requires an admitted host owner")
    return host_admission.allocation_slice(allocation)


def physical_cpus(count: int) -> list[int]:
    """Select declared physical cores independently of the launcher's affinity."""
    allocation = inherit_host_owner()
    if allocation is None:
        raise SupervisorError("Scientific role requires an actual host allocation")
    cores = list(allocation.profile.cores)
    host_admission.cpu_set(cores)  # Validate this machine's physical topology.
    if len(cores) < count:
        raise SupervisorError("Admitted host lane cannot fit the scientific CPU width")
    return cores[:count]


def group_for_slice(name: str) -> Path:
    root = str(SCRIPT.parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)
    from scripts import pse_env  # noqa: PLC0415 -- existing placement owner

    return pse_env.slice_group(name)


def effective_limits(group: Path) -> tuple[int | None, float | None]:
    memory = None
    cpu = None
    root = Path("/sys/fs/cgroup")
    while group != root and group.name:
        memory_path = group / "memory.max"
        if memory_path.is_file():
            value = memory_path.read_text().strip()
            if value != "max":
                memory = min(memory, int(value)) if memory is not None else int(value)
        cpu_path = group / "cpu.max"
        if cpu_path.is_file():
            quota, period = cpu_path.read_text().split()
            if quota != "max":
                value = int(quota) / int(period)
                cpu = min(cpu, value) if cpu is not None else value
        group = group.parent
    return memory, cpu


def host_memory() -> tuple[int, int]:
    fields = dict(
        line.split(":", 1)
        for line in Path("/proc/meminfo").read_text().splitlines()
        if ":" in line
    )
    return int(fields["MemTotal"].split()[0]) * 1024, int(
        fields["MemAvailable"].split()[0]
    ) * 1024


def placement_observation(state: Path) -> dict[str, object] | None:
    path = state / "execution-placement.json"
    return read_json(path) if path.is_file() else None


def ensure_execution_placement(state: Path, allocation: dict[str, object]) -> None:
    execution = allocation.get("execution")
    if not isinstance(execution, dict):
        return

    owner = inherit_host_owner()
    if owner is None:
        raise SupervisorError("Scientific placement requires actual host ownership")
    width = integer(execution["cpu_threads"])
    cpus = physical_cpus(width)
    deadline = getattr(_STARTUP, "deadline", None)
    try:
        host_admission.enforce_allocation(
            owner, systemd_environment(), deadline=deadline
        )
    except host_admission.AdmissionError:
        if deadline is not None:
            remaining(deadline)
        raise
    group = group_for_slice(host_admission.allocation_slice(owner))
    if owner.profile.unbounded:
        if allocation == reference_resources():
            raise SupervisorError(
                "Reference execution requires its finite isolated allocation"
            )
        pse_env.require_unbounded_slice(host_admission.allocation_slice(owner))
        total = None
    else:
        total = int((group / "memory.max").read_text().strip())
    if total is not None and (
        integer(allocation["native_worker_memory_bytes"])
        + integer(execution["observer_memory_bytes"])
        > total
    ):
        raise SupervisorError("Scientific roles exceed their originating allocation")
    physical, available = host_memory()
    current = group / "memory.current"
    owned = int(current.read_text()) if current.is_file() else 0
    observed_memory, observed_cpu = effective_limits(group)
    if observed_memory != total or observed_cpu is None or observed_cpu < width:
        raise SupervisorError(
            "Execution placement readback differs from the exact reference envelope"
        )
    effective = group / "cpuset.cpus.effective"
    if effective.is_file() and not set(cpus) <= cpu_list(effective.read_text().strip()):
        raise SupervisorError(
            "Execution physical CPU placement readback differs from its selection"
        )
    # User managers may have cpu/memory delegation without cpuset. Every role
    # also starts with inherited scheduler affinity, verified on its actual PID.
    # A recorded AllowedCPUs property alone never establishes enforcement.
    if shutil.which("taskset") is None:
        raise SupervisorError("Reference role placement requires taskset")
    # Caps are ceilings, not upfront physical reservations or measured demand.
    if deadline is not None:
        remaining(deadline)
    write_json(
        state / "execution-placement.json",
        {
            "allocation_slice": host_admission.allocation_slice(owner),
            "allocation": str(owner.directory / owner.nonce),
            "host_memory_bytes": physical,
            "host_available_bytes": available,
            "owned_resident_bytes": owned,
            "available_plus_owned_bytes": available + owned,
            "memory_max_bytes": observed_memory,
            "cpu_quota_cores": observed_cpu,
            "physical_cpus": cpus,
            "cpu_placement": "cpuset-and-process-affinity"
            if effective.is_file()
            else "process-affinity",
            "physically_reserved": False,
        },
    )


def cpu_list(value: str) -> set[int]:
    result: set[int] = set()
    for item in value.split(","):
        lower, separator, upper = item.partition("-")
        result.update(range(int(lower), int(upper) + 1) if separator else [int(lower)])
    return result


def selected_slice(state: Path, allocation: dict[str, object]) -> list[str]:
    if "execution" in allocation:
        return [f"--slice={execution_slice(state)}"]

    owner = inherit_host_owner()
    if owner is None:
        raise SupervisorError(
            "Managed role requires an admitted originating host allocation"
        )
    return [f"--slice={host_admission.allocation_slice(owner)}"]


def role_command(allocation: dict[str, object], command: list[str]) -> list[str]:
    """Set affinity before the role creates children or native worker threads."""
    execution = allocation.get("execution")
    if not isinstance(execution, dict):
        return command
    executable = shutil.which("taskset")
    if executable is None:
        raise SupervisorError("Reference role placement requires taskset")
    cpus = physical_cpus(integer(execution["cpu_threads"]))
    return [executable, "--cpu-list", ",".join(map(str, cpus)), *command]


def role_affinity_ready(
    pid: int, execution: dict[str, object], cpus: list[int] | None = None
) -> bool:
    try:
        return observed_role_affinity(pid, execution, cpus)
    except (OSError, ValueError, SupervisorError):
        return False


def observed_role_affinity(
    pid: int, execution: dict[str, object], cpus: list[int] | None = None
) -> bool:
    """Read the actual leader and every current thread's kernel affinity."""
    allowed = set(
        physical_cpus(integer(execution["cpu_threads"])) if cpus is None else cpus
    )
    if os.sched_getaffinity(pid) != allowed:
        return False
    return all(
        bool(actual := os.sched_getaffinity(int(thread.name))) and actual <= allowed
        for thread in (Path("/proc") / str(pid) / "task").iterdir()
    )


def worker_scope_command(
    state: Path,
    slot: int,
    allocation: dict[str, object],
    command: list[str],
    *,
    capabilities: tuple[str, ...] = WORKER_CAPABILITIES,
) -> list[str]:
    return [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={worker_unit(state, slot)}",
        *selected_slice(state, allocation),
        f"--property=MemoryMax={integer(allocation['native_worker_memory_bytes'])}",
        "--property=MemorySwapMax=0",
        "--property=TasksMax=2048"
        if "execution" in allocation
        else "--property=TasksMax=128",
        "--",
        *role_command(
            allocation,
            [
                str(Path(__file__).resolve().parents[1] / ".venv/bin/python"),
                str(Path(__file__).resolve().with_name("native_operation.py")),
                "--capabilities",
                ",".join(capabilities),
                "--",
                *command,
            ],
        ),
    ]


def worker(
    state: Path,
    command: list[str],
    *,
    capabilities: tuple[str, ...] = WORKER_CAPABILITIES,
    deadline: float | None = None,
) -> int:
    # Move startup's existing budget ahead of worker admission. It does not
    # bound the admitted child's healthy execution or its owned cleanup.
    previous = getattr(_STARTUP, "deadline", None)
    deadline = time.monotonic() + 40 if deadline is None else deadline
    _STARTUP.deadline = deadline if previous is None else min(previous, deadline)
    try:
        return _worker(state, command, capabilities=capabilities)
    finally:
        _STARTUP.deadline = previous


def _worker(state: Path, command: list[str], *, capabilities: tuple[str, ...]) -> int:
    if not command:
        raise SupervisorError(
            "worker requires --worker-command followed by an executable and arguments"
        )
    config_for(state)
    state = checked_directory(state)
    # Observe systemd outside exclusion, then reserve only the selected metadata.
    config = config_for(state)
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    systemctl("show-environment")
    ensure_execution_placement(state, allocation)
    if "execution" in allocation and worker_busy(primary_observation(state)):
        raise SupervisorError(
            "The primary process already owns the shared reference allocation"
        )
    available = [
        n
        for n in range(integer(allocation["native_workers"]))
        if not worker_busy(settled_worker(state, n))
    ]
    token = uuid.uuid4().hex

    if any(
        capability not in native_operation.CAPABILITIES for capability in capabilities
    ):
        raise SupervisorError("Unknown managed worker native capability")
    with context_admission(state):
        config = config_for(state)
        if config["admission"] != "open" or not config["accepting_writes"]:
            raise SupervisorError("Worker admission is closed")
        if config["resources"] != allocation:
            raise SupervisorError("Worker allocation changed during admission")
        primary_pending = state / "primary-admission.json"
        if (
            "execution" in allocation
            and primary_pending.exists()
            and reservation_live(read_json(primary_pending))
        ):
            raise SupervisorError(
                "Primary receiver admission already owns this allocation"
            )
        slot = next(
            (
                n
                for n in available
                if not (state / f"worker-admission-{n}.json").exists()
                or not reservation_live(read_json(state / f"worker-admission-{n}.json"))
            ),
            None,
        )
        if slot is None:
            raise SupervisorError("All configured native worker slots are occupied")
        pending = state / f"worker-admission-{slot}.json"
        remaining(_STARTUP.deadline)
        write_json(
            pending,
            {
                "nonce": token,
                "pid": os.getpid(),
                "start": operation_lifetime.start_identity(os.getpid()),
                "unit": worker_unit(state, slot),
                "boot": host_admission.boot(),
            },
        )
    child: subprocess.Popen[bytes] | None = None
    registered = False
    launcher_result: int | None = None
    cleanup_complete = False
    cleanup_cause = ""
    host_owner: host_admission.Allocation | None = None
    handoff: Path | None = None
    predecessor: dict[str, object] = {}
    cleanup_binding: dict[str, object] = {}
    try:
        # Once reserved, an offline operation refuses this pending launch rather
        # than racing its process registration. No metadata lock crosses IPC.
        if worker_busy(settled_worker(state, slot)) or (
            "execution" in allocation and worker_busy(primary_observation(state))
        ):
            raise SupervisorError(  # noqa: TRY301 -- owned-launch failures must enter registration cleanup
                "Selected worker allocation became occupied during admission"
            )
        start(state, config)
        # Pin the launch window before crossing into an independent worker scope.
        # The common child owner binds this guard using actual kernel membership.
        root = str(Path(__file__).resolve().parents[1])
        if root not in sys.path:
            sys.path.insert(0, root)

        try:
            handoff = native_operation.prepare_handoff(
                worker_unit(state, slot),
                reservation=(pending, token),
                deadline=_STARTUP.deadline,
            )
        except operation_lifetime.DeadlineExpiredError as error:
            raise DeadlineExpiredError(
                f"Worker handoff exceeded its original admission clock; selected={worker_unit(state, slot)}"
            ) from error

        host_owner = register_role_owner(worker_unit(state, slot), _STARTUP.deadline)
        predecessor = registration_predecessor(
            worker_unit(state, slot), host_owner, deadline=_STARTUP.deadline
        )
        remaining(_STARTUP.deadline)
        child = subprocess.Popen(
            worker_scope_command(
                state, slot, allocation, command, capabilities=capabilities
            ),
            env=worker_environment(state, slot, allocation, handoff),
        )
        deadline = min(time.monotonic() + 10, _STARTUP.deadline)
        while child.poll() is None:
            observation = worker_observation(state, slot, deadline=deadline)
            if observation["ActiveState"] == "active":
                if observation.get("MemoryMax") != str(
                    allocation["native_worker_memory_bytes"]
                ):
                    raise SupervisorError(  # noqa: TRY301 -- owned-launch failures must enter registration cleanup
                        "Managed worker memory cap differs from its configured allocation"
                    )
                registered = True
                break
            if time.monotonic() >= deadline:
                raise DeadlineExpiredError("Managed worker scope did not register")  # noqa: TRY301 -- owned-launch failures must enter registration cleanup
            time.sleep(0.02)
        if not registered:
            # A short payload can finish before the first active observation.
            # Reconcile this launch's exact completed generation, preserving the
            # systemd-run status rather than manufacturing a registration error.
            launcher_result = child.wait()
            if host_owner is None:
                raise SupervisorError("Completed worker has no admitted host owner")  # noqa: TRY301 -- retain ownership through registration cleanup
            stop_registered_owner(
                worker_unit(state, slot),
                host_owner,
                predecessor=predecessor,
                handoff=handoff,
                stop=False,
                evidence=cleanup_binding,
            )
            cleanup_complete = True
    except BaseException as failure:
        if child is not None:
            # Registration failure and owned cleanup have distinct lifetimes.
            # Joining this exact launcher excludes further caller dispatch; its
            # exit alone never proves that the independent scope drained.
            admission_deadline = getattr(_STARTUP, "deadline", None)
            _STARTUP.deadline = None
            try:
                if child.poll() is None:
                    child.kill()
                if launcher_result is None:
                    launcher_result = child.wait()
                if host_owner is None:
                    raise SupervisorError("Worker cleanup has no admitted host owner")  # noqa: TRY301 -- owned-launch failures must enter registration cleanup
                stop_registered_owner(
                    worker_unit(state, slot),
                    host_owner,
                    predecessor=predecessor,
                    handoff=handoff,
                    evidence=cleanup_binding,
                )
                cleanup_complete = True
            except (
                OSError,
                KeyError,
                ValueError,
                SupervisorError,
                host_admission.AdmissionError,
                subprocess.SubprocessError,
            ) as cleanup_error:
                cleanup_cause = f"{type(cleanup_error).__name__}: {cleanup_error}"
                failure.add_note(
                    f"Worker cleanup unresolved; selected={worker_unit(state, slot)}; admission reservation retained: {cleanup_cause}"
                )
            finally:
                _STARTUP.deadline = admission_deadline
        raise
    finally:
        with state_lock(state, cleanup=True):
            if pending.exists() and read_json(pending).get("nonce") == token:
                if child is None or registered or cleanup_complete:
                    pending.unlink()
                else:
                    retained = read_json(pending)
                    retained.update(
                        unresolved=True,
                        cleanup_cause=cleanup_cause,
                        predecessor=predecessor,
                        binding=cleanup_binding or None,
                        handoff=str(handoff) if handoff is not None else None,
                        allocation=str(host_owner.directory / host_owner.nonce)
                        if host_owner is not None
                        else None,
                    )
                    write_json(pending, retained)
    if child is None:
        raise SupervisorError("Worker launcher was not created")
    result = child.wait() if launcher_result is None else launcher_result
    return result if result >= 0 else 128 - result


def inherit_host_owner(
    deadline: float | None = None,
) -> host_admission.Allocation | None:
    """Consume the same role/startup clock through supported host observation."""
    inherited = getattr(_STARTUP, "deadline", None)
    deadline = (
        inherited
        if deadline is None
        else deadline
        if inherited is None
        else min(deadline, inherited)
    )
    try:
        return host_admission.inherit(
            os.environ, deadline=deadline, services=ServiceAdmission()
        )
    except host_admission.AdmissionError:
        if deadline is not None:
            remaining(deadline)
        raise


def register_role_owner(unit: str, deadline: float) -> host_admission.Allocation:
    """Join and register the originating host owner on the offered clock."""
    try:
        owner = inherit_host_owner(deadline)
        if owner is None:
            raise SupervisorError(
                f"Managed native role requires an admitted host allocation; selected={unit}"
            )
        owner.register(unit, deadline=deadline)
    except host_admission.AdmissionError:
        # Preserve supervisor's typed original-clock failure when host metadata
        # reports timeout; other ownership refusals retain their own cause.
        remaining(deadline)
        raise
    return owner


def registration_predecessor(
    unit: str, host_owner: host_admission.Allocation, *, deadline: float
) -> dict[str, object]:
    """Snapshot before dispatch; register() intentionally preserves old bindings."""
    ledger = host_admission.readonly_snapshot(host_owner.directory, deadline=deadline)
    owner = ledger["owners"].get(host_owner.nonce)
    if not isinstance(owner, dict):
        raise SupervisorError("Registration allocation is unavailable before dispatch")
    return dict(object_mapping(owner["units"].get(unit, {})))


def reconcile_registration(state: Path, pending: Path, *, deadline: float) -> None:
    """Retry an unresolved launch through the existing offline drain owner."""
    selected = read_json(pending, deadline=deadline)
    if (
        selected.get("unresolved") is not True
        or selected.get("boot") != host_admission.boot()
    ):
        return
    unit = selected.get("unit")
    if unit not in {
        primary_unit(state),
        *(
            worker_unit(state, n)
            for n in range(
                integer(
                    object_mapping(config_for(state)["resources"])["native_workers"]
                )
            )
        ),
    }:
        raise SupervisorError("Retained registration has an invalid selected unit")
    marker = selected.get("allocation")
    if not isinstance(marker, str):
        raise SupervisorError("Retained registration has no allocation owner")
    allocation = Path(marker)
    if not allocation.is_absolute() or not re.fullmatch(
        r"[a-f0-9]{32}", allocation.name
    ):
        raise SupervisorError("Retained registration allocation is invalid")
    predecessor = selected.get("predecessor")
    if not isinstance(predecessor, dict):
        raise SupervisorError("Retained registration lacks its prelaunch generation")
    handoff = selected.get("handoff")
    binding = selected.get("binding")
    if binding is None and isinstance(handoff, str):
        try:
            proof = read_json(Path(handoff), deadline=deadline)
        except FileNotFoundError as error:
            raise SupervisorError(
                f"Retained registration fresh handoff is unavailable; selected={unit}"
            ) from error
        scope = proof.get("scope")
        if (
            proof.get("registration")
            == {"path": str(pending), "nonce": selected.get("nonce")}
            and proof.get("handoff") == unit
            and isinstance(scope, dict)
            and scope.get("unit") == unit
            and type(proof.get("scope_inode")) is int
        ):
            binding = {**scope, "inode": proof["scope_inode"]}

    def retain(binding: dict[str, object]) -> None:
        nonlocal selected
        if selected.get("binding") == binding:
            return
        with state_lock(state, deadline=deadline):
            if (
                not pending.exists()
                or read_json(pending, deadline=deadline) != selected
            ):
                raise SupervisorError("Retained registration changed during recovery")
            updated = {**selected, "binding": dict(binding)}
            remaining(deadline)
            write_json(pending, updated)
            selected = updated

    stop_registered_owner(
        str(unit),
        allocation,
        predecessor=predecessor,
        handoff=Path(handoff) if isinstance(handoff, str) else None,
        deadline=deadline,
        retained_binding=binding,
        retain=retain,
    )
    with state_lock(state, deadline=deadline):
        if pending.exists() and read_json(pending, deadline=deadline) == selected:
            remaining(deadline)
            pending.unlink()


def stop_registered_owner(
    unit: str,
    host_owner: host_admission.Allocation | Path,
    *,
    predecessor: dict[str, object],
    handoff: Path | None,
    deadline: float | None = None,
    stop: bool = True,
    evidence: dict[str, object] | None = None,
    retained_binding: object = None,
    retain: Callable[[dict[str, object]], None] | None = None,
) -> None:
    """Settle this fresh launch, never a predecessor's already completed binding."""
    marker = (
        host_owner
        if isinstance(host_owner, Path)
        else host_owner.directory / host_owner.nonce
    )
    with host_admission.allocation_metadata(marker.parent, deadline=deadline) as ledger:
        owner = ledger["owners"].get(marker.name)
        if not isinstance(owner, dict):
            if not isinstance(retained_binding, dict):
                raise SupervisorError(
                    "Registration cleanup allocation retired without retained fresh binding"
                )
            binding = dict(retained_binding)
        else:
            binding = {"unit": unit, **object_mapping(owner["units"][unit])}
            if retained_binding is not None and binding != retained_binding:
                raise SupervisorError(
                    "Registration cleanup allocation binding differs from retained launch"
                )
    if (
        binding.get("unit") != unit
        or not isinstance(binding.get("invocation"), str)
        or not re.fullmatch(r"[a-f0-9]{32}", str(binding.get("invocation", "")))
        or not isinstance(binding.get("group"), str)
        or type(binding.get("inode")) is not int
        or binding.get("invocation") == predecessor.get("invocation")
    ):
        raise SupervisorError("Registration cleanup has no fresh bound invocation")
    if retained_binding is not None:
        if not isinstance(retained_binding, dict) or binding != retained_binding:
            raise SupervisorError("Retained registration binding is invalid")
    elif handoff is not None:
        # Handoff writes publish atomically; a detached read never waits behind
        # child binding/cancellation while this cleanup clock is expiring.
        record = read_json(handoff, deadline=deadline)
        scope = record.get("scope")
        if (
            record.get("handoff") != unit
            or not isinstance(scope, dict)
            or scope
            != {
                "unit": unit,
                "group": binding.get("group"),
                "invocation": binding.get("invocation"),
            }
        ):
            raise SupervisorError(
                "Registration cleanup fresh handoff is unbound or differs from charged generation"
            )
    if evidence is not None:
        # Only an authenticated fresh generation can outlive ledger retirement.
        evidence.clear()
        evidence.update(binding)
    if retain is not None:
        retain(binding)
    observed = operation_lifetime.unit_observation(unit, deadline=deadline)
    inode = host_admission.group_identity(str(binding.get("group", "")))
    if (
        isinstance(binding.get("group"), str)
        and isinstance(binding.get("invocation"), str)
        and type(binding.get("inode")) is int
        and observed.get("LoadState") == "not-found"
        and inode in {None, binding["inode"]}
    ):
        settled = operation_lifetime.drained(
            {
                "scope": {
                    "unit": unit,
                    "group": binding["group"],
                    "invocation": binding["invocation"],
                }
            },
            deadline=deadline,
        )
        if settled.complete:
            return
        raise SupervisorError(settled.diagnostic())
    if not stop:
        if (
            observed.get("InvocationID") != binding.get("invocation")
            or observed.get("ControlGroup") not in {binding.get("group"), ""}
            or inode not in {None, binding.get("inode")}
        ):
            raise SupervisorError(
                "Completed registration generation differs from selected owner"
            )
        completion = operation_lifetime.drained(
            {
                "scope": {
                    "unit": unit,
                    "group": binding["group"],
                    "invocation": binding["invocation"],
                }
            },
            deadline=deadline,
        )
        if not completion.complete:
            raise SupervisorError(completion.diagnostic())
        return
    if (
        not isinstance(binding.get("group"), str)
        or not isinstance(binding.get("invocation"), str)
        or type(binding.get("inode")) is not int
        or observed.get("ControlGroup") != binding["group"]
        or observed.get("InvocationID") != binding["invocation"]
        or inode != binding["inode"]
    ):
        raise SupervisorError(
            f"Registration cleanup invocation/group/inode is unresolved or changed; selected={binding}; observed={observed}; inode={inode}"
        )
    if stop:
        systemctl("stop", unit, deadline=deadline)
    drained = operation_lifetime.drained(
        {
            "scope": {
                "unit": unit,
                "group": binding["group"],
                "invocation": binding["invocation"],
            }
        },
        deadline=deadline,
    )
    if not drained.complete:
        raise SupervisorError(drained.diagnostic())
    if host_admission.group_identity(binding["group"]) not in {None, binding["inode"]}:
        raise SupervisorError(
            "Registration cleanup group generation changed during drain"
        )


def primary_unit(state: Path) -> str:
    return worker_unit(state, 0).removesuffix(".scope") + ".service"


def observer_scope_command(
    state: Path,
    allocation: dict[str, object],
    unit: str,
    command: list[str],
    *,
    unbounded: bool = False,
) -> list[str]:
    execution = allocation.get("execution")
    if not isinstance(execution, dict) or not command:
        raise SupervisorError(
            "Observer requires a selected execution profile and command"
        )
    if unbounded:
        owner = inherit_host_owner()
        if (
            owner is None
            or not owner.profile.unbounded
            or allocation != execution_resources("exclusive-observer")
        ):
            raise SupervisorError(
                "Unbounded observer requires its admitted ordinary build owner"
            )
        from scripts.pse_env import (  # noqa: PLC0415 -- placement owner
            current_memory_ceiling,
            require_unbounded_slice,
        )

        require_unbounded_slice(host_admission.allocation_slice(owner))
        if current_memory_ceiling() is not None:
            raise SupervisorError(
                "Ordinary build observer has a finite originating role cap"
            )
    return [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={unit}",
        *selected_slice(state, allocation),
        *(
            []
            if unbounded
            else [
                f"--property=MemoryMax={integer(execution['observer_memory_bytes'])}",
                "--property=MemorySwapMax=0",
            ]
        ),
        "--property=TasksMax=2048",
        "--",
        *role_command(
            allocation,
            [
                sys.executable,
                str(SCRIPT.with_name("native_operation.py")),
                "--capabilities",
                "solver,klu,isolation,uno,petsc",
                "--",
                *command,
            ],
        ),
    ]


def observer_launch_busy(state: Path) -> bool:
    path = state / "observer-launch.json"
    if not path.exists():
        return False
    launch = read_json(path)
    process = Path(f"/proc/{integer(launch['pid'])}/stat")
    if (
        process.exists()
        and process_observation.start_identity(integer(launch["pid"]))
        == launch["start"]
    ):
        return True
    result = systemctl(
        "show", "--property=ControlGroup", str(launch["unit"]), check=False
    )
    group = next(
        (
            line.removeprefix("ControlGroup=")
            for line in result.stdout.splitlines()
            if line.startswith("ControlGroup=")
        ),
        "",
    )
    return bool(group and group_populated(group))


def reference_state(selected: Path) -> Path:
    """Demand the explicit unchanged reference store before observer admission."""
    config_for(selected)
    selected = service_directory(selected)
    config = config_for(selected)
    if config["resources"] == reference_resources():
        state = selected
    else:
        state_home = Path(
            os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state"))
        )
        state = state_home / "pse-arrow/surreal-reference-v2"
    if not (state / "config.json").exists():
        worker = os.environ.get("PSE_WORKER_BINARY")
        if not worker:
            raise SupervisorError(
                "Reference setup requires an explicitly qualified PSE_WORKER_BINARY"
            )
        args = parser().parse_args(
            [
                "setup",
                "--state",
                str(state),
                "--port",
                "18241",
                "--execution-profile",
                "reference",
                "--worker-executable",
                worker,
                "--interpretation",
                SUBSTRATE_INTERPRETATION,
            ]
        )
        setup(args)
    config = config_for(state)
    if config["resources"] != reference_resources():
        raise SupervisorError(
            "Selected reference service must retain the exact reference allocation"
        )
    # Explicit reference demand uses the ordinary owned startup boundary. It
    # preserves restored validation, active-unit readiness and parked guards.
    start(state, config)
    return state


class ObserverControlScope:
    """The exact caller lifetime and finite cap borrowed by one observer."""

    __slots__ = (
        "group",
        "inode",
        "invocation",
        "memory",
        "nonce",
        "observer",
        "temporary_memory",
        "unit",
    )

    def __init__(
        self,
        unit: str,
        group: str,
        invocation: str,
        inode: int,
        observer: str,
    ) -> None:
        self.unit = unit
        self.group = group
        self.invocation = invocation
        self.inode = inode
        self.memory = 0
        self.temporary_memory = 0
        self.nonce = uuid.uuid4().hex
        self.observer = observer


def observer_control_reservation(
    owner: host_admission.Allocation,
    scope: ObserverControlScope,
    action: str,
    *,
    deadline: float | None = None,
) -> None:
    """Exclude overlapping cap borrowers without holding a lock through execution."""
    expected = {
        "owner": owner.nonce,
        "nonce": scope.nonce,
        "observer": scope.observer,
        "group": scope.group,
        "invocation": scope.invocation,
        "inode": scope.inode,
    }
    path = owner.directory / "observer-control-borrows.json"
    with host_admission.allocation_metadata(
        owner.directory, deadline=deadline
    ) as ledger:
        current = ledger["owners"].get(owner.nonce)
        if current is None:
            raise SupervisorError("Observer host allocation is no longer live")
        registered = object_mapping(
            object_mapping(current.get("units")).get(scope.unit)
        )
        if registered != {
            "group": scope.group,
            "invocation": scope.invocation,
            "inode": scope.inode,
        }:
            raise SupervisorError("Observer registered control identity changed")
        if path.is_symlink():
            raise SupervisorError("Unsafe observer control borrow metadata")
        reservations = read_json(path) if path.exists() else {}
        if action == "claim":
            if scope.unit in reservations:
                raise SupervisorError(
                    "Observer control scope already has a cap borrower"
                )
            reservations[scope.unit] = expected
        elif reservations.get(scope.unit) != expected:
            raise SupervisorError("Observer control borrow identity changed")
        elif action == "release":
            del reservations[scope.unit]
        elif action != "verify":
            raise SupervisorError("Unknown observer control borrow action")
        if action != "verify":
            write_json(path, reservations)


def observer_control_observation(
    scope: ObserverControlScope, *, deadline: float | None = None
) -> dict[str, str]:
    """Refuse to change a replaced unit or a caller outside its original group."""
    result = systemctl(
        "show",
        "--property=LoadState,ActiveState,ControlGroup,InvocationID,MemoryMax",
        scope.unit,
        deadline=deadline,
    )
    observed = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    caller = operation_lifetime.process_group(os.getpid())
    if (
        observed.get("LoadState") != "loaded"
        or observed.get("ActiveState") != "active"
        or observed.get("ControlGroup") != scope.group
        or observed.get("InvocationID") != scope.invocation
        or host_admission.group_identity(scope.group) != scope.inode
        or not (caller == scope.group or caller.startswith(scope.group + "/"))
    ):
        raise SupervisorError("Observer control scope identity changed")
    return observed


def observer_allocation_drained(
    owner: host_admission.Allocation, unit: str, *, deadline: float | None = None
) -> bool:
    """Use the existing registered kernel lifetime, never guess an empty launch."""
    with host_admission.allocation_metadata(
        owner.directory, deadline=deadline
    ) as ledger:
        current = ledger["owners"].get(owner.nonce)
        if current is None:
            raise SupervisorError("Observer host allocation is no longer live")
        record = dict(current)
        units = object_mapping(current.get("units"))
        bound = object_mapping(units.get(unit, {}))
        group = bound.get("group")
        invocation = bound.get("invocation")
        if (
            not isinstance(group, str)
            or not group
            or not isinstance(invocation, str)
            or not re.fullmatch(r"[a-f0-9]{32}", invocation)
            or type(bound.get("inode")) is not int
        ):
            return False
        record["units"] = {unit: dict(bound)}
        record["released"] = True
        # This observer's physical lifetime must settle before its parent can
        # compose and acknowledge the selected terminal. The enclosing host
        # owner still requires every terminal guard when releasing its charge.
        record.pop("terminal_guards", None)
    return host_admission.drained(
        record, deadline=deadline, services=ServiceAdmission()
    )


@contextlib.contextmanager
def observer_control_limit(
    owner: host_admission.Allocation,
    unit: str,
    profile: str | None,
    *,
    launch_attempted: Callable[[], bool] | None = None,
    cancellation_deadline: Callable[[], float | None] | None = None,
) -> Generator[None, None, None]:
    """Borrow the caller's control cap only for the observer's verified lifetime."""
    if owner.profile.unbounded:
        if profile != "exclusive-observer":
            raise SupervisorError(
                "Unbounded observer control requires the ordinary test profile"
            )
        from scripts.pse_env import (  # noqa: PLC0415 -- placement owner
            current_memory_ceiling,
            require_unbounded_slice,
        )

        require_unbounded_slice(host_admission.allocation_slice(owner))
        if current_memory_ceiling() is not None:
            raise SupervisorError(
                "Unbounded build observer control has a finite role cap"
            )
        yield
        return
    deadline = cancellation_deadline() if cancellation_deadline is not None else None
    scope = None
    if profile is not None and profile != "reference":
        caller = operation_lifetime.process_group(os.getpid())
        with host_admission.allocation_metadata(
            owner.directory, deadline=deadline
        ) as ledger:
            units = dict(object_mapping(ledger["owners"][owner.nonce].get("units")))
        for control, value in units.items():
            registered = object_mapping(value)
            group = registered.get("group", "")
            if not isinstance(group, str):
                raise SupervisorError("Observer control group is invalid")
            if group and (caller == group or caller.startswith(group + "/")):
                invocation = registered.get("invocation")
                inode = registered.get("inode")
                if (
                    not isinstance(invocation, str)
                    or not re.fullmatch(r"[a-f0-9]{32}", invocation)
                    or not isinstance(inode, int)
                    or isinstance(inode, bool)
                ):
                    raise SupervisorError("Observer control scope is not bound")
                scope = ObserverControlScope(control, group, invocation, inode, unit)
                break
        if scope is None:
            raise SupervisorError("Observer caller has no registered control scope")
    if scope is not None:
        observer_control_reservation(owner, scope, "claim")
    captured = False
    yielded = False
    try:
        if scope is not None and profile is not None:
            memory = observer_control_observation(scope).get("MemoryMax", "")
            if not memory.isdecimal() or int(memory) <= 0:
                raise SupervisorError("Observer control cap must be finite")
            scope.memory = int(memory)
            ceiling = host_admission.settings(profile)["control_gib"] * GIB
            scope.temporary_memory = min(scope.memory, ceiling)
            captured = True
            systemctl(
                "set-property",
                "--runtime",
                scope.unit,
                f"MemoryMax={scope.temporary_memory}",
                "MemorySwapMax=0",
            )
            if observer_control_observation(scope).get("MemoryMax") != str(
                scope.temporary_memory
            ):
                raise SupervisorError("Observer control cap readback differs")
        yielded = True
        yield
    finally:
        deadline = (
            cancellation_deadline() if cancellation_deadline is not None else None
        )
        never_launched = not yielded or (
            launch_attempted is not None and not launch_attempted()
        )
        if scope is not None and (
            never_launched
            or observer_allocation_drained(owner, unit, deadline=deadline)
        ):
            observer_control_reservation(
                owner,
                scope,
                "verify",
                deadline=deadline,
            )
            if captured:
                current = observer_control_observation(scope, deadline=deadline).get(
                    "MemoryMax"
                )
                if current not in {str(scope.temporary_memory), str(scope.memory)}:
                    raise SupervisorError(
                        "Observer control cap changed before restoration"
                    )
                if current != str(scope.memory):
                    systemctl(
                        "set-property",
                        "--runtime",
                        scope.unit,
                        f"MemoryMax={scope.memory}",
                        deadline=deadline,
                    )
                if observer_control_observation(scope, deadline=deadline).get(
                    "MemoryMax"
                ) != str(scope.memory):
                    raise SupervisorError(
                        "Observer control cap restoration readback differs"
                    )
            observer_control_reservation(
                owner,
                scope,
                "release",
                deadline=deadline,
            )


def observer(
    state: Path,
    command: list[str],
    profile: str | None = None,
    *,
    observation: dict[str, object] | None = None,
) -> int:
    """Run one foreground observer in the same finite primary/server envelope."""
    config_for(state)
    state = checked_directory(state)
    unit = f"pse-native-{uuid.uuid4().hex}.scope"
    if observer_launch_busy(state):
        raise SupervisorError("The observer allocation is occupied")
    with context_admission(state):
        config = config_for(state)
        allocation = (
            config["resources"] if profile is None else execution_resources(profile)
        )
        if not isinstance(allocation, dict) or not isinstance(
            allocation.get("execution"), dict
        ):
            raise SupervisorError(
                "Observer requires the explicit shared execution profile"
            )
        if (
            not command
            or config["admission"] != "open"
            or not config["accepting_writes"]
        ):
            raise SupervisorError("Observer command requires open admission")
        # Check only recorded caller liveness under exclusion. Actual unit/drain
        # observation above never holds this metadata lock.
        pending = state / "observer-launch.json"
        if pending.exists():
            previous = read_json(pending)
            process = Path(f"/proc/{integer(previous['pid'])}/stat")
            if (
                process.exists()
                and process_observation.start_identity(integer(previous["pid"]))
                == previous["start"]
            ):
                raise SupervisorError("The observer allocation is occupied")
        registration = state / "primary-observer.json"
        if registration.exists():
            prior = read_json(registration)
            process = Path(f"/proc/{integer(prior['pid'])}/stat")
            if (
                process.exists()
                and process_observation.start_identity(integer(prior["pid"]))
                == prior["start"]
            ):
                raise SupervisorError("The reference observer allocation is occupied")
        start = process_observation.start_identity(os.getpid())
        write_json(
            state / "observer-launch.json",
            {"pid": os.getpid(), "start": start, "unit": unit},
        )
    root = str(SCRIPT.parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)

    ensure_execution_placement(state, allocation)

    owner = host_admission.inherit(os.environ, services=ServiceAdmission())
    if owner is None:
        raise SupervisorError("Observer must belong to an admitted host allocation")
    attempted = False
    handoff = None
    launch_observation = observation if observation is not None else {}
    parent_operation = native_operation.owner_record()

    def cancellation_deadline() -> float | None:
        clocks = [launch_observation.get("cancellation_deadline")]
        for path in (handoff, parent_operation):
            if path is not None:
                clocks.append(native_operation._record(path).get("cancel_deadline"))  # noqa: SLF001 -- exact observer and parent cancellation owners
        selected_clocks: list[float] = []
        for value in clocks:
            if value is None:
                continue
            if (
                not isinstance(value, (int, float))
                or isinstance(value, bool)
                or not math.isfinite(value)
            ):
                raise SupervisorError("Observer original cancellation clock is invalid")
            selected_clocks.append(value)
        return min(selected_clocks) if selected_clocks else None

    try:
        with observer_control_limit(
            owner,
            unit,
            profile,
            launch_attempted=lambda: attempted,
            cancellation_deadline=cancellation_deadline,
        ):
            owner.register(unit)
            handoff = native_operation.prepare_handoff(unit, foreground=True)
            environment = systemd_environment()
            for key in (
                "PSE_NATIVE_OPERATION",
                "PSE_NATIVE_HANDOFF",
                "PSE_NATIVE_WORKER_SLOT",
                "PSE_NATIVE_WORKER_MEMORY_BYTES",
                "PSE_MEMORY_MAX",
            ):
                environment.pop(key, None)
            if handoff is not None:
                environment["PSE_NATIVE_HANDOFF"] = str(handoff)
            environment["PSE_SURREAL_STATE"] = str(state)
            if profile is not None:
                environment["PSE_TEST_EXECUTION_PROFILE"] = profile
            selected_command = observer_scope_command(
                state,
                allocation,
                unit,
                command,
                unbounded=owner.profile.unbounded and profile == "exclusive-observer",
            )
            attempted = True
            # The operation creator owns cancellation of this handoff. Keep its
            # native supervisor outside a validation gate's process-group kill,
            # so it can drain runners that establish their own sessions.
            return native_operation.run(
                selected_command,
                environment,
                observation=launch_observation,
                foreground=handoff,
            )
    finally:
        # Keep launch exclusion through the control-scope restoration attempt.
        # Surviving or unbound scopes retain launch metadata and host charge.
        launch_observation["owned_drain"] = "unresolved"
        deadline = cancellation_deadline()
        drained = not attempted or observer_allocation_drained(
            owner, unit, deadline=deadline
        )
        launch_observation["owned_drain"] = (
            "not-started" if not attempted else "complete" if drained else "unresolved"
        )
        if drained:
            with state_lock(state, deadline=deadline):
                launch = read_json(state / "observer-launch.json")
                if launch.get("unit") == unit:
                    (state / "observer-launch.json").unlink()


def primary_observation(state: Path) -> dict[str, str]:
    result = systemctl(
        "show",
        "--property=LoadState",
        "--property=ActiveState",
        "--property=ControlGroup",
        "--property=MemoryMax",
        "--property=MainPID",
        primary_unit(state),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if result.returncode or any(
        key not in fields for key in ("LoadState", "ActiveState", "ControlGroup")
    ):
        raise SupervisorError(
            "Cannot verify the primary receiver's systemd association"
        )
    return fields


def primary_association(config: dict[str, object]) -> dict[str, str]:
    """Validate the exact configured paths, separately from content qualification."""
    receiver = config.get("primary_receiver")
    if not isinstance(receiver, dict) or not all(
        isinstance(value, str) for value in receiver.values()
    ):
        raise SupervisorError(
            "Selected execution requires a configured primary receiver"
        )
    import msgspec  # noqa: PLC0415 -- private typed boundary

    try:
        result = string_mapping(
            msgspec.to_builtins(
                msgspec.convert(
                    receiver, type=deployment.ManagedPrimaryReceiver, strict=True
                )
            )
        )
    except msgspec.ValidationError as error:
        raise SupervisorError(
            "Primary receiver association has unsupported fields"
        ) from error
    if any(
        not Path(result[key]).is_absolute()
        for key in ("supervisor_executable", "supervisor_script", "worker_executable")
    ):
        raise SupervisorError("Primary receiver paths must be explicit and absolute")
    return result


def recorded_primary(config: dict[str, object]) -> dict[str, str]:
    """Read the admitted generation, without reopening a replaced executable."""
    receiver = primary_association(config)
    verify_generation(Path(receiver["supervisor_script"]).parents[1])
    return receiver


def checked_primary(config: dict[str, object]) -> dict[str, str]:
    receiver = primary_association(config)
    directory = Path(receiver["supervisor_script"]).parents[1]
    verified = activatable_generation(directory)
    script = Path(receiver["supervisor_script"])
    worker = Path(receiver["worker_executable"])
    if (
        not script.is_relative_to(directory)
        or not worker.is_relative_to(directory)
        or verified.files.get(str(script.relative_to(directory)))
        != receiver["supervisor_sha256"]
        or verified.files.get(str(worker.relative_to(directory)))
        != receiver["worker_sha256"]
    ):
        raise SupervisorError(
            "Primary receiver bytes changed; offline profile readmission is required"
        )
    return receiver


def primary_ready(
    state: Path,
    config: dict[str, object],
    observation: dict[str, str],
    database: str | None = None,
    qualification: Path | None = None,
) -> bool:
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        return False
    owner = getattr(_STARTUP, "verification", None)
    if owner is not None:
        # A changed process or association is a fresh trust transition.
        # Always retain _primary_process's actual executable/nonce/database and
        # placement qualification; only unchanged immutable generation is shared.
        # Pending readiness is not a new process on every poll. A transition
        # into or out of an observed lifetime still captures generation bytes.
        process = None
        try:
            marker = read_json(state / "primary-receiver.json")
            process = json.dumps(
                {
                    "state": str(state),
                    "selection": startup_selection(config),
                    "observation": observation,
                    "marker": marker,
                    "launch": read_json(state / "primary-launch.json"),
                    "start": operation_lifetime.start_identity(integer(marker["pid"])),
                    "database": database,
                    "qualification": str(qualification),
                },
                sort_keys=True,
            )
        except (OSError, KeyError, SupervisorError):
            pass
        if owner.process_observed and process != owner.process:
            owner.generations.clear()
        owner.process = process
        owner.process_observed = True
    return (
        _primary_process(
            state,
            config,
            observation,
            checked_primary(config),
            database,
            qualification,
            receiving=True,
        )
        is not None
    )


def primary_drain_pid(
    state: Path, config: dict[str, object], observation: dict[str, str]
) -> int | None:
    """Identify the admitted live receiver even after its disk files are replaced."""
    if "primary_receiver" in config and config["primary_receiver"] is None:
        # Functional observer contexts explicitly admit no primary receiver.
        # Its absence does not excuse a surviving process at that context's unit.
        if worker_busy(observation):
            raise SupervisorError(
                "Unconfigured primary receiver remains active; drain refused"
            )
        return None
    return _primary_process(state, config, observation, recorded_primary(config))


def primary_dispatch_owner(
    state: Path,
    config: dict[str, object],
    database: str,
    qualification: Path | None,
) -> dict[str, object]:
    """Bind dispatch to the ready receiver's actual kernel and manager lifetime."""
    unit = primary_unit(state)
    before = operation_lifetime.unit_observation(unit, properties=("MemoryMax",))
    group = before["ControlGroup"]
    invocation = before["InvocationID"]
    if (
        not re.fullmatch(r"[a-f0-9]{32}", invocation)
        or not group.startswith("/")
        or group == "/"
        or ".." in Path(group).parts
        or not primary_ready(state, config, before, database, qualification)
    ):
        raise SupervisorError("Primary changed before dispatch observation admission")
    inode = host_admission.group_identity(group)
    after = operation_lifetime.unit_observation(unit)
    if (
        inode is None
        or after["InvocationID"] != invocation
        or after["ControlGroup"] != group
        or after["ActiveState"] != "active"
        or host_admission.group_identity(group) != inode
    ):
        raise SupervisorError(
            "Primary lifetime changed during dispatch observation admission"
        )
    return {
        "scope": {"unit": unit, "group": group, "invocation": invocation},
        "scope_inode": inode,
    }


def observe_primary(
    state: Path, database: str | None, owner: dict[str, object]
) -> dict[str, object]:
    """Observe one admitted receiver, without starting, stopping or settling work."""
    state = receiver_context(state, database)
    config = config_for(state)
    if database is None or config["database"] != database:
        raise SupervisorError("Primary observation requires its canonical database")
    scope = owner.get("scope")
    if (
        set(owner) != {"scope", "scope_inode"}
        or not isinstance(scope, dict)
        or set(scope) != {"unit", "group", "invocation"}
        or scope.get("unit") != primary_unit(state)
        or not isinstance(scope.get("group"), str)
        or not scope["group"].startswith("/")
        or scope["group"] == "/"
        or ".." in Path(scope["group"]).parts
        or not isinstance(scope.get("invocation"), str)
        or not re.fullmatch(r"[a-f0-9]{32}", scope["invocation"])
        or type(owner.get("scope_inode")) is not int
        or integer(owner["scope_inode"]) <= 0
    ):
        raise SupervisorError("Primary observation owner is not the selected receiver")
    observation = operation_lifetime.drained(owner)
    return {
        "canonical_database": database,
        "primary_owner": owner,
        "phase": observation.phase.value,
    }


def primary_dispatch_handoff(
    state: Path,
    config: dict[str, object],
    database: str,
    qualification: Path | None,
) -> dict[str, object]:
    """Compose actual dispatch admission with an optional unchanged marker copy."""
    try:
        marker = read_json(state / "primary-receiver.json")
    except (OSError, ValueError, SupervisorError):
        marker = None
    owner = primary_dispatch_owner(state, config, database, qualification)
    return {
        "primary_owner": owner,
        "diagnostics": primary_diagnostics(state, database, marker),
    }


def primary_diagnostics(
    state: Path, database: str, admitted_marker: dict[str, object] | None
) -> dict[str, object]:
    """Copy optional receiver observation after readiness's own admission.

    This handoff supplies no process-stop or allocation-release permission.
    Missing publication, opt-out and writer failure never affect readiness.
    """
    try:
        marker = read_json(state / "primary-receiver.json")
        if admitted_marker is None or marker != admitted_marker:
            return {
                "available": False,
                "failure": "receiver marker changed during dispatch admission",
            }
        launch = read_json(state / "primary-launch.json")
        if (
            marker.get("canonical_database") != database
            or marker.get("ready") is not True
            or not marker.get("nonce")
            or marker.get("nonce") != launch.get("nonce")
        ):
            return {"available": False, "failure": "receiver association changed"}
        handoff = marker.get("diagnostics")
        if not isinstance(handoff, dict):
            return {
                "available": False,
                "failure": "receiver diagnostic locator unavailable",
            }
        identity = handoff.get("identity")
        if not isinstance(identity, dict):
            return {
                "available": False,
                "failure": "receiver diagnostic identity unavailable",
            }
        locator = identity.get("locator")
        if not isinstance(locator, dict):
            return {
                "available": False,
                "failure": "receiver diagnostic identity unavailable",
            }
        if (
            locator.get("pid") != marker.get("pid")
            or type(locator.get("start_ticks")) is not int
            or str(locator["start_ticks"])
            != operation_lifetime.start_identity(integer(marker["pid"]))
        ):
            return {
                "available": False,
                "failure": "receiver diagnostic start identity unavailable",
            }
        if not isinstance(identity.get("process"), str) or not isinstance(
            handoff.get("checkpoint"), str
        ):
            return {
                "available": False,
                "failure": "receiver diagnostic checkpoint unavailable",
            }
        return {
            **handoff,
            "available": bool(handoff.get("enabled")),
            "receiver_state": str(state),
            "canonical_database": database,
            "nonce": marker["nonce"],
        }
    except (OSError, ValueError, KeyError, TypeError, SupervisorError) as error:
        return {"available": False, "failure": str(error)}


def _primary_process(
    state: Path,
    config: dict[str, object],
    observation: dict[str, str],
    receiver: dict[str, str],
    database: str | None = None,
    qualification: Path | None = None,
    *,
    receiving: bool = False,
) -> int | None:
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        return None
    execution = allocation["execution"]
    try:
        launch = read_json(state / "primary-launch.json")
        marker = read_json(state / "primary-receiver.json")
        pid = integer(marker["pid"])
        if qualification is not None and marker.get(
            "qualification_native_entry"
        ) != str(qualification):
            return None
        selected = marker.get("canonical_database")
        if (
            not isinstance(selected, str)
            or not selected
            or (database is not None and selected != database)
        ):
            return None
        if (
            observation["ActiveState"] != "active"
            or not group_populated(observation["ControlGroup"])
            or marker.get("ready") is not True
            or marker.get("nonce") != launch["nonce"]
        ):
            return None
        if observation.get("MemoryMax") != str(
            allocation["native_worker_memory_bytes"]
        ):
            return None
        if any(
            marker.get(key) != execution[key]
            for key in (
                "pool_memory_bytes",
                "worker_bytes",
                "cpu_threads",
                "case_lanes",
                "math_jobs",
            )
        ):
            return None
        process = Path(f"/proc/{pid}")
        relative = next(
            line.removeprefix("0::")
            for line in (process / "cgroup").read_text().splitlines()
            if line.startswith("0::")
        )
        if relative != observation["ControlGroup"] and not relative.startswith(
            observation["ControlGroup"] + "/"
        ):
            return None
        executable = process / "exe"
        # Linux keeps the admitted executable open after atomic replacement. Its
        # proc link gains this suffix; resolving it against the replacement path
        # would confuse the running admission with the next one.
        actual_path = str(executable.readlink()).removesuffix(" (deleted)")
        if file_digest(executable) != receiver["worker_sha256"] or Path(
            actual_path
        ) != Path(receiver["worker_executable"]):
            return None
        assignments = (process / "environ").read_bytes().split(b"\0")
        if [
            item.removeprefix(b"PSE_PRIMARY_NONCE=")
            for item in assignments
            if item.startswith(b"PSE_PRIMARY_NONCE=")
        ] != [os.fsencode(str(launch["nonce"]))]:
            return None
        if receiving and not primary_receipt_ready(
            process, marker, systemd_environment()
        ):
            return None
        placement = placement_observation(state)
        if not isinstance(placement, dict) or not isinstance(
            placement.get("physical_cpus"), list
        ):
            return None
        if not role_affinity_ready(pid, execution, placement["physical_cpus"]):
            return None
        memory, cpu = effective_limits(Path("/sys/fs/cgroup") / relative.lstrip("/"))
        if (
            memory != integer(allocation["native_worker_memory_bytes"])
            or cpu is None
            or cpu < integer(execution["cpu_threads"])
        ):
            return None
    except (OSError, KeyError, StopIteration, SupervisorError):
        return None
    else:
        return pid


def verify_observer(state: Path, config: dict[str, object], pid: int) -> None:
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        raise SupervisorError("Observer requires the selected shared execution profile")
    execution = allocation["execution"]
    if pid <= 0:
        raise SupervisorError("Observer PID must be positive")
    cgroups = Path(f"/proc/{pid}/cgroup").read_text()
    relative = next(
        (
            line.removeprefix("0::")
            for line in cgroups.splitlines()
            if line.startswith("0::")
        ),
        "",
    )
    group = Path("/sys/fs/cgroup") / relative.lstrip("/")
    profile = group_for_slice(execution_slice(state))
    placement = placement_observation(state)
    if not isinstance(placement, dict):
        raise SupervisorError("Observer lacks actual execution placement")
    cpus = placement.get("physical_cpus")
    if not isinstance(cpus, list) or not all(isinstance(value, int) for value in cpus):
        raise SupervisorError("Observer lacks declared physical CPU placement")
    memory, cpu = effective_limits(group)
    if (
        not group.is_relative_to(profile)
        or memory is None
        or memory > integer(execution["observer_memory_bytes"])
        or cpu is None
        or cpu < integer(execution["cpu_threads"])
        or not role_affinity_ready(pid, execution, cpus)
    ):
        # Explain the actual failed role without exposing environment values.
        # The launch receipt precedes native library/thread initialization, so
        # identify later affinity changes on the process that requested admission.
        affinity: dict[str, object] = {"expected": cpus}
        try:
            affinity["leader"] = sorted(os.sched_getaffinity(pid))
            mismatches: list[dict[str, object]] = []
            inspected = 0
            for thread in (Path("/proc") / str(pid) / "task").iterdir():
                if inspected == 128:
                    affinity["truncated"] = True
                    break
                inspected += 1
                try:
                    actual = os.sched_getaffinity(int(thread.name))
                    if (not actual or not actual <= set(cpus)) and len(mismatches) < 16:
                        mismatches.append(
                            {
                                "tid": int(thread.name),
                                "comm": (thread / "comm").read_text().strip()[:64],
                                "cpus": sorted(actual),
                            }
                        )
                except OSError as error:
                    if len(mismatches) < 16:
                        mismatches.append(
                            {"tid": int(thread.name), "observation_error": str(error)}
                        )
            affinity["threads_inspected"] = inspected
            affinity["thread_mismatches"] = mismatches
        except OSError as error:
            affinity["observation_error"] = str(error)
        raise SupervisorError(
            "Observer placement differs: "
            + json.dumps(
                {
                    "pid": pid,
                    "expected_group": str(profile),
                    "actual_group": str(group),
                    "expected_memory_max": integer(execution["observer_memory_bytes"]),
                    "actual_memory_max": memory,
                    "minimum_cpu_quota": integer(execution["cpu_threads"]),
                    "actual_cpu_quota": cpu,
                    "affinity": affinity,
                },
                sort_keys=True,
            )
        )
    # The envelope includes one observer process, not arbitrarily many independent
    # four-GiB ceilings. Kernel membership and PID start time establish live reuse.
    start = process_observation.start_identity(pid)
    registration = state / "primary-observer.json"
    if registration.exists():
        prior = read_json(registration)
        previous = integer(prior["pid"])
        previous_stat = Path(f"/proc/{previous}/stat")
        if previous != pid and previous_stat.exists():
            observed = process_observation.start_identity(previous)
            if observed == prior["start"]:
                raise SupervisorError(
                    "The reference envelope's observer allocation is occupied"
                )
    write_json(registration, {"pid": pid, "start": start, "group": relative})


def _checked_current_observer(state: Path) -> dict[str, object]:
    """Validate the live caller's own registration while holding state_lock."""
    registration = read_json(state / "primary-observer.json")
    pid = os.getpid()
    if integer(registration["pid"]) != pid:
        raise SupervisorError("Current caller does not own the observer registration")
    start = process_observation.start_identity(pid)
    group = process_observation.process_group(pid)
    if (
        not group
        or registration.get("start") != start
        or registration.get("group") != group
    ):
        raise SupervisorError("Current observer process identity changed")
    return registration


def release_current_observer(state: Path) -> None:
    """Relinquish only this caller's registration after primary capacity drains.

    The observer's live PID is never overridden by a second caller. A sequential
    owner may explicitly hand off within its existing aggregate group after the
    associated primary is stopped and the kernel confirms its allocation empty.
    """
    config_for(state)
    state = checked_directory(state)
    prior = getattr(_STARTUP, "deadline", None)
    deadline = time.monotonic() + 10
    _STARTUP.deadline = deadline if prior is None else min(deadline, prior)
    try:
        with state_lock(state):
            registration = _checked_current_observer(state)
        observed = primary_observation(state)
        group = observed["ControlGroup"]
        if observed["ActiveState"] not in {"inactive", "failed"} or (
            group and group_populated(group)
        ):
            raise SupervisorError(
                "Observer handoff requires a stopped, drained primary"
            )
        with state_lock(state):
            if _checked_current_observer(state) != registration:
                raise SupervisorError("Observer registration changed before handoff")
            remaining(_STARTUP.deadline)
            (state / "primary-observer.json").unlink()
    finally:
        _STARTUP.deadline = prior


def qualification_directory(state: Path, selected: Path) -> Path:
    """Accept an explicit private diagnostic control, never ambient execution policy."""
    selected = lexical_absolute(selected)
    state = lexical_absolute(state)
    if selected == state or not selected.is_relative_to(state):
        raise SupervisorError("Qualification control must be beneath the owned state")
    if any(path.is_symlink() for path in (selected, *selected.parents)):
        raise SupervisorError("Qualification control must not traverse symlinks")
    if (
        not selected.is_dir()
        or selected.stat().st_uid != os.getuid()
        or selected.stat().st_mode & 0o077
    ):
        raise SupervisorError(
            "Qualification control must be an existing private owned directory"
        )
    return selected


def ensure_primary(
    state: Path,
    observer_pid: int | None = None,
    database: str | None = None,
    qualification: Path | None = None,
) -> dict[str, object]:
    state = receiver_context(state, database)
    # Existing nested control calls consume this owner's clock; they do not renew it.
    deadline = time.monotonic() + 30
    prior = getattr(_STARTUP, "deadline", None)
    reservation = state / "primary-admission.json"
    token = uuid.uuid4().hex

    deadline = deadline if prior is None else min(deadline, prior)
    selection = startup_selection(config_for(state))
    while True:
        remaining(deadline)
        with context_admission(state, deadline=deadline):
            if startup_selection(config_for(state)) != selection:
                raise SupervisorError(
                    "Primary selection changed while joining admission"
                )
            if any(
                reservation_live(read_json(pending))
                for pending in state.glob("worker-admission-*.json")
            ):
                raise SupervisorError(
                    "A pending native worker launch owns this receiver allocation"
                )
            owner = read_json(reservation) if reservation.exists() else None
            if owner is None or not reservation_live(owner):
                remaining(deadline)
                write_json(
                    reservation,
                    {
                        "nonce": token,
                        "pid": os.getpid(),
                        "start": operation_lifetime.start_identity(os.getpid()),
                        "boot": host_admission.boot(),
                    },
                )
                break
        wait_reservation(state, reservation, owner["nonce"], deadline)
    _STARTUP.deadline = deadline if prior is None else min(deadline, prior)
    prior_verification = getattr(_STARTUP, "verification", None)
    _STARTUP.verification = PrimaryVerification()
    try:
        return _ensure_primary(
            state, observer_pid, database, _STARTUP.deadline, qualification
        )
    finally:
        _STARTUP.verification = prior_verification
        with state_lock(state, cleanup=True):
            if reservation.exists():
                selected = read_json(reservation)
                if (
                    selected.get("nonce") == token
                    and selected.get("unresolved") is not True
                ):
                    reservation.unlink()
        _STARTUP.deadline = prior


def _ensure_primary(
    state: Path,
    observer_pid: int | None,
    database: str | None,
    deadline: float,
    qualification: Path | None = None,
) -> dict[str, object]:
    config_for(state)
    state = checked_directory(state)
    if qualification is not None:
        qualification = qualification_directory(state, qualification)
    config = config_for(state)
    if config["admission"] != "open" or not config["accepting_writes"]:
        raise SupervisorError("Primary admission is closed")
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        raise SupervisorError(
            "Primary serving requires an explicit reference execution profile"
        )
    receiver = checked_primary(config)
    primary_environment(state, allocation, receiver)
    selected_database = database if database is not None else str(config["database"])
    if not selected_database or len(selected_database) > 1024:
        raise SupervisorError(
            "Primary receiver requires a bounded canonical database identity"
        )
    ensure_execution_placement(state, allocation)
    if observer_pid is not None:
        verify_observer(state, config, observer_pid)
    observed = primary_observation(state)
    ready_arguments = () if qualification is None else (qualification,)
    if primary_ready(state, config, observed, selected_database, *ready_arguments):
        start(state, config, deadline=deadline)
        return {
            "ready": True,
            "unit": primary_unit(state),
            **primary_dispatch_handoff(state, config, selected_database, qualification),
            "execution": allocation["execution"],
            "canonical_database": selected_database,
            "placement": placement_observation(state),
        }
    if worker_busy(observed) or worker_busy(settled_worker(state, 0)):
        raise SupervisorError(
            "An active unmatched receiver or canonical database owns the primary allocation; quiesce and drain it before reuse"
        )
    start(state, config, deadline=deadline)
    root = str(SCRIPT.parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)

    host_owner = register_role_owner(primary_unit(state), deadline)
    predecessor = registration_predecessor(
        primary_unit(state), host_owner, deadline=deadline
    )
    reservation = state / "primary-admission.json"
    selected_reservation = read_json(reservation, deadline=deadline)
    try:
        handoff = native_operation.prepare_handoff(
            primary_unit(state),
            reservation=(reservation, str(selected_reservation["nonce"])),
            deadline=deadline,
        )
    except operation_lifetime.DeadlineExpiredError as error:
        raise DeadlineExpiredError(
            f"Primary handoff exceeded its original admission clock; selected={primary_unit(state)}"
        ) from error
    nonce = uuid.uuid4().hex
    write_json(state / "primary-launch.json", {"nonce": nonce})
    (state / "primary-receiver.json").unlink(missing_ok=True)
    environment, names = primary_environment(state, allocation, receiver, handoff)
    environment["PSE_PRIMARY_NONCE"] = nonce
    command = [
        "systemd-run",
        "--user",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={primary_unit(state)}",
        *selected_slice(state, allocation),
        "--service-type=exec",
        "--property=KillMode=control-group",
        "--property=Restart=no",
        "--property=TimeoutStopSec=45",
        "--property=TasksMax=2048",
        "--property=MemorySwapMax=0",
        f"--property=MemoryMax={allocation['native_worker_memory_bytes']}",
    ]
    for key in (
        "PSE_SURREAL_STATE",
        "PSE_NATIVE_WORKER_SLOT",
        "PSE_NATIVE_WORKER_MEMORY_BYTES",
        "PSE_NATIVE_HANDOFF",
        "PSE_PRIMARY_NONCE",
    ):
        command.append(f"--setenv={key}={environment[key]}")
    command.extend(primary_service_environment(environment, names))
    command.extend(
        [
            "--",
            *role_command(
                allocation,
                [
                    receiver["supervisor_executable"],
                    "-I",
                    "-B",
                    str(
                        Path(receiver["supervisor_script"]).with_name(
                            "native_operation.py"
                        )
                    ),
                    "--capabilities",
                    ",".join(WORKER_CAPABILITIES),
                    "--prepared-environment",
                    str(
                        Path(receiver["supervisor_script"]).parents[1]
                        / "native-environment.json"
                    ),
                    "--",
                    receiver["worker_executable"],
                    "--maximum-in-flight",
                    str(allocation["execution"]["case_lanes"]),
                    "--canonical-database",
                    selected_database,
                ],
            ),
        ]
    )
    if qualification is not None:
        command.extend(["--qualification-native-entry", str(qualification)])
    launch_attempted = False
    cleanup_binding: dict[str, object] = {}
    try:
        launch_timeout = remaining(deadline)
        launch_attempted = True
        try:
            result = subprocess.run(
                command,
                env=environment,
                capture_output=True,
                timeout=launch_timeout,
                check=False,
            )
        except subprocess.TimeoutExpired as error:
            raise DeadlineExpiredError(
                "Primary launch exceeded its original admission clock"
            ) from error
        if result.returncode:
            raise SupervisorError("Cannot launch the capped primary receiver")  # noqa: TRY301 -- owned-launch failures must enter registration cleanup
        while time.monotonic() < deadline:
            observed = primary_observation(state)
            if primary_ready(
                state, config, observed, selected_database, *ready_arguments
            ):
                return {
                    "ready": True,
                    "unit": primary_unit(state),
                    **primary_dispatch_handoff(
                        state, config, selected_database, qualification
                    ),
                    "execution": allocation["execution"],
                    "canonical_database": selected_database,
                    "placement": placement_observation(state),
                }
            if observed["ActiveState"] == "failed":
                break
            time.sleep(min(0.05, remaining(deadline)))
        raise SupervisorError(  # noqa: TRY301 -- owned-launch failures must enter registration cleanup
            "Primary receiver did not establish its actual runtime readiness"
        )
    except BaseException as failure:
        if not launch_attempted:
            raise
        # Preserve the existing ten-second primary cleanup lifetime. Every
        # cleanup control consumes that clock independently of expired admission.
        cleanup_deadline = time.monotonic() + 10
        admission_deadline = getattr(_STARTUP, "deadline", None)
        _STARTUP.deadline = None
        try:
            stop_registered_owner(
                primary_unit(state),
                host_owner,
                predecessor=predecessor,
                handoff=handoff,
                deadline=cleanup_deadline,
                evidence=cleanup_binding,
            )
        except (
            OSError,
            KeyError,
            ValueError,
            SupervisorError,
            host_admission.AdmissionError,
            subprocess.SubprocessError,
        ) as cleanup_error:
            cause = f"{type(cleanup_error).__name__}: {cleanup_error}"
            with state_lock(state, cleanup=True):
                path = state / "primary-admission.json"
                if path.exists():
                    retained = read_json(path)
                    retained.update(
                        unresolved=True,
                        cleanup_cause=cause,
                        predecessor=predecessor,
                        binding=cleanup_binding or None,
                        unit=primary_unit(state),
                        handoff=str(handoff) if handoff is not None else None,
                        allocation=str(host_owner.directory / host_owner.nonce),
                    )
                    write_json(path, retained)
            failure.add_note(
                f"Primary cleanup unresolved; selected={primary_unit(state)}; admission reservation retained: {cause}"
            )
        finally:
            _STARTUP.deadline = admission_deadline
        raise


def drain_context(
    state: Path, database: str, *, deadline: float | None = None
) -> list[dict[str, str]]:
    """Cooperatively drain only the receiver associated with this exact context."""
    deadline = time.monotonic() + 45 if deadline is None else deadline
    previous = getattr(_STARTUP, "deadline", None)
    deadline = deadline if previous is None else min(previous, deadline)
    _STARTUP.deadline = deadline
    try:
        remaining(deadline)
        if not (state / ".contexts" / f"{database}.json").exists():
            with lifecycle_reservation(state, deadline=deadline):
                base = config_for(state)
                if not (state / ".contexts" / f"{database}.json").exists() and (
                    base["database"] != database
                    or (
                        "execution" not in object_mapping(base["resources"])
                        and base.get("primary_receiver") is None
                    )
                ):
                    # A service may offer native execution without this codec-only
                    # fixture admitting a receiver. Missing association alone is
                    # not drain: preserve every actual base/registered descendant
                    # obligation while lifecycle admission remains excluded.
                    for context in managed_contexts(state):
                        context_processes_drained(context, deadline=deadline)
                    return []
        selected = receiver_context(state, database)
        config = config_for(selected)
        if config["database"] != database:
            raise SupervisorError(
                "Drain requires an explicit receiver database association"
            )
        # The caller's clock includes configuration/identity observation and
        # child startup. Never signal a receiver under an expired original offer.
        remaining(deadline)
        observed = primary_observation(selected)
        group = observed.get("ControlGroup", "")
        if observed["ActiveState"] in {"inactive", "failed"} and (
            not group or not group_populated(group)
        ):
            context_processes_drained(selected, deadline=deadline)
            return []
        pid = primary_drain_pid(selected, config, observed)
        if pid is None:
            raise SupervisorError(
                "Unmatched live receiver remains retained; drain refused"
            )
        owner = operation_lifetime.unit_observation(primary_unit(selected))
        association = {
            "unit": primary_unit(selected),
            "group": owner["ControlGroup"],
            "invocation": owner["InvocationID"],
        }
        descriptor = os.pidfd_open(pid)
        try:
            if (
                primary_drain_pid(selected, config, primary_observation(selected))
                != pid
            ):
                raise SupervisorError(
                    "Receiver identity changed before cooperative drain"
                )
            remaining(deadline)
            signal.pidfd_send_signal(descriptor, signal.SIGINT)
            while True:
                completion = operation_lifetime.drained(
                    {"scope": association}, deadline=_STARTUP.deadline
                )
                if completion.complete:
                    break
                if completion.phase in {
                    operation_lifetime.DrainPhase.MISMATCH,
                    operation_lifetime.DrainPhase.UNAVAILABLE,
                }:
                    raise SupervisorError(
                        completion.diagnostic() + "; receiver drain unresolved"
                    )
                try:
                    time.sleep(min(0.02, remaining(_STARTUP.deadline)))
                except DeadlineExpiredError as error:
                    raise DeadlineExpiredError(
                        completion.diagnostic() + "; receiver drain deadline expired"
                    ) from error
        finally:
            os.close(descriptor)
        context_processes_drained(selected, deadline=deadline)
        return [association]
    finally:
        _STARTUP.deadline = previous


def public_status(state: Path, config: dict[str, object]) -> dict[str, object]:
    port = integer(config["port"])
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    observed = service_readiness(state, config)
    is_active = observed.manager.get("ActiveState") == "active"
    listener_owned = observed.listener_owned
    telemetry_identity = None
    if listener_owned:
        # Receiver contexts borrow this listener; only its storage owner records
        # the actual server process incarnation. Worker placement stays local.
        process = read_json(service_directory(state) / "server-process.json")
        telemetry_identity = process.get("telemetry_service_instance_id")
    return {
        "state": str(state),
        "active": is_active,
        "listener_owned": listener_owned,
        "authenticated_grpc_ready": observed.protocol_ready,
        "lifecycle": {
            **observed.document(),
            "reservation": observed.reservation,
            "manager": dict(observed.manager),
        },
        "service_generation": config["instance_id"],
        "telemetry_service_instance_id": telemetry_identity,
        "supervisor_generation": config.get("service_supervisor"),
        "parked": bool(config.get("parked")),
        "backup_pending": config.get("backup_pending"),
        "tooling_fixture": config.get("tooling_fixture") is True,
        "unit": unit_name(state),
        "endpoint": f"127.0.0.1:{port}",
        "grpc_endpoint": config["endpoint"],
        "credentials_file": str(state / "credentials.json"),
        "config_file": str(state / "config.json"),
        "server": config["server"],
        "study_module": config.get("study_module"),
        "study_module_qualification": config.get("study_module_qualification"),
        "admission": config["admission"],
        "interpretation": config["interpretation"],
        "resources": config["resources"],
        "execution_placement": placement_observation(state),
        "accepting_writes": config["accepting_writes"],
        "rpc_max_message_bytes": MESSAGE_BYTES,
        "worker_units": [
            worker_unit(state, slot)
            for slot in range(integer(allocation["native_workers"]))
        ],
    }


def listener_ready(state: Path, config: dict[str, object]) -> bool:
    owner = service_directory(state)
    if owner != state:
        return listener_ready(owner, config_for(owner))
    if not owns_listener(state, config):
        return False
    try:
        with urllib.request.urlopen(
            f"http://127.0.0.1:{config['port']}/health", timeout=1
        ) as response:
            return response.status == 200 and active(state)
    except (OSError, urllib.error.URLError):
        return False


def protocol_ready(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> bool:
    owner = service_directory(state, deadline=deadline)
    if owner != state:
        return protocol_ready(
            owner, config_for(owner, deadline=deadline), deadline=deadline
        )
    path = state / "protocol-readiness.json"
    try:
        if not stat.S_ISREG(path.stat().st_mode):
            raise SupervisorError("Protocol readiness is not a regular owned record")
    except FileNotFoundError:
        return False
    proof = read_json(path, deadline=deadline)
    observed = systemctl(
        "show",
        "--property=InvocationID",
        "--value",
        unit_name(state),
        check=False,
        deadline=deadline,
    )
    if observed.returncode:
        raise SupervisorError("Protocol invocation observation is unavailable")
    return (
        proof.get("schema") == "native-grpc-readiness-v1"
        and proof.get("datastore") == database_endpoint(state)
        and proof.get("instance_id") == config["instance_id"]
        and proof.get("invocation") == observed.stdout.strip()
        and proof.get("binary_sha256")
        == object_mapping(config["server"]).get("binary_sha256")
        and proof.get("credentials_sha256") == file_digest(state / "credentials.json")
        and proof.get("study_module_sha256") == study_module(config)["sha256"]
        and proof.get("study_module_qualification_sha256")
        == (
            None
            if qualification_module(config) is None
            else object_mapping(config["study_module_qualification"])["sha256"]
        )
    )


def establish_protocol_readiness(
    state: Path,
    config: dict[str, object],
    deadline: float,
    *,
    record_readiness: bool = True,
) -> None:
    """Authenticate native gRPC under the startup clock without installing schema."""
    state = checked_directory(state)
    credentials = read_json(state / "credentials.json")
    if (
        credentials.get("selection_username") != "pse-selection"
        or not isinstance(credentials.get("selection_password"), str)
        or not credentials["selection_password"]
    ):
        raise SupervisorError(
            "Selection credentials are missing; stop, quiesce and explicitly readmit this service"
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
    query = f"DEFINE USER OVERWRITE `pse-selection` ON ROOT PASSWORD {json.dumps(credentials['selection_password'])} ROLES VIEWER; INFO FOR ROOT;\n"
    result = subprocess.run(
        [
            str(object_mapping(config["server"])["binary"]),
            "sql",
            "--endpoint",
            str(config["endpoint"]),
            "--json",
            "--hide-welcome",
        ],
        input=query,
        env=environment,
        cwd=state,
        capture_output=True,
        text=True,
        timeout=remaining(deadline),
        check=False,
    )
    try:
        # The pinned CLI prints a REPL prompt even for non-interactive pipes.
        lines = [line.removeprefix("> ").strip() for line in result.stdout.splitlines()]
        returned = json.loads("\n".join(line for line in lines if line and line != ">"))
        valid = (
            isinstance(returned, list)
            and len(returned) == 2
            and returned[0] is None
            and isinstance(returned[1], dict)
            and isinstance(returned[1].get("namespaces"), dict)
        )
    except ValueError:
        valid = False
    if result.returncode or not valid:
        raise SupervisorError(
            "Authenticated native gRPC readiness failed; admission remains closed"
        )
    if not record_readiness:
        return
    invocation = systemctl(
        "show", "--property=InvocationID", "--value", unit_name(state)
    ).stdout.strip()
    write_json(
        state / "protocol-readiness.json",
        {
            "schema": "native-grpc-readiness-v1",
            "datastore": database_endpoint(state),
            "instance_id": config["instance_id"],
            "invocation": invocation,
            "binary_sha256": object_mapping(config["server"]).get("binary_sha256"),
            "credentials_sha256": file_digest(state / "credentials.json"),
            "study_module_sha256": study_module(config)["sha256"],
            "study_module_qualification_sha256": None
            if qualification_module(config) is None
            else object_mapping(config["study_module_qualification"])["sha256"],
        },
    )


def ready(state: Path, config: dict[str, object]) -> bool:
    return listener_ready(state, config) and protocol_ready(state, config)


def process_owns_listening_socket(proc: Path, inodes: set[str]) -> bool:
    sockets = {f"socket:[{inode}]" for inode in inodes}
    for descriptor in (proc / "fd").iterdir():
        try:
            target = str(descriptor.readlink())
        except FileNotFoundError:
            # Other connections may close after enumeration. Only a live
            # descriptor for the selected listening socket establishes ownership.
            continue
        if target in sockets:
            return True
    return False


def owns_listener(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> bool:
    """Status-only projection; lifecycle decisions consume service_readiness."""
    try:
        return _owns_listener(state, config, deadline=deadline)
    except (
        OSError,
        KeyError,
        ValueError,
        StopIteration,
        SupervisorError,
        host_admission.AdmissionError,
    ):
        return False


def service_readiness(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> ServiceObservation:
    """One selected lifecycle observation supplies phase, identity and refusal cause."""
    fields: dict[str, str] = {}
    reservation: dict[str, object] | None = None
    unit = ""
    listener_owned = False

    def result(
        phase: Readiness, cause: str, *, protocol: bool = False
    ) -> ServiceObservation:
        return ServiceObservation(
            phase,
            state,
            config.get("instance_id"),
            unit,
            cause,
            dict(reservation) if reservation is not None else None,
            dict(fields),
            listener_owned,
            protocol,
        )

    try:
        if deadline is not None:
            remaining(deadline)
        owner = service_directory(state, deadline=deadline)
        if owner != state:
            return service_readiness(
                owner, config_for(owner, deadline=deadline), deadline=deadline
            )
        unit = unit_name(state)
        path = state / "lifecycle-owner.json"
        if getattr(_LIFECYCLE, "state", None) != state and path.exists():
            selected = read_json(path, deadline=deadline)
            if reservation_live(selected):
                reservation = selected
                purpose = selected.get("purpose")
                if not isinstance(purpose, str):
                    return result(
                        Readiness.UNAVAILABLE, "live reservation purpose unavailable"
                    )
                phase = {
                    "startup": Readiness.STARTING,
                    "stop": Readiness.STOPPING,
                    "maintenance": Readiness.MAINTENANCE,
                }.get(purpose)
                if phase is None:
                    return result(
                        Readiness.UNAVAILABLE, "live reservation purpose unavailable"
                    )
                return result(phase, "live owned " + purpose + " reservation")
        observed = systemctl(
            "show",
            "--property=ActiveState",
            "--property=ControlGroup",
            "--property=InvocationID",
            unit,
            check=False,
            deadline=deadline,
        )
        if observed.returncode:
            return result(Readiness.UNAVAILABLE, "user manager observation failed")
        for line in observed.stdout.splitlines():
            if "=" in line:
                key, value = line.split("=", 1)
                fields[key] = value
        status = fields.get("ActiveState")
        if status in {"inactive", "failed"}:
            try:
                process = read_json(state / "server-process.json", deadline=deadline)
                live = operation_lifetime.start_identity(
                    integer(process["pid"])
                ) == process.get("start")
            except FileNotFoundError:
                live = False
            group = fields.get("ControlGroup", "")
            if live or (group and group_populated(group)) or listening_inodes(config):
                return result(
                    Readiness.MISMATCH,
                    "inactive unit retains a process, populated group or listener",
                )
            return result(
                Readiness.ACTIONABLE_ABSENT,
                "unit terminal with no surviving listener or process",
            )
        if status == "activating" or (
            status == "active" and not listening_inodes(config)
        ):
            # Merely holding lifecycle exclusion does not authenticate a unit.
            # Joining requires its exact already charged launch and kernel group.
            _, launch = recorded_storage_owner(state, config, deadline=deadline)
            binding = object_mapping(launch["binding"])
            if (
                fields.get("InvocationID") != binding.get("invocation")
                or fields.get("ControlGroup") != binding.get("group")
                or host_admission.group_identity(fields.get("ControlGroup", ""))
                != binding.get("inode")
            ):
                return result(
                    Readiness.MISMATCH, "starting invocation differs from owned launch"
                )
            return result(
                Readiness.STARTING, "authenticated owned unit awaiting listener"
            )
        if status == "deactivating":
            return result(
                Readiness.STOPPING, "unit deactivating; replacement remains excluded"
            )
        if status != "active":
            return result(Readiness.UNAVAILABLE, "unit phase unavailable")
        if not _owns_listener(state, config, deadline=deadline):
            return result(Readiness.MISMATCH, "selected listener ownership differs")
        listener_owned = True
        with urllib.request.urlopen(
            f"http://127.0.0.1:{config['port']}/health",
            timeout=1 if deadline is None else min(1, remaining(deadline)),
        ) as response:
            if response.status != 200:
                return result(Readiness.UNAVAILABLE, "owned listener health refused")
        if deadline is not None:
            remaining(deadline)
        if (state / ".maintenance-listener.json").exists():
            require_maintenance_owner(state, config)
            return result(Readiness.MAINTENANCE, "owned maintenance listener")
        if protocol_ready(state, config, deadline=deadline):
            return result(
                Readiness.READY, "authenticated native protocol ready", protocol=True
            )
        return result(
            Readiness.STARTING, "owned listener awaiting authenticated protocol"
        )
    except DeadlineExpiredError as error:
        raise DeadlineExpiredError(
            result(
                Readiness.UNAVAILABLE, "original control deadline expired"
            ).diagnostic()
            + "; admission refused"
        ) from error
    except (
        OSError,
        SupervisorError,
        host_admission.AdmissionError,
        subprocess.SubprocessError,
    ) as error:
        return result(Readiness.UNAVAILABLE, f"{type(error).__name__}: {error}")
    except (KeyError, ValueError, StopIteration) as error:
        return result(Readiness.MISMATCH, f"{type(error).__name__}: {error}")


def _owns_listener(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> bool:
    """Linux listener ownership, independent of another server's /health response."""
    owner_state = service_directory(state, deadline=deadline)
    if owner_state != state:
        return _owns_listener(
            owner_state, config_for(owner_state, deadline=deadline), deadline=deadline
        )
    if deadline is not None:
        remaining(deadline)
    process = read_json(state / "server-process.json", deadline=deadline)
    if process.get("instance_id") != config["instance_id"]:
        return False
    pid = integer(process["pid"])
    proc = Path("/proc") / str(pid)
    if process.get("start") != operation_lifetime.start_identity(pid):
        return False
    membership = (proc / "cgroup").read_text()
    owner, launch = recorded_storage_owner(state, config, deadline=deadline)
    placement = storage_placement(config, owner)
    observed = storage_unit_observation(state, deadline=deadline)
    binding = object_mapping(launch["binding"])
    relative = next(
        line.removeprefix("0::")
        for line in membership.splitlines()
        if line.startswith("0::")
    )
    group = Path("/sys/fs/cgroup") / relative.lstrip("/")
    memory, cpu = effective_limits(group)
    quota, period = (group / "cpu.max").read_text().split()
    if (
        observed.get("ActiveState") != "active"
        or observed.get("InvocationID") != binding["invocation"]
        or observed.get("ControlGroup") != binding["group"]
        or relative != binding["group"]
        or host_admission.group_identity(relative) != binding["inode"]
        or process.get("allocation") != launch["allocation"]
        or memory != placement["memory_bytes"]
        or cpu != placement["cpu_threads"]
        or (group / "memory.max").read_text().strip() != str(placement["memory_bytes"])
        or quota == "max"
        or int(quota) / int(period) != placement["cpu_threads"]
        or not group.is_relative_to(group_for_slice(str(placement["slice"])))
        or not observed_role_affinity(
            pid,
            {"cpu_threads": placement["cpu_threads"]},
            list(host_admission.cpu_set(owner.profile.cores)),
        )
    ):
        return False
    inodes = listening_inodes(config)
    matched = process_owns_listening_socket(proc, inodes)
    if deadline is not None:
        remaining(deadline)
    return matched


def listening_inodes(config: dict[str, object]) -> set[str]:
    address = f"0100007F:{integer(config['port']):04X}"
    return {
        row.split()[9]
        for row in Path("/proc/net/tcp").read_text().splitlines()[1:]
        if row.split()[1] == address and row.split()[3] == "0A"
    }


def remaining(deadline: float) -> float:
    value = deadline - time.monotonic()
    if value <= 0:
        raise DeadlineExpiredError(
            "Managed startup exceeded its original admission clock"
        )
    return value


def storage_placement(
    config: dict[str, object], owner: host_admission.Allocation
) -> dict[str, object]:
    """Derive storage caps from its host owner, independently of science roles."""
    memory = integer(object_mapping(config["resources"])["server_memory_bytes"])
    service_class = config.get("service_class", "functional")
    if service_class not in {"functional", "timing"}:
        raise SupervisorError("Unknown storage service class")
    if not owner.profile.exclusive:
        expected = host_admission.select("store-" + str(service_class), str(memory))
        if owner.profile != expected:
            raise SupervisorError(
                "Storage owner differs from the selected service lane"
            )
    else:
        storage_memory = (
            integer(execution_profiles.profile("reference")["server_memory_bytes"])
            if owner.profile.name == "reference"
            else host_admission.settings(owner.profile.name)["store_gib"] * GIB
        )
        if memory > storage_memory:
            raise SupervisorError("Storage exceeds its borrowed exclusive partition")
    logical_cpus = host_admission.cpu_set(owner.profile.cores)
    return {
        "memory_bytes": memory,
        "cpu_threads": len(logical_cpus),
        "physical_cpus": list(owner.profile.cores),
        "slice": host_admission.allocation_slice(owner)
        if owner.profile.exclusive
        else "pse.slice",
    }


def storage_launch(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> dict[str, object]:
    """Require the exact immutable service selection associated with a launch."""
    launch = read_json(state / "service-launch.json", deadline=deadline)
    launch_deadline = launch.get("deadline")
    if not isinstance(launch_deadline, (int, float)) or not math.isfinite(
        launch_deadline
    ):
        raise SupervisorError("Storage launch lacks a finite admission clock")
    if (
        launch.get("generation") != config["instance_id"]
        or launch.get("supervisor_generation") != config.get("service_supervisor")
        or launch.get("server_generation") != config.get("server")
        or launch.get("service_class") != config.get("service_class", "functional")
        or launch.get("telemetry_state") != config.get("telemetry_state")
        or launch.get("server_memory_bytes")
        != object_mapping(config["resources"])["server_memory_bytes"]
    ):
        raise SupervisorError(
            "Storage launch differs from the admitted service generation"
        )
    return launch


def storage_launch_drain_config(
    state: Path,
    config: dict[str, object],
    launch: dict[str, object],
    *,
    deadline: float | None = None,
) -> dict[str, object]:
    """Resolve an unbound failed launch for drain, never for execution/admission.

    Explicit supervisor readmission archives its exact predecessor. A failed
    pre-bind launch can retain that predecessor after the config moves forward;
    only this closed, otherwise identical selection may use its historical
    immutable code witness. Bound lifetimes keep their original kernel proof.
    """
    if "binding" in launch:
        raise SupervisorError("Historical drain cannot replace a bound storage proof")
    try:
        if storage_launch(state, config, deadline=deadline) == launch:
            return config
    except SupervisorError:
        pass
    marker_path = state / ".maintenance-listener.json"
    pending = config.get("derived_rebuild_pending")
    if (
        isinstance(pending, dict)
        and pending.get("kind") == "initialize"
        and pending.get("phase") == "initializing"
        and (marker_path.exists() or marker_path.is_symlink())
    ):
        return interrupted_initializer_drain_config(
            state, config, launch, deadline=deadline
        )
    if (
        config.get("owner") != OWNER
        or config.get("accepting_writes") is not False
        or config.get("admission") != "quiesced"
        or launch.get("generation") != config.get("instance_id")
        or launch.get("server_generation") != config.get("server")
        or launch.get("service_class") != config.get("service_class", "functional")
        or launch.get("telemetry_state") != config.get("telemetry_state")
        or launch.get("server_memory_bytes")
        != object_mapping(config.get("resources"))["server_memory_bytes"]
    ):
        raise SupervisorError(
            "Failed launch lacks an exact closed readmission predecessor"
        )
    history = state / ".profile-history"
    information = history.lstat()
    if (
        not stat.S_ISDIR(information.st_mode)
        or information.st_uid != os.getuid()
        or stat.S_IMODE(information.st_mode) != 0o700
    ):
        raise SupervisorError("Readmission history lost private ownership")
    unchanged = {
        key: value
        for key, value in config.items()
        if key not in {"service_supervisor", "restart_qualified"}
    }
    for path in sorted(history.glob("*.json")):
        if not re.fullmatch(r"[a-f0-9]{32}\.json", path.name):
            continue
        if deadline is not None:
            remaining(deadline)
        predecessor = object_mapping(json.loads(private_record_bytes(path, deadline)))
        if {
            key: value
            for key, value in predecessor.items()
            if key not in {"service_supervisor", "restart_qualified"}
        } != unchanged:
            continue
        if predecessor.get("service_supervisor") != launch.get("supervisor_generation"):
            continue
        if storage_launch(state, predecessor, deadline=deadline) != launch:
            continue
        supervisor = object_mapping(predecessor.get("service_supervisor"))
        script = Path(str(supervisor.get("supervisor_script", "")))
        if (
            not script.is_absolute()
            or script.name != "surreal_server.py"
            or script.parent.name != "scripts"
            or script.parent.parent.parent != state / ".generations"
        ):
            raise SupervisorError(
                "Historical launch source escaped its immutable state"
            )
        with lifecycle_deadline(deadline):
            verified = verify_generation(script.parent.parent)
        if verified.files.get("scripts/surreal_server.py") != supervisor.get(
            "supervisor_sha256"
        ):
            raise SupervisorError("Historical launch supervisor identity differs")
        return predecessor
    raise SupervisorError("Exact historical failed-launch selection is unavailable")


def interrupted_initializer_drain_config(
    state: Path,
    config: dict[str, object],
    launch: dict[str, object],
    *,
    deadline: float | None = None,
) -> dict[str, object]:
    """Authenticate a lost initializer publication only to judge storage drain.

    The pending initializer and private listener marker retain the actual launch
    generation even if an interrupted caller wrote an older config snapshot.
    This detached selection is never persisted, executed or used for admission.
    """
    marker_path = state / ".maintenance-listener.json"
    if "binding" in launch or any(
        path.is_symlink() for path in (marker_path, *marker_path.parents)
    ):
        raise SupervisorError(
            "Interrupted initializer drain lacks an unbound private witness"
        )
    raw = private_record_bytes(marker_path, deadline, maximum_bytes=1024 * 1024)
    marker = object_mapping(json.loads(raw))
    pending = object_mapping(config.get("derived_rebuild_pending", {}))
    supervisor = object_mapping(marker.get("supervisor", {}))
    owner = object_mapping(marker.get("owner", {}))
    if (
        config.get("owner") != OWNER
        or config.get("accepting_writes") is not False
        or config.get("admission") not in {"validation_required", "quiesced"}
        or pending.get("kind") != "initialize"
        or pending.get("phase") != "initializing"
        or owner != pending.get("initializer")
        or owner.get("purpose") != "maintenance"
        or not re.fullmatch(r"[a-f0-9]{32}", str(owner.get("nonce", "")))
        or marker.get("instance_id") != config.get("instance_id")
        or marker.get("server") != config.get("server")
        or marker.get("resources") != config.get("resources")
        or supervisor != launch.get("supervisor_generation")
        or marker.get("credentials_sha256") != file_digest(state / "credentials.json")
        or (pending.get("target_namespace"), pending.get("target_database"))
        != (config.get("namespace"), config.get("database"))
    ):
        raise SupervisorError(
            "Interrupted initializer drain witness differs from its exact pending owner"
        )
    script = Path(str(supervisor.get("supervisor_script", "")))
    if (
        not script.is_absolute()
        or script.name != "surreal_server.py"
        or script.parent.name != "scripts"
        or script.parent.parent.parent != state / ".generations"
        or supervisor.get("supervisor_executable")
        not in {
            str(Path(sys.executable).absolute()),
            str(Path(sys.executable).resolve()),
        }
    ):
        raise SupervisorError(
            "Interrupted initializer drain source escaped its immutable state"
        )
    with lifecycle_deadline(deadline):
        verified = verify_generation(script.parent.parent)
    if verified.files.get("scripts/surreal_server.py") != supervisor.get(
        "supervisor_sha256"
    ):
        raise SupervisorError(
            "Interrupted initializer drain supervisor identity differs"
        )
    reconstructed = {**config, "service_supervisor": supervisor}
    if (
        storage_launch(state, reconstructed, deadline=deadline) != launch
        or private_record_bytes(marker_path, deadline, maximum_bytes=1024 * 1024) != raw
        or config_for(state, deadline=deadline) != config
    ):
        raise SupervisorError("Interrupted initializer drain observations changed")
    return reconstructed


def recorded_storage_owner(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> tuple[host_admission.Allocation, dict[str, object]]:
    """Read actual charged storage ownership without adopting the caller's lane."""
    launch = storage_launch(state, config, deadline=deadline)
    path = Path(str(launch["allocation"]))
    if not re.fullmatch(r"[a-f0-9]{32}", path.name):
        raise SupervisorError("Invalid storage allocation marker")
    snapshot = (
        contextlib.nullcontext(
            host_admission.readonly_snapshot(path.parent, deadline=deadline)
        )
        if deadline is not None
        else host_admission.allocation_metadata(path.parent)
    )
    with snapshot as ledger:
        record = ledger["owners"].get(path.name)
        if (
            record is None
            or record["boot"] != host_admission.boot()
            or record.get("released")
        ):
            raise SupervisorError("Storage allocation is absent or stale")
        bound = dict(record["units"].get(unit_name(state), {}))
        if (
            (
                record.get("service") != str(state)
                and str(state) not in record.get("borrowed_services", [])
            )
            or not isinstance(bound.get("group"), str)
            or not isinstance(bound.get("invocation"), str)
            or not re.fullmatch(r"[a-f0-9]{32}", str(bound.get("invocation", "")))
            or type(bound.get("inode")) is not int
        ):
            raise SupervisorError("Storage allocation does not own this service")
        profile = host_admission.Profile(
            record["class"],
            record["memory"],
            record["lane"],
            record["slots"],
            tuple(record["cores"]),
            record["exclusive"],
        )
        deadline = record["deadline"]
    if bound != launch.get("binding"):
        raise SupervisorError("Storage launch does not match its bound kernel lifetime")
    return host_admission.Allocation(
        path.parent, path.name, profile, deadline, services=ServiceAdmission()
    ), launch


def storage_unit_observation(
    state: Path, *, deadline: float | None = None
) -> dict[str, str]:
    """Observe storage identity through the supervisor's remaining original clock."""
    if deadline is not None:
        return host_admission.control_unit_observation(unit_name(state), deadline)
    result = systemctl(
        "show",
        "--property=InvocationID",
        "--property=ControlGroup",
        "--property=ActiveState",
        unit_name(state),
    )
    observed = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if not re.fullmatch(r"[a-f0-9]{32}", observed.get("InvocationID", "")):
        raise SupervisorError("Storage unit lacks an actual invocation identity")
    return observed


def resume_storage_allocation(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> host_admission.Allocation:
    """Reuse charged capacity only after the exact owned predecessor has drained."""
    owner, launch = recorded_storage_owner(state, config, deadline=deadline)
    observed = storage_unit_observation(state, deadline=deadline)
    group = observed.get("ControlGroup", "")
    binding = object_mapping(launch["binding"])
    if (
        observed.get("ActiveState") not in {"active", "activating"}
        or not group.startswith("/")
        or group == "/"
        or ".." in Path(group).parts
        or operation_lifetime.process_group(os.getpid()) != group
        or observed["InvocationID"] == binding.get("invocation")
        or group != binding.get("group")
    ):
        raise SupervisorError("Cannot adopt an uncertain storage restart lifetime")
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    processes = {
        int(pid)
        for path in directory.rglob("cgroup.procs")
        for pid in path.read_text().split()
    }
    if processes != {os.getpid()}:
        raise SupervisorError("Storage predecessor or descendants remain undrained")
    if deadline is not None:
        remaining(deadline)
    return owner


def materialize_service(
    state: Path,
    config: dict[str, object],
    owner: host_admission.Allocation | None = None,
) -> None:
    """Persistent owned unit; ExecStart only starts storage, never science."""
    supervisor = config.get("service_supervisor")
    if not isinstance(supervisor, dict):
        raise SupervisorError(
            "Profile has no immutable service supervisor; explicitly readmit it"
        )
    activatable_generation(Path(str(supervisor["supervisor_script"])).parents[1])
    directory = (
        Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
        / "systemd/user"
    )
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / unit_name(state)
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Missing service allocation")

    # Escaping is systemd ExecStart syntax, never shell syntax.
    def argument(value: object) -> str:
        text = str(value)
        if any(character in text for character in "\n\r\0"):
            raise SupervisorError("Invalid systemd unit argument")
        return (
            '"'
            + text.replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%")
            + '"'
        )

    executable = " ".join(
        argument(value)
        for value in (
            supervisor["supervisor_executable"],
            "-I",
            "-B",
            supervisor["supervisor_script"],
            "_serve",
            "--state",
            state,
        )
    )
    config["restart_qualified"] = recovery_qualified(state, config)
    maintenance = (state / ".maintenance-listener.json").exists()
    restart = "on-failure" if config["restart_qualified"] and not maintenance else "no"

    if owner is None:
        admitted = host_admission.inherit(os.environ, services=ServiceAdmission())
        if admitted is not None and admitted.profile.exclusive:
            owner = admitted
        else:
            profile = host_admission.select(
                "store-" + str(config.get("service_class", "functional")),
                str(allocation["server_memory_bytes"]),
            )
            owner = host_admission.Allocation(
                state, "", profile, 0, services=ServiceAdmission()
            )
    placement = storage_placement(config, owner)
    text = f"[Unit]\nDescription=pse-arrow owned storage {config['instance_id']}\nStartLimitIntervalSec=300\nStartLimitBurst=3\n\n[Service]\nType=exec\nExecStart={executable}\nKillMode=control-group\nTimeoutStopSec=45\nRestart={restart}\nRestartSec=5\nSlice={placement['slice']}\nMemoryMax={placement['memory_bytes']}\nCPUQuota={integer(placement['cpu_threads']) * 100}%\nMemorySwapMax=0\nTasksMax=128\nStandardOutput=null\nStandardError=null\n\n[Install]\nWantedBy=default.target\n"
    if path.exists() and not config.get("unit_materialized"):
        selected = config_for(state)
        # Recover a lost publication ACK only for the exact private declaration
        # under its closed, drained lifecycle owner. Equality never proves drain.
        if (
            getattr(_LIFECYCLE, "state", None) != state
            or config.get("accepting_writes") is not False
            or config.get("admission") not in {"quiesced", "validation_required"}
            or any(candidate.is_symlink() for candidate in (path, *path.parents))
            or not path.is_file()
            or path.stat().st_uid != os.getuid()
            or stat.S_IMODE(path.stat().st_mode) != 0o600
            or path.read_text() != text
            or any(
                selected.get(key) != config.get(key)
                for key in (
                    "instance_id",
                    "service_supervisor",
                    "server",
                    "resources",
                    "service_class",
                )
            )
        ):
            raise SupervisorError(
                "Existing user unit is not this generation's materialization"
            )
        all_contexts_drained(state)
        observed = service_readiness(
            state, config, deadline=getattr(_STARTUP, "deadline", None)
        )
        if (
            observed.phase is not Readiness.ACTIONABLE_ABSENT
            or observed.state != state
            or observed.unit != unit_name(state)
            or observed.instance_id != config["instance_id"]
            or observed.manager.get("ActiveState") not in {"inactive", "failed"}
            or observed.listener_owned
        ):
            raise SupervisorError(
                "Materialization recovery requires the exact inactive drained service"
            )
    temporary = path.with_suffix(".pending")
    temporary.write_text(text)
    temporary.chmod(0o600)
    temporary.replace(path)
    config["unit_materialized"] = True
    write_json(state / "config.json", config)
    systemctl("daemon-reload")
    if config.get("resident") and not maintenance:
        systemctl("enable", unit_name(state))


def service_allocation(
    state: Path, config: dict[str, object], deadline: float
) -> host_admission.Allocation:
    selected_telemetry = recorded_telemetry_state(config)

    inherited = inherit_host_owner(deadline)
    if inherited is not None and inherited.profile.exclusive:
        allocation = inherited
    else:
        profile = (
            "store-timing"
            if config.get("service_class") == "timing"
            else "store-functional"
        )
        requested = str(object_mapping(config["resources"])["server_memory_bytes"])
        allocation = host_admission.acquire(
            host_admission.select(profile, requested),
            deadline=deadline,
            services=ServiceAdmission(),
        )
    storage_placement(config, allocation)
    allocation.register(unit_name(state), deadline=deadline)
    if allocation.profile.exclusive:
        with host_admission.allocation_metadata(
            allocation.directory, deadline=deadline
        ) as ledger:
            borrowed = ledger["owners"][allocation.nonce].setdefault(
                "borrowed_services", []
            )
            if str(state) not in borrowed:
                borrowed.append(str(state))
    if not allocation.profile.exclusive:
        with host_admission.allocation_metadata(
            allocation.directory, deadline=deadline
        ) as ledger:
            ledger["owners"][allocation.nonce].update(
                service=str(state), parkable=bool(config.get("resident"))
            )
    remaining(deadline)
    write_json(
        state / "service-launch.json",
        {
            "allocation": str(allocation.directory / allocation.nonce),
            "deadline": deadline,
            "generation": config["instance_id"],
            "supervisor_generation": config.get("service_supervisor"),
            "server_generation": config.get("server"),
            "service_class": config.get("service_class", "functional"),
            "telemetry_state": selected_telemetry,
            "server_memory_bytes": object_mapping(config["resources"])[
                "server_memory_bytes"
            ],
        },
    )
    return allocation


def park_service(
    state: Path,
    *,
    deadline: float | None = None,
    expected_binding: host_admission.BoundUnit | None = None,
) -> bool:
    """Park only this materialized idle service; preserve all disk and evidence."""
    previous = getattr(_STARTUP, "deadline", None)
    if deadline is not None:
        _STARTUP.deadline = deadline if previous is None else min(previous, deadline)
    try:
        with lifecycle_reservation(state):
            return (
                _park_service(state)
                if expected_binding is None
                else _park_service(state, expected_binding=expected_binding)
            )
    finally:
        _STARTUP.deadline = previous


def _park_service(
    state: Path, *, expected_binding: host_admission.BoundUnit | None = None
) -> bool:
    config = config_for(state)
    if not config.get("resident") or not config.get("unit_materialized"):
        raise SupervisorError(
            "Service lifecycle is not eligible for coordinated parking"
        )
    deadline = getattr(_STARTUP, "deadline", None)
    observed = host_admission.control_unit_observation(unit_name(state), deadline)
    if observed.get("LoadState") == "not-found" or observed.get("ActiveState") in {
        "inactive",
        "failed",
    }:
        return False
    group = observed.get("ControlGroup", "")
    if (
        observed.get("ActiveState") != "active"
        or not group.startswith("/")
        or group == "/"
        or ".." in Path(group).parts
        or not re.fullmatch(r"[a-f0-9]{32}", observed.get("InvocationID", ""))
    ):
        raise SupervisorError("Service parking requires its actual running lifetime")
    if expected_binding is not None and (
        type(expected_binding.get("inode")) is not int
        or group != expected_binding.get("group")
        or observed["InvocationID"] != expected_binding.get("invocation")
        or host_admission.group_identity(group) != expected_binding.get("inode")
    ):
        raise SupervisorError("Service parking lifetime changed since host admission")
    all_contexts_drained(state)
    if host_admission.control_unit_observation(
        unit_name(state), deadline
    ) != observed or (
        expected_binding is not None
        and host_admission.group_identity(group) != expected_binding.get("inode")
    ):
        raise SupervisorError("Service parking lifetime changed while draining")
    with state_lock(state):
        config = config_for(state)
        config["parked"] = True
        write_json(state / "config.json", config)
    stop(state, config)
    return True


def unpark_service(state: Path, *, deadline: float | None = None) -> None:
    with lifecycle_reservation(state, purpose="startup", deadline=deadline):
        config = config_for(state)
        if config.get("parked"):
            start(state, config, deadline=deadline)


def startup_selection(config: dict[str, object]) -> dict[str, object]:
    return {
        key: value
        for key, value in config.items()
        if key
        not in {
            "admission",
            "accepting_writes",
            "unit_materialized",
            "restart_qualified",
            "parked",
        }
    }


def start(
    state: Path,
    config: dict[str, object],
    *,
    validation: bool = False,
    deadline: float | None = None,
) -> None:
    if config.get("backup_pending") is not None:
        raise SupervisorError(
            "Backup pending owner keeps startup closed until explicit resume completes"
        )
    deadline = time.monotonic() + 40 if deadline is None else deadline
    previous = getattr(_STARTUP, "deadline", None)
    deadline = deadline if previous is None else min(previous, deadline)
    _STARTUP.deadline = deadline
    try:
        remaining(deadline)
        owner = service_directory(state)
        if owner != state:
            return start(
                owner, config_for(owner), validation=validation, deadline=deadline
            )
        recorded_telemetry_state(config)
        if (
            config["admission"] == "validation_required"
            or config.get("derived_rebuild_pending")
            or config.get("surrealkit_rollout_pending")
        ) and not validation:
            raise SupervisorError(
                "Restored database requires validate before normal server start"
            )
        selection = startup_selection(config)
        while True:
            remaining(deadline)
            current = config_for(state)
            if startup_selection(current) != selection:
                raise SupervisorError(
                    "Service selection changed while joining lifecycle owner"
                )
            config.update(current)
            observed = service_readiness(state, config, deadline=deadline)
            if observed.phase in {Readiness.UNAVAILABLE, Readiness.MISMATCH}:
                raise SupervisorError(observed.diagnostic() + "; startup refused")
            if observed.reservation is not None:
                wait_reservation(
                    state,
                    state / "lifecycle-owner.json",
                    observed.reservation["nonce"],
                    deadline,
                )
                continue
            if observed.phase is Readiness.STOPPING:
                raise SupervisorError(
                    observed.diagnostic() + "; startup replacement refused"
                )
            if observed.phase is Readiness.STARTING and not observed.listener_owned:
                time.sleep(min(0.02, remaining(deadline)))
                continue
            if (
                observed.phase is Readiness.READY
                and config["admission"] == "open"
                and config["accepting_writes"]
                and not config.get("parked")
                and not (state / ".maintenance-listener.json").exists()
            ):
                return None
            try:
                with lifecycle_reservation(state, purpose="startup", deadline=deadline):
                    current = config_for(state)
                    if startup_selection(current) != selection:
                        raise SupervisorError(
                            "Service selection changed before startup reservation"
                        )
                    config.update(current)
                    _start(state, config, validation=validation, deadline=deadline)
            except LifecycleBusyError as busy:
                # The live owner may be stopping or maintaining, not starting.
                # Wait only for its release, then observe again before reserving.
                wait_reservation(
                    state, state / "lifecycle-owner.json", busy.nonce, deadline
                )
            else:
                return None
    finally:
        _STARTUP.deadline = previous


def reset_maintenance_failure_window(
    state: Path, config: dict[str, object], deadline: float
) -> None:
    """Intentional terminal maintenance launches do not spend the crash restart budget."""
    with lifecycle_deadline(deadline) as selected_deadline:
        if selected_deadline is None:
            raise SupervisorError(
                "Maintenance failure-window reset requires a deadline"
            )
        require_maintenance_owner(state, config)
        all_contexts_drained(state)
        observed = service_readiness(state, config, deadline=selected_deadline)
        if observed.phase is not Readiness.ACTIONABLE_ABSENT:
            raise SupervisorError(
                "Maintenance failure-window reset requires the exact drained terminal service"
            )
        remaining(selected_deadline)
        reset = systemctl("reset-failed", unit_name(state), check=False)
        if reset.returncode:
            # A newly materialized or collected unit can refuse reset-failed
            # because no executed invocation/failure window exists. Reauthenticate
            # that exact absence; other manager refusals remain fatal.
            selected = config_for(state, deadline=selected_deadline)
            require_maintenance_owner(state, selected)
            all_contexts_drained(state)
            absent = service_readiness(state, selected, deadline=selected_deadline)
            manager = systemctl(
                "show",
                "--property=LoadState",
                "--property=ActiveState",
                "--property=ControlGroup",
                "--property=InvocationID",
                unit_name(state),
                check=False,
            )
            fields = dict(
                line.split("=", 1)
                for line in manager.stdout.splitlines()
                if "=" in line
            )
            if (
                selected != config
                or absent.phase is not Readiness.ACTIONABLE_ABSENT
                or absent.state != state
                or absent.unit != unit_name(state)
                or absent.instance_id != config["instance_id"]
                or absent.listener_owned
                or manager.returncode
                or fields.get("LoadState") not in {"loaded", "not-found"}
                or fields.get("ActiveState") != "inactive"
                or fields.get("ControlGroup") != ""
                or fields.get("InvocationID") != ""
                or any(
                    absent.manager.get(key) != fields.get(key)
                    for key in ("ActiveState", "ControlGroup", "InvocationID")
                )
                or config_for(state, deadline=selected_deadline) != selected
            ):
                raise SupervisorError(
                    "Exact owned maintenance failure-window reset was refused"
                )
            require_maintenance_owner(state, selected)
            remaining(selected_deadline)


def _start(
    state: Path,
    config: dict[str, object],
    *,
    validation: bool,
    deadline: float,
) -> None:
    if config.get("backup_pending") is not None:
        raise SupervisorError(
            "Backup pending owner keeps startup closed until explicit resume completes"
        )
    supervisor = config.get("service_supervisor")
    if not isinstance(supervisor, dict):
        raise SupervisorError(
            "Profile has no immutable service supervisor; explicitly readmit it"
        )
    activatable_generation(Path(str(supervisor["supervisor_script"])).parents[1])
    owner = service_directory(state)
    if owner != state:
        return start(owner, config_for(owner), validation=validation, deadline=deadline)
    if deadline is None:
        deadline = time.monotonic() + 40
    if (
        config["admission"] == "validation_required"
        or config.get("derived_rebuild_pending")
        or config.get("surrealkit_rollout_pending")
    ) and not validation:
        raise SupervisorError(
            "Restored database requires validate before normal server start"
        )
    maintenance = (state / ".maintenance-listener.json").exists()
    if maintenance:
        require_maintenance_owner(state, config)
        if not validation:
            raise SupervisorError("Maintenance listener refuses ordinary startup")
    parked = bool(config.get("parked"))
    resumed = False
    try:
        observed = service_readiness(state, config, deadline=deadline)
        if observed.phase in {Readiness.UNAVAILABLE, Readiness.MISMATCH}:
            raise SupervisorError(observed.diagnostic() + "; startup refused")
        while observed.phase is Readiness.STARTING and not observed.listener_owned:
            time.sleep(min(0.02, remaining(deadline)))
            observed = service_readiness(state, config, deadline=deadline)
        if observed.phase in {
            Readiness.UNAVAILABLE,
            Readiness.MISMATCH,
            Readiness.STOPPING,
        }:
            raise SupervisorError(observed.diagnostic() + "; startup refused")
        if observed.phase in {
            Readiness.READY,
            Readiness.STARTING,
            Readiness.MAINTENANCE,
        }:
            if not observed.listener_owned:
                raise SupervisorError(
                    observed.diagnostic() + "; owned listener not ready"
                )
            if not maintenance and not observed.protocol_ready:
                establish_protocol_readiness(state, config, deadline)
            if parked:
                with state_lock(state):
                    config.update(config_for(state))
                    config["parked"] = False
                    write_json(state / "config.json", config)
            if not validation:
                open_context_admission(state)
                # Admission consumes the acknowledged payload freeze. Replace
                # the caller's snapshot so a later stop cannot republish it.
                current = dict(config_for(state))
                config.clear()
                config.update(current)
            resumed = True
            return None
        try:
            current_build = surreal_source.admitted_receipt(
                object_mapping(config["server"])
            )
        except surreal_source.BuildError as error:
            raise SupervisorError(str(error)) from error
        if not current_build:
            raise SupervisorError(
                "Server package changed or receipt is incomplete; explicitly install the pinned package before launch"
            )
        systemctl("show-environment")
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            raise SupervisorError("Invalid resource configuration")
        remaining(deadline)
        allocation_owner = service_allocation(state, config, deadline)
        environment = systemd_environment()
        host_admission.enforce_parent(
            allocation_owner.profile, environment, deadline=deadline
        )
        materialize_service(state, config, allocation_owner)
        remaining(deadline)
        if parked:
            # _serve may proceed only after deliberate startup owns its allocation.
            # Failed admission must retain the premise used by queued reconciliation.
            with state_lock(state):
                config.update(config_for(state))
                config["parked"] = False
                write_json(state / "config.json", config)
            # An admitted park/unpark is a new startup, not another failure retry.
            systemctl("reset-failed", unit_name(state), check=False)
        elif maintenance:
            reset_maintenance_failure_window(state, config, deadline)
        result = systemctl("start", unit_name(state), check=False)
        if result.returncode:
            allocation_owner.release()
            raise SupervisorError("Cannot start owned persistent service unit")
        while time.monotonic() < deadline:
            observed = service_readiness(state, config, deadline=deadline)
            if observed.phase is Readiness.UNAVAILABLE:
                time.sleep(min(0.1, remaining(deadline)))
                continue
            if observed.phase is Readiness.MISMATCH:
                raise SupervisorError(
                    observed.diagnostic() + "; admission remains closed"
                )
            if observed.listener_owned:
                if not maintenance:
                    establish_protocol_readiness(state, config, deadline)
                if not validation:
                    open_context_admission(state)
                    current = dict(config_for(state))
                    config.clear()
                    config.update(current)
                resumed = True
                return None
            if observed.phase is Readiness.ACTIONABLE_ABSENT:
                break
            time.sleep(min(0.1, remaining(deadline)))
        observed = service_readiness(state, config, deadline=deadline)
        if observed.phase in {
            Readiness.UNAVAILABLE,
            Readiness.MISMATCH,
        }:
            raise SupervisorError(observed.diagnostic() + "; admission remains closed")
        systemctl("stop", unit_name(state), check=False)
        raise SupervisorError(
            "Server did not become healthy; inspect the private bounded server.log"
        )
    finally:
        if parked and not resumed:
            # Keep any partial lifetime charged through its existing owner. Closing
            # admission and retaining parking intent does not prove a service drain.
            with state_lock(state, cleanup=True):
                current = dict(config_for(state))
                config.clear()
                config.update(current)
                config["parked"] = True
                write_json(state / "config.json", config)
            close_context_admission(state, cleanup=True)
            config.update(config_for(state))


def stop(state: Path, config: dict[str, object], *, abrupt: bool = False) -> None:
    prior = getattr(_STARTUP, "deadline", None)
    deadline = time.monotonic() + 90
    _STARTUP.deadline = deadline if prior is None else min(deadline, prior)
    try:
        with lifecycle_reservation(state, purpose="stop", deadline=_STARTUP.deadline):
            close_context_admission(state)
            config.update(config_for(state))
            _stop(state, config, abrupt=abrupt)
    finally:
        _STARTUP.deadline = prior


def _stop(state: Path, config: dict[str, object], *, abrupt: bool = False) -> None:
    group_result = systemctl(
        "show", "--property=ControlGroup", "--value", unit_name(state), check=False
    )
    group = group_result.stdout.strip()
    group_path = (
        Path("/sys/fs/cgroup") / group.lstrip("/")
        if group.startswith("/") and group != "/"
        else None
    )
    config["accepting_writes"] = False
    if config["admission"] != "validation_required":
        config["admission"] = "quiesced"
        write_json(state / "config.json", config)
    if not abrupt:
        all_contexts_drained(state)
    if abrupt:
        if active(state):
            killed = systemctl(
                "kill",
                "--kill-whom=all",
                "--signal=SIGKILL",
                unit_name(state),
                check=False,
            )
            if killed.returncode and active(state):
                raise SupervisorError("Abrupt fixture SIGKILL was refused")
            # Do not enqueue a competing graceful stop while the kill is being
            # processed. Wait for collection of the killed process group.
        while True:
            remaining(_STARTUP.deadline)
            observation = systemctl(
                "show",
                "--property=ActiveState",
                "--value",
                unit_name(state),
                check=False,
            )
            if observation.returncode == 0 and observation.stdout.strip() in {
                "inactive",
                "failed",
            }:
                break
            time.sleep(0.05)
    else:
        # is-active excludes activating/deactivating, but they still require an
        # actual stop job before an offline action can claim terminal state.
        systemctl("stop", unit_name(state), check=False)
    result = systemctl(
        "show", "--property=ActiveState", "--value", unit_name(state), check=False
    )
    if result.returncode or result.stdout.strip() not in {"inactive", "failed"}:
        raise SupervisorError("Server remains active; database copy is unsafe")
    if group_path is not None:
        while group_path.exists():
            events = group_path / "cgroup.events"
            if not events.exists() or "populated 1" not in events.read_text():
                break
            remaining(_STARTUP.deadline)
            time.sleep(0.05)

    # Kernel cgroup population excludes zombies. Keep the allocation charged
    # until the recorded lifetime also disappears, as restart readiness requires.
    try:
        process = read_json(state / "server-process.json")
    except FileNotFoundError:
        process = None
    if process is not None:
        pid = integer(process["pid"])
        while True:
            try:
                identity = operation_lifetime.start_identity(pid)
            except FileNotFoundError:
                break
            if identity != process.get("start"):
                break
            remaining(_STARTUP.deadline)
            time.sleep(0.05)

    release_stopped_service(state, config)


def release_stopped_service(state: Path, config: dict[str, object]) -> None:
    """Release drained storage affiliation, never its caller's borrowed host lane."""
    launch_path = state / "service-launch.json"
    if not launch_path.exists():
        return
    deadline = getattr(_STARTUP, "deadline", None)
    launch = read_json(launch_path, deadline=deadline)
    if launch.get("generation") != config["instance_id"]:
        raise SupervisorError("Stopped service allocation has a different generation")

    try:
        owner = host_admission.inherit(
            {host_admission.MARKER: str(launch["allocation"])},
            handoff=True,
            deadline=deadline,
            services=ServiceAdmission(),
        )
    except host_admission.AdmissionError:
        # A dead launch caller is settled by host reconciliation using the
        # recorded cgroup inode/invocation, not by guessing a live replacement.
        if deadline is not None:
            remaining(deadline)
        return
    if owner is None:
        return
    unit = unit_name(state)
    if owner.profile.exclusive:
        with host_admission.allocation_metadata(
            owner.directory, deadline=deadline
        ) as ledger:
            record = ledger["owners"].get(owner.nonce)
            if not isinstance(record, dict):
                raise SupervisorError("Stopped storage allocation is unavailable")
            borrowed = record.get("borrowed_services", [])
            if not isinstance(borrowed, list):
                raise SupervisorError("Invalid borrowed storage affiliation")
            if str(state) not in borrowed:
                return  # A prior authenticated stop already returned this affiliation.
            binding = dict(record["units"].get(unit, {}))
            if not binding or binding != launch.get("binding"):
                raise SupervisorError("Stopped borrowed storage binding changed")
            # This projection observes storage only. Execution ACK guards stay
            # on the enclosing allocation, whose release checks them separately.
            guards = {}
            for nonce, guard in record.get("terminal_guards", {}).items():
                scope = guard.get("scope")
                if not isinstance(scope, dict) or not isinstance(
                    scope.get("unit"), str
                ):
                    raise SupervisorError("Invalid stopped storage terminal guard")
                if scope["unit"] == unit:
                    guards[nonce] = guard
            selected = {
                **record,
                "service": str(state),
                "borrowed_services": [],
                "units": {unit: binding},
                "terminal_guards": guards,
                "released": True,
            }
        # Kernel and recorded process proof occurs outside metadata exclusion.
        # Keeping the stopped affiliation pins a disposable profile until its
        # enclosing caller exits, even though that caller may own other work.
        if not host_admission.drained(
            selected,
            allocation=owner.directory / owner.nonce,
            deadline=deadline,
            services=ServiceAdmission(),
        ):
            raise SupervisorError("Stopped borrowed storage lifetime is undrained")
        with host_admission.allocation_metadata(
            owner.directory, deadline=deadline
        ) as ledger:
            record = ledger["owners"].get(owner.nonce)
            if (
                not isinstance(record, dict)
                or record["units"].get(unit) != binding
                or read_json(launch_path, deadline=deadline) != launch
            ):
                raise SupervisorError("Stopped borrowed storage affiliation changed")
            borrowed = record.get("borrowed_services", [])
            if not isinstance(borrowed, list):
                raise SupervisorError("Invalid borrowed storage affiliation")
            if str(state) in borrowed:
                if deadline is not None:
                    remaining(deadline)
                borrowed.remove(str(state))
        return
    with host_admission.allocation_metadata(
        owner.directory, deadline=deadline
    ) as ledger:
        record = ledger["owners"].get(owner.nonce)
        if (
            not isinstance(record, dict)
            or record.get("service") != str(state)
            or unit_name(state) not in record["units"]
        ):
            raise SupervisorError(
                "Stopped storage allocation does not own this service"
            )
    if not owner.release(deadline=deadline):
        raise SupervisorError(
            "Stopped storage allocation still owns an undrained lifetime"
        )


def study_module(config: dict[str, object]) -> dict[str, object]:
    """Authenticate the exact immutable policy artifact selected by this service."""
    try:
        module = surrealism_module.verify(config.get("study_module"))
    except (surrealism_module.ModuleError, OSError, ValueError) as error:
        raise SupervisorError(
            "Study module is absent or mismatched: " + str(error)
        ) from error
    if object_mapping(module["inputs"]).get("builder_sha256") != object_mapping(
        config["server"]
    ).get("binary_sha256"):
        raise SupervisorError(
            "Study module was not built by this exact admitted server"
        )
    return module


def qualification_module(config: dict[str, object]) -> dict[str, object] | None:
    """A separate explicit probe receipt never acquires production module authority."""
    value = config.get("study_module_qualification")
    if value is None:
        return None
    try:
        module = surrealism_module.verify(
            value,
            qualification=True,
            long_hold=object_mapping(value).get("kind")
            == surrealism_module.LONG_HOLD_KIND,
        )
    except (surrealism_module.ModuleError, OSError, ValueError) as error:
        raise SupervisorError(
            "Qualification study module is mismatched: " + str(error)
        ) from error
    if object_mapping(module["inputs"]).get("builder_sha256") != object_mapping(
        config["server"]
    ).get("binary_sha256"):
        raise SupervisorError(
            "Qualification module builder differs from the selected server"
        )
    return module


def select_module_qualification(
    state: Path, receipt_path: Path | None, *, long_hold: bool = False
) -> dict[str, object]:
    """Select/remove probe access only after the actual owned service and queries drain."""
    if long_hold and receipt_path is None:
        raise SupervisorError(
            "Long-hold selection requires its explicit qualification receipt"
        )
    with lifecycle_deadline(time.monotonic() + 30), lifecycle_reservation(state):
        config = config_for(state)
        private_offline_state(state, config)
        if active(state) or config.get("accepting_writes") is not False:
            raise SupervisorError(
                "Qualification module selection requires stopped closed service admission"
            )
        all_contexts_drained(state)
        selected = dict(config)
        if receipt_path is None:
            selected.pop("study_module_qualification", None)
        else:
            receipt = read_json(
                receipt_path, deadline=getattr(_STARTUP, "deadline", None)
            )
            expected_kind = (
                surrealism_module.LONG_HOLD_KIND
                if long_hold
                else surrealism_module.QUALIFICATION_KIND
            )
            if receipt.get("kind") != expected_kind:
                raise SupervisorError(
                    "Qualification receipt differs from the explicit long-hold selection"
                )
            selected["study_module_qualification"] = receipt
            qualification_module(selected)
        selected["restart_qualified"] = False
        history = state / ".profile-history"
        history.mkdir(mode=0o700, exist_ok=True)
        write_json(history / (uuid.uuid4().hex + ".json"), config)
        with state_lock(state):
            write_json(state / "config.json", selected)
        return public_status(state, selected)


def module_runtime_allocation(allocation: dict[str, object]) -> dict[str, int]:
    """Keep retained module linear memory within one eighth of the server budget.

    Cache size bounds retained entries, not evicted in-flight runtime aliases.
    Aggregate controller admission is independently selected below.
    """
    memory = integer(surrealism_module.LIMITS["max_memory_bytes"])
    pool = integer(surrealism_module.LIMITS["max_pool_size"])
    controllers = integer(allocation["server_memory_bytes"]) // 8 // memory
    cache = min(4, controllers // pool)
    if cache < 1:
        raise SupervisorError(
            "Server budget cannot admit one complete bounded study module pool"
        )
    return {
        "cache_modules": cache,
        "max_controllers": controllers,
        "max_memory_bytes": memory,
    }


def recorded_telemetry_state(config: dict[str, object]) -> str:
    selected = config.get("telemetry_state")
    if (
        not isinstance(selected, str)
        or not Path(selected).is_absolute()
        or str(telemetry_service.state_directory(selected)) != selected
    ):
        raise SupervisorError(
            "Service lacks a recorded telemetry selection; readmit its stopped supervisor with admission closed"
        )
    return selected


def server_environment(
    config: dict[str, object],
    credentials: dict[str, object],
    *,
    bootstrap: bool = True,
    telemetry_instance_id: str | None = None,
) -> dict[str, str]:
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid allocation")
    module = study_module(config)
    qualification = qualification_module(config)
    module_directories = [str(Path(str(module["artifact"])).parent)]
    if qualification is not None:
        module_directories.append(str(Path(str(qualification["artifact"])).parent))
    runtime = module_runtime_allocation(allocation)
    service_class = str(config.get("service_class", "functional"))
    pool_threads = str(
        len(
            host_admission.cpu_set(
                host_admission.select("store-" + service_class).cores
            )
        )
    )
    execution_ms = integer(object_mapping(module["limits"])["max_execution_time_ms"])
    if qualification is not None:
        execution_ms = max(
            execution_ms,
            integer(object_mapping(qualification["limits"])["max_execution_time_ms"]),
        )
    # Inherited SURREAL_* settings cannot silently override the owned profile.
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SURREAL_", "OTEL_"))
    }
    environment.update(
        {
            "SURREAL_USER": str(credentials["username"]),
            "SURREAL_PASS": str(credentials["password"]),
            "SURREAL_GRPC_MAX_MESSAGE_SIZE": str(MESSAGE_BYTES),
            "SURREAL_GRPC_MAX_ATTACHED_SESSIONS": str(
                allocation["grpc_max_attached_sessions"]
            ),
            "SURREAL_MEMORY_THRESHOLD": str(allocation["memory_threshold_bytes"]),
            "SURREAL_CAPS_ALLOW_EXPERIMENTAL": "files,surrealism",
            "SURREAL_BUCKET_FOLDER_ALLOWLIST": os.pathsep.join(module_directories),
            "SURREAL_SURREALISM_CACHE_SIZE": str(runtime["cache_modules"]),
            "SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS": str(runtime["max_controllers"]),
            "SURREAL_SURREALISM_MAX_MEMORY": str(runtime["max_memory_bytes"]),
            "SURREAL_SURREALISM_MAX_EXECUTION_TIME": str(execution_ms),
            "SURREAL_SURREALISM_MAX_KV_ENTRIES": "0",
            "SURREAL_SURREALISM_MAX_KV_VALUE_BYTES": "0",
            "SURREAL_SURREALISM_MAX_FS_BYTES": "0",
            "SURREAL_SURREALISM_LOG_LEVEL": "warn",
            "SURREAL_RUNTIME_WORKER_THREADS": pool_threads,
            "TOKIO_WORKER_THREADS": pool_threads,
            "RAYON_NUM_THREADS": pool_threads,
        }
    )
    instance = telemetry_instance_id or uuid.uuid4().hex
    if not re.fullmatch(r"[a-f0-9]{32}", instance):
        raise SupervisorError("Invalid server telemetry process incarnation")
    environment["OTEL_RESOURCE_ATTRIBUTES"] = (
        f"service.instance.id={instance},pse.configuration.id={config['instance_id']}"
    )
    selected_state = recorded_telemetry_state(config)
    environment["PSE_TELEMETRY_STATE"] = selected_state
    selection = telemetry_service.producer_configuration(
        {"PSE_TELEMETRY_STATE": selected_state}
    )
    if selection is not None:
        endpoint = selection.get("otlp_grpc_endpoint")
        if not isinstance(endpoint, str) or not re.fullmatch(
            r"http://127\.0\.0\.1:[0-9]{1,5}", endpoint
        ):
            raise SupervisorError("Selected telemetry gRPC ingress must be loopback")
        environment.update(
            SURREAL_TELEMETRY_PROVIDER="otlp",
            SURREAL_TELEMETRY_DISABLE_METRICS="false",
            SURREAL_TELEMETRY_DISABLE_TRACING="false",
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT=endpoint,
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=endpoint,
            OTEL_METRIC_EXPORT_INTERVAL="10000",
        )
    telemetry = config.get("diagnostic_telemetry")
    if telemetry is not None:
        selected_telemetry = object_mapping(telemetry)
        endpoint = selected_telemetry.get("endpoint")
        if (
            set(selected_telemetry) != {"endpoint", "capture_nonce"}
            or not isinstance(endpoint, str)
            or not re.fullmatch(r"http://127\.0\.0\.1:[0-9]{1,5}", endpoint)
            or not 0 < int(endpoint.rsplit(":", 1)[1]) < 65536
            or not re.fullmatch(
                r"[a-f0-9]{32}", str(selected_telemetry.get("capture_nonce"))
            )
        ):
            raise SupervisorError(
                "Temporary diagnostic telemetry must name its owned loopback collector"
            )
        environment.update(
            SURREAL_TELEMETRY_PROVIDER="otlp",
            SURREAL_TELEMETRY_DISABLE_TRACING="false",
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=endpoint,
            OTEL_BSP_SCHEDULE_DELAY="100",
        )
        if selection is None:
            environment["SURREAL_TELEMETRY_DISABLE_METRICS"] = "true"
    if not bootstrap:
        environment.pop("SURREAL_USER", None)
        environment.pop("SURREAL_PASS", None)
    return environment


LOG_CONFIGURATIONS = {
    "baseline": {"filter": "warn", "format": "text"},
    "mcp-audit": {"filter": "warn,surrealdb::mcp::audit=info", "format": "json"},
}


def log_configuration(config: Mapping[str, object]) -> dict[str, str]:
    selection = config.get("diagnostic_logging", "baseline")
    if not isinstance(selection, str) or selection not in LOG_CONFIGURATIONS:
        raise SupervisorError("Unknown explicit diagnostic log configuration")
    return dict(LOG_CONFIGURATIONS[selection])


def log_arguments(config: Mapping[str, object]) -> list[str]:
    selected = log_configuration(config)
    return ["--log=" + selected["filter"], "--log-format=" + selected["format"]]


def inspect_log_configuration(state: Path, deadline: float) -> dict[str, object]:
    config = config_for(state, deadline=deadline)
    receipt = state / "diagnostic-log-change.json"
    return {
        "state": str(state),
        "instance_id": config["instance_id"],
        "selected": log_configuration(config),
        "candidate": LOG_CONFIGURATIONS["mcp-audit"],
        "effect": "explicit owned restart required; no live reload or allocation reconfiguration",
        "change": read_json(receipt, deadline=deadline) if receipt.exists() else None,
    }


def mcp_audit_records(state: Path, deadline: float) -> dict[str, object]:
    """Return only native audit metadata; never echo arbitrary server log text or payloads."""
    selected = selected_service(state, deadline)
    maximum = integer(config_for(selected.state, deadline=deadline)["log_max_bytes"])
    raw = private_record_bytes(
        selected.state / "server.log", deadline, maximum_bytes=maximum
    )
    events: list[dict[str, object]] = []
    malformed = 0
    for line in raw.splitlines():
        remaining(deadline)
        try:
            event = object_mapping(json.loads(line))
        except (ValueError, SupervisorError):
            malformed += 1
            continue
        if event.get("target") != "surrealdb::mcp::audit":
            continue
        fields = object_mapping(event.get("fields", {}))
        events.append(
            {
                name: fields.get(name)
                for name in (
                    "tool",
                    "subject",
                    "namespace",
                    "database",
                    "outcome",
                    "kind",
                    "time_ms",
                )
            }
        )
    selected.recheck(deadline)
    return {
        "identity": selected.identity,
        "records": events[-64:],
        "omitted_records": max(0, len(events) - 64),
        "unparsed_lines": malformed,
        "scope": "current bounded private log only; rotation or baseline text can omit observations",
    }


def restart_log_configuration(
    state: Path, selection: str, deadline: float
) -> dict[str, object]:
    """Apply/restore only selected logging through the shared explicit diagnostic restart owner."""
    if selection not in LOG_CONFIGURATIONS:
        raise SupervisorError("Unsupported diagnostic logging selection")
    return restart_diagnostic_configuration(
        state, {"diagnostic_logging": selection}, selection, "log", deadline
    )


def restart_diagnostic_configuration(
    state: Path,
    updates: dict[str, object],
    selection: str,
    category: str,
    deadline: float,
) -> dict[str, object]:
    """Explicit logging/temporary telemetry changes share drain, immutable publication and restart."""
    if set(updates) - {
        "diagnostic_logging",
        "diagnostic_telemetry",
    } or category not in {"log", "trace"}:
        raise SupervisorError("Unsupported diagnostic service configuration")
    state = lexical_absolute(state)
    with (
        lifecycle_deadline(deadline),
        lifecycle_reservation(state, purpose="maintenance", deadline=deadline),
    ):
        config = config_for(state, deadline=deadline)
        if (
            config.get("service_state") is not None
            or config.get("derived_rebuild_pending")
            or config.get("surrealkit_rollout_pending")
        ):
            raise SupervisorError(
                "Log restart requires an independent settled service owner"
            )
        before = selected_service(state, deadline)
        receipt: dict[str, object] = {
            "schema": "diagnostic-log-change-v1",
            "selection": selection,
            "previous": {
                "logging": log_configuration(config),
                "telemetry": config.get("diagnostic_telemetry"),
            },
            "before": before.identity,
            "phase": "quiescing",
        }
        path = state / ("diagnostic-" + category + "-change.json")
        write_json(path, receipt)

        def restart_selected() -> None:
            close_context_admission(state, admission="quiescing")
            for context in managed_contexts(state):
                selected = config_for(context, deadline=deadline)
                drain_context(context, str(selected["database"]), deadline=deadline)
            all_contexts_drained(state)
            if (
                storage_unit_observation(state, deadline=deadline).get("InvocationID")
                != before.identity["invocation"]
            ):
                raise SupervisorError("Selected service changed before log restart")
            stop(state, config_for(state, deadline=deadline))
            receipt["phase"] = "stopped"
            write_json(path, receipt)
            with state_lock(state):
                current = config_for(state, deadline=deadline)
                if current["instance_id"] != before.identity["instance_id"]:
                    raise SupervisorError(
                        "Selected generation changed before log application"
                    )
                current.update(updates)
                current["restart_qualified"] = False
                write_json(state / "config.json", current)
            # Publish the current immutable supervisor explicitly; old artifacts are never edited.
            readmit_supervisor(state, deadline=deadline)
            receipt["phase"] = "starting"
            write_json(path, receipt)
            start(state, config_for(state, deadline=deadline), deadline=deadline)
            after = selected_service(state, deadline)
            if after.identity["invocation"] == before.identity["invocation"]:
                raise SupervisorError("Log restart did not produce a new invocation")
            receipt.update(
                phase="applied"
                if selection in {"mcp-audit", "telemetry-capture"}
                else "restored",
                after=after.identity,
            )
            write_json(path, receipt)

        try:
            restart_selected()
        except BaseException:
            # A partial start is not a successful application. Keep admission closed and
            # preserve the selected phase for explicit stop/recovery rather than restart it.
            close_context_admission(state)
            receipt["outcome"] = (
                "interrupted; admission closed, inspect selected service and pending phase"
            )
            write_json(path, receipt)
            raise

        return receipt


class BoundedLog:
    """Bound each write and retain a fixed number of private rotated files."""

    def __init__(self, path: Path, maximum: int, backups: int, secret: str):
        self.path, self.maximum, self.backups, self.secret = (
            path,
            maximum,
            backups,
            secret.encode(),
        )

    def append(self, block: bytes) -> None:
        block = block.replace(self.secret, b"[REDACTED]")
        for offset in range(0, len(block), self.maximum):
            part = block[offset : offset + self.maximum]
            if (
                self.path.exists()
                and self.path.stat().st_size + len(part) > self.maximum
            ):
                for index in range(self.backups, 0, -1):
                    source = (
                        self.path
                        if index == 1
                        else self.path.with_suffix(f".log.{index - 1}")
                    )
                    destination = self.path.with_suffix(f".log.{index}")
                    if source.exists():
                        source.replace(destination)
            fd = os.open(
                self.path, os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW, 0o600
            )
            with os.fdopen(fd, "ab") as output:
                output.write(part)

    def capture(self, stream: io.BufferedIOBase) -> None:
        # Retain overlap so a secret split across pipe reads is still redacted.
        tail = b""
        overlap = len(self.secret)
        while block := stream.read1(8192):
            tail += block
            safe = len(tail) - overlap
            # Complete records can be published immediately: generated credentials
            # contain no newline, so a token cannot cross this record boundary.
            if b"\n" not in self.secret:
                safe = max(safe, tail.rfind(b"\n") + 1)
            if safe > 0:
                # Redact before splitting; retaining the overlap handles partial tokens.
                redacted = tail.replace(self.secret, b"[REDACTED]")
                if redacted != tail:
                    tail = redacted
                    safe = max(0, len(tail) - overlap)
                    if b"\n" not in self.secret:
                        safe = max(safe, tail.rfind(b"\n") + 1)
                self.append(tail[:safe])
                tail = tail[safe:]
        self.append(tail)


def serve(state: Path) -> int:
    """Give each manager startup one finite clock, independently of science."""
    prior = getattr(_STARTUP, "deadline", None)
    deadline = time.monotonic() + 40
    _STARTUP.deadline = deadline if prior is None else min(prior, deadline)
    try:
        return _serve(state)
    finally:
        _STARTUP.deadline = prior


def _serve(state: Path) -> int:
    config = config_for(state)
    if config.get("parked"):
        return 0
    maintenance = (state / ".maintenance-listener.json").exists()
    if maintenance:
        require_maintenance_owner(state, config)
    deadline = _STARTUP.deadline
    launch_path = state / "service-launch.json"
    if launch_path.is_file():
        launch = storage_launch(state, config, deadline=deadline)
        if "binding" in launch:
            allocation_owner = resume_storage_allocation(
                state, config, deadline=deadline
            )
        else:
            launch_deadline = launch["deadline"]
            if not isinstance(launch_deadline, (int, float)):
                raise SupervisorError("Storage launch lacks a finite admission clock")
            _STARTUP.deadline = min(deadline, launch_deadline)
            allocation_owner = host_admission.inherit(
                {host_admission.MARKER: str(launch["allocation"])},
                handoff=True,
                deadline=_STARTUP.deadline,
                services=ServiceAdmission(),
            )
        if allocation_owner is None:
            raise SupervisorError("Storage launch has no admitted owner")
    else:
        allocation_owner = service_allocation(state, config, deadline)
        launch = storage_launch(state, config, deadline=deadline)
    storage_placement(config, allocation_owner)
    if "binding" not in launch:
        launch_deadline = launch["deadline"]
        if not isinstance(launch_deadline, (int, float)):
            raise SupervisorError("Storage launch lacks a finite admission clock")
        _STARTUP.deadline = min(deadline, launch_deadline)
    remaining(_STARTUP.deadline)
    allocation_owner.bind(unit_name(state), deadline=_STARTUP.deadline)
    with host_admission.allocation_metadata(
        allocation_owner.directory, deadline=_STARTUP.deadline
    ) as ledger:
        binding = dict(
            ledger["owners"][allocation_owner.nonce]["units"][unit_name(state)]
        )
    launch["binding"] = binding
    remaining(_STARTUP.deadline)
    write_json(launch_path, launch)
    os.environ.update(allocation_owner.environment())
    host_admission.enforce_parent(
        allocation_owner.profile, systemd_environment(), deadline=_STARTUP.deadline
    )
    remaining(_STARTUP.deadline)
    os.sched_setaffinity(0, host_admission.cpu_set(allocation_owner.profile.cores))
    remaining(_STARTUP.deadline)
    credentials_path = state / "credentials.json"
    if (
        credentials_path.stat().st_mode & 0o077
        or credentials_path.stat().st_uid != os.getuid()
    ):
        raise SupervisorError(
            "Credentials must be owned by the current user with mode 0600"
        )
    credentials = read_json(credentials_path)
    server = config["server"]
    if not isinstance(server, dict):
        raise SupervisorError("Invalid server receipt")
    if server.get("binary_sha256") != file_digest(Path(str(server["binary"]))):
        raise SupervisorError(
            "Storage binary differs from its admitted generation; explicitly readmit"
        )
    database = database_endpoint(state)
    command = [
        str(server["binary"]),
        "start",
        "--no-banner",
        *log_arguments(config),
        "--no-defaults",
        "--deny-guests",
        "--deny-net",
        "--bind",
        f"127.0.0.1:{config['port']}",
        "--query-timeout=90s",
        "--transaction-timeout=90s",
        "--temporary-directory",
        str(state / "tmp"),
        database,
    ]
    log = BoundedLog(
        state / "server.log",
        integer(config["log_max_bytes"]),
        integer(config["log_backups"]),
        str(credentials["password"]),
    )
    # A recovery listener observes existing ROOT authorities; it must never
    # bootstrap a supplied candidate into an empty or unknown catalog.
    bootstrap = False
    if maintenance:
        require_maintenance_owner(state, config)
        listener = read_json(state / ".maintenance-listener.json")
        authority = listener.get("root_bootstrap")
        if authority is not None:
            if authority != fresh_root_authority(state, config):
                raise SupervisorError("Fresh ROOT bootstrap authority changed")
            bootstrap = True
    telemetry_instance_id = uuid.uuid4().hex
    environment = server_environment(
        config,
        credentials,
        bootstrap=not maintenance or bootstrap,
        telemetry_instance_id=telemetry_instance_id,
    )
    remaining(_STARTUP.deadline)
    child = subprocess.Popen(
        command,
        env=environment,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    write_json(
        state / "server-process.json",
        {
            "instance_id": config["instance_id"],
            "telemetry_service_instance_id": telemetry_instance_id,
            "pid": child.pid,
            "start": operation_lifetime.start_identity(child.pid),
            "allocation": launch["allocation"],
        },
    )

    def forward(signum: int, _frame: object) -> None:
        if child.poll() is None:
            child.send_signal(signum)

    signal.signal(signal.SIGTERM, forward)
    signal.signal(signal.SIGINT, forward)
    stream = child.stdout
    if stream is None:
        raise SupervisorError("Server output pipe was not created")

    if maintenance:
        threading.Thread(
            target=watch_maintenance_owner, args=(state, child), daemon=True
        ).start()
    reader = threading.Thread(target=log.capture, args=(stream,), daemon=True)
    reader.start()
    result = child.wait()
    reader.join(timeout=5)
    stream.close()
    return result


def payload_verifier(path: Path) -> dict[str, object]:
    path = lexical_absolute(path)
    if (
        any(part.is_symlink() for part in (path, *path.parents))
        or not path.is_file()
        or not os.access(path, os.X_OK)
    ):
        raise SupervisorError(
            "Payload verifier must be an explicit executable without symlinks"
        )
    return {"binary": str(path), "sha256": file_digest(path)}


def admitted_payload_verifier(config: Mapping[str, object]) -> dict[str, object]:
    verifier = object_mapping(config.get("payload_verifier", {}))
    binary = verifier.get("binary")
    if binary is None:
        binary = str(
            Path(__file__).resolve().parents[1] / "target/debug/pse-canonical-recovery"
        )
    if not isinstance(binary, str):
        raise SupervisorError("Native payload helper artifact is unavailable")
    try:
        actual = payload_verifier(Path(binary))
    except (SupervisorError, OSError) as error:
        raise SupervisorError(
            "Native payload helper unavailable; build pse-operations --bin pse-canonical-recovery or select --payload-verifier"
        ) from error
    if verifier and actual != verifier:
        raise SupervisorError(
            "Native payload helper differs from its selected artifact"
        )
    return actual


def set_payload_verifier(
    state: Path, config: dict[str, object], verifier: Path | None
) -> None:
    """Select an explicit helper without changing an original pending backup."""
    pending = backup_pending(config) if "backup_pending" in config else None
    if pending is not None and (
        config.get("instance_id") != pending["source_instance_id"]
        or backup_configuration_identity(config)
        != pending["source_configuration_sha256"]
    ):
        raise SupervisorError(
            "Backup source configuration differs from its original frozen owner"
        )
    if verifier is None:
        return
    selected = payload_verifier(verifier)
    original = config.get("payload_verifier")
    if pending is not None and selected != original:
        raise SupervisorError(
            "Pending backup requires its original exact native payload helper"
        )
    if selected != original:
        config["payload_verifier"] = selected
        write_json(state / "config.json", config)


def native_payload_operation(
    state: Path,
    config: Mapping[str, object],
    operation: str,
    inventory: list[dict[str, object]] | None = None,
    *,
    destination: Path | None = None,
    placement: Mapping[str, object] | None = None,
    target: Mapping[str, object] | None = None,
) -> list[dict[str, object]]:
    """Transport bounded metadata to the product's complete physical operation."""
    actual = admitted_payload_verifier(config)
    binary = str(actual["binary"])
    if isinstance(config, dict) and not config.get("payload_verifier"):
        config["payload_verifier"] = actual
    command = [binary, "--operation", operation, "--root", str(state)]
    if destination is not None:
        command.extend(("--destination", str(destination)))
    for option, value in (("--placement", placement), ("--target", target)):
        if value is not None:
            command.extend(
                (
                    option,
                    json.dumps(
                        {
                            name: value[name]
                            for name in ("payload_incarnation", "database")
                        },
                        separators=(",", ":"),
                    ),
                )
            )
    with (
        tempfile.TemporaryFile(mode="w+b") as request,
        tempfile.TemporaryFile(mode="w+b") as response,
        tempfile.TemporaryFile(mode="w+b") as errors,
    ):
        inventory_digest = hashlib.sha256()
        for entry in inventory or []:
            line = (
                json.dumps(entry, sort_keys=True, separators=(",", ":")) + "\n"
            ).encode()
            if len(line) > 64 * 1024:
                raise SupervisorError(
                    "Native payload entry exceeds its bounded envelope"
                )
            request.write(line)
            inventory_digest.update(line)
        request.seek(0)
        checkpoint = recovery_operation.checkpoint_path(
            command, inventory_digest.hexdigest(), str(actual["sha256"])
        )
        if (
            operation == "backup"
            and isinstance(config, dict)
            and config.get("backup_pending") is not None
        ):
            pending = object_mapping(config["backup_pending"])
            if pending.get("payload_checkpoint") not in {None, str(checkpoint)}:
                raise SupervisorError(
                    "Backup native checkpoint differs from its original pending owner"
                )
            pending["payload_checkpoint"] = str(checkpoint)
            config["backup_pending"] = pending
            write_json(state / "config.json", config)
        try:
            result = recovery_operation.run(
                command,
                request,
                response,
                errors,
                checkpoint,
                deadline=getattr(_STARTUP, "deadline", None),
            )
        except (
            recovery_operation.RecoveryError,
            host_admission.AdmissionError,
        ) as error:
            raise SupervisorError(
                f"{error}; maintenance remains incomplete; checkpoint={checkpoint}"
            ) from error
        if payload_verifier(Path(binary)) != actual:
            raise SupervisorError(
                "Native payload helper changed during its operation; maintenance remains incomplete"
            )
        if result:
            errors.seek(0)
            detail = errors.read(4096).decode(errors="replace").strip()
            raise SupervisorError(
                f"Native canonical payload operation refused; maintenance remains incomplete; checkpoint={checkpoint}: {detail}"
            )
        response.seek(0)
        values: list[dict[str, object]] = []
        while line := response.readline(64 * 1024 + 1):
            if len(line) > 64 * 1024:
                raise SupervisorError(
                    "Native payload response exceeds its bounded envelope"
                )
            try:
                values.append(object_mapping(json.loads(line)))
            except (ValueError, TypeError) as error:
                raise SupervisorError("Native payload response is malformed") from error
        if operation in {"inventory", "backup", "verify-live", "verify", "restore"}:
            if len(values) != len(inventory or []):
                raise SupervisorError(
                    "Native payload operation omitted an inventory acknowledgment"
                )
            if operation != "inventory" and values != (inventory or []):
                raise SupervisorError(
                    "Native payload operation returned a different immutable inventory"
                )
        return values


def selected_payload_helper(config: Mapping[str, object]) -> Path | None:
    binary = object_mapping(config.get("payload_verifier", {})).get("binary")
    return Path(binary) if isinstance(binary, str) else None


def payload_root(state: Path, selected: Mapping[str, object]) -> Path:
    value = native_payload_operation(state, selected, "layout", placement=selected)
    if (
        len(value) != 1
        or set(value[0]) != {"root"}
        or not isinstance(value[0]["root"], str)
    ):
        raise SupervisorError("Native payload layout acknowledgment is invalid")
    return Path(value[0]["root"])


def verify_payload_inventory(
    state: Path,
    config: dict[str, object],
    inventory: list[dict[str, object]],
    *,
    read_only_source: bool = False,
) -> None:
    """Verify one complete immutable archive; live inventory owns its locks itself."""
    if read_only_source:
        native_payload_operation(state, config, "verify", inventory, placement=config)
    else:
        native_payload_operation(
            state, config, "verify-live", inventory, placement=config
        )


def payload_query(state: Path, config: dict[str, object], body: str) -> object:
    """One bounded administrative transaction; no scientific RPC or implicit catalog creation."""
    deadline = getattr(_STARTUP, "deadline", None) or time.monotonic() + 90
    response = administrative_query(
        state,
        "BEGIN; " + body + " COMMIT;",
        deadline,
        namespace=str(config["namespace"]),
        database=str(config["database"]),
    )
    values = [row["result"] for row in response if row.get("result") is not None]
    if len(values) != 1:
        raise SupervisorError(
            "File maintenance did not return exactly one acknowledgment"
        )
    return values[0]


def payload_expiry() -> int:
    return time.time_ns() // 1000 + 90_000_000


def payload_pages(
    state: Path, config: dict[str, object], table: str
) -> Generator[list[dict[str, object]], None, None]:
    if table not in {"canonical_file_owners", "canonical_file_instances"}:
        raise SupervisorError("Unknown file inventory table")
    columns = (
        "key,issuing_store,storage_incarnation,boot,pid_namespace,<string>pid AS pid,start,invocation,unit,group,"
        "(IF group_inode=NONE {NONE} ELSE {<string>group_inode}) AS group_inode,drained"
        if table == "canonical_file_owners"
        else "key,intent,request_digest,owner,owner_kind,selection,issuing_store,storage_incarnation,digest,"
        "<string>payload_bytes AS payload_bytes,encoding,<string>batch_count AS batch_count,schema,interpretation,state"
    )
    after = ""
    while True:
        value = payload_query(
            state,
            config,
            # Table/columns are fixed above; cursor is a JSON string value.
            f"LET $rows=SELECT {columns} FROM {table} WHERE key>{json.dumps(after)} ORDER BY key LIMIT 64; RETURN $rows;",  # noqa: S608
        )
        if not isinstance(value, list) or len(value) > 64:
            raise SupervisorError("Invalid bounded file catalog inventory")
        rows: list[dict[str, object]] = []
        for raw in value:
            row = object_mapping(raw)
            if table == "canonical_file_owners":
                # Explicit SELECT projections encode absent native option fields
                # as JSON null. Restore their canonical absent-field shape only.
                row = {
                    name: field
                    for name, field in row.items()
                    if not (
                        name in {"group", "group_inode", "invocation", "unit"}
                        and field is None
                    )
                }
            key = row.get("key")
            if not isinstance(key, str) or key <= after:
                raise SupervisorError("File inventory cursor did not advance")
            rows.append(row)
            after = key
        if not rows:
            break
        yield rows


def managed_file_owner_drained(owner: dict[str, object]) -> bool:
    """A managed leader's death does not release its still-populated exact cgroup."""
    invocation = owner.get("invocation")
    if invocation is None:
        return True
    if not isinstance(invocation, str) or not invocation:
        raise SupervisorError("File owner has an invalid managed invocation")
    if owner.get("boot") != process_observation.boot():
        return True
    unit, group, inode = owner.get("unit"), owner.get("group"), owner.get("group_inode")
    if (
        not isinstance(unit, str)
        or not unit.endswith((".scope", ".service"))
        or "/" in unit
        or not isinstance(group, str)
        or Path(group).name != unit
    ):
        raise SupervisorError(
            "Invocation-bound file owner lacks exact managed scope identity"
        )
    if isinstance(inode, str) and inode.isascii() and inode.isdecimal():
        inode = int(inode)
    if type(inode) is not int or inode <= 0:
        raise SupervisorError("File owner managed scope identity is incomplete")
    observed = host_admission.group_identity(group)
    if observed is None or observed != inode:
        # Linux cannot remove a cgroup with live members. A replacement inode is
        # a different lifetime, not a target this owner may stop or adopt.
        return True
    observation = systemctl(
        "show", "--property=ControlGroup", "--property=InvocationID", unit, check=False
    )
    fields = dict(
        line.split("=", 1) for line in observation.stdout.splitlines() if "=" in line
    )
    if observation.returncode or fields != {
        "ControlGroup": group,
        "InvocationID": invocation,
    }:
        raise SupervisorError("File owner managed invocation cannot be authenticated")
    return not group_populated(group)


def payload_owner_live_refusal(
    row: dict[str, object], config: dict[str, object], observation: dict[str, object]
) -> str | None:
    """Return a live borrower reason, or authenticate a dead and drained owner."""
    if row.get("storage_incarnation") != config["payload_incarnation"]:
        raise SupervisorError("File owner belongs to another storage incarnation")
    if observation.get("key") != row.get("key"):
        raise SupervisorError("Native file owner observation identity differs")
    if "refusal" in observation:
        raise SupervisorError(
            "File owner drain is unobservable; admission remains closed"
        )
    if set(observation) != {"key", "dead"} or type(observation["dead"]) is not bool:
        raise SupervisorError("Native file owner observation is invalid")
    if observation["dead"] is not True:
        return "A live independent file owner still borrows this store"
    if not managed_file_owner_drained(row):
        return "File owner managed descendants still borrow this store"
    return None


def reap_payload_owners(
    state: Path, config: dict[str, object], *, require_all: bool
) -> dict[str, int]:
    counts = {"recovered": 0, "live": 0, "unknown": 0}
    for rows in payload_pages(state, config, "canonical_file_owners"):
        selected = [row for row in rows if row.get("drained") is not True]
        if not selected:
            continue
        try:
            observations = native_payload_operation(
                state, config, "owner-observations", selected
            )
        except (OSError, SupervisorError):
            if require_all:
                raise
            counts["unknown"] += len(selected)
            continue
        if len(observations) != len(selected):
            if require_all:
                raise SupervisorError("Native owner observation closure differs")
            counts["unknown"] += len(selected)
            continue
        for row, observation in zip(selected, observations, strict=True):
            try:
                live_refusal = payload_owner_live_refusal(row, config, observation)
            except (OSError, SupervisorError) as error:
                if require_all:
                    if isinstance(error, SupervisorError):
                        raise
                    raise SupervisorError(
                        "File owner drain is unobservable; admission remains closed"
                    ) from error
                counts["unknown"] += 1
                continue
            if live_refusal is not None:
                counts["live"] += 1
                if require_all:
                    raise SupervisorError(live_refusal)
                continue
            fields: list[str] = []
            for key, value in row.items():
                if key == "id":
                    continue
                if key in {"pid", "group_inode"}:
                    # JSON administrative transport does not retain native
                    # Decimal kind. Reconstruct only declared unsigned fields.
                    text_value = str(value)
                    if not text_value.isascii() or not text_value.isdecimal():
                        raise SupervisorError("File owner unsigned identity is invalid")
                    encoded = "<decimal>" + json.dumps(text_value)
                else:
                    encoded = json.dumps(value)
                fields.append(json.dumps(key) + ":" + encoded)
            expected_sql = "{" + ",".join(fields) + "}"
            acknowledged = payload_query(
                state,
                config,
                f"RETURN fn::pse_files_v1::drain_owner({payload_expiry()},{expected_sql});",
            )
            if acknowledged is not True:
                if require_all:
                    raise SupervisorError("Exact file owner drain was not acknowledged")
                counts["unknown"] += 1
                continue
            counts["recovered"] += 1
    return counts


def fixture_database_absent(state: Path, database: str, deadline: float) -> bool:
    """Observe exact namespace membership without selecting or creating the fixture DB."""
    rows = administrative_query(state, "INFO FOR NS;", deadline)
    if len(rows) != 1:
        raise SupervisorError("Fixture namespace inventory acknowledgment differs")
    catalog = object_mapping(rows[0]["result"])
    databases = catalog.get("databases")
    if not isinstance(databases, dict) or any(
        not isinstance(name, str) for name in databases
    ):
        raise SupervisorError("Fixture namespace database inventory is unavailable")
    return database not in databases


def empty_fixture_catalog(
    state: Path, config: Mapping[str, object], deadline: float
) -> str | None:
    """Distinguish absent or exactly blank creation from a published catalog."""
    database = str(config["database"])
    if fixture_database_absent(state, database, deadline):
        return "absent"
    rows = administrative_query(
        state,
        "INFO FOR DB;",
        deadline,
        namespace=str(config["namespace"]),
        database=database,
    )
    if len(rows) != 1:
        raise SupervisorError("Fixture database inventory acknowledgment differs")
    catalog = object_mapping(rows[0]["result"])
    if set(catalog) != MAINTENANCE_CATALOG_FIELDS["DB"] or any(
        not isinstance(value, dict) for value in catalog.values()
    ):
        raise SupervisorError("Fixture database catalog inventory is unavailable")
    return "blank" if all(not value for value in catalog.values()) else None


def require_fixture_recovery_idle(state: Path, config: Mapping[str, object]) -> None:
    """An orphan observation never takes over an original maintenance operation."""
    if (
        any(
            key in config
            for key in (
                "backup_pending",
                "derived_rebuild_pending",
                "surrealkit_rollout_pending",
            )
        )
        or (state / ".maintenance-listener.json").exists()
    ):
        raise SupervisorError(
            "Pending maintenance requires its original owner; orphan recovery refused"
        )


def recover_file_owners(
    state: Path,
    *,
    database: str | None = None,
    require_all: bool = False,
    fixture_recovery: bool = False,
    verifier: Path | None = None,
) -> dict[str, int]:
    """Explicit orphan recovery; ordinary service restart neither reaps nor invents owners."""
    if fixture_recovery or verifier is not None:
        if not fixture_recovery or database is None or not require_all:
            raise SupervisorError(
                "Explicit recovery helper requires exact fixture settlement"
            )
        require_fixture_recovery_idle(state, config_for(state))
    with lifecycle_reservation(state):
        config = config_for(state)
        if fixture_recovery:
            require_fixture_recovery_idle(state, config)
        if (
            service_directory(state) != state
            or not owns_listener(state, config)
            or not listener_ready(state, config)
        ):
            raise SupervisorError(
                "File-owner recovery requires the active owned catalog"
            )
        if database is not None:
            # The enclosing fixture owner supplies its exact registered database.
            # Codec-only fixtures share the service placement without a receiver.
            sql_identifier(database)
            selected = receiver_context(state, database)
            candidate = config_for(selected) if selected != state else {**config}
            if any(
                candidate.get(key) != config.get(key)
                for key in ("instance_id", "namespace", "payload_incarnation")
            ):
                raise SupervisorError(
                    "File-owner recovery context service identity differs"
                )
            config = candidate
            config["database"] = database
        if fixture_recovery:
            require_fixture_recovery_idle(state, config)
        # This descriptor belongs only to this new observation operation. Neither
        # the service's original selection nor any pending operation is readmitted.
        selected_verifier = payload_verifier(verifier) if verifier is not None else None
        if selected_verifier is not None:
            config = {**config, "payload_verifier": selected_verifier}
        elif fixture_recovery and not config.get("payload_verifier"):
            raise SupervisorError(
                "Orphan recovery requires its selected or explicit native helper"
            )
        if (
            fixture_recovery
            and config.get("interpretation") != SUBSTRATE_INTERPRETATION
        ):
            raise SupervisorError(
                "Orphan recovery requires the current selected interpretation"
            )
        deadline = getattr(_STARTUP, "deadline", None) or time.monotonic() + 90
        empty_catalog = (
            empty_fixture_catalog(state, config, deadline)
            if fixture_recovery and database is not None
            else None
        )
        if empty_catalog is not None:
            # A failed fixture creation need not have published a database. Its
            # atomic catalog may also have left only the blank database. Neither
            # observation settles bytes: the native owner must independently
            # reject every unknown file, lock and symlink in the exact placement.
            result = native_payload_operation(
                state, config, "retired-check", [], placement=config
            )
            if result != []:
                raise SupervisorError("Empty fixture placement acknowledgment differs")
            if empty_fixture_catalog(state, config, deadline) != empty_catalog:
                raise SupervisorError("Fixture catalog changed during orphan recovery")
            counts = {"recovered": 0, "live": 0, "unknown": 0}
        else:
            counts = recover_selected_file_owners(
                state, config, require_all=require_all
            )
        if verifier is not None and payload_verifier(verifier) != selected_verifier:
            raise SupervisorError("Explicit recovery helper changed during observation")
        return counts


def recover_selected_file_owners(
    state: Path, config: dict[str, object], *, require_all: bool
) -> dict[str, int]:
    """Require the complete current interpretation before native catalog recovery."""
    marker = payload_query(
        state,
        config,
        "LET $marker=SELECT * FROM ONLY canonical_interpretations:current; "
        "RETURN {payload_incarnation:$marker.payload_incarnation,interpretation:$marker.interpretation,study_module_sha256:$marker.study_module_sha256};",
    )
    if (
        marker
        != {
            "payload_incarnation": config["payload_incarnation"],
            "interpretation": SUBSTRATE_INTERPRETATION,
            "study_module_sha256": study_module(config)["sha256"],
        }
        or config["interpretation"] != SUBSTRATE_INTERPRETATION
    ):
        raise SupervisorError(
            "File-owner recovery requires the current selected incarnation and interpretation"
        )
    return reap_payload_owners(state, config, require_all=require_all)


def freeze_payloads(state: Path, config: dict[str, object]) -> list[dict[str, object]]:
    """Close admission/GC, authenticate every owner, then capture bounded catalog pages."""
    if service_directory(state) != state:
        raise SupervisorError("File maintenance requires the selected service owner")
    if not (state / "database").is_dir():
        raise SupervisorError("No initialized database to back up")
    close_context_admission(state)
    contexts = managed_contexts(state)
    for context in contexts:
        selected = config_for(context)
        selected["payload_frozen"] = True
        write_json(context / "config.json", selected)
    config.update(config_for(state))
    # A stopped listener cannot be started underneath undrained managed scopes.
    # This preflight does not capture a catalog or authorize any file effects.
    all_contexts_drained(state)
    # Start a closed validation listener when the operator previously stopped it.
    start(state, config, validation=True)
    for context in contexts:
        selected = config_for(context)
        acknowledged = payload_query(
            state,
            selected,
            f"RETURN fn::pse_files_v1::freeze({payload_expiry()},{json.dumps(selected['payload_incarnation'])});",
        )
        if acknowledged is not True:
            raise SupervisorError(
                "File admission and collection freeze was not acknowledged"
            )
    # File-owner death alone cannot establish managed descendant drain.
    all_contexts_drained(state)
    inventory: list[dict[str, object]] = []
    for context in contexts:
        selected = config_for(context)
        reap_payload_owners(state, selected, require_all=True)
        entries = [
            {
                "context": str(context.relative_to(state)),
                "database": selected["database"],
                "payload_incarnation": selected["payload_incarnation"],
                "instance": {key: value for key, value in row.items() if key != "id"},
                "files": {},
            }
            for rows in payload_pages(state, selected, "canonical_file_instances")
            for row in rows
        ]
        inventory.extend(
            native_payload_operation(
                state, selected, "inventory", entries, placement=selected
            )
        )
    return inventory


def copy_payload_inventory(
    state: Path, destination: Path, inventory: list[dict[str, object]]
) -> None:
    config = config_for(state)
    native_payload_operation(
        state, config, "backup", inventory, destination=destination, placement=config
    )


def validate_payload_inventory(
    source: Path, config: dict[str, object], value: object
) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise SupervisorError("Backup is missing its file-tier closure inventory")
    inventory = object_inventory(value)
    for entry in inventory:
        if (
            entry.get("context") != "."
            or entry.get("database") != config["database"]
            or entry.get("payload_incarnation") != config["payload_incarnation"]
        ):
            raise SupervisorError("Backup file-tier context differs from its profile")
    native_payload_operation(source, config, "verify", inventory, placement=config)
    return inventory


def application_backup_tables() -> tuple[str, ...]:
    """Read generated classification only when a backup operation selects it."""
    return tuple(_POLICY["application_backup_tables"])


def application_inventory_query(
    table: str, after: str, *, maintenance: bool = False
) -> str:
    if table not in application_backup_tables():
        raise SupervisorError("Unknown application backup inventory owner")
    # Key and digest bind every original row, including opaque authored bytes. The
    # reference projection is explanatory only and never a new owner of those bytes.
    fields = (
        "project",
        "draft",
        "branch",
        "version",
        "base_revision",
        "problem",
        "revision",
        "run",
        "attempt",
        "artifact",
        "dataset",
        "selection",
        "owner",
        "owner_kind",
        "generation",
        "state",
        "phase",
        "activated",
        "closed",
        "abandoned",
        "cleanup_complete",
        "terminal",
        "active",
        "complete",
        "retired",
        "retiring",
    )
    projection = "{" + ",".join(f"{field}:$row.{field}" for field in fields) + "}"
    # Maintenance owns the sole RETURN acknowledgement; a RETURN in an assigned
    # block would escape the transaction before that original nonce is emitted.
    output = "LET $value=" if maintenance else "RETURN "
    return (
        f"LET $rows=SELECT * FROM {table} WHERE key>{json.dumps(after)} ORDER BY key LIMIT 64; "  # noqa: S608 -- fixed owner allowlist, JSON cursor
        f"{output}$rows.map(|$row| {{ {{key:$row.key,"
        "sha256:crypto::sha256(<string>object::remove($row,['id'])),"
        f"references:<string>{projection}" + "} });"
    )


def capture_application_inventory(
    state: Path,
    config: dict[str, object],
    credentials: dict[str, object] | None = None,
) -> dict[str, object]:
    """Capture frozen original metadata with bounded pages, without acquiring bytes."""

    def query(body: str) -> object:
        if credentials is None:
            return payload_query(state, config, body)
        # Existing authenticated native maintenance transport; never opens an engine.
        return maintenance_query(
            state,
            config,
            credentials,
            body,
        ).get("value")

    marker_query = (
        "LET $value=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;"
        if credentials is not None
        else "RETURN (SELECT * FROM ONLY canonical_interpretations:current).issuing_store;"
    )
    marker = query(marker_query)
    if not isinstance(marker, str):
        raise SupervisorError("Application backup authority is unavailable")
    try:
        uuid.UUID(marker)
    except ValueError as error:
        raise SupervisorError("Application backup authority is invalid") from error
    tables: dict[str, object] = {}
    for table in application_backup_tables():
        rows: list[dict[str, object]] = []
        after = ""
        while True:
            page = query(
                application_inventory_query(
                    table, after, maintenance=credentials is not None
                )
            )
            if not isinstance(page, list) or len(page) > 64:
                raise SupervisorError("Application backup inventory page is invalid")
            if not page:
                break
            for raw in page:
                row = object_mapping(raw)
                key = row.get("key")
                if not isinstance(key, str) or not key or key <= after:
                    raise SupervisorError(
                        "Application inventory cursor did not advance"
                    )
                rows.append(row)
                after = key
        tables[table] = rows
    result: dict[str, object] = {
        "version": 1,
        "issuing_store": marker,
        "tables": tables,
    }
    validate_application_inventory(result)
    return result


def validate_application_inventory(value: object) -> dict[str, object]:
    selected = object_mapping(value)
    if (
        set(selected) != {"version", "issuing_store", "tables"}
        or selected.get("version") != 1
    ):
        raise SupervisorError("Backup is missing its application closure inventory")
    try:
        uuid.UUID(str(selected["issuing_store"]))
    except ValueError as error:
        raise SupervisorError("Backup application authority is invalid") from error
    tables = object_mapping(selected.get("tables"))
    if set(tables) != set(application_backup_tables()):
        raise SupervisorError("Backup application inventory omits required owners")
    for value in tables.values():
        if not isinstance(value, list):
            raise SupervisorError("Backup application inventory rows are invalid")
        after = ""
        for raw in value:
            row = object_mapping(raw)
            key = row.get("key")
            digest = row.get("sha256")
            if (
                set(row) != {"key", "sha256", "references"}
                or not isinstance(key, str)
                or not key
                or key <= after
                or not isinstance(digest, str)
                or re.fullmatch(r"[a-f0-9]{64}", digest) is None
                or not isinstance(row.get("references"), str)
            ):
                raise SupervisorError("Backup application row commitment is invalid")
            after = key
    return selected


def inspect_backup(
    source: Path,
    interpretation: str,
    verifier: Path | None = None,
) -> dict[str, object]:
    """Read-only inspection shares the exact restore verifier and performs no repair."""
    try:
        manifest, config, payloads = validated_restore_backup(
            source, interpretation, verifier
        )
        if verifier is not None:
            config = {**config, "payload_verifier": payload_verifier(verifier)}
        verify_payload_inventory(source, config, payloads, read_only_source=True)
    except (SupervisorError, OSError, ValueError) as error:
        return {
            "backup": str(source),
            "complete": False,
            "restorable": False,
            "refusal": str(error),
            "physical_reclaimed_bytes": None,
        }
    application = validate_application_inventory(manifest.get("application_inventory"))
    return {
        "backup": str(source),
        "backup_version": 4,
        "complete": True,
        "state_root": manifest["state_root"],
        "interpretation": config["interpretation"],
        "application_inventory": application,
        "payload_inventory": payloads,
        "copied_bytes": sum(
            (source / name).stat().st_size for name in object_mapping(manifest["files"])
        ),
        "physical_reclaimed_bytes": None,
        "unsettled_effects": "Original stage, attempt, file and history states are preserved in the inventories; inspection does not settle them.",
    }


def require_private_service_context(state: Path, action: str) -> None:
    """Backup and replacement cover only the service owner's sole database."""
    if service_directory(state) != state or managed_contexts(state) != [state]:
        raise SupervisorError(f"{action} refuses shared service contexts")


def backup(
    state: Path, config: dict[str, object], destination: Path
) -> dict[str, object]:
    with lifecycle_reservation(state):
        service_borrowers.service_registry_binding(state)
        return _backup(state, config, destination)


def backup_configuration_identity(config: Mapping[str, object]) -> str:
    return hashlib.sha256(
        json.dumps(
            {key: value for key, value in config.items() if key != "backup_pending"},
            sort_keys=True,
            separators=(",", ":"),
        ).encode()
    ).hexdigest()


def backup_sources(state: Path) -> tuple[Path, ...]:
    return (
        state / "database",
        state / "config.json",
        state / "credentials.json",
        *sorted(owned_generations(state)),
        *((state / ".contexts",) if (state / ".contexts").exists() else ()),
    )


def backup_pending(config: Mapping[str, object]) -> dict[str, object]:
    import msgspec  # noqa: PLC0415 -- private typed boundary

    pending = object_mapping(config["backup_pending"])
    if "version" not in pending:
        raise SupervisorError("Backup pending owner omitted its persisted version")
    try:
        return object_mapping(
            msgspec.to_builtins(
                msgspec.convert(pending, type=deployment.BackupOperation, strict=True)
            )
        )
    except msgspec.ValidationError as error:
        raise SupervisorError(
            "Backup pending owner is malformed; admission remains closed"
        ) from error


def backup_operation_directory(state: Path, pending: Mapping[str, object]) -> Path:
    nonce = pending.get("nonce")
    if not isinstance(nonce, str) or not re.fullmatch(r"[a-f0-9]{32}", nonce):
        raise SupervisorError("Backup pending operation identity is invalid")
    return checked_directory(state / ".backup-operations" / nonce)


def backup_destination(pending: Mapping[str, object], selected: Path) -> Path:
    destination = checked_directory(selected)
    information = destination.stat()
    if (
        str(destination) != pending.get("destination")
        or information.st_dev != pending.get("destination_device")
        or information.st_ino != pending.get("destination_inode")
    ):
        raise SupervisorError(
            "Backup destination differs from its original pending owner"
        )
    return destination


def _backup(
    state: Path, config: dict[str, object], destination: Path
) -> dict[str, object]:
    if config.get("derived_rebuild_pending") or config.get(
        "surrealkit_rollout_pending"
    ):
        raise SupervisorError(
            "Incomplete derived rebuild cannot be backed up as current state"
        )
    destination = lexical_absolute(destination)
    if (
        destination == state
        or state in destination.parents
        or destination in state.parents
    ):
        raise SupervisorError(
            "Backup destination must be outside the live state directory"
        )
    require_private_service_context(state, "Backup")
    try:
        if config.get("backup_pending") is None:
            # Reserve an actually empty private destination before source effects.
            destination = checked_directory(destination, empty=True)
            payload_inventory = freeze_payloads(state, config)
            # Native inventory already checked these held immutable bytes. The
            # post-stop native copy independently rechecks the original snapshot.
            application_inventory = capture_application_inventory(state, config)
            stop(state, config)
            config.update(config_for(state))
            private_offline_state(state, config, backup_resume=True)
            config["payload_verifier"] = admitted_payload_verifier(config)
            write_json(state / "config.json", config)
            metadata = backup_image.files(state, backup_sources(state))
            files = dict(metadata)
            for entry in payload_inventory:
                for name, binding in object_mapping(entry["files"]).items():
                    digest = object_mapping(binding).get("sha256")
                    if not isinstance(digest, str) or not re.fullmatch(
                        r"[a-f0-9]{64}", digest
                    ):
                        raise SupervisorError(
                            "Native payload inventory SHA256 binding is invalid"
                        )
                    files[name] = digest
            draft = {
                "owner": OWNER,
                "backup_version": 4,
                "state_root": str(state),
                "interpretation": config["interpretation"],
                "files": files,
                "payload_inventory": payload_inventory,
                "application_inventory": application_inventory,
            }
            nonce = uuid.uuid4().hex
            directory = checked_directory(
                state / ".backup-operations" / nonce, empty=True
            )
            backup_image.copy(
                state / "config.json",
                directory / "config.json",
                metadata["config.json"],
            )
            write_json(directory / "backup.json", draft)
            information = destination.stat()
            config["backup_pending"] = {
                "version": 1,
                "nonce": nonce,
                "destination": str(destination),
                "destination_device": information.st_dev,
                "destination_inode": information.st_ino,
                "source_instance_id": str(config["instance_id"]),
                "source_configuration_sha256": backup_configuration_identity(config),
                "manifest_sha256": backup_image.digest(directory / "backup.json"),
                "payload_checkpoint": None,
                "phase": "metadata",
            }
            write_json(state / "config.json", config)
        return resume_backup(state, config, destination)
    except backup_image.ImageError as error:
        raise SupervisorError(
            f"{error}; backup remains incomplete and source admission closed"
        ) from error


def resume_backup(
    state: Path, config: dict[str, object], destination: Path
) -> dict[str, object]:
    pending = backup_pending(config)
    private_offline_state(state, config, backup_resume=True)
    if (
        str(config.get("instance_id")) != pending["source_instance_id"]
        or backup_configuration_identity(config)
        != pending["source_configuration_sha256"]
        or config.get("payload_frozen") is not True
    ):
        raise SupervisorError(
            "Backup source configuration differs from its original frozen owner"
        )
    destination = backup_destination(pending, destination)
    directory = backup_operation_directory(state, pending)
    if backup_image.digest(directory / "backup.json") != pending["manifest_sha256"]:
        raise SupervisorError("Backup original manifest snapshot changed")
    draft = read_json(directory / "backup.json")
    files = string_mapping(draft["files"])
    metadata = {
        name: digest
        for name, digest in files.items()
        if Path(name).parts[0] != "payloads"
    }
    observed = backup_image.files(state, backup_sources(state))
    observed["config.json"] = backup_image.digest(directory / "config.json")
    if observed != metadata:
        raise SupervisorError(
            "Stopped backup source image differs from its original snapshot"
        )
    backup_image.check_destination(destination, files, str(pending["manifest_sha256"]))
    for name, digest in sorted(metadata.items()):
        backup_destination(pending, destination)
        source = directory / "config.json" if name == "config.json" else state / name
        backup_image.copy(source, destination / name, digest)
    pending["phase"] = "payload"
    config["backup_pending"] = pending
    write_json(state / "config.json", config)
    # Re-reading the original retained inventory preserves exact native request
    # bytes, including after a process crash or a completed-prefix copy.
    copy_payload_inventory(
        state, destination, object_inventory(draft["payload_inventory"])
    )
    config.update(config_for(state))
    pending = backup_pending(config)
    backup_destination(pending, destination)
    backup_image.check_destination(destination, files, str(pending["manifest_sha256"]))
    backup_image.copy(
        directory / "backup.json",
        destination / "backup.json",
        str(pending["manifest_sha256"]),
    )
    pending["phase"] = "published"
    config["backup_pending"] = pending
    write_json(state / "config.json", config)
    # A completed backup does not thaw files or start either service role.
    config.pop("backup_pending")
    write_json(state / "config.json", config)
    for name in ("config.json", "backup.json"):
        (directory / name).unlink()
    directory.rmdir()
    return {
        "backup": str(destination),
        "interpretation": config["interpretation"],
        "server_stopped": True,
    }


def private_offline_state(
    state: Path, config: dict[str, object], *, backup_resume: bool = False
) -> None:
    """Only the stopped service's sole database may be replaced by its owner."""
    if config.get("backup_pending") is not None and not backup_resume:
        raise SupervisorError(
            "Explicit original backup resume is required before another offline mutation"
        )
    require_private_service_context(state, "Derived cutover")
    if config["accepting_writes"] or config["admission"] not in {
        "quiesced",
        "validation_required",
    }:
        raise SupervisorError("Derived cutover requires closed admission")
    all_contexts_drained(state)
    if (state / ".maintenance-listener.json").exists():
        # Recovery first drains a predecessor invocation; no ordinary listener
        # or live competing lifecycle owner may be adopted.
        drain_maintenance_listener(state)
        config.update(config_for(state))
    if active(state):
        raise SupervisorError("Derived cutover requires the owned server stopped")
    # is-active alone does not establish process-group drain.
    stop(state, config)


def database_endpoint(state: Path) -> str:
    allocation = object_mapping(config_for(state)["resources"])
    # Synchronous append and fsync are explicit. MX's key/value allowance is
    # smaller than the process envelope to reserve snapshots, RPC and overhead.
    return (
        f"mem://{state / 'database'}?aol=sync&sync=every&versioned=false"
        f"&snapshot={allocation['surrealmx_snapshot_interval_seconds']}s"
        f"&memory_limit={allocation['surrealmx_memory_limit_bytes']}"
        f"&max_transaction_age={allocation['surrealmx_max_transaction_age_seconds']}s"
        f"&max_async_queue_depth={allocation['surrealmx_max_async_queue_depth']}"
        f"&max_async_queue_bytes={allocation['surrealmx_max_async_queue_bytes']}"
    )


def sql_identifier(value: object) -> str:
    if (
        not isinstance(value, str)
        or re.fullmatch(r"[A-Za-z0-9_-]{1,128}", value) is None
    ):
        raise SupervisorError("Unsupported owned database identifier")
    return "`" + value + "`"


def require_maintenance_owner(state: Path, config: dict[str, object]) -> None:
    selected = read_json(state / ".maintenance-listener.json")
    owner = object_mapping(selected.get("owner"))
    if (
        config.get("accepting_writes") is not False
        or config.get("admission") not in {"quiesced", "validation_required"}
        or selected.get("instance_id") != config.get("instance_id")
        or selected.get("server") != config.get("server")
        or selected.get("resources") != config.get("resources")
        or selected.get("supervisor") != config.get("service_supervisor")
        or selected.get("credentials_sha256") != file_digest(state / "credentials.json")
        or not reservation_live(owner)
        or read_json(state / "lifecycle-owner.json") != owner
        or service_directory(state) != state
        or managed_contexts(state) != [state]
    ):
        raise SupervisorError(
            "Maintenance listener requires its exact live closed owner"
        )


def watch_maintenance_owner(state: Path, child: subprocess.Popen[bytes]) -> None:
    """Owner loss closes the child; recovery still owns exact drain/release."""
    while child.poll() is None:
        try:
            require_maintenance_owner(state, config_for(state))
        except (SupervisorError, OSError, KeyError, ValueError):
            if child.poll() is None:
                child.terminate()
            return
        time.sleep(0.1)


def drain_maintenance_listener(state: Path) -> None:
    path = state / ".maintenance-listener.json"
    if not path.exists():
        return
    config = config_for(state)
    selected = read_json(path)
    owner = object_mapping(selected.get("owner"))
    if (
        selected.get("instance_id") != config.get("instance_id")
        or selected.get("server") != config.get("server")
        or selected.get("resources") != config.get("resources")
        or (
            reservation_live(owner)
            and read_json(state / "lifecycle-owner.json") != owner
        )
    ):
        raise SupervisorError(
            "Unknown maintenance listener retained; admission remains closed"
        )
    observation = systemctl(
        "show",
        "--property=LoadState",
        "--property=ActiveState",
        "--property=ControlGroup",
        "--property=InvocationID",
        unit_name(state),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in observation.stdout.splitlines() if "=" in line
    )
    if observation.returncode or any(
        key not in fields for key in ("LoadState", "ActiveState", "ControlGroup")
    ):
        raise SupervisorError(
            "Cannot observe maintenance lifetime; admission remains closed"
        )
    group = fields["ControlGroup"]
    # is-active excludes activating/deactivating. Every extant group or busy
    # invocation needs exact authority before stop can target its unit name.
    if fields["ActiveState"] not in {"inactive", "failed"} or group:
        launch = storage_launch(state, config)
        binding = object_mapping(launch.get("binding"))
        if (
            fields.get("InvocationID") != binding.get("invocation")
            or group != binding.get("group")
            or host_admission.group_identity(group) != binding.get("inode")
        ):
            raise SupervisorError(
                "Maintenance invocation changed; admission remains closed"
            )
    # _stop proves the exact service cgroup/process drain before releasing its
    # allocation. Failure preserves the durable listener and pending phase.
    stop(state, config)
    path.unlink()
    descriptor = os.open(state, os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def fresh_root_authority(
    state: Path, config: dict[str, object]
) -> dict[str, object] | None:
    """Fresh creation alone can mint bootstrap authority; recovery cannot infer it."""
    authority = config.get("fresh_root_bootstrap")
    if authority is None:
        return None
    authority = object_mapping(authority)
    nonce = authority.get("nonce")
    if (
        config.get("accepting_writes") is not False
        or config.get("admission") != "validation_required"
        or authority.get("instance_id") != config.get("instance_id")
        or authority.get("credentials_sha256")
        != file_digest(state / "credentials.json")
        or not isinstance(nonce, str)
        or re.fullmatch(r"[a-f0-9]{32}", nonce) is None
    ):
        raise SupervisorError(
            "Fresh ROOT bootstrap authority does not match this closed target"
        )
    initialization = object_mapping(config.get("derived_rebuild_pending", {}))
    rollout = object_mapping(config.get("surrealkit_rollout_pending", {}))
    if initialization and rollout:
        raise SupervisorError("Competing fresh bootstrap operations")
    owner = read_json(state / "lifecycle-owner.json")
    permitted = (
        initialization.get("kind") == "initialize"
        and initialization.get("phase") == "initializing"
        and initialization.get("initializer") == owner
        and (
            initialization.get("target_namespace"),
            initialization.get("target_database"),
        )
        == (config.get("namespace"), config.get("database"))
    ) or (
        rollout.get("phase") == "prepared"
        and rollout.get("sql_outcome") == "not_submitted"
        and rollout.get("instance_id") == config.get("instance_id")
        and rollout.get("owner") == owner
    )
    if not permitted:
        raise SupervisorError(
            "Fresh ROOT bootstrap requires its explicit initialization or rollout owner"
        )
    return authority


def maintenance_endpoint(state: Path) -> str:
    if getattr(_LIFECYCLE, "state", None) != state:
        raise SupervisorError("Maintenance requires a reserved lifecycle owner")
    config = config_for(state)
    if (state / ".maintenance-listener.json").exists():
        require_maintenance_owner(state, config)
        if not listener_ready(state, config):
            raise SupervisorError(
                "Maintenance listener became unavailable; admission remains closed"
            )
    else:
        private_offline_state(state, config)
        bootstrap = fresh_root_authority(state, config)
        if not config.get("unit_materialized"):
            # Prove a retained declaration against its current generation before
            # publishing the next one. A lost ACK cannot authorize replacement.
            materialize_service(state, config)
        config["service_supervisor"] = publish_generation(state)
        write_json(state / "config.json", config)
        write_json(
            state / ".maintenance-listener.json",
            {
                "owner": read_json(state / "lifecycle-owner.json"),
                "instance_id": config["instance_id"],
                "server": config["server"],
                "resources": config["resources"],
                "supervisor": config["service_supervisor"],
                "credentials_sha256": file_digest(state / "credentials.json"),
                "root_bootstrap": bootstrap,
            },
        )
        start(state, config, validation=True)
        if bootstrap is not None:
            # Authenticate ROOT and install the bounded selection account, with
            # no schema/context creation or ordinary readiness publication.
            establish_protocol_readiness(
                state,
                config,
                getattr(_STARTUP, "deadline", None) or time.monotonic() + 40,
                record_readiness=False,
            )
            current = config_for(state)
            require_maintenance_owner(state, current)
            if current.get("fresh_root_bootstrap") != bootstrap:
                raise SupervisorError("Fresh ROOT acknowledgment changed authority")
            current.pop("fresh_root_bootstrap")
            write_json(state / "config.json", current)
    return "http://127.0.0.1:" + str(config["port"])


def maintenance_sql(
    state: Path, config: dict[str, object], credentials: dict[str, object], body: str
) -> tuple[str, subprocess.CompletedProcess[str]]:
    """Fresh authenticated CLI session on the reserved supervised maintenance child."""
    endpoint = maintenance_endpoint(state)
    selection = []
    for key in ("namespace", "database"):
        if config.get(key):
            selection.extend(["--" + key, str(config[key])])
    nonce = uuid.uuid4().hex
    query = (
        "BEGIN; "
        + body
        + f" RETURN {{maintenance: '{nonce}', value: $value}}; COMMIT;\n"
    )
    result = subprocess.run(
        [
            str(object_mapping(config["server"])["binary"]),
            "sql",
            "--log=none",
            "--endpoint",
            endpoint,
            *selection,
            "--json",
            "--hide-welcome",
        ],
        input=query,
        env=server_environment(config, credentials),
        cwd=state,
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    return nonce, result


def maintenance_results(config: dict[str, object], output: str) -> list[object]:
    """Decode the pinned native SQL prompt framing without accepting extra results."""
    prompt = (
        "/".join(
            str(config[key]) for key in ("namespace", "database") if config.get(key)
        )
        + ">"
    )
    lines = [line.strip().removeprefix(prompt).strip() for line in output.splitlines()]
    returned = json.loads("\n".join(line for line in lines if line))
    if not isinstance(returned, list):
        raise TypeError("Native SQL output is not a result list")
    return [item for item in returned if item is not None]


def maintenance_failure(
    state: Path,
    credentials: dict[str, object],
    result: subprocess.CompletedProcess[str],
) -> None:
    # Keep bounded private failure evidence without exposing query text,
    # authentication arguments, or candidate account passwords.
    diagnostics = {"stdout": result.stdout, "stderr": result.stderr}
    authorities = [credentials]
    candidate = state / ".maintenance-credentials.json"
    if candidate.is_file():
        authorities.append(read_json(candidate))
    for name, raw in diagnostics.items():
        value = raw
        for authority in authorities:
            for credential_key in ("password", "selection_password"):
                secret = authority.get(credential_key)
                if isinstance(secret, str) and secret:
                    value = value.replace(secret, "[REDACTED]")
        diagnostics[name] = value[-MESSAGE_BYTES:]
    slot = state / "maintenance-cli-failure.json"
    if slot.is_symlink():
        raise SupervisorError(
            "Unknown maintenance diagnostic retained; admission remains closed"
        )
    if slot.exists():
        identity = slot.stat()
        # Each retained Unicode character needs at most two six-byte JSON
        # surrogate escapes. The envelope includes the full signed exit-code range.
        envelope = {
            "owner": OWNER,
            "kind": "maintenance-cli-failure-v1",
            "returncode": -2147483648,
            "stdout": "",
            "stderr": "",
        }
        maximum_size = (
            24 * MESSAGE_BYTES
            + len(json.dumps(envelope, indent=2, sort_keys=True).encode())
            + 1
        )
        if (
            not stat.S_ISREG(identity.st_mode)
            or identity.st_uid != os.getuid()
            or stat.S_IMODE(identity.st_mode) != 0o600
            or identity.st_nlink != 1
            or identity.st_size > maximum_size
        ):
            raise SupervisorError(
                "Unknown maintenance diagnostic retained; admission remains closed"
            )
        previous = read_json(slot)
        if (
            previous.get("owner") != OWNER
            or previous.get("kind") != "maintenance-cli-failure-v1"
        ):
            raise SupervisorError(
                "Unknown maintenance diagnostic retained; admission remains closed"
            )
    write_json(
        slot,
        {
            "owner": OWNER,
            "kind": "maintenance-cli-failure-v1",
            "returncode": result.returncode,
            **diagnostics,
        },
    )
    raise SupervisorError("Maintenance acknowledgment failed; admission remains closed")


def maintenance_query(
    state: Path, config: dict[str, object], credentials: dict[str, object], body: str
) -> dict[str, object]:
    """Require exactly one transaction acknowledgment from an output-neutral query."""
    nonce, result = maintenance_sql(state, config, credentials, body)
    nonempty: list[object] = []
    try:
        nonempty = maintenance_results(config, result.stdout)
        valid = (
            len(nonempty) == 1
            and isinstance(nonempty[0], dict)
            and nonempty[0].get("maintenance") == nonce
        )
    except (ValueError, TypeError):
        valid = False
    if result.returncode or not valid:
        maintenance_failure(state, credentials, result)
    return object_mapping(nonempty[0])


MAINTENANCE_CATALOG_FIELDS = {
    "ROOT": frozenset(
        {"accesses", "defaults", "namespaces", "nodes", "system", "users", "config"}
    ),
    "NS": frozenset({"accesses", "databases", "users"}),
    "DB": frozenset(
        {
            "accesses",
            "apis",
            "analyzers",
            "buckets",
            "functions",
            "modules",
            "models",
            "params",
            "tables",
            "users",
            "configs",
            "sequences",
        }
    ),
}


def maintenance_catalog(
    state: Path, config: dict[str, object], credentials: dict[str, object], scope: str
) -> dict[str, object]:
    """Read one pinned catalog and its exact transaction acknowledgment.

    INFO must be a statement: assigning unselected ROOT INFO requires a namespace
    in the pinned release. Only select NS/DB after the caller observes it exists.
    """
    if scope not in MAINTENANCE_CATALOG_FIELDS:
        raise SupervisorError("Unknown maintenance catalog scope")
    selected = dict(config)
    if scope == "ROOT":
        selected.update(namespace=None, database=None)
    elif scope == "NS":
        selected["database"] = None
        if not selected.get("namespace"):
            raise SupervisorError("Namespace catalog requires an observed namespace")
    elif not selected.get("namespace") or not selected.get("database"):
        raise SupervisorError("Database catalog requires an observed database")
    nonce, result = maintenance_sql(
        state, selected, credentials, f"INFO FOR {scope}; LET $value=true;"
    )
    nonempty: list[object] = []
    try:
        nonempty = maintenance_results(selected, result.stdout)
        valid = (
            len(nonempty) == 2
            and isinstance(nonempty[0], dict)
            and set(nonempty[0]) == MAINTENANCE_CATALOG_FIELDS[scope]
            and all(isinstance(value, dict) for value in nonempty[0].values())
            and isinstance(nonempty[1], dict)
            and set(nonempty[1]) == {"maintenance", "value"}
            and nonempty[1].get("maintenance") == nonce
            and nonempty[1].get("value") is True
        )
    except (ValueError, TypeError):
        valid = False
    if result.returncode or not valid:
        maintenance_failure(state, credentials, result)
    return object_mapping(nonempty[0])


def maintenance_domain_call(
    state: Path,
    config: dict[str, object],
    credentials: dict[str, object] | None,
    function: str,
    arguments: list[object],
) -> object:
    """Call a fixed generated domain owner; never accept SQL from an operator input."""
    if function not in {
        "current",
        "claim",
        "page",
        "rehome_complete",
        "validation_reads",
    }:
        raise SupervisorError("Unknown generated maintenance operation")
    # Generated option<object> arguments use Surreal NONE for absence; JSON NULL
    # is a distinct value and cannot satisfy the original first-claim guard.
    encoded = ",".join(
        "NONE" if value is None else json.dumps(value, allow_nan=False)
        for value in arguments
    )
    call = (
        f"fn::pse_maintenance_v1::{function}({payload_expiry()}"
        + ("," + encoded if encoded else "")
        + ")"
    )
    # `current` legitimately returns NONE before the first claim. Preserve it
    # as an explicit value, distinct from a missing transaction acknowledgment.
    evaluation = "LET $domain=" + call + ";"
    value = "IF $domain=NONE {NULL} ELSE {$domain}"
    if credentials is None:
        acknowledgment = payload_query(
            state, config, evaluation + " RETURN {value:" + value + "};"
        )
        if not isinstance(acknowledgment, dict) or set(acknowledgment) != {"value"}:
            raise SupervisorError("Malformed generated maintenance acknowledgment")
    else:
        acknowledgment = maintenance_query(
            state, config, credentials, evaluation + " LET $value=" + value + ";"
        )
        if "value" not in acknowledgment:
            raise SupervisorError("Missing generated maintenance acknowledgment")
    return acknowledgment["value"]


def maintenance_domain_context(
    state: Path,
    config: dict[str, object],
    credentials: dict[str, object] | None,
    purpose: str,
    source: str,
    target: str,
) -> tuple[str, str, str]:
    """Fence one explicit phase under its actual live supervisor generation."""
    owner = read_json(state / "lifecycle-owner.json")
    generation = owner.get("nonce")
    if not reservation_live(owner) or not isinstance(generation, str) or not generation:
        raise SupervisorError("Domain maintenance requires its actual live generation")
    pending = config.get("derived_rebuild_pending")
    persisted = config_for(state)
    if isinstance(pending, dict):
        phase = pending.get("phase")
        nonce = pending.get("domain_nonce")
        actual = object_mapping(persisted.get("derived_rebuild_pending"))
        identity = (
            "kind",
            "phase",
            "namespace",
            "database",
            "target_namespace",
            "target_database",
            "payload_incarnation",
            "backup_digest",
        )
        if (
            any(actual.get(key) != pending.get(key) for key in identity)
            or actual.get("domain_nonce") != nonce
        ):
            raise SupervisorError(
                "Domain maintenance phase or original operation changed"
            )
        if nonce is None:
            nonce = uuid.uuid4().hex
            actual["domain_nonce"] = nonce
            persisted["derived_rebuild_pending"] = actual
            write_json(state / "config.json", persisted)
            pending["domain_nonce"] = nonce
    else:
        if persisted.get("derived_rebuild_pending"):
            raise SupervisorError("Domain maintenance phase changed")
        phase, nonce = "validation", uuid.uuid4().hex
    if (
        not isinstance(phase, str)
        or not isinstance(nonce, str)
        or re.fullmatch(r"[a-f0-9]{32}", nonce) is None
    ):
        raise SupervisorError("Invalid original domain maintenance operation")
    operation = ":".join(
        (
            nonce,
            purpose,
            source,
            target,
            str(config["namespace"]),
            str(config["database"]),
        )
    )
    previous = maintenance_domain_call(state, config, credentials, "current", [])
    if previous is not None and (
        not isinstance(previous, dict)
        or set(previous) != {"operation", "generation", "phase"}
        or any(not isinstance(value, str) or not value for value in previous.values())
    ):
        raise SupervisorError("Malformed current maintenance generation")
    acknowledged = maintenance_domain_call(
        state,
        config,
        credentials,
        "claim",
        [operation, generation, phase, source, target, previous],
    )
    if acknowledged != {
        "operation": operation,
        "generation": generation,
        "phase": phase,
    }:
        raise SupervisorError("Maintenance generation was not acknowledged")
    return operation, generation, phase


def maintenance_domain_page(value: object, after: str) -> tuple[int, str, bool]:
    """Closed typed progress; errors/unknown responses never become a successful page."""
    if not isinstance(value, dict) or set(value) != {"count", "after", "complete"}:
        raise SupervisorError("Malformed generated maintenance page")
    count, cursor, complete = value["count"], value["after"], value["complete"]
    if (
        type(count) is not int
        or not 0 <= count <= 64
        or not isinstance(cursor, str)
        or type(complete) is not bool
        or complete != (count == 0)
        or (count and cursor <= after)
        or (not count and cursor != after)
    ):
        raise SupervisorError("Invalid generated maintenance progress")
    return count, cursor, complete


def rotate_source_authorities(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Fence old creation grants while preserving admitted science and its borrowers."""
    pending = object_mapping(config.get("derived_rebuild_pending"))
    incarnation = str(pending["payload_incarnation"])
    context = maintenance_domain_context(
        state, config, credentials, "rotate", incarnation, incarnation
    )
    after = ""
    while True:
        value = maintenance_domain_call(
            state, config, credentials, "page", [*context, "rotate", after]
        )
        _count, after, complete = maintenance_domain_page(value, after)
        if complete:
            return


def fresh_database_identity(config: dict[str, object]) -> dict[str, object]:
    selected = dict(config)
    # Fresh ROOT authority is minted only for an empty engine, never inherited
    # by database reconstruction or restore of an existing account catalog.
    selected.pop("fresh_root_bootstrap", None)
    identity = uuid.uuid4().hex
    selected.update(
        namespace="pse_" + identity,
        database="canonical_" + identity,
        instance_id=str(uuid.uuid4()),
        payload_incarnation=str(uuid.uuid4()),
        admission="validation_required",
        accepting_writes=False,
        unit_materialized=False,
        restart_qualified=False,
    )
    return selected


def fresh_owned_database_identity(
    state: Path, config: dict[str, object]
) -> dict[str, object]:
    """Rebind a proved-offline owned service to a new database generation."""
    if (
        getattr(_LIFECYCLE, "state", None) != state
        or active(state)
        or (state / ".maintenance-listener.json").exists()
        or config.get("accepting_writes") is not False
        or config_for(state)["instance_id"] != config["instance_id"]
    ):
        raise SupervisorError(
            "Fresh database identity requires its drained lifecycle owner"
        )
    launch = state / "service-launch.json"
    if launch.exists():
        # The old binding cannot be carried into another service generation.
        # Preserve unknown receipts rather than hiding an ownership mismatch.
        storage_launch(state, config)
        launch.unlink()
        descriptor = os.open(state, os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    selected = fresh_database_identity(config)
    # The state-path unit is still this owner's materialization. Its immutable
    # executable and generation are replaced normally on the next start.
    selected["unit_materialized"] = config.get("unit_materialized", False)
    return selected


def fresh_credentials() -> dict[str, object]:
    return {
        "username": "pse-local-" + uuid.uuid4().hex,
        "password": secrets.token_urlsafe(36),
        "selection_username": "pse-selection",
        "selection_password": secrets.token_urlsafe(36),
    }


def require_private_catalog(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Global credential changes require actual sole ownership of stored content."""
    pending = object_mapping(config.get("derived_rebuild_pending", {}))
    allowed = {(config["namespace"], config["database"])}
    if pending:
        allowed.add((pending["namespace"], pending["database"]))
    root = maintenance_catalog(state, config, credentials, "ROOT")
    for namespace in object_mapping(root["namespaces"]):
        selected = {**config, "namespace": namespace}
        namespace_catalog = maintenance_catalog(state, selected, credentials, "NS")
        for database in object_mapping(namespace_catalog["databases"]):
            if (namespace, database) in allowed:
                continue
            content = maintenance_catalog(
                state, {**selected, "database": database}, credentials, "DB"
            )
            # Keep even empty databases/namespaces; refuse any unknown content.
            if any(content.values()):
                raise SupervisorError(
                    "Derived cutover refuses an unknown nonempty database"
                )


def rotate_database_accounts(
    state: Path,
    config: dict[str, object],
    old: dict[str, object],
    new: dict[str, object],
) -> None:
    pending = state / ".maintenance-credentials.json"
    write_json(pending, new)
    require_private_catalog(state, config, old)
    response = maintenance_query(
        state,
        config,
        old,
        f"DEFINE USER {sql_identifier(new['username'])} ON ROOT PASSWORD {json.dumps(new['password'])} ROLES OWNER; "
        f"DEFINE USER OVERWRITE `pse-selection` ON ROOT PASSWORD {json.dumps(new['selection_password'])} ROLES VIEWER; "
        f"REMOVE USER {sql_identifier(old['username'])} ON ROOT; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError(
            "Account rotation was not acknowledged; admission remains closed"
        )
    drain_maintenance_listener(state)
    config.update(config_for(state))
    write_json(state / "credentials.json", new)
    pending.unlink()


def discard_database(
    state: Path,
    selected: dict[str, object],
    old: dict[str, object],
    credentials: dict[str, object],
) -> None:
    # REMOVE DATABASE requires only its existing namespace (Base::Ns). Never
    # select the disposed database: native CLI session followup can resolve its
    # selected context after the removal. Preserve its namespace and service.
    response = maintenance_query(
        state,
        {**selected, "namespace": old["namespace"], "database": None},
        credentials,
        f"REMOVE DATABASE {sql_identifier(old['database'])}; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError(
            "Old database disposal was not acknowledged; admission remains closed"
        )

    if database_present(state, old, credentials):
        raise SupervisorError(
            "Old database remains addressable; admission remains closed"
        )


def database_present(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> bool:
    # Select no unknown namespace/database during an observational catalog read.
    root = maintenance_catalog(state, config, credentials, "ROOT")
    if config["namespace"] not in object_mapping(root["namespaces"]):
        return False
    namespace = maintenance_catalog(state, config, credentials, "NS")
    return config["database"] in object_mapping(namespace["databases"])


def maintenance_phase(state: Path, config: dict[str, object], phase: str) -> None:
    pending = object_mapping(config["derived_rebuild_pending"])
    config["service_supervisor"] = config_for(state)["service_supervisor"]
    pending["phase"] = phase
    config["derived_rebuild_pending"] = pending
    config["accepting_writes"] = False
    write_json(state / "config.json", config)


def current_marker(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    response = maintenance_query(
        state,
        config,
        credentials,
        "LET $marker=SELECT * FROM ONLY canonical_interpretations:current; "  # noqa: S608 -- Fixed policy interpretation is JSON quoted.
        f"IF $marker=NONE OR $marker.interpretation!={json.dumps(SUBSTRATE_INTERPRETATION)} OR $marker.study_module_sha256!={json.dumps(study_module(config)['sha256'])} {{ THROW 'current interpretation or study module unavailable'; }}; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError("Current interpretation was not acknowledged")


def maintenance_payloads(state: Path, pending: dict[str, object]) -> dict[str, object]:
    """Load exact frozen retirement evidence separately from bounded startup metadata."""
    names = ("payload_inventory", "payload_targets", "retired_payloads")
    if any(name in pending for name in names):
        raise SupervisorError("Inline maintenance payload evidence is unsupported")
    reference = pending.get("payload_retirement")
    if reference is None:
        if pending.get("payload_retirement_nonce") is not None or (
            pending.get("kind") == "rebuild"
            and pending.get("payload_incarnation") is not None
        ):
            raise SupervisorError("Maintenance payload evidence is missing")
        return {name: [] for name in names}
    reference = object_mapping(reference)
    nonce = pending.get("payload_retirement_nonce")
    name = reference.get("path")
    if (
        not isinstance(nonce, str)
        or re.fullmatch(r"[a-f0-9]{32}", nonce) is None
        or set(reference) != {"path", "bytes", "sha256"}
        or type(reference.get("bytes")) is not int
        or not isinstance(name, str)
        or re.fullmatch(
            r"\.maintenance-payloads-" + nonce + r"-[a-f0-9]{32}\.json", name
        )
        is None
    ):
        raise SupervisorError("Maintenance payload evidence identity differs")
    raw = private_record_bytes(state / name, getattr(_STARTUP, "deadline", None))
    if (
        reference.get("bytes") != len(raw)
        or reference.get("sha256") != hashlib.sha256(raw).hexdigest()
    ):
        raise SupervisorError("Maintenance payload evidence changed")
    record = object_mapping(json.loads(raw))
    source = {
        key: pending.get(key)
        for key in ("namespace", "database", "payload_incarnation")
    }
    if (
        record.get("schema") != "maintenance-payloads-v1"
        or record.get("operation") != nonce
        or record.get("source") != source
        or set(record) != {"schema", "operation", "source", *names}
    ):
        raise SupervisorError("Maintenance payload evidence source differs")
    return {name: record[name] for name in names}


def retire_maintenance_payloads(state: Path, pending: dict[str, object]) -> None:
    """Remove only the exact record whose config reference has durably departed."""
    if pending.get("payload_retirement") is None:
        return
    if getattr(_LIFECYCLE, "state", None) != state:
        raise SupervisorError(
            "Maintenance payload retirement requires its lifecycle owner"
        )
    reference = object_mapping(pending["payload_retirement"])
    current = config_for(state).get("derived_rebuild_pending")
    if current is not None:
        current_reference = object_mapping(current).get("payload_retirement")
        if (
            current_reference is not None
            and object_mapping(current_reference).get("path") == reference["path"]
        ):
            raise SupervisorError("Maintenance payload evidence remains referenced")
    maintenance_payloads(state, pending)
    (state / str(reference["path"])).unlink()
    descriptor = os.open(state, os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_maintenance_payloads(
    state: Path, config: dict[str, object], payloads: dict[str, object]
) -> None:
    """Publish immutable evidence before its bounded pointer; preserve crash orphans."""
    if getattr(_LIFECYCLE, "state", None) != state:
        raise SupervisorError(
            "Maintenance payload publication requires its lifecycle owner"
        )
    if set(payloads) != {"payload_inventory", "payload_targets", "retired_payloads"}:
        raise SupervisorError("Unknown maintenance payload evidence fields")
    pending = object_mapping(config["derived_rebuild_pending"])
    previous = dict(pending)
    if previous.get("payload_retirement") is not None:
        maintenance_payloads(state, previous)
    nonce = pending.setdefault("payload_retirement_nonce", uuid.uuid4().hex)
    if not isinstance(nonce, str) or re.fullmatch(r"[a-f0-9]{32}", nonce) is None:
        raise SupervisorError("Invalid maintenance payload operation")
    name = ".maintenance-payloads-" + nonce + "-" + uuid.uuid4().hex + ".json"
    path = state / name
    if path.exists() or path.is_symlink():
        raise SupervisorError("Maintenance payload record already exists")
    record = {
        "schema": "maintenance-payloads-v1",
        "operation": nonce,
        "source": {
            key: pending.get(key)
            for key in ("namespace", "database", "payload_incarnation")
        },
        **payloads,
    }
    write_json(path, record)
    raw = private_record_bytes(path, getattr(_STARTUP, "deadline", None))
    pending["payload_retirement"] = {
        "path": name,
        "sha256": hashlib.sha256(raw).hexdigest(),
        "bytes": len(raw),
    }
    config["derived_rebuild_pending"] = pending
    write_json(state / "config.json", config)
    retire_maintenance_payloads(state, previous)


def retire_partial_payload_target(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Failed initializer catalogs cannot be discarded underneath surviving file owners."""
    tables = object_mapping(
        maintenance_catalog(state, config, credentials, "DB")["tables"]
    )
    tier = {
        "canonical_file_admission",
        "canonical_file_owners",
        "canonical_file_instances",
    }
    present = tier.intersection(tables)
    if present and present != tier:
        raise SupervisorError(
            "Partial file catalog has unknown ownership; recovery remains closed"
        )
    directory = payload_root(state, config)
    if not present:
        if directory.exists() and any(directory.rglob("*")):
            raise SupervisorError(
                "Payload bytes without an owner catalog cannot be discarded"
            )
        return
    inventory = freeze_payloads(state, config)
    owners = [
        {key: value for key, value in row.items() if key != "id"}
        for page in payload_pages(state, config, "canonical_file_owners")
        for row in page
    ]
    if any(owner.get("drained") is not True for owner in owners):
        raise SupervisorError("Partial target file owners remain undrained")
    pending = object_mapping(config["derived_rebuild_pending"])
    payloads = maintenance_payloads(state, pending)
    retired = payloads["retired_payloads"]
    if not isinstance(retired, list):
        raise SupervisorError("Invalid retired file-tier recovery inventory")
    retirement = {
        "payload_incarnation": config["payload_incarnation"],
        "database": config["database"],
        "inventory": inventory,
        "owners": owners,
    }
    if retirement not in retired:
        retired.append(retirement)
    write_maintenance_payloads(state, config, payloads)


def retired_payload_cleanup_files(
    state: Path,
    selected_root: Path,
    placement: dict[str, object],
    inventory: list[dict[str, object]],
) -> Path | None:
    """Native physical validation precedes the supervisor's disposal decision."""
    config = config_for(state)
    directory = payload_root(state, {**config, **placement})
    if directory == selected_root:
        return None
    native_payload_operation(
        state, config, "retired-check", inventory, placement=placement
    )
    return directory


def cleanup_retired_payloads(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Retire only inventoried placement after exact old-owner drain; unknown bytes survive."""
    pending = object_mapping(config["derived_rebuild_pending"])
    payloads = maintenance_payloads(state, pending)
    old_incarnation = pending.get("payload_incarnation")
    if old_incarnation is None:
        # Maintenance begun before a file tier existed carries no deletion authority.
        return
    source = {
        **config,
        "namespace": pending["namespace"],
        "database": pending["database"],
    }
    if database_present(state, source, credentials):
        response = maintenance_query(
            state,
            source,
            credentials,
            "LET $owners=SELECT key FROM canonical_file_owners WHERE !drained LIMIT 1; "
            "IF array::len($owners)!=0 { THROW 'old file owners refuse root cleanup'; }; LET $value=true;",
        )
        if response.get("value") is not True:
            raise SupervisorError("Old file-owner drain was not acknowledged")
    elif not pending.get("payload_cleanup_done"):
        raise SupervisorError("Old owner catalog unavailable for payload cleanup")
    if pending.get("kind") == "current-restore":
        backup = Path(str(pending.get("backup", "")))
        _manifest, _original, inventory = validated_restore_backup(
            backup, SUBSTRATE_INTERPRETATION, selected_payload_helper(config)
        )
    else:
        inventory = object_inventory(payloads["payload_inventory"])
    placements = [
        {"payload_incarnation": old_incarnation, "database": pending["database"]},
        *object_inventory(payloads["payload_targets"]),
    ]
    work = [(placement, inventory) for placement in placements]
    for retired in object_inventory(payloads["retired_payloads"]):
        owners = retired.get("owners")
        retired_inventory = object_inventory(retired.get("inventory"))
        if any(owner.get("drained") is not True for owner in object_inventory(owners)):
            raise SupervisorError(
                "Retired payload target lost its authenticated drain inventory"
            )
        work.append((retired, retired_inventory))
    selected_root = payload_root(state, config)
    # Every inventory, placement, lock and unknown-file check precedes deletion.
    for placement, placement_inventory in work:
        retired_payload_cleanup_files(
            state, selected_root, placement, placement_inventory
        )
    for placement, placement_inventory in work:
        # Reobserve each placement immediately before deletion, retaining its
        # original path/symlink/lock/unknown-content checks after global preflight.
        removal = retired_payload_cleanup_files(
            state, selected_root, placement, placement_inventory
        )
        if removal is None:
            continue
        native_payload_operation(
            state, config, "retired-remove", placement_inventory, placement=placement
        )
    pending["payload_cleanup_done"] = True
    config["derived_rebuild_pending"] = pending
    write_json(state / "config.json", config)


def finish_maintenance(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    pending = object_mapping(config["derived_rebuild_pending"])
    require_private_catalog(state, config, credentials)
    maintenance_phase(state, config, "disposing")
    cleanup_retired_payloads(state, config, credentials)
    old = {**config, "namespace": pending["namespace"], "database": pending["database"]}
    if database_present(state, old, credentials):
        discard_database(state, config, old, credentials)
    drain_maintenance_listener(state)
    config.update(config_for(state))
    config.pop("derived_rebuild_pending")
    config["admission"] = (
        "quiesced" if pending["kind"] == "rebuild" else "validation_required"
    )
    write_json(state / "config.json", config)
    retire_maintenance_payloads(state, pending)


def copy_current_database(
    state: Path,
    source: dict[str, object],
    target: dict[str, object],
    credentials: dict[str, object],
) -> None:
    # Reserve the exact path durably before its exclusive creation. An empty,
    # still-unbound file at that reservation is the only pre-inode crash state.
    pending = object_mapping(target["derived_rebuild_pending"])
    nonce = pending.get("export_nonce")
    if nonce is None:
        if any(state.glob(".maintenance-export*.surql")):
            raise SupervisorError(
                "Unknown maintenance export retained; admission remains closed"
            )
        nonce = uuid.uuid4().hex
        pending["export_nonce"] = nonce
        pending["export_path"] = ".maintenance-export-" + nonce + ".surql"
        target["derived_rebuild_pending"] = pending
        write_json(state / "config.json", target)
    if (
        not isinstance(nonce, str)
        or re.fullmatch(r"[a-f0-9]{32}", nonce) is None
        or pending.get("export_path") != ".maintenance-export-" + nonce + ".surql"
    ):
        raise SupervisorError("Unknown maintenance export reservation")
    exported = state / str(pending["export_path"])
    if exported.is_symlink():
        raise SupervisorError("Maintenance export must not traverse symlinks")
    if exported.exists():
        descriptor = os.open(exported, os.O_WRONLY | os.O_NOFOLLOW)
        observed = os.fstat(descriptor)
        bound = pending.get("export_identity")
        private = (
            stat.S_ISREG(observed.st_mode)
            and observed.st_uid == os.getuid()
            and stat.S_IMODE(observed.st_mode) == 0o600
            and observed.st_nlink == 1
        )
        if (
            not private
            or (bound is None and observed.st_size != 0)
            or (bound is not None and bound != [observed.st_dev, observed.st_ino])
        ):
            os.close(descriptor)
            raise SupervisorError(
                "Unknown maintenance export retained; admission remains closed"
            )
    else:
        descriptor = os.open(
            exported, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600
        )
    identity = os.fstat(descriptor)
    os.close(descriptor)
    pending["export_identity"] = [identity.st_dev, identity.st_ino]
    target["derived_rebuild_pending"] = pending
    write_json(state / "config.json", target)
    # A prior interrupted export is overwritten only after binding its identity.
    descriptor = os.open(exported, os.O_WRONLY | os.O_NOFOLLOW)
    try:
        observed = os.fstat(descriptor)
        if [observed.st_dev, observed.st_ino] != pending["export_identity"]:
            raise SupervisorError(
                "Maintenance export ownership changed; admission remains closed"
            )
        os.ftruncate(descriptor, 0)
    finally:
        os.close(descriptor)
    try:
        endpoint = maintenance_endpoint(state)
        binary = str(object_mapping(source["server"])["binary"])
        for action, identity in (("export", source), ("import", target)):
            result = subprocess.run(
                [
                    binary,
                    action,
                    "--log=none",
                    "--endpoint",
                    endpoint,
                    "--namespace",
                    str(identity["namespace"]),
                    "--database",
                    str(identity["database"]),
                    str(exported),
                ],
                env=server_environment(source, credentials),
                cwd=state,
                capture_output=True,
                check=False,
            )
            if result.returncode:
                raise SupervisorError(
                    "Native current restore copy failed; admission remains closed"
                )
    finally:
        drain_maintenance_listener(state)
        target.update(config_for(state))
        pending = object_mapping(target["derived_rebuild_pending"])
        if exported.exists():
            observed = exported.stat()
            if (
                exported.is_symlink()
                or [observed.st_dev, observed.st_ino] != pending["export_identity"]
            ):
                raise SupervisorError(
                    "Maintenance export ownership changed; admission remains closed"
                )
            exported.unlink()
        for field in ("export_identity", "export_path", "export_nonce"):
            pending.pop(field, None)
        write_json(state / "config.json", target)


def rehome_restored_payloads(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Copy verified backup bytes to fresh placement, then re-fence the imported catalog."""
    pending = object_mapping(config["derived_rebuild_pending"])
    backup = Path(str(pending.get("backup", "")))
    _manifest, original, inventory = validated_restore_backup(
        backup, SUBSTRATE_INTERPRETATION, selected_payload_helper(config)
    )
    if file_digest(backup / "backup.json") != pending.get("backup_digest"):
        raise SupervisorError(
            "Payload restore source changed; admission remains closed"
        )
    old_incarnation = original["payload_incarnation"]
    new_incarnation = config["payload_incarnation"]
    context = maintenance_domain_context(
        state, config, credentials, "rehome", str(old_incarnation), str(new_incarnation)
    )
    target = {"payload_incarnation": new_incarnation, "database": config["database"]}
    payloads = maintenance_payloads(state, pending)
    targets = object_inventory(payloads["payload_targets"])
    for placement in targets:
        payload_root(state, placement)
    if target not in targets:
        targets.append(target)
        payloads["payload_targets"] = targets
        write_maintenance_payloads(state, config, payloads)
    native_payload_operation(
        backup,
        config,
        "restore",
        inventory,
        destination=state,
        placement=original,
        target=target,
    )
    # The imported gate remains closed throughout bounded catalog updates. These
    # physical fields are absent from scientific descriptors and manifest bytes.
    after = ""
    while True:
        value = maintenance_domain_call(
            state, config, credentials, "page", [*context, "rehome", after]
        )
        _count, after, complete = maintenance_domain_page(value, after)
        if complete:
            break
    if (
        maintenance_domain_call(
            state, config, credentials, "rehome_complete", list(context)
        )
        is not True
    ):
        raise SupervisorError("Restored file incarnation was not acknowledged")
    config["payload_frozen"] = True
    write_json(state / "config.json", config)


def initialize(state: Path, initializer: list[str]) -> dict[str, object]:
    """Install current schema under the existing exact maintenance authority."""
    if not initializer:
        raise SupervisorError(
            "initialize requires an explicit current initializer command"
        )
    with (
        lifecycle_deadline(time.monotonic() + 90) as deadline,
        lifecycle_reservation(state, deadline=deadline),
    ):
        selected = config_for(state)
        pending = selected.get("derived_rebuild_pending")
        if selected.get("accepting_writes") is not False or selected.get(
            "surrealkit_rollout_pending"
        ):
            raise SupervisorError(
                "Initialization requires closed independent application admission"
            )
        if pending is None:
            if selected.get("admission") != "validation_required":
                raise SupervisorError(
                    "Initialization selects only a fresh closed target"
                )
            pending = {
                "kind": "initialize",
                "phase": "initializing",
                "nonce": uuid.uuid4().hex,
                "target_namespace": selected["namespace"],
                "target_database": selected["database"],
            }
        pending = object_mapping(pending)
        if (
            pending.get("kind") != "initialize"
            or pending.get("phase") not in {"initializing", "validated"}
            or (pending.get("target_namespace"), pending.get("target_database"))
            != (selected["namespace"], selected["database"])
        ):
            raise SupervisorError(
                "Initialization pending identity does not match this exact target"
            )
        private_offline_state(state, selected)
        selected["derived_rebuild_pending"] = pending
        write_json(state / "config.json", selected)
        initialize_rebuild(state, selected, initializer)
        drain_maintenance_listener(state)
        selected.update(config_for(state))
        completed = object_mapping(selected.get("derived_rebuild_pending"))
        if (
            completed.get("nonce") != pending["nonce"]
            or completed.get("phase") != "validated"
        ):
            raise SupervisorError(
                "Initialization settlement changed; admission remains closed"
            )
        selected.pop("derived_rebuild_pending")
        selected.update(admission="quiesced", accepting_writes=False)
        write_json(state / "config.json", selected)
        return public_status(state, selected)


def discard_generated_tree(state: Path, relative: str) -> None:
    """Discard only a fixed, private, drained design-phase generated subtree."""
    if relative not in {"database", "payloads", "tmp", ".contexts", ".receivers"}:
        raise SupervisorError(
            "Fresh-engine disposal escaped its fixed generated owners"
        )
    target = state / relative
    if not target.exists() and not target.is_symlink():
        return
    entries = [target, *target.rglob("*")] if target.is_dir() else [target]
    for path in entries:
        observed = path.lstat()
        if (
            path.is_symlink()
            or observed.st_uid != os.getuid()
            or not (stat.S_ISREG(observed.st_mode) or stat.S_ISDIR(observed.st_mode))
        ):
            raise SupervisorError(
                "Fresh-engine disposal requires private ordinary owned files"
            )
    if not target.is_dir():
        raise SupervisorError("Fresh-engine generated subtree is not a directory")
    shutil.rmtree(target)


def fresh_engine(state: Path, tool_root: Path | None = None) -> dict[str, object]:
    """Retire the stopped old engine and publish one fresh, closed MX identity.

    Acceptance of an old profile exists only inside this retirement owner. No
    query, import, old-engine reopen or data-preservation step occurs here.
    """
    state = checked_directory(state)
    previous = getattr(_LIFECYCLE, "retire_engine", False)
    _LIFECYCLE.retire_engine = True
    try:
        with lifecycle_reservation(state):
            old = config_for(state)
            if old.get("accepting_writes") is not False or old.get("admission") not in {
                "quiesced",
                "validation_required",
            }:
                raise SupervisorError(
                    "Fresh-engine requires already closed application admission"
                )
            if old.get("derived_rebuild_pending") or old.get(
                "surrealkit_rollout_pending"
            ):
                raise SupervisorError(
                    "Settle the exact existing maintenance owner before fresh-engine"
                )
            # Caller acknowledgment does not discharge any actual process owner.
            all_contexts_drained(state)
            if active(state):
                raise SupervisorError(
                    "Fresh-engine requires the selected listener already stopped"
                )
            stop(state, old)
            pending_path = state / "fresh-engine-pending.json"
            if pending_path.exists():
                pending = read_json(pending_path)
                target = object_mapping(pending.get("target"))
                if (
                    pending.get("owner") != OWNER
                    or pending.get("phase") not in {"discarding", "publishing"}
                    or old["instance_id"]
                    not in {pending.get("predecessor"), target.get("instance_id")}
                    or not surreal_source.admitted_receipt(
                        object_mapping(target.get("server"))
                    )
                ):
                    raise SupervisorError(
                        "Fresh-engine pending identity does not match this exact target"
                    )
                credentials = object_mapping(pending.get("credentials"))
            else:
                if tool_root is None:
                    tool_root = default_tool_root()
                server = install(None, tool_root)
                try:
                    module = surrealism_module.install(server, tool_root)
                except (surrealism_module.ModuleError, OSError, ValueError) as error:
                    raise SupervisorError(str(error)) from error
                budget = object_mapping(old["resources"])
                allocation = resources(
                    integer(budget["total_memory_bytes"]),
                    integer(budget["server_memory_bytes"]),
                    integer(budget["native_workers"]),
                    integer(budget["native_worker_memory_bytes"]),
                    budget.get("execution"),
                )
                target = {
                    key: old[key]
                    for key in (
                        "owner",
                        "port",
                        "credentials_file",
                        "log_max_bytes",
                        "log_backups",
                        "service_class",
                        "resident",
                        "tooling_fixture",
                        "telemetry_state",
                    )
                    if key in old
                }
                target = fresh_database_identity(target)
                # The state-path service file remains this authenticated,
                # physically drained owner's materialization. Rebinding its
                # immutable generation must not turn it into an unknown unit.
                target["unit_materialized"] = old.get("unit_materialized", False)
                target.update(
                    profile_version=3,
                    server=server,
                    study_module=module,
                    resources=allocation,
                    endpoint=f"grpc://127.0.0.1:{old['port']}",
                    interpretation=SUBSTRATE_INTERPRETATION,
                    schema_interpretation=SUBSTRATE_INTERPRETATION,
                    max_message_bytes=MESSAGE_BYTES,
                    rpc_max_message_bytes=MESSAGE_BYTES,
                    service_supervisor=publish_generation(state),
                )
                # Receiver and verifier executables are explicitly readmitted
                # from current builds; the retired generation cannot execute.
                credentials = fresh_credentials()
                target["fresh_root_bootstrap"] = {
                    "instance_id": target["instance_id"],
                    "credentials_sha256": hashlib.sha256(
                        (
                            json.dumps(credentials, sort_keys=True, indent=2) + "\n"
                        ).encode()
                    ).hexdigest(),
                    "nonce": uuid.uuid4().hex,
                }
                pending = {
                    "owner": OWNER,
                    "nonce": uuid.uuid4().hex,
                    "predecessor": old["instance_id"],
                    "phase": "discarding",
                    "target": target,
                    "credentials": credentials,
                }
                write_json(pending_path, pending)
            if pending["phase"] == "discarding":
                for relative in (
                    "database",
                    "payloads",
                    "tmp",
                    ".contexts",
                    ".receivers",
                ):
                    discard_generated_tree(state, relative)
                for relative in (
                    "protocol-readiness.json",
                    "recovery-qualification.json",
                    "server-process.json",
                    "service-launch.json",
                    ".maintenance-payloads.json",
                ):
                    path = state / relative
                    if path.is_symlink():
                        raise SupervisorError(
                            "Fresh-engine receipt must not be a symlink"
                        )
                    path.unlink(missing_ok=True)
                pending["phase"] = "publishing"
                write_json(pending_path, pending)
            write_json(state / "credentials.json", credentials)
            write_json(state / "config.json", target)
            (state / "tmp").mkdir(mode=0o700, exist_ok=True)
            payload_root(state, target).mkdir(mode=0o700, parents=True, exist_ok=True)
            write_json(state / "config.json", target)
            # The config remains closed after the pending fence is removed.
            # Schema installation and admission are distinct explicit owners.
            pending_path.unlink()
            descriptor = os.open(state, os.O_DIRECTORY)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
            return public_status(state, config_for(state))
    finally:
        _LIFECYCLE.retire_engine = previous


def rebuild(state: Path, initializer: list[str]) -> dict[str, object]:
    """Regenerate owned design-phase state from current sources, without retaining old data."""
    if not initializer:
        raise SupervisorError("Fresh rebuild requires a current initializer command")
    with lifecycle_reservation(state):
        old = config_for(state)
        private_offline_state(state, old)
        if old.get("derived_rebuild_pending") or old.get("surrealkit_rollout_pending"):
            raise SupervisorError(
                "Incomplete rebuild remains closed; explicit owner recovery is required"
            )
        old_payloads = freeze_payloads(state, old)
        stop(state, old)
        selected = fresh_owned_database_identity(state, old)
        selected.pop("payload_frozen", None)
        initializer_owner = read_json(state / "lifecycle-owner.json")
        credentials = fresh_credentials()
        prior = read_json(state / "credentials.json")
        selected.update(
            interpretation=SUBSTRATE_INTERPRETATION,
            schema_interpretation=SUBSTRATE_INTERPRETATION,
            derived_rebuild_pending={
                "namespace": old["namespace"],
                "database": old["database"],
                "payload_incarnation": old["payload_incarnation"],
                "initializer": initializer_owner,
                "kind": "rebuild",
                "phase": "accounts",
                "target_namespace": selected["namespace"],
                "target_database": selected["database"],
                "previous_username": prior["username"],
                "next_username": credentials["username"],
            },
        )
        write_maintenance_payloads(
            state,
            selected,
            {
                "payload_inventory": old_payloads,
                "payload_targets": [],
                "retired_payloads": [],
            },
        )
        rotate_database_accounts(state, selected, prior, credentials)
        initialize_rebuild(state, selected, initializer)
        finish_maintenance(state, selected, credentials)
        return public_status(state, selected)


def initialize_rebuild(
    state: Path,
    selected: dict[str, object],
    initializer: list[str],
) -> None:
    drain_maintenance_listener(state)
    selected["service_supervisor"] = config_for(state)["service_supervisor"]
    initializer_owner = read_json(state / "lifecycle-owner.json")
    pending = object_mapping(selected["derived_rebuild_pending"])
    pending["initializer"] = initializer_owner
    selected["derived_rebuild_pending"] = pending
    maintenance_phase(state, selected, "initializing")
    # Credentials and fresh target are durable before any listener can start.
    try:
        maintenance_endpoint(state)
        selected.pop("fresh_root_bootstrap", None)
        selected.update(config_for(state))
        if pending.get("kind") == "initialize":
            # A fresh catalog has no bootstrap root or VIEWER account yet.
            # The exact closed maintenance owner authorizes their installation.
            establish_protocol_readiness(
                state,
                selected,
                getattr(_STARTUP, "deadline", None) or time.monotonic() + 90,
            )
        # Ordinary admission stays closed, including after owner death. Only
        # this explicit child can borrow the exact live lifecycle authority.
        diagnostic = state / ("canonical-initializer-" + uuid.uuid4().hex + ".log")
        descriptor = os.open(diagnostic, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "wb") as output:
            status = run_file_effect(
                initializer,
                {
                    **os.environ,
                    "PSE_SURREAL_STATE": str(state),
                    "PSE_CANONICAL_INITIALIZER": str(initializer_owner["nonce"]),
                },
                **operation_lifetime.deadline_options(
                    getattr(_STARTUP, "deadline", None)
                ),
                stdout=output.fileno(),
                stderr=output.fileno(),
            )
        if status:
            raise SupervisorError(
                "Current initializer failed; admission closed and exact target retained; private diagnostic "
                + str(diagnostic)
            )
        validate_interpretation(state, selected, SUBSTRATE_INTERPRETATION)
        if read_json(
            state / "lifecycle-owner.json"
        ) != initializer_owner or not reservation_live(initializer_owner):
            raise SupervisorError("Initializer owner changed; admission remains closed")
        maintenance_phase(state, selected, "validated")
    finally:
        # Listener setup can publish a supervisor and materialize its unit before
        # failing. Preserve those authoritative effects, including their ACK,
        # rather than rolling them back with this caller's pre-start snapshot.
        current = config_for(state)
        if current.get("instance_id") != selected.get("instance_id"):
            raise SupervisorError(
                "Initializer service identity changed; admission remains closed"
            )
        selected.clear()
        selected.update(current)
        selected["accepting_writes"] = False
        write_json(state / "config.json", selected)
        # Settle the listener's ownership as well as its physical process. A
        # later maintenance phase must start a new listener under this owner,
        # rather than find a retained marker for an already stopped process.
        drain_maintenance_listener(state)


def recover_credentials(
    state: Path, config: dict[str, object]
) -> tuple[dict[str, object], bool]:
    """Observe an atomic account change before deciding whether it committed."""
    pending = object_mapping(config["derived_rebuild_pending"])
    path = state / ".maintenance-credentials.json"
    current = read_json(state / "credentials.json")
    candidate = read_json(path) if path.exists() else None
    if candidate is not None and candidate.get("username") != pending["next_username"]:
        raise SupervisorError(
            "Unknown pending maintenance credentials; admission remains closed"
        )
    candidates = (
        [current] if candidate is None or candidate == current else [current, candidate]
    )
    admitted = []
    for credentials in candidates:
        try:
            root = maintenance_catalog(state, config, credentials, "ROOT")
        except SupervisorError:
            continue
        users = object_mapping(root["users"])
        value = {
            "current": credentials["username"] in users,
            "previous": pending["previous_username"] in users,
            "next": pending["next_username"] in users,
        }
        if (
            any(
                type(value.get(name)) is not bool
                for name in ("current", "previous", "next")
            )
            or value["previous"] == value["next"]
        ):
            raise SupervisorError(
                "Maintenance root-account state is uncertain; admission remains closed"
            )
        if value["current"]:
            admitted.append((credentials, value))
    drain_maintenance_listener(state)
    config.update(config_for(state))
    if len(admitted) != 1:
        raise SupervisorError(
            "Maintenance account authority is uncertain; admission remains closed"
        )
    credentials, value = admitted[0]
    if credentials["username"] == pending["next_username"] and value["next"]:
        write_json(state / "credentials.json", credentials)
        path.unlink(missing_ok=True)
        return credentials, True
    if credentials["username"] != pending["previous_username"] or not value["previous"]:
        raise SupervisorError("Maintenance account does not match recorded authority")
    if pending["phase"] in {"cleanup", "copy"}:
        if candidate is not None:
            raise SupervisorError("Unexpected account phase; admission remains closed")
        return credentials, False
    if pending["phase"] != "accounts":
        raise SupervisorError("Recorded account rotation was not observed")
    # The old root is still present and authenticates, while the exact candidate
    # is absent. The atomic transaction is established not to have committed.
    if candidate is None:
        candidate = fresh_credentials()
        pending["next_username"] = candidate["username"]
        config["derived_rebuild_pending"] = pending
        write_json(state / "config.json", config)
    rotate_database_accounts(state, config, credentials, candidate)
    return candidate, True


def recover_maintenance(
    state: Path, initializer: list[str], verifier: Path | None = None
) -> dict[str, object]:
    with lifecycle_reservation(state):
        service_borrowers.service_registry_binding(state)
        config = config_for(state)
        set_payload_verifier(state, config, verifier)
        if "backup_pending" in config:
            private_offline_state(state, config)
        if config.get("surrealkit_rollout_pending"):
            raise SupervisorError(
                "Interrupted beta rollout requires its exact rollout owner and settlement; rebuild recovery refuses it"
            )
        pending = object_mapping(config.get("derived_rebuild_pending"))
        maintenance_payloads(state, pending)
        if (
            pending.get("kind") == "current-restore"
            and pending.get("phase") == "backup-copy"
        ):
            complete_backup_copy(state, config)
            config = config_for(state)
            pending = object_mapping(config["derived_rebuild_pending"])
        private_offline_state(state, config)
        kind, phase = pending.get("kind"), pending.get("phase")
        phases = {
            "rebuild": {"accounts", "initializing", "validated", "disposing"},
            "current-restore": {
                "cleanup",
                "copy",
                "accounts",
                "validated",
                "disposing",
            },
        }
        if (
            not isinstance(kind, str)
            or not isinstance(phase, str)
            or kind not in phases
            or phase not in phases[kind]
            or config["interpretation"] != SUBSTRATE_INTERPRETATION
        ):
            raise SupervisorError("Unknown maintenance phase; admission remains closed")
        for name in (
            "namespace",
            "database",
            "target_namespace",
            "target_database",
            "previous_username",
            "next_username",
        ):
            sql_identifier(pending.get(name))
        if (config["namespace"], config["database"]) != (
            pending["target_namespace"],
            pending["target_database"],
        ) or (pending["namespace"], pending["database"]) == (
            config["namespace"],
            config["database"],
        ):
            raise SupervisorError(
                "Maintenance source/target identity differs; admission remains closed"
            )
        if (
            kind == "rebuild"
            and phase in {"accounts", "initializing"}
            and not initializer
        ):
            raise SupervisorError(
                "Interrupted initializer requires explicit current initializer command"
            )
        credentials, rotated = recover_credentials(state, config)
        require_private_catalog(state, config, credentials)
        if kind == "rebuild" and phase in {"accounts", "initializing"}:
            if database_present(state, config, credentials):
                retire_partial_payload_target(state, config, credentials)
                pending = object_mapping(config["derived_rebuild_pending"])
                discard_database(state, config, config, credentials)
            drain_maintenance_listener(state)
            config.update(config_for(state))
            selected = fresh_owned_database_identity(state, config)
            pending.update(
                target_namespace=selected["namespace"],
                target_database=selected["database"],
            )
            selected["derived_rebuild_pending"] = pending
            config = selected
            initialize_rebuild(state, config, initializer)
        elif kind == "current-restore" and phase in {"cleanup", "copy"}:
            if rotated:
                raise SupervisorError("Unexpected account rotation before current copy")
            source = {
                **config,
                "namespace": pending["namespace"],
                "database": pending["database"],
            }
            if not database_present(state, source, credentials):
                raise SupervisorError(
                    "Current restore source is absent; admission remains closed"
                )
            current_marker(state, source, credentials)
            preserved, _original, _files = validated_restore_backup(
                Path(str(pending["backup"])),
                SUBSTRATE_INTERPRETATION,
                selected_payload_helper(config),
            )
            if (
                capture_application_inventory(state, source, credentials)
                != preserved["application_inventory"]
            ):
                raise SupervisorError(
                    "Restored application catalogue differs from its required inventory"
                )
            # Re-mint source authorities before copying again: no old creation
            # capability from an unadmitted partial target survives recovery.
            rotate_source_authorities(state, source, credentials)
            if database_present(state, config, credentials):
                discard_database(state, config, config, credentials)
            drain_maintenance_listener(state)
            config.update(config_for(state))
            selected = fresh_owned_database_identity(state, config)
            pending.update(
                target_namespace=selected["namespace"],
                target_database=selected["database"],
            )
            selected["derived_rebuild_pending"] = pending
            config = selected
            maintenance_phase(state, config, "copy")
            copy_current_database(state, source, config, credentials)
            rehome_restored_payloads(state, config, credentials)
            pending = object_mapping(config["derived_rebuild_pending"])
            new_credentials = fresh_credentials()
            pending["next_username"] = new_credentials["username"]
            config["derived_rebuild_pending"] = pending
            maintenance_phase(state, config, "accounts")
            rotate_database_accounts(state, config, credentials, new_credentials)
            credentials = new_credentials
            current_marker(state, config, credentials)
            maintenance_phase(state, config, "validated")
        elif kind == "current-restore" and phase == "accounts":
            current_marker(state, config, credentials)
            maintenance_phase(state, config, "validated")
        else:
            current_marker(state, config, credentials)
        finish_maintenance(state, config, credentials)
        return public_status(state, config)


def validated_restore_backup(
    source: Path, interpretation: str, verifier: Path | None = None
) -> tuple[dict[str, object], dict[str, object], list[dict[str, object]]]:
    if any(path.is_symlink() for path in (source, *source.parents)) or any(
        path.is_symlink() for path in source.rglob("*")
    ):
        raise SupervisorError("Backup must not traverse symlinks")
    manifest = read_json(source / "backup.json")
    if (
        manifest.get("owner") != OWNER
        or manifest.get("backup_version") != 4
        or manifest.get("interpretation") != interpretation
    ):
        raise SupervisorError("Unknown backup or incompatible interpretation")
    files = manifest.get("files")
    if not isinstance(files, dict):
        raise SupervisorError("Invalid backup file inventory")
    original = manifest.get("state_root")
    if not isinstance(original, str) or not Path(original).is_absolute():
        raise SupervisorError("Missing backup source-state identity")
    actual = {
        str(p.relative_to(source))
        for p in source.rglob("*")
        if p.is_file() and p != source / "backup.json"
    }
    if actual != set(files):
        missing = sorted(set(files) - actual)
        extra = sorted(actual - set(files))
        raise SupervisorError(
            f"Backup file inventory differs from its manifest; missing={missing!r}, unexpected={extra!r}"
        )
    for name, digest in files.items():
        path = source / name
        if (
            Path(name).is_absolute()
            or ".." in Path(name).parts
            or path.is_symlink()
            or (Path(name).parts[0] != "payloads" and file_digest(path) != digest)
        ):
            raise SupervisorError("Backup path or content failed validation")
    validate_application_inventory(manifest.get("application_inventory"))
    config = config_for(source)
    if verifier is not None:
        config["payload_verifier"] = payload_verifier(verifier)
    inventory = validate_payload_inventory(
        source, config, manifest.get("payload_inventory")
    )
    for entry in inventory:
        for name, binding in object_mapping(entry["files"]).items():
            if files.get(name) != object_mapping(binding).get("sha256"):
                raise SupervisorError(
                    "Backup payload manifest differs from native inventory"
                )
    if config.get("derived_rebuild_pending") or config.get(
        "surrealkit_rollout_pending"
    ):
        raise SupervisorError(
            "Interrupted maintenance must be recovered by its original owner"
        )
    if (
        config["interpretation"] != interpretation
        or config["schema_interpretation"] != interpretation
    ):
        raise SupervisorError("Backup metadata has an incompatible interpretation")
    require_private_service_context(source, "Restore")
    if interpretation != SUBSTRATE_INTERPRETATION:
        raise SupervisorError(
            "Unsupported restore requires preserved authored inputs and explicit rebuild"
        )
    return manifest, config, inventory


def reroot_restored(value: object, original: str, state: Path) -> object:
    if isinstance(value, dict):
        return {
            key: reroot_restored(item, original, state) for key, item in value.items()
        }
    if isinstance(value, list):
        return [reroot_restored(item, original, state) for item in value]
    if isinstance(value, str) and (
        value == original or value.startswith(original + "/")
    ):
        return str(state) + value[len(original) :]
    return value


def metadata_write_scratch(path: Path, state: Path) -> bool:
    """Identify private atomic-write residue; never consume or delete its bytes."""
    if (
        path.parent != state
        or re.fullmatch(
            r"\.(config\.json|lifecycle-owner\.json)-[a-z0-9_]{8}", path.name
        )
        is None
        or path.is_symlink()
    ):
        return False
    observed = path.stat()
    return (
        stat.S_ISREG(observed.st_mode)
        and observed.st_uid == os.getuid()
        and stat.S_IMODE(observed.st_mode) == 0o600
        and observed.st_nlink == 1
    )


def complete_backup_copy(state: Path, config: dict[str, object]) -> None:
    """Resume only exact declared backup files, under the still-closed owner."""
    pending = object_mapping(config["derived_rebuild_pending"])
    source = Path(str(pending.get("backup", "")))
    if (
        not source.is_absolute()
        or source == state
        or source in state.parents
        or state in source.parents
    ):
        raise SupervisorError("Invalid preserved backup location")
    if file_digest(source / "backup.json") != pending.get("backup_digest"):
        raise SupervisorError(
            "Preserved backup manifest changed; admission remains closed"
        )
    manifest, original_config, inventory = validated_restore_backup(
        source, SUBSTRATE_INTERPRETATION, selected_payload_helper(config)
    )
    if (original_config["namespace"], original_config["database"]) != (
        pending["namespace"],
        pending["database"],
    ):
        raise SupervisorError("Preserved backup source identity changed")
    if (config["namespace"], config["database"]) != (
        pending["target_namespace"],
        pending["target_database"],
    ):
        raise SupervisorError("Preserved backup target identity changed")
    if active(state):
        raise SupervisorError(
            "Interrupted backup copy requires the owned server stopped"
        )
    all_contexts_drained(state)
    files = object_mapping(manifest["files"])
    roots = {
        "database",
        "credentials.json",
        ".generations",
        ".contexts",
        ".receivers",
        "payloads",
    }
    selected = {
        name: digest for name, digest in files.items() if Path(name).parts[0] in roots
    }
    if "credentials.json" not in selected or not (source / "database").is_dir():
        raise SupervisorError("Backup is missing owned state")
    permitted = set(selected) | {
        "config.json",
        ".supervisor.lock",
        "lifecycle-owner.json",
        ".resource-owner/registry.json",
    }
    for path in state.rglob("*"):
        if path.relative_to(state).parts[0] == "payloads":
            # Native restore validates the complete destination tree and its fresh locks.
            continue
        metadata_scratch = metadata_write_scratch(path, state)
        # Never parse, adopt or delete interrupted atomic-write scratch. Its
        # known basename is disjoint from every copied file and authority.
        if path.is_symlink() or (
            path.is_file()
            and str(path.relative_to(state)) not in permitted
            and not metadata_scratch
        ):
            raise SupervisorError(
                "Unknown interrupted backup content retained; admission remains closed"
            )
    # Known partial file copies can be overwritten from the immutable verified
    # backup. Never import its old, normally startable config.json.
    for directory in sorted(source.rglob("*")):
        if directory.is_dir() and directory.relative_to(source).parts[0] in roots - {
            "payloads"
        }:
            (state / directory.relative_to(source)).mkdir(
                mode=0o700, parents=True, exist_ok=True
            )
    for name, digest in sorted(selected.items()):
        if Path(name).parts[0] == "payloads":
            continue
        target = state / name
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        shutil.copy2(source / name, target)
        with target.open("rb") as stream:
            os.fsync(stream.fileno())
        if file_digest(target) != digest:
            raise SupervisorError(
                "Restored backup copy differs; admission remains closed"
            )
    native_payload_operation(
        source,
        config,
        "restore",
        inventory,
        destination=state,
        placement=original_config,
    )
    validated_restore_backup(
        source, SUBSTRATE_INTERPRETATION, selected_payload_helper(config)
    )
    if file_digest(source / "backup.json") != pending["backup_digest"]:
        raise SupervisorError("Preserved backup changed during copy")
    original = str(manifest["state_root"])
    for descriptor in (state / ".contexts").glob("*.json"):
        selected_context = reroot_restored(read_json(descriptor), original, state)
        if not isinstance(selected_context, dict):
            raise SupervisorError("Invalid restored context descriptor")
        write_json(descriptor, selected_context)
    for receiver in (state / ".receivers").glob("*/*/config.json"):
        selected_receiver = reroot_restored(read_json(receiver), original, state)
        if not isinstance(selected_receiver, dict):
            raise SupervisorError("Invalid restored receiver profile")
        selected_receiver.update(
            instance_id=config["instance_id"],
            payload_incarnation=config["payload_incarnation"],
            admission="validation_required",
            accepting_writes=False,
            unit_materialized=False,
            restart_qualified=False,
        )
        write_json(receiver, selected_receiver)
    for generation in (
        (state / ".generations").iterdir() if (state / ".generations").exists() else []
    ):
        verify_generation(generation)
    owned_generations(state)
    (state / "tmp").mkdir(mode=0o700, exist_ok=True)
    # Flush copied directory entries before advancing the durable owner phase.
    for directory in [state, *(path for path in state.rglob("*") if path.is_dir())]:
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    maintenance_phase(state, config, "cleanup")


def restore(
    source: Path, state: Path, interpretation: str, verifier: Path | None = None
) -> dict[str, object]:
    manifest, original, _inventory = validated_restore_backup(
        source, interpretation, verifier
    )
    state = checked_directory(state)
    with lifecycle_reservation(state):
        binding = state / ".resource-owner"
        if binding.exists() or binding.is_symlink():
            # An interrupted first publication can leave its established local
            # registry binding. Verify that exact owner; never import a backup's
            # resource authority or infer coverage for a previously live state.
            if (
                binding.is_symlink()
                or not binding.is_dir()
                or set(binding.iterdir()) != {binding / "registry.json"}
            ):
                raise SupervisorError("Unknown restore resource authority retained")
            service_borrowers.service_registry_binding(state)
        if any(
            path.name
            not in {".supervisor.lock", "lifecycle-owner.json", ".resource-owner"}
            and not metadata_write_scratch(path, state)
            for path in state.iterdir()
        ):
            raise SupervisorError(
                "Refusing to initialize or replace a nonempty directory"
            )
        selected = reroot_restored(
            fresh_database_identity(original), str(manifest["state_root"]), state
        )
        if not isinstance(selected, dict):
            raise SupervisorError("Invalid restored profile")
        selected["credentials_file"] = str(state / "credentials.json")
        selected["payload_frozen"] = True
        if verifier is not None:
            selected["payload_verifier"] = payload_verifier(verifier)
        credentials = read_json(source / "credentials.json")
        selected["derived_rebuild_pending"] = {
            "kind": "current-restore",
            "phase": "backup-copy",
            "namespace": original["namespace"],
            "database": original["database"],
            "target_namespace": selected["namespace"],
            "target_database": selected["database"],
            "previous_username": credentials["username"],
            "next_username": fresh_credentials()["username"],
            "backup": str(source.absolute()),
            "backup_digest": file_digest(source / "backup.json"),
            "payload_incarnation": original["payload_incarnation"],
        }
        # Publish the fresh, closed identity before any bytes of the old database or
        # credentials. Every interruption is recoverable only through this owner.
        service_borrowers.service_registry_binding(state, establish=True)
        write_json(state / "config.json", selected)
        return recover_maintenance(state, [])


def run_file_effect(
    command: list[str],
    environment: dict[str, str],
    *,
    deadline: float | None = None,
    stdout: int | None = None,
    stderr: int | None = None,
) -> int:
    """Drain a managed file borrower separately; preserve observable bare callers."""
    if deadline is not None and not math.isfinite(deadline):
        raise SupervisorError("Invalid file effect deadline")
    group = operation_lifetime.process_group(os.getpid())
    managed = "PSE_HOST_ALLOCATION" in environment or any(
        part.startswith(("pse-native-", "pse-surreal-worker-"))
        and part.endswith((".scope", ".service"))
        for part in Path(group).parts
    )
    if managed:
        # Missing or contradictory managed authority must refuse in the existing
        # factory. It is never permission to launch an untracked bare borrower.
        return native_operation.run_child_scope(
            command,
            environment,
            stdout=stdout,
            stderr=stderr,
            **operation_lifetime.deadline_options(deadline),
        )
    timeout = None if deadline is None else deadline - time.monotonic()
    if timeout is not None and timeout <= 0:
        raise subprocess.TimeoutExpired(command, 0)
    return subprocess.run(
        command,
        env=environment,
        stdout=stdout,
        stderr=stderr,
        timeout=timeout,
        check=False,
    ).returncode


def validation_payload_reads(
    state: Path, config: dict[str, object], interpretation: str
) -> None:
    """Admit validator file readers atomically while application writes/GC stay closed."""
    selected = config_for(state)
    if (
        selected.get("accepting_writes") is not False
        or selected.get("admission") != "validation_required"
        or selected.get("payload_frozen") is not True
        or selected.get("derived_rebuild_pending")
        or selected.get("surrealkit_rollout_pending")
        or selected.get("interpretation") != interpretation
        or any(
            selected.get(key) != config.get(key)
            for key in ("instance_id", "namespace", "database", "payload_incarnation")
        )
    ):
        raise SupervisorError(
            "Validator file reads require the exact closed restored profile"
        )
    incarnation = str(config["payload_incarnation"])
    context = maintenance_domain_context(
        state, config, None, "validation", incarnation, incarnation
    )
    acknowledged = maintenance_domain_call(
        state, config, None, "validation_reads", [*context, interpretation]
    )
    if acknowledged is not True:
        raise SupervisorError("Validator file-read admission was not acknowledged")


def close_validation_payload_reads(state: Path, config: dict[str, object]) -> None:
    """Reclose before authenticating actual validator owners and managed descendants."""
    if (
        payload_query(
            state,
            config,
            f"RETURN fn::pse_files_v1::freeze({payload_expiry()},{json.dumps(config['payload_incarnation'])});",
        )
        is not True
    ):
        raise SupervisorError("Validator file-read closure was not acknowledged")
    all_contexts_drained(state)
    reap_payload_owners(state, config, require_all=True)


def validate(
    state: Path, config: dict[str, object], interpretation: str, command: list[str]
) -> dict[str, object]:
    if (
        config["admission"] != "validation_required"
        or config.get("accepting_writes") is not False
        or config.get("payload_frozen") is not True
        or config.get("derived_rebuild_pending")
        or config.get("surrealkit_rollout_pending")
        or config["interpretation"] != interpretation
        or not command
    ):
        raise SupervisorError(
            "Validation requires a gated restore, matching interpretation and a check command"
        )
    start(state, config, validation=True)
    environment = os.environ.copy()
    environment["PSE_SURREAL_STATE"] = str(state)
    # The validator owns exact values/schema/operation identities; no check is inferred from /health.
    try:
        validate_interpretation(state, config, interpretation)
        all_contexts_drained(state)
        validation_payload_reads(state, config, interpretation)
        status = run_file_effect(
            command,
            environment,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            deadline=time.monotonic() + 120,
        )
        if status:
            raise SupervisorError(
                "Restored database semantic validation failed; admission remains closed"
            )
    finally:
        # Even an unacknowledged admission transaction or timed-out validator
        # remains protected until file admission is closed and exact owners drain.
        close_validation_payload_reads(state, config)
        stop(state, config)
    config["admission"] = "quiesced"
    write_json(state / "config.json", config)
    return public_status(state, config)


def validate_interpretation(
    state: Path, config: dict[str, object], interpretation: str
) -> None:
    """Authenticate and verify the restored canonical reader interpretation."""
    credentials = read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    request = urllib.request.Request(
        f"http://127.0.0.1:{config['port']}/sql",
        data=b"SELECT interpretation, study_module_sha256 FROM canonical_interpretations:current;",
        headers={
            "Authorization": f"Basic {token}",
            "Accept": "application/json",
            "surreal-ns": str(config["namespace"]),
            "surreal-db": str(config["database"]),
        },
    )
    deadline = getattr(_STARTUP, "deadline", None)
    timeout = 10 if deadline is None else min(10, remaining(deadline))
    with urllib.request.urlopen(request, timeout=timeout) as response:  # noqa: S310 -- Literal authenticated loopback HTTP endpoint from owned configuration.
        result = json.load(response)
    if deadline is not None:
        remaining(deadline)
    if (
        not isinstance(result, list)
        or len(result) != 1
        or not isinstance(result[0], dict)
        or result[0].get("status") != "OK"
        or result[0].get("result")
        != [
            {
                "interpretation": interpretation,
                "study_module_sha256": study_module(config)["sha256"],
            }
        ]
    ):
        raise SupervisorError(
            "Canonical database interpretation was not validated; writes remain closed"
        )


def recovery_generation(state: Path, config: dict[str, object]) -> str:
    """Bind recovery evidence to the exact owned storage and service closure."""
    supervisor = config.get("service_supervisor")
    if not isinstance(supervisor, dict) or not supervisor.get("supervisor_script"):
        raise SupervisorError(
            "Recovery qualification requires an immutable service supervisor"
        )
    directory = Path(str(supervisor["supervisor_script"])).parents[1]
    if directory.parent != state / ".generations":
        raise SupervisorError(
            "Recovery service closure is outside its owned generation"
        )
    verify_generation(directory)
    binary = config.get("server")
    if not isinstance(binary, dict) or file_digest(
        Path(str(binary["binary"]))
    ) != binary.get("binary_sha256"):
        raise SupervisorError("Recovery qualification storage artifact changed")
    selected = {
        key: config.get(key)
        for key in (
            "instance_id",
            "server",
            "study_module",
            "study_module_qualification",
            "service_supervisor",
            "resources",
            "namespace",
            "database",
            "endpoint",
            "schema_interpretation",
            "rpc_max_message_bytes",
        )
    }
    selected["credentials_sha256"] = file_digest(state / "credentials.json")
    return hashlib.sha256(
        json.dumps(selected, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def recovery_qualified(state: Path, config: dict[str, object]) -> bool:
    proof = state / "recovery-qualification.json"
    if not config.get("restart_qualified") or not proof.is_file():
        return False
    result = read_json(proof)
    return (
        result.get("version") == 1
        and result.get("outcome") == "passed"
        and result.get("generation") == recovery_generation(state, config)
    )


def administrative_headers(
    config: dict[str, object],
    token: str,
    namespace: str | None,
    database: str | None,
    *,
    select_context: bool = True,
) -> dict[str, str]:
    """Select one validated administrative context or explicitly remain unselected."""
    if (namespace is None) != (database is None):
        raise SupervisorError(
            "Administrative context overrides require both identifiers"
        )
    selected = (
        (config["namespace"], config["database"])
        if namespace is None
        else (namespace, database)
    )
    if any(
        not isinstance(value, str)
        or not 1 <= len(value) <= 128
        or not all(
            character.isascii() and (character.isalnum() or character == "_")
            for character in value
        )
        for value in selected
    ):
        raise SupervisorError(
            "Administrative context requires bounded ASCII identifiers"
        )
    headers = {"Authorization": f"Basic {token}", "Accept": "application/json"}
    if select_context:
        headers.update({"surreal-ns": str(selected[0]), "surreal-db": str(selected[1])})
    return headers


def administrative_results(
    state: Path,
    query: str,
    deadline: float,
    *,
    namespace: str | None = None,
    database: str | None = None,
    select_context: bool = True,
) -> list[dict[str, object]]:
    """Preserve SQL bytes and explicit per-statement outcomes in bounded control."""
    config = config_for(state)
    credentials = read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    request = urllib.request.Request(
        f"http://127.0.0.1:{config['port']}/sql",
        data=query.encode(),
        headers=administrative_headers(
            config, token, namespace, database, select_context=select_context
        ),
    )
    with urllib.request.urlopen(request, timeout=remaining(deadline)) as response:  # noqa: S310 -- literal authenticated loopback endpoint; config_for validates port
        payload = response.read(MESSAGE_BYTES + 1)
    if len(payload) > MESSAGE_BYTES:
        raise SupervisorError("Administrative response exceeded its bounded extent")
    try:
        results = json.loads(payload)
    except ValueError as error:
        raise SupervisorError("Administrative response was not valid JSON") from error
    if (
        not isinstance(results, list)
        or not results
        or any(
            not isinstance(row, dict)
            or not isinstance(row.get("status"), str)
            or row.get("status") not in {"OK", "ERR"}
            or "result" not in row
            or (row["status"] == "ERR" and not isinstance(row["result"], str))
            for row in results
        )
    ):
        raise SupervisorError(
            "Administrative response contained an invalid or incomplete statement"
        )
    remaining(deadline)
    return results


def administrative_query(
    state: Path,
    query: str,
    deadline: float,
    *,
    namespace: str | None = None,
    database: str | None = None,
    select_context: bool = True,
) -> list[dict[str, object]]:
    """Bounded all-OK administrative control; scientific RPC remains native WS."""
    results = administrative_results(
        state,
        query,
        deadline,
        namespace=namespace,
        database=database,
        select_context=select_context,
    )
    if any(row["status"] != "OK" for row in results):
        raise SupervisorError(
            "Administrative query contained a failed or incomplete statement"
        )
    return results


def abandon_administrative_response(
    state: Path,
    query: str,
    deadline: float,
    *,
    namespace: str | None = None,
    database: str | None = None,
) -> None:
    """Submit the complete small HTTP probe and abandon its unknown acknowledgment."""
    config = config_for(state)
    credentials = read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    headers = administrative_headers(config, token, namespace, database)
    connection = http.client.HTTPConnection(
        "127.0.0.1", integer(config["port"]), timeout=remaining(deadline)
    )
    try:
        connection.request(
            "POST",
            "/sql",
            body=query.encode(),
            headers=headers,
        )
        # No getresponse(): complete request transmission does not establish commit.
    finally:
        connection.close()


def qualify_recovery(state: Path) -> dict[str, object]:
    """Destructive, explicitly selected administrative recovery qualification."""
    deadline = time.monotonic() + 90
    prior = getattr(_STARTUP, "deadline", None)
    _STARTUP.deadline = deadline if prior is None else min(prior, deadline)
    deadline = _STARTUP.deadline
    try:
        remaining(deadline)
        with lifecycle_reservation(state):
            if config_for(state).get("surrealkit_rollout_pending"):
                raise SupervisorError(
                    "Pending beta rollout requires exact child drain and operation settlement before recovery qualification"
                )
            close_context_admission(state)
            all_contexts_drained(state)
            config = config_for(state)
            config["restart_qualified"] = False
            write_json(state / "config.json", config)
            generation = recovery_generation(state, config)
            operation = uuid.uuid4().hex
            unknown_operation = uuid.uuid4().hex
            namespace = "pse_recovery"
            database = "recovery_" + operation
            controls = checked_directory(state / ".recovery-controls")
            context_directory = controls / operation
            context_directory.mkdir(mode=0o700)
            context: dict[str, object] = {
                "owner": "supervisor-recovery-v1",
                "generation": generation,
                "namespace": namespace,
                "database": database,
                "operation": operation,
                "unknown_operation": unknown_operation,
                "outcome": "incomplete",
            }
            write_json(context_directory / "context.json", context)
            key = "supervisor_recovery_probe:" + operation
            unknown_key = "supervisor_recovery_probe:" + unknown_operation
            acknowledged = {"operation": operation, "payload": "acknowledged-v1"}
            unknown = {"operation": unknown_operation, "payload": "unknown-v1"}
            try:
                stop(state, config)
                reset_failure_window(state)
                start(state, config, validation=True, deadline=deadline)
                provisioned = administrative_query(
                    state,
                    f"DEFINE NAMESPACE IF NOT EXISTS `{namespace}`; USE NS `{namespace}`; DEFINE DATABASE `{database}`;",
                    deadline,
                    namespace=namespace,
                    database=database,
                    select_context=False,
                )
                # Released 3.3 USE returns the selected context; DEFINE returns NONE.
                if (
                    len(provisioned) != 3
                    or any(
                        row.get("status") != "OK" or "result" not in row
                        for row in provisioned
                    )
                    or [row["result"] for row in provisioned]
                    != [None, {"namespace": namespace, "database": None}, None]
                ):
                    raise SupervisorError(
                        "Recovery context provisioning lacked complete acknowledgments"
                    )
                results = administrative_query(
                    state,
                    f"UPSERT {key} CONTENT {json.dumps(acknowledged)}; SELECT operation,payload FROM ONLY {key};",  # noqa: S608 -- local UUID records and JSON-encoded payload
                    deadline,
                    namespace=namespace,
                    database=database,
                )
                if len(results) != 2 or results[1].get("result") != acknowledged:
                    raise SupervisorError(
                        "Recovery acknowledged receipt did not match its original identity"
                    )
                abandon_administrative_response(
                    state,
                    f"UPSERT {unknown_key} CONTENT {json.dumps(unknown)};",
                    deadline,
                    namespace=namespace,
                    database=database,
                )
                stop(state, config, abrupt=True)
                reset_failure_window(state)
                start(state, config, validation=True, deadline=deadline)
                readback = administrative_query(
                    state,
                    f"SELECT operation,payload FROM ONLY {key}; SELECT operation,payload FROM ONLY {unknown_key};",  # noqa: S608 -- identifiers are local generated UUID records
                    deadline,
                    namespace=namespace,
                    database=database,
                )
                if (
                    len(readback) != 2
                    or readback[0].get("result") != acknowledged
                    or readback[1].get("result") not in (None, unknown)
                ):
                    raise SupervisorError(
                        "Recovery lost the acknowledged original operation or returned conflicting bytes"
                    )
                # Old process-group drain plus reopen bounds settlement of this
                # exact probe. No absent-ack replay or scientific claim occurs.
                recovered = readback[1]["result"]
                context.update(
                    outcome="settled",
                    acknowledged_survived=True,
                    unknown_outcome="committed"
                    if recovered == unknown
                    else "not_committed",
                    retirement="pending",
                )
                write_json(context_directory / "context.json", context)
                # Only this newly generated, settled probe is disposable. Leave
                # historical/incomplete contexts intact and do not select the
                # removed database in the administrative request's session.
                retired = administrative_query(
                    state,
                    f"USE NS `{namespace}`; REMOVE DATABASE `{database}`; INFO FOR NS;",
                    deadline,
                    namespace=namespace,
                    database=database,
                    select_context=False,
                )
                if (
                    len(retired) != 3
                    or any(
                        row.get("status") != "OK" or "result" not in row
                        for row in retired
                    )
                    or retired[0]["result"]
                    != {"namespace": namespace, "database": None}
                    or retired[1]["result"] is not None
                ):
                    raise SupervisorError(
                        "Recovery probe retirement lacked complete acknowledgments"
                    )
                catalog = object_mapping(retired[2]["result"])
                if database in object_mapping(catalog.get("databases")):
                    raise SupervisorError(
                        "Recovery probe database remains addressable after retirement"
                    )
                context["retirement"] = "complete"
                write_json(context_directory / "context.json", context)
            finally:
                cleanup_prior = _STARTUP.deadline
                _STARTUP.deadline = time.monotonic() + 90
                try:
                    stop(state, config)
                finally:
                    _STARTUP.deadline = cleanup_prior
            remaining(deadline)
            if recovery_generation(state, config_for(state)) != generation:
                raise SupervisorError(
                    "Service generation changed during recovery qualification"
                )
            write_json(
                state / "recovery-qualification.json",
                {
                    "version": 1,
                    "outcome": "passed",
                    "generation": generation,
                    "operation": operation,
                    "unknown_operation": unknown_operation,
                    "administrative_context": {
                        "namespace": namespace,
                        "database": database,
                        "evidence": str(context_directory / "context.json"),
                        "retired": True,
                    },
                    "acknowledged_survived": True,
                    "unknown_outcome": "committed"
                    if recovered == unknown
                    else "not_committed",
                    "scope": "process-crash/reopen; administrative fixed probe; no scientific or host-crash qualification",
                },
            )
            context["outcome"] = "passed"
            write_json(context_directory / "context.json", context)
            config.update(config_for(state))
            config["restart_qualified"] = True
            write_json(state / "config.json", config)
            materialize_service(state, config)
            reset_failure_window(state)
            return public_status(state, config)
    finally:
        _STARTUP.deadline = prior


def private_record_bytes(
    path: Path, deadline: float | None, *, maximum_bytes: int | None = None
) -> bytes:
    """Read an owned regular record within its clock and the caller's size policy."""
    if deadline is not None:
        remaining(deadline)
    descriptor = os.open(
        path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    )
    with os.fdopen(descriptor, "rb") as stream:
        information = os.fstat(stream.fileno())
        if (
            not stat.S_ISREG(information.st_mode)
            or information.st_uid != os.getuid()
            or stat.S_IMODE(information.st_mode) != 0o600
        ):
            raise SupervisorError("Owned private record is unavailable")
        if maximum_bytes is not None and information.st_size > maximum_bytes:
            raise SupervisorError("Private record exceeds its bound")
        raw = bytearray()
        while chunk := stream.read(64 * 1024):
            if deadline is not None:
                remaining(deadline)
            if maximum_bytes is not None and len(raw) + len(chunk) > maximum_bytes:
                raise SupervisorError("Private record exceeds its bound")
            raw.extend(chunk)
    if deadline is not None:
        remaining(deadline)
    return bytes(raw)


def mcp_private_record(path: Path, deadline: float) -> tuple[dict[str, object], str]:
    """Read a bounded MCP credential/protocol record under the owned-file contract."""
    raw = private_record_bytes(path, deadline, maximum_bytes=64 * 1024)
    value = object_mapping(json.loads(raw))
    return value, hashlib.sha256(raw).hexdigest()


@dataclass(frozen=True, repr=False)
class SelectedService:
    """Frozen live owner; credentials stay internal to fixed endpoint consumers."""

    state: Path
    port: int
    _config: dict[str, object] = field(repr=False)
    _credential_digest: str = field(repr=False)
    _proof_digest: str = field(repr=False)
    _invocation: str = field(repr=False)
    _password: str = field(repr=False)
    _telemetry_service_instance_id: str | None = None

    @property
    def identity(self) -> dict[str, object]:
        return {
            "state": str(self.state),
            "instance_id": self._config["instance_id"],
            "telemetry_service_instance_id": self._telemetry_service_instance_id,
            "invocation": self._invocation,
            "port": self.port,
            "binary_sha256": object_mapping(self._config["server"]).get(
                "binary_sha256"
            ),
        }

    @property
    def selection_fence(self) -> str:
        """Nonsecret exact-generation fence for a later fixed header helper."""
        selected = {
            "identity": self.identity,
            "credentials": self._credential_digest,
            "proof": self._proof_digest,
        }
        return hashlib.sha256(
            json.dumps(selected, sort_keys=True, separators=(",", ":")).encode()
        ).hexdigest()

    def recheck(self, deadline: float) -> None:
        if (
            config_for(self.state, deadline=deadline) != self._config
            or mcp_private_record(self.state / "credentials.json", deadline)[1]
            != self._credential_digest
            or mcp_private_record(self.state / "protocol-readiness.json", deadline)[1]
            != self._proof_digest
            or storage_unit_observation(self.state, deadline=deadline).get(
                "InvocationID"
            )
            != self._invocation
            or not owns_listener(self.state, self._config, deadline=deadline)
        ):
            raise SupervisorError("Selected service changed during observation")
        remaining(deadline)

    def authorization(self) -> dict[str, str]:
        token = base64.b64encode(f"pse-selection:{self._password}".encode()).decode()
        return {"Authorization": "Basic " + token}


def selected_service(state: Path, deadline: float) -> SelectedService:
    """Validate the existing selected listener, proof and root VIEWER identity; no effects."""
    remaining(deadline)
    state = lexical_absolute(state)
    if any(path.is_symlink() for path in (state, *state.parents)):
        raise SupervisorError("MCP state must not traverse links")
    information = state.stat()
    if (
        not stat.S_ISDIR(information.st_mode)
        or information.st_uid != os.getuid()
        or stat.S_IMODE(information.st_mode) != 0o700
    ):
        raise SupervisorError("MCP state must already be private and owned")
    config = config_for(state, deadline=deadline)
    if (
        config.get("service_state") is not None
        or config.get("credentials_file") != str(state / "credentials.json")
        or not owns_listener(state, config, deadline=deadline)
    ):
        raise SupervisorError("MCP endpoint does not match its live owned service")
    credentials, credential_digest = mcp_private_record(
        state / "credentials.json", deadline
    )
    proof, proof_digest = mcp_private_record(
        state / "protocol-readiness.json", deadline
    )
    observed = storage_unit_observation(state, deadline=deadline)
    if (
        proof.get("schema") != "native-grpc-readiness-v1"
        or proof.get("instance_id") != config["instance_id"]
        or proof.get("invocation") != observed.get("InvocationID")
        or proof.get("binary_sha256")
        != object_mapping(config["server"]).get("binary_sha256")
        or proof.get("credentials_sha256") != credential_digest
        or proof.get("study_module_sha256") != study_module(config)["sha256"]
        or proof.get("study_module_qualification_sha256")
        != (
            None
            if qualification_module(config) is None
            else object_mapping(config["study_module_qualification"])["sha256"]
        )
        or credentials.get("selection_username") != "pse-selection"
        or not isinstance(credentials.get("selection_password"), str)
        or not credentials["selection_password"]
    ):
        raise SupervisorError("MCP VIEWER readiness identity is unavailable")
    process_path = state / "server-process.json"
    telemetry_identity = (
        read_json(process_path, deadline=deadline).get("telemetry_service_instance_id")
        if process_path.is_file()
        else None
    )
    if telemetry_identity is not None and (
        not isinstance(telemetry_identity, str)
        or not re.fullmatch(r"[a-f0-9]{32}", telemetry_identity)
    ):
        raise SupervisorError("Invalid native telemetry process association")
    selected = SelectedService(
        state,
        integer(config["port"]),
        config,
        credential_digest,
        proof_digest,
        observed.get("InvocationID", ""),
        credentials["selection_password"],
        telemetry_identity,
    )
    selected.recheck(deadline)
    return selected


def mcp_authorization(
    state: Path,
    endpoint: str,
    deadline: float,
    *,
    expected_selection: str | None = None,
) -> dict[str, str]:
    """Release a header only for the exact selected native MCP endpoint."""
    selected = selected_service(state, deadline)
    if endpoint != f"http://127.0.0.1:{selected.port}/mcp":
        raise SupervisorError("MCP endpoint does not match its live owned service")
    if expected_selection is not None and (
        not re.fullmatch(r"[a-f0-9]{64}", expected_selection)
        or not secrets.compare_digest(selected.selection_fence, expected_selection)
    ):
        raise SupervisorError("MCP selected generation changed before authentication")
    return selected.authorization()


MX_METRIC_PREFIX = "surrealdb_storage_surrealmx_"
MX_METRIC_FIELDS = (
    "live_keys",
    "live_versions",
    "approx_data_bytes",
    "active_readers",
    "oldest_pinned_version",
    "gc_candidate_keys",
    "commit_ring_occupancy",
    "async_queue_depth",
    "async_queue_bytes",
    "async_dropped_batches",
    "poisoned",
)
METRIC_FAMILIES = ("surrealdb_process_", "surrealdb_query_", "surrealdb_transaction_")


def selected_metric_values(exposition: str) -> dict[str, object]:
    """Keep selected native metadata/series; distinguish missing samples from zero."""
    families: dict[str, dict[str, object]] = {
        MX_METRIC_PREFIX + field: {"availability": "unregistered", "samples": []}
        for field in MX_METRIC_FIELDS
    }
    for line in exposition.splitlines():
        if line.startswith(("# HELP ", "# TYPE ")):
            kind, _, rest = line[2:].partition(" ")
            name, _, description = rest.partition(" ")
            if name not in families and not name.startswith(METRIC_FAMILIES):
                continue
            entry = families.setdefault(name, {"samples": []})
            entry[kind.lower()] = description
            entry.setdefault("availability", "registered_no_sample")
            if entry["availability"] == "unregistered":
                entry["availability"] = "registered_no_sample"
        elif line and not line.startswith("#"):
            match = re.fullmatch(
                r"([a-zA-Z_:][a-zA-Z0-9_:]*)(\{.*\})?\s+([^\s]+)(?:\s+[0-9]+)?", line
            )
            if match is None:
                if line.startswith((MX_METRIC_PREFIX, *METRIC_FAMILIES)):
                    raise SupervisorError(
                        "Selected native metric exposition is malformed"
                    )
                continue
            name, labels, raw = match.groups()
            if name not in families and not name.startswith(METRIC_FAMILIES):
                continue
            try:
                value = float(raw)
            except ValueError:
                raise SupervisorError(
                    "Selected native metric value is malformed"
                ) from None
            if not math.isfinite(value):
                raise SupervisorError("Selected native metric value is nonfinite")
            if name.startswith(MX_METRIC_PREFIX) and (
                not value.is_integer() or value < 0
            ):
                raise SupervisorError("Native MX gauge is not a nonnegative integer")
            entry = families.setdefault(name, {"samples": []})
            entry["availability"] = "observed"
            samples = entry["samples"]
            if not isinstance(samples, list):
                raise SupervisorError("Selected metric series is malformed")
            samples.append(
                {
                    "labels": labels or "",
                    "value": int(raw) if raw.isdecimal() else value,
                }
            )
    return {
        "families": families,
        "oldest_pinned_version_absence": "normal when no reader; absence alone does not establish reader state",
    }


def metric_snapshot(state: Path, deadline: float) -> dict[str, object]:
    """One root VIEWER request to the frozen loopback /metrics listener; no redirects."""
    selected = selected_service(state, deadline)
    started = time.time_ns()
    connection = http.client.HTTPConnection(
        "127.0.0.1", selected.port, timeout=remaining(deadline)
    )
    try:
        connection.request("GET", "/metrics", headers=selected.authorization())
        response = connection.getresponse()
        if response.status != 200:
            raise SupervisorError(
                "Selected native metrics request/authentication failed"
            )
        raw = bytearray()
        while True:
            timeout = remaining(deadline)
            if connection.sock is not None:
                connection.sock.settimeout(timeout)
            block = response.read1(min(64 << 10, (4 << 20) + 1 - len(raw)))
            remaining(deadline)
            if not block:
                break
            raw.extend(block)
            if len(raw) > 4 << 20:
                raise SupervisorError(
                    "Selected native metrics exceeds its response bound"
                )
        values = selected_metric_values(raw.decode())
    except (OSError, UnicodeError, http.client.HTTPException):
        raise SupervisorError("Selected native metrics request/parse failed") from None
    finally:
        connection.close()
    selected.recheck(deadline)
    return {
        "schema": "selected-native-metrics-v1",
        "identity": selected.identity,
        "started_unix_ns": started,
        "finished_unix_ns": time.time_ns(),
        **values,
        "scope": "shared server observation; no exclusive operation attribution; estimated data bytes is not RSS or disk; queue zero does not establish synchronous durability",
    }


def compare_metric_snapshots(
    before: dict[str, object], after: dict[str, object]
) -> dict[str, object]:
    """Compare like identities and series; gauge differences remain gauge differences."""
    if before.get("identity") != after.get("identity"):
        raise SupervisorError("Selected service changed between metric snapshots")
    changes: dict[str, object] = {}
    left, right = object_mapping(before["families"]), object_mapping(after["families"])
    for name in sorted(left.keys() | right.keys()):
        first, last = (
            object_mapping(left.get(name, {})),
            object_mapping(right.get(name, {})),
        )
        if first.get("samples") != last.get("samples"):
            changes[name] = {
                "type": last.get("type", first.get("type", "unknown")),
                "before": first,
                "after": last,
            }
    return {
        "identity": after["identity"],
        "changes": changes,
        "scope": "shared service; gauge changes are not counter deltas and do not establish operation attribution",
    }


def mcp_headers_main(arguments: list[str]) -> int:
    """Fixed, admitted control path with one eight-second clock and redacted errors."""
    deadline = time.monotonic() + 8
    child = arguments[0] == "_mcp-headers-child"
    command = argparse.ArgumentParser(prog="surreal-server mcp-headers")
    command.add_argument("--state", type=Path, required=True)
    command.add_argument("--expected-endpoint", required=True)
    command.add_argument("--expected-selection")
    if child:
        command.add_argument("--deadline", type=float, required=True)
    args = command.parse_args(arguments[1:])
    if not re.fullmatch(r"http://127\.0\.0\.1:[0-9]{1,5}/mcp", args.expected_endpoint):
        command.error(
            "--expected-endpoint must be an explicit native loopback /mcp endpoint"
        )
    try:
        from scripts import pse_env  # noqa: PLC0415 -- fixed control environment owner

        if child:
            if not math.isfinite(args.deadline) or args.deadline > deadline:
                raise SupervisorError("Invalid fixed helper clock")  # noqa: TRY301 -- fixed boundary redacts every failure
            deadline = args.deadline
            remaining(deadline)
            try:
                owner = host_admission.inherit(
                    os.environ, deadline=deadline, services=ServiceAdmission()
                )
            except host_admission.AdmissionError:
                owner = host_admission.verify_light_control_child(
                    os.environ, deadline=deadline
                )
            if owner is None:
                raise SupervisorError("Missing admitted fixed helper ownership")  # noqa: TRY301 -- fixed boundary redacts every failure
            headers = mcp_authorization(
                args.state,
                args.expected_endpoint,
                deadline,
                expected_selection=args.expected_selection,
            )
        else:
            environment = pse_env.control_environment(
                SCRIPT.parents[1], os.environ, deadline=deadline
            )
            result = pse_env.run_light_control(
                [
                    sys.executable,
                    str(SCRIPT),
                    "_mcp-headers-child",
                    "--state",
                    str(args.state),
                    "--expected-endpoint",
                    args.expected_endpoint,
                    "--deadline",
                    str(deadline),
                    *(
                        ["--expected-selection", args.expected_selection]
                        if args.expected_selection is not None
                        else []
                    ),
                ],
                environment,
                deadline=deadline,
            )
            remaining(deadline)
            if result.returncode:
                raise SupervisorError("Fixed helper child failed")  # noqa: TRY301 -- fixed boundary redacts every failure
            headers = json.loads(result.stdout)
            if (
                not isinstance(headers, dict)
                or set(headers) != {"Authorization"}
                or not isinstance(headers["Authorization"], str)
                or not headers["Authorization"].startswith("Basic ")
                or not base64.b64decode(headers["Authorization"][6:], validate=True)
                .decode()
                .startswith("pse-selection:")
            ):
                raise SupervisorError("Invalid fixed helper output")  # noqa: TRY301 -- fixed boundary redacts every failure
    except Exception:
        # Child output, HTTP errors and JSON parse errors can contain credentials.
        # Failure must never echo any of them or return partial headers.
        print(
            "pse-env: MCP credentials unavailable; verify the existing owned service and private VIEWER readiness",
            file=sys.stderr,
        )
        return 125
    print(json.dumps(headers))
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument(
        "command",
        choices=[
            "setup",
            "build-server",
            "module-qualification",
            "initialize",
            "fresh-engine",
            "readmit",
            "ensure",
            "recover",
            "qualify-recovery",
            "drain",
            "reconfigure",
            "start",
            "status",
            "quiesce",
            "stop",
            "kill",
            "backup",
            "inspect-backup",
            "restore",
            "rebuild",
            "recover-maintenance",
            "recover-file-owners",
            "validate",
            "worker",
            "ensure-primary",
            "observe-primary",
            "observer",
            "_serve",
        ],
    )
    result.add_argument(
        "--state",
        type=Path,
        default=Path(
            os.environ.get(
                "PSE_SURREAL_STATE",
                str(
                    Path(
                        os.environ.get(
                            "XDG_STATE_HOME", str(Path.home() / ".local/state")
                        )
                    )
                    / "pse-arrow/surreal-functional-v2"
                ),
            )
        ),
    )
    result.add_argument(
        "--primary-owner", help="Exact owner returned by ensure-primary"
    )
    result.add_argument(
        "--tool-root",
        type=Path,
    )
    result.add_argument(
        "--version", help="Pinned source release v3.3.4; other versions are refused"
    )
    result.add_argument("--port", type=int)
    result.add_argument(
        "--tooling-fixture",
        action="store_true",
        help="Disposable beta tooling profile: permit one additional cloned SDK session per socket",
    )
    result.add_argument(
        "--resident",
        action=argparse.BooleanOptionalAction,
        default=None,
        help="Resident services may be parked and resumed across host lanes; disable for owned disposable controls",
    )
    result.add_argument("--memory-mib", type=int)
    result.add_argument("--server-memory-mib", type=int)
    result.add_argument("--native-workers", type=int)
    result.add_argument("--native-worker-memory-mib", type=int)
    result.add_argument(
        "--execution-profile",
        choices=[
            profile.value
            for profile in deployment.ProductionProfileName
            if profile is not deployment.ProductionProfileName.SERVICE
        ],
    )
    result.add_argument("--worker-executable", type=Path)
    result.add_argument(
        "--payload-verifier",
        type=Path,
        help="Exact built native canonical payload verifier artifact; never builds implicitly",
    )
    result.add_argument("--observer-pid", type=int)
    result.add_argument("--canonical-database")
    result.add_argument(
        "--qualification-native-entry",
        type=Path,
        help="Explicit private native-entry control for a qualification-feature worker",
    )
    result.add_argument("--interpretation")
    result.add_argument(
        "--drained",
        action="store_true",
        help="Caller has quiesced admission and drained managed native workers",
    )
    result.add_argument("--destination", type=Path)
    result.add_argument("--source", type=Path)
    result.add_argument(
        "--module-receipt",
        type=Path,
        help="Immutable qualification module receipt; omit to restore production-only module access",
    )
    result.add_argument(
        "--long-hold",
        action="store_true",
        help="Explicitly select the separate long-hold module qualification receipt",
    )
    result.add_argument("--initializer-command", nargs=argparse.REMAINDER)
    result.add_argument("--check-command", nargs=argparse.REMAINDER)
    result.add_argument("--worker-command", nargs=argparse.REMAINDER)
    result.add_argument("--worker-capabilities", default=",".join(WORKER_CAPABILITIES))
    result.add_argument("--observer-command", nargs=argparse.REMAINDER)
    return result


def dispatch(args: argparse.Namespace) -> int:
    # These controls already own these budgets in their implementation. Capture
    # them before dispatch's reservation, rather than after metadata acquisition.
    budget = {"stop": 90, "kill": 90, "recover": 40, "qualify-recovery": 90}.get(
        args.command
    )
    previous = getattr(_STARTUP, "deadline", None)
    if budget is not None:
        deadline = time.monotonic() + budget
        _STARTUP.deadline = deadline if previous is None else min(previous, deadline)
    try:
        return _dispatch(args)
    finally:
        _STARTUP.deadline = previous


def _dispatch(args: argparse.Namespace) -> int:
    if args.long_hold and args.command != "module-qualification":
        raise SupervisorError("Long-hold is only available with module-qualification")
    if args.qualification_native_entry is not None and args.command != "ensure-primary":
        raise SupervisorError(
            "Qualification control is only available with ensure-primary"
        )
    if args.command == "build-server":
        output = install(
            args.version, args.tool_root or default_tool_root(), build=True
        )
    elif args.command == "module-qualification":
        output = select_module_qualification(
            args.state.absolute(), args.module_receipt, long_hold=args.long_hold
        )
    elif args.command == "initialize":
        output = initialize(args.state.absolute(), args.initializer_command or [])
    elif args.command == "fresh-engine":
        if not args.drained:
            raise SupervisorError(
                "fresh-engine requires --drained plus actual owned process drain"
            )
        output = fresh_engine(args.state.absolute(), args.tool_root)
    elif args.command == "setup":
        output = setup(args)
    elif args.command == "rebuild":
        if args.source is not None or args.destination is not None:
            raise SupervisorError(
                "rebuild regenerates current state; preservation source/destination are unsupported"
            )
        output = rebuild(args.state.absolute(), args.initializer_command or [])
    elif args.command == "recover-maintenance":
        output = recover_maintenance(
            args.state.absolute(), args.initializer_command or [], args.payload_verifier
        )
    elif args.command == "inspect-backup":
        if args.source is None:
            raise SupervisorError("inspect-backup requires --source")
        output = inspect_backup(
            args.source.absolute(),
            args.interpretation or SUBSTRATE_INTERPRETATION,
            args.payload_verifier,
        )
    elif args.command == "restore":
        if args.source is None:
            raise SupervisorError("restore requires --source")
        output = restore(
            args.source.absolute(),
            args.state.absolute(),
            args.interpretation or "",
            args.payload_verifier,
        )
    elif args.command == "worker":
        return worker(
            args.state.absolute(),
            args.worker_command or [],
            capabilities=tuple(
                value for value in args.worker_capabilities.split(",") if value
            ),
        )
    elif args.command == "observer":
        profile = args.execution_profile
        return observer(
            args.state.absolute(), args.observer_command or [], profile=profile
        )
    elif args.command == "ensure-primary":
        output = ensure_primary(
            args.state.absolute(),
            args.observer_pid,
            args.canonical_database,
            args.qualification_native_entry,
        )
    elif args.command == "observe-primary":
        if args.primary_owner is None or len(args.primary_owner) > 4096:
            raise SupervisorError(
                "Primary observation requires a bounded admitted owner"
            )
        owner = json.loads(args.primary_owner)
        if not isinstance(owner, dict):
            raise SupervisorError("Primary observation owner must be an object")
        output = observe_primary(args.state.absolute(), args.canonical_database, owner)
    elif args.command == "_serve":
        return serve(args.state.absolute())
    else:
        state = args.state.absolute()
        config_for(
            state
        )  # Reject unowned state before changing permissions or writing a lock.
        state = checked_directory(state)
        with (
            contextlib.nullcontext()
            if args.command in {"status", "start", "ensure"}
            else lifecycle_reservation(
                state,
                purpose=(
                    "stop"
                    if args.command in {"stop", "kill"}
                    else "startup"
                    if args.command == "recover"
                    else "maintenance"
                ),
            )
        ):
            config = config_for(state)
            if args.command == "readmit":
                if any(
                    value is not None
                    for value in (
                        args.memory_mib,
                        args.server_memory_mib,
                        args.native_workers,
                        args.native_worker_memory_mib,
                        args.port,
                        args.version,
                        args.interpretation,
                        args.tool_root,
                    )
                ):
                    raise SupervisorError(
                        "Readmission preserves the selected allocation and state identity"
                    )
                output = readmit_supervisor(
                    state,
                    execution_profile=args.execution_profile,
                    worker_executable=args.worker_executable,
                    verifier=args.payload_verifier,
                )
                print(json.dumps(output, sort_keys=True))
                return 0
            if args.payload_verifier is not None:
                if args.command not in {"backup", "recover-maintenance"}:
                    raise SupervisorError(
                        "Verifier selection is only available with setup, backup, restore or recover-maintenance"
                    )
                set_payload_verifier(state, config, args.payload_verifier)
            if args.command == "recover-file-owners":
                output = recover_file_owners(state)
                print(json.dumps(output, sort_keys=True))
                return 0
            if args.command == "qualify-recovery":
                output = qualify_recovery(state)
                print(json.dumps(output, sort_keys=True))
                return 0
            if args.command in {"start", "ensure", "recover"}:
                if args.command == "recover":
                    reset_failure_window(state)
                start(state, config)
            elif args.command == "reconfigure":
                reconfigure(state, config, args)
            elif args.command in {"quiesce", "drain"}:
                close_context_admission(state, admission="quiescing")
                config.update(config_for(state))
                for context in managed_contexts(state):
                    selected = config_for(context)
                    allocation = selected["resources"]
                    if (
                        not isinstance(allocation, dict)
                        or "execution" not in allocation
                    ):
                        continue
                    observed = primary_observation(context)
                    pid = primary_drain_pid(context, selected, observed)
                    if pid is not None:
                        # Signal the actual receiver only. The installation wrapper's
                        # cancellation path kills its scope, which is recovery, not
                        # cooperative native drain.
                        descriptor = os.pidfd_open(pid)
                        try:
                            if (
                                primary_drain_pid(
                                    context, selected, primary_observation(context)
                                )
                                == pid
                            ):
                                signal.pidfd_send_signal(descriptor, signal.SIGINT)
                        finally:
                            os.close(descriptor)
            elif args.command in {"stop", "kill", "backup"}:
                if args.command != "kill" and not args.drained:
                    raise SupervisorError(
                        "Quiesce admission and drain managed workers, then supply --drained"
                    )
                if args.command == "backup":
                    if args.destination is None:
                        raise SupervisorError("backup requires --destination")
                    output = backup(state, config, args.destination)
                    print(json.dumps(output, sort_keys=True))
                    return 0
                # An operator stop/kill withdraws coordinated resume intent. The
                # internal parking path still calls stop directly and retains it.
                # Clear the service premise before the host queue so a concurrent
                # reconciler's old snapshot cannot resurrect this explicit stop.
                service = service_directory(state)
                with state_lock(service):
                    selected = config_for(service)
                    selected["parked"] = False
                    write_json(service / "config.json", selected)
                host_admission.withdraw_parked_service(service)
                config.update(config_for(state))
                stop(state, config, abrupt=args.command == "kill")
            elif args.command == "validate":
                output = validate(
                    state, config, args.interpretation or "", args.check_command or []
                )
                print(json.dumps(output, sort_keys=True))
                return 0
            output = public_status(state, config)
    print(json.dumps(output, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    # Fixed credential retrieval must never reach general lifecycle dispatch.
    if arguments and arguments[0] in {"mcp-headers", "_mcp-headers-child"}:
        return mcp_headers_main(arguments)
    args = parser().parse_args(arguments)
    try:
        return dispatch(args)
    except (
        SupervisorError,
        OSError,
        ValueError,
        KeyError,
        subprocess.SubprocessError,
    ) as error:
        # Raw subprocess errors may contain secret-bearing arguments from a validator.
        message = (
            str(error)
            if isinstance(error, SupervisorError)
            else f"{type(error).__name__}: supervisor operation failed"
        )
        print(message, file=sys.stderr)
        for note in getattr(error, "__notes__", ()):
            # Only these supervisor-authored notes contain safe owner metadata;
            # notes from arbitrary validators may include secret-bearing text.
            if note.startswith(
                ("Worker cleanup unresolved;", "Primary cleanup unresolved;")
            ):
                print(note, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
