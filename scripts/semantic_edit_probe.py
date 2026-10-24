# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit real-server fixture qualification, separate from host unit tests."""

from __future__ import annotations

import argparse
import asyncio
import json
import subprocess
import sys
import tempfile
from pathlib import Path

from scripts import semantic_edits, semantic_navigation


def _require_probe(condition: object, reason: object) -> None:
    """Probe checks remain explicit even if Python assertion optimization is enabled."""
    if not condition:
        raise AssertionError(reason)


def processes() -> set[int]:
    """Observe only language/bridge process names, without reading their environments."""
    selected = set()
    for entry in Path("/proc").iterdir():
        if not entry.name.isdecimal():
            continue
        try:
            if (entry / "comm").read_text().strip() in {
                "mcpls",
                "pyrefly",
                "rust-analyzer",
            }:
                selected.add(int(entry.name))
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return selected


def _calls(path: Path, line: int, character: int, end_character: int) -> list[dict]:
    position = {"file_path": str(path), "line": line, "character": character}
    return [
        {"name": "get_tool_support", "arguments": {}},
        {"name": "prepare_rename", "arguments": position},
        {"name": "rename_symbol", "arguments": {**position, "new_name": "evaluate"}},
        {
            "name": "get_code_actions",
            "arguments": {
                "file_path": str(path),
                "start_line": line,
                "start_character": character,
                "end_line": line,
                "end_character": end_character,
            },
        },
    ]


async def cancelled_request(root: Path, language: str) -> dict:
    before = processes()
    task = asyncio.create_task(
        semantic_navigation.request(
            root,
            language,
            [{"name": "get_tool_support", "arguments": {}}] * 1000,
            fixture=True,
        )
    )
    deadline = asyncio.get_running_loop().time() + 10
    while not (processes() - before):
        if task.done():
            await task
            raise AssertionError(
                "request completed before cancellation could be exercised"
            )
        if asyncio.get_running_loop().time() > deadline:
            task.cancel()
            raise AssertionError("request-owned server launch was not observed")
        await asyncio.sleep(0.01)
    task.cancel()
    try:
        await task
    except asyncio.CancelledError:
        remaining = sorted(processes() - before)
        _require_probe(
            not remaining, f"cancelled request leaked owned processes: {remaining}"
        )
        return {"cancelled": True, "new_processes_after_close": remaining}
    raise AssertionError("request did not propagate cancellation")


