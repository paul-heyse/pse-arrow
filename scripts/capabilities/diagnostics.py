# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Safe host interpretations; native owners alone issue recovery identities."""

from __future__ import annotations

import json

from fastmcp.exceptions import ToolError


def refusal(code: str, *, operation: str | None = None) -> ToolError:
    """Explain a known boundary without rendering an exception or supplied value."""
    if code.endswith("restart_required"):
        remedy = "Restart or reattach this endpoint, then describe and select the capability again."
    elif "unknown_" in code or code.endswith("binding_required"):
        remedy = "Browse exact domains/workflows, describe the capability and select its exact ID before invoking it."
    elif code.endswith("source_unavailable") or code.endswith("guidance_unavailable"):
        remedy = "Inspect the selected source owner's availability; explicitly restore or prepare it before reading again."
    else:
        remedy = "Describe this operation's input schema and correct the selected input. No retry or effect settlement is inferred."
    value = {
        "code": code,
        "stage": "host_selection" if "capabilities." in code else "owner_read",
        "message": "The selected request could not be served; supplied values are redacted.",
        "remedy": remedy,
    }
    if operation is not None:
        value["operation"] = operation
    return ToolError(json.dumps(value, sort_keys=True))


def schema_keys(value: object) -> frozenset[str]:
    """Vocabulary declared by a schema, excluding arbitrary instance-map keys."""
    keys: set[str] = set()
    if isinstance(value, dict):
        properties = value.get("properties")
        if isinstance(properties, dict):
            keys.update(key for key in properties if isinstance(key, str))
        for child in value.values():
            keys.update(schema_keys(child))
    elif isinstance(value, list):
        for child in value:
            keys.update(schema_keys(child))
    return frozenset(keys)


def safe_path(path: list[object], keys: frozenset[str]) -> list[object]:
    return [item if isinstance(item, int) or isinstance(item, str) and item in keys
            else "<redacted-key>" for item in path[:32]]
