"""P1: typed containers in markdown-it-py 4.2.0 + mdit-py-plugins 0.6.1."""
import json, re, sys
from pathlib import Path
from markdown_it import MarkdownIt
from markdown_it.tree import SyntaxTreeNode
from mdit_py_plugins.front_matter import front_matter_plugin
from mdit_py_plugins.attrs import attrs_block_plugin
from mdit_py_plugins.attrs.parse import parse as parse_attrs, ParseError
from mdit_py_plugins.container import container_plugin
from mdit_py_plugins.field_list import fieldlist_plugin
from mdit_py_plugins.colon_fence import colon_fence_plugin

FAMILIES = {"finding", "scenario", "disposition"}

def validate(params, *_):
    words = params.strip().split(None, 1)
    return bool(words) and words[0] in FAMILIES

def parser(*, container=True, colon=None):
    md = MarkdownIt("commonmark").use(front_matter_plugin).use(attrs_block_plugin).use(fieldlist_plugin)
    if colon == "before":
        md.use(colon_fence_plugin)
    if container:
        md.use(container_plugin, name="typed", validate=validate)
    if colon == "after":
        md.use(colon_fence_plugin)
    return md

def info_attrs(info):
    m = re.search(r"\{.*\}\s*$", info)
    if not m:
        return {}
    try:
        _, attrs = parse_attrs(m.group(0))
        return dict(attrs)
    except ParseError as e:
        return {"<parse-error>": str(e)}

def elements(src, md):
    tokens = md.parse(src)
    out = []
    def walk(node, parent):
        for child in node.children:
            if child.type == "container_typed":
                tok = child.nester_tokens.opening
                fam = tok.info.strip().split(None, 1)[0]
                rec = {
                    "family": fam,
                    "info": tok.info,
                    "attrs_from_block_line": dict(tok.attrs),
                    "attrs_from_info": info_attrs(tok.info),
                    "open_map": list(tok.map),
                    "close_markup": child.nester_tokens.closing.markup,
                    "close_map": child.nester_tokens.closing.map,
                    "first_body_block": None,
                    "parent": parent,
                    "field_list": {},
                }
                body = [c for c in child.children]
                if body:
                    rec["first_body_block"] = {"type": body[0].type, "map": list(body[0].map) if body[0].map else None}
                for c in body:
                    if c.type == "field_list":
                        for item in c.children:
                            pass
                ident = rec["attrs_from_block_line"].get("id") or rec["attrs_from_info"].get("id")
                rec["id"] = ident
                out.append(rec)
                walk(child, ident)
            else:
                walk(child, parent)
    walk(SyntaxTreeNode(tokens), None)
    return tokens, out

def field_lists(tokens):
    res = []
    for i, t in enumerate(tokens):
        if t.type == "fieldlist_name_open":
            res.append({"name": tokens[i + 1].content, "map": t.map})
        if t.type == "fieldlist_body_open":
            res[-1]["body_map"] = t.map
            # body content is the next inline
            for j in range(i, len(tokens)):
                if tokens[j].type == "inline":
                    res[-1]["body"] = tokens[j].content
                    break
    return res

src = Path(sys.argv[1]).read_text()
lines = src.splitlines(keepends=True)
report = {}

tokens, els = elements(src, parser())
report["elements"] = els
report["field_lists"] = field_lists(tokens)
report["front_matter"] = [{"map": t.map, "content": t.content} for t in tokens if t.type == "front_matter"]
report["top_level_types"] = [t.type for t in tokens if t.level == 0 and t.nesting >= 0]
# attrs-line recovery: is line open_map[0]-1 the {..} line that produced attrs_from_block_line?
rec = []
for e in els:
    if e["attrs_from_block_line"]:
        prev = lines[e["open_map"][0] - 1].rstrip("\n")
        rec.append({"id": e["id"], "line_before_open": prev, "is_attrs_line": prev.startswith("{") and prev.endswith("}")})
report["attrs_line_recovery"] = rec

# Control: no container plugin
ctl_tokens = parser(container=False).parse(src)
report["control_no_container_types"] = sorted({t.type for t in ctl_tokens})
report["control_has_container_tokens"] = any(t.type.startswith("container_") for t in ctl_tokens)
# The attrs on a paragraph in the control (attrs_block attaches to the next block = paragraph)
report["control_paragraph_attrs"] = [dict(t.attrs) for t in ctl_tokens if t.type == "paragraph_open" and t.attrs]

# colon_fence interplay
for order in ("before", "after"):
    tk = parser(colon=order).parse(src)
    report[f"colon_fence_{order}"] = {
        "colon_fence_tokens": sum(t.type == "colon_fence" for t in tk),
        "container_open_tokens": sum(t.type == "container_typed_open" for t in tk),
    }

# Splice edits: change one attribute value on exactly one line
def splice(src, line_no, old, new):
    ls = src.splitlines(keepends=True)
    assert ls[line_no].count(old) == 1, (line_no, ls[line_no])
    ls[line_no] = ls[line_no].replace(old, new)
    return "".join(ls)

def shape(tokens):
    return [(t.type, t.map, t.info, tuple(sorted(t.attrs.items()))) for t in tokens]

edits = []
by_id = {e["id"]: e for e in els}
# F-13: attribute in fence info (line = open_map[0])
e = by_id["F-13"]
new = splice(src, e["open_map"][0], "status=deferred", "status=resolved")
edits.append(("F-13 info attrs", src, new))
# F-12: attribute on attrs line (line = open_map[0]-1)
e = by_id["F-12"]
new2 = splice(src, e["open_map"][0] - 1, "status=open", "status=resolved")
edits.append(("F-12 attrs line", src, new2))
report["edits"] = []
for label, before, after in edits:
    t0, e0 = elements(before, parser())
    t1, e1 = elements(after, parser())
    s0, s1 = shape(t0), shape(t1)
    diff_tokens = [(a, b) for a, b in zip(s0, s1) if a != b]
    bl, al = before.splitlines(True), after.splitlines(True)
    changed_lines = [i for i, (a, b) in enumerate(zip(bl, al)) if a != b]
    report["edits"].append({
        "edit": label,
        "same_token_count": len(s0) == len(s1),
        "changed_source_lines": changed_lines,
        "bytes_outside_changed_lines_identical": all(a == b for i, (a, b) in enumerate(zip(bl, al)) if i not in changed_lines) and len(bl) == len(al),
        "token_differences": [{"before": a, "after": b} for a, b in diff_tokens],
        "element_differences": [(x["id"], x["attrs_from_block_line"], x["attrs_from_info"], y["attrs_from_block_line"], y["attrs_from_info"]) for x, y in zip(e0, e1) if x != y],
    })
print(json.dumps(report, indent=1, default=str))
