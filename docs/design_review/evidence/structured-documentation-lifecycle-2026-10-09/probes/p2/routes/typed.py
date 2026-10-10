"""Rewrite typed containers (P1 syntax) to HTML div blocks; shared by the staging route and the preprocessor route."""
import html, re
from markdown_it import MarkdownIt
from markdown_it.tree import SyntaxTreeNode
from mdit_py_plugins.attrs import attrs_block_plugin
from mdit_py_plugins.attrs.parse import parse as parse_attrs
from mdit_py_plugins.container import container_plugin
from mdit_py_plugins.field_list import fieldlist_plugin

FAMILIES = {"finding", "scenario", "disposition", "packet", "investigation"}

def _validate(params, *_):
    w = params.strip().split(None, 1)
    return bool(w) and w[0] in FAMILIES

MD = (MarkdownIt("commonmark").use(attrs_block_plugin).use(fieldlist_plugin)
      .use(container_plugin, name="typed", validate=_validate))

def _attrs(tok):
    attrs = dict(tok.attrs)
    m = re.search(r"\{.*\}\s*$", tok.info)
    if m:
        attrs.update(dict(parse_attrs(m.group(0))[1]))
    title = re.sub(r"\{.*\}\s*$", "", tok.info).strip().split(None, 1)
    return tok.info.strip().split(None, 1)[0], attrs, (title[1] if len(title) > 1 else "")

def rewrite(text):
    lines = text.splitlines(keepends=True)
    tokens = MD.parse(text)
    for node in SyntaxTreeNode(tokens).walk():
        if node.type != "container_typed":
            continue
        tok = node.nester_tokens.opening
        family, attrs, title = _attrs(tok)
        start, close = tok.map
        if tok.attrs:  # consume the attrs_block line(s) above, across blank lines
            j = start - 1
            while j >= 0 and (lines[j].strip() == "" or re.fullmatch(r"\s{0,3}\{.*\}\s*", lines[j].rstrip("\n"))):
                if lines[j].strip():
                    lines[j] = "\n"
                j -= 1
        ident = attrs.pop("id", "")
        classes = " ".join([family, *attrs.pop("class", "").split()])
        data = " ".join(f'data-{k}="{html.escape(str(v), quote=True)}"' for k, v in attrs.items())
        open_tag = f'<div id="{html.escape(ident)}" class="{classes}" {data}>'.replace(" >", ">")
        label = f"**{ident}** {title}".strip() if (ident or title) else ""
        lines[start] = open_tag + "\n\n" + (label + "\n\n" if label else "")
        if node.nester_tokens.closing.markup:  # explicit close line
            lines[close] = "\n</div>\n"
        else:
            lines[close - 1] = lines[close - 1] + "\n</div>\n"
    return "".join(lines)
