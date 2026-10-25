# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cyclopts consumer of canonical capabilities, independent of server imports."""

from __future__ import annotations

import asyncio
from pathlib import Path

from cyclopts import App

from scripts import capability_cli_runtime as runtime


def application() -> App:
    app = App(name="capability", help=__doc__)

    @app.command
    def diagnose(*, probe: bool = False, capability: str | None = None) -> None:
        """Explain independent launch placement; --probe explicitly handshakes a fresh endpoint, with optional passive capability readiness. Never prepares or repairs."""
        if capability is not None and not probe:
            raise runtime.ClientRefusedError(
                "--capability needs explicit --probe for live description/readiness."
            )
        report = asyncio.run(runtime.probe(capability)) if probe else runtime.diagnose()
        runtime.emit(report)
        if (
            report.get("connection") in {"unavailable", "refused"}
            or report.get("original_result") is not None
        ):
            raise SystemExit(1)

    @app.command
    def reconnect(
        *, socket: Path | None = None, thread: str | None = None, timeout: float = 15
    ) -> None:
        """Queue Codex MCP reload for ALL loaded threads, then check this thread's benign bootstrap. Never replays actions; queued is not refreshed."""
        # Independent optional host control, also available without generation.
        from scripts import codex_mcp  # noqa: PLC0415

        report = asyncio.run(
            codex_mcp.reconnect(socket_path=socket, thread_id=thread, timeout=timeout)
        )
        runtime.emit(report)
        if report["reload"] != "queued":
            raise SystemExit(1)

    # Diagnose and generic help remain available when generation is absent/broken.
    try:
        from scripts.generated.capabilities.cli import (  # noqa: PLC0415 -- optional generated consumer.
            OPERATIONS,
            register,
        )
    except (ImportError, SyntaxError) as error:
        app.help = f"{__doc__}\nGenerated commands unavailable ({type(error).__name__}); run the original codegen-capability-cli owner."
        return app
    app.command(register())

    @app.command
    def session() -> None:
        """Retain one foreground connection for explicit JSONL requests; EOF closes it. No crash recovery of endpoint previews."""
        asyncio.run(runtime.session(OPERATIONS))

    @app.command
    def invoke(
        capability: str,
        *,
        input_json: str | None = None,
        input_file: Path | None = None,
    ) -> None:
        """Invoke an exact operation with lossless JSON object input, selecting in one connection."""
        if (input_json is None) == (input_file is None):
            raise runtime.ClientRefusedError(
                "Select exactly one of --input-json or --input-file."
            )
        expected = OPERATIONS.get(capability)
        if expected is None:
            raise runtime.ClientRefusedError(
                "Unknown generated callable operation; describe the native route or regenerate."
            )
        if input_json is not None:
            raw = input_json
        elif isinstance(input_file, Path):
            raw = input_file.read_text(encoding="utf-8")
        else:
            raise runtime.ClientRefusedError(
                "Select an input object or file before effects."
            )
        runtime.call(capability, runtime.parse_request(raw), expected)

    @app.command
    def describe(capability: str) -> None:
        """Read the exact live schema, source requirements and original recovery routes."""

        async def read() -> dict[str, object]:
            async with runtime.connection(runtime.checkout_root()) as client:
                result = await client.call_tool(
                    "capabilities_describe",
                    {"capability": capability, "include_schema": True},
                    raise_on_error=False,
                )
                return runtime.result_document(result)

        result = asyncio.run(read())
        runtime.emit(result)
        if result["is_error"]:
            raise SystemExit(1)

    return app


def main() -> int:
    try:
        application()()
    except Exception as error:
        return runtime.failure(error)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
