from markdown_it import MarkdownIt
from mdit_py_plugins.container import container_plugin
md = MarkdownIt("commonmark").use(container_plugin, name="typed", validate=lambda p, *a: (p.split() or [""])[0] in {"finding", "scenario"})
src = "::: finding {#A}\nOuter.\n\n::: scenario {#B}\nInner.\n:::\n\nTail.\n:::\n"
print(repr(src))
for x in md.parse(src):
    print(x.type, x.map, repr(x.markup), repr(x.info), repr(x.content) if x.type == "inline" else "")
