#!/usr/bin/env python3
"""Dump a library's public API surface to JSON. The Python analogue of rustdoc JSON.
Usage: apidump.py <out_dir> <module> [<module> ...]"""
import importlib, importlib.metadata as md, inspect, json, pkgutil, sys, types, os

SKIP_NAME = ("test", "tests", "_test", "conftest", "benchmark")

def sig(obj):
    try:
        return str(inspect.signature(obj))
    except Exception:
        return None

def doc1(obj):
    d = inspect.getdoc(obj)
    return d.strip().split("\n")[0][:220] if d else None

def describe_class(cls):
    members = {}
    for n, m in inspect.getmembers(cls):
        if n.startswith("_") and not (n.startswith("__arrow") or n in ("__init__", "__call__")):
            continue
        kind = ("method" if inspect.isfunction(m) or inspect.ismethod(m)
                else "property" if isinstance(m, property)
                else "attr")
        entry = {"kind": kind}
        if kind == "method":
            entry["sig"] = sig(m)
            entry["doc"] = doc1(m)
        members[n] = entry
    return {
        "kind": "class",
        "bases": [b.__name__ for b in getattr(cls, "__mro__", [])[1:6]],
        "doc": doc1(cls),
        "sig": sig(cls),
        "members": members,
    }

def walk(modname, out, seen, depth=0):
    if modname in seen or depth > 3:
        return
    seen.add(modname)
    try:
        mod = importlib.import_module(modname)
    except Exception as e:
        out["_errors"][modname] = f"{type(e).__name__}: {e}"
        return
    entry = {"all": list(getattr(mod, "__all__", []) or []), "items": {}}
    for n, obj in vars(mod).items():
        if n.startswith("_"):
            continue
        try:
            if inspect.isclass(obj):
                entry["items"][n] = describe_class(obj)
            elif inspect.isfunction(obj) or inspect.isbuiltin(obj) or callable(obj) and not isinstance(obj, type) and inspect.isroutine(obj):
                entry["items"][n] = {"kind": "function", "sig": sig(obj), "doc": doc1(obj)}
            elif isinstance(obj, (str, int, float, bool, tuple)) and not isinstance(obj, types.ModuleType):
                entry["items"][n] = {"kind": "const", "value": repr(obj)[:120]}
        except Exception:
            continue
    out["modules"][modname] = entry
    # descend into subpackages
    if hasattr(mod, "__path__"):
        for mi in pkgutil.iter_modules(mod.__path__):
            if any(s in mi.name for s in SKIP_NAME) or mi.name.startswith("_"):
                continue
            walk(f"{modname}.{mi.name}", out, seen, depth + 1)

def main():
    out_dir, mods = sys.argv[1], sys.argv[2:]
    os.makedirs(out_dir, exist_ok=True)
    for top in mods:
        dist = {"pyomo": "Pyomo", "pyarrow": "pyarrow", "attr": "attrs", "attrs": "attrs",
                "cattrs": "cattrs", "cattr": "cattrs", "msgspec": "msgspec",
                "numpy": "numpy", "scipy": "scipy", "idaes": "idaes-pse"}.get(top, top)
        try: version = md.version(dist)
        except Exception: version = "unknown"
        out = {"library": top, "distribution": dist, "version": version,
               "python": sys.version.split()[0], "modules": {}, "_errors": {}}
        walk(top, out, set())
        path = os.path.join(out_dir, f"{top}.json")
        with open(path, "w") as f:
            json.dump(out, f)
        n_mods = len(out["modules"])
        n_items = sum(len(m["items"]) for m in out["modules"].values())
        print(f"  {top:10} v{version:10} modules={n_mods:5} items={n_items:6} errors={len(out['_errors'])}  -> {os.path.basename(path)}")

main()
