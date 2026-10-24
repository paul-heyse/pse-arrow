# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Safe structural input errors around the actual FastMCP dispatch boundary."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

from fastmcp.exceptions import NotFoundError, ToolError
from fastmcp.exceptions import ValidationError as InputValidationError
from fastmcp.server.middleware import Middleware
from pydantic import ValidationError

from scripts.capabilities.diagnostics import refusal, safe_path, schema_keys

if TYPE_CHECKING:
    from fastmcp import FastMCP
    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from fastmcp.tools import ToolResult
    from mcp.types import CallToolRequestParams


class InputDiagnostics(Middleware):
    def __init__(self, server: FastMCP) -> None:
        self.server = server

    async def on_call_tool(
        self,
        context: MiddlewareContext[CallToolRequestParams],
        call_next: CallNext[CallToolRequestParams, ToolResult],
    ) -> ToolResult:
        try:
            return await call_next(context)
        except NotFoundError as error:
            raise refusal("pse.capabilities.unknown_or_hidden_selection") from error
        except (InputValidationError, ValidationError) as error:
            original = (
                error.__cause__ if isinstance(error, InputValidationError) else error
            )
            tool = await self.server.get_tool(context.message.name)
            keys = schema_keys(tool.parameters) if tool is not None else frozenset()
            locations = []
            if isinstance(original, ValidationError):
                locations = [
                    {"path": safe_path(list(item["loc"]), keys), "code": item["type"]}
                    for item in original.errors(
                        include_input=False, include_context=False
                    )[:16]
                ]
            raise ToolError(
                json.dumps(
                    {
                        "code": "pse.capabilities.input_refused",
                        "stage": "input_validation",
                        "locations": locations,
                        "message": "Input does not match the selected operation's schema; supplied values are redacted.",
                        "remedy": "Describe the selected capability with include_schema=true and correct the indicated fields. No action was admitted by this validation failure.",
                    }
                )
            ) from error
