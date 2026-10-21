import threading, time, json, sys
sys.path.insert(0, '.')
from q import run
def show(tag, out):
    if isinstance(out, dict): print(tag, out); return
    for i, r in enumerate(out):
        r.pop('time', None); r.pop('type', None); print(f"  {tag}[{i}] {json.dumps(r)}")
def scenario(name, setup, t1, t2, delay=0.5, check=""):
    print(f"== {name}")
    show("setup", run(setup, "q3"))
    res = {}
    def go(k, sql): res[k] = (time.time(), run(sql, "q3"), time.time())
    a = threading.Thread(target=go, args=("T1", t1)); b = threading.Thread(target=go, args=("T2", t2))
    t0 = time.time(); a.start(); time.sleep(delay); b.start(); a.join(); b.join()
    for k in ("T1", "T2"):
        s, out, e = res[k]; print(f" {k} start+{s-t0:.2f}s end+{e-t0:.2f}s"); show(k, out)
    show("check", run(check, "q3"))
C = "SELECT id, owner, owner.retired AS r, (SELECT VALUE id FROM ONLY {tgt}) AS target_exists FROM obligations WHERE id IN [{ids}];"
