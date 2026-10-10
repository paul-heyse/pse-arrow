"""Probe: query plan sections and table rows by kind, with neighbours, relations and guidance.

Parse-on-demand over plain Markdown (markdown-it-py 4.2.0 + mdit-py-plugins 0.6.1).
Kinds come from, in order: a heading attribute `{#id kind=x}`, an inline or preceding HTML
comment `<!-- kind: x -->`, or a legacy title alias. Output is the source bytes with line
numbers so an agent can edit by exact-string replacement.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

from markdown_it import MarkdownIt
from mdit_py_plugins.attrs.parse import ParseError, parse as parse_attrs
from mdit_py_plugins.front_matter import front_matter_plugin

MD = MarkdownIt("commonmark").enable("table").use(front_matter_plugin)

# Legacy aliases: title regex -> kind. Only used when no explicit marker is present.
LEGACY = {
    "purpose": r"^(purpose|scope)\b",
    "baseline": r"baseline|foundations",
    "target": r"target design|design and contracts",
    "choices": r"choices|alternatives",
    "packets": r"(implementation|investigation)? ?packets|execution sequence|work packages",
    "migration": r"migration|adoption",
    "verification": r"verification|acceptance|qualification",
    "dispositions": r"finding dispositions?|dispositions",
    "checkpoint": r"current (checkpoint|state)|checkpoint|handoff",
    "outcome": r"^outcome",
}
ID_RE = re.compile(r"\b([A-Z]{1,4}\d{2,3}[a-z]?|ADR-\d{4}|F\d{2}|R-\d{2,3})\b")
ANCHOR_RE = re.compile(r'<a id="([^"]+)"></a>')
TRAIL_ATTR = re.compile(r"\s*(\{[^{}]*\})\s*$")
COMMENT = re.compile(r"<!--\s*kind:\s*([\w-]+)\s*-->")


@dataclass
class Section:
    level: int
    title: str
    start: int  # 0-based heading line
    end: int  # exclusive
    kind: str | None
    kind_source: str
    anchor: str | None
    parent: "Section | None" = None
    children: list = field(default_factory=list)
    tables: list = field(default_factory=list)  # (header, rows[(line, cells)])


def slug(text: str) -> str:
    text = re.sub(r"<[^>]+>", "", text).strip().lower()
    return re.sub(r"[^\w\- ]", "", text).replace(" ", "-")


def parse(path: Path):
    src = path.read_text(encoding="utf-8")
    lines = src.splitlines(keepends=True)
    toks = MD.parse(src)
    fm = next((t.content for t in toks if t.type == "front_matter"), "")
    secs: list[Section] = []
    stack: list[Section] = []
    i = 0
    while i < len(toks):
        t = toks[i]
        if t.type == "heading_open":
            level = int(t.tag[1])
            raw = toks[i + 1].content
            kind, how, anchor = None, "", None
            m = TRAIL_ATTR.search(raw)
            if m:
                try:
                    _, attrs = parse_attrs(m.group(1))
                    raw = raw[: m.start()]
                    anchor = attrs.get("id")
                    kind = attrs.get("kind") or attrs.get("data-kind")
                    how = "attribute" if kind else how
                except ParseError:
                    pass
            c = COMMENT.search(raw)
            if c:
                kind, how = c.group(1), "inline-comment"
                raw = COMMENT.sub("", raw).strip()
            prev = t.map[0] - 1
            if not kind and prev >= 0 and (c := COMMENT.search(lines[prev])):
                kind, how = c.group(1), "comment-above"
            title = raw.strip()
            if not kind and level == 2:
                for k, pat in LEGACY.items():
                    if re.search(pat, title, re.I):
                        kind, how = k, "legacy-title"
                        break
            s = Section(level, title, t.map[0], len(lines), kind, how, anchor or slug(title))
            while stack and stack[-1].level >= level:
                stack.pop().end = t.map[0]
            if stack:
                s.parent = stack[-1]
                stack[-1].children.append(s)
            stack.append(s)
            secs.append(s)
        elif t.type == "table_open" and secs:
            owner = [s for s in secs if s.start <= t.map[0]][-1]
            header, rows, j, cells, row_line, in_head = [], [], i, [], None, True
            while toks[j].type != "table_close":
                tj = toks[j]
                if tj.type == "tbody_open":
                    in_head = False
                if tj.type == "tr_open":
                    cells, row_line = [], tj.map[0]
                if tj.type == "inline":
                    cells.append(tj.content)
                if tj.type == "tr_close":
                    if in_head:
                        header = cells
                    else:
                        rows.append((row_line, cells))
                j += 1
            owner.tables.append((header, rows))
            i = j
        i += 1
    return fm, lines, secs


def numbered(lines, start, end):
    return "".join(f"{n + 1:5}| {lines[n]}" for n in range(start, end))


def guidance(kind: str, root: Path) -> list[str]:
    """Collect guidance without copying it: plan-structure table row + marked skill blocks."""
    out = []
    ps = root / "skills/plan-structure.md"
    _, lines, secs = parse(ps)
    part = {
        "checkpoint": "Finding disposition and current state",
        "dispositions": "Finding disposition and current state",
        "packets": "Execution sequence",
        "verification": "Verification and acceptance",
        "migration": "Migration and adoption",
        "target": "Target design and contracts",
        "baseline": "Current baseline and affected foundations",
        "choices": "Choices and alternatives",
        "purpose": "Purpose, scope and basis",
    }.get(kind)
    for s in secs:
        for header, rows in s.tables:
            for line, cells in rows:
                if part and cells and cells[0] == part:
                    out.append(f"plan-structure.md:{line + 1} | {cells[0]}: {cells[1]}")
    for f in sorted((root / "skills").glob("*.md")):
        flines = f.read_text(encoding="utf-8").splitlines()
        for n, ln in enumerate(flines):
            m = re.search(r"<!--\s*guides:\s*([\w ,-]+)-->", ln)
            if m and kind in [k.strip() for k in m.group(1).split(",")]:
                para = []
                for nxt in flines[n + 1 :]:
                    if not nxt.strip():
                        break
                    para.append(nxt)
                out.append(f"{f.name}:{n + 2}\n" + "\n".join(para))
    return out


def relations(lines, s: Section, secs, me: Path):
    body = "".join(lines[s.start : s.end])
    ids = sorted(set(ID_RE.findall(body)))
    links = sorted(set(re.findall(r"\]\(([^)\s]+)\)", body)))
    return ids, links


def cmd_section(a):
    fm, lines, secs = parse(a.file)
    hits = [s for s in secs if a.target in (s.kind, s.anchor)]
    if not hits:
        sys.exit(f"no section with kind or anchor {a.target!r}")
    for s in hits:
        path, p = [], s.parent
        while p:
            path.insert(0, p.title)
            p = p.parent
        sib = (s.parent.children if s.parent else [x for x in secs if x.level == s.level])
        k = sib.index(s)
        print(f"## {a.file.name}:{s.start + 1}-{s.end}  kind={s.kind} ({s.kind_source})  #{s.anchor}")
        print("path: " + " > ".join(path + [s.title]))
        if k > 0:
            print(f"previous: {sib[k - 1].title} (lines {sib[k - 1].start + 1}-{sib[k - 1].end})")
        if k + 1 < len(sib):
            print(f"next: {sib[k + 1].title} (lines {sib[k + 1].start + 1}-{sib[k + 1].end})")
        ids, links = relations(lines, s, secs, a.file)
        print("mentions: " + ", ".join(ids))
        if a.links:
            print("links: " + ", ".join(links))
        if a.guide:
            print("--- guidance")
            for g in guidance(s.kind or "", a.root):
                print(g)
        print("--- text")
        stop = s.children[0].start if (a.own and s.children) else s.end
        end = stop if a.full else min(stop, s.start + a.max_lines)
        sys.stdout.write(numbered(lines, s.start, end))
        if end < stop:
            print(f"      … {stop - end} more lines (use --full)")
        if a.own and s.children:
            print("subsections: " + "; ".join(f"{c.title} [{c.kind or '-'}] {c.start + 1}-{c.end}" for c in s.children))


def cmd_rows(a):
    fm, lines, secs = parse(a.file)
    for s in secs:
        if a.kind not in (s.kind, s.anchor):
            continue
        for header, rows in s.tables:
            def keep(cells):
                row = dict(zip(header, cells))
                for cond in a.where:
                    col, neg, pat = re.match(r"(.+?)(!?)~(.*)", cond).groups()
                    col = next((h for h in header if h.lower().startswith(col.lower())), None)
                    if col is None:
                        return False
                    hit = re.search(pat, row.get(col, ""), re.I) is not None
                    if hit == bool(neg):
                        return False
                return True
            sel = [(n, c) for n, c in rows if keep(c)]
            if not sel:
                continue
            print(f"## {a.file.name} § {s.title} — {len(sel)}/{len(rows)} rows; columns: {header}")
            for n, c in sel:
                print(f"{n + 1:5}| {lines[n]}", end="")


def cmd_elements(a):
    """Heading-per-element: child headings of a kind section; fields from a leading `- Key: value` list."""
    fm, lines, secs = parse(a.file)
    for s in secs:
        if a.kind not in (s.kind, s.anchor):
            continue
        for c in s.children:
            fields, n = {}, c.start + 1
            while n < c.end and not lines[n].strip():
                n += 1
            while n < c.end and (m := re.match(r"- ([A-Z][\w ]*): (.*)", lines[n])):
                fields[m.group(1).lower()] = (n, m.group(2))
                n += 1
            ok = True
            for cond in a.where:
                col, neg, pat = re.match(r"(.+?)(!?)~(.*)", cond).groups()
                hit = re.search(pat, fields.get(col.lower(), (0, ""))[1], re.I) is not None
                ok &= hit != bool(neg)
            if ok:
                print(f"{a.file.name}:{c.start + 1}-{c.end} #{c.anchor} {c.title}")
                for k, (ln, v) in fields.items():
                    print(f"{ln + 1:5}| - {k}: {v}")


def cmd_mentions(a):
    fm, lines, secs = parse(a.file)
    pat = re.compile(rf"\b{re.escape(a.id)}\b")
    for s in secs:
        own = [n for n in range(s.start, s.children[0].start if s.children else s.end) if pat.search(lines[n])]
        if not own:
            continue
        rows = {n for _, r in s.tables for n, _ in r}
        kinds = ["row" if n in rows else "prose" for n in own]
        print(f"{a.file.name}:{s.start + 1} § {s.title} [kind={s.kind}] lines {[n + 1 for n in own]} ({', '.join(sorted(set(kinds)))})")


def cmd_outline(a):
    fm, lines, secs = parse(a.file)
    for s in secs:
        rows = sum(len(r) for _, r in s.tables)
        print(f"{'  ' * (s.level - 1)}{s.start + 1:5}-{s.end:<5} {s.title}  [kind={s.kind or '-'} via {s.kind_source or '-'}; rows={rows}]")


def cmd_active(a):
    for f in sorted(a.dir.glob("*.md")):
        fm, lines, secs = parse(f)
        status = re.search(r"^status:\s*(\S+)", fm, re.M)
        if not status or status.group(1) not in ("in-progress", "active", "proposed"):
            continue
        cps = [s for s in secs if s.kind == "checkpoint"]
        print(f"# {f.name} status={status.group(1)}")
        for s in cps[:1]:
            first = []
            for n in range(s.start + 1, s.end):
                if lines[n].strip() == "" and first:
                    break
                if lines[n].strip():
                    first.append(lines[n])
            print(f"  {f.name}:{s.start + 1}-{s.end} § {s.title}: " + " ".join(x.strip() for x in first)[:400])


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, default=Path(__file__).parent / "corpus")
    sub = p.add_subparsers(dest="cmd", required=True)
    o = sub.add_parser("outline"); o.add_argument("file", type=Path); o.set_defaults(fn=cmd_outline)
    s = sub.add_parser("section"); s.add_argument("file", type=Path); s.add_argument("target")
    s.add_argument("--guide", action="store_true"); s.add_argument("--links", action="store_true")
    s.add_argument("--full", action="store_true"); s.add_argument("--own", action="store_true"); s.add_argument("--max-lines", type=int, default=40)
    s.set_defaults(fn=cmd_section)
    r = sub.add_parser("rows"); r.add_argument("file", type=Path); r.add_argument("kind")
    r.add_argument("--where", action="append", default=[]); r.set_defaults(fn=cmd_rows)
    e = sub.add_parser("elements"); e.add_argument("file", type=Path); e.add_argument("kind")
    e.add_argument("--where", action="append", default=[]); e.set_defaults(fn=cmd_elements)
    m = sub.add_parser("mentions"); m.add_argument("file", type=Path); m.add_argument("id"); m.set_defaults(fn=cmd_mentions)
    ac = sub.add_parser("active"); ac.add_argument("dir", type=Path); ac.set_defaults(fn=cmd_active)
    a = p.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
