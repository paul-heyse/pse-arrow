# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Service borrower discovery through the existing exact resource authority.

Fixture and report owners publish their associations in FileStore. Service
supervision consumes only these obligations and never owns their disposition.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import process_observation, resource_store, host_metadata

if TYPE_CHECKING:
    from collections.abc import Mapping


def storage() -> resource_store.FileStore:
    return resource_store.FileStore(registry())


def registry() -> Path:
    selected = os.environ.get("PSE_TEST_RESOURCE_STATE")
    if selected:
        path = Path(selected)
        if not path.is_absolute():
            raise resource_store.StoreError(
                "PSE_TEST_RESOURCE_STATE must be an absolute private directory"
            )
        return path
    return host_metadata.root_path().parent / "test-resources"


def service_registry_binding(
    owner: Path, *, establish: bool = False, deadline: float | None = None
) -> None:
    """Bind covered discovery to the actual service, including empty cohorts."""
    path = owner / ".resource-owner" / "registry.json"
    existing = resource_store.read(path) if path.exists() else None
    if existing is not None and existing.get("registry") != str(registry().resolve()):
        raise resource_store.StoreError(
            "Service resource registry coverage differs from selected authority"
        )
    if existing is None and not establish:
        raise resource_store.StoreError("Service resource registry coverage is unestablished")
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
            raise resource_store.StoreError(
                "Service resource registry coverage differs from selected authority"
            )
    elif establish:
        resource_store.publish(path, expected)
    else:
        raise resource_store.StoreError("Service resource registry coverage is unestablished")


def service_resources(
    owner: Path, *, deadline: float | None = None
) -> dict[str, dict]:
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
            raise resource_store.StoreError(
                "Original service association remains unresolved; quiet is unestablished"
            )
    result = {}
    for resource in store.access_ids(
        "service-access/" + resource_store.digest(str(owner)), deadline=deadline
    ):
        entry = store.lookup(resource, deadline=deadline)
        if entry.phase != "active":
            continue  # a protected interrupted departure can leave a stale marker
        record = entry.facts.copy()
        scope = record.get("namespace") if record.get("kind") == "namespace" else record.get("database")
        if (entry.identity != [record.get("state"), scope, record.get("kind")]
                or not isinstance(record.get("state"), str)
                or type(record.get("drained")) is not bool
                or not isinstance(record.get("borrows"), dict)
                or record.get("cleanup") not in {"retained", "removing", "removed", "compacted"}):
            raise resource_store.StoreError("Invalid service borrower authority")
        if record.get("service_owner") != str(owner):
            raise resource_store.StoreError("Service access projection contradicts its authority")
        result[resource] = record.copy()
    return result



def borrower_alive(record: Mapping[str, object]) -> bool:
    pid = record.get("pid")
    if type(pid) is not int or not isinstance(record.get("start"), str):
        return False  # Nonowner metadata carries no process identity.
    try:
        return record.get("boot") == process_observation.boot() and (
            process_observation.start_identity(pid) == record["start"])
    except FileNotFoundError:
        return False
    except (OSError, ValueError, IndexError):
        return True


@dataclass(frozen=True)
class ServiceBorrower:
    """The service-facing view carries no fixture/report retention decisions."""

    state: Path
    protected: str | None
    requires_association: bool
    active_owner: bool


def obligations(owner: Path, *, deadline: float | None = None) -> tuple[ServiceBorrower, ...]:
    result = []
    for record in service_resources(owner, deadline=deadline).values():
        if record.get("kind") == "evidence" or record.get("cleanup") == "removed":
            continue
        protected = None
        if record.get("creation_borrower"):
            protected = "An accepted fixture creation still borrows this storage service; actual settlement and drain are unobserved"
        elif record.get("borrows"):
            protected = "An admitted service resource still has protected borrowers"
        else:
            cleaner = record.get("cleaner")
            if record.get("cleanup") == "removing" and isinstance(cleaner, dict) and borrower_alive(cleaner):
                protected = "A live context cleanup still borrows this storage service"
        live = borrower_alive(record)
        explicitly_drained = record.get("drained") is True
        # A retired receiver can keep protected failure evidence without retaining
        # its service association once the original effect owner actually drained.
        requires_association = not (explicitly_drained and not record.get("borrows") and not live)
        result.append(ServiceBorrower(Path(record["state"]), protected, requires_association,
                                      not record.get("drained") and live))
    return tuple(result)
