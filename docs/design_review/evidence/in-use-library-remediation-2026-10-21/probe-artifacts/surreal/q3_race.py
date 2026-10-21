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
if __name__=="__main__":
  pass
def _unused():
 scenario("A: T1 create-ref then sleep; T2 delete target during T1 sleep",
    "DELETE obligations WHERE id IN [obligations:r1]; CREATE studies:c1;",
    "BEGIN; CREATE obligations:r1 SET owner=studies:c1; sleep(2s); COMMIT;",
    "BEGIN; DELETE studies:c1; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r1; SELECT * FROM studies:c1;")
scenario("B: T2 delete target then sleep; T1 create-ref during T2 sleep (T1 commits first)",
    "CREATE studies:c2;",
    "BEGIN; CREATE obligations:r2 SET owner=studies:c2; COMMIT;",
    "BEGIN; DELETE studies:c2; sleep(2s); COMMIT;", delay=0.0,
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r2; SELECT * FROM studies:c2;")
# B swaps roles: start T2 first
scenario("B2: delete first (T1 here = delete+sleep), create-ref 0.5s later",
    "CREATE studies:c3;",
    "BEGIN; DELETE studies:c3; sleep(2s); COMMIT;",
    "BEGIN; CREATE obligations:r3 SET owner=studies:c3; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r3; SELECT * FROM studies:c3;")
scenario("C: like A but T1 reads target FOR UPDATE first",
    "CREATE studies:c4;",
    "BEGIN; SELECT * FROM studies:c4 FOR UPDATE; CREATE obligations:r4 SET owner=studies:c4; sleep(2s); COMMIT;",
    "BEGIN; DELETE studies:c4; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r4; SELECT * FROM studies:c4;")
scenario("D: like B2 but create-ref reads target FOR UPDATE first",
    "CREATE studies:c5;",
    "BEGIN; DELETE studies:c5; sleep(2s); COMMIT;",
    "BEGIN; SELECT * FROM studies:c5 FOR UPDATE; CREATE obligations:r5 SET owner=studies:c5; COMMIT;",
    check="SELECT id, owner, owner.id AS target_id FROM obligations:r5; SELECT * FROM studies:c5;")
