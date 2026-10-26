# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native operation observation and terminal ownership without setup composition."""

from __future__ import annotations

import json
import os
import re
import subprocess
import time
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, TypedDict

from scripts import process_observation, resource_store
from scripts.process_observation import populated, process_group, start_identity

if TYPE_CHECKING:
    from collections.abc import Mapping


VERSION = 2


def caller_alive(record: Mapping[str, object]) -> bool:
    if record.get("boot") != process_observation.boot():
        return False
    pid = record.get("pid")
    if type(pid) is not int:
        raise ValueError("Invalid terminal owner PID")
    try:
        return start_identity(pid) == record["start"]
    except FileNotFoundError:
        return False


class DeadlineOptions(TypedDict, total=False):
    """Forward an existing clock only when its caller supplied one."""

    deadline: float



def deadline_options(deadline: float | None) -> DeadlineOptions:
    return {"deadline": deadline} if deadline is not None else {}



class ScopeOwner(TypedDict):
    unit: str
    group: str
    invocation: str



def write_json(path: Path, value: Mapping[str, object]) -> None:
    temporary = path.with_suffix(f".{os.getpid()}.tmp")
    descriptor = os.open(
        temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600
    )
    with os.fdopen(descriptor, "w") as output:
        output.write(json.dumps(value, sort_keys=True) + "\n")
    temporary.chmod(0o600)
    temporary.replace(path)



class DeadlineExpiredError(ValueError):
    """An offered absolute native admission/control clock expired."""



def admission_remaining(deadline: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise DeadlineExpiredError("native operation original admission clock expired")
    return remaining



def manager_control_environment() -> dict[str, str]:
    # The foreground observer can run outside the composed child environment.
    # Observation, stop and mandatory kill must address the same user manager.
    from scripts import systemd_environment

    return {**os.environ, **systemd_environment.manager_environment()}



def _manager_detail(value: str | bytes | None, limit: int) -> str:
    raw = value.encode() if isinstance(value, str) else value or b""
    suffix = " [truncated]" if len(raw) > limit else ""
    return raw[:limit].decode(errors="replace").strip() + suffix



def _manager_endpoint(environment: Mapping[str, str]) -> str:
    return ", ".join(
        f"{name}={_manager_detail(environment.get(name), 256)}"
        for name in ("XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS")
    )



class ManagerObservationTimeout(subprocess.TimeoutExpired):
    """A manager timeout retaining its selected endpoint and bounded diagnostics."""

    def __init__(self, error: subprocess.TimeoutExpired, context: str) -> None:
        super().__init__(
            error.cmd, error.timeout, stderr=_manager_detail(error.stderr, 4096)
        )
        self.context = context

    def __str__(self) -> str:
        return (
            f"{super().__str__()} ({self.context}); stderr: {self.stderr or '(empty)'}"
        )



def manager_observation(*arguments: str, deadline: float | None = None) -> str:
    """Read the selected user manager without preparation or execution admission."""
    environment = manager_control_environment()
    context = _manager_endpoint(environment)
    timeout = 10 if deadline is None else min(10, admission_remaining(deadline))
    try:
        result = subprocess.run(
            ["systemctl", "--user", *arguments],
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=environment,
        )
    except subprocess.TimeoutExpired as error:
        if deadline is not None and time.monotonic() >= deadline:
            raise DeadlineExpiredError(
                "native operation original observation clock expired"
            ) from error
        raise ManagerObservationTimeout(error, context) from error
    except OSError as error:
        raise ValueError(
            f"systemctl {arguments[0]} unavailable ({context}): {type(error).__name__}: {error}"
        ) from error
    if deadline is not None:
        admission_remaining(deadline)
    if result.returncode:
        detail = _manager_detail(result.stderr, 4096) or "(empty)"
        raise ValueError(
            f"systemctl {arguments[0]} failed ({result.returncode}; {context}); stderr: {detail}"
        )
    return result.stdout



def unit_observation(
    unit: str, *, deadline: float | None = None, properties: tuple[str, ...] = ()
) -> dict[str, str]:
    requested = tuple(
        dict.fromkeys(
            ("InvocationID", "ControlGroup", "LoadState", "ActiveState", *properties)
        )
    )
    value = manager_observation(
        "show", *(f"--property={key}" for key in requested), unit, deadline=deadline
    )
    context = _manager_endpoint(manager_control_environment())
    fields: dict[str, str] = {}
    for line in value.splitlines():
        if "=" not in line:
            raise ValueError(f"malformed systemctl properties for {unit} ({context})")
        key, observed = line.split("=", 1)
        if key in fields:
            raise ValueError(
                f"duplicate systemctl property {key} for {unit} ({context})"
            )
        fields[key] = observed
    if any(key not in fields for key in requested) or not all(
        fields.get(key) for key in ("LoadState", "ActiveState")
    ):
        raise ValueError(f"incomplete systemctl properties for {unit} ({context})")
    return fields



def scope_owner(*, deadline: float | None = None) -> ScopeOwner | None:
    """Only real scope membership and its invocation grant operation reuse."""
    if deadline is not None:
        admission_remaining(deadline)
    try:
        group = process_group(os.getpid())
        unit = next(
            part
            for part in Path(group).parts
            if re.fullmatch(
                r"(?:pse-native-[a-f0-9]{32}\.scope|pse-surreal-worker-[a-f0-9]{16}-[0-9]+\.(?:scope|service))",
                part,
            )
        )
        observed = unit_observation(unit, **deadline_options(deadline))
        control = observed["ControlGroup"]
        invocation = observed["InvocationID"]
        if (
            observed["LoadState"] != "loaded"
            or not re.fullmatch(r"[a-f0-9]{32}", invocation)
            or not control.startswith("/")
            or control == "/"
            or ".." in Path(control).parts
            or not (group == control or group.startswith(control + "/"))
        ):
            return None
    except DeadlineExpiredError:
        raise
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError):
        if deadline is not None:
            admission_remaining(deadline)
        return None

    else:
        if deadline is not None:
            admission_remaining(deadline)
        return {"unit": unit, "group": control, "invocation": invocation}



