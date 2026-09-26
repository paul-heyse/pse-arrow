# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Publish Markdown collections with mdBook and Pagefind; no product imports."""

from __future__ import annotations

import argparse
import fnmatch
import html
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path

from scripts.adr import markdown_headings, parse_scalar, section_owners

ROOT = Path(__file__).resolve().parents[1]
SCOPES = {"Current", "Reference", "History"}


@dataclass(frozen=True)
class Page:
    path: Path
    title: str
    body: str
    collection: str
    scope: str
    status: str = ""


def configuration(root: Path) -> dict:
    return tomllib.loads((root / "docs/site.toml").read_text(encoding="utf-8"))


def tool_versions(root: Path = ROOT) -> dict[str, str]:
    return configuration(root)["tools"]


def metadata(text: str) -> tuple[dict[str, str], str]:
    """Read only scalar publication fields, leaving other YAML to its existing owners."""
    values = {}
    if text.startswith("---\n"):
        end = text.find("\n---\n", 3)
        if end < 0:
            raise ValueError("unterminated front matter")
        for key, raw in re.findall(
            r"^(title|status):[ \t]*(.*)$", text[4:end], re.MULTILINE
        ):
            value = parse_scalar(raw)
            if isinstance(value, str) and value not in {"|", ">"}:
                values[key] = value
        text = text[end + 5 :]
    return values, text


def matches(path: Path, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatchcase(path.as_posix(), pattern) for pattern in patterns)


def checked_path(docs: Path, relative: str) -> Path:
    path = docs / relative
    if not path.resolve().is_relative_to(docs.resolve()):
        raise ValueError(f"publication path escapes docs: {relative}")
    return path


def discover(root: Path, config: dict) -> list[Page]:
    docs = root / "docs"
    publication = config["publication"]
    pages = []
    seen = set()
    for collection in config["collections"]:
        base = checked_path(docs, collection["root"])
        if not base.is_dir():
            raise ValueError(f"missing collection: {base}")
        pattern = "**/*.md" if collection.get("recursive", True) else "*.md"
        paths = sorted(
            base.glob(pattern), key=lambda p: (p.name != "README.md", p.as_posix())
        )
        for path in paths:
            relative = path.relative_to(docs)
            checked_path(docs, relative.as_posix())
            if matches(relative, publication["exclude"]):
                continue
            if relative in seen:
                raise ValueError(f"duplicate publication path: {relative}")
            seen.add(relative)
            front, body = metadata(path.read_text(encoding="utf-8"))
            heading = next(
                (
                    line[2:].strip()
                    for _, line in markdown_headings(body)
                    if line.startswith("# ")
                ),
                "",
            )
            title = front.get("title") or heading
            if not title.strip():
                raise ValueError(f"{relative}: supply a scalar title or first H1")
            status = front.get("status", "")
            scope = collection["scope"]
            if relative.parts[0] == "adr" and re.match(r"\d{4}-", relative.name):
                scope = {
                    "accepted": "Current",
                    "superseded": "History",
                    "deprecated": "History",
                    "rejected": "History",
                }.get(status, "Reference")
            if (
                matches(relative, publication["current_work"])
                or relative.as_posix() in publication["entrypoints"]
            ):
                scope = "Current"
            for override in config.get("overrides", []):
                if matches(relative, override["patterns"]):
                    scope = override["scope"]
            if scope not in SCOPES:
                raise ValueError(f"{relative}: invalid search scope {scope!r}")
            pages.append(
                Page(relative, title, body, collection["title"], scope, status)
            )
    for entry in publication["entrypoints"]:
        if Path(entry) not in seen:
            raise ValueError(f"entrypoint is not published: {entry}")
    for pattern in publication["current_work"]:
        if not any(matches(p.path, [pattern]) for p in pages):
            raise ValueError(f"current-work pattern matches no page: {pattern}")
    return pages


def section_anchor(ident: str) -> str:
    return "section-" + ident.lower().replace(".", "-")


def architecture_index(
    root: Path, pages: list[Page]
) -> tuple[Page, dict[Path, dict[str, str]]]:
    owners = section_owners(root / "docs/authoritative_design/blueprint.md")
    annotations: dict[Path, dict[str, str]] = {}
    rows = [
        "# Architecture sections",
        "",
        "Generated from authoritative numbered headings. See the [reading guide](authoritative_design/README.md) for current contract precedence.",
        "",
    ]
    for ident, (path, heading) in owners.items():
        relative = path.relative_to(root / "docs")
        annotations.setdefault(relative, {})[heading] = section_anchor(ident)
        rows.append(
            f"- [§{ident} — {heading.lstrip('# ')}]({relative.as_posix()}#{section_anchor(ident)})"
        )
    if any(p.path == Path("architecture-sections.md") for p in pages):
        raise ValueError(
            "architecture-sections.md is reserved for the derived section directory"
        )
    return Page(
        Path("architecture-sections.md"),
        "Architecture sections",
        "\n".join(rows) + "\n",
        "Architecture",
        "Reference",
    ), annotations


