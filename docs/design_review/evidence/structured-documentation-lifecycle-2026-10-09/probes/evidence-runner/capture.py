"""Run a Python probe and record, with no author effort, what it actually touched."""
import atexit, hashlib, json, os, runpy, subprocess, sys
from pathlib import Path
root = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())
probe = Path(sys.argv[1]).resolve(); out = Path(sys.argv[2])
opened, spawned = set(), []
def hook(event, args):
    if event == "open" and isinstance(args[0], (str, bytes, os.PathLike)):
        mode = args[1] or "r"
        if isinstance(mode, str) and not any(c in mode for c in "wax+"):
            opened.add(os.fsdecode(args[0]))
    elif event == "subprocess.Popen":
        spawned.append([os.fsdecode(a) for a in (args[1] or [])][:6])
sys.addaudithook(hook)
def digest(p):
    try: return hashlib.blake2b(Path(p).read_bytes(), digest_size=16).hexdigest()
    except OSError: return None
def inside(p):
    try: return str(Path(p).resolve().relative_to(root))
    except (ValueError, OSError): return None
def dump():
    mods = {inside(m.__file__) for m in list(sys.modules.values()) if getattr(m, "__file__", None)}
    reads = {inside(p) for p in opened if not os.path.isabs(p) or True}
    files = sorted(x for x in (mods | reads) - {None} if not x.startswith(".venv/"))
    dists = {}
    from importlib import metadata
    for m in list(sys.modules.values()):
        f = getattr(m, "__file__", None) or ""
        if "site-packages" in f:
            top = m.__name__.split(".")[0]
            for d in metadata.packages_distributions().get(top, []):
                try: dists[d] = metadata.version(d)
                except Exception: pass
    out.write_text(json.dumps({
        "git_head": subprocess.check_output(["git","-C",root,"rev-parse","HEAD"], text=True).strip(),
        "python": sys.version.split()[0],
        "repo_inputs": {f: digest(root / f) for f in files},
        "distributions": dict(sorted(dists.items())),
        "subprocesses": spawned,
    }, indent=1))
atexit.register(dump)
sys.argv = [str(probe)] + sys.argv[3:]
runpy.run_path(str(probe), run_name="__main__")