class DrainPhase(Enum):
    COMPLETE = "complete"
    POPULATED = "populated"
    ACTIVE = "active"
    MISMATCH = "mismatch"
    UNAVAILABLE = "unavailable"



@dataclass(frozen=True)
class DrainObservation:
    owner: Mapping[str, object] | None
    phase: DrainPhase
    cause: str
    observed: Mapping[str, str] | None = None

    @property
    def complete(self) -> bool:
        return self.phase is DrainPhase.COMPLETE

    def diagnostic(self) -> str:
        return f"native drain {self.phase.value}: {self.cause}; selected={self.owner}; observed={self.observed}"

    def __bool__(self) -> bool:
        raise TypeError("Consume DrainObservation.complete or its phase explicitly")



def drained(
    record: Mapping[str, object], *, deadline: float | None = None
) -> DrainObservation:
    """Observe this native owner; unknown premises keep its obligations pinned."""
    owner = record.get("scope")
    if not isinstance(owner, dict) or any(
        not isinstance(owner.get(key), str) for key in ("unit", "group", "invocation")
    ):
        return DrainObservation(None, DrainPhase.UNAVAILABLE, "missing scope identity")
    selected = dict(owner)
    try:
        if deadline is not None:
            admission_remaining(deadline)
        inode = record.get("scope_inode")
        if type(inode) is int:
            if process_observation.group_identity(owner["group"]) != inode:
                return DrainObservation(
                    selected, DrainPhase.COMPLETE, "bound kernel scope lifetime ended"
                )
        if populated(owner["group"]):
            return DrainObservation(
                selected, DrainPhase.POPULATED, "owned descendants remain"
            )
        observed = unit_observation(owner["unit"], **deadline_options(deadline))
        if observed.get("LoadState") == "not-found":
            # A group can repopulate while the manager is being observed.
            if populated(owner["group"]):
                return DrainObservation(
                    selected,
                    DrainPhase.POPULATED,
                    "group populated during observation",
                    observed,
                )
            return DrainObservation(
                selected,
                DrainPhase.COMPLETE,
                "unit absent and owned group empty",
                observed,
            )
        if observed.get("LoadState") != "loaded" or any(
            key not in observed
            for key in ("InvocationID", "ControlGroup", "ActiveState")
        ):
            return DrainObservation(
                selected,
                DrainPhase.UNAVAILABLE,
                "unit observation lacks required identity or phase",
                observed,
            )
        if observed.get("InvocationID") != owner["invocation"] or observed.get(
            "ControlGroup"
        ) not in {owner["group"], ""}:
            return DrainObservation(
                selected,
                DrainPhase.MISMATCH,
                "unit identity differs from selected owner",
                observed,
            )
        if observed.get("ActiveState") not in {"inactive", "failed"}:
            return DrainObservation(
                selected, DrainPhase.ACTIVE, "owned unit is not terminal", observed
            )
        if populated(owner["group"]):
            return DrainObservation(
                selected,
                DrainPhase.POPULATED,
                "group populated during observation",
                observed,
            )
        return DrainObservation(
            selected,
            DrainPhase.COMPLETE,
            "owned unit terminal and group empty",
            observed,
        )
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        return DrainObservation(
            selected, DrainPhase.UNAVAILABLE, f"{type(error).__name__}: {error}"
        )



