# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded library probe for the accompanying opportunity review.

Uses an in-process MCP client and a stub for generated CLI forwarding. It opens
no project database, runs no workload and changes no agent configuration.
"""

from __future__ import annotations

import asyncio
import importlib.metadata
import json
import logging
import warnings
from types import SimpleNamespace
from typing import Annotated, cast

from fastmcp import Client, FastMCP
from fastmcp.cli.generate import generate_cli_script
from pydantic import MISSING, BaseModel, ConfigDict, Field, TypeAdapter, ValidationError
from typing_extensions import TypedDict


class Request(TypedDict):
    handle: Annotated[str, Field(min_length=1)]
    limit: Annotated[int, Field(gt=0, le=10)]


class Patch(BaseModel):
    model_config = ConfigDict(extra="forbid")
    nullable: int | None | MISSING = MISSING
    tags: list[str] | MISSING = MISSING


class Reply(BaseModel):
    value: int


async def main() -> None:
    logging.disable(logging.CRITICAL)  # Expected refusal cases contain synthetic inputs.
    adapter = TypeAdapter(Request)
    assert adapter.validate_python(
        {"handle": "native:probe", "limit": 2}, strict=True
    )["limit"] == 2
    try:
        adapter.validate_python({"handle": "native:probe", "limit": "2"}, strict=True)
    except ValidationError as error:
        input_errors = [
            {"type": item["type"], "loc": list(item["loc"])}
            for item in error.errors(include_input=False, include_url=False)
        ]
    else:
        raise AssertionError("strict integer accepted a string")
    assert Patch().model_dump() == {}
    assert Patch(nullable=None, tags=[]).model_dump() == {"nullable": None, "tags": []}

    server = FastMCP("review-probe", strict_input_validation=True, mask_error_details=True)

    @server.tool
    async def inspect_handle(
        handle: str, limit: Annotated[int, Field(gt=0, le=10)] = 2
    ) -> Reply:
        return Reply(value=limit)

    @server.tool
    async def body_error() -> Reply:
        return Reply.model_validate({"value": "invalid"})

    @server.tool
    async def wrong_output() -> Reply:
        # Deliberately bypass the declared return contract to test the receiver.
        return cast(Reply, {"value": "invalid"})

    async with Client(server) as client:
        catalogue = await client.list_tools()
        tools = {item.name: item for item in catalogue}
        assert tools["inspect_handle"].input_schema["properties"]["limit"]["maximum"] == 10
        good = await client.call_tool("inspect_handle", {"handle": "native:probe", "limit": 2})
        assert good.structured_content == {"value": 2}
        bad = await client.call_tool(
            "inspect_handle", {"handle": "native:probe", "limit": "2"}, raise_on_error=False
        )
        body = await client.call_tool("body_error", {}, raise_on_error=False)
        assert bad.is_error and body.is_error
        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")
                await client.call_tool("wrong_output", {}, raise_on_error=False)
        except RuntimeError as error:
            assert "Invalid structured content" in str(error)
            output_refused = True
        else:
            raise AssertionError("receiver accepted an invalid declared output")

        source = generate_cli_script("probe", "probe.py", repr("probe.py"), set(), catalogue)
        namespace = {"__name__": "generated_review_probe"}
        exec(compile(source, "<generated-review-probe>", "exec"), namespace)
        received = []

        class StubClient:
            def __init__(self, *args: object, **kwargs: object) -> None:
                pass

            async def __aenter__(self) -> StubClient:
                return self

            async def __aexit__(self, *args: object) -> None:
                return None

            async def call_tool(self, name: str, arguments: dict, **kwargs: object) -> object:
                received.append(arguments)
                return SimpleNamespace(is_error=False)

        namespace["Client"] = StubClient
        namespace["_print_tool_result"] = lambda result: None
        await namespace["_call_tool"]("patch", {"nullable": None, "tags": []})
        assert received == [{}]
    print(
        json.dumps(
            {
                "versions": {
                    name: importlib.metadata.version(name)
                    for name in ("fastmcp", "mcp", "pydantic", "pydantic-core")
                },
                "strict_error": input_errors,
                "fastmcp_bad_input_is_error": bad.is_error,
                "fastmcp_body_error_is_error": body.is_error,
                "receiver_refused_invalid_output": output_refused,
                "pydantic_patch": Patch(nullable=None, tags=[]).model_dump(),
                "generated_cli_forwarded": received[0],
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    asyncio.run(main())
