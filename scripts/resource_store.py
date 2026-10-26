# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact durable resource files; retention and effect policy belongs to test_resources.

Lock order is invocation, identity, resource. Resource-first operations never
acquire invocation exclusion. Locks survive phase changes and disposal. External
effects run outside every metadata lock. A deadline is absolute and never renewed.
"""

from __future__ import annotations

import contextlib
import copy
import fcntl
import hashlib
import json
import os
import re
import time
import uuid
from dataclasses import dataclass
from typing import TYPE_CHECKING

from scripts import host_metadata as host

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping, Sequence
    from pathlib import Path

IDENTIFIER = re.compile(r"[a-f0-9]{32}")


class StoreError(ValueError):
    """Metadata authority is unknown, contradictory or unavailable."""


def clock(deadline: float | None = None) -> float:
    return time.monotonic() + 5 if deadline is None else deadline


def digest(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def read(path: Path) -> dict:
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    with os.fdopen(descriptor) as source:
        host.validate_private_file(os.fstat(source.fileno()), "resource metadata")
        value = json.load(source)
    if not isinstance(value, dict):
        raise StoreError("Invalid resource metadata object")
    return value


def sync_directory(directory: Path) -> None:
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def settle(path: Path) -> None:
    """Settle an already-replaced object after an interrupted publication retry."""
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        host.validate_private_file(os.fstat(descriptor), "resource metadata")
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    sync_directory(path.parent)
    sync_directory(path.parent.parent)


def protected(directory: Path) -> None:
    missing = []
    current = directory
    while not current.exists():
        missing.append(current)
        current = current.parent
    host.protected(directory)
    for created in reversed(missing):
        sync_directory(created)
        sync_directory(created.parent)


def publish(path: Path, value: Mapping[str, object]) -> None:
    protected(path.parent)
    # Persist newly created directory entries as well as the file replacement.
    sync_directory(path.parent.parent)
    temporary = path.with_name(f".{path.name}.{uuid.uuid4().hex}.tmp")
    descriptor = os.open(
        temporary,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o600,
    )
    try:
        with os.fdopen(descriptor, "w") as output:
            output.write(json.dumps(value, sort_keys=True) + "\n")
            output.flush()
            os.fsync(output.fileno())
        temporary.replace(path)
        sync_directory(path.parent)
    finally:
        temporary.unlink(missing_ok=True)


@dataclass
class Entry:
    resource: str
    identity: list[str]
    facts: dict
    phase: str = "active"


class FileStore:
    def __init__(self, root: Path) -> None:
        self.root = root

    def path(self, family: str, key: str) -> Path:
        if not re.fullmatch(r"[a-z0-9_-]+", key):
            raise StoreError("Invalid exact metadata key")
        return self.root / family / f"{key}.json"

    @contextlib.contextmanager
    def lock(
        self, family: str, key: str, deadline: float
    ) -> Generator[None, None, None]:
        if not re.fullmatch(r"[a-z0-9_-]+", key):
            raise StoreError("Invalid exact metadata lock key")
        directory = self.root / "locks" / family
        protected(directory)
        try:
            descriptor = os.open(
                directory / key,
                os.O_CREAT | os.O_EXCL | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
                0o600,
            )
        except FileExistsError:
            descriptor = os.open(
                directory / key, os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC
            )
        else:
            os.fsync(descriptor)
            sync_directory(directory)
        try:
            host.validate_private_file(os.fstat(descriptor), "resource lock")
            host.lock_until(descriptor, fcntl.LOCK_EX, deadline)
            yield
        finally:
            os.close(descriptor)

    def ensure(self, deadline: float) -> None:
        host.remaining(deadline, 1)
        protected(self.root)
        activation = self.root / "activation.json"
        transition = self.root / "format-transition.json"
        if transition.exists() and read(transition).get("phase") != "complete":
            raise StoreError(
                "Resource format conversion remains fenced; resume explicit recovery"
            )
        if activation.exists():
            selected = read(activation)
            if (
                selected.get("version") != 3
                or selected.get("format") != "exact-resource-files"
                or not isinstance(selected.get("migration"), str)
                or not re.fullmatch(r"[a-f0-9]{64}", selected["migration"])
            ):
                raise StoreError(
                    "Resource registry requires explicit quiescent format conversion"
                )
            if read(self.root / "allocations.json") != {
                "version": 2,
                "format": "legacy-refusal",
                "migration": selected["migration"],
            }:
                raise StoreError(
                    "Activated registry lacks its durable legacy-writer fence"
                )
            return
        protected(self.root)
        if (self.root / "allocations.json").exists() or (
            self.root / "migration.json"
        ).exists():
            # A concurrent first opener may still be publishing activation.
            # Wait on its existing exclusion, then distinguish finished startup
            # from a crashed/unactivated registry. Never auto-resume old state.
            with self.legacy_lock(deadline):
                if activation.exists():
                    self.ensure(deadline)
                    return
                raise StoreError(
                    "Resource registry requires explicit fenced migration recovery"
                )
        self.migrate(deadline=deadline)

    @contextlib.contextmanager
    def legacy_lock(self, deadline: float) -> Generator[None, None, None]:
        descriptor = os.open(
            self.root / ".lock",
            os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
            0o600,
        )
        try:
            host.validate_private_file(os.fstat(descriptor), "legacy resource lock")
            host.lock_until(descriptor, fcntl.LOCK_EX, deadline)
            yield
        finally:
            os.close(descriptor)

    def _entry(self, resource: str) -> Entry:
        if not IDENTIFIER.fullmatch(resource):
            raise StoreError("Invalid resource identity")
        value = read(self.path("resources", resource))
        if (
            value.get("version") != 3
            or value.get("resource") != resource
            or value.get("phase") not in {"active", "cold", "disposed"}
            or not isinstance(value.get("identity"), list)
            or not all(isinstance(item, str) for item in value["identity"])
            or not isinstance(value.get("facts"), dict)
        ):
            raise StoreError("Invalid exact resource authority")
        return Entry(resource, value["identity"], value["facts"], value["phase"])

    def resource_snapshot(self, resource: str) -> Entry:
        """Read one atomic authority image without initialization, locks or repair.

        This detached metadata view grants no payload protection. Explicit reads
        acquire their resource borrow separately before using referenced files.
        """
        return self._entry(resource)

    def _save(self, entry: Entry, *, prior: Entry | None = None) -> None:
        previous_path = self.path("resources", entry.resource)
        previous = (
            prior
            if prior is not None
            else self._entry(entry.resource)
            if previous_path.exists()
            else None
        )
        markers = self._markers(entry)
        old_markers = self._markers(previous) if previous is not None else set()
        # Access precedes active authority, while departure publishes authority
        # before removing access. A crash may leave a stale marker, never a live
        # accepted borrower without its exact discovery path.
        for marker in markers:
            if not marker.exists():
                publish(marker, {"resource": entry.resource})
            elif read(marker) != {"resource": entry.resource}:
                # The resource exclusion and exact authority resolve stale
                # discovery metadata; markers never grant their own permission.
                publish(marker, {"resource": entry.resource})
            else:
                settle(marker)
        publish(
            self.path("resources", entry.resource),
            {
                "version": 3,
                "resource": entry.resource,
                "identity": entry.identity,
                "phase": entry.phase,
                "facts": entry.facts,
            },
        )
        for marker in old_markers - markers:
            if marker.exists():
                marker.unlink()
                sync_directory(marker.parent)

    def _markers(self, entry: Entry) -> set[Path]:
        if entry.phase != "active":
            return set()
        paths = {self.path("active", entry.resource)}
        facts = entry.facts
        if facts.get("kind") != "evidence" and isinstance(
            facts.get("service_owner"), str
        ):
            cohort = digest(facts["service_owner"])
            paths.add(self.path("service-access/" + cohort, entry.resource))
        elif facts.get("kind") != "evidence" and isinstance(
            facts.get("unresolved_service"), str
        ):
            paths.add(
                self.path(
                    "uncovered-service-access/" + digest(facts["unresolved_service"]),
                    entry.resource,
                )
            )
        settled = (
            facts.get("drained") is True
            and facts.get("pin") is False
            and facts.get("references") == {}
            and facts.get("borrows") == {}
            and not facts.get("creation_borrower")
            and (facts.get("disposition") == "pass" or facts.get("explicit_release"))
            and facts.get("cleanup") in {"retained", "removing"}
        )
        if settled:
            if facts.get("kind") == "evidence":
                if facts.get("artifacts") and facts.get("provenance_verified") is True:
                    paths.add(self.path("scratch-candidates", entry.resource))
            else:
                paths.add(self.path("fixture-candidates", entry.resource))
        return paths

    def access_ids(
        self,
        family: str,
        *,
        limit: int | None = None,
        after: str = "",
        deadline: float | None = None,
    ) -> list[str]:
        """Read only a covered cohort; bounded sweeps do not sort its full population."""
        deadline = clock(deadline)
        self.ensure(deadline)
        directory = self.root / family
        if not directory.exists():
            return []
        # Directory insertion order plus a rotating cursor supplies fair bounded
        # passes. No global sort or complete materialization for a bounded request.
        names = []
        deferred = []
        found = not after
        with os.scandir(directory) as entries:
            for entry in entries:
                host.remaining(deadline, 1)
                name = entry.name
                if (
                    not name.endswith(".json")
                    or not IDENTIFIER.fullmatch(name[:-5])
                    or entry.is_symlink()
                ):
                    raise StoreError("Unknown resource discovery access path")
                identity = name[:-5]
                if found:
                    names.append(identity)
                    if limit is not None and len(names) == limit:
                        return names
                elif identity == after:
                    found = True
                    if limit is None or len(deferred) < limit:
                        deferred.append(identity)
                elif limit is None or len(deferred) < limit:
                    deferred.append(identity)
        remaining = None if limit is None else max(0, limit - len(names))
        return names + (deferred if remaining is None else deferred[:remaining])

    @contextlib.contextmanager
    def edit(
        self,
        resource: str,
        *,
        deadline: float | None = None,
        settle_if_unchanged: bool = False,
    ) -> Generator[Entry, None, None]:
        deadline = clock(deadline)
        self.ensure(deadline)
        with self.lock("resources", resource, deadline):
            entry = self._entry(resource)
            original = copy.deepcopy(entry) if settle_if_unchanged else None
            yield entry
            host.remaining(deadline, 1)
            if (
                settle_if_unchanged
                and original == entry
                and self._settle_unchanged(entry, deadline)
            ):
                return
            self._save(entry)

    def _settle_unchanged(self, entry: Entry, deadline: float) -> bool:
        """Settle a prior projection's durability without replacing its files."""
        marker = self.path("active", entry.resource)
        paths = []
        for access in self._markers(entry):
            if not access.exists() or read(access) != {"resource": entry.resource}:
                return False
            paths.append(access)
        if entry.phase != "active" and marker.exists():
            return False
        paths.append(self.path("resources", entry.resource))
        for path in paths:
            host.remaining(deadline, 1)
            descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
            try:
                host.validate_private_file(os.fstat(descriptor), "resource metadata")
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
            sync_directory(path.parent)
            sync_directory(path.parent.parent)
        if entry.phase != "active" and marker.parent.exists():
            # A previous departure may have unlinked the access marker before
            # its directory fsync failed. Settle that absence as well.
            host.remaining(deadline, 1)
            sync_directory(marker.parent)
        host.remaining(deadline, 1)
        return True

    def lookup(self, resource: str, *, deadline: float | None = None) -> Entry:
        deadline = clock(deadline)
        self.ensure(deadline)
        with self.lock("resources", resource, deadline):
            return self._entry(resource)

    def identity_lookup(
        self, identity: Sequence[str], *, deadline: float | None = None
    ) -> str | None:
        deadline = clock(deadline)
        self.ensure(deadline)
        key = digest(list(identity))
        with self.lock("identities", key, deadline):
            path = self.path("identities", key)
            pending = self.path("identity-pending", key)
            if not path.exists():
                if pending.exists():
                    raise StoreError(
                        "Uncertain logical identity publication remains protected"
                    )
                return None
            index = read(path)
            if index.get("identity") != list(identity):
                raise StoreError(
                    "Logical identity hash collision or contradictory index"
                )
            resource = index.get("resource")
            if not isinstance(resource, str):
                raise StoreError("Invalid logical identity resource")
            with self.lock("resources", resource, deadline):
                if self._entry(resource).identity != list(identity):
                    raise StoreError("Logical identity differs from resource authority")
            return resource

    def active_ids(
        self,
        *,
        limit: int | None = None,
        after: str = "",
        deadline: float | None = None,
    ) -> list[str]:
        deadline = clock(deadline)
        self.ensure(deadline)
        directory = self.root / "active"
        if not directory.exists():
            return []
        # Only live access paths; never enumerate resources or historical records.
        names = []
        for path in directory.iterdir():
            host.remaining(deadline, 1)
            if (
                path.suffix != ".json"
                or not IDENTIFIER.fullmatch(path.stem)
                or path.is_symlink()
            ):
                raise StoreError("Unknown active resource access path")
            names.append(path.stem)
        names.sort()
        names = [name for name in names if name > after] + [
            name for name in names if name <= after
        ]
        return names if limit is None else names[:limit]

    def _invocation_object(self, family: str, value: dict) -> str:
        """Publish immutable owner data before any coordination pointer exposes it."""
        identity = digest(value)
        path = self.path(family, identity)
        if path.exists():
            if read(path) != value:
                raise StoreError("Immutable invocation object differs from its digest")
            settle(path)
        else:
            publish(path, value)
        return identity

    def _invocation_view(self, nonce: str) -> dict:
        state = read(self.path("invocation-state", nonce))
        if (
            state.get("version") != 3
            or state.get("phase") not in {"open", "frozen", "unresolved", "abandoned"}
            or not isinstance(state.get("members"), dict)
        ):
            raise StoreError("Unknown invocation coordination authority")
        reference = state.get("descriptor")
        if reference is None:
            inputs = read(self.path("invocation-staging", nonce))
        else:
            if not isinstance(reference, str) or not re.fullmatch(
                r"[a-f0-9]{64}", reference
            ):
                raise StoreError("Invalid collected invocation reference")
            inputs = read(self.path("invocation-inputs", reference))
            if digest(inputs) != reference:
                raise StoreError("Collected invocation digest mismatch")
        if inputs.get("nonce") != nonce or not isinstance(
            inputs.get("declaration"), dict
        ):
            raise StoreError("Invocation input association mismatch")
        state["declaration"] = inputs["declaration"]
        state["requirements"] = inputs.get("requirements", {})
        state["requirement_policy"] = inputs.get("policy_version")
        state["collected"] = reference is not None
        terminal = state.get("terminal_ref")
        if terminal is not None:
            value = read(self.path("invocation-terminals", terminal))
            if digest(value) != terminal or value.get("nonce") != nonce:
                raise StoreError("Invalid frozen invocation terminal detail")
            state["terminal"] = value["terminal"]
        return state

    def _publish_invocation(
        self, nonce: str, state: dict, *, deadline: float | None = None
    ) -> None:
        deadline = clock(deadline)
        host.remaining(deadline, 1)
        inputs = {
            "version": 1,
            "policy_version": state.get("requirement_policy"),
            "nonce": nonce,
            "declaration": state["declaration"],
            "requirements": state.get("requirements", {}),
        }
        coordination = {
            key: value
            for key, value in state.items()
            if key
            not in {
                "declaration",
                "requirements",
                "requirement_policy",
                "collected",
                "terminal",
            }
        }
        coordination["version"] = 3
        reference = state.get("descriptor")
        if reference is not None:
            if digest(inputs) != reference:
                raise StoreError("Collected invocation facts are immutable")
        elif state.get("collected"):
            reference = self._invocation_object("invocation-inputs", inputs)
            coordination["descriptor"] = reference
            state["descriptor"] = reference
        else:
            staging = self.path("invocation-staging", nonce)
            if not staging.exists() or read(staging) != inputs:
                publish(staging, inputs)
        if "terminal" in state:
            coordination["terminal_ref"] = self._invocation_object(
                "invocation-terminals", {"nonce": nonce, "terminal": state["terminal"]}
            )
        host.remaining(deadline, 1)
        publish(self.path("invocation-state", nonce), coordination)
        host.remaining(deadline, 1)

    def create_invocation(
        self,
        declaration: Mapping[str, object],
        *,
        requirement_policy: int | None = None,
        deadline: float | None = None,
    ) -> None:
        deadline = clock(deadline)
        self.ensure(deadline)
        nonce = str(declaration["nonce"])
        with self.lock("invocations", nonce, deadline):
            path = self.path("invocation-state", nonce)
            if path.exists():
                raise StoreError("Invocation identity already exists")
            self._publish_invocation(
                nonce,
                {
                    "version": 3,
                    "declaration": dict(declaration),
                    "requirement_policy": requirement_policy,
                    "phase": "open",
                    "members": {},
                },
                deadline=deadline,
            )

    @contextlib.contextmanager
    def invocation_edit(
        self, nonce: str, *, deadline: float | None = None
    ) -> Generator[dict, None, None]:
        deadline = clock(deadline)
        self.ensure(deadline)
        with self.lock("invocations", nonce, deadline):
            state = self._invocation_view(nonce)
            yield state
            self._publish_invocation(nonce, state, deadline=deadline)

    def register(
        self,
        identity: Sequence[str],
        facts: dict,
        *,
        invocation: str | None,
        declaration: Mapping[str, object] | None = None,
        deadline: float | None = None,
    ) -> str:
        deadline = clock(deadline)
        self.ensure(deadline)
        resource = uuid.uuid4().hex
        # Bare owners are pinned by the policy layer, and have no terminal fence.
        owner = (
            self.invocation_edit(invocation, deadline=deadline)
            if invocation
            else contextlib.nullcontext(None)
        )
        with owner as state:
            if state is not None and state["phase"] != "open":
                raise StoreError(
                    "Invocation is frozen; registration refused before effects"
                )
            if (
                state is not None
                and declaration is not None
                and state["declaration"] != declaration
            ):
                raise StoreError(
                    "Selected declaration differs from invocation authority"
                )
            key = digest(list(identity))
            with self.lock("identities", key, deadline):
                index = self.path("identities", key)
                pending = self.path("identity-pending", key)
                if index.exists() or pending.exists():
                    existing = read(index if index.exists() else pending)
                    if existing.get("identity") != list(identity):
                        raise StoreError("Logical identity hash collision")
                    raise StoreError("Fixture identity already registered")
                member: dict | None = None
                coordination = (
                    self.path("invocation-state", invocation) if invocation else None
                )
                if state is not None:
                    if coordination is None:
                        raise StoreError("Invocation membership lacks its authority")
                    member = {
                        "phase": "pending",
                        "identity": list(identity),
                        "publication": "not-started",
                    }
                    state["members"][resource] = member
                    self._publish_invocation(str(invocation), state, deadline=deadline)
                    member["publication"] = "uncertain"
                    self._publish_invocation(str(invocation), state, deadline=deadline)
                with self.lock("resources", resource, deadline):
                    # Reserve exact access before authority publication. A crashed
                    # bare registration must not silently recreate this identity.
                    publish(pending, {"identity": list(identity), "resource": resource})
                    self._save(Entry(resource, list(identity), dict(facts)))
                    publish(index, {"identity": list(identity), "resource": resource})
                    pending.unlink()
                    sync_directory(pending.parent)
                    if member is not None and coordination is not None:
                        if state is None:
                            raise StoreError(
                                "Registered invocation lacks its authority"
                            )
                        member["phase"] = "registered"
                        member["publication"] = "durable"
                        # invocation_edit publishes completion once, still under
                        # invocation exclusion, after the child locks release.
                        # If that publication fails, freeze reconstructs the
                        # durable authority from the pending/uncertain member.
        return resource

    def _settle_members(self, state: dict, nonce: str, deadline: float) -> None:
        """Resolve exact Pending metadata under the invocation exclusion."""
        for resource, member in list(state["members"].items()):
            if member.get("phase") == "registered":
                continue
            if member.get("phase") != "pending" or not isinstance(
                member.get("identity"), list
            ):
                raise StoreError("Unknown pending invocation member")
            logical = member["identity"]
            with (
                self.lock("identities", digest(logical), deadline),
                self.lock("resources", resource, deadline),
            ):
                path = self.path("resources", resource)
                if not path.exists():
                    if member.get("publication") == "not-started":
                        del state["members"][resource]
                        continue
                    raise StoreError("Uncertain resource publication remains pinned")
                entry = self._entry(resource)
                if entry.identity != logical or entry.facts.get("invocation") != nonce:
                    raise StoreError("Pending member contradicts resource authority")
                index = self.path("identities", digest(logical))
                expected = {"identity": logical, "resource": resource}
                if index.exists() and read(index) != expected:
                    raise StoreError("Pending member has contradictory logical index")
                publish(index, expected)
                pending = self.path("identity-pending", digest(logical))
                if pending.exists():
                    if read(pending) != expected:
                        raise StoreError("Pending logical identity conflicts")
                    pending.unlink()
                    sync_directory(pending.parent)
                self._save(entry)
                member.update(phase="registered", publication="durable")

    def freeze(
        self,
        nonce: str,
        terminal: dict,
        *,
        declaration: Mapping[str, object] | None = None,
        deadline: float | None = None,
        on_frozen: Callable[[], None] | None = None,
    ) -> list[str]:
        deadline = clock(deadline)
        identity = digest(terminal)
        with self.invocation_edit(nonce, deadline=deadline) as state:
            if state["phase"] == "abandoned":
                raise StoreError(
                    "Invocation membership was explicitly abandoned; terminal freeze refused"
                )
            if declaration is not None and state["declaration"] != declaration:
                raise StoreError(
                    "Invocation declaration differs from terminal selection authority"
                )
            if state["phase"] == "frozen":
                if (
                    state.get("terminal_identity") != identity
                    or state.get("terminal") != terminal
                ):
                    raise StoreError(
                        "Invocation already has another immutable terminal composition"
                    )
                if on_frozen is not None:
                    on_frozen()
                return list(state["members"])
            self._settle_members(state, nonce, deadline)
            state.update(phase="frozen", terminal=terminal, terminal_identity=identity)
            self._publish_invocation(nonce, state, deadline=deadline)
            if on_frozen is not None:
                on_frozen()
            return list(state["members"])

    def terminal(
        self, nonce: str, *, deadline: float | None = None
    ) -> tuple[str, dict]:
        deadline = clock(deadline)
        self.ensure(deadline)
        with self.lock("invocations", nonce, deadline):
            state = self._invocation_view(nonce)
            if (
                state.get("version") != 3
                or state.get("phase") != "frozen"
                or not isinstance(state.get("terminal"), dict)
                or state.get("terminal_identity") != digest(state["terminal"])
            ):
                raise StoreError("Invocation has no frozen terminal composition")
            return state["terminal_identity"], state["terminal"]

    def invocation_snapshot(self, nonce: str) -> dict:
        """Read an atomic metadata image without initializing or repairing its owner.

        Publication replaces a whole file. This detached view grants no payload
        protection and deliberately does not call ensure, edit, freeze or migrate.
        """
        state = self._invocation_view(nonce)
        if (
            state.get("version") != 3
            or state.get("phase") not in {"open", "frozen", "unresolved", "abandoned"}
            or not isinstance(state.get("members"), dict)
            or not isinstance(state.get("declaration"), dict)
            or state["declaration"].get("nonce") != nonce
        ):
            raise StoreError("Invalid invocation authority image")
        if state["phase"] == "frozen" and (
            not isinstance(state.get("terminal"), dict)
            or state.get("terminal_identity") != digest(state["terminal"])
            or state["terminal"].get("selection")
            != digest(state["declaration"].get("selected"))
        ):
            raise StoreError("Invalid frozen invocation terminal")
        return state

    def abandon(
        self, nonce: str, expected: dict, *, deadline: float | None = None
    ) -> dict:
        """Close exact membership without manufacturing a terminal or an ACK.

        Physical observation precedes this short compare-and-fence transition.
        Any changed member/composer requires a new observation, never a blind retry.
        """
        deadline = clock(deadline)
        with self.invocation_edit(nonce, deadline=deadline) as state:
            if state != expected:
                raise StoreError(
                    "Invocation changed since abandonment inspection; inspect again"
                )
            if state.get("abandonment") is not None:
                return state["abandonment"]
            self._settle_members(state, nonce, deadline)
            disposition = {
                "version": 1,
                "identity": uuid.uuid4().hex,
                "selection": digest(state["declaration"]["selected"]),
                "members": list(state["members"]),
                "phase": "fenced",
                "resources": {},
            }
            state["abandonment"] = disposition
            if state["phase"] != "frozen":
                state["phase"] = "abandoned"
            # invocation_edit durably publishes the fence before caller effects.
            return disposition

    def migrate(self, *, deadline: float | None = None) -> None:
        """Explicit cutover/recovery. Callers must separately authorize existing content."""
        deadline = clock(deadline)
        protected(self.root)
        descriptor = os.open(
            self.root / ".lock",
            os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
            0o600,
        )
        try:
            host.validate_private_file(os.fstat(descriptor), "legacy resource lock")
            host.lock_until(descriptor, fcntl.LOCK_EX, deadline)
            activation = self.root / "activation.json"
            if activation.exists():
                self.ensure(deadline)
                return
            legacy = self.root / "allocations.json"
            manifest_path = self.root / "migration.json"
            current = read(legacy) if legacy.exists() else {"version": 1, "owners": {}}
            fenced = (
                current.get("version") == 2
                and current.get("format") == "legacy-refusal"
            )
            snapshots = {}
            for family, pattern in (
                ("completed", r"[a-f0-9]{32}"),
                ("reports", r"[a-f0-9]{64}"),
            ):
                snapshots[family] = {}
                directory = self.root / family
                if directory.exists():
                    for path in directory.iterdir():
                        host.remaining(deadline, 1)
                        if path.suffix != ".json" or not re.fullmatch(
                            pattern, path.stem
                        ):
                            raise StoreError("Unknown legacy exact access path")
                        snapshots[family][path.stem] = read(path)
            if manifest_path.exists():
                manifest = read(manifest_path)
                if (
                    manifest.get("version") != 2
                    or not isinstance(manifest.get("records"), dict)
                    or not isinstance(manifest.get("legacy"), dict)
                    or digest(manifest["legacy"]) != manifest.get("legacy_digest")
                ):
                    raise StoreError("Invalid staged migration authority")
                if fenced and current.get("migration") != digest(manifest):
                    raise StoreError(
                        "Legacy refusal fence differs from staged migration"
                    )
                if manifest.get("snapshots") != snapshots:
                    raise StoreError(
                        "Legacy access paths changed after staging; retain evidence"
                    )
                if not fenced and digest(current) != manifest.get("legacy_digest"):
                    raise StoreError(
                        "Legacy writer changed staged migration; retain evidence"
                    )
            else:
                if (
                    fenced
                    or current.get("version") != 1
                    or not isinstance(current.get("owners"), dict)
                ):
                    raise StoreError("Unknown legacy registry; migration refused")
                records = dict(current["owners"])
                for resource, record in snapshots["completed"].items():
                    if resource in records and records[resource] != record:
                        raise StoreError("Ambiguous legacy active/completed authority")
                    records[resource] = record
                manifest = {
                    "version": 2,
                    "legacy_digest": digest(current),
                    "legacy": current,
                    "records": records,
                    "snapshots": snapshots,
                }
                publish(manifest_path, manifest)
            imported_records = manifest.get("records")
            if not isinstance(imported_records, dict):
                raise StoreError("Invalid staged migration records")
            for key, index in snapshots["reports"].items():
                resource = index.get("resource")
                state = index.get("state")
                facts = imported_records.get(resource)
                if (
                    not isinstance(state, str)
                    or hashlib.sha256(state.encode()).hexdigest() != key
                    or not isinstance(facts, dict)
                    or facts.get("kind") != "evidence"
                    or facts.get("state") != state
                ):
                    raise StoreError("Dangling or contradictory legacy report index")
            identities: dict[str, list[str]] = {}
            invocations: dict[str, dict] = {}
            for resource, facts in imported_records.items():
                host.remaining(deadline, 1)
                if not IDENTIFIER.fullmatch(resource) or not isinstance(facts, dict):
                    raise StoreError("Unknown legacy resource authority")
                if not all(
                    name in facts
                    for name in (
                        "state",
                        "database",
                        "kind",
                        "pin",
                        "drained",
                        "cleanup",
                        "references",
                        "borrows",
                    )
                ):
                    raise StoreError("Incomplete legacy resource remains protected")
                identity = [
                    str(facts["state"]),
                    str(facts["database"]),
                    str(facts["kind"]),
                ]
                key = digest(identity)
                if key in identities:
                    raise StoreError("Ambiguous duplicate legacy logical identity")
                identities[key] = identity
                phase = (
                    "cold"
                    if facts.get("kind") == "evidence"
                    and facts.get("cleanup") == "compacted"
                    and facts.get("drained") is True
                    and facts.get("disposition") == "pass"
                    and facts.get("pin") is False
                    and not facts.get("references")
                    and not facts.get("borrows")
                    and not facts.get("creation_borrower")
                    else "active"
                )
                entry = Entry(resource, identity, facts, phase)
                target = self.path("resources", resource)
                if target.exists() and self._entry(resource) != entry:
                    raise StoreError(
                        "Staged imported resource differs from original authority"
                    )
                self._save(entry)
                index = self.path("identities", key)
                expected = {"identity": identity, "resource": resource}
                if index.exists() and read(index) != expected:
                    raise StoreError("Staged logical index conflicts")
                publish(index, expected)
                nonce = facts.get("invocation")
                if nonce is not None:
                    if not isinstance(nonce, str) or not IDENTIFIER.fullmatch(nonce):
                        raise StoreError("Unknown legacy invocation identity")
                    if nonce not in invocations:
                        declaration = read(self.path("invocations", nonce))
                        # Legacy absence of a terminal is not evidence that registration
                        # is still open. Only authenticated recovery may freeze/project.
                        invocations[nonce] = {
                            "version": 2,
                            "declaration": declaration,
                            "phase": "unresolved",
                            "members": {},
                        }
                    invocations[nonce]["members"][resource] = {
                        "phase": "registered",
                        "identity": identity,
                        "publication": "durable",
                    }
            for nonce, state in invocations.items():
                self._publish_invocation(nonce, state, deadline=deadline)
            # A waiting old writer rejects this durable version before activation.
            publish(
                legacy,
                {
                    "version": 2,
                    "format": "legacy-refusal",
                    "migration": digest(manifest),
                },
            )
            publish(
                activation,
                {
                    "version": 3,
                    "format": "exact-resource-files",
                    "migration": digest(manifest),
                },
            )
        finally:
            os.close(descriptor)