def probe(language: str) -> dict:
    before_processes = processes()
    with tempfile.TemporaryDirectory(prefix="pse-semantic-probe-") as directory:
        root = Path(directory)
        if language == "rust":
            # rustup chooses by child cwd; keep the temporary language workspace on
            # the exact checkout toolchain rather than the host default nightly.
            (root / "rust-toolchain.toml").write_bytes(
                (semantic_navigation.ROOT / "rust-toolchain.toml").read_bytes()
            )
            (root / "src").mkdir()
            (root / "Cargo.toml").write_text(
                '[package]\nname = "semantic-edit-fixture"\nversion = "0.0.0"\nedition = "2024"\n'
            )
            (root / "src/model.rs").write_text(
                "pub fn calculate(value: i32) -> i32 {\n    value + 1\n}\n"
            )
            (root / "src/lib.rs").write_text(
                "mod model;\n#[test]\nfn renamed_consumer() { assert_eq!(model::calculate(2), 3); }\n"
            )
            paths = ["src/model.rs", "src/lib.rs"]
            calls = _calls(root / paths[0], 1, 8, 17)
        else:
            (root / "model.py").write_text(
                "def calculate(value: int) -> int:\n    result = value + 1\n    return result\n\nassert calculate(2) == 3\n"
            )
            paths = ["model.py"]
            calls = _calls(root / paths[0], 1, 5, 14)
            calls[-1]["arguments"].update(
                start_line=2,
                start_character=5,
                end_line=2,
                end_character=23,
                kind_filter="refactor.extract",
            )
        cut = semantic_edits.capture(root, paths)
        observed = asyncio.run(
            semantic_navigation.request(root, language, calls, fixture=True)
        )
        _require_probe(
            all(not item["isError"] for item in observed["results"]),
            observed["results"],
        )
        _require_probe(
            observed["results"][1]["result"]["status"]
            in {"renameable", "default_behavior"},
            "Selected semantic-edit qualification assertion failed",
        )
        selected = semantic_edits.preview(
            root,
            cut,
            semantic_navigation.edit_result(observed["results"][2]["result"]),
            producer="real mcpls:" + language,
        )
        if language == "rust":
            _require_probe(
                len(selected["files"]) == 2,
                "Rust rename did not reach the cross-file consumer",
            )
        applied = semantic_edits.apply(selected, root / "rename-receipt.json")
        _require_probe(
            applied["complete"], "Selected semantic-edit qualification assertion failed"
        )
        if language == "rust":
            command = [
                "rustc",
                "--edition=2024",
                "--test",
                str(root / "src/lib.rs"),
                "-o",
                str(root / "fixture-test"),
            ]
            built = subprocess.run(
                command, cwd=root, text=True, capture_output=True, check=True
            )
            behavior = subprocess.run(
                [str(root / "fixture-test")],
                cwd=root,
                text=True,
                capture_output=True,
                check=True,
            )
            qualification = {
                "command": command,
                "build_stdout": built.stdout,
                "build_stderr": built.stderr,
                "behavior": behavior.stdout,
            }
        else:
            run = subprocess.run(
                [sys.executable, str(root / "model.py")],
                text=True,
                capture_output=True,
                check=True,
            )
            qualification = {
                "command": [sys.executable, str(root / "model.py")],
                "stdout": run.stdout,
                "stderr": run.stderr,
            }
        # Code-action edits describe the original cut; restore this owned fixture
        # before independently qualifying the selected extraction.
        action_result = observed["results"][3]["result"]
        semantic_edits.require_complete(action_result)
        action_observation = {"returned": action_result}
        if language == "python":
            for name, content in cut.items():
                (root / name).write_bytes(content.encode("utf-8"))
            actions = [
                item
                for item in action_result.get("actions", [])
                if item.get("edit") and not item.get("command")
            ]
            _require_probe(actions, "Pyrefly did not return an editable extraction")
            action = actions[0]
            action_preview = semantic_edits.preview(
                root,
                cut,
                semantic_navigation.edit_result(action["edit"]),
                producer="real Pyrefly:" + action["title"],
            )
            action_applied = semantic_edits.apply(
                action_preview, root / "action-receipt.json"
            )
            _require_probe(
                action_applied["complete"],
                "Selected semantic-edit qualification assertion failed",
            )
            action_run = subprocess.run(
                [sys.executable, str(root / "model.py")],
                text=True,
                capture_output=True,
                check=True,
            )
            action_observation.update(
                preview=action_preview,
                applied=action_applied,
                behavior_stdout=action_run.stdout,
                behavior_stderr=action_run.stderr,
            )
        failure = asyncio.run(
            semantic_navigation.request(
                root,
                language,
                [
                    {
                        "name": "prepare_rename",
                        "arguments": {
                            "file_path": str(root / paths[0]),
                            "line": 10000,
                            "character": 1,
                        },
                    }
                ],
                fixture=True,
            )
        )
        negative = {"response": failure["results"]}
        _require_probe(
            failure["results"][0]["isError"],
            "invalid source position was reported as successful",
        )
        _require_probe(
            failure["results"][0]["result"]["error"]["code"] == -32602,
            "invalid-position rejection was not observed",
        )
        cancellation = asyncio.run(cancelled_request(root, language))
        remaining = sorted(processes() - before_processes)
        _require_probe(
            not remaining,
            f"request-owned language processes survived cleanup: {remaining}",
        )
        return {
            "language": language,
            "source_cut": cut,
            "toolchain_sha256": semantic_edits.digest(
                (semantic_navigation.ROOT / "rust-toolchain.toml").read_text()
            )
            if language == "rust"
            else None,
            "observed": observed,
            "rename_preview": selected,
            "applied": applied,
            "qualification": qualification,
            "code_actions": action_observation,
            "negative": negative,
            "cancellation": cancellation,
            "new_processes_after_close": remaining,
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--language", choices=("python", "rust", "both"), default="both"
    )
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = {"baseline_failures": 0, "results": []}
    for language in ("python", "rust") if args.language == "both" else (args.language,):
        result["results"].append(probe(language))
        args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(
        json.dumps(
            {
                "output": str(args.output),
                "languages": [item["language"] for item in result["results"]],
                "failures": 0,
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
