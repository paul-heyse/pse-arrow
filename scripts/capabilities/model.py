# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact exposure metadata, separate from native schemas and current plan state."""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field
from scripts.capabilities.workers import WorkerCall

Identifier = Annotated[str, Field(pattern=r"^[a-z][a-z0-9_-]{0,95}$")]


class Contract(BaseModel):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )


class Guide(Contract):
    id: Identifier
    title: str
    path: str
    kind: Literal["markdown", "skill"] = "markdown"

    @property
    def uri(self) -> str:
        return "pse://guidance/" + self.id


class Domain(Contract):
    id: Identifier
    title: str
    description: str
    guides: tuple[str, ...] = ()


class WorkflowStep(Contract):
    id: Identifier
    capabilities: tuple[str, ...]
    purpose: str
    guides: tuple[str, ...] = ()
    default: bool = True


class Workflow(Contract):
    id: Identifier
    title: str
    description: str
    guides: tuple[str, ...] = ()
    steps: tuple[WorkflowStep, ...] = ()


class Prerequisite(Contract):
    kind: Literal["python-extension", "selected-service", "language-provider", "executable", "checkout-environment"]
    owner: str
    preparation: tuple[str, ...] = ()
    provider: str | None = None
    mode: Literal["dev", "native"] | None = None
    validated: bool | None = None


class Capability(Contract):
    id: Identifier
    title: str
    description: str
    domain: Identifier
    family: Identifier | None = None
    workflows: tuple[str, ...] = ()
    guides: tuple[str, ...] = ()
    owner: str
    source: str
    exposure: Literal["tool", "native", "deferred", "internal", "unsupported"]
    effects: str
    prerequisites: tuple[str, ...] = ()
    route: tuple[str, ...] = ()
    reason: str = ""
    work_owner: str | None = None
    worker_calls: tuple[WorkerCall, ...] = ()
    requirements: tuple[Prerequisite, ...] = ()

    @property
    def tags(self) -> set[str]:
        return (
            {"capability:" + self.id, "domain:" + self.domain}
            | {"workflow:" + workflow for workflow in self.workflows}
            | ({"family:" + self.family} if self.family is not None else set())
        )
