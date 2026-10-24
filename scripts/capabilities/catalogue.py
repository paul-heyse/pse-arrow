# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One exposure index derives navigation, selection and actual tool descriptions."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import TypeVar

from fastmcp.tools import Tool
from pydantic import JsonValue, TypeAdapter

from scripts.capabilities.model import Capability, Domain, Guide, Workflow

JSON_OBJECT = TypeAdapter(dict[str, JsonValue])
T = TypeVar("T", Domain, Workflow, Guide, Capability)


@dataclass(frozen=True)
class Selection:
    capabilities: tuple[str, ...]
    guides: tuple[str, ...]


class Catalogue:
    def __init__(
        self,
        domains: tuple[Domain, ...],
        workflows: tuple[Workflow, ...],
        guides: tuple[Guide, ...],
        capabilities: tuple[Capability, ...],
        tools: dict[str, Tool],
    ) -> None:
        self.domains = self._index(domains)
        self.workflows = self._index(workflows)
        self.guides = self._index(guides)
        self.capabilities = self._index(capabilities)
        self.tools = dict(tools)
        for item in capabilities:
            if item.domain not in self.domains:
                raise ValueError(f"unknown domain: {item.id}: {item.domain}")
            if len(set(item.workflows)) != len(item.workflows):
                raise ValueError(f"duplicate workflow membership: {item.id}")
            for workflow in item.workflows:
                if workflow not in self.workflows:
                    raise ValueError(f"unknown workflow: {item.id}: {workflow}")
            if (item.exposure == "tool") != (item.id in tools):
                raise ValueError(f"callable declaration/binding mismatch: {item.id}")
            if item.exposure == "deferred" and (not item.reason or not item.work_owner):
                raise ValueError(f"deferred adaptation needs reason and owner: {item.id}")
            if item.id in tools and tools[item.id].name != item.id:
                raise ValueError(f"tool identity mismatch: {item.id}")
        if set(tools) - set(self.capabilities):
            raise ValueError("undeclared callable tools")
        for item in (*domains, *workflows, *capabilities):
            for guide in item.guides:
                if guide not in self.guides:
                    raise ValueError(f"unknown guidance: {item.id}: {guide}")
        self._by_domain = {
            key: tuple(sorted(item.id for item in capabilities if item.domain == key))
            for key in self.domains
        }
        self._by_workflow = {
            key: tuple(sorted(item.id for item in capabilities if key in item.workflows))
            for key in self.workflows
        }
        declaration = {
            "domains": [item.model_dump(mode="json") for item in domains],
            "workflows": [item.model_dump(mode="json") for item in workflows],
            "guides": [item.model_dump(mode="json") for item in guides],
            "capabilities": [item.model_dump(mode="json") for item in capabilities],
            "schemas": {
                key: {"input": tool.parameters, "output": tool.output_schema}
                for key, tool in sorted(tools.items())
            },
            "handlers": {
                path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                for path in sorted(Path(__file__).parent.glob("*.py"))
            },
        }
        self.revision = hashlib.sha256(
            json.dumps(declaration, sort_keys=True, allow_nan=False).encode()
        ).hexdigest()

    @staticmethod
    def _index(items: tuple[T, ...]) -> dict[str, T]:
        result = {}
        for item in items:
            if item.id in result:
                raise ValueError(f"duplicate exposure identity: {item.id}")
            result[item.id] = item
        return result

    def browse(self, kind: str, identity: str | None, offset: int, limit: int) -> dict[str, JsonValue]:
        if offset < 0 or not 1 <= limit <= 100:
            raise ValueError("offset must be nonnegative and limit in 1..100")
        if kind == "domain":
            entities, members = self.domains, self._by_domain
        elif kind == "workflow":
            entities, members = self.workflows, self._by_workflow
        else:
            raise ValueError("browse domain or workflow")
        if identity is None:
            values = [
                {**item.model_dump(mode="json"), "capability_count": len(members[key])}
                for key, item in sorted(entities.items())
            ]
        else:
            if identity not in entities:
                raise ValueError(f"unknown {kind}: {identity}")
            values = [self.capabilities[key].model_dump(mode="json") for key in members[identity]]
        return JSON_OBJECT.validate_python({
            "revision": self.revision,
            "kind": kind,
            "identity": identity,
            "items": values[offset:offset + limit],
            "total": len(values),
            "next_offset": offset + limit if offset + limit < len(values) else None,
            "availability": "declared prerequisites; not live service readiness",
        })

    def describe(self, identity: str, *, include_schema: bool = False) -> dict[str, JsonValue]:
        if identity not in self.capabilities:
            raise ValueError(f"unknown capability: {identity}")
        item = self.capabilities[identity]
        tool = self.tools.get(identity)
        result = {
            "revision": self.revision,
            "capability": item.model_dump(mode="json"),
            "callable": tool is not None,
            "availability": "source-only" if tool else "native route or pending adaptation; not observed",
            "guidance": [self.guides[key].model_dump(mode="json") for key in item.guides],
        }
        if include_schema:
            result["input_schema"] = None if tool is None else tool.parameters
            result["output_schema"] = None if tool is None else tool.output_schema
        return JSON_OBJECT.validate_python(result)

    def select(self, domains: list[str], workflows: list[str], capabilities: list[str]) -> Selection:
        chosen = set(capabilities)
        guides = {"root"} if "root" in self.guides else set()
        for identity in domains:
            if identity not in self.domains:
                raise ValueError(f"unknown domain: {identity}")
            chosen.update(self._by_domain[identity])
            guides.update(self.domains[identity].guides)
        for identity in workflows:
            if identity not in self.workflows:
                raise ValueError(f"unknown workflow: {identity}")
            chosen.update(self._by_workflow[identity])
            guides.update(self.workflows[identity].guides)
        unknown = chosen - set(self.capabilities)
        if unknown:
            raise ValueError("unknown capabilities: " + ", ".join(sorted(unknown)))
        for identity in chosen:
            guides.update(self.capabilities[identity].guides)
        return Selection(tuple(sorted(chosen)), tuple(sorted(guides)))
