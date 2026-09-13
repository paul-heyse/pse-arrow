#!/usr/bin/env python3
"""Query an API dump: apiq.py <dump.json> <mode> [args]
modes: cls <Name> [filter] | find <substr> [kind] | mod <modname> | has <Name>"""
import json, sys, os
d = json.load(open(sys.argv[1])); mode = sys.argv[2]
print(f"# {d['library']} v{d['version']} (py{d['python']})")
mods = d["modules"]
if mode == "cls":
    name = sys.argv[3]; filt = sys.argv[4].lower() if len(sys.argv) > 4 else ""
    for mn, m in mods.items():
        it = m["items"].get(name)
        if it and it.get("kind") == "class":
            print(f"  {mn}.{name}{it.get('sig') or ''}")
            if it.get("bases"): print(f"    bases: {', '.join(it['bases'])}")
            if it.get("doc"): print(f"    doc: {it['doc']}")
            for k, v in sorted(it["members"].items()):
                if filt and filt not in k.lower(): continue
                s = v.get("sig") or ""
                print(f"      [{v['kind']:8}] {k}{s}")
            return_ = True
            break
elif mode == "find":
    sub = sys.argv[3].lower(); kind = sys.argv[4] if len(sys.argv) > 4 else None
    seen = set()
    for mn, m in mods.items():
        for n, it in m["items"].items():
            if sub in n.lower() and (not kind or it["kind"] == kind) and (n, it["kind"]) not in seen:
                seen.add((n, it["kind"])); print(f"  {it['kind']:9} {mn}.{n}{it.get('sig') or ''}")
elif mode == "mod":
    mn = sys.argv[3]; m = mods.get(mn)
    if not m: print("  (no such module)"); sys.exit(0)
    print(f"  __all__: {len(m['all'])} entries")
    for n, it in sorted(m["items"].items()):
        print(f"    {it['kind']:9} {n}{it.get('sig') or ''}")
elif mode == "has":
    name = sys.argv[3]
    hits = [f"{mn}.{name}" for mn, m in mods.items() if name in m["items"]]
    print("  " + (", ".join(hits) if hits else "** NOT FOUND **"))