def stage(root: Path, source: Path, pages: list[Page], config: dict) -> list[Page]:
    docs = root / "docs"
    source.mkdir(parents=True)
    # Copy assets only from declared collections. Markdown is handled below.
    for collection in config["collections"]:
        base = checked_path(docs, collection["root"])
        pattern = "**/*" if collection.get("recursive", True) else "*"
        for path in base.glob(pattern):
            relative = path.relative_to(docs)
            if (
                not path.is_file()
                or path.suffix == ".md"
                or matches(relative, config["publication"]["exclude"])
            ):
                continue
            checked_path(docs, relative.as_posix())
            target = source / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
    directory, annotations = architecture_index(root, pages)
    pages = [*pages, directory]
    summary = ["# Summary", ""]
    for collection in config["collections"]:
        members = [page for page in pages if page.collection == collection["title"]]
        landing = next(
            (p for p in members if p.path == Path(collection["root"]) / "README.md"),
            None,
        )
        summary.extend([f"# {collection['title']}", ""])
        if landing:
            members.remove(landing)
            members.insert(0, landing)
        for page in members:
            title = page.title.replace("[", "\\[").replace("]", "\\]")
            indent = "  " if landing and page != landing else ""
            summary.append(f"{indent}- [{title}]({page.path.as_posix()})")
    (source / "SUMMARY.md").write_text("\n".join(summary) + "\n", encoding="utf-8")
    for page in pages:
        lines = page.body.splitlines(keepends=True)
        for position, heading in markdown_headings(page.body):
            anchor = annotations.get(page.path, {}).get(heading)
            if anchor:
                lines[position] = f'<a id="{anchor}"></a>\n\n' + lines[position]
        body = "".join(lines)
        target = source / page.path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(body, encoding="utf-8")
    shutil.copy2(docs / "book.toml", source / "book.toml")
    return pages


def annotate(output: Path, pages: list[Page], root: Path) -> None:
    shutil.copytree(root / "docs/theme", output / "theme", dirs_exist_ok=True)
    for page in pages:
        path = output / page.path.with_suffix(".html")
        text = path.read_text(encoding="utf-8")
        if text.count("<main>") != 1 or text.count("</head>") != 1:
            raise ValueError(f"{page.path}: unsupported mdBook HTML structure")
        prefix = "../" * (len(page.path.parts) - 1)
        attrs = f'data-pagefind-body data-pagefind-filter="scope:{page.scope}"'
        title = html.escape(page.title, quote=True)
        text = text.replace(
            "<main>",
            f'<main {attrs}><span hidden data-pagefind-ignore data-pagefind-meta="title">{title}</span>',
            1,
        )
        text = text.replace(
            "</head>",
            f'<link rel="stylesheet" href="{prefix}pagefind/pagefind-component-ui.css">\n<link rel="stylesheet" href="{prefix}theme/search.css">\n<script type="module" src="{prefix}theme/search.js"></script>\n</head>',
            1,
        )
        label = html.escape(
            f"{page.scope} · {page.collection}"
            + (f" · {page.status}" if page.status else "")
        )
        text = text.replace(
            "</main>",
            f'<p class="document-scope" data-pagefind-ignore>{label}</p></main>',
            1,
        )
        if page.path == Path("architecture-sections.md"):
            text = re.sub(
                r'<a href="[^"]+"[^>]*title="Suggest an edit".*?</a>',
                "",
                text,
                flags=re.DOTALL,
            )
        path.write_text(text, encoding="utf-8")
    # mdBook's landing alias should display the same UI without becoming a second record.
    first = pages[0].path.with_suffix(".html")
    if first != Path("index.html"):
        text = (
            (output / first)
            .read_text(encoding="utf-8")
            .replace("data-pagefind-body", "data-pagefind-ignore")
        )
        (output / "index.html").write_text(text, encoding="utf-8")


def check_tools(root: Path) -> None:
    for name, version in tool_versions(root).items():
        binary = shutil.which(name)
        if not binary:
            raise ValueError(f"missing {name}; run just bootstrap-docs")
        result = subprocess.run(
            [binary, "--version"], check=True, text=True, capture_output=True
        )
        if not re.search(rf"(?:^|\s)v?{re.escape(version)}(?:\s|$)", result.stdout):
            raise ValueError(
                f"expected {name} {version}, got {result.stdout.strip()}; run just bootstrap-docs"
            )


def publish(candidate: Path, destination: Path) -> None:
    """Replace only a fully built artifact, restoring the old one if rename fails."""
    backup = candidate.parent / "previous"
    if destination.exists():
        destination.rename(backup)
    try:
        candidate.rename(destination)
    except OSError:
        if backup.exists():
            backup.rename(destination)
        raise


def build(root: Path = ROOT) -> Path:
    check_tools(root)
    config = configuration(root)
    pages = discover(root, config)
    work = root / "build/docs"
    work.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="publish-", dir=work) as temporary:
        scratch = Path(temporary)
        source, output = scratch / "source", scratch / "site"
        pages = stage(root, source, pages, config)
        subprocess.run(
            ["mdbook", "build", str(source), "--dest-dir", str(output)], check=True
        )
        annotate(output, pages, root)
        subprocess.run(["pagefind", "--site", str(output)], check=True)
        rustdoc = root / "docs/generated/rustdoc"
        if rustdoc.is_dir():
            shutil.copytree(rustdoc, output / "generated/rustdoc", dirs_exist_ok=True)
        publish(output, root / "docs/book")
    print(f"docs: published {len(pages)} chapters with scoped search")
    return root / "docs/book"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("build", "serve", "install", "tool-specs"))
    args = parser.parse_args()
    try:
        if args.command == "tool-specs":
            print(
                ",".join(
                    f"{name}@{version}" for name, version in tool_versions().items()
                )
            )
        elif args.command == "install":
            if not shutil.which("cargo-binstall"):
                subprocess.run(
                    ["cargo", "install", "cargo-binstall", "--locked"], check=True
                )
            subprocess.run(
                [
                    "cargo",
                    "binstall",
                    "--no-confirm",
                    *(f"{name}@{version}" for name, version in tool_versions().items()),
                ],
                check=True,
            )
        else:
            output = build()
            if args.command == "serve":
                print(
                    "Rebuild with just docs after edits; refresh your browser.",
                    flush=True,
                )
                subprocess.run(
                    ["pagefind", "--site", str(output), "--serve"], check=True
                )
    except (ValueError, OSError, subprocess.CalledProcessError) as exc:
        print(f"docs: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
