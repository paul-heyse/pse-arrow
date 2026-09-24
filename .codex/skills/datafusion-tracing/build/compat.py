"""Render historical manifest combinations without inferring general compatibility.

Requirements, resolved lockfiles and executed profiles are distinct evidence.
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path

DEP = re.compile(
    r'^(?P<name>[A-Za-z0-9_-]+)\s*=\s*(?:"(?P<bare>[^"]+)"|\{[^}]*?version\s*=\s*"(?P<in_table>[^"]+)")',
    re.MULTILINE,
)


def _versions(manifest_text: str | None, wanted: list[str]) -> dict[str, str]:
    if not manifest_text:
        return {}
    found: dict[str, str] = {}
    for match in DEP.finditer(manifest_text):
        name = match.group("name")
        if name in wanted:
            found[name] = match.group("bare") or match.group("in_table") or "-"
    return found


def _datafusion_of(own_manifest: str | None) -> str:
    if not own_manifest:
        return "-"
    try:
        document = tomllib.loads(own_manifest)
    except tomllib.TOMLDecodeError:
        return "-"
    spec = (document.get("dependencies") or {}).get("datafusion")
    if isinstance(spec, str):
        return spec
    if isinstance(spec, dict):
        return str(spec.get("version", "-"))
    return "-"


def _minor(version: str) -> tuple[int, int] | None:
    match = re.match(r"^[~^=]?(\d+)\.(\d+)", version or "")
    return (int(match.group(1)), int(match.group(2))) if match else None


def write_all(content: Path, history: dict, manifest: dict) -> dict[str, int]:
    tracked = manifest["compatibility"]["track"]
    catalogs = content / "catalogs"
    index = content / "index"
    catalogs.mkdir(parents=True, exist_ok=True)
    index.mkdir(parents=True, exist_ok=True)

    def sort_key(number: str) -> tuple:
        return tuple(int(p) if p.isdigit() else 0 for p in number.split("."))

    rows: list[str] = []
    table: list[dict] = []
    for number in sorted(history["releases"], key=sort_key, reverse=True):
        record = history["releases"][number]
        pins = _versions(record.get("example_manifest"), tracked)
        entry = {
            "version": number,
            "published": record["published"],
            "yanked": record["yanked"],
            "datafusion": _datafusion_of(record.get("own_manifest")),
            "rust_version": record.get("rust_version") or "-",
            "edition": record.get("edition") or "-",
            "pins": pins,
            "readable": bool(record.get("example_manifest")),
        }
        table.append(entry)
        rows.append(
            "\t".join(
                (
                    number,
                    entry["published"],
                    "yanked" if entry["yanked"] else "-",
                    entry["datafusion"],
                    entry["rust_version"],
                    entry["edition"],
                    pins.get("opentelemetry", "-"),
                    pins.get("opentelemetry_sdk", "-"),
                    pins.get("opentelemetry-otlp", "-"),
                    pins.get("tracing-opentelemetry", "-"),
                    "manifest-read" if entry["readable"] else "manifest-unavailable",
                )
            )
        )
    (index / "compatibility.tsv").write_text("".join(f"{row}\n" for row in rows))

    offsets = set()
    for entry in table:
        core = _minor(entry["pins"].get("opentelemetry", ""))
        bridge = _minor(entry["pins"].get("tracing-opentelemetry", ""))
        if core and bridge and core[0] == bridge[0] == 0:
            offsets.add(bridge[1] - core[1])

    _write_catalog(catalogs, history, table, offsets, tracked)
    return {"releases": len(rows)}


def _write_catalog(
    catalogs: Path, history: dict, table: list[dict], offsets: set[int], tracked: list[str]
) -> None:
    readable = [e for e in table if e["readable"]]
    offset_claim = (
        (
            f"exactly **+{offsets.pop()} minor**"
            if len(offsets) == 1
            else f"one of {sorted(offsets)} minors"
        )
        if offsets
        else "not derivable from these releases"
    )

    lines = [
        "# Version compatibility",
        "",
        f"Every published release of `{history['crate']}` — {len(table)} of them, "
        f"{len(readable)} with a readable example manifest. Registry read "
        f"{history['retrieved']}; this is the one table here whose source is a release history "
        "rather than a pin, so it describes the world on that date.",
        "",
        "## Declared combinations",
        "",
        f"In this captured table, the bridge minor is {offset_claim} ahead of the SDK family. "
        "This is a historical observation, not a compatibility rule. "
        "Inspect manifest requirements, "
        "the resolved lockfile and features, then compile the actual composition.",
        "",
        "DataFusion requirements below are read from release manifests. A requirement is not "
        "the exact version selected by Cargo; runtime export and receiver receipt "
        "are separate evidence.",
        "",
        "## Releases",
        "",
        "| datafusion-tracing | Published | datafusion | MSRV | Edition | opentelemetry* | "
        "tracing-opentelemetry |",
        "|---|---|---|---|---|---|---|",
    ]
    for entry in table:
        core = entry["pins"].get("opentelemetry", "—")
        sdk = entry["pins"].get("opentelemetry_sdk", "")
        otlp = entry["pins"].get("opentelemetry-otlp", "")
        family = core if {core} >= {sdk, otlp} - {""} else f"{core} / {sdk} / {otlp}"
        bridge = entry["pins"].get("tracing-opentelemetry", "—")
        flag = " *(yanked)*" if entry["yanked"] else ""
        lines.append(
            f"| `{entry['version']}`{flag} | {entry['published']} | `{entry['datafusion']}` | "
            f"{entry['rust_version']} | {entry['edition']} | `{family}` | `{bridge}` |"
        )

    lines += [
        "",
        "`opentelemetry*` is the three crates that move together: `opentelemetry`, "
        "`opentelemetry_sdk` and `opentelemetry-otlp`. A single value means all three agree.",
        "",
        "## Checking a manifest",
        "",
        "ast-grep cannot parse TOML — measured: `ast-grep run --lang toml` is rejected by "
        "0.45.3 with *`toml is not supported!`* — so there is no shipped rule for this and one "
        "would be the wrong tier. Read the pins out with ripgrep and compare them to the row "
        "above:",
        "",
        "```",
        "rg -N '^(tracing-opentelemetry|opentelemetry|opentelemetry_sdk|opentelemetry-otlp)"
        "\\s*=' <your>/Cargo.toml",
        "```",
        "",
        "That block is not marked `bash`, deliberately: `verify.py` executes every fenced bash "
        "command in these pages and fails one that finds nothing, and this one reads a manifest "
        "that is not in this directory. A command shown as runnable here has been run.",
        "",
        "## What a missing pair means",
        "",
        "A combination absent from this table has no evidence here. It is neither known "
        "incompatible nor proven untested elsewhere. "
        "This table does not track current latest releases.",
    ]
    (catalogs / "compatibility.md").write_text("\n".join(lines) + "\n")
