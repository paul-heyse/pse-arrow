"""Executable preservation, reachability, retrieval and continuation checks."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import build
import contracts

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    acquisition = build.load_acquisition()
    records = [
        r
        for folder in ("operations", "modules")
        for p in sorted((ROOT / "content" / folder).glob("*.json"))
        for r in json.loads(p.read_text())
    ]
    raw = {}
    for name, fact in acquisition["crates"].items():
        for capture, suffix in [
            ("hosted", ""),
            *([("private", ".private")] if "private" in fact else []),
        ]:
            raw[(name, capture)] = json.loads(build.document(fact, suffix))
    for r in records:
        source = raw[(r["crate"], r["artifact"]["capture"])]["index"][r["artifact"]["item_id"]]
        assert r["docs"] == (source.get("docs") or ""), r["path"]
        assert r["type_tree"] == source["inner"], r["path"]
        assert r["span"] == source.get("span"), r["path"]
        page = (ROOT / "content" / r["page"]).read_text()
        assert f'id="{r["anchor"]}"' in page
        for target in r["resolved_doc_links"].values():
            path, anchor = target.split("#")
            target_page = (ROOT / "content" / Path(r["page"]).parent / path).resolve()
            assert target_page.exists(), target
            assert f'id="{anchor}"' in target_page.read_text(), target
    # Recovering a private capture must not turn its canonical path into a callable import.
    builder = [
        r for r in records if r["path"].endswith("InstrumentationOptionsBuilder::add_custom_field")
    ]
    assert builder and all(r["reachability"] == "reachable-undocumented" for r in builder)
    assert all(r["reached_via"] == "InstrumentationOptions::builder()" for r in builder)
    internal = [
        r for r in records if r["path"] == "datafusion_tracing::instrumented_exec::InstrumentedExec"
    ]
    assert internal and all(r["reachability"] == "internal" for r in internal)

    # Document-local item IDs cannot be identity: renumber a synthetic capture and its links.
    def fixture(n: int) -> dict:
        return {
            "format_version": 61,
            "index": {
                str(n): {
                    "name": "f",
                    "crate_id": 0,
                    "visibility": "public",
                    "docs": "contract",
                    "inner": {
                        "function": {
                            "sig": {"inputs": [], "output": None, "is_c_variadic": False},
                            "generics": {"params": [], "where_predicates": []},
                            "header": {
                                "is_const": False,
                                "is_unsafe": False,
                                "is_async": False,
                                "abi": "Rust",
                            },
                            "has_body": True,
                        }
                    },
                }
            },
            "paths": {str(n): {"path": ["sample", "f"]}},
        }

    first = contracts.collect("sample", "1", fixture(1), {"sample::f"})
    second = contracts.collect("sample", "1", fixture(900), {"sample::f"})
    assert first[0]["id"] == second[0]["id"]
    assert first[0]["artifact"]["id"] != second[0]["artifact"]["id"]
    module_path = ROOT / "scripts/reference.py"
    spec = importlib.util.spec_from_file_location("reader", module_path)
    assert spec and spec.loader
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    payload = {"contract": "λ🙂完整契約" * 2000}
    expected = json.dumps(payload, ensure_ascii=False, indent=2)
    offset = 0
    parts = []
    fingerprints = set()
    while True:
        text = reader.bounded(payload, 1200, offset)
        assert len(text.encode()) + 1 <= 1200
        part = json.loads(text)
        fingerprints.add(part["result_sha256"])
        parts.append(part["text_fragment"])
        if part["next_offset"] is None:
            break
        assert part["next_offset"] > offset
        offset = part["next_offset"]
    assert "".join(parts) == expected and len(fingerprints) == 1

    def lookup(*args: str) -> dict:
        done = subprocess.run(
            [sys.executable, str(module_path), *args, "--max-bytes", "1000000"],
            cwd="/",
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(done.stdout)

    known = lookup(
        "show",
        "datafusion_tracing::InstrumentationOptions",
        "--member",
        "preview_fn",
        "--view",
        "contract",
    )
    canonical = lookup(
        "show",
        "datafusion_tracing::options::InstrumentationOptions",
        "--member",
        "preview_fn",
        "--view",
        "contract",
    )
    assert known["matches"] == canonical["matches"]
    # A path under a module that shares its name with a re-export (alias `tracing::instrument`
    # -> `tracing_attributes::instrument`) must resolve to itself, not be rewritten through the
    # alias into a path that does not exist. 489 real paths failed this before the guard.
    shadowed = lookup("show", "tracing::instrument::Instrumented", "--view", "contract")
    assert {m["path"] for m in shadowed["matches"]} == {"tracing::instrument::Instrumented"}
    discovery = {}
    for task, expected_id in [
        ("positive limit omitted formatter", "tracing.preview"),
        ("only physical optimization", "tracing.rules.selected"),
        ("stdout works collector empty", "tracing.filtering"),
        ("spawn task parent context", "tracing.context"),
        ("get ranges bytes stream", "tracing.storage"),
    ]:
        found = lookup("find", "--task", task)
        ids = [r["id"] for r in found["capabilities"]]
        assert expected_id in ids, (task, ids)
        discovery[task] = ids
    comparison = lookup("compare", "tracing.rules.phase", "tracing.rules.full")
    assert len(comparison["comparison"]) == 2
    report = {
        "state": "passed",
        "preserved_records": len(records),
        "document_link_targets": sum(len(r["resolved_doc_links"]) for r in records),
        "capture_profiles": len(raw),
        "continuation_fragments": len(parts),
        "discovery_cases": discovery,
        "scope": (
            "Preservation, identity, reachability and deterministic reader assertions; "
            "not agent decision evaluation."
        ),
    }
    sys.stdout.write(json.dumps(report, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
