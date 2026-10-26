# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit quiescent conversion of the existing resource owner, retaining originals.

This is a one-time format transition, never an ordinary reader or writer branch.
Supported workload writers belong to host allocations; exact recorded registry
process/scope owners are checked. Capability-server presence is not registry activity.
Unknown or active registry owners refuse before the pending fence.
After fencing, interrupted work remains protected and is resumed explicitly.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import time
from contextlib import contextmanager
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, TypedDict, cast

from scripts import host_admission as host
from scripts import operation_lifetime, service_borrowers
from scripts import resource_store as files
from scripts import test_resources as resources

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping, Sequence


class ConversionTransition(TypedDict):
    version: int
    phase: str
    original_activation: dict[str, object]
    services: list[str]
    sources: NotRequired[dict[str, str]]


FAMILIES = ("invocations", "invocation-state", "resources")
SOURCE = re.compile(r"(?:invocations|invocation-state|resources)/[a-f0-9]{32}\.json")
DIGEST = re.compile(r"[a-f0-9]{64}")
MAX_BYTES = 512 * 1024**2
MAX_IMAGES = 100_000


@contextmanager
def directory(path: Path) -> Generator[int, None, None]:
    """Anchor every parent with nofollow, including shared outer directories."""
    path = path.absolute()
    if ".." in path.parts:
        raise files.StoreError("Ambiguous original conversion directory")
    descriptor = os.open(path.anchor, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
    try:
        for part in path.parts[1:]:
            following = os.open(
                part,
                os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC,
                dir_fd=descriptor,
            )
            os.close(descriptor)
            descriptor = following
        information = os.fstat(descriptor)
        if information.st_uid != os.getuid() or information.st_mode & 0o077:
            raise files.StoreError("Unsafe original conversion directory")
        yield descriptor
    finally:
        os.close(descriptor)


def read_image(path: Path, remaining: int) -> bytes:
    with directory(path.parent) as parent:
        return read_file(parent, path.name, remaining)


def read_file(parent: int, name: str, remaining: int) -> bytes:
    descriptor = os.open(
        name, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent
    )
    with os.fdopen(descriptor, "rb") as stream:
        host.validate_private_file(
            os.fstat(stream.fileno()), "original conversion image"
        )
        if os.fstat(stream.fileno()).st_size > remaining:
            raise files.StoreError("Explicit conversion byte bound exceeded")
        data = stream.read(remaining + 1)
        if len(data) > remaining:
            raise files.StoreError("Explicit conversion byte bound exceeded")
        return data


def validate_transition(value: Mapping[str, object]) -> None:
    activation = value.get("original_activation")
    if (
        value.get("version") != 1
        or value.get("phase") not in {"pending", "complete"}
        or not isinstance(activation, dict)
        or activation.get("version") != 2
        or activation.get("format") != "exact-resource-files"
        or not isinstance(activation.get("migration"), str)
        or DIGEST.fullmatch(activation["migration"]) is None
    ):
        raise files.StoreError("Unknown conversion recovery authority")
    services = value.get("services")
    if not isinstance(services, list) or any(
        not isinstance(item, str)
        or not Path(item).is_absolute()
        or ".." in Path(item).parts
        or any(parent.is_symlink() for parent in (Path(item), *Path(item).parents))
        for item in services
    ):
        raise files.StoreError("Invalid conversion service authority")
    sources = value.get("sources")
    if "sources" not in value and value["phase"] == "pending":
        return
    if (
        not isinstance(sources, dict)
        or len(sources) > MAX_IMAGES
        or any(
            not isinstance(name, str)
            or SOURCE.fullmatch(name) is None
            or not isinstance(digest, str)
            or DIGEST.fullmatch(digest) is None
            for name, digest in sources.items()
        )
    ):
        raise files.StoreError("Invalid conversion source authority")


def originals(
    root: Path, transition: Mapping[str, object], deadline: float
) -> dict[str, bytes]:
    validate_transition(transition)
    result = {}
    size = 0
    source_images = transition.get("sources")
    if not isinstance(source_images, dict):
        raise files.StoreError("Conversion originals require a bound source population")
    for name, expected in source_images.items():
        host.remaining(deadline, 1)
        data = read_image(root / "original-v2" / name, MAX_BYTES - size)
        if hashlib.sha256(data).hexdigest() != expected:
            raise files.StoreError("Original conversion custody digest mismatch")
        result[name] = data
        size += len(data)
    validate_originals(result)
    return result


def processes(deadline: float, proc: Path = Path("/proc")) -> list[dict]:
    """Read stable same-user identities; unreadable live peers are ambiguous."""
    result = []
    for entry in proc.iterdir():
        host.remaining(deadline, 1)
        if not entry.name.isdecimal():
            continue
        try:
            if entry.stat().st_uid != os.getuid():
                continue
            before = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            if before[0] in {"Z", "X"}:
                continue
            with (entry / "cmdline").open("rb") as stream:
                command = stream.read(1024**2 + 1)
            if len(command) > 1024**2:
                raise files.StoreError("Conversion process command bound exceeded")
            after = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            if before[19] != after[19]:
                raise files.StoreError(
                    "Process identity changed during conversion cohort discovery"
                )
            if not command:
                raise files.StoreError(
                    "Unresolved live process command during conversion cohort discovery"
                )
            result.append(
                {
                    "pid": int(entry.name),
                    "start": before[19],
                    "argv": os.fsdecode(command).rstrip("\0").split("\0"),
                }
            )
        except (FileNotFoundError, ProcessLookupError) as error:
            try:
                survivor = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            except (FileNotFoundError, ProcessLookupError):
                continue
            except (OSError, IndexError, UnicodeError) as unresolved:
                raise files.StoreError(
                    "Unresolved disappearing process during conversion cohort discovery: "
                    + entry.name
                ) from unresolved
            if survivor[0] not in {"Z", "X"}:
                raise files.StoreError(
                    "Unresolved live process during conversion cohort discovery: "
                    + entry.name
                ) from error
        except (OSError, IndexError, UnicodeError) as error:
            raise files.StoreError(
                "Unresolved live process during conversion cohort discovery: "
                + entry.name
            ) from error
        if len(result) > MAX_IMAGES:
            raise files.StoreError("Conversion process discovery bound exceeded")
    return result


def validate_originals(values: dict[str, bytes]) -> None:
    for name, data in values.items():
        value = json.loads(data)
        if not isinstance(value, dict):
            raise files.StoreError("Invalid original conversion image")
        if name.startswith("invocation-state/"):
            header = values.get("invocations/" + Path(name).name)
            if (
                value.get("version") != 2
                or not isinstance(value.get("declaration"), dict)
                or header is None
                or json.loads(header) != value["declaration"]
            ):
                raise files.StoreError(
                    "Original invocation header/state conflict; conversion refused"
                )


def images(root: Path, deadline: float) -> dict[str, bytes]:
    result = {}
    size = 0
    for family in FAMILIES:
        try:
            (root / family).lstat()
        except FileNotFoundError:
            continue
        with directory(root / family) as parent, os.scandir(parent) as entries:
            for entry in entries:
                host.remaining(deadline, 1)
                name = family + "/" + entry.name
                if SOURCE.fullmatch(name) is None:
                    continue  # terminal/capture siblings are not replaced
                if len(result) >= MAX_IMAGES:
                    raise files.StoreError("Explicit conversion image bound exceeded")
                data = read_file(parent, entry.name, MAX_BYTES - size)
                size += len(data)
                result[name] = data
    return result


def quiescent(values: dict[str, bytes], deadline: float) -> None:
    """Establish actual registry-owner cohorts, without payload drain claims."""
    scopes = {}
    snapshot = host.readonly_snapshot(deadline=deadline)
    allocation = host.inherit(os.environ, deadline=deadline)
    own_group = operation_lifetime.process_group(os.getpid())
    for nonce, owner in snapshot["owners"].items():
        # Long-lived admitted store/cache services cannot mutate fixture metadata.
        # Workload scopes can spawn writers even after their coordinator exits.
        for unit, binding in owner["units"].items():
            if unit.endswith(".scope"):
                if (
                    allocation is not None
                    and allocation.nonce == nonce
                    and isinstance(binding.get("group"), str)
                    and (
                        own_group == binding["group"]
                        or own_group.startswith(binding["group"] + "/")
                    )
                ):
                    for peer in processes(deadline):
                        if peer["pid"] == os.getpid():
                            continue
                        try:
                            group = operation_lifetime.process_group(peer["pid"])
                            if (
                                operation_lifetime.start_identity(peer["pid"])
                                != peer["start"]
                            ):
                                raise files.StoreError(
                                    "Process identity changed during conversion scope discovery"
                                )
                        except (FileNotFoundError, ProcessLookupError):
                            continue
                        except (OSError, ValueError) as error:
                            raise files.StoreError(
                                "Unresolved conversion scope peer"
                            ) from error
                        if group == binding["group"] or group.startswith(
                            binding["group"] + "/"
                        ):
                            raise files.StoreError(
                                f"Other live conversion scope member pid={peer['pid']} start={peer['start']}; conversion refused"
                            )
                    continue
                if host.caller_alive(owner):
                    raise files.StoreError(
                        "Active admitted writer cohort; conversion refused"
                    )
                scopes[unit] = {"unit": unit, **binding}
    for name, data in values.items():
        value = json.loads(data)
        facts = value.get("facts", {}) if name.startswith("resources/") else value
        owners = [
            facts.get("producer"),
            facts.get("composer"),
            facts,
            facts.get("creation_borrower"),
            facts.get("cleaner"),
            *facts.get("borrows", {}).values(),
        ]
        for owner in owners:
            if not isinstance(owner, dict):
                continue
            if service_borrowers.borrower_alive(owner):
                raise files.StoreError(
                    "Live exact metadata writer/reader; conversion refused"
                )
            if (
                "boot" not in owner
                and type(owner.get("pid")) is int
                and isinstance(owner.get("start"), str)
            ):
                try:
                    if (
                        operation_lifetime.start_identity(owner["pid"])
                        == owner["start"]
                    ):
                        raise files.StoreError(
                            "Unresolved original process identity; conversion refused"
                        )
                except FileNotFoundError:
                    pass
            scope = owner.get("scope")
            if isinstance(scope, dict):
                scopes[str(scope.get("unit"))] = scope
        for bound in facts.get("units", []):
            if isinstance(bound, dict) and str(bound.get("unit", "")).endswith(
                ".scope"
            ):
                scopes[bound["unit"]] = bound
    for scope in scopes.values():
        observation = operation_lifetime.drained({"scope": scope}, deadline=deadline)
        if not observation.complete:
            raise files.StoreError(
                "Writer descendants are unobserved: " + observation.diagnostic()
            )


def archive(path: Path, data: bytes) -> None:
    files.protected(path.parent)
    with directory(path.parent) as parent:
        try:
            descriptor = os.open(
                path.name,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
                0o600,
                dir_fd=parent,
            )
        except FileExistsError:
            descriptor = os.open(
                path.name,
                os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW | os.O_CLOEXEC,
                dir_fd=parent,
            )
            with os.fdopen(descriptor, "rb") as stream:
                host.validate_private_file(
                    os.fstat(stream.fileno()), "original conversion image"
                )
                if (
                    os.fstat(stream.fileno()).st_size != len(data)
                    or stream.read(len(data) + 1) != data
                ):
                    raise files.StoreError(
                        "Original conversion image differs; retain fenced recovery"
                    ) from None
                os.fsync(stream.fileno())
        else:
            with os.fdopen(descriptor, "wb") as stream:
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
        os.fsync(parent)


def service_association(facts: dict, deadline: float) -> tuple[Path | None, str | None]:
    from scripts import surreal_server  # noqa: PLC0415

    state = Path(facts["state"])
    if not state.is_absolute() or any(
        path.is_symlink() for path in (state, *state.parents)
    ):
        raise files.StoreError("Original service path ownership is ambiguous")
    if (state / "config.json").is_file():
        return surreal_server.service_directory(state, deadline=deadline), None
    # Missing configuration cannot prove that a retired receiver was a standalone
    # service. Keep an explicitly uncovered possible cohort, never a quiet answer.
    parts = state.parts
    possible = (
        Path(*parts[: parts.index(".receivers")]) if ".receivers" in parts else state
    )
    return None, str(possible)


def convert(root: Path, *, deadline: float, services: Sequence[Path] = ()) -> dict:
    with directory(root):
        pass
    store = files.FileStore(root)
    transition_path = root / "format-transition.json"
    with store.legacy_lock(deadline):
        activation = files.read(root / "activation.json")
        if activation.get("version") == 3 and not transition_path.exists():
            return activation
        transition: ConversionTransition | None = (
            cast("ConversionTransition", files.read(transition_path))
            if transition_path.exists()
            else None
        )
        if transition is not None:
            validate_transition(transition)
        if transition is None:
            if (
                activation.get("version") != 2
                or activation.get("format") != "exact-resource-files"
            ):
                raise files.StoreError(
                    "Only the authenticated v2 owner can be converted"
                )
            source = images(root, deadline)
            validate_originals(source)
            quiescent(source, deadline)  # no effects before this complete cohort check
            transition = {
                "version": 1,
                "phase": "pending",
                "original_activation": activation,
                "services": [str(path.absolute()) for path in services],
            }
            validate_transition(transition)
            if files.read(root / "allocations.json") != {
                "version": 2,
                "format": "legacy-refusal",
                "migration": activation["migration"],
            }:
                raise files.StoreError(
                    "Original registry lacks its durable legacy-writer fence"
                )
            files.publish(transition_path, transition)
            files.publish(
                root / "activation.json", {"version": 0, "format": "conversion-pending"}
            )
        elif transition.get("phase") == "complete":
            store.ensure(deadline)
            return activation
        elif transition.get("version") != 1 or transition.get("phase") != "pending":
            raise files.StoreError("Unknown conversion recovery authority")
        if "sources" not in transition:
            source = images(root, deadline)
            validate_originals(source)
            quiescent(
                source, deadline
            )  # catch an already-admitted writer crossing the fence
            for name, data in source.items():
                archive(root / "original-v2" / name, data)
            transition["sources"] = {
                name: hashlib.sha256(data).hexdigest() for name, data in source.items()
            }
            files.publish(transition_path, transition)
        else:
            source = originals(root, transition, deadline)
            quiescent(source, deadline)
        converted = files.digest(
            {
                "sources": transition["sources"],
                "activation": transition["original_activation"],
            }
        )
        for name, data in source.items():
            host.remaining(deadline, 1)
            if not name.startswith("invocation-state/"):
                continue
            nonce = Path(name).stem
            original = json.loads(data)
            if original.get("version") != 2 or not isinstance(
                original.get("declaration"), dict
            ):
                raise files.StoreError(
                    "Unknown original invocation authority; conversion fenced"
                )
            declaration = dict(original["declaration"])
            header = json.loads(source["invocations/" + nonce + ".json"])
            if header != declaration:
                raise files.StoreError(
                    "Original invocation header/state conflict; conversion fenced"
                )
            declaration.update(
                version=2, terminal_owner=declaration.get("terminal_owner", "runner")
            )
            state = {
                **original,
                "version": 3,
                "declaration": declaration,
                "original_codec": "exact-resource-files-invocation-v2",
                "requirement_policy": 1,
                "conversion": converted,
            }
            if state["phase"] == "open":
                state["phase"] = "unresolved"
            requirements = state.get("requirements")
            selected = declaration.get("selected", [])
            identities = {
                item["binary_id"] + "::" + item["name"]
                if declaration.get("kind") == "rust"
                else item["name"]
                for item in selected
            }
            state["collected"] = (
                isinstance(requirements, dict)
                and set(requirements) == identities
                and (
                    declaration.get("kind") == "rust"
                    or "collection_owner" in declaration
                    or bool(selected)
                )
            )
            if not state["collected"]:
                state["original_basis"] = (
                    "unqualified; original facts retained without invented requirements"
                )
            with store.lock("invocations", nonce, deadline):
                current = files.read(root / name)
                if current.get("version") not in {2, 3}:
                    raise files.StoreError(
                        "Original invocation changed during fenced conversion"
                    )
                if current.get("version") == 2 and current != original:
                    raise files.StoreError(
                        "Original invocation changed during fenced conversion"
                    )
                store._publish_invocation(nonce, state)  # noqa: SLF001 -- fenced format-transition friend of the storage owner
                files.publish(
                    store.path("invocations", nonce),
                    resources.invocation_locator(declaration),
                )
        service_roots = {Path(value) for value in transition.get("services", [])}
        for name, data in source.items():
            if not name.startswith("resources/"):
                continue
            original = json.loads(data)
            identity = Path(name).stem
            with store.lock("resources", identity, deadline):
                current = files.read(root / name)
                if current.get("version") not in {2, 3}:
                    raise files.StoreError(
                        "Unknown resource changed during fenced conversion"
                    )
                facts = dict(original["facts"])
                expected = {**original, "version": current["version"]}
                if current["version"] == 3 and facts.get("kind") != "evidence":
                    for key in ("service_owner", "unresolved_service"):
                        if key in current["facts"]:
                            facts[key] = current["facts"][key]
                    expected["facts"] = facts
                if current != expected:
                    raise files.StoreError(
                        "Resource changed during fenced conversion; retain recovery"
                    )
                prior = files.Entry(
                    identity, current["identity"], current["facts"], current["phase"]
                )
                entry = files.Entry(
                    identity,
                    current["identity"],
                    dict(current["facts"]),
                    current["phase"],
                )
                if entry.facts.get("kind") != "evidence":
                    owner, unresolved = service_association(entry.facts, deadline)
                    if owner is not None:
                        entry.facts["service_owner"] = str(owner)
                        service_roots.add(owner)
                    else:
                        entry.facts["unresolved_service"] = unresolved
                store._save(entry, prior=prior)  # noqa: SLF001 -- fenced format-transition friend of the storage owner
        activation = {
            **transition["original_activation"],
            "version": 3,
            "conversion": converted,
        }
        # Service coverage is bound before activation. Old generations' payloads,
        # terminal identities and sealed report bytes have not been rewritten.
        identity = root.resolve().stat()
        from scripts import surreal_server  # noqa: PLC0415

        for owner in service_roots:
            if owner.is_dir():
                with surreal_server.state_lock(owner, deadline=deadline):
                    files.publish(
                        owner / ".resource-owner" / "registry.json",
                        {
                            "version": 1,
                            "registry": str(root.resolve()),
                            "identity": [identity.st_dev, identity.st_ino],
                            "activation": activation,
                        },
                    )
        files.publish(root / "activation.json", activation)
        files.publish(
            transition_path,
            {**transition, "phase": "complete", "conversion": converted},
        )
        return activation


def original(root: Path, nonce: str) -> dict:
    if not resources.IDENTIFIER.fullmatch(nonce):
        raise files.StoreError(
            "Original inspection requires an exact invocation identity"
        )
    with directory(root):
        pass
    transition = files.read(root / "format-transition.json")
    validate_transition(transition)
    if "sources" not in transition:
        raise files.StoreError("Original conversion custody is not captured")
    source = originals(root, transition, time.monotonic() + 10)
    result = {"inspection": "protected original v2", "automatic_reuse": False}
    for family in ("invocations", "invocation-state"):
        name = family + "/" + nonce + ".json"
        expected = transition["sources"].get(name)
        if expected is None:
            continue
        data = source[name]
        result[family] = json.loads(data)
    return result
