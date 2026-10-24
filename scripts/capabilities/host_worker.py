# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed main-thread host kernel worker, launched through the checkout admission owner.

This is an internal transport for three closed operations. It does not parse or
reconstruct their CLI namespaces, select executables or own native child lifetimes.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from pydantic import JsonValue, ValidationError


def invoke(operation: str, document: str) -> dict[str, JsonValue]:
    if operation == "archive":
        from scripts.test_archives import ArchiveRequest, archive  # noqa: PLC0415

        return archive(ArchiveRequest.model_validate_json(document))
    if operation == "replay":
        from scripts.test_archives import ReplayRequest, replay  # noqa: PLC0415

        request = ReplayRequest.model_validate_json(document)
        return replay(Path(request.archive), Path(request.output))
    if operation == "diagnostic":
        from scripts.active_diagnostics import ActionRequest, execute  # noqa: PLC0415

        return execute(ActionRequest.model_validate_json(document))
    raise ValueError("unsupported fixed host operation")


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    if len(arguments) != 3 or arguments[0] not in {"archive", "replay", "diagnostic"}:
        raise ValueError("select one fixed operation, request file and response file")
    operation, source, destination = arguments
    response: dict[str, object]
    try:
        result = invoke(operation, Path(source).read_text())
        response = {"result": result}
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        detail: dict[str, object] = {
            "code": "pse.host." + operation + ".refused",
            "cause": type(error).__name__,
            "message": str(error),
        }
        if isinstance(error, ValidationError):
            detail["message"] = "Invalid host request; supplied values omitted."
            detail["fields"] = error.errors(include_input=False, include_context=False)
        response = {"error": detail}
    with Path(destination).open("x") as output:
        json.dump(response, output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
