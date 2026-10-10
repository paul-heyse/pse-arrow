import re, sys, json
from pathlib import Path
on, off = Path(sys.argv[1]), Path(sys.argv[2])
def main(p):
    s = p.read_text(encoding="utf-8")
    m = re.search(r"<main>(.*?)</main>", s, re.S)
    return m.group(1) if m else ""
pages = sorted(p.relative_to(on) for p in on.rglob("*.html") if "generated/rustdoc" not in str(p))
diff, dl = [], []
for rel in pages:
    a, b = main(on / rel), main(off / rel)
    if "<dl>" in a:
        dl.append({"page": str(rel), "dl_count": a.count("<dl>"), "dt_samples": re.findall(r"<dt[^>]*>(.*?)</dt>", a, re.S)[:3]})
    if a != b:
        diff.append(str(rel))
print(json.dumps({"pages_compared": len(pages), "pages_differing": diff, "pages_with_dl_when_on": dl}, indent=1))