def composer_registrations(
    record: Mapping[str, object],
) -> dict[str, dict[str, object]]:
    selected = record.get("selected_composers", {})
    if not isinstance(selected, dict) or any(
        not isinstance(nonce, str)
        or not re.fullmatch(r"[a-f0-9]{32}", nonce)
        or not isinstance(value, dict)
        for nonce, value in selected.items()
    ):
        raise ValueError("invalid selected terminal associations")
    return selected



def terminal_guard_complete(
    nonce: str, guard: object, *, deadline: float | None = None
) -> bool:
    """Authentic terminal ACK or explicit abandoned membership, plus exact ownership.

    Abandonment is a separate disposition and never creates an acknowledgment or
    changes the selected composer's outcome record. Physical drain remains required.
    """
    try:
        if not isinstance(guard, dict) or not re.fullmatch(r"[a-f0-9]{32}", nonce):
            return False
        scope = guard.get("scope")
        if (
            not isinstance(scope, dict)
            or not re.fullmatch(
                r"(?:pse-native-[a-f0-9]{32}\.scope|pse-surreal-worker-[a-f0-9]{16}-[0-9]+\.(?:scope|service))",
                str(scope.get("unit", "")),
            )
            or not isinstance(scope.get("group"), str)
            or not scope["group"].startswith("/")
            or not re.fullmatch(r"[a-f0-9]{32}", str(scope.get("invocation", "")))
            or not re.fullmatch(r"[a-f0-9]{32}", str(guard.get("token", "")))
        ):
            return False
        path = Path(str(guard["operation"]))
        if (
            not path.is_absolute()
            or path != path.resolve()
            or path.parent.name != ".operations"
            or not re.fullmatch(r"[a-f0-9]{32}\.json", path.name)
        ):
            return False
        if guard.get("phase") == "acknowledged":
            # The existing admitted row records only verification of immutable
            # terminal identities, never another copy of outcomes. Temporary
            # evidence retirement cannot erase an acknowledgment already proved.
            return all(
                re.fullmatch(r"[a-f0-9]{64}", str(guard.get(key, ""))) is not None
                for key in ("terminal_identity", "evidence_identity")
            )
        record = resource_store.read(path)
        if record.get("version") != VERSION or record.get("scope") != guard["scope"]:
            return False
        composer = composer_registrations(record).get(nonce)
        if composer is None or composer.get("token") != guard["token"]:
            return False
        registry = Path(str(composer["registry"]))
        invocation = registry / "invocations" / f"{nonce}.json"
        if (
            composer.get("invocation") != str(invocation)
            or invocation != invocation.resolve()
        ):
            return False
        if composer.get("phase") != "complete":
            if composer.get("phase") != "registered":
                return False
            state = resource_store.FileStore(registry).invocation_snapshot(nonce)
            abandonment = state.get("abandonment")
            selected_composer = state.get("composer")
            if isinstance(abandonment, dict):
                return (
                    state["phase"] in {"abandoned", "frozen"}
                    and re.fullmatch(
                        r"[a-f0-9]{32}", str(abandonment.get("identity", ""))
                    )
                    is not None
                    and abandonment.get("selection")
                    == resource_store.digest(state["declaration"]["selected"])
                    and abandonment.get("members") == list(state["members"])
                    and isinstance(selected_composer, dict)
                    and all(
                        selected_composer.get(key) == composer.get(key)
                        for key in ("pid", "start", "scope")
                    )
                    and selected_composer.get("scope") == guard["scope"]
                    and not caller_alive(selected_composer)
                    and drained(
                        {"scope": guard["scope"]}, **deadline_options(deadline)
                    ).complete
                )
            return False
        identity, _terminal = resource_store.FileStore(registry).terminal(
            nonce, deadline=deadline
        )
        evidence = resource_store.read(invocation.with_suffix(".terminal.json"))
        return composer.get("terminal_identity") == identity and composer.get(
            "evidence_identity"
        ) == resource_store.digest(evidence)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError):
        return False

